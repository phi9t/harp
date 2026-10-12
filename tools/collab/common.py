"""Shared collab helpers with no storage or Git side effects."""
from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import os
from pathlib import Path
import re


class Refused(Exception):
    pass


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def valid_id(value: str) -> str:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,99}", value):
        raise Refused(f"Invalid identifier: {value!r}")
    return value


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def child_environment() -> dict[str, str]:
    environment = os.environ.copy()
    # Git and check children must resolve the repository from their assigned cwd.
    for key in list(environment):
        if key.startswith("GIT_") and key not in ("GIT_SSH_COMMAND", "GIT_SSH"):
            environment.pop(key)
    environment["GIT_TERMINAL_PROMPT"] = "0"
    return environment


def output_path(value: str | Path) -> Path:
    path = Path(value).expanduser().resolve()
    forbidden = Path("/") / "tmp"
    if path == forbidden or forbidden in path.parents or path == Path("/"):
        raise Refused("Output paths must be outside the system temporary directory and filesystem root")
    return path


def failpoint(name: str) -> None:
    # Test-only process death; deliberately bypass Python cleanup.
    if os.environ.get("COLLAB_FAILPOINT") == name:
        os._exit(86)
