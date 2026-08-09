from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path

from meta_harness_trae import (
    assess_workspace,
    audit_event_log,
    create_deterministic_archive,
    scan_for_secrets,
    validate_proposal,
)


ROOT = Path(__file__).resolve().parents[2]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_strict_json_object(path: Path) -> dict:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain one JSON object")
    return value


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("run_dir", type=Path)
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / "evidence/meta_harness/trae_run",
    )
    args = parser.parse_args()
    run_dir = args.run_dir.resolve()
    output = args.output.resolve()
    if output.exists():
        raise SystemExit(f"refusing to replace published run: {output}")
    workspace = run_dir / "workspace"
    final = read_strict_json_object(run_dir / "final_response.json")
    pending = read_strict_json_object(workspace / "logs/pending_eval.json")
    candidates = validate_proposal(final, pending)
    digests = json.loads((run_dir / "reference-digests.json").read_text())
    validation = assess_workspace(workspace, candidates, digests)
    audit_event_log(run_dir / "events.jsonl")
    raw_files = [path for path in run_dir.rglob("*") if path.is_file()]
    scan_for_secrets(raw_files)

    normalized = output / "normalized"
    normalized.mkdir(parents=True)
    shutil.copyfile(run_dir / "prompt.md", normalized / "prompt.md")
    shutil.copyfile(
        workspace / "reference/reference_state.json",
        normalized / "reference_state.json",
    )
    shutil.copyfile(
        workspace / "logs/pending_eval.json",
        normalized / "pending_eval.json",
    )
    shutil.copyfile(run_dir / "final_response.json", normalized / "final_response.json")
    for candidate in candidates:
        source = workspace / candidate["path"]
        destination = normalized / candidate["path"]
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    (normalized / "validation.json").write_text(
        json.dumps(validation, indent=2, sort_keys=True) + "\n"
    )
    create_deterministic_archive(
        run_dir,
        output / "raw_traecli_run.tar.gz",
        output / "raw_members.tsv",
    )
    normalized_files = sorted(
        path for path in normalized.rglob("*") if path.is_file()
    )
    (output / "normalized_manifest.tsv").write_text(
        "path\tbytes\tsha256\n"
        + "".join(
            f"{path.relative_to(output).as_posix()}\t{path.stat().st_size}\t{sha256(path)}\n"
            for path in normalized_files
        )
    )
    command = json.loads((run_dir / "command.json").read_text())
    receipt = {
        "schema_version": "harp-meta-harness-trae-run/v1",
        "source_id": "META-HARNESS-TRAE-RUN",
        "repository_source_id": "META-HARNESS-REPO",
        "repository_revision": "44b9942127847f7421db70d8c7e48407f09a3c70",
        "model": "gpt-5.4",
        "candidate_count": 3,
        "valid_candidate_count": validation["valid_candidate_count"],
        "first_schema_valid_attempt": 3,
        "first_model_reaching_attempt": 2,
        "command_returncode": command["returncode"],
        "proposal_schema_validated": True,
        "candidate_interfaces_assessed": True,
        "candidate_interfaces_passed": validation["interface_checks_passed"],
        "invalid_candidates_retained": validation["valid_candidate_count"] < 3,
        "workspace_boundary_validated": validation["workspace_boundary_passed"],
        "benchmark_invoked": False,
        "held_out_test_invoked": False,
        "paid_model_evaluation_reproduced": False,
        "benchmark_score_reproduced": False,
        "secret_scan": "passed-without-redaction",
        "raw_archive_sha256": sha256(output / "raw_traecli_run.tar.gz"),
        "raw_archive_bytes": (output / "raw_traecli_run.tar.gz").stat().st_size,
        "limitations": [
            "candidate quality was not evaluated",
            "per-dataset validation history was not reconstructed",
            "raw upstream traces were not reconstructed",
            "site aggregates are first-party reported state",
        ],
    }
    (output / "receipt.json").write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    )


if __name__ == "__main__":
    main()
