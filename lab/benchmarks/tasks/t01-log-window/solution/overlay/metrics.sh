#!/usr/bin/env bash
set -euo pipefail
file="$1"
start="$2"
end="$3"
awk -v s="$start" -v e="$end" 'NR >= s && NR <= e { if ($2=="ERROR") err++; if ($2=="WARN") warn++ } END { printf "errors=%d warnings=%d\n", err, warn }' "$file"
