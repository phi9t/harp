#!/usr/bin/env python3
from __future__ import annotations

import pathlib
import sys


def main() -> int:
    if "--version" in sys.argv:
        print("Lean fake 4.0.0")
        return 0
    if len(sys.argv) != 2:
        print("usage: fake-lean.py <module.lean>", file=sys.stderr)
        return 2
    path = pathlib.Path(sys.argv[1])
    text = path.read_text(encoding="utf-8")
    if "LEAN_SUITE_FAIL" in text:
        print("fake lean: type mismatch", file=sys.stderr)
        return 1
    print(f"fake lean checked {path.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
