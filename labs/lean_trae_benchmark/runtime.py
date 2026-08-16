"""Sealed, Harp-owned TRAE runtime provisioning and resolution.

This module deliberately accepts a supplied artifact rather than acquiring one.
Acquisition is an explicit operator step: the provisioner verifies its bytes
against the committed lock before the executable is ever run.
"""

from __future__ import annotations

import ctypes
import errno
import json
import os
import platform
import shutil
import stat
import subprocess
import uuid
from dataclasses import dataclass
from pathlib import Path

from model import RuntimeLock, ValidationError, parse_json_object, require_exact_fields, sha256_bytes


DEFAULT_RUNTIME_LOCK = Path(__file__).with_name("runtime.lock.json")
_INVENTORY_NAME = "inventory.json"
_INVENTORY_SCHEMA = "harp-lean-runtime-inventory/v1"
_VERSION_TIMEOUT_SECONDS = 30
_DARWIN_AT_FDCWD = -2
_DARWIN_RENAME_EXCL = 0x00000004


@dataclass(frozen=True)
class ResolvedRuntime:
    """A verified Harp-owned runtime tree, never a caller-selected CLI."""

    runtime_id: str
    root: Path
    trae: Path
    inventory: Path


def load_runtime_lock(path: Path) -> RuntimeLock:
    """Load one explicit, strict lock without consulting environment overrides."""
    _require_regular_file(path, "runtime lock")
    try:
        return RuntimeLock.from_value(parse_json_object(path.read_bytes(), "runtime lock"))
    except (OSError, ValidationError) as error:
        raise ValidationError(f"cannot load runtime lock: {error}") from error


def provision_runtime(lock: RuntimeLock, artifact: Path, runtime_base: Path) -> ResolvedRuntime:
    """Verify and atomically publish a new, immutable runtime directory.

    ``runtime_base`` is a Harp-controlled directory, not the runtime itself.
    The lock's ``published_path`` determines the sole destination beneath that
    base.  It must not exist: a provisioner never replaces a possibly valid
    runtime.  If validation after publication fails, this call removes the new
    tree; an existing destination is rejected before staging and remains
    untouched.
    """
    _require_host_platform(lock)
    artifact_mode = _verify_supplied_artifact(lock, artifact)
    root = _prepare_runtime_destination(lock, runtime_base)
    _prepare_destination(root)
    _reject_stale_staging(root)
    stage = root.parent / f".{root.name}.runtime-stage-{uuid.uuid4().hex}"
    _create_real_directory(stage, "runtime staging directory")
    published_identity: tuple[int, int] | None = None
    try:
        staged_trae = stage / "traecli"
        _copy_executable(artifact, staged_trae, artifact_mode)
        # The source can be replaced between its initial verification and copy.
        # Never run the staged result until its own bytes are lock-verified.
        _verify_supplied_artifact(lock, staged_trae)
        _verify_trae_version(staged_trae, lock)
        _write_inventory(stage, lock)
        _verify_runtime_tree(stage, lock)
        _publish_new_directory(stage, root)
        published_identity = _directory_identity(root, "published runtime directory")
        _verify_runtime_tree(root, lock)
        return _resolved_runtime(lock, root)
    except Exception:
        if published_identity is not None:
            _remove_new_runtime(root, published_identity)
        raise
    finally:
        if stage.exists() or stage.is_symlink():
            _remove_stage(stage)


def resolve_runtime(lock: RuntimeLock, runtime_base: Path) -> ResolvedRuntime:
    """Resolve only a previously published Harp-owned runtime after rechecking it."""
    _require_host_platform(lock)
    root = _runtime_root(lock, runtime_base)
    _verify_runtime_tree(root, lock)
    return _resolved_runtime(lock, root)


def _resolved_runtime(lock: RuntimeLock, root: Path) -> ResolvedRuntime:
    return ResolvedRuntime(
        runtime_id=lock.runtime_id,
        root=root,
        trae=root / "traecli",
        inventory=root / _INVENTORY_NAME,
    )


def _require_host_platform(lock: RuntimeLock) -> None:
    os_name = platform.system().lower()
    machine = platform.machine().lower()
    aliases = {"aarch64": "arm64", "amd64": "x86_64"}
    machine = aliases.get(machine, machine)
    if (os_name, machine) != (lock.os, lock.arch):
        raise ValidationError(
            f"runtime lock platform {lock.os}/{lock.arch} does not support host {os_name}/{machine}"
        )


def _runtime_root(lock: RuntimeLock, runtime_base: Path) -> Path:
    _require_real_directory(runtime_base, "runtime base")
    parts = lock.trae.published_path.split("/")
    if len(parts) < 2 or parts[-1] != "traecli":
        raise ValidationError("runtime lock published path must name the fixed traecli executable")
    return runtime_base.joinpath(*parts[:-1])


