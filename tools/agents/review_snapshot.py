#!/usr/bin/env python3
"""Capture a review-only WIP artifact without modifying the source repository."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tempfile


class SnapshotError(Exception):
    """The source cannot be captured completely and consistently."""


def git(repo, *args, data=None, success_codes=(0,)):
    # Ignore caller Git redirections, replacement objects and optional index writes.
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    env.update(GIT_OPTIONAL_LOCKS="0", GIT_NO_REPLACE_OBJECTS="1",
               GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
               GIT_TERMINAL_PROMPT="0", GIT_NO_LAZY_FETCH="1")
    result = subprocess.run(
        ["git", "--no-optional-locks", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null",
         "-C", os.fspath(repo), *args], input=data, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, env=env, check=False)
    if result.returncode not in success_codes:
        raise SnapshotError(result.stderr.decode(errors="replace").strip())
    return result.stdout


def safe_path(path):
    if not path or path.startswith(b"/") or any(
            part in (b"", b".", b"..", b".git") for part in path.split(b"/")):
        raise SnapshotError(f"unsafe Git path: {path!r}")
    return path


def fingerprint(info):
    return (info.st_dev, info.st_ino, info.st_mode, info.st_size,
            info.st_mtime_ns, info.st_ctime_ns)


def read_worktree_file(root_fd, path):
    """No followed symlinks, including ancestors; never open a FIFO/device."""
    parts = safe_path(path).split(b"/")
    parent = os.dup(root_fd)
    try:
        for part in parts[:-1]:
            try:
                ancestor = os.stat(part, dir_fd=parent, follow_symlinks=False)
                if stat.S_ISREG(ancestor.st_mode):
                    return None
                child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent)
            except FileNotFoundError:
                return None
            os.close(parent)
            parent = child
        try:
            before = os.stat(parts[-1], dir_fd=parent, follow_symlinks=False)
        except FileNotFoundError:
            return None
        if stat.S_ISDIR(before.st_mode):
            return None
        if stat.S_ISLNK(before.st_mode):
            content = os.readlink(parts[-1], dir_fd=parent)
            mode = b"120000"
        elif stat.S_ISREG(before.st_mode):
            fd = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
            with os.fdopen(fd, "rb") as stream:
                if fingerprint(os.fstat(stream.fileno())) != fingerprint(before):
                    raise SnapshotError(f"input changed while opening {path!r}")
                content = stream.read()
                if fingerprint(os.fstat(stream.fileno())) != fingerprint(before):
                    raise SnapshotError(f"input changed while reading {path!r}")
            mode = b"100755" if before.st_mode & 0o111 else b"100644"
        else:
            raise SnapshotError(f"unsupported file kind: {path!r}")
        after = os.stat(parts[-1], dir_fd=parent, follow_symlinks=False)
        if fingerprint(before) != fingerprint(after):
            raise SnapshotError(f"input changed while reading {path!r}")
        return mode, content, fingerprint(after)
    finally:
        os.close(parent)


def entries(repo, revision=None):
    raw = git(repo, "ls-tree", "-rz", "--full-tree", revision) if revision else git(repo, "ls-files", "--stage", "-z")
    result = {}
    for record in raw.split(b"\0"):
        if not record:
            continue
        header, path = record.split(b"\t", 1)
        safe_path(path)
        mode, middle, final = header.split()
        oid = final if revision else middle
        if not revision and final != b"0":
            raise SnapshotError(f"unmerged index entry: {path!r}")
        if mode not in (b"100644", b"100755", b"120000"):
            raise SnapshotError(f"unsupported Git mode {mode!r}: {path!r}")
        result[path] = (mode, oid)
    return result


def source_state(repo, base):
    """Capture identities surrounding each scan, including the raw source index."""
    index = Path(os.fsdecode(git(repo, "rev-parse", "--path-format=absolute", "--git-path", "index").removesuffix(b"\n")))
    index_bytes = index.read_bytes() if index.exists() else b""
    return {
        "head": git(repo, "rev-parse", "--verify", "HEAD^{commit}").strip().decode(),
        "base": git(repo, "rev-parse", "--verify", "--end-of-options", base + "^{commit}").strip().decode(),
        "refs": git(repo, "for-each-ref", "--format=%(refname) %(objectname)"),
        "symbolic_head": git(repo, "rev-parse", "--symbolic-full-name", "HEAD"),
        "index_sha256": hashlib.sha256(index_bytes).hexdigest(),
    }


def scan_worktree(repo, index_entries):
    result = {}
    root_fd = os.open(repo, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        paths = set(index_entries)
        tracked_directories = {path[:offset] for path in paths for offset, byte in enumerate(path) if byte == ord("/")}

        def walk(directory_fd, prefix=b""):
            names = sorted(os.fsencode(name) for name in os.listdir(directory_fd))
            candidates = [prefix + name for name in names if prefix or name != b".git"]
            if not candidates:
                return
            ignored = set(git(repo, "check-ignore", "--stdin", "-z", data=b"\0".join(candidates) + b"\0",
                              success_codes=(0, 1)).split(b"\0"))
            for path in candidates:
                if path in ignored and path not in paths and path not in tracked_directories:
                    continue
                safe_path(path)
                name = path.rsplit(b"/", 1)[-1]
                info = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
                if stat.S_ISDIR(info.st_mode):
                    fd = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=directory_fd)
                    try:
                        walk(fd, path + b"/")
                    finally:
                        os.close(fd)
                else:
                    paths.add(path)

        walk(root_fd)
        for path in sorted(paths):
            entry = read_worktree_file(root_fd, path)
            if entry is not None:
                result[path] = entry
    finally:
        os.close(root_fd)
    return result


def capture(repo, base):
    initial = source_state(repo, base)
    merge_bases = git(repo, "merge-base", "--all", initial["base"], initial["head"]).splitlines()
    if len(merge_bases) != 1:
        raise SnapshotError("review requires exactly one merge-base")
    merge_base = merge_bases[0].decode()
    states = {"base": entries(repo, merge_base), "head": entries(repo, initial["head"]), "index": entries(repo)}
    worktree = scan_worktree(repo, states["index"])
    object_ids = sorted({oid for state in states.values() for _, oid in state.values()})
    raw = git(repo, "cat-file", "--batch", data=b"\n".join(object_ids) + b"\n" if object_ids else b"")
    blobs, offset = {}, 0
    for oid in object_ids:
        end = raw.index(b"\n", offset)
        header = raw[offset:end].split()
        if len(header) != 3 or header[0] != oid or header[1] != b"blob":
            raise SnapshotError(f"missing or invalid source blob: {oid!r}")
        size = int(header[2])
        blobs[oid] = raw[end + 1:end + 1 + size]
        offset = end + size + 2
    contents = {name: {path: (mode, blobs[oid]) for path, (mode, oid) in state.items()} for name, state in states.items()}
    contents["worktree"] = {path: (mode, content) for path, (mode, content, _) in worktree.items()}
    if (source_state(repo, base) != initial or entries(repo) != states["index"]
            or scan_worktree(repo, states["index"]) != worktree
            or source_state(repo, base) != initial):
        raise SnapshotError("input changed during capture; retry with writers paused")
    return initial, merge_base, contents, worktree


def write_tree(destination, contents, blob_cache=None):
    if blob_cache is None:
        blob_cache = {}
    records = []
    inventory = []
    git(destination, "read-tree", "--empty")
    for path, (mode, content) in sorted(contents.items()):
        digest = hashlib.sha256(content).hexdigest()
        if digest not in blob_cache:
            blob_cache[digest] = git(destination, "hash-object", "-w", "--stdin", data=content).strip()
        oid = blob_cache[digest]
        records.append(mode + b" " + oid + b"\t" + path + b"\0")
        inventory.append({"path": os.fsdecode(path), "path_base64": base64.b64encode(path).decode(),
                          "mode": mode.decode(), "git_blob": oid.decode(),
                          "sha256": digest, "size": len(content)})
    git(destination, "update-index", "-z", "--index-info", data=b"".join(records))
    return git(destination, "write-tree").strip().decode(), inventory


def snapshot(repo, base, output):
    repo = Path(os.fsdecode(git(repo, "rev-parse", "--show-toplevel").removesuffix(b"\n"))).resolve()
    output = Path(output).absolute()
    if output.exists() or output.is_symlink():
        raise SnapshotError("output must not exist")
    parent = output.parent.resolve(strict=True)
    output = parent / output.name
    common = Path(os.fsdecode(git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").removesuffix(b"\n"))).resolve()
    if output.is_relative_to(repo) or output.is_relative_to(common):
        raise SnapshotError("output must be outside the source worktree and Git directory")
    initial, merge_base, contents, worktree = capture(repo, base)
    # Only private output state is mutated. Publish after the final stability check.
    staging = Path(tempfile.mkdtemp(prefix=".review-snapshot-", dir=parent))
    try:
        destination = staging / "repository"
        destination.mkdir()
        git(destination, "init", "--bare", "--template=", "--object-format=sha1", ".")
        trees, inventory, blob_cache = {}, {}, {}
        for name, state in contents.items():
            trees[name], inventory[name] = write_tree(destination, state, blob_cache)
            git(destination, "update-ref", "refs/snapshots/" + name, trees[name])
        for filename, left, right in [("committed", "base", "head"), ("staged", "head", "index"),
                                      ("unstaged", "index", "worktree"), ("combined", "base", "worktree")]:
            patch = git(destination, "diff", "--no-ext-diff", "--no-textconv", "--no-renames", "--binary",
                        "--full-index", trees[left], trees[right], "--")
            (staging / (filename + ".patch")).write_bytes(patch)
        manifest = {"schema": 1, "kind": "wip-review-snapshot", "landing_acceptance": False,
                    "source_repository": str(repo), "requested_base": base,
                    "resolved_base": initial["base"], "source_head": initial["head"], "merge_base": merge_base,
                    "source_index_sha256": initial["index_sha256"], "trees": trees, "files": inventory,
                    "limitations": ["Review aid only; not a Candidate, Integration commit or landing acceptance.",
                                    "Includes all indexed files and non-ignored untracked files; ignored files are excluded.",
                                    "Worktree bytes are raw: no clean/smudge filters or line-ending conversion.",
                                    "Git modes preserve executable and symlink identity, not ownership, xattrs or other filesystem metadata.",
                                    "Two-pass change detection is cooperative, not an atomic filesystem snapshot or a hostile-writer boundary.",
                                    "Conflicts, submodules, special files and symlink ancestors are refused."]}
        (staging / "manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=True) + "\n")
        (staging / "README.md").write_text(
            "# WIP review snapshot\n\nReview committed.patch, staged.patch and unstaged.patch together. "
            "combined.patch shows the net result and can hide intermediate staged changes.\n\n"
            "manifest.json records source identities, limitations and each tree's paths, modes and blob hashes. "
            "The bare repository is self-contained; inspect bytes with "
            "`git --git-dir=repository show refs/snapshots/worktree:path` or inspect any of base/head/index/worktree. "
            "No checkout is needed. Do not execute captured files or hooks.\n\n"
            "This artifact is a review aid, not a Candidate or landing acceptance.\n")
        if source_state(repo, base) != initial or scan_worktree(repo, entries(repo)) != worktree or source_state(repo, base) != initial:
            raise SnapshotError("input changed during artifact creation; retry with writers paused")
        # mkdir reserves the output without replacing another process's artifact.
        output.mkdir(mode=0o700)
        try:
            staging.rename(output)
        except BaseException:
            output.rmdir()
            raise
        return manifest
    finally:
        if staging.exists():
            shutil.rmtree(staging)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default=".")
    parser.add_argument("--base", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    try:
        manifest = snapshot(args.repo, args.base, args.output)
    except (SnapshotError, OSError) as error:
        parser.exit(1, f"review snapshot refused: {error}\n")
    print(json.dumps({"output": str(Path(args.output).absolute()), "trees": manifest["trees"]}))


if __name__ == "__main__":
    main()
