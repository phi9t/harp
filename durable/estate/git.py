"""Constrained Git primitives shared by admission and workspace owners."""
from __future__ import annotations
import os
import hashlib
import stat
from pathlib import Path, PurePosixPath
import re
import subprocess

def relative_path(value):
    path = PurePosixPath(value)
    if (not isinstance(value, str) or not value or path.is_absolute() or
            str(path) != value or any(p in {"..", ".git"} for p in path.parts) or
            "\\" in value or "\x00" in value):
        raise ValueError("expected a canonical repository-relative path outside .git")
    return value


def git(repo, *args, output=None):
    environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL="/dev/null", GIT_TERMINAL_PROMPT="0", GIT_ATTR_NOSYSTEM="1")
    result = subprocess.run(["git", "--no-replace-objects", "--no-optional-locks", "-c", "core.hooksPath=/dev/null",
                           "-c", "core.fsmonitor=false", "-c", "core.attributesFile=/dev/null", "-C", str(repo), *args],
                          env=environment, stdout=output or subprocess.PIPE,
                          stderr=subprocess.PIPE)
    if result.returncode:
        raise ValueError("Git admission failed: " + result.stderr.decode(errors="replace")[:4096].strip())
    return result.stdout


def export_bundle(repo, base, destination, temporary):
    # A raw commit ID is not an advertised bundle ref. Create that ref only in
    # an owned export repository, leaving the source checkout and its refs alone.
    export = Path(temporary) / "export.git"
    export.mkdir()
    git(export, "init", "--bare", "-q")
    git(export, "fetch", "--no-tags", str(repo), base + ":refs/heads/source")
    git(export, "bundle", "create", str(destination), "refs/heads/source")


def commit(repo, ref):
    result = git(repo, "rev-parse", "--verify", "--end-of-options", ref + "^{commit}").decode().strip()
    if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", result):
        raise ValueError("unsupported Git object identity")
    return result


def blob(repo, revision, path):
    relative_path(path)
    entry = git(repo, "ls-tree", "-z", revision, "--", path)
    if not entry or entry.split(b" ", 1)[0] not in {b"100644", b"100755"}:
        raise ValueError("task and acceptance inputs must be committed regular files")
    data = git(repo, "cat-file", "blob", revision + ":" + path)
    if len(data) > 256 * 1024:
        raise ValueError("task/acceptance input exceeds 256 KiB")
    return data


def tracked_tree(workspace, commit):
    result = []
    for entry in git(workspace, "ls-tree", "-r", "-z", commit).split(b"\0"):
        if not entry:
            continue
        metadata, raw_name = entry.split(b"\t", 1)
        mode, kind, blob = metadata.decode().split()
        if kind != "blob" or mode not in {"100644", "100755", "120000"}:
            raise ValueError("verification does not support Git submodules or special tree entries")
        name = os.fsdecode(raw_name)
        relative = Path(name)
        if relative.is_absolute() or any(p in {"..", ".git"} for p in relative.parts):
            raise ValueError("unsafe tracked candidate path")
        result.append((name, mode, blob))
    return result


def unchanged(workspace, tree):
    """Ignore the mutable index and compare filesystem bytes directly to Git blobs."""
    try:
        for name, mode, expected in tree:
            path = workspace / name
            if any(parent.is_symlink() for parent in path.parents if parent != workspace.parent):
                return False
            info = path.lstat()
            hashed = hashlib.sha1() if len(expected) == 40 else hashlib.sha256()
            if mode == "120000":
                if not stat.S_ISLNK(info.st_mode):
                    return False
                data = os.fsencode(os.readlink(path))
                hashed.update(b"blob " + str(len(data)).encode() + b"\0" + data)
            else:
                if not stat.S_ISREG(info.st_mode) or bool(info.st_mode & stat.S_IXUSR) != (mode == "100755"):
                    return False
                fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
                with os.fdopen(fd, "rb") as source:
                    actual = os.fstat(source.fileno())
                    if not stat.S_ISREG(actual.st_mode):
                        return False
                    hashed.update(b"blob " + str(actual.st_size).encode() + b"\0")
                    for block in iter(lambda: source.read(1 << 20), b""):
                        hashed.update(block)
            if hashed.hexdigest() != expected:
                return False
    except OSError:
        return False
    return True
