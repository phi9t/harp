#!/usr/bin/env python3
"""Lead-driven readiness scenarios with scripted participants and real git state."""

from __future__ import annotations

if __name__ == "__main__":
    import sys as _retired_sys
    print('Retired imported benchmark execution. Use harp durable --help; only historical solo reference/noop replay remains supported.', file=_retired_sys.stderr)
    raise SystemExit(2)

import argparse
import hashlib
import json
import os
import re
import shutil
import shlex
import signal
import stat
import subprocess
import sys
import time
import tomllib
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

import results
import tasklib

ROOT = Path(__file__).resolve().parents[2]
READINESS = ROOT / "bench" / "readiness"
WORKER = ROOT / "tools" / "agents" / "worker.py"
LEAD_LAUNCH = ROOT / "tools" / "agents" / "lead_launch.py"
TRACKER = ROOT / "tools" / "agents" / "tracker.py"
sys.path.insert(0, str(ROOT / "tools" / "agents"))
import worker as worker_tool  # noqa: E402

RUNNING_NOTES = ROOT / "docs" / "efforts" / "lead-worker-readiness" / "evidence" / "lead" / "notes.md"
LAND = ROOT / "tools" / "agents" / "land.py"
ACCEPTANCE = ROOT / "tools" / "agents" / "acceptance.py"
EFFORT = ROOT / "tools" / "agents" / "effort.py"
GIT_ENV = ["-c", "user.email=bench@corenius", "-c", "user.name=bench", "-c", "commit.gpgsign=false"]
SCENARIO_RESULTS = {"passes", "passes with intervention", "not applicable by design", "fails"}
NOTE_KINDS = {"obstacle", "intervention", "rule"}
STALE_CANDIDATE = 6
MAINLINE_REFUSED = 8
RECORD_BRANCH_REQUIRED = 11
NETWORK_COMMAND = ("curl ", "curl\t", "wget ", "wget\t")
CONTAINER_COMMANDS = ("docker build", "docker buildx", "podman build", "buildah build", "nerdctl build")
SHARED_CACHE_COMMANDS = ("pip install", "pip3 install", "uv pip install")
EXTERNAL_DIGEST_LIMIT_BYTES = 1024 * 1024
ACCEPTANCE_FAILED = 7
STALE_REFUSED = 6


class LeadDrivenError(Exception):
    pass


TOOLS = {
    "worker": ("CORENIUS_READINESS_WORKER", WORKER),
    "lead_launch": ("CORENIUS_READINESS_LEAD_LAUNCH", LEAD_LAUNCH),
    "tracker": ("CORENIUS_READINESS_TRACKER", TRACKER),
    "land": ("CORENIUS_READINESS_LAND", LAND),
    "acceptance": ("CORENIUS_READINESS_ACCEPTANCE", ACCEPTANCE),
    "effort": ("CORENIUS_READINESS_EFFORT", EFFORT),
}


def tool_path(name: str, env: dict | None = None) -> Path:
    variable, default = TOOLS[name]
    source = env if env is not None else os.environ
    return Path(source.get(variable, str(default)))


def tool_cmd(name: str, *args: str, env: dict | None = None) -> list[str]:
    path = tool_path(name, env)
    command = [str(path), *args]
    if path.suffix == ".py":
        command.insert(0, sys.executable)
    return command


@dataclass
class Participant:
    id: str
    mode: str
    expect_ready: bool
    stories: list[int] | None = None
    handoff_mode: str = "exit"
    handoff_marker: str | None = None
    expect_candidate_moved: bool = False
    default: bool = True
    network: str = "on"
    command_timeout_seconds: int | None = None
    external_add_paths: list[str] | None = None
    external_protect_paths: list[str] | None = None
    allow_container_builds: bool = False
    allow_shared_cache_writes: bool = False


@dataclass
class AcceptanceCase:
    effort: str
    ticket: str
    mode: str


@dataclass
class Scenario:
    id: str
    path: Path
    task: Path
    environment: Path
    benchmark_task: str | None
    stories: list[int] | None
    allow: list[str]
    participants: list[Participant]
    kind: str = "handoff"
    landing_expect: str | None = None
    effort: str | None = None
    integration_branch: str | None = None
    parallel_cap: int = 3
    cases: list[dict] | None = None
    acceptance: AcceptanceCase | None = None
    landing_mainline: str | None = None
    landing_repo: str = "target"
    landing_target_worktree: bool = False


def git_env(env: dict[str, str] | None = None) -> dict[str, str]:
    cleaned = dict(os.environ if env is None else env)
    for name in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR"):
        cleaned.pop(name, None)
    cleaned["GIT_OPTIONAL_LOCKS"] = "0"
    return cleaned


def git(repo: Path, *args: str, check: bool = True) -> str:
    proc = subprocess.run(["git", "-C", str(repo), *GIT_ENV, *args], capture_output=True, text=True, env=git_env())
    if check and proc.returncode != 0:
        raise LeadDrivenError(f"git {' '.join(args)} failed in {repo}: {proc.stderr.strip() or proc.stdout.strip()}")
    return proc.stdout


def is_ancestor(repo: Path, older: str, newer: str) -> bool:
    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", older, newer],
        capture_output=True,
        env=git_env(),
    ).returncode == 0


def load_scenario(name: str) -> Scenario:
    path = READINESS / name
    cfg_path = path / "scenario.toml"
    if not cfg_path.is_file():
        raise LeadDrivenError(f"unknown readiness scenario {name!r}")
    cfg = tomllib.loads(cfg_path.read_text())
    worker_cfg = cfg.get("worker", {})
    participants = [
        Participant(
            id=p["id"],
            mode=p["mode"],
            expect_ready=bool(p["expect_ready"]),
            stories=list(p["stories"]) if "stories" in p else None,
            handoff_mode=p.get("handoff_mode", worker_cfg.get("handoff_mode", "exit")),
            handoff_marker=p.get("handoff_marker", worker_cfg.get("handoff_marker")),
            expect_candidate_moved=bool(p.get("expect_candidate_moved", False)),
            default=bool(p.get("default", True)),
            network=p.get("network", worker_cfg.get("network", "on")),
            command_timeout_seconds=p.get("command_timeout_seconds", worker_cfg.get("command_timeout_seconds")),
            external_add_paths=list(p.get("external_add_paths", worker_cfg.get("external_add_paths", []))),
            external_protect_paths=list(p.get("external_protect_paths", worker_cfg.get("external_protect_paths", []))),
            allow_container_builds=bool(p.get("allow_container_builds", worker_cfg.get("allow_container_builds", False))),
            allow_shared_cache_writes=bool(p.get("allow_shared_cache_writes", worker_cfg.get("allow_shared_cache_writes", False))),
        )
        for p in cfg.get("participants", [])
    ]
    landing = cfg.get("landing", {})
    effort = cfg.get("effort", {})
    acceptance_cfg = cfg.get("acceptance")
    acceptance_case = None
    if acceptance_cfg:
        acceptance_case = AcceptanceCase(
            effort=acceptance_cfg["effort"],
            ticket=acceptance_cfg["ticket"],
            mode=acceptance_cfg["mode"],
        )
    return Scenario(
        id=cfg["id"],
        path=path,
        task=path / cfg["task"],
        environment=path / cfg.get("environment", "environment"),
        benchmark_task=cfg.get("benchmark_task"),
        stories=list(cfg["stories"]) if "stories" in cfg else None,
        allow=list(worker_cfg.get("allow", [])),
        participants=participants,
        kind=cfg.get("kind", "handoff"),
        landing_expect=landing.get("expect"),
        effort=effort.get("name"),
        integration_branch=effort.get("integration_branch"),
        parallel_cap=int(effort.get("parallel_cap", 3)),
        cases=list(cfg.get("cases", [])),
        acceptance=acceptance_case,
        landing_mainline=landing.get("mainline"),
        landing_repo=landing.get("repo", "target"),
        landing_target_worktree=bool(landing.get("target_worktree", False)),
    )


def load_effort_metadata(repo: Path, effort: str) -> dict[str, str | None]:
    path = repo / "docs" / "efforts" / effort / "EFFORT.md"
    if not path.exists():
        return {"base": None, "integration_branch": None}
    text = path.read_text()
    out = {"base": None, "integration_branch": None}
    for key, field in (("Base", "base"), ("Integration branch", "integration_branch")):
        match = re.search(rf"^{re.escape(key)}:\s*(\S.*)?$", text, re.M)
        if match:
            value = (match.group(1) or "").strip()
            out[field] = value or None
    return out


def seed_acceptance_ticket(repo: Path, case: AcceptanceCase) -> None:
    effort = repo / "docs" / "efforts" / case.effort
    ticket = effort / "tickets" / f"{case.ticket}-acceptance.md"
    ticket.parent.mkdir(parents=True, exist_ok=True)
    (effort / "evidence").mkdir(parents=True, exist_ok=True)
    (effort / "EFFORT.md").write_text(
        f"# Effort: {case.effort}\n\n"
        "Status: active\n"
        "Base: main\n"
        f"Integration branch: effort/{case.effort}\n"
        "Lead: lead\n"
    )
    ticket.write_text(
        f"# {case.ticket} acceptance: Acceptance fixture\n\n"
        "Type: feature\n"
        "Status: claimed\n"
        "Owner: worker-01\n"
        "Blocked by:\n"
        "Claimed: 2026-10-05T10:00:00Z expires 2026-10-06T10:00:00Z\n\n"
        "## Acceptance\n"
        "- [ ] works\n\n"
        "## Comments\n"
    )


def init_repo(scenario: Scenario, repo: Path) -> str:
    if scenario.benchmark_task:
        task = tasklib.load_tasks(only=[scenario.benchmark_task])[0]
        tasklib.materialize(task, repo)
    else:
        shutil.copytree(scenario.environment, repo, symlinks=True)
    if scenario.acceptance:
        seed_acceptance_ticket(repo, scenario.acceptance)
    git(repo, "init", "-q", "-b", "main")
    git(repo, "config", "--local", "collab.overrideOwners", "owner")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "start")
    git(repo, "branch", "side")
    if scenario.acceptance:
        git(repo, "branch", f"effort/{scenario.acceptance.effort}", "main")
    return git(repo, "rev-parse", "HEAD").strip()


def materialize_external_path(scenario: Scenario, case_dir: Path, relative: str) -> Path:
    raw = Path(relative)
    target = raw if raw.is_absolute() else case_dir / raw
    source = scenario.path / relative
    if target.exists() or target.is_symlink():
        return target
    if source.is_symlink():
        target.parent.mkdir(parents=True, exist_ok=True)
        target.symlink_to(os.readlink(source))
    elif source.is_dir():
        shutil.copytree(source, target, symlinks=True)
    elif source.is_file():
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target, follow_symlinks=False)
    else:
        target.mkdir(parents=True)
    return target


def command_events(run_dir: Path) -> list[str]:
    events = run_dir / "events.jsonl"
    if not events.exists():
        return []
    runtime = json.loads((run_dir / "run.json").read_text()).get("runtime", "traecli")
    parsed = []
    for line in events.read_text(errors="replace").splitlines():
        try:
            parsed.append(json.loads(line))
        except ValueError:
            continue
    return worker_tool.event_stream_commands(parsed, runtime)


def command_violations(run: dict, commands: list[str]) -> list[str]:
    profile = run.get("access_profile") or {}
    network = profile.get("network", "on")
    container_builds = bool(profile.get("container_builds", False))
    shared_cache_writes = bool(profile.get("shared_cache_writes", False))
    violations = []
    for command in commands:
        padded = command + " "
        if network == "off" and any(token in padded for token in NETWORK_COMMAND):
            violations.append("network-command")
        if not container_builds and any(token in command for token in CONTAINER_COMMANDS):
            violations.append("container-build")
        if not shared_cache_writes and any(token in command for token in SHARED_CACHE_COMMANDS):
            violations.append("shared-cache-write")
    return sorted(set(violations))

def path_inside(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def digest_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(1024 * 1024)
            if not chunk:
                break
            hasher.update(chunk)
    return hasher.hexdigest()


def external_entry(root: Path, path: Path) -> dict:
    st = os.lstat(path)
    mode = st.st_mode
    if stat.S_ISLNK(mode):
        kind = "symlink"
    elif stat.S_ISREG(mode):
        kind = "file"
    elif stat.S_ISDIR(mode):
        kind = "directory"
    else:
        kind = "other"
    entry = {
        "root": str(root),
        "path": str(path),
        "relative": "." if path == root else str(path.relative_to(root)),
        "kind": kind,
        "size": st.st_size,
        "mtime_ns": st.st_mtime_ns,
    }
    if kind == "symlink":
        entry["target"] = os.readlink(path)
    elif kind == "file" and st.st_size <= EXTERNAL_DIGEST_LIMIT_BYTES:
        entry["digest"] = digest_file(path)
    return entry


def external_snapshot(paths: list[Path]) -> list[dict]:
    entries = []
    for root in paths:
        if not os.path.lexists(root):
            continue
        stack = [root]
        while stack:
            path = stack.pop()
            try:
                entry = external_entry(root, path)
            except FileNotFoundError:
                continue
            entries.append(entry)
            if entry["kind"] == "directory":
                try:
                    stack.extend(sorted(path.iterdir(), key=lambda p: p.name, reverse=True))
                except FileNotFoundError:
                    continue
    return sorted(entries, key=lambda e: (e["root"], e["relative"]))


def verify_external_state(run: dict) -> dict:
    state = run.get("external_state")
    if not state:
        return {"violations": [], "modified": [], "removed": [], "added": []}
    baseline = json.loads(Path(state["baseline_file"]).read_text())
    before = {entry["path"]: entry for entry in baseline["entries"]}
    current = external_snapshot([Path(p) for p in baseline["protected_paths"]])
    after = {entry["path"]: entry for entry in current}
    add_paths = [Path(p) for p in (run.get("access_profile") or {}).get("external_add_paths", [])]

    removed = sorted(path for path in before if path not in after)
    added = sorted(path for path in after if path not in before and not any(path_inside(Path(path), root) for root in add_paths))
    modified = []
    for path in sorted(set(before) & set(after)):
        left, right = before[path], after[path]
        if left.get("kind") == "directory" and right.get("kind") == "directory":
            continue
        fields = ("kind", "size", "mtime_ns", "target", "digest")
        if any(left.get(field) != right.get(field) for field in fields):
            modified.append(path)
    violations = []
    if modified:
        violations.append("external-state-modified")
    if removed:
        violations.append("external-state-removed")
    if added:
        violations.append("external-state-added")
    return {"violations": violations, "modified": modified, "removed": removed, "added": added}


def init_launch_repo(scenario: Scenario, repo: Path, layout: str = "single") -> tuple[str, str, Path]:
    shutil.copytree(scenario.environment, repo, symlinks=True)
    git(repo, "init", "-q", "-b", "main")
    git(repo, "config", "user.email", "bench@corenius")
    git(repo, "config", "user.name", "bench")
    git(repo, "config", "--local", "collab.overrideOwners", "owner")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "main")
    main = git(repo, "rev-parse", "HEAD").strip()
    integration = scenario.integration_branch or "effort/demo"
    if layout == "single":
        checkout = repo
        git(checkout, "checkout", "-q", "-b", integration)
    elif layout == "two-worktree":
        checkout = repo.parent / "integration"
        git(repo, "worktree", "add", "-q", "-b", integration, str(checkout), "main")
    else:
        raise LeadDrivenError(f"unknown launch layout {layout!r}")
    (checkout / "integration.txt").write_text("integration branch tip\n")
    git(checkout, "add", "integration.txt")
    git(checkout, "commit", "-q", "-m", "integration")
    if scenario.id == "launch-by-edges-and-triage":
        effort = scenario.effort or "demo"
        candidate = commit_candidate(repo, integration, "fixture-01", "resolved blocker landed\n")
        git(checkout, "merge", "--ff-only", "-q", candidate)
        record_acceptance(checkout, effort, "01", candidate)
    return main, git(checkout, "rev-parse", "HEAD").strip(), checkout


