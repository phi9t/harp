#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/bridge/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-gym-batch-benchmark-rootfs.sh" \
  "$OUT_DIR" \
  "bridge-transport" \
  "bridge-transport" \
  "/workspace/soma/benchmarks/emacs-gym/bridge/elisp/run-benchmark.el" \
  "bridge transport"
