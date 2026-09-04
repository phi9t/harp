#!/bin/sh
set -eu

: "${HOME:?HOME must be set}"

harp_data_home=${XDG_DATA_HOME:-"$HOME/.local/share"}
harp_cache_home=${XDG_CACHE_HOME:-"$HOME/.cache"}

case "$harp_data_home" in
  /*) ;;
  *) printf '%s\n' 'XDG_DATA_HOME must be absolute' >&2; return 2 ;;
esac
case "$harp_cache_home" in
  /*) ;;
  *) printf '%s\n' 'XDG_CACHE_HOME must be absolute' >&2; return 2 ;;
esac

export HARP_DATA_ROOT="$harp_data_home/harp"
export HARP_CACHE_ROOT="$harp_cache_home/harp"
export HARP_TARGET_DIR="$HARP_CACHE_ROOT/rust/harp-target"
export HARP_INPROCESS_TARGET_DIR="$HARP_CACHE_ROOT/rust/codex-inprocess-target"
export HARP_PYTHON_ENV="$HARP_CACHE_ROOT/python/verification"
export HARP_LEAN_CACHE_ROOT=${HARP_LEAN_CACHE_ROOT:-"$HARP_CACHE_ROOT/lean/lean-4.32.1"}
export HARP_ELAN_HOME=${HARP_ELAN_HOME:-"$HARP_CACHE_ROOT/lean/elan"}
export HARP_LEAN_TOOLCHAIN=${HARP_LEAN_TOOLCHAIN:-"leanprover/lean4:v4.32.1"}
export HARP_LEAN_TOOLCHAIN_DIR=${HARP_LEAN_TOOLCHAIN_DIR:-"leanprover--lean4---v4.32.1"}
export HARP_LEAN_TOOLCHAIN_ROOT=${HARP_LEAN_TOOLCHAIN_ROOT:-"$HARP_ELAN_HOME/toolchains/$HARP_LEAN_TOOLCHAIN_DIR"}
export HARP_LEAN_TOOLCHAIN_BIN=${HARP_LEAN_TOOLCHAIN_BIN:-"$HARP_LEAN_TOOLCHAIN_ROOT/bin"}
export HARP_ATLAS_NODE_MODULES="$HARP_CACHE_ROOT/node/atlas-node_modules"
export UV_CACHE_DIR="$HARP_CACHE_ROOT/uv"
export PNPM_STORE_DIR="$HARP_CACHE_ROOT/pnpm-store"

unset harp_data_home harp_cache_home
