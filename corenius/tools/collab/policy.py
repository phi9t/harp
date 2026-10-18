"""Collab admission policy for protected paths and check command shape."""
from __future__ import annotations

import math

from common import Refused

DEFAULT_PROTECTED_PATHS = [
    ".tickets/",
    ".github/",
    ".corenius/",
    "docs/agents/",
    "AGENTS.md",
    "CLAUDE.md",
    "tools/agents/checks.d/",
    "tools/agents/check.sh",
    "CONSTITUTION.md",
]


def validate_check_argv(check_argv: object) -> list[str]:
    if (
        not isinstance(check_argv, list)
        or not check_argv
        or not all(isinstance(item, str) and item and "\0" not in item for item in check_argv)
    ):
        raise Refused("Provide a nonempty check argv JSON array")
    return check_argv


def validate_override_owners(owners: object | None) -> list[str]:
    if owners is None:
        return []
    if not isinstance(owners, list) or any(
        not isinstance(owner, str) or not owner.strip() or "\0" in owner or owner != owner.strip()
        for owner in owners
    ):
        raise Refused("Override owners must be a JSON array of nonempty owner names")
    if len(owners) != len(set(owners)):
        raise Refused("Override owners must not contain duplicates")
    return owners


def validate_check_timeout(timeout_seconds: float | None) -> float | None:
    if timeout_seconds is None:
        return None
    if timeout_seconds <= 0 or not math.isfinite(timeout_seconds):
        raise Refused("Provide a finite positive check timeout in seconds")
    return timeout_seconds


def validate_protected_paths(protected_paths: object | None) -> list[str]:
    if protected_paths is None:
        return list(DEFAULT_PROTECTED_PATHS)
    if not isinstance(protected_paths, list):
        raise Refused("provide protected paths as a JSON array")
    normalized = []
    for path in protected_paths:
        if not isinstance(path, str) or not path or "\0" in path:
            raise Refused("protected paths must be nonempty strings")
        if path.startswith("/"):
            raise Refused("protected paths must be repository-relative")
        stem = path[:-1] if path.endswith("/") else path
        parts = stem.split("/")
        if not stem or any(part in ("", ".", "..") for part in parts):
            raise Refused("protected paths must be normalized repository paths")
        normalized.append(path)
    if len(normalized) != len(set(normalized)):
        raise Refused("protected paths must not contain duplicates")
    return normalized


def _matches_protected_path(path: str, protected_path: str) -> bool:
    if protected_path.endswith("/"):
        return path.startswith(protected_path)
    return path == protected_path


def protect_candidate_paths(paths: list[str], protected_paths: object | None) -> None:
    protected = validate_protected_paths(protected_paths)
    for path in paths:
        if not path:
            continue
        if path.startswith("docs/efforts/") and "/tickets/" in path:
            raise Refused(f"Candidate changes protected tracker ticket: {path!r}")
        if any(_matches_protected_path(path, protected_path) for protected_path in protected):
            raise Refused(
                f"Candidate changes protected path {path!r} (coordination/policy); "
                "use a separately approved operator change"
            )
