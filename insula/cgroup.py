"""Optional, fail-closed cgroup v2 cleanup for an activity attempt.

The host guardian creates a subgroup, persists ``identity()`` before release,
and supplies ``join_current_process`` as its trusted, single-threaded Popen
preexec_fn. Joining happens before exec; descendants inherit that membership.
The payload's Insula must expose neither cgroup control paths nor these fds.
Keep the ownership flock until ``kill_and_wait`` returns kernel evidence.
Recovery uses ``resume`` with the persisted attempt identity, never saved PIDs.

This module does not delegate privileges, expose resource controllers, remove
groups, or fall back to process groups. Keep empty groups for recovery proof.
Kernel interface: https://docs.kernel.org/admin-guide/cgroup-v2.html
"""
from __future__ import annotations

from contextlib import contextmanager
import ctypes
import hashlib
import math
import os
from pathlib import Path
import re
import secrets
import sys
import time
from typing import Any, Iterator, Mapping


class CgroupUnavailable(RuntimeError):
    """Delegation, capability, or durable identity cannot be verified."""


class CgroupCleanupUnproven(RuntimeError):
    """Kernel evidence did not prove the attempt subtree empty."""


_CGROUP2_MAGIC = 0x63677270
_ATTEMPT = re.compile(r"[A-Za-z0-9_-]{1,128}\Z")
_GENERATION = re.compile(r"[0-9a-f]{32}\Z")
_IDENTITY_KEYS = frozenset({"protocol", "root", "root_device", "root_inode",
                            "group_device", "group_inode", "attempt_id", "generation"})
_DIR_FLAGS = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW


def _attempt(value: Any) -> str:
    if not isinstance(value, str) or not _ATTEMPT.fullmatch(value):
        raise CgroupUnavailable("attempt_id must be 1..128 ASCII letters, digits, _ or -")
    return value


def _root_path(value: Any) -> Path:
    if not isinstance(value, (str, Path)) or not str(value) or not Path(value).is_absolute():
        raise CgroupUnavailable("an explicit absolute delegated cgroup root is required")
    path = Path(value)
    if str(path) != str(value) or ".." in path.parts:
        raise CgroupUnavailable("delegated cgroup root must be a canonical absolute path")
    return path


def _validate_identity(value: Any) -> dict[str, Any]:
    if not isinstance(value, Mapping) or set(value) != _IDENTITY_KEYS:
        raise CgroupUnavailable("malformed persisted cgroup identity")
    result = dict(value)
    if type(result["protocol"]) is not int or result["protocol"] != 1:
        raise CgroupUnavailable("unsupported cgroup identity protocol")
    _attempt(result["attempt_id"])
    _root_path(result["root"])
    if not isinstance(result["generation"], str) or not _GENERATION.fullmatch(result["generation"]):
        raise CgroupUnavailable("malformed cgroup generation")
    for key in ["root_device", "root_inode", "group_device", "group_inode"]:
        if type(result[key]) is not int or not 0 < result[key] < 2**64:
            raise CgroupUnavailable("malformed cgroup filesystem identity")
    return result


def _verify_cgroup2(fd: int) -> None:
    if sys.platform != "linux":
        raise CgroupUnavailable("cgroup v2 requires Linux")
    # fstatfs's first native long is f_type on Linux. Reserve more than the
    # largest supported struct statfs so libc, rather than a path/name, proves
    # that this descriptor refers to the real kernel cgroup2 filesystem.
    buffer = ctypes.create_string_buffer(512)
    libc = ctypes.CDLL(None, use_errno=True)
    libc.fstatfs.argtypes = [ctypes.c_int, ctypes.c_void_p]
    libc.fstatfs.restype = ctypes.c_int
    if libc.fstatfs(fd, buffer) != 0:
        raise OSError(ctypes.get_errno(), "fstatfs")
    if ctypes.c_long.from_buffer(buffer).value != _CGROUP2_MAGIC:
        raise CgroupUnavailable("delegated root is not a kernel cgroup v2 filesystem")


def _file(fd: int, name: str, flags: int) -> int:
    return os.open(name, flags | os.O_CLOEXEC | os.O_NOFOLLOW, dir_fd=fd)


def _read(fd: int, name: str) -> str:
    with os.fdopen(_file(fd, name, os.O_RDONLY), "r", encoding="ascii") as stream:
        return stream.read(65536)


