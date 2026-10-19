#!/usr/bin/env bash
set -euo pipefail

id="${1:?usage: worker.sh ID STATE}"
state="${2:?usage: worker.sh ID STATE}"
mkdir -p "$state"

lock="$state/worker.lock"
if [ -e "$lock" ]; then
  echo "busy $id"
  exit 75
fi

: > "$state/checked.$id"
deadline=$((SECONDS + 5))
while [ "$(find "$state" -maxdepth 1 -name 'checked.*' | wc -l)" -lt 2 ]; do
  if [ "$SECONDS" -ge "$deadline" ]; then
    echo "timed out waiting for peer" >&2
    exit 1
  fi
  sleep 0.01
done

printf '%s\n' "$id" > "$lock"
sleep 0.05
printf '%s\n' "$id" >> "$state/results.txt"
echo "acquired $id"
rm -f "$lock"
