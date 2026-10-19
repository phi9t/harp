#!/usr/bin/env python3
from __future__ import annotations

import pathlib
import sys


def main() -> int:
    if "--version" in sys.argv:
        print("Lake fake 4.0.0")
        return 0
    if sys.argv[1:] != ["build"]:
        print("usage: fake-lake.py build", file=sys.stderr)
        return 2
    if not pathlib.Path("lakefile.toml").is_file():
        print("missing lakefile.toml", file=sys.stderr)
        return 1
    print("fake lake build ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
