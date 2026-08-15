from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from protocol import (
    ValidationError,
    normalize_historical_prompt,
    sha256_bytes,
    validate_run_spec,
    verify_historical_prompt,
)


LAB = Path(__file__).resolve().parent
TASK_MARKER = b"Current task statement"
PROMPTS = (
    "controller.md",
    "critic.md",
    "redirect.md",
    "repair.md",
    "route_worker.md",
    "synthesizer.md",
)
SCHEMAS = {
    "historical": ("historical_final.schema.json",),
    "orchestrated": (
        "controller.schema.json",
        "critic.schema.json",
        "repair.schema.json",
        "route.schema.json",
        "synthesis.schema.json",
    ),
}


def extract_theorem(prompt: bytes) -> bytes:
    if prompt.count(TASK_MARKER) != 1:
        raise ValidationError("historical prompt must contain exactly one task marker")
    theorem, _ = prompt.split(TASK_MARKER, 1)
    return theorem


def prepare_run(
    *,
    run_dir: Path,
    arm: str,
    historical_prompt_path: Path,
    cli_path: Path,
    model: str,
    timeout_seconds: int,
    created_at_utc: str | None = None,
) -> Path:
    if arm not in SCHEMAS:
        raise ValidationError("prepare supports historical or orchestrated arms")
    destination = run_dir.expanduser().absolute()
    if destination.exists() or destination.is_symlink():
        raise ValidationError(f"refusing to replace existing run directory: {destination}")
    source_path = _regular_file(historical_prompt_path, "historical prompt")
    executable = _regular_file(cli_path, "TRAE CLI executable")
    if not os.access(executable, os.X_OK):
        raise ValidationError("TRAE CLI executable is not executable")

    source = source_path.read_bytes()
    verify_historical_prompt(source)
    execution, normalization = normalize_historical_prompt(source)
    theorem = extract_theorem(source)
    version = _cli_version(executable)

    destination.mkdir(parents=True, mode=0o700)
    try:
        _create_tree(destination, arm)
        (destination / "inputs/theorem.txt").write_bytes(theorem)
        (destination / "inputs/execution_prompt.txt").write_bytes(execution)
        (destination / "inputs/prompt_normalization.json").write_text(
            json.dumps(normalization, indent=2, sort_keys=True) + "\n"
        )
        for name in SCHEMAS[arm]:
            (destination / "schemas" / name).write_bytes(
                (LAB / "schemas" / name).read_bytes()
            )
        if arm == "orchestrated":
            for name in PROMPTS:
                (destination / "prompts" / name).write_bytes(
                    (LAB / "prompts" / name).read_bytes()
                )

        visible = [
            "inputs/execution_prompt.txt",
            "inputs/theorem.txt",
            *[f"schemas/{name}" for name in SCHEMAS[arm]],
        ]
        if arm == "orchestrated":
            visible.extend(f"prompts/{name}" for name in PROMPTS)
        spec = {
            "schema_version": "crouzeix-run-spec/v1",
            "run_id": destination.name,
            "arm": arm,
            "leakage": "L1",
            "model": model,
            "cli": {
                "path": str(executable),
                "version": version,
                "sha256": sha256_bytes(executable.read_bytes()),
            },
            "historical_prompt": {
                "source_bytes": len(source),
                "source_sha256": sha256_bytes(source),
                "execution_bytes": len(execution),
                "execution_sha256": sha256_bytes(execution),
                "normalization": (
                    "replace the historical machine-specific output directory "
                    "with candidate.tex exactly once"
                ),
            },
            "sandbox": "workspace-write",
            "approval_policy": "never",
            "allowed_tools": (
                ["Write", "spawn_agent"] if arm == "historical" else ["Write"]
            ),
            "network_access": False,
            "timeout_seconds": timeout_seconds,
            "max_calls": 1 if arm == "historical" else 9,
            "token_accounting": {
                "boundary": "all provider calls launched by this run",
            },
            "generation_visible_files": sorted(visible),
            "generation_excluded_classes": [
                "Harp Crouzeix packet",
                "evaluator rubrics with proof-specific checkpoints",
                "formalization source and declaration names",
                "public proof manuscripts",
            ],
            "created_at_utc": created_at_utc or _now(),
        }
        validate_run_spec(spec)
        (destination / "run_spec.json").write_text(
            json.dumps(spec, indent=2, sort_keys=True) + "\n"
        )
    except BaseException:
        _remove_new_tree(destination)
        raise
    return destination.resolve()


def _create_tree(root: Path, arm: str) -> None:
    for relative in ("inputs", "schemas", "calls", "candidate", "review"):
        (root / relative).mkdir(mode=0o700)
    if arm == "orchestrated":
        (root / "prompts").mkdir(mode=0o700)


def _regular_file(path: Path, label: str) -> Path:
    candidate = path.expanduser().absolute()
    try:
        metadata = candidate.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise ValidationError(f"{label} must be a regular file")
    return candidate.resolve()


def _cli_version(executable: Path) -> str:
    try:
        completed = subprocess.run(
            [str(executable), "--version"],
            text=True,
            capture_output=True,
            timeout=10,
            check=False,
            env={"PATH": os.environ.get("PATH", "")},
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ValidationError(f"cannot query TRAE CLI version: {error}") from error
    if completed.returncode != 0:
        raise ValidationError("TRAE CLI version command failed")
    output = completed.stdout.strip()
    if not output or len(output) > 256 or "\0" in output:
        raise ValidationError("TRAE CLI version output is invalid")
    return output


def _remove_new_tree(root: Path) -> None:
    if not root.exists():
        return
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_symlink() or path.is_file():
            path.unlink()
        elif path.is_dir():
            path.rmdir()
    root.rmdir()


def _now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def _installed_cli() -> Path:
    path = shutil.which("traecli")
    if path is None:
        raise ValidationError("traecli is not installed on PATH")
    return Path(path).resolve()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--arm", required=True, choices=sorted(SCHEMAS))
    parser.add_argument("--historical-prompt", required=True, type=Path)
    parser.add_argument(
        "--cli",
        type=Path,
        default=None,
    )
    parser.add_argument("--model", default="gpt-5.6-sol")
    parser.add_argument("--timeout-seconds", type=int, default=3600)
    args = parser.parse_args()
    print(
        prepare_run(
            run_dir=args.run_dir,
            arm=args.arm,
            historical_prompt_path=args.historical_prompt,
            cli_path=args.cli or _installed_cli(),
            model=args.model,
            timeout_seconds=args.timeout_seconds,
        )
    )


if __name__ == "__main__":
    main()
