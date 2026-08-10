#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && /bin/pwd -P)
cleaner="$repo_root/scripts/clean_build.sh"

if [[ ! -x "$cleaner" ]]; then
  printf 'cleanup helper is missing or not executable: %s\n' "$cleaner" >&2
  exit 1
fi

fixture=$(mktemp -d "${TMPDIR:-/tmp}/harp-clean-build.XXXXXX")
fixture=$(cd -P -- "$fixture" && /bin/pwd -P)
cleanup() {
  if [[ -n "${cleaner_pid:-}" ]]; then
    kill "$cleaner_pid" 2>/dev/null || true
    wait "$cleaner_pid" 2>/dev/null || true
  fi
  rm -rf -- "$fixture"
}
trap cleanup EXIT

config_root="$fixture/worktree"
outside_root="$fixture/outside"
original_build="$config_root/.build"
ready_fifo="$fixture/ready"
continue_fifo="$fixture/continue"
log="$fixture/cleaner.log"

mkdir -p \
  "$original_build/harp-target" \
  "$original_build/codex-inprocess-target" \
  "$outside_root/harp-target" \
  "$outside_root/codex-inprocess-target"
touch "$outside_root/harp-target/sentinel" "$outside_root/codex-inprocess-target/sentinel"

HARP_CLEAN_BUILD_TESTING=1 "$cleaner" \
  "$config_root" \
  "$original_build/harp-target" \
  "$original_build/codex-inprocess-target" \
  --test-sync "$ready_fifo" "$continue_fifo" >"$log" 2>&1 &
cleaner_pid=$!

for _ in {1..500}; do
  if [[ -e "$ready_fifo" ]]; then
    break
  fi
  if ! kill -0 "$cleaner_pid" 2>/dev/null; then
    printf 'cleanup exited before synchronization\n' >&2
    cat "$log" >&2
    exit 1
  fi
  sleep 0.01
done
if [[ ! -e "$ready_fifo" ]]; then
  printf 'cleanup synchronization timed out\n' >&2
  cat "$log" >&2
  exit 1
fi

mv -- "$original_build" "$config_root/.build.original"
ln -s -- "$outside_root" "$original_build"
: >"$continue_fifo"

set +e
wait "$cleaner_pid"
cleaner_status=$?
set -e
cleaner_pid=

if ((cleaner_status == 0)); then
  printf 'cleanup unexpectedly succeeded after parent-directory replacement\n' >&2
  cat "$log" >&2
  exit 1
fi

for sentinel in \
  "$outside_root/harp-target/sentinel" \
  "$outside_root/codex-inprocess-target/sentinel"; do
  if [[ ! -f "$sentinel" ]]; then
    printf 'cleanup escaped the worktree and deleted %s\n' "$sentinel" >&2
    cat "$log" >&2
    exit 1
  fi
done

printf 'cleanup parent-swap regression: PASS\n'
