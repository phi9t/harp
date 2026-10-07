#!/usr/bin/env python3
"""Fake worker binary for the bookkeeping readiness scenario."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path


def main() -> int:
    args = sys.argv[1:]
    root = Path(args[args.index("-C") + 1])
    effort = os.environ.get("CORENIUS_BOOKKEEPING_EFFORT", "demo")
    print(json.dumps({"type": "thread.started"}), flush=True)
    (root / "result.txt").write_text("ticket one\n")
    report = root / "docs" / "efforts" / effort / "evidence" / "01-report.md"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(
        "# Ticket 01 report\n\n"
        "## For later tickets\n"
        "- to 02: Imported report finding.\n"
    )
    subprocess.run(["git", "-C", str(root), "add", "result.txt"], check=True)
    subprocess.run(["git", "-C", str(root), "add", f"docs/efforts/{effort}/evidence/01-report.md"], check=True)
    subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "-c",
            "user.email=bench@corenius",
            "-c",
            "user.name=bench",
            "commit",
            "-q",
            "-m",
            "ticket 01: ready for review",
        ],
        check=True,
    )
    print(json.dumps({"type": "item.completed", "item": {"type": "command_execution", "command": "git commit ticket 01"}}), flush=True)
    time.sleep(30)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
