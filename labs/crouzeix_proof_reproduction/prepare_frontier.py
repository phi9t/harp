from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import stat
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Mapping

import expert_contracts
import formal_target
import tickets
from protocol import (
    ValidationError,
    normalize_historical_prompt,
    sha256_bytes,
    validate_run_spec,
    verify_historical_prompt,
)


LAB = Path(__file__).resolve().parent
TASK_MARKER = b"Current task statement"
FRONTIER_MODEL = "gpt-5.6-sol"
SELECTION_SEED = 20260814
TIMEOUT_SECONDS = 3600
MAX_OUTPUT_BYTES = 1024 * 1024
PREFLIGHT_MEMORY_MIB = 4096
CHILD_GENERATIONS = 3
DRAWS_PER_GENERATION = 2
ADMITTED_MATHEMATICAL_NODE_BUDGET = 11
TOTAL_CALL_BUDGET = 33
TRAECLI_VERSION = re.compile(r"^traecli(?:\s|$)")
EXPERT_ROLES = (
    "function_theory",
    "operator_dilation",
    "matrix_extremal",
    "completion_positivity",
    "approximation_audit",
)
PROOF_PROGRESS_PROBE_IDS = expert_contracts.PROOF_PROGRESS_PROBE_IDS
PROMPTS = ("expert.md", "proof_progress_evaluator.md")
SCHEMAS = (
    "expert_result.schema.json",
    "admission_decision.schema.json",
    "mathematical_node.schema.json",
    "node_evaluation_payload.schema.json",
    "node_evaluation.schema.json",
)
CREATE_ONLY_DIRECTORIES = (
    "inputs",
    "prompts",
    "schemas",
    "tickets",
    "attempts",
    "calls",
    "admissions",
    "mathematical_nodes",
    "node_evaluations",
    "archive_entries",
    "candidate_projections",
    "review",
    "contexts",
)
EXCLUDED_SOURCE_CLASSES = (
    "public proof manuscripts",
    "Harp Crouzeix packet",
    "reference-aware correctness reviews",
    "formalization source and declaration names",
    "evaluator rubrics with proof-specific checkpoints",
)
THEOREM_TEXT = (
    "Prove Crouzeix's conjecture: for every square complex matrix A and every "
    "polynomial p, ||p(A)|| is at most 2 times the supremum of |p(z)| over the "
    "numerical range of A."
)
FORMAL_TARGET_DIGEST_KEY = "formal_target/formal_target.lock.json"
REFERENCE_AWARE_FORBIDDEN_TEXT = (
    "CrouzeixConjecture.crouzeixConjecture",
    "source-map.json",
    "formal_targets/jin-565b6a3",
    "formal_targets/lorist-schwenninger",
    "crouzeix-formal-attempt-receipt/v2",
    "crouzeix-formal-ledger-row/v1",
    "jin-terminal-crouzeix",
    "ls-terminal-crouzeix",
)


