#!/bin/sh
# Trusted regression fixtures only. This is not a hostile-source runner.
set -eu
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
repository_dir=$(CDPATH= cd -- "$script_dir/.." && pwd -P)
cd "$repository_dir"
unset HARP_LEAN_CACHE_ROOT HARP_ELAN_HOME HARP_LEAN_TOOLCHAIN HARP_LEAN_TOOLCHAIN_DIR
unset HARP_LEAN_TOOLCHAIN_ROOT HARP_LEAN_TOOLCHAIN_BIN LEAN_PATH LEAN_SRC_PATH
unset LEAN_SYSROOT ELAN_TOOLCHAIN
. scripts/harp_xdg_env.sh
export ELAN_HOME="$HARP_ELAN_HOME"
export PATH="$HARP_LEAN_TOOLCHAIN_BIN:/usr/bin:/bin"
cd formalization/lean
audit_output=$(mktemp -d /tmp/harp-cs-audit-tests.XXXXXX)
cleanup() {
  if [ -f "$audit_output/AuditFixture.olean" ]; then
    rm -- "$audit_output/AuditFixture.olean"
  fi
  rmdir -- "$audit_output"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' HUP TERM
"$HARP_LEAN_TOOLCHAIN_BIN/lake" env "$HARP_LEAN_TOOLCHAIN_BIN/lean" \
  --root=../tests/cauchy_schwarz \
  -o "$audit_output/AuditFixture.olean" ../tests/cauchy_schwarz/AuditFixture.lean
"$HARP_LEAN_TOOLCHAIN_BIN/lake" env /bin/sh -c \
  'export LEAN_PATH="$1:$LEAN_PATH"; exec "$2" --root=../tests/cauchy_schwarz ../tests/cauchy_schwarz/AuditControls.lean' \
  sh "$audit_output" "$HARP_LEAN_TOOLCHAIN_BIN/lean"
"$HARP_LEAN_TOOLCHAIN_BIN/lake" env "$HARP_LEAN_TOOLCHAIN_BIN/lean" \
  --root=../tests/cauchy_schwarz \
  ../tests/cauchy_schwarz/PolicyControls.lean
