#!/usr/bin/env python3
"""Launch a worker agent in its own private git workspace and supervise it deterministically.

The lead/supervisor/worker form factor (docs/practice/lead-supervisor-worker.md):

* ``launch``    create a private git workspace and branch, write the prompt file with the
                standard rules, start ``traecli exec`` detached, record a run.
* ``supervise`` block until the worker exits (or ``--max-wait`` elapses), check
                the boundary every cycle, and write ``verdict.json``.
                Exit 0 CLEAN, 1 DEGRADED, 2 VIOLATED, 3 still RUNNING.
* ``status``    one-shot check of a run without changing anything.

The checks read process and git state, never artifacts, so a supervisor cannot
mistake a finished output file for a finished process. Standard library only.
Set CORENIUS_TRAECLI or CORENIUS_CODEX to override the worker binary and
CORENIUS_RUNS to move the run directory root.
"""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import hashlib
import json
import os
import re
import shutil
import shlex
import stat
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_FORBIDDEN = (
    r"\bgit\s+(?:-C\s+\S+\s+)?(?:push|reset|checkout|switch|rebase|merge|stash|clean|am|apply|restore)\b",
)
NETWORK_COMMAND = r"\b(?:curl|wget)\b"
CONTAINER_BUILD_COMMAND = r"\b(?:docker|podman|buildah|nerdctl)\s+(?:build|buildx)\b"
SHARED_CACHE_COMMAND = r"\b(?:pip3?|uv\s+pip)\s+install\b"
EXTERNAL_DIGEST_LIMIT_BYTES = 1024 * 1024
GIT_ENV_VARS = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR")

# Older runs recorded this pattern in run.json. `rm` is now judged by rm_findings():
# its harm is visible in git and file state, so text alone is not a violation.
LEGACY_RM = r"\brm\s+-[A-Za-z]*[rf]"

SEGMENT_SPLIT = re.compile(r"\|\||&&|[;|\n]")
HOME_VARS = ("$HOME", "${HOME}", "~")

RUNNING, CLEAN, DEGRADED, VIOLATED = 3, 0, 1, 2
VERDICT_NAMES = {CLEAN: "CLEAN", DEGRADED: "DEGRADED", VIOLATED: "VIOLATED"}
RUNTIMES = ("traecli", "codex")
RUNTIME_OBSERVABILITY = {
    "traecli": {
        "observed": (
            "TraeCLI JSON command_execution events, session id when emitted, "
            "last-message file, runtime transcript path when the session store is visible, "
            "interactive transcript discovery from session_meta cwd, "
            "process exit, git branch and worktree state"
        ),
        "not_observed": "private reasoning and prompts after launch that are not in the JSON stream",
    },
    "codex": {
        "observed": (
            "Codex JSON CommandExecution and file_change events, thread_id when emitted, "
            "last-message file, runtime transcript path when the session store is visible, "
            "interactive transcript discovery from session_meta cwd when present, "
            "process exit, git branch and worktree state"
        ),
        "not_observed": (
            "private reasoning and Codex per-shell timeout enforcement; the worker wall-clock cap "
            "and quiet-time monitor still apply"
        ),
    },
}
RUNTIME_HELP = """Runtime observation:
  traecli observed: TraeCLI JSON command_execution events, session id when emitted, last-message file, runtime transcript path when CORENIUS_TRAECLI_TRANSCRIPT_DIR or the TraeCLI session store is visible, interactive transcript discovery from session_meta cwd, process exit, git branch and worktree state.
  traecli not observed: private reasoning and prompts after launch that are not in the JSON stream.
  codex observed: Codex JSON CommandExecution and file_change events, thread_id when emitted, last-message file, runtime transcript path when CORENIUS_CODEX_TRANSCRIPT_DIR or the Codex session store is visible, interactive transcript discovery from session_meta cwd when present, process exit, git branch and worktree state.
  codex not observed: private reasoning and Codex per-shell timeout enforcement; the worker wall-clock cap and quiet-time monitor still apply.
  interactive launches pass -c projects={"<worker workspace>"={trust_level="trusted"}} for traecli and codex so fresh private clones do not stop at the folder-trust prompt.
"""
ACCESS_PROFILE_SUPPORT = {
    "traecli": {
        "enforced": [
            "command_timeout_seconds",
            "wall_clock_cap",
            "capsule_network",
            "capsule_writable_roots",
        ],
        "declared": [
            "network",
            "external_add_paths",
            "container_builds",
            "shared_cache_writes",
        ],
        "detected": [
            "network-command",
            "external-state",
            "container-build",
            "shared-cache-write",
            "workspace-change",
        ],
    },
    "codex": {
        "enforced": [
            "wall_clock_cap",
            "capsule_network",
            "capsule_writable_roots",
        ],
        "declared": [
            "network",
            "command_timeout_seconds",
            "external_add_paths",
            "container_builds",
            "shared_cache_writes",
        ],
        "detected": [
            "network-command",
            "external-state",
            "container-build",
            "shared-cache-write",
            "workspace-change",
        ],
    },
}
ACCESS_PROFILE_HELP = """Access profile support:
  traecli access profile:
    enforced: command_timeout_seconds, wall_clock_cap; capsule mode also enforces network and writable roots.
    declared: network, external_add_paths, container_builds, shared_cache_writes.
    detected: network-command, external-state, container-build, shared-cache-write, workspace-change.
  codex access profile:
    enforced: wall_clock_cap; capsule mode also enforces network and writable roots.
    declared: network, command_timeout_seconds, external_add_paths, container_builds, shared_cache_writes.
    detected: network-command, external-state, container-build, shared-cache-write, workspace-change.
"""
WORKSPACE_ATTRIBUTION_HELP = """Workspace change attribution:
  unaccounted workspace changes are reported when an allowed dirty path is not named by any recorded worker command or file-change event.
  The supervisor cannot identify the writer process; this is event-record accounting, not kernel-level attribution.
"""


class WorkerError(Exception):
    pass


def runtime_observability(runtime: str) -> dict:
    return dict(RUNTIME_OBSERVABILITY[runtime])


def access_profile_support() -> dict:
    return {
        runtime: {kind: list(values) for kind, values in support.items()}
        for runtime, support in ACCESS_PROFILE_SUPPORT.items()
    }


def runtime_binary(runtime: str) -> str:
    if runtime == "codex":
        return os.environ.get("CORENIUS_CODEX", "codex")
    return os.environ.get("CORENIUS_TRAECLI", "traecli")


def command_text(value) -> str | None:
    if isinstance(value, str):
        return value
    if isinstance(value, list) and all(isinstance(item, str) for item in value):
        if len(value) >= 3 and value[-2] in ("-c", "-lc"):
            return value[-1]
        return shlex.join(value)
    return None


def output_text(value) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", "replace")
    return str(value)


def event_containers(event: dict) -> list[dict]:
    containers = [event]
    payload = event.get("payload")
    if isinstance(payload, dict):
        containers.append(payload)
    return containers


def event_session_id(event: dict, runtime: str) -> str | None:
    for container in event_containers(event):
        for key in ("thread_id", "session_id"):
            value = container.get(key)
            if isinstance(value, str) and value:
                return value
    if runtime == "traecli" and event.get("type") == "thread.started":
        value = event.get("id")
        if isinstance(value, str) and value:
            return value
    return None


def transcript_session_id(event: dict, runtime: str, path: Path) -> str | None:
    session_id = event_session_id(event, runtime)
    if session_id:
        return session_id
    payload = event.get("payload")
    if isinstance(payload, dict):
        value = payload.get("id")
        if isinstance(value, str) and value:
            return value
    match = re.match(r"rollout-\d{4}-\d\d-\d\dT\d\d-\d\d-\d\d-(.+)\.jsonl$", path.name)
    if match:
        return match.group(1)
    return None


def event_working_directory(event: dict) -> str | None:
    for container in event_containers(event):
        for key in ("cwd", "working_directory", "workdir"):
            value = container.get(key)
            if isinstance(value, str) and value:
                return value
    return None


def same_path(left: str | Path, right: str | Path) -> bool:
    try:
        return Path(left).expanduser().resolve() == Path(right).expanduser().resolve()
    except OSError:
        return os.path.abspath(os.path.expanduser(str(left))) == os.path.abspath(os.path.expanduser(str(right)))


def parse_event_epoch(event: dict) -> float | None:
    for container in event_containers(event):
        value = container.get("timestamp")
        if not isinstance(value, str) or not value:
            continue
        try:
            return datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp()
        except ValueError:
            continue
    return None


def event_usage(event: dict, runtime: str) -> dict | None:
    for container in event_containers(event):
        event_type = container.get("type")
        usage = container.get("usage")
        if event_type in ("turn.completed", "turn_completed") and isinstance(usage, dict):
            return usage
    return None


def event_command_items(event: dict, runtime: str) -> list[dict]:
    items = []
    for container in event_containers(event):
        item = container.get("item")
        if not isinstance(item, dict) or item.get("type") not in ("command_execution", "CommandExecution"):
            continue
        command = command_text(item.get("command"))
        if not command:
            continue
        item_id = item.get("id")
        items.append({
            "id": item_id if isinstance(item_id, str) and item_id else None,
            "command": command,
            "event_type": container.get("type"),
            "status": item.get("status"),
        })
    return items


def normalize_event_path(raw: str, worktree: Path) -> str | None:
    worktree = Path(os.path.abspath(worktree))
    path = Path(raw)
    if path.is_absolute():
        absolute = Path(os.path.abspath(raw))
        try:
            return absolute.relative_to(worktree).as_posix()
        except ValueError:
            return None
    normalized = Path(os.path.normpath(raw))
    if normalized == Path(".") or normalized == Path("..") or str(normalized).startswith(".." + os.sep):
        return None
    return normalized.as_posix()


def event_file_change_items(event: dict, runtime: str, worktree: Path) -> list[dict]:
    items = []
    for container in event_containers(event):
        item = container.get("item")
        if not isinstance(item, dict) or item.get("type") != "file_change":
            continue
        paths = []
        changes = item.get("changes")
        if isinstance(changes, list):
            for change in changes:
                if not isinstance(change, dict) or not isinstance(change.get("path"), str):
                    continue
                path = normalize_event_path(change["path"], worktree)
                if path:
                    paths.append(path)
        elif isinstance(item.get("path"), str):
            path = normalize_event_path(item["path"], worktree)
            if path:
                paths.append(path)
        if not paths:
            continue
        item_id = item.get("id")
        items.append({
            "id": item_id if isinstance(item_id, str) and item_id else None,
            "paths": sorted(set(paths)),
            "event_type": container.get("type"),
            "status": item.get("status"),
        })
    return items


def command_item_completed(command_item: dict) -> bool:
    return command_item["event_type"] in ("item.completed", "item_completed") or command_item["status"] == "completed"


def command_item_started(command_item: dict) -> bool:
    return command_item["event_type"] in ("item.started", "item_started") or command_item["status"] == "in_progress"


def event_commands(event: dict, runtime: str) -> list[str]:
    return [item["command"] for item in event_command_items(event, runtime) if command_item_completed(item)]


class RuntimeEventAdapter:
    def __init__(self, runtime: str):
        self.runtime = runtime
        self.pending: dict[str, str] = {}
        self.emitted: set[str] = set()
        self.pending_file_changes: dict[str, list[str]] = {}
        self.emitted_file_changes: set[str] = set()

    def feed(self, event: dict) -> list[str]:
        commands = []
        for item in event_command_items(event, self.runtime):
            key = item["id"] or f"command:{item['command']}"
            if command_item_completed(item):
                self.pending.pop(key, None)
                if key not in self.emitted:
                    commands.append(item["command"])
                    self.emitted.add(key)
            elif command_item_started(item) and key not in self.emitted:
                self.pending[key] = item["command"]
        return commands

    def feed_file_changes(self, event: dict, worktree: Path) -> list[str]:
        paths = []
        for item in event_file_change_items(event, self.runtime, worktree):
            key = item["id"] or f"file_change:{','.join(item['paths'])}"
            if command_item_completed(item):
                self.pending_file_changes.pop(key, None)
                if key not in self.emitted_file_changes:
                    paths.extend(item["paths"])
                    self.emitted_file_changes.add(key)
            elif command_item_started(item) and key not in self.emitted_file_changes:
                paths.extend(item["paths"])
                self.emitted_file_changes.add(key)
        return paths

    def flush_pending(self) -> list[str]:
        commands = []
        for key, command in list(self.pending.items()):
            if key not in self.emitted:
                commands.append(command)
                self.emitted.add(key)
        self.pending.clear()
        return commands

    def flush_pending_file_changes(self) -> list[str]:
        paths = []
        for key, pending_paths in list(self.pending_file_changes.items()):
            if key not in self.emitted_file_changes:
                paths.extend(pending_paths)
                self.emitted_file_changes.add(key)
        self.pending_file_changes.clear()
        return paths


