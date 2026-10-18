#!/usr/bin/env python3
"""Fake worker binary for the parallel-overlap readiness scenario."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path


def emit(event: dict) -> None:
    print(json.dumps(event), flush=True)


def ticket_id(prompt: str, root: Path) -> str:
    match = re.search(r"^Ticket:\s+(\d\d)-", prompt, re.M)
    if match:
        return match.group(1)
    return "01" if "01-alpha" in str(root) else "02"


def wait_for(path: Path, timeout: float = 10.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.exists():
            return
        time.sleep(0.05)
    raise RuntimeError(f"timed out waiting for {path}")


def git_commit(root: Path, path: str, message: str) -> None:
    subprocess.run(["git", "-C", str(root), "add", path], check=True)
    subprocess.run(
        ["git", "-C", str(root), "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "-q", "-m", message],
        check=True,
    )


def main() -> int:
    args = sys.argv[1:]
    if args[:2] == ["exec", "resume"]:
        root = Path.cwd()
        integration = os.environ["CORENIUS_PARALLEL_INTEGRATION"]
        session = args[2]
        message = sys.stdin.read()
        emit({"type": "thread.started", "thread_id": session})
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": f"read findings {message}"}})
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": f"git rebase {integration}"}})
        subprocess.run(["git", "-C", str(root), "rebase", integration], check=True)
        if "-o" in args:
            Path(args[args.index("-o") + 1]).write_text("ready for review after rebase\n")
        emit({"type": "turn.completed"})
        return 0

    root = Path(args[args.index("-C") + 1])
    prompt = sys.stdin.read()
    ticket = ticket_id(prompt, root)
    marker = os.environ.get("CORENIUS_HANDOFF_MARKER", f"ticket {ticket}: ready for review")
    signals = Path(os.environ["CORENIUS_PARALLEL_SIGNAL_DIR"])
    signals.mkdir(parents=True, exist_ok=True)

    emit({"type": "thread.started", "thread_id": f"parallel-{ticket}"})
    if ticket == "01":
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "write temporary overlap files"}})
        (root / "outside.txt").write_text("outside ticket 01\n")
        (root / "beta.txt").write_text("ticket 01 touched ticket 02 output\n")
        (root / "shared-undecided.txt").write_text("ticket 01 shared change\n")
    else:
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "write shared temporary overlap file"}})
        (root / "shared-undecided.txt").write_text("ticket 02 shared change\n")
    (signals / f"started-{ticket}.json").write_text(json.dumps({"ticket": ticket, "time": time.time()}) + "\n")
    wait_for(signals / "release")

    for path in ("outside.txt", "shared-undecided.txt"):
        (root / path).unlink(missing_ok=True)
    if ticket == "01":
        (root / "beta.txt").unlink(missing_ok=True)
        output = "alpha.txt"
        text = "alpha ready\n"
    else:
        output = "beta.txt"
        text = "beta ready\n"
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": f"write and commit {output}"}})
    (root / output).write_text(text)
    git_commit(root, output, marker)
    if "-o" in args:
        Path(args[args.index("-o") + 1]).write_text("ready for review\n")
    (signals / f"candidate-{ticket}.json").write_text(json.dumps({"ticket": ticket, "time": time.time()}) + "\n")
    time.sleep(3)
    emit({"type": "turn.completed"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
