#!/bin/sh
# Repository gate for the agentic-engineering tools (features spec, "Cross-cutting acceptance").
#   tools/agents/check.sh            existing suites, the tracker lint once it exists, every checks.d script
#   CHECK_BENCH=1 ...                also the benchmark harness and capsule suites
#   CHECK_SYNC=1 ...                 also `sync_agentic_workflow.py --check` (depends on the sibling repos' state)
#   CHECK_CARGO=0 ...                skip `cargo test --offline` (on by default since the AIME work landed, c1f458b)
# Each feature adds one script, tools/agents/checks.d/NN-name.sh, that runs its own suite; nothing else in this file changes per feature.
cd "$(dirname "$0")/../.." || exit 2
fail=0
run() {
  printf '== %s\n' "$*"
  "$@" || { echo "FAILED: $*"; fail=1; }
}
run python3 tools/agents/test_worker.py
run python3 tools/agents/test_effort.py
run python3 tools/test_sync_agentic_workflow.py
[ -f tools/agents/tracker.py ] && run python3 tools/agents/tracker.py lint
for c in tools/agents/checks.d/*.sh; do
  [ -f "$c" ] && run sh "$c"
done
if [ "${CHECK_BENCH:-0}" = 1 ]; then
  for t in bench/harness/test_*.py tools/insula/test_capsule.py; do run python3 "$t"; done
fi
[ "${CHECK_SYNC:-0}" = 1 ] && run python3 tools/sync_agentic_workflow.py --check
[ "${CHECK_CARGO:-1}" = 1 ] && run cargo test --offline
if [ "$fail" = 0 ]; then echo "check: all green"; else echo "check: FAILED"; fi
exit $fail
