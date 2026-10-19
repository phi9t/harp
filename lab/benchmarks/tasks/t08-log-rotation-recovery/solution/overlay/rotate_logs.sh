#!/usr/bin/env bash
set -euo pipefail

dir="${1:?usage: rotate_logs.sh LOG_DIR RETAIN}"
retain="${2:?usage: rotate_logs.sh LOG_DIR RETAIN}"
mkdir -p "$dir"

current="$dir/app.log"
stamp="${ROTATE_NOW:-$(date -u +%Y%m%d%H%M%S)}"

if [ -f "$current" ]; then
  base="$dir/app.log.$stamp"
  archive="$base"
  suffix=1
  while [ -e "$archive" ]; do
    archive="$base.$suffix"
    suffix=$((suffix + 1))
  done
  mv "$current" "$archive"
fi
: > "$current"

mapfile -t archives < <(find "$dir" -maxdepth 1 -type f -name 'app.log.*' | sort)
extra=$((${#archives[@]} - retain))
if [ "$extra" -gt 0 ]; then
  for ((i = 0; i < extra; i++)); do
    rm -f "${archives[$i]}"
  done
fi
