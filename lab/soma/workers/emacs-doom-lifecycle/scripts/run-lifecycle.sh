#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="${1:-"$ROOT/workers/emacs-doom-lifecycle/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-doom-lifecycle-rootfs.sh" "$OUT_DIR"
