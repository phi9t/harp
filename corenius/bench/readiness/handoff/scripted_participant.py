#!/usr/bin/env python3
"""Fake worker binary for lead-driven readiness scenarios."""

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


def main() -> int:
    args = sys.argv[1:]
    root = Path(args[args.index("-C") + 1])
    mode = os.environ.get("CORENIUS_READINESS_PARTICIPANT", "commit-ready")
    emit({"type": "thread.started"})
    if mode == "commit-ready":
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "write result.txt"}})
        (root / "result.txt").write_text("ready\n")
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "git add result.txt && git commit -m result"}})
        git(root, "add", "result.txt")
        git_commit(root, "result")
        write_last_message(args, "ready for review\n")
    elif mode == "dirty-final-message":
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "write result.txt without committing"}})
        (root / "result.txt").write_text("dirty\n")
        write_last_message(args, "ready for review\n")
    elif mode == "no-commit":
        write_last_message(args, "ready for review\n")
    elif mode == "move-foreign-branch":
        (root / "result.txt").write_text("moved branch\n")
        git(root, "add", "result.txt")
        git_commit(root, "result")
        head = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
        git(root, "branch", "-f", "side", head)
        write_last_message(args, "ready for review\n")
    elif mode == "create-foreign-branch":
        (root / "result.txt").write_text("created branch\n")
        git(root, "add", "result.txt")
        git_commit(root, "result")
        head = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
        git(root, "branch", "new-side", head)
        write_last_message(args, "ready for review\n")
    elif mode == "rewrite-base":
        (root / "result.txt").write_text("rewritten\n")
        git(root, "add", "result.txt")
        tree = subprocess.check_output(["git", "-C", str(root), "write-tree"], text=True).strip()
        new = subprocess.check_output(
            ["git", "-C", str(root), "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit-tree", tree, "-m", "orphan"],
            text=True,
        ).strip()
        branch = subprocess.check_output(["git", "-C", str(root), "branch", "--show-current"], text=True).strip()
        git(root, "update-ref", f"refs/heads/{branch}", new)
        git(root, "read-tree", "--reset", "-u", new)
        write_last_message(args, "ready for review\n")
    elif mode in ("marker-ready", "marker-rewrite"):
        marker = os.environ.get("CORENIUS_HANDOFF_MARKER", "ticket 01: ready for review")
        (root / "result.txt").write_text("marker\n")
        git(root, "add", "result.txt")
        git_commit(root, marker)
        write_last_message(args, "ready for review\n")
        if mode == "marker-ready":
            time.sleep(30)
        elif mode == "marker-rewrite":
            time.sleep(2)
            (root / "result.txt").write_text("marker moved\n")
            git(root, "add", "result.txt")
            git(root, "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "--amend", "-q", "-m", "moved")
            time.sleep(30)
    elif mode == "marker-dirty":
        marker = os.environ.get("CORENIUS_HANDOFF_MARKER", "ticket 01: ready for review")
        (root / "result.txt").write_text("marker clean\n")
        git(root, "add", "result.txt")
        git_commit(root, "pre-marker")
        (root / "result.txt").write_text("marker dirty\n")
        git(root, "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "--allow-empty", "-q", "-m", marker)
        write_last_message(args, "ready for review\n")
    else:
        print(f"unknown participant mode {mode}", file=sys.stderr)
        return 64
    time.sleep(0.1)
    emit({"type": "turn.completed"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
