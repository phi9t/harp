#!/usr/bin/env python3
"""Sample verification-report claims and record lead check outcomes.

Commands
  sample FILE --n N [--seed S] [--tags R,X]
      parse tagged report lines, choose a deterministic sample, and print one
      JSON object per claim with the verification command or UNCITED status.
  record FILE --claim ID (--passed | --failed) [--output FILE] [--evidence DIGEST]
      append a local outcome under .corenius/claims/ and optionally append it
      to an evidence record.
  summary FILE
      print counts and latest local outcomes for the report's claims.

Standard library only. Evidence integration is deliberately lazy because F9 and
F10 are implemented by separate workers in Milestone B.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import re
import shlex
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from worker import WorkerError, git, main_worktree, now  # noqa: E402,F401

OK, ERROR = 0, 64
ID_LEN = 12
CLAIM_RE = re.compile(r"^\s*(?:[-*+]\s+|\d+[.)]\s+)\[(?P<tag>[RXI])\]\s+(?P<body>.+?)\s*$")
BACKTICK_RE = re.compile(r"`([^`]+)`")
PATH_LINE_RE = re.compile(r"^(?P<path>.+):(?P<start>[1-9]\d*)(?:-(?P<end>[1-9]\d*))?$")
VALID_TAGS = {"R", "X", "I"}


@dataclass(frozen=True)
class Claim:
    claim_id: str
    tag: str
    line: int
    text: str
    citation: str | None
    command: str | None

    @property
    def status(self) -> str:
        return "CITED" if self.command else "UNCITED"


def file_digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return "sha256:" + h.hexdigest()


def normalize_claim(tag: str, body: str) -> str:
    return re.sub(r"\s+", " ", f"[{tag}] {body.strip()}").strip()


def claim_id(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:ID_LEN]


def is_git_show_command(text: str) -> bool:
    try:
        words = shlex.split(text)
    except ValueError:
        return False
    if not words or words[0] != "git":
        return False
    try:
        show_at = words.index("show")
    except ValueError:
        return False
    return any(":" in word for word in words[show_at + 1 :])


def path_line_command(text: str) -> str | None:
    match = PATH_LINE_RE.match(text.strip())
    if not match:
        return None
    path = match.group("path")
    if not path or path.startswith(":"):
        return None
    start = int(match.group("start"))
    end = int(match.group("end") or start)
    return f"sed -n '{start},{end}p' {shlex.quote(path)}"


def verification_command(body: str) -> tuple[str | None, str | None]:
    citations = [c.strip() for c in BACKTICK_RE.findall(body)]
    for citation in citations:
        if is_git_show_command(citation):
            return citation, citation
    for citation in citations:
        command = path_line_command(citation)
        if command:
            return citation, command
    return None, None


def parse_claims(path: str | Path, tags: set[str] | None = None) -> list[Claim]:
    claims: list[Claim] = []
    for lineno, line in enumerate(Path(path).read_text().splitlines(), 1):
        match = CLAIM_RE.match(line)
        if not match:
            continue
        tag = match.group("tag")
        if tags is not None and tag not in tags:
            continue
        body = match.group("body")
        text = normalize_claim(tag, body)
        citation, command = verification_command(body)
        claims.append(
            Claim(
                claim_id=claim_id(text),
                tag=tag,
                line=lineno,
                text=text,
                citation=citation,
                command=command,
            )
        )
    return claims


def parse_tags(raw: str | None) -> set[str] | None:
    if not raw:
        return None
    tags = {part.strip().strip("[]") for part in raw.split(",") if part.strip()}
    bad = sorted(tags - VALID_TAGS)
    if bad:
        raise WorkerError(f"unknown tag(s): {', '.join(bad)}")
    return tags


def sample_claims(path: Path, claims: list[Claim], n: int, seed: str | None) -> list[Claim]:
    if n < 0:
        raise WorkerError("--n must be non-negative")
    if n == 0 or not claims:
        return []
    material = path.read_bytes()
    report_digest = hashlib.sha256(material).hexdigest()
    seed_text = "" if seed is None else str(seed)

    def key(claim: Claim) -> str:
        data = f"{seed_text}\0{report_digest}\0{claim.claim_id}".encode("utf-8")
        return hashlib.sha256(data).hexdigest()

    return sorted(claims, key=key)[: min(n, len(claims))]


def claim_json(claim: Claim) -> dict:
    return {
        "id": claim.claim_id,
        "tag": claim.tag,
        "line": claim.line,
        "text": claim.text,
        "citation": claim.citation,
        "command": claim.command,
        "status": claim.status,
    }


def safe_stem(path: Path) -> str:
    stem = re.sub(r"[^A-Za-z0-9._-]+", "-", path.stem).strip("-")
    return stem or "report"


def claims_root(report: Path, create: bool) -> Path:
    try:
        root = main_worktree(report.parent)
    except WorkerError as error:
        raise WorkerError(f"cannot locate report worktree for .corenius/claims: {error}") from error
    out = root / ".corenius" / "claims"
    if create:
        out.mkdir(parents=True, exist_ok=True)
    return out


def outcome_path(report: Path, create: bool = False) -> Path:
    resolved = report.resolve()
    key = hashlib.sha256(str(resolved).encode("utf-8")).hexdigest()[:16]
    return claims_root(resolved, create) / f"{safe_stem(resolved)}-{key}.jsonl"


def load_outcomes(report: Path) -> list[dict]:
    path = outcome_path(report)
    if not path.exists():
        return []
    outcomes = []
    for line in path.read_text().splitlines():
        if line.strip():
            outcomes.append(json.loads(line))
    return outcomes


def load_evidence_module():
    try:
        return importlib.import_module("evidence")
    except ModuleNotFoundError as error:
        raise WorkerError("evidence module is required when --evidence is used") from error


def find_claim(claims: list[Claim], wanted: str) -> Claim:
    for claim in claims:
        if claim.claim_id == wanted:
            return claim
    raise WorkerError(f"claim {wanted} not found in report")


def cmd_sample(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def cmd_record(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def cmd_summary(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def main(argv: list[str] | None = None) -> int:
    print('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.', file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