def prepare_frontier(
    *,
    run_dir: Path,
    historical_prompt_path: Path,
    cli_path: Path,
    model: str,
    timeout_seconds: int,
    created_at_utc: str | None = None,
) -> Path:
    if model != FRONTIER_MODEL:
        raise ValidationError(f"model must be {FRONTIER_MODEL}")
    if timeout_seconds != TIMEOUT_SECONDS:
        raise ValidationError(f"timeout_seconds must be {TIMEOUT_SECONDS}")
    destination = _new_directory_path(run_dir, "frontier run directory")
    if destination.exists() or destination.is_symlink():
        raise ValidationError(f"refusing to replace existing run directory: {destination}")
    source_path = _regular_file(historical_prompt_path, "historical prompt")
    executable = _regular_file(cli_path, "TRAE CLI executable")
    if not os.access(executable, os.X_OK):
        raise ValidationError("TRAE CLI executable is not executable")

    source = source_path.read_bytes()
    verify_historical_prompt(source)
    execution, normalization = normalize_historical_prompt(source)
    theorem = _extract_theorem_or_default(source)
    version = _cli_version(executable)
    _validate_cli_identity(version)
    created = created_at_utc or _now()

    destination.mkdir(parents=True, mode=0o700)
    try:
        _create_tree(destination)
        _write_bytes_create_only(destination / "inputs/theorem.txt", theorem, "theorem")
        _write_bytes_create_only(
            destination / "inputs/execution_prompt.txt",
            execution,
            "execution prompt",
        )
        _write_json_create_only(
            destination / "inputs/prompt_normalization.json",
            normalization,
            "prompt normalization",
        )
        for name in PROMPTS:
            _copy_create_only(LAB / "prompts" / name, destination / "prompts" / name)
        for name in SCHEMAS:
            _copy_create_only(LAB / "schemas" / name, destination / "schemas" / name)

        frontier = _frontier_config()
        context_digests = _write_root_contexts(
            destination,
            run_id=destination.name,
            theorem_text=theorem.decode("utf-8", errors="replace"),
            created_at_utc=created,
        )
        digests = _input_digests(destination, frontier, executable)
        digests.update(context_digests)
        spec = {
            "schema_version": "crouzeix-run-spec/v1",
            "run_id": destination.name,
            "arm": "expert_frontier",
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
            "allowed_tools": ["Write"],
            "network_access": False,
            "timeout_seconds": timeout_seconds,
            "max_calls": TOTAL_CALL_BUDGET,
            "token_accounting": {
                "boundary": "all expert and evaluator provider calls launched by this run",
            },
            "generation_visible_files": sorted(
                [
                    "inputs/execution_prompt.txt",
                    "inputs/theorem.txt",
                    *[f"prompts/{name}" for name in PROMPTS],
                    *[f"schemas/{name}" for name in SCHEMAS],
                ]
            ),
            "generation_excluded_classes": list(EXCLUDED_SOURCE_CLASSES),
            "frontier": frontier,
            "digests": digests,
            "created_at_utc": created,
        }
        validate_run_spec(spec)
        _write_json_create_only(destination / "run_spec.json", spec, "run specification")
        _publish_root_tickets(
            destination,
            run_id=destination.name,
            context_digests=context_digests,
            prompt_sha256=digests["prompt/expert.md"],
            schema_sha256=digests["schema/expert_result.schema.json"],
            created_at_utc=created,
        )
    except BaseException:
        _remove_new_tree(destination)
        raise
    return destination.resolve()


def check_frontier_preparation(run_dir: Path) -> dict[str, object]:
    root = _existing_directory(run_dir, "frontier run directory")
    spec_path = root / "run_spec.json"
    raw_spec = _read_json_object(spec_path, "run specification")
    _validate_prepared_digests(root, raw_spec)
    spec = validate_run_spec(raw_spec)
    if spec["arm"] != "expert_frontier":
        raise ValidationError("run specification is not an expert_frontier run")
    for relative in CREATE_ONLY_DIRECTORIES:
        _existing_directory(root / relative, f"{relative} directory")
    if (root / "inputs/historical_prompt.txt").exists():
        raise ValidationError("raw historical prompt bytes must not be retained")
    scan_frontier_context(root)
    ticket_ids: list[str] = []
    for role in EXPERT_ROLES:
        ticket_id = root_ticket_id(role)
        ticket_path = root / "tickets" / ticket_id / "ticket.json"
        ticket = tickets.validate_runtime_ticket(_read_json_object(ticket_path, "runtime ticket"))
        if ticket["run_id"] != spec["run_id"] or ticket["role"] != role:
            raise ValidationError(f"root ticket {ticket_id} does not match run spec")
        if ticket["context_sha256"] != spec["digests"][f"context/{ticket_id}.json"]:
            raise ValidationError(f"root ticket {ticket_id} context digest mismatch")
        if (root / "tickets" / ticket_id / "ticket_events.jsonl").exists():
            raise ValidationError("root ticket event log must not exist before execution")
        ticket_ids.append(ticket_id)
    return {
        "schema_version": "crouzeix-frontier-preparation-check/v1",
        "run_id": spec["run_id"],
        "run_spec_sha256": _file_sha256(spec_path),
        "root_ticket_count": len(ticket_ids),
        "root_ticket_ids": ticket_ids,
    }


