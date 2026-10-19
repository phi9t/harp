#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
OUT_DIR="${1:-"$ROOT/benchmarks/emacs-gym/magit/traces/latest"}"

exec "$ROOT/scripts/insula/run-emacs-gym-magit-benchmark-rootfs.sh" "$OUT_DIR"
