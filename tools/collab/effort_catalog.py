"""Effort metadata and record discovery shared by legacy Python callers.

This catalog does not load the optional record-branch landing extension.
"""
from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
import re

from worker import WorkerError, git

SLUG = re.compile(r"^[a-z0-9][a-z0-9-]*$")


@dataclass(frozen=True)
class WorktreeRecord:
    path: Path
    branch: str | None


@dataclass(frozen=True)
class EffortRecord:
    effort: str
    worktree: Path
    branch: str | None
    path: Path
    metadata: dict[str, str | None]
    text: str


@dataclass(frozen=True)
class EffortRecordProblem:
    effort: str
    path: Path
    message: str


def normalize_branch(branch: str | None) -> str | None:
    return branch.removeprefix("refs/heads/") if branch else None


def parse_effort_metadata(text: str) -> dict[str, str | None]:
    out = {"base": None, "integration_branch": None, "record_branch": None}
    for key, field in (
        ("Base", "base"),
        ("Integration branch", "integration_branch"),
        ("Record branch", "record_branch"),
    ):
        match = re.search(rf"^{re.escape(key)}:\s*(\S.*)?$", text, re.M)
        if match:
            value = (match.group(1) or "").strip()
            out[field] = value or None
    return out


def repository_worktrees(repo: Path) -> list[WorktreeRecord]:
    root = repo.resolve()
    entries: list[WorktreeRecord] = []
    current: dict[str, str] = {}
    for line in git(repo, "worktree", "list", "--porcelain").splitlines() + [""]:
        if not line:
            if current.get("path"):
                entries.append(WorktreeRecord(Path(current["path"]).resolve(), normalize_branch(current.get("branch"))))
            current = {}
        elif line.startswith("worktree "):
            current["path"] = line[9:]
        elif line.startswith("branch "):
            current["branch"] = line[7:]
    if root not in [entry.path for entry in entries]:
        branch = git(root, "symbolic-ref", "-q", "--short", "HEAD", check=False).strip()
        entries.append(WorktreeRecord(root, normalize_branch(branch)))
    entries.sort(key=lambda entry: (entry.path != root, str(entry.path)))
    return entries


def scan_effort_records(
    repo: Path, effort: str | None = None,
) -> tuple[dict[str, list[EffortRecord]], list[EffortRecordProblem]]:
    if effort is not None and not SLUG.match(effort):
        raise WorkerError(f"bad effort slug {effort!r}: use lowercase letters, digits, hyphens")
    candidates: dict[str, list[EffortRecord]] = {}
    problems: list[EffortRecordProblem] = []
    for worktree in repository_worktrees(repo):
        efforts = worktree.path / "docs" / "efforts"
        if effort is not None:
            paths = [efforts / effort / "EFFORT.md"]
        elif efforts.is_dir():
            paths = [path / "EFFORT.md" for path in sorted(efforts.iterdir()) if path.is_dir()]
        else:
            paths = []
        for path in paths:
            if not path.is_file():
                continue
            try:
                text = path.read_text()
            except OSError as error:
                problems.append(EffortRecordProblem(path.parent.name, path, f"could not read effort record: {error}"))
                continue
            record = EffortRecord(
                effort=path.parent.name,
                worktree=worktree.path,
                branch=worktree.branch,
                path=path,
                metadata=parse_effort_metadata(text),
                text=text,
            )
            candidates.setdefault(record.effort, []).append(record)
    return candidates, problems


def effort_record_candidates(repo: Path, effort: str | None = None) -> dict[str, list[EffortRecord]]:
    candidates, problems = scan_effort_records(repo, effort)
    relevant = [problem for problem in problems if effort is None or problem.effort == effort]
    if relevant:
        raise WorkerError("; ".join(f"{problem.path}: {problem.message}" for problem in relevant))
    return candidates


def effort_record_locations(records: list[EffortRecord]) -> str:
    return "; ".join(
        f"{record.path} (branch {record.branch or 'detached'}, "
        f"Integration branch {record.metadata.get('integration_branch') or 'missing'}, "
        f"Record branch {record.metadata.get('record_branch') or 'missing'})"
        for record in records
    )


def selected_by_metadata_branch(records: list[EffortRecord], field: str) -> list[EffortRecord]:
    return [
        record for record in records
        if record.metadata.get(field) and record.branch == normalize_branch(record.metadata.get(field))
    ]


def select_effort_record(repo: Path, effort: str, records: list[EffortRecord]) -> EffortRecord:
    if not records:
        searched = ", ".join(str(worktree.path) for worktree in repository_worktrees(repo))
        raise WorkerError(f"effort record docs/efforts/{effort}/EFFORT.md not found in {searched}")
    preferred = selected_by_metadata_branch(records, "record_branch")
    if len(preferred) == 1:
        return preferred[0]
    preferred = selected_by_metadata_branch(records, "integration_branch")
    if len(preferred) == 1:
        return preferred[0]
    local = [record for record in records if record.worktree == repo.resolve()]
    if local:
        return local[0]
    if len(records) == 1:
        return records[0]
    if all(
        not record.metadata.get("integration_branch") and not record.metadata.get("record_branch")
        for record in records
    ):
        raise WorkerError(
            f"effort record for {effort!r} is not in this checkout; found only other worktree copies: "
            f"{effort_record_locations(records)}"
        )
    raise WorkerError(
        f"ambiguous effort record for {effort!r}: found multiple records and no single worktree is checked out "
        f"on the record's Record branch or Integration branch: {effort_record_locations(records)}"
    )


def resolve_effort_record(repo: Path, effort: str) -> EffortRecord:
    return select_effort_record(repo, effort, effort_record_candidates(repo, effort).get(effort, []))


def resolve_effort_records(repo: Path) -> list[EffortRecord]:
    records = effort_record_candidates(repo)
    return [select_effort_record(repo, effort, records[effort]) for effort in sorted(records)]


def effort_metadata(repo: Path, effort: str) -> dict[str, str | None]:
    return resolve_effort_record(repo, effort).metadata


def resolve_target_branch(args: argparse.Namespace, repo: Path) -> tuple[str, EffortRecord | None]:
    if not args.ticket:
        return args.mainline or "main", None
    record = resolve_effort_record(repo, args.effort)
    target = args.mainline or record.metadata.get("integration_branch")
    if not target:
        raise WorkerError(f"effort {args.effort!r} does not name an integration branch")
    return target, record
