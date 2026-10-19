#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR_HOST="${1:-"$ROOT/benchmarks/emacs-gym/traces/latest"}"
RUN_ID="${SOMA_INSULA_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$$}"
NETWORK_MODE="${SOMA_INSULA_NETWORK:-disabled}"

log() { printf '\n== %s ==\n' "$*" >&2; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || die "python3 is required on the host coordinator"

mkdir -p "$OUT_DIR_HOST"
find "$OUT_DIR_HOST" -maxdepth 1 -type f \
  \( -name '*.json' -o -name '*.md' -o -name '*.txt' \) \
  -delete

run_benchmark() {
  local name="$1"
  local task_id="$2"
  local runner="$3"
  local out_dir="$OUT_DIR_HOST/$name"

  mkdir -p "$out_dir"
  log "running $name benchmark under selected Insula"
  case "$NETWORK_MODE" in
    disabled)
      SOMA_INSULA_RUN_ID="$RUN_ID" "$runner" "$out_dir"
      ;;
    host|shared|share-net)
      SOMA_INSULA_RUN_ID="$RUN_ID" \
      SOMA_INSULA_NETWORK="$NETWORK_MODE" \
      SOMA_INSULA_ALLOW_NETWORK_FOR="$task_id" \
        "$runner" "$out_dir"
      ;;
    *)
      die "unknown SOMA_INSULA_NETWORK value: $NETWORK_MODE"
      ;;
  esac
}

run_benchmark \
  magit \
  magit-selective-staging \
  "$ROOT/scripts/insula/run-emacs-gym-magit-benchmark-rootfs.sh"
run_benchmark \
  comint \
  comint-process \
  "$ROOT/scripts/insula/run-emacs-gym-comint-benchmark-rootfs.sh"
run_benchmark \
  jupyter \
  jupyter-existing-kernel \
  "$ROOT/scripts/insula/run-emacs-gym-jupyter-benchmark-rootfs.sh"
run_benchmark \
  lsp \
  lsp-source \
  "$ROOT/scripts/insula/run-emacs-gym-lsp-benchmark-rootfs.sh"
run_benchmark \
  org \
  org-structure \
  "$ROOT/scripts/insula/run-emacs-gym-org-benchmark-rootfs.sh"
run_benchmark \
  dired \
  dired-filesystem \
  "$ROOT/scripts/insula/run-emacs-gym-dired-benchmark-rootfs.sh"
run_benchmark \
  bridge \
  bridge-transport \
  "$ROOT/scripts/insula/run-emacs-gym-bridge-benchmark-rootfs.sh"
run_benchmark \
  bridge-daemon \
  bridge-daemon-transport \
  "$ROOT/scripts/insula/run-emacs-gym-bridge-daemon-benchmark-rootfs.sh"

python3 "$ROOT/scripts/insula/aggregate-emacs-gym-benchmark-results.py" \
  "$OUT_DIR_HOST" \
  "$RUN_ID" \
  --store-root "${SOMA_INSULA_ROOTFS_STORE:-$ROOT/.scratch/emacs-gym/insula/rootfs-store}"
