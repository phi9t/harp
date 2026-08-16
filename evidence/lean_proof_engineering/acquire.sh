#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
bundle_relative='evidence/lean_proof_engineering'
published_bundle_root="$repo_root/$bundle_relative"
bundle_root="$published_bundle_root"

bind_bundle_paths() {
  artifacts_root="$bundle_root/artifacts"
  metadata_root="$bundle_root/metadata"
  text_root="$bundle_root/text"
  source_lock="$bundle_root/source_lock.tsv"
  closure="$bundle_root/closure.tsv"
  manifest="$bundle_root/manifest.tsv"
  inventory="$bundle_root/artifact_inventory.tsv"
  receipt="$bundle_root/capture_receipt.tsv"
}

bind_bundle_paths
max_response_bytes=$((16 * 1024 * 1024))
mode=capture
refresh_stage_root=''
refresh_backup_root=''
refresh_lock_dir=''
refresh_lock_acquired=0

case "${1:-}" in
  "")
    ;;
  --refresh)
    mode=refresh
    ;;
  --check)
    mode=check
    ;;
  *)
    printf 'usage: %s [--refresh|--check]\n' "$0" >&2
    exit 2
    ;;
esac
if [[ "$#" -gt 1 ]]; then
  printf 'usage: %s [--refresh|--check]\n' "$0" >&2
  exit 2
fi

failure_class=unexpected_failure
failure_message='capture command exited unexpectedly'
failure_receipt_written=0

write_failure_receipt() {
  [[ "$mode" == capture && "$failure_receipt_written" -eq 0 ]] || return 0
  failure_receipt_written=1
  set +e
  mkdir -p "$bundle_root"
  {
    printf 'key\tvalue\n'
    printf 'capture_state\tfailed\n'
    printf 'failure_class\t%s\n' "$failure_class"
    printf 'failure_message\t%s\n' "$failure_message"
    printf 'failed_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'harp_commit\t%s\n' "$(git rev-parse HEAD 2>/dev/null || printf unknown)"
    if command -v shasum >/dev/null 2>&1 && [[ -f "$bundle_root/acquire.sh" ]]; then
      printf 'acquisition_script_sha256\tsha256:%s\n' \
        "$(shasum -a 256 "$bundle_root/acquire.sh" | awk '{print $1}')"
    fi
    if command -v shasum >/dev/null 2>&1 && [[ -f "$source_lock" ]]; then
      printf 'source_lock_sha256\tsha256:%s\n' \
        "$(shasum -a 256 "$source_lock" | awk '{print $1}')"
    fi
  } > "$receipt.failure.part"
  mv -- "$receipt.failure.part" "$receipt"
}

cleanup_refresh() {
  local status="$1"
  if [[ -n "$refresh_backup_root" && -d "$refresh_backup_root" ]]; then
    if [[ -d "$published_bundle_root" && -n "$refresh_stage_root" && ! -e "$refresh_stage_root" ]]; then
      mv -- "$published_bundle_root" "$refresh_stage_root" \
        || printf 'lean-proof-engineering.rollback_failed: could not retain failed publication\n' >&2
    fi
    if [[ ! -e "$published_bundle_root" ]]; then
      mv -- "$refresh_backup_root" "$published_bundle_root" \
        || printf 'lean-proof-engineering.rollback_failed: prior bundle remains at %s\n' "$refresh_backup_root" >&2
    fi
  fi
  if [[ -n "$refresh_stage_root" && -d "$refresh_stage_root" ]]; then
    rm -rf -- "$refresh_stage_root"
  fi
  if [[ "$refresh_lock_acquired" -eq 1 && -n "$refresh_lock_dir" ]]; then
    rmdir -- "$refresh_lock_dir" 2>/dev/null \
      || printf 'lean-proof-engineering.refresh_lock_cleanup_failed: %s\n' "$refresh_lock_dir" >&2
  fi
  return "$status"
}

on_exit() {
  local status=$?
  if [[ "$status" -ne 0 ]]; then
    write_failure_receipt
  fi
  cleanup_refresh "$status" || status=$?
  exit "$status"
}
trap on_exit EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

fail() {
  failure_class="$1"
  shift
  failure_message="$*"
  printf 'lean-proof-engineering.%s: %s\n' "$failure_class" "$failure_message" >&2
  exit 1
}

for required in awk curl file find git shasum wc; do
  command -v "$required" >/dev/null 2>&1 \
    || fail missing_tool "required tool is unavailable: $required"
done

