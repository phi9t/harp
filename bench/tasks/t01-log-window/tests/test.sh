#!/usr/bin/env bash
set -euo pipefail
out="$(bash metrics.sh events.log 3 4)"
test "$out" = "errors=1 warnings=1"