def move_records_to_branch(repo: Path, code_checkout: Path, effort: str, record_branch: str) -> Path:
    records = repo.parent / "records"
    git(repo, "worktree", "add", "-q", "-b", record_branch, str(records), git(code_checkout, "rev-parse", "HEAD").strip())
    effort_file = records / "docs" / "efforts" / effort / "EFFORT.md"
    effort_file.write_text(effort_file.read_text() + f"Record branch: {record_branch}\n")
    git(records, "add", f"docs/efforts/{effort}/EFFORT.md")
    git(records, "commit", "-q", "-m", "move effort records")
    for checkout, subject in ((repo, "remove records from main"), (code_checkout, "remove records from code branch")):
        if (checkout / "docs" / "efforts" / effort).exists():
            git(checkout, "rm", "-qr", f"docs/efforts/{effort}")
            git(checkout, "commit", "-q", "-m", subject)
    return records


def add_record_branch_worktree(repo: Path, effort_checkout: Path, effort: str, record_branch: str, records: Path) -> Path:
    git(repo, "worktree", "add", "-q", "-b", record_branch, str(records), git(effort_checkout, "rev-parse", "HEAD").strip())
    effort_file = records / "docs" / "efforts" / effort / "EFFORT.md"
    if not re.search(r"^Record branch:\s*", effort_file.read_text(), re.M):
        effort_file.write_text(effort_file.read_text() + f"Record branch: {record_branch}\n")
        git(records, "add", f"docs/efforts/{effort}/EFFORT.md")
        git(records, "commit", "-q", "-m", "move effort records")
    return records


def verify_handoff(run_dir: Path) -> dict:
    run = json.loads((run_dir / "run.json").read_text())
    repo = Path(run["worktree"])
    exit_path = run_dir / "exit_code"
    exit_text = exit_path.read_text().strip() if exit_path.exists() else ""
    exit_code = int(exit_text) if exit_text else None
    status = git(repo, "status", "--porcelain", "--untracked-files=all", check=False)
    head = git(repo, "rev-parse", "HEAD", check=False).strip()
    contains_base = subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", run["base_sha"], "HEAD"],
        capture_output=True,
        env=git_env(),
    ).returncode == 0
    reasons = []
    violations = []
    mode = run.get("handoff_mode", "exit")
    if mode == "exit" and exit_code != 0:
        reasons.append("exit-code")
    if mode == "marker":
        subject = git(repo, "log", "-1", "--format=%s", "HEAD", check=False).strip()
        if subject != run.get("handoff_marker"):
            reasons.append("handoff-marker")
    if status.strip():
        reasons.append("dirty-tree")
    if not contains_base:
        reasons.append("base-not-ancestor")
        violations.append("base-not-ancestor")
        reasons.append("boundary-violation")
    count = 0
    if contains_base:
        count = int(git(repo, "rev-list", "--count", f"{run['base_sha']}..HEAD", check=False).strip() or "0")
    if count < 1:
        reasons.append("no-commit-after-base")
    access_violations = command_violations(run, command_events(run_dir))
    if access_violations:
        violations.extend(access_violations)
        reasons.append("boundary-violation")
    external = verify_external_state(run)
    if external["violations"]:
        violations.extend(external["violations"])
        reasons.append("boundary-violation")

    for name, before in sorted(run.get("branch_baselines", {}).items()):
        after = git(repo, "rev-parse", "--verify", f"refs/heads/{name}", check=False).strip()
        if after != before:
            violations.append("foreign-branch-moved")
            reasons.append("boundary-violation")
    for name, after in sorted(local_branch_heads(repo).items()):
        if name != run["branch"] and name not in run.get("branch_baselines", {}):
            violations.append("foreign-branch-moved")
            reasons.append("boundary-violation")
    ready = not reasons
    candidate = None
    if ready:
        candidate = {
            "commit": head,
            "tree": git(repo, "rev-parse", "HEAD^{tree}", check=False).strip(),
            "base": run["base_sha"],
            "brief_digest": run["brief_digest"],
        }
    return {
        "ready_for_review": ready,
        "candidate": candidate,
        "not_ready": sorted(set(reasons)),
        "violations": sorted(set(violations)),
        "external_state": external,
        "exit_code": exit_code,
        "worktree_status": status,
        "branch_tip": head,
    }


def local_branch_heads(repo: Path) -> dict[str, str]:
    out = git(repo, "for-each-ref", "--format=%(refname:short) %(objectname)", "refs/heads", check=False)
    heads = {}
    for line in out.splitlines():
        if " " in line:
            name, sha = line.split(" ", 1)
            heads[name] = sha
    return heads


def judge_result(expected_ready: bool, toolbox_verdict: dict, verification: dict) -> tuple[str, list[str]]:
    details = []
    toolbox_ready = bool(toolbox_verdict.get("ready_for_review"))
    verified_ready = bool(verification.get("ready_for_review"))
    if toolbox_ready != expected_ready:
        details.append("expected.ready_for_review")
    if toolbox_ready != verified_ready:
        details.append("ready_for_review")
    toolbox_violations = {f.get("kind") for f in toolbox_verdict.get("findings", []) if f.get("severity") == "VIOLATION"}
    for kind in verification.get("violations", []):
        if kind not in toolbox_violations:
            details.append(f"violation.{kind}")
    if set(toolbox_verdict.get("not_ready", [])) != set(verification.get("not_ready", [])):
        details.append("not_ready")
    if toolbox_ready:
        expected_candidate = verification.get("candidate") or {}
        actual_candidate = toolbox_verdict.get("candidate") or {}
        for field in ("commit", "tree", "base", "brief_digest"):
            if actual_candidate.get(field) != expected_candidate.get(field):
                details.append(f"candidate.{field}")
    return ("fails" if details else "passes", details)


def read_json_stdout(proc: subprocess.CompletedProcess, context: str):
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as error:
        raise LeadDrivenError(f"{context} returned invalid JSON") from error


def status_run(run_dir: str, env: dict) -> dict:
    proc = run_cmd(tool_cmd("worker", "status", run_dir, env=env), env=env)
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return read_json_stdout(proc, "worker status")


def worker_execution_failures(run_dir: Path) -> list[str]:
    details = []
    events = []
    unexpected_output = []
    events_path = run_dir / "events.jsonl"
    if events_path.exists():
        for line in events_path.read_text(errors="replace").splitlines():
            text = line.strip()
            if not text:
                continue
            try:
                events.append(json.loads(text))
            except json.JSONDecodeError:
                unexpected_output.append(text)
    if not events and not unexpected_output:
        details.append("worker-not-started")
    details.extend(f"worker-unexpected-output:{line}" for line in unexpected_output)
    stderr = run_dir / "stderr.log"
    if stderr.exists() and stderr.read_text().strip():
        details.append("worker-stderr")
    exit_file = run_dir / "exit_code"
    if exit_file.exists() and exit_file.read_text().strip():
        try:
            exit_code = int(exit_file.read_text().strip())
        except ValueError:
            details.append("worker-exit-code")
        else:
            if exit_code != 0:
                details.append("worker-exit-code")
    return details


def observed_participant_actions(scenario: Scenario, participant: Participant, case_dir: Path, run_dir: Path) -> dict:
    if scenario.id != "access-profile":
        return {}
    run = json.loads((run_dir / "run.json").read_text())
    worktree = Path(run["worktree"])
    result = (worktree / "result.txt").read_text() if (worktree / "result.txt").exists() else ""
    observed: dict = {"failure_details": []}
    if participant.id == "default-network":
        observed["loopback_fetch"] = "ok" if "loopback ok" in result else "missing"
        if observed["loopback_fetch"] != "ok":
            observed["failure_details"].append("access.loopback")
    if participant.id == "long-command":
        exit_file = run_dir / "exit_code"
        elapsed = 0.0
        if exit_file.exists():
            elapsed = max(0.0, exit_file.stat().st_mtime - float(run.get("started_epoch", exit_file.stat().st_mtime)))
        observed["long_command_seconds"] = elapsed
        if elapsed <= 120.0:
            observed["failure_details"].append("access.long-command")
    if participant.id == "external-add-path":
        roots = run.get("access_profile", {}).get("external_add_paths") or []
        target = Path(roots[0]) / "allowed-output.txt" if roots else case_dir / "external-adds" / "allowed-output.txt"
        observed["external_add_file"] = str(target)
        if not target.is_file():
            observed["failure_details"].append("access.external-add")
    return observed


def stop_run(run_dir: str) -> None:
    run = json.loads((Path(run_dir) / "run.json").read_text())
    if run.get("pid") is None:
        return
    pid = int(run["pid"])
    try:
        group = os.getpgid(pid)
        os.killpg(group, signal.SIGTERM)
    except ProcessLookupError:
        return
    deadline = time.monotonic() + 5.0
    while time.monotonic() < deadline:
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return
        time.sleep(0.05)
    try:
        os.killpg(group, signal.SIGKILL)
    except ProcessLookupError:
        pass


def wait_for_candidate_moved(run_dir: str, env: dict, timeout: float = 5.0) -> dict:
    deadline = datetime.now(timezone.utc).timestamp() + timeout
    last = {}
    while datetime.now(timezone.utc).timestamp() < deadline:
        last = status_run(run_dir, env)
        if "candidate-moved" in json.dumps(last.get("findings", [])):
            return last
    return last


def move_target(repo: Path) -> str:
    target = repo / "target-moved.txt"
    target.write_text("target moved\n")
    git(repo, "add", target.name)
    git(repo, "commit", "-q", "-m", "target moved")
    return git(repo, "rev-parse", "main").strip()


def commit_candidate(repo: Path, base: str, branch: str, text: str) -> str:
    worktree = repo.parent / branch.replace("/", "-")
    git(repo, "worktree", "add", "-q", "-b", branch, str(worktree), base)
    (worktree / "result.txt").write_text(text)
    git(worktree, "add", "result.txt")
    git(worktree, "commit", "-q", "-m", f"candidate {branch}")
    tip = git(repo, "rev-parse", branch).strip()
    git(repo, "worktree", "remove", "--force", str(worktree))
    return tip


def record_acceptance(repo: Path, effort: str, ticket: str, candidate: str, *, commit: bool = True) -> dict:
    proc = run_cmd(
        tool_cmd(
            "acceptance",
            "--repo",
            str(repo),
            "record",
            effort,
            ticket,
            "--candidate",
            candidate,
            "--reviewer",
            "lead",
            "--reviewer-context",
            "lead",
            "--standards",
            "passed",
            "--spec",
            "passed",
            "--required-check",
            "unit",
            "--check",
            "unit=passed",
        )
    )
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    printed = read_json_stdout(proc, "acceptance record")
    record = json.loads(Path(printed["recorded"]).read_text().splitlines()[-1])
    rel = f"docs/efforts/{effort}/evidence/acceptance-{ticket}.jsonl"
    if commit:
        add_paths = [rel]
        worker_report = record.get("worker_report")
        if worker_report:
            add_paths.append(worker_report["path"])
        git(repo, "add", *add_paths)
        git(repo, "commit", "-q", "-m", f"accept {ticket}")
    return {"printed": printed, "record": record}


def record_review(repo: Path, effort: str, ticket: str, candidate: str, findings: Path) -> dict:
    proc = run_cmd(
        tool_cmd(
            "acceptance",
            "--repo",
            str(repo),
            "review",
            effort,
            ticket,
            "--candidate",
            candidate,
            "--reviewer",
            "lead",
            "--reviewer-context",
            "lead",
            "--standards",
            "passed",
            "--spec",
            "failed",
            "--findings-file",
            str(findings),
        )
    )
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return acceptance_record_from_output(proc.stdout)


def tracker_command(repo: Path, effort: str, *args: str, env: dict | None = None) -> subprocess.CompletedProcess:
    return run_cmd(tool_cmd("tracker", "--repo", str(repo), *args, env=env), env=env)


def commit_all(repo: Path, message: str) -> str:
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", message)
    return git(repo, "rev-parse", "HEAD").strip()


def effort_status_row(repo: Path, effort: str) -> dict:
    proc = run_cmd(tool_cmd("effort", "--repo", str(repo), "status"))
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    for line in proc.stdout.splitlines():
        if not line.strip():
            continue
        row = json.loads(line)
        if row.get("effort") == effort:
            return row
    raise LeadDrivenError(f"effort {effort} missing from status output")


def status_paths(text: str) -> set[str]:
    paths = set()
    for line in text.splitlines():
        if len(line) > 3:
            paths.add(line[3:].strip().strip('"'))
    return paths


def landing_target_branch(scenario: Scenario, repo: Path) -> str:
    if scenario.landing_mainline:
        return scenario.landing_mainline
    if scenario.acceptance:
        meta = load_effort_metadata(repo, scenario.acceptance.effort)
        integration = meta.get("integration_branch")
        if integration:
            return integration
    return "main"


def prepare_target_checkout(scenario: Scenario, repo: Path, case_dir: Path, target_branch: str) -> Path:
    if scenario.landing_target_worktree or git(repo, "branch", "--show-current").strip() != target_branch:
        target = case_dir / f"{target_branch.replace('/', '-')}-checkout"
        if git(repo, "rev-parse", "--verify", "-q", f"refs/heads/{target_branch}", check=False).strip():
            git(repo, "worktree", "add", "-q", str(target), target_branch)
        else:
            git(repo, "worktree", "add", "-q", "-b", target_branch, str(target), "main")
        return target
    return repo


def prepare_acceptance(scenario: Scenario, repo: Path, target_checkout: Path, case_dir: Path, candidate: str | None) -> dict:
    case = scenario.acceptance
    if not case:
        return {"repo": str(target_checkout), "record": None, "stdout": "", "stderr": "", "return_code": None}
    if not candidate:
        return {"repo": str(target_checkout), "record": None, "stdout": "", "stderr": "no candidate", "return_code": None}

    record_branch = f"effort/{case.effort}-records"
    record_checkout = add_record_branch_worktree(repo, target_checkout, case.effort, record_branch, case_dir / "records")
    if case.mode == "missing-record":
        return {"repo": str(record_checkout), "record": None, "stdout": "", "stderr": "", "return_code": None}
    record_candidate = git(target_checkout, "rev-parse", "HEAD").strip() if case.mode == "other-candidate" else candidate
    check_result = "failed" if case.mode in {"failing-check", "waived-check"} else "passed"
    command = tool_cmd(
        "acceptance",
        "--repo",
        str(record_checkout),
        "record",
        case.effort,
        case.ticket,
        "--candidate",
        record_candidate,
        "--reviewer",
        "lead",
        "--reviewer-context",
        "lead",
        "--standards",
        "passed",
        "--spec",
        "passed",
        "--required-check",
        "unit",
        "--check",
        f"unit={check_result}",
    )
    if case.mode == "waived-check":
        command += ["--waiver-owner", "owner", "--waiver-reason", "accepted fixture failure"]
    result = run_cmd(command)
    record = None
    record_path = None
    record_rel = f"docs/efforts/{case.effort}/evidence/acceptance-{case.ticket}.jsonl"
    record_text = None
    if result.returncode == 0:
        printed = read_json_stdout(result, "acceptance record")
        recorded = Path(printed["recorded"])
        record_path = str(recorded)
        record = json.loads(recorded.read_text().splitlines()[-1])
        record_text = recorded.read_text()
        git(record_checkout, "add", record_rel)
        git(record_checkout, "commit", "-q", "-m", f"accept {case.ticket}")
    return {
        "repo": str(record_checkout),
        "command": command,
        "return_code": result.returncode,
        "stdout": result.stdout.strip(),
        "stderr": result.stderr.strip(),
        "record": record,
        "record_path": record_path,
        "record_rel": record_rel,
        "record_text": record_text,
    }


