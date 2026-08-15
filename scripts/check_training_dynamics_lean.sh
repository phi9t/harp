#!/bin/sh
set -eu

script_path=$0
case "$script_path" in
  */*) script_dir=${script_path%/*} ;;
  *) script_dir=. ;;
esac
project_root=$(CDPATH= cd "$script_dir/../formalization/training_dynamics" && pwd -P)

if ! command -v lake >/dev/null 2>&1; then
  printf '%s\n' 'Training Dynamics Lean toolchain is unavailable' >&2
  exit 1
fi

cd "$project_root"
exec lake build
