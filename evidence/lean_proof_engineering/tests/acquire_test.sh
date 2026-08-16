#!/usr/bin/env bash
set -euo pipefail

source_root="$(git rev-parse --show-toplevel)"
source_bundle="$source_root/evidence/lean_proof_engineering"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/harp-lean-acquire-test.XXXXXX")"
trap 'rm -rf -- "$test_root"' EXIT HUP INT TERM

fixture_root="$test_root/repository"
fixture_bundle="$fixture_root/evidence/lean_proof_engineering"
mkdir -p "$fixture_root/evidence"
cp -R "$source_bundle" "$fixture_bundle"
git -C "$fixture_root" init -q
git -C "$fixture_root" -c user.name='Harp test' -c user.email='harp-test@example.invalid' \
  commit -q --allow-empty -m fixture

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

replace_receipt_value() {
  local key="$1"
  local value="$2"
  local receipt="$fixture_bundle/capture_receipt.tsv"
  awk -F '\t' -v OFS='\t' -v key="$key" -v value="$value" '
    $1 == key { $2 = value }
    { print }
  ' "$receipt" > "$receipt.part"
  mv -- "$receipt.part" "$receipt"
}

refresh_control_receipts() {
  replace_receipt_value source_lock_sha256 "sha256:$(sha256_file "$fixture_bundle/source_lock.tsv")"
  replace_receipt_value closure_sha256 "sha256:$(sha256_file "$fixture_bundle/closure.tsv")"
  replace_receipt_value manifest_sha256 "sha256:$(sha256_file "$fixture_bundle/manifest.tsv")"
}

refresh_observation_dates() {
  local current_date
  current_date="$(TZ=America/Los_Angeles date +%F)"
  awk -F '\t' -v OFS='\t' -v current_date="$current_date" '
    $4 == "dated-html" { sub(/^observed:[0-9-]+/, "observed:" current_date, $6) }
    { print }
  ' "$fixture_bundle/source_lock.tsv" > "$fixture_bundle/source_lock.tsv.part"
  mv -- "$fixture_bundle/source_lock.tsv.part" "$fixture_bundle/source_lock.tsv"
}

(cd "$fixture_root" && bash "$fixture_bundle/acquire.sh" --check >/dev/null)
(cd "$fixture_root" && HARP_LEAN_BUNDLE_ROOT=/does/not/exist \
  bash "$fixture_bundle/acquire.sh" --check >/dev/null)

cp "$fixture_bundle/source_lock.tsv" "$fixture_bundle/source_lock.valid.tsv"
awk -F '\t' -v OFS='\t' '
  $4 == "dated-html" { sub(/^observed:[0-9-]+/, "observed:2000-01-01", $6) }
  { print }
' "$fixture_bundle/source_lock.tsv" > "$fixture_bundle/source_lock.tsv.part"
mv -- "$fixture_bundle/source_lock.tsv.part" "$fixture_bundle/source_lock.tsv"
if (cd "$fixture_root" && bash "$fixture_bundle/acquire.sh" --refresh) >"$test_root/stale-date.out" 2>&1; then
  printf 'expected stale observation date to reject refresh\n' >&2
  exit 1
fi
grep -q 'lean-proof-engineering.refresh_observation_date:' "$test_root/stale-date.out"
mv -- "$fixture_bundle/source_lock.valid.tsv" "$fixture_bundle/source_lock.tsv"
refresh_observation_dates

refresh_lock="$fixture_root/evidence/.lean_proof_engineering.refresh.lock"
mkdir "$refresh_lock"
if (cd "$fixture_root" && bash "$fixture_bundle/acquire.sh" --refresh) >"$test_root/concurrent.out" 2>&1; then
  printf 'expected concurrent refresh to be rejected\n' >&2
  exit 1
fi
grep -q 'lean-proof-engineering.concurrent_refresh:' "$test_root/concurrent.out"
rmdir "$refresh_lock"

fake_bin="$test_root/bin"
curl_log="$test_root/curl.log"
mkdir "$fake_bin"
cat > "$fake_bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
disabled=0
https_source=0
location=0
output=''
proto=''
proto_redir=''
while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --disable) disabled=1 ;;
    --location) location=1 ;;
    --proto) shift; proto="$1" ;;
    --proto-redir) shift; proto_redir="$1" ;;
    -o) shift; output="$1" ;;
    https://*) https_source=1 ;;
  esac
  shift
