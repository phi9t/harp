#!/usr/bin/env python3
"""Fake worker binary for exact-candidate landing readiness scenarios."""

from __future__ import annotations

import json
import os
import sys
import time
import subprocess
from pathlib import Path


def emit(event: dict) -> None:
    print(json.dumps(event), flush=True)


def main() -> int:
    args = sys.argv[1:]
    root = Path(args[args.index("-C") + 1])
    mode = os.environ.get("CORENIUS_READINESS_PARTICIPANT", "")
    emit({"type": "thread.started"})
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "write result.txt"}})
    (root / "result.txt").write_text("ready\n")
    paths = ["result.txt"]
    if mode == "ticket-tamper":
        ticket = root / "docs" / "efforts" / "readiness" / "tickets" / "01-acceptance.md"
        ticket.write_text(ticket.read_text().replace("Status: claimed", "Status: resolved").replace("- [ ] works", "- [x] works"))
        paths.append("docs/efforts/readiness/tickets/01-acceptance.md")
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "edit ticket status and acceptance items"}})
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "git add allowed files && git commit -m result"}})
    subprocess.run(["git", "-C", str(root), "add", *paths], check=True)
    subprocess.run(["git", "-C", str(root), "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "-q", "-m", "result"], check=True)
    if "-o" in args:
        Path(args[args.index("-o") + 1]).write_text("ready for review\n")
    time.sleep(0.1)
    emit({"type": "turn.completed"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
