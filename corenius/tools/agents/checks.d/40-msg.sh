#!/bin/sh
set -eu
cd "$(dirname "$0")/../../.." || exit 2
"${PYTHON:-python3}" tools/agents/test_msg.py
