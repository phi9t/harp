"""JSON protocol and filesystem commit utilities. Trusted local activities only."""
from __future__ import annotations

import hashlib
import json
import os
import stat
from pathlib import Path
import tempfile
from typing import Any
from tools.temp_root import temporary_root

MAX_JSON = 1 << 20  # Small control data only; large artifacts are external files.
MANAGED_TEMP_ENV = frozenset({"TMPDIR", "TMP", "TEMP", "TMUX_TMPDIR", "TMPPREFIX"})


def temporary_environment() -> dict[str, str]:
    """Approved host temporaries, injected at launch rather than journaled."""
    root = str(temporary_root())
    environment = dict(TMPDIR=root, TMP=root, TEMP=root)
    if "TMUX_TMPDIR" in os.environ:
        environment["TMUX_TMPDIR"] = str(temporary_root(os.environ["TMUX_TMPDIR"]))
    if "TMPPREFIX" in os.environ:
        prefix = Path(os.environ["TMPPREFIX"]).expanduser()
        if not prefix.is_absolute() or prefix == Path("/"):
            raise ValueError("TMPPREFIX must be an absolute owned prefix")
        parent = temporary_root(str(prefix.parent))
        environment["TMPPREFIX"] = str(parent / prefix.name)
    return environment


def encode(value: Any) -> str:
    text = json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)
    if len(text.encode()) > MAX_JSON:
        raise ValueError("Control JSON exceeds 1 MiB; use an artifact reference")
    return text


def digest(value: Any) -> str:
    return hashlib.sha256(encode(value).encode()).hexdigest()


def fsync_dir(path: Path) -> None:
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def durable_mkdir(path: Path) -> None:
    if path.is_dir():
        return
    durable_mkdir(path.parent)
    try:
        path.mkdir(mode=0o700)
        fsync_dir(path.parent)
    except FileExistsError:
        if not path.is_dir():
            raise


def atomic_json(path: Path, value: Any) -> None:
    """Atomic replacement on one filesystem, including file + directory fsync."""
    data = encode(value).encode()
    durable_mkdir(path.parent)
    fd, name = tempfile.mkstemp(prefix=".tmp-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as out:
            out.write(data)
            out.flush()
            os.fsync(out.fileno())
        os.replace(name, path)
        fsync_dir(path.parent)
    finally:
        if os.path.exists(name):
            os.unlink(name)


def read_json(path: Path) -> Any:
    # Result files come from the payload. A FIFO must not park the guardian
    # forever after cleanup; only bounded ordinary files are protocol data.
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as inp:
        info = os.fstat(inp.fileno())
        if not stat.S_ISREG(info.st_mode):
            raise ValueError("control JSON must be a regular file")
        if info.st_size > MAX_JSON:
            raise ValueError("Oversized JSON")
        data = inp.read(MAX_JSON + 1)
    if len(data) > MAX_JSON:
        raise ValueError("Oversized JSON")
    value = json.loads(data)
    encode(value)  # Reject NaN/Infinity and enforce the canonical protocol limit.
    return value


def publish_result(request: dict[str, Any], result: Any) -> None:
    """Activity-side result. This is NOT the authoritative workflow commit."""
    atomic_json(Path(request["attempt_dir"]) / "result.json", result)
