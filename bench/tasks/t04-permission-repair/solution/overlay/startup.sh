#!/usr/bin/env bash
set -euo pipefail

root="${1:?usage: startup.sh ROOT}"
run_dir="$root/run"
sessions_dir="$run_dir/sessions"

mkdir -p "$sessions_dir"
chmod 0755 "$run_dir"
chmod 1777 "$sessions_dir"

printf 'runtime=%s\n' "$run_dir"
