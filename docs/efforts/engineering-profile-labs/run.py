#!/usr/bin/env python3
"""Run bounded real PostgreSQL/Temporal learning fixtures; retain local evidence."""

import argparse
import hashlib
import json
import os
import re
import signal
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
COMPONENT = ROOT / "experiments/team_control_plane"
IMAGE = (
    "postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3"
)
EXPECTED = {
    "postgres": [
        "admission_is_durable_atomic_and_idempotent",
        "authority_expiry_scopes_generations_and_epoch_rotation",
        "distinct_assignments_capacity_and_cumulative_budget",
        "rollback_at_every_admission_write_stage",
        "a_cached_command_never_restores_wider_authority",
        "rejected_command_is_durable_and_cannot_be_reused",
        "token_expires_while_waiting_for_the_transaction_lock",
    ],
    "temporal": [
        "public_migration_installs_transactional_workflow_registry",
        "real_temporal_recovers_lost_hints_rollover_and_retained_replay",
        "real_temporal_mailbox_wait_survives_worker_death_and_lost_answer_hint",
    ],
}


def capture(command):
    result = subprocess.run(
        command, cwd=ROOT, capture_output=True, text=True, timeout=20
    )
    if result.returncode:
        raise RuntimeError(f"preflight failed: {command[0]} (exit {result.returncode})")
    return result.stdout.strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def run_lab(lab, run_root, environment, seconds):
    evidence = run_root / lab
    evidence.mkdir(mode=0o700)
    fixture = "run-postgres.sh" if lab == "postgres" else "run-temporal.sh"
    suite = "kernel" if lab == "postgres" else "temporal"
    command = [
        "bash",
        str(COMPONENT / "tests" / fixture),
        "--offline",
        "--test",
        suite,
        "--",
        "--nocapture",
        "--test-threads=1",
    ]
    child_environment = dict(environment, TEMPORAL_FIXTURE_EVIDENCE=str(evidence))
    started = datetime.now(timezone.utc).isoformat()
    monotonic_start = time.monotonic()
    timed_out = False
    with (evidence / "test.log").open("w") as output:
        process = subprocess.Popen(
            command,
            cwd=ROOT,
            env=child_environment,
            stdout=output,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            code = process.wait(timeout=seconds)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            code = 124
        except BaseException:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            raise
    log = (evidence / "test.log").read_text(errors="replace")
    missing = [
        name
        for name in EXPECTED[lab]
        if not re.search(
            r"^test " + re.escape(name) + r" \.\.\. ok$", log, re.MULTILINE
        )
    ]
    cleanup = {}
    for filename in ["postgres-cleanup.json", "cleanup.json"]:
        path = evidence / filename
        if path.exists():
            cleanup[filename] = json.loads(path.read_text())
    cleanup_ok = (
        cleanup.get("postgres-cleanup.json", {}).get("owned_container_removed") is True
    )
    if lab == "temporal":
        cleanup_ok = (
            cleanup_ok
            and cleanup.get("cleanup.json", {}).get("temporal_listener_closed") is True
        )
    passed = code == 0 and not missing and cleanup_ok
    record = {
        "lab": lab,
        "started_at": started,
        "finished_at": datetime.now(timezone.utc).isoformat(),
        "elapsed_seconds": round(time.monotonic() - monotonic_start, 3),
        "command": command,
        "timeout_seconds": seconds,
        "timed_out": timed_out,
        "exit_code": code,
        "required_tests_missing": missing,
        "test_summary": re.findall(r"^test result:.*$", log, re.MULTILINE),
        "cleanup": cleanup,
        "accepted": passed,
        "artifacts": {
            path.name: {"bytes": path.stat().st_size, "sha256": digest(path)}
            for path in sorted(evidence.iterdir())
            if path.is_file()
        },
    }
    write_json(evidence / "result.json", record)
    print(f"{lab}: {'PASS' if passed else 'FAIL'}; evidence: {evidence}", flush=True)
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lab", choices=["all", "postgres", "temporal"], default="all")
    parser.add_argument("--timeout-seconds", type=int, default=1200)
    args = parser.parse_args()
    if not 30 <= args.timeout_seconds <= 1800:
        parser.error("timeout must be between 30 and 1800 seconds")
    selected_tmp = os.environ.get("TMPDIR", "")
    temporary = Path(selected_tmp).resolve()
    if (
        not selected_tmp
        or not Path(selected_tmp).is_absolute()
        or temporary == Path("/")
        or temporary == Path("/tmp")
        or Path("/tmp") in temporary.parents
        or temporary.stat().st_uid != os.getuid()
        or temporary.stat().st_mode & 0o022
    ):
        parser.error(
            "run through python3 tools/temp_root.py --; use an owned TMPDIR outside /tmp"
        )
    scratch = ROOT / ".scratch/engineering-profile-labs"
    scratch.mkdir(parents=True, exist_ok=True)
    run_root = scratch / (
        datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + f"-{os.getpid()}"
    )
    run_root.mkdir(mode=0o700)
    target = scratch / "cargo-target"
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target), RUST_TEST_THREADS="1")
    # Existing CLI envconfig may point at user settings. Select fixture-only profile inputs.
    environment["TEMPORAL_NAMESPACE"] = "default"
    environment["TEMPORAL_API_KEY"] = ""
    environment["TEMPORAL_TLS"] = "false"
    image = json.loads(capture(["docker", "image", "inspect", IMAGE]))[0]
    pg_versions = [
        entry
        for entry in image["Config"]["Env"]
        if entry.startswith(("PG_MAJOR=", "PG_VERSION="))
    ]
    paths = [
        COMPONENT / "Cargo.toml",
        COMPONENT / "Cargo.lock",
        COMPONENT / "tests/kernel.rs",
        COMPONENT / "tests/temporal.rs",
        COMPONENT / "tests/run-postgres.sh",
        COMPONENT / "tests/run-temporal.sh",
        COMPONENT / "src/storage.rs",
        COMPONENT / "src/admission.rs",
        COMPONENT / "src/auth.rs",
        COMPONENT / "src/temporal.rs",
        Path(__file__).resolve(),
        *sorted(
            (
                ROOT
                / "docs/efforts/team-control-plane/evidence/ticket13-temporal-bridge"
            ).glob("*.json")
        ),
    ]
    metadata = {
        "schema_version": 1,
        "source_revision": capture(["git", "rev-parse", "HEAD"]),
        "branch": capture(["git", "branch", "--show-current"]),
        "worktree_status": capture(["git", "status", "--short"]),
        "rustc": capture(["rustc", "--version"]),
        "cargo": capture(["cargo", "--version"]),
        "temporal": capture(["temporal", "--version"]),
        "docker_server": capture(
            ["docker", "version", "--format", "{{.Server.Version}}"]
        ),
        "postgres": {
            "pinned_reference": IMAGE,
            "image_id": image["Id"],
            "version_env": pg_versions,
        },
        "input_sha256": {str(path.relative_to(ROOT)): digest(path) for path in paths},
        "execution": {
            "TMPDIR": str(temporary),
            "CARGO_TARGET_DIR": str(target),
            "cargo_resolution": "locked and offline",
            "test_threads": 1,
            "boundary": "host-native learning fixtures; no Insula proof",
        },
    }
    write_json(run_root / "metadata.json", metadata)
    labs = ["postgres", "temporal"] if args.lab == "all" else [args.lab]
    results = []
    for lab in labs:
        print(
            f"Running {lab}; deadline {args.timeout_seconds}s; {run_root}", flush=True
        )
        results.append(run_lab(lab, run_root, environment, args.timeout_seconds))
        if not results[-1]["accepted"]:
            break
    accepted = len(results) == len(labs) and all(
        result["accepted"] for result in results
    )
    write_json(run_root / "summary.json", {"accepted": accepted, "results": results})
    return 0 if accepted else 1


if __name__ == "__main__":
    sys.exit(main())
