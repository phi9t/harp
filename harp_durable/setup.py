"""Prepare a code-pinned, network-free execution on a delegated cgroup.

Preparation records inputs and uses the imported runtime's capsule builder. It
never starts an activity or installs a service.
"""
from __future__ import annotations

import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import tempfile
from typing import Any

from controller.durable.cgroup import Cgroup
from controller.durable.common import atomic_json, durable_mkdir, encode, fsync_dir, read_json
from controller.durable.insula import InsulaExecution, snapshot_code
from tools.insula import capsule


def native_manifest(rootfs: Path) -> dict[str, Any]:
    """Inventory every native file, mode and link without following links.

    Unlike the capsule host fingerprint, this includes file bytes and files
    normally excluded from source-tree digests. Absolute links are interpreted
    by the rootfs at execution, so their literal targets are retained.
    """
    if rootfs.is_symlink() or not rootfs.is_dir():
        raise ValueError("rootfs must be an ordinary directory")
    rootfs = rootfs.resolve(strict=True)
    entries: list[dict[str, Any]] = []
    for path in sorted(rootfs.rglob("*")):
        info = path.lstat()
        item: dict[str, Any] = {"path": path.relative_to(rootfs).as_posix(),
                                "mode": stat.S_IMODE(info.st_mode)}
        if stat.S_ISLNK(info.st_mode):
            item.update(kind="symlink", target=os.readlink(path))
        elif stat.S_ISDIR(info.st_mode):
            item.update(kind="directory")
        elif stat.S_ISREG(info.st_mode):
            fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
            with os.fdopen(fd, "rb") as stream:
                before = os.fstat(stream.fileno())
                if not stat.S_ISREG(before.st_mode):
                    raise ValueError("native rootfs changed during inventory")
                hasher = hashlib.sha256()
                for block in iter(lambda: stream.read(1 << 20), b""):
                    hasher.update(block)
                digest = hasher.hexdigest()
                after = os.fstat(stream.fileno())
            if ((before.st_dev, before.st_ino, before.st_mode, before.st_size,
                 before.st_mtime_ns, before.st_ctime_ns) !=
                (after.st_dev, after.st_ino, after.st_mode, after.st_size,
                 after.st_mtime_ns, after.st_ctime_ns)):
                raise ValueError("native rootfs changed during inventory")
            item.update(kind="file", size=after.st_size, sha256=digest)
        else:
            raise ValueError("native rootfs contains a special file")
        entries.append(item)
    return {"protocol": 1, "rootfs": str(rootfs),
            "root_mode": stat.S_IMODE(rootfs.stat().st_mode), "entries": entries}


def _manifest_bytes(manifest: dict[str, Any]) -> bytes:
    return json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode()


def _publish_new(path: Path, data: bytes) -> None:
    """Publish fsynced bytes without replacing any existing path."""
    durable_mkdir(path.parent)
    fd, temporary = tempfile.mkstemp(prefix=".prepare-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.link(temporary, path)
        fsync_dir(path.parent)
    finally:
        os.unlink(temporary)


def verify_native_manifest(binding: dict[str, Any]) -> None:
    """Reject changed native bytes or replacement of their pinned inventory."""
    if (not isinstance(binding, dict) or set(binding) != {"path", "sha256", "rootfs"} or
            any(not isinstance(value, str) for value in binding.values())):
        raise ValueError("invalid native manifest binding")
    path = Path(binding["path"])
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise ValueError("native manifest must be an ordinary file")
        recorded = stream.read()
    if hashlib.sha256(recorded).hexdigest() != binding["sha256"]:
        raise ValueError("native manifest binding changed")
    if _manifest_bytes(native_manifest(Path(binding["rootfs"]))) != recorded:
        raise ValueError("admitted native rootfs changed")


def _stage_code(destination: Path) -> Path:
    """Admit only the public Python package and the imported runtime closure."""
    source = Path(__file__).resolve().parent
    imports = destination / "imports"
    package = imports / "harp_durable"
    package.mkdir(parents=True)
    for path in source.rglob("*.py"):
        if ("__pycache__" in path.parts or path.name.startswith("test_") or
                path.name.endswith("_test.py")):
            continue
        if path.is_symlink():
            raise ValueError("public runtime source must not be symlinked")
        target = package / path.relative_to(source)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)
    return snapshot_code(destination / "code", imports)


def _private_state(state: Path) -> Path:
    if state.is_symlink():
        raise ValueError("state must not be symlinked")
    durable_mkdir(state)
    info = state.stat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise ValueError("state must be a private directory owned by the current user")
    return state.resolve(strict=True)


def prepare(state: Path, rootfs: Path, cgroup_root: Path, execution_file: Path) -> dict[str, Any]:
    """Pin an existing native closure and code; require real cgroup delegation.

    Repeating preparation is safe only while the admitted inputs remain exact.
    Changing source, native files, state, store or cgroup generation requires a
    separate execution file; existing records are never replaced.
    """
    state = _private_state(state)
    os.environ["CORENIUS_CAPSULES"] = str(state / "capsules")
    Cgroup.validate_root(cgroup_root)
    manifest = native_manifest(rootfs)
    rootfs = Path(manifest["rootfs"])
    execution_file = execution_file.absolute()
    if state.is_relative_to(rootfs) or execution_file.is_relative_to(rootfs):
        raise ValueError("mutable execution state must be outside the native rootfs")
    if execution_file.is_symlink():
        raise ValueError("execution file must not be symlinked")
    lock = os.open(state / "prepare.lock", os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        with tempfile.TemporaryDirectory(dir=state, prefix="prepare-") as temporary:
            staging = Path(temporary)
            source = _stage_code(staging)
            if execution_file.exists():
                config = read_json(execution_file)
                execution = InsulaExecution(config)
                execution.verify()
                verify_native_manifest(config["native_manifest"])
                if (config["state"] != str(state) or config["backend"] != "cgroup" or
                        config["cgroup_root"] != str(cgroup_root.resolve()) or
                        config["native_manifest"]["rootfs"] != str(rootfs) or
                        config.get("inputs_root") != str(state / "inputs") or
                        config["code_digest"] != capsule.tree_digest(source) or
                        config["capsule"]["policy"]["net"] != "none"):
                    raise ValueError("execution file already binds different inputs")
                return config
            request = {"name": "harp-durable", "layers": [
                {"kind": "rootfs", "name": "native", "source": str(rootfs)}],
                "policy": {"net": "none", "gpu": False, "proxy": False}}
            request_path = staging / "capsule.json"
            atomic_json(request_path, request)
            record = capsule.build(str(request_path))
            digest = capsule.build_tree_layer({"kind": "tree", "name": "harp-durable-code",
                                               "source": str(source)}, capsule.store_root())
            code = capsule.store_root() / "layers" / digest / "tree"
            manifest_path = execution_file.with_name(execution_file.name + ".native.json")
            manifest_bytes = _manifest_bytes(manifest)
            binding = {"path": str(manifest_path), "rootfs": str(rootfs),
                       "sha256": hashlib.sha256(manifest_bytes).hexdigest()}
            if manifest_path.is_symlink():
                raise ValueError("native manifest must not be symlinked")
            if manifest_path.exists():
                verify_native_manifest(binding)
            else:
                _publish_new(manifest_path, manifest_bytes)
            execution = InsulaExecution.create(record, code, state, backend="cgroup",
                cgroup_root=cgroup_root, inputs_root=state / "inputs", native_manifest=binding)
            execution.verify()
            _publish_new(execution_file, encode(execution.config).encode())
            return execution.config
    finally:
        os.close(lock)
