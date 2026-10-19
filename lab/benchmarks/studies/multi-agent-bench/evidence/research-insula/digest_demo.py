#!/usr/bin/env python3
"""Canonical tree digest demo for capsule layers."""

from __future__ import annotations

import hashlib
import os
import stat
import sys
from pathlib import Path


EXCLUDE_NAMES = {
    ".DS_Store",
    "__pycache__",
}
EXCLUDE_SUFFIXES = {
    ".pyc",
    ".pyo",
}


def include_path(path: Path) -> bool:
    if path.name in EXCLUDE_NAMES:
        return False
    if path.suffix in EXCLUDE_SUFFIXES:
        return False
    return True


def canonical_tree_digest(root: Path) -> str:
    h = hashlib.sha256()
    paths = sorted(path for path in root.rglob("*") if include_path(path))
    for path in paths:
        rel = path.relative_to(root).as_posix()
        info = path.lstat()
        mode = stat.S_IMODE(info.st_mode)
        if stat.S_ISDIR(info.st_mode):
            h.update(f"D {mode:o} {rel}\n".encode())
        elif stat.S_ISLNK(info.st_mode):
            h.update(f"L {mode:o} {rel} -> {os.readlink(path)}\n".encode())
        elif stat.S_ISREG(info.st_mode):
            fh = hashlib.sha256()
            with path.open("rb") as handle:
                for chunk in iter(lambda: handle.read(1024 * 1024), b""):
                    fh.update(chunk)
            h.update(f"F {mode:o} {rel} {fh.hexdigest()}\n".encode())
        else:
            h.update(f"O {stat.S_IFMT(info.st_mode):o} {mode:o} {rel} {info.st_rdev}\n".encode())
    return "sha256:" + h.hexdigest()


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: digest_demo.py <tree>", file=sys.stderr)
        return 2
    print(canonical_tree_digest(Path(argv[1]).resolve(strict=True)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
