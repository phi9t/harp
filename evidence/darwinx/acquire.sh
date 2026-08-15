#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
bundle_root="$repo_root/evidence/darwinx"
artifacts_root="$bundle_root/artifacts"
metadata_root="$bundle_root/metadata"
text_root="$bundle_root/text"
private_root="$repo_root/target/private/harp-darwinx-capture"
refresh=0

if [[ "${1:-}" == "--refresh" ]]; then
  refresh=1
elif [[ -n "${1:-}" ]]; then
  echo "usage: $0 [--refresh]" >&2
  exit 2
fi

test ! -L "$bundle_root"
mkdir -p "$artifacts_root" "$metadata_root" "$text_root" "$private_root"

for required in awk curl file git shasum wc; do
  command -v "$required" >/dev/null
done

resolve_tool() {
  local tool_name="$1"
  local homebrew_fallback="/opt/homebrew/bin/$tool_name"
  if command -v "$tool_name" >/dev/null 2>&1; then
    command -v "$tool_name"
  elif [[ -x "$homebrew_fallback" ]]; then
    printf '%s\n' "$homebrew_fallback"
  else
    echo "missing required tool: $tool_name" >&2
    return 1
  fi
}

pdftotext_bin="$(resolve_tool pdftotext)"

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

bytes_file() {
  wc -c < "$1" | tr -d ' '
}

fetch_file() {
  local source_url="$1"
  local destination="$2"
  local partial="$destination.part"

  if [[ -s "$destination" && "$refresh" -eq 0 ]]; then
    return 0
  fi

  rm -f -- "$partial"
  curl \
    -L \
    --fail \
    --silent \
    --show-error \
    --max-time 120 \
    --retry 2 \
    --retry-delay 1 \
    --user-agent 'harp-darwinx-capture/1.0' \
    "$source_url" \
    -o "$partial"
  test -s "$partial"
  mv -- "$partial" "$destination"
}

paper_id='2608.07545v1'
paper_pdf_url="https://arxiv.org/pdf/$paper_id"
paper_abs_url="https://arxiv.org/abs/$paper_id"
paper_html_url="https://arxiv.org/html/$paper_id"
paper_api_url='https://export.arxiv.org/api/query?id_list=2608.07545'
harnessx_id='2606.14249v1'
harnessx_pdf_url="https://arxiv.org/pdf/$harnessx_id"
harnessx_abs_url="https://arxiv.org/abs/$harnessx_id"
harnessx_html_url="https://arxiv.org/html/$harnessx_id"
harnessx_api_url='https://export.arxiv.org/api/query?id_list=2606.14249'

paper_pdf="$artifacts_root/darwinx-$paper_id.pdf"
paper_abs="$metadata_root/darwinx-$paper_id-arxiv.html"
paper_html="$metadata_root/darwinx-$paper_id-paper.html"
paper_api="$metadata_root/darwinx-$paper_id-api.xml"
paper_text="$text_root/darwinx-$paper_id.txt"
supplied_review="$artifacts_root/supplied_review.md"
harnessx_pdf="$artifacts_root/harnessx-$harnessx_id.pdf"
harnessx_abs="$metadata_root/harnessx-$harnessx_id-arxiv.html"
harnessx_html="$metadata_root/harnessx-$harnessx_id-paper.html"
harnessx_api="$metadata_root/harnessx-$harnessx_id-api.xml"
harnessx_text="$text_root/harnessx-$harnessx_id.txt"

fetch_file "$paper_pdf_url" "$paper_pdf"
fetch_file "$paper_abs_url" "$paper_abs"
fetch_file "$paper_html_url" "$paper_html"
fetch_file "$paper_api_url" "$paper_api"
fetch_file "$harnessx_pdf_url" "$harnessx_pdf"
fetch_file "$harnessx_abs_url" "$harnessx_abs"
fetch_file "$harnessx_html_url" "$harnessx_html"
fetch_file "$harnessx_api_url" "$harnessx_api"

file "$paper_pdf" | grep -q 'PDF document'
file "$harnessx_pdf" | grep -q 'PDF document'
"$pdftotext_bin" -raw "$paper_pdf" "$paper_text"
"$pdftotext_bin" -raw "$harnessx_pdf" "$harnessx_text"
test -s "$paper_text"
test -s "$harnessx_text"
test -s "$supplied_review"

manifest_tmp="$(mktemp "$private_root/manifest.XXXXXX")"
printf '%s\n' \
  'source_id	artifact_type	status	local_path	source_url	resolved_version	bytes	sha256	license_status	note' \
  > "$manifest_tmp"

write_manifest_row() {
  local source_id="$1"
  local artifact_type="$2"
  local local_file="$3"
  local source_url="$4"
  local resolved_version="$5"
  local license_status="$6"
  local note="$7"
  printf '%s\t%s\tcaptured\t%s\t%s\t%s\t%s\tsha256:%s\t%s\t%s\n' \
    "$source_id" \
    "$artifact_type" \
    "${local_file#"$repo_root/"}" \
    "$source_url" \
    "$resolved_version" \
    "$(bytes_file "$local_file")" \
    "$(sha256_file "$local_file")" \
    "$license_status" \
    "$note" \
    >> "$manifest_tmp"
}

