#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
site_root="$root/site"
base_url="https://yoonholee.com/meta-harness/"

assets=(
  "index.html"
  "static/css/style.css"
  "static/data/evo.js"
  "static/data/pareto.js"
  "static/js/evo-chart.js"
  "static/js/pareto-chart.js"
  "static/js/nav.js"
  "static/js/init.js"
  "static/images/method.webp"
  "static/images/learning_curves.svg"
  "assets/fonts/SourceSerif4-Regular-subset.woff2"
  "assets/fonts/SourceSerif4-Semibold-subset.woff2"
  "assets/fonts/SourceSans3-Variable-subset.woff2"
)

for relative in "${assets[@]}"; do
  destination="$site_root/$relative"
  mkdir -p "$(dirname "$destination")"
  curl \
    --fail \
    --location \
    --silent \
    --show-error \
    --max-time 60 \
    "$base_url$relative" \
    --output "$destination"
done

printf 'Fetched %s Meta-Harness site assets.\n' "${#assets[@]}"
