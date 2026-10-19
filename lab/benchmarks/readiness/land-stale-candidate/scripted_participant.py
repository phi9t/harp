#!/usr/bin/env python3
"""Fake worker binary for stale-candidate landing readiness scenarios."""

from __future__ import annotations

import json
import sys
import time
import subprocess
from pathlib import Path


def emit(event: dict) -> None:
    print(json.dumps(event), flush=True)


def main() -> int:
    args = sys.argv[1:]
    root = Path(args[args.index("-C") + 1])
    emit({"type": "thread.started"})
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "write result.txt"}})
    (root / "result.txt").write_text("ready\n")
    emit({"type": "item.completed", "item": {"type": "command_execution", "command": "git add result.txt && git commit -m result"}})
    subprocess.run(["git", "-C", str(root), "add", "result.txt"], check=True)
    subprocess.run(["git", "-C", str(root), "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "-q", "-m", "result"], check=True)
    if "-o" in args:
        Path(args[args.index("-o") + 1]).write_text("ready for review\n")
    time.sleep(0.1)
    emit({"type": "turn.completed"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