write_manifest_row \
  DARWINX-SUPPLIED-REVIEW \
  user-supplied-provisional-analysis \
  "$supplied_review" \
  'local-session:user-message' \
  'captured 2026-08-14 America/Los_Angeles' \
  'user supplied; no upstream license asserted' \
  'Preserved verbatim as provisional input; not evidence for paper claims.'
write_manifest_row \
  DARWINX-PAPER \
  pdf \
  "$paper_pdf" \
  "$paper_pdf_url" \
  'arXiv:2608.07545v1' \
  'CC BY 4.0' \
  'Primary paper PDF; author-reported method and results, not independent reproduction.'
write_manifest_row \
  DARWINX-PAPER \
  abstract-html \
  "$paper_abs" \
  "$paper_abs_url" \
  'arXiv:2608.07545v1' \
  'CC BY 4.0' \
  'Revision-pinned arXiv abstract and license metadata.'
write_manifest_row \
  DARWINX-PAPER \
  semantic-html \
  "$paper_html" \
  "$paper_html_url" \
  'arXiv:2608.07545v1' \
  'CC BY 4.0' \
  'Revision-pinned semantic HTML used for sections, tables, equations, and links.'
write_manifest_row \
  DARWINX-PAPER \
  atom-metadata \
  "$paper_api" \
  "$paper_api_url" \
  'arXiv:2608.07545v1 observed 2026-08-14 America/Los_Angeles' \
  'arXiv metadata' \
  'Atom metadata response for title, authors, dates, abstract, and category.'
write_manifest_row \
  DARWINX-PAPER \
  extracted-text \
  "$paper_text" \
  "$paper_pdf_url" \
  'pdftotext -raw from arXiv:2608.07545v1' \
  'derived from arXiv paper' \
  'Deterministic text sidecar; PDF remains the source of record.'
write_manifest_row \
  HARNESSX-PAPER \
  pdf \
  "$harnessx_pdf" \
  "$harnessx_pdf_url" \
  'arXiv:2606.14249v1' \
  'CC BY 4.0' \
  'Primary comparison paper; author-reported method and results, not independent reproduction.'
write_manifest_row \
  HARNESSX-PAPER \
  abstract-html \
  "$harnessx_abs" \
  "$harnessx_abs_url" \
  'arXiv:2606.14249v1' \
  'CC BY 4.0' \
  'Revision-pinned HarnessX abstract and license metadata.'
write_manifest_row \
  HARNESSX-PAPER \
  semantic-html \
  "$harnessx_html" \
  "$harnessx_html_url" \
  'arXiv:2606.14249v1' \
  'CC BY 4.0' \
  'Revision-pinned HarnessX semantic HTML used for comparison.'
write_manifest_row \
  HARNESSX-PAPER \
  atom-metadata \
  "$harnessx_api" \
  "$harnessx_api_url" \
  'arXiv:2606.14249v1 observed 2026-08-14 America/Los_Angeles' \
  'arXiv metadata' \
  'Atom metadata response for HarnessX title, authors, dates, abstract, and category.'
write_manifest_row \
  HARNESSX-PAPER \
  extracted-text \
  "$harnessx_text" \
  "$harnessx_pdf_url" \
  'pdftotext -raw from arXiv:2606.14249v1' \
  'derived from arXiv paper' \
  'Deterministic comparison-paper text sidecar; PDF remains the source of record.'

mv -- "$manifest_tmp" "$bundle_root/manifest.tsv"

inventory_tmp="$(mktemp "$private_root/inventory.XXXXXX")"
printf '%s\n' 'path	bytes	sha256' > "$inventory_tmp"
find "$artifacts_root" "$metadata_root" "$text_root" -type f -print \
  | LC_ALL=C sort \
  | while IFS= read -r captured_file; do
      printf '%s\t%s\tsha256:%s\n' \
        "${captured_file#"$repo_root/"}" \
        "$(bytes_file "$captured_file")" \
        "$(sha256_file "$captured_file")"
    done \
  >> "$inventory_tmp"
mv -- "$inventory_tmp" "$bundle_root/artifact_inventory.tsv"

{
  printf 'captured_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'harp_commit\t%s\n' "$(git rev-parse HEAD)"
  printf 'harp_tree_status\tdirty-worktree-preserved\n'
  printf 'paper_revision\tarXiv:2608.07545v1\n'
  printf 'comparison_paper_revision\tarXiv:2606.14249v1\n'
  printf 'pdftotext\t%s\n' "$("$pdftotext_bin" -v 2>&1 | head -n 1)"
  printf 'acquisition_script_sha256\tsha256:%s\n' "$(sha256_file "$bundle_root/acquire.sh")"
} > "$bundle_root/capture_receipt.tsv"

printf 'wrote %s\n' "$bundle_root/manifest.tsv"
