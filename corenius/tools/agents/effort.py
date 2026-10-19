#!/usr/bin/env python3
"""Durable efforts: decisions and state in the repo, execution detail in scratch.

An effort has two homes (docs/agents/agentic-engineering.md, "Efforts"):

* ``docs/efforts/<slug>/`` tracked: EFFORT.md (charter), STATE.md (the
  dehydrated snapshot), decisions/, tickets/, evidence/.
* ``.scratch/<slug>/`` gitignored: worktrees' build output, run logs, drafts.

Commands
  new <slug>          create the effort branch, worktree and durable files
  move-records <slug> move an effort's tracked records to a record branch
  decide <slug> <t>   add a decision file (append-only; supersede, never edit)
  snapshot <slug>     refresh STATE.md without staging or committing; dirty
                      contents remain local and are not backed up
  dehydrate <slug>    refresh STATE.md's machine block from git and run dirs,
                      require the human part, refuse dirty worktrees, commit
  rehydrate <slug>    verify every pointer in STATE.md, restore missing
                      worktrees, print a briefing and a drift report
  status              list efforts with status and staleness

Exit codes: 0 ok, 1 warnings (for example lost scratch), 2 errors. Standard
library only; reuses the git helpers in worker.py.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "collab"))
import land as land_tool  # noqa: E402
from landing_protocol import guard_initialized_trunk_mutation  # noqa: E402
from common import Refused  # noqa: E402
from worker import WorkerError, git, main_worktree, now  # noqa: E402

OK, WARN, ERROR = 0, 1, 2
START, END = "<!-- effort:machine:start -->", "<!-- effort:machine:end -->"
SLUG = re.compile(r"^[a-z0-9][a-z0-9-]*$")
HUMAN_REQUIRED = ("Where we are", "Next actions")
HUMAN_SECTIONS = ("Where we are", "Next actions", "Open questions", "Traps", "Do not redo")

EFFORT_TEMPLATE = """\
# Effort: {slug}

Status: active
Base: {base}
Integration branch: effort/{slug}
Lead:

## Intent

_TODO: what this effort is for, in a paragraph._

## Scope and non-goals

## Milestones and exit criteria

- [ ] _TODO: each milestone with a command or document that checks it._
"""

STATE_TEMPLATE = """\
# State: {slug}

{start}
{end}

## Where we are

_TODO_

## Next actions

_TODO_

## Open questions

## Traps

Things a fresh agent would get wrong.

## Do not redo
"""

DECISION_TEMPLATE = """\
# {title}

Date: {date}
Supersedes:

## Decision

## Why

