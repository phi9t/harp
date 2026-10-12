"""Collab admission policy for protected paths and check command shape."""
from __future__ import annotations

from common import Refused

PROTECTED_PREFIXES = (".tickets/", ".github/", ".corenius/", "docs/agents/")
PROTECTED_FILES = ("AGENTS.md", "CLAUDE.md")


def validate_check_argv(check_argv: object) -> list[str]:
    if (
        not isinstance(check_argv, list)
        or not check_argv
        or not all(isinstance(item, str) and item and "\0" not in item for item in check_argv)
    ):
        raise Refused("Provide a nonempty check argv JSON array")
    return check_argv


def protect_candidate_paths(paths: list[str]) -> None:
    if any(path in PROTECTED_FILES or path.startswith(PROTECTED_PREFIXES) for path in paths if path):
        raise Refused("Candidate changes coordination/policy paths; use a separately approved operator change")
