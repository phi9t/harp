#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

mode_of() {
  stat -c '%a' "$1"
}

assert_mode() {
  local path="$1"
  local expected="$2"
  local actual
  actual="$(mode_of "$path")"
  test "$actual" = "$expected" || fail "$path mode is $actual, expected $expected"
}

tmp_root="$PWD/.test-tmp"
mkdir -p "$tmp_root"
root="$(mktemp -d -p "$tmp_root" runtime.XXXXXX)"

bash startup.sh "$root" >/dev/null
test -d "$root/run" || fail "missing run dir"
test -d "$root/run/sessions" || fail "missing sessions dir"
assert_mode "$root/run" 755
assert_mode "$root/run/sessions" 1777
ls "$root/run" >/dev/null || fail "run dir is not readable by the current user"
printf 'created by verifier\n' > "$root/run/sessions/current-user.txt"

printf 'keep me\n' > "$root/run/sessions/keep.txt"
chmod 0700 "$root/run"
chmod 0777 "$root/run/sessions"
bash startup.sh "$root" >/dev/null
assert_mode "$root/run" 755
assert_mode "$root/run/sessions" 1777
grep -qx 'keep me' "$root/run/sessions/keep.txt" || fail "startup removed existing session data"
