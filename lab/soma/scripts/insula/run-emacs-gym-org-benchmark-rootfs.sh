#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/org/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-gym-batch-benchmark-rootfs.sh" \
  "$OUT_DIR" \
  "org-structure" \
  "org-structure" \
  "/workspace/soma/benchmarks/emacs-gym/org/elisp/run-benchmark.el" \
  "Org"
