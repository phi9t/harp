#!/bin/sh
set -eu

task_block() {
  task=$1
  awk -v header="[tasks.$task]" '
    $0 == header { active = 1; next }
    active && /^\[tasks\./ { exit }
    active { print }
  ' mise.toml
}

for task in lean-training lean-foundations lean-nng4 lean-autodiff lean-crouzeix lean-crouzeix-textbook crouzeix-textbook-publication lean-crouzeix-jin lean-crouzeix-ls lean-crouzeix-harp lean-all; do
  block=$(task_block "$task")
  printf '%s\n' "$block" | grep -F '. scripts/harp_xdg_env.sh' >/dev/null
  printf '%s\n' "$block" | grep -F 'export ELAN_HOME="$HARP_ELAN_HOME"' >/dev/null
  printf '%s\n' "$block" | grep -F 'export PATH="$HARP_LEAN_TOOLCHAIN_BIN:$PATH"' >/dev/null
  if printf '%s\n' "$block" | grep -E 'lean-cache|cache get|lake update' >/dev/null; then
    printf '%s\n' "$task must not hydrate dependencies" >&2
    exit 1
  fi
done

lean_env=$(task_block lean-env)
printf '%s\n' "$lean_env" | grep -F '. scripts/harp_xdg_env.sh' >/dev/null
printf '%s\n' "$lean_env" | grep -F 'if [ "$config_root" = "$primary" ]; then' >/dev/null
printf '%s\n' "$lean_env" | grep -F 'test ! -L formalization/lean/.lake' >/dev/null
printf '%s\n' "$lean_env" | grep -F 'test -L formalization/lean/.lake' >/dev/null
if printf '%s\n' "$lean_env" | grep -E '(^|[;&|[:space:]])lake[[:space:]]|mkdir|cache get|lake update' >/dev/null; then
  printf '%s\n' 'lean-env must remain a non-hydrating preflight' >&2
  exit 1
fi

printf '%s\n' 'Lean task graph tests passed'
