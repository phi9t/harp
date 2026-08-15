#!/bin/sh
set -eu

script_path=$0
case "$script_path" in
  */*) script_dir=${script_path%/*} ;;
  *) script_dir=. ;;
esac
project_root=$(CDPATH= cd "$script_dir/../formalization/training_dynamics" && pwd -P)

cd "$project_root"

if ! command -v lake >/dev/null 2>&1; then
  printf '%s\n' 'Training Dynamics Lean toolchain is unavailable' >&2
  exit 1
fi

proof_holes=$(find . -type d -name .lake -prune -o -type f -name '*.lean' -exec grep -nE '(^|[^[:alnum:]_])sorry([^[:alnum:]_]|$)' {} \;)
if [ -n "$proof_holes" ]; then
  printf '%s\n%s\n' \
    'Training Dynamics Lean sources contain a proof hole; replace every `sorry` before building:' \
    "$proof_holes" >&2
  exit 1
fi

exec lake build
