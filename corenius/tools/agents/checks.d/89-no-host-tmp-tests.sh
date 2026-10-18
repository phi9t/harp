#!/bin/sh
set -eu
cd "$(dirname "$0")/../../.." || exit 2
PYTHONDONTWRITEBYTECODE=1 exec python3 tools/agents/test_no_host_tmp.py
