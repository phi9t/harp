#!/bin/sh
set -eu

case "$0" in
  /*) script_path=$0 ;;
  *) script_path=$PWD/$0 ;;
esac
script_dir=${script_path%/*}
canonical_project_dir=$(CDPATH= cd -- "$script_dir/../formalization/mathematical_foundations" && pwd)

case "$#" in
  0)
    project_dir=$canonical_project_dir
    ;;
  2)
    if [ "$1" = "--project-for-test" ] && [ -d "$2" ]; then
      project_dir=$(CDPATH= cd -- "$2" && pwd)
    else
      printf '%s\n' "usage: $0 [--project-for-test <directory>]" >&2
      exit 2
    fi
    ;;
  *)
    printf '%s\n' "usage: $0 [--project-for-test <directory>]" >&2
    exit 2
    ;;
esac

if ! command -v lake >/dev/null 2>&1; then
  printf '%s\n' "Mathematical Foundations Lean toolchain is unavailable" >&2
  exit 1
fi

matches=$(find "$project_dir" -type d -name .lake -prune -o -type f -name '*.lean' -exec grep -n -H 'sorry' {} + || true)
if [ -n "$matches" ]; then
  printf '%s\n' "Mathematical Foundations Lean source contains forbidden proof-placeholder text \`sorry\`." >&2
  printf '%s\n' "$matches" >&2
  exit 1
fi

cd "$project_dir"
lake build
