#!/usr/bin/env bash
set -euo pipefail

blade-offline build //lib:main
out="$(./build64_release/lib/main)"
test "$out" = "BLADE_L0_INCS_SRCS ok"
