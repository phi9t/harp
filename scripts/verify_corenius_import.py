#!/usr/bin/env python3
"""Check that the Corenius absorption preserves its declared source and history."""
import hashlib
import json
from pathlib import Path
import re
import stat
import subprocess
import sys


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root)


def verify(root):
    component = root / "corenius"
    if component.is_symlink():
        raise ValueError("Corenius component must not be a symlink")
    manifest_path = component / "HARP_IMPORT.json"
    if manifest_path.is_symlink():
        raise ValueError("Corenius import manifest must not be a symlink")
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("schema") != "harp-corenius-import/v1":
        raise ValueError("unsupported Corenius import manifest")
    source = manifest["source_commit"]
    if not re.fullmatch(r"[0-9a-f]{40}", source):
        raise ValueError("Corenius source must be an exact Git commit")
    tree = {}
    for row in git(root, "ls-tree", "-r", "-z", source).split(b"\0"):
        if not row:
            continue
        metadata, name = row.split(b"\t", 1)
        mode, kind, blob = metadata.decode().split()
        if kind != "blob" or mode not in ("100644", "100755"):
            raise ValueError("unsupported imported source entry: " + name.decode())
        tree[name.decode()] = (mode, blob)
    adaptations = manifest.get("adaptations", {})
    if not set(adaptations) <= set(tree):
        raise ValueError("adaptations must identify retained source paths")
    retirements = manifest.get("retirements", {})
    if not isinstance(retirements, dict) or not set(retirements) <= set(tree):
        raise ValueError("retirements must identify retained source paths")
    if set(retirements) & set(adaptations):
        raise ValueError("retired source paths cannot also be adapted")
    for name, record in retirements.items():
        if (not isinstance(record, dict)
                or set(record) != {"reason", "source_blob", "source_mode", "source_sha256"}
                or not isinstance(record["reason"], str) or not record["reason"].strip()
                or not isinstance(record["source_sha256"], str)
                or not re.fullmatch(r"[0-9a-f]{64}", record["source_sha256"])):
            raise ValueError("invalid retirement provenance: " + name)
        mode, blob = tree[name]
        if record["source_mode"] != mode or record["source_blob"] != blob:
            raise ValueError("retirement source identity differs: " + name)
        original = git(root, "cat-file", "blob", blob)
        if hashlib.sha256(original).hexdigest() != record["source_sha256"]:
            raise ValueError("retirement source digest differs: " + name)
    expected = (set(tree) - set(retirements)) | {"HARP_IMPORT.json"}
    current = {
        path.decode().removeprefix("corenius/")
        for path in git(root, "ls-files", "-z", "--", "corenius").split(b"\0")
        if path
    }
    if current != expected:
        raise ValueError("imported path set differs: " + repr(sorted(current ^ expected)))
    for name, (mode, blob) in tree.items():
        relative = Path(name)
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("unsafe imported path: " + name)
        path = component
        for part in relative.parts:
            path /= part
            if path.is_symlink():
                raise ValueError("symlinked imported path: " + name)
        if name in retirements:
            if path.exists():
                raise ValueError("retired source path is present: " + name)
            continue
        metadata = path.stat()
        if not stat.S_ISREG(metadata.st_mode):
            raise ValueError("imported path is not a regular file: " + name)
        actual_mode = "100755" if metadata.st_mode & stat.S_IXUSR else "100644"
        if actual_mode != mode:
            raise ValueError("imported executable mode changed: " + name)
        data = path.read_bytes()
        if name in adaptations:
            matches = hashlib.sha256(data).hexdigest() == adaptations[name]["sha256"]
        else:
            header = b"blob " + str(len(data)).encode() + b"\0"
            matches = hashlib.sha1(header + data).hexdigest() == blob
        if not matches:
            raise ValueError("imported bytes changed without provenance: " + name)
    tips = {entry["commit"] for entry in manifest["branches"]}
    tips.update(entry.get("snapshot_commit") or entry["head"] for entry in manifest["worktrees"])
    for tip in tips:
        if not re.fullmatch(r"[0-9a-f]{40}", tip):
            raise ValueError("Corenius source tips must be exact Git commits")
        subprocess.run(["git", "merge-base", "--is-ancestor", tip, source], cwd=root, check=True)
    anchors = ["HEAD"]
    pending = subprocess.run(["git", "rev-parse", "--verify", "MERGE_HEAD"],
                             cwd=root, capture_output=True, text=True)
    if pending.returncode == 0:
        anchors.append(pending.stdout.strip())
    if not any(subprocess.run(["git", "merge-base", "--is-ancestor", source, anchor],
                              cwd=root).returncode == 0 for anchor in anchors):
        raise ValueError("Corenius history is not included in this candidate")
    print(f"Corenius import: {len(tree)} source files, {len(retirements)} declared retirements, "
          f"{len(manifest['branches'])} branches, "
          f"{len(manifest['worktrees'])} worktrees; bytes, modes and ancestry verified")


if __name__ == "__main__":
    try:
        root = Path(git(Path.cwd(), "rev-parse", "--show-toplevel").decode().strip())
        verify(root)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print("Corenius import verification failed: " + str(error), file=sys.stderr)
        sys.exit(1)
