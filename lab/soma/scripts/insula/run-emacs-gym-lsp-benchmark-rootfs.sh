#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/lsp/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-gym-batch-benchmark-rootfs.sh" \
  --setenv LSP_USE_PLISTS=true \
  --plan-string fixture_origin=generated-inside-bwrap-private-project-runtime \
  "$OUT_DIR" \
  "lsp-source" \
  "lsp-source" \
  "/workspace/soma/benchmarks/emacs-gym/lsp/elisp/run-benchmark.el" \
  "LSP"
