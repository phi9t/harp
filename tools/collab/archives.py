"""Verified, self-contained archives of a worktree and its staging state."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import tarfile
import tempfile
import uuid

from common import Refused, output_path
from workspace import run_git


def inventory(root: Path) -> dict:
    result = {}
    for directory, dirs, files in os.walk(root, followlinks=False):
        if Path(directory) == root:
            dirs[:] = [name for name in dirs if name != ".git"]
            files = [name for name in files if name != ".git"]
        elif ".git" in dirs or ".git" in files:
            raise Refused("Nested Git checkout requires separate preservation; retaining worktree")
        for name in dirs + files:
            path = Path(directory) / name
            mode = path.lstat().st_mode
            relative = str(path.relative_to(root))
            if stat.S_ISLNK(mode):
                value = {"link": os.readlink(path)}
            elif stat.S_ISREG(mode):
                value = {"sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
            elif stat.S_ISDIR(mode):
                value = {"directory": True}
            else:
                raise Refused(f"Unsupported special file; retaining worktree: {path}")
            result[relative] = {"mode": stat.S_IMODE(mode), **value}
    return result


def restore_archive(source: str, destination: str) -> dict:
    archive = output_path(source)
    target = output_path(destination)
    if target.exists() or target.is_symlink():
        raise Refused("Restore destination must not exist")
    manifest = json.loads((archive / "manifest.json").read_text())
    for name, expected in manifest["artifacts"].items():
        if hashlib.sha256((archive / name).read_bytes()).hexdigest() != expected:
            raise Refused(f"Archive artifact failed verification: {name}")
    target.mkdir(parents=True)
    run_git(target, "init")
    run_git(target, "fetch", str(archive / "history.bundle"),
            manifest["head_ref"] + ":refs/collab/restored/head",
            manifest["index_ref"] + ":refs/collab/restored/index")
    run_git(target, "checkout", "-b", manifest["branch"], manifest["head"])
    for child in target.iterdir():
        if child.name == ".git":
            continue
        if child.is_dir() and not child.is_symlink():
            shutil.rmtree(child)
        else:
            child.unlink()
    with tarfile.open(archive / "files.tar", "r") as stream:
        members = stream.getmembers()
        for member in members:
            relative = Path(member.name)
            if relative.is_absolute() or ".." in relative.parts or relative.parts[0] == ".git":
                raise Refused("Unsafe path in worktree archive")
            if member.islnk() or not (member.isfile() or member.isdir() or member.issym()):
                raise Refused("Unsupported archive member")
            if any((target / parent).is_symlink() for parent in relative.parents):
                raise Refused("Archive entry crosses a symlink")
            stream.extract(member, target)
    shutil.copyfile(archive / "index", target / ".git" / "index")
    if inventory(target) != manifest["files"]:
        raise Refused("Restored file inventory differs from archive")
    return {"restored": str(target), "archive": str(archive), "head": manifest["head"]}


def archive_worktree(workspace, home: Path, item: dict) -> dict:
    path = Path(item["path"])
    before = inventory(path)
    gitdir = Path(workspace.git("rev-parse", "--path-format=absolute", "--git-dir", cwd=path).strip())
    if list(gitdir.glob("sharedindex.*")):
        raise Refused("Split index needs separate preservation; retaining worktree")
    index = (gitdir / "index").read_bytes()
    token = uuid.uuid4().hex
    directory = home / "archives" / token
    directory.mkdir(parents=True, mode=0o700)
    head_ref = item.get("archive_ref") or f"refs/collab/archive/{token}/head"
    workspace.git("update-ref", head_ref, item["head"], "")
    (directory / "index").write_bytes(index)
    staging_index = directory / "staging-index"
    staging_index.write_bytes(index)
    tree = workspace.git("write-tree", cwd=path, env={"GIT_INDEX_FILE": str(staging_index)}).strip()
    staging_index.unlink()
    staging = workspace.git(
        "commit-tree", tree, "-p", item["head"], input="Preserved sweep index\n",
        env={"GIT_AUTHOR_NAME": "Collab archive", "GIT_AUTHOR_EMAIL": "archive@example.invalid",
             "GIT_COMMITTER_NAME": "Collab archive", "GIT_COMMITTER_EMAIL": "archive@example.invalid"},
    ).strip()
    index_ref = f"refs/collab/archive/{token}/index"
    workspace.git("update-ref", index_ref, staging, "")
    workspace.git("bundle", "create", str(directory / "history.bundle"), head_ref, index_ref)
    workspace.git("bundle", "verify", str(directory / "history.bundle"))
    (directory / "index").write_bytes(index)
    (directory / "staged.patch").write_bytes(
        workspace.git("diff", "--binary", "--cached", cwd=path, raw=True, env={"GIT_OPTIONAL_LOCKS": "0"})
    )
    (directory / "unstaged.patch").write_bytes(
        workspace.git("diff", "--binary", cwd=path, raw=True, env={"GIT_OPTIONAL_LOCKS": "0"})
    )
    with tarfile.open(directory / "files.tar", "w", dereference=False) as stream:
        for name in sorted(before):
            stream.add(path / name, arcname=name, recursive=False)
    artifacts = {
        entry.name: hashlib.sha256(entry.read_bytes()).hexdigest() for entry in directory.iterdir()
    }
    manifest = {"schema": 1, "branch": item["branch"], "head": item["head"], "files": before,
                "artifacts": artifacts, "head_ref": head_ref, "index_ref": index_ref}
    (directory / "manifest.json").write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
    if inventory(path) != before or (gitdir / "index").read_bytes() != index:
        raise Refused("Worktree changed during archive; retaining worktree")
    with tempfile.TemporaryDirectory(prefix="verify-", dir=directory) as verification:
        restored = Path(verification) / "restored"
        restore_archive(str(directory), str(restored))
        if (restored / ".git" / "index").read_bytes() != index:
            raise Refused("Archive did not restore the exact index")
    if inventory(path) != before or (gitdir / "index").read_bytes() != index:
        raise Refused("Worktree changed during archive verification; retaining worktree")
    return {"archive": str(directory), "ref": head_ref, "index_ref": index_ref,
            "branch": item["branch"], "head": item["head"], "verified": True}


def clear_archived_changes(workspace, path: Path) -> None:
    """Make only a verified, identity-checked retired tree clean for normal Git removal."""
    for entry in workspace.git("ls-files", "-v", "-z", cwd=path).split("\0"):
        if entry and (entry[0].islower() or entry[0] == "S"):
            workspace.git("update-index", "--no-assume-unchanged", "--no-skip-worktree", "--", entry[2:], cwd=path)
    workspace.git("restore", "--source=HEAD", "--staged", "--worktree", "--", ".", cwd=path)
    names = set()
    for flags in (("--others", "--exclude-standard"), ("--others", "--ignored", "--exclude-standard")):
        names.update(workspace.git("ls-files", *flags, "-z", cwd=path).split("\0"))
    for name in sorted(names - {""}, reverse=True):
        file = path / name
        if file.is_symlink() or file.is_file():
            file.unlink()
    for directory, dirs, files in os.walk(path, topdown=False, followlinks=False):
        current = Path(directory)
        if current != path and current.name != ".git" and not current.is_symlink():
            try:
                current.rmdir()
            except OSError:
                pass
    workspace.clean(path, include_ignored=True)
