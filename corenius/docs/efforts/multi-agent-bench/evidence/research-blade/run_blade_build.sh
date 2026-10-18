#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <project-name> <target>" >&2
  exit 2
fi

S="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project="$S/projects/$1"
target="$2"

mkdir -p "$S/home"
mkdir -p "$S/home/.blade/blade_status"
executor_size="$(stat --printf="%s" /opt/tiger/typhoon-blade/executor)"
touch "$S/home/.blade/blade_status/pip_checking.$executor_size"
touch "$S/home/.blade/blade_status/pip3_checking.$executor_size"

bwrap --ro-bind / / \
  --dev-bind /dev /dev \
  --bind "$S" "$S" \
  --tmpfs /home \
  --tmpfs /tmp \
  --unshare-net \
  --clearenv \
  --setenv HOME "$S/home" \
  --setenv PATH /opt/tiger/ss_bin:/usr/local/bin:/usr/bin:/bin \
  --setenv USER bench \
  --chdir "$project" \
  /opt/tiger/ss_bin/blade build "$target"