def _prepare_runtime_destination(lock: RuntimeLock, runtime_base: Path) -> Path:
    root = _runtime_root(lock, runtime_base)
    _ensure_real_directories(runtime_base, root.parent)
    return root


def _verify_supplied_artifact(lock: RuntimeLock, artifact: Path) -> int:
    _require_regular_file(artifact, "supplied TRAE artifact")
    metadata = artifact.lstat()
    if not metadata.st_mode & 0o111:
        raise ValidationError("supplied TRAE artifact must be executable")
    if metadata.st_size != lock.trae.artifact.bytes:
        raise ValidationError("supplied TRAE artifact byte size does not match runtime lock")
    if sha256_bytes(artifact.read_bytes()) != lock.trae.artifact.digest:
        raise ValidationError("supplied TRAE artifact digest does not match runtime lock")
    return metadata.st_mode & 0o777


def _prepare_destination(root: Path) -> None:
    _require_real_directory(root.parent, "runtime destination parent")
    if _path_exists_or_symlink(root):
        if root.is_symlink():
            raise ValidationError("runtime destination must not be a symlink")
        raise ValidationError("runtime destination already exists and will not be overwritten")


def _stage_prefix(root: Path) -> str:
    return f".{root.name}.runtime-stage-"


def _reject_stale_staging(root: Path) -> None:
    prefix = _stage_prefix(root)
    try:
        stale = [entry for entry in root.parent.iterdir() if entry.name.startswith(prefix)]
    except OSError as error:
        raise ValidationError(f"cannot inspect runtime staging parent: {error}") from error
    if stale:
        raise ValidationError("stale runtime staging directory exists; inspect and remove it before provisioning")


def _create_real_directory(path: Path, label: str) -> None:
    try:
        path.mkdir(mode=0o700)
    except OSError as error:
        raise ValidationError(f"cannot create {label}: {error}") from error
    _require_real_directory(path, label)


def _copy_executable(source: Path, target: Path, mode: int) -> None:
    try:
        _copy_regular_file(source, target)
        target.chmod(mode)
    except OSError as error:
        raise ValidationError(f"cannot copy supplied TRAE artifact: {error}") from error
    _require_regular_file(target, "staged TRAE executable")
    if not target.lstat().st_mode & 0o111:
        raise ValidationError("staged TRAE executable is not executable")


def _copy_regular_file(source: Path, target: Path) -> None:
    """Small test seam around the byte-copy operation; verification stays outside it."""
    shutil.copyfile(source, target, follow_symlinks=False)


