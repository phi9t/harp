"""Controller-backed benchmark mode for deterministic single-ticket proof.

The benchmark supplies disposable task fixtures and observes the result. The
SQLite controller owns admission, runtime dispatch, exact candidate
verification, acceptance and protected-ref landing.
"""

from __future__ import annotations

import base64
import hashlib
import io
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.error
import urllib.request
import uuid
from dataclasses import dataclass
from pathlib import Path

import fleet
import tasklib

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


CONTROLLER_SCHEMA = "corenius.bench.controller.v2"
EXECUTION_DEADLINE_SECONDS = 30 * 60
SETUP_STAGE_TIMEOUT_SECONDS = 300.0
CLEANUP_TIMEOUT_SECONDS = 60.0
PINNED_VERIFIER_TIMEOUT_SECONDS = 120.0
PREFLIGHT_FAILURE_STAGES = {"missing_prerequisite", "preflight", "unsupported_task"}
DISPATCH_FAILURE_STAGES = {"timeout", "provider_failure"}
CONTROLLED_FAILURE_STAGES = (
    PREFLIGHT_FAILURE_STAGES
    | DISPATCH_FAILURE_STAGES
    | {"delivery_failure", "failed_verifier", "generic_failure", "missing_evidence", "manual_intervention"}
)
DEADLINE_GATED_COMMANDS = {"create_kata_task", "admit", "verify_candidate", "accept_candidate", "land_candidate"}
LIFECYCLE_FAULTS = {
    "delivery-failure",
    "elapsed-deadline",
    "failed-verifier",
    "final-deadline",
    "final-inflight-timeout",
    "generic-failure",
    "manual-intervention",
    "missing-evidence",
    "missing-prerequisite",
    "provider-failure",
}
PROJECT = "controller-bench"
EXECUTION_SOURCE_PATHS = [
    "tools/temp_root.py",
    "bench/harness/controller_mode.py",
    "bench/harness/run.py",
    "bench/harness/results.py",
    "bench/harness/tasklib.py",
    "bench/harness/fleet.py",
]
KATA_SHA256 = "8b0f05b1071f5e7a2b8173c9e5ef358d550a601cebd8c9d9cae6e45975e837f6"
KATA = Path(os.environ.get(
    "CORENIUS_KATA_PINNED_BINARY",
    Path.home() / ".local/share/corenius/pinned-tools/kata" / KATA_SHA256 / "kata-pinned",
))


class ControllerFailure(RuntimeError):
    def __init__(self, stage: str, message: str, details: dict | None = None):
        super().__init__(message)
        self.stage = stage
        self.message = message
        self.details = details or {}


class StageBudget:
    def __init__(self, *, seconds: float, failure_stage: str, label: str):
        self.seconds = float(seconds)
        self.failure_stage = failure_stage
        self.label = label
        self.started_monotonic = time.monotonic()
        self.deadline_monotonic = self.started_monotonic + self.seconds

    def remaining(self, boundary: str) -> float:
        remaining = self.deadline_monotonic - time.monotonic()
        if remaining <= 0:
            raise ControllerFailure(
                self.failure_stage,
                f"{self.label} budget elapsed before {boundary}",
                {
                    "boundary": boundary,
                    "budget_seconds": self.seconds,
                    "elapsed_seconds": round(time.monotonic() - self.started_monotonic, 3),
                },
            )
        return remaining

    def timeout(self, requested: float, boundary: str) -> float:
        return max(0.001, min(float(requested), self.remaining(boundary)))


class CleanupBudget:
    def __init__(self, seconds: float):
        self.seconds = float(seconds)
        self.started_monotonic = time.monotonic()
        self.deadline_monotonic = self.started_monotonic + self.seconds

    def remaining(self) -> float:
        return max(0.0, self.deadline_monotonic - time.monotonic())

    def wait_slice(self, requested: float) -> float:
        return min(float(requested), self.remaining())

    def elapsed(self) -> float:
        return round(time.monotonic() - self.started_monotonic, 3)

PARTIAL_CONTRACTS = {
    "t1": {
        "allow": ["strings_util.py"],
        "selected_tests": ["T.test_title_keeps_small_words_lower_and_collapses_spaces"],
        "check": (
            "import strings_util\n"
            "assert strings_util.title_case('the  lord of the rings') == 'The Lord of the Rings'\n"
        ),
    },
    "t2": {
        "allow": ["numbers_util.py"],
        "selected_tests": ["T.test_clamp_both_ends"],
        "check": (
            "import numbers_util\n"
            "assert numbers_util.clamp(5, 0, 3) == 3\n"
            "assert numbers_util.clamp(-1, 0, 3) == 0\n"
            "assert numbers_util.clamp(2, 0, 3) == 2\n"
        ),
    },
}

WORKER_SCRIPT = r"""
import json
import pathlib
import shutil
import subprocess
import sys
import time

sys.dont_write_bytecode = True
scratch = pathlib.Path("/scratch")
workspace = pathlib.Path("/workspace")
sys.path.insert(0, str(workspace))
deadline = time.monotonic() + 30
while not (scratch / "action.json").exists():
    if time.monotonic() > deadline:
        raise SystemExit("controller-bench action deadline")
    time.sleep(0.02)

action = json.loads((scratch / "action.json").read_text())
if action.get("wait_for_useful_start"):
    (scratch / "waiting_for_useful_start").write_text("ready\n")
    while not (scratch / "start_useful").exists():
        if time.monotonic() > deadline:
            raise SystemExit("controller-bench useful-start deadline")
        time.sleep(0.02)

def git(*args):
    return subprocess.check_output(
        [
            "/usr/bin/git",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-C",
            "/workspace",
            *args,
        ],
        text=True,
    ).strip()

useful_start_monotonic_ns = time.monotonic_ns()
useful_start_time_ns = time.time_ns()
useful_steps = []
changed = []
for rel, content in action.get("files", {}).items():
    path = pathlib.Path(rel)
    if path.is_absolute() or ".." in path.parts:
        raise SystemExit("unsafe fixture path")
    target = workspace / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)
    changed.append(rel)
    useful_steps.append({"kind": "write", "path": rel})

if action.get("check"):
    repeat = int(action.get("check_repeat", 1))
    for _ in range(repeat):
        exec(action["check"], {})
    useful_steps.append({"kind": "check", "repeat": repeat})
for pycache in workspace.rglob("__pycache__"):
    shutil.rmtree(pycache)

if changed:
    git("add", "--", *changed)
    git(
        "-c",
        "user.name=Controller bench worker",
        "-c",
        "user.email=controller-bench@invalid",
        "commit",
        "-m",
        "controller bench " + action["ticket"],
    )
    useful_steps.append({"kind": "commit", "paths": changed})
useful_end_monotonic_ns = time.monotonic_ns()
useful_end_time_ns = time.time_ns()

(scratch / "ready.json").write_text(
    json.dumps({
        "ticket": action["ticket"],
        "commit": git("rev-parse", "HEAD"),
        "changed": changed,
        "useful_interval": {
            "start_monotonic_ns": useful_start_monotonic_ns,
            "end_monotonic_ns": useful_end_monotonic_ns,
            "start_time_ns": useful_start_time_ns,
            "end_time_ns": useful_end_time_ns,
            "steps": useful_steps,
        },
    })
)
while not (scratch / "finish").exists():
    time.sleep(0.02)
"""


@dataclass
class ManagedProcess:
    name: str
    process: subprocess.Popen
    log: Path
    start_ticks: str

    def record(self) -> dict:
        return {
            "name": self.name,
            "pid": self.process.pid,
            "start_ticks": self.start_ticks,
            "exit": self.process.poll(),
            "log": str(self.log),
        }


def pin(path: Path | str) -> dict:
    path = Path(path).resolve()
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    path.chmod(0o600)


def execution_window(admitted_at: float, first_submission_at: float | None = None) -> dict:
    first_submission_at = admitted_at if first_submission_at is None else first_submission_at
    return {
        "clock": "durable_admission_to_final_verification",
        "deadline_seconds": float(EXECUTION_DEADLINE_SECONDS),
        "setup_timeout_seconds": float(SETUP_STAGE_TIMEOUT_SECONDS),
        "cleanup_timeout_seconds": float(CLEANUP_TIMEOUT_SECONDS),
        "pinned_verifier_timeout_seconds": float(PINNED_VERIFIER_TIMEOUT_SECONDS),
        "durable_run_admission_unix": admitted_at,
        "first_mission_submission_unix": first_submission_at,
        "dispatch_cutoff_unix": admitted_at + EXECUTION_DEADLINE_SECONDS,
        "failed_preflight_consumed_model_calls": False,
        "automatic_rerun_after_timeout_or_provider_failure": False,
        "live_host_requirement": "m01",
        "platform": sys.platform,
    }


def controller_outcomes(task_verdict: str, orchestration_verdict: str, effect_disposition: str) -> dict:
    return {
        "task": {"verdict": task_verdict},
        "orchestration": {"verdict": orchestration_verdict},
        "effects": {"disposition": effect_disposition},
    }


def no_model_usage(status: str = "not_applicable") -> dict:
    return {"status": status, "input_tokens": None, "output_tokens": None, "usd": None, "complete": False}


def evidence_entry(kind: str, path: Path) -> dict:
    entry = {"kind": kind, "path": str(path)}
    if path.is_file():
        entry["sha256"] = sha256_bytes(path.read_bytes())
    return entry


def retained_evidence(root: Path, repositories: list[dict] | None = None,
                      manifests: list[dict] | None = None, events: list[dict] | None = None,
                      proofs: list[dict] | None = None, logs: list[dict] | None = None) -> dict:
    return {
        "run_root": str(root),
        "repositories": repositories or [],
        "manifests": manifests or [],
        "events": events or [],
        "proofs": proofs or [],
        "logs": logs or [],
    }


def controller_result_metadata(
    *,
    root: Path,
    admitted_at: float,
    first_submission_at: float,
    task_verdict: str,
    orchestration_verdict: str,
    effect_disposition: str,
    attempts_total: int,
    initial_attempts: int,
    fresh_integration_attempts: int,
    interventions: list[dict],
    retained: dict,
    failure_stage: str | None = None,
    failure_kind: str | None = None,
    failure_message: str | None = None,
    model_usage: dict | None = None,
) -> dict:
    manual_rescue = any(str(item.get("kind", "")).startswith("manual") for item in interventions)
    return {
        "execution_window": execution_window(admitted_at, first_submission_at),
        "outcomes": controller_outcomes(task_verdict, orchestration_verdict, effect_disposition),
        "failure": {"stage": failure_stage, "kind": failure_kind, "message": failure_message},
        "interventions": interventions,
        "autonomous_pass_eligible": not manual_rescue,
        "attempts": {
            "total": attempts_total,
            "initial": initial_attempts,
            "fresh_integration": fresh_integration_attempts,
            "max_fresh_integration_attempts": 1,
            "automatic_retries_after_failure": 0,
            "max_concurrent_worker_executions": 2,
        },
        "model_usage": model_usage or no_model_usage(),
        "retained_evidence": retained,
    }


def failing_overlay(ticket: fleet.Ticket) -> dict[str, str]:
    if ticket.id == "t1":
        return {"strings_util.py": "def title_case(value):\n    return value\n"}
    if ticket.id == "t2":
        return {"numbers_util.py": "def clamp(value, low, high):\n    return value\n"}
    raise RuntimeError(f"no deterministic failing overlay for {ticket.id}")


def execution_clock_from_control(control: dict | None) -> dict:
    if control and isinstance(control.get("execution_clock"), dict):
        clock = dict(control["execution_clock"])
        if "first_mission_submission_unix" not in clock:
            clock["first_mission_submission_unix"] = clock["durable_run_admission_unix"]
        return clock
    now = time.time()
    return {"durable_run_admission_unix": now, "first_mission_submission_unix": now}


def runtime_closure_for_failure(root: Path, failure_manifest: Path) -> dict:
    policy_path = root / "git.json"
    if policy_path.is_file():
        policy = json.loads(policy_path.read_text())
        pinned = {
            "git": policy.get("git"),
            "python_executable": policy.get("python"),
            "bubblewrap": policy.get("bubblewrap"),
            "verifier_runtime": policy.get("verifier_runtime"),
        }
        return {"status": "pinned", "pinned": {key: value for key, value in pinned.items() if value is not None}}
    return {
        "status": "limited",
        "pinned": {},
        "limitation": "verifier runtime policy was unavailable before this failure; closure status is unknown",
        "failure_manifest": {"path": str(failure_manifest), "sha256": sha256_bytes(failure_manifest.read_bytes())},
    }


def jsonable(value):
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, dict):
        return {str(key): jsonable(item) for key, item in value.items()}
    if isinstance(value, list):
        return [jsonable(item) for item in value]
    if isinstance(value, tuple):
        return [jsonable(item) for item in value]
    return value


def unknown_source_identity() -> dict:
    document = {
        "schema_version": "corenius.bench.source-manifest.v1",
        "root": str(tasklib.ROOT),
        "digest": "unknown",
        "entries": [{"path": "source_identity", "kind": "unknown"}],
    }
    return {
        "head": None,
        "tree": None,
        "dirty": None,
        "status_sha256": None,
        "diff_sha256": None,
        "content_manifest": document,
    }


def timeout_failure_from_exception(exc: BaseException, run: "ControllerRun | None") -> ControllerFailure | None:
    if isinstance(exc, subprocess.TimeoutExpired):
        stage = "timeout" if run is not None and run.execution_budget is not None else "preflight"
        details = {
            "exception_type": "TimeoutExpired",
            "timeout_seconds": exc.timeout,
            "boundary": " ".join(str(part) for part in exc.cmd) if isinstance(exc.cmd, (list, tuple)) else str(exc.cmd),
        }
        if run is not None and run.execution_budget is not None:
            details.update(run.attempt_count_details())
            details.update({
                "dispatch_cutoff_unix": run.dispatch_cutoff_unix,
                "observed_unix": time.time(),
            })
        else:
            details.update({"attempts_total": 0, "initial_attempts": 0, "fresh_integration_attempts": 0})
        return ControllerFailure(stage, f"{stage} subprocess timeout: {details['boundary']}", details)
    if isinstance(exc, (TimeoutError, socket.timeout)):
        stage = "timeout" if run is not None and run.execution_budget is not None else "preflight"
        details = {
            "exception_type": type(exc).__name__,
            "exception_message": str(exc),
            "boundary": str(exc),
        }
        if run is not None and run.execution_budget is not None:
            details.update(run.attempt_count_details())
            details.update({
                "dispatch_cutoff_unix": run.dispatch_cutoff_unix,
                "observed_unix": time.time(),
            })
        else:
            details.update({"attempts_total": 0, "initial_attempts": 0, "fresh_integration_attempts": 0})
        return ControllerFailure(stage, f"{stage} timeout: {exc}", details)
    return None


