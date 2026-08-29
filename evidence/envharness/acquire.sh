#!/bin/sh
set -eu

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

root=evidence/envharness
mkdir -p "$root/artifacts/pdf" "$root/metadata" "$root/text"
curl -L --fail --max-time 120 -sS 'https://arxiv.org/pdf/2608.19880v1' -o "$root/artifacts/pdf/envharness-2608.19880.pdf"
curl -L --fail --max-time 120 -sS 'https://export.arxiv.org/api/query?id_list=2608.19880' -o "$root/metadata/envharness-2608.19880-atom.xml"
curl -L --fail --max-time 120 -sS 'https://arxiv.org/abs/2608.19880v1' -o "$root/metadata/envharness-2608.19880-abstract.html"
pdftotext -layout "$root/artifacts/pdf/envharness-2608.19880.pdf" "$root/text/envharness-2608.19880.txt"
file "$root/artifacts/pdf/envharness-2608.19880.pdf"
test -s "$root/text/envharness-2608.19880.txt"