def scan_frontier_context(run_dir: Path) -> dict[str, object]:
    root = _existing_directory(run_dir, "frontier run directory")
    scanned = 0
    for relative in (
        "inputs",
        "prompts",
        "schemas",
        "contexts",
        "tickets",
    ):
        directory = root / relative
        if not directory.exists():
            continue
        for path in sorted(directory.rglob("*")):
            if not path.is_file() or path.is_symlink():
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            scanned += 1
            for forbidden in REFERENCE_AWARE_FORBIDDEN_TEXT:
                if forbidden in text:
                    raise ValidationError(
                        f"reference-aware FormalTarget artifact leaked into frontier context: {forbidden}"
                    )
    return {
        "schema_version": "crouzeix-frontier-context-scan/v1",
        "status": "clean",
        "scanned_files": scanned,
    }


def root_ticket_id(role: str) -> str:
    role = _role(role)
    return f"expert-g0-{role.replace('_', '-')}"


def root_node_id(role: str) -> str:
    role = _role(role)
    return f"node-g0-{role.replace('_', '-')}"


def root_direction(role: str) -> dict[str, object]:
    role = _role(role)
    return {
        "direction_id": f"root-{role.replace('_', '-')}",
        "kind": "root_task",
        "statement": f"Attempt an independent {role.replace('_', ' ')} route for Crouzeix's conjecture.",
        "strength": "root",
        "recommended_role": role,
        "source_parent_node_id": None,
        "source_node_artifact_sha256": None,
        "source_reconciliation_sha256": None,
    }


def _write_root_contexts(
    root: Path,
    *,
    run_id: str,
    theorem_text: str,
    created_at_utc: str,
) -> dict[str, str]:
    del created_at_utc
    context_root = root / "contexts"
    digests: dict[str, str] = {}
    for role in EXPERT_ROLES:
        ticket_id = root_ticket_id(role)
        context = expert_contracts.build_expert_context(
            run_id=run_id,
            attempt_id=ticket_id,
            ticket_id=ticket_id,
            proposed_node_id=root_node_id(role),
            parent_node=None,
            generation=0,
            expert_role=role,
            selected_direction=root_direction(role),
            theorem_text=theorem_text,
            forbidden_sources=list(EXCLUDED_SOURCE_CLASSES),
        )
        path = context_root / f"{ticket_id}.json"
        _write_json_create_only(path, context, "root expert context")
        digests[f"context/{ticket_id}.json"] = _file_sha256(path)
    return digests


def _publish_root_tickets(
    root: Path,
    *,
    run_id: str,
    context_digests: Mapping[str, str],
    prompt_sha256: str,
    schema_sha256: str,
    created_at_utc: str,
) -> None:
    ticket_root = root / "tickets"
    for role in EXPERT_ROLES:
        ticket_id = root_ticket_id(role)
        ticket = {
            "schema_version": "crouzeix-runtime-ticket/v1",
            "ticket_id": ticket_id,
            "run_id": run_id,
            "task_kind": "expert",
            "node_id": root_node_id(role),
            "parent_node_id": None,
            "generation": 0,
            "direction_id": f"root-{role.replace('_', '-')}",
            "role": role,
            "objective": (
                f"Attempt an independent {role.replace('_', ' ')} root route "
                "for Crouzeix's conjecture without reference-aware sources."
            ),
            "expected_deliverable": "Strict expert_result JSON or typed provider failure.",
            "dependency_ticket_ids": [],
            "context_sha256": context_digests[f"context/{ticket_id}.json"],
            "schema_sha256": schema_sha256,
            "prompt_sha256": prompt_sha256,
            "parent_artifact_sha256": None,
            "allowed_tools": ["Write"],
            "forbidden_sources": list(EXCLUDED_SOURCE_CLASSES),
            "timeout_seconds": TIMEOUT_SECONDS,
            "max_output_bytes": MAX_OUTPUT_BYTES,
            "owner_type": "expert",
            "created_at_utc": created_at_utc,
            "state": "created",
        }
        tickets.publish_ticket(ticket_root, ticket)