def event_stream_commands(events: list[dict], runtime: str) -> list[str]:
    adapter = RuntimeEventAdapter(runtime)
    commands = []
    for event in events:
        commands.extend(adapter.feed(event))
    commands.extend(adapter.flush_pending())
    return commands


def event_stream_file_change_paths(events: list[dict], runtime: str, worktree: Path) -> list[str]:
    adapter = RuntimeEventAdapter(runtime)
    paths = []
    for event in events:
        paths.extend(adapter.feed_file_changes(event, worktree))
    paths.extend(adapter.flush_pending_file_changes())
    return paths


def update_session_id_from_events(run: dict, *, persist: bool = True) -> str | None:
    if run.get("session_id"):
        update_runtime_transcript(run, run["session_id"])
        if persist:
            save_run(run)
        return run["session_id"]
    events = Path(run["run_dir"]) / "events.jsonl"
    if not events.exists():
        return None
    runtime = run.get("runtime", "traecli")
    for line in events.read_text(errors="replace").splitlines():
        try:
            event = json.loads(line)
        except ValueError:
            continue
        session_id = event_session_id(event, runtime)
        if session_id:
            run["session_id"] = session_id
            update_runtime_transcript(run, session_id)
            if persist:
                save_run(run)
            return session_id
    if run.get("interactive"):
        session_id = update_runtime_transcript_from_workspace(run)
        if session_id:
            if persist:
                save_run(run)
            return session_id
    return None


def transcript_roots(runtime: str) -> list[Path]:
    roots: list[Path] = []
    env_name = f"CORENIUS_{runtime.upper()}_TRANSCRIPT_DIR"
    if os.environ.get(env_name):
        roots.append(Path(os.environ[env_name]).expanduser())
    if runtime == "traecli":
        home = Path(os.environ.get("TRAECLI_HOME", str(Path.home() / ".trae" / "cli"))).expanduser()
        roots.append(home / "sessions")
    elif runtime == "codex":
        home = Path(os.environ.get("CODEX_HOME", str(Path.home() / ".codex"))).expanduser()
        roots.extend([home / "sessions", home / "archived_sessions"])
    unique = []
    seen = set()
    for root in roots:
        resolved = str(root)
        if resolved not in seen:
            unique.append(root)
            seen.add(resolved)
    return unique


def runtime_trust_config(runtime: str, worktree: Path) -> dict:
    if runtime not in RUNTIMES:
        raise WorkerError(f"runtime {runtime!r} is not supported")
    workspace = str(worktree.expanduser().resolve())
    escaped = workspace.replace("\\", "\\\\").replace('"', '\\"')
    return {
        "runtime": runtime,
        "method": "cli-config",
        "workspace": workspace,
        "config": f'projects={{"{escaped}"={{trust_level="trusted"}}}}',
    }


def find_runtime_transcript(runtime: str, session_id: str) -> Path | None:
    names = (f"{session_id}.jsonl", f"{session_id}.json", session_id)
    for root in transcript_roots(runtime):
        if not root.exists():
            continue
        for name in names:
            candidate = root / name
            if candidate.is_file():
                return candidate
        try:
            matches = sorted(
                (path for path in root.rglob(f"*{session_id}*") if path.is_file()),
                key=lambda path: path.stat().st_mtime,
                reverse=True,
            )
        except OSError:
            matches = []
        if matches:
            return matches[0]
    return None


def transcript_first_record(path: Path) -> dict | None:
    try:
        with path.open("r", encoding="utf-8", errors="replace") as handle:
            for line in handle:
                if not line.strip():
                    continue
                try:
                    record = json.loads(line)
                except ValueError:
                    return None
                return record if isinstance(record, dict) else None
    except OSError:
        return None
    return None


def find_runtime_transcript_for_worktree(runtime: str, worktree: str | Path, started_epoch: float) -> tuple[str, Path] | None:
    matches: list[tuple[float, float, str, Path]] = []
    for root in transcript_roots(runtime):
        if not root.exists():
            continue
        candidates: list[Path] = []
        for pattern in ("*.jsonl", "*.json"):
            try:
                candidates.extend(path for path in root.rglob(pattern) if path.is_file())
            except OSError:
                continue
        for path in candidates:
            try:
                stat_result = path.stat()
            except OSError:
                continue
            first = transcript_first_record(path)
            if not first:
                continue
            cwd = event_working_directory(first)
            if not cwd or not same_path(cwd, worktree):
                continue
            session_epoch = parse_event_epoch(first) or stat_result.st_mtime
            if started_epoch and session_epoch < started_epoch:
                continue
            session_id = transcript_session_id(first, runtime, path)
            if session_id:
                matches.append((session_epoch, stat_result.st_mtime, session_id, path))
    if not matches:
        return None
    _, _, session_id, path = max(matches, key=lambda item: (item[0], item[1], str(item[3])))
    return session_id, path


def record_runtime_transcript(run: dict, session_id: str, path: Path | None) -> None:
    runtime = run.get("runtime", "traecli")
    run["runtime_transcript"] = {
        "runtime": runtime,
        "session_id": session_id,
        "path": str(path) if path else None,
        "observed": runtime_observability(runtime)["observed"],
    }
    if path is None:
        run["runtime_transcript"]["reason"] = "session transcript file not found in configured runtime stores"


def update_runtime_transcript(run: dict, session_id: str) -> None:
    current = run.get("runtime_transcript") or {}
    if current.get("session_id") == session_id and current.get("path"):
        return
    path = find_runtime_transcript(run.get("runtime", "traecli"), session_id)
    record_runtime_transcript(run, session_id, path)


def update_runtime_transcript_from_workspace(run: dict) -> str | None:
    current = run.get("runtime_transcript") or {}
    if current.get("session_id") and current.get("path"):
        run["session_id"] = current["session_id"]
        return current["session_id"]
    found = find_runtime_transcript_for_worktree(
        run.get("runtime", "traecli"),
        run["worktree"],
        float(run.get("started_epoch") or 0),
    )
    if not found:
        return None
    session_id, path = found
    run["session_id"] = session_id
    record_runtime_transcript(run, session_id, path)
    return session_id


def rm_findings(command: str, worktree: str) -> list[tuple[str, str]]:
    """Classify recursive/forced `rm` in a command: (severity, kind) per rm segment.

    VIOLATION when a target leaves the worktree (outside absolute path, `..`, home, `/`),
    NOTE otherwise. Deleting tracked files or touching other paths is caught by the
    git and file-state checks, not here.
    """
    found = []
    root = str(Path(worktree).resolve())
    for segment in SEGMENT_SPLIT.split(command):
        try:
            words = shlex.split(segment)
        except ValueError:
            words = segment.split()
        words = [w.strip("'\"`(){}") for w in words]
        if "rm" not in words:
            continue
        args = words[words.index("rm") + 1 :]
        flags = [a for a in args if a.startswith("-") and not a.startswith("--")]
        if not any(re.search(r"[rf]", f) for f in flags) and "--recursive" not in args and "--force" not in args:
            continue
        targets = [a for a in args if not a.startswith("-")]
        escapes = False
        for target in targets:
            if target.startswith(HOME_VARS) or target in ("/", "/*", "*/", ".."):
                escapes = True
            elif ".." in Path(target).parts:
                escapes = True
            elif target.startswith("/"):
                resolved = str(Path(target).resolve())
                escapes = escapes or not (resolved == root or resolved.startswith(root + "/"))
        found.append(("VIOLATION", "rm-outside-worktree") if escapes else ("NOTE", "rm-recursive"))
    return found


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def git(repo: str | Path, *args: str, check: bool = True) -> str:
    result = git_proc(repo, *args)
    if check and result.returncode != 0:
        raise WorkerError(f"git {' '.join(args)} failed in {repo}: {result.stderr.strip()}")
    return result.stdout


def git_env(env: dict[str, str] | None = None) -> dict[str, str]:
    cleaned = dict(os.environ if env is None else env)
    for name in GIT_ENV_VARS:
        cleaned.pop(name, None)
    cleaned["GIT_OPTIONAL_LOCKS"] = "0"
    return cleaned


def git_proc(repo: str | Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True, env=git_env())


def local_branch_heads(repo: str | Path) -> dict[str, str]:
    lines = git(repo, "for-each-ref", "--format=%(refname:short) %(objectname)", "refs/heads", check=False)
    heads = {}
    for line in lines.splitlines():
        if " " in line:
            name, sha = line.split(" ", 1)
            heads[name] = sha
    return heads


def ref_commit(repo: str | Path, ref: str) -> str | None:
    value = git(repo, "rev-parse", "--verify", "-q", f"{ref}^{{commit}}", check=False).strip()
    return value or None


def main_worktree(repo: str | Path) -> Path:
    common = git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").strip()
    return Path(common).parent


def runs_root() -> Path:
    override = os.environ.get("CORENIUS_RUNS")
    if override:
        return Path(override)
    cache = os.environ.get("XDG_CACHE_HOME") or str(Path.home() / ".cache")
    return Path(cache) / "corenius"


def temp_parent() -> Path:
    override = os.environ.get("TMPDIR")
    policy = Path(__file__).resolve().parents[1] / "temp_root.py"
    import runpy
    return runpy.run_path(str(policy))["temporary_root"](
        override or str(runs_root() / "tmp")
    )


def temporary_directory(prefix: str):
    return tempfile.TemporaryDirectory(prefix=prefix, dir=str(temp_parent()))


def new_run_temp(run_dir: Path, *, keep: bool = False) -> dict:
    root = run_dir / "tmp"
    tmux = root / "tmux"
    root.mkdir(parents=True, exist_ok=True)
    tmux.mkdir(parents=True, exist_ok=True)
    return {
        "tmpdir": str(root),
        "tmux_tmpdir": str(tmux),
        "tmpprefix": str(root / "zsh"),
        "keep": keep,
        "cleanup": "remove-when-stopped" if not keep else "keep",
    }


def ensure_run_temp(run: dict) -> dict:
    temp = dict(run.get("temp") or new_run_temp(Path(run["run_dir"])))
    root = Path(temp["tmpdir"])
    tmux = Path(temp.get("tmux_tmpdir") or root / "tmux")
    root.mkdir(parents=True, exist_ok=True)
    tmux.mkdir(parents=True, exist_ok=True)
    temp["tmpdir"] = str(root)
    temp["tmux_tmpdir"] = str(tmux)
    temp["tmpprefix"] = str(temp.get("tmpprefix") or root / "zsh")
    temp.pop("removed_at", None)
    run["temp"] = temp
    return temp


def runtime_temp_env(temp: dict) -> dict[str, str]:
    return {
        "TMPDIR": temp["tmpdir"],
        "TMUX_TMPDIR": temp["tmux_tmpdir"],
        "TMPPREFIX": temp["tmpprefix"],
    }


def run_environment(temp: dict, *, env: dict[str, str] | None = None) -> dict[str, str]:
    cleaned = git_env(env)
    cleaned.update(runtime_temp_env(temp))
    return cleaned


def cleanup_run_temp(run: dict) -> str | None:
    temp = run.get("temp") or {}
    if temp.get("keep"):
        return None
    raw = temp.get("tmpdir")
    if not raw:
        return None
    root = Path(os.path.abspath(raw))
    run_dir = Path(os.path.abspath(run["run_dir"]))
    if root != run_dir and path_inside(root, run_dir) and root.exists():
        shutil.rmtree(root, ignore_errors=True)
        temp["removed_at"] = now()
        run["temp"] = temp
        save_run(run)
        return str(root)
    return None


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    # A zombie still answers kill(0); the wrapper writes exit_code when done.
    return True


def live_session_pids(session_id: int | None) -> list[int]:
    if session_id is None:
        return []
    proc = Path("/proc")
    if not proc.is_dir():
        return []
    pids: list[int] = []
    for child in proc.iterdir():
        if not child.name.isdigit():
            continue
        try:
            raw = (child / "stat").read_text()
        except OSError:
            continue
        end = raw.rfind(")")
        if end == -1:
            continue
        fields = raw[end + 2 :].split()
        if len(fields) < 4:
            continue
        try:
            state = fields[0]
            sid = int(fields[3])
        except (ValueError, IndexError):
            continue
        if sid == session_id and state != "Z":
            pids.append(int(child.name))
    return sorted(pids)


def process_identity(process: subprocess.Popen) -> dict:
    identity = {"pid": process.pid}
    try:
        identity["process_group_id"] = os.getpgid(process.pid)
    except ProcessLookupError:
        identity["process_group_id"] = None
    try:
        identity["process_session_id"] = os.getsid(process.pid)
    except ProcessLookupError:
        identity["process_session_id"] = None
    return identity


# --- launch -------------------------------------------------------------------


