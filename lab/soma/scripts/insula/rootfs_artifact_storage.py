#!/usr/bin/env python3
"""Rootfs artifact packaging and Waystone storage-request helpers.

Soma owns selected-rootfs identity, local archive manifests, and post-restore
validation requirements. Waystone owns effectful storage traffic. It owns HDFS
today; TOS traffic should be added there instead of in Soma.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Mapping, Sequence

try:
    from scripts.insula import soma_insula
except ModuleNotFoundError:
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))
    from scripts.insula import soma_insula


DEFAULT_STORAGE_PROVIDER = "tos"
DEFAULT_BUCKET_NAME = "example-artifacts"
DEFAULT_SITE = "example-region"
DEFAULT_PROFILE_HINT = "soma-insula-artifacts"
DEFAULT_PREFIX = "soma/insula-rootfs/"
DEFAULT_WAYSTONE_REPO_HINT = "external-operator-storage"
DEFAULT_HDFS_TICKET_DIR = pathlib.Path(".scratch") / "soma-hdfs-artifact-tickets"
DEFAULT_STORE_ROOT = pathlib.Path(".scratch") / "emacs-gym" / "insula" / "rootfs-store"
DEFAULT_ARTIFACT_ROOT = pathlib.Path(".scratch") / "emacs-gym" / "insula" / "artifacts"


class RootfsArtifactStorageError(RuntimeError):
    """A rootfs artifact packaging or storage-request error."""


@dataclass(frozen=True)
class ArtifactPaths:
    artifact_dir: pathlib.Path
    archive_path: pathlib.Path
    manifest_path: pathlib.Path
    request_path: pathlib.Path


@dataclass(frozen=True)
class StorageTarget:
    provider: str
    bucket_name: str
    prefix: str
    selected_rootfs_store_id: str
    site: str
    profile_hint: str
    waystone_repo_hint: str

    @property
    def archive_object_key(self) -> str:
        return f"{self.prefix}{self.selected_rootfs_store_id}.tar.zst"

    @property
    def manifest_object_key(self) -> str:
        return f"{self.prefix}{self.selected_rootfs_store_id}.manifest.json"

    @property
    def hdfs_storage_child(self) -> str:
        return f"{self.prefix}{self.selected_rootfs_store_id}".lstrip("/")


def repo_root() -> pathlib.Path:
    return pathlib.Path(__file__).resolve().parents[2]


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def selected_rootfs(repo_root_path: pathlib.Path) -> soma_insula.SelectedRootfs:
    return soma_insula.resolve_selected_rootfs(repo_root_path / DEFAULT_STORE_ROOT)


def normalize_prefix(prefix: str) -> str:
    return prefix if not prefix or prefix.endswith("/") else f"{prefix}/"


def artifact_paths(
    repo_root_path: pathlib.Path,
    store_id: str,
    *,
    out_dir: pathlib.Path | None = None,
) -> ArtifactPaths:
    base = out_dir if out_dir is not None else repo_root_path / DEFAULT_ARTIFACT_ROOT / store_id
    base = base if base.is_absolute() else repo_root_path / base
    return ArtifactPaths(
        artifact_dir=base,
        archive_path=base / f"{store_id}.tar.zst",
        manifest_path=base / f"{store_id}.manifest.json",
        request_path=base / f"{store_id}.waystone-storage-request.json",
    )


def build_storage_target(
    *,
    repo_root: pathlib.Path,
    provider: str = DEFAULT_STORAGE_PROVIDER,
    bucket_name: str = DEFAULT_BUCKET_NAME,
    prefix: str = DEFAULT_PREFIX,
    site: str = DEFAULT_SITE,
    profile_hint: str = DEFAULT_PROFILE_HINT,
    waystone_repo_hint: str = DEFAULT_WAYSTONE_REPO_HINT,
    environ: Mapping[str, str] | None = None,
) -> StorageTarget:
    env = environ if environ is not None else {}
    selected = selected_rootfs(repo_root)
    return StorageTarget(
        provider=env.get("SOMA_ARTIFACT_STORAGE_PROVIDER", provider),
        bucket_name=env.get("SOMA_TOS_ARTIFACT_BUCKET_NAME", bucket_name),
        prefix=normalize_prefix(env.get("SOMA_TOS_ARTIFACT_PREFIX", prefix)),
        selected_rootfs_store_id=selected.store_id,
        site=env.get("SOMA_TOS_SITE", site),
        profile_hint=env.get("SOMA_TOS_PROFILE_HINT", profile_hint),
        waystone_repo_hint=env.get("SOMA_WAYSTONE_REPO_HINT", waystone_repo_hint),
    )


def storage_descriptor(target: StorageTarget) -> dict[str, Any]:
    descriptor: dict[str, Any] = {
        "provider": target.provider,
        "owner": "waystone",
        "waystone_repo_hint": target.waystone_repo_hint,
        "credentials_included": False,
        "credential_policy": "Waystone resolves external-storage credentials from its own local configuration; Soma request files must not carry AK, SK, tokens, bucket ids, or transport credential material.",
    }
    if target.provider == "tos":
        descriptor["tos"] = {
            "bucket_name": target.bucket_name,
            "site": target.site,
            "profile_hint": target.profile_hint,
            "object_prefix": target.prefix,
            "archive_object_key": target.archive_object_key,
            "manifest_object_key": target.manifest_object_key,
        }
    if target.provider == "hdfs":
        descriptor["hdfs"] = {
            "waystone_storage_child": target.hdfs_storage_child,
            "archive_name": f"{target.selected_rootfs_store_id}.tar.zst",
            "manifest_name": f"{target.selected_rootfs_store_id}.manifest.json",
            "request_name": f"{target.selected_rootfs_store_id}.waystone-storage-request.json",
            "remote_path_rule": "Resolve with: scripts/waystone storage-prefix --child <waystone_storage_child> --json",
        }
    return descriptor


def validation_descriptor(store_id: str) -> dict[str, Any]:
    rootfs = DEFAULT_STORE_ROOT / "content" / store_id
    return {
        "required_after_restore": True,
        "selected_rootfs_content_path": str(rootfs),
        "command": [
            "scripts/insula/validate-emacs-rootfs-contract.py",
            str(rootfs),
            "--verify-rootfs-tree",
        ],
        "selection_rule": "Update .scratch/emacs-gym/insula/rootfs-store/selected.json only after validation succeeds.",
    }


def artifact_manifest(
    target: StorageTarget,
    *,
    repo_root: pathlib.Path,
    archive_path: pathlib.Path | None = None,
) -> dict[str, Any]:
    selected = selected_rootfs(repo_root)
    if target.selected_rootfs_store_id != selected.store_id:
        raise RootfsArtifactStorageError(
            "storage target selected rootfs store id does not match current selection: "
            f"{target.selected_rootfs_store_id} != {selected.store_id}"
        )
    contract = selected.contract
    archive: dict[str, Any] = {
        "object_key": target.archive_object_key,
        "format": "tar.zst",
    }
    if archive_path is not None and archive_path.exists():
        archive.update(
            {
                "host_path": str(archive_path),
                "size_bytes": archive_path.stat().st_size,
                "sha256": sha256_file(archive_path),
            }
        )
    return {
        "schema_version": 1,
        "kind": "soma_insula_rootfs_artifact_manifest",
        "generated_at": utc_now(),
        "store_id": selected.store_id,
        "rootfs_contract": {
            "host_path": str(selected.contract_path),
            "sha256": sha256_file(selected.contract_path),
        },
        "rootfs_tree_manifest": contract.get("rootfs_tree_manifest", {}),
        "base_image": contract.get("base_image", ""),
        "image_tag": contract.get("image_tag", ""),
        "image_id": contract.get("image_id", ""),
        "rootfs_recipe_sha256": contract.get("rootfs_recipe_sha256", ""),
        "storage": storage_descriptor(target),
        "archive": archive,
        "restore_validation": validation_descriptor(selected.store_id),
    }


def write_artifact_manifest(
    target: StorageTarget,
    *,
    repo_root: pathlib.Path,
    manifest_path: pathlib.Path,
    archive_path: pathlib.Path | None = None,
) -> dict[str, Any]:
    manifest = artifact_manifest(target, repo_root=repo_root, archive_path=archive_path)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest


def waystone_storage_request(
    target: StorageTarget,
    *,
    repo_root: pathlib.Path,
    operation: str,
    archive_path: pathlib.Path,
    manifest_path: pathlib.Path,
) -> dict[str, Any]:
    if operation not in {"upload", "download"}:
        raise RootfsArtifactStorageError(f"unsupported storage request operation: {operation}")
    selected = selected_rootfs(repo_root)
    local_artifacts = {
        "dir": str(archive_path.parent),
        "archive": str(archive_path),
        "manifest": str(manifest_path),
    }
    if archive_path.exists():
        local_artifacts["archive_sha256"] = sha256_file(archive_path)
        local_artifacts["archive_size_bytes"] = archive_path.stat().st_size
    if manifest_path.exists():
        local_artifacts["manifest_sha256"] = sha256_file(manifest_path)

    return {
        "schema_version": 1,
        "kind": "soma_insula_waystone_storage_request",
        "generated_at": utc_now(),
        "operation": operation,
        "source_system": "soma",
        "effectful_storage_owner": "waystone",
        "soma_repo": str(repo_root),
        "store_id": selected.store_id,
        "local_artifacts": local_artifacts,
        "storage": storage_descriptor(target),
        "traffic_policy": {
            "soma_performs_storage_transport": False,
            "waystone_handles_tos_or_hdfs_traffic": True,
        },
        "restore_validation": validation_descriptor(selected.store_id),
    }


def write_waystone_storage_request(
    target: StorageTarget,
    *,
    repo_root: pathlib.Path,
    request_path: pathlib.Path,
    operation: str,
    archive_path: pathlib.Path,
    manifest_path: pathlib.Path,
) -> dict[str, Any]:
    request = waystone_storage_request(
        target,
        repo_root=repo_root,
        operation=operation,
        archive_path=archive_path,
        manifest_path=manifest_path,
    )
    request_path.parent.mkdir(parents=True, exist_ok=True)
    request_path.write_text(json.dumps(request, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return request


def plan(target: StorageTarget, *, repo_root: pathlib.Path, operation: str) -> dict[str, Any]:
    paths = artifact_paths(repo_root, target.selected_rootfs_store_id)
    return waystone_storage_request(
        target,
        repo_root=repo_root,
        operation=operation,
        archive_path=paths.archive_path,
        manifest_path=paths.manifest_path,
    )


def pack_rootfs(
    target: StorageTarget,
    *,
    repo_root: pathlib.Path,
    out_dir: pathlib.Path | None = None,
) -> ArtifactPaths:
    selected = selected_rootfs(repo_root)
    if target.selected_rootfs_store_id != selected.store_id:
        raise RootfsArtifactStorageError(
            "storage target selected rootfs store id does not match current selection: "
            f"{target.selected_rootfs_store_id} != {selected.store_id}"
        )
    paths = artifact_paths(repo_root, selected.store_id, out_dir=out_dir)
    paths.artifact_dir.mkdir(parents=True, exist_ok=True)
    tar = shutil.which("tar") or "tar"
    zstd = shutil.which("zstd") or "zstd"
    command = [
        tar,
        "--numeric-owner",
        "--xattrs",
        "--acls",
        "-C",
        str(selected.rootfs.parent),
        "-I",
        zstd,
        "-cf",
        str(paths.archive_path),
        selected.rootfs.name,
    ]
    subprocess.run(command, check=True)
    write_artifact_manifest(
        target,
        repo_root=repo_root,
        manifest_path=paths.manifest_path,
        archive_path=paths.archive_path,
    )
    write_waystone_storage_request(
        target,
        repo_root=repo_root,
        request_path=paths.request_path,
        operation="upload",
        archive_path=paths.archive_path,
        manifest_path=paths.manifest_path,
    )
    return paths


def print_json(value: dict[str, Any]) -> None:
    print(json.dumps(value, indent=2, sort_keys=True))


def command_output_text(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode(errors="replace")
    return value


def command_result(
    argv: Sequence[str],
    *,
    cwd: pathlib.Path,
    timeout_secs: int,
) -> dict[str, Any]:
    command = [str(item) for item in argv]
    try:
        proc = subprocess.run(
            command,
            cwd=str(cwd),
            text=True,
            capture_output=True,
            check=False,
            timeout=timeout_secs,
        )
        return {
            "command": command,
            "cwd": str(cwd),
            "returncode": proc.returncode,
            "stdout": proc.stdout,
            "stderr": proc.stderr,
            "timed_out": False,
        }
    except subprocess.TimeoutExpired as exc:
        return {
            "command": command,
            "cwd": str(cwd),
            "returncode": None,
            "stdout": command_output_text(exc.stdout),
            "stderr": command_output_text(exc.stderr),
            "timed_out": True,
            "timeout_secs": timeout_secs,
        }


def local_artifact_status(paths: ArtifactPaths, *, hash_local: bool) -> dict[str, Any]:
    result: dict[str, Any] = {"dir": str(paths.artifact_dir), "files": {}}
    for label, path in (
        ("archive", paths.archive_path),
        ("manifest", paths.manifest_path),
        ("request", paths.request_path),
    ):
        file_info: dict[str, Any] = {
            "path": str(path),
            "exists": path.is_file(),
        }
        if path.is_file():
            file_info["size_bytes"] = path.stat().st_size
            if hash_local:
                file_info["sha256"] = sha256_file(path)
        result["files"][label] = file_info
    return result


def parse_json_output(result: dict[str, Any]) -> dict[str, Any] | None:
    if result.get("returncode") != 0:
        return None
    try:
        return json.loads(str(result.get("stdout") or "{}"))
    except json.JSONDecodeError:
        return None


def add_target_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--repo-root", type=pathlib.Path, default=repo_root())
    parser.add_argument("--provider", default=DEFAULT_STORAGE_PROVIDER)
    parser.add_argument("--bucket-name", default=DEFAULT_BUCKET_NAME)
    parser.add_argument("--prefix", default=DEFAULT_PREFIX)
    parser.add_argument("--site", default=DEFAULT_SITE)
    parser.add_argument("--profile-hint", default=DEFAULT_PROFILE_HINT)
    parser.add_argument("--waystone-repo-hint", default=DEFAULT_WAYSTONE_REPO_HINT)


def load_target(args: argparse.Namespace, *, root: pathlib.Path) -> StorageTarget:
    return build_storage_target(
        repo_root=root,
        provider=args.provider,
        bucket_name=args.bucket_name,
        prefix=args.prefix,
        site=args.site,
        profile_hint=args.profile_hint,
        waystone_repo_hint=args.waystone_repo_hint,
        environ={},
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    plan_parser = subparsers.add_parser("plan", help="print sanitized Waystone storage request")
    add_target_args(plan_parser)
    plan_parser.add_argument("--operation", choices=("upload", "download"), default="upload")

    pack_parser = subparsers.add_parser(
        "pack",
        help="archive selected rootfs, write artifact manifest, and write upload request",
    )
    add_target_args(pack_parser)
    pack_parser.add_argument("--out-dir", type=pathlib.Path)

    request_parser = subparsers.add_parser("request", help="write a Waystone storage request")
    add_target_args(request_parser)
    request_parser.add_argument("--operation", choices=("upload", "download"), required=True)
    request_parser.add_argument("--out-dir", type=pathlib.Path)
    request_parser.add_argument("--output", type=pathlib.Path)

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        root = args.repo_root.resolve()
        target = load_target(args, root=root)
        if args.command == "plan":
            print_json(plan(target, repo_root=root, operation=args.operation))
            return 0

        if args.command == "pack":
            packed = pack_rootfs(target, repo_root=root, out_dir=args.out_dir)
            print_json(
                {
                    "ok": True,
                    "archive": str(packed.archive_path),
                    "manifest": str(packed.manifest_path),
                    "waystone_storage_request": str(packed.request_path),
                }
            )
            return 0

        if args.command == "request":
            paths = artifact_paths(root, target.selected_rootfs_store_id, out_dir=args.out_dir)
            request_path = args.output or paths.request_path
            write_waystone_storage_request(
                target,
                repo_root=root,
                request_path=request_path,
                operation=args.operation,
                archive_path=paths.archive_path,
                manifest_path=paths.manifest_path,
            )
            print_json({"ok": True, "waystone_storage_request": str(request_path)})
            return 0

        raise AssertionError(f"unhandled command: {args.command}")
    except (
        OSError,
        subprocess.CalledProcessError,
        soma_insula.InsulaError,
        RootfsArtifactStorageError,
    ) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
