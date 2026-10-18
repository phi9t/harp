"""JSON protocol and filesystem commit utilities. Trusted local activities only."""
from __future__ import annotations

import hashlib
import json
import os
import stat
from pathlib import Path, PurePosixPath
import tempfile
from typing import Any
from tools.temp_root import temporary_root

MAX_JSON = 1 << 20  # Small control data only; large artifacts are external files.
MANAGED_TEMP_ENV = frozenset({"TMPDIR", "TMP", "TEMP", "TMUX_TMPDIR", "TMPPREFIX"})


def input_digest(value: Any) -> str:
    if not isinstance(value, str) or len(value) != 64 or any(c not in "0123456789abcdef" for c in value):
        raise ValueError("input digest must be 64 lowercase hexadecimal characters")
    return value


def verify_inputs_root(root: Path) -> Path:
    """Validate an existing private store without resolving away symlink ancestry."""
    root = Path(root)
    if not root.is_absolute() or ".." in root.parts:
        raise ValueError("inputs root must be an absolute path without traversal")
    for parent in (root, *root.parents):
        if parent.is_symlink():
            raise ValueError("inputs root must not contain symlinks")
    info = root.stat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise ValueError("inputs root must be an owned private directory (mode 0700)")
    return root


def verify_input_bundle(root: Path, bundle_digest: str) -> Path:
    """Verify precisely the declared immutable inputs before projecting them read-only."""
    root = verify_inputs_root(root)
    bundle = root / input_digest(bundle_digest)
    info = bundle.lstat()
    if not stat.S_ISDIR(info.st_mode):
        raise ValueError("input bundle must be a regular directory, not a symlink")
    manifest_path = bundle / "manifest.json"
    manifest = read_json(manifest_path)
    if (not isinstance(manifest, dict) or set(manifest) != {"schema", "files"}
            or type(manifest["schema"]) is not int or manifest["schema"] != 1
            or not isinstance(manifest["files"], dict)):
        raise ValueError("invalid input manifest")
    if digest(manifest) != bundle_digest or manifest_path.read_bytes() != encode(manifest).encode():
        raise ValueError("input manifest digest or canonical encoding changed")
    directories = set()
    for name, description in manifest["files"].items():
        if (not isinstance(name, str) or not name or "\\" in name or "\x00" in name
                or PurePosixPath(name).is_absolute() or PurePosixPath(name).as_posix() != name
                or any(part in {".", ".."} for part in PurePosixPath(name).parts)
                or name == "manifest.json"):
            raise ValueError("input manifest paths must be canonical relative file paths")
        if (not isinstance(description, dict) or set(description) != {"sha256", "size"}
                or type(description["size"]) is not int or description["size"] < 0):
            raise ValueError("invalid input file description")
        input_digest(description["sha256"])
        directories.update(str(parent) for parent in PurePosixPath(name).parents if str(parent) != ".")
    actual_files = set()
    actual_directories = set()
    for directory, names, files in os.walk(bundle, followlinks=False):
        for name in [*names, *files]:
            path = Path(directory) / name
            info = path.lstat()
            relative = path.relative_to(bundle).as_posix()
            if stat.S_ISDIR(info.st_mode):
                actual_directories.add(relative)
            elif stat.S_ISREG(info.st_mode):
                actual_files.add(relative)
            else:
                raise ValueError("input bundle contains a symlink or special file")
    if (actual_files != {"manifest.json", *manifest["files"]}
            or actual_directories != directories):
        raise ValueError("input bundle files or directories differ from its manifest")
    for name, description in manifest["files"].items():
        fd = os.open(bundle / name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as source:
            info = os.fstat(source.fileno())
            if not stat.S_ISREG(info.st_mode) or info.st_size != description["size"]:
                raise ValueError("input file type or size changed")
            hashed = hashlib.sha256()
            size = 0
            while block := source.read(1 << 20):
                hashed.update(block)
                size += len(block)
            if size != description["size"] or hashed.hexdigest() != description["sha256"]:
                raise ValueError("input file bytes changed")
    return bundle


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
