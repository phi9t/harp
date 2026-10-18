#!/bin/sh
set -eu

python3 - <<'PY'
from pathlib import Path

build = Path("BUILD")
lines = build.read_text().splitlines()
updated = []
for line in lines:
    if line.startswith("FUSED_OP_COMPONENTS = "):
        present = [name for name in ("params-codegen", "native-plugin", "python-wrapper") if name in line]
        if "native-plugin" not in present:
            present.append("native-plugin")
        ordered = [name for name in ("params-codegen", "native-plugin", "python-wrapper") if name in present]
        line = 'FUSED_OP_COMPONENTS = [%s]' % ", ".join('"%s"' % name for name in ordered)
    updated.append(line)
build.write_text("\n".join(updated) + "\n")

Path("csrc/BUILD").write_text('''cc_library(
    name="fused_core",
    srcs=["fused.cc", "fused.h"],
    deps=["//:params_codegen"],
    incs=["."],
    export_incs=["."],
)

cc_plugin(
    name="fused_plugin",
    srcs=["plugin.cc"],
    deps=[
        "//:params_codegen",
        ":fused_core",
    ],
    incs=["."],
)
''')

Path("csrc/fused.cc").write_text('''#include "fused.h"

void fused_scale_add(const Params *params, const float *x, float *y, int n) {
  if (!params || !x || !y || n <= 0) {
    return;
  }
  for (int i = 0; i < n; ++i) {
    y[i] = x[i] * params->scale + params->bias;
  }
}
''')

Path("csrc/plugin.cc").write_text('''#include "fused.h"

extern "C" int fused_op_apply_v2(const Params *params, const float *x, float *y, int n) {
  if (!params || (!x && n > 0) || (!y && n > 0) || n < 0) {
    return 1;
  }
  fused_scale_add(params, x, y, n);
  return 0;
}
''')
PY