def verify_landing(scenario: Scenario, record: dict, case_dir: Path) -> dict:
    repo = Path(record["repo"])
    run = record["run"]
    candidate = (record["toolbox_verdict"].get("candidate") or {}).get("commit")
    worker = Path(run["worktree"])
    worker_branch = run["branch"]
    target_branch = landing_target_branch(scenario, repo)
    target_checkout = prepare_target_checkout(scenario, repo, case_dir, target_branch)
    land_repo = repo if scenario.landing_repo == "source" else target_checkout
    acceptance_prepared = prepare_acceptance(scenario, repo, target_checkout, case_dir, candidate)
    before = {
        "worker_branch": git(repo, "rev-parse", worker_branch, check=False).strip(),
        "worker_status": git(worker, "status", "--porcelain", "--untracked-files=all", check=False),
    }
    if scenario.landing_expect == "stale":
        target_before = move_target(repo)
    else:
        target_before = git(repo, "rev-parse", target_branch).strip()
    gate_seen = case_dir / "land-gate-tip"
    gate = f'test "$LAND_TIP" = "$(git rev-parse HEAD)" && printf %s "$LAND_TIP" > {shlex.quote(str(gate_seen))}'
    command = tool_cmd(
        "land",
        candidate or "",
        "--repo",
        str(land_repo),
        "--no-push",
        "--gate",
        gate,
    )
    if scenario.landing_mainline:
        command += ["--mainline", scenario.landing_mainline]
    if scenario.acceptance:
        command += ["--effort", scenario.acceptance.effort, "--ticket", scenario.acceptance.ticket]
    elif target_branch == "main":
        command += ["--owner-mainline-override", "owner"]
    result = run_cmd(command)
    target_after = git(repo, "rev-parse", target_branch, check=False).strip()
    target_status_after = git(target_checkout, "status", "--porcelain", "--untracked-files=all", check=False)
    landed_result = None
    if (target_checkout / "result.txt").exists():
        landed_result = (target_checkout / "result.txt").read_text()
    after = {
        "worker_branch": git(repo, "rev-parse", worker_branch, check=False).strip(),
        "worker_status": git(worker, "status", "--porcelain", "--untracked-files=all", check=False),
    }
    observed = "other"
    gate_text = gate_seen.read_text() if gate_seen.exists() else None
    record_after = None
    if acceptance_prepared.get("record_path") and Path(acceptance_prepared["record_path"]).exists():
        record_after = Path(acceptance_prepared["record_path"]).read_text()
    status_ok = target_status_after == ""
    record_rel = acceptance_prepared.get("record_rel")
    if scenario.acceptance and record_rel and acceptance_prepared.get("record_text"):
        status_ok = status_paths(target_status_after) <= {record_rel}
    if (
        result.returncode == 0
        and result.stdout.strip() == candidate
        and target_after == candidate
        and status_ok
        and landed_result is not None
        and gate_text == candidate
        and record_after == acceptance_prepared.get("record_text")
    ):
        observed = "landed"
    elif (
        result.returncode == STALE_CANDIDATE
        and target_after == target_before
        and after == before
        and gate_text is None
    ):
        observed = "stale"
    elif (
        result.returncode == ACCEPTANCE_FAILED
        and target_after == target_before
        and after == before
        and gate_text is None
    ):
        observed = "acceptance-refused"
    details = []
    if observed != scenario.landing_expect:
        details.append("landing.expected")
    if scenario.landing_expect == "landed":
        if result.returncode != 0:
            details.append("landing.return_code")
        if result.stdout.strip() != candidate:
            details.append("landing.stdout")
        if target_after != candidate:
            details.append("landing.target")
        if gate_text != candidate:
            details.append("landing.gate")
        if not status_ok:
            details.append("landing.target_status")
        if record_after != acceptance_prepared.get("record_text"):
            details.append("landing.acceptance_record")
    if scenario.landing_expect == "stale":
        if result.returncode != STALE_CANDIDATE:
            details.append("landing.return_code")
        if target_after != target_before:
            details.append("landing.target")
        if after["worker_branch"] != before["worker_branch"]:
            details.append("landing.worker_branch")
        if after["worker_status"] != before["worker_status"]:
            details.append("landing.worker_status")
        if gate_text is not None:
            details.append("landing.gate")
    if scenario.landing_expect == "acceptance-refused":
        if result.returncode != ACCEPTANCE_FAILED:
            details.append("landing.return_code")
        if target_after != target_before:
            details.append("landing.target")
        if after["worker_branch"] != before["worker_branch"]:
            details.append("landing.worker_branch")
        if after["worker_status"] != before["worker_status"]:
            details.append("landing.worker_status")
        if gate_text is not None:
            details.append("landing.gate")
    return {
        "command": command,
        "return_code": result.returncode,
        "stdout": result.stdout.strip(),
        "stderr": result.stderr.strip(),
        "candidate": candidate,
        "target_before": target_before,
        "target_after": target_after,
        "target_status_after": target_status_after,
        "landed_result": landed_result,
        "gate_seen": gate_text,
        "acceptance": acceptance_prepared,
        "acceptance_record": acceptance_prepared.get("record"),
        "acceptance_record_text_after": record_after,
        "worker_branch_before": before["worker_branch"],
        "worker_branch_after": after["worker_branch"],
        "worker_status_before": before["worker_status"],
        "worker_status_after": after["worker_status"],
        "observed": observed,
        "details": details,
    }


def launch_and_supervise(scenario: Scenario, participant: Participant, case_dir: Path, runtime: str = "scripted") -> dict:
    raise RuntimeError('Retired imported benchmark execution. Use harp durable --help; only historical solo reference/noop replay remains supported.')


def commit_in_worktree(checkout: Path, path: str, text: str, message: str) -> None:
    (checkout / path).write_text(text)
    git(checkout, "add", path)
    git(checkout, "commit", "-q", "-m", message)


def init_decoy_repo(path: Path) -> str:
    path.mkdir(parents=True)
    git(path, "init", "-q", "-b", "main")
    (path / "README").write_text("decoy\n")
    git(path, "add", "README")
    git(path, "commit", "-q", "-m", "decoy")
    return git(path, "rev-parse", "HEAD").strip()


def run_shared_repository_case(scenario: Scenario, participant: Participant, case_dir: Path, runtime: str = "scripted") -> dict:
    repo = case_dir / "repo"
    case_dir.mkdir(parents=True)
    base = init_repo(scenario, repo)
    git(repo, "branch", "effort/demo", "main")
    decoy = case_dir / "decoy"
    decoy_before = init_decoy_repo(decoy)
    env = os.environ.copy()
    if worker_runtime(runtime) == "codex":
        env["CORENIUS_CODEX"] = str(scenario.path / "scripted_participant.py")
    else:
        env["CORENIUS_TRAECLI"] = str(scenario.path / "scripted_participant.py")
    env["CORENIUS_RUNS"] = str(case_dir / "runs")
    env["CORENIUS_READINESS_PARTICIPANT"] = participant.mode
    env["GIT_DIR"] = str(decoy / ".git")
    launch = run_cmd(
        tool_cmd(
            "worker",
            "launch",
            "--task",
            str(scenario.task),
            "--name",
            f"{scenario.id}-{participant.id}",
            "--repo",
            str(repo),
            "--base",
            base,
            "--runtime",
            worker_runtime(runtime),
            "--stall",
            "10",
            "--allow",
            "out.md",
            env=env,
        ),
        env=env,
    )
    if launch.returncode != 0:
        raise LeadDrivenError(launch.stderr.strip() or launch.stdout.strip())
    launched = read_json_stdout(launch, "worker launch")
    try:
        unrelated = case_dir / "unrelated-agent"
        new_branch = case_dir / "new-agent"
        integration = case_dir / "integration"
        git(repo, "worktree", "add", "-q", str(unrelated), "side")
        git(repo, "worktree", "add", "-q", "-b", "agent/new-branch", str(new_branch), "main")
        git(repo, "worktree", "add", "-q", str(integration), "effort/demo")
        commit_in_worktree(unrelated, "side.txt", "side moved\n", "move side")
        commit_in_worktree(new_branch, "new.txt", "new branch moved\n", "move new branch")
        commit_in_worktree(integration, "integration.txt", "integration moved\n", "move integration")
        supervise = run_cmd(
            tool_cmd(
                "worker",
                "supervise",
                launched["run_dir"],
                "--interval",
                "0.1",
                "--max-wait",
                "20",
                env=env,
            ),
            env=env,
        )
        if supervise.returncode not in (0, 1, 2):
            raise LeadDrivenError(supervise.stderr.strip() or supervise.stdout.strip())
        toolbox_verdict = json.loads(supervise.stdout.strip().splitlines()[-1])
        verification = verify_handoff(Path(launched["run_dir"]))
        outcome, details = judge_result(participant.expect_ready, toolbox_verdict, verification)
        branch_findings = [f for f in toolbox_verdict.get("findings", []) if f.get("kind") == "shared-branch-moved"]
        if any(f.get("severity") == "VIOLATION" for f in branch_findings):
            details.append("shared-branch-violation")
            outcome = "fails"
        if "boundary-violation" in toolbox_verdict.get("not_ready", []):
            details.append("boundary-violation")
            outcome = "fails"
        decoy_after = git(decoy, "rev-parse", "HEAD").strip()
        if decoy_after != decoy_before:
            details.append("decoy-mutated")
            outcome = "fails"
        return {
            "scenario": scenario.id,
            "participant": participant.id,
            "scenario_result": outcome,
            "failure_details": details,
            "expected_ready": participant.expect_ready,
            "repo": str(repo),
            "case_dir": str(case_dir),
            "base": base,
            "observed": {
                "decoy_untouched": decoy_after == decoy_before,
                "branch_findings": branch_findings,
            },
            "run": {**json.loads((Path(launched["run_dir"]) / "run.json").read_text()),
                    "last_message": str(Path(launched["run_dir"]) / "last-message.md")},
            "toolbox_verdict": toolbox_verdict,
            "verification": verification,
        }
    finally:
        stop_run(launched["run_dir"])


def run_cmd(args: list[str], env: dict | None = None, cwd: Path | None = None) -> subprocess.CompletedProcess:
    try:
        return subprocess.run(args, capture_output=True, text=True, env=git_env(env), cwd=cwd)
    except OSError as error:
        raise LeadDrivenError(f"{args[0]} failed to start: {error}") from error


def ticket_file(repo: Path, effort: str, ticket: str) -> Path:
    matches = sorted((repo / "docs" / "efforts" / effort / "tickets").glob(f"{ticket}-*.md"))
    if not matches:
        raise LeadDrivenError(f"ticket {ticket} not found")
    return matches[0]


def set_blocker(repo: Path, effort: str, ticket: str, value: str) -> None:
    path = ticket_file(repo, effort, ticket)
    lines = path.read_text().splitlines()
    for index, line in enumerate(lines):
        if line.startswith("Blocked by:"):
            lines[index] = f"Blocked by: {value}"
            break
    path.write_text("\n".join(lines) + "\n")
    git(repo, "add", str(path.relative_to(repo)))
    git(repo, "commit", "-q", "-m", f"mutate ticket {ticket} blocker")


def source_still_pinned(repo: Path, source: dict) -> bool:
    data = subprocess.check_output(["git", "-C", str(repo), "show", f"{source['commit']}:{source['path']}"])
    return hashlib.sha256(data).hexdigest() == source["sha256"]


def stop_launch(launched: dict | None) -> None:
    if not launched:
        return
    try:
        stop_run(launched["run_dir"])
    except (FileNotFoundError, ProcessLookupError):
        pass


def launch_repo_arg(repo: Path, case: dict) -> tuple[str, Path | None]:
    if case.get("repo_arg") == ".":
        return ".", repo
    return str(repo), None


def lead_command_repo_arg(repo: Path) -> tuple[str, Path]:
    return ".", repo


def worker_runtime(runtime: str) -> str:
    return "codex" if runtime == "codex" else "traecli"


def runtime_binary_path(runtime: str, env: dict | None = None) -> Path:
    source = env if env is not None else os.environ
    if worker_runtime(runtime) == "codex":
        return Path(source.get("CORENIUS_CODEX", "codex"))
    return Path(source.get("CORENIUS_TRAECLI", "traecli"))


def is_real_runtime_binary(path: Path) -> bool:
    if not path.is_absolute():
        resolved = shutil.which(str(path))
        if not resolved:
            return False
        path = Path(resolved)
    try:
        resolved = path.resolve()
    except OSError:
        return False
    return path.exists() and not resolved.is_relative_to(READINESS)


def effective_proof(runtime: str, proof: str, records: list[dict] | None = None) -> str:
    if proof != "live":
        return proof
    runtime_flags = [
        record.get("runtime_binary_real")
        for record in records or []
        if "runtime_binary_real" in record
    ]
    if runtime_flags:
        return "live" if all(runtime_flags) else "deterministic"
    return "deterministic"


def effective_runtime(requested_runtime: str, proof: str) -> str:
    return requested_runtime if proof == "live" else "scripted"


def record_names(records: list[dict]) -> list[str]:
    return [str(record.get("participant") or record.get("case")) for record in records if record.get("participant") or record.get("case")]


def record_stories(records: list[dict], scenario: Scenario) -> list[int]:
    stories: set[int] = set()
    for record in records:
        value = record.get("stories")
        if value is None:
            value = scenario.stories or []
        stories.update(int(story) for story in value)
    return sorted(stories)


def case_story_list(scenario: Scenario, case: dict) -> list[int] | None:
    return list(case["stories"]) if "stories" in case else scenario.stories


def set_runtime_env(env: dict, scenario: Scenario, runtime: str) -> None:
    worker_override = os.environ.get("CORENIUS_READINESS_WORKER")
    if worker_override:
        env["CORENIUS_TRAECLI"] = worker_override
        env["CORENIUS_CODEX"] = worker_override
    elif worker_runtime(runtime) == "codex":
        env["CORENIUS_CODEX"] = str(scenario.path / "scripted_participant.py")
    else:
        env["CORENIUS_TRAECLI"] = str(scenario.path / "scripted_participant.py")


