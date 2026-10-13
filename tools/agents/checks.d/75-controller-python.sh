#!/bin/sh
# Controller Python support: launch, operator and support tests that cargo test does not run.
set -eu
PYTHONDONTWRITEBYTECODE=1 python3 controller/launch/sqlite_baseline_test.py
PYTHONDONTWRITEBYTECODE=1 python3 controller/launch/sqlite_responses_relay_test.py
PYTHONDONTWRITEBYTECODE=1 python3 controller/operator/mac_dashboard_test.py
PYTHONDONTWRITEBYTECODE=1 python3 controller/operator/mac_state_view_test.py
PYTHONDONTWRITEBYTECODE=1 python3 controller/operator/mac_tmux_test.py
PYTHONDONTWRITEBYTECODE=1 python3 controller/tests/support/test_correction_authority.py
