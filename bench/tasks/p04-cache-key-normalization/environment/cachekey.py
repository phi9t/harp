from __future__ import annotations

import hashlib
from pathlib import Path


def build_cache_key(source: str | Path, workspace: str | Path) -> str:
    source_path = Path(source).resolve()
    digest = hashlib.sha256(source_path.read_bytes()).hexdigest()
    return f"{source_path}:{digest}"
