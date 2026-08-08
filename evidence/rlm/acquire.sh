#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
bundle_root="$repo_root/evidence/rlm"
private_root="$repo_root/target/private/harp-rlm-capture"
refresh=0

if [[ "${1:-}" == "--refresh" ]]; then
  refresh=1
elif [[ -n "${1:-}" ]]; then
  echo "usage: $0 [--refresh]" >&2
  exit 2
fi

test ! -L "$bundle_root"
mkdir -p \
  "$bundle_root/artifacts/html" \
  "$bundle_root/artifacts/pdf" \
  "$bundle_root/artifacts/git" \
  "$bundle_root/artifacts/images" \
  "$bundle_root/metadata" \
  "$bundle_root/text" \
  "$bundle_root/references" \
  "$private_root"

for required in awk curl file git rg sed shasum wc xmllint xsltproc; do
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

pandoc_bin="$(resolve_tool pandoc)"
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
    --user-agent 'harp-a1zhang-rlm-capture/1.0' \
    "$source_url" \
    -o "$partial"
  test -s "$partial"
  mv -- "$partial" "$destination"
}

blog_url='https://alexzhang13.github.io/blog/2026/harness/'
paper_pdf_url='https://arxiv.org/pdf/2512.24601v3'
paper_abs_url='https://arxiv.org/abs/2512.24601v3'
paper_html_url='https://arxiv.org/html/2512.24601v3'
repo_commit='72d6940142ddfb84ee6be573dc999a37e633e671'
repo_raw_root="https://raw.githubusercontent.com/alexzhang13/rlm/$repo_commit"

blog_html="$bundle_root/artifacts/html/language-model-harnesses-are-compositional-generalizers.html"
paper_pdf="$bundle_root/artifacts/pdf/recursive-language-models-2512.24601v3.pdf"
paper_abs="$bundle_root/metadata/recursive-language-models-2512.24601v3-arxiv.html"
paper_html="$bundle_root/metadata/recursive-language-models-2512.24601v3-paper.html"
repo_readme="$bundle_root/artifacts/git/README.md"
repo_license="$bundle_root/artifacts/git/LICENSE"

fetch_file "$blog_url" "$blog_html"
fetch_file "$paper_pdf_url" "$paper_pdf"
fetch_file "$paper_abs_url" "$paper_abs"
fetch_file "$paper_html_url" "$paper_html"
fetch_file "$repo_raw_root/README.md" "$repo_readme"
fetch_file "$repo_raw_root/LICENSE" "$repo_license"

image_names=(
  fig1a_rlm_trajectory_isomorphism.png
  fig1b_length_strategy_generalization_lift.png
  fig2_mgh_long_task_decomposition.png
  fig3_locally_in_distribution.png
  fig4_context_offloading_programmatic_subcalls.png
  fig5_length_generalization_curves.png
  fig6_strategy_generalization_curves.png
  fig7_trajectory_similarity.png
)
for image_name in "${image_names[@]}"; do
  fetch_file \
    "https://alexzhang13.github.io/assets/img/lm_compo/$image_name" \
    "$bundle_root/artifacts/images/$image_name"
done

fetch_file \
  'https://alexzhang13.github.io/assets/bibliography/locally_in_distribution.bib' \
  "$bundle_root/references/locally_in_distribution.bib"

file "$paper_pdf" | rg -q 'PDF document'
"$pandoc_bin" \
  --from=html \
  --to=plain \
  --wrap=none \
  "$blog_html" \
  -o "$bundle_root/text/language-model-harnesses-are-compositional-generalizers.txt"
"$pdftotext_bin" \
  -raw \
  "$paper_pdf" \
  "$bundle_root/text/recursive-language-models-2512.24601v3.txt"
cp "$repo_readme" "$bundle_root/text/rlm-repository-readme.txt"

xsltproc \
  --html \
  --stringparam source_id RLM-PAPER \
  "$bundle_root/arxiv-bibliography.xsl" \
  "$paper_html" \
  > "$bundle_root/references/rlm-paper-citations.tsv" \
  2> "$private_root/xslt-errors.log"

manifest_tmp="$(mktemp "$private_root/manifest.XXXXXX")"
printf '%s\n' 'source_id	artifact_type	status	local_path	source_url	resolved_version	bytes	sha256	license_status	note' > "$manifest_tmp"

write_manifest_row() {
  local source_id="$1"
  local artifact_type="$2"
  local local_file="$3"
  local source_url="$4"
  local resolved_version="$5"
  local license_status="$6"
  local note="$7"
  printf '%s\t%s\tfetched\t%s\t%s\t%s\t%s\tsha256:%s\t%s\t%s\n' \
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
  A1ZHANG-X-POST \
  local-clipping \
  "$repo_root/knowledge/capture/clippings/Post by @a1zhang on X.md" \
  'https://x.com/a1zhang/status/2079203524395573442' \
  'X status 2079203524395573442' \
  unknown \
  'Repository-owned clipping supplied the seed; the canonical blog is the substantive source.'
write_manifest_row \
  A1ZHANG-HARNESS-BLOG \
  html \
  "$blog_html" \
  "$blog_url" \
  '2026-07-20 capture' \
  'copyright; no reuse license located' \
  'Author blog with eight first-party figures and its BibTeX file captured alongside.'
write_manifest_row \
  RLM-PAPER \
  pdf \
  "$paper_pdf" \
  "$paper_pdf_url" \
  'arXiv:2512.24601v3' \
  'CC BY 4.0' \
  'ArXiv PDF, abstract metadata, semantic HTML, text, and 51 structured bibliography records captured.'
write_manifest_row \
  RLM-REPO \
  git-metadata \
  "$repo_readme" \
  'https://github.com/alexzhang13/rlm' \
  "git:$repo_commit" \
  MIT \
  'Pinned README and LICENSE only; no source tree was materialized or executed.'
mv -- "$manifest_tmp" "$bundle_root/manifest.tsv"

inventory_tmp="$(mktemp "$private_root/inventory.XXXXXX")"
printf '%s\n' 'path	bytes	sha256' > "$inventory_tmp"
find \
  "$bundle_root/artifacts" \
  "$bundle_root/metadata" \
  "$bundle_root/text" \
  "$bundle_root/references" \
  -type f -print \
  | LC_ALL=C sort \
  | while IFS= read -r captured_file; do
      printf '%s\t%s\tsha256:%s\n' \
        "${captured_file#"$repo_root/"}" \
        "$(bytes_file "$captured_file")" \
        "$(sha256_file "$captured_file")"
    done >> "$inventory_tmp"
mv -- "$inventory_tmp" "$bundle_root/artifact_inventory.tsv"

{
  printf 'captured_at_utc\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'harp_commit\t%s\n' "$(git rev-parse HEAD)"
  printf 'harp_tree_status\tdirty-worktree-preserved\n'
  printf 'repo_commit\t%s\n' "$repo_commit"
  printf 'pandoc\t%s\n' "$("$pandoc_bin" --version | head -n 1)"
  printf 'pdftotext\t%s\n' "$("$pdftotext_bin" -v 2>&1 | head -n 1)"
  printf 'acquisition_script_sha256\tsha256:%s\n' "$(sha256_file "$bundle_root/acquire.sh")"
  printf 'source_table_sha256\tsha256:%s\n' "$(sha256_file "$bundle_root/sources.tsv")"
} > "$bundle_root/run-receipt.tsv"

printf 'wrote %s\n' "$bundle_root/manifest.tsv"
