#!/usr/bin/env bash
set -euo pipefail
file="$1"
start="$2"
end="$3"
sed -n "1,${end}p" "$file" | awk '{ if ($2=="ERROR") e++; if ($2=="WARN") w++ } END { printf "errors=%d warnings=%d\n", e, w }'
