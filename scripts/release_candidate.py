#!/usr/bin/env python3
"""Build a verified native candidate; never tag, upload, or select a license."""

import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tarfile
import tempfile


class ReleaseError(Exception):
    pass


def capture(root, *args):
    return subprocess.check_output(args, cwd=root, text=True).strip()


def clean_commit(root, expected=None):
    if capture(root, "git", "status", "--porcelain", "--untracked-files=all"):
        raise ReleaseError("release requires a clean checkout, including untracked inputs")
    commit = capture(root, "git", "rev-parse", "HEAD")
    if expected is not None and commit != expected:
        raise ReleaseError("commit changed during release verification")
    return commit


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            result.update(block)
    return result.hexdigest()


def real_directory(path):
    if path.is_symlink() or path.resolve() != path.absolute():
        raise ReleaseError(f"symlinked directory is not allowed: {path}")
    if path.exists() and not path.is_dir():
        raise ReleaseError(f"expected directory: {path}")


def add_member(bundle, path, name, mode):
    member = bundle.gettarinfo(str(path), arcname=name)
    member.mode = mode
    member.uid = member.gid = 0
    member.uname = member.gname = ""
    with path.open("rb") as source:
        bundle.addfile(member, source)


def create_candidate(root):
    root = root.resolve()
    if Path(capture(root, "git", "rev-parse", "--show-toplevel")).resolve() != root:
        raise ReleaseError("release must run at the repository root")
    dist = root / "dist"
    real_directory(dist)
    commit = clean_commit(root)
    if subprocess.run(["git", "check-ignore", "-q", "dist/.candidate"], cwd=root).returncode:
        raise ReleaseError("dist/ must be ignored before creating release artifacts")

    # A candidate cannot bypass verification through a CLI option or environment flag.
    subprocess.run(["mise", "run", "verify"], cwd=root, check=True)
    clean_commit(root, commit)
    metadata = json.loads(capture(root, "cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"))
    version = next(package["version"] for package in metadata["packages"] if package["name"] == "harp")
    compiler = capture(root, "rustc", "-vV")
    target = next(line.removeprefix("host: ") for line in compiler.splitlines() if line.startswith("host: "))
    for value in (version, target):
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.+-]*", value):
            raise ReleaseError("invalid Cargo version or native target")
    if "windows" in target:
        raise ReleaseError("this candidate workflow supports native Unix hosts")
    name = f"harp-{version}-{target}-{commit[:12]}"
    destination = dist / name
    if destination.exists() or destination.is_symlink():
        raise ReleaseError(f"candidate already exists: {destination}")

    target_dir = Path(os.environ.get("HARP_TARGET_DIR", str(root / ".build/harp-target")))
    if not target_dir.is_absolute():
        target_dir = root / target_dir
    real_directory(target_dir)
    subprocess.run([
        "cargo", "build", "--locked", "--profile", "size", "-p", "harp",
        "--bin", "harp", "--target", target, "--target-dir", str(target_dir),
    ], cwd=root, check=True)
    binary = target_dir / target / "size/harp"
    if binary.is_symlink() or not binary.is_file() or binary.resolve() != binary.absolute():
        raise ReleaseError(f"expected a regular native executable: {binary}")
    clean_commit(root, commit)
    real_directory(dist)
    dist.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".candidate-", dir=dist) as temporary:
        staging = Path(temporary)
        payload = staging / "payload"
        payload.mkdir()
        notes = (
            f"# Harp {version} native release candidate\n\n"
            f"Commit: `{commit}`\n\nTarget: `{target}`\n\n"
            "Built with the locked Cargo dependency graph and size profile after "
            "the complete `mise run verify` gate. The extracted executable passed "
            "`harp --version` on the build host.\n\n"
            "Verify SHA256SUMS, extract the archive, and install `harp` into a "
            "directory on PATH. Repository commands require a matching content "
            "checkout; the archive contains only the executable and these notes. "
            "Live provider commands require separately installed/configured providers. "
            "See docs/releasing.md in the matching checkout.\n\n"
            "Compatibility is verified only on the recorded build host. No signing "
            "or notarization is supplied. Source-specific license records remain "
            "in force; this candidate does not establish redistribution permission "
            "or select a repository-wide license.\n"
        )
        notes_file = payload / "RELEASE_NOTES.md"
        notes_file.write_text(notes, encoding="utf-8")
        archive = payload / f"{name}.tar.gz"
        with tarfile.open(archive, "w:gz") as bundle:
            add_member(bundle, binary, "harp", 0o755)
            add_member(bundle, notes_file, "RELEASE_NOTES.md", 0o644)
        smoke = staging / "smoke"
        smoke.mkdir()
        # Extract only our two regular members; do not use unrestricted extractall.
        with tarfile.open(archive, "r:gz") as bundle:
            members = bundle.getmembers()
            if {member.name for member in members} != {"harp", "RELEASE_NOTES.md"} or len(members) != 2:
                raise ReleaseError("unexpected archive contents")
            for member in members:
                if not member.isfile():
                    raise ReleaseError("archive contains a non-regular member")
                with bundle.extractfile(member) as source:
                    (smoke / member.name).write_bytes(source.read())
                (smoke / member.name).chmod(member.mode)
        extracted = smoke / "harp"
        observed = subprocess.check_output([str(extracted), "--version"], cwd=smoke, text=True, timeout=30).strip()
        if observed != f"harp {version}":
            raise ReleaseError(f"extracted binary version mismatch: {observed!r}")
        binary_digest = digest(extracted)
        if binary_digest != digest(binary):
            raise ReleaseError("binary changed during packaging")
        manifest = {
            "schema_version": 1,
            "version": version,
            "commit": commit,
            "target": target,
            "profile": "size",
            "rustc": compiler,
            "build_host": {"system": platform.system(), "release": platform.release(), "machine": platform.machine()},
            "verification": {"command": "mise run verify", "result": "passed", "smoke": observed},
            "binary": {"name": "harp", "sha256": binary_digest, "size": extracted.stat().st_size},
            "archive": {"name": archive.name, "sha256": digest(archive), "size": archive.stat().st_size},
        }
        (payload / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        checksums = "".join(f"{digest(item)}  {item.name}\n" for item in sorted(payload.iterdir()))
        (payload / "SHA256SUMS").write_text(checksums, encoding="utf-8")
        clean_commit(root, commit)
        if destination.exists() or destination.is_symlink():
            raise ReleaseError(f"candidate already exists: {destination}")
        payload.rename(destination)
    return destination


if __name__ == "__main__":
    try:
        if len(sys.argv) != 1:
            raise ReleaseError("usage: mise run release-candidate (no gate-bypass options)")
        print(create_candidate(Path(__file__).resolve().parents[1]))
    except (ReleaseError, subprocess.SubprocessError, OSError, ValueError, StopIteration) as error:
        print(f"release candidate failed: {error}", file=sys.stderr)
        sys.exit(1)
