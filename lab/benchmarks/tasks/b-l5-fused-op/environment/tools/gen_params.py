#!/usr/bin/env python3
import re
import sys
from pathlib import Path


FIELD_RE = re.compile(r"^\s*(float)\s+([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$")


def parse_fields(text):
    fields = []
    in_struct = False
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line == "struct Params {":
            in_struct = True
            continue
        if line == "}":
            in_struct = False
            continue
        if not in_struct:
            raise ValueError(f"unexpected line outside Params struct: {raw}")
        match = FIELD_RE.match(raw)
        if not match:
            raise ValueError(f"unsupported IDL field: {raw}")
        c_type, name = match.groups()
        fields.append((c_type, name))
    if not fields:
        raise ValueError("Params must declare at least one field")
    if fields[0] != ("float", "scale"):
        raise ValueError("Params first field must be float scale")
    return fields


def render(fields):
    lines = [
        "#pragma once",
        "",
        "#ifdef __cplusplus",
        'extern "C" {',
        "#endif",
        "",
        "typedef struct Params {",
    ]
    for c_type, name in fields:
        lines.append(f"  {c_type} {name};")
    lines.extend([
        "} Params;",
        "",
        f"#define PARAMS_FIELD_COUNT {len(fields)}",
        "",
        "#ifdef __cplusplus",
        "}",
        "#endif",
        "",
    ])
    return "\n".join(lines)


def main():
    if len(sys.argv) != 3:
        print("usage: gen_params.py <params.idl> <out.h>", file=sys.stderr)
        return 2
    fields = parse_fields(Path(sys.argv[1]).read_text())
    out = Path(sys.argv[2])
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(render(fields))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
