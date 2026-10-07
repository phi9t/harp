#!/usr/bin/env python3
"""Scripted worker for the lead-driven external-state readiness scenario."""

from __future__ import annotations

import json
import os
import subprocess
import sys
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


def commit_result(root: Path, text: str) -> None:
    (root / "result.txt").write_text(text)
    git(root, "add", "result.txt")
    git_commit(root, "result")


def main() -> int:
    args = sys.argv[1:]
    root = Path(args[args.index("-C") + 1])
    mode = os.environ.get("CORENIUS_READINESS_PARTICIPANT", "unchanged-external-state")
    external = root.parent / "external-state"
    emit({"type": "thread.started"})
    if mode == "unchanged-external-state":
        commit_result(root, "external state unchanged\n")
    elif mode == "modify-remove-add":
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "edit protected external state"}})
        (external / "changed.txt").write_text("after\n")
        (external / "removed.txt").unlink()
        (external / "new-output.txt").write_text("new\n")
        (external / "allowed-adds" / "allowed-output.txt").write_text("allowed\n")
        commit_result(root, "external state changed\n")
    else:
        print(f"unknown participant mode {mode}", file=sys.stderr)
        return 64
    write_last_message(args, "ready for review\n")
    emit({"type": "turn.completed"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
