#!/usr/bin/env python3
"""Validate the selected Soma Emacs rootfs contract.

This is a host-side coordinator check. It does not execute rootfs tools
directly; it verifies that the selected rootfs contains the source-built Emacs
runtime and the ABI evidence that bwrap runners will later consume.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import struct
import sys
from typing import Any

from rootfs_tree_manifest import verify_recorded_manifest


REQUIRED_TOOLS = (
    "emacs",
    "emacsclient",
    "python3",
    "jupyter",
    "pylsp",
    "pyright",
    "pyright-langserver",
    "clang",
    "clangd",
    "rust-analyzer",
    "gcc",
    "g++",
    "rustc",
    "cargo",
    "git",
    "sh",
    "bash",
    "tmux",
)

REQUIRED_EMACS_SOURCE_FILES = (
    "src/eval.c",
    "src/lread.c",
    "src/keyboard.c",
    "src/buffer.h",
    "src/insdel.c",
    "src/xdisp.c",
)

EXPECTED_EMACS_SOURCE_SHA256 = {
    "30.2": "b3f36f18a6dd2715713370166257de2fae01f9d38cfe878ced9b1e6ded5befd9",
}


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def require(errors: list[str], condition: bool, message: str) -> None:
    if not condition:
        fail(errors, message)


def sandbox_path(rootfs: pathlib.Path, absolute_path: str) -> pathlib.Path:
    if not absolute_path.startswith("/"):
        raise ValueError(f"sandbox path must be absolute: {absolute_path}")
    return rootfs / absolute_path.lstrip("/")


def resolve_sandbox_path(rootfs: pathlib.Path, absolute_path: str) -> pathlib.Path:
    """Resolve symlinks as if absolute links are rooted inside rootfs."""

    rootfs = rootfs.resolve()
    current = rootfs
    parts = [part for part in pathlib.PurePosixPath(absolute_path).parts if part != "/"]
    seen = 0
    while parts:
        current = current / parts.pop(0)
        while current.is_symlink():
            seen += 1
            if seen > 64:
                raise RuntimeError(f"too many symlinks resolving {absolute_path}")
            target = os.readlink(current)
            if os.path.isabs(target):
                current = rootfs / target.lstrip("/")
            else:
                current = current.parent / target
            current = pathlib.Path(os.path.normpath(str(current)))
            try:
                current.relative_to(rootfs)
            except ValueError as exc:
                raise RuntimeError(
                    f"sandbox path escapes rootfs while resolving {absolute_path}: {current}"
                ) from exc
    return current


def extract_interpreter(emacs_binary: pathlib.Path) -> str:
    data = emacs_binary.read_bytes()
    if len(data) < 64 or data[:4] != b"\x7fELF":
        return ""
    elf_class = data[4]
    endian = "<" if data[5] == 1 else ">" if data[5] == 2 else ""
    if elf_class != 2 or not endian:
        return ""
    e_phoff = struct.unpack_from(endian + "Q", data, 32)[0]
    e_phentsize = struct.unpack_from(endian + "H", data, 54)[0]
    e_phnum = struct.unpack_from(endian + "H", data, 56)[0]
    for index in range(e_phnum):
        off = e_phoff + index * e_phentsize
        if off + 56 > len(data):
            return ""
        p_type = struct.unpack_from(endian + "I", data, off)[0]
        if p_type != 3:  # PT_INTERP
            continue
        p_offset = struct.unpack_from(endian + "Q", data, off + 8)[0]
        p_filesz = struct.unpack_from(endian + "Q", data, off + 32)[0]
        raw = data[p_offset : p_offset + p_filesz].split(b"\0", 1)[0]
        return raw.decode("utf-8", errors="replace")
    return ""


def extract_shebang(path: pathlib.Path) -> str:
    try:
        with path.open("rb") as fh:
            first = fh.readline(4096)
    except OSError:
        return ""
    if not first.startswith(b"#!"):
        return ""
    text = first[2:].decode("utf-8", errors="replace").strip()
    if not text:
        return ""
    return text.split()[0]


def ldd_paths(lines: list[str]) -> list[str]:
    paths: list[str] = []
    for line in lines:
        if "not found" in line:
            paths.append("not found")
            continue
        for match in re.finditer(r"(?:=>\s+)?(/[^ \t()]+)", line):
            paths.append(match.group(1))
    return paths


def string_values(value: Any, *, skip_key: str | None = None) -> list[str]:
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        values: list[str] = []
        for item in value:
            values.extend(string_values(item, skip_key=skip_key))
        return values
    if isinstance(value, dict):
        values = []
        for key, item in value.items():
            if key == skip_key:
                continue
            values.extend(string_values(item, skip_key=skip_key))
        return values
    return []


def validate(rootfs: pathlib.Path, *, verify_rootfs_tree: bool = False) -> tuple[dict[str, Any], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    original_rootfs = rootfs
    require(errors, not original_rootfs.is_symlink(), f"rootfs must not be a symlink: {original_rootfs}")
    rootfs = rootfs.resolve()

    require(errors, rootfs.is_dir(), f"rootfs is not a directory: {rootfs}")

    contract_path = rootfs / "etc" / "soma-insula-rootfs-contract.json"
    require(errors, contract_path.is_file(), f"missing rootfs contract: {contract_path}")
    if errors:
        return {"ok": False, "errors": errors, "warnings": warnings}, errors

    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    forbidden_prefixes = tuple(contract.get("forbidden_abi_prefixes") or ["/nix", "/data02/home"])
    for value in string_values(contract, skip_key="forbidden_abi_prefixes"):
        for prefix in forbidden_prefixes:
            if prefix and value.startswith(prefix):
                fail(errors, f"contract value uses forbidden ABI prefix: {value}")

    require(errors, contract.get("kind") == "soma_emacs_rootfs_contract", "wrong contract kind")
    require(errors, contract.get("dependency_closure") == "rootfs_owned", "dependency closure is not rootfs_owned")
    require(
        errors,
        contract.get("abi_guarantee") == "container-rootfs-abi-manifest",
        "ABI guarantee is not container-rootfs-abi-manifest",
    )
    require(errors, contract.get("abi_base") == "container-image-rootfs", "ABI base is not container-image-rootfs")
    require(errors, contract.get("glibc_owner") == "container-image-rootfs", "glibc owner is not container-image-rootfs")
    require(errors, contract.get("nix_allowed") is False, "contract must forbid Nix")
    require(errors, contract.get("materialization_kind") == "source-built-emacs-v8", "wrong materialization kind")
    require(errors, contract.get("base_image_digest_required") is True, "base image digest requirement is not recorded")
    for error in verify_recorded_manifest(rootfs, verify_tree=verify_rootfs_tree):
        fail(errors, error)
    materialization = contract.get("materialization") or {}
    require(
        errors,
        materialization.get("kind") == "docker-cuda-source-build",
        "materialization kind is not docker-cuda-source-build",
    )
    base_image = str(contract.get("base_image") or "")
    require(errors, "@sha256:" in base_image, "base image is not digest-pinned")
    require(errors, "cuda" in base_image.lower(), "base image does not look CUDA-enabled")

    emacs = contract.get("emacs") or {}
    version = str(emacs.get("version") or "")
    executable = str(emacs.get("executable") or "")
    prefix = str(emacs.get("prefix") or "")
    source_url = str(emacs.get("source_url") or "")
    source_sha = str(emacs.get("source_sha256") or "")
    expected_source_sha = str(emacs.get("expected_source_sha256") or "")
    executable_sha = str(emacs.get("executable_sha256") or "")

    require(errors, version != "", "missing Emacs version")
    require(errors, emacs.get("build_kind") == "gnu-source-tarball", "Emacs build kind is not GNU source tarball")
    require(errors, emacs.get("build_location") == "inside-rootfs", "Emacs build location is not inside-rootfs")
    require(errors, emacs.get("patch_variant") == "stock", "Emacs patch variant is not stock")
    require(errors, emacs.get("patch_set") == [], "Emacs patch set must be empty for stock baseline")
    require(errors, prefix == "/opt/soma/emacs", "Emacs prefix is not /opt/soma/emacs")
    require(errors, executable == "/opt/soma/emacs/bin/emacs", "Emacs executable is not /opt/soma/emacs/bin/emacs")
    require(errors, source_url == f"https://ftp.gnu.org/gnu/emacs/emacs-{version}.tar.xz", "Emacs source URL is not the pinned GNU tarball")
    require(errors, re.fullmatch(r"[0-9a-f]{64}", source_sha or "") is not None, "Emacs source SHA256 is missing or malformed")
    known_source_sha = EXPECTED_EMACS_SOURCE_SHA256.get(version)
    require(errors, known_source_sha is not None, f"unsupported Emacs version without validator-pinned source SHA256: {version}")
    if known_source_sha is not None:
        require(errors, source_sha == known_source_sha, "Emacs source SHA256 does not match validator-pinned release hash")
        require(errors, expected_source_sha == known_source_sha, "Emacs expected source SHA256 does not match validator-pinned release hash")
        require(errors, emacs.get("source_sha256_verified_before_extract") is True, "Emacs source SHA256 was not recorded as verified before extraction")
    require(errors, re.fullmatch(r"[0-9a-f]{64}", executable_sha or "") is not None, "Emacs executable SHA256 is missing or malformed")

    emacs_binary = sandbox_path(rootfs, "/opt/soma/emacs/bin/emacs")
    source_archive = sandbox_path(rootfs, f"/opt/soma/src/emacs-{version}.tar.xz")
    source_tree = sandbox_path(rootfs, f"/opt/soma/build/emacs-{version}")
    require(errors, emacs_binary.is_file() and os.access(emacs_binary, os.X_OK), "rootfs lacks executable /opt/soma/emacs/bin/emacs")
    require(errors, source_archive.is_file(), f"rootfs lacks Emacs source archive {source_archive}")
    require(errors, source_tree.is_dir(), f"rootfs lacks Emacs source tree {source_tree}")

    if source_archive.is_file() and source_sha:
        require(errors, sha256_file(source_archive) == source_sha, "Emacs source archive SHA256 does not match contract")
    if emacs_binary.is_file() and executable_sha:
        require(errors, sha256_file(emacs_binary) == executable_sha, "Emacs executable SHA256 does not match contract")
    for rel in REQUIRED_EMACS_SOURCE_FILES:
        require(errors, (source_tree / rel).is_file(), f"Emacs source tree lacks {rel}")

    configure_flags = emacs.get("configure_flags") or []
    require(errors, "--prefix=/opt/soma/emacs" in configure_flags, "Emacs configure flags do not set /opt/soma/emacs")
    require(errors, "--with-native-compilation" in configure_flags, "Emacs is not configured with native compilation")
    require(errors, "--with-tree-sitter" in configure_flags, "Emacs is not configured with tree-sitter support")
    require(errors, "--with-json" not in configure_flags, "Emacs configure flags include ignored --with-json")
    config_log = source_tree / "config.log"
    if config_log.is_file():
        for line in config_log.read_text(encoding="utf-8", errors="replace").splitlines():
            if "WARNING: unrecognized options:" in line:
                fail(errors, f"Emacs configure reported unrecognized options: {line.strip()}")
                break
    else:
        fail(errors, f"Emacs source tree lacks config.log: {config_log}")
    capabilities = emacs.get("capabilities") or {}
    require(errors, capabilities.get("json_parse_string") is True, "Emacs JSON parser capability is not recorded")
    require(errors, capabilities.get("native_comp_available") is True, "Emacs native-comp capability is not recorded")
    require(errors, capabilities.get("tree_sitter_available") is True, "Emacs tree-sitter capability is not recorded")

    interpreter = extract_interpreter(emacs_binary)
    if not interpreter:
        interpreter = str(((contract.get("abi") or {}).get("dynamic_loader")) or "")
    if not interpreter:
        fail(errors, "could not determine Emacs dynamic loader")
    else:
        try:
            resolved = resolve_sandbox_path(rootfs, interpreter)
            require(errors, resolved.exists(), f"dynamic loader is missing in rootfs: {interpreter}")
        except RuntimeError as exc:
            fail(errors, str(exc))

    ldd_lines = emacs.get("ldd") or []
    require(errors, isinstance(ldd_lines, list) and len(ldd_lines) > 0, "contract lacks Emacs ldd output")
    for path in ldd_paths([str(line) for line in ldd_lines]):
        if path == "not found":
            fail(errors, "Emacs ldd output contains a missing library")
            continue
        for prefix_forbidden in forbidden_prefixes:
            if prefix_forbidden and path.startswith(prefix_forbidden):
                fail(errors, f"Emacs ldd path uses forbidden prefix: {path}")
        try:
            resolved = resolve_sandbox_path(rootfs, path)
            require(errors, resolved.exists(), f"Emacs ldd path is missing in rootfs: {path}")
        except RuntimeError as exc:
            fail(errors, str(exc))

    doom = contract.get("doom") or {}
    require(errors, doom.get("kind") == "rootfs-owned-doom-sync", "Doom contract is not rootfs-owned-doom-sync")
    require(errors, sandbox_path(rootfs, "/opt/soma/doom-home/.emacs.d").is_dir(), "rootfs lacks Doom core")
    require(errors, sandbox_path(rootfs, "/opt/soma/doom-home/.doom.d").is_dir(), "rootfs lacks Doom user config")
    require(errors, bool(doom.get("commit")), "Doom commit is missing")

    inventory = contract.get("tool_inventory") or {}
    missing_tools = [tool for tool in REQUIRED_TOOLS if not (inventory.get(tool) or {}).get("path")]
    require(errors, not missing_tools, f"rootfs tool inventory is missing required tools: {', '.join(missing_tools)}")
    for tool, info in inventory.items():
        path = str((info or {}).get("path") or "")
        realpath = str((info or {}).get("realpath") or "")
        tool_sha = str((info or {}).get("sha256") or "")
        version_output = str((info or {}).get("version") or "")
        recorded_elf = (info or {}).get("elf")
        recorded_loader = str((info or {}).get("dynamic_loader") or "")
        recorded_ldd = (info or {}).get("ldd") or []
        is_required = tool in REQUIRED_TOOLS
        if not path:
            continue
        if is_required:
            require(errors, path.startswith("/"), f"required tool {tool} path is not absolute: {path}")
            require(errors, realpath.startswith("/"), f"required tool {tool} realpath is not absolute: {realpath}")
            require(errors, re.fullmatch(r"[0-9a-f]{64}", tool_sha or "") is not None, f"required tool {tool} SHA256 is missing or malformed")
            require(errors, version_output != "", f"required tool {tool} lacks version/probe output")
        for prefix_forbidden in forbidden_prefixes:
            if prefix_forbidden and path.startswith(prefix_forbidden):
                fail(errors, f"tool {tool} resolves through forbidden prefix: {path}")
            if prefix_forbidden and realpath.startswith(prefix_forbidden):
                fail(errors, f"tool {tool} realpath uses forbidden prefix: {realpath}")
        if path.startswith("/"):
            try:
                resolved = resolve_sandbox_path(rootfs, path)
                require(errors, resolved.exists(), f"tool {tool} path is missing in rootfs: {path}")
                if is_required:
                    require(errors, os.access(resolved, os.X_OK), f"required tool {tool} is not executable: {path}")
            except RuntimeError as exc:
                fail(errors, str(exc))
        if realpath.startswith("/"):
            try:
                resolved_realpath = resolve_sandbox_path(rootfs, realpath)
                require(errors, resolved_realpath.exists(), f"tool {tool} realpath is missing in rootfs: {realpath}")
                if is_required:
                    require(errors, os.access(resolved_realpath, os.X_OK), f"required tool {tool} realpath is not executable: {realpath}")
                    if resolved_realpath.is_file() and re.fullmatch(r"[0-9a-f]{64}", tool_sha or ""):
                        require(errors, sha256_file(resolved_realpath) == tool_sha, f"required tool {tool} SHA256 does not match realpath")
                    interp = extract_interpreter(resolved_realpath)
                    actual_elf = bool(interp)
                    if not actual_elf:
                        try:
                            with resolved_realpath.open("rb") as fh:
                                actual_elf = fh.read(4) == b"\x7fELF"
                        except OSError:
                            actual_elf = False
                    require(errors, recorded_elf is actual_elf, f"required tool {tool} ELF metadata does not match realpath")
                    if interp:
                        require(errors, recorded_loader == interp, f"required tool {tool} dynamic loader metadata does not match realpath")
                        require(errors, resolve_sandbox_path(rootfs, interp).exists(), f"required tool {tool} ELF interpreter is missing in rootfs: {interp}")
                    if actual_elf:
                        require(errors, isinstance(recorded_ldd, list) and len(recorded_ldd) > 0, f"required tool {tool} lacks ldd metadata")
                        for ldd_path in ldd_paths([str(line) for line in recorded_ldd]):
                            if ldd_path == "not found":
                                fail(errors, f"required tool {tool} ldd output contains a missing library")
                                continue
                            for prefix_forbidden in forbidden_prefixes:
                                if prefix_forbidden and ldd_path.startswith(prefix_forbidden):
                                    fail(errors, f"required tool {tool} ldd path uses forbidden prefix: {ldd_path}")
                            require(errors, resolve_sandbox_path(rootfs, ldd_path).exists(), f"required tool {tool} ldd path is missing in rootfs: {ldd_path}")
                    shebang = extract_shebang(resolved_realpath)
                    if shebang and shebang.startswith("/"):
                        require(errors, resolve_sandbox_path(rootfs, shebang).exists(), f"required tool {tool} shebang interpreter is missing in rootfs: {shebang}")
            except RuntimeError as exc:
                fail(errors, str(exc))

    summary = {
        "ok": not errors,
        "rootfs": str(rootfs),
        "contract": str(contract_path),
        "base_image": base_image,
        "image_tag": contract.get("image_tag", ""),
        "image_id": contract.get("image_id", ""),
        "glibc_owner": contract.get("glibc_owner", ""),
        "dynamic_loader": interpreter,
        "rootfs_tree_manifest": contract.get("rootfs_tree_manifest", {}),
        "emacs": {
            "version": version,
            "executable": executable,
            "source_url": source_url,
            "source_sha256": source_sha,
            "executable_sha256": executable_sha,
            "source_tree": f"/opt/soma/build/emacs-{version}",
        },
        "missing_required_tools": missing_tools,
        "warnings": warnings,
        "errors": errors,
    }
    return summary, errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rootfs", type=pathlib.Path)
    parser.add_argument(
        "--verify-rootfs-tree",
        action="store_true",
        help="recompute the rootfs tree hash instead of only checking recorded manifest files",
    )
    args = parser.parse_args()
    summary, errors = validate(args.rootfs, verify_rootfs_tree=args.verify_rootfs_tree)
    stream = sys.stderr if errors else sys.stdout
    json.dump(summary, stream, indent=2, sort_keys=True)
    stream.write("\n")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