def append_scenario_row(
    out: Path,
    scenario: Scenario,
    records: list[dict],
    *,
    runtime: str,
    proof: str,
    interventions: list[dict],
    elapsed_seconds: float,
    reason: str | None = None,
) -> None:
    proof = effective_proof(runtime, proof, records)
    results.append_readiness_row(
        out,
        results.readiness_row(
            scenario=scenario.id,
            instantiation="lead-driven",
            runtime=effective_runtime(runtime, proof),
            proof=proof,
            outcome="fails" if reason else scenario_outcome(records, interventions),
            reason=reason,
            interventions=interventions,
            elapsed_seconds=elapsed_seconds,
            model_usage="unknown",
            participants=record_names(records),
            stories=record_stories(records, scenario),
            requested_runtime=runtime,
        ),
    )


def interactive_tmux_env(env: dict, case_dir: Path) -> dict:
    # Scenario names and retained output roots can exceed sockaddr_un's budget.
    # This private directory belongs to the scenario and is removed after tmux.
    parent = worker_tool.temp_parent()
    # Include mkdtemp's random suffix and tmux's own UID/socket-name components.
    longest_socket = parent / "tmux-XXXXXXXX" / "tmux" / f"tmux-{os.getuid()}" / "default"
    if len(os.fsencode(longest_socket)) > 107:
        raise LeadDrivenError("interactive tmux socket path exceeds 107 bytes; select a shorter owned TMPDIR")
    tmp = Path(tempfile.mkdtemp(prefix="tmux-", dir=parent))
    tmux_tmp = tmp / "tmux"
    tmp.mkdir(parents=True, exist_ok=True)
    tmux_tmp.mkdir(parents=True, exist_ok=True)
    return {**env, "TMPDIR": str(tmp), "TMUX_TMPDIR": str(tmux_tmp), "TMPPREFIX": str(tmp / "zsh")}


def tmux_window_names(session: str, env: dict) -> list[str]:
    proc = run_cmd(["tmux", "list-windows", "-t", session, "-F", "#{window_name}"], env=env)
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def kill_tmux_session(session: str, env: dict) -> None:
    run_cmd(["tmux", "kill-session", "-t", session], env=env)


def lead_json(repo_arg: str, cwd: Path | None, env: dict, *args: str) -> dict:
    proc = run_cmd(tool_cmd("lead_launch", "--repo", repo_arg, *args, env=env), env=env, cwd=cwd)
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return read_json_stdout(proc, "lead_launch")


def ticket_monitor_row(repo_arg: str, cwd: Path | None, env: dict, effort: str, ticket: str) -> dict:
    report = lead_json(repo_arg, cwd, env, "monitor-tickets", effort, "--json")
    for row in report["tickets"]:
        if row["ticket"] == ticket:
            return row
    raise LeadDrivenError(f"ticket {ticket} missing from monitor")


def wait_for_ticket_state(repo_arg: str, cwd: Path | None, env: dict, effort: str, ticket: str, state: str,
                          timeout: float = 10.0) -> dict:
    deadline = time.monotonic() + timeout
    last = {}
    while time.monotonic() < deadline:
        last = ticket_monitor_row(repo_arg, cwd, env, effort, ticket)
        if last.get("worker_state") == state:
            return last
        time.sleep(0.05)
    raise LeadDrivenError(f"ticket {ticket} did not reach {state}; last={last}")


def wait_command(repo_arg: str, cwd: Path | None, env: dict, effort: str, ticket: str, timeout: str) -> dict:
    proc = run_cmd(
        tool_cmd("lead_launch", "--repo", repo_arg, "wait", effort, ticket, "--timeout", timeout, "--json", env=env),
        env=env,
        cwd=cwd,
    )
    if proc.returncode not in (0, 1):
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return read_json_stdout(proc, "lead_launch wait")


def launch_command(scenario: Scenario, repo: Path, case: dict, env: dict, ticket: str | None = None,
                   cap: int | None = None, runtime: str = "scripted",
                   worker_cap: int = 30, stall: int = 10) -> subprocess.CompletedProcess:
    repo_arg, cwd = launch_repo_arg(repo, case)
    return run_cmd(
        tool_cmd(
            "lead_launch",
            "--repo",
            repo_arg,
            "launch",
            scenario.effort or "demo",
            ticket or str(case["ticket"]),
            "--parallel-cap",
            str(cap if cap is not None else scenario.parallel_cap),
            "--worker-cap",
            str(worker_cap),
            "--stall",
            str(stall),
            "--runtime",
            worker_runtime(runtime),
            env=env,
        ),
        env=env,
        cwd=cwd,
    )


def run_interactive_launch_and_monitors_case(scenario: Scenario, case: dict, case_dir: Path,
                                             runtime: str = "scripted") -> dict:
    repo = case_dir / "repo"
    case_dir.mkdir(parents=True)
    main_tip, integration_tip, integration = init_launch_repo(scenario, repo, "two-worktree")
    effort = scenario.effort or "demo"
    integration_branch = scenario.integration_branch or "effort/demo"
    records = move_records_to_branch(repo, integration, effort, f"{integration_branch}-records")
    signals = case_dir / "signals"
    signals.mkdir()
    env = interactive_tmux_env(os.environ.copy(), case_dir)
    env["CORENIUS_TRAECLI"] = str(scenario.path / "scripted_participant.py")
    env["CORENIUS_CODEX"] = str(scenario.path / "scripted_participant.py")
    env["CORENIUS_RUNS"] = str(case_dir / "runs")
    env["CORENIUS_TRACKER_NOW"] = "2026-10-05T12:00:00Z"
    env["CORENIUS_INTERACTIVE_SIGNAL_DIR"] = str(signals)
    env["CORENIUS_INTERACTIVE_INTEGRATION"] = integration_branch
    session = f"readiness-{os.getpid()}-{case_dir.name[-10:]}"
    repo_arg, cwd = lead_command_repo_arg(repo)
    launched_items: list[dict] = []
    observed: dict = {
        "main": main_tip,
        "initial_integration": integration_tip,
        "layout": "main-integration-record-workers",
        "record_worktree": str(records),
        "integration_worktree": str(integration),
    }
    details: list[str] = []
    try:
        open_payload = lead_json(repo_arg, cwd, env, "open-session", effort, "--tmux-session", session, "--interval", "1")
        observed["open_session_payload"] = open_payload
        observed["open_session_windows"] = tmux_window_names(session, env)
        if observed["open_session_windows"] != ["tickets", "workers"]:
            details.append("open-session")

        launch_proc = run_cmd(
            tool_cmd(
                "lead_launch",
                "--repo",
                repo_arg,
                "launch",
                effort,
                "01",
                "02",
                "03",
                "04",
                "--parallel-cap",
                str(scenario.parallel_cap),
                "--worker-cap",
                "180",
                "--stall",
                "10",
                "--runtime",
                worker_runtime(runtime),
                "--interactive",
                "--tmux-session",
                session,
                env=env,
            ),
            env=env,
            cwd=cwd,
        )
        observed["launch_returncode"] = launch_proc.returncode
        observed["launch_stderr"] = launch_proc.stderr.strip()
        if launch_proc.returncode != 0:
            details.append("interactive-launch")
            return {
                "scenario": scenario.id,
                "case": case["id"],
                "scenario_result": "fails",
                "failure_details": details,
                "expected": {key: value for key, value in case.items() if key.startswith("expect_")},
                "observed": observed,
                "repo": str(repo),
            }
        launch_payload = read_json_stdout(launch_proc, "lead_launch launch")
        launched_items = launch_payload["launched"]
        runs_by_ticket: dict[str, tuple[dict, Path]] = {}
        for item in launched_items:
            run_path = Path(item["run_dir"]) / "run.json"
            run = json.loads(run_path.read_text())
            ticket = run["lead_launch"]["ticket"]
            runs_by_ticket[ticket] = (run, Path(item["run_dir"]))
        observed["worker_workspace_count"] = len(runs_by_ticket)
        if observed["worker_workspace_count"] != 4:
            details.append("worker-workspaces")
        observed["interactive_window_available"] = all(
            run.get("lead_launch", {}).get("tmux_window", {}).get("available") for run, _ in runs_by_ticket.values()
        )
        if not observed["interactive_window_available"]:
            details.append("interactive-window")
        if not wait_for_files([signals / f"started-{ticket}.json" for ticket in ("01", "02", "03", "04")]):
            details.append("interactive-start")

        state_sequence = []
        running = ticket_monitor_row(repo_arg, cwd, env, effort, "01")
        state_sequence.append(running["worker_state"])
        trust_ready = wait_for_ticket_state(repo_arg, cwd, env, effort, "04", "ready")
        wait_trust_ready = wait_command(repo_arg, cwd, env, effort, "04", "0")
        observed["trust_override"] = {
            "state": trust_ready["worker_state"],
            "prompt": trust_ready.get("blocked_prompt"),
            "wait_trigger": wait_trust_ready.get("trigger"),
        }
        if (
            trust_ready["worker_state"] != "ready"
            or trust_ready.get("blocked_prompt")
            or not wait_trust_ready.get("triggered")
            or wait_trust_ready.get("trigger", {}).get("state") != "ready"
        ):
            details.append("trust-override")
        for ticket in ("01", "02", "03"):
            (signals / f"release-initial-{ticket}").write_text("go\n")

        first_status = wait_for_worker_status(runs_by_ticket["01"][1], env, "ready_for_review", update_run_record=True)
        first_candidate = first_status["candidate"]["commit"]
        finishing = wait_for_ticket_state(repo_arg, cwd, env, effort, "01", "finishing")
        state_sequence.append(finishing["worker_state"])
        needs_info = wait_for_ticket_state(repo_arg, cwd, env, effort, "02", "needs-info")
        stopped = wait_for_ticket_state(repo_arg, cwd, env, effort, "03", "stopped")

        findings = records / "docs" / "efforts" / effort / "evidence" / "review-01.md"
        findings.parent.mkdir(parents=True, exist_ok=True)
        findings.write_text("Fix result.txt before the ticket can land.\n")
        first_review = record_review(records, effort, "01", first_candidate, findings)
        commit_all(records, "review ticket 01")
        delivery_payload = lead_json(
            repo_arg,
            cwd,
            env,
            "rework",
            str(runs_by_ticket["01"][1]),
            "--findings-file",
            str(findings),
            "--ack-timeout",
            "1",
        )
        observed["review_delivery"] = delivery_payload["delivery"]
        if delivery_payload.get("mode") != "terminal":
            details.append("terminal-delivery")
        wait_rework = wait_command(repo_arg, cwd, env, effort, "01", "0")
        rework = wait_for_ticket_state(repo_arg, cwd, env, effort, "01", "rework")
        state_sequence.append(rework["worker_state"])
        worker_monitor = lead_json(repo_arg, cwd, env, "monitor-workers", effort, "--json")
        worker_row = next(row for row in worker_monitor["workers"] if row["ticket"] == "01")
        observed["worker_monitor"] = worker_row
        if not worker_row.get("last_delivery", {}).get("acknowledged"):
            details.append("worker-monitor-delivery")
        if not worker_row.get("current_activity") or not worker_row.get("context_remaining"):
            details.append("worker-monitor-pane")
        if wait_rework.get("triggered"):
            details.append("wait-rework")

        (signals / "release-fix-01").write_text("go\n")
        waiting = wait_for_ticket_state(repo_arg, cwd, env, effort, "01", "waiting for re-integration")
        state_sequence.append(waiting["worker_state"])
        wait_waiting = wait_command(repo_arg, cwd, env, effort, "01", "0")
        if not wait_waiting.get("triggered") or wait_waiting.get("trigger", {}).get("state") != "waiting for re-integration":
            details.append("wait-waiting")

        (integration / "integration-moved.txt").write_text("integration moved before re-integration\n")
        git(integration, "add", "integration-moved.txt")
        git(integration, "commit", "-q", "-m", "move integration before re-integration")
        moved_tip = git(repo, "rev-parse", integration_branch).strip()
        stale_findings = records / "docs" / "efforts" / effort / "evidence" / "stale-base-01.md"
        stale_findings.parent.mkdir(parents=True, exist_ok=True)
        stale_findings.write_text("The integration branch moved; re-integrate before the final marker.\n")
        stale_delivery_payload = lead_json(
            repo_arg,
            cwd,
            env,
            "rework",
            str(runs_by_ticket["01"][1]),
            "--findings-file",
            str(stale_findings),
            "--kind",
            "stale-base",
            "--ack-timeout",
            "1",
        )
        observed["stale_delivery"] = stale_delivery_payload["delivery"]
        if stale_delivery_payload["delivery"]["receipt"].get("refreshed_integration", {}).get("commit") != moved_tip:
            details.append("stale-refresh")
        if not wait_for_files([signals / "stale-received-01.json"]):
            details.append("stale-received")
        second_status = wait_for_successor_candidate(
            runs_by_ticket["01"][1],
            env,
            first_candidate,
            update_run_record=True,
        )
        second_candidate = second_status["candidate"]["commit"]
        ready = wait_for_ticket_state(repo_arg, cwd, env, effort, "01", "ready")
        state_sequence.append(ready["worker_state"])

        accepted = record_acceptance(records, effort, "01", second_candidate)
        commit_all(records, "accept ticket 01")
        gate_seen = case_dir / "land-gate-tip"
        gate = f'test "$LAND_TIP" = "$(git rev-parse HEAD)" && printf %s "$LAND_TIP" > {shlex.quote(str(gate_seen))}'
        land_proc = run_cmd(
            tool_cmd(
                "land",
                second_candidate,
                "--repo",
                repo_arg,
                "--no-push",
                "--effort",
                effort,
                "--ticket",
                "01",
                "--gate",
                gate,
                env=env,
            ),
            env=env,
            cwd=cwd,
        )
        landing = {
            "return_code": land_proc.returncode,
            "stdout": land_proc.stdout.strip(),
            "stderr": land_proc.stderr.strip(),
            "target_after": git(repo, "rev-parse", integration_branch, check=False).strip(),
            "gate_seen": gate_seen.read_text() if gate_seen.exists() else None,
            "observed": "landed" if land_proc.returncode == 0 and gate_seen.exists() and gate_seen.read_text() == second_candidate else "other",
        }
        observed["landing"] = landing
        if landing["observed"] != "landed" or landing["target_after"] != second_candidate:
            details.append("landing")
        resolved = tracker_command(records, effort, "resolve", effort, "01", "--owner", "worker-01", "--force", env=env)
        if resolved.returncode == 0:
            commit_all(records, "resolve ticket 01")
        landed = ticket_monitor_row(repo_arg, cwd, env, effort, "01")
        state_sequence.append(landed["worker_state"])
        state_sequence.append(needs_info["worker_state"])
        state_sequence.append(stopped["worker_state"])

        run_01 = json.loads((runs_by_ticket["01"][1] / "run.json").read_text())
        worker_tree = Path(run_01["worktree"])
        monitor_private_clone = {
            "worktree": str(worker_tree),
            "ahead": waiting["ahead"],
            "dirty_count": waiting["dirty_count"],
            "head": git(worker_tree, "rev-parse", "HEAD").strip(),
            "shared_branch_exists": bool(git(repo, "rev-parse", "--verify", "-q", run_01["branch"], check=False).strip()),
        }
        observed.update({
            "state_sequence": state_sequence,
            "first_candidate": first_candidate,
            "second_candidate": second_candidate,
            "moved_integration": moved_tip,
            "accepted_candidate": accepted["record"]["candidate"],
            "first_review": first_review["record"],
            "monitor_private_clone": monitor_private_clone,
            "ticket_01_worktree": str(worker_tree),
            "ticket_01_workspace_kind": run_01.get("workspace_kind"),
            "ticket_01_git_dir_layout": run_01.get("git_dir_layout"),
            "effort_record_exists_in_main_checkout": (repo / "docs" / "efforts" / effort / "EFFORT.md").exists(),
            "effort_record_exists_in_integration_worktree": (integration / "docs" / "efforts" / effort / "EFFORT.md").exists(),
            "effort_record_exists_in_record_worktree": (records / "docs" / "efforts" / effort / "EFFORT.md").exists(),
        })
        expected_states = [
            "running",
            "finishing",
            "rework",
            "waiting for re-integration",
            "ready",
            "landed",
            "needs-info",
            "stopped",
        ]
        if state_sequence != expected_states:
            details.append("states")
        if monitor_private_clone["dirty_count"] != 0 or monitor_private_clone["ahead"] < 2:
            details.append("private-clone-monitor")
        if not monitor_private_clone["shared_branch_exists"]:
            details.append("candidate-import")
        if run_01.get("workspace_kind") != "private-clone" or run_01.get("git_dir_layout") != "separate":
            details.append("workspace-kind")
        if observed["effort_record_exists_in_main_checkout"] or observed["effort_record_exists_in_integration_worktree"]:
            details.append("record-branch-layout")
        if not observed["effort_record_exists_in_record_worktree"]:
            details.append("record-worktree")
    finally:
        try:
            for item in launched_items:
                stop_launch(item)
        finally:
            try:
                kill_tmux_session(session, env)
            finally:
                shutil.rmtree(env["TMPDIR"])
    return {
        "scenario": scenario.id,
        "case": case["id"],
        "stories": case_story_list(scenario, case),
        "scenario_result": "fails" if details else "passes",
        "failure_details": details,
        "expected": {key: value for key, value in case.items() if key.startswith("expect_")},
        "observed": observed,
        "repo": str(repo),
    }


