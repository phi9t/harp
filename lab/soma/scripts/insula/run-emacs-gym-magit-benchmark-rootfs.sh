#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/magit/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-gym-batch-benchmark-rootfs.sh" \
  "$OUT_DIR" \
  "magit-selective-staging" \
  "magit-selective-staging" \
  "/workspace/soma/benchmarks/emacs-gym/magit/elisp/run-benchmark.el" \
  "Magit"
