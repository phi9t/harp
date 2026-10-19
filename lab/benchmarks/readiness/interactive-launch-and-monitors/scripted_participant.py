#!/usr/bin/env python3
"""Scripted interactive runtime for the interactive launch readiness scenario."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path


def git(*args: str) -> str:
    proc = subprocess.run(["git", *args], capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip())
    return proc.stdout


def commit(message: str, *paths: str, allow_empty: bool = False) -> str:
    if paths:
        git("add", *paths)
    args = ["-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "-q", "-m", message]
    if allow_empty:
        args.insert(-2, "--allow-empty")
    git(*args)
    return git("rev-parse", "HEAD").strip()


def prompt_text() -> str:
    return sys.argv[-1] if sys.argv[1:] else ""


def ticket_id(prompt: str) -> str:
    match = re.search(r"^Ticket:\s+(\d\d)-", prompt, re.M)
    return match.group(1) if match else "01"


def marker(ticket: str) -> str:
    return f"ticket {ticket}: ready for review"


def config_values(argv: list[str]) -> list[str]:
    values = []
    index = 0
    while index < len(argv):
        arg = argv[index]
        if arg == "-c" and index + 1 < len(argv):
            values.append(argv[index + 1])
            index += 2
            continue
        if arg.startswith("-c") and len(arg) > 2:
            values.append(arg[2:])
        index += 1
    return values


def trust_config(workspace: str) -> str:
    escaped = workspace.replace("\\", "\\\\").replace('"', '\\"')
    return f'projects={{"{escaped}"={{trust_level="trusted"}}}}'


def trust_override_applies(argv: list[str], workspace: str) -> bool:
    configs = config_values(argv)
    if any(value.startswith("projects.") for value in configs):
        return False
    return trust_config(workspace) in configs


def signal_dir() -> Path:
    return Path(os.environ["CORENIUS_INTERACTIVE_SIGNAL_DIR"])


def current_run_events_file() -> Path | None:
    runs = Path(os.environ.get("CORENIUS_RUNS", ""))
    if not runs.is_dir():
        return None
    worktree = str(Path.cwd().resolve())
    for run_json in sorted(runs.glob("workers/*/run.json")):
        try:
            run = json.loads(run_json.read_text())
        except (OSError, ValueError):
            continue
        if str(Path(run.get("worktree", "")).resolve()) == worktree:
            return Path(run.get("active_events_file") or run_json.parent / "events.jsonl")
    return None


def record_findings_read(message: str) -> None:
    path = current_run_events_file()
    if path is None:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as handle:
        handle.write(json.dumps({"type": "item.completed", "item": {"type": "command_execution", "command": message}}) + "\n")


def wait_for(path: Path, timeout: float = 60.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.exists():
            return
        time.sleep(0.05)
    raise RuntimeError(f"timed out waiting for {path}")


def write_signal(path: Path, text: str) -> None:
    path.write_text(text + "\n")


def print_trust_prompt(workspace: str) -> None:
    print("Folder access", flush=True)
    print(workspace, flush=True)
    print(
        "Do you trust the contents of this directory? Working with untrusted contents comes with higher risk of",
        flush=True,
    )
    print(
        "prompt injection. Trusting the directory allows project-local config, hooks, and exec policies to load.",
        flush=True,
    )
    print("> 1. Yes, continue", flush=True)
    print("  2. No, quit", flush=True)
    print("enter continue  |  esc quit", flush=True)


def read_review_line(timeout: float = 15.0) -> str:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        print("Current activity: waiting for review findings", flush=True)
        print("Context remaining: 87%", flush=True)
        print("esc to interrupt", flush=True)
        line = sys.stdin.readline()
        if line:
            print("\x1b[2J\x1b[H", end="", flush=True)
            return line
        time.sleep(0.05)
    raise RuntimeError("timed out waiting for review findings")


def ticket_01(signals: Path) -> int:
    write_signal(signals / "started-01.json", "started")
    wait_for(signals / "release-initial-01")
    print("Current activity: preparing first candidate", flush=True)
    print("Context remaining: 91%", flush=True)
    Path("result.txt").write_text("needs rework\n")
    Path("docs/efforts/demo/evidence/01-report.md").parent.mkdir(parents=True, exist_ok=True)
    Path("docs/efforts/demo/evidence/01-report.md").write_text("# Report\n\n## For later tickets\nnone\n")
    commit(marker("01"), "result.txt", "docs/efforts/demo/evidence/01-report.md")
    write_signal(signals / "first-marker-01.json", git("rev-parse", "HEAD").strip())
    review = read_review_line()
    if "review-01.md" not in review:
        raise RuntimeError("expected review findings path")
    record_findings_read(review)
    print(f"Current activity: received {review.strip()}", flush=True)
    print("Context remaining: 86%", flush=True)
    wait_for(signals / "release-fix-01")
    Path("result.txt").write_text("fixed before re-integration\n")
    commit("fix review findings", "result.txt")
    write_signal(signals / "fix-committed-01.json", git("rev-parse", "HEAD").strip())
    stale = read_review_line()
    if "stale-base-01.md" not in stale:
        raise RuntimeError("expected stale-base findings path")
    record_findings_read(stale)
    print(f"Current activity: received {stale.strip()}", flush=True)
    print("Context remaining: 85%", flush=True)
    import os

    integration = os.environ["CORENIUS_INTERACTIVE_INTEGRATION"]
    git("rebase", integration)
    commit(marker("01"), allow_empty=True)
    write_signal(signals / "stale-received-01.json", "received")
    write_signal(signals / "second-marker-01.json", git("rev-parse", "HEAD").strip())
    print("Current activity: idle after marker", flush=True)
    print("Context remaining: 84%", flush=True)
    while True:
        time.sleep(1)


def ticket_02(signals: Path) -> int:
    write_signal(signals / "started-02.json", "started")
    wait_for(signals / "release-initial-02")
    Path("needs-info.txt").write_text("need a decision\n")
    commit("ticket 02: needs-info", "needs-info.txt")
    print("Current activity: needs info", flush=True)
    print("Context remaining: 90%", flush=True)
    while True:
        time.sleep(1)


def ticket_03(signals: Path) -> int:
    write_signal(signals / "started-03.json", "started")
    wait_for(signals / "release-initial-03")
    print("Current activity: stopping without handoff", flush=True)
    print("Context remaining: 89%", flush=True)
    return 42


def ticket_04(signals: Path) -> int:
    write_signal(signals / "started-04.json", "started")
    Path("prompt.txt").write_text("trusted\n")
    commit(marker("04"), "prompt.txt")
    print("Current activity: idle after marker", flush=True)
    print("Context remaining: 88%", flush=True)
    while True:
        time.sleep(1)


def main() -> int:
    import os

    signals = signal_dir()
    signals.mkdir(parents=True, exist_ok=True)
    ticket = ticket_id(prompt_text())
    if not trust_override_applies(sys.argv[1:], os.getcwd()):
        write_signal(signals / f"started-{ticket}.json", "started")
        print_trust_prompt(os.getcwd())
        sys.stdin.readline()
    if ticket == "01":
        return ticket_01(signals)
    if ticket == "02":
        return ticket_02(signals)
    if ticket == "03":
        return ticket_03(signals)
    if ticket == "04":
        return ticket_04(signals)
    return 64


if __name__ == "__main__":
    raise SystemExit(main())