## Consequences
"""


def repo_root(repo: str | None) -> Path:
    return Path(git(repo or ".", "rev-parse", "--show-toplevel").strip())


def effort_dir(root: Path, slug: str) -> Path:
    if not SLUG.match(slug):
        raise WorkerError(f"bad effort slug {slug!r}: use lowercase letters, digits, hyphens")
    return root / "docs" / "efforts" / slug


def effort_record(root: Path, slug: str) -> land_tool.EffortRecord:
    return land_tool.resolve_effort_record(root, slug)


def effort_record_dir(root: Path, slug: str) -> Path:
    return effort_record(root, slug).path.parent


def effort_metadata(root: Path, slug: str) -> dict[str, str | None]:
    return effort_record(root, slug).metadata


def scratch_dir(root: Path, slug: str) -> Path:
    return main_worktree(root) / ".scratch" / slug


def rev(repo: Path | str, ref: str) -> str | None:
    out = git(repo, "rev-parse", "--verify", "-q", ref + "^{commit}", check=False).strip()
    return out or None


def is_ancestor(repo: Path | str, older: str, newer: str) -> bool:
    import subprocess

    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", older, newer], capture_output=True
    ).returncode == 0


def merge_base(repo: Path | str, left: str, right: str) -> str | None:
    out = git(repo, "merge-base", left, right, check=False).strip()
    return out or None


def accepted_candidates(root: Path, effort_root: Path, fork: str | None, integration: str | None) -> list[dict]:
    if not fork or not integration or not rev(root, integration):
        return []
    commits = git(root, "rev-list", "--reverse", f"{fork}..{integration}", check=False).splitlines()
    order = {commit: index for index, commit in enumerate(commits)}
    candidates = []
    evidence = effort_root / "evidence"
    for path in sorted(evidence.glob("acceptance-*.jsonl")) if evidence.is_dir() else []:
        for line in path.read_text().splitlines():
            if not line.strip():
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                continue
            candidate = record.get("candidate")
            ticket = record.get("ticket")
            if record.get("accepted") is True and candidate in order and ticket:
                candidates.append({"ticket": ticket, "candidate": candidate, "_order": order[candidate]})
    candidates.sort(key=lambda item: (item["_order"], item["ticket"]))
    return [{"ticket": item["ticket"], "candidate": item["candidate"]} for item in candidates]


def only_effort_changes(repo: Path, old: str, new: str, slug: str) -> bool:
    """True if old..new touches nothing outside the effort's own directory.

    The effort branch's recorded head predates the dehydrate commit itself, so
    that one commit must not count as drift.
    """
    names = git(repo, "diff", "--name-only", old, new).split()
    return all(n.startswith(f"docs/efforts/{slug}/") for n in names)


# --- STATE.md -----------------------------------------------------------------


def split_state(text: str) -> tuple[dict, str]:
    """Return (machine dict, text with the machine block removed)."""
    if START not in text or END not in text:
        raise WorkerError("STATE.md has no machine block markers")
    head, rest = text.split(START, 1)
    block, tail = rest.split(END, 1)
    match = re.search(r"```json\n(.*?)\n```", block, re.S)
    machine = json.loads(match.group(1)) if match else {}
    return machine, head + START + "\n" + END + tail


def join_state(text: str, machine: dict) -> str:
    block = f"{START}\n```json\n{json.dumps(machine, indent=2, sort_keys=True)}\n```\n{END}"
    head, rest = text.split(START, 1)
    _, tail = rest.split(END, 1)
    return head + block + tail


def human_sections(text: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for match in re.finditer(r"^## (.+?)\n(.*?)(?=^## |\Z)", text, re.M | re.S):
        out[match.group(1).strip()] = match.group(2).strip()
    return out


def human_problems(text: str) -> list[str]:
    sections = human_sections(text)
    problems = []
    for name in HUMAN_REQUIRED:
        body = sections.get(name, "")
        if not body or "_TODO" in body:
            problems.append(f"STATE.md section '{name}' is empty or still a placeholder")
    return problems


def dirty_paths(worktree: Path) -> list[str]:
    paths = []
    entries = iter(git(worktree, "--no-optional-locks", "status", "--porcelain", "-z", "-uall").split("\0"))
    for entry in entries:
        if not entry:
            continue
        paths.append(entry[3:])
        if "R" in entry[:2] or "C" in entry[:2]:
            next(entries)  # -z reports the destination first, then the old path.
    return paths


def registered_worktrees(root: Path) -> dict[str, dict]:
    """Map path -> {branch, head} for every registered worktree."""
    out: dict[str, dict] = {}
    current: dict = {}
    for line in git(root, "worktree", "list", "--porcelain").splitlines() + [""]:
        if not line:
            if current.get("path"):
                out[current["path"]] = current
            current = {}
        elif line.startswith("worktree "):
            current["path"] = line[9:]
        elif line.startswith("HEAD "):
            current["head"] = line[5:]
        elif line.startswith("branch "):
            current["branch"] = line[7:].removeprefix("refs/heads/")
    return out


def describe_worktree(root: Path, path: str, own_effort: Path | None) -> dict:
    p = Path(path)
    if not p.exists():
        raise WorkerError(f"worktree {path} does not exist")
    branch = git(p, "rev-parse", "--abbrev-ref", "HEAD").strip()
    head = git(p, "rev-parse", "HEAD").strip()
    dirty = dirty_paths(p)
    pushed = bool(git(p, "branch", "-r", "--contains", head, check=False).strip())
    return {"path": str(p), "branch": branch, "head": head, "dirty": dirty, "pushed": pushed}


def read_run(run_dir: str) -> dict:
    d = Path(run_dir)
    info: dict = {"run_dir": str(d)}
    verdict = d / "verdict.json"
    if verdict.exists():
        data = json.loads(verdict.read_text())
        info["verdict"] = data.get("verdict")
        info["exit_code"] = data.get("exit_code")
    else:
        info["verdict"] = None
    return info


# --- commands -----------------------------------------------------------------


def cmd_new(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def record_branch_name(slug: str) -> str:
    return f"effort/{slug}-records"


def append_record_branch(text: str, branch: str) -> str:
    if re.search(r"^Record branch:\s*", text, re.M):
        return re.sub(r"^Record branch:\s*.*$", f"Record branch: {branch}", text, count=1, flags=re.M)
    if re.search(r"^Integration branch:\s*.*$", text, re.M):
        return re.sub(
            r"^(Integration branch:\s*.*)$",
            rf"\1\nRecord branch: {branch}",
            text,
            count=1,
            flags=re.M,
        )
    return text.rstrip() + f"\nRecord branch: {branch}\n"


def current_branch(root: Path) -> str:
    return git(root, "branch", "--show-current").strip()


def clean_worktree(root: Path) -> bool:
    return not git(root, "status", "--porcelain", "--untracked-files=all").strip()


def cmd_move_records(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def cmd_decide(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def cmd_snapshot(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def cmd_dehydrate(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def cmd_rehydrate(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def cmd_status(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def main(argv: list[str] | None = None) -> int:
    print('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.', file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
