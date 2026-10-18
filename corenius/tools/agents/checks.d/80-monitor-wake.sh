#!/bin/sh
# The gate runs each checks.d script from the repository root.
exec python3 tools/agents/test_monitor_wake.py
