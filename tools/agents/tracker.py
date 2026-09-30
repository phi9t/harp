#!/usr/bin/env python3
"""Tracked effort tickets and claims.

Tickets live at ``docs/efforts/<effort>/tickets/NN-slug.md`` and use the
contract in ``docs/efforts/agentic-engineering/milestone-b/contracts``.

Commands
  new EFFORT SLUG --title T [--type feature|bug|task] [--blocked-by NN,NN]
  claim EFFORT NN --owner NAME [--ttl-hours H]
  resolve EFFORT NN --owner NAME [--force]
  list EFFORT [--json]
  lint [EFFORT...]

Exit codes: 0 ok, 1 refusal or lint problems, 64 usage/tool error.
Set CORENIUS_TRACKER_NOW to an UTC timestamp (YYYY-MM-DDTHH:MM:SSZ) in tests.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from worker import WorkerError, now  # noqa: E402

OK, REFUSED, USAGE = 0, 1, 64

HEADER_KEYS = ("Type", "Status", "Owner", "Blocked by", "Claimed")
TYPES = {"feature", "bug", "task"}
STATUSES = {"open", "claimed", "resolved"}
EFFORT_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
SLUG_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
ID_RE = re.compile(r"^\d{2}$")
FILE_RE = re.compile(r"^(\d{2})-([a-z0-9][a-z0-9-]*)\.md$")
TITLE_RE = re.compile(r"^# (\d{2}) ([a-z0-9][a-z0-9-]*): (.+)$")
STAMP_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
CLAIMED_RE = re.compile(rf"^({STAMP_RE.pattern[1:-1]}) expires ({STAMP_RE.pattern[1:-1]})$")
COMMENT_RE = re.compile(rf"^- ({STAMP_RE.pattern[1:-1]}) [^:]+: .+$")
CHECK_RE = re.compile(r"^- \[([ xX])\] .+")


@dataclass(order=True)
class Problem:
    path: Path
    line: int
    message: str

    def format(self, root: Path) -> str:
        try:
            shown = self.path.relative_to(root)
        except ValueError:
            shown = self.path
        return f"{shown}:{self.line}: {self.message}"


@dataclass
class Ticket:
    path: Path
    lines: list[str]
    id: str = ""
    slug: str = ""
    title: str = ""
    type: str = ""
    status: str = ""
    owner: str = ""
    blocked_by: list[str] = field(default_factory=list)
    claimed: str = ""
    acceptance: list[tuple[int, bool]] = field(default_factory=list)
    comments_line: int | None = None
    blocker_line: int = 6
    problems: list[Problem] = field(default_factory=list)


def parse_utc(stamp: str) -> datetime:
    if not STAMP_RE.match(stamp):
        raise WorkerError(f"bad UTC timestamp {stamp!r}: expected YYYY-MM-DDTHH:MM:SSZ")
    try:
        return datetime.strptime(stamp, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    except ValueError as error:
        raise WorkerError(f"bad UTC timestamp {stamp!r}: {error}") from error


def format_utc(value: datetime) -> str:
    return value.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def current_time() -> datetime:
    return parse_utc(os.environ.get("CORENIUS_TRACKER_NOW") or now())


def repo_root(repo: str | None) -> Path:
    return Path(repo or ".").resolve()


def validate_effort(effort: str) -> None:
    if not EFFORT_RE.match(effort):
        raise WorkerError(f"bad effort slug {effort!r}: use lowercase letters, digits, hyphens")


def validate_slug(slug: str) -> None:
    if not SLUG_RE.match(slug):
        raise WorkerError(f"bad ticket slug {slug!r}: use lowercase letters, digits, hyphens")


def effort_dir(root: Path, effort: str) -> Path:
    validate_effort(effort)
    return root / "docs" / "efforts" / effort


def tickets_dir(root: Path, effort: str) -> Path:
    return effort_dir(root, effort) / "tickets"


def parse_blockers(value: str, path: Path, line: int, problems: list[Problem]) -> list[str]:
    if not value.strip():
        return []
    blockers = []
    for raw in value.split(","):
        item = raw.strip()
        if not ID_RE.match(item):
            problems.append(Problem(path, line, f"bad blocker id {item!r}"))
            continue
        blockers.append(item)
    return blockers


def parse_claimed(value: str) -> tuple[datetime, datetime] | None:
    match = CLAIMED_RE.match(value)
    if not match:
        return None
    try:
        return parse_utc(match.group(1)), parse_utc(match.group(2))
    except WorkerError:
        return None


def valid_comment(line: str) -> bool:
    match = COMMENT_RE.match(line)
    if not match:
        return False
    try:
        parse_utc(match.group(1))
    except WorkerError:
        return False
    return True


def section_bounds(lines: list[str], heading: str) -> tuple[int | None, int | None]:
    start = None
    for index, line in enumerate(lines):
        if line == heading:
            start = index
            break
    if start is None:
        return None, None
    end = len(lines)
    for index in range(start + 1, len(lines)):
        if lines[index].startswith("## "):
            end = index
            break
    return start, end


def parse_ticket(path: Path) -> Ticket:
    text = path.read_text()
    lines = text.splitlines()
    ticket = Ticket(path=path, lines=lines)
    problems = ticket.problems

    file_match = FILE_RE.match(path.name)
    if not file_match:
        problems.append(Problem(path, 1, "bad ticket filename"))
    else:
        ticket.id, ticket.slug = file_match.groups()

    if not lines:
        problems.append(Problem(path, 1, "empty ticket file"))
        return ticket

    title_match = TITLE_RE.match(lines[0])
    if not title_match:
        problems.append(Problem(path, 1, "bad title line"))
    else:
        title_id, title_slug, title = title_match.groups()
        ticket.title = title
        if ticket.id and title_id != ticket.id:
            problems.append(Problem(path, 1, f"title id {title_id} does not match filename id {ticket.id}"))
        if ticket.slug and title_slug != ticket.slug:
            problems.append(Problem(path, 1, f"title slug {title_slug} does not match filename slug {ticket.slug}"))
        ticket.id = ticket.id or title_id
        ticket.slug = ticket.slug or title_slug

    if len(lines) < 2 or lines[1] != "":
        problems.append(Problem(path, 2 if len(lines) >= 2 else len(lines) + 1,
                                "header must follow one blank line after title"))

    values: dict[str, str] = {}
    for offset, key in enumerate(HEADER_KEYS, start=2):
        line_no = offset + 1
        prefix = f"{key}:"
        if len(lines) <= offset or not lines[offset].startswith(prefix):
            problems.append(Problem(path, min(line_no, len(lines) + 1),
                                    f"missing or reordered header line: expected {prefix}"))
            continue
        values[key] = lines[offset][len(prefix):].strip()

    ticket.type = values.get("Type", "")
    ticket.status = values.get("Status", "")
    ticket.owner = values.get("Owner", "")
    ticket.claimed = values.get("Claimed", "")
    ticket.blocked_by = parse_blockers(values.get("Blocked by", ""), path, ticket.blocker_line, problems)

    if ticket.type and ticket.type not in TYPES:
        problems.append(Problem(path, 3, f"bad Type {ticket.type!r}"))
    if ticket.status and ticket.status not in STATUSES:
        problems.append(Problem(path, 4, f"bad Status {ticket.status!r}"))
    if ticket.status == "claimed" and (not ticket.owner or not ticket.claimed):
        problems.append(Problem(path, 4, "Status claimed requires Owner and Claimed"))
    if ticket.status in {"open", "resolved"} and ticket.claimed:
        problems.append(Problem(path, 7, f"Status {ticket.status} must leave Claimed empty"))
    if ticket.claimed and parse_claimed(ticket.claimed) is None:
        problems.append(Problem(path, 7, "malformed Claimed line"))

    acceptance_start, acceptance_end = section_bounds(lines, "## Acceptance")
    if acceptance_start is None:
        problems.append(Problem(path, len(lines) + 1, "missing Acceptance section"))
    else:
        for index in range(acceptance_start + 1, acceptance_end or len(lines)):
            stripped = lines[index].strip()
            if not stripped:
                continue
            match = CHECK_RE.match(stripped)
            if match:
                ticket.acceptance.append((index + 1, match.group(1).lower() == "x"))
            elif stripped.startswith("- "):
                problems.append(Problem(path, index + 1, "malformed acceptance item"))
        if not ticket.acceptance:
            problems.append(Problem(path, acceptance_start + 1, "empty Acceptance"))

    comments_start, comments_end = section_bounds(lines, "## Comments")
    if comments_start is None:
        problems.append(Problem(path, len(lines) + 1, "missing Comments section"))
    else:
        ticket.comments_line = comments_start
        for index in range(comments_start + 1, comments_end or len(lines)):
            stripped = lines[index].strip()
            if stripped and not valid_comment(stripped):
                problems.append(Problem(path, index + 1, "malformed comment line"))

    return ticket


def load_tickets(root: Path, effort: str) -> tuple[Path, list[Ticket], list[Problem]]:
    directory = tickets_dir(root, effort)
    if not directory.is_dir():
        raise WorkerError(f"no tickets directory at {directory}")
    tickets = [parse_ticket(path) for path in sorted(directory.glob("*.md"))]
    problems: list[Problem] = []
    for ticket in tickets:
        problems.extend(ticket.problems)

    seen: dict[str, Ticket] = {}
    by_id: dict[str, Ticket] = {}
    for ticket in tickets:
        if not ticket.id:
            continue
        if ticket.id in seen:
            first = seen[ticket.id]
            problems.append(Problem(first.path, 1, f"duplicate id {ticket.id}"))
            problems.append(Problem(ticket.path, 1, f"duplicate id {ticket.id}"))
        else:
            seen[ticket.id] = ticket
            by_id[ticket.id] = ticket

    for ticket in tickets:
        for blocker in ticket.blocked_by:
            if blocker == ticket.id:
                problems.append(Problem(ticket.path, ticket.blocker_line, f"self blocker {blocker}"))
            elif blocker not in by_id:
                problems.append(Problem(ticket.path, ticket.blocker_line, f"unknown blocker {blocker}"))

    def reaches(start: str, current: str, seen_stack: set[str]) -> bool:
        ticket = by_id.get(current)
        if ticket is None:
            return False
        for blocker in ticket.blocked_by:
            if blocker == start:
                return True
            if blocker in seen_stack:
                continue
            if reaches(start, blocker, seen_stack | {blocker}):
                return True
        return False

    for ticket_id, ticket in by_id.items():
        if any(blocker == ticket_id or blocker not in by_id for blocker in ticket.blocked_by):
            continue
        if reaches(ticket_id, ticket_id, {ticket_id}):
            problems.append(Problem(ticket.path, ticket.blocker_line, "cyclic blocker"))

    return directory, tickets, sorted(problems)


def tickets_by_id(tickets: list[Ticket]) -> dict[str, Ticket]:
    out: dict[str, Ticket] = {}
    for ticket in tickets:
        if ticket.id and ticket.id not in out:
            out[ticket.id] = ticket
    return out


def ensure_clean(problems: list[Problem]) -> None:
    if problems:
        first = problems[0]
        raise WorkerError(f"ticket lint problem: {first.path}:{first.line}: {first.message}")


def find_ticket(root: Path, effort: str, ticket_id: str) -> tuple[Ticket, dict[str, Ticket]]:
    if not ID_RE.match(ticket_id):
        raise WorkerError(f"bad ticket id {ticket_id!r}: expected NN")
    _, tickets, problems = load_tickets(root, effort)
    ensure_clean(problems)
    by_id = tickets_by_id(tickets)
    ticket = by_id.get(ticket_id)
    if ticket is None:
        raise WorkerError(f"ticket {ticket_id} not found in effort {effort}")
    return ticket, by_id


def blockers_resolved(ticket: Ticket, by_id: dict[str, Ticket]) -> bool:
    return all(by_id[blocker].status == "resolved" for blocker in ticket.blocked_by)


def claimed_until(ticket: Ticket) -> datetime | None:
    parsed = parse_claimed(ticket.claimed)
    return parsed[1] if parsed else None


def write_transition(ticket: Ticket, updates: dict[str, str], actor: str, text: str, stamp: datetime) -> None:
    if ticket.comments_line is None:
        raise WorkerError(f"{ticket.path} has no Comments section")
    lines = list(ticket.lines)
    for key, value in updates.items():
        if key not in HEADER_KEYS:
            raise WorkerError(f"unknown header key {key}")
        index = 2 + HEADER_KEYS.index(key)
        lines[index] = f"{key}: {value}" if value else f"{key}:"
    lines.append(f"- {format_utc(stamp)} {actor}: {text}")
    ticket.path.write_text("\n".join(lines) + "\n")


def all_acceptance_checked(ticket: Ticket) -> bool:
    return bool(ticket.acceptance) and all(checked for _, checked in ticket.acceptance)


def parse_blockers_arg(value: str | None) -> list[str]:
    if not value:
        return []
    blockers: list[str] = []
    for raw in value.split(","):
        item = raw.strip()
        if not ID_RE.match(item):
            raise WorkerError(f"bad blocker id {item!r}: expected NN")
        blockers.append(item)
    return blockers


def next_ticket_id(directory: Path) -> str:
    ids = []
    for path in directory.glob("*.md"):
        match = FILE_RE.match(path.name)
        if match:
            ids.append(int(match.group(1)))
    for number in range(1, 100):
        if number not in ids:
            return f"{number:02d}"
    raise WorkerError("no ticket ids left in this effort")


def cmd_new(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    validate_slug(args.slug)
    effort = effort_dir(root, args.effort)
    if not effort.is_dir():
        raise WorkerError(f"no effort {args.effort} at {effort}")
    directory = effort / "tickets"
    directory.mkdir(exist_ok=True)
    _, tickets, problems = load_tickets(root, args.effort)
    ensure_clean(problems)
    by_id = tickets_by_id(tickets)
    ticket_id = next_ticket_id(directory)
    blockers = parse_blockers_arg(args.blocked_by)
    for blocker in blockers:
        if blocker == ticket_id:
            raise WorkerError(f"ticket {ticket_id} cannot block itself")
        if blocker not in by_id:
            raise WorkerError(f"unknown blocker {blocker}")
    path = directory / f"{ticket_id}-{args.slug}.md"
    if path.exists():
        raise WorkerError(f"{path} already exists")
    blocked_by = ", ".join(blockers)
    path.write_text(
        f"# {ticket_id} {args.slug}: {args.title}\n\n"
        f"Type: {args.type}\n"
        "Status: open\n"
        "Owner:\n"
        f"Blocked by: {blocked_by}\n"
        "Claimed:\n\n"
        "## Acceptance\n"
        "- [ ] _TODO_\n\n"
        "## Comments\n"
    )
    print(str(path.relative_to(root)))
    return OK


def cmd_claim(args: argparse.Namespace) -> int:
    if not args.owner.strip():
        raise WorkerError("owner is required")
    if args.ttl_hours <= 0:
        raise WorkerError("--ttl-hours must be positive")
    root = repo_root(args.repo)
    stamp = current_time()
    expires = stamp + timedelta(hours=args.ttl_hours)
    ticket, by_id = find_ticket(root, args.effort, args.ticket)
    if ticket.status == "resolved":
        print(f"ticket {ticket.id} is resolved", file=sys.stderr)
        return REFUSED
    if not blockers_resolved(ticket, by_id):
        print(f"ticket {ticket.id} is blocked", file=sys.stderr)
        return REFUSED
    if ticket.status == "claimed":
        expiry = claimed_until(ticket)
        if expiry is not None and stamp < expiry:
            print(f"ticket {ticket.id} is already claimed by {ticket.owner}", file=sys.stderr)
            return REFUSED
    claim_text = f"{format_utc(stamp)} expires {format_utc(expires)}"
    write_transition(
        ticket,
        {"Status": "claimed", "Owner": args.owner, "Claimed": claim_text},
        args.owner,
        f"claimed until {format_utc(expires)}",
        stamp,
    )
    print(str(ticket.path.relative_to(root)))
    return OK


def cmd_resolve(args: argparse.Namespace) -> int:
    if not args.owner.strip():
        raise WorkerError("owner is required")
    root = repo_root(args.repo)
    stamp = current_time()
    ticket, _ = find_ticket(root, args.effort, args.ticket)
    if ticket.status != "claimed":
        print(f"ticket {ticket.id} is not claimed", file=sys.stderr)
        return REFUSED
    if ticket.owner != args.owner:
        print(f"ticket {ticket.id} is claimed by {ticket.owner}", file=sys.stderr)
        return REFUSED
    if not all_acceptance_checked(ticket) and not args.force:
        print(f"ticket {ticket.id} has unchecked acceptance", file=sys.stderr)
        return REFUSED
    text = "resolved"
    if not all_acceptance_checked(ticket):
        text = "resolved with --force despite unchecked acceptance"
    write_transition(ticket, {"Status": "resolved", "Claimed": ""}, args.owner, text, stamp)
    print(str(ticket.path.relative_to(root)))
    return OK


def ticket_category(ticket: Ticket, by_id: dict[str, Ticket]) -> str:
    if ticket.status == "resolved":
        return "resolved"
    if ticket.status == "claimed":
        return "claimed"
    if not blockers_resolved(ticket, by_id):
        return "blocked"
    return "open"


def ticket_row(root: Path, ticket: Ticket, by_id: dict[str, Ticket]) -> dict:
    category = ticket_category(ticket, by_id)
    try:
        path = str(ticket.path.relative_to(root))
    except ValueError:
        path = str(ticket.path)
    return {
        "id": ticket.id,
        "slug": ticket.slug,
        "title": ticket.title,
        "type": ticket.type,
        "status": ticket.status,
        "owner": ticket.owner,
        "blocked_by": ticket.blocked_by,
        "claimed": ticket.claimed,
        "category": category,
        "path": path,
    }


def frontier_rows(root: Path, effort: str) -> list[dict]:
    _, tickets, problems = load_tickets(root, effort)
    ensure_clean(problems)
    by_id = tickets_by_id(tickets)
    order = {"open": 0, "claimed": 1, "blocked": 2, "resolved": 3}
    rows = [ticket_row(root, ticket, by_id) for ticket in tickets]
    return sorted(rows, key=lambda row: (order[row["category"]], int(row["id"])))


def cmd_list(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    rows = frontier_rows(root, args.effort)
    if args.json:
        print(json.dumps(rows, indent=2))
        return OK
    for row in rows:
        owner = row["owner"] or "-"
        blockers = ",".join(row["blocked_by"]) or "-"
        print(f"{row['id']} {row['category']} owner={owner} blockers={blockers} {row['slug']}: {row['title']}")
    return OK


def discover_efforts(root: Path) -> list[str]:
    efforts = root / "docs" / "efforts"
    if not efforts.is_dir():
        return []
    out = []
    for path in sorted(efforts.iterdir()):
        if path.is_dir() and (path / "tickets").is_dir():
            out.append(path.name)
    return out


def cmd_lint(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    efforts = args.efforts or discover_efforts(root)
    problems: list[Problem] = []
    for effort in efforts:
        try:
            _, _, effort_problems = load_tickets(root, effort)
        except WorkerError as error:
            print(f"error: {error}", file=sys.stderr)
            return USAGE
        problems.extend(effort_problems)
    for problem in sorted(problems):
        print(problem.format(root))
    return REFUSED if problems else OK


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", default=".", help="repository root (default: cwd)")
    sub = parser.add_subparsers(dest="cmd", required=True)

    def add_repo_arg(p: argparse.ArgumentParser) -> None:
        p.add_argument("--repo", default=argparse.SUPPRESS, help=argparse.SUPPRESS)

    p = sub.add_parser("new")
    p.add_argument("effort")
    p.add_argument("slug")
    p.add_argument("--title", required=True)
    p.add_argument("--type", choices=sorted(TYPES), default="task")
    p.add_argument("--blocked-by")
    add_repo_arg(p)
    p.set_defaults(fn=cmd_new)

    p = sub.add_parser("claim")
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--owner", required=True)
    p.add_argument("--ttl-hours", type=float, default=24.0)
    add_repo_arg(p)
    p.set_defaults(fn=cmd_claim)

    p = sub.add_parser("resolve")
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--owner", required=True)
    p.add_argument("--force", action="store_true")
    add_repo_arg(p)
    p.set_defaults(fn=cmd_resolve)

    p = sub.add_parser("list")
    p.add_argument("effort")
    p.add_argument("--json", action="store_true")
    add_repo_arg(p)
    p.set_defaults(fn=cmd_list)

    p = sub.add_parser("lint")
    p.add_argument("efforts", nargs="*")
    add_repo_arg(p)
    p.set_defaults(fn=cmd_lint)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        return args.fn(args)
    except WorkerError as error:
        print(f"error: {error}", file=sys.stderr)
        return USAGE


if __name__ == "__main__":
    raise SystemExit(main())
