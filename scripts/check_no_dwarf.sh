#!/usr/bin/env bash
set -euo pipefail

if (($# != 1)); then
  printf 'usage: %s <binary>\n' "${0##*/}" >&2
  exit 2
fi

binary="$1"
if [[ -L "$binary" ]]; then
  printf 'binary must not be a symlink: %s\n' "$binary" >&2
  exit 2
fi
if [[ ! -f "$binary" ]]; then
  printf 'binary does not exist: %s\n' "$binary" >&2
  exit 2
fi

otool=/usr/bin/otool
grep_tool=/usr/bin/grep
for tool in "$otool" "$grep_tool"; do
  if [[ ! -x "$tool" ]]; then
    printf 'required Mach-O inspection tool is unavailable: %s\n' "$tool" >&2
    exit 2
  fi
done

binary_name=${binary##*/}
binary_dir=${binary%/*}
if [[ "$binary_dir" == "$binary" ]]; then
  binary_dir=.
elif [[ -z "$binary_dir" ]]; then
  binary_dir=/
fi
if [[ -e "$binary_dir/$binary_name.dSYM" ]]; then
  printf 'unexpected dSYM beside %s: %s\n' "$binary" "$binary_dir/$binary_name.dSYM" >&2
  exit 1
fi

shopt -s nullglob dotglob
sibling_dsyms=("$binary_dir"/*.dSYM)
if ((${#sibling_dsyms[@]} != 0)); then
  printf 'unexpected sibling dSYM output beside %s: %s\n' "$binary" "${sibling_dsyms[0]}" >&2
  exit 1
fi

if ! load_commands=$("$otool" -l "$binary"); then
  printf 'failed to inspect Mach-O load commands for %s\n' "$binary" >&2
  exit 2
fi

if "$grep_tool" -Eq '^[[:space:]]*segname[[:space:]]+__DWARF([[:space:]]|$)' \
  <<<"$load_commands"; then
  printf 'unexpected __DWARF segment in %s\n' "$binary" >&2
  exit 1
else
  grep_status=$?
  if ((grep_status != 1)); then
    printf 'failed to inspect Mach-O segments for %s\n' "$binary" >&2
    exit 2
  fi
fi
