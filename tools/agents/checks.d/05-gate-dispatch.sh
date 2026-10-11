#!/bin/sh
# Gate dispatch: proves check.sh runs every checks.d script and honors its opt-in switches.
set -eu
PYTHONDONTWRITEBYTECODE=1 exec python3 tools/agents/test_check.py
