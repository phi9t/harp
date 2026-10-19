#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/dired/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-gym-batch-benchmark-rootfs.sh" \
  "$OUT_DIR" \
  "dired-filesystem" \
  "dired-filesystem" \
  "/workspace/soma/benchmarks/emacs-gym/dired/elisp/run-benchmark.el" \
  "Dired"