HEADER = """You are a worker agent, one of several roles in a lead/supervisor/worker setup.
Your working root is the private git workspace {worktree} (branch {branch}). Read its
AGENTS.md first and follow it.

Hard rules for this run:
- You may create or edit only these paths, relative to the worktree: {allowed}.
  Any other change is a violation.
- Commit your finished work on branch {branch}. Do not push, merge, rebase, reset,
  stash, switch branches, rewrite history below the recorded base, or move any
  branch other than {branch}.
- Hand-off rule: {handoff_rule}
{protected_rule}
- Access profile:
{access_rules}
- Use only this run's declared temp directory: {temp_dir}.
- Use a fresh unique directory (`mktemp -d` under an allowed path) for each experiment.
  Deleting outside the worktree, or with `..` or `~`, is a violation.
- Do not print secrets. If a rule blocks the task, stop and say so in your final message.
- Finish with a short final message: what you produced, what you did not check.

TASK
"""


def sandbox_binds(worktree: Path, allowed: list[str]) -> tuple[list[tuple[Path, str]], list[Path]]:
    """Writable binds for `--sandbox insula`: the worktree is mounted read-only and only the allowed paths are
    bound writable. bwrap can bind only what exists, so a missing allowed path is created first: a directory when
    the entry ends with '/', otherwise an empty file (with its missing parents). Returns (binds, created); the
    created paths are removed again after the run if the worker left them untouched."""
    binds: list[tuple[Path, str]] = []
    created: list[Path] = []
    validate_allowed_paths(allowed)
    for entry in allowed:
        relative = entry.rstrip("/")
        host = worktree / relative
        if not host.exists():
            for parent in reversed(host.parents):
                if worktree in parent.parents or parent == worktree:
                    if not parent.exists():
                        parent.mkdir()
                        created.append(parent)
            if entry.endswith("/"):
                host.mkdir()
            else:
                host.touch()
            created.append(host)
        binds.append((host, f"/workspace/{relative}"))
    return binds, created


def remove_untouched(created: list[str | Path]) -> list[str]:
    """Remove pre-created empty files and empty directories (deepest first); returns what was removed."""
    removed = []
    for path in sorted((Path(p) for p in created), key=lambda p: len(p.parts), reverse=True):
        try:
            if path.is_file() and path.stat().st_size == 0:
                path.unlink()
                removed.append(str(path))
            elif path.is_dir() and not any(path.iterdir()):
                path.rmdir()
                removed.append(str(path))
        except OSError:
            continue
    return removed


def cross_device_clone_error(result: subprocess.CompletedProcess) -> bool:
    text = (result.stderr or "") + (result.stdout or "")
    return "Invalid cross-device link" in text or "cross-device link" in text


def clone_command(repo: Path, worktree: Path, git_dir: Path | None, object_mode: str) -> list[str]:
    cmd = ["git", "clone", "-q"]
    if object_mode == "local-hardlink":
        cmd.append("--local")
    elif object_mode == "copy":
        cmd.append("--no-hardlinks")
    else:
        raise WorkerError(f"unknown clone object mode {object_mode!r}")
    cmd.append("--no-checkout")
    if git_dir is not None:
        cmd += ["--separate-git-dir", str(git_dir)]
    cmd += [str(repo), str(worktree)]
    return cmd


def create_private_worker_clone(
    repo: Path,
    worktree: Path,
    branch: str,
    base: str,
    git_dir: Path | None = None,
    *,
    base_repo: Path | None = None,
    base_sha: str | None = None,
) -> dict:
    if ref_commit(repo, f"refs/heads/{branch}"):
        raise WorkerError(f"branch {branch} already exists")
    base_source = base_repo or repo
    base_sha = base_sha or ref_commit(base_source, base)
    if base_sha is None:
        raise WorkerError(f"ref {base!r} not found")
    if git_dir is not None:
        git_dir.parent.mkdir(parents=True, exist_ok=True)
    try:
        result = subprocess.run(clone_command(repo, worktree, git_dir, "local-hardlink"), capture_output=True, text=True, env=git_env())
        object_mode = "local-hardlink"
        if result.returncode != 0 and cross_device_clone_error(result):
            if worktree.exists():
                shutil.rmtree(worktree, ignore_errors=True)
            if git_dir is not None and git_dir.exists():
                shutil.rmtree(git_dir, ignore_errors=True)
            result = subprocess.run(clone_command(repo, worktree, git_dir, "copy"), capture_output=True, text=True, env=git_env())
            object_mode = "copy"
        if result.returncode != 0:
            raise WorkerError(f"git clone failed: {result.stderr.strip() or result.stdout.strip()}")
        git(worktree, "checkout", "-q", "-b", branch, base_sha)
        git(worktree, "remote", "remove", "origin", check=False)
        return {
            "base_sha": base_sha,
            "git_dir": str(git_dir if git_dir is not None else worktree / ".git"),
            "git_dir_layout": "separate" if git_dir is not None else "embedded",
            "clone_object_mode": object_mode,
        }
    except Exception:
        if worktree.exists():
            shutil.rmtree(worktree, ignore_errors=True)
        if git_dir is not None and git_dir.exists():
            shutil.rmtree(git_dir, ignore_errors=True)
        raise


def shared_repo(run: dict) -> Path | None:
    path = run.get("shared_repo")
    return Path(path) if path else None


def import_candidate_to_shared_repo(run: dict, candidate: dict) -> None:
    repo = shared_repo(run)
    if repo is None:
        return
    commit = candidate.get("commit")
    branch = run["branch"]
    if not commit:
        return
    git(repo, "fetch", "-q", "--no-tags", str(Path(run["worktree"])), commit)
    ref = f"refs/heads/{branch}"
    current = ref_commit(repo, ref)
    expected = current or "0" * 40
    git(repo, "update-ref", ref, commit, expected)
    run.setdefault("published_candidates", []).append(
        {"branch": branch, "commit": commit, "published_at": now()}
    )


def refresh_integration_branch(run: dict) -> dict | None:
    repo = shared_repo(run)
    integration = run.get("lead_launch", {}).get("integration")
    if repo is None or not integration:
        return None
    head = ref_commit(repo, f"refs/heads/{integration}")
    if not head:
        return None
    worktree = Path(run["worktree"])
    current = git(worktree, "branch", "--show-current", check=False).strip()
    if current != integration:
        ref = f"refs/heads/{integration}"
        git(worktree, "fetch", "-q", "--no-tags", str(repo), f"{ref}:{ref}")
    run.setdefault("branch_baselines", {})[integration] = head
    run.setdefault("shared_branch_baselines", {})[integration] = head
    return {"branch": integration, "commit": head}


def launch_access_profile(args: argparse.Namespace) -> dict:
    return {
        "network": args.network,
        "command_timeout_seconds": int(args.command_timeout),
        "external_add_paths": [str(Path(p).resolve()) for p in (args.external_add_path or [])],
        "container_builds": bool(args.allow_container_builds),
        "shared_cache_writes": bool(args.allow_shared_cache_writes),
    }


def access_profile(run: dict) -> dict:
    if "access_profile" in run:
        profile = dict(run["access_profile"])
        profile.setdefault("network", "on")
        profile.setdefault("command_timeout_seconds", 120)
        profile.setdefault("external_add_paths", [])
        profile.setdefault("container_builds", False)
        profile.setdefault("shared_cache_writes", False)
        return profile
    legacy_forbidden = set(run.get("forbidden", []))
    return {
        "network": "off" if NETWORK_COMMAND in legacy_forbidden else "on",
        "command_timeout_seconds": 120,
        "external_add_paths": [],
        "container_builds": False,
        "shared_cache_writes": False,
    }


def validate_allowed_paths(allowed: list[str]) -> None:
    for entry in allowed:
        relative = entry.rstrip("/")
        parts = Path(relative).parts
        if not relative or Path(relative).is_absolute() or ".." in parts:
            raise WorkerError(f"allowed path {entry!r} must be relative and inside the worktree")


def validate_external_add_paths(paths: list[str], repo: Path) -> list[str]:
    allowed = []
    for raw in paths:
        absolute = Path(raw).resolve()
        if absolute == repo or path_inside(absolute, repo):
            raise WorkerError(f"external add path {absolute} must be outside the repository")
        if path_inside(repo, absolute):
            raise WorkerError(f"external add path {absolute} must not contain the repository")
        allowed.append(str(absolute))
    return allowed


def access_rules(profile: dict) -> str:
    lines = [
        "- Network access is available." if profile["network"] == "on"
        else "- Network access is off for this launch.",
        f"- A single worker command may run for at most {profile['command_timeout_seconds']} seconds.",
    ]
    if profile["external_add_paths"]:
        lines.append("- You may add entries under these paths outside the repository: "
                     + ", ".join(profile["external_add_paths"]) + ".")
    else:
        lines.append("- You may not add entries outside the repository.")
    lines.append("- Container builds are allowed." if profile["container_builds"]
                 else "- Container builds are not allowed.")
    lines.append("- Writes to shared caches are allowed." if profile["shared_cache_writes"]
                 else "- Writes to shared caches are not allowed.")
    lines.append("- Do not install packages globally.")
    return "\n".join(lines)


def exec_argv(
    runtime: str,
    args: argparse.Namespace,
    worktree: Path,
    run_dir: Path,
    sandbox: str,
    writable_roots: list[Path] | None = None,
) -> list[str]:
    binary = runtime_binary(runtime)
    common = [binary, "exec", "-m", args.model, "-c", f"model_reasoning_effort={args.effort}",
              "-c", "model_reasoning_summary=detailed", "--json", "-s", sandbox,
              "-C", str(worktree), "-o", str(run_dir / "last-message.md")]
    for root in writable_roots or []:
        common += ["--add-dir", str(root)]
    if runtime == "traecli":
        common += ["--shell-tool-timeout", f"{int(args.command_timeout)}s"]
    return common + ["-"]


def toml_string(value: str | Path) -> str:
    return json.dumps(str(value))


def toml_array(values: list[str | Path]) -> str:
    return "[" + ",".join(toml_string(value) for value in values) + "]"


def run_sandbox_mode(run: dict) -> str:
    if run.get("sandbox_mode"):
        return str(run["sandbox_mode"])
    value = str(run.get("sandbox", "workspace-write"))
    if value.startswith("insula"):
        return "insula"
    if value.startswith("danger-full-access"):
        return "danger-full-access"
    return value


def resume_argv(
    runtime: str,
    run: dict,
    session_id: str,
    last_message: Path,
    *,
    worktree: Path | None = None,
    writable_roots: list[Path] | None = None,
    sandbox_mode: str | None = None,
) -> list[str]:
    binary = runtime_binary(runtime)
    worktree = Path(worktree or run["worktree"])
    sandbox_mode = sandbox_mode or run_sandbox_mode(run)
    profile = access_profile(run)
    command = [binary, "exec", "resume", session_id, "-m", run.get("model", "GPT-5.5"),
               "-c", f"model_reasoning_effort={run.get('effort', 'xhigh')}",
               "-c", "model_reasoning_summary=detailed",
               "-c", f"sandbox_mode={toml_string(sandbox_mode)}",
               "-c", f"cwd={toml_string(worktree)}",
               "-c", f"sandbox_workspace_write.network_access={'true' if profile['network'] == 'on' else 'false'}"]
    if writable_roots:
        command += ["-c", f"sandbox_workspace_write.writable_roots={toml_array(writable_roots)}"]
    command += ["--json", "-o", str(last_message)]
    if runtime == "traecli":
        command += ["--shell-tool-timeout", f"{int(access_profile(run)['command_timeout_seconds'])}s"]
    return command + ["-"]


def interactive_argv(runtime: str, args: argparse.Namespace, worktree: Path, prompt: str) -> list[str]:
    binary = runtime_binary(runtime)
    trust = runtime_trust_config(runtime, worktree)
    return [
        binary,
        "-m",
        args.model,
        "-c",
        f"model_reasoning_effort={args.effort}",
        "-c",
        "model_reasoning_summary=detailed",
        "-c",
        trust["config"],
        "--yolo",
        prompt,
    ]


