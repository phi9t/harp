#!/usr/bin/env python3
"""Apply Harp's reviewed portable Obsidian profile to one explicit vault."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import stat
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path


PROFILE_VERSION = "harp-obsidian-profile/v1"
REQUIRED_PROFILE_FILES = {
    "app.json",
    "core-plugins.json",
    "bookmarks.json",
    "workspace.json",
    "snippets/harp-knowledge.css",
}


class ProfileError(ValueError):
    """Raised when applying the portable profile would be unsafe."""


def is_regular_non_symlink(path: Path) -> bool:
    try:
        return stat.S_ISREG(path.lstat().st_mode)
    except FileNotFoundError:
        return False


def reject_symlinked_path(path: Path, description: str) -> None:
    if path.is_symlink():
        raise ProfileError(f"{description} must not be a symlink: {path}")


def validate_vault(vault: Path) -> Path:
    if not vault.is_absolute():
        raise ProfileError("vault path must be absolute")
    reject_symlinked_path(vault, "vault")
    if not vault.is_dir():
        raise ProfileError(f"vault must be an existing directory: {vault}")
    return vault.resolve()


def validate_profile(profile_root: Path) -> None:
    reject_symlinked_path(profile_root, "profile source")
    if not profile_root.is_dir():
        raise ProfileError(f"profile source must be a directory: {profile_root}")
    found_files = {
        path.relative_to(profile_root).as_posix()
        for path in profile_root.rglob("*")
        if path.is_file() or path.is_symlink()
    }
    missing = sorted(REQUIRED_PROFILE_FILES - found_files)
    if missing:
        raise ProfileError(f"missing profile source file: {missing[0]}")
    unexpected = sorted(found_files - REQUIRED_PROFILE_FILES)
    if unexpected:
        raise ProfileError(f"profile source file is not allowlisted: {unexpected[0]}")
    for relative_path in REQUIRED_PROFILE_FILES:
        source = profile_root / relative_path
        reject_symlinked_path(source, "profile source file")
        if not is_regular_non_symlink(source):
            raise ProfileError(f"profile source file must be regular: {source}")


def ensure_real_directory(path: Path, mode: int, description: str) -> None:
    if path.exists() or path.is_symlink():
        reject_symlinked_path(path, description)
        if not path.is_dir():
            raise ProfileError(f"{description} must be a directory: {path}")
    else:
        path.mkdir(mode=mode)
    os.chmod(path, mode)


def destination_for(destination_root: Path, relative_path: str) -> Path:
    target = destination_root / relative_path
    parent = target.parent
    components: list[Path] = []
    while parent != destination_root:
        components.append(parent)
        parent = parent.parent
    for component in reversed(components):
        ensure_real_directory(component, 0o700, "profile destination component")
    reject_symlinked_path(target, "profile destination file")
    return target


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def atomic_write_json(destination: Path, value: dict[str, object]) -> None:
    reject_symlinked_path(destination, "profile receipt")
    file_descriptor, temporary_name = tempfile.mkstemp(
        prefix=".harp-profile-receipt-",
        dir=destination.parent,
    )
    temporary_path = Path(temporary_name)
    try:
        with os.fdopen(file_descriptor, "w", encoding="utf-8") as temporary_file:
            json.dump(value, temporary_file, indent=2, sort_keys=True)
            temporary_file.write("\n")
            temporary_file.flush()
            os.fsync(temporary_file.fileno())
        os.chmod(temporary_path, 0o600)
        os.replace(temporary_path, destination)
    except BaseException:
        temporary_path.unlink(missing_ok=True)
        raise


def apply_profile(
    vault: Path, profile_root: Path | None = None, replace: bool = False
) -> dict[str, object]:
    vault_root = validate_vault(vault)
    source_root = (
        profile_root
        if profile_root is not None
        else Path(__file__).resolve().parent / "profile"
    )
    source_root = source_root.absolute()
    validate_profile(source_root)

    destination_root = vault_root / ".obsidian"
    ensure_real_directory(destination_root, 0o700, ".obsidian")
    destinations = {
        relative_path: destination_for(destination_root, relative_path)
        for relative_path in sorted(REQUIRED_PROFILE_FILES)
    }
    existing = [relative_path for relative_path, target in destinations.items() if target.exists()]
    if existing and not replace:
        raise ProfileError(
            "profile destination already exists; rerun with --replace for allowlisted files: "
            + ", ".join(existing)
        )
    for relative_path, destination in destinations.items():
        source = source_root / relative_path
        if destination.exists() and not is_regular_non_symlink(destination):
            raise ProfileError(f"profile destination file must be regular: {destination}")
        shutil.copyfile(source, destination)
        os.chmod(destination, 0o600)

    receipt = {
        "profile_version": PROFILE_VERSION,
        "applied_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "files": {
            relative_path: sha256(source_root / relative_path)
            for relative_path in sorted(REQUIRED_PROFILE_FILES)
        },
    }
    atomic_write_json(destination_root / "harp-profile-receipt.json", receipt)
    return receipt


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vault", required=True, type=Path, help="absolute vault directory")
    parser.add_argument(
        "--replace",
        action="store_true",
        help="replace only the profile's allowlisted destination files",
    )
    options = parser.parse_args(arguments)
    try:
        receipt = apply_profile(options.vault, replace=options.replace)
    except (OSError, UnicodeError, ProfileError) as error:
        print(f"Profile application failed: {error}", file=sys.stderr)
        return 1
    print(
        "Applied "
        f"{receipt['profile_version']} to {options.vault / '.obsidian'} "
        f"at {receipt['applied_at_utc']}."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