def _frontier_config() -> dict[str, object]:
    return {
        "selection_seed": SELECTION_SEED,
        "expert_roles": list(EXPERT_ROLES),
        "proof_progress_probe_ids": list(PROOF_PROGRESS_PROBE_IDS),
        "root_expert_count": len(EXPERT_ROLES),
        "child_generations": CHILD_GENERATIONS,
        "draws_per_generation": DRAWS_PER_GENERATION,
        "admitted_mathematical_node_budget": ADMITTED_MATHEMATICAL_NODE_BUDGET,
        "total_call_budget": TOTAL_CALL_BUDGET,
        "per_call_preflight": {"memory_mib": PREFLIGHT_MEMORY_MIB},
    }


def _input_digests(root: Path, frontier: Mapping[str, object], executable: Path) -> dict[str, str]:
    digests = {
        "cli": sha256_bytes(executable.read_bytes()),
        "config/frontier": _canonical_sha256(frontier),
        FORMAL_TARGET_DIGEST_KEY: _file_sha256(formal_target.PRODUCTION_LOCK_PATH),
        "input/execution_prompt.txt": _file_sha256(root / "inputs/execution_prompt.txt"),
        "input/theorem.txt": _file_sha256(root / "inputs/theorem.txt"),
        "input/prompt_normalization.json": _file_sha256(
            root / "inputs/prompt_normalization.json"
        ),
    }
    for name in PROMPTS:
        digests[f"prompt/{name}"] = _file_sha256(root / "prompts" / name)
    for name in SCHEMAS:
        digests[f"schema/{name}"] = _file_sha256(root / "schemas" / name)
    return digests


def _validate_prepared_digests(root: Path, spec: Mapping[str, Any]) -> None:
    digests = spec.get("digests")
    if not isinstance(digests, Mapping):
        raise ValidationError("digests must be an object")
    frontier = spec.get("frontier")
    if not isinstance(frontier, Mapping):
        raise ValidationError("frontier must be an object")
    _expect_digest(digests, "config/frontier", _canonical_sha256(frontier))

    cli = spec.get("cli")
    if not isinstance(cli, Mapping):
        raise ValidationError("cli must be an object")
    cli_path = cli.get("path")
    if not isinstance(cli_path, str):
        raise ValidationError("cli.path must be a string")
    executable = _regular_file(Path(cli_path), "TRAE CLI executable")
    _expect_digest(digests, "cli", _file_sha256(executable))
    _validate_cli_identity(_cli_version(executable))

    for name in PROMPTS:
        _expect_digest(digests, f"prompt/{name}", _file_sha256(root / "prompts" / name))
    for name in SCHEMAS:
        _expect_digest(digests, f"schema/{name}", _file_sha256(root / "schemas" / name))
    _expect_digest(digests, FORMAL_TARGET_DIGEST_KEY, _file_sha256(formal_target.PRODUCTION_LOCK_PATH))


def _expect_digest(digests: Mapping[str, Any], key: str, observed: str) -> None:
    expected = digests.get(key)
    if expected != observed:
        raise ValidationError(f"{key} digest mismatch")


def _extract_theorem_or_default(prompt: bytes) -> bytes:
    if prompt.count(TASK_MARKER) != 1:
        return THEOREM_TEXT.encode("utf-8")
    theorem, _ = prompt.split(TASK_MARKER, 1)
    theorem = theorem.strip()
    return theorem + b"\n" if theorem else THEOREM_TEXT.encode("utf-8")


