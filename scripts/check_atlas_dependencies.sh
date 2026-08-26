#!/bin/sh
set -eu

missing_dependencies() {
  echo "Atlas dependencies are missing; run mise run bootstrap" >&2
  exit 1
}

if [ "$#" -ne 1 ] || [ -z "$1" ] || [ ! -d "$1" ]; then
  missing_dependencies
fi

atlas_dir=$1
for tool in eslint tsc vitest vite; do
  if [ ! -x "$atlas_dir/node_modules/.bin/$tool" ]; then
    missing_dependencies
  fi
done
