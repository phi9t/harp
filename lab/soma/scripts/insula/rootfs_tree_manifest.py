#!/usr/bin/env python3
"""Write and verify Soma rootfs tree manifests.

The manifest records rootfs-owned file identity separately from the rootfs
contract so the contract can point at the manifest without creating a recursive
hash. The tree hash intentionally excludes the contract and manifest files.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shutil
import stat
import sys
import tempfile
from datetime import datetime, timezone
from typing import Any, Iterable, TextIO


SCHEMA_VERSION = 1
MANIFEST_KIND = "soma_rootfs_tree_manifest_v1"
SUMMARY_KIND = "soma_rootfs_tree_manifest_summary"
DEFAULT_MANIFEST_PATH = "/etc/soma-insula-rootfs-tree-manifest.jsonl"
DEFAULT_SUMMARY_PATH = "/etc/soma-insula-rootfs-tree-summary.json"
DEFAULT_CONTRACT_PATH = "/etc/soma-insula-rootfs-contract.json"
DEFAULT_EXCLUDED_PATHS = (
    DEFAULT_CONTRACT_PATH,
    DEFAULT_MANIFEST_PATH,
    DEFAULT_SUMMARY_PATH,
)


def sandbox_path(rootfs: pathlib.Path, absolute_path: str) -> pathlib.Path:
    if not absolute_path.startswith("/"):
        raise ValueError(f"sandbox path must be absolute: {absolute_path}")
    return rootfs / absolute_path.lstrip("/")


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalize_rootfs_path(path: pathlib.Path, rootfs: pathlib.Path) -> str:
    rel = path.relative_to(rootfs)
    if str(rel) == ".":
        return "/"
    return "/" + rel.as_posix()


def file_type(mode: int) -> str:
    if stat.S_ISDIR(mode):
        return "dir"
    if stat.S_ISREG(mode):
        return "file"
    if stat.S_ISLNK(mode):
        return "symlink"
    if stat.S_ISCHR(mode):
        return "char"
    if stat.S_ISBLK(mode):
        return "block"
    if stat.S_ISFIFO(mode):
        return "fifo"
    if stat.S_ISSOCK(mode):
        return "socket"
    return "unknown"


def iter_paths(rootfs: pathlib.Path, excluded_paths: set[str]) -> Iterable[pathlib.Path]:
    yield rootfs
    for current, dirnames, filenames in os.walk(rootfs, topdown=True, followlinks=False):
        current_path = pathlib.Path(current)
        kept_dirs = []
        for dirname in sorted(dirnames):
            path = current_path / dirname
            rootfs_path = normalize_rootfs_path(path, rootfs)
            if rootfs_path in excluded_paths:
                continue
            kept_dirs.append(dirname)
            yield path
        dirnames[:] = kept_dirs
        for filename in sorted(filenames):
            path = current_path / filename
            rootfs_path = normalize_rootfs_path(path, rootfs)
            if rootfs_path in excluded_paths:
                continue
            yield path


def entry_for(path: pathlib.Path, rootfs: pathlib.Path) -> dict[str, Any]:
    st = os.lstat(path)
    mode = stat.S_IFMT(st.st_mode) | stat.S_IMODE(st.st_mode)
    entry: dict[str, Any] = {
        "path": normalize_rootfs_path(path, rootfs),
        "type": file_type(st.st_mode),
        "mode": format(mode, "o"),
        "uid": st.st_uid,
        "gid": st.st_gid,
    }
    if stat.S_ISREG(st.st_mode):
        entry["size"] = st.st_size
        entry["sha256"] = sha256_file(path)
    elif stat.S_ISLNK(st.st_mode):
        entry["target"] = os.readlink(path)
    elif stat.S_ISCHR(st.st_mode) or stat.S_ISBLK(st.st_mode):
        entry["rdev"] = st.st_rdev
    return entry


def canonical_line(entry: dict[str, Any]) -> bytes:
    return (json.dumps(entry, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")


def compute_manifest(
    rootfs: pathlib.Path,
    *,
    excluded_paths: Iterable[str] = DEFAULT_EXCLUDED_PATHS,
    out: TextIO | None = None,
) -> dict[str, Any]:
    rootfs = rootfs.resolve()
    excluded = set(excluded_paths)
    tree_digest = hashlib.sha256()
    entry_count = 0
    file_count = 0
    symlink_count = 0
    dir_count = 0
    special_count = 0
    total_file_bytes = 0

    for path in iter_paths(rootfs, excluded):
        entry = entry_for(path, rootfs)
        line = canonical_line(entry)
        tree_digest.update(line)
        if out is not None:
            out.buffer.write(line)
        entry_count += 1
        if entry["type"] == "file":
            file_count += 1
            total_file_bytes += int(entry.get("size", 0))
        elif entry["type"] == "symlink":
            symlink_count += 1
        elif entry["type"] == "dir":
            dir_count += 1
        else:
            special_count += 1

    return {
        "schema_version": SCHEMA_VERSION,
        "kind": SUMMARY_KIND,
        "manifest_kind": MANIFEST_KIND,
        "algorithm": "sha256(canonical-json-lines(sorted-rootfs-entries))",
        "excluded_paths": sorted(excluded),
        "entry_count": entry_count,
        "file_count": file_count,
        "directory_count": dir_count,
        "symlink_count": symlink_count,
        "special_count": special_count,
        "total_file_bytes": total_file_bytes,
        "tree_hash_sha256": tree_digest.hexdigest(),
    }


def write_manifest(rootfs: pathlib.Path) -> dict[str, Any]:
    rootfs = rootfs.resolve()
    manifest_path = sandbox_path(rootfs, DEFAULT_MANIFEST_PATH)
    summary_path = sandbox_path(rootfs, DEFAULT_SUMMARY_PATH)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as fh:
        tmp_manifest = pathlib.Path(fh.name)
        summary = compute_manifest(rootfs, out=fh)
    shutil.copyfile(tmp_manifest, manifest_path)
    tmp_manifest.unlink(missing_ok=True)

    summary.update(
        {
            "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "manifest_path": DEFAULT_MANIFEST_PATH,
            "summary_path": DEFAULT_SUMMARY_PATH,
            "manifest_sha256": sha256_file(manifest_path),
        }
    )
    tmp_summary = summary_path.with_name(summary_path.name + ".tmp")
    tmp_summary.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp_summary, summary_path)
    return summary


def contract_manifest_record(summary: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "kind": MANIFEST_KIND,
        "manifest_path": DEFAULT_MANIFEST_PATH,
        "summary_path": DEFAULT_SUMMARY_PATH,
        "algorithm": summary["algorithm"],
        "excluded_paths": summary["excluded_paths"],
        "entry_count": summary["entry_count"],
        "file_count": summary["file_count"],
        "directory_count": summary["directory_count"],
        "symlink_count": summary["symlink_count"],
        "special_count": summary["special_count"],
        "total_file_bytes": summary["total_file_bytes"],
        "tree_hash_sha256": summary["tree_hash_sha256"],
        "manifest_sha256": summary["manifest_sha256"],
    }


def update_contract(rootfs: pathlib.Path, summary: dict[str, Any]) -> None:
    contract_path = sandbox_path(rootfs.resolve(), DEFAULT_CONTRACT_PATH)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    contract["rootfs_tree_manifest"] = contract_manifest_record(summary)
    contract_path.write_text(json.dumps(contract, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def verify_recorded_manifest(rootfs: pathlib.Path, *, verify_tree: bool = False) -> list[str]:
    errors: list[str] = []
    rootfs = rootfs.resolve()
    contract_path = sandbox_path(rootfs, DEFAULT_CONTRACT_PATH)
    if not contract_path.is_file():
        return [f"missing rootfs contract: {contract_path}"]
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    record = contract.get("rootfs_tree_manifest")
    if not isinstance(record, dict):
        return ["contract lacks rootfs_tree_manifest"]

    manifest_path_value = str(record.get("manifest_path") or "")
    summary_path_value = str(record.get("summary_path") or "")
    if manifest_path_value != DEFAULT_MANIFEST_PATH:
        errors.append(f"rootfs_tree_manifest manifest_path is not {DEFAULT_MANIFEST_PATH}: {manifest_path_value}")
    if summary_path_value != DEFAULT_SUMMARY_PATH:
        errors.append(f"rootfs_tree_manifest summary_path is not {DEFAULT_SUMMARY_PATH}: {summary_path_value}")

    manifest_path = sandbox_path(rootfs, manifest_path_value or DEFAULT_MANIFEST_PATH)
    summary_path = sandbox_path(rootfs, summary_path_value or DEFAULT_SUMMARY_PATH)
    if not manifest_path.is_file():
        errors.append(f"missing rootfs tree manifest: {manifest_path}")
    if not summary_path.is_file():
        errors.append(f"missing rootfs tree summary: {summary_path}")
    if errors:
        return errors

    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    manifest_sha = sha256_file(manifest_path)
    if manifest_sha != record.get("manifest_sha256"):
        errors.append("rootfs tree manifest SHA256 does not match contract")
    if manifest_sha != summary.get("manifest_sha256"):
        errors.append("rootfs tree manifest SHA256 does not match summary")

    fields = (
        "schema_version",
        "algorithm",
        "excluded_paths",
        "entry_count",
        "file_count",
        "directory_count",
        "symlink_count",
        "special_count",
        "total_file_bytes",
        "tree_hash_sha256",
    )
    for field in fields:
        if summary.get(field) != record.get(field):
            errors.append(
                f"rootfs_tree_manifest {field} mismatch: contract={record.get(field)!r} summary={summary.get(field)!r}"
            )

    if record.get("kind") != MANIFEST_KIND:
        errors.append(f"rootfs_tree_manifest kind is not {MANIFEST_KIND}")
    if summary.get("kind") != SUMMARY_KIND:
        errors.append(f"rootfs tree summary kind is not {SUMMARY_KIND}")
    if summary.get("manifest_kind") != MANIFEST_KIND:
        errors.append(f"rootfs tree summary manifest_kind is not {MANIFEST_KIND}")

    if verify_tree:
        recomputed = compute_manifest(rootfs)
        for field in fields:
            if recomputed.get(field) != record.get(field):
                errors.append(
                    f"rootfs tree recompute {field} mismatch: contract={record.get(field)!r} recomputed={recomputed.get(field)!r}"
                )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    write_parser = subparsers.add_parser("write", help="write manifest and summary files")
    write_parser.add_argument("rootfs", type=pathlib.Path)
    write_parser.add_argument("--update-contract", action="store_true")

    verify_parser = subparsers.add_parser("verify", help="verify recorded manifest files")
    verify_parser.add_argument("rootfs", type=pathlib.Path)
    verify_parser.add_argument("--verify-tree", action="store_true")

    args = parser.parse_args()
    if args.command == "write":
        summary = write_manifest(args.rootfs)
        if args.update_contract:
            update_contract(args.rootfs, summary)
        print(json.dumps(summary, indent=2, sort_keys=True))
        return 0

    errors = verify_recorded_manifest(args.rootfs, verify_tree=args.verify_tree)
    result = {"ok": not errors, "errors": errors}
    print(json.dumps(result, indent=2, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
