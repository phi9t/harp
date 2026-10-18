#!/bin/sh
# Disposable registrations and command shims prevent recursive gate execution.
set -eu
PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_check.py
PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/check_test.py