def controller_failure_record(
    *,
    task: tasklib.Task,
    agent,
    rid: str,
    ticket_id: str,
    root: Path,
    source: dict,
    build: dict,
    failure: ControllerFailure,
    control: dict | None,
    process_records: list[dict],
    setup_seconds: float,
    execution_seconds: float,
    cleanup_seconds: float,
    total_wall_seconds: float,
    source_capture_seconds: float,
    failure_evidence: dict | None = None,
) -> dict:
    classification = _fixture_classification(failure.stage)
    details = jsonable(failure.details)
    attempts_total = int(details.get("attempts_total", classification["attempts"]))
    initial_attempts = int(details.get("initial_attempts", min(attempts_total, 2)))
    fresh_integration_attempts = int(details.get("fresh_integration_attempts", 0))
    reconciliation = next(
        (entry.get("record") for entry in process_records if entry.get("name") == "reconciliation" and isinstance(entry.get("record"), dict)),
        None,
    )
    failure_root = root / "failure"
    failure_root.mkdir(mode=0o700, parents=True, exist_ok=True)
    failure_manifest = failure_root / "failure-manifest.json"
    missing_paths = [str(path) for path in details.get("missing_paths", [])]
    write_json(
        failure_manifest,
        {
            "schema_version": "corenius.bench.controller-lifecycle-failure.v1",
            "stage": failure.stage,
            "message": failure.message,
            "details": details,
            "missing_paths": missing_paths,
            "task": task.id,
            "ticket": ticket_id,
        },
    )
    failure_log = failure_root / "failure.log"
    failure_log.write_text(f"{failure.stage}: {failure.message}\n")
    failure_log.chmod(0o600)
    harness_events_path = failure_root / "harness-events.jsonl"
    harness_events_path.write_text(
        json.dumps(
            {"kind": "harness_lifecycle_failure", "stage": failure.stage, "details": details},
            sort_keys=True,
        )
        + "\n"
    )
    harness_events_path.chmod(0o600)
    proof_path = failure_root / "harness-proof-status.json"
    write_json(
        proof_path,
        {
            "stage": failure.stage,
            "verdict": classification["task_verdict"],
            "trusted": False,
            "status": "unknown",
            "details": details,
        },
    )
    failure_evidence = failure_evidence or {}
    evidence_events = []
    actual_events = failure_evidence.get("events")
    if isinstance(actual_events, list):
        events_path = failure_root / "controller-events.json"
        write_json(events_path, {"schema_version": "corenius.bench.controller-events.v1", "events": actual_events})
        evidence_events.append({
            "kind": "controller",
            "path": str(events_path),
            "sha256": sha256_bytes(events_path.read_bytes()),
            "first_seq": actual_events[0]["seq"] if actual_events else 0,
            "last_seq": actual_events[-1]["seq"] if actual_events else 0,
            "count": len(actual_events),
        })
    else:
        evidence_events.append({
            "kind": "controller",
            "status": "unknown",
            "reason": failure_evidence.get("events_error", "controller events unavailable"),
        })
    evidence_events.append({
        "kind": "harness_annotation",
        "path": str(harness_events_path),
        "sha256": sha256_bytes(harness_events_path.read_bytes()),
    })
    evidence_proofs = []
    if isinstance(details.get("verification"), dict):
        actual_proof_path = failure_root / "controller-verification-proof.json"
        write_json(actual_proof_path, details["verification"])
        evidence_proofs.append({
            "kind": "controller_verification",
            "path": str(actual_proof_path),
            "sha256": sha256_bytes(actual_proof_path.read_bytes()),
            "verdict": details["verification"].get("verdict", "unknown"),
        })
    else:
        evidence_proofs.append({
            "kind": "controller_verification",
            "status": "unknown",
            "reason": "no controller verification proof was produced before failure",
        })
    process_logs = [
        evidence_entry(entry["name"], Path(entry["log"]))
        for entry in process_records
        if entry.get("log") and Path(entry["log"]).exists()
    ]
    retained = retained_evidence(
        root,
        repositories=[
            {
                "kind": "canonical",
                "path": str(root / "canonical"),
                "head": failure_evidence.get("canonical_head"),
                "head_status": "observed" if failure_evidence.get("canonical_head") else "unknown",
            },
            {"kind": "authority_store", "path": str(root / "authority/objects/store.git")},
        ],
        manifests=[
            evidence_entry("source", root / "source-manifest.json"),
            evidence_entry("failure", failure_manifest),
            evidence_entry("controller_build", Path(build["log"])) if build.get("log") else {},
        ],
        events=evidence_events,
        proofs=[
            *evidence_proofs,
            {
                "kind": "harness_proof_status",
                "path": str(proof_path),
                "sha256": sha256_bytes(proof_path.read_bytes()),
                "status": "unknown",
            },
        ],
        logs=[evidence_entry("failure", failure_log), *process_logs],
    )
    interventions = []
    if failure.stage == "manual_intervention":
        observed_intervention = details.get("intervention")
        interventions = [
            observed_intervention
            if isinstance(observed_intervention, dict)
            else {"kind": "manual_rescue", "stage": failure.stage, "summary": "manual intervention prevents autonomous PASS"}
        ]
    clock = execution_clock_from_control(control)
    model_usage = classification["model_usage"]
    unresolved = classification["effect_disposition"] == "unresolved"
    remaining_uncertain = (
        reconciliation.get("remaining_uncertain", []) if isinstance(reconciliation, dict) else []
    ) or (
        [{"kind": "effect", "stage": failure.stage, "reason": "physical stop is not inferred from lifecycle failure"}]
        if unresolved else []
    )
    reconciled_resolved = isinstance(reconciliation, dict) and reconciliation.get("status") == "resolved"
    effect_disposition = classification["effect_disposition"]
    if reconciled_resolved:
        effect_disposition = "resolved"
        remaining_uncertain = []
    elif isinstance(reconciliation, dict) and reconciliation.get("status") == "unresolved":
        effect_disposition = "unresolved"
    service_uncertain = [
        item for item in remaining_uncertain
        if isinstance(item, dict) and item.get("kind") == "service"
    ]
    attempt_uncertain = [
        item for item in remaining_uncertain
        if isinstance(item, dict) and item.get("kind") != "service"
    ]
    metadata = controller_result_metadata(
        root=root,
        admitted_at=clock["durable_run_admission_unix"],
        first_submission_at=clock["first_mission_submission_unix"],
        task_verdict=classification["task_verdict"],
        orchestration_verdict="fail",
        effect_disposition=effect_disposition,
        attempts_total=attempts_total,
        initial_attempts=initial_attempts,
        fresh_integration_attempts=fresh_integration_attempts,
        interventions=interventions,
        retained=retained,
        failure_stage=failure.stage,
        failure_kind=failure.stage,
        failure_message=failure.message,
        model_usage=model_usage,
    )
    verify_record = tasklib.capsule_for(task)
    controller = {
        "schema_version": CONTROLLER_SCHEMA,
        "mode": "deterministic",
        "task": task.id,
        "ticket": ticket_id,
        **metadata,
        "run_root": str(root),
        "source": source,
        "build": build,
        "control": control or {},
        "partial_contract": {
            "suite_id": "not_run" if failure.stage in PREFLIGHT_FAILURE_STAGES else "failed_lifecycle",
            "suite_digest": "not_run",
            "selected_tests": [],
            "noop_verdict": "not_run",
            "fixed_verdict": classification["task_verdict"],
        },
        "landing": {"state": "not_landed"},
        "process_disposition": {
            "owned_attempts": "unresolved" if attempt_uncertain else "stopped",
            "services": "unresolved" if service_uncertain else "stopped",
            "effect_disposition": effect_disposition,
            "processes": process_records,
            "remaining_uncertain": remaining_uncertain,
            **({"reconciliation": reconciliation} if isinstance(reconciliation, dict) else {}),
        },
        "runtime_closure": runtime_closure_for_failure(root, failure_manifest),
        "cleanup": {
            "bounded_timeout_seconds": CLEANUP_TIMEOUT_SECONDS,
            "seconds": cleanup_seconds,
            "observed_owned_processes": len(process_records),
        },
        "qualification_limits": [
            "controlled deterministic lifecycle failure only",
            "no live model calls",
            "no one-run speed, cost or recovery superiority claim",
        ],
        "failure_details": details,
        "orchestration_passed": False,
    }
    return {
        "schema_version": "corenius.bench.result.v1",
        "run_id": rid,
        "task_id": task.id,
        "condition": f"controller:{ticket_id}",
        "agent": {"spec": agent.spec},
        "capsule": {
            "digest": verify_record["capsule_digest"],
            "net": verify_record["policy"]["net"],
            "hermetic": verify_record["hermetic"],
            "non_hermetic_layers": verify_record["non_hermetic_layers"],
        },
        "outcome": {
            "passed": False,
            "reward": 0.0,
            "verifier_rc": 1 if classification["task_verdict"] == "fail" else None,
            "timed_out": failure.stage == "timeout",
            "agent_rc": None,
            "task_verdict": classification["task_verdict"],
            "orchestration_verdict": "fail",
            "effect_disposition": effect_disposition,
            "failure_stage": failure.stage,
        },
        "timing": {
            "wall_seconds": total_wall_seconds,
            "agent_seconds": 0.0,
            "verify_seconds": float(details.get("verify_seconds", 0.0)),
            "source_capture_seconds": source_capture_seconds,
            "build_seconds": build.get("seconds", 0.0),
            "setup_seconds": setup_seconds,
            "execution_seconds": execution_seconds,
            "controller_flow_seconds": execution_seconds,
            "cleanup_seconds": cleanup_seconds,
        },
        "coordination": {
            "conflicts": 0,
            "stale_refusals": 0,
            "redos": 0,
            "contamination": 0,
            "contamination_paths": [],
            "tickets": 2 if ticket_id == "complete" else 1,
            "tickets_landed": 0,
            "condition_mode": "controller",
            "attempts": attempts_total,
            "manual_interventions": len(interventions),
        },
        "cost": {
            "usd": model_usage["usd"],
            "input_tokens": model_usage["input_tokens"],
            "output_tokens": model_usage["output_tokens"],
        },
        "provenance": {
            "outcome": "controller lifecycle failure measured",
            "timing": "measured by harness phase timers",
            "coordination": "controller lifecycle observed",
            "cost": f"{model_usage['status']}; no synthesized usage total",
        },
        "controller": controller,
    }


def budget_timeout(budget, requested: float, boundary: str) -> float:
    if budget is None:
        return float(requested)
    if hasattr(budget, "timeout"):
        return budget.timeout(float(requested), boundary)
    return max(0.001, min(float(requested), budget.remaining()))


def budget_remaining(budget, boundary: str) -> float:
    if budget is None:
        return float("inf")
    if isinstance(budget, StageBudget):
        return budget.deadline_monotonic - time.monotonic()
    return budget.remaining()


def wait(description: str, predicate, seconds: float = 30.0, budget=None):
    deadline = time.monotonic() + budget_timeout(budget, seconds, description)
    last = None
    while time.monotonic() < deadline:
        if isinstance(budget, StageBudget):
            budget.remaining(description)
        try:
            value = predicate()
            if value:
                return value
            last = value
        except (ConnectionRefusedError, FileNotFoundError, TimeoutError, urllib.error.URLError, socket.timeout) as exc:
            last = exc
        time.sleep(0.05)
        if budget is not None:
            if budget_remaining(budget, description) <= 0:
                break
    if isinstance(budget, StageBudget) and budget_remaining(budget, description) <= 0:
        budget.remaining(description)
    raise TimeoutError(f"{description}: {last!r}")


def port() -> int:
    with socket.socket() as stream:
        stream.bind(("127.0.0.1", 0))
        return int(stream.getsockname()[1])


def fixed_git(path: Path, *args: str, check: bool = True, timeout: float | None = None) -> subprocess.CompletedProcess:
    proc = subprocess.run(
        [
            "/usr/bin/git",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-C",
            str(path),
            *args,
        ],
        env={
            "PATH": "/usr/bin:/bin",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": "/dev/null",
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_TERMINAL_PROMPT": "0",
        },
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    if check and proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or f"git {' '.join(args)} failed")
    return proc


def checkout_commit(repo: Path, commit: str, ws: Path, budget=None) -> None:
    ws.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [
            "/usr/bin/git",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-C",
            str(repo),
            "archive",
            "--format=tar",
            commit,
        ],
        env={
            "PATH": "/usr/bin:/bin",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": "/dev/null",
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_TERMINAL_PROMPT": "0",
        },
        capture_output=True,
        text=False,
        timeout=budget_timeout(budget, 30.0, "git archive"),
    )
    if proc.returncode != 0:
        stderr = proc.stderr.decode(errors="replace").strip()
        raise RuntimeError(stderr or f"git archive {commit} failed")
    extract_git_archive_safely(proc.stdout, ws)


def extract_git_archive_safely(data: bytes, ws: Path) -> None:
    ws.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:") as archive:
        members = archive.getmembers()
        total_bytes = 0
        if len(members) > 1024:
            raise RuntimeError("archive member bound exceeded")
        for member in members:
            if not member.name or member.name == ".":
                raise RuntimeError(f"unsafe archive member path: {member.name}")
            name = Path(member.name)
            if name.is_absolute() or "" in name.parts or ".." in name.parts:
                raise RuntimeError(f"unsafe archive member path: {member.name}")
            target = (ws / name).resolve()
            try:
                target.relative_to(ws.resolve())
            except ValueError as exc:
                raise RuntimeError(f"unsafe archive member path: {member.name}") from exc
            if member.isdir():
                continue
            if member.isfile():
                total_bytes += member.size
                if total_bytes > 16 * 1024 * 1024:
                    raise RuntimeError("archive byte bound exceeded")
                continue
            raise RuntimeError(f"unsupported archive member type: {member.name}")
        for member in members:
            target = ws / Path(member.name)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            source = archive.extractfile(member)
            if source is None:
                raise RuntimeError(f"archive member content unavailable: {member.name}")
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("wb") as stream:
                shutil.copyfileobj(source, stream)


def check_pin(record: dict) -> None:
    if sha256_bytes(Path(record["path"]).read_bytes()) != record["sha256"]:
        raise RuntimeError(f"pinned tool digest changed: {record['path']}")


def workspace_view_identity(root: Path) -> dict:
    entries = []
    for path in sorted(root.rglob("*")):
        rel = path.relative_to(root).as_posix()
        stat = path.lstat()
        if path.is_symlink():
            raise RuntimeError(f"unsupported verifier view symlink: {rel}")
        if path.is_dir():
            entries.append({"path": rel, "kind": "dir", "mode": stat.st_mode & 0o777})
            continue
        if path.is_file():
            entries.append({
                "path": rel,
                "kind": "file",
                "mode": stat.st_mode & 0o777,
                "bytes": stat.st_size,
                "sha256": sha256_bytes(path.read_bytes()),
            })
            continue
        raise RuntimeError(f"unsupported verifier view file type: {rel}")
    document = {"schema_version": "corenius.bench.verifier-view.v1", "entries": entries}
    return {"digest": sha256_bytes(json.dumps(document, sort_keys=True, separators=(",", ":")).encode()), "document": document}


def verify_runtime_manifest(runtime: dict) -> None:
    root = Path(runtime["root"])
    manifest = Path(runtime["manifest"]["path"])
    if sha256_bytes(manifest.read_bytes()) != runtime["manifest"]["sha256"]:
        raise RuntimeError("verifier runtime manifest digest changed")
    expected = json.loads(manifest.read_text())
    actual = runtime_manifest(root, manifest.parent / "independent-runtime-manifest-check.json")
    if actual["sha256"] != runtime["manifest"]["sha256"]:
        raise RuntimeError("verifier runtime manifest mismatch")
    if expected.get("schema_version") != "corenius.verifier-runtime.v1":
        raise RuntimeError("verifier runtime manifest schema changed")


def run_pinned_complete_suite(ws: Path, suite: dict, runtime: dict, tools: dict, host_root: Path, name: str,
                              budget=None, timeout_override: float | None = None) -> dict:
    verify_runtime_manifest(runtime)
    for key in ["python", "bubblewrap"]:
        check_pin(tools[key])
    check_pin({"path": suite["path"], "sha256": suite["sha256"]})
    suite_doc = complete_suite_entry("m01-disjoint-utils", "t2", suite, host_root)
    suite_digest = sha256_bytes(json.dumps(suite_doc, sort_keys=True, separators=(",", ":")).encode())
    started = time.time()
    argv = [
        tools["bubblewrap"]["path"],
        "--unshare-all",
        "--new-session",
        "--die-with-parent",
        "--as-pid-1",
        "--clearenv",
        "--setenv", "TMPDIR", "/run/corenius-tmp",
        "--ro-bind",
        runtime["root"],
        "/usr",
        "--symlink",
        "usr/bin",
        "/bin",
        "--symlink",
        "usr/lib",
        "/lib",
        "--symlink",
        "usr/lib64",
        "/lib64",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/run/corenius-tmp",
        "--dir",
        "/home",
        "--dir",
        "/home/worker",
        "--setenv",
        "HOME",
        "/home/worker",
        "--setenv",
        "PATH",
        "/usr/bin:/bin",
        "--bind",
        str(ws),
        "/workspace",
        "--tmpfs",
        "/scratch",
        "--chdir",
        "/workspace",
        "--ro-bind",
        suite["path"],
        "/run/verification.py",
        tools["python"]["path"],
        "/run/verification.py",
        str(host_root),
        suite["protected_test_sha256"],
    ]
    verifier_timeout = (
        float(timeout_override)
        if timeout_override is not None
        else budget_timeout(budget, PINNED_VERIFIER_TIMEOUT_SECONDS, f"independent verifier {name}")
    )
    try:
        proc = subprocess.run(
            argv,
            env={},
            capture_output=True,
            text=True,
            timeout=verifier_timeout,
        )
    except subprocess.TimeoutExpired as exc:
        raise subprocess.TimeoutExpired(
            ["independent verifier", name, *argv],
            exc.timeout,
            output=exc.output,
            stderr=exc.stderr,
        ) from exc
    elapsed = round(time.time() - started, 3)
    log = {
        "name": name,
        "argv": [tools["python"]["path"], "/run/verification.py"],
        "args": [str(Path(suite["path"]).parent), suite["protected_test_sha256"]],
        "exit": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "suite": suite_doc,
        "suite_digest": suite_digest,
        "tools": {"python": tools["python"], "bubblewrap": tools["bubblewrap"]},
        "runtime": {"manifest": runtime["manifest"]["sha256"], "root": runtime["root"]},
        "seconds": elapsed,
    }
    return {
        "passed": proc.returncode == 0,
        "rc": proc.returncode,
        "seconds": elapsed,
        "tail": "\n".join((proc.stdout + proc.stderr).splitlines()[-25:]),
        "log": log,
        "suite": {"id": suite_doc["id"], "digest": suite_digest},
    }


