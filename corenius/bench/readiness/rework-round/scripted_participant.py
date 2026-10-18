#!/usr/bin/env python3
"""Fake worker binary for lead-driven rework readiness scenarios."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path


def emit(event: dict) -> None:
    print(json.dumps(event), flush=True)


def git(root: Path, *args: str) -> None:
    subprocess.run(["git", "-C", str(root), *args], check=True)


def git_commit(root: Path, message: str) -> None:
    git(root, "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "-q", "-m", message)


def write_last_message(args: list[str], text: str) -> None:
    if "-o" in args:
        Path(args[args.index("-o") + 1]).write_text(text)


def marker() -> str:
    return os.environ.get("CORENIUS_HANDOFF_MARKER", "ticket 01: ready for review")


def launch(args: list[str]) -> int:
    root = Path(args[args.index("-C") + 1])
    emit({"type": "thread.started", "thread_id": "rework-session-1"})
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "write initial result.txt"}})
    (root / "result.txt").write_text("needs rework\n")
    git(root, "add", "result.txt")
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "git commit marker"}})
    git_commit(root, marker())
    write_last_message(args, "ready for review\n")
    time.sleep(0.1)
    emit({"type": "turn.completed"})
    return 0


def resume(args: list[str]) -> int:
    root = Path.cwd()
    message = sys.stdin.read()
    emit({"type": "thread.started", "thread_id": args[2]})
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": f"read findings {message}"}})
    if "Findings sha256:" not in message:
        print("findings digest missing", file=sys.stderr)
        return 64
    (root / "result.txt").write_text("ready after rework\n")
    git(root, "add", "result.txt")
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "git commit fix"}})
    git_commit(root, "fix review findings")
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "git commit marker"}})
    git(root, "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "--allow-empty", "-q", "-m", marker())
    write_last_message(args, "ready for review\n")
    time.sleep(0.1)
    emit({"type": "turn.completed"})
    return 0


def main() -> int:
    args = sys.argv[1:]
    if args[:2] == ["exec", "resume"]:
        return resume(args)
    return launch(args)


if __name__ == "__main__":
    raise SystemExit(main())
