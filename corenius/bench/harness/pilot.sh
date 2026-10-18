#!/usr/bin/env bash
# Live pilot: every single-ticket task, N repeats, a real traecli agent inside the agent capsule.
# Usage: bench/harness/pilot.sh OUT_DIR [REPEAT] [PARALLEL]
set -u
out=${1:?out dir}; repeat=${2:-3}; par=${3:-5}
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$out"
python3 - <<'PY' > "$out/tasks.txt"
import sys
sys.path.insert(0, "bench/harness")
import tasklib
for t in tasklib.load_tasks():
    if "multi" not in t.tags:
        print(t.id)
PY
xargs -P "$par" -I{} sh -c "python3 $here/run.py {} --agent traecli --authorize-model-spend --repeat $repeat --out $out > $out/log-{}.txt 2>&1" < "$out/tasks.txt"
python3 "$here/run.py" --report "$out" > "$out/report.txt"