def run_launch_case(scenario: Scenario, case: dict, case_dir: Path, runtime: str = "scripted") -> dict:
    if case["action"] == "interactive-full-flow":
        return run_interactive_launch_and_monitors_case(scenario, case, case_dir, runtime)
    repo = case_dir / "repo"
    case_dir.mkdir(parents=True)
    layout = case.get("layout", "single")
    main_tip, integration_tip, launch_repo = init_launch_repo(scenario, repo, layout)
    env = os.environ.copy()
    set_runtime_env(env, scenario, runtime)
    env["CORENIUS_RUNS"] = str(case_dir / "runs")
    env["CORENIUS_TRACKER_NOW"] = "2026-10-05T12:00:00Z"
    env["CORENIUS_READINESS_PARTICIPANT"] = case.get("participant", "sleep")
    observed: dict = {"main": main_tip, "initial_integration": integration_tip, "layout": layout}
    launched = None
    details: list[str] = []
    try:
        action = case["action"]
        if action == "launchable":
            proc = run_cmd(
                tool_cmd("tracker", "--repo", str(launch_repo), "launchable", scenario.effort or "demo", "--json", env=env),
                env=env,
            )
            try:
                ids = [row["id"] for row in json.loads(proc.stdout)] if proc.returncode == 0 else []
            except json.JSONDecodeError:
                ids = []
                details.append("launchable-json")
            observed.update({"returncode": proc.returncode, "launchable_ids": ids, "stderr": proc.stderr})
            if proc.returncode != 0 or ids != list(case["expect_launchable"]):
                details.append("launchable")
        elif action == "refuse":
            if case.get("mutate_blocker"):
                set_blocker(launch_repo, scenario.effort or "demo", str(case["ticket"]), str(case["mutate_blocker"]))
            proc = launch_command(scenario, launch_repo, case, env, runtime=runtime)
            observed.update({"returncode": proc.returncode, "stderr": proc.stderr})
            if proc.returncode != 1 or str(case["expect_refusal"]) not in proc.stderr:
                details.append("refusal")
            run_jsons = list((case_dir / "runs").glob("workers/*/run.json")) if (case_dir / "runs").exists() else []
            if run_jsons:
                details.append("unexpected-worker")
        elif action == "parallel-cap":
            first = launch_command(scenario, launch_repo, case, env, ticket=str(case["holder_ticket"]), cap=1, runtime=runtime)
            if first.returncode == 0:
                launched = read_json_stdout(first, "lead launch")
            second = launch_command(scenario, launch_repo, case, env, ticket=str(case["ticket"]), cap=1, runtime=runtime)
            observed.update({"first_returncode": first.returncode, "second_returncode": second.returncode,
                             "stderr": second.stderr})
            if first.returncode != 0 or second.returncode != 1 or "parallel cap" not in second.stderr:
                details.append("parallel-cap")
        elif action == "launch":
            proc = launch_command(scenario, launch_repo, case, env, runtime=runtime)
            observed.update({"returncode": proc.returncode, "stderr": proc.stderr})
            if proc.returncode != 0:
                details.append("launch")
            else:
                launched = read_json_stdout(proc, "lead launch")
                run = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
                base = run["base_sha"]
                current_integration = git(repo, "rev-parse", scenario.integration_branch or "effort/demo").strip()
                branch_tip = git(Path(launched["worktree"]), "rev-parse", "HEAD").strip()
                sources = run["lead_launch"]["sources"]
                old_instruction = sources["worker_instructions"][0]
                (launch_repo / "AGENTS.md").write_text("agents changed after launch\n")
                git(launch_repo, "add", "AGENTS.md")
                git(launch_repo, "commit", "-q", "-m", "change instructions")
                after = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
                observed.update({
                    "base": "integration" if base == current_integration and base != main_tip else "other",
                    "branch_tip_is_base": branch_tip == base,
                    "sources": sorted(k for k in sources),
                    "worker_runtime": run.get("runtime"),
                    "lead_launch_runtime": run.get("lead_launch", {}).get("runtime"),
                    "record_sources_pinned": (
                        after["lead_launch"]["sources"]["worker_instructions"][0] == old_instruction
                        and source_still_pinned(repo, old_instruction)
                    ),
                })
                if case.get("expect_finding_texts"):
                    launch_findings = run["lead_launch"].get("findings", [])
                    prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
                    expected_texts = [str(text) for text in case["expect_finding_texts"]]
                    expected_sources = [str(source) for source in case["expect_finding_sources"]]
                    observed.update({
                        "finding_sources": [source.get("label") for finding in launch_findings for source in finding.get("sources", [])],
                        "prompt_contains_findings": all(text in prompt for text in expected_texts),
                    })
                    texts = [finding.get("text") for finding in launch_findings]
                    if texts != expected_texts or observed["finding_sources"] != expected_sources or not observed["prompt_contains_findings"]:
                        details.append("handoff-findings")
                if layout == "two-worktree":
                    observed.update({
                        "claim_commit_on_integration": git(launch_repo, "log", "-2", "--format=%s").splitlines()[1]
                        == f"Claim ticket {case['ticket']} for worker",
                        "main_checkout_untouched": (
                            git(repo, "branch", "--show-current").strip() == "main"
                            and git(repo, "rev-parse", "HEAD").strip() == main_tip
                            and git(repo, "status", "--porcelain", "--untracked-files=all") == ""
                        ),
                        "mainline_untouched": git(repo, "rev-parse", "main").strip() == main_tip,
                    })
                if observed["base"] != "integration" or not observed["branch_tip_is_base"] or not observed["record_sources_pinned"]:
                    details.append("launch-record")
                if layout == "two-worktree" and not (
                    observed["claim_commit_on_integration"]
                    and observed["main_checkout_untouched"]
                    and observed["mainline_untouched"]
                ):
                    details.append("two-worktree")
        elif action == "question":
            proc = launch_command(scenario, launch_repo, case, env, runtime=runtime)
            observed.update({"launch_returncode": proc.returncode, "launch_stderr": proc.stderr})
            if proc.returncode == 0:
                launched = read_json_stdout(proc, "lead launch")
                question = str(case["question"])
                repo_arg, cwd = launch_repo_arg(launch_repo, case)
                q = run_cmd(
                    tool_cmd(
                        "lead_launch",
                        "--repo",
                        repo_arg,
                        "question",
                        scenario.effort or "demo",
                        str(case["ticket"]),
                        "--owner",
                        f"worker-{case['ticket']}",
                        "--question",
                        question,
                        env=env,
                    ),
                    env=env,
                    cwd=cwd,
                )
                lint = run_cmd(tool_cmd("tracker", "--repo", str(launch_repo), "lint", scenario.effort or "demo", env=env), env=env)
                text = ticket_file(launch_repo, scenario.effort or "demo", str(case["ticket"])).read_text()
                observed.update({"question_returncode": q.returncode, "lint_ok": lint.returncode == 0,
                                 "triage": "needs-info" if "Triage: needs-info" in text else "other",
                                 "comment_contains_question": question in text})
                if q.returncode != 0 or lint.returncode != 0 or observed["triage"] != "needs-info" or not observed["comment_contains_question"]:
                    details.append("question")
            else:
                details.append("question-launch")
        elif action == "tmux-missing-launch":
            fake_bin = case_dir / "bin"
            fake_bin.mkdir()
            fake_tmux = fake_bin / "tmux"
            fake_tmux.write_text("#!/bin/sh\necho tmux unavailable >&2\nexit 127\n")
            fake_tmux.chmod(0o755)
            tmux_env = {**env, "PATH": str(fake_bin) + os.pathsep + env.get("PATH", "")}
            proc = run_cmd(
                tool_cmd(
                    "lead_launch",
                    "--repo",
                    str(launch_repo),
                    "launch",
                    scenario.effort or "demo",
                    str(case["ticket"]),
                    "--parallel-cap",
                    str(scenario.parallel_cap),
                    "--worker-cap",
                    "30",
                    "--stall",
                    "10",
                    "--tmux-session",
                    "readiness-watch",
                    env=tmux_env,
                ),
                env=tmux_env,
            )
            observed.update({"returncode": proc.returncode, "stderr": proc.stderr})
            if proc.returncode == 0:
                launched = read_json_stdout(proc, "lead launch")
                run = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
                observed["tmux_window"] = run["lead_launch"]["tmux_window"]
                commands = []
                for _ in range(20):
                    commands = command_events(Path(launched["run_dir"]))
                    if "printf watching-limits-accounting" in commands:
                        break
                    time.sleep(0.05)
                stop_launch(launched)
                observed["action_record_commands"] = command_events(Path(launched["run_dir"])) or commands
                launched = None
            if proc.returncode != 0 or observed.get("tmux_window", {}).get("available") is not False:
                details.append("tmux-missing")
            if "printf watching-limits-accounting" not in observed.get("action_record_commands", []):
                details.append("action-record")
        elif action == "limit-notices-and-accounting":
            env["CORENIUS_READINESS_PARTICIPANT"] = "stall"
            proc = launch_command(scenario, launch_repo, case, env, runtime=runtime, worker_cap=2, stall=1)
            observed.update({"launch_returncode": proc.returncode, "launch_stderr": proc.stderr})
            if proc.returncode != 0:
                details.append("launch")
            else:
                launched = json.loads(proc.stdout)
                run_path = Path(launched["run_dir"]) / "run.json"
                run = json.loads(run_path.read_text())
                supervise = run_cmd(
                    tool_cmd("worker", "supervise", launched["run_dir"], "--interval", "0.1", "--max-wait", "6", env=env),
                    env=env,
                )
                if supervise.returncode not in (worker_tool.RUNNING, worker_tool.CLEAN, worker_tool.DEGRADED, worker_tool.VIOLATED):
                    details.append("supervise")
                status_rows = json_lines(Path(launched["run_dir"]) / "status.jsonl")
                notice_kinds = [notice["kind"] for row in status_rows for notice in row.get("notices", [])]
                wall_notice = next((row for row in status_rows for notice in row.get("notices", []) if notice["kind"] == "wall-clock-limit"), None)
                launched = None
                account = run_cmd(
                    tool_cmd(
                        "lead_launch",
                        "--repo",
                        str(launch_repo),
                        "account",
                        scenario.effort or "demo",
                        "--parallel-cap",
                        str(scenario.parallel_cap),
                        "--json",
                        env=env,
                    ),
                    env=env,
                )
                if account.returncode != 0:
                    details.append("account")
                    account_report = {"workers": []}
                else:
                    account_report = json.loads(account.stdout)
                rows = account_report.get("workers", [])
                row = next((item for item in rows if item.get("ticket") == str(case["ticket"])), {})
                observed.update({
                    "status_notice_kinds": notice_kinds,
                    "elapsed_before_notice": wall_notice.get("elapsed") if wall_notice else None,
                    "cap_seconds": run.get("cap_seconds"),
                    "supervise_returncode": supervise.returncode,
                    "accounting_process_state": row.get("process_state"),
                    "disk_usage": row.get("disk_usage") or {},
                    "available_slots": account_report.get("available_slots"),
                })
                if "wall-clock-limit" not in notice_kinds:
                    details.append("wall-clock-limit")
                if "quiet-time" not in notice_kinds:
                    details.append("quiet-time")
                if not wall_notice or wall_notice.get("elapsed", 0) < int(run.get("cap_seconds", 0)):
                    details.append("wall-clock-elapsed")
                if row.get("process_state") != "stopped":
                    details.append("process-state")
                disk = row.get("disk_usage") or {}
                if not disk.get("worktree_bytes") or not disk.get("worker_repo_bytes"):
                    details.append("disk-usage")
        else:
            raise LeadDrivenError(f"unknown launch case action {action!r}")
    finally:
        stop_launch(launched)
    return {
        "scenario": scenario.id,
        "case": case["id"],
        "stories": case_story_list(scenario, case),
        "scenario_result": "fails" if details else "passes",
        "failure_details": details,
        "expected": {key: value for key, value in case.items() if key.startswith("expect_")},
        "observed": observed,
        "repo": str(repo),
    }


def wait_for_files(paths: list[Path], timeout: float = 10.0) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if all(path.exists() for path in paths):
            return True
        time.sleep(0.05)
    return False


def lead_overlap(repo: Path, effort: str, env: dict) -> dict:
    repo_arg, cwd = lead_command_repo_arg(repo)
    proc = run_cmd(tool_cmd("lead_launch", "--repo", repo_arg, "overlap", effort, env=env), env=env, cwd=cwd)
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return json.loads(proc.stdout)


def worker_status(run_dir: Path, env: dict, *, update_run_record: bool = False) -> dict:
    args = ["status", str(run_dir)]
    if update_run_record:
        args.append("--update-run-record")
    proc = run_cmd(tool_cmd("worker", *args, env=env), env=env)
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return read_json_stdout(proc, "worker status")


def wait_for_worker_status(
    run_dir: Path,
    env: dict,
    field: str,
    timeout: float = 10.0,
    *,
    update_run_record: bool = False,
) -> dict:
    deadline = time.monotonic() + timeout
    last = {}
    while time.monotonic() < deadline:
        last = worker_status(run_dir, env, update_run_record=update_run_record)
        if last.get(field):
            return last
        time.sleep(0.05)
    raise LeadDrivenError(f"worker status field {field} did not become truthy; last={last}")


def wait_for_successor_candidate(
    run_dir: Path,
    env: dict,
    previous: str,
    timeout: float = 10.0,
    *,
    update_run_record: bool = False,
) -> dict:
    deadline = time.monotonic() + timeout
    last = {}
    while time.monotonic() < deadline:
        last = worker_status(run_dir, env, update_run_record=update_run_record)
        candidate = (last.get("candidate") or {}).get("commit")
        if last.get("ready_for_review") and candidate and candidate != previous:
            return last
        time.sleep(0.05)
    raise LeadDrivenError(f"worker did not pin a successor candidate; last={last}")


