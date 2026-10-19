#!/usr/bin/env bash
set -euo pipefail
rm -f out.tar manifest.txt
bash safe_pack.sh out.tar "input/a b.txt" "input/c.txt"
tar -tf out.tar | sort > listing.txt
printf 'input/a b.txt\ninput/c.txt\n' > expected.txt
diff -u expected.txt listing.txt
diff -u expected.txt manifest.txt
