#!/bin/sh
set -eu

cp /tmp/ticket/overlay/src/rules.cpp src/rules.cpp

python3 - <<'PY'
from pathlib import Path

path = Path("include/rules_engine.hpp")
text = path.read_text()
ordered = ["tokenizer", "expression", "rule-store", "driver"]
present = [name for name in ordered if f'"{name}"' in text]
if "rule-store" not in present:
    present.append("rule-store")
present = [name for name in ordered if name in present]
line = "    return {%s};" % ", ".join(f'"{name}"' for name in present)
lines = []
for old in text.splitlines():
    if old.strip().startswith("return {"):
        lines.append(line)
    else:
        lines.append(old)
path.write_text("\n".join(lines) + "\n")
PY
