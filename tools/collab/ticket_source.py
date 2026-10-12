"""One ticket contract for standalone headers and Corenius tracker tickets.

Both layouts read committed bytes, validate dependencies at an explicit commit,
and enumerate their configured source. Corenius claims and resolution belong to
tracker.py; this adapter never writes tracker files or maintains a second ledger.
Qualified effort:number identities select Corenius for explicit legacy callers.
"""
from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess
import sys

from common import Refused, child_environment, digest, valid_id

TRACKER = Path(__file__).resolve().parents[1] / "agents" / "tracker.py"
TRACKER_TITLE = re.compile(r"^# (?P<id>\d{2}) (?P<slug>[a-z0-9][a-z0-9-]*): (?P<title>.+)$")
TRACKER_FIELD = re.compile(r"^(?P<key>Type|Status|Owner|Blocked by|Claimed):\s*(?P<value>.*)$")
TRACKER_STATUSES = {"open", "claimed", "resolved"}


def ticket_sections(raw: bytes) -> tuple[bytes, bytes, bytes]:
    delimiters = re.finditer(rb"(?m)^---[ \t]*\r?$", raw)
    opening = next(delimiters, None)
    closing = next(delimiters, None)
    if opening is None or closing is None or raw[:opening.start()].strip():
        raise Refused("Ticket needs standalone --- front matter delimiters")
    return raw[:opening.end()], raw[opening.end():closing.start()], raw[closing.start():]


def parse_ticket(raw: bytes, expected_id: str) -> dict:
    """Read the deliberately narrow tk-compatible header subset.

    Required: id, status, deps (inline list). Unknown scalar fields are allowed.
    This is NOT a general YAML parser; multiline/block metadata is refused.
    """
    try:
        raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise Refused("Ticket must be UTF-8") from exc
    _, header, body = ticket_sections(raw)
    fields = {}
    for line in header.decode("utf-8").strip().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        if line[:1].isspace() or ":" not in line:
            raise Refused("Only scalar/inline-list front matter is supported")
        key, value = line.split(":", 1)
        if key in fields or not re.fullmatch(r"[a-z][a-z_-]*", key):
            raise Refused("Duplicate or unsupported metadata key")
        value = value.strip()
        if value in ("|", ">", "|-", ">-"):
            raise Refused("Multiline metadata is unsupported")
        fields[key] = value
    ticket_id = fields.get("id", "").strip("\"'")
    status = fields.get("status", "").strip("\"'")
    if ticket_id != expected_id:
        raise Refused("Ticket filename/id mismatch")
    if status not in ("open", "in_progress", "closed"):
        raise Refused("Ticket status must be open, in_progress, or closed")
    deps = fields.get("deps", "")
    if not deps.startswith("[") or not deps.endswith("]"):
        raise Refused("deps must be an inline list, e.g. [T-1, T-2]")
    deps = [valid_id(item.strip().strip("\"'")) for item in deps[1:-1].split(",") if item.strip()]
    if len(deps) != len(set(deps)) or expected_id in deps:
        raise Refused("Duplicate or self dependency")
    title = next(
        (line[2:].strip() for line in body.decode("utf-8").splitlines() if line.startswith("# ")),
        expected_id,
    )
    return {
        "id": ticket_id,
        "status": status,
        "deps": deps,
        "title": title,
        "sha256": digest(raw),
    }


def split_corenius_ticket(ticket: str) -> tuple[str, str] | None:
    if ":" not in ticket:
        return None
    effort, ticket_id = ticket.split(":", 1)
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]*", effort):
        raise Refused(f"Invalid effort: {effort!r}")
    if not re.fullmatch(r"\d{2}", ticket_id):
        raise Refused(f"Invalid tracker ticket id: {ticket_id!r}")
    return effort, ticket_id


def corenius_ticket_id(effort: str, ticket_id: str) -> str:
    split_corenius_ticket(f"{effort}:{ticket_id}")
    return f"{effort}:{ticket_id}"


