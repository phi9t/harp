#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="${1:-"$ROOT/probes/emacs-internals/traces/latest"}"

if [[ "${SOMA_USE_DEVELOPMENT_INSULA:-0}" == "1" ]]; then
  exec "$ROOT/scripts/insula/run-emacs-internals-probes.sh" "$OUT_DIR"
fi

exec "$ROOT/scripts/insula/run-emacs-internals-probes-rootfs.sh" "$OUT_DIR"
