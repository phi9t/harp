#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
bundle_root="$repo_root/evidence/weng"
private_root="$repo_root/target/private/harp-weng-capture"
source_table="$bundle_root/sources.tsv"
refresh=0

if [[ "${1:-}" == "--refresh" ]]; then
  refresh=1
elif [[ -n "${1:-}" ]]; then
  echo "usage: $0 [--refresh]" >&2
  exit 2
fi

test -f "$source_table"
test ! -L "$bundle_root"
mkdir -p \
  "$bundle_root/artifacts/pdf" \
  "$bundle_root/artifacts/html" \
  "$bundle_root/artifacts/git" \
  "$bundle_root/licenses" \
  "$bundle_root/metadata" \
  "$bundle_root/text" \
  "$bundle_root/references" \
  "$bundle_root/references/structured" \
  "$bundle_root/receipts" \
  "$private_root"

for required in awk curl file find git pandoc pdftotext rg sed shasum wc xmllint xsltproc; do
  command -v "$required" >/dev/null
done

sanitize_field() {
  tr '\t\r\n' '   ' | sed 's/[[:space:]][[:space:]]*/ /g; s/^ //; s/ $//'
}

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

bytes_file() {
  wc -c < "$1" | tr -d ' '
}

fetch_file() {
  local url="$1"
  local destination="$2"
  local partial="$destination.part"

  if [[ -s "$destination" && "$refresh" -eq 0 ]]; then
    return 0
  fi

  rm -f -- "$partial"
  if ! curl \
    -L \
    --fail \
    --silent \
    --show-error \
    --max-time 120 \
    --retry 2 \
    --retry-delay 1 \
    --user-agent 'harp-rsi-source-capture/1.0' \
    "$url" \
    -o "$partial"; then
    rm -f -- "$partial"
    return 1
  fi

  test -s "$partial"
  mv -- "$partial" "$destination"
}

fetch_license_references() {
  fetch_file \
    'https://creativecommons.org/licenses/by/4.0/legalcode.txt' \
    "$bundle_root/licenses/CC-BY-4.0.txt" || true
  fetch_file \
    'https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt' \
    "$bundle_root/licenses/CC-BY-SA-4.0.txt" || true
  fetch_file \
    'https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode.txt' \
    "$bundle_root/licenses/CC-BY-NC-SA-4.0.txt" || true
  fetch_file \
    'https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode.txt' \
    "$bundle_root/licenses/CC-BY-NC-ND-4.0.txt" || true
  fetch_file \
    'https://arxiv.org/licenses/nonexclusive-distrib/1.0/' \
    "$bundle_root/licenses/arxiv-nonexclusive-distribution-1.0.html" || true
}

fetch_license_references

