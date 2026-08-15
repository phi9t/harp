#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
bundle_root="$repo_root/evidence/verified_coevolution_agenda"
artifact_root="$bundle_root/artifacts"
observed_at="${HARP_CAPTURED_AT:-2026-08-14T00:00:00Z}"

if [[ "${1:-}" == "--refresh" ]]; then
  refresh=1
elif [[ -n "${1:-}" ]]; then
  echo "usage: $0 [--refresh]" >&2
  exit 2
else
  refresh=0
fi

for required in curl jq pandoc pdftotext perl shasum wc; do
  command -v "$required" >/dev/null
done

test ! -L "$bundle_root"
mkdir -p "$artifact_root"

fetch_file() {
  local url="$1"
  local destination="$2"
  local partial="$destination.part"
  mkdir -p "$(dirname "$destination")"
  if [[ -s "$destination" && "$refresh" -eq 0 ]]; then
    return 0
  fi
  rm -f -- "$partial"
  curl \
    --location \
    --fail \
    --silent \
    --show-error \
    --max-time 180 \
    --retry 2 \
    --retry-delay 1 \
    --user-agent 'harp-verified-coevolution-agenda-capture/1.0' \
    "$url" \
    --output "$partial"
  test -s "$partial"
  mv -- "$partial" "$destination"
}

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

bytes_file() {
  wc -c < "$1" | tr -d ' '
}

plain_from_html() {
  local html="$1"
  local text="$2"
  pandoc --from html --to plain "$html" --output "$text"
}

capture_url() {
  local source_id="$1"
  local url="$2"
  local destination="$3"
  local status="$4"
  local rights="$5"
  local claim_ceiling="$6"
  local note="$7"
  fetch_file "$url" "$destination"
  printf '%s\t%s\t%s\t%s\t%s\t%s\tsha256:%s\t%s\t%s\t%s\n' \
    "$source_id" \
    "$url" \
    "${destination#"$repo_root"/}" \
    "$observed_at" \
    "$(bytes_file "$destination")" \
    "$status" \
    "$(sha256_file "$destination")" \
    "$rights" \
    "$claim_ceiling" \
    "$note"
}

capture_generated() {
  local source_id="$1"
  local source_path="$2"
  local destination="$3"
  local status="$4"
  local rights="$5"
  local claim_ceiling="$6"
  local note="$7"
  printf '%s\tgenerated-from:%s\t%s\t%s\t%s\t%s\tsha256:%s\t%s\t%s\t%s\n' \
    "$source_id" \
    "$source_path" \
    "${destination#"$repo_root"/}" \
    "$observed_at" \
    "$(bytes_file "$destination")" \
    "$status" \
    "$(sha256_file "$destination")" \
    "$rights" \
    "$claim_ceiling" \
    "$note"
}

capture_manifest="$bundle_root/manifest.tsv"
{
  printf 'source_id\turl\tlocal_path\tobserved_at\tbytes\tstatus\tsha256\trights\tclaim_ceiling\tnote\n'
} > "$capture_manifest.part"

capture_arxiv_full() {
  local source_id="$1"
  local arxiv_version="$2"
  local slug="$3"
  local title_probe="$4"
  local root="$artifact_root/$slug"
  mkdir -p "$root/metadata" "$root/pdf" "$root/html" "$root/text"

  capture_url "$source_id" \
    "https://export.arxiv.org/api/query?id_list=${arxiv_version}" \
    "$root/metadata/arxiv-${arxiv_version}.atom" \
    "fetched-arxiv" \
    "arXiv record; full text captured only for CC-BY work per arXiv license metadata" \
    "arXiv metadata identity, authors, abstract, categories, version, and license link" \
    "Atom metadata record" \
    >> "$capture_manifest.part"
  capture_url "$source_id" \
    "https://arxiv.org/abs/${arxiv_version}" \
    "$root/metadata/arxiv-${arxiv_version}.html" \
    "fetched-arxiv" \
    "arXiv abstract page" \
    "Abstract page identity, version metadata, and license/status fields" \
    "Abstract page" \
    >> "$capture_manifest.part"
  capture_url "$source_id" \
    "https://arxiv.org/html/${arxiv_version}" \
    "$root/html/arxiv-${arxiv_version}.html" \
    "fetched-arxiv-semantic-html" \
    "arXiv semantic HTML for CC-BY full-text work" \
    "Paper text, method, reported results, and limitations; no independent reproduction" \
    "Semantic HTML" \
    >> "$capture_manifest.part"
  capture_url "$source_id" \
    "https://arxiv.org/pdf/${arxiv_version}" \
    "$root/pdf/arxiv-${arxiv_version}.pdf" \
    "fetched-arxiv-pdf" \
    "arXiv PDF for CC-BY full-text work" \
    "Paper text, figures, tables, and appendices; no independent reproduction" \
    "PDF" \
    >> "$capture_manifest.part"

  if ! grep -qi "$title_probe" "$root/html/arxiv-${arxiv_version}.html"; then
    echo "semantic HTML for $source_id did not contain expected title probe $title_probe" >&2
    exit 1
  fi
  pdftotext -raw \
    "$root/pdf/arxiv-${arxiv_version}.pdf" \
    "$root/text/arxiv-${arxiv_version}.raw.txt"
  test -s "$root/text/arxiv-${arxiv_version}.raw.txt"
  capture_generated "$source_id" \
    "${root#"$repo_root"/}/pdf/arxiv-${arxiv_version}.pdf" \
    "$root/text/arxiv-${arxiv_version}.raw.txt" \
    "generated-pdftotext-raw" \
    "Derived text from captured CC-BY arXiv PDF" \
    "Deterministic reading text for local inspection; PDF remains authority" \
    "pdftotext -raw output" \
    >> "$capture_manifest.part"
}

