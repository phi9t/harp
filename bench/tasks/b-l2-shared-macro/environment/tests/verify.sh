#!/usr/bin/env bash
set -euo pipefail

blade-offline build //alpha:alpha_tool //beta:beta_tool //gamma:gamma_tool
test "$(./build64_release/alpha/alpha_tool)" = "BLADE_L2_MACRO alpha=5"
test "$(./build64_release/beta/beta_tool)" = "BLADE_L2_MACRO beta=11"
test "$(./build64_release/gamma/gamma_tool)" = "BLADE_L2_MACRO gamma=13"
