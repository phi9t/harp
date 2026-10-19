from __future__ import annotations

import hashlib
from pathlib import Path


def build_cache_key(source: str | Path, workspace: str | Path) -> str:
    workspace_path = Path(workspace).resolve()
    source_path = Path(source).resolve()
    try:
        relative = source_path.relative_to(workspace_path)
    except ValueError as exc:
        raise ValueError(f"{source_path} is not inside {workspace_path}") from exc
    digest = hashlib.sha256(source_path.read_bytes()).hexdigest()
    return f"{relative.as_posix()}:{digest}"
