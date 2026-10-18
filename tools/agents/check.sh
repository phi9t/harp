#!/bin/sh
# Repository gate. Run --help or --list for suite selection and timeout options.
cd "$(dirname "$0")/../.." || exit 2
TMPDIR="$(python3 tools/temp_root.py)" || exit 2
export TMPDIR
TMP="$TMPDIR"; TEMP="$TMPDIR"; export TMP TEMP
exec python3 tools/agents/check.py "$@"
