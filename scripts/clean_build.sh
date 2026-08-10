#!/usr/bin/env bash
set -euo pipefail

if (($# != 3 && $# != 6)); then
  printf 'usage: %s <config-root> <harp-target> <inprocess-target>\n' "${0##*/}" >&2
  exit 2
fi

stat_tool=/usr/bin/stat
if [[ ! -x "$stat_tool" ]]; then
  printf 'required directory identity tool is unavailable: %s\n' "$stat_tool" >&2
  exit 2
fi

config_root=$(cd -P -- "$1" && /bin/pwd -P)
build_root="$config_root/.build"
expected_harp_target="$build_root/harp-target"
expected_inprocess_target="$build_root/codex-inprocess-target"

if [[ "$2" != "$expected_harp_target" ]]; then
  printf 'refusing cleanup: HARP_TARGET_DIR is not the configured worktree target: %s\n' \
    "$2" >&2
  exit 1
fi
if [[ "$3" != "$expected_inprocess_target" ]]; then
  printf 'refusing cleanup: HARP_INPROCESS_TARGET_DIR is not the configured worktree target: %s\n' \
    "$3" >&2
  exit 1
fi

if [[ -L "$build_root" ]]; then
  printf 'refusing cleanup: build root is a symlink: %s\n' "$build_root" >&2
  exit 1
fi
mkdir -p -- "$build_root"
if [[ -L "$build_root" || ! -d "$build_root" ]]; then
  printf 'refusing cleanup: build root is not a real directory: %s\n' "$build_root" >&2
  exit 1
fi
build_identity=$("$stat_tool" -f '%d:%i' "$build_root")

if (($# == 6)); then
  if [[ "${HARP_CLEAN_BUILD_TESTING:-}" != 1 || "$4" != --test-sync ]]; then
    printf 'refusing cleanup test synchronization outside the test fixture\n' >&2
    exit 2
  fi
  : >"$5"
  for _ in {1..500}; do
    if [[ -e "$6" ]]; then
      break
    fi
    sleep 0.01
  done
  if [[ ! -e "$6" ]]; then
    printf 'cleanup test synchronization timed out\n' >&2
    exit 2
  fi
fi

cd -P -- "$build_root"
current_path=$(/bin/pwd -P)
if [[ "$current_path" != "$build_root" ]]; then
  printf 'refusing cleanup: entered build root resolves outside the worktree: %s\n' \
    "$current_path" >&2
  exit 1
fi
current_identity=$("$stat_tool" -f '%d:%i' .)
if [[ "$current_identity" != "$build_identity" ]]; then
  printf 'refusing cleanup: build root identity changed before directory entry\n' >&2
  exit 1
fi

verify_relative_target() {
  local label="$1"
  local target="$2"
  if [[ -L "$target" ]]; then
    printf 'refusing cleanup: %s is a symlink: %s\n' "$label" "$target" >&2
    exit 1
  fi
  if [[ -e "$target" ]]; then
    if [[ ! -d "$target" ]]; then
      printf 'refusing cleanup: %s is not a directory: %s\n' "$label" "$target" >&2
      exit 1
    fi
  fi
}

verify_relative_target HARP_TARGET_DIR harp-target
verify_relative_target HARP_INPROCESS_TARGET_DIR codex-inprocess-target
rm -rf -- harp-target codex-inprocess-target
