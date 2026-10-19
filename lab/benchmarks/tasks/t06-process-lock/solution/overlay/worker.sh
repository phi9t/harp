#!/usr/bin/env bash
set -euo pipefail

id="${1:?usage: worker.sh ID STATE}"
state="${2:?usage: worker.sh ID STATE}"
mkdir -p "$state"

lock="$state/worker.lock"
if ! mkdir "$lock" 2>/dev/null; then
  echo "busy $id"
  exit 75
fi

cleanup() {
  rm -f "$lock/owner"
  rmdir "$lock" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

printf '%s\n' "$id" > "$lock/owner"
sleep 0.05
printf '%s\n' "$id" >> "$state/results.txt"
echo "acquired $id"
