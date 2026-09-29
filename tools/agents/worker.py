#!/usr/bin/env python3
"""Launch a worker agent in its own worktree and supervise it deterministically.

The lead/supervisor/worker form factor (docs/practice/lead-supervisor-worker.md):

* ``launch``    create a worktree and branch, write the prompt file with the
                standard rules, start ``traecli exec`` detached, record a run.
* ``supervise`` block until the worker exits (or ``--max-wait`` elapses), check
                the boundary every cycle, and write ``verdict.json``.
                Exit 0 CLEAN, 1 DEGRADED, 2 VIOLATED, 3 still RUNNING.
* ``status``    one-shot check of a run without changing anything.

The checks read process and git state, never artifacts, so a supervisor cannot
mistake a finished output file for a finished process. Standard library only.
Set CORENIUS_TRAECLI to override the worker binary and CORENIUS_RUNS to move
the run directory root.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_FORBIDDEN = (
    r"\bgit\s+(?:-C\s+\S+\s+)?(?:commit|push|reset|checkout|switch|rebase|merge|stash|clean|am|apply|restore)\b",
    r"\b(?:curl|wget)\b",
    r"\b(?:pip3?|uv\s+pip)\s+install\b",
)

# Older runs recorded this pattern in run.json. `rm` is now judged by rm_findings():
# its harm is visible in git and file state, so text alone is not a violation.
LEGACY_RM = r"\brm\s+-[A-Za-z]*[rf]"

SEGMENT_SPLIT = re.compile(r"\|\||&&|[;|\n]")
HOME_VARS = ("$HOME", "${HOME}", "~")

RUNNING, CLEAN, DEGRADED, VIOLATED = 3, 0, 1, 2
VERDICT_NAMES = {CLEAN: "CLEAN", DEGRADED: "DEGRADED", VIOLATED: "VIOLATED"}


class WorkerError(Exception):
    pass


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
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)
    if check and result.returncode != 0:
        raise WorkerError(f"git {' '.join(args)} failed in {repo}: {result.stderr.strip()}")
    return result.stdout


def main_worktree(repo: str | Path) -> Path:
    common = git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").strip()
    return Path(common).parent


def runs_root() -> Path:
    override = os.environ.get("CORENIUS_RUNS")
    if override:
        return Path(override)
    cache = os.environ.get("XDG_CACHE_HOME") or str(Path.home() / ".cache")
    return Path(cache) / "corenius"


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    # A zombie still answers kill(0); the wrapper writes exit_code when done.
    return True


# --- launch -------------------------------------------------------------------


HEADER = """You are a worker agent, one of several roles in a lead/supervisor/worker setup.
Your working root is the git worktree {worktree} (branch {branch}). Read its
AGENTS.md first and follow it.

Hard rules for this run:
- You may create or edit only these paths, relative to the worktree: {allowed}.
  Any other change is a violation.
- Do not commit, push, merge, reset, stash or switch branches. The lead commits.
- These repositories are read-only for you: {protected}. Read them at their
  recorded commit with `git -C <repo> show <commit>:<path>`; make no git writes there.
- No network access and no package installs.
- Use a fresh unique directory (`mktemp -d` under an allowed path) for each experiment.
  Deleting outside the worktree, or with `..` or `~`, is a violation.
- Do not print secrets. If a rule blocks the task, stop and say so in your final message.
- Finish with a short final message: what you produced, what you did not check.

