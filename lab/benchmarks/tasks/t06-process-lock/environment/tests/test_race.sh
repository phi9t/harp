#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

tmp_root="$PWD/.test-tmp"
mkdir -p "$tmp_root"

for attempt in 1 2 3; do
  state="$(mktemp -d -p "$tmp_root" race.XXXXXX)"
  output="$(bash run_workers.sh "$state")" || fail "run_workers failed on attempt $attempt"
  echo "$output" | grep -qx 'acquired=1 busy=1' || fail "unexpected summary on attempt $attempt: $output"
  test ! -e "$state/worker.lock" || fail "lock path remained after attempt $attempt"
  test -f "$state/results.txt" || fail "missing results file on attempt $attempt"
  lines="$(wc -l < "$state/results.txt" | tr -d ' ')"
  test "$lines" = "1" || fail "expected one result line on attempt $attempt, got $lines"
done