def _create_tree(root: Path) -> None:
    for relative in CREATE_ONLY_DIRECTORIES:
        path = root / relative
        if path.exists() or path.is_symlink():
            raise ValidationError(f"create-only directory already exists: {relative}")
        path.mkdir(mode=0o700)


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


def _new_directory_path(path: Path, label: str) -> Path:
    candidate = path.expanduser().absolute()
    if candidate.exists() or candidate.is_symlink():
        raise ValidationError(f"refusing to replace existing {label}: {candidate}")
    current = Path(candidate.anchor)
    parts = candidate.parts[1:]
    for index, part in enumerate(parts):
        current = current / part
        try:
            metadata = current.lstat()
        except FileNotFoundError:
            if index != len(parts) - 1:
                raise ValidationError(f"{label} parent does not exist: {current}")
            return candidate
        if stat.S_ISLNK(metadata.st_mode):
            raise ValidationError(f"{label} contains symlink: {current}")
        if index == len(parts) - 1:
            raise ValidationError(f"refusing to replace existing {label}: {current}")
        if not stat.S_ISDIR(metadata.st_mode):
            raise ValidationError(f"{label} parent must be a directory: {current}")
    return candidate


def _existing_directory(path: Path, label: str) -> Path:
    candidate = path.expanduser().absolute()
    try:
        metadata = candidate.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISDIR(metadata.st_mode):
        raise ValidationError(f"{label} must be a directory")
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


def _validate_cli_identity(version: str) -> None:
    if TRAECLI_VERSION.match(version.lower()) is None:
        raise ValidationError("TRAE CLI identity is invalid")


def _copy_create_only(source: Path, destination: Path) -> None:
    _write_bytes_create_only(destination, source.read_bytes(), str(destination))


def _write_bytes_create_only(path: Path, data: bytes, label: str) -> None:
    _reject_symlink(path, label)
    try:
        with path.open("xb") as handle:
            handle.write(data)
    except FileExistsError as error:
        raise ValidationError(f"{label} already exists: {path}") from error


def _write_json_create_only(path: Path, value: Mapping[str, Any], label: str) -> None:
    data = json.dumps(value, indent=2, sort_keys=True).encode("utf-8") + b"\n"
    _write_bytes_create_only(path, data, label)


def _read_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise ValidationError(f"{label} must be a regular file")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must be a JSON object")
    return value


def _reject_symlink(path: Path, label: str) -> None:
    try:
        mode = path.lstat().st_mode
    except FileNotFoundError:
        return
    if stat.S_ISLNK(mode):
        raise ValidationError(f"{label} cannot be a symlink")


def _remove_new_tree(root: Path) -> None:
    if not root.exists() and not root.is_symlink():
        return
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_symlink() or path.is_file():
            path.unlink()
        elif path.is_dir():
            path.rmdir()
    root.rmdir()


def _file_sha256(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def _canonical_sha256(value: Mapping[str, Any]) -> str:
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return sha256_bytes(data.encode("utf-8"))


def _role(role: str) -> str:
    if role not in EXPERT_ROLES:
        raise ValidationError("role must be one of the fixed expert roles")
    return role


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
    parser.add_argument("--historical-prompt", type=Path)
    parser.add_argument("--cli", type=Path, default=None)
    parser.add_argument("--model", default=FRONTIER_MODEL)
    parser.add_argument("--timeout-seconds", type=int, default=TIMEOUT_SECONDS)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        print(json.dumps(check_frontier_preparation(args.run_dir), indent=2, sort_keys=True))
        return
    if args.historical_prompt is None:
        raise SystemExit("--historical-prompt is required unless --check is used")
    print(
        prepare_frontier(
            run_dir=args.run_dir,
            historical_prompt_path=args.historical_prompt,
            cli_path=args.cli or _installed_cli(),
            model=args.model,
            timeout_seconds=args.timeout_seconds,
        )
    )


if __name__ == "__main__":
    main()
