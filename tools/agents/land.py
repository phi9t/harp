#!/usr/bin/env python3
"""Deprecated: use tools/collab/collab.py. Legacy flags and exit codes remain supported."""
from __future__ import annotations

import importlib
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "collab"))

_CATALOG_EXPORTS = {
    "EffortRecord",
    "EffortRecordProblem",
    "SLUG",
    "WorktreeRecord",
    "effort_metadata",
    "effort_record_candidates",
    "effort_record_locations",
    "normalize_branch",
    "parse_effort_metadata",
    "repository_worktrees",
    "resolve_effort_record",
    "resolve_effort_records",
    "resolve_target_branch",
    "scan_effort_records",
    "select_effort_record",
    "selected_by_metadata_branch",
}


def __getattr__(name: str):
    if name in _CATALOG_EXPORTS:
        module = "effort_catalog"
    elif name in {"record_mainline_override", "record_landing", "commit_effort_record_change"}:
        module = "effort_records"
    else:
        module = "legacy_landing"
    return getattr(importlib.import_module(module), name)


def main(argv: list[str] | None = None) -> int:
    from legacy_landing import main as legacy_main
    return legacy_main(argv)


if __name__ == "__main__":
    raise SystemExit(main())
