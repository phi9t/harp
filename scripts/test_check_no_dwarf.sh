#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && /bin/pwd -P)
checker="$repo_root/scripts/check_no_dwarf.sh"
fixture=$(mktemp -d "${TMPDIR:-/tmp}/harp-no-dwarf.XXXXXX")
trap 'rm -rf -- "$fixture"' EXIT

expect_failure() {
  local label="$1"
  shift
  if "$@" >"$fixture/output" 2>&1; then
    printf '%s unexpectedly succeeded\n' "$label" >&2
    exit 1
  fi
}

cp /bin/ls "$fixture/harp"
ln -s /bin/ls "$fixture/harp-link"

expect_failure "symlinked binary check" "$checker" "$fixture/harp-link"

mkdir "$fixture/harp.dSYM"
expect_failure "target dSYM check" "$checker" "$fixture/harp"
rmdir "$fixture/harp.dSYM"

mkdir "$fixture/sibling.dSYM"
expect_failure "sibling dSYM check" "$checker" "$fixture/harp"

printf 'no-DWARF shell regressions: PASS\n'
