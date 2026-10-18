#!/bin/sh
cd "$(dirname "$0")/../../.." || exit 2
PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py