def _verify_trae_version(trae: Path, lock: RuntimeLock) -> None:
    _require_regular_file(trae, "TRAE executable")
    try:
        completed = subprocess.run(
            [str(trae), "--version"],
            cwd=trae.parent,
            env={"PATH": os.defpath},
            text=True,
            capture_output=True,
            check=False,
            timeout=_VERSION_TIMEOUT_SECONDS,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ValidationError(f"cannot run staged TRAE --version: {error}") from error
    if completed.returncode != 0:
        raise ValidationError("staged TRAE --version command failed")
    expected = f"{lock.trae.version_stdout}\n"
    if completed.stdout != expected:
        raise ValidationError("staged TRAE version stdout does not exactly match runtime lock")


def _inventory_value(lock: RuntimeLock, trae: Path) -> dict[str, object]:
    data = trae.read_bytes()
    return {
        "files": [
            {
                "bytes": len(data),
                "path": "traecli",
                "sha256": sha256_bytes(data).hex,
            }
        ],
        "runtime_id": lock.runtime_id,
        "schema_version": _INVENTORY_SCHEMA,
    }


def _write_inventory(stage: Path, lock: RuntimeLock) -> None:
    inventory = stage / _INVENTORY_NAME
    if _path_exists_or_symlink(inventory):
        raise ValidationError("runtime inventory target already exists")
    try:
        with inventory.open("xb") as handle:
            handle.write(_canonical_inventory_bytes(_inventory_value(lock, stage / "traecli")))
            handle.write(b"\n")
    except OSError as error:
        raise ValidationError(f"cannot write runtime inventory: {error}") from error


def _verify_runtime_tree(root: Path, lock: RuntimeLock) -> None:
    _require_real_directory(root, "runtime directory")
    try:
        entries = {entry.name: entry for entry in root.iterdir()}
    except OSError as error:
        raise ValidationError(f"cannot inspect runtime directory: {error}") from error
    if set(entries) != {"traecli", _INVENTORY_NAME}:
        raise ValidationError("runtime directory contains an unexpected file set")
    trae = entries["traecli"]
    inventory = entries[_INVENTORY_NAME]
    _require_regular_file(trae, "published TRAE executable")
    _require_regular_file(inventory, "runtime inventory")
    if not trae.lstat().st_mode & 0o111:
        raise ValidationError("published TRAE executable is not executable")
    _verify_supplied_artifact(lock, trae)
    _verify_trae_version(trae, lock)
    expected = _canonical_inventory_bytes(_inventory_value(lock, trae)) + b"\n"
    try:
        raw = inventory.read_bytes()
    except OSError as error:
        raise ValidationError(f"cannot read runtime inventory: {error}") from error
    if raw != expected:
        raise ValidationError("runtime inventory is not the deterministic locked inventory")
    # Parse once too, so malformed JSON is never accepted through byte equality
    # if this serializer is changed in a future schema revision.
    parsed = parse_json_object(raw, "runtime inventory")
    require_exact_fields(parsed, ("schema_version", "runtime_id", "files"), "runtime inventory")
    if parsed != _inventory_value(lock, trae):
        raise ValidationError("runtime inventory does not match published runtime")


def _publish_new_directory(stage: Path, root: Path) -> None:
    # This precheck makes ordinary conflicts clear; the OS primitive below is
    # still required because a competitor can appear after this check.
    if _path_exists_or_symlink(root):
        raise ValidationError("runtime destination appeared during provisioning and will not be overwritten")
    _rename_directory_no_replace(stage, root)


def _rename_directory_no_replace(stage: Path, root: Path) -> None:
    """Publish with Darwin's kernel-enforced no-replace rename operation."""
    if platform.system() != "Darwin":
        raise ValidationError("atomic no-replace runtime publication is unsupported on this host OS")
    stage_bytes = os.fsencode(stage)
    root_bytes = os.fsencode(root)
    if b"\x00" in stage_bytes or b"\x00" in root_bytes:
        raise ValidationError("runtime publication paths must not contain NUL bytes")
    try:
        renameatx_np = ctypes.CDLL(None, use_errno=True).renameatx_np
    except AttributeError as error:
        raise ValidationError("host Darwin runtime lacks renameatx_np no-replace publication") from error
    renameatx_np.argtypes = (
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_uint,
    )
    renameatx_np.restype = ctypes.c_int
    result = renameatx_np(
        _DARWIN_AT_FDCWD,
        stage_bytes,
        _DARWIN_AT_FDCWD,
        root_bytes,
        _DARWIN_RENAME_EXCL,
    )
    if result == 0:
        return
    error_number = ctypes.get_errno()
    if error_number in (errno.EEXIST, errno.ENOTEMPTY):
        raise ValidationError("runtime destination appeared during publication and was not overwritten")
    raise ValidationError(
        f"cannot atomically publish runtime with no-replace rename: {os.strerror(error_number)}"
    )


def _remove_new_runtime(root: Path, expected_identity: tuple[int, int]) -> None:
    if not _path_exists_or_symlink(root):
        return
    if _directory_identity(root, "failed runtime directory") != expected_identity:
        raise ValidationError("refusing to remove a replaced failed runtime")
    try:
        shutil.rmtree(root)
    except OSError as error:
        raise ValidationError(f"cannot roll back failed runtime publication: {error}") from error


def _remove_stage(stage: Path) -> None:
    if stage.is_symlink():
        raise ValidationError("refusing to remove a symlinked runtime staging path")
    try:
        shutil.rmtree(stage)
    except OSError as error:
        raise ValidationError(f"cannot remove runtime staging directory: {error}") from error


def _require_regular_file(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        raise ValidationError(f"{label} must be a regular non-symlink file")


def _require_real_directory(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode):
        raise ValidationError(f"{label} must be a real non-symlink directory")


def _ensure_real_directories(base: Path, target: Path) -> None:
    """Create only missing descendants of a checked real runtime base."""
    try:
        relative = target.relative_to(base)
    except ValueError as error:
        raise ValidationError("runtime destination must remain below its controlled base") from error
    current = base
    for part in relative.parts:
        current = current / part
        if _path_exists_or_symlink(current):
            _require_real_directory(current, "runtime destination parent")
            continue
        try:
            current.mkdir(mode=0o700)
        except OSError as error:
            raise ValidationError(f"cannot create runtime destination parent: {error}") from error
        _require_real_directory(current, "runtime destination parent")


def _directory_identity(path: Path, label: str) -> tuple[int, int]:
    _require_real_directory(path, label)
    metadata = path.lstat()
    return (metadata.st_dev, metadata.st_ino)


def _path_exists_or_symlink(path: Path) -> bool:
    try:
        path.lstat()
    except FileNotFoundError:
        return False
    except OSError as error:
        raise ValidationError(f"cannot inspect runtime path: {error}") from error
    return True


def _canonical_inventory_bytes(value: dict[str, object]) -> bytes:
    """Encode the closed inventory with stable JSON syntax and no floats."""
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")
