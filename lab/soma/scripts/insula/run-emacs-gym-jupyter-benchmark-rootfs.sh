#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/jupyter/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-gym-batch-benchmark-rootfs.sh" \
  --private-jupyter-runtime \
  "$OUT_DIR" \
  "jupyter-existing-kernel" \
  "jupyter-existing-kernel" \
  "/workspace/soma/benchmarks/emacs-gym/jupyter/elisp/run-benchmark.el" \
  "Jupyter"