def supervise_worker(run_dir: Path, env: dict) -> dict:
    proc = run_cmd(
        tool_cmd("worker", "supervise", str(run_dir), "--interval", "0.1", "--max-wait", "10", env=env),
        env=env,
    )
    if proc.returncode not in (0, 1, 2):
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return json.loads(proc.stdout.strip().splitlines()[-1])


def record_and_land(repo: Path, effort: str, ticket: str, candidate: str, env: dict, acceptance_repo: Path | None = None) -> dict:
    acceptance_record = record_acceptance(acceptance_repo or repo, effort, ticket, candidate)
    gate_seen = repo.parent / f"gate-{ticket}.txt"
    gate = f'test "$LAND_TIP" = "$(git rev-parse HEAD)" && printf %s "$LAND_TIP" > {shlex.quote(str(gate_seen))}'
    proc = run_cmd(
        tool_cmd(
            "land",
            candidate,
            "--repo",
            str(repo),
            "--no-push",
            "--effort",
            effort,
            "--ticket",
            ticket,
            "--gate",
            gate,
            env=env,
        ),
        env=env,
    )
    if proc.returncode == 0:
        pass
    return {
        "returncode": proc.returncode,
        "stdout": proc.stdout.strip(),
        "stderr": proc.stderr.strip(),
        "candidate": candidate,
        "gate_seen": gate_seen.read_text() if gate_seen.exists() else None,
        "acceptance": acceptance_record,
        "integration_tip": git(repo, "rev-parse", "HEAD").strip(),
    }


def run_parallel_overlap_case(scenario: Scenario, case: dict, case_dir: Path, runtime: str = "scripted") -> dict:
    repo = case_dir / "repo"
    case_dir.mkdir(parents=True)
    _, _, integration = init_launch_repo(scenario, repo, "two-worktree")
    effort = scenario.effort or "demo"
    integration_branch = scenario.integration_branch or "effort/demo"
    env = os.environ.copy()
    set_runtime_env(env, scenario, runtime)
    env["CORENIUS_RUNS"] = str(case_dir / "runs")
    env["CORENIUS_TRACKER_NOW"] = "2026-10-05T12:00:00Z"
    signals = case_dir / "signals"
    signals.mkdir()
    env["CORENIUS_PARALLEL_SIGNAL_DIR"] = str(signals)
    env["CORENIUS_PARALLEL_INTEGRATION"] = integration_branch

    repo_arg, cwd = lead_command_repo_arg(repo)
    proc = run_cmd(
        tool_cmd(
            "lead_launch",
            "--repo",
            repo_arg,
            "launch",
            effort,
            "01",
            "02",
            "--parallel-cap",
            str(scenario.parallel_cap),
            "--worker-cap",
            "20",
            "--stall",
            "10",
            "--runtime",
            worker_runtime(runtime),
            env=env,
        ),
        env=env,
        cwd=cwd,
    )
    observed: dict = {"launch_returncode": proc.returncode, "launch_stderr": proc.stderr}
    details: list[str] = []
    launched_items: list[dict] = []
    try:
        if proc.returncode != 0:
            details.append("launch")
            return {
                "scenario": scenario.id,
                "case": case["id"],
                "scenario_result": "fails",
                "failure_details": details,
                "expected": {key: value for key, value in case.items() if key.startswith("expect_")},
                "observed": observed,
                "repo": str(repo),
            }
        launch_record = json.loads(proc.stdout)
        launched_items = launch_record["launched"]
        observed["launch_overlap_findings"] = launch_record.get("overlap", {}).get("findings", [])
        run_dirs = {json.loads((Path(item["run_dir"]) / "run.json").read_text())["lead_launch"]["ticket"]: Path(item["run_dir"]) for item in launched_items}
        if not wait_for_files([signals / "started-01.json", signals / "started-02.json"]):
            details.append("workers-started")
        running_overlap = lead_overlap(repo, effort, env)
        observed["running_overlap_findings"] = running_overlap["findings"]
        observed["active_tickets"] = [entry["ticket"] for entry in running_overlap["active"]]

        (signals / "release").write_text("go\n")
        verdict_01 = supervise_worker(run_dirs["01"], env)
        verdict_02 = supervise_worker(run_dirs["02"], env)
        worker_01 = Path(json.loads((run_dirs["01"] / "run.json").read_text())["worktree"])
        (worker_01 / "alpha.txt").write_text("lead write after candidate\n")
        status_01 = worker_status(run_dirs["01"], env)
        observed["workspace_status_findings"] = status_01["findings"]

        run_01 = json.loads((run_dirs["01"] / "run.json").read_text())
        run_02 = json.loads((run_dirs["02"] / "run.json").read_text())
        end_01 = (run_dirs["01"] / "exit_code").stat().st_mtime if (run_dirs["01"] / "exit_code").exists() else time.time()
        end_02 = (run_dirs["02"] / "exit_code").stat().st_mtime if (run_dirs["02"] / "exit_code").exists() else time.time()
        observed["intervals"] = {
            "01": {"start": run_01["started_epoch"], "end": end_01},
            "02": {"start": run_02["started_epoch"], "end": end_02},
        }
        observed["intervals_overlap"] = max(run_01["started_epoch"], run_02["started_epoch"]) <= min(end_01, end_02)

        candidate_01 = verdict_01.get("candidate", {}).get("commit")
        candidate_02_first = verdict_02.get("candidate", {}).get("commit")
        observed["first_02_candidate"] = candidate_02_first
        records = add_record_branch_worktree(repo, integration, effort, f"{integration_branch}-records", case_dir / "records")
        land_01 = record_and_land(integration, effort, "01", candidate_01, env, records) if candidate_01 else {"returncode": -1}
        observed["land_01"] = land_01
        after_first = git(integration, "rev-parse", "HEAD").strip()
        observed["after_first_landing"] = after_first
        stale_02 = run_cmd(
            tool_cmd(
                "land",
                candidate_02_first or "",
                "--repo",
                str(integration),
                "--mainline",
                integration_branch,
                "--no-push",
                "--allow-no-gate",
                env=env,
            ),
            env=env,
        )
        observed["stale_02_returncode"] = stale_02.returncode
        observed["stale_02_stderr"] = stale_02.stderr.strip()
        stale_findings = case_dir / "stale-base-02.md"
        stale_findings.write_text("Rebase ticket 02 onto the moved integration tip.\n")
        rework_02 = run_cmd(
            tool_cmd(
                "lead_launch",
                "--repo",
                repo_arg,
                "rework",
                str(run_dirs["02"]),
                "--findings-file",
                str(stale_findings),
                "--kind",
                "stale-base",
                env=env,
            ),
            env=env,
            cwd=cwd,
        )
        rework_02_payload = json.loads(rework_02.stdout) if rework_02.returncode == 0 else {}
        observed["rework_02_returncode"] = rework_02.returncode
        observed["rework_02_stderr"] = rework_02.stderr.strip()
        observed["rework_02_mode"] = rework_02_payload.get("mode")
        observed["resume_02_refreshed_integration"] = (rework_02_payload.get("resume") or {}).get("refreshed_integration")
        second_02_verdict = supervise_worker(run_dirs["02"], env) if rework_02.returncode == 0 else {}
        candidate_02_second = (second_02_verdict.get("candidate") or {}).get("commit")
        observed["second_02_candidate"] = candidate_02_second
        observed["second_02_base"] = (
            git(Path(run_02["worktree"]), "rev-parse", "HEAD^").strip()
            if candidate_02_second
            else None
        )
        land_02 = record_and_land(integration, effort, "02", candidate_02_second, env, records) if candidate_02_second else {"returncode": -1}
        observed["land_02"] = land_02
        observed["landed_tickets"] = [
            ticket for ticket, record in (("01", land_01), ("02", land_02)) if record.get("returncode") == 0
        ]

        if "declared-path-overlap" not in json.dumps(observed["launch_overlap_findings"]):
            details.append("launch-overlap")
        for kind in ("changed-outside-declared", "changed-path-declared-by-other", "common-undeclared-changed-path"):
            if kind not in json.dumps(observed["running_overlap_findings"]):
                details.append(kind)
        if "unaccounted-worktree-change" not in json.dumps(observed["workspace_status_findings"]):
            details.append("unaccounted-worktree-change")
        if not observed["intervals_overlap"]:
            details.append("interval-overlap")
        if observed["landed_tickets"] != ["01", "02"]:
            details.append("landing")
        if stale_02.returncode != STALE_REFUSED:
            details.append("stale-refusal")
        if rework_02.returncode != 0 or rework_02_payload.get("mode") != "resumed":
            details.append("redo-resume")
        if observed["second_02_base"] != after_first:
            details.append("redo-base")
        if candidate_02_first == candidate_02_second:
            details.append("new-candidate")
    finally:
        for item in launched_items:
            stop_launch(item)
    return {
        "scenario": scenario.id,
        "case": case["id"],
        "stories": case_story_list(scenario, case),
        "scenario_result": "fails" if details else "passes",
        "failure_details": details,
        "expected": {key: value for key, value in case.items() if key.startswith("expect_")},
        "observed": observed,
        "repo": str(repo),
    }


def run_parallel_overlap_scenario(
    scenario: Scenario,
    out: Path,
    only: list[str] | None = None,
    *,
    runtime: str = "scripted",
    proof: str = "deterministic",
    interventions: list[dict] | None = None,
) -> list[dict]:
    cases = scenario.cases or []
    if only:
        known = {case["id"] for case in cases}
        unknown = set(only) - known
        if unknown:
            raise LeadDrivenError(f"unknown case(s): {', '.join(sorted(unknown))}")
        cases = [case for case in cases if case["id"] in only]
    else:
        cases = [case for case in cases if case.get("default", True)]
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    records = []
    start = time.monotonic()
    interventions = interventions or []
    for case in cases:
        case_dir = out / f"{stamp}-{scenario.id}-{case['id']}"
        record = run_parallel_overlap_case(scenario, case, case_dir, runtime)
        (case_dir / "result.json").write_text(json.dumps(record, indent=2) + "\n")
        records.append(record)
    append_scenario_row(
        out,
        scenario,
        records,
        runtime=runtime,
        proof=proof,
        interventions=interventions,
        elapsed_seconds=time.monotonic() - start,
    )
    return records