def resume_command(runtime: str, run: dict, session_id: str, prefix: Path) -> tuple[list[str], Path]:
    run_dir = Path(run["run_dir"])
    host_last_message = Path(str(prefix) + "-last-message.md")
    temp = ensure_run_temp(run)
    capsule_ref = run.get("capsule")
    if capsule_ref:
        inner_prefix = Path("/home/bench/run") / prefix.name
        inner_last_message = Path(str(inner_prefix) + "-last-message.md")
        inner = resume_argv(
            runtime,
            run,
            session_id,
            inner_last_message,
            worktree=Path("/workspace"),
            sandbox_mode="danger-full-access",
        )
        capsule_tool = Path(__file__).resolve().parents[1] / "insula" / "capsule.py"
        command = [
            sys.executable, str(capsule_tool), "run", str(capsule_ref), "--workspace", run["worktree"],
            "--net", "host" if access_profile(run)["network"] == "on" else "none",
            "--rw", f"{run_dir}:/home/bench/run", "--env", "TRAE_HOME=/home/bench/.trae",
            "--env", "TRAECLI_HOME=/home/bench/.trae/cli",
            "--env", "TMUX_TMPDIR=/run/corenius-tmp",
            "--env", "TMPPREFIX=/run/corenius-tmp/zsh",
            "--record", str(Path(str(prefix) + "-capsule-run.json")),
        ]
        if run_sandbox_mode(run) == "insula":
            command.append("--ro-workspace")
            binds, _ = sandbox_binds(Path(run["worktree"]), list(run.get("allowed", [])))
            binds.append((Path(run["worktree"]) / ".git", "/workspace/.git"))
            for host, dest in binds:
                command += ["--rw", f"{host}:{dest}"]
        return command + ["--", *inner], host_last_message

    writable = [Path(run["git_dir"])] if run.get("git_dir_layout") == "separate" else []
    writable.append(Path(temp["tmpdir"]))
    return (
        resume_argv(runtime, run, session_id, host_last_message, worktree=Path(run["worktree"]), writable_roots=writable),
        host_last_message,
    )


