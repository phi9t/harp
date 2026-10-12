#!/bin/sh
set -eu

run_collab_tests() {
  python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -v
}

run_undetermined() {
  echo "RUN: collab tests (change set undetermined)"
  run_collab_tests
}

base_ref=${CHECK_COLLAB_BASE_REF:-main}
if ! git rev-parse --verify --quiet "$base_ref" >/dev/null 2>&1; then
  run_undetermined
  exit $?
fi

if ! base=$(git merge-base HEAD "$base_ref" 2>/dev/null); then
  run_undetermined
  exit $?
fi

if ! changed=$(
  git diff --name-only "$base" HEAD &&
    git diff --name-only &&
    git diff --name-only --cached
); then
  run_undetermined
  exit $?
fi

touches_toolset=0
while IFS= read -r path; do
  case "$path" in
    tools/collab/*)
      touches_toolset=1
      break
      ;;
  esac
done <<EOF
$changed
EOF

if [ "$touches_toolset" = 0 ]; then
  echo "SKIP: collab tests (no tools/collab/ changes)"
  exit 0
fi

run_collab_tests
