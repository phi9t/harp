#!/usr/bin/env python3
"""Fake worker binary for the integration-branch readiness scenario."""

from __future__ import annotations

import json
import time


def main() -> int:
    print(json.dumps({"type": "thread.started"}), flush=True)
    time.sleep(30)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
