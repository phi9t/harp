#!/bin/sh
set -eu

case "$0" in
  /*) script_path=$0 ;;
  *) script_path=$PWD/$0 ;;
esac
script_dir=${script_path%/*}
canonical_project_dir=$(CDPATH= cd -- "$script_dir/../formalization/autodiff_geometry" && pwd)

case "$#" in
  0)
    project_dir=$canonical_project_dir
    normal_build=true
    ;;
  2)
    if [ "$1" = "--project-for-test" ] && [ -d "$2" ]; then
      project_dir=$(CDPATH= cd -- "$2" && pwd)
      normal_build=false
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
  printf '%s\n' "Autodiff Geometry Lean toolchain is unavailable" >&2
  exit 1
fi

if [ "$normal_build" = true ]; then
  if [ -z "${ELAN_HOME:-}" ]; then
    printf '%s\n' "Autodiff Geometry Lean verification requires a task-scoped ELAN_HOME" >&2
    exit 1
  fi
  if ! elan_home_dir=$(CDPATH= cd -P -- "$ELAN_HOME" && pwd -P); then
    printf '%s\n' "Autodiff Geometry Lean verification requires a canonical task-scoped ELAN_HOME directory" >&2
    exit 1
  fi
  allowed_elan_root=/private/tmp/harp-mathematical-foundations-elan
  case "$elan_home_dir" in
    "$allowed_elan_root"|"$allowed_elan_root"/*)
      ;;
    *)
      printf '%s\n' "Autodiff Geometry Lean verification requires a task-scoped ELAN_HOME" >&2
      exit 1
      ;;
  esac
  if ! IFS= read -r pinned_toolchain < "$canonical_project_dir/lean-toolchain" || [ -z "$pinned_toolchain" ]; then
    printf '%s\n' "Autodiff Geometry Lean toolchain pin is unavailable" >&2
    exit 1
  fi
  export ELAN_TOOLCHAIN=$pinned_toolchain
fi

if ! scan_list=$(mktemp "${TMPDIR:-/tmp}/autodiff-geometry-lean.XXXXXX"); then
  printf '%s\n' "Autodiff Geometry Lean source scan failed: could not create a temporary file" >&2
  exit 1
fi
trap 'rm -f "$scan_list"' EXIT HUP INT TERM

if ! find "$project_dir" -type d -name .lake -prune -o -type f -name '*.lean' -print > "$scan_list"; then
  printf '%s\n' "Autodiff Geometry Lean source scan failed" >&2
  exit 1
fi

proof_holes_found=false
while IFS= read -r source_file || [ -n "$source_file" ]; do
  if grep -n -H 'sorry' "$source_file"; then
    proof_holes_found=true
  else
    grep_status=$?
    if [ "$grep_status" -ne 1 ]; then
      printf '%s\n' "Autodiff Geometry Lean source scan failed" >&2
      exit 1
    fi
  fi
done < "$scan_list"

if [ "$proof_holes_found" = true ]; then
  printf '%s\n' "Autodiff Geometry Lean source contains forbidden proof-placeholder text \`sorry\`." >&2
  exit 1
fi

cd "$project_dir"
lake build