capture_arxiv_metadata_only() {
  local source_id="$1"
  local arxiv_version="$2"
  local slug="$3"
  local title_probe="$4"
  local root="$artifact_root/$slug"
  mkdir -p "$root/metadata"
  capture_url "$source_id" \
    "https://export.arxiv.org/api/query?id_list=${arxiv_version}" \
    "$root/metadata/arxiv-${arxiv_version}.atom" \
    "fetched-arxiv" \
    "First-party arXiv metadata only; full text not vendored under unclear or nonexclusive redistribution terms" \
    "Metadata identity, authors, abstract, categories, version, and license/status fields only" \
    "Atom metadata record; no full text vendored" \
    >> "$capture_manifest.part"
  capture_url "$source_id" \
    "https://arxiv.org/abs/${arxiv_version}" \
    "$root/metadata/arxiv-${arxiv_version}.html" \
    "fetched-arxiv" \
    "First-party arXiv abstract page only; full text not vendored under unclear or nonexclusive redistribution terms" \
    "Abstract page identity and version metadata only" \
    "Abstract page; no full text vendored" \
    >> "$capture_manifest.part"
  if ! grep -qi "$title_probe" "$root/metadata/arxiv-${arxiv_version}.html"; then
    echo "abstract page for $source_id did not contain expected title probe $title_probe" >&2
    exit 1
  fi
}

capture_arxiv_full "LADDER" "2503.00735v3" "ladder" "LADDER"
capture_arxiv_full "PRIME-TTRL" "2504.16084v3" "prime_ttrl" "TTRL"
capture_arxiv_full "NSRSA" "2603.21558v1" "nsrsa" "Symbolic Recursive Self-Alignment"
capture_arxiv_full "SAHOO" "2603.06333v1" "sahoo" "SAHOO"
capture_arxiv_full "SCRIVENS-VERIFICATION" "2603.28650v1" "scrivens_verification" "Information-Theoretic"

model_collapse_root="$artifact_root/model_collapse"
mkdir -p "$model_collapse_root/html" "$model_collapse_root/text"
capture_url "MODEL-COLLAPSE" \
  "https://www.nature.com/articles/s41586-024-07566-y" \
  "$model_collapse_root/html/nature-s41586-024-07566-y.html" \
  "fetched-nature" \
  "Nature CC-BY first-party article page" \
  "Article text and methods on recursive generated-data collapse; no local reproduction" \
  "First-party Nature HTML" \
  >> "$capture_manifest.part"
if ! grep -qi "AI models collapse" "$model_collapse_root/html/nature-s41586-024-07566-y.html"; then
  echo "Nature model-collapse HTML did not contain expected title probe" >&2
  exit 1
fi
plain_from_html \
  "$model_collapse_root/html/nature-s41586-024-07566-y.html" \
  "$model_collapse_root/text/nature-s41586-024-07566-y.txt"
test -s "$model_collapse_root/text/nature-s41586-024-07566-y.txt"
capture_generated "MODEL-COLLAPSE" \
  "${model_collapse_root#"$repo_root"/}/html/nature-s41586-024-07566-y.html" \
  "$model_collapse_root/text/nature-s41586-024-07566-y.txt" \
  "generated-pandoc-plain" \
  "Derived text from captured Nature CC-BY HTML" \
  "Deterministic reading text for local inspection; Nature HTML remains authority" \
  "pandoc plain-text extraction" \
  >> "$capture_manifest.part"

