#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/bridge/traces/latest-daemon"}"

exec python3 "$ROOT/scripts/insula/soma_insula.py" run-command \
  --process-model emacs-daemon-plus-external-jsonl-client \
  --private-jupyter-runtime \
  --daemon-bridge-runtime \
  --require-emacsclient \
  --clean-pattern '*.json' \
  --clean-pattern '*.md' \
  --clean-pattern '*.txt' \
  --clean-pattern '*.log' \
  --clean-pattern '*.out' \
  --clean-pattern '*.err' \
  "$OUT_DIR" \
  bridge-daemon-transport \
  bridge-daemon-transport \
  "bridge daemon benchmark" \
  -- \
  /bin/bash \
  /workspace/soma/benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark-in-rootfs.sh \
  "{out_dir_sandbox}"
