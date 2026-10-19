#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/comint/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-gym-batch-benchmark-rootfs.sh" \
  "$OUT_DIR" \
  "comint-process" \
  "comint-process" \
  "/workspace/soma/benchmarks/emacs-gym/comint/elisp/run-benchmark.el" \
  "Comint"