def independent_verify_commit(task: tasklib.Task, repo: Path, commit: str, suite: dict, runtime: dict, tools: dict,
                              root: Path, name: str,
                              tamper_tests: bool = False, budget=None,
                              timeout_override: float | None = None) -> dict:
    tree = fixed_git(
        repo,
        "rev-parse",
        "--verify",
        f"{commit}^{{tree}}",
        timeout=budget_timeout(budget, 10.0, f"independent verifier {name} rev-parse"),
    ).stdout.strip()
    ws = root / "independent-verifier" / name / "workspace"
    checkout_commit(repo, commit, ws, budget=budget)
    if tamper_tests:
        tests = ws / "tests/test_utils.py"
        tests.parent.mkdir(parents=True, exist_ok=True)
        tests.write_text("import unittest\n\nclass T(unittest.TestCase):\n    def test_tampered(self):\n        self.assertTrue(True)\n")
    source_view = workspace_view_identity(ws)
    verdict = run_pinned_complete_suite(
        ws,
        suite,
        runtime,
        tools,
        root,
        name,
        budget=budget,
        timeout_override=timeout_override,
    )
    verifier_view = workspace_view_identity(ws)
    evidence_root = root / "independent-verifier" / name
    log_path = evidence_root / "verify-log.json"
    tail_path = evidence_root / "verify-tail.txt"
    write_json(log_path, verdict["log"])
    tail_path.write_text(verdict["tail"])
    tail_path.chmod(0o600)
    return {
        "source_commit": commit,
        "source_tree": tree,
        "source_view": source_view,
        "verifier_view": verifier_view,
        "commit": commit,
        "verdict": "pass" if verdict["passed"] else "fail",
        "rc": verdict["rc"],
        "seconds": verdict["seconds"],
        "tail_sha256": sha256_bytes(verdict["tail"].encode()),
        "tail": str(tail_path),
        "log": {"path": str(log_path), "sha256": pin(log_path)["sha256"]},
        "suite": verdict["suite"],
        "script": {"path": suite["path"], "sha256": suite["sha256"]},
        "runtime": {"status": "pinned", "root": runtime["root"], "manifest": runtime["manifest"]},
        "tools": {"python": tools["python"], "bubblewrap": tools["bubblewrap"]},
        "tampered_tests_before_restore": tamper_tests,
    }


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_root_parent() -> Path:
    selected = os.environ.get("TMPDIR")
    if not selected:
        raise RuntimeError("Run with tools/temp_root.py or set an owned TMPDIR")
    parent = Path(selected).resolve()
    forbidden = Path("/") / "tmp"
    if parent == forbidden or forbidden in parent.parents:
        raise RuntimeError("TMPDIR must be outside the system temporary directory")
    parent.mkdir(parents=True, exist_ok=True)
    return parent


def copy_runtime_path(source: Path, runtime_root: Path) -> Path:
    source = Path(source)
    source_bytes = source.resolve() if source.is_symlink() else source
    text = str(source)
    if text.startswith("/usr/"):
        rel = source.relative_to("/usr")
    elif text.startswith("/lib64/"):
        rel = Path("lib64") / source.relative_to("/lib64")
    elif text.startswith("/lib/"):
        rel = Path("lib") / source.relative_to("/lib")
    else:
        raise RuntimeError(f"runtime dependency outside supported roots: {source}")
    dest = runtime_root / rel
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists() and sha256_bytes(dest.read_bytes()) == sha256_bytes(source_bytes.read_bytes()):
        return dest
    shutil.copy2(source_bytes, dest)
    return dest


def selected_python_layout(python: Path, budget=None) -> dict:
    proc = subprocess.run(
        [
            str(python),
            "-I",
            "-S",
            "-c",
            (
                "import json,sysconfig;"
                "print(json.dumps({"
                "'stdlib':sysconfig.get_path('stdlib'),"
                "'destshared':sysconfig.get_config_var('DESTSHARED') or ''"
                "},sort_keys=True))"
            ),
        ],
        capture_output=True,
        text=True,
        check=True,
        timeout=budget_timeout(budget, 10.0, "python runtime layout"),
    )
    return json.loads(proc.stdout)


def ldd_dependencies(paths: list[Path], budget=None) -> list[Path]:
    found: set[Path] = set()
    for path in paths:
        proc = subprocess.run(
            ["/usr/bin/ldd", str(path)],
            capture_output=True,
            text=True,
            check=True,
            timeout=budget_timeout(budget, 10.0, f"ldd {path}"),
        )
        for line in proc.stdout.splitlines():
            parts = line.strip().split()
            candidates = []
            if "=>" in parts:
                idx = parts.index("=>")
                if idx + 1 < len(parts):
                    candidates.append(parts[idx + 1])
            elif parts and parts[0].startswith("/"):
                candidates.append(parts[0])
            for item in candidates:
                candidate = Path(item)
                if item.startswith(("/lib/", "/lib64/", "/usr/lib/")) and candidate.is_file():
                    found.add(candidate)
    return sorted(found)


