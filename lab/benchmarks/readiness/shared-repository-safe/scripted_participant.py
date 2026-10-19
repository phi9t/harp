#!/usr/bin/env python3
"""Fake worker for the shared-repository safety scenario."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path


def emit(event: dict) -> None:
    print(json.dumps(event), flush=True)


def main() -> int:
    args = sys.argv[1:]
    root = Path(args[args.index("-C") + 1])
    mode = os.environ.get("CORENIUS_READINESS_PARTICIPANT", "slow-commit")
    emit({"type": "thread.started"})
    if mode != "slow-commit":
        print(f"unknown participant mode {mode}", file=sys.stderr)
        return 64
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "write out.md"}})
    (root / "out.md").write_text("ready\n")
    subprocess.run(["git", "-C", str(root), "add", "out.md"], check=True)
    subprocess.run(
        ["git", "-C", str(root), "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "-q", "-m", "result"],
        check=True,
    )
    if "-o" in args:
        Path(args[args.index("-o") + 1]).write_text("ready for review\n")
    time.sleep(2)
    emit({"type": "turn.completed"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