def path_inside(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def validate_external_protected(paths: list[str], repo: Path, run_root: Path | None = None) -> list[Path]:
    protected = []
    run_roots = []
    if run_root is not None:
        run_roots = [Path(os.path.abspath(run_root)), Path(os.path.abspath(run_root)) / "workers"]
    for raw in paths:
        path = Path(raw)
        if not path.is_absolute():
            raise WorkerError(f"external protected path {raw!r} must be absolute")
        absolute = Path(os.path.abspath(raw))
        if not os.path.lexists(absolute):
            raise WorkerError(f"external protected path {absolute} does not exist")
        if absolute == repo or path_inside(absolute, repo):
            raise WorkerError(f"external protected path {absolute} must be outside the repository")
        if path_inside(repo, absolute):
            raise WorkerError(f"external protected path {absolute} must not contain the repository")
        for root in run_roots:
            if absolute == root or path_inside(root, absolute):
                raise WorkerError(f"external protected path {absolute} must not contain worker run storage")
        protected.append(absolute)
    return protected


def protected_snapshots(paths: list[str]) -> list[dict]:
    protected = []
    for path in paths:
        resolved = str(Path(path).resolve())
        protected.append(
            {
                "path": resolved,
                "head": git(resolved, "rev-parse", "HEAD").strip(),
                "dirty": git(resolved, "status", "--porcelain", "--untracked-files=no"),
            }
        )
    return protected


def digest_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(1024 * 1024)
            if not chunk:
                break
            hasher.update(chunk)
    return hasher.hexdigest()


def external_state_entry(root: Path, path: Path, *, hash_files: bool, digest_limit: int) -> dict:
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
    elif kind == "file" and st.st_size <= digest_limit:
        entry["digest"] = digest_file(path) if hash_files else None
    return entry


def snapshot_external_state(paths: list[Path], *, hash_files: bool, digest_limit: int) -> list[dict]:
    entries = []
    for root in paths:
        if not os.path.lexists(root):
            continue
        stack = [root]
        while stack:
            path = stack.pop()
            try:
                entry = external_state_entry(root, path, hash_files=hash_files, digest_limit=digest_limit)
            except FileNotFoundError:
                continue
            entries.append(entry)
            if entry["kind"] == "directory":
                try:
                    stack.extend(sorted(path.iterdir(), key=lambda p: p.name, reverse=True))
                except FileNotFoundError:
                    continue
    return sorted(entries, key=lambda e: (e["root"], e["relative"]))


def timed_external_snapshot(paths: list[Path], *, hash_files: bool, digest_limit: int) -> tuple[list[dict], dict]:
    started = time.perf_counter()
    entries = snapshot_external_state(paths, hash_files=hash_files, digest_limit=digest_limit)
    return entries, {
        "entry_count": len(entries),
        "scan_seconds": time.perf_counter() - started,
        "recorded": now(),
    }


def write_external_baseline(paths: list[Path], run_dir: Path, digest_limit: int) -> dict | None:
    if not paths:
        return None
    entries, scan = timed_external_snapshot(paths, hash_files=True, digest_limit=digest_limit)
    baseline = {
        "recorded": now(),
        "recorded_epoch": time.time(),
        "digest_limit_bytes": digest_limit,
        "protected_paths": [str(p) for p in paths],
        "entries": entries,
    }
    baseline_file = run_dir / "external-state-baseline.json"
    baseline_file.write_text(json.dumps(baseline, indent=2) + "\n")
    return {
        "protected_paths": [str(p) for p in paths],
        "baseline_file": str(baseline_file),
        "baseline_file_timestamp_epoch": baseline_file.stat().st_mtime,
        "digest_limit_bytes": digest_limit,
        "baseline_entry_count": scan["entry_count"],
        "baseline_scan_seconds": scan["scan_seconds"],
    }


def in_any_path(path: str, roots: list[str]) -> bool:
    item = Path(path)
    for raw in roots:
        root = Path(raw)
        if item == root or path_inside(item, root):
            return True
    return False


def external_state_changes(run: dict, *, hash_files: bool) -> dict:
    state = run.get("external_state")
    if not state:
        return {"modified": [], "removed": [], "added": []}
    baseline = json.loads(Path(state["baseline_file"]).read_text())
    digest_limit = int(state.get("digest_limit_bytes", baseline.get("digest_limit_bytes", EXTERNAL_DIGEST_LIMIT_BYTES)))
    paths = [Path(p) for p in baseline["protected_paths"]]
    current, scan = timed_external_snapshot(paths, hash_files=hash_files, digest_limit=digest_limit)
    state["last_scan"] = scan
    before = {entry["path"]: entry for entry in baseline["entries"]}
    after = {entry["path"]: entry for entry in current}
    add_paths = access_profile(run).get("external_add_paths", [])

    removed = sorted(path for path in before if path not in after)
    added = sorted(path for path in after if path not in before and not in_any_path(path, add_paths))
    modified = []
    for path in sorted(set(before) & set(after)):
        left, right = before[path], after[path]
        if left.get("kind") == "directory" and right.get("kind") == "directory":
            continue
        fields = ["kind", "size", "mtime_ns", "target"]
        if hash_files:
            fields.append("digest")
        if any(left.get(field) != right.get(field) for field in fields):
            modified.append(path)
    return {"modified": modified, "removed": removed, "added": added}


def path_usage_bytes(path: Path) -> int:
    total = 0
    if not os.path.lexists(path):
        return total
    stack = [path]
    while stack:
        item = stack.pop()
        try:
            st = os.lstat(item)
        except FileNotFoundError:
            continue
        total += st.st_size
        if stat.S_ISDIR(st.st_mode):
            try:
                stack.extend(item.iterdir())
            except (FileNotFoundError, PermissionError):
                continue
    return total


def disk_usage(run: dict) -> dict:
    run_dir = Path(run["run_dir"])
    worktree = Path(run["worktree"])
    worktree_bytes = path_usage_bytes(worktree)
    git_dir = Path(run.get("git_dir") or worktree / ".git")
    git_dir_bytes = path_usage_bytes(git_dir)
    git_dir_extra_bytes = 0 if path_inside(git_dir, worktree) else git_dir_bytes
    cache_paths = [run_dir]
    return {
        "worktree_bytes": worktree_bytes,
        "git_dir": str(git_dir),
        "git_dir_bytes": git_dir_bytes,
        "worker_repo_bytes": worktree_bytes + git_dir_extra_bytes,
        "run_dir_bytes": path_usage_bytes(run_dir),
        "caches": [{"path": str(path), "bytes": path_usage_bytes(path)} for path in cache_paths],
        "recorded": now(),
    }


def external_change_labels(run: dict, paths: list[str]) -> list[str]:
    state = run.get("external_state") or {}
    roots = [Path(p) for p in state.get("protected_paths", [])]
    labels = []
    for raw in paths:
        path = Path(raw)
        label = path.name
        for index, root in enumerate(roots, start=1):
            if path == root:
                label = f"external[{index}]:{root.name}"
                break
            if path_inside(path, root):
                label = f"external[{index}]:{root.name}/{path.relative_to(root)}"
                break
        labels.append(label)
    return labels


def launch(args: argparse.Namespace) -> int:
    if args.sandbox == "insula" and not getattr(args, "capsule", None):
        raise WorkerError("--sandbox insula needs --capsule: the capsule is what enforces the paths")
    if args.cap <= 0:
        raise WorkerError("--cap must be positive")
    if args.stall <= 0:
        raise WorkerError("--stall must be positive")
    if args.command_timeout <= 0:
        raise WorkerError("--command-timeout must be positive")
    validate_allowed_paths(list(args.allow))
    if not Path(args.task).is_file():
        raise WorkerError(f"task file {args.task} not found")
    if args.handoff_mode == "marker" and not args.handoff_marker:
        raise WorkerError("--handoff-marker is required when --handoff-mode marker")
    requested_repo = Path(args.repo).resolve()
    repo = main_worktree(requested_repo)
    task = Path(args.task).read_text()
    run_root = runs_root()
    args.external_add_path = validate_external_add_paths(list(args.external_add_path or []), repo)
    external_protected = validate_external_protected(list(args.external_protect_path or []), repo, run_root)
    name = re.sub(r"[^a-z0-9-]", "-", args.name.lower()).strip("-")
    if not name:
        raise WorkerError("worker name is empty")
    branch = f"worker/{name}"
    worktree = repo.parent / f"{repo.name}-worker-{name}"
    if worktree.exists():
        raise WorkerError(f"{worktree} already exists; choose another name or remove it")
    if ref_commit(repo, f"refs/heads/{branch}"):
        raise WorkerError(f"branch {branch} already exists")
    base_sha = ref_commit(requested_repo, args.base)
    if base_sha is None:
        raise WorkerError(f"ref {args.base!r} not found")
    protected = protected_snapshots([str(repo)] + list(args.protect or []))
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_dir = run_root / "workers" / f"{name}-{stamp}"
    run_dir.mkdir(parents=True)
    capsule_ref = getattr(args, "capsule", None)
    enforced = args.sandbox == "insula"
    separate_git_dir = None if capsule_ref else run_dir / "git"
    shared_branch_baselines = local_branch_heads(repo)
    clone = create_private_worker_clone(repo, worktree, branch, args.base, separate_git_dir, base_repo=requested_repo, base_sha=base_sha)

    # The main checkout is protected as-is: the worker must not touch it.

    precreated: list[Path] = []
    rw_binds: list[tuple[Path, str]] = []
    if enforced:
        rw_binds, precreated = sandbox_binds(worktree, list(args.allow))
        rw_binds.append((worktree / ".git", "/workspace/.git"))
    allowed_text = ", ".join(args.allow) or "(none: read-only run)"
    if enforced:
        allowed_text += " (enforced: every other path under /workspace is mounted read-only)"
    if capsule_ref:
        # Inside a capsule only the worktree is mounted (at /workspace); other repos and git metadata are not visible.
        shown_tree, shown_branch = "/workspace", "(not visible in the capsule)"
        protected_rule = "- Only /workspace is visible to you; other repositories and git metadata are not mounted."
    else:
        shown_tree, shown_branch = str(worktree), branch
        protected_rule = (
            f"- These repositories are read-only for you: {', '.join(p['path'] for p in protected)}. Read them at their\n"
            "  recorded commit with `git -C <repo> show <commit>:<path>`; make no git writes there."
        )
    if external_protected:
        protected_rule += (
            "\n- These external paths are protected and observed for changes during the run: "
            + ", ".join(str(p) for p in external_protected) + "."
        )
    handoff_rule = (
        "Exit after committing the finished work."
        if args.handoff_mode == "exit"
        else f"Commit the ready hand-off with subject exactly {args.handoff_marker!r}; the supervisor may mark it ready before this process exits."
    )
    profile = launch_access_profile(args)
    temp = new_run_temp(run_dir, keep=args.keep_temp)
    prompt_temp = "/run/corenius-tmp (private capsule tmpfs)" if capsule_ref else temp["tmpdir"]
    prompt = HEADER.format(
        worktree=shown_tree,
        branch=shown_branch,
        allowed=allowed_text,
        protected_rule=protected_rule,
        handoff_rule=handoff_rule,
        access_rules=access_rules(profile),
        temp_dir=prompt_temp,
    ) + task
    (run_dir / "prompt.md").write_text(prompt)
    brief_digest = hashlib.sha256(prompt.encode()).hexdigest()

    runtime = args.runtime
    worker_cmd = None
    if args.interactive and capsule_ref:
        raise WorkerError("interactive launch does not support --capsule")
    runtime_trust = runtime_trust_config(runtime, worktree) if args.interactive else None
    if args.interactive:
        worker_cmd = interactive_argv(runtime, args, worktree, prompt)
    elif capsule_ref:
        # The capsule is the boundary, so the runtime's own sandbox is switched off.
        inner = exec_argv(runtime, args, Path("/workspace"), Path("/home/bench/run"), "danger-full-access")
        capsule_tool = Path(__file__).resolve().parents[1] / "insula" / "capsule.py"
        worker_cmd = [
            sys.executable, str(capsule_tool), "run", capsule_ref, "--workspace", str(worktree),
            "--net", "host" if profile["network"] == "on" else "none",
            "--rw", f"{run_dir}:/home/bench/run", "--env", "TRAE_HOME=/home/bench/.trae",
            "--env", "TRAECLI_HOME=/home/bench/.trae/cli",
            "--env", "TMPDIR=/run/corenius-tmp", "--env", "TMUX_TMPDIR=/run/corenius-tmp",
            "--env", "TMPPREFIX=/run/corenius-tmp/zsh", "--record", str(run_dir / "capsule-run.json"),
        ]
        if enforced:
            # F12: the worktree is read-only inside the capsule; only the allowed paths are writable. A write
            # elsewhere fails with EROFS at the kernel instead of being caught afterwards by the supervisor.
            worker_cmd.append("--ro-workspace")
            for host, dest in rw_binds:
                worker_cmd += ["--rw", f"{host}:{dest}"]
        worker_cmd += ["--", *inner]
    else:
        extra_roots = [Path(clone["git_dir"])] if clone["git_dir_layout"] == "separate" else []
        extra_roots.append(Path(temp["tmpdir"]))
        worker_cmd = exec_argv(runtime, args, worktree, run_dir, args.sandbox, extra_roots)
    (run_dir / "events.jsonl").touch()
    (run_dir / "stderr.log").touch()
    external_state = write_external_baseline(external_protected, run_dir, EXTERNAL_DIGEST_LIMIT_BYTES)
    tmux_window = None
    interactive_script = None
    process = None
    pid = None
    if args.interactive:
        if not args.tmux_session:
            raise WorkerError("interactive launch requires --tmux-session")
        window_name = args.tmux_window or name[:32]
        shell_pid = run_dir / "shell.pid"
        interactive_script = run_dir / "interactive-launch.sh"
        command_text = (
            "#!/bin/sh\n"
            f"export TMPDIR={shlex.quote(temp['tmpdir'])}\n"
            f"export TMUX_TMPDIR={shlex.quote(temp['tmux_tmpdir'])}\n"
            f"export TMPPREFIX={shlex.quote(temp['tmpprefix'])}\n"
            f"printf '%s\\n' \"$$\" > {shlex.quote(str(shell_pid))}\n"
            f"cd {shlex.quote(str(worktree))}\n"
            f"timeout --foreground {int(args.cap)} {shlex.join(worker_cmd)}\n"
            "status=$?\n"
            f"printf '%s\\n' \"$status\" > {shlex.quote(str(run_dir / 'exit_code'))}\n"
            "printf '\\n[corenius] runtime exited with status %s; terminal retained for review/rework.\\n' \"$status\"\n"
            "exec ${SHELL:-sh}\n"
        )
        interactive_script.write_text(command_text)
        interactive_script.chmod(0o755)
        tmux_window = tmux_interactive_window(
            args.tmux_session,
            window_name,
            worktree,
            shlex.quote(str(interactive_script)),
            runtime_temp_env(temp),
        )
        pid = read_pid_file(shell_pid)
    else:
        script = (
            f"timeout --foreground {int(args.cap)} {shlex.join(worker_cmd)} "
            f"< {shlex.quote(str(run_dir / 'prompt.md'))} "
            f"> {shlex.quote(str(run_dir / 'events.jsonl'))} "
            f"2> {shlex.quote(str(run_dir / 'stderr.log'))}; "
            f"echo $? > {shlex.quote(str(run_dir / 'exit_code'))}"
        )
        process = subprocess.Popen(
            ["sh", "-c", script],
            cwd=worktree,
            start_new_session=True,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            env=run_environment(temp),
        )
        pid = process.pid
    run = {
        "name": name,
        "run_dir": str(run_dir),
        "worktree": str(worktree),
        "workspace_kind": "private-clone",
        "shared_repo": str(repo),
        "git_dir": clone["git_dir"],
        "git_dir_layout": clone["git_dir_layout"],
        "clone_object_mode": clone["clone_object_mode"],
        "branch": branch,
        "base_sha": base_sha,
        "allowed": list(args.allow),
        "protected": protected,
        "branch_baselines": {k: v for k, v in local_branch_heads(worktree).items() if k != branch},
        "shared_branch_baselines": shared_branch_baselines,
        "forbidden": list(DEFAULT_FORBIDDEN) + list(args.forbid or []),
        "runtime": runtime,
        "runtime_observability": runtime_observability(runtime),
        "interactive": bool(args.interactive),
        "interactive_command": worker_cmd if args.interactive else None,
        "runtime_trust": runtime_trust,
        "interactive_script": str(interactive_script) if interactive_script else None,
        "tmux_window": tmux_window,
        "access_profile": profile,
        "access_profile_support": access_profile_support(),
        "external_state": external_state,
        "temp": temp,
        "brief_digest": brief_digest,
        "handoff_mode": args.handoff_mode,
        "handoff_marker": args.handoff_marker,
        "model": args.model,
        "effort": args.effort,
        "sandbox_mode": args.sandbox,
        "sandbox": ("insula (capsule, read-only worktree, allowed paths bound writable)" if enforced
                    else "danger-full-access (capsule)" if capsule_ref else args.sandbox),
        "capsule": capsule_ref,
        "precreated": [str(p) for p in precreated],
        "cap_seconds": int(args.cap),
        "stall_seconds": int(args.stall),
        "started": now(),
        "started_epoch": time.time(),
    }
    run.update(process_identity(process) if process is not None else {
        "pid": pid,
        "process_group_id": None,
        "process_session_id": None,
    })
    (run_dir / "run.json").write_text(json.dumps(run, indent=2) + "\n")
    print(json.dumps({"run_dir": str(run_dir), "worktree": str(worktree), "branch": branch, "pid": pid}))
    return 0


# --- checks -------------------------------------------------------------------


def load_run(run_dir: str | Path) -> dict:
    path = Path(run_dir) / "run.json"
    if not path.exists():
        raise WorkerError(f"{run_dir} is not a worker run (no run.json)")
    return json.loads(path.read_text())


@contextlib.contextmanager
def run_record_lock(run_dir: str | Path):
    path = Path(run_dir) / "run.lock"
    with path.open("a+") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


@contextlib.contextmanager
def locked_run(run_dir: str | Path):
    with run_record_lock(run_dir):
        yield load_run(run_dir)


def save_run(run: dict) -> None:
    (Path(run["run_dir"]) / "run.json").write_text(json.dumps(run, indent=2) + "\n")


def active_events_file(run: dict) -> Path:
    return Path(run.get("active_events_file") or (Path(run["run_dir"]) / "events.jsonl"))


def allowed_path(relative: str, allowed: list[str]) -> bool:
    relative = relative.rstrip("/")
    return any(relative == entry.rstrip("/") or relative.startswith(entry.rstrip("/") + "/") for entry in allowed)


def porcelain_paths(text: str) -> list[str]:
    paths = []
    for line in text.splitlines():
        if len(line) > 3:
            path = line[3:]
            if " -> " in path:
                path = path.split(" -> ", 1)[1]
            paths.append(path.strip('"'))
    return paths


def command_mentions_path(command: str, relative: str) -> bool:
    path = relative.replace("\\", "/")
    names = {path}
    if "/" in path:
        names.add(str(Path(path).name))
    return any(name and name in command for name in names)


def file_change_mentions_path(changed: str, relative: str) -> bool:
    changed = changed.rstrip("/")
    relative = relative.rstrip("/")
    return changed == relative or relative.startswith(changed + "/")


class Checker:
    """Stateful boundary checks for one run; each cycle returns new findings."""

    def __init__(self, run: dict, *, persist: bool = True):
        self.run = run
        self.persist = persist
        self.run_dir = Path(run["run_dir"])
        self.offset = 0
        self.events = 0
        self.seen: set[tuple[str, str]] = set()
        self.commands: list[str] = []
        self.file_change_paths: list[str] = []
        self.event_adapter = RuntimeEventAdapter(run.get("runtime", "traecli"))
        self.profile = access_profile(run)
        ignored = {LEGACY_RM}
        if "access_profile" not in run:
            ignored.update({NETWORK_COMMAND, CONTAINER_BUILD_COMMAND, SHARED_CACHE_COMMAND})
        self.patterns = [re.compile(p) for p in run["forbidden"] if p not in ignored]
        self.network_pattern = re.compile(NETWORK_COMMAND)
        self.container_pattern = re.compile(CONTAINER_BUILD_COMMAND)
        self.shared_cache_pattern = re.compile(SHARED_CACHE_COMMAND)

    def command_findings(self, command: str) -> list[dict]:
        found = []
        for severity, kind in rm_findings(command, self.run["worktree"]):
            finding = self._finding(severity, kind, command[:200])
            if finding:
                found.append(finding)
        if self.profile["network"] == "off" and self.network_pattern.search(command):
            finding = self._finding("VIOLATION", "network-command", command[:200])
            if finding:
                found.append(finding)
        if not self.profile["container_builds"] and self.container_pattern.search(command):
            finding = self._finding("VIOLATION", "container-build", command[:200])
            if finding:
                found.append(finding)
        if not self.profile["shared_cache_writes"] and self.shared_cache_pattern.search(command):
            finding = self._finding("VIOLATION", "shared-cache-write", command[:200])
            if finding:
                found.append(finding)
        for pattern in self.patterns:
            if pattern.search(command):
                finding = self._finding("VIOLATION", "forbidden-command", command[:200])
                if finding:
                    found.append(finding)
                break
        return found

    def unaccounted_workspace_findings(self, paths: list[str], *, after_candidate: bool = False) -> list[dict]:
        found = []
        for path in paths:
            if not allowed_path(path, self.run["allowed"]):
                continue
            if not after_candidate and (
                any(command_mentions_path(command, path) for command in self.commands)
                or any(file_change_mentions_path(changed, path) for changed in self.file_change_paths)
            ):
                continue
            if after_candidate:
                detail = f"ready candidate is followed by an allowed dirty path not accounted for by the event record: {path}"
                severity = "VIOLATION"
            else:
                detail = f"allowed path changed without a recorded worker command or file-change event naming it: {path}"
                severity = "NOTE"
            finding = self._finding(
                severity,
                "unaccounted-worktree-change",
                detail,
            )
            if finding:
                found.append(finding)
        return found

    def _finding(self, severity: str, kind: str, detail: str) -> dict | None:
        key = (kind, detail)
        if key in self.seen:
            return None
        self.seen.add(key)
        return {"severity": severity, "kind": kind, "detail": detail}

    def external_state_findings(self, *, hash_files: bool) -> list[dict]:
        changes = external_state_changes(self.run, hash_files=hash_files)
        if self.persist and self.run.get("external_state"):
            save_run(self.run)
        found = []
        for kind in ("modified", "removed", "added"):
            paths = changes[kind]
            if not paths:
                continue
            counts = {"modified": 0, "removed": 0, "added": 0}
            counts[kind] = len(paths)
            detail = (
                f"external protected state changed during the run: "
                f"modified={counts['modified']} removed={counts['removed']} added={counts['added']}; "
                f"{kind}: {', '.join(external_change_labels(self.run, paths))}"
            )
            finding = self._finding("VIOLATION", f"external-state-{kind}", detail)
            if finding:
                found.append(finding)
        return found

    def cycle(self, alive: bool) -> list[dict]:
        run, found = self.run, []

        def add(finding):
            if finding:
                found.append(finding)

        worktree = run["worktree"]
        head = git(worktree, "rev-parse", "HEAD", check=False).strip()
        if head and git_proc(worktree, "merge-base", "--is-ancestor", run["base_sha"], "HEAD").returncode != 0:
            add(self._finding("VIOLATION", "base-not-ancestor", f"HEAD {head[:12]} does not contain base {run['base_sha'][:12]}"))
        current_branches = local_branch_heads(worktree)
        baselines = run.get("branch_baselines", {})
        peer_branches = set(run.get("peer_worker_branches", []))
        for name, before in sorted(baselines.items()):
            if name in peer_branches:
                continue
            after = current_branches.get(name)
            if after != before:
                detail = f"{name} {before[:12]} -> {after[:12] if after else '<missing>'}"
                add(self._finding("VIOLATION", "foreign-branch-moved", detail))
        for name, after in sorted(current_branches.items()):
            if name in peer_branches:
                continue
            if name != run["branch"] and name not in baselines:
                add(self._finding("VIOLATION", "foreign-branch-moved", f"{name} <new> -> {after[:12]}"))
        shared = shared_repo(run)
        if shared is not None:
            shared_current = local_branch_heads(shared)
            for name, before in sorted(run.get("shared_branch_baselines", {}).items()):
                if name in peer_branches:
                    continue
                after = shared_current.get(name)
                if after != before:
                    detail = f"{name} {before[:12]} -> {after[:12] if after else '<missing>'}"
                    add(self._finding("NOTE", "shared-branch-moved", detail))
            for name, after in sorted(shared_current.items()):
                if name in peer_branches:
                    continue
                if name != run["branch"] and name not in run.get("shared_branch_baselines", {}):
                    add(self._finding("NOTE", "shared-branch-moved", f"{name} <new> -> {after[:12]}"))
        status = git(worktree, "status", "--porcelain", "--untracked-files=all", check=False)
        dirty_paths = porcelain_paths(status)
        for path in dirty_paths:
            if not allowed_path(path, run["allowed"]):
                add(self._finding("VIOLATION", "unexpected-change", path))

        for protected in run["protected"]:
            current_head = git(protected["path"], "rev-parse", "HEAD", check=False).strip()
            current_dirty = git(protected["path"], "status", "--porcelain", "--untracked-files=no", check=False)
            if current_head != protected["head"]:
                # The lead lands into these repos by fast-forward while workers run; only a rewrite is a violation.
                advanced = subprocess.run(
                    ["git", "-C", protected["path"], "merge-base", "--is-ancestor", protected["head"], current_head],
                    capture_output=True,
                    env=git_env(),
                ).returncode == 0
                if advanced:
                    add(self._finding("NOTE", "protected-head-advanced", f"{protected['path']} {current_head[:12]}"))
                else:
                    add(self._finding("VIOLATION", "protected-head-moved", f"{protected['path']} {current_head[:12]}"))
            if current_dirty != protected["dirty"]:
                add(self._finding("VIOLATION", "protected-dirty-changed", protected["path"]))

        for finding in self.external_state_findings(hash_files=False):
            add(finding)

        events = active_events_file(run)
        if events.exists():
            with events.open("rb") as handle:
                handle.seek(self.offset)
                chunk = handle.read()
            complete = chunk[: chunk.rfind(b"\n") + 1]
            self.offset += len(complete)
            dirty_run = False
            for line in complete.decode("utf-8", "replace").splitlines():
                self.events += 1
                try:
                    event = json.loads(line)
                except ValueError:
                    continue
                session_id = event_session_id(event, run.get("runtime", "traecli"))
                if session_id and not run.get("session_id"):
                    run["session_id"] = session_id
                    update_runtime_transcript(run, session_id)
                    dirty_run = True
                usage = event_usage(event, run.get("runtime", "traecli"))
                if usage is not None:
                    run["model_usage"] = usage
                    dirty_run = True
                for command in self.event_adapter.feed(event):
                    self.commands.append(command)
                    for finding in self.command_findings(command):
                        add(finding)
                self.file_change_paths.extend(self.event_adapter.feed_file_changes(event, Path(worktree)))
            if self.persist and dirty_run:
                save_run(run)
            if not alive:
                for command in self.event_adapter.flush_pending():
                    self.commands.append(command)
                    for finding in self.command_findings(command):
                        add(finding)
                self.file_change_paths.extend(self.event_adapter.flush_pending_file_changes())

        if alive and not run.get("candidate"):
            for finding in self.unaccounted_workspace_findings(dirty_paths):
                add(finding)

        if alive and events.exists():
            quiet = time.time() - events.stat().st_mtime
            if quiet > run["stall_seconds"]:
                add(self._finding("STALL", "stall", f"no events for {int(quiet)}s"))
        stderr = self.run_dir / "stderr.log"
        if stderr.exists() and stderr.stat().st_size:
            add(self._finding("DEGRADED", "stderr", f"{stderr.stat().st_size} bytes in stderr.log"))
        return found


def head_candidate(run: dict, head: str) -> dict:
    return {
        "commit": head,
        "tree": git(run["worktree"], "rev-parse", f"{head}^{{tree}}", check=False).strip(),
        "base": run["base_sha"],
        "brief_digest": run.get("brief_digest"),
    }


def ensure_candidate_history(run: dict) -> list[dict]:
    history = run.setdefault("candidates", [])
    if run.get("candidate") and not history:
        history.append({
            "round": 0,
            "kind": "initial",
            "candidate": run["candidate"],
            "pinned_at": run.get("candidate_pinned"),
        })
    return history


def shared_rework_context(run: dict) -> tuple[object, Path, str, str] | None:
    lead = run.get("lead_launch") or {}
    effort = lead.get("effort")
    ticket = lead.get("ticket")
    repo = run.get("shared_repo")
    if not (effort and ticket and repo):
        return None
    import acceptance  # noqa: WPS433
    import land as land_tool  # noqa: WPS433

    root = land_tool.resolve_effort_record(Path(repo), effort).worktree
    return acceptance, root, effort, ticket


def review_round_count(run: dict) -> int:
    context = shared_rework_context(run)
    if context is not None:
        acceptance, root, effort, ticket = context
        return acceptance.rework_rounds_used(root, effort, ticket)
    return sum(1 for delivery in run.get("rework_deliveries", []) if delivery.get("counts_toward_cap"))


def next_rework_round(run: dict, kind: str, actor: str) -> int:
    context = shared_rework_context(run)
    if context is not None:
        acceptance, root, effort, ticket = context
        return acceptance.ensure_rework_round_available(root, effort, ticket, kind, actor)
    used = review_round_count(run)
    if kind == "review" and used >= 3:
        raise WorkerError("review-driven rework cap exhausted")
    return used + (1 if kind == "review" else 0)


def record_shared_rework(run: dict, delivery: dict, actor: str) -> dict | None:
    context = shared_rework_context(run)
    if context is None:
        return None
    acceptance, root, effort, ticket = context
    record, path = acceptance.record_rework_round(
        root,
        effort,
        ticket,
        candidate=delivery["candidate"],
        kind=delivery["kind"],
        findings=Path(delivery["findings_file"]),
        actor=actor,
        extra={
            "delivery_id": delivery["id"],
            "run_dir": delivery["run_dir"],
            "receipt": delivery.get("receipt"),
        },
    )
    delivery["effort_rework_record"] = {"path": str(path), "round": record["round"]}
    return record


def delivery_acknowledged(delivery: dict) -> bool:
    receipt = delivery.get("receipt") or {}
    return receipt.get("acknowledged") is True


def current_candidate_entry(run: dict) -> dict | None:
    history = ensure_candidate_history(run)
    return history[-1] if history else None


def pending_delivery(run: dict) -> dict | None:
    deliveries = run.get("rework_deliveries", [])
    used = {
        entry.get("source_delivery_id")
        for entry in run.get("candidates", [])
        if entry.get("source_delivery_id")
    }
    latest_used_index = -1
    for index, delivery in enumerate(deliveries):
        if delivery.get("id") in used:
            latest_used_index = index
    for index, delivery in reversed(list(enumerate(deliveries))):
        if index <= latest_used_index:
            break
        if delivery_acknowledged(delivery) and delivery.get("id") not in used:
            return delivery
    return None


def findings_message(run: dict, findings: Path, digest: str, kind: str) -> str:
    ticket = (run.get("lead_launch") or {}).get("ticket")
    if kind == "stale-base":
        action = "The integration branch moved. Re-integrate onto the new base, preserving the earlier Candidate."
    else:
        action = "The lead reviewed your branch and requires changes before it can land."
    marker = run.get("handoff_marker")
    finish = f" Finish again with a `{marker}` commit." if marker else " Commit the finished work again."
    ticket_text = f" ticket {ticket}" if ticket else ""
    return f"{action} Read {findings.resolve()} (sha256={digest}) for{ticket_text}.{finish}"


def event_acknowledgement(run: dict, offset: int, findings: Path, digest: str | None, events_file: str | None = None) -> dict | None:
    events = Path(events_file) if events_file else active_events_file(run)
    if not events.exists() or events.stat().st_size <= offset:
        return None
    resolved = str(findings.resolve())
    with events.open("rb") as handle:
        handle.seek(offset)
        chunk = handle.read()
    complete = chunk[: chunk.rfind(b"\n") + 1]
    for raw in complete.decode("utf-8", "replace").splitlines():
        try:
            event = json.loads(raw)
        except ValueError:
            continue
        text = json.dumps(event, sort_keys=True)
        if resolved not in text and (not digest or digest not in text):
            continue
        payload = event.get("payload") if isinstance(event.get("payload"), dict) else {}
        event_type = event.get("type") or payload.get("type") or "<unknown>"
        return {
            "kind": "event",
            "events_file": str(events),
            "offset": offset,
            "event_type": event_type,
            "session_id": event_session_id(event, run.get("runtime", "traecli")),
        }
    return None


def commit_acknowledgement(run: dict, head_before: str | None) -> dict | None:
    head = git(run["worktree"], "rev-parse", "HEAD", check=False).strip()
    if head_before and head and head != head_before:
        marker = run.get("handoff_marker")
        if marker:
            subject = git(run["worktree"], "log", "-1", "--format=%s", "HEAD", check=False).strip()
            if subject != marker:
                return None
        return {"kind": "commit", "head_before": head_before, "head": head}
    return None


def tmux_pane_text(target: str) -> str | None:
    proc = subprocess.run(["tmux", "capture-pane", "-p", "-t", target], capture_output=True, text=True)
    if proc.returncode != 0:
        return None
    return proc.stdout


def pane_acknowledgement(target: str | None, pane_before: str | None, findings: Path) -> dict | None:
    if not target:
        return None
    current = tmux_pane_text(target)
    if current is None:
        return None
    findings_path = str(findings.resolve())
    current_compact = re.sub(r"\s+", "", current)
    if (
        current != (pane_before or "")
        and (findings_path in current or re.sub(r"\s+", "", findings_path) in current_compact)
    ):
        return {"kind": "pane-activity", "target": target}
    return None


def observe_delivery_acknowledgement(
    run: dict,
    *,
    events_offset: int,
    head_before: str | None,
    terminal_target: str | None,
    pane_before: str | None,
    findings: Path,
    findings_digest: str | None = None,
    events_file: str | None = None,
) -> dict | None:
    return (
        event_acknowledgement(run, events_offset, findings, findings_digest, events_file)
        or commit_acknowledgement(run, head_before)
    )


def wait_for_delivery_acknowledgement(
    run: dict,
    *,
    events_offset: int,
    head_before: str | None,
    terminal_target: str | None,
    pane_before: str | None,
    findings: Path,
    findings_digest: str | None = None,
    events_file: str | None = None,
    wait_seconds: float,
) -> dict | None:
    deadline = time.time() + max(0.0, wait_seconds)
    while True:
        acknowledgement = observe_delivery_acknowledgement(
            run,
            events_offset=events_offset,
            head_before=head_before,
            terminal_target=terminal_target,
            pane_before=pane_before,
            findings=findings,
            findings_digest=findings_digest,
            events_file=events_file,
        )
        if acknowledgement or time.time() >= deadline:
            return acknowledgement
        time.sleep(min(0.05, max(0.0, deadline - time.time())))


def terminal_delivery_receipt(run: dict, target: str | None, input_file: str | None, message: str, findings: Path, wait_seconds: float) -> dict:
    events = active_events_file(run)
    events_offset = events.stat().st_size if events.exists() else 0
    head_before = git(run["worktree"], "rev-parse", "HEAD", check=False).strip()
    pane_before = tmux_pane_text(target) if target else None
    digest = digest_file(findings)
    if input_file:
        path = Path(input_file)
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a") as handle:
            handle.write(message + "\n")
        target_label = str(path)
    elif not target:
        raise WorkerError("terminal delivery needs --terminal-target or --terminal-input-file")
    else:
        first = subprocess.run(["tmux", "send-keys", "-t", target, "-l", message], capture_output=True, text=True)
        if first.returncode != 0:
            raise WorkerError(first.stderr.strip() or first.stdout.strip() or "tmux send-keys failed")
        second = subprocess.run(["tmux", "send-keys", "-t", target, "Enter"], capture_output=True, text=True)
        if second.returncode != 0:
            raise WorkerError(second.stderr.strip() or second.stdout.strip() or "tmux send-keys Enter failed")
        target_label = target
    sent_epoch = time.time()
    receipt = {
        "transport": "terminal",
        "target": target_label,
        "terminal_target": target,
        "pane_before": pane_before,
        "sent": True,
        "sent_at": now(),
        "sent_epoch": sent_epoch,
        "events_file": str(events),
        "events_offset": events_offset,
        "head_before": head_before,
        "ack_wait_seconds": wait_seconds,
        "acknowledged": False,
    }
    acknowledgement = wait_for_delivery_acknowledgement(
        run,
        events_offset=events_offset,
        head_before=head_before,
        terminal_target=target,
        pane_before=pane_before,
        findings=findings,
        findings_digest=digest,
        events_file=str(events),
        wait_seconds=wait_seconds,
    )
    if acknowledgement:
        receipt["acknowledged"] = True
        receipt["acknowledged_at"] = now()
        receipt["acknowledgement"] = acknowledgement
    else:
        receipt["reason"] = f"no worker acknowledgement observed within {wait_seconds:g}s"
    return receipt


def attach_delivery_review(run: dict, delivery: dict) -> None:
    for entry in reversed(ensure_candidate_history(run)):
        candidate = entry.get("candidate") or {}
        if candidate.get("commit") == delivery.get("candidate"):
            entry["review"] = {
                "delivery_id": delivery["id"],
                "findings_file": delivery["findings_file"],
                "findings_sha256": delivery["findings_sha256"],
                "delivered_at": delivery["delivered_at"],
            }
            return


def acknowledge_delivery(run: dict, delivery: dict, acknowledgement: dict) -> bool:
    receipt = delivery.setdefault("receipt", {})
    if receipt.get("acknowledged") is True:
        return False
    receipt["acknowledged"] = True
    receipt["acknowledged_at"] = now()
    receipt["acknowledgement"] = acknowledgement
    if delivery.get("kind") == "review" and not delivery.get("counts_toward_cap"):
        delivery["round"] = review_round_count(run) + 1
        delivery["counts_toward_cap"] = True
    attach_delivery_review(run, delivery)
    with (Path(run["run_dir"]) / "rework.jsonl").open("a") as handle:
        handle.write(json.dumps({
            "schema": "corenius.rework_acknowledgement.v1",
            "delivery_id": delivery["id"],
            "acknowledged_at": receipt["acknowledged_at"],
            "acknowledgement": acknowledgement,
        }, sort_keys=True) + "\n")
    return True


def acknowledge_pending_rework_deliveries(run: dict) -> None:
    changed = False
    for delivery in run.get("rework_deliveries", []):
        receipt = delivery.get("receipt") or {}
        if receipt.get("acknowledged") is True:
            continue
        acknowledgement = observe_delivery_acknowledgement(
            run,
            events_offset=int(receipt.get("events_offset") or 0),
            head_before=receipt.get("head_before"),
            terminal_target=receipt.get("terminal_target"),
            pane_before=receipt.get("pane_before"),
            findings=Path(delivery["findings_file"]),
            findings_digest=delivery.get("findings_sha256"),
            events_file=receipt.get("events_file"),
        )
        if acknowledgement:
            changed = acknowledge_delivery(run, delivery, acknowledgement) or changed
    if changed:
        save_run(run)


def handoff_findings(run: dict) -> list[dict]:
    candidate = run.get("candidate")
    if not candidate:
        return []
    head = git(run["worktree"], "rev-parse", "HEAD", check=False).strip()
    if head and head != candidate.get("commit") and pending_delivery(run) is None:
        return [{
            "severity": "VIOLATION",
            "kind": "candidate-moved",
            "detail": f"{candidate.get('commit')} -> {head}",
        }]
    return []


def handoff_state(run: dict, exit_code: int | None, findings: list[dict], persist: bool = False) -> dict:
    worktree = run["worktree"]
    status = git(worktree, "status", "--porcelain", "--untracked-files=all", check=False)
    head = git(worktree, "rev-parse", "HEAD", check=False).strip()
    if run.get("candidate"):
        delivery = pending_delivery(run)
        if delivery:
            mode = run.get("handoff_mode", "exit")
            marker_ok = True
            if mode == "marker":
                subject = git(worktree, "log", "-1", "--format=%s", "HEAD", check=False).strip()
                marker_ok = subject == run.get("handoff_marker")
            status_ok = not status.strip()
            contains_base = git_proc(worktree, "merge-base", "--is-ancestor", run["base_sha"], "HEAD").returncode == 0
            if head and head != run["candidate"].get("commit") and status_ok and contains_base and not marker_ok:
                return {
                    "ready_for_review": False,
                    "candidate": None,
                    "not_ready": ["fixes-committed-waiting-for-re-integration"],
                }
            if head and head != run["candidate"].get("commit") and marker_ok and status_ok and contains_base:
                candidate = head_candidate(run, head)
                if persist:
                    entry = {
                        "round": delivery["round"],
                        "kind": "successor",
                        "candidate": candidate,
                        "pinned_at": now(),
                        "source_delivery_id": delivery["id"],
                    }
                    ensure_candidate_history(run).append(entry)
                    import_candidate_to_shared_repo(run, candidate)
                    run["candidate"] = candidate
                    run["candidate_pinned"] = entry["pinned_at"]
                    save_run(run)
                return {"ready_for_review": True, "candidate": candidate, "not_ready": []}
            return {"ready_for_review": False, "candidate": None, "not_ready": ["awaiting-rework"]}
        return {"ready_for_review": True, "candidate": run["candidate"], "not_ready": []}
    reasons = []
    mode = run.get("handoff_mode", "exit")
    if mode == "exit" and exit_code != 0:
        reasons.append("exit-code")
    if mode == "marker":
        subject = git(worktree, "log", "-1", "--format=%s", "HEAD", check=False).strip()
        if subject != run.get("handoff_marker"):
            reasons.append("handoff-marker")
    if status.strip():
        reasons.append("dirty-tree")
    contains_base = git_proc(worktree, "merge-base", "--is-ancestor", run["base_sha"], "HEAD").returncode == 0
    if not contains_base:
        reasons.append("base-not-ancestor")
    count = 0
    if contains_base:
        out = git(worktree, "rev-list", "--count", f"{run['base_sha']}..HEAD", check=False).strip()
        count = int(out or "0")
    if count < 1:
        reasons.append("no-commit-after-base")
    if any(f["severity"] == "VIOLATION" for f in findings):
        reasons.append("boundary-violation")
    ready = not reasons
    candidate = None
    if ready:
        candidate = head_candidate(run, head)
        if persist:
            import_candidate_to_shared_repo(run, candidate)
            run["candidate"] = candidate
            run["candidate_pinned"] = now()
            ensure_candidate_history(run)
            save_run(run)
    return {"ready_for_review": ready, "candidate": candidate, "not_ready": reasons}


def post_candidate_workspace_findings(run: dict, checker: Checker) -> list[dict]:
    if not run.get("candidate"):
        return []
    status = git(run["worktree"], "status", "--porcelain", "--untracked-files=all", check=False)
    return checker.unaccounted_workspace_findings(porcelain_paths(status), after_candidate=True)


def process_state(run: dict) -> tuple[bool, int | None]:
    exit_file = Path(run["run_dir"]) / "exit_code"
    session_id = run.get("process_session_id")
    try:
        session_id = int(session_id) if session_id is not None else None
    except (TypeError, ValueError):
        session_id = None
    if live_session_pids(session_id):
        return True, None
    if exit_file.exists() and exit_file.read_text().strip():
        return False, int(exit_file.read_text().strip())
    pid = run.get("pid")
    if pid is None:
        return False, None
    return pid_alive(int(pid)), None


def running_notices(run: dict, alive: bool, elapsed: int, exit_code: int | None = None) -> list[dict]:
    notices = []
    cap = int(run.get("cap_seconds", 0) or 0)
    cap_reached = cap and (alive or exit_code == 124) and elapsed >= cap
    if cap_reached:
        notices.append({
            "severity": "NOTICE",
            "kind": "wall-clock-limit",
            "detail": f"elapsed {elapsed}s reached cap {cap}s",
        })
    if not alive:
        return notices
    events = active_events_file(run)
    if events.exists():
        quiet = time.time() - events.stat().st_mtime
        if quiet > int(run.get("stall_seconds", 0) or 0):
            notices.append({
                "severity": "NOTICE",
                "kind": "quiet-time",
                "detail": f"no events for {int(quiet)}s",
            })
    return notices


def run_tmux(*args: str) -> tuple[subprocess.CompletedProcess | None, str]:
    try:
        return subprocess.run(["tmux", *args], capture_output=True, text=True), ""
    except OSError as exc:
        return None, str(exc)


def tmux_environment_options(extra: dict[str, str] | None = None) -> list[str]:
    env = {
        key: value
        for key, value in os.environ.items()
        if key.startswith("CORENIUS_") or key in {"TMPDIR", "TMUX_TMPDIR", "TMPPREFIX"}
    }
    env.update(extra or {})
    forwarded = []
    for key, value in sorted(env.items()):
        forwarded.extend(["-e", f"{key}={value}"])
    return forwarded


def tmux_interactive_window(session: str, window_name: str, cwd: Path, command: str, env: dict[str, str] | None = None) -> dict:
    base = {"available": False, "session": session, "window": window_name, "command": command}
    has, error = run_tmux("has-session", "-t", session)
    if has is None:
        raise WorkerError(f"interactive launch requires tmux: {error}")
    if has.returncode != 0:
        created, error = run_tmux(
            "new-session", "-d", "-s", session, "-n", "overview", "-c", str(cwd), "while :; do sleep 3600; done"
        )
        if created is None:
            raise WorkerError(f"interactive launch requires tmux: {error}")
        if created.returncode != 0:
            detail = (created.stderr or created.stdout or has.stderr or has.stdout).strip()
            raise WorkerError(f"interactive launch requires tmux: {detail}")
    new, error = run_tmux(
        "new-window",
        "-d",
        "-P",
        "-F",
        "#{window_id} #{pane_id}",
        *tmux_environment_options(env),
        "-t",
        f"{session}:",
        "-n",
        window_name,
        "-c",
        str(cwd),
        command,
    )
    if new is None:
        raise WorkerError(f"interactive launch requires tmux: {error}")
    if new.returncode != 0:
        detail = (new.stderr or new.stdout).strip()
        raise WorkerError(f"interactive launch requires tmux: {detail}")
    fields = new.stdout.strip().split()
    record = {**base, "available": True, "target": f"{session}:{window_name}"}
    if fields:
        record["window_id"] = fields[0]
    if len(fields) > 1:
        record["pane_id"] = fields[1]
    return record


def read_pid_file(path: Path, timeout: float = 2.0) -> int | None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            text = path.read_text().strip()
        except FileNotFoundError:
            time.sleep(0.05)
            continue
        if text:
            try:
                return int(text)
            except ValueError:
                return None
    return None


def supervise(args: argparse.Namespace) -> int:
    run = load_run(args.run_dir)
    run_dir = Path(run["run_dir"])
    checker = Checker(run)
    findings: list[dict] = []
    started_here = time.time()
    status_log = run_dir / "status.jsonl"
    seen_notice_kinds: set[str] = set()
    while True:
        alive, exit_code = process_state(run)
        new = checker.cycle(alive)
        findings.extend(new)
        acknowledge_pending_rework_deliveries(run)
        elapsed = int(time.time() - run["started_epoch"])
        notices = []
        for notice in running_notices(run, alive, elapsed, exit_code):
            if notice["kind"] not in seen_notice_kinds:
                notices.append(notice)
                seen_notice_kinds.add(notice["kind"])
        with status_log.open("a") as handle:
            handle.write(
                json.dumps(
                    {"t": now(), "elapsed": elapsed, "alive": alive, "events": checker.events,
                     "new": new, "notices": notices}
                )
                + "\n"
            )
        if not alive:
            # Paths pre-created for the sandbox binds that the worker never wrote are removed first, so an empty
            # placeholder neither hides a missing output nor shows up as the worker's change.
            removed = remove_untouched(run.get("precreated", []))
            if removed:
                findings.append({"severity": "NOTE", "kind": "precreated-removed", "detail": ", ".join(removed)})
            # One more pass now that the process has exited: catch late writes.
            findings.extend(checker.cycle(False))
            acknowledge_pending_rework_deliveries(run)
            findings.extend(checker.external_state_findings(hash_files=True))
            removed_temp = cleanup_run_temp(run)
            if removed_temp:
                findings.append({"severity": "NOTE", "kind": "temp-removed", "detail": removed_temp})
            break
        handoff = handoff_state(run, exit_code, findings, persist=True)
        if run.get("handoff_mode") == "marker" and handoff["ready_for_review"]:
            break
        if time.time() - started_here >= args.max_wait:
            print(json.dumps({"verdict": "RUNNING", "elapsed": elapsed, "events": checker.events,
                              "findings": findings, "disk_usage": disk_usage(run)}))
            return RUNNING
        time.sleep(args.interval)

    missing = [a for a in run["allowed"] if not (Path(run["worktree"]) / a).exists()]
    handoff = handoff_state(run, exit_code, findings, persist=True)
    findings.extend(post_candidate_workspace_findings(run, checker))
    findings.extend(handoff_findings(run))
    severities = {f["severity"] for f in findings} - {"NOTE"}
    bad_exit = run.get("handoff_mode", "exit") == "exit" and exit_code not in (0,)
    if "VIOLATION" in severities:
        code = VIOLATED
    elif severities or bad_exit or missing or not handoff["ready_for_review"]:
        code = DEGRADED
    else:
        code = CLEAN
    verdict = {
        "verdict": VERDICT_NAMES[code],
        "exit_code": exit_code,
        "elapsed_seconds": int(time.time() - run["started_epoch"]),
        "events": checker.events,
        "missing_allowed_outputs": missing,
        "findings": findings,
        **handoff,
        "disk_usage": disk_usage(run),
        "finished": now(),
        "run_dir": str(run_dir),
        "worktree": run["worktree"],
        "branch": run["branch"],
    }
    verdict_file = run_dir / "verdict.json"
    if verdict_file.exists():
        verdict_file.replace(run_dir / "verdict.prev.json")
    verdict_file.write_text(json.dumps(verdict, indent=2) + "\n")
    print(json.dumps(verdict))
    return code


def status(args: argparse.Namespace) -> int:
    if args.update_run_record:
        with locked_run(args.run_dir) as run:
            result = status_verdict(run, persist=True)
    else:
        run = load_run(args.run_dir)
        result = status_verdict(run, persist=False)
    print(json.dumps(result))
    return 0


def status_verdict(run: dict, *, persist: bool) -> dict:
    alive, exit_code = process_state(run)
    session_id = update_session_id_from_events(run, persist=persist)
    checker = Checker(run, persist=persist)
    findings = checker.cycle(alive)
    if persist:
        acknowledge_pending_rework_deliveries(run)
    handoff = handoff_state(run, exit_code, findings, persist=persist)
    findings.extend(post_candidate_workspace_findings(run, checker))
    findings.extend(handoff_findings(run))
    if persist and not alive:
        removed_temp = cleanup_run_temp(run)
        if removed_temp:
            findings.append({"severity": "NOTE", "kind": "temp-removed", "detail": removed_temp})
    return {
        "alive": alive, "exit_code": exit_code, "events": checker.events,
        "elapsed_seconds": int(time.time() - run["started_epoch"]),
        "session_id": run.get("session_id") or session_id,
        "runtime_transcript": run.get("runtime_transcript"),
        "findings": findings,
        "verdict_file": (Path(run["run_dir"]) / "verdict.json").exists(),
        **handoff,
    }


def resume(args: argparse.Namespace) -> int:
    with locked_run(args.run_dir) as run:
        return resume_locked(args, run)


def resume_locked(args: argparse.Namespace, run: dict) -> int:
    runtime = run.get("runtime", "traecli")
    if runtime not in RUNTIMES:
        raise WorkerError(f"runtime {runtime!r} cannot resume: unknown runtime")
    session_id = update_session_id_from_events(run)
    if not session_id:
        raise WorkerError(f"runtime {runtime} cannot resume: no session id observed in events.jsonl")
    message = args.message
    if args.message_file:
        message = Path(args.message_file).read_text()
    if message is None:
        message = sys.stdin.read()
    if not message:
        raise WorkerError("resume message is empty")

    refreshed_integration = refresh_integration_branch(run)
    run_dir = Path(run["run_dir"])
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    prefix = run_dir / f"resume-{stamp}"
    events_file = Path(str(prefix) + "-events.jsonl")
    stderr_file = Path(str(prefix) + "-stderr.log")
    command, last_message = resume_command(runtime, run, session_id, prefix)
    timeout = args.timeout if args.timeout is not None else float(run.get("cap_seconds", 120))
    if args.detach:
        message_file = Path(str(prefix) + "-message.txt")
        message_file.write_text(message)
        script = (
            f"timeout --foreground {int(timeout)} {shlex.join(command)} "
            f"< {shlex.quote(str(message_file))} "
            f"> {shlex.quote(str(events_file))} "
            f"2> {shlex.quote(str(stderr_file))}; "
            f"echo $? > {shlex.quote(str(run_dir / 'exit_code'))}"
        )
        events_file.touch()
        stderr_file.touch()
        exit_file = run_dir / "exit_code"
        if exit_file.exists():
            exit_file.unlink()
        process = subprocess.Popen(
            ["sh", "-c", script],
            cwd=run["worktree"],
            start_new_session=True,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            env=run_environment(ensure_run_temp(run)),
        )
        receipt = {
            "runtime": runtime,
            "session_id": session_id,
            "command": command,
            "events_file": str(events_file),
            "stderr_file": str(stderr_file),
            "last_message": str(last_message),
            "message_file": str(message_file),
            "message_sha256": hashlib.sha256(message.encode()).hexdigest(),
            "timeout_seconds": timeout,
            "stall_seconds": int(run.get("stall_seconds", 300)),
            "transport": "resume",
            "acknowledged": False,
            "reason": "resume process started; no worker-side acknowledgement observed yet",
            "detached": True,
            "refreshed_integration": refreshed_integration,
            **process_identity(process),
            "started": now(),
            "started_epoch": time.time(),
        }
        run["pid"] = process.pid
        run["process_group_id"] = receipt.get("process_group_id")
        run["process_session_id"] = receipt.get("process_session_id")
        run["started"] = receipt["started"]
        run["started_epoch"] = receipt["started_epoch"]
        run["active_events_file"] = str(events_file)
        run.setdefault("resumes", []).append(receipt)
        save_run(run)
        print(json.dumps(receipt))
        return RUNNING
    timed_out = False
    error = None
    try:
        result = subprocess.run(
            command,
            input=message,
            capture_output=True,
            text=True,
            cwd=run["worktree"],
            timeout=timeout,
            env=run_environment(ensure_run_temp(run)),
        )
        stdout, stderr, return_code = result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        error = f"resume timed out after {timeout:g}s"
        stdout, stderr, return_code = output_text(exc.stdout), output_text(exc.stderr), None
    events_file.write_text(output_text(stdout))
    stderr_file.write_text(output_text(stderr))
    receipt = {
        "runtime": runtime,
        "session_id": session_id,
        "command": command,
        "events_file": str(events_file),
        "stderr_file": str(stderr_file),
        "last_message": str(last_message),
        "message_sha256": hashlib.sha256(message.encode()).hexdigest(),
        "timeout_seconds": timeout,
        "transport": "resume",
        "acknowledged": return_code == 0 and not timed_out,
        "reason": "runtime exited 0" if return_code == 0 and not timed_out else (
            error if error else f"runtime exited {return_code}"
        ),
        "detached": False,
        "refreshed_integration": refreshed_integration,
        "timed_out": timed_out,
        "return_code": return_code,
        "error": error,
        "resumed": now(),
    }
    run.setdefault("resumes", []).append(receipt)
    save_run(run)
    print(json.dumps(receipt))
    return 0 if return_code == 0 else DEGRADED


def deliver_findings(args: argparse.Namespace) -> int:
    with locked_run(args.run_dir) as run:
        return deliver_findings_locked(args, run)


def deliver_findings_locked(args: argparse.Namespace, run: dict) -> int:
    findings = Path(args.findings_file)
    if not findings.is_file():
        raise WorkerError(f"findings file {findings} not found")
    current = current_candidate_entry(run)
    if current is None:
        raise WorkerError("cannot deliver findings before a Candidate is pinned")

    digest = digest_file(findings)
    round_number = next_rework_round(run, args.kind, "lead")
    message = args.message or findings_message(run, findings, digest, args.kind)
    receipt = terminal_delivery_receipt(run, args.terminal_target, args.terminal_input_file, message, findings, args.ack_timeout)
    acknowledged = receipt.get("acknowledged") is True
    delivery = {
        "id": f"{args.kind}-{len(run.get('rework_deliveries', [])) + 1}",
        "kind": args.kind,
        "round": round_number,
        "counts_toward_cap": args.kind == "review",
        "run_dir": str(Path(run["run_dir"])),
        "candidate": current["candidate"]["commit"],
        "findings_file": str(findings.resolve()),
        "findings_sha256": digest,
        "delivered_at": receipt["sent_at"],
        "receipt": receipt,
    }
    record_shared_rework(run, delivery, "lead")
    run.setdefault("rework_deliveries", []).append(delivery)
    if acknowledged:
        attach_delivery_review(run, delivery)
    with (Path(run["run_dir"]) / "rework.jsonl").open("a") as handle:
        handle.write(json.dumps(delivery, sort_keys=True) + "\n")
    save_run(run)
    print(json.dumps(delivery, sort_keys=True))
    return CLEAN if acknowledged else DEGRADED


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("launch", epilog=RUNTIME_HELP + "\n" + ACCESS_PROFILE_HELP, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--task", required=True, help="file with the task text")
    p.add_argument("--name", required=True)
    p.add_argument("--runtime", choices=RUNTIMES, default="traecli",
                   help="worker runtime to launch (default: traecli)")
    p.add_argument("--allow", action="append", default=[], help="path (relative to the worktree) the worker may write; repeatable")
    p.add_argument("--protect", action="append", default=[], help="extra repo that must stay unchanged; repeatable")
    p.add_argument("--capsule", help="capsule digest prefix or request file: run the worker inside it")
    p.add_argument("--network", choices=("on", "off"), default="on",
                   help="worker network access for this launch (default: on)")
    p.add_argument("--command-timeout", type=int, default=120,
                   help="maximum seconds for one worker shell command")
    p.add_argument("--external-add-path", action="append", default=[],
                   help="absolute or relative host path outside the repository where the worker may add entries")
    p.add_argument("--external-protect-path", action="append", default=[],
                   help="absolute host file or directory outside the repository to baseline and report changes for")
    p.add_argument("--allow-container-builds", action="store_true",
                   help="declare container builds allowed for this launch")
    p.add_argument("--allow-shared-cache-writes", action="store_true",
                   help="declare writes to shared caches allowed for this launch")
    p.add_argument("--forbid", action="append", default=[], help="extra regex forbidden in worker commands")
    p.add_argument("--repo", default=".")
    p.add_argument("--base", default="main")
    p.add_argument("--model", default="GPT-5.5")
    p.add_argument("--effort", default="xhigh")
    p.add_argument("--sandbox", default="workspace-write",
                   help="traecli sandbox mode, or `insula` (needs --capsule): worktree read-only, allowed paths bound writable")
    p.add_argument("--handoff-mode", choices=("exit", "marker"), default="exit",
                   help="exit: ready after process exits 0; marker: ready when HEAD subject matches --handoff-marker")
    p.add_argument("--handoff-marker", help="required marker commit subject for --handoff-mode marker")
    p.add_argument("--interactive", action="store_true",
                   help="start the runtime in an interactive tmux window instead of detached exec JSON mode")
    p.add_argument("--tmux-session",
                   help="named tmux session for --interactive; uses the user's default tmux server unless TMUX_TMPDIR is set")
    p.add_argument("--tmux-window", help="window name for --interactive")
    p.add_argument("--cap", type=int, default=2700, help="hard wall-clock cap in seconds")
    p.add_argument("--stall", type=int, default=300, help="seconds without events before STALL")
    p.add_argument("--keep-temp", action="store_true", help="keep the run temp directory after the worker is stopped")
    p.set_defaults(func=launch)
    p = sub.add_parser("supervise", epilog=WORKSPACE_ATTRIBUTION_HELP, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("run_dir")
    p.add_argument("--interval", type=float, default=30)
    p.add_argument("--max-wait", type=float, default=540, help="return RUNNING (exit 3) after this many seconds")
    p.set_defaults(func=supervise)
    p = sub.add_parser(
        "status",
        description=(
            "One-shot check of a run. By default this writes nothing. Use --update-run-record to "
            "write observed Candidate state, runtime session id, model usage or worker-side acknowledgement."
        ),
    )
    p.add_argument("run_dir")
    p.add_argument("--update-run-record", action="store_true",
                   help="write observed Candidate state, runtime session id, model usage and acknowledgements to run.json")
    p.set_defaults(func=status)
    p = sub.add_parser("resume")
    p.add_argument("run_dir")
    p.add_argument("--message", help="follow-up message to send to the same runtime session")
    p.add_argument("--message-file", help="file containing the follow-up message")
    p.add_argument("--timeout", type=float, default=None,
                   help="seconds before a foreground resume is marked timed out (default: run wall-clock cap)")
    p.add_argument("--detach", action="store_true",
                   help="start the resume turn detached and let supervise watch the run")
    p.set_defaults(func=resume)
    p = sub.add_parser("deliver-findings")
    p.add_argument("run_dir")
    p.add_argument("--findings-file", required=True)
    p.add_argument("--kind", choices=("review", "stale-base"), default="review")
    p.add_argument("--transport", choices=("terminal",), default="terminal")
    p.add_argument("--terminal-target")
    p.add_argument("--terminal-input-file")
    p.add_argument("--message")
    p.add_argument("--ack-timeout", type=float, default=2.0,
                   help="seconds to wait for a worker-side acknowledgement after sending terminal findings")
    p.set_defaults(func=deliver_findings)
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except WorkerError as error:
        print(f"worker: {error}", file=sys.stderr)
        return 64


if __name__ == "__main__":
    raise SystemExit(main())
