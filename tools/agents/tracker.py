#!/usr/bin/env python3
"""Tracked effort tickets and claims.

Tickets live at ``docs/efforts/<effort>/tickets/NN-slug.md`` and use the
contract in ``docs/efforts/agentic-engineering/milestone-b/contracts``.

Commands
  new EFFORT SLUG --title T [--type feature|bug|task] [--blocked-by NN,NN]
  claim EFFORT NN --owner NAME [--ttl-hours H]
  resolve EFFORT NN --owner NAME [--force]
  add-finding EFFORT NN --to NN --text TEXT
  findings EFFORT NN [--json]
  list EFFORT [--json]
  launchable EFFORT [--json]
  question EFFORT NN --owner NAME --question TEXT
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
import land as land_tool  # noqa: E402
from worker import WorkerError, now  # noqa: E402

OK, REFUSED, USAGE = 0, 1, 64

HEADER_KEYS = ("Type", "Status", "Owner", "Blocked by", "Claimed")
TYPES = {"feature", "bug", "task"}
STATUSES = {"open", "claimed", "resolved"}
TRIAGES = {"needs-triage", "needs-info", "ready-for-agent", "ready-for-human", "wontfix"}
EFFORT_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
SLUG_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
ID_RE = re.compile(r"^\d{2}$")
FILE_RE = re.compile(r"^(\d{2})-([a-z0-9][a-z0-9-]*)\.md$")
TITLE_RE = re.compile(r"^# (\d{2}) ([a-z0-9][a-z0-9-]*): (.+)$")
TRIAGE_RE = re.compile(r"^Triage:\s*(\S+)\s*$")
STAMP_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
CLAIMED_RE = re.compile(rf"^({STAMP_RE.pattern[1:-1]}) expires ({STAMP_RE.pattern[1:-1]})$")
COMMENT_RE = re.compile(rf"^- ({STAMP_RE.pattern[1:-1]}) [^:]+: .+$")
CHECK_RE = re.compile(r"^- \[([ xX])\] .+")
LATER_FINDING_RE = re.compile(r"^- to (\d{2}): (.+)$")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")


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
class LaterFinding:
    source_kind: str
    source_id: str
    source_slug: str
    source_label: str
    target: str
    text: str
    path: Path
    line: int


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
    triage: str = ""
    triage_line: int | None = None
    claimed: str = ""
    acceptance: list[tuple[int, bool]] = field(default_factory=list)
    later_findings: list[LaterFinding] = field(default_factory=list)
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
    return land_tool.repo_root(repo or ".").resolve()


def effort_record_root(root: Path, effort: str) -> Path:
    try:
        return land_tool.resolve_effort_record(root, effort).worktree
    except WorkerError:
        if effort_dir(root, effort).is_dir():
            return root
        raise


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


def parse_later_finding_lines(
    path: Path,
    lines: list[str],
    start: int,
    end: int,
    source_kind: str,
    source_id: str,
    source_slug: str,
    source_label: str,
    problems: list[Problem],
    *,
    allow_none: bool = False,
    bullet_only: bool = False,
) -> list[LaterFinding]:
    findings: list[LaterFinding] = []
    nonempty = [(index, lines[index].strip()) for index in range(start, end) if lines[index].strip()]
    if allow_none and len(nonempty) == 1 and nonempty[0][1] == "none":
        return findings
    for index, stripped in nonempty:
        if bullet_only and not stripped.startswith("- "):
            continue
        match = LATER_FINDING_RE.match(stripped)
        if not match or not match.group(2).strip():
            problems.append(Problem(path, index + 1, "malformed later-ticket finding"))
            continue
        findings.append(
            LaterFinding(
                source_kind=source_kind,
                source_id=source_id,
                source_slug=source_slug,
                source_label=source_label,
                target=match.group(1),
                text=match.group(2).strip(),
                path=path,
                line=index + 1,
            )
        )
    return findings


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

    for index, line in enumerate(lines, start=1):
        if not line.startswith("Triage:"):
            continue
        match = TRIAGE_RE.match(line)
        if not match:
            problems.append(Problem(path, index, "malformed Triage line"))
            continue
        if ticket.triage_line is not None:
            problems.append(Problem(path, index, "duplicate Triage line"))
            continue
        ticket.triage = match.group(1)
        ticket.triage_line = index - 1
        if ticket.triage not in TRIAGES:
            problems.append(Problem(path, index, f"bad Triage {ticket.triage!r}"))

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

    findings_start, findings_end = section_bounds(lines, "## For later tickets")
    if findings_start is not None:
        ticket.later_findings.extend(parse_later_finding_lines(
            path,
            lines,
            findings_start + 1,
            findings_end or len(lines),
            "ticket",
            ticket.id,
            ticket.slug,
            f"ticket {ticket.id}-{ticket.slug}",
            problems,
        ))

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
        for finding in ticket.later_findings:
            if finding.target == ticket.id:
                problems.append(Problem(ticket.path, finding.line, f"self later-ticket target {finding.target}"))
            elif finding.target not in by_id:
                problems.append(Problem(ticket.path, finding.line, f"unknown later-ticket target {finding.target}"))

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


REPORT_RE = re.compile(r"^(\d{2})-report(?:-round-(\d+))?\.md$")
PENDING_REPORT_RE = re.compile(r"^(\d{2})-report\.md$")


def evidence_dir(root: Path, effort: str) -> Path:
    return effort_dir(root, effort) / "evidence"


def report_label(source_id: str, round_number: str | None = None, *, pending: bool = False) -> str:
    label = f"report {source_id}"
    if round_number:
        label += f" round {int(round_number)}"
    if pending:
        label += " (not yet accepted)"
    return label


def load_report_path(
    path: Path,
    *,
    source_id: str,
    round_number: str | None = None,
    pending: bool = False,
) -> tuple[list[LaterFinding], list[Problem]]:
    lines = path.read_text().splitlines()
    problems: list[Problem] = []
    start, end = section_bounds(lines, "## For later tickets")
    if start is None:
        return [], problems
    return (
        parse_later_finding_lines(
            path,
            lines,
            start + 1,
            end or len(lines),
            "report",
            source_id,
            "",
            report_label(source_id, round_number, pending=pending),
            problems,
            allow_none=True,
        ),
        problems,
    )


def effort_metadata(root: Path, effort: str) -> dict[str, str | None]:
    path = effort_dir(root, effort) / "EFFORT.md"
    if not path.is_file():
        return {"base": None, "integration_branch": None, "record_branch": None}
    return land_tool.parse_effort_metadata(path.read_text())


def code_report_ref(root: Path, integration: str) -> str | None:
    for ref in (integration, f"origin/{integration}"):
        result = land_tool.git_result(root, "rev-parse", "--verify", "-q", f"{ref}^{{commit}}")
        if result.returncode == 0 and result.stdout.strip():
            return ref
    return None


def load_report_from_ref(root: Path, ref: str, relpath: str, source_id: str) -> tuple[list[LaterFinding], list[Problem]]:
    result = land_tool.git_result(root, "show", f"{ref}:{relpath}")
    if result.returncode != 0:
        return [], []
    lines = result.stdout.splitlines()
    path = Path(f"{ref}:{relpath}")
    problems: list[Problem] = []
    start, end = section_bounds(lines, "## For later tickets")
    if start is None:
        return [], problems
    return (
        parse_later_finding_lines(
            path,
            lines,
            start + 1,
            end or len(lines),
            "report",
            source_id,
            "",
            report_label(source_id, pending=True),
            problems,
            allow_none=True,
        ),
        problems,
    )


def load_pending_code_report_findings(root: Path, effort: str, accepted_ids: set[str]) -> tuple[list[LaterFinding], list[Problem]]:
    meta = effort_metadata(root, effort)
    integration = meta.get("integration_branch")
    if not (meta.get("record_branch") and integration):
        return [], []
    findings: list[LaterFinding] = []
    problems: list[Problem] = []
    ref = code_report_ref(root, integration)
    if ref is None:
        return [], []
    listing = land_tool.git_result(root, "ls-tree", "-r", "--name-only", ref, f"docs/efforts/{effort}/evidence")
    if listing.returncode != 0:
        return [], []
    for relpath in sorted(line for line in listing.stdout.splitlines() if line.strip()):
        match = PENDING_REPORT_RE.match(Path(relpath).name)
        if not match or match.group(1) in accepted_ids:
            continue
        report_findings, report_problems = load_report_from_ref(root, ref, relpath, match.group(1))
        findings.extend(report_findings)
        problems.extend(report_problems)
    return findings, problems


def load_report_findings(root: Path, effort: str, *, include_unaccepted_reports: bool = False) -> tuple[list[LaterFinding], list[Problem]]:
    directory = evidence_dir(root, effort)
    findings: list[LaterFinding] = []
    problems: list[Problem] = []
    accepted_ids: set[str] = set()
    if directory.is_dir():
        for path in sorted(directory.glob("[0-9][0-9]-report*.md")):
            match = REPORT_RE.match(path.name)
            if not match:
                continue
            source_id, round_number = match.groups()
            accepted_ids.add(source_id)
            report_findings, report_problems = load_report_path(path, source_id=source_id, round_number=round_number)
            findings.extend(report_findings)
            problems.extend(report_problems)
    if include_unaccepted_reports:
        report_findings, report_problems = load_pending_code_report_findings(root, effort, accepted_ids)
        findings.extend(report_findings)
        problems.extend(report_problems)
    return findings, problems


def load_lead_handoff_findings(root: Path, effort: str) -> tuple[list[LaterFinding], list[Problem]]:
    path = evidence_dir(root, effort) / "lead-handoffs.md"
    if not path.is_file():
        return [], []
    lines = path.read_text().splitlines()
    problems: list[Problem] = []
    findings = parse_later_finding_lines(
        path,
        lines,
        0,
        len(lines),
        "lead",
        "lead",
        "",
        "lead",
        problems,
        bullet_only=True,
    )
    return findings, problems


def load_later_findings(root: Path, effort: str, *, include_unaccepted_reports: bool = False) -> tuple[list[Ticket], list[LaterFinding], list[Problem]]:
    _, tickets, problems = load_tickets(root, effort)
    findings = [finding for ticket in tickets for finding in ticket.later_findings]
    report_findings, report_problems = load_report_findings(root, effort, include_unaccepted_reports=include_unaccepted_reports)
    lead_findings, lead_problems = load_lead_handoff_findings(root, effort)
    findings.extend(report_findings)
    findings.extend(lead_findings)
    problems.extend(report_problems)
    problems.extend(lead_problems)
    by_id = tickets_by_id(tickets)
    for finding in findings:
        if finding.source_kind == "ticket":
            continue
        if finding.source_kind == "report" and finding.target == finding.source_id:
            problems.append(Problem(finding.path, finding.line, f"self later-ticket target {finding.target}"))
        if finding.target not in by_id:
            problems.append(Problem(finding.path, finding.line, f"unknown later-ticket target {finding.target}"))
    return tickets, findings, sorted(problems)


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


def load_jsonl(path: Path) -> list[dict]:
    if not path.is_file():
        return []
    records = []
    for line_no, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        try:
            records.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise WorkerError(f"{path}:{line_no}: bad JSON: {error}") from error
    return records


def valid_accepted_candidate(record: dict, effort: str, ticket: str) -> str | None:
    candidate = record.get("candidate")
    axes = record.get("axes")
    checks = record.get("required_checks")
    if (
        record.get("schema") != "corenius.acceptance.v1"
        or record.get("effort") != effort
        or record.get("ticket") != ticket
        or record.get("accepted") is not True
        or not isinstance(candidate, str)
        or not SHA_RE.match(candidate)
        or not isinstance(axes, dict)
        or axes.get("standards") != "passed"
        or axes.get("spec") != "passed"
        or not isinstance(checks, list)
        or not checks
    ):
        return None
    for check in checks:
        if not isinstance(check, dict) or check.get("result") not in {"passed", "failed"} or not check.get("name"):
            return None
        if check.get("result") == "failed":
            waiver = record.get("owner_waiver")
            if not isinstance(waiver, dict) or not waiver.get("owner") or not waiver.get("reason"):
                return None
    return candidate


def accepted_candidates(root: Path, effort: str, ticket: str) -> list[str]:
    path = evidence_dir(root, effort) / f"acceptance-{ticket}.jsonl"
    candidates = []
    for record in load_jsonl(path):
        candidate = valid_accepted_candidate(record, effort, ticket)
        if candidate:
            candidates.append(candidate)
    return candidates


def landing_record_candidates(root: Path, effort: str, ticket: str) -> set[str]:
    path = evidence_dir(root, effort) / f"landing-{ticket}.jsonl"
    candidates = set()
    for record in load_jsonl(path):
        candidate = record.get("candidate")
        if (
            record.get("schema") == "corenius.ticket_landing.v1"
            and record.get("effort") == effort
            and record.get("ticket") == ticket
            and isinstance(candidate, str)
            and SHA_RE.match(candidate)
        ):
            candidates.add(candidate)
    return candidates


def candidate_in_integration_history(root: Path, effort: str, candidate: str) -> bool:
    integration = effort_metadata(root, effort).get("integration_branch")
    if not integration:
        return False
    result = land_tool.git_result(root, "merge-base", "--is-ancestor", candidate, integration)
    return result.returncode == 0


def has_accepted_landed_candidate(root: Path, effort: str, ticket: Ticket) -> bool:
    landed = landing_record_candidates(root, effort, ticket.id)
    for candidate in accepted_candidates(root, effort, ticket.id):
        if candidate in landed or candidate_in_integration_history(root, effort, candidate):
            return True
    return False


def blockers_resolved(root: Path, effort: str, ticket: Ticket, by_id: dict[str, Ticket]) -> bool:
    return all(
        by_id[blocker].status == "resolved" and has_accepted_landed_candidate(root, effort, by_id[blocker])
        for blocker in ticket.blocked_by
    )


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


def write_triage_comment(ticket: Ticket, triage: str, actor: str, text: str, stamp: datetime) -> None:
    if triage not in TRIAGES:
        raise WorkerError(f"bad triage {triage!r}")
    if ticket.comments_line is None:
        raise WorkerError(f"{ticket.path} has no Comments section")
    lines = list(ticket.lines)
    if ticket.triage_line is None:
        insert = 7
        while insert < len(lines) and lines[insert] == "":
            insert += 1
        lines[insert:insert] = [f"Triage: {triage}", ""]
    else:
        lines[ticket.triage_line] = f"Triage: {triage}"
    lines.append(f"- {format_utc(stamp)} {actor}: {text}")
    ticket.path.write_text("\n".join(lines) + "\n")


def all_acceptance_checked(ticket: Ticket) -> bool:
    return bool(ticket.acceptance) and all(checked for _, checked in ticket.acceptance)


def finding_source_row(root: Path, finding: LaterFinding) -> dict:
    try:
        path = str(finding.path.relative_to(root))
    except ValueError:
        path = str(finding.path)
    return {
        "kind": finding.source_kind,
        "id": finding.source_id,
        "slug": finding.source_slug,
        "label": finding.source_label,
        "path": path,
        "line": finding.line,
    }


def collect_findings_addressed_to(root: Path, findings: list[LaterFinding], target: str) -> list[dict]:
    rows: list[dict] = []
    by_key: dict[tuple[str, str], dict] = {}
    for finding in findings:
        if finding.target != target:
            continue
        key = (finding.target, finding.text)
        row = by_key.get(key)
        if row is None:
            row = {"target": finding.target, "text": finding.text, "sources": []}
            by_key[key] = row
            rows.append(row)
        row["sources"].append(finding_source_row(root, finding))
    return rows


def findings_addressed_to(root: Path, effort: str, target: str) -> list[dict]:
    if not ID_RE.match(target):
        raise WorkerError(f"bad ticket id {target!r}: expected NN")
    tickets, findings, problems = load_later_findings(root, effort, include_unaccepted_reports=True)
    ensure_clean(problems)
    by_id = tickets_by_id(tickets)
    if target not in by_id:
        raise WorkerError(f"ticket {target} not found in effort {effort}")
    return collect_findings_addressed_to(root, findings, target)


def append_later_finding(ticket: Ticket, target: str, text: str) -> None:
    lines = list(ticket.lines)
    start, end = section_bounds(lines, "## For later tickets")
    if start is None:
        if ticket.comments_line is None:
            raise WorkerError(f"{ticket.path} has no Comments section")
        insert = ticket.comments_line
        section = []
        if insert > 0 and lines[insert - 1]:
            section.append("")
        section.extend(["## For later tickets", f"- to {target}: {text}", ""])
        lines[insert:insert] = section
    else:
        insert = end or len(lines)
        if insert > start + 1 and lines[insert - 1] == "":
            insert -= 1
        lines.insert(insert, f"- to {target}: {text}")
    ticket.path.write_text("\n".join(lines) + "\n")


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
    root = effort_record_root(repo_root(args.repo), args.effort)
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
        "Triage: needs-triage\n\n"
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
    root = effort_record_root(repo_root(args.repo), args.effort)
    stamp = current_time()
    expires = stamp + timedelta(hours=args.ttl_hours)
    ticket, by_id = find_ticket(root, args.effort, args.ticket)
    if ticket.status == "resolved":
        print(f"ticket {ticket.id} is resolved", file=sys.stderr)
        return REFUSED
    if not blockers_resolved(root, args.effort, ticket, by_id):
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
    root = effort_record_root(repo_root(args.repo), args.effort)
    stamp = current_time()
    ticket, _ = find_ticket(root, args.effort, args.ticket)
    if ticket.status != "claimed":
        print(f"ticket {ticket.id} is not claimed", file=sys.stderr)
        return REFUSED
    if ticket.owner != args.owner:
        print(f"ticket {ticket.id} is claimed by {ticket.owner}", file=sys.stderr)
        return REFUSED
    missing_landed = not has_accepted_landed_candidate(root, args.effort, ticket)
    unchecked_acceptance = not all_acceptance_checked(ticket)
    if unchecked_acceptance and getattr(args, "mark_reviewed", False):
        if missing_landed:
            print("cannot mark acceptance without accepted, landed evidence", file=sys.stderr)
            return REFUSED
        lines = list(ticket.lines)
        for line_number, _ in ticket.acceptance:
            lines[line_number - 1] = re.sub(r"^- \[[ ]\]", "- [x]", lines[line_number - 1])
        ticket.path.write_text("\n".join(lines) + "\n")
        ticket, _ = find_ticket(root, args.effort, args.ticket)
        unchecked_acceptance = not all_acceptance_checked(ticket)
    if unchecked_acceptance and not args.force:
        print(f"ticket {ticket.id} has unchecked acceptance", file=sys.stderr)
        return REFUSED
    if missing_landed and not args.force:
        print(f"ticket {ticket.id} has no accepted, landed Candidate", file=sys.stderr)
        return REFUSED
    text = "resolved"
    forced_reasons = []
    if unchecked_acceptance:
        forced_reasons.append("unchecked acceptance")
    if missing_landed:
        forced_reasons.append("missing accepted, landed Candidate")
    if forced_reasons:
        text = f"resolved with --force despite {' and '.join(forced_reasons)}"
    write_transition(ticket, {"Status": "resolved", "Claimed": ""}, args.owner, text, stamp)
    print(str(ticket.path.relative_to(root)))
    return OK


def cmd_add_finding(args: argparse.Namespace) -> int:
    root = effort_record_root(repo_root(args.repo), args.effort)
    text = " ".join(args.text.split())
    if not text:
        raise WorkerError("finding text is required")
    ticket, by_id = find_ticket(root, args.effort, args.ticket)
    if not ID_RE.match(args.target):
        raise WorkerError(f"bad target ticket id {args.target!r}: expected NN")
    if args.target == ticket.id:
        raise WorkerError(f"ticket {ticket.id} cannot leave a finding to itself")
    if args.target not in by_id:
        raise WorkerError(f"unknown target ticket {args.target}")
    append_later_finding(ticket, args.target, text)
    print(str(ticket.path.relative_to(root)))
    return OK


def cmd_findings(args: argparse.Namespace) -> int:
    root = effort_record_root(repo_root(args.repo), args.effort)
    rows = findings_addressed_to(root, args.effort, args.ticket)
    if args.json:
        print(json.dumps(rows, indent=2))
        return OK
    for row in rows:
        sources = ", ".join(source["label"] for source in row["sources"])
        print(f"{sources}: {row['text']}")
    return OK


def ticket_category(root: Path, effort: str, ticket: Ticket, by_id: dict[str, Ticket]) -> str:
    if ticket.status == "resolved":
        return "resolved"
    if ticket.status == "claimed":
        return "claimed"
    if not blockers_resolved(root, effort, ticket, by_id):
        return "blocked"
    return "open"


def ticket_row(root: Path, effort: str, ticket: Ticket, by_id: dict[str, Ticket]) -> dict:
    category = ticket_category(root, effort, ticket, by_id)
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
        "triage": ticket.triage,
        "claimed": ticket.claimed,
        "category": category,
        "path": path,
    }


def frontier_rows(root: Path, effort: str) -> list[dict]:
    _, tickets, problems = load_tickets(root, effort)
    ensure_clean(problems)
    by_id = tickets_by_id(tickets)
    order = {"open": 0, "claimed": 1, "blocked": 2, "resolved": 3}
    rows = [ticket_row(root, effort, ticket, by_id) for ticket in tickets]
    return sorted(rows, key=lambda row: (order[row["category"]], int(row["id"])))


def cmd_list(args: argparse.Namespace) -> int:
    root = effort_record_root(repo_root(args.repo), args.effort)
    rows = frontier_rows(root, args.effort)
    if args.json:
        print(json.dumps(rows, indent=2))
        return OK
    for row in rows:
        owner = row["owner"] or "-"
        blockers = ",".join(row["blocked_by"]) or "-"
        triage = row["triage"] or "-"
        print(f"{row['id']} {row['category']} owner={owner} triage={triage} blockers={blockers} {row['slug']}: {row['title']}")
    return OK


def launchable_rows(root: Path, effort: str) -> list[dict]:
    _, tickets, problems = load_tickets(root, effort)
    ensure_clean(problems)
    by_id = tickets_by_id(tickets)
    rows = [
        ticket_row(root, effort, ticket, by_id)
        for ticket in tickets
        if ticket.status == "open" and blockers_resolved(root, effort, ticket, by_id) and ticket.triage == "ready-for-agent"
    ]
    return sorted(rows, key=lambda row: int(row["id"]))


def cmd_launchable(args: argparse.Namespace) -> int:
    root = effort_record_root(repo_root(args.repo), args.effort)
    rows = launchable_rows(root, args.effort)
    if args.json:
        print(json.dumps(rows, indent=2))
        return OK
    for row in rows:
        print(f"{row['id']} {row['triage']} {row['slug']}: {row['title']}")
    return OK


def cmd_question(args: argparse.Namespace) -> int:
    if not args.owner.strip():
        raise WorkerError("owner is required")
    question = " ".join(args.question.split())
    if not question:
        raise WorkerError("question is required")
    root = effort_record_root(repo_root(args.repo), args.effort)
    stamp = current_time()
    ticket, _ = find_ticket(root, args.effort, args.ticket)
    write_triage_comment(ticket, "needs-info", args.owner, f"question: {question}", stamp)
    print(str(ticket.path.relative_to(root)))
    return OK


def discover_efforts(root: Path) -> list[tuple[str, Path]]:
    discovered: dict[str, Path] = {}
    local_efforts = root / "docs" / "efforts"
    if local_efforts.is_dir():
        for path in sorted(local_efforts.iterdir()):
            if path.is_dir() and (path / "tickets").is_dir():
                discovered[path.name] = root
    candidates, _ = land_tool.scan_effort_records(root)
    for effort in sorted(candidates):
        try:
            record = land_tool.select_effort_record(root, effort, candidates[effort])
        except WorkerError as error:
            if "not in this checkout" in str(error):
                continue
            raise
        if record.worktree != root.resolve() and not record.metadata.get("record_branch"):
            continue
        if (record.worktree / "docs" / "efforts" / effort / "tickets").is_dir():
            discovered.setdefault(effort, record.worktree)
    return sorted(discovered.items())


def cmd_lint(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    efforts = (
        [(effort, effort_record_root(root, effort)) for effort in args.efforts]
        if args.efforts
        else discover_efforts(root)
    )
    problems: list[Problem] = []
    for effort, effort_root in efforts:
        try:
            _, _, effort_problems = load_later_findings(effort_root, effort)
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
    p.add_argument("--mark-reviewed", action="store_true",
                   help="operator attests all criteria reviewed against accepted, landed evidence")
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--owner", required=True)
    p.add_argument("--force", action="store_true")
    add_repo_arg(p)
    p.set_defaults(fn=cmd_resolve)

    p = sub.add_parser("add-finding")
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--to", dest="target", required=True)
    p.add_argument("--text", required=True)
    add_repo_arg(p)
    p.set_defaults(fn=cmd_add_finding)

    p = sub.add_parser("findings")
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--json", action="store_true")
    add_repo_arg(p)
    p.set_defaults(fn=cmd_findings)

    p = sub.add_parser("list")
    p.add_argument("effort")
    p.add_argument("--json", action="store_true")
    add_repo_arg(p)
    p.set_defaults(fn=cmd_list)

    p = sub.add_parser("launchable")
    p.add_argument("effort")
    p.add_argument("--json", action="store_true")
    add_repo_arg(p)
    p.set_defaults(fn=cmd_launchable)

    p = sub.add_parser("question")
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--owner", required=True)
    p.add_argument("--question", required=True)
    add_repo_arg(p)
    p.set_defaults(fn=cmd_question)

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
