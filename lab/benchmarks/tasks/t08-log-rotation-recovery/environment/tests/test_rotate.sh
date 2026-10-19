#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

tmp_root="$PWD/.test-tmp"
mkdir -p "$tmp_root"
root="$(mktemp -d -p "$tmp_root" rotate.XXXXXX)"

case1="$root/case1"
mkdir -p "$case1"
printf 'first line\nsecond line\n' > "$case1/app.log"
ROTATE_NOW=20260102030405 bash rotate_logs.sh "$case1" 3
test -f "$case1/app.log.20260102030405" || fail "missing first archive"
cmp "$case1/app.log.20260102030405" - <<'EOF' || fail "archive did not preserve current log"
first line
second line
EOF
test ! -s "$case1/app.log" || fail "current log is not empty after rotation"

case2="$root/case2"
mkdir -p "$case2"
printf 'alpha\n' > "$case2/app.log"
ROTATE_NOW=20260102030405 bash rotate_logs.sh "$case2" 5
printf 'beta\n' > "$case2/app.log"
ROTATE_NOW=20260102030405 bash rotate_logs.sh "$case2" 5
archive_count="$(find "$case2" -maxdepth 1 -type f -name 'app.log.20260102030405*' | wc -l | tr -d ' ')"
test "$archive_count" = "2" || fail "same timestamp overwrote an archive"
combined="$(find "$case2" -maxdepth 1 -type f -name 'app.log.20260102030405*' | sort | xargs cat)"
test "$combined" = "$(printf 'alpha\nbeta')" || fail "same timestamp archives have wrong contents"

case3="$root/case3"
mkdir -p "$case3"
printf 'old1\n' > "$case3/app.log.20260101000001"
printf 'old2\n' > "$case3/app.log.20260101000002"
printf 'old3\n' > "$case3/app.log.20260101000003"
printf 'new\n' > "$case3/app.log"
ROTATE_NOW=20260101000004 bash rotate_logs.sh "$case3" 2
find "$case3" -maxdepth 1 -type f -name 'app.log.*' -printf '%f\n' | sort > "$case3/actual.txt"
cat > "$case3/expected.txt" <<'EOF'
app.log.20260101000003
app.log.20260101000004
EOF
diff -u "$case3/expected.txt" "$case3/actual.txt"
