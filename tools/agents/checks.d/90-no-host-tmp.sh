#!/bin/sh
# Use the stronger shared source policy; no operational sandbox exceptions.
set -eu
exec python3 "$(dirname "$0")/../../check_temp_paths.py" --root "$PWD"
