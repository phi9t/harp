#!/bin/sh
set -eu

script_path=$0
case "$script_path" in
  */*) script_dir=${script_path%/*} ;;
  *) script_dir=. ;;
esac
default_project_root=$(CDPATH= cd "$script_dir/../formalization/training_dynamics" && pwd -P)
project_root=${HARP_TRAINING_DYNAMICS_PROJECT_ROOT:-$default_project_root}

if ! project_root=$(CDPATH= cd "$project_root" && pwd -P); then
  printf '%s\n' 'Training Dynamics Lean project directory is unavailable' >&2
  exit 1
fi

cd "$project_root"

if ! command -v lake >/dev/null 2>&1; then
  printf '%s\n' 'Training Dynamics Lean toolchain is unavailable' >&2
  exit 1
fi

# This is deliberately a strict textual all-occurrences ban, not a Lean token
# parser. It also rejects the prohibited proof-placeholder spelling in comments
# and strings, so remove every occurrence from scoped Lean files before building.
proof_holes=$(find . -type d -name .lake -prune -o -type f -name '*.lean' -exec grep -nE '(^|[^[:alnum:]_])sorry([^[:alnum:]_]|$)' {} \;)
if [ -n "$proof_holes" ]; then
  printf '%s\n%s\n' \
    'Training Dynamics Lean sources contain prohibited proof-placeholder text; remove every occurrence from scoped Lean files before building:' \
    "$proof_holes" >&2
  exit 1
fi

exec lake build