capture_arxiv_metadata_only "GODEL-AGENT" "2410.04444v4" "godel_agent" "Gödel Agent"
capture_arxiv_metadata_only "MENDEL-GODEL-MACHINE" "2608.07645v1" "mendel_godel_machine" "Mendel"
capture_arxiv_metadata_only "GODEL-MACHINE" "cs/0309048v5" "godel_machine_arxiv" "Goedel Machines"

if [[ ! -s "$artifact_root/supplied_research_agenda.txt" ]]; then
  echo "missing supplied research agenda; preserve the user-provided agenda before running acquire.sh" >&2
  exit 1
fi
capture_generated "SUPPLIED-RESEARCH-AGENDA" \
  "prior transcript user message" \
  "$artifact_root/supplied_research_agenda.txt" \
  "supplied-input-preserved" \
  "User-supplied design input; not source evidence for paper claims" \
  "Design agenda only; must not verify paper mechanisms or results" \
  "Preserved verbatim from the prior planning transcript" \
  >> "$capture_manifest.part"

mv "$capture_manifest.part" "$capture_manifest"

artifact_inventory="$bundle_root/artifact_inventory.tsv"
{
  printf 'path\tkind\tbytes\tsha256\n'
  find "$artifact_root" -type f \
    | LC_ALL=C sort \
    | while IFS= read -r path; do
      relative="${path#"$repo_root"/}"
      case "$relative" in
        */pdf/*) kind="pdf" ;;
        */html/*) kind="html" ;;
        */text/*) kind="text" ;;
        */metadata/*) kind="metadata" ;;
        */artifacts/supplied_research_agenda.txt) kind="supplied-input" ;;
        *) kind="evidence" ;;
      esac
      printf '%s\t%s\t%s\tsha256:%s\n' \
        "$relative" \
        "$kind" \
        "$(bytes_file "$path")" \
        "$(sha256_file "$path")"
    done
} > "$artifact_inventory.part"
mv "$artifact_inventory.part" "$artifact_inventory"

source_record_count="$(($(wc -l < "$capture_manifest" | tr -d ' ') - 1))"
inventory_count="$(($(wc -l < "$artifact_inventory" | tr -d ' ') - 1))"
if [[ "$source_record_count" != "34" ]]; then
  echo "expected 34 capture records, found $source_record_count" >&2
  exit 1
fi
if [[ "$inventory_count" != "34" ]]; then
  echo "expected 34 source artifact inventory rows, found $inventory_count" >&2
  exit 1
fi

capture_receipt="$bundle_root/capture_receipt.tsv"
{
  printf 'key\tvalue\n'
  printf 'repository\tHarp\n'
  printf 'source_id\tVERIFIED-COEVOLUTION-AGENDA\n'
  printf 'capture_state\tcaptured-and-offline-verifiable\n'
  printf 'source_record_count\t%s\n' "$source_record_count"
  printf 'source_artifact_inventory_count\t%s\n' "$inventory_count"
  printf 'source_artifact_count\t34\n'
  printf 'cc_by_full_text_pdf_count\t5\n'
  printf 'rights_restricted_full_text_policy\tmetadata-only\n'
  printf 'observed_at\t%s\n' "$observed_at"
  printf 'manifest_sha256\tsha256:%s\n' "$(sha256_file "$capture_manifest")"
  printf 'artifact_inventory_sha256\tsha256:%s\n' "$(sha256_file "$artifact_inventory")"
  printf 'acquisition_script_sha256\tsha256:%s\n' "$(sha256_file "$bundle_root/acquire.sh")"
  printf 'supplied_research_agenda_sha256\tsha256:%s\n' "$(sha256_file "$artifact_root/supplied_research_agenda.txt")"
} > "$capture_receipt"

{
  printf 'path\tkind\tbytes\tsha256\n'
  find "$artifact_root" -type f \
    | LC_ALL=C sort \
    | while IFS= read -r path; do
      relative="${path#"$repo_root"/}"
      case "$relative" in
        */pdf/*) kind="pdf" ;;
        */html/*) kind="html" ;;
        */text/*) kind="text" ;;
        */metadata/*) kind="metadata" ;;
        */artifacts/supplied_research_agenda.txt) kind="supplied-input" ;;
        *) kind="evidence" ;;
      esac
      printf '%s\t%s\t%s\tsha256:%s\n' \
        "$relative" \
        "$kind" \
        "$(bytes_file "$path")" \
        "$(sha256_file "$path")"
    done
} > "$artifact_inventory.part"
mv "$artifact_inventory.part" "$artifact_inventory"
inventory_count="$(($(wc -l < "$artifact_inventory" | tr -d ' ') - 1))"
if [[ "$inventory_count" != "34" ]]; then
  echo "expected 34 source artifact inventory rows, found $inventory_count" >&2
  exit 1
fi
