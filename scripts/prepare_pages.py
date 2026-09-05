#!/usr/bin/env python3
"""Stage only the receipt-bound Atlas export for GitHub Pages."""

import hashlib
import json
from pathlib import Path
import re
import sys
import tempfile


def reject_symlinks(path):
    for part in (path, *path.parents):
        if part.is_symlink():
            raise ValueError(f"symlink is not allowed: {part}")


def read_regular(path):
    reject_symlinks(path)
    if not path.is_file():
        raise ValueError(f"expected a regular file: {path}")
    return path.read_bytes()


def prepare(root):
    output = root / ".build/pages"
    reject_symlinks(output)
    if output.exists():
        raise ValueError(f"output already exists: {output}")

    html = read_regular(root / "atlas/dist/harp-atlas.html")
    receipt_bytes = read_regular(root / "atlas/dist/harp-atlas.receipt.json")
    corpus = read_regular(root / "atlas/src/content/generated/corpus.json")
    try:
        receipt = json.loads(receipt_bytes)
    except (ValueError, UnicodeError) as error:
        raise ValueError("invalid export receipt JSON") from error
    digests = ("html_sha256", "corpus_sha256", "app_inputs_sha256")
    if (
        not isinstance(receipt, dict)
        or set(receipt) != {"schema_version", *digests}
        or receipt["schema_version"] != "harp-atlas-export/v1"
        or any(
            not isinstance(receipt[key], str)
            or re.fullmatch(r"[0-9a-f]{64}", receipt[key]) is None
            for key in digests
        )
    ):
        raise ValueError("invalid export receipt")
    for key, payload in (("html_sha256", html), ("corpus_sha256", corpus)):
        if hashlib.sha256(payload).hexdigest() != receipt[key]:
            raise ValueError(f"{key} digest mismatch; regenerate the Atlas")

    output.parent.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="pages-", dir=output.parent) as staging:
        staged = Path(staging) / "site"
        staged.mkdir()
        (staged / "index.html").write_bytes(html)
        (staged / "harp-atlas.receipt.json").write_bytes(receipt_bytes)
        reject_symlinks(output)
        if output.exists():
            raise ValueError(f"output already exists: {output}")
        staged.rename(output)
    print(f"GitHub Pages artifact ready: {output}")


if __name__ == "__main__":
    try:
        prepare(Path.cwd())
    except (OSError, ValueError) as error:
        print(f"Pages preparation failed: {error}", file=sys.stderr)
        sys.exit(1)