extract_references() {
  local text_path="$1"
  local references_path="$2"
  local partial="$references_path.part"

  awk '
    {
      lines[NR] = $0
      normalized = tolower($0)
      gsub(/^[[:space:]#0-9.:-]+/, "", normalized)
      gsub(/[[:space:]#]+$/, "", normalized)
      if (normalized == "references" || normalized == "bibliography") {
        heading = NR
      }
    }
    END {
      if (heading > 0) {
        for (line_number = heading; line_number <= NR; line_number++) {
          print lines[line_number]
        }
      }
    }
  ' "$text_path" > "$partial"

  if [[ -s "$partial" ]]; then
    mv -- "$partial" "$references_path"
    return 0
  fi

  rm -f -- "$partial" "$references_path"
  return 1
}

extract_arxiv_bibliography() {
  local source_id="$1"
  local arxiv_html_path="$2"
  local structured_path="$3"
  local partial="$structured_path.part"

  if [[ ! -s "$arxiv_html_path" ]] || ! rg -q 'ltx_bibitem' "$arxiv_html_path"; then
    rm -f -- "$partial" "$structured_path"
    return 1
  fi

  if ! xsltproc \
    --html \
    --stringparam source_id "$source_id" \
    "$bundle_root/arxiv-bibliography.xsl" \
    "$arxiv_html_path" \
    > "$partial" \
    2>/dev/null; then
    rm -f -- "$partial" "$structured_path"
    return 1
  fi

  if [[ -s "$partial" ]]; then
    mv -- "$partial" "$structured_path"
    return 0
  fi

  rm -f -- "$partial" "$structured_path"
  return 1
}

extract_numbered_bibliography() {
  local source_id="$1"
  local references_path="$2"
  local structured_path="$3"
  local partial="$structured_path.part"

  if [[ ! -s "$references_path" ]]; then
    return 1
  fi

  awk -v source_id="$source_id" '
    function emit() {
      if (reference_key != "") {
        gsub(/[[:space:]][[:space:]]+/, " ", citation)
        sub(/^ /, "", citation)
        sub(/ $/, "", citation)
        print source_id "\t" reference_key "\t" citation "\t\tplain-numbered"
      }
    }
    match($0, /^\[[0-9]+\][[:space:]]*/) {
      emit()
      reference_key = substr($0, RSTART, RLENGTH)
      gsub(/[[:space:]]/, "", reference_key)
      citation = substr($0, RSTART + RLENGTH)
      next
    }
    reference_key != "" && $0 !~ /^[[:space:]]*$/ {
      citation = citation " " $0
    }
    END { emit() }
  ' "$references_path" > "$partial"

  if [[ -s "$partial" ]]; then
    mv -- "$partial" "$structured_path"
    return 0
  fi

  rm -f -- "$partial"
  return 1
}

detect_license() {
  local evidence_path="$1"
  if [[ ! -s "$evidence_path" ]]; then
    printf '%s' 'unknown'
    return
  fi

  local detected
  detected="$(
    rg -io \
      'https?://creativecommons\.org/licenses/[a-z-]+/[0-9.]+/?|https?://arxiv\.org/licenses/[a-z0-9./_-]+' \
      "$evidence_path" \
      2>/dev/null \
      | head -n 1 \
      || true
  )"
  if [[ -n "$detected" ]]; then
    printf '%s' "$detected"
  else
    printf '%s' 'unknown'
  fi
}

write_receipt() {
  local receipt_path="$1"
  shift
  printf '%s\n' "$*" > "$receipt_path"
}

process_source() {
  local source_id="$1"
  local depth="$2"
  local cohort="$3"
  local title="$4"
  local relationship="$5"
  local artifact_kind="$6"
  local original_locator="$7"
  local version_hint="$8"
  local acquisition_url="$9"
  local license_metadata_url="${10}"
  local claim_ceiling="${11}"

  local slug
  slug="$(printf '%s' "$source_id" | tr '[:upper:]_' '[:lower:]-')"
  local receipt_path="$bundle_root/receipts/$slug.tsv"
  local artifact_path=""
  local text_path="$bundle_root/text/$slug.txt"
  local references_path="$bundle_root/references/$slug.txt"
  local metadata_path="$bundle_root/metadata/$slug.html"
  local arxiv_html_path=""
  local structured_path="$bundle_root/references/structured/$slug.tsv"
  local resolved_version="$version_hint"
  local status="fetched"
  local note=""
  local license_status="unknown"

  case "$artifact_kind" in
    pdf_arxiv)
      local arxiv_id
      arxiv_id="${version_hint#arXiv:}"
      local arxiv_base
      arxiv_base="${arxiv_id%%v[0-9]*}"
      metadata_path="$bundle_root/metadata/$slug-arxiv.html"
      fetch_file "https://arxiv.org/abs/$arxiv_base" "$metadata_path" || true
      arxiv_html_path="$bundle_root/metadata/$slug-arxiv-paper.html"
      fetch_file "https://arxiv.org/html/$arxiv_id" "$arxiv_html_path" || true
      license_status="$(detect_license "$metadata_path")"
      artifact_path="$bundle_root/artifacts/pdf/$slug.pdf"
      ;;
    pdf_direct|pdf_openreview|pdf_pmlr)
      artifact_path="$bundle_root/artifacts/pdf/$slug.pdf"
      ;;
    html)
      artifact_path="$bundle_root/artifacts/html/$slug.html"
      metadata_path="$artifact_path"
      ;;
    git_metadata)
      local git_dir="$bundle_root/artifacts/git/$slug"
      mkdir -p "$git_dir"
      local commit_sha
      commit_sha="$(git ls-remote "$acquisition_url" HEAD 2>/dev/null | awk 'NR == 1 {print $1}')"
      if [[ ! "$commit_sha" =~ ^[0-9a-f]{40}$ ]]; then
        status="blocked_git_resolution"
        note="unable to resolve immutable HEAD"
        artifact_path=""
      else
        resolved_version="git:$commit_sha"
        local repo_path
        repo_path="${acquisition_url#https://github.com/}"
        artifact_path="$git_dir/README.md"
        fetch_file "https://raw.githubusercontent.com/$repo_path/$commit_sha/README.md" "$artifact_path" || status="blocked_http"
        fetch_file "https://raw.githubusercontent.com/$repo_path/$commit_sha/LICENSE" "$git_dir/LICENSE" || true
        if [[ -s "$git_dir/LICENSE" ]]; then
          license_status="captured:$git_dir/LICENSE"
        fi
      fi
      ;;
    *)
      status="invalid_artifact_kind"
      note="unsupported artifact kind: $artifact_kind"
      ;;
  esac

  if [[ -n "$artifact_path" && "$status" == "fetched" ]]; then
    if [[ "$artifact_kind" != "git_metadata" ]]; then
      if ! fetch_file "$acquisition_url" "$artifact_path"; then
        status="blocked_http"
        note="curl failed for acquisition URL"
      fi
    fi
  fi

  if [[ -n "$artifact_path" && -s "$artifact_path" && "$status" == "fetched" ]]; then
    if [[ "$artifact_kind" == pdf_* ]]; then
      if ! file "$artifact_path" | rg -q 'PDF document'; then
        status="blocked_non_pdf"
        note="response was not a PDF"
      elif ! pdftotext -raw "$artifact_path" "$text_path"; then
        status="fetched_parse_error"
        note="PDF fetched but pdftotext failed"
      fi
    elif [[ "$artifact_kind" == "html" ]]; then
      if rg -qi 'cf-chl-|just a moment|attention required|captcha' "$artifact_path"; then
        status="blocked_challenge"
        note="response appears to be a verification page"
      elif ! pandoc --from=html --to=plain --wrap=none "$artifact_path" -o "$text_path"; then
        status="fetched_parse_error"
        note="HTML fetched but pandoc conversion failed"
      fi
    elif [[ "$artifact_kind" == "git_metadata" ]]; then
      cp "$artifact_path" "$text_path"
    fi
  fi

  local reference_status="not_parsed"
  if [[ -s "$text_path" ]]; then
    if extract_references "$text_path" "$references_path"; then
      reference_status="references_extracted"
    else
      reference_status="references_heading_not_found"
    fi
  fi
  if [[ -n "$arxiv_html_path" ]]; then
    extract_arxiv_bibliography "$source_id" "$arxiv_html_path" "$structured_path" || true
  fi
  if [[ ! -s "$structured_path" ]]; then
    extract_numbered_bibliography "$source_id" "$references_path" "$structured_path" || true
  fi

  if [[ "$license_status" == "unknown" && -s "$metadata_path" ]]; then
    license_status="$(detect_license "$metadata_path")"
  fi

  local artifact_bytes="0"
  local artifact_sha="MISSING"
  local text_bytes="0"
  local text_sha="MISSING"
  local references_bytes="0"
  local references_sha="MISSING"
  local artifact_rel="MISSING"
  local text_rel="MISSING"
  local references_rel="MISSING"

  if [[ -n "$artifact_path" && -s "$artifact_path" ]]; then
    artifact_bytes="$(bytes_file "$artifact_path")"
    artifact_sha="sha256:$(sha256_file "$artifact_path")"
    artifact_rel="${artifact_path#"$repo_root/"}"
  fi
  if [[ -s "$text_path" ]]; then
    text_bytes="$(bytes_file "$text_path")"
    text_sha="sha256:$(sha256_file "$text_path")"
    text_rel="${text_path#"$repo_root/"}"
  fi
  if [[ -s "$references_path" ]]; then
    references_bytes="$(bytes_file "$references_path")"
    references_sha="sha256:$(sha256_file "$references_path")"
    references_rel="${references_path#"$repo_root/"}"
  fi

  local safe_title safe_note safe_ceiling
  safe_title="$(printf '%s' "$title" | sanitize_field)"
  safe_note="$(printf '%s' "$note" | sanitize_field)"
  safe_ceiling="$(printf '%s' "$claim_ceiling" | sanitize_field)"

  write_receipt "$receipt_path" "$(
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s' \
      "$source_id" \
      "$depth" \
      "$cohort" \
      "$safe_title" \
      "$artifact_kind" \
      "$status" \
      "$artifact_rel" \
      "$acquisition_url" \
      "$original_locator" \
      "$resolved_version" \
      "$artifact_bytes" \
      "$artifact_sha" \
      "$text_rel" \
      "$text_bytes" \
      "$text_sha" \
      "$references_rel" \
      "$references_bytes" \
      "$references_sha" \
      "$reference_status" \
      "$license_status" \
      "$license_metadata_url" \
      "$safe_note; claim ceiling: $safe_ceiling"
  )"
}

