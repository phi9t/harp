#!/bin/sh
# Public no-effect retirement behavior, plus retained parser/diagnostic boundaries.
set -eu
PYTHONDONTWRITEBYTECODE=1 exec python3 tools/agents/test_wt.py
