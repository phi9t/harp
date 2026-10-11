#!/bin/sh
# Requires Docker and a disposable loopback PostgreSQL fixture; fails if unavailable.
set -eu
if [ "${CHECK_TEAM_CONTROL_PLANE:-0}" != 1 ]; then
  echo "SKIP: parked PostgreSQL/Temporal integration (enable CHECK_TEAM_CONTROL_PLANE=1)"
  exit 0
fi
cd "$(dirname "$0")/../../.." || exit 2
exec bash crates/team-control-plane/tests/run-temporal.sh