def _check_writable(fd: int, name: str) -> None:
    os.close(_file(fd, name, os.O_WRONLY))


@contextmanager
def _root(value: Any) -> Iterator[tuple[Path, int]]:
    path = _root_path(value)
    fd = None
    try:
        if path.resolve(strict=True) != path:
            raise CgroupUnavailable("delegated cgroup root must not contain symlinks")
        fd = os.open(path, _DIR_FLAGS)
        _verify_cgroup2(fd)
        if not os.access(path, os.W_OK | os.X_OK, effective_ids=True):
            raise CgroupUnavailable("delegated cgroup root is not writable")
        if _read(fd, "cgroup.type").strip() != "domain":
            raise CgroupUnavailable("delegated cgroup root must have domain type")
        _check_writable(fd, "cgroup.procs")
        _check_writable(fd, "cgroup.subtree_control")
        yield path, fd
    except OSError as exc:
        raise CgroupUnavailable(f"delegated cgroup root unavailable: {exc}") from exc
    finally:
        if fd is not None:
            os.close(fd)


def _matches(fd: int, identity: Mapping[str, Any], prefix: str) -> None:
    stat = os.fstat(fd)
    if (stat.st_dev, stat.st_ino) != (identity[f"{prefix}_device"], identity[f"{prefix}_inode"]):
        raise CgroupUnavailable(f"{prefix} cgroup identity changed")


def _bound_root(identity: Mapping[str, Any], root_identity: Mapping[str, Any] | None) -> None:
    if root_identity is None:
        return
    keys = {"root", "root_device", "root_inode"}
    if not isinstance(root_identity, Mapping) or set(root_identity) != keys:
        raise CgroupUnavailable("malformed delegated root binding")
    for key in keys:
        if type(root_identity[key]) is not type(identity[key]) or root_identity[key] != identity[key]:
            raise CgroupUnavailable("delegated root identity disagrees with execution binding")


def _populated(fd: int) -> tuple[int, str]:
    events = _read(fd, "cgroup.events")
    values = [line.split() for line in events.splitlines() if line.startswith("populated ")]
    if len(values) != 1 or len(values[0]) != 2 or values[0][1] not in {"0", "1"}:
        raise CgroupUnavailable("kernel cgroup.events has no valid populated evidence")
    return int(values[0][1]), events


