#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "usage: $0 <loader-path> <binary> [args...]" >&2
  exit 2
fi

S="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
loader="$1"
shift

mkdir -p "$S/home"

bwrap --ro-bind / / \
  --dev-bind /dev /dev \
  --bind "$S" "$S" \
  --ro-bind "$loader" /ld-linux-x86-64.so.2 \
  --tmpfs /home \
  --tmpfs /tmp \
  --unshare-net \
  --clearenv \
  --setenv HOME "$S/home" \
  --setenv PATH /usr/local/bin:/usr/bin:/bin \
  --setenv USER bench \
  "$@"
