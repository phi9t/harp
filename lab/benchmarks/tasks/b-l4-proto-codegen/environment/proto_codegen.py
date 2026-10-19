#!/usr/bin/env python3
import re
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: proto_codegen.py <schema.proto> <out.h>", file=sys.stderr)
        return 2

    schema = Path(sys.argv[1]).read_text()
    out = Path(sys.argv[2])
    fields = {}
    for line in schema.splitlines():
        match = re.match(r"\s*(\w+)\s+(\w+)\s*=\s*(\d+)\s*;", line)
        if match:
            _kind, name, number = match.groups()
            fields[name] = int(number)

    if fields.get("id") != 1 or fields.get("shard") != 2:
        print("record schema must contain id=1 and shard=2", file=sys.stderr)
        return 1

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        "#pragma once\n\n"
        "namespace bladebench {\n"
        "inline int record_id_field_number() { return 1; }\n"
        "inline int record_shard_field_number() { return 2; }\n"
        "inline const char* record_shard_field_name() { return \"shard\"; }\n"
        "}  // namespace bladebench\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
