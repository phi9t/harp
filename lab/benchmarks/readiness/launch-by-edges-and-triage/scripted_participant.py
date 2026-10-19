#!/usr/bin/env python3
"""Fake worker binary for lead-launch readiness scenarios."""

from __future__ import annotations

import json
import os
import sys
import time


def main() -> int:
    print(json.dumps({"type": "thread.started"}), flush=True)
    if os.environ.get("CORENIUS_READINESS_PARTICIPANT", "sleep") == "sleep":
        time.sleep(30)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
