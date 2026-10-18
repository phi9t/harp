#!/usr/bin/env bash
set -euo pipefail

dir="${1:?usage: rotate_logs.sh LOG_DIR RETAIN}"
retain="${2:?usage: rotate_logs.sh LOG_DIR RETAIN}"
mkdir -p "$dir"

current="$dir/app.log"
stamp="${ROTATE_NOW:-$(date -u +%Y%m%d%H%M%S)}"
archive="$dir/app.log.$stamp"

: > "$current"
cp "$current" "$archive"

archives="$(find "$dir" -maxdepth 1 -type f -name 'app.log.*' | sort | tail -n "$retain" || true)"
if [ -n "$archives" ]; then
  printf '%s\n' "$archives" | xargs -r rm -f
fi
