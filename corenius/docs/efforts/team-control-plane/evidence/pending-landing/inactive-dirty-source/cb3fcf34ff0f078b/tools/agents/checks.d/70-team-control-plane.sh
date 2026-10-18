#!/bin/sh
# Requires Docker and a disposable loopback PostgreSQL fixture; fails if unavailable.
set -eu
cd "$(dirname "$0")/../../.." || exit 2
exec bash crates/team-control-plane/tests/run-postgres.sh