done
[[ "$disabled" -eq 1 && "$location" -eq 1 && "$proto" == '=https' \
  && "$proto_redir" == '=https' && "$https_source" -eq 1 && -n "$output" ]]
basename="${output##*/}"
basename="${basename%.part}"
source_file="$(find "$HARP_TEST_CAPTURE_SOURCE" -type f -name "$basename" -print -quit)"
[[ -n "$source_file" ]]
cp "$source_file" "$output"
printf 'secure\n' >> "$HARP_TEST_CURL_LOG"
CURL
chmod +x "$fake_bin/curl"
(cd "$fixture_root" && \
  PATH="$fake_bin:$PATH" \
  HARP_TEST_CAPTURE_SOURCE="$source_bundle" \
  HARP_TEST_CURL_LOG="$curl_log" \
  bash "$fixture_bundle/acquire.sh" --refresh >/dev/null)
[[ "$(wc -l < "$curl_log" | tr -d ' ')" -eq 11 ]]

real_shasum="$(command -v shasum)"
prior_receipt_digest="$(sha256_file "$fixture_bundle/capture_receipt.tsv")"
cat > "$fake_bin/shasum" <<'SHASUM'
#!/usr/bin/env bash
set -euo pipefail
target="${!#}"
if [[ "$target" == */evidence/lean_proof_engineering/source_lock.tsv ]]; then
  printf '%064d  %s\n' 0 "$target"
  exit 0
fi
exec "$HARP_TEST_REAL_SHASUM" "$@"
SHASUM
chmod +x "$fake_bin/shasum"
if (cd "$fixture_root" && \
  PATH="$fake_bin:$PATH" \
  HARP_TEST_CAPTURE_SOURCE="$source_bundle" \
  HARP_TEST_CURL_LOG="$curl_log" \
  HARP_TEST_REAL_SHASUM="$real_shasum" \
  bash "$fixture_bundle/acquire.sh" --refresh) >"$test_root/rollback.out" 2>&1; then
  printf 'expected post-publication check failure\n' >&2
  exit 1
fi
grep -q 'lean-proof-engineering.publish_check_failed:' "$test_root/rollback.out"
[[ "$(sha256_file "$fixture_bundle/capture_receipt.tsv")" == "$prior_receipt_digest" ]]
(cd "$fixture_root" && bash "$fixture_bundle/acquire.sh" --check >/dev/null)
[[ ! -e "$refresh_lock" ]]
[[ -z "$(find "$fixture_root/evidence" -maxdepth 1 -name 'lean_proof_engineering.previous.*' -print -quit)" ]]

cp "$fixture_bundle/closure.tsv" "$fixture_bundle/closure.valid.tsv"
awk 'NR != 2 { print }' "$fixture_bundle/closure.tsv" > "$fixture_bundle/closure.tsv.part"
mv -- "$fixture_bundle/closure.tsv.part" "$fixture_bundle/closure.tsv"
refresh_control_receipts
if (cd "$fixture_root" && bash "$fixture_bundle/acquire.sh" --check) >"$test_root/root-count.out" 2>&1; then
  printf 'expected missing closure root to be rejected\n' >&2
  exit 1
fi
grep -q 'lean-proof-engineering.closure_root_count:' "$test_root/root-count.out"

mv -- "$fixture_bundle/closure.valid.tsv" "$fixture_bundle/closure.tsv"
awk -F '\t' -v OFS='\t' '
  $2 == 1 && $6 == "accepted" && !changed { $7 = "MISSING-LOCK"; changed = 1 }
  { print }
' "$fixture_bundle/closure.tsv" > "$fixture_bundle/closure.tsv.part"
mv -- "$fixture_bundle/closure.tsv.part" "$fixture_bundle/closure.tsv"
refresh_control_receipts
if (cd "$fixture_root" && bash "$fixture_bundle/acquire.sh" --check) >"$test_root/child-lock.out" 2>&1; then
  printf 'expected mismatched accepted closure child to be rejected\n' >&2
  exit 1
fi
grep -q 'lean-proof-engineering.closure_lock_correspondence:' "$test_root/child-lock.out"

printf 'lean evidence acquisition contract tests passed\n'
