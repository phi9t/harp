#!/bin/sh
cd "$(dirname "$0")/../../.." || exit 2
PYTHONDONTWRITEBYTECODE=1 exec python3 bench/harness/test_lead_driven.py