resolve_pdftotext() {
  local candidate
  for candidate in "$(command -v pdftotext 2>/dev/null || true)" \
    /opt/homebrew/bin/pdftotext \
    /usr/local/bin/pdftotext; do
    if [[ -n "$candidate" && -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  fail missing_tool "required tool is unavailable: pdftotext"
}

pdftotext_bin="$(resolve_pdftotext)"

sha256_file() {
  shasum -a 256 "$1" | sed 's/[[:space:]].*$//'
}

bytes_file() {
  wc -c < "$1" | tr -d ' '
}

bundle_file() {
  local relative="$1"
  case "$relative" in
    "$bundle_relative"/*) printf '%s/%s\n' "$bundle_root" "${relative#"$bundle_relative"/}" ;;
    *) fail manifest_path "artifact is outside $bundle_relative: $relative" ;;
  esac
}

canonical_relative() {
  local path="$1"
  case "$path" in
    "$bundle_root"/*) printf '%s/%s\n' "$bundle_relative" "${path#"$bundle_root"/}" ;;
    *) fail manifest_path "artifact is outside staging bundle: $path" ;;
  esac
}

reject_symlink() {
  local path="$1"
  [[ ! -L "$path" ]] || fail symlink "symlinked path is forbidden: ${path#"$repo_root/"}"
}

check_real_path_chain() {
  local path="$1"
  local current="$path"
  while [[ "$current" != "$repo_root" && "$current" != "/" ]]; do
    reject_symlink "$current"
    current="${current%/*}"
    [[ -n "$current" ]] || current=/
  done
}

reject_bundle_symlinks() {
  local symlink
  symlink="$(find "$bundle_root" -type l -print -quit 2>/dev/null || true)"
  [[ -z "$symlink" ]] \
    || fail symlink "symlinked path is forbidden: ${symlink#"$repo_root/"}"
}

assert_locked_url() {
  local source_url="$1"
  local found=0
  local capture_url
  while IFS= read -r capture_url; do
    if [[ "$capture_url" == "$source_url" ]]; then
      found=1
      break
    fi
  done < <(sed '1d' "$source_lock" | cut -f 5)
  [[ "$found" -eq 1 ]] || fail unlocked_url "refusing URL absent from source_lock.tsv: $source_url"
}

fetch_file() {
  local source_url="$1"
  local destination="$2"
  local partial="$destination.part"

  assert_locked_url "$source_url"
  check_real_path_chain "$destination"
  reject_symlink "$partial"
  if [[ "$mode" == capture && -s "$destination" ]]; then
    rm -f -- "$partial"
    return 0
  fi

  rm -f -- "$partial"
  if ! curl \
    --disable \
    --proto '=https' \
    --proto-redir '=https' \
    --location \
    --fail \
    --silent \
    --show-error \
    --max-time 120 \
    --max-filesize "$max_response_bytes" \
    --retry 2 \
    --retry-delay 1 \
    --user-agent 'harp-lean-proof-engineering-capture/1.0' \
    "$source_url" \
    -o "$partial"; then
    rm -f -- "$partial"
    fail capture_failed "failed to capture $source_url"
  fi
  [[ -s "$partial" ]] || fail empty_response "captured response is empty: $source_url"
  local response_bytes
  response_bytes="$(bytes_file "$partial")"
  [[ "$response_bytes" -le "$max_response_bytes" ]] \
    || fail response_too_large "$source_url returned $response_bytes bytes"
  mv -- "$partial" "$destination"
}

verify_pdf() {
  local pdf="$1"
  reject_symlink "$pdf"
  [[ -s "$pdf" ]] || fail missing_artifact "missing PDF: ${pdf#"$repo_root/"}"
  [[ "$(head -c 5 "$pdf")" == '%PDF-' ]] \
    || fail invalid_pdf "missing PDF signature: ${pdf#"$repo_root/"}"
  file "$pdf" | grep -q 'PDF document' \
    || fail invalid_pdf "file(1) rejected PDF: ${pdf#"$repo_root/"}"
}

verify_table_headers() {
  [[ "$(head -n 1 "$source_lock")" == $'source_id\tparent_source_id\tdepth\tartifact_type\tcapture_url\timmutable_identity\tlicense_status\tredistribution_status' ]] \
    || fail source_lock_header "source_lock.tsv has an unexpected header"
  [[ "$(head -n 1 "$closure")" == $'parent_source_id\tdepth\tcandidate_url\tdiscovery_locator\tcategory\tdisposition\tcapture_source_id\tdecision' ]] \
    || fail closure_header "closure.tsv has an unexpected header"
  [[ "$(head -n 1 "$manifest")" == $'source_id\tartifact_type\tstatus\tlocal_path\tsource_url\tresolved_identity\tbytes\tsha256\tlicense_status\tredistribution_status\tnote' ]] \
    || fail manifest_header "manifest.tsv has an unexpected header"
  [[ "$(head -n 1 "$inventory")" == $'path\tbytes\tsha256' ]] \
    || fail inventory_header "artifact_inventory.tsv has an unexpected header"
}

verify_source_lock() {
  awk -F '\t' '
    NR == 1 { next }
    NF != 8 {
      printf "source lock row %d has %d fields, expected 8\n", NR, NF > "/dev/stderr"
      failed = 1
      next
    }
    $1 == "" || $4 == "" {
      printf "source lock row %d lacks a source or artifact type\n", NR > "/dev/stderr"
      failed = 1
    }
    $3 != 0 && $3 != 1 {
      printf "source lock row %d has invalid depth %s\n", NR, $3 > "/dev/stderr"
      failed = 1
    }
    $5 !~ /^https:\/\// {
      printf "source lock row %d is not HTTPS: %s\n", NR, $5 > "/dev/stderr"
      failed = 1
    }
    $3 == 0 {
      if ($2 != "-") {
        printf "source lock root %s must use parent -\n", $1 > "/dev/stderr"
        failed = 1
      }
      roots[$1] = 1
    }
    $3 == 1 && $2 == "-" {
      printf "source lock child %s lacks a root parent\n", $1 > "/dev/stderr"
      failed = 1
    }
    {
      if ($1 in source_depth && source_depth[$1] != $3) {
        printf "source lock source %s has inconsistent depths\n", $1 > "/dev/stderr"
        failed = 1
      }
      if ($1 in source_parent && source_parent[$1] != $2) {
        printf "source lock source %s has inconsistent parents\n", $1 > "/dev/stderr"
        failed = 1
      }
      source_depth[$1] = $3
      source_parent[$1] = $2
      tuple = $1 SUBSEP $4
      if (seen_tuple[tuple]++) {
        printf "source lock duplicates %s:%s\n", $1, $4 > "/dev/stderr"
        failed = 1
      }
    }
    END {
      for (root in roots) root_count++
      if (root_count != 4) {
        printf "source lock has %d unique roots, expected 4\n", root_count > "/dev/stderr"
        failed = 1
      }
      for (source in source_depth) {
        if (source_depth[source] == 1 && !(source_parent[source] in roots)) {
          printf "source lock child %s names unknown root %s\n", source, source_parent[source] > "/dev/stderr"
          failed = 1
        }
      }
      exit failed
    }
  ' "$source_lock" || fail source_lock_rows "source_lock.tsv violates the sealed depth-one contract"
}

verify_closure() {
  local accepted_roots accepted_children deferred not_needed maximum_depth
  awk -F '\t' '
    NR == 1 { next }
    NF != 8 {
      printf "closure row %d has %d fields, expected 8\n", NR, NF > "/dev/stderr"
      failed = 1
    }
    $2 !~ /^[0-9]+$/ || $2 > 1 {
      printf "closure row %d has invalid depth %s\n", NR, $2 > "/dev/stderr"
      failed = 1
    }
    $6 != "accepted" && $6 != "deferred" && $6 != "not-needed" {
      printf "closure row %d has invalid disposition %s\n", NR, $6 > "/dev/stderr"
      failed = 1
    }
    $2 == 0 && ($1 != "-" || $6 != "accepted" || $7 == "-") {
      printf "closure root row %d has an invalid parent, disposition, or capture source\n", NR > "/dev/stderr"
      failed = 1
    }
    $2 == 1 && $1 == "-" {
      printf "closure child row %d lacks a root parent\n", NR > "/dev/stderr"
      failed = 1
    }
    $6 == "accepted" && ($7 == "" || $7 == "-") {
      printf "closure row %d lacks an accepted capture source\n", NR > "/dev/stderr"
      failed = 1
    }
    $6 != "accepted" && $7 != "-" {
      printf "closure row %d assigns a capture source to %s\n", NR, $6 > "/dev/stderr"
      failed = 1
    }
    {
      candidate_key = $1 SUBSEP $3
      if (seen_candidate[candidate_key]++) {
        printf "closure row %d duplicates a parent/candidate route\n", NR > "/dev/stderr"
        failed = 1
      }
      if ($6 == "accepted" && seen_capture[$7]++) {
        printf "closure row %d duplicates accepted capture source %s\n", NR, $7 > "/dev/stderr"
        failed = 1
      }
    }
    END { exit failed }
  ' "$closure" || fail closure_rows "closure.tsv violates the depth-one closure contract"
  accepted_roots="$(awk -F '\t' 'NR > 1 && $2 == 0 && $6 == "accepted" { count++ } END { print count + 0 }' "$closure")"
  accepted_children="$(awk -F '\t' 'NR > 1 && $2 == 1 && $6 == "accepted" { count++ } END { print count + 0 }' "$closure")"
  deferred="$(awk -F '\t' 'NR > 1 && $6 == "deferred" { count++ } END { print count + 0 }' "$closure")"
  not_needed="$(awk -F '\t' 'NR > 1 && $6 == "not-needed" { count++ } END { print count + 0 }' "$closure")"
  maximum_depth="$(awk -F '\t' 'NR > 1 && $2 > maximum { maximum = $2 } END { print maximum + 0 }' "$closure")"
  [[ "$maximum_depth" -le 1 ]] \
    || fail closure_depth "closure depth exceeds one: $maximum_depth"
  [[ "$accepted_roots" -eq 4 ]] \
    || fail closure_root_count "expected 4 accepted roots, found $accepted_roots"
  [[ "$accepted_children" -eq 6 ]] \
    || fail closure_width "expected 6 accepted children, found $accepted_children"
  [[ "$deferred" -eq 10 ]] \
    || fail closure_deferred "expected 10 deferred children, found $deferred"
  [[ "$not_needed" -eq 24 ]] \
    || fail closure_not_needed "expected 24 not-needed children, found $not_needed"

  awk -F '\t' '
    FNR == NR {
      if (FNR == 1) next
      source_exists[$1] = 1
      source_depth[$1] = $3
      source_parent[$1] = $2
      source_rows[$1]++
      source_url[$1 SUBSEP $5]++
      next
    }
    FNR == 1 { next }
    $6 == "accepted" {
      capture = $7
      accepted_count[capture]++
      if (!(capture in source_exists)) {
        printf "accepted closure source %s is absent from source_lock.tsv\n", capture > "/dev/stderr"
        failed = 1
        next
      }
      if (source_depth[capture] != $2 || source_parent[capture] != $1) {
        printf "accepted closure source %s disagrees with lock parent/depth\n", capture > "/dev/stderr"
        failed = 1
      }
      if (!source_url[capture SUBSEP $3]) {
        printf "accepted closure source %s disagrees with lock URL\n", capture > "/dev/stderr"
        failed = 1
      }
      if ($2 == 1 && source_rows[capture] != 1) {
        printf "accepted closure child %s maps to %d lock rows, expected 1\n", capture, source_rows[capture] > "/dev/stderr"
        failed = 1
      }
    }
    END {
      for (source in source_exists) {
        if (accepted_count[source] != 1) {
          printf "locked source %s maps to %d accepted closure rows, expected 1\n", source, accepted_count[source] > "/dev/stderr"
          failed = 1
        }
      }
      exit failed
    }
  ' "$source_lock" "$closure" \
    || fail closure_lock_correspondence "accepted closure rows and locked sources are not one-to-one"
}

expected_local_path() {
  case "$1:$2" in
    LEANSTRAL-ANNOUNCEMENT:dated-html) printf '%s\n' "$bundle_relative/metadata/leanstral-1.5-announcement.html" ;;
    LEAN4AGENT-ARXIV:pdf) printf '%s\n' "$bundle_relative/artifacts/lean4agent-2606.06523v2.pdf" ;;
    LEAN4AGENT-ARXIV:abstract-html) printf '%s\n' "$bundle_relative/metadata/lean4agent-2606.06523v2-arxiv.html" ;;
    LEAN4-REPOSITORY:commit-metadata) printf '%s\n' "$bundle_relative/metadata/lean4-commit-57eb1ae3d0d440f29d1f35e9699c6df4d46c2620.json" ;;
    LEAN4-README:readme) printf '%s\n' "$bundle_relative/metadata/lean4-README.md" ;;
    LEAN4-LICENSE:license) printf '%s\n' "$bundle_relative/metadata/lean4-LICENSE" ;;
    LEAN-LEARN:dated-html) printf '%s\n' "$bundle_relative/metadata/learn-lean.html" ;;
    LEAN-FPIL:dated-html) printf '%s\n' "$bundle_relative/metadata/functional-programming-in-lean.html" ;;
    LEAN-TPIL:dated-html) printf '%s\n' "$bundle_relative/metadata/theorem-proving-in-lean4.html" ;;
    LEAN-MIL:dated-html) printf '%s\n' "$bundle_relative/metadata/mathematics-in-lean.html" ;;
    LEAN-REFERENCE:dated-html) printf '%s\n' "$bundle_relative/metadata/lean-reference.html" ;;
    *) fail source_lock_mapping "source_lock.tsv has no approved local path mapping for $1:$2" ;;
  esac
}

source_lock_identity() {
  local source_id="$1"
  local artifact_type="$2"
  awk -F '\t' -v source_id="$source_id" -v artifact_type="$artifact_type" '
    NR > 1 && $1 == source_id && $4 == artifact_type { print $6; found = 1; exit }
    END { if (!found) exit 1 }
  ' "$source_lock" \
    || fail source_lock_mapping "source_lock.tsv has no identity for $source_id:$artifact_type"
}

verify_manifest_bindings() {
  local source_id parent_source_id depth artifact_type capture_url immutable_identity license_status redistribution_status
  local expected_path matching_rows row_count manifest_status manifest_path manifest_url manifest_identity manifest_license manifest_redistribution
  awk -F '\t' '
    NR > 1 {
      if (NF != 8) { exit 1 }
      key = $1 SUBSEP $4
      if (seen[key]++) { exit 1 }
    }
  ' "$source_lock" || fail source_lock_duplicates "source_lock.tsv has duplicate or malformed source tuples"
  while IFS=$'\034' read -r source_id parent_source_id depth artifact_type capture_url immutable_identity license_status redistribution_status; do
    expected_path="$(expected_local_path "$source_id" "$artifact_type")"
    matching_rows="$(awk -F '\t' -v source_id="$source_id" -v artifact_type="$artifact_type" 'NR > 1 && $1 == source_id && $2 == artifact_type { print }' "$manifest")"
    row_count="$(printf '%s\n' "$matching_rows" | awk 'NF { count++ } END { print count + 0 }')"
    [[ "$row_count" -eq 1 ]] || fail manifest_lock_cardinality "expected exactly one manifest row for $source_id:$artifact_type, found $row_count"
    IFS=$'\t' read -r source_id artifact_type manifest_status manifest_path manifest_url manifest_identity _ _ manifest_license manifest_redistribution _ <<< "$matching_rows"
    [[ "$manifest_status" == captured ]] || fail manifest_lock_status "locked source $source_id:$artifact_type is not captured"
    [[ "$manifest_path" == "$expected_path" ]] || fail manifest_lock_path "locked source $source_id:$artifact_type has unexpected local path: $manifest_path"
    [[ "$manifest_url" == "$capture_url" ]] || fail manifest_lock_url "locked source $source_id:$artifact_type has an unexpected capture URL"
    [[ "$manifest_identity" == "$immutable_identity" ]] || fail manifest_lock_identity "locked source $source_id:$artifact_type has an unexpected identity"
    [[ "$manifest_license" == "$license_status" ]] || fail manifest_lock_license "locked source $source_id:$artifact_type has an unexpected license status"
    [[ "$manifest_redistribution" == "$redistribution_status" ]] || fail manifest_lock_redistribution "locked source $source_id:$artifact_type has an unexpected redistribution status"
  done < <(awk -F '\t' 'NR > 1 { print $1 "\034" $2 "\034" $3 "\034" $4 "\034" $5 "\034" $6 "\034" $7 "\034" $8 }' "$source_lock")
  [[ "$(awk -F '\t' 'NR > 1 && $3 == "captured" { count++ } END { print count + 0 }' "$manifest")" -eq 11 ]] \
    || fail manifest_lock_coverage "manifest has captured rows not represented by source_lock.tsv"
  [[ "$(awk -F '\t' 'NR > 1 && $1 == "LEAN4AGENT-ARXIV" && $2 == "pdftotext-raw" && $3 == "derived" && $4 == "evidence/lean_proof_engineering/text/lean4agent-2606.06523v2.txt" && $5 == "https://arxiv.org/pdf/2606.06523v2" && $6 == "derived-from-arXiv:2606.06523v2" && $9 == "CC BY 4.0" && $10 == "derived from permitted source" && $11 == "pdftotext -raw; PDF is source of record" { count++ } END { print count + 0 }' "$manifest")" -eq 1 ]] \
    || fail manifest_derived_binding "manifest has an invalid derived pdftotext sidecar row"
}

verify_manifest() {
  local count=0
  local source_id artifact_type status relative source_url resolved_identity
  local expected_bytes expected_digest license_status redistribution_status note extra
  local captured actual_bytes actual_digest
  while IFS=$'\t' read -r source_id artifact_type status relative source_url resolved_identity \
    expected_bytes expected_digest license_status redistribution_status note extra; do
    [[ -z "${extra:-}" ]] || fail manifest_columns "unexpected column for $relative"
    [[ "$relative" == evidence/lean_proof_engineering/artifacts/* \
      || "$relative" == evidence/lean_proof_engineering/metadata/* \
      || "$relative" == evidence/lean_proof_engineering/text/* ]] \
      || fail manifest_path "artifact escapes captured roots: $relative"
    captured="$(bundle_file "$relative")"
    check_real_path_chain "$captured"
    [[ -f "$captured" ]] || fail missing_artifact "missing manifest artifact: $relative"
    actual_bytes="$(bytes_file "$captured")"
    actual_digest="sha256:$(sha256_file "$captured")"
    [[ "$actual_bytes" == "$expected_bytes" ]] \
      || fail bytes_mismatch "$relative expected $expected_bytes bytes, found $actual_bytes"
    [[ "$actual_digest" == "$expected_digest" ]] \
      || fail digest_mismatch "$relative digest does not match manifest"
    if [[ "$status" == derived ]]; then
      [[ "$relative" == evidence/lean_proof_engineering/text/lean4agent-2606.06523v2.txt ]] \
        || fail manifest_status "unexpected derived artifact: $relative"
    else
      [[ "$status" == captured ]] || fail manifest_status "unexpected status for $relative: $status"
    fi
    count=$((count + 1))
  done < <(sed '1d' "$manifest")
  [[ "$count" -eq 12 ]] || fail manifest_count "expected 12 artifacts, found $count"
  verify_manifest_bindings
}

verify_inventory() {
  local count=0
  local relative expected_bytes expected_digest extra captured actual_bytes actual_digest
  while IFS=$'\t' read -r relative expected_bytes expected_digest extra; do
    [[ -z "${extra:-}" ]] || fail inventory_columns "unexpected column for $relative"
    [[ "$relative" == evidence/lean_proof_engineering/* ]] \
      || fail inventory_path "artifact escapes bundle: $relative"
    captured="$(bundle_file "$relative")"
    check_real_path_chain "$captured"
    [[ -f "$captured" ]] || fail missing_artifact "missing inventory artifact: $relative"
    actual_bytes="$(bytes_file "$captured")"
    actual_digest="sha256:$(sha256_file "$captured")"
    [[ "$actual_bytes" == "$expected_bytes" ]] \
      || fail bytes_mismatch "$relative expected $expected_bytes bytes, found $actual_bytes"
    [[ "$actual_digest" == "$expected_digest" ]] \
      || fail digest_mismatch "$relative digest does not match inventory"
    count=$((count + 1))
  done < <(sed '1d' "$inventory")
  [[ "$count" -eq 12 ]] || fail inventory_count "expected 12 artifacts, found $count"
  local captured_count
  captured_count="$(
    find "$artifacts_root" "$metadata_root" "$text_root" -type f -print \
      | wc -l \
      | tr -d ' '
  )"
  [[ "$captured_count" -eq 12 ]] \
    || fail uninventoried_artifact "expected 12 captured files, found $captured_count"
}

verify_retained_sidecar() {
  local sidecar='evidence/lean_proof_engineering/text/lean4agent-2606.06523v2.txt'
  local expected_digest actual_digest
  expected_digest="$(awk -F '\t' -v path="$sidecar" 'NR > 1 && $4 == path { print $8; found = 1; exit } END { if (!found) exit 1 }' "$manifest")" \
    || fail sidecar_manifest "manifest is missing the derived pdftotext sidecar"
  actual_digest="sha256:$(sha256_file "$(bundle_file "$sidecar")")"
  [[ "$actual_digest" == "$expected_digest" ]] \
    || fail sidecar_digest "retained pdftotext sidecar digest does not match manifest"
}

receipt_value() {
  local key="$1"
  sed -n "s/^${key}"$'\t'"//p" "$receipt"
}

verify_receipt() {
  local expected actual
  for pair in \
    "acquisition_script_sha256:$bundle_root/acquire.sh" \
    "source_lock_sha256:$source_lock" \
    "closure_sha256:$closure" \
    "manifest_sha256:$manifest" \
    "artifact_inventory_sha256:$inventory"; do
    expected="$(receipt_value "${pair%%:*}")"
    actual="sha256:$(sha256_file "${pair#*:}")"
    [[ "$expected" == "$actual" ]] \
      || fail stale_receipt "${pair%%:*} does not match captured receipt"
  done
  [[ "$(receipt_value closure_max_depth)" == 1 ]] \
    || fail stale_receipt "closure_max_depth must be 1"
  [[ "$(receipt_value capture_state)" == captured-and-offline-verifiable ]] \
    || fail stale_receipt "capture_state is invalid"
  [[ "$(receipt_value accepted_root_count)" == 4 ]] \
    || fail stale_receipt "accepted_root_count must be 4"
  [[ "$(receipt_value accepted_child_count)" == 6 ]] \
    || fail stale_receipt "accepted_child_count must be 6"
  [[ "$(receipt_value deferred_child_count)" == 10 ]] \
    || fail stale_receipt "deferred_child_count must be 10"
  [[ "$(receipt_value not_needed_child_count)" == 24 ]] \
    || fail stale_receipt "not_needed_child_count must be 24"
  [[ "$(receipt_value artifact_count)" == 12 ]] \
    || fail stale_receipt "artifact_count must be 12"
  [[ "$(receipt_value lean4_commit)" == 57eb1ae3d0d440f29d1f35e9699c6df4d46c2620 ]] \
    || fail stale_receipt "lean4_commit is invalid"
  [[ -n "$(receipt_value harp_commit)" ]] || fail stale_receipt "harp_commit is missing"
  [[ -n "$(receipt_value captured_at_utc)" ]] || fail stale_receipt "captured_at_utc is missing"
  local captured_pdftotext
  captured_pdftotext="$(receipt_value pdftotext_identity)"
  [[ "$captured_pdftotext" =~ ^pdftotext[[:space:]]+version[[:space:]]+[^[:space:]]+$ ]] \
    || fail stale_receipt "pdftotext_identity is missing or malformed"
}

verify_offline_bundle() {
  local partial_file
  for path in "$bundle_root" "$artifacts_root" "$metadata_root" "$text_root" \
    "$source_lock" "$closure" "$manifest" "$inventory" "$receipt"; do
    check_real_path_chain "$path"
  done
  reject_bundle_symlinks
  partial_file="$(find "$bundle_root" -type f -name '*.part' -print -quit)"
  [[ -z "$partial_file" ]] \
    || fail partial_file "bundle contains an incomplete .part file: ${partial_file#"$repo_root/"}"
  verify_table_headers
  verify_source_lock
  verify_closure
  verify_manifest
  verify_inventory
  verify_retained_sidecar
  verify_pdf "$artifacts_root/lean4agent-2606.06523v2.pdf"
  verify_receipt
  printf 'verified %s (12 artifacts, closure depth 1)\n' "${bundle_root#"$repo_root/"}"
}

prepare_refresh() {
  local evidence_root="$repo_root/evidence"
  local observed_mismatch current_date
  current_date="$(TZ=America/Los_Angeles date +%F)"
  observed_mismatch="$(awk -F '\t' -v current_date="$current_date" '
    NR > 1 && $4 == "dated-html" {
      if ($6 !~ /^observed:[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]([; ].*)?$/) {
        print $1 ":malformed:" $6
        next
      }
      observed_date = substr($6, 10, 10)
      if (observed_date != current_date) print $1 ":" observed_date
    }
  ' "$source_lock")"
  [[ -z "$observed_mismatch" ]] \
    || fail refresh_observation_date "lock observation date must equal $current_date: $observed_mismatch"

  refresh_lock_dir="$evidence_root/.lean_proof_engineering.refresh.lock"
  if ! mkdir -- "$refresh_lock_dir" 2>/dev/null; then
    fail concurrent_refresh "another Lean evidence refresh holds $refresh_lock_dir"
  fi
  refresh_lock_acquired=1
  refresh_stage_root="$(mktemp -d "$evidence_root/lean_proof_engineering.refresh.XXXXXX")"
  refresh_backup_root="$evidence_root/lean_proof_engineering.previous.$$"
  [[ ! -e "$refresh_backup_root" ]] \
    || fail publish_backup_exists "$refresh_backup_root already exists"
  for control in PROVENANCE.md acquire.sh source_lock.tsv closure.tsv; do
    cp "$published_bundle_root/$control" "$refresh_stage_root/$control"
  done
  bundle_root="$refresh_stage_root"
  bind_bundle_paths
  verify_source_lock
  verify_closure
}

publish_refresh() {
  mv -- "$published_bundle_root" "$refresh_backup_root" \
    || fail publish_failed "could not retain prior bundle"
  if ! mv -- "$refresh_stage_root" "$published_bundle_root"; then
    fail publish_failed "could not publish staged bundle"
  fi
  bundle_root="$published_bundle_root"
  bind_bundle_paths
  if ! (verify_offline_bundle); then
    fail publish_check_failed "post-publication verification failed; restoring prior bundle"
  fi
  rm -rf -- "$refresh_backup_root"
  refresh_backup_root=''
  refresh_stage_root=''
  rmdir -- "$refresh_lock_dir"
  refresh_lock_acquired=0
}

if [[ "$mode" == refresh ]]; then
  prepare_refresh
fi

if [[ "$mode" == check ]]; then
  verify_offline_bundle
  trap - EXIT
  exit 0
fi

if [[ "$mode" == capture && -f "$manifest" && -f "$inventory" && -f "$receipt" ]]; then
  verify_offline_bundle
  trap - EXIT
  exit 0
fi

for path in "$bundle_root" "$artifacts_root" "$metadata_root" "$text_root"; do
  check_real_path_chain "$path"
done
mkdir -p "$artifacts_root" "$metadata_root" "$text_root"
reject_bundle_symlinks
verify_source_lock
verify_closure

leanstral_url='https://mistral.ai/news/leanstral-1-5/'
paper_pdf_url='https://arxiv.org/pdf/2606.06523v2'
paper_abs_url='https://arxiv.org/abs/2606.06523v2'
lean_commit='57eb1ae3d0d440f29d1f35e9699c6df4d46c2620'
lean_commit_url="https://api.github.com/repos/leanprover/lean4/commits/$lean_commit"
lean_readme_url="https://raw.githubusercontent.com/leanprover/lean4/$lean_commit/README.md"
lean_license_url="https://raw.githubusercontent.com/leanprover/lean4/$lean_commit/LICENSE"
learn_url='https://lean-lang.org/learn/'
fpil_url='https://lean-lang.org/functional_programming_in_lean/'
tpil_url='https://lean-lang.org/theorem_proving_in_lean4/'
mil_url='https://leanprover-community.github.io/mathematics_in_lean/'
reference_url='https://lean-lang.org/doc/reference/latest/'

paper_pdf="$artifacts_root/lean4agent-2606.06523v2.pdf"
paper_text="$text_root/lean4agent-2606.06523v2.txt"
paper_text_partial="$paper_text.part"
leanstral_page="$metadata_root/leanstral-1.5-announcement.html"
paper_abs="$metadata_root/lean4agent-2606.06523v2-arxiv.html"
lean_commit_metadata="$metadata_root/lean4-commit-$lean_commit.json"
lean_readme="$metadata_root/lean4-README.md"
lean_license="$metadata_root/lean4-LICENSE"
learn_page="$metadata_root/learn-lean.html"
fpil_page="$metadata_root/functional-programming-in-lean.html"
tpil_page="$metadata_root/theorem-proving-in-lean4.html"
mil_page="$metadata_root/mathematics-in-lean.html"
reference_page="$metadata_root/lean-reference.html"

fetch_file "$leanstral_url" "$leanstral_page"
fetch_file "$paper_pdf_url" "$paper_pdf"
fetch_file "$paper_abs_url" "$paper_abs"
fetch_file "$lean_commit_url" "$lean_commit_metadata"
fetch_file "$lean_readme_url" "$lean_readme"
fetch_file "$lean_license_url" "$lean_license"
fetch_file "$learn_url" "$learn_page"
fetch_file "$fpil_url" "$fpil_page"
fetch_file "$tpil_url" "$tpil_page"
fetch_file "$mil_url" "$mil_page"
fetch_file "$reference_url" "$reference_page"

verify_pdf "$paper_pdf"
reject_symlink "$paper_text"
reject_symlink "$paper_text_partial"
if [[ "$mode" == refresh || ! -s "$paper_text" ]]; then
  rm -f -- "$paper_text_partial"
  "$pdftotext_bin" -raw "$paper_pdf" "$paper_text_partial" \
    || fail extraction_failed "pdftotext failed for ${paper_pdf#"$repo_root/"}"
  [[ -s "$paper_text_partial" ]] || fail extraction_failed "pdftotext produced an empty sidecar"
  mv -- "$paper_text_partial" "$paper_text"
fi

manifest_tmp="$manifest.part"
inventory_tmp="$inventory.part"
receipt_tmp="$receipt.part"
for partial in "$manifest_tmp" "$inventory_tmp" "$receipt_tmp"; do
  reject_symlink "$partial"
  rm -f -- "$partial"
done

printf '%s\n' $'source_id\tartifact_type\tstatus\tlocal_path\tsource_url\tresolved_identity\tbytes\tsha256\tlicense_status\tredistribution_status\tnote' > "$manifest_tmp"

write_manifest_row() {
  local source_id="$1"
  local artifact_type="$2"
  local local_file="$3"
  local source_url="$4"
  local resolved_identity="$5"
  local license_status="$6"
  local redistribution_status="$7"
  local note="$8"
  local status=captured
  if [[ "$artifact_type" == pdftotext-raw ]]; then
    status=derived
  fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\tsha256:%s\t%s\t%s\t%s\n' \
    "$source_id" \
    "$artifact_type" \
    "$status" \
    "$(canonical_relative "$local_file")" \
    "$source_url" \
    "$resolved_identity" \
    "$(bytes_file "$local_file")" \
    "$(sha256_file "$local_file")" \
    "$license_status" \
    "$redistribution_status" \
    "$note" \
    >> "$manifest_tmp"
}

write_manifest_row LEANSTRAL-ANNOUNCEMENT dated-html "$leanstral_page" "$leanstral_url" \
  "$(source_lock_identity LEANSTRAL-ANNOUNCEMENT dated-html)" 'no page license identified; Mistral terms apply' 'restricted; preserved as dated first-party evidence' \
  'Dated first-party announcement observation; not an immutable product specification.'
write_manifest_row LEAN4AGENT-ARXIV pdf "$paper_pdf" "$paper_pdf_url" \
  arXiv:2606.06523v2 'CC BY 4.0' 'permitted with attribution' \
  'Revision-pinned paper PDF and source of record.'
write_manifest_row LEAN4AGENT-ARXIV abstract-html "$paper_abs" "$paper_abs_url" \
  arXiv:2606.06523v2 'CC BY 4.0' 'permitted with attribution' \
  'Revision-pinned arXiv abstract and license metadata.'
write_manifest_row LEAN4-REPOSITORY commit-metadata "$lean_commit_metadata" "$lean_commit_url" \
  "git:$lean_commit" 'GitHub API metadata; embedded Lean source is Apache-2.0' 'preserved for source identity' \
  'Commit metadata records the resolved immutable Lean repository identity.'
write_manifest_row LEAN4AGENT-ARXIV pdftotext-raw "$paper_text" "$paper_pdf_url" \
  derived-from-arXiv:2606.06523v2 'CC BY 4.0' 'derived from permitted source' \
  'pdftotext -raw; PDF is source of record'
write_manifest_row LEAN4-README readme "$lean_readme" "$lean_readme_url" \
  "git:$lean_commit:README.md" Apache-2.0 'permitted with license preservation' \
  'Upstream README bytes captured at the resolved immutable Lean commit.'
write_manifest_row LEAN4-LICENSE license "$lean_license" "$lean_license_url" \
  "git:$lean_commit:LICENSE" Apache-2.0 'permitted with license preservation' \
  'Upstream license bytes captured at the resolved immutable Lean commit.'
write_manifest_row LEAN-LEARN dated-html "$learn_page" "$learn_url" \
  "$(source_lock_identity LEAN-LEARN dated-html)" 'no explicit page license identified' 'restricted; preserved as dated first-party evidence' \
  'Dated Learn Lean landing-page observation; outgoing closure is bounded to depth one.'
write_manifest_row LEAN-FPIL dated-html "$fpil_page" "$fpil_url" \
  "$(source_lock_identity LEAN-FPIL dated-html)" 'copyright Microsoft Corporation and Lean FRO; no explicit page license identified' 'restricted; preserved as dated documentation evidence' \
  'Allowed direct Learn Lean child; no deeper links were fetched.'
write_manifest_row LEAN-TPIL dated-html "$tpil_page" "$tpil_url" \
  "$(source_lock_identity LEAN-TPIL dated-html)" 'no explicit page license identified' 'restricted; preserved as dated documentation evidence' \
  'Allowed direct Learn Lean child; no deeper links were fetched.'
write_manifest_row LEAN-MIL dated-html "$mil_page" "$mil_url" \
  "$(source_lock_identity LEAN-MIL dated-html)" 'CC BY 4.0' 'permitted with attribution' \
  'Allowed direct Learn Lean child; no deeper links were fetched.'
write_manifest_row LEAN-REFERENCE dated-html "$reference_page" "$reference_url" \
  "$(source_lock_identity LEAN-REFERENCE dated-html)" 'no explicit page license identified' 'restricted; preserved as dated documentation evidence' \
  'Allowed direct Lean reference landing page; no deeper links were fetched.'
mv -- "$manifest_tmp" "$manifest"

printf '%s\n' $'path\tbytes\tsha256' > "$inventory_tmp"
for captured_file in \
  "$paper_pdf" \
  "$lean_license" \
  "$lean_readme" \
  "$fpil_page" \
  "$learn_page" \
  "$reference_page" \
  "$lean_commit_metadata" \
  "$paper_abs" \
  "$leanstral_page" \
  "$mil_page" \
  "$tpil_page" \
  "$paper_text"; do
  printf '%s\t%s\tsha256:%s\n' \
    "$(canonical_relative "$captured_file")" \
    "$(bytes_file "$captured_file")" \
    "$(sha256_file "$captured_file")" \
    >> "$inventory_tmp"
done
mv -- "$inventory_tmp" "$inventory"

{
  printf 'key\tvalue\n'
  printf 'capture_state\tcaptured-and-offline-verifiable\n'
  printf 'captured_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'harp_commit\t%s\n' "$(git rev-parse HEAD)"
  printf 'lean4_commit\t%s\n' "$lean_commit"
  printf 'closure_max_depth\t1\n'
  printf 'accepted_root_count\t4\n'
  printf 'accepted_child_count\t6\n'
  printf 'deferred_child_count\t10\n'
  printf 'not_needed_child_count\t24\n'
  printf 'artifact_count\t12\n'
  printf 'pdftotext_identity\t%s\n' "$("$pdftotext_bin" -v 2>&1 | head -n 1)"
  printf 'acquisition_script_sha256\tsha256:%s\n' "$(sha256_file "$bundle_root/acquire.sh")"
  printf 'source_lock_sha256\tsha256:%s\n' "$(sha256_file "$source_lock")"
  printf 'closure_sha256\tsha256:%s\n' "$(sha256_file "$closure")"
  printf 'manifest_sha256\tsha256:%s\n' "$(sha256_file "$manifest")"
  printf 'artifact_inventory_sha256\tsha256:%s\n' "$(sha256_file "$inventory")"
} > "$receipt_tmp"
mv -- "$receipt_tmp" "$receipt"

verify_offline_bundle
if [[ "$mode" == refresh ]]; then
  publish_refresh
fi
trap - EXIT
printf 'captured %s (12 artifacts, closure depth 1)\n' "${bundle_root#"$repo_root/"}"
