#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/probes/emacs-internals/traces/latest"}"

exec python3 "$ROOT/scripts/insula/soma_insula.py" run-command \
  --process-model batch-emacs-internals-probes \
  --plan-string "emacs_patch_policy=Patched Emacs is a distinct Insula runtime and requires source revision, configure flags, patch provenance, executable hash, ABI manifest, and stock-vs-patched probe evidence." \
  --clean-pattern '[0-9][0-9]-*.json' \
  --clean-pattern summary.json \
  --clean-pattern summary.md \
  --clean-pattern insula-plan.json \
  "$OUT_DIR" \
  emacs-internals-probes \
  emacs-internals-probes \
  "Emacs internals probes" \
  -- \
  /workspace/soma/probes/emacs-internals/scripts/run-probes-batch.sh \
  "{out_dir_sandbox}"
