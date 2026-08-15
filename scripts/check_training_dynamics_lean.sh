#!/bin/sh
set -eu

script_path=$0
case "$script_path" in
  */*) script_dir=${script_path%/*} ;;
  *) script_dir=. ;;
esac
canonical_project_root=$(CDPATH= cd "$script_dir/../formalization/training_dynamics" && pwd -P)
project_root=$canonical_project_root

# This test-only mode is command-line-only so normal invocations cannot be
# redirected through ambient environment state.
case "$#" in
  0) ;;
  2)
    if [ "$1" != '--project-for-test' ]; then
      printf '%s\n' 'Usage: check_training_dynamics_lean.sh [--project-for-test <directory>]' >&2
      exit 1
    fi
    project_root=$2
    ;;
  *)
    printf '%s\n' 'Usage: check_training_dynamics_lean.sh [--project-for-test <directory>]' >&2
    exit 1
    ;;
esac

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
