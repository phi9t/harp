#!/bin/sh
# State consistency: operational_state tests, then the live check of CURRENT.json against STATE.md.
set -eu
PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_operational_state.py
PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/operational_state.py
