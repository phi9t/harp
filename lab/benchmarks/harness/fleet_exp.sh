#!/usr/bin/env bash
# Live solo-versus-fleet experiment on multi-ticket tasks. Runs are interleaved (solo, fleet, solo, fleet, ...) so load
# and API latency affect both conditions alike. Usage: lab/benchmarks/harness/fleet_exp.sh OUT [REPEAT] [TASK...]
set -u
out=${1:?out dir}; repeat=${2:-5}; shift 2 2>/dev/null || shift $#
here="$(cd "$(dirname "$0")" && pwd)"
tasks=("$@")
if [ ${#tasks[@]} -eq 0 ]; then
  while read -r t; do tasks+=("$t"); done < <(python3 - <<'PY'
import sys
sys.path.insert(0, "lab/benchmarks/harness")
import tasklib, fleet
for t in tasklib.load_tasks():
    if "multi" in t.tags:
        print(t.id)
PY
)
fi
mkdir -p "$out"
for i in $(seq 1 "$repeat"); do
  for t in "${tasks[@]}"; do
    n=$(python3 -c "import sys; sys.path.insert(0,'lab/benchmarks/harness'); import tasklib, fleet; print(len(fleet.load_tickets(tasklib.load_tasks(only=['$t'])[0])))")
    python3 "$here/run.py" "$t" --condition solo --agent traecli --authorize-model-spend --out "$out" >> "$out/log.txt" 2>&1
    python3 "$here/run.py" "$t" --condition "fleet:$n" --agent traecli --authorize-model-spend --out "$out" >> "$out/log.txt" 2>&1
  done
done
python3 "$here/run.py" --report "$out" > "$out/report.txt"