TASK
"""


def launch(args: argparse.Namespace) -> int:
    repo = main_worktree(args.repo)
    if not Path(args.task).is_file():
        raise WorkerError(f"task file {args.task} not found")
    task = Path(args.task).read_text()
    name = re.sub(r"[^a-z0-9-]", "-", args.name.lower()).strip("-")
    if not name:
        raise WorkerError("worker name is empty")
    branch = f"worker/{name}"
    worktree = repo.parent / f"{repo.name}-worker-{name}"
    if worktree.exists():
        raise WorkerError(f"{worktree} already exists; choose another name or remove it")
    git(repo, "worktree", "add", "-q", "-b", branch, str(worktree), args.base)
    base_sha = git(worktree, "rev-parse", "HEAD").strip()

    protected = []
    for path in [str(repo)] + list(args.protect or []):
        path = str(Path(path).resolve())
        protected.append(
            {
                "path": path,
                "head": git(path, "rev-parse", "HEAD").strip(),
                "dirty": git(path, "status", "--porcelain", "--untracked-files=no"),
            }
        )
    # The main checkout is protected as-is: the worker must not touch it.

    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run_dir = runs_root() / "workers" / f"{name}-{stamp}"
    run_dir.mkdir(parents=True)
    prompt = HEADER.format(
        worktree=worktree,
        branch=branch,
        allowed=", ".join(args.allow) or "(none: read-only run)",
        protected=", ".join(p["path"] for p in protected),
    ) + task
    (run_dir / "prompt.md").write_text(prompt)

    traecli = os.environ.get("CORENIUS_TRAECLI", "traecli")
    worker_cmd = [
        traecli, "exec",
        "-m", args.model,
        "-c", f"model_reasoning_effort={args.effort}",
        "-c", "model_reasoning_summary=detailed",
        "--json", "-s", args.sandbox,
        "-C", str(worktree),
        "-o", str(run_dir / "last-message.md"),
        "--shell-tool-timeout", "120s",
        "-",
    ]
    script = (
        f"timeout {int(args.cap)} {shlex.join(worker_cmd)} "
        f"< {shlex.quote(str(run_dir / 'prompt.md'))} "
        f"> {shlex.quote(str(run_dir / 'events.jsonl'))} "
        f"2> {shlex.quote(str(run_dir / 'stderr.log'))}; "
        f"echo $? > {shlex.quote(str(run_dir / 'exit_code'))}"
    )
    (run_dir / "events.jsonl").touch()
    (run_dir / "stderr.log").touch()
    process = subprocess.Popen(
        ["sh", "-c", script],
        cwd=worktree,
        start_new_session=True,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    run = {
        "name": name,
        "run_dir": str(run_dir),
        "worktree": str(worktree),
        "branch": branch,
        "base_sha": base_sha,
        "allowed": list(args.allow),
        "protected": protected,
        "forbidden": list(DEFAULT_FORBIDDEN) + list(args.forbid or []),
        "model": args.model,
        "effort": args.effort,
        "sandbox": args.sandbox,
        "cap_seconds": int(args.cap),
        "stall_seconds": int(args.stall),
        "pid": process.pid,
        "started": now(),
        "started_epoch": time.time(),
    }
    (run_dir / "run.json").write_text(json.dumps(run, indent=2) + "\n")
    print(json.dumps({"run_dir": str(run_dir), "worktree": str(worktree), "branch": branch, "pid": process.pid}))
    return 0


# --- checks -------------------------------------------------------------------


def load_run(run_dir: str | Path) -> dict:
    path = Path(run_dir) / "run.json"
    if not path.exists():
        raise WorkerError(f"{run_dir} is not a worker run (no run.json)")
    return json.loads(path.read_text())


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


class Checker:
    """Stateful boundary checks for one run; each cycle returns new findings."""

    def __init__(self, run: dict):
        self.run = run
        self.run_dir = Path(run["run_dir"])
        self.offset = 0
        self.events = 0
        self.seen: set[tuple[str, str]] = set()
        self.patterns = [re.compile(p) for p in run["forbidden"] if p != LEGACY_RM]

    def _finding(self, severity: str, kind: str, detail: str) -> dict | None:
        key = (kind, detail)
        if key in self.seen:
            return None
        self.seen.add(key)
        return {"severity": severity, "kind": kind, "detail": detail}

    def cycle(self, alive: bool) -> list[dict]:
        run, found = self.run, []

        def add(finding):
            if finding:
                found.append(finding)

        worktree = run["worktree"]
        head = git(worktree, "rev-parse", "HEAD", check=False).strip()
        if head != run["base_sha"]:
            add(self._finding("VIOLATION", "worker-committed", f"HEAD {head[:12]} != base {run['base_sha'][:12]}"))
        status = git(worktree, "status", "--porcelain", "--untracked-files=all", check=False)
        for path in porcelain_paths(status):
            if not allowed_path(path, run["allowed"]):
                add(self._finding("VIOLATION", "unexpected-change", path))

        for protected in run["protected"]:
            current_head = git(protected["path"], "rev-parse", "HEAD", check=False).strip()
            current_dirty = git(protected["path"], "status", "--porcelain", "--untracked-files=no", check=False)
            if current_head != protected["head"]:
                add(self._finding("VIOLATION", "protected-head-moved", f"{protected['path']} {current_head[:12]}"))
            if current_dirty != protected["dirty"]:
                add(self._finding("VIOLATION", "protected-dirty-changed", protected["path"]))

        events = self.run_dir / "events.jsonl"
        if events.exists():
            with events.open("rb") as handle:
                handle.seek(self.offset)
                chunk = handle.read()
            complete = chunk[: chunk.rfind(b"\n") + 1]
            self.offset += len(complete)
            for line in complete.decode("utf-8", "replace").splitlines():
                self.events += 1
                try:
                    event = json.loads(line)
                except ValueError:
                    continue
                item = event.get("item") or {}
                command = item.get("command") if item.get("type") == "command_execution" else None
                if command:
                    for severity, kind in rm_findings(command, run["worktree"]):
                        add(self._finding(severity, kind, command[:200]))
                    for pattern in self.patterns:
                        if pattern.search(command):
                            add(self._finding("VIOLATION", "forbidden-command", command[:200]))
                            break

        if alive and events.exists():
            quiet = time.time() - events.stat().st_mtime
            if quiet > run["stall_seconds"]:
                add(self._finding("STALL", "stall", f"no events for {int(quiet)}s"))
        stderr = self.run_dir / "stderr.log"
        if stderr.exists() and stderr.stat().st_size:
            add(self._finding("DEGRADED", "stderr", f"{stderr.stat().st_size} bytes in stderr.log"))
        return found


def process_state(run: dict) -> tuple[bool, int | None]:
    exit_file = Path(run["run_dir"]) / "exit_code"
    if exit_file.exists() and exit_file.read_text().strip():
        return False, int(exit_file.read_text().strip())
    return pid_alive(run["pid"]), None


def supervise(args: argparse.Namespace) -> int:
    run = load_run(args.run_dir)
    run_dir = Path(run["run_dir"])
    checker = Checker(run)
    findings: list[dict] = []
    started_here = time.time()
    status_log = run_dir / "status.jsonl"
    while True:
        alive, exit_code = process_state(run)
        new = checker.cycle(alive)
        findings.extend(new)
        elapsed = int(time.time() - run["started_epoch"])
        with status_log.open("a") as handle:
            handle.write(
                json.dumps(
                    {"t": now(), "elapsed": elapsed, "alive": alive, "events": checker.events,
                     "new": new}
                )
                + "\n"
            )
        if not alive:
            # One more pass now that the process has exited: catch late writes.
            findings.extend(checker.cycle(False))
            break
        if time.time() - started_here >= args.max_wait:
            print(json.dumps({"verdict": "RUNNING", "elapsed": elapsed, "events": checker.events,
                              "findings": findings}))
            return RUNNING
        time.sleep(args.interval)

    missing = [a for a in run["allowed"] if not (Path(run["worktree"]) / a).exists()]
    severities = {f["severity"] for f in findings} - {"NOTE"}
    if "VIOLATION" in severities:
        code = VIOLATED
    elif severities or exit_code not in (0,) or missing:
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
    run = load_run(args.run_dir)
    alive, exit_code = process_state(run)
    checker = Checker(run)
    findings = checker.cycle(alive)
    print(json.dumps({
        "alive": alive, "exit_code": exit_code, "events": checker.events,
        "elapsed_seconds": int(time.time() - run["started_epoch"]),
        "findings": findings,
        "verdict_file": (Path(run["run_dir"]) / "verdict.json").exists(),
    }))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("launch")
    p.add_argument("--task", required=True, help="file with the task text")
    p.add_argument("--name", required=True)
    p.add_argument("--allow", action="append", default=[], help="path (relative to the worktree) the worker may write; repeatable")
    p.add_argument("--protect", action="append", default=[], help="extra repo that must stay unchanged; repeatable")
    p.add_argument("--forbid", action="append", default=[], help="extra regex forbidden in worker commands")
    p.add_argument("--repo", default=".")
    p.add_argument("--base", default="main")
    p.add_argument("--model", default="GPT-5.5")
    p.add_argument("--effort", default="xhigh")
    p.add_argument("--sandbox", default="workspace-write")
    p.add_argument("--cap", type=int, default=2700, help="hard wall-clock cap in seconds")
    p.add_argument("--stall", type=int, default=300, help="seconds without events before STALL")
    p.set_defaults(func=launch)
    p = sub.add_parser("supervise")
    p.add_argument("run_dir")
    p.add_argument("--interval", type=float, default=30)
    p.add_argument("--max-wait", type=float, default=540, help="return RUNNING (exit 3) after this many seconds")
    p.set_defaults(func=supervise)
    p = sub.add_parser("status")
    p.add_argument("run_dir")
    p.set_defaults(func=status)
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except WorkerError as error:
        print(f"worker: {error}", file=sys.stderr)
        return 64


if __name__ == "__main__":
    raise SystemExit(main())