def parse_corenius_ticket(raw: bytes, effort: str, expected_id: str, relpath: str) -> dict:
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise Refused("Tracker ticket must be UTF-8") from exc
    lines = text.splitlines()
    if not lines:
        raise Refused("Tracker ticket is empty")
    match = TRACKER_TITLE.match(lines[0])
    if not match:
        raise Refused("Tracker ticket title is malformed")
    if match.group("id") != expected_id:
        raise Refused("Tracker ticket filename/id mismatch")
    fields = {}
    for line in lines[2:7]:
        field = TRACKER_FIELD.match(line)
        if field:
            fields[field.group("key")] = field.group("value").strip()
    status = fields.get("Status", "")
    if status not in TRACKER_STATUSES:
        raise Refused("Tracker ticket status must be open, claimed, or resolved")
    blocked = [item.strip() for item in fields.get("Blocked by", "").split(",") if item.strip()]
    for blocker in blocked:
        if not re.fullmatch(r"\d{2}", blocker):
            raise Refused("Tracker ticket has an invalid blocker")
    return {
        "id": corenius_ticket_id(effort, expected_id),
        "number": expected_id,
        "slug": match.group("slug"),
        "title": match.group("title"),
        "status": status,
        "owner": fields.get("Owner", ""),
        "blocked_by": blocked,
        "path": relpath,
        "sha256": digest(raw),
        "source": "corenius",
        "effort": effort,
    }


