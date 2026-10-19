#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${1:-"$ROOT/workers/emacs-doom-lifecycle/traces/latest"}"

exec python3 "$ROOT/scripts/insula/soma_insula.py" run-command \
  --process-model batch-doom-lifecycle-worker \
  --worker-id doom-lifecycle \
  --doom-environment \
  --writable-doom-local-state \
  --include-doom-profile-source \
  "$OUT_DIR" \
  doom-lifecycle \
  doom-lifecycle \
  "Doom lifecycle worker" \
  -- \
  /opt/soma/emacs/bin/emacs \
  --quick \
  --batch \
  -l \
  /workspace/soma/workers/emacs-doom-lifecycle/elisp/run-lifecycle.el \
  -- \
  "{out_dir_sandbox}"
