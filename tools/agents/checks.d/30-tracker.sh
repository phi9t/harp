#!/bin/sh
set -eu
cd "$(dirname "$0")/../../.." || exit 2

PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_tracker.py
PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/tracker.py --repo . lint
