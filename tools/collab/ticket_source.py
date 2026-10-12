"""Committed ticket source adapter for standalone collab tickets."""
from __future__ import annotations

import re

from common import Refused, digest, valid_id


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


class TicketSource:
    def __init__(self, workspace):
        self.workspace = workspace

    def ticket(self, tid: str, commit: str | None = None) -> tuple[dict, bytes]:
        valid_id(tid)
        revision = commit or self.workspace.trunk()
        path = ".tickets/" + tid + ".md"
        entry = self.workspace.git("ls-tree", revision, "--", path).strip()
        if not entry or not entry.startswith("100644 blob "):
            raise Refused(f"Missing/non-regular committed ticket: {tid}")
        raw = self.workspace.git("show", revision + ":" + path, raw=True)
        return parse_ticket(raw, tid), raw

    def deps_ok(self, tid: str, commit: str) -> None:
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

    def same_contract(self, attempt: dict, commit: str) -> None:
        meta, _ = self.ticket(attempt["ticket"], commit)
        if meta["sha256"] != attempt["ticket_hash"]:
            raise Refused("Ticket changed since claim; explicitly abandon/re-admit instead")
        self.deps_ok(attempt["ticket"], commit)
