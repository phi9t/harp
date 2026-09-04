#!/bin/sh
set -eu

case "$0" in
  /*) script_path=$0 ;;
  *) script_path=$PWD/$0 ;;
esac
script_dir=${script_path%/*}
helper=$script_dir/harp_xdg_env.sh
tmp_root=$(mktemp -d "${TMPDIR:-/tmp}/harp-xdg-env-test.XXXXXX")
trap 'rm -rf "$tmp_root"' EXIT HUP INT TERM

capture() {
  /usr/bin/env -i "$@" /bin/sh -eu <<EOF
. "$helper"
printf '%s\n' "\$HARP_DATA_ROOT" "\$HARP_CACHE_ROOT" "\$HARP_LEAN_CACHE_ROOT" "\$HARP_ELAN_HOME" "\$HARP_LEAN_TOOLCHAIN_BIN" "\$HARP_ATLAS_NODE_MODULES"
EOF
}

default_output=$(capture HOME=/home/alice)
expected_default=$(printf '%s\n' \
  /home/alice/.local/share/harp \
  /home/alice/.cache/harp \
  /home/alice/.cache/harp/lean/lean-4.32.1 \
  /home/alice/.cache/harp/lean/elan \
  /home/alice/.cache/harp/lean/elan/toolchains/leanprover--lean4---v4.32.1/bin \
  /home/alice/.cache/harp/node/atlas-node_modules)
[ "$default_output" = "$expected_default" ]

explicit_output=$(capture HOME=/home/alice XDG_DATA_HOME=/data XDG_CACHE_HOME=/cache)
expected_explicit=$(printf '%s\n' \
  /data/harp \
  /cache/harp \
  /cache/harp/lean/lean-4.32.1 \
  /cache/harp/lean/elan \
  /cache/harp/lean/elan/toolchains/leanprover--lean4---v4.32.1/bin \
  /cache/harp/node/atlas-node_modules)
[ "$explicit_output" = "$expected_explicit" ]

for assignment in XDG_DATA_HOME=relative XDG_CACHE_HOME=relative; do
  if /usr/bin/env -i HOME=/home/alice "$assignment" /bin/sh -eu -c ". '$helper'" >/dev/null 2>&1; then
    printf '%s\n' "expected $assignment to fail" >&2
    exit 1
  fi
done
if /usr/bin/env -i XDG_DATA_HOME=/data XDG_CACHE_HOME=/cache /bin/sh -eu -c ". '$helper'" >/dev/null 2>&1; then
  printf '%s\n' 'expected missing HOME to fail' >&2
  exit 1
fi

side_effect_home=$tmp_root/home
capture HOME="$side_effect_home" >/dev/null
[ ! -e "$side_effect_home" ]

printf '%s\n' 'harp XDG environment tests passed'
