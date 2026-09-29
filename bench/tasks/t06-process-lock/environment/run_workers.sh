#!/usr/bin/env bash
set -euo pipefail

state="${1:?usage: run_workers.sh STATE}"
mkdir -p "$state"
rm -f "$state/results.txt" "$state/out.alpha" "$state/out.beta" "$state/checked.alpha" "$state/checked.beta" "$state/worker.lock"

bash worker.sh alpha "$state" > "$state/out.alpha" 2>&1 &
pid_alpha=$!
bash worker.sh beta "$state" > "$state/out.beta" 2>&1 &
pid_beta=$!

rc_alpha=0
wait "$pid_alpha" || rc_alpha=$?
rc_beta=0
wait "$pid_beta" || rc_beta=$?

cat "$state/out.alpha" "$state/out.beta" > "$state/worker.out"
cat "$state/worker.out"

acquired="$(grep -c '^acquired ' "$state/worker.out" || true)"
busy="$(grep -c '^busy ' "$state/worker.out" || true)"
printf 'acquired=%s busy=%s\n' "$acquired" "$busy"

for rc in "$rc_alpha" "$rc_beta"; do
  if [ "$rc" -ne 0 ] && [ "$rc" -ne 75 ]; then
    exit "$rc"
  fi
done