class Cgroup:
    """Persistable attempt binding; every operation revalidates kernel identity."""

    def __init__(self, identity: Mapping[str, Any]):
        self._identity = _validate_identity(identity)

    @property
    def path(self) -> Path:
        identity = self._identity
        name = "_corenius-attempt-" + hashlib.sha256(identity["attempt_id"].encode()).hexdigest()[:24]
        return Path(identity["root"]) / (name + "-" + identity["generation"])

    def identity(self) -> dict[str, Any]:
        return dict(self._identity)

    @classmethod
    def validate_root(cls, root: str | Path) -> dict[str, Any]:
        """Read-only admission check; subgroup kill support is checked at create."""
        with _root(root) as (path, fd):
            stat = os.fstat(fd)
            return dict(root=str(path), root_device=stat.st_dev, root_inode=stat.st_ino)

    @classmethod
    def create(cls, root: str | Path, attempt_id: str, *,
               root_identity: Mapping[str, Any] | None = None) -> Cgroup:
        _attempt(attempt_id)
        with _root(root) as (path, root_fd):
            root_stat = os.fstat(root_fd)
            identity = dict(protocol=1, root=str(path), attempt_id=attempt_id,
                            generation=secrets.token_hex(16), root_device=root_stat.st_dev,
                            root_inode=root_stat.st_ino, group_device=root_stat.st_dev, group_inode=1)
            _bound_root(identity, root_identity)
            group = cls(identity)
            created = False
            try:
                os.mkdir(group.path.name, mode=0o700, dir_fd=root_fd)
                created = True
                fd = os.open(group.path.name, _DIR_FLAGS, dir_fd=root_fd)
                try:
                    _verify_cgroup2(fd)
                    if _read(fd, "cgroup.type").strip() != "domain":
                        raise CgroupUnavailable("attempt cgroup must have domain type")
                    _check_writable(fd, "cgroup.procs")
                    _check_writable(fd, "cgroup.kill")
                    if _populated(fd)[0] != 0:
                        raise CgroupUnavailable("new attempt cgroup is already populated")
                    stat = os.fstat(fd)
                    identity.update(group_device=stat.st_dev, group_inode=stat.st_ino)
                finally:
                    os.close(fd)
                return cls(identity)
            except (OSError, CgroupUnavailable) as exc:
                if created:
                    # No payload was launched. Never recursively remove or kill
                    # an unexpected subgroup after failed capability checks.
                    try:
                        os.rmdir(group.path.name, dir_fd=root_fd)
                    except OSError:
                        pass
                raise CgroupUnavailable(f"attempt cgroup creation failed: {exc}") from exc

    @classmethod
    def resume(cls, identity: Mapping[str, Any], *, attempt_id: str,
               root_identity: Mapping[str, Any] | None = None) -> Cgroup:
        group = cls(identity)
        if group._identity["attempt_id"] != _attempt(attempt_id):
            raise CgroupUnavailable("persisted cgroup identity belongs to another attempt")
        _bound_root(group._identity, root_identity)
        with group._open():
            pass
        return group

    @contextmanager
    def _open(self) -> Iterator[int]:
        with _root(self._identity["root"]) as (_, root_fd):
            _matches(root_fd, self._identity, "root")
            fd = None
            try:
                fd = os.open(self.path.name, _DIR_FLAGS, dir_fd=root_fd)
                _verify_cgroup2(fd)
                _matches(fd, self._identity, "group")
                if _read(fd, "cgroup.type").strip() != "domain":
                    raise CgroupUnavailable("attempt cgroup no longer has domain type")
                _check_writable(fd, "cgroup.procs")
                _check_writable(fd, "cgroup.kill")
                _populated(fd)
                yield fd
            except OSError as exc:
                raise CgroupUnavailable(f"attempt cgroup unavailable: {exc}") from exc
            finally:
                if fd is not None:
                    os.close(fd)

    def join_current_process(self) -> None:
        """Trusted preexec gate. An error prevents the payload executable launch.

        Reopen control descriptors here rather than passing them through exec.
        Use in a single-threaded guardian: Python preexec_fn is not thread-safe.
        Only the current process is moved; this API never accepts a saved PID.
        """
        with self._open() as fd:
            pid = os.getpid()
            procs = _file(fd, "cgroup.procs", os.O_WRONLY)
            try:
                data = f"{pid}\n".encode("ascii")
                if os.write(procs, data) != len(data):
                    raise CgroupUnavailable("incomplete cgroup membership write")
            finally:
                os.close(procs)
            if str(pid) not in _read(fd, "cgroup.procs").split():
                raise CgroupUnavailable("kernel did not confirm own cgroup membership")

    def observation(self) -> dict[str, Any]:
        """Read identity-bound kernel population without signaling any process.

        Emptiness alone does not prove that no pending launch can materialize;
        the authoritative caller must fence admissions before using this result.
        """
        with self._open() as fd:
            populated, events = _populated(fd)
            return dict(backend="cgroup-v2", identity=self.identity(),
                        populated=populated, events=events, observed_at=time.time())

    def kill_and_wait(self, *, timeout: float = 3.0) -> dict[str, Any]:
        """Kill the exact subtree, returning proof only after populated becomes 0.

        The caller must prevent new release while holding its ownership flock.
        Missing/replaced cgroups are unproven, including after a host reboot.
        """
        if type(timeout) not in {int, float} or not math.isfinite(timeout) or not 0 < timeout <= 60:
            raise ValueError("cleanup timeout must be finite and in (0, 60] seconds")
        with self._open() as fd:
            kill = _file(fd, "cgroup.kill", os.O_WRONLY)
            try:
                if os.write(kill, b"1\n") != 2:
                    raise CgroupCleanupUnproven("incomplete cgroup.kill write")
            finally:
                os.close(kill)
            deadline = time.monotonic() + timeout
            while True:
                populated, events = _populated(fd)
                if populated == 0:
                    return dict(backend="cgroup-v2", identity=self.identity(),
                                kill_written=True, populated=0, events=events,
                                observed_at=time.time())
                if time.monotonic() >= deadline:
                    raise CgroupCleanupUnproven("cgroup subtree remains populated after cleanup deadline")
                time.sleep(min(0.01, max(0, deadline - time.monotonic())))


def validate_root(root: str | Path) -> Path:
    """Read-only root validation for launcher admission.

    Cgroup.validate_root returns the identity dictionary when an immutable
    execution binding must also pin the root's device and inode.
    """
    return Path(Cgroup.validate_root(root)["root"])
