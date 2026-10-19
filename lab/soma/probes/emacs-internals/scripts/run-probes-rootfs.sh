#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="${1:-"$ROOT/probes/emacs-internals/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-internals-probes-rootfs.sh" "$OUT_DIR"
