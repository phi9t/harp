#!/bin/sh
set -eu

mkdir -p /tmp/matrix-home-compact /tmp/matrix-home-wide

env HOME=/tmp/matrix-home-compact FEATURE_MODE=compact blade-offline build //:matrix_app
./build64_release/matrix_app > /tmp/matrix-compact.out
grep -qx 'MATRIX_RESULT mode=compact value=32' /tmp/matrix-compact.out

rm -rf .blade_tools build64_release
env HOME=/tmp/matrix-home-wide FEATURE_MODE=wide blade-offline build //:matrix_app
./build64_release/matrix_app > /tmp/matrix-wide.out
grep -qx 'MATRIX_RESULT mode=wide value=64' /tmp/matrix-wide.out
