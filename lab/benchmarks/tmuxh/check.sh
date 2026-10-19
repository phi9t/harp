#!/bin/sh
# Run every tmuxh test (tmux integration tests need libtmux: they run through the uv project).
set -e
dir="$(cd "$(dirname "$0")" && pwd)"
for t in test_state_stream test_pane test_dashboard test_scheduler; do
  printf '%-22s' "$t"
  uv run --quiet --project "$dir" python "$dir/$t.py" 2>&1 | grep -E '^(OK|FAILED|Ran)' | tr '\n' ' '
  echo
done