def runtime_manifest(runtime_root: Path, manifest_path: Path) -> dict:
    entries = []
    for path in sorted(runtime_root.rglob("*")):
        rel = path.relative_to(runtime_root).as_posix()
        stat = path.lstat()
        if path.is_symlink():
            target = os.readlink(path)
            if Path(target).is_absolute() or ".." in Path(rel).parent.joinpath(target).parts:
                raise RuntimeError(f"escaping runtime symlink refused: {rel} -> {target}")
            entries.append({"path": rel, "kind": "symlink", "target": target})
        elif path.is_file():
            data = path.read_bytes()
            entries.append({"path": rel, "kind": "regular", "bytes": stat.st_size, "sha256": sha256_bytes(data)})
    document = {"schema_version": "corenius.verifier-runtime.v1", "entries": entries}
    manifest_path.write_text(json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n")
    manifest_path.chmod(0o600)
    return {"path": str(manifest_path), "sha256": sha256_bytes(manifest_path.read_bytes()), "entries": len(entries)}


def build_verifier_runtime(root: Path, python_pin: dict, budget=None) -> dict:
    runtime_root = root / "runtime-snapshot" / "usr"
    runtime_root.mkdir(parents=True, exist_ok=True)
    python = Path(python_pin["path"])
    python_dest = copy_runtime_path(python, runtime_root)
    if not python_dest.is_file() or sha256_bytes(python_dest.read_bytes()) != python_pin["sha256"]:
        raise RuntimeError(f"runtime snapshot missing pinned interpreter: {python}")
    layout = selected_python_layout(python, budget=budget)
    stdlib = Path(layout["stdlib"]).resolve()
    shutil.copytree(stdlib, runtime_root / stdlib.relative_to("/usr"), dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    dynload = Path(layout["destshared"] or stdlib / "lib-dynload").resolve()
    if dynload.exists():
        shutil.copytree(dynload, runtime_root / dynload.relative_to("/usr"), dirs_exist_ok=True,
                        ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    shared_inputs = [python]
    if dynload.exists():
        shared_inputs.extend(path for path in dynload.glob("*.so") if path.is_file())
    for dep in ldd_dependencies(shared_inputs, budget=budget):
        copy_runtime_path(dep, runtime_root)
    ambient = root / "ambient-runtime-source"
    ambient.mkdir(mode=0o700, parents=True, exist_ok=True)
    control_source = ambient / "value.txt"
    control_source.write_text("snapshotted\n")
    control = runtime_root / "share/corenius-runtime-control/value.txt"
    control.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(control_source, control)
    (ambient / "ambient_only.py").write_text("raise AssertionError('ambient source imported')\n")
    control_source.write_text("ambient changed after snapshot\n")
    manifest = runtime_manifest(runtime_root, root / "runtime-manifest.json")
    return {
        "root": str(runtime_root),
        "manifest": manifest,
        "control": {"path": str(control), "sha256": sha256_bytes(control.read_bytes())},
    }


def git_path_list(*args: str, budget=None) -> list[str]:
    proc = subprocess.run(
        ["/usr/bin/git", "-C", str(tasklib.ROOT), *args],
        capture_output=True,
        check=True,
        timeout=budget_timeout(budget, 20.0, f"source manifest git {' '.join(args)}"),
    )
    return [item.decode() for item in proc.stdout.split(b"\0") if item]


def source_content_manifest(budget=None) -> dict:
    paths = set(EXECUTION_SOURCE_PATHS)
    paths.update(git_path_list("diff", "--name-only", "-z", "HEAD", "--", budget=budget))
    paths.update(git_path_list("diff", "--cached", "--name-only", "-z", "--", budget=budget))
    paths.update(git_path_list("ls-files", "--others", "--exclude-standard", "-z", budget=budget))
    entries = []
    for rel in sorted(paths):
        path = tasklib.ROOT / rel
        if path.is_file():
            data = path.read_bytes()
            entries.append({"path": rel, "kind": "file", "bytes": len(data), "sha256": sha256_bytes(data)})
        else:
            entries.append({"path": rel, "kind": "missing"})
    digest = sha256_bytes(json.dumps(entries, sort_keys=True, separators=(",", ":")).encode())
    return {"schema_version": "corenius.bench.source-manifest.v1", "root": str(tasklib.ROOT), "digest": digest,
            "entries": entries}


def source_identity(budget=None) -> dict:
    status = fixed_git(
        tasklib.ROOT,
        "status",
        "--porcelain",
        "--untracked-files=all",
        timeout=budget_timeout(budget, 30.0, "source identity git status"),
    ).stdout
    diff = subprocess.run(
        ["/usr/bin/git", "-C", str(tasklib.ROOT), "diff", "--binary", "HEAD"],
        capture_output=True,
        text=False,
        check=True,
        timeout=budget_timeout(budget, 30.0, "source identity git diff"),
    ).stdout
    manifest = source_content_manifest(budget=budget)
    return {
        "head": fixed_git(
            tasklib.ROOT,
            "rev-parse",
            "HEAD",
            timeout=budget_timeout(budget, 10.0, "source identity git rev-parse HEAD"),
        ).stdout.strip(),
        "tree": fixed_git(
            tasklib.ROOT,
            "rev-parse",
            "HEAD^{tree}",
            timeout=budget_timeout(budget, 10.0, "source identity git rev-parse tree"),
        ).stdout.strip(),
        "dirty": bool(status.strip()),
        "status_sha256": sha256_bytes(status.encode()),
        "diff_sha256": sha256_bytes(diff),
        "content_manifest": manifest,
    }


def build_controller(root: Path, budget=None) -> dict:
    target = tasklib.ROOT / ".scratch" / "controller-backed-bench" / "target"
    target.mkdir(parents=True, exist_ok=True)
    log = root / "controller-build.log"
    started = time.time()
    with log.open("ab") as stream:
        try:
            proc = subprocess.run(
                [
                    "cargo",
                    "build",
                    "--manifest-path",
                    str(tasklib.ROOT / "crates/sqlite-controller/Cargo.toml"),
                    "--target-dir",
                    str(target),
                    "--locked",
                    "--offline",
                    "--bins",
                ],
                stdout=stream,
                stderr=subprocess.STDOUT,
                timeout=budget_timeout(budget, SETUP_STAGE_TIMEOUT_SECONDS, "cargo build sqlite controller"),
            )
        except subprocess.TimeoutExpired as exc:
            raise ControllerFailure(
                "preflight",
                f"controller build exceeded setup budget; see {log}",
                {"boundary": "cargo build", "timeout_seconds": exc.timeout, "log": str(log)},
            ) from exc
    elapsed = round(time.time() - started, 3)
    controller = target / "debug/corenius-sqlite-controller"
    supervisor = target / "debug/corenius-attempt-supervisor"
    gate = target / "debug/corenius-attempt-gate"
    record = {
        "command": "cargo build --manifest-path crates/sqlite-controller/Cargo.toml --target-dir .scratch/controller-backed-bench/target --locked --offline --bins",
        "exit": proc.returncode,
        "seconds": elapsed,
        "log": str(log),
    }
    if proc.returncode != 0:
        raise ControllerFailure(
            "preflight",
            f"controller build failed; see {log}",
            {"boundary": "cargo build", "returncode": proc.returncode, "log": str(log)},
        )
    record.update({
        "controller": pin(controller),
        "supervisor": pin(supervisor),
        "attempt_gate": pin(gate),
    })
    return record


def write_partial_suite(path: Path, task: tasklib.Task) -> dict:
    protected = (task.path / "tests/test_utils.py").read_bytes()
    encoded = base64.b64encode(protected).decode()
    selected = {ticket: data["selected_tests"][0] for ticket, data in PARTIAL_CONTRACTS.items()}
    script = f"""import base64
import hashlib
import importlib.util
import pathlib
import sys
import unittest

workspace = pathlib.Path("/workspace")
sys.path.insert(0, str(workspace))
ticket = sys.argv[1]
host_root = pathlib.Path(sys.argv[2])
expected_digest = sys.argv[3]
selected = {json.dumps(selected, sort_keys=True)}[ticket]
assert not host_root.exists(), "controller root must be absent from verifier"
assert "CapEff:\\t0000000000000000" in pathlib.Path("/proc/self/status").read_text()
control = pathlib.Path("/usr/share/corenius-runtime-control/value.txt")
assert control.read_text() == "snapshotted\\n", "verifier consumed ambient runtime instead of snapshot"
try:
    control.write_text("verifier wrote runtime")
    raise AssertionError("verifier runtime snapshot is writable")
except OSError:
    pass
try:
    __import__("ambient_only")
    raise AssertionError("omitted ambient module loaded")
except ModuleNotFoundError:
    pass
test_bytes = base64.b64decode({encoded!r})
assert hashlib.sha256(test_bytes).hexdigest() == expected_digest
target = workspace / "tests" / "test_utils.py"
target.parent.mkdir(parents=True, exist_ok=True)
target.write_bytes(test_bytes)
spec = importlib.util.spec_from_file_location("protected_test_utils", target)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
suite = unittest.defaultTestLoader.loadTestsFromName(selected, module)
result = unittest.TextTestRunner(verbosity=2).run(suite)
if result.wasSuccessful():
    print("PARTIAL_CONTRACT_PASS", ticket, selected, "PROTECTED_TESTS_RESTORED")
sys.exit(0 if result.wasSuccessful() else 1)
"""
    path.write_text(script)
    path.chmod(0o600)
    return {"path": str(path), "sha256": sha256_bytes(script.encode()), "protected_test_sha256": sha256_bytes(protected)}


def write_complete_suite(path: Path, task: tasklib.Task) -> dict:
    protected = (task.path / "tests/test_utils.py").read_bytes()
    encoded = base64.b64encode(protected).decode()
    selected = ["T.test_title_keeps_small_words_lower_and_collapses_spaces", "T.test_clamp_both_ends"]
    script = f"""import base64
import hashlib
import importlib.util
import pathlib
import sys
import unittest

workspace = pathlib.Path("/workspace")
sys.path.insert(0, str(workspace))
host_root = pathlib.Path(sys.argv[1])
expected_digest = sys.argv[2]
selected = {json.dumps(selected, sort_keys=True)}
assert not host_root.exists(), "controller root must be absent from verifier"
assert "CapEff:\\t0000000000000000" in pathlib.Path("/proc/self/status").read_text()
control = pathlib.Path("/usr/share/corenius-runtime-control/value.txt")
assert control.read_text() == "snapshotted\\n", "verifier consumed ambient runtime instead of snapshot"
try:
    control.write_text("verifier wrote runtime")
    raise AssertionError("verifier runtime snapshot is writable")
except OSError:
    pass
try:
    __import__("ambient_only")
    raise AssertionError("omitted ambient module loaded")
except ModuleNotFoundError:
    pass
test_bytes = base64.b64decode({encoded!r})
assert hashlib.sha256(test_bytes).hexdigest() == expected_digest
target = workspace / "tests" / "test_utils.py"
target.parent.mkdir(parents=True, exist_ok=True)
target.write_bytes(test_bytes)
spec = importlib.util.spec_from_file_location("protected_test_utils", target)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromName(name, module) for name in selected)
result = unittest.TextTestRunner(verbosity=2).run(suite)
if result.wasSuccessful():
    print("COMPLETE_CONTRACT_PASS", ",".join(selected), "PROTECTED_TESTS_RESTORED")
sys.exit(0 if result.wasSuccessful() else 1)
"""
    path.write_text(script)
    path.chmod(0o600)
    return {
        "path": str(path),
        "sha256": sha256_bytes(script.encode()),
        "protected_test_sha256": sha256_bytes(protected),
        "selected_tests": selected,
    }


def suite_entry(task_id: str, task_name: str, base: str, suite: dict, root: Path, ticket_id: str, suffix: str = "") -> dict:
    return {
        "id": f"{task_id}:{ticket_id}:partial{suffix}",
        "task": task_name,
        "base": base,
        "script": {"path": suite["path"], "sha256": suite["sha256"]},
        "args": [ticket_id, str(root), suite["protected_test_sha256"]],
    }


def complete_suite_entry(task_id: str, task_name: str, suite: dict, root: Path) -> dict:
    return {
        "id": f"{task_id}:complete",
        "task": task_name,
        "base": None,
        "script": {"path": suite["path"], "sha256": suite["sha256"]},
        "args": [str(root), suite["protected_test_sha256"]],
    }


def controller_reply(socket_path: Path, value: dict, timeout: float = 40.0) -> dict:
    with socket.socket(socket.AF_UNIX) as stream:
        stream.settimeout(max(0.001, float(timeout)))
        stream.connect(str(socket_path))
        stream.sendall(json.dumps(value).encode() + b"\n")
        return json.loads(stream.makefile("rb").readline())


def controller_request(socket_path: Path, value: dict, timeout: float = 40.0) -> dict:
    reply = controller_reply(socket_path, value, timeout=timeout)
    if "error" in reply:
        raise RuntimeError(reply["error"])
    return reply["ok"]


def http_json(origin: str, method: str, path: str, token: str | None = None, body: dict | None = None,
              timeout: float = 5.0) -> dict:
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = "Bearer " + token
    req = urllib.request.Request(
        origin + "/api/v1" + path,
        data=None if body is None else json.dumps(body).encode(),
        method=method,
        headers=headers,
    )
    with urllib.request.urlopen(req, timeout=max(0.001, float(timeout))) as response:
        return json.load(response)


def extract_diff_paths(diff: str) -> list[str]:
    paths = []
    for line in diff.splitlines():
        match = re.match(r"diff --git a/(.+?) b/(.+)", line)
        if match:
            paths.append(match.group(2))
    return sorted(set(paths))


def ticket_overlay(ticket: fleet.Ticket) -> dict[str, str]:
    return {
        str(path.relative_to(ticket.path / "overlay")): path.read_text()
        for path in sorted((ticket.path / "overlay").rglob("*"))
        if path.is_file()
    }


def useful_interval(ticket: str, attempt: dict) -> dict:
    raw = attempt["ready"]["useful_interval"]
    return {
        "ticket": ticket,
        "attempt": attempt["grant"]["attempt"],
        "base": attempt["grant"]["base"],
        "candidate": attempt["candidate"]["candidate"],
        "commit": attempt["candidate"]["commit"],
        "start_monotonic_ns": raw["start_monotonic_ns"],
        "end_monotonic_ns": raw["end_monotonic_ns"],
        "start_time_ns": raw["start_time_ns"],
        "end_time_ns": raw["end_time_ns"],
        "steps": raw["steps"],
    }


def measured_overlap(first: dict, second: dict) -> dict:
    intervals = [first, second]
    start = max(entry["start_monotonic_ns"] for entry in intervals)
    end = min(entry["end_monotonic_ns"] for entry in intervals)
    seconds = round(max(0, end - start) / 1_000_000_000, 6)
    return {
        "basis": "worker-reported monotonic interval from file write/check/commit only",
        "seconds": seconds,
        "intervals": intervals,
    }


def public_grant(grant: dict) -> dict:
    return {k: grant[k] for k in ["attempt", "operation", "slot", "principal", "scope_epoch", "base", "brief"]}


def stopped_services(processes: list[ManagedProcess], timeout_seconds: float = CLEANUP_TIMEOUT_SECONDS,
                     budget: CleanupBudget | None = None) -> list[dict]:
    local_budget = budget or CleanupBudget(timeout_seconds)
    records = []
    for entry in reversed(processes):
        proc = entry.process
        if proc.poll() is None:
            proc.terminate()
            if local_budget.remaining() > 0:
                try:
                    proc.wait(timeout=local_budget.wait_slice(5.0))
                except subprocess.TimeoutExpired:
                    proc.kill()
                    if local_budget.remaining() > 0:
                        try:
                            proc.wait(timeout=local_budget.wait_slice(5.0))
                        except subprocess.TimeoutExpired:
                            pass
            elif proc.poll() is None:
                proc.kill()
        records.append(entry.record())
    return list(reversed(records))


def observed_process_disposition(process_records: list[dict], attempt_records: list[dict]) -> dict:
    """Report physical closure independently of task and orchestration results."""
    services = [
        {"kind": "service", "id": f"service:{record.get('name')}", "pid": record.get("pid"),
         "reason": "owned service exit was not observed after bounded cleanup"}
        for record in process_records if record.get("exit") is None
    ]
    attempts = [
        {"kind": "attempt", "id": record.get("attempt"), "record": record.get("record"),
         "state": record.get("state"), "reason": "identity-bound attempt exit was not observed"}
        for record in attempt_records if record.get("state") != "exited"
    ]
    remaining = services + attempts
    return {
        "owned_attempts": "unresolved" if attempts else "stopped",
        "services": "unresolved" if services else "stopped",
        "effect_disposition": "unresolved" if remaining else "resolved",
        "processes": process_records,
        "attempts": attempt_records,
        "remaining_uncertain": remaining,
    }


class ControllerRun:
    def __init__(self, task: tasklib.Task, ticket: fleet.Ticket, root: Path, build: dict, source: dict,
                 fault: str | None = None, setup_budget: StageBudget | None = None):
        if fault is not None and fault not in LIFECYCLE_FAULTS:
            raise ValueError(f"unsupported controller lifecycle fault: {fault}")
        self.task = task
        self.ticket = ticket
        self.root = root
        self.build = build
        self.source = source
        self.fault = fault
        self.setup_budget = setup_budget
        self.execution_budget: StageBudget | None = None
        self.processes: list[ManagedProcess] = []
        self.admitted_attempt_ids: list[str] = []
        self.initial_attempt_ids: list[str] = []
        self.fresh_integration_attempt_ids: list[str] = []
        self.operator_token = uuid.uuid4().hex + uuid.uuid4().hex
        self.supervisor_token = uuid.uuid4().hex + uuid.uuid4().hex
        self.kata_bootstrap = uuid.uuid4().hex + uuid.uuid4().hex
        self.kata_token = ""
        self.kata_origin = ""
        self.project_revision = 0
        self.dispatch_cutoff_unix: float | None = None
        self.control: dict | None = None
        self.failure_evidence_before_service_stop: dict | None = None
        self.setup_seconds = 0.0
        self.flow_started: float | None = None

    def setup_timeout(self, seconds: float, boundary: str) -> float:
        return budget_timeout(self.setup_budget, seconds, boundary)

    def execution_timeout(self, seconds: float, boundary: str) -> float:
        return budget_timeout(self.execution_budget, seconds, boundary)

    @property
    def controller_socket(self) -> Path:
        return self.root / "controller.sock"

    def spawn(self, name: str, argv: list[Path | str], env: dict[str, str]) -> ManagedProcess:
        log = self.root / f"{name}.log"
        with log.open("ab") as stream:
            proc = subprocess.Popen(
                [str(a) for a in argv],
                cwd=self.root,
                env=env,
                stdout=stream,
                stderr=subprocess.STDOUT,
                start_new_session=True,
            )
        start_ticks = Path(f"/proc/{proc.pid}/stat").read_text().rsplit(") ", 1)[1].split()[19]
        entry = ManagedProcess(name, proc, log, start_ticks)
        self.processes.append(entry)
        return entry

    def query(self, kind: str = "snapshot", args: dict | None = None, auth: dict | None = None,
              timeout: float = 40.0, bypass_execution_deadline: bool = False, budget=None) -> dict:
        socket_timeout = budget_timeout(
            budget,
            timeout,
            f"query {kind}",
        ) if budget is not None else (
            timeout if bypass_execution_deadline or self.execution_budget is None
            else self.execution_timeout(timeout, f"query {kind}")
        )
        return controller_request(
            self.controller_socket,
            {"auth": auth or {"principal": "operator", "token": self.operator_token},
             "project": PROJECT, "query": {"kind": kind, "args": args or {}}},
            timeout=socket_timeout,
        )

    def command(self, kind: str, args: dict, auth: dict | None = None, key: str | None = None,
                allow_error: bool = False, bypass_execution_deadline: bool = False, budget=None) -> dict:
        if not bypass_execution_deadline:
            self.enforce_execution_deadline(kind)
        auth = auth or {"principal": "operator", "token": self.operator_token}
        idempotency_key = key or f"controller-bench-{kind}-{uuid.uuid4()}"
        last = None
        for _ in range(25):
            revision = 0 if kind == "create_project" else self.query(
                bypass_execution_deadline=bypass_execution_deadline,
                budget=budget,
            )["revision"]
            socket_timeout = budget_timeout(
                budget,
                40.0,
                f"command {kind}",
            ) if budget is not None else (
                40.0 if bypass_execution_deadline or self.execution_budget is None
                else self.execution_timeout(40.0, f"command {kind}")
            )
            reply = controller_reply(
                self.controller_socket,
                {"auth": auth,
                 "project": PROJECT,
                 "command": {
                     "kind": kind,
                     "expected_revision": revision,
                     "idempotency_key": idempotency_key,
                     "args": args,
                 }},
                timeout=socket_timeout,
            )
            if not bypass_execution_deadline:
                self.enforce_execution_deadline(kind)
            if "ok" in reply:
                return reply["ok"]
            last = RuntimeError(reply["error"])
            if not str(last).startswith("revision conflict:"):
                if allow_error:
                    return reply
                raise last
            time.sleep(0.02)
        raise RuntimeError(str(last))

    def enforce_execution_deadline(self, kind: str) -> None:
        if self.execution_budget is None or self.dispatch_cutoff_unix is None:
            return
        remaining = self.execution_budget.deadline_monotonic - time.monotonic()
        if remaining > 0:
            return
        observed = time.time()
        counts = self.attempt_count_details()
        raise ControllerFailure(
            "timeout",
            f"controller execution deadline elapsed before {kind}",
            {
                "command": kind,
                "dispatch_cutoff_unix": self.dispatch_cutoff_unix,
                "observed_unix": observed,
                "deadline_monotonic_elapsed_seconds": round(-remaining, 6),
                **counts,
            },
        )

    def flow_seconds_so_far(self) -> float:
        if self.flow_started is None:
            return 0.0
        return round(time.time() - self.flow_started, 3)

    def setup_git(self, cwd: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess:
        proc = subprocess.run(
            ["git", *fleet.GIT_ENV, "-C", str(cwd), *args],
            capture_output=True,
            text=True,
            timeout=self.setup_timeout(30.0, f"setup git {' '.join(args)}"),
        )
        if check and proc.returncode != 0:
            raise ControllerFailure(
                "preflight",
                f"git {' '.join(args)} failed in {cwd}: {proc.stderr.strip() or proc.stdout.strip()}",
                {"boundary": "setup git", "args": list(args), "cwd": str(cwd), "returncode": proc.returncode},
            )
        return proc

    def observed_attempt_records(self) -> list[dict]:
        records = []
        attempts_root = self.root / "attempts"
        seen: set[str] = set()
        if attempts_root.is_dir():
            paths = sorted(attempts_root.glob("*.json"))
        else:
            paths = []
        for path in paths:
            attempt_id = path.stem
            try:
                value = json.loads(path.read_text())
            except (OSError, json.JSONDecodeError) as exc:
                seen.add(attempt_id)
                records.append({
                    "name": f"attempt:{attempt_id}",
                    "attempt": attempt_id,
                    "pid": None,
                    "start_ticks": None,
                    "exit": None,
                    "state": None,
                    "outcome": None,
                    "record": str(path),
                    "identity_evidence": "unreadable",
                    "identity_error": f"{type(exc).__name__}: {exc}",
                })
                continue
            binding = value.get("binding") if isinstance(value.get("binding"), dict) else {}
            operation_id = str(value.get("operation") or binding.get("operation") or path.stem)
            attempt_id = str(value.get("attempt") or binding.get("attempt") or attempt_id)
            seen.add(attempt_id)
            records.append({
                "name": f"attempt:{attempt_id}",
                "attempt": attempt_id,
                "operation": operation_id,
                "pid": value.get("pid"),
                "start_ticks": value.get("start_ticks"),
                "exit": value.get("exit"),
                "state": value.get("state"),
                "outcome": value.get("outcome"),
                "record": str(path),
                "log": str(path),
                "identity_evidence": "read",
            })
        for attempt_id in sorted(set(self.admitted_attempt_ids) - seen):
            path = attempts_root / f"{attempt_id}.json"
            records.append({
                "name": f"attempt:{attempt_id}",
                "attempt": attempt_id,
                "pid": None,
                "start_ticks": None,
                "exit": None,
                "state": None,
                "outcome": None,
                "record": str(path),
                "identity_evidence": "missing",
            })
        return records

    def attempt_count_details(self) -> dict:
        total = len(set(self.admitted_attempt_ids))
        try:
            snapshot = self.query(timeout=3.0, bypass_execution_deadline=True)
            total = max(total, len(snapshot.get("attempts", [])))
        except Exception:
            total = max(total, len(self.observed_attempt_records()))
        return {
            "attempts_total": total,
            "initial_attempts": len(set(self.initial_attempt_ids)),
            "fresh_integration_attempts": len(set(self.fresh_integration_attempt_ids)),
        }

    def active_attempt_ids(self, budget: CleanupBudget | None = None) -> list[str]:
        ids: set[str] = set(self.admitted_attempt_ids)
        proven_exited: set[str] = set()
        if budget is None or budget.remaining() > 0:
            try:
                snapshot = self.query(timeout=3.0, bypass_execution_deadline=True, budget=budget)
                for attempt in snapshot.get("attempts", []):
                    if attempt.get("state") != "exited":
                        ids.add(attempt["id"])
                    else:
                        proven_exited.add(attempt["id"])
            except Exception:
                pass
        for record in self.observed_attempt_records():
            attempt_id = record.get("attempt") or Path(record.get("record", "")).stem
            if not attempt_id:
                continue
            if record.get("identity_evidence") == "read" and record.get("state") == "exited":
                proven_exited.add(str(attempt_id))
            else:
                ids.add(str(attempt_id))
        return sorted(ids - proven_exited)

    def reconcile_owned_processes(self, timeout_seconds: float = CLEANUP_TIMEOUT_SECONDS) -> list[dict]:
        budget = CleanupBudget(timeout_seconds)
        reconciliation = {
            "status": "unknown",
            "authority": "controller.cancel_attempt",
            "bounded": True,
            "identity_bound": True,
            "timeout_seconds": float(timeout_seconds),
            "cancel_attempts": [],
            "observed_attempt_ids": self.active_attempt_ids(budget=budget),
        }
        for attempt in reconciliation["observed_attempt_ids"]:
            if budget.remaining() <= 0:
                break
            try:
                reply = self.command(
                    "cancel_attempt",
                    {"attempt": attempt},
                    allow_error=True,
                    bypass_execution_deadline=True,
                    budget=budget,
                    key=f"cancel-{attempt}",
                )
                reconciliation["cancel_attempts"].append({"attempt": attempt, "reply": reply})
            except Exception as exc:
                reconciliation["cancel_attempts"].append(
                    {"attempt": attempt, "error": f"{type(exc).__name__}: {exc}"}
                )
        try:
            if reconciliation["observed_attempt_ids"] and budget.remaining() > 0:
                wait(
                    "attempt cancellation reconciliation",
                    lambda: all(
                        attempt.get("state") == "exited"
                        for attempt in self.query(timeout=3.0, bypass_execution_deadline=True, budget=budget).get("attempts", [])
                    ),
                    seconds=budget.remaining(),
                    budget=budget,
                )
        except Exception as exc:
            reconciliation["wait_error"] = f"{type(exc).__name__}: {exc}"
        attempt_records = self.observed_attempt_records()
        self.failure_evidence_before_service_stop = collect_failure_evidence(self, budget=budget)
        service_records = stopped_services(self.processes, timeout_seconds=timeout_seconds, budget=budget)
        unresolved_attempts = [
            record
            for record in attempt_records
            if not (record.get("identity_evidence") == "read" and record.get("state") == "exited")
        ]
        unresolved_services = [
            record
            for record in service_records
            if record.get("exit") is None
        ]
        reconciliation["cleanup_seconds"] = budget.elapsed()
        reconciliation["final_attempt_records"] = attempt_records
        reconciliation["final_service_records"] = service_records
        if unresolved_attempts or unresolved_services:
            reconciliation["status"] = "unresolved"
            reconciliation["remaining_uncertain"] = [
                {"kind": "attempt", "id": record.get("name"), "record": record.get("record"), "state": record.get("state")}
                for record in unresolved_attempts
            ] + [
                {
                    "kind": "service",
                    "id": f"service:{record.get('name')}",
                    "pid": record.get("pid"),
                    "start_ticks": record.get("start_ticks"),
                    "log": record.get("log"),
                    "reason": "service exit was not observed within cleanup budget",
                }
                for record in unresolved_services
            ]
        else:
            reconciliation["status"] = "resolved"
        records = service_records + attempt_records
        records.append({"name": "reconciliation", "record": reconciliation})
        return records

    def setup_repositories(self, tickets: list[fleet.Ticket] | None = None) -> tuple[str, str]:
        tickets = tickets or [self.ticket]
        for name in ["authority", "attempts", "canonical", "seed", "intent", "kata"]:
            path = self.root / name
            path.mkdir(mode=0o700, parents=True, exist_ok=True)
        self.setup_timeout(30.0, "materialize setup repositories")
        self.setup_git(self.root / "canonical", "init", "--bare", "--template=")
        tasklib.materialize(self.task, self.root / "seed")
        self.setup_git(self.root / "seed", "init", "-q", "-b", "main")
        self.setup_git(self.root / "seed", "add", "-A")
        self.setup_git(self.root / "seed", "commit", "-q", "-m", "m01 start")
        base = self.setup_git(self.root / "seed", "rev-parse", "HEAD").stdout.strip()
        self.setup_git(self.root / "seed", "push", str(self.root / "canonical"), f"{base}:refs/heads/protected")

        self.setup_git(self.root / "intent", "init", "-q", "-b", "main")
        (self.root / "intent/spec.md").write_text((self.task.path / "instruction.md").read_text())
        for ticket in tickets:
            (self.root / "intent" / f"{ticket.id}.md").write_text(ticket.instruction)
        self.setup_git(self.root / "intent", "add", ".")
        self.setup_git(
            self.root / "intent",
            "-c",
            "user.name=Controller bench intent",
            "-c",
            "user.email=controller-bench@invalid",
            "commit",
            "-q",
            "-m",
            "m01 controller intent",
        )
        intent = self.setup_git(self.root / "intent", "rev-parse", "HEAD").stdout.strip()
        return base, intent

    def setup_kata(self) -> dict:
        if self.fault == "missing-prerequisite":
            missing = self.root / "missing-kata-binary"
            raise ControllerFailure(
                "missing_prerequisite",
                f"pinned Kata binary unavailable: {missing}",
                {"boundary": "setup_kata", "missing_paths": [missing], "attempts_total": 0, "initial_attempts": 0},
            )
        if not KATA.is_file() or pin(KATA)["sha256"] != KATA_SHA256:
            raise ControllerFailure(
                "missing_prerequisite",
                "pinned Kata 0.24.0 binary unavailable or hash changed",
                {"boundary": "setup_kata", "missing_paths": [KATA], "attempts_total": 0, "initial_attempts": 0},
            )
        (self.root / "kata/config.toml").write_text(
            f'[auth]\ntoken="{self.kata_bootstrap}"\nrequire_token_identity=true\n'
        )
        (self.root / "kata/config.toml").chmod(0o600)
        kport = port()
        self.kata_origin = f"http://127.0.0.1:{kport}"
        env = {k: v for k, v in os.environ.items() if not k.startswith("KATA_")}
        env["KATA_HOME"] = str(self.root / "kata")
        self.spawn("kata", [KATA, "daemon", "start", "--foreground", "--listen", f"127.0.0.1:{kport}"], env)
        wait(
            "Kata health",
            lambda: http_json(
                self.kata_origin,
                "GET",
                "/health",
                token=None,
                timeout=self.setup_timeout(5.0, "kata health"),
            ),
            20,
            budget=self.setup_budget,
        )
        token = http_json(
            self.kata_origin,
            "POST",
            "/tokens",
            token=self.kata_bootstrap,
            body={"actor": "corenius-controller", "name": "controller-bench"},
            timeout=self.setup_timeout(5.0, "kata token"),
        )["plaintext"]
        self.kata_token = token
        (self.root / "kata.token").write_text(token)
        (self.root / "kata.token").chmod(0o600)
        project = http_json(
            self.kata_origin,
            "POST",
            "/projects",
            token=token,
            body={"name": self.root.name},
            timeout=self.setup_timeout(5.0, "kata project"),
        )["project"]
        instance = http_json(
            self.kata_origin,
            "GET",
            "/instance",
            token=token,
            timeout=self.setup_timeout(5.0, "kata instance"),
        )["instance_uid"]
        return {"binary": pin(KATA), "origin": self.kata_origin, "instance_uid": instance, "project": project}

    def create_kata_issue(self, title: str, body: str, project: dict) -> dict:
        if self.fault == "delivery-failure" and title == f"{self.task.id} {self.ticket.id}":
            try:
                http_json(
                    self.kata_origin,
                    "POST",
                    f"/projects/{project['id']}-missing/issues",
                    token=self.kata_token,
                    body={"title": title, "body": body, "force_new": True},
                    timeout=self.execution_timeout(5.0, "kata issue delivery fault"),
                )
            except Exception as exc:
                raise ControllerFailure(
                    "delivery_failure",
                    f"Kata issue delivery failed at controlled transport boundary: {exc}",
                    {
                        **self.attempt_count_details(),
                        "boundary": "kata issue delivery",
                        "transport_error": f"{type(exc).__name__}: {exc}",
                    },
                ) from exc
            raise ControllerFailure(
                "delivery_failure",
                "controlled delivery failure did not fail at Kata issue delivery boundary",
                {**self.attempt_count_details(), "boundary": "kata issue delivery"},
            )
        return http_json(
            self.kata_origin,
            "POST",
            f"/projects/{project['id']}/issues",
            token=self.kata_token,
            body={"title": title, "body": body, "force_new": True},
            timeout=self.execution_timeout(5.0, "kata issue"),
        )["issue"]

    def setup_policy(self, base: str, suite: dict, kata: dict, tickets: list[fleet.Ticket] | None = None,
                     complete_suite: dict | None = None) -> dict:
        tickets = tickets or [self.ticket]
        python = pin("/usr/bin/python3")
        verifier_runtime = build_verifier_runtime(self.root, python, budget=self.setup_budget)
        supervisor_config = {
            "root": str(self.root / "attempts"),
            "profile": {"executable": "/usr/bin/python3", "args": ["-c", WORKER_SCRIPT], "codex": None},
        }
        write_json(self.root / "supervisor.json", supervisor_config)
        suites = []
        for ticket in tickets:
            suites.extend([
                suite_entry(self.task.id, f"{ticket.id}-noop", base, suite, self.root, ticket.id, ":noop"),
                suite_entry(self.task.id, ticket.id, base, suite, self.root, ticket.id),
            ])
        if complete_suite is not None:
            suites.append(complete_suite_entry(self.task.id, tickets[-1].id, complete_suite, self.root))
        policy = {
            "project": PROJECT,
            "root": str(self.root / "authority"),
            "attempts_root": str(self.root / "attempts"),
            "canonical": str(self.root / "canonical"),
            "protected_ref": "refs/heads/protected",
            "git": pin("/usr/bin/git"),
            "python": python,
            "bubblewrap": pin("/usr/bin/bwrap"),
            "verifier_runtime": {
                "root": verifier_runtime["root"],
                "manifest": {
                    "path": verifier_runtime["manifest"]["path"],
                    "sha256": verifier_runtime["manifest"]["sha256"],
                },
            },
            "suites": suites,
        }
        write_json(self.root / "git.json", policy)
        kata_config = {
            "schema_migration": "kata-v1",
            "controller_id": self.root.name,
            "actor": "corenius-controller",
            "api_schema_version": "0.24.0",
            "origin_instance_uid": kata["instance_uid"],
            "origin": kata["origin"],
            "token_file": str(self.root / "kata.token"),
            "project_id": kata["project"]["id"],
            "project_uid": kata["project"]["uid"],
            "repositories": {"intent": str(self.root / "intent")},
        }
        write_json(self.root / "kata.json", kata_config)
        return {
            "git_policy_sha256": pin(self.root / "git.json")["sha256"],
            "kata_config_sha256": pin(self.root / "kata.json")["sha256"],
            "supervisor_config_sha256": pin(self.root / "supervisor.json")["sha256"],
            "tools": {key: policy[key] for key in ["git", "python", "bubblewrap"]},
            "verifier_runtime": verifier_runtime,
            "suites": suites,
            "complete_suite": complete_suite,
        }

    def start_services(self) -> None:
        supervisor_env = {
            "PATH": "/usr/bin:/bin",
            "CORENIUS_SUPERVISOR_TOKEN": self.supervisor_token,
            "CORENIUS_GIT_CONFIG": str(self.root / "git.json"),
        }
        self.spawn(
            "supervisor",
            [self.build["supervisor"]["path"], self.root / "supervisor.json", self.root / "supervisor.sock"],
            supervisor_env,
        )
        wait("supervisor socket", lambda: (self.root / "supervisor.sock").exists(), 20, budget=self.setup_budget)
        controller_env = {
            "PATH": "/usr/bin:/bin",
            "CORENIUS_OPERATOR_TOKEN": self.operator_token,
            "CORENIUS_SUPERVISOR_TOKEN": self.supervisor_token,
            "CORENIUS_SUPERVISOR_SOCKET": str(self.root / "supervisor.sock"),
            "CORENIUS_GIT_CONFIG": str(self.root / "git.json"),
            "CORENIUS_KATA_CONFIG": str(self.root / "kata.json"),
        }
        self.spawn(
            "controller",
            [self.build["controller"]["path"], self.root / "state.db", self.controller_socket],
            controller_env,
        )
        wait("controller socket", lambda: self.controller_socket.exists(), 20, budget=self.setup_budget)
        wait(
            "controller storage",
            lambda: self.query("storage_configuration", bypass_execution_deadline=True, budget=self.setup_budget),
            20,
            budget=self.setup_budget,
        )

    def create_project_and_lead(self, base: str, intent: str, tickets: list[fleet.Ticket] | None = None,
                                complete: bool = False) -> tuple[dict, dict]:
        tickets = tickets or [self.ticket]
        ticket_ids = [ticket.id for ticket in tickets]
        project = self.command(
            "create_project",
            {"name": PROJECT, "base": base},
            key="create-project",
            bypass_execution_deadline=True,
            budget=self.setup_budget,
        )
        lead = self.command(
            "replace_lead",
            {},
            key="replace-lead",
            bypass_execution_deadline=True,
            budget=self.setup_budget,
        )["result"]
        lead_auth = {"principal": lead["principal"], "token": lead["token"]}
        mission = {
            "objective": (
                f"Run complete {self.task.id} through the controller-backed benchmark path"
                if complete else f"Run {self.task.id} {self.ticket.id} through the controller-backed benchmark path"
            ),
            "plan": (
                "deterministic lead creates two Kata-backed tasks, admits two supervised workers from the same base, "
                "verifies and accepts both exact candidates, lands the first, observes stale refusal for the second, "
                "then admits one fresh integration attempt with fresh verification and acceptance"
                if complete else
                "deterministic lead creates one Kata-backed task, admits one supervised worker, verifies the exact candidate, accepts it and lands the disposable protected ref"
            ),
            "constraints": "no model calls; disposable repository only; protected verifier inputs restored by pinned suite",
            "intent": f"{self.task.id}:{','.join(ticket_ids)}:{intent}",
        }
        write_json(self.root / "mission.json", mission)
        admitted_at = time.time()
        if self.fault == "elapsed-deadline":
            admitted_at -= EXECUTION_DEADLINE_SECONDS + 0.001
        self.dispatch_cutoff_unix = admitted_at + EXECUTION_DEADLINE_SECONDS
        self.execution_budget = StageBudget(
            seconds=EXECUTION_DEADLINE_SECONDS,
            failure_stage="timeout",
            label="controller execution",
        )
        if self.fault == "elapsed-deadline":
            self.execution_budget.deadline_monotonic = time.monotonic() - 0.001
        write_json(
            self.root / "durable-run-admission.json",
            {
                "schema_version": "corenius.bench.durable-run-admission.v1",
                "task": self.task.id,
                "ticket": "complete" if complete else self.ticket.id,
                "admitted_unix": admitted_at,
                "deadline_seconds": EXECUTION_DEADLINE_SECONDS,
                "dispatch_cutoff_unix": self.dispatch_cutoff_unix,
                "mission_sha256": sha256_bytes(json.dumps(mission, sort_keys=True).encode()),
            },
        )
        control = {
            "project": project,
            "lead": {k: lead[k] for k in ["principal", "role_epoch"]},
            "mission": None,
            "execution_clock": {
                "durable_run_admission_unix": admitted_at,
                "first_mission_submission_unix": admitted_at,
            },
        }
        self.control = control
        mission_receipt = self.command("set_mission", mission, auth=lead_auth, key="set-mission")
        first_submission_at = time.time()
        control["mission"] = mission_receipt
        control["execution_clock"]["first_mission_submission_unix"] = first_submission_at
        self.control = control
        return lead_auth, control

    def raise_observed_manual_intervention(self, lead_auth: dict) -> None:
        receipt = self.command(
            "pause_dispatch",
            {"paused": True},
            auth=lead_auth,
            key="manual-pause-dispatch",
        )
        snapshot = self.query()
        project = snapshot.get("project") if isinstance(snapshot.get("project"), dict) else {}
        observed = {
            "paused": project.get("paused"),
            "revision": snapshot.get("revision"),
            "event_seq": receipt.get("event_seq"),
        }
        if observed["paused"] is not True:
            raise RuntimeError(f"manual intervention pause was not observed: {observed}")
        raise ControllerFailure(
            "manual_intervention",
            "controller dispatch paused by observed manual intervention",
            {
                **self.attempt_count_details(),
                "intervention": {
                    "kind": "manual_pause_dispatch",
                    "authority": "controller.pause_dispatch",
                    "receipt": receipt,
                    "observation": observed,
                },
            },
        )

    def create_controller_task(self, task_name: str, issue: dict, intent: str, lead_auth: dict,
                               ticket: fleet.Ticket | None = None) -> dict:
        ticket = ticket or self.ticket
        args = {
            "task": task_name,
            "item": issue["uid"],
            "expected_source_revision": issue["revision"],
            "source_files": {
                "storage": "intent",
                "mode": "git",
                "commit": intent,
                "paths": ["spec.md", f"{ticket.id}.md"],
            },
        }
        return self.command("create_kata_task", args, auth=lead_auth, key=f"create-{task_name}")

    def begin_attempt(self, task_name: str, lead_auth: dict, action: dict, key_prefix: str) -> dict:
        admission = self.command("admit", {"task": task_name}, auth=lead_auth, key=f"admit-{key_prefix}")
        grant = admission["result"]
        self.admitted_attempt_ids.append(grant["attempt"])
        if key_prefix.startswith("fresh-"):
            self.fresh_integration_attempt_ids.append(grant["attempt"])
        else:
            self.initial_attempt_ids.append(grant["attempt"])
        root = self.root / "attempts" / grant["attempt"]
        wait(
            "attempt workspace",
            lambda: (root / "workspace/.git/objects").is_dir(),
            30,
            budget=self.execution_budget,
        )
        if self.fault == "provider-failure" and task_name == self.ticket.id:
            action = {
                "ticket": action.get("ticket", self.ticket.id),
                "files": {},
                "check": "raise SystemExit('controlled provider failure before candidate')",
            }
        write_json(root / "scratch/action.json", action)
        return {
            "admission": admission,
            "grant": grant,
            "root": str(root),
        }

    def release_useful_start(self, attempt: dict) -> None:
        root = Path(attempt["root"])
        (root / "scratch/start_useful").write_text("start\n")

    def wait_attempt_candidate(self, task_name: str, lead_auth: dict, attempt: dict, key_prefix: str) -> dict:
        root = Path(attempt["root"])
        ready = wait(
            "worker useful checkpoint",
            lambda: json.loads((root / "scratch/ready.json").read_text()) if (root / "scratch/ready.json").exists() else None,
            30,
            budget=self.execution_budget,
        )
        worker_auth = {"principal": attempt["grant"]["principal"], "token": attempt["grant"]["token"]}
        progress = self.command(
            "report_progress",
            {"task": task_name, "text": f"deterministic worker produced {ready['commit']} for {ready['ticket']}"},
            auth=worker_auth,
            key=f"progress-{key_prefix}",
        )
        candidate = self.command(
            "capture_candidate",
            {"report": f"deterministic worker commit {ready['commit']} for {self.task.id} {ready['ticket']}"},
            auth=worker_auth,
            key=f"capture-{key_prefix}",
        )
        return {**attempt, "ready": ready, "progress": progress, "candidate": candidate["result"]}

    def run_attempt(self, task_name: str, lead_auth: dict, action: dict, key_prefix: str | None = None) -> dict:
        key_prefix = key_prefix or task_name
        attempt = self.begin_attempt(task_name, lead_auth, action, key_prefix)
        try:
            return self.wait_attempt_candidate(task_name, lead_auth, attempt, key_prefix)
        except TimeoutError as exc:
            if self.fault == "provider-failure" and task_name == self.ticket.id:
                raise ControllerFailure(
                    "provider_failure",
                    f"participant failed before producing a candidate: {exc}",
                    {
                        **self.attempt_count_details(),
                        "boundary": "worker useful checkpoint",
                        "attempt": attempt["grant"]["attempt"],
                        "attempt_records": self.observed_attempt_records(),
                    },
                ) from exc
            raise

    def finish_attempt(self, attempt: str) -> dict:
        root = self.root / "attempts" / attempt
        (root / "scratch/finish").write_text("controller-bench deterministic attempt complete\n")
        return wait(
            "attempt exited",
            lambda: self.query() if any(a["id"] == attempt and a["state"] == "exited" for a in self.query()["attempts"]) else None,
            30,
            budget=self.execution_budget,
        )

    def attempt_process_record(self, grant: dict) -> dict:
        record_path = self.root / "attempts" / f"{grant['operation']}.json"
        value = json.loads(record_path.read_text())
        return {
            "attempt": grant["attempt"],
            "operation": grant["operation"],
            "pid": value.get("pid"),
            "start_ticks": value.get("start_ticks"),
            "namespace_init": value.get("namespace_init"),
            "init_start_ticks": value.get("init_start_ticks"),
            "pid_namespace": value.get("pid_namespace"),
            "state": value.get("state"),
            "outcome": value.get("outcome"),
            "record": str(record_path),
        }

    def verify_accept_land(self, candidate: dict, lead_auth: dict, base: str, ticket: fleet.Ticket | None = None,
                           key_prefix: str = "fixed") -> tuple[dict, dict, dict, dict, dict]:
        ticket = ticket or self.ticket
        timings = {}
        started = time.time()
        comparison = self.command("compare_candidates", {"candidate": candidate["candidate"]}, auth=lead_auth, key=f"compare-{key_prefix}")
        timings["compare_seconds"] = round(time.time() - started, 3)
        diff_paths = extract_diff_paths(comparison["result"]["diff"])
        contamination = sorted(p for p in diff_paths if not fleet.allowed(p, ticket.allow))
        started = time.time()
        verification = self.command("verify_candidate", {"candidate": candidate["candidate"]}, auth=lead_auth, key=f"verify-{key_prefix}")
        timings["fixed_verify_seconds"] = round(time.time() - started, 3)
        if contamination:
            raise RuntimeError(f"controller candidate changed paths outside ticket allowlist: {contamination}")
        if verification["result"]["verdict"] != "pass":
            raise ControllerFailure(
                "failed_verifier",
                f"controller partial verifier did not pass: {verification['result']['verdict']}",
                {
                    "verification": verification["result"],
                    "verify_seconds": timings["fixed_verify_seconds"],
                    **self.attempt_count_details(),
                },
            )
        started = time.time()
        acceptance = self.command(
            "accept_candidate",
            {"candidate": candidate["candidate"], "verification": verification["result"]["verification"]},
            auth=lead_auth,
            key=f"accept-{key_prefix}",
        )
        timings["accept_seconds"] = round(time.time() - started, 3)
        started = time.time()
        landing = self.command(
            "land_candidate",
            {"candidate": candidate["candidate"], "expected_trunk": base},
            auth=lead_auth,
            key=f"land-{key_prefix}",
        )
        timings["land_seconds"] = round(time.time() - started, 3)
        return comparison["result"], verification["result"], acceptance["result"], landing["result"], timings

    def verify_accept(self, candidate: dict, lead_auth: dict, ticket: fleet.Ticket, key_prefix: str) -> tuple[dict, dict, dict, dict]:
        timings = {}
        started = time.time()
        comparison = self.command("compare_candidates", {"candidate": candidate["candidate"]}, auth=lead_auth, key=f"compare-{key_prefix}")
        timings["compare_seconds"] = round(time.time() - started, 3)
        diff_paths = extract_diff_paths(comparison["result"]["diff"])
        contamination = sorted(p for p in diff_paths if not fleet.allowed(p, ticket.allow))
        if contamination:
            raise RuntimeError(f"controller candidate changed paths outside ticket allowlist: {contamination}")
        started = time.time()
        verification = self.command("verify_candidate", {"candidate": candidate["candidate"]}, auth=lead_auth, key=f"verify-{key_prefix}")
        timings["verify_seconds"] = round(time.time() - started, 3)
        if verification["result"]["verdict"] != "pass":
            raise ControllerFailure(
                "failed_verifier",
                f"controller verifier did not pass: {verification['result']['verdict']}",
                {
                    "verification": verification["result"],
                    "verify_seconds": timings["verify_seconds"],
                    **self.attempt_count_details(),
                },
            )
        started = time.time()
        acceptance = self.command(
            "accept_candidate",
            {"candidate": candidate["candidate"], "verification": verification["result"]["verification"]},
            auth=lead_auth,
            key=f"accept-{key_prefix}",
        )
        timings["accept_seconds"] = round(time.time() - started, 3)
        return comparison["result"], verification["result"], acceptance["result"], timings

    def run(self) -> dict:
        started = time.time()
        setup_started = time.time()
        base, intent = self.setup_repositories()
        suite = write_partial_suite(self.root / "partial_verifier.py", self.task)
        kata = self.setup_kata()
        policy = self.setup_policy(base, suite, kata)
        self.start_services()
        self.setup_seconds = round(time.time() - setup_started, 3)
        lead_auth, control = self.create_project_and_lead(base, intent)
        if self.fault == "generic-failure":
            raise RuntimeError("controlled generic harness failure after durable mission admission")
        if self.fault == "manual-intervention":
            self.raise_observed_manual_intervention(lead_auth)
        setup_seconds = round(time.time() - setup_started, 3)
        self.setup_seconds = setup_seconds
        flow_started = time.time()
        self.flow_started = flow_started
        project = kata["project"]
        noop_issue = self.create_kata_issue(
            f"{self.task.id} {self.ticket.id} noop control",
            "Negative control: submit the base tree for the selected partial contract.",
            project,
        )
        fixed_issue = self.create_kata_issue(
            f"{self.task.id} {self.ticket.id}",
            self.ticket.instruction,
            project,
        )
        noop_task = f"{self.ticket.id}-noop"
        self.create_controller_task(noop_task, noop_issue, intent, lead_auth)
        phase_started = time.time()
        noop = self.run_attempt(noop_task, lead_auth, {"ticket": self.ticket.id, "files": {}, "check": ""})
        noop_attempt_seconds = round(time.time() - phase_started, 3)
        phase_started = time.time()
        noop_verification = self.command(
            "verify_candidate",
            {"candidate": noop["candidate"]["candidate"]},
            auth=lead_auth,
            key="verify-noop",
        )["result"]
        noop_verify_seconds = round(time.time() - phase_started, 3)
        phase_started = time.time()
        self.finish_attempt(noop["grant"]["attempt"])
        noop_finish_seconds = round(time.time() - phase_started, 3)

        overlay = failing_overlay(self.ticket) if self.fault == "failed-verifier" else ticket_overlay(self.ticket)
        self.create_controller_task(self.ticket.id, fixed_issue, intent, lead_auth)
        phase_started = time.time()
        fixed = self.run_attempt(
            self.ticket.id,
            lead_auth,
            {
                "ticket": self.ticket.id,
                "files": overlay,
                "check": "" if self.fault == "failed-verifier" else PARTIAL_CONTRACTS[self.ticket.id]["check"],
            },
        )
        fixed_attempt_seconds = round(time.time() - phase_started, 3)
        comparison, verification, acceptance, landing, authority_timings = self.verify_accept_land(fixed["candidate"], lead_auth, base)
        phase_started = time.time()
        final_snapshot = self.finish_attempt(fixed["grant"]["attempt"])
        fixed_finish_seconds = round(time.time() - phase_started, 3)
        inventory = self.query("workspace_inventory")
        events = self.query("events", {"after": 0, "limit": 100})
        final_ref = fixed_git(
            self.root / "canonical",
            "rev-parse",
            "refs/heads/protected",
            timeout=self.execution_timeout(10.0, "final protected ref read"),
        ).stdout.strip()
        flow_seconds = round(time.time() - flow_started, 3)
        cleanup_started = time.time()
        process_records = stopped_services(self.processes)
        cleanup_seconds = round(time.time() - cleanup_started, 3)
        attempt_records = [self.attempt_process_record(noop["grant"]), self.attempt_process_record(fixed["grant"])]
        if self.fault == "missing-evidence":
            missing_path = self.root / "source-manifest.json"
            missing_path.unlink(missing_ok=True)
            raise ControllerFailure(
                "missing_evidence",
                f"required retained evidence artifact is missing: {missing_path}",
                {
                    "missing_paths": [missing_path],
                    "attempts_total": 2,
                    "initial_attempts": 1,
                    "cleanup_seconds_before_failure": cleanup_seconds,
                    "observed_processes_before_failure": process_records,
                    "observed_attempts_before_failure": attempt_records,
                },
            )
        timing = {
            "wall_seconds": round(time.time() - started, 3),
            "setup_seconds": setup_seconds,
            "controller_flow_seconds": flow_seconds,
            "attempt_seconds": round(noop_attempt_seconds + fixed_attempt_seconds, 3),
            "verify_seconds": round(noop_verify_seconds + authority_timings["fixed_verify_seconds"], 3),
            "noop_attempt_seconds": noop_attempt_seconds,
            "noop_verify_seconds": noop_verify_seconds,
            "noop_finish_seconds": noop_finish_seconds,
            "fixed_attempt_seconds": fixed_attempt_seconds,
            "fixed_verify_seconds": authority_timings["fixed_verify_seconds"],
            "fixed_finish_seconds": fixed_finish_seconds,
            "compare_seconds": authority_timings["compare_seconds"],
            "accept_seconds": authority_timings["accept_seconds"],
            "land_seconds": authority_timings["land_seconds"],
            "cleanup_seconds": cleanup_seconds,
        }
        partial = {
            "suite_id": verification["document"]["suite_id"],
            "suite_digest": verification["document"]["suite"],
            "selected_tests": PARTIAL_CONTRACTS[self.ticket.id]["selected_tests"],
            "protected_test_sha256": suite["protected_test_sha256"],
            "complete_original_test_file_restored": True,
            "noop_verdict": noop_verification["verdict"],
            "fixed_verdict": verification["verdict"],
        }
        process_disposition = observed_process_disposition(process_records, attempt_records)
        orchestration_passed = (
            process_disposition["effect_disposition"] == "resolved"
            and noop_verification["verdict"] == "fail"
            and verification["verdict"] == "pass"
            and acceptance["candidate"] == fixed["candidate"]["candidate"]
            and landing["state"] == "landed"
            and final_ref == fixed["candidate"]["commit"]
            and not comparison["diff"] == ""
            and not any(p for p in extract_diff_paths(comparison["diff"]) if not fleet.allowed(p, self.ticket.allow))
            and any(a["id"] == fixed["grant"]["attempt"] and a["state"] == "exited" for a in final_snapshot["attempts"])
        )
        task_verdict = "partial_pass" if partial["fixed_verdict"] == "pass" else "fail"
        orchestration_verdict = "pass" if orchestration_passed else "fail"
        retained = retained_evidence(
            self.root,
            repositories=[
                {"kind": "canonical", "path": str(self.root / "canonical"), "protected_ref": "refs/heads/protected", "head": final_ref},
                {"kind": "authority_store", "path": str(self.root / "authority/objects/store.git")},
            ],
            manifests=[
                evidence_entry("source", self.root / "source-manifest.json"),
                evidence_entry("git_policy", self.root / "git.json"),
                evidence_entry("kata_config", self.root / "kata.json"),
                evidence_entry("supervisor_config", self.root / "supervisor.json"),
            ],
            events=[{"kind": "controller", "first_seq": 1, "last_seq": events[-1]["seq"] if events else 0, "count": len(events)}],
            proofs=[
                {"kind": "noop", "verification": noop_verification["verification"], "report": noop_verification["report"],
                 "verdict": noop_verification["verdict"]},
                {"kind": "fixed", "verification": verification["verification"], "report": verification["report"],
                 "verdict": verification["verdict"]},
            ],
            logs=[evidence_entry(entry["name"], Path(entry["log"])) for entry in process_records],
        )
        metadata = controller_result_metadata(
            root=self.root,
            admitted_at=control["execution_clock"]["durable_run_admission_unix"],
            first_submission_at=control["execution_clock"]["first_mission_submission_unix"],
            task_verdict=task_verdict,
            orchestration_verdict=orchestration_verdict,
            effect_disposition=process_disposition["effect_disposition"],
            attempts_total=2,
            initial_attempts=1,
            fresh_integration_attempts=0,
            interventions=[],
            retained=retained,
        )
        return {
            "schema_version": CONTROLLER_SCHEMA,
            "mode": "deterministic",
            "task": self.task.id,
            "ticket": self.ticket.id,
            **metadata,
            "run_root": str(self.root),
            "source": self.source,
            "build": self.build,
            "control": control,
            "roles": {"lead": control["lead"], "noop_attempt": public_grant(noop["grant"]), "fixed_attempt": public_grant(fixed["grant"])},
            "kata": {"binary": kata["binary"], "instance_uid": kata["instance_uid"], "project": kata["project"],
                     "issues": {"noop": noop_issue, "fixed": fixed_issue}, "config_sha256": policy["kata_config_sha256"]},
            "git": {"policy_sha256": policy["git_policy_sha256"], "canonical": str(self.root / "canonical"),
                    "protected_ref": "refs/heads/protected", "base": base, "final": final_ref},
            "supervisor": {"config_sha256": policy["supervisor_config_sha256"]},
            "runtime_closure": {
                "status": "pinned",
                "pinned": {
                    "verification_adapter": {"path": suite["path"], "sha256": suite["sha256"]},
                    "protected_test_sha256": suite["protected_test_sha256"],
                    "git": policy["tools"]["git"],
                    "python_executable": policy["tools"]["python"],
                    "bubblewrap": policy["tools"]["bubblewrap"],
                    "verifier_runtime": policy["verifier_runtime"],
                },
                "enforced": "sqlite-controller validates the pinned runtime manifest, ro-binds that private snapshot at /usr, and binds the runtime digest into verifier proof",
            },
            "partial_contract": partial,
            "noop": {"task": noop_task, "candidate": noop["candidate"], "verification": noop_verification},
            "candidate": fixed["candidate"],
            "comparison": {"changed_paths": extract_diff_paths(comparison["diff"]), "contamination": []},
            "verification": verification,
            "acceptance": acceptance,
            "landing": landing,
            "process_disposition": process_disposition,
            "events_observed": {"count": len(events), "last_seq": events[-1]["seq"] if events else 0},
            "workspace_inventory": {
                "configured": inventory.get("configured"),
                "backend": inventory.get("backend"),
                "workspaces": len(inventory.get("workspaces", [])),
                "history": len(inventory.get("history", [])),
            },
            "timing": timing,
            "qualification_limits": [
                "single deterministic ticket only",
                "no live model calls",
                "no two-worker overlap, stale-base integration or permanent readiness claim",
            ],
            "orchestration_passed": orchestration_passed,
        }

    def run_complete(self, tickets: list[fleet.Ticket]) -> dict:
        tickets_by_id = {ticket.id: ticket for ticket in tickets}
        if set(tickets_by_id) != {"t1", "t2"}:
            raise RuntimeError("complete controller mode requires m01 tickets t1 and t2")

        started = time.time()
        setup_started = time.time()
        base, intent = self.setup_repositories(tickets)
        partial_suite = write_partial_suite(self.root / "partial_verifier.py", self.task)
        complete_suite = write_complete_suite(self.root / "complete_verifier.py", self.task)
        kata = self.setup_kata()
        policy = self.setup_policy(base, partial_suite, kata, tickets, complete_suite=complete_suite)
        self.start_services()
        self.setup_seconds = round(time.time() - setup_started, 3)
        lead_auth, control = self.create_project_and_lead(base, intent, tickets, complete=True)
        if self.fault == "generic-failure":
            raise RuntimeError("controlled generic harness failure after durable mission admission")
        if self.fault == "manual-intervention":
            self.raise_observed_manual_intervention(lead_auth)
        setup_seconds = round(time.time() - setup_started, 3)
        self.setup_seconds = setup_seconds
        flow_started = time.time()
        self.flow_started = flow_started
        project = kata["project"]

        issues = {
            ticket.id: self.create_kata_issue(f"{self.task.id} {ticket.id}", ticket.instruction, project)
            for ticket in tickets
        }
        for ticket in tickets:
            self.create_controller_task(ticket.id, issues[ticket.id], intent, lead_auth, ticket)

        begin_started = time.time()
        initial_attempts = {}
        for ticket in tickets:
            initial_attempts[ticket.id] = self.begin_attempt(
                ticket.id,
                lead_auth,
                {
                    "ticket": ticket.id,
                    "files": ticket_overlay(ticket),
                    "check": PARTIAL_CONTRACTS[ticket.id]["check"],
                    "check_repeat": 20_000,
                    "wait_for_useful_start": True,
                },
                key_prefix=f"initial-{ticket.id}",
            )
        for attempt in initial_attempts.values():
            root = Path(attempt["root"])
            wait(
                "worker ready for useful release",
                lambda root=root: (root / "scratch/waiting_for_useful_start").exists(),
                30,
                budget=self.execution_budget,
            )
        for attempt in initial_attempts.values():
            self.release_useful_start(attempt)
        initial = {
            ticket_id: self.wait_attempt_candidate(ticket_id, lead_auth, attempt, key_prefix=f"initial-{ticket_id}")
            for ticket_id, attempt in initial_attempts.items()
        }
        initial_attempt_wall_seconds = round(time.time() - begin_started, 3)
        if {initial["t1"]["grant"]["base"], initial["t2"]["grant"]["base"]} != {base}:
            raise RuntimeError("initial worker attempts did not start from the same canonical base")
        two_worker_snapshot = self.query()
        if two_worker_snapshot["occupied_slots"] != 2:
            raise RuntimeError(f"expected two occupied worker slots, saw {two_worker_snapshot['occupied_slots']}")
        overlap = measured_overlap(useful_interval("t1", initial["t1"]), useful_interval("t2", initial["t2"]))
        if overlap["seconds"] <= 0:
            raise RuntimeError(f"useful worker execution did not overlap: {overlap}")

        initial_proofs = {}
        authority_timings = {}
        for ticket_id in ["t1", "t2"]:
            comparison, verification, acceptance, timings = self.verify_accept(
                initial[ticket_id]["candidate"],
                lead_auth,
                tickets_by_id[ticket_id],
                key_prefix=f"initial-{ticket_id}",
            )
            initial_proofs[ticket_id] = {
                "comparison": {"changed_paths": extract_diff_paths(comparison["diff"]), "contamination": []},
                "verification": verification,
                "acceptance": acceptance,
            }
            authority_timings[ticket_id] = timings

        land_started = time.time()
        first_landing = self.command(
            "land_candidate",
            {"candidate": initial["t1"]["candidate"]["candidate"], "expected_trunk": base},
            auth=lead_auth,
            key="land-initial-t1",
        )["result"]
        first_land_seconds = round(time.time() - land_started, 3)
        if first_landing["state"] != "landed":
            raise RuntimeError(f"first candidate did not land: {first_landing}")
        stale_started = time.time()
        stale_landing = self.command(
            "land_candidate",
            {"candidate": initial["t2"]["candidate"]["candidate"], "expected_trunk": base},
            auth=lead_auth,
            key="land-initial-t2-stale",
        )["result"]
        stale_land_seconds = round(time.time() - stale_started, 3)
        if stale_landing["state"] != "stale_base" or not stale_landing.get("requires_new_attempt"):
            raise RuntimeError(f"second candidate did not receive stale-base refusal: {stale_landing}")

        finish_started = time.time()
        initial_finish_snapshots = {
            "t1": self.finish_attempt(initial["t1"]["grant"]["attempt"]),
            "t2": self.finish_attempt(initial["t2"]["grant"]["attempt"]),
        }
        initial_finish_seconds = round(time.time() - finish_started, 3)
        if initial_finish_snapshots["t2"]["occupied_slots"] != 0:
            raise RuntimeError("initial worker attempts did not release owned slots before integration")

        integration_started = time.time()
        fresh = self.run_attempt(
            "t2",
            lead_auth,
            {
                "ticket": "t2",
                "files": ticket_overlay(tickets_by_id["t2"]),
                "check": PARTIAL_CONTRACTS["t1"]["check"] + PARTIAL_CONTRACTS["t2"]["check"],
            },
            key_prefix="fresh-t2",
        )
        fresh_attempt_seconds = round(time.time() - integration_started, 3)
        if fresh["grant"]["base"] != first_landing["commit"]:
            raise RuntimeError("fresh integration attempt did not start from current canonical base")

        old_proof_refusal = self.command(
            "accept_candidate",
            {
                "candidate": fresh["candidate"]["candidate"],
                "verification": initial_proofs["t2"]["verification"]["verification"],
            },
            auth=lead_auth,
            key="reject-old-proof-for-fresh-candidate",
            allow_error=True,
        )
        if "error" not in old_proof_refusal:
            raise RuntimeError("old proof unexpectedly authorized the fresh integration candidate")

        fresh_comparison, fresh_verification, fresh_acceptance, fresh_timings = self.verify_accept(
            fresh["candidate"],
            lead_auth,
            tickets_by_id["t2"],
            key_prefix="fresh-t2",
        )
        if fresh_verification["document"]["suite_id"] != f"{self.task.id}:complete":
            raise RuntimeError(f"fresh integration used unexpected verifier suite: {fresh_verification['document']['suite_id']}")
        final_land_started = time.time()
        final_landing = self.command(
            "land_candidate",
            {"candidate": fresh["candidate"]["candidate"], "expected_trunk": first_landing["commit"]},
            auth=lead_auth,
            key="land-fresh-t2",
        )["result"]
        final_land_seconds = round(time.time() - final_land_started, 3)
        if final_landing["state"] != "landed":
            raise RuntimeError(f"fresh integration candidate did not land: {final_landing}")
        fresh_finish_started = time.time()
        final_snapshot = self.finish_attempt(fresh["grant"]["attempt"])
        fresh_finish_seconds = round(time.time() - fresh_finish_started, 3)

        store = self.root / "authority/objects/store.git"
        final_ref = fixed_git(
            self.root / "canonical",
            "rev-parse",
            "refs/heads/protected",
            timeout=self.execution_timeout(10.0, "final protected ref read"),
        ).stdout.strip()
        if self.fault == "final-deadline":
            self.execution_budget.deadline_monotonic = time.monotonic() - 0.001
        self.enforce_execution_deadline("final_independent_verifier")
        try:
            final_external = independent_verify_commit(
                self.task,
                self.root / "canonical",
                final_ref,
                complete_suite,
                policy["verifier_runtime"],
                policy["tools"],
                self.root,
                "final",
                budget=self.execution_budget,
                timeout_override=0.001 if self.fault == "final-inflight-timeout" else None,
            )
        except subprocess.TimeoutExpired as exc:
            raise ControllerFailure(
                "timeout",
                f"independent final verifier timed out in flight: {exc}",
                {
                    **self.attempt_count_details(),
                    "command": "final_independent_verifier",
                    "exception_type": "TimeoutExpired",
                    "timeout_seconds": exc.timeout,
                    "boundary": " ".join(str(part) for part in exc.cmd) if isinstance(exc.cmd, (list, tuple)) else str(exc.cmd),
                    "dispatch_cutoff_unix": self.dispatch_cutoff_unix,
                    "observed_unix": time.time(),
                },
            ) from exc
        t1_only = independent_verify_commit(
            self.task,
            store,
            initial["t1"]["candidate"]["commit"],
            complete_suite,
            policy["verifier_runtime"],
            policy["tools"],
            self.root,
            "t1-only",
            budget=self.execution_budget,
        )
        t2_only = independent_verify_commit(
            self.task,
            store,
            initial["t2"]["candidate"]["commit"],
            complete_suite,
            policy["verifier_runtime"],
            policy["tools"],
            self.root,
            "t2-only",
            budget=self.execution_budget,
        )
        tampered = independent_verify_commit(
            self.task,
            store,
            initial["t1"]["candidate"]["commit"],
            complete_suite,
            policy["verifier_runtime"],
            policy["tools"],
            self.root,
            "tampered-t1-only",
            tamper_tests=True,
            budget=self.execution_budget,
        )
        if final_external["verdict"] != "pass" or t1_only["verdict"] != "fail" or t2_only["verdict"] != "fail" or tampered["verdict"] != "fail":
            raise RuntimeError(
                "independent final verifier controls did not match expectations: "
                + json.dumps({"final": final_external, "t1_only": t1_only, "t2_only": t2_only, "tampered": tampered}, sort_keys=True)
            )

        inventory = self.query("workspace_inventory")
        events = self.query("events", {"after": 0, "limit": 200})
        flow_seconds = round(time.time() - flow_started, 3)
        cleanup_started = time.time()
        process_records = stopped_services(self.processes)
        cleanup_seconds = round(time.time() - cleanup_started, 3)
        attempt_records = [
            self.attempt_process_record(initial["t1"]["grant"]),
            self.attempt_process_record(initial["t2"]["grant"]),
            self.attempt_process_record(fresh["grant"]),
        ]
        useful_seconds = {
            ticket_id: round(
                (initial[ticket_id]["ready"]["useful_interval"]["end_monotonic_ns"]
                 - initial[ticket_id]["ready"]["useful_interval"]["start_monotonic_ns"]) / 1_000_000_000,
                6,
            )
            for ticket_id in ["t1", "t2"]
        }
        useful_seconds["fresh_t2"] = round(
            (fresh["ready"]["useful_interval"]["end_monotonic_ns"]
             - fresh["ready"]["useful_interval"]["start_monotonic_ns"]) / 1_000_000_000,
            6,
        )
        controller_verify_seconds = round(
            sum(authority_timings[t]["verify_seconds"] for t in ["t1", "t2"]) + fresh_timings["verify_seconds"],
            3,
        )
        independent_verify_seconds = round(
            sum(item["seconds"] for item in [final_external, t1_only, t2_only, tampered]),
            3,
        )
        timing = {
            "wall_seconds": round(time.time() - started, 3),
            "setup_seconds": setup_seconds,
            "controller_flow_seconds": flow_seconds,
            "attempt_seconds": round(sum(useful_seconds.values()), 3),
            "initial_attempt_wall_seconds": initial_attempt_wall_seconds,
            "useful_attempt_seconds": useful_seconds,
            "verify_seconds": round(controller_verify_seconds + independent_verify_seconds, 3),
            "controller_verify_seconds": controller_verify_seconds,
            "independent_verify_seconds": independent_verify_seconds,
            "accept_seconds": round(sum(authority_timings[t]["accept_seconds"] for t in ["t1", "t2"]) + fresh_timings["accept_seconds"], 3),
            "land_seconds": round(first_land_seconds + stale_land_seconds + final_land_seconds, 3),
            "initial_finish_seconds": initial_finish_seconds,
            "fresh_attempt_wall_seconds": fresh_attempt_seconds,
            "fresh_finish_seconds": fresh_finish_seconds,
            "cleanup_seconds": cleanup_seconds,
        }
        complete_contract = {
            "suite_id": fresh_verification["document"]["suite_id"],
            "suite_digest": fresh_verification["document"]["suite"],
            "selected_tests": complete_suite["selected_tests"],
            "protected_test_sha256": complete_suite["protected_test_sha256"],
            "complete_original_test_file_restored": True,
            "controller_verdict": fresh_verification["verdict"],
            "final_verdict": final_external["verdict"],
            "final_external": final_external,
            "omitted_controls": {"t1_only": t1_only["verdict"], "t2_only": t2_only["verdict"]},
            "omitted_control_evidence": {"t1_only": t1_only, "t2_only": t2_only},
            "tampered_tests_control": tampered["verdict"],
            "tampered_tests_evidence": tampered,
        }
        partial_contracts = {
            ticket_id: {
                "suite_id": initial_proofs[ticket_id]["verification"]["document"]["suite_id"],
                "suite_digest": initial_proofs[ticket_id]["verification"]["document"]["suite"],
                "selected_tests": PARTIAL_CONTRACTS[ticket_id]["selected_tests"],
                "protected_test_sha256": partial_suite["protected_test_sha256"],
                "fixed_verdict": initial_proofs[ticket_id]["verification"]["verdict"],
                "candidate": initial[ticket_id]["candidate"],
                "acceptance": initial_proofs[ticket_id]["acceptance"],
            }
            for ticket_id in ["t1", "t2"]
        }
        process_disposition = observed_process_disposition(process_records, attempt_records)
        orchestration_passed = (
            process_disposition["effect_disposition"] == "resolved"
            and all(partial_contracts[t]["fixed_verdict"] == "pass" for t in ["t1", "t2"])
            and first_landing["state"] == "landed"
            and stale_landing["state"] == "stale_base"
            and "error" in old_proof_refusal
            and fresh_verification["verdict"] == "pass"
            and fresh_acceptance["candidate"] == fresh["candidate"]["candidate"]
            and final_landing["state"] == "landed"
            and final_ref == fresh["candidate"]["commit"]
            and complete_contract["final_verdict"] == "pass"
            and complete_contract["omitted_controls"] == {"t1_only": "fail", "t2_only": "fail"}
            and complete_contract["tampered_tests_control"] == "fail"
            and overlap["seconds"] > 0
            and any(a["id"] == fresh["grant"]["attempt"] and a["state"] == "exited" for a in final_snapshot["attempts"])
        )
        task_verdict = "pass" if complete_contract["final_verdict"] == "pass" else "fail"
        orchestration_verdict = "pass" if orchestration_passed else "fail"
        retained = retained_evidence(
            self.root,
            repositories=[
                {"kind": "canonical", "path": str(self.root / "canonical"), "protected_ref": "refs/heads/protected", "head": final_ref},
                {"kind": "authority_store", "path": str(store)},
            ],
            manifests=[
                evidence_entry("source", self.root / "source-manifest.json"),
                evidence_entry("git_policy", self.root / "git.json"),
                evidence_entry("kata_config", self.root / "kata.json"),
                evidence_entry("supervisor_config", self.root / "supervisor.json"),
                {
                    "kind": "verifier_runtime",
                    "path": policy["verifier_runtime"]["manifest"]["path"],
                    "sha256": policy["verifier_runtime"]["manifest"]["sha256"],
                },
            ],
            events=[{"kind": "controller", "first_seq": 1, "last_seq": events[-1]["seq"] if events else 0, "count": len(events)}],
            proofs=[
                {"kind": "initial_t1", "verification": initial_proofs["t1"]["verification"]["verification"],
                 "report": initial_proofs["t1"]["verification"]["report"], "verdict": initial_proofs["t1"]["verification"]["verdict"]},
                {"kind": "initial_t2", "verification": initial_proofs["t2"]["verification"]["verification"],
                 "report": initial_proofs["t2"]["verification"]["report"], "verdict": initial_proofs["t2"]["verification"]["verdict"]},
                {"kind": "fresh_integration", "verification": fresh_verification["verification"],
                 "report": fresh_verification["report"], "verdict": fresh_verification["verdict"]},
                {"kind": "final_external", "path": final_external["log"]["path"], "sha256": final_external["log"]["sha256"],
                 "verdict": final_external["verdict"]},
            ],
            logs=[evidence_entry(entry["name"], Path(entry["log"])) for entry in process_records],
        )
        metadata = controller_result_metadata(
            root=self.root,
            admitted_at=control["execution_clock"]["durable_run_admission_unix"],
            first_submission_at=control["execution_clock"]["first_mission_submission_unix"],
            task_verdict=task_verdict,
            orchestration_verdict=orchestration_verdict,
            effect_disposition=process_disposition["effect_disposition"],
            attempts_total=3,
            initial_attempts=2,
            fresh_integration_attempts=1,
            interventions=[],
            retained=retained,
        )
        return {
            "schema_version": CONTROLLER_SCHEMA,
            "mode": "deterministic",
            "task": self.task.id,
            "ticket": "complete",
            **metadata,
            "run_root": str(self.root),
            "source": self.source,
            "build": self.build,
            "control": control,
            "roles": {
                "lead": control["lead"],
                "initial_attempts": {ticket_id: public_grant(initial[ticket_id]["grant"]) for ticket_id in ["t1", "t2"]},
                "fresh_integration_attempt": public_grant(fresh["grant"]),
            },
            "kata": {"binary": kata["binary"], "instance_uid": kata["instance_uid"], "project": kata["project"],
                     "issues": issues, "config_sha256": policy["kata_config_sha256"]},
            "git": {"policy_sha256": policy["git_policy_sha256"], "canonical": str(self.root / "canonical"),
                    "protected_ref": "refs/heads/protected", "base": base, "after_first_landing": first_landing["commit"], "final": final_ref},
            "supervisor": {"config_sha256": policy["supervisor_config_sha256"]},
            "runtime_closure": {
                "status": "pinned",
                "pinned": {
                    "partial_verification_adapter": {"path": partial_suite["path"], "sha256": partial_suite["sha256"]},
                    "complete_verification_adapter": {"path": complete_suite["path"], "sha256": complete_suite["sha256"]},
                    "protected_test_sha256": complete_suite["protected_test_sha256"],
                    "git": policy["tools"]["git"],
                    "python_executable": policy["tools"]["python"],
                    "bubblewrap": policy["tools"]["bubblewrap"],
                    "verifier_runtime": policy["verifier_runtime"],
                },
                "enforced": "sqlite-controller validates the pinned runtime manifest, ro-binds that private snapshot at /usr, and binds the runtime digest into verifier proof",
            },
            "partial_contract": {
                "suite_id": "multiple",
                "suite_digest": "multiple",
                "selected_tests": complete_suite["selected_tests"],
                "noop_verdict": "not_run",
                "fixed_verdict": "pass" if all(partial_contracts[t]["fixed_verdict"] == "pass" for t in ["t1", "t2"]) else "fail",
            },
            "partial_contracts": partial_contracts,
            "complete_contract": complete_contract,
            "initial_candidates": {ticket_id: initial[ticket_id]["candidate"] for ticket_id in ["t1", "t2"]},
            "comparison": {
                "changed_paths": sorted(set(initial_proofs["t1"]["comparison"]["changed_paths"] + fresh["ready"]["changed"])),
                "contamination": [],
            },
            "landing_sequence": [
                {"ticket": "t1", **first_landing},
                {"ticket": "t2", **final_landing},
            ],
            "landing": final_landing,
            "stale_landing": stale_landing,
            "fresh_integration": {
                "attempt_count": 1,
                "base": fresh["grant"]["base"],
                "candidate": fresh["candidate"],
                "comparison": {"changed_paths": extract_diff_paths(fresh_comparison["diff"]), "contamination": []},
                "old_proof_for_fresh_candidate": "rejected",
                "old_proof_for_fresh_candidate_error": old_proof_refusal["error"],
                "verification": fresh_verification,
                "acceptance": fresh_acceptance,
                "landing": final_landing,
            },
            "useful_execution_overlap": overlap,
            "capacity": {
                "initial_occupied_slots": two_worker_snapshot["occupied_slots"],
                "after_initial_release_occupied_slots": initial_finish_snapshots["t2"]["occupied_slots"],
                "max_concurrent_worker_executions": 2,
            },
            "process_disposition": process_disposition,
            "events_observed": {"count": len(events), "last_seq": events[-1]["seq"] if events else 0},
            "workspace_inventory": {
                "configured": inventory.get("configured"),
                "backend": inventory.get("backend"),
                "workspaces": len(inventory.get("workspaces", [])),
                "history": len(inventory.get("history", [])),
            },
            "timing": timing,
            "qualification_limits": [
                "deterministic m01 complete path only",
                "no live model calls",
                "no dependent-task, textual-conflict or broad recovery qualification",
                "no permanent readiness claim",
            ],
            "orchestration_passed": orchestration_passed,
        }


def controller_ticket(task: tasklib.Task, ticket_id: str) -> fleet.Ticket:
    if task.id != "m01-disjoint-utils":
        raise SystemExit("controller mode currently supports only m01-disjoint-utils")
    tickets = {ticket.id: ticket for ticket in fleet.load_tickets(task)}
    if ticket_id not in tickets or ticket_id not in PARTIAL_CONTRACTS:
        raise SystemExit(f"controller mode unsupported ticket {ticket_id!r}")
    ticket = tickets[ticket_id]
    if ticket.depends:
        raise SystemExit("controller mode rejects dependency graphs before execution")
    if set(ticket.allow) != set(PARTIAL_CONTRACTS[ticket_id]["allow"]):
        raise SystemExit("controller mode task allowlist does not match the pinned partial contract")
    return ticket


def controller_tickets(task: tasklib.Task, ticket_id: str) -> list[fleet.Ticket]:
    if ticket_id != "complete":
        return [controller_ticket(task, ticket_id)]
    if task.id != "m01-disjoint-utils":
        raise SystemExit("controller mode currently supports only m01-disjoint-utils")
    tickets = {ticket.id: ticket for ticket in fleet.load_tickets(task)}
    if set(tickets) != {"t1", "t2"}:
        raise SystemExit("controller complete mode requires m01 tickets t1 and t2")
    selected = [tickets["t1"], tickets["t2"]]
    for ticket in selected:
        if ticket.depends:
            raise SystemExit("controller mode rejects dependency graphs before execution")
        if set(ticket.allow) != set(PARTIAL_CONTRACTS[ticket.id]["allow"]):
            raise SystemExit("controller mode task allowlist does not match the pinned partial contract")
    return selected


def validate_controller_selection(tasks: list[tasklib.Task], ticket_id: str) -> None:
    for task in tasks:
        controller_tickets(task, ticket_id)


def _fixture_classification(stage: str) -> dict:
    if stage in {"missing_prerequisite", "unsupported_task", "preflight"}:
        return {
            "task_verdict": "unknown",
            "effect_disposition": "resolved",
            "attempts": 0,
            "execution_seconds": 0.0,
            "model_usage": no_model_usage(),
        }
    if stage == "generic_failure":
        return {
            "task_verdict": "unknown",
            "effect_disposition": "unresolved",
            "attempts": 0,
            "execution_seconds": 0.0,
            "model_usage": no_model_usage("unknown"),
        }
    if stage == "failed_verifier":
        return {
            "task_verdict": "fail",
            "effect_disposition": "unresolved",
            "attempts": 1,
            "execution_seconds": 1.0,
            "model_usage": no_model_usage(),
        }
    if stage == "manual_intervention":
        return {
            "task_verdict": "unknown",
            "effect_disposition": "resolved",
            "attempts": 1,
            "execution_seconds": 1.0,
            "model_usage": no_model_usage("unknown"),
        }
    if stage in {"timeout", "provider_failure"}:
        return {
            "task_verdict": "unknown",
            "effect_disposition": "unresolved",
            "attempts": 1,
            "execution_seconds": 1.0,
            "model_usage": no_model_usage("unknown"),
        }
    if stage in {"delivery_failure", "missing_evidence"}:
        return {
            "task_verdict": "unknown",
            "effect_disposition": "unresolved" if stage == "delivery_failure" else "unknown",
            "attempts": 1,
            "execution_seconds": 1.0,
            "model_usage": no_model_usage("unknown"),
        }
    raise ValueError(f"unsupported controller failure fixture stage: {stage}")


def failure_fixture_record(task: tasklib.Task, agent, out: Path, index: int, ticket_id: str,
                           stage: str, message: str) -> dict:
    """Build a deterministic controller failure result without launching providers."""
    classification = _fixture_classification(stage)
    out = Path(out)
    rid = f"{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-controller{ticket_id}-{stage}-{task.id}-{index:03d}"
    root = out / rid / "retained"
    root.mkdir(mode=0o700, parents=True, exist_ok=True)
    source = source_identity()
    write_json(root / "source-manifest.json", source["content_manifest"])
    failure_manifest = {
        "schema_version": "corenius.bench.controller-failure-fixture.v1",
        "stage": stage,
        "message": message,
        "task": task.id,
        "ticket": ticket_id,
    }
    write_json(root / "failure-manifest.json", failure_manifest)
    failure_log = root / "failure.log"
    failure_log.write_text(f"{stage}: {message}\n")
    failure_log.chmod(0o600)
    events_path = root / "events.jsonl"
    events_path.write_text(json.dumps({"seq": 1, "kind": "failure", "stage": stage}, sort_keys=True) + "\n")
    events_path.chmod(0o600)
    proof_path = root / "proof.json"
    write_json(proof_path, {"stage": stage, "verdict": classification["task_verdict"], "trusted": False})
    retained = retained_evidence(
        root,
        repositories=[
            {"kind": "canonical", "path": str(root / "canonical.git"), "head": None},
        ],
        manifests=[
            evidence_entry("source", root / "source-manifest.json"),
            evidence_entry("failure", root / "failure-manifest.json"),
        ],
        events=[{"kind": "controller", "path": str(events_path), "sha256": sha256_bytes(events_path.read_bytes()), "first_seq": 1, "last_seq": 1}],
        proofs=[{"kind": "failure", "path": str(proof_path), "sha256": sha256_bytes(proof_path.read_bytes()), "verdict": classification["task_verdict"]}],
        logs=[evidence_entry("failure", failure_log)],
    )
    admitted_at = time.time()
    first_submission_at = admitted_at if classification["execution_seconds"] == 0.0 else admitted_at + 0.001
    interventions = (
        [{"kind": "manual_rescue", "stage": stage, "summary": "fixture manual intervention prevents autonomous PASS"}]
        if stage == "manual_intervention" else []
    )
    metadata = controller_result_metadata(
        root=root,
        admitted_at=admitted_at,
        first_submission_at=first_submission_at,
        task_verdict=classification["task_verdict"],
        orchestration_verdict="fail",
        effect_disposition=classification["effect_disposition"],
        attempts_total=classification["attempts"],
        initial_attempts=min(classification["attempts"], 2),
        fresh_integration_attempts=0,
        interventions=interventions,
        retained=retained,
        failure_stage=stage,
        failure_kind=stage,
        failure_message=message,
        model_usage=classification["model_usage"],
    )
    verify_record = tasklib.capsule_for(task)
    controller = {
        "schema_version": CONTROLLER_SCHEMA,
        "mode": "deterministic",
        "task": task.id,
        "ticket": ticket_id,
        **metadata,
        "run_root": str(root),
        "source": source,
        "build": {"seconds": 0.0, "exit": 0, "log": str(failure_log)},
        "partial_contract": {
            "suite_id": "not_run",
            "suite_digest": "not_run",
            "selected_tests": [],
            "noop_verdict": "not_run",
            "fixed_verdict": classification["task_verdict"],
        },
        "landing": {"state": "not_landed"},
        "process_disposition": {
            "owned_attempts": "unresolved" if classification["effect_disposition"] == "unresolved" else "stopped",
            "services": "stopped",
            "effect_disposition": classification["effect_disposition"],
            "remaining_uncertain": (
                [{"kind": "attempt", "id": f"fixture-{stage}", "reason": "physical stop not inferred from failure"}]
                if classification["effect_disposition"] == "unresolved" else []
            ),
        },
        "runtime_closure": {
            "status": "pinned",
            "pinned": {"verifier_runtime": {"manifest": {"sha256": sha256_bytes((root / "failure-manifest.json").read_bytes())}}},
        },
        "qualification_limits": [
            "controlled deterministic failure fixture only",
            "no live model calls",
            "no one-run speed, cost or recovery superiority claim",
        ],
        "orchestration_passed": False,
    }
    execution_seconds = classification["execution_seconds"]
    return {
        "schema_version": "corenius.bench.result.v1",
        "run_id": rid,
        "task_id": task.id,
        "condition": f"controller:{ticket_id}",
        "agent": {"spec": agent.spec},
        "capsule": {
            "digest": verify_record["capsule_digest"],
            "net": verify_record["policy"]["net"],
            "hermetic": verify_record["hermetic"],
            "non_hermetic_layers": verify_record["non_hermetic_layers"],
        },
        "outcome": {
            "passed": False,
            "reward": 0.0,
            "verifier_rc": 1,
            "timed_out": stage == "timeout",
            "agent_rc": None,
            "task_verdict": classification["task_verdict"],
            "orchestration_verdict": "fail",
            "effect_disposition": classification["effect_disposition"],
            "failure_stage": stage,
        },
        "timing": {
            "wall_seconds": round(0.5 + execution_seconds, 3),
            "agent_seconds": 0.0,
            "verify_seconds": 0.0 if stage != "failed_verifier" else 0.5,
            "source_capture_seconds": 0.0,
            "build_seconds": 0.0,
            "setup_seconds": 0.5,
            "execution_seconds": execution_seconds,
            "controller_flow_seconds": execution_seconds,
            "cleanup_seconds": 0.5,
        },
        "coordination": {
            "conflicts": 0,
            "stale_refusals": 0,
            "redos": 0,
            "contamination": 0,
            "contamination_paths": [],
            "tickets": 2 if ticket_id == "complete" else 1,
            "tickets_landed": 0,
            "condition_mode": "controller",
            "attempts": classification["attempts"],
            "manual_interventions": len(interventions),
        },
        "cost": {
            "usd": classification["model_usage"]["usd"],
            "input_tokens": classification["model_usage"]["input_tokens"],
            "output_tokens": classification["model_usage"]["output_tokens"],
        },
        "provenance": {
            "outcome": "controlled deterministic controller failure fixture",
            "timing": "measured by fixture phase timers",
            "coordination": "controller fixture measured",
            "cost": f"{classification['model_usage']['status']}; no synthesized usage total",
        },
        "controller": controller,
    }


def collect_failure_evidence(run: ControllerRun | None, budget=None) -> dict:
    if run is None:
        return {"events_error": "controller services were not started"}
    evidence: dict = {}
    try:
        if isinstance(budget, CleanupBudget) and budget.remaining() <= 0:
            evidence["events_error"] = "cleanup evidence budget exhausted before controller event export"
        else:
            evidence["events"] = run.query(
                "events",
                {"after": 0, "limit": 200},
                timeout=3.0,
                bypass_execution_deadline=True,
                budget=budget,
            )
    except Exception as exc:
        evidence["events_error"] = f"{type(exc).__name__}: {exc}"
    try:
        canonical = run.root / "canonical"
        if isinstance(budget, CleanupBudget) and budget.remaining() <= 0:
            evidence.setdefault("canonical_head_error", "cleanup evidence budget exhausted before canonical head export")
        elif canonical.exists():
            evidence["canonical_head"] = fixed_git(
                canonical,
                "rev-parse",
                "refs/heads/protected",
                check=False,
                timeout=budget_timeout(budget, 3.0, "failure evidence canonical head"),
            ).stdout.strip() or None
    except Exception as exc:
        evidence["canonical_head_error"] = f"{type(exc).__name__}: {exc}"
    return evidence


def run_controller_condition(task: tasklib.Task, agent, out: Path, index: int, ticket_id: str, rid: str | None = None,
                             observer=None, fault: str | None = None) -> dict:
    if agent.kind != "reference":
        raise SystemExit("controller mode currently supports the deterministic reference agent only")
    if fault is not None and fault not in LIFECYCLE_FAULTS:
        raise SystemExit(f"unsupported controller lifecycle fault {fault!r}")
    observer = observer or (lambda event, **fields: None)
    tickets = controller_tickets(task, ticket_id)
    ticket = tickets[0]
    complete = ticket_id == "complete"
    verify_record = tasklib.capsule_for(task)
    rid = rid or f"{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-controller{ticket_id}-{task.id}-reference-{index:03d}"
    total_started = time.time()
    root = Path(tempfile.mkdtemp(prefix=f"cb-{ticket_id}-", dir=str(run_root_parent())))
    setup_budget = StageBudget(seconds=SETUP_STAGE_TIMEOUT_SECONDS, failure_stage="preflight", label="controller setup")
    source = unknown_source_identity()
    source_capture_seconds = 0.0
    build = {"seconds": 0.0, "exit": None, "log": str(root / "controller-build.log")}
    run = None

    def failure_result(exc: ControllerFailure, process_records: list[dict], cleanup_seconds: float) -> dict:
        failure_evidence = (
            getattr(run, "failure_evidence_before_service_stop", None)
            if run is not None else None
        )
        if failure_evidence is None:
            failure_evidence = collect_failure_evidence(run)
        rec = controller_failure_record(
            task=task,
            agent=agent,
            rid=rid,
            ticket_id=ticket_id,
            root=root,
            source=source,
            build=build,
            failure=exc,
            control=run.control if run is not None else None,
            process_records=process_records,
            setup_seconds=run.setup_seconds if run is not None else round(time.time() - total_started, 3),
            execution_seconds=run.flow_seconds_so_far() if run is not None else 0.0,
            cleanup_seconds=cleanup_seconds,
            total_wall_seconds=round(time.time() - total_started, 3),
            source_capture_seconds=source_capture_seconds,
            failure_evidence=failure_evidence,
        )
        write_json(root / "failure-result.json", rec)
        return rec

    try:
        source_started = time.time()
        source = source_identity(budget=setup_budget)
        source_capture_seconds = round(time.time() - source_started, 3)
        write_json(root / "source-manifest.json", source["content_manifest"])
        observer("controller-setup")
        build = build_controller(root, budget=setup_budget)
        run = ControllerRun(task, ticket, root, build, source, fault=fault, setup_budget=setup_budget)
        controller = run.run_complete(tickets) if complete else run.run()
    except ControllerFailure as exc:
        cleanup_started = time.time()
        process_records = run.reconcile_owned_processes() if run is not None else []
        cleanup_seconds = round(time.time() - cleanup_started, 3)
        rec = failure_result(exc, process_records, cleanup_seconds)
        observer("controller-done", passed=False, failure_stage=exc.stage)
        return rec
    except BaseException as exc:
        cleanup_started = time.time()
        process_records = run.reconcile_owned_processes() if run is not None else []
        cleanup_seconds = round(time.time() - cleanup_started, 3)
        failure = timeout_failure_from_exception(exc, run)
        if failure is None:
            failure = ControllerFailure(
                "generic_failure",
                f"{type(exc).__name__}: {exc}",
                {
                    "exception_type": type(exc).__name__,
                    "exception_message": str(exc),
                    "boundary": "controller-run",
                    **(run.attempt_count_details() if run is not None else {"attempts_total": 0, "initial_attempts": 0, "fresh_integration_attempts": 0}),
                },
            )
        rec = failure_result(failure, process_records, cleanup_seconds)
        observer("controller-done", passed=False, failure_stage=failure.stage)
        return rec
    observer("controller-done", passed=controller["orchestration_passed"])
    total_wall_seconds = round(time.time() - total_started, 3)
    task_verdict = controller["outcomes"]["task"]["verdict"]
    orchestration_verdict = controller["outcomes"]["orchestration"]["verdict"]
    effect_disposition = controller["outcomes"]["effects"]["disposition"]
    task_passed = task_verdict in ("pass", "partial_pass")
    rec = {
        "schema_version": "corenius.bench.result.v1",
        "run_id": rid,
        "task_id": task.id,
        "condition": f"controller:{ticket_id}",
        "agent": {"spec": agent.spec},
        "capsule": {
            "digest": verify_record["capsule_digest"],
            "net": verify_record["policy"]["net"],
            "hermetic": verify_record["hermetic"],
            "non_hermetic_layers": verify_record["non_hermetic_layers"],
        },
        "outcome": {
            "passed": bool(controller["orchestration_passed"]),
            "reward": 1.0 if controller["orchestration_passed"] else 0.0,
            "verifier_rc": 0 if task_passed else 1,
            "timed_out": False,
            "agent_rc": 0,
            "task_verdict": task_verdict,
            "orchestration_verdict": orchestration_verdict,
            "effect_disposition": effect_disposition,
            "failure_stage": controller["failure"]["stage"],
        },
        "timing": {
            "wall_seconds": total_wall_seconds,
            "agent_seconds": controller["timing"]["attempt_seconds"],
            "verify_seconds": controller["timing"]["verify_seconds"],
            "source_capture_seconds": source_capture_seconds,
            "build_seconds": controller["build"]["seconds"],
            "setup_seconds": round(source_capture_seconds + controller["build"]["seconds"] + controller["timing"]["setup_seconds"], 3),
            "execution_seconds": controller["timing"]["controller_flow_seconds"],
            "controller_flow_seconds": controller["timing"]["controller_flow_seconds"],
            "cleanup_seconds": controller["timing"]["cleanup_seconds"],
        },
        "coordination": {
            "conflicts": 0,
            "redos": 1 if complete else 0,
            "stale_refusals": 1 if complete and controller["stale_landing"]["state"] == "stale_base" else 0,
            "contamination": 0,
            "contamination_paths": [],
            "tickets": 2 if complete else 1,
            "tickets_landed": 2 if complete and controller["fresh_integration"]["landing"]["state"] == "landed" else (
                1 if not complete and controller["landing"]["state"] == "landed" else 0
            ),
            "condition_mode": "controller",
            "attempts": 3 if complete else 2,
            "manual_interventions": 0,
        },
        "cost": {
            "usd": controller["model_usage"]["usd"],
            "input_tokens": controller["model_usage"]["input_tokens"],
            "output_tokens": controller["model_usage"]["output_tokens"],
        },
        "provenance": {
            "outcome": "controller measured",
            "timing": "measured by harness phase timers",
            "coordination": "controller measured",
            "cost": f"{controller['model_usage']['status']}; deterministic no-model mode",
        },
        "controller": controller,
    }
    return rec
