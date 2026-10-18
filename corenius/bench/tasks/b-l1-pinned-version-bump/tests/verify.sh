#!/usr/bin/env bash
set -euo pipefail

blade-offline build //app:main
out="$(./build64_release/app/main)"
test "$out" = "BLADE_L1_PIN bumped=v2 score=42"
