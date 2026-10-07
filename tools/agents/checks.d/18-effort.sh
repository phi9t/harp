#!/bin/sh
cd "$(dirname "$0")/../../.." || exit 2
PYTHONDONTWRITEBYTECODE=1 exec python3 tools/agents/test_effort.py
