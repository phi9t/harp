#!/usr/bin/env bash
set -euo pipefail

root="${1:-.}"
minimum_gib="${HARP_MIN_FREE_GIB:-8}"

for tool in df awk; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    printf 'required disk inspection tool is unavailable: %s\n' "$tool" >&2
    exit 2
  fi
done

if ! [[ "$minimum_gib" =~ ^[0-9]+$ ]]; then
  printf 'HARP_MIN_FREE_GIB must be a non-negative integer; got %q\n' "$minimum_gib" >&2
  exit 2
fi

if ! available_kib=$(df -Pk "$root" | awk 'NR == 2 {print $4}'); then
  printf 'could not inspect free space for %s\n' "$root" >&2
  exit 2
fi

if ! [[ "$available_kib" =~ ^[0-9]+$ ]]; then
  printf 'could not determine available space for %s\n' "$root" >&2
  exit 2
fi

if ! disk_status=$(awk -v available_kib="$available_kib" -v minimum_gib="$minimum_gib" \
  'BEGIN { print (available_kib < minimum_gib * 1024 * 1024 ? "low" : "ok") }'); then
  printf 'could not compare available and required space for %s\n' "$root" >&2
  exit 2
fi

case "$disk_status" in
  ok) ;;
  low)
    available_gib=$(awk -v available_kib="$available_kib" \
      'BEGIN { printf "%.2f", available_kib / 1024 / 1024 }')
    printf 'need at least %s GiB free for %s; only %s GiB available\n' \
      "$minimum_gib" "$root" "$available_gib" >&2
    exit 1
    ;;
  *)
    printf 'could not compare available and required space for %s\n' "$root" >&2
    exit 2
    ;;
esac
