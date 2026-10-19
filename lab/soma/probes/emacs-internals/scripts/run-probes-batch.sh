#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="${1:-"$ROOT/probes/emacs-internals/traces/latest"}"

if [[ "${SOMA_INSULA_ACTIVE:-}" != "1" && "${SOMA_ALLOW_NON_INSULA_PROBES:-}" != "1" ]]; then
  printf 'error: run-probes-batch.sh is the inner runner and must run under Soma Insula\n' >&2
  printf 'hint: use probes/emacs-internals/scripts/run-probes.sh, or set SOMA_ALLOW_NON_INSULA_PROBES=1 for non-authoritative development\n' >&2
  exit 2
fi

mkdir -p "$OUT_DIR"

emacs --quick --batch \
  --load "$ROOT/probes/emacs-internals/elisp/run-probes.el" \
  "$OUT_DIR"

printf 'wrote probe traces to %s\n' "$OUT_DIR"
