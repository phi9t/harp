#!/bin/sh
# Historical projection validation is pure; do not query current old tracker state.
set -eu
PYTHONDONTWRITEBYTECODE=1 exec python3 tools/agents/test_operational_state.py