tail -n +2 "$source_table" | while IFS=$'\t' read -r \
  source_id \
  depth \
  cohort \
  title \
  relationship \
  artifact_kind \
  original_locator \
  version_hint \
  acquisition_url \
  license_metadata_url \
  claim_ceiling; do
  printf 'acquiring %s\n' "$source_id"
  process_source \
    "$source_id" \
    "$depth" \
    "$cohort" \
    "$title" \
    "$relationship" \
    "$artifact_kind" \
    "$original_locator" \
    "$version_hint" \
    "$acquisition_url" \
    "$license_metadata_url" \
    "$claim_ceiling"
done

manifest_tmp="$(mktemp "$private_root/manifest.XXXXXX")"
printf '%s\n' 'source_id	depth	cohort	title	artifact_kind	status	local_path	source_url	original_locator	resolved_version	bytes	sha256	text_path	text_bytes	text_sha256	references_path	references_bytes	references_sha256	reference_status	license_status	license_locator	note' > "$manifest_tmp"
find "$bundle_root/receipts" -type f -name '*.tsv' -print | LC_ALL=C sort | while IFS= read -r receipt; do
  cat "$receipt"
done >> "$manifest_tmp"
mv -- "$manifest_tmp" "$bundle_root/manifest.tsv"

license_assignments_tmp="$(mktemp "$private_root/license-assignments.XXXXXX")"
printf '%s\n' 'source_id	license_status	license_evidence_locator' > "$license_assignments_tmp"
awk -F '\t' -v OFS='\t' 'NR > 1 { print $1, $20, $21 }' \
  "$bundle_root/manifest.tsv" \
  >> "$license_assignments_tmp"
