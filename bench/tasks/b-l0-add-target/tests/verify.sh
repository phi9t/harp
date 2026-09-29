#!/usr/bin/env bash
set -euo pipefail

blade-offline build //app:main
out="$(./build64_release/app/main)"
test "$out" = "BLADE_L0_ADD_TARGET ok"
