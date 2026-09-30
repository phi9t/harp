#!/bin/sh
set -eu

python3 tools/agents/test_tracker.py
python3 tools/agents/tracker.py --repo . lint agentic-engineering