mv -- "$license_assignments_tmp" "$bundle_root/license_assignments.tsv"

structured_tmp="$(mktemp "$private_root/outgoing-citations.XXXXXX")"
printf '%s\n' 'source_id	source_reference_id	citation_text	outbound_locator	parser' > "$structured_tmp"
find "$bundle_root/references/structured" -type f -name '*.tsv' -print | LC_ALL=C sort | while IFS= read -r structured; do
  cat "$structured"
done >> "$structured_tmp"
mv -- "$structured_tmp" "$bundle_root/outgoing_citations.tsv"

inventory_tmp="$(mktemp "$private_root/artifact-inventory.XXXXXX")"
printf '%s\n' 'path	kind	bytes	sha256' > "$inventory_tmp"
find \
  "$bundle_root/artifacts" \
  "$bundle_root/licenses" \
  "$bundle_root/metadata" \
  "$bundle_root/text" \
  "$bundle_root/references" \
  -type f -print \
  | LC_ALL=C sort \
  | while IFS= read -r captured; do
      captured_rel="${captured#"$repo_root/"}"
      captured_kind="${captured_rel#evidence/weng/}"
      captured_kind="${captured_kind%%/*}"
      printf '%s\t%s\t%s\tsha256:%s\n' \
        "$captured_rel" \
        "$captured_kind" \
        "$(bytes_file "$captured")" \
        "$(sha256_file "$captured")"
    done >> "$inventory_tmp"
mv -- "$inventory_tmp" "$bundle_root/artifact_inventory.tsv"

{
  printf 'captured_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'harp_commit\t%s\n' "$(git rev-parse HEAD)"
  printf 'harp_tree_status\tdirty-worktree-preserved\n'
  printf 'curl\t%s\n' "$(curl --version | head -n 1 | sanitize_field)"
  printf 'pandoc\t%s\n' "$(pandoc --version | head -n 1 | sanitize_field)"
  printf 'pdftotext\t%s\n' "$(pdftotext -v 2>&1 | head -n 1 | sanitize_field)"
  printf 'acquisition_script_sha256\tsha256:%s\n' "$(sha256_file "$bundle_root/acquire.sh")"
  printf 'source_table_sha256\tsha256:%s\n' "$(sha256_file "$source_table")"
} > "$bundle_root/run-receipt.tsv"

printf 'wrote %s\n' "$bundle_root/manifest.tsv"