class TicketSource:
    def __init__(self, workspace):
        self.workspace = workspace

    def list_tickets(self, commit: str) -> list[dict]:
        """Enumerate the configured source, retaining malformed tickets as errors."""
        source = self.workspace.require_config().get("ticket_source", "standalone")
        directory = "docs/efforts/" if source == "corenius" else ".tickets/"
        paths = self.workspace.git("ls-tree", "-r", "--name-only", commit, "--", directory).splitlines()
        identities = []
        for path in paths:
            if source == "corenius":
                match = re.fullmatch(r"docs/efforts/([a-z0-9][a-z0-9-]*)/tickets/(\d{2})-[^/]+\.md", path)
                if match:
                    identities.append(corenius_ticket_id(*match.groups()))
            elif path.count("/") == 1 and path.endswith(".md"):
                identities.append(Path(path).stem)
        rows = []
        for identity in sorted(set(identities)):
            try:
                meta, _ = self.ticket(identity, commit)
                rows.append(meta)
            except Refused as exc:
                rows.append({"id": identity, "error": str(exc)})
        return rows

    def require_ready(self, meta: dict, commit: str) -> None:
        """Use the selected source's authoritative admission rules and blockers."""
        self.deps_ok(meta["id"], commit)
        corenius = split_corenius_ticket(meta["id"])
        if corenius:
            self.require_corenius_ready(*corenius)
        elif meta["status"] == "closed":
            raise Refused("Ticket is closed")

    def require_refresh_claim(self, attempt: dict, commit: str) -> None:
        """Transfer an unchanged Corenius claim only to the same stopped owner."""
        self.same_contract(attempt, commit)
        corenius = split_corenius_ticket(attempt["ticket"])
        if not corenius:
            return
        meta, _ = self.ticket(attempt["ticket"], commit)
        row = self.tracker_row(*corenius)
        if any(record.get("status") != "claimed" or record.get("owner") != attempt["owner"]
               for record in (meta, row)):
            raise Refused("Tracker claim no longer belongs to this Attempt's owner; explicitly re-admit")

    def ticket(self, tid: str, commit: str | None = None) -> tuple[dict, bytes]:
        corenius = split_corenius_ticket(tid)
        if corenius:
            return self.corenius_ticket(corenius[0], corenius[1], commit)
        valid_id(tid)
        revision = commit or self.workspace.trunk()
        path = ".tickets/" + tid + ".md"
        entry = self.workspace.git("ls-tree", revision, "--", path).strip()
        if not entry or not entry.startswith("100644 blob "):
            raise Refused(f"Missing/non-regular committed ticket: {tid}")
        raw = self.workspace.git("show", revision + ":" + path, raw=True)
        return parse_ticket(raw, tid), raw

    def corenius_ticket(self, effort: str, ticket_id: str, commit: str | None = None) -> tuple[dict, bytes]:
        corenius_ticket_id(effort, ticket_id)
        revision = commit or self.workspace.trunk()
        directory = f"docs/efforts/{effort}/tickets"
        paths = self.workspace.git("ls-tree", "-r", "--name-only", revision, "--", directory).splitlines()
        prefix = f"{directory}/{ticket_id}-"
        matches = [path for path in paths if path.startswith(prefix) and path.endswith(".md")]
        if len(matches) != 1:
            raise Refused(f"Expected one committed tracker ticket for {effort}:{ticket_id}")
        relpath = matches[0]
        entry = self.workspace.git("ls-tree", revision, "--", relpath).strip()
        if not entry or not entry.startswith("100644 blob "):
            raise Refused(f"Missing/non-regular committed tracker ticket: {effort}:{ticket_id}")
        raw = self.workspace.git("show", revision + ":" + relpath, raw=True)
        return parse_corenius_ticket(raw, effort, ticket_id, relpath), raw

    def tracker_rows(self, effort: str) -> list[dict]:
        self.require_in_tree_effort(effort)
        root = Path(self.workspace.require_config()["root"])
        command = [sys.executable, str(TRACKER), "--repo", str(root), "list", effort, "--json"]
        process = subprocess.run(command, cwd=root, text=True, capture_output=True, env=child_environment())
        if process.returncode:
            raise Refused(process.stderr.strip() or process.stdout.strip() or "tracker list failed")
        return json.loads(process.stdout)

    def require_in_tree_effort(self, effort: str) -> None:
        """Refuse legacy routing before tracker.py can select another checkout."""
        config = self.workspace.require_config()
        revision = self.workspace.trunk()
        path = f"docs/efforts/{effort}/EFFORT.md"
        raw = self.workspace.git("show", f"{revision}:{path}")
        fields = {}
        for line in raw.splitlines():
            if line.startswith("Integration branch:"):
                fields["integration"] = line.partition(":")[2].strip().strip("`")
            elif line.startswith("Record branch:"):
                fields["record"] = line.partition(":")[2].strip().strip("`")
        integration = fields.get("integration")
        trunk = config["trunk_ref"].removeprefix("refs/heads/")
        if fields.get("record") or (integration and integration.removeprefix("refs/heads/") != trunk):
            raise Refused("Legacy effort routing requires the optional effort-record module; no tracker writes made")

    def tracker_row(self, effort: str, ticket_id: str) -> dict:
        for row in self.tracker_rows(effort):
            if row.get("id") == ticket_id:
                return row
        raise Refused(f"tracker ticket {effort}:{ticket_id} not found")

    def require_corenius_ready(self, effort: str, ticket_id: str) -> None:
        row = self.tracker_row(effort, ticket_id)
        category = row.get("category")
        if category != "open" or row.get("status") != "open":
            raise Refused(f"tracker ticket {ticket_id} is {category or row.get('status')}")
        if row.get("triage") != "ready-for-agent":
            raise Refused(f"tracker ticket {ticket_id} is not ready-for-agent")

    def claim_corenius(self, effort: str, ticket_id: str, owner: str, ttl_hours: float) -> tuple[dict, bytes]:
        self.require_in_tree_effort(effort)
        root = Path(self.workspace.require_config()["root"])
        command = [
            sys.executable,
            str(TRACKER),
            "--repo",
            str(root),
            "claim",
            effort,
            ticket_id,
            "--owner",
            owner,
            "--ttl-hours",
            str(ttl_hours),
        ]
        process = subprocess.run(command, cwd=root, text=True, capture_output=True, env=child_environment())
        if process.returncode:
            raise Refused(process.stderr.strip() or process.stdout.strip() or "tracker claim failed")
        relpath = process.stdout.strip()
        if not relpath:
            raise Refused("tracker claim did not report a ticket path")
        self.workspace.git("add", "--", relpath, cwd=root)
        self.workspace.git("commit", "-q", "-m", f"Claim ticket {ticket_id} for {owner}", cwd=root)
        return self.corenius_ticket(effort, ticket_id)

    def deps_ok(self, tid: str, commit: str) -> None:
        corenius = split_corenius_ticket(tid)
        if corenius:
            self.corenius_blockers_ok(corenius[0], corenius[1], commit)
            return
        visiting = set()
        done = set()

        def visit(ticket_id: str) -> None:
            if ticket_id in visiting:
                raise Refused("Dependency cycle")
            if ticket_id in done:
                return
            visiting.add(ticket_id)
            meta, _ = self.ticket(ticket_id, commit)
            for parent in meta["deps"]:
                visit(parent)
            visiting.remove(ticket_id)
            done.add(ticket_id)

        visit(tid)
        meta, _ = self.ticket(tid, commit)
        for parent in meta["deps"]:
            parent_meta, _ = self.ticket(parent, commit)
            if parent_meta["status"] != "closed":
                raise Refused(f"Dependency not closed on trunk: {parent}")

    def corenius_blockers_ok(self, effort: str, ticket_id: str, commit: str) -> None:
        meta, _ = self.corenius_ticket(effort, ticket_id, commit)
        for blocker in meta["blocked_by"]:
            parent, _ = self.corenius_ticket(effort, blocker, commit)
            if parent["status"] != "resolved":
                raise Refused(f"Tracker blocker not resolved on trunk: {blocker}")

    def same_contract(self, attempt: dict, commit: str) -> None:
        meta, _ = self.ticket(attempt["ticket"], commit)
        if meta["sha256"] != attempt["ticket_hash"]:
            raise Refused("Ticket changed since claim; explicitly abandon/re-admit instead")
        self.deps_ok(attempt["ticket"], commit)