def run_integration_branch_case(scenario: Scenario, case: dict, case_dir: Path, runtime: str = "scripted") -> dict:
    repo = case_dir / "repo"
    case_dir.mkdir(parents=True)
    _, _, integration = init_launch_repo(scenario, repo, "two-worktree")
    effort = scenario.effort or "demo"
    integration_branch = scenario.integration_branch or "effort/demo"
    action = case["action"]
    observed: dict = {}
    details: list[str] = []
    env = os.environ.copy()
    set_runtime_env(env, scenario, runtime)
    env["CORENIUS_RUNS"] = str(case_dir / "runs")
    env["CORENIUS_TRACKER_NOW"] = "2026-10-05T12:00:00Z"
    env["CORENIUS_BOOKKEEPING_EFFORT"] = effort
    launched = None
    try:
        if action == "land-default":
            candidate = commit_candidate(repo, integration_branch, "candidate-01", "landed on integration\n")
            records = add_record_branch_worktree(repo, integration, effort, f"{integration_branch}-records", case_dir / "records")
            record_acceptance(records, effort, "01", candidate)
            main_before = git(repo, "rev-parse", "main").strip()
            command = [
                *tool_cmd("land"),
                candidate,
                "--repo",
                str(integration),
                "--no-push",
                "--effort",
                effort,
                "--ticket",
                "01",
                "--allow-no-gate",
            ]
            proc = run_cmd(command)
            observed.update({
                "candidate": candidate,
                "returncode": proc.returncode,
                "stderr": proc.stderr,
                "land_command_uses_effort_record": "--mainline" not in command,
                "integration_after": git(repo, "rev-parse", integration_branch).strip(),
                "main_before": main_before,
                "main_after": git(repo, "rev-parse", "main").strip(),
            })
            if proc.returncode != 0 or observed["integration_after"] != candidate or observed["main_after"] != main_before:
                details.append("land-default")
        elif action == "mainline-refused":
            candidate = commit_candidate(repo, "main", "candidate-main-refused", "mainline candidate\n")
            records = add_record_branch_worktree(repo, integration, effort, f"{integration_branch}-records", case_dir / "records")
            record_acceptance(records, effort, "01", candidate)
            main_before = git(repo, "rev-parse", "main").strip()
            integration_before = git(repo, "rev-parse", integration_branch).strip()
            proc = run_cmd([
                *tool_cmd("land"),
                candidate,
                "--repo",
                str(repo),
                "--mainline",
                "main",
                "--no-push",
                "--effort",
                effort,
                "--ticket",
                "01",
            ])
            observed.update({
                "candidate": candidate,
                "returncode": proc.returncode,
                "stderr": proc.stderr,
                "main_before": main_before,
                "main_after": git(repo, "rev-parse", "main").strip(),
                "integration_before": integration_before,
                "integration_after": git(repo, "rev-parse", integration_branch).strip(),
            })
            if proc.returncode != MAINLINE_REFUSED or observed["main_after"] != main_before or observed["integration_after"] != integration_before:
                details.append("mainline-refused")
        elif action == "mainline-owner-override":
            candidate = commit_candidate(repo, "main", "candidate-main-override", "mainline override\n")
            records = add_record_branch_worktree(repo, integration, effort, f"{integration_branch}-records", case_dir / "records")
            record_acceptance(records, effort, "01", candidate)
            integration_before = git(repo, "rev-parse", integration_branch).strip()
            proc = run_cmd([
                *tool_cmd("land"),
                candidate,
                "--repo",
                str(repo),
                "--mainline",
                "main",
                "--no-push",
                "--effort",
                effort,
                "--ticket",
                "01",
                "--owner-mainline-override",
                "owner",
                "--allow-no-gate",
            ])
            record_path = records / "docs" / "efforts" / effort / "evidence" / "mainline-landing-overrides.jsonl"
            override_record = json.loads(record_path.read_text().splitlines()[-1]) if record_path.exists() else {}
            observed.update({
                "candidate": candidate,
                "returncode": proc.returncode,
                "stderr": proc.stderr,
                "main_after": git(repo, "rev-parse", "main").strip(),
                "integration_before": integration_before,
                "integration_after": git(repo, "rev-parse", integration_branch).strip(),
                "override_record": override_record,
            })
            if proc.returncode != 0 or observed["main_after"] != candidate or observed["integration_after"] != integration_before:
                details.append("mainline-owner-override")
            if override_record.get("owner") != "owner" or override_record.get("candidate") != candidate:
                details.append("override-record")
        elif action == "status":
            records = add_record_branch_worktree(repo, integration, effort, f"{integration_branch}-records", case_dir / "records")
            candidate_01 = commit_candidate(repo, integration_branch, "candidate-status-01", "status one\n")
            git(integration, "merge", "--ff-only", "-q", candidate_01)
            record_acceptance(records, effort, "01", candidate_01)
            candidate_02 = commit_candidate(repo, integration_branch, "candidate-status-02", "status two\n")
            git(integration, "merge", "--ff-only", "-q", candidate_02)
            record_acceptance(records, effort, "02", candidate_02)
            before = {
                "main": git(repo, "rev-parse", "main").strip(),
                "integration": git(repo, "rev-parse", integration_branch).strip(),
            }
            (repo / "mainline.txt").write_text("main moved\n")
            git(repo, "add", "mainline.txt")
            git(repo, "commit", "-q", "-m", "move mainline")
            status_before = {
                "main": git(repo, "rev-parse", "main").strip(),
                "integration": git(repo, "rev-parse", integration_branch).strip(),
            }
            row = effort_status_row(integration, effort)
            status_after = {
                "main": git(repo, "rev-parse", "main").strip(),
                "integration": git(repo, "rev-parse", integration_branch).strip(),
            }
            observed.update({
                "integration_behind_mainline": row.get("integration_behind_mainline"),
                "landed_candidates": row.get("landed_candidates"),
                "refs_unchanged_by_status": status_after == status_before,
            })
            if not observed["integration_behind_mainline"]:
                details.append("status-behind")
            if row.get("landed_candidates") != [
                {"ticket": "01", "candidate": candidate_01},
                {"ticket": "02", "candidate": candidate_02},
            ]:
                details.append("status-candidates")
            if status_after != status_before:
                details.append("status-mutated-ref")
        elif action == "ticket-without-record-branch-refused":
            candidate = commit_candidate(repo, integration_branch, "candidate-no-record-branch", "no record branch\n")
            record_acceptance(integration, effort, "01", candidate, commit=False)
            git(integration, "add", f"docs/efforts/{effort}/evidence/acceptance-01.jsonl")
            git(integration, "commit", "-q", "-m", "accept 01 on integration")
            integration_before = git(repo, "rev-parse", integration_branch).strip()
            gate_seen = case_dir / "gate-ran"
            proc = run_cmd(tool_cmd(
                "land",
                candidate,
                "--repo",
                str(integration),
                "--no-push",
                "--effort",
                effort,
                "--ticket",
                "01",
                "--gate",
                f"touch {shlex.quote(str(gate_seen))}",
            ))
            observed.update({
                "candidate": candidate,
                "returncode": proc.returncode,
                "stderr": proc.stderr,
                "integration_before": integration_before,
                "integration_after": git(repo, "rev-parse", integration_branch).strip(),
                "gate_ran": gate_seen.exists(),
            })
            if (
                proc.returncode != RECORD_BRANCH_REQUIRED
                or observed["integration_after"] != integration_before
                or observed["gate_ran"]
                or "effort.py move-records" not in proc.stderr
            ):
                details.append("ticket-without-record-branch-refused")
        elif action == "launch":
            command = [
                *tool_cmd("lead_launch", env=env),
                "--repo",
                str(integration),
                "launch",
                effort,
                "01",
                "--parallel-cap",
                str(scenario.parallel_cap),
                "--worker-cap",
                "30",
                "--stall",
                "10",
            ]
            proc = run_cmd(command, env=env)
            observed.update({
                "returncode": proc.returncode,
                "stderr": proc.stderr,
                "command_passed_integration": "--integration" in command,
            })
            if proc.returncode == 0:
                launched = json.loads(proc.stdout)
                run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
                observed.update({
                    "recorded_integration": run_data["lead_launch"]["integration"],
                    "base_is_integration": run_data["base_sha"] == git(repo, "rev-parse", integration_branch).strip(),
                })
            if (
                proc.returncode != 0
                or observed["command_passed_integration"]
                or observed.get("recorded_integration") != integration_branch
                or not observed.get("base_is_integration")
            ):
                details.append("launch-from-effort-record")
        elif action == "main-checkout-effort-worktree":
            (repo / "docs" / "efforts").mkdir(parents=True, exist_ok=True)
            git(repo, "rm", "-qr", "docs/efforts")
            git(repo, "commit", "-q", "-m", "main without effort record")
            candidate = commit_candidate(repo, integration_branch, "candidate-main-checkout", "main checkout land\n")
            records = add_record_branch_worktree(repo, integration, effort, f"{integration_branch}-records", case_dir / "records")
            record_acceptance(records, effort, "01", candidate)
            main_before = git(repo, "rev-parse", "main").strip()
            command = [
                *tool_cmd("land"),
                candidate,
                "--repo",
                str(repo),
                "--no-push",
                "--effort",
                effort,
                "--ticket",
                "01",
                "--allow-no-gate",
            ]
            proc = run_cmd(command)
            row = effort_status_row(repo, effort)
            observed.update({
                "candidate": candidate,
                "returncode": proc.returncode,
                "stderr": proc.stderr,
                "land_command_repo": "main",
                "integration_worktree": str(integration.resolve()),
                "record_worktree": str(records.resolve()),
                "effort_record_exists_in_main_checkout": (repo / "docs" / "efforts" / effort / "EFFORT.md").exists(),
                "integration_after": git(repo, "rev-parse", integration_branch).strip(),
                "main_before": main_before,
                "main_after": git(repo, "rev-parse", "main").strip(),
                "main_checkout_status": git(repo, "status", "--porcelain", "--untracked-files=all"),
                "status_found_effort": row.get("effort") == effort,
                "status_effort_worktree": row.get("effort_worktree"),
                "status_landed_candidates": row.get("landed_candidates"),
            })
            if (
                proc.returncode != 0
                or observed["integration_after"] != candidate
                or observed["main_after"] != main_before
                or observed["main_checkout_status"]
                or observed["effort_record_exists_in_main_checkout"]
            ):
                details.append("main-checkout-land")
            if row.get("effort_worktree") != str(records.resolve()) or row.get("landed_candidates") != [{"ticket": "01", "candidate": candidate}]:
                details.append("main-checkout-status")
        elif action == "bookkeeping-while-worker-runs":
            records = move_records_to_branch(repo, integration, effort, f"{integration_branch}-records")
            code_tip = git(repo, "rev-parse", integration_branch).strip()
            launch_proc = run_cmd(
                tool_cmd(
                    "lead_launch",
                    "--repo",
                    str(repo),
                    "launch",
                    effort,
                    "01",
                    "--parallel-cap",
                    str(scenario.parallel_cap),
                    "--worker-cap",
                    "30",
                    "--stall",
                    "10",
                    env=env,
                ),
                env=env,
            )
            observed.update({"launch_returncode": launch_proc.returncode, "launch_stderr": launch_proc.stderr})
            if launch_proc.returncode != 0:
                details.append("launch")
            else:
                launched = json.loads(launch_proc.stdout)
                run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
                supervise = run_cmd(
                    tool_cmd("worker", "supervise", launched["run_dir"], "--interval", "0.1", "--max-wait", "5", env=env),
                    env=env,
                )
                if supervise.returncode not in (worker_tool.CLEAN, worker_tool.DEGRADED, worker_tool.VIOLATED):
                    details.append("worker-supervise")
                    candidate = None
                    verdict = {}
                else:
                    try:
                        verdict = json.loads(supervise.stdout.strip().splitlines()[-1])
                    except (IndexError, json.JSONDecodeError):
                        details.append("worker-supervise-json")
                        verdict = {}
                    candidate = (verdict.get("candidate") or {}).get("commit")
                    if not verdict.get("ready_for_review") or not candidate:
                        details.append("worker-candidate")
                observed.update({
                    "base": code_tip,
                    "worker_base": run_data["base_sha"],
                    "worker_branch_tip_before_land": candidate,
                })
                if run_data["base_sha"] != code_tip:
                    details.append("candidate-base")
                if candidate:
                    claim = tracker_command(records, effort, "claim", effort, "02", "--owner", "lead", env=env)
                    resolve = tracker_command(records, effort, "resolve", effort, "03", "--owner", "worker-03", "--force", env=env)
                    if claim.returncode == 0 or resolve.returncode == 0:
                        commit_all(records, "claim and resolve tickets")
                    candidate_04 = commit_candidate(repo, integration_branch, "candidate-04", "ticket four\n")
                    acceptance_record = record_acceptance(records, effort, "04", candidate_04)
                    acceptance_01 = record_acceptance(records, effort, "01", candidate)
                    record_tip = git(records, "rev-parse", "HEAD").strip()
                    gate_seen = case_dir / "land-gate-tip"
                    land_proc = run_cmd([
                        *tool_cmd("land"),
                        candidate,
                        "--repo",
                        str(repo),
                        "--no-push",
                        "--effort",
                        effort,
                        "--ticket",
                        "01",
                        "--gate",
                        f'test "$LAND_TIP" = "$(git rev-parse HEAD)" && printf %s "$LAND_TIP" > {shlex.quote(str(gate_seen))}',
                    ])
                    status_row = effort_status_row(records, effort)
                    records_clone = case_dir / "fresh"
                    git(case_dir, "clone", "-q", str(repo), str(records_clone))
                    git(records_clone, "checkout", "-q", f"{integration_branch}-records")
                    rows = effect_history(records_clone, effort)
                    findings_02 = tracker_command(records_clone, effort, "findings", effort, "02", "--json", env=env)
                    observed.update({
                        "record_tip_after_bookkeeping": record_tip,
                        "candidate_parent": git(repo, "rev-parse", f"{candidate}^").strip(),
                        "integration_after": git(repo, "rev-parse", integration_branch).strip(),
                        "record_branch_after": git(repo, "rev-parse", f"{integration_branch}-records").strip(),
                        "land_returncode": land_proc.returncode,
                        "land_stdout": land_proc.stdout.strip(),
                        "land_stderr": land_proc.stderr.strip(),
                        "gate_seen": gate_seen.read_text() if gate_seen.exists() else None,
                        "claim_committed": "Status: claimed" in (records / f"docs/efforts/{effort}/tickets/02-second.md").read_text(),
                        "resolve_committed": "Status: resolved" in (records / f"docs/efforts/{effort}/tickets/03-third.md").read_text(),
                        "resolve_forced_logged": "missing accepted, landed Candidate" in (records / f"docs/efforts/{effort}/tickets/03-third.md").read_text(),
                        "acceptance_committed": acceptance_record["record"]["candidate"] == candidate_04,
                        "landing_status": status_row.get("landed_candidates"),
                        "accepted_candidate": acceptance_01["record"]["candidate"],
                        "fresh_history": rows,
                        "findings_02_returncode": findings_02.returncode,
                        "findings_02": json.loads(findings_02.stdout) if findings_02.returncode == 0 else None,
                        "findings_02_stderr": findings_02.stderr.strip(),
                    })
                    if observed["candidate_parent"] != code_tip:
                        details.append("candidate-base")
                    if land_proc.returncode != 0 or observed["land_stdout"] != candidate or observed["integration_after"] != candidate:
                        details.append("landing")
                    if not is_ancestor(repo, record_tip, observed["record_branch_after"]) or observed["gate_seen"] != candidate:
                        details.append("record-or-gate")
                    if not (
                        observed["claim_committed"]
                        and observed["resolve_committed"]
                        and observed["resolve_forced_logged"]
                        and observed["acceptance_committed"]
                    ):
                        details.append("bookkeeping")
                    if status_row.get("landed_candidates") != [{"ticket": "01", "candidate": candidate}]:
                        details.append("landed-candidates")
                    if not history_has_bookkeeping(rows, candidate_04):
                        details.append("fresh-history")
                    if (
                        findings_02.returncode != 0
                        or not observed["findings_02"]
                        or observed["findings_02"][0]["text"] != "Imported report finding."
                        or observed["findings_02"][0]["sources"][0]["label"] != "report 01 round 1"
                    ):
                        details.append("fresh-report-findings")
        else:
            raise LeadDrivenError(f"unknown integration branch action {action!r}")
    finally:
        stop_launch(launched)
    return {
        "scenario": scenario.id,
        "case": case["id"],
        "stories": case_story_list(scenario, case),
        "scenario_result": "fails" if details else "passes",
        "failure_details": details,
        "expected": {key: value for key, value in case.items() if key.startswith("expect_")},
        "observed": observed,
        "repo": str(repo),
    }


def effect_history(repo: Path, effort: str) -> dict:
    tickets = repo / "docs" / "efforts" / effort / "tickets"
    evidence = repo / "docs" / "efforts" / effort / "evidence"
    return {
        "ticket_02": (tickets / "02-second.md").read_text(),
        "ticket_03": (tickets / "03-third.md").read_text(),
        "acceptance_01": (evidence / "acceptance-01.jsonl").read_text(),
        "acceptance_04": (evidence / "acceptance-04.jsonl").read_text(),
        "report_01": (evidence / "01-report-round-1.md").read_text(),
        "landing_01": (evidence / "landing-01.jsonl").read_text(),
        "record_branch": git(repo, "branch", "--show-current").strip(),
    }


def history_has_bookkeeping(rows: dict, candidate: str) -> bool:
    return (
        rows["record_branch"].endswith("-records")
        and "Status: claimed" in rows["ticket_02"]
        and "Status: resolved" in rows["ticket_03"]
        and candidate in rows["acceptance_04"]
        and "Imported report finding." in rows["report_01"]
        and rows["landing_01"]
    )


def run_integration_branch_scenario(
    scenario: Scenario,
    out: Path,
    only: list[str] | None = None,
    *,
    runtime: str = "scripted",
    proof: str = "deterministic",
    interventions: list[dict] | None = None,
) -> list[dict]:
    cases = scenario.cases or []
    if only:
        known = {case["id"] for case in cases}
        unknown = set(only) - known
        if unknown:
            raise LeadDrivenError(f"unknown case(s): {', '.join(sorted(unknown))}")
        cases = [case for case in cases if case["id"] in only]
    else:
        cases = [case for case in cases if case.get("default", True)]
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    records = []
    start = time.monotonic()
    interventions = interventions or []
    for case in cases:
        case_dir = out / f"{stamp}-{scenario.id}-{case['id']}"
        record = run_integration_branch_case(scenario, case, case_dir, runtime)
        (case_dir / "result.json").write_text(json.dumps(record, indent=2) + "\n")
        records.append(record)
    append_scenario_row(
        out,
        scenario,
        records,
        runtime=runtime,
        proof=proof,
        interventions=interventions,
        elapsed_seconds=time.monotonic() - start,
    )
    return records


def parse_interventions(values: list[str]) -> list[dict]:
    interventions = []
    for value in values:
        if ":" not in value:
            raise LeadDrivenError("intervention must be WHO:WHAT")
        who, what = (part.strip() for part in value.split(":", 1))
        if not who or not what:
            raise LeadDrivenError("intervention must be WHO:WHAT")
        interventions.append({"who": who, "what": what})
    return interventions


def scenario_outcome(records: list[dict], interventions: list[dict]) -> str:
    if not records:
        return "fails"
    if any(record.get("scenario_result") != "passes" for record in records):
        return "fails"
    return "passes with intervention" if interventions else "passes"


