#!/bin/sh
# Controller Python support: relay and correction-authority tests that cargo test does not run.
set -eu
PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_sqlite_responses_relay.py
PYTHONDONTWRITEBYTECODE=1 python3 crates/sqlite-controller/tests/support/test_correction_authority.py
