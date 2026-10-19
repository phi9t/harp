#!/bin/sh
set -eu

mkdir -p /run/insula-tmp/matrix-home-compact /run/insula-tmp/matrix-home-wide

env HOME=/run/insula-tmp/matrix-home-compact FEATURE_MODE=compact blade-offline build //:matrix_app
./build64_release/matrix_app > /run/insula-tmp/matrix-compact.out
grep -qx 'MATRIX_RESULT mode=compact value=32' /run/insula-tmp/matrix-compact.out

rm -rf .blade_tools build64_release
env HOME=/run/insula-tmp/matrix-home-wide FEATURE_MODE=wide blade-offline build //:matrix_app
./build64_release/matrix_app > /run/insula-tmp/matrix-wide.out
grep -qx 'MATRIX_RESULT mode=wide value=64' /run/insula-tmp/matrix-wide.out