def run_scenario(
    name: str,
    out: Path,
    only: list[str] | None = None,
    *,
    runtime: str = "scripted",
    proof: str = "deterministic",
    interventions: list[dict] | None = None,
) -> list[dict]:
    scenario = load_scenario(name)
    if scenario.kind == "integration-branch":
        return run_integration_branch_scenario(scenario, out, only, runtime=runtime, proof=proof, interventions=interventions)
    if scenario.kind == "lead-launch":
        return run_launch_scenario(scenario, out, only, runtime=runtime, proof=proof, interventions=interventions)
    if scenario.kind == "rework":
        return run_rework_scenario(scenario, out, only, runtime=runtime, proof=proof, interventions=interventions)
    if scenario.kind == "parallel-overlap":
        return run_parallel_overlap_scenario(scenario, out, only, runtime=runtime, proof=proof, interventions=interventions)
    participants = scenario.participants
    if only:
        known = {p.id for p in participants}
        unknown = set(only) - known
        if unknown:
            raise LeadDrivenError(f"unknown participant(s): {', '.join(sorted(unknown))}")
        participants = [p for p in participants if p.id in only]
    else:
        participants = [p for p in participants if p.default]
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    records = []
    start = time.monotonic()
    interventions = interventions or []
    for participant in participants:
        case_dir = out / f"{stamp}-{scenario.id}-{participant.id}"
        if scenario.kind == "shared-repository-safe":
            record = run_shared_repository_case(scenario, participant, case_dir, runtime)
        else:
            record = launch_and_supervise(scenario, participant, case_dir, runtime)
        result_path = case_dir / "result.json"
        result_path.write_text(json.dumps(record, indent=2) + "\n")
        records.append(record)
    append_scenario_row(
        out,
        scenario,
        records,
        runtime=runtime,
        proof=proof,
        interventions=interventions,
        elapsed_seconds=time.monotonic() - start,
    )
    return records


def json_lines(path: Path) -> list[dict]:
    if not path.exists():
        return []
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def latest_json_line(path: Path) -> dict:
    rows = json_lines(path)
    if not rows:
        raise LeadDrivenError(f"{path} has no JSON records")
    return rows[-1]


def rework_env(scenario: Scenario, case_dir: Path, runtime: str) -> dict:
    env = os.environ.copy()
    set_runtime_env(env, scenario, runtime)
    env["CORENIUS_RUNS"] = str(case_dir / "runs")
    env["CORENIUS_TRACKER_NOW"] = "2026-10-05T12:00:00Z"
    env["CORENIUS_HANDOFF_MARKER"] = "ticket 01: ready for review"
    return env


def init_rework_repo(scenario: Scenario, case: dict, case_dir: Path) -> dict:
    repo = case_dir / "repo"
    base = init_repo(scenario, repo)
    effort = (scenario.acceptance.effort if scenario.acceptance else None) or "readiness"
    integration_branch = load_effort_metadata(repo, effort).get("integration_branch") or f"effort/{effort}"
    layout = case.get("layout", "integration-worktree")
    integration = None
    if layout in {"integration-worktree", "main-checkout-effort-worktree"}:
        integration = case_dir / "integration"
        git(repo, "worktree", "add", "-q", str(integration), integration_branch)
        base = git(integration, "rev-parse", "HEAD").strip()
        if layout == "main-checkout-effort-worktree":
            git(repo, "rm", "-qr", f"docs/efforts/{effort}")
            git(repo, "commit", "-q", "-m", "main without effort record")
    elif layout != "single":
        raise LeadDrivenError(f"unknown rework layout {layout!r}")
    return {
        "repo": repo,
        "base": base,
        "layout": layout,
        "effort": effort,
        "integration_branch": integration_branch,
        "effort_worktree": integration or repo,
    }


def launch_rework_participant(scenario: Scenario, case: dict, case_dir: Path, env: dict, runtime: str) -> dict:
    raise RuntimeError('Retired imported benchmark execution. Use harp durable --help; only historical solo reference/noop replay remains supported.')


def acceptance_record_from_output(stdout: str) -> dict:
    printed = json.loads(stdout)
    path = Path(printed["recorded"])
    return {"record": latest_json_line(path), "record_path": str(path), "printed": printed}


def commit_record_for_output(repo: Path, output: dict, message: str) -> None:
    record_path = Path(output["record_path"])
    rel = record_path.resolve().relative_to(repo.resolve())
    git(repo, "add", str(rel))
    git(repo, "commit", "-q", "-m", message)


def write_rework_review(repo: Path, candidate: str, findings: Path) -> dict:
    proc = run_cmd(
        tool_cmd(
            "acceptance",
            "--repo",
            str(repo),
            "review",
            "readiness",
            "01",
            "--candidate",
            candidate,
            "--reviewer",
            "lead",
            "--reviewer-context",
            "lead",
            "--standards",
            "passed",
            "--spec",
            "failed",
            "--findings-file",
            str(findings),
        )
    )
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    output = acceptance_record_from_output(proc.stdout)
    commit_record_for_output(repo, output, "record review")
    return output


def write_rework_acceptance(repo: Path, candidate: str) -> dict:
    proc = run_cmd(
        tool_cmd(
            "acceptance",
            "--repo",
            str(repo),
            "record",
            "readiness",
            "01",
            "--candidate",
            candidate,
            "--reviewer",
            "lead",
            "--reviewer-context",
            "lead",
            "--standards",
            "passed",
            "--spec",
            "passed",
            "--required-check",
            "unit",
            "--check",
            "unit=passed",
        )
    )
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    output = acceptance_record_from_output(proc.stdout)
    commit_record_for_output(repo, output, "record acceptance")
    return output


def request_rework(repo: Path, candidate: str, kind: str, findings: Path) -> subprocess.CompletedProcess:
    return run_cmd(
        tool_cmd(
            "acceptance",
            "--repo",
            str(repo),
            "request-rework",
            "readiness",
            "01",
            "--candidate",
            candidate,
            "--kind",
            kind,
            "--findings-file",
            str(findings),
            "--actor",
            "lead",
        )
    )


def run_rework_delivery(repo: Path, run_dir: str, findings: Path, env: dict) -> dict:
    proc = run_cmd(
        tool_cmd("lead_launch", "--repo", str(repo), "rework", run_dir, "--findings-file", str(findings), env=env),
        env=env,
    )
    if proc.returncode != 0:
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return json.loads(proc.stdout)


def supervise_rework(run_dir: str, env: dict) -> dict:
    proc = run_cmd(tool_cmd("worker", "supervise", run_dir, "--interval", "0.1", "--max-wait", "20", env=env), env=env)
    if proc.returncode not in (worker_tool.CLEAN, worker_tool.DEGRADED, worker_tool.VIOLATED):
        raise LeadDrivenError(proc.stderr.strip() or proc.stdout.strip())
    return json.loads(proc.stdout.strip().splitlines()[-1])


def land_candidate(repo: Path, candidate: str, gate_seen: Path, target_branch: str) -> dict:
    gate = f'test "$LAND_TIP" = "$(git rev-parse HEAD)" && printf %s "$LAND_TIP" > {shlex.quote(str(gate_seen))}'
    proc = run_cmd(
        tool_cmd(
            "land",
            candidate,
            "--repo",
            str(repo),
            "--no-push",
            "--effort",
            "readiness",
            "--ticket",
            "01",
            "--gate",
            gate,
        )
    )
    target = git(repo, "rev-parse", target_branch, check=False).strip()
    observed = "other"
    if proc.returncode == 0 and target == candidate and gate_seen.read_text() == candidate:
        observed = "landed"
    elif proc.returncode == ACCEPTANCE_FAILED and not gate_seen.exists():
        observed = "acceptance-refused"
    return {
        "return_code": proc.returncode,
        "stdout": proc.stdout.strip(),
        "stderr": proc.stderr.strip(),
        "target_branch": target_branch,
        "target_after": target,
        "gate_seen": gate_seen.read_text() if gate_seen.exists() else None,
        "observed": observed,
    }


def run_rework_round_case(scenario: Scenario, case: dict, case_dir: Path, runtime: str) -> dict:
    env = rework_env(scenario, case_dir, runtime)
    setup = launch_rework_participant(scenario, case, case_dir, env, runtime)
    repo = setup["repo"]
    effort_worktree = setup["effort_worktree"]
    effort = setup["effort"]
    records = add_record_branch_worktree(repo, effort_worktree, effort, f"{setup['integration_branch']}-records", case_dir / "records")
    effort_worktree = records
    run_dir = setup["launched"]["run_dir"]
    first = setup["initial_verdict"]["candidate"]["commit"]
    findings = case_dir / "review-1.md"
    findings.write_text("Fix result.txt so it contains ready after rework.\n")
    first_review = write_rework_review(effort_worktree, first, findings)
    if case["id"] == "second-review-required":
        write_rework_acceptance(effort_worktree, first)
    delivery = run_rework_delivery(repo, run_dir, findings, env)
    second_verdict = supervise_rework(run_dir, env)
    second = second_verdict["candidate"]["commit"] if second_verdict.get("candidate") else None
    second_acceptance = None
    if case["id"] == "one-round-accepted" and second:
        second_acceptance = write_rework_acceptance(effort_worktree, second)
    landing = land_candidate(repo, second or "", case_dir / "land-gate-tip", setup["integration_branch"])
    records = json.loads((Path(run_dir) / "run.json").read_text())
    candidates = [
        {"candidate": first, "review": first_review["record"], "review_record_path": first_review["record_path"]},
        {
            "candidate": second,
            "review": second_acceptance["record"] if second_acceptance else None,
            "review_record_path": second_acceptance["record_path"] if second_acceptance else None,
        },
    ]
    expected_landing = case.get("expect_landing")
    details = []
    if not second or len(records.get("candidates", [])) != 2:
        details.append("candidates")
    if first_review["record"].get("accepted") is not False:
        details.append("first-review")
    if delivery.get("mode") != "resumed":
        details.append("delivery")
    if landing["observed"] != expected_landing:
        details.append("landing")
    if case.get("layout") == "main-checkout-effort-worktree":
        if (repo / "docs" / "efforts" / effort / "EFFORT.md").exists():
            details.append("main-checkout-effort-record")
        if not Path(first_review["record_path"]).is_relative_to(effort_worktree):
            details.append("review-record-worktree")
        if second_acceptance and not Path(second_acceptance["record_path"]).is_relative_to(effort_worktree):
            details.append("acceptance-record-worktree")
    return {
        "scenario": scenario.id,
        "case": case["id"],
        "stories": case_story_list(scenario, case),
        "scenario_result": "fails" if details else "passes",
        "failure_details": details,
        "observed": {
            "layout": setup["layout"],
            "command_repo": str(repo.resolve()),
            "effort_worktree": str(effort_worktree.resolve()),
            "effort_record_exists_in_command_repo": (repo / "docs" / "efforts" / effort / "EFFORT.md").exists(),
            "delivery": delivery["delivery"],
            "candidates": candidates,
            "run_candidates": records.get("candidates", []),
            "landing": landing,
        },
        "repo": str(repo),
    }


def run_rework_cap_case(scenario: Scenario, case: dict, case_dir: Path) -> dict:
    repo = case_dir / "repo"
    init_repo(scenario, repo)
    candidate = git(repo, "rev-parse", "HEAD").strip()
    findings = case_dir / "review.md"
    findings.write_text("Needs more work.\n")
    details = []
    for kind in ("review", "stale-base", "review", "review"):
        proc = request_rework(repo, candidate, kind, findings)
        if proc.returncode != 0:
            details.append(f"request-{kind}")
    fourth = request_rework(repo, candidate, "review", findings)
    if fourth.returncode != 64:
        details.append("fourth-return-code")
    if "review-driven rework cap exhausted" not in fourth.stderr:
        details.append("fourth-stderr")
    records = json_lines(repo / "docs" / "efforts" / "readiness" / "evidence" / "rework-01.jsonl")
    ticket = ticket_file(repo, "readiness", "01").read_text()
    triage = "ready-for-human" if "Triage: ready-for-human" in ticket else "other"
    if [record["round"] for record in records] != [1, 1, 2, 3]:
        details.append("rounds")
    if [record["counts_toward_cap"] for record in records] != [True, False, True, True]:
        details.append("stale-redo")
    if triage != "ready-for-human":
        details.append("triage")
    return {
        "scenario": scenario.id,
        "case": case["id"],
        "stories": case_story_list(scenario, case),
        "scenario_result": "fails" if details else "passes",
        "failure_details": details,
        "observed": {
            "rounds": [record["round"] for record in records],
            "counts_toward_cap": [record["counts_toward_cap"] for record in records],
            "triage": triage,
            "fourth_stderr": fourth.stderr.strip(),
        },
        "repo": str(repo),
    }


def run_rework_case(scenario: Scenario, case: dict, case_dir: Path, runtime: str = "scripted") -> dict:
    case_dir.mkdir(parents=True)
    if case["id"] == "round-cap-and-stale-redo":
        return run_rework_cap_case(scenario, case, case_dir)
    return run_rework_round_case(scenario, case, case_dir, runtime)


def run_rework_scenario(
    scenario: Scenario,
    out: Path,
    only: list[str] | None = None,
    *,
    runtime: str = "scripted",
    proof: str = "deterministic",
    interventions: list[dict] | None = None,
) -> list[dict]:
    cases = scenario.cases or []
    if only:
        known = {case["id"] for case in cases}
        unknown = set(only) - known
        if unknown:
            raise LeadDrivenError(f"unknown case(s): {', '.join(sorted(unknown))}")
        cases = [case for case in cases if case["id"] in only]
    else:
        cases = [case for case in cases if case.get("default", True)]
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    records = []
    start = time.monotonic()
    interventions = interventions or []
    for case in cases:
        case_dir = out / f"{stamp}-{scenario.id}-{case['id']}"
        record = run_rework_case(scenario, case, case_dir, runtime)
        (case_dir / "result.json").write_text(json.dumps(record, indent=2) + "\n")
        records.append(record)
    append_scenario_row(
        out,
        scenario,
        records,
        runtime=runtime,
        proof=proof,
        interventions=interventions,
        elapsed_seconds=time.monotonic() - start,
    )
    return records


def run_launch_scenario(
    scenario: Scenario,
    out: Path,
    only: list[str] | None = None,
    *,
    runtime: str = "scripted",
    proof: str = "deterministic",
    interventions: list[dict] | None = None,
) -> list[dict]:
    cases = scenario.cases or []
    if only:
        known = {case["id"] for case in cases}
        unknown = set(only) - known
        if unknown:
            raise LeadDrivenError(f"unknown case(s): {', '.join(sorted(unknown))}")
        cases = [case for case in cases if case["id"] in only]
    else:
        cases = [case for case in cases if case.get("default", True)]
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    records = []
    start = time.monotonic()
    interventions = interventions or []
    for case in cases:
        case_dir = out / f"{stamp}-{scenario.id}-{case['id']}"
        record = run_launch_case(scenario, case, case_dir, runtime)
        (case_dir / "result.json").write_text(json.dumps(record, indent=2) + "\n")
        records.append(record)
    append_scenario_row(
        out,
        scenario,
        records,
        runtime=runtime,
        proof=proof,
        interventions=interventions,
        elapsed_seconds=time.monotonic() - start,
    )
    return records


def append_running_note(notes_file: Path, kind: str, note: str) -> None:
    if kind not in NOTE_KINDS:
        raise ValueError(f"unknown note kind {kind!r}")
    text = " ".join(note.split())
    if not text:
        raise ValueError("note text must not be empty")
    if not notes_file.exists():
        notes_file.parent.mkdir(parents=True, exist_ok=True)
        notes_file.write_text(
            "# Running notes: getting the lead-driven instantiation working\n\n"
            "Kinds: obstacle, intervention, rule. Newest last.\n\n"
        )
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%MZ")
    with notes_file.open("a") as fh:
        fh.write(f"- {stamp} {kind}: {text}\n")


def main(argv: list[str] | None = None) -> int:
    raise RuntimeError('Retired imported benchmark execution. Use harp durable --help; only historical solo reference/noop replay remains supported.')


if __name__ == "__main__":
    raise SystemExit(main())
