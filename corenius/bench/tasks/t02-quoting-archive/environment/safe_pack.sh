#!/usr/bin/env bash
set -euo pipefail
out=$1
shift
: > manifest.txt
for f in $@; do
  printf '%s\n' "$f" >> manifest.txt
done
tar -cf $out $@
