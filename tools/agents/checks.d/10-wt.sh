#!/bin/sh
cd "$(dirname "$0")/../../.." || exit 2
exec python3 tools/agents/test_wt.py
