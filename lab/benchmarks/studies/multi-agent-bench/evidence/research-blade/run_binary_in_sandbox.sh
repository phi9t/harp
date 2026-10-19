#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: $0 <binary> [args...]" >&2
  exit 2
fi

S="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$S/home"

bwrap --ro-bind / / \
  --dev-bind /dev /dev \
  --bind "$S" "$S" \
  --tmpfs /home \
  --tmpfs /tmp \
  --unshare-net \
  --clearenv \
  --setenv HOME "$S/home" \
  --setenv PATH /usr/local/bin:/usr/bin:/bin \
  --setenv USER bench \
  "$@"
