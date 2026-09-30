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
    path = Path(args.file)
    tags = parse_tags(args.tags)
    claims = parse_claims(path, tags=tags)
    for claim in sample_claims(path, claims, args.n, args.seed):
        print(json.dumps(claim_json(claim), sort_keys=True))
    return OK


def cmd_record(args: argparse.Namespace) -> int:
    report = Path(args.file)
    claims = parse_claims(report)
    claim = find_claim(claims, args.claim)
    passed = bool(args.passed)
    output_digest = None
    output_path = None
    if args.output:
        output_path = Path(args.output)
        if not output_path.is_file():
            raise WorkerError(f"output file {output_path} not found")
        output_digest = file_digest(output_path)
    evidence_module = None
    if args.evidence:
        if not output_digest:
            raise WorkerError("--evidence requires --output so an output digest can be recorded")
        evidence_module = load_evidence_module()

    record = {
        "schema": "corenius.claim-outcome.v1",
        "report": str(report.resolve()),
        "claim_id": claim.claim_id,
        "passed": passed,
        "command": claim.command,
        "output": str(output_path.resolve()) if output_path else None,
        "output_digest": output_digest,
        "evidence": args.evidence,
        "recorded_at": now(),
    }
    path = outcome_path(report, create=True)
    with path.open("a") as handle:
        handle.write(json.dumps(record, sort_keys=True) + "\n")
    if evidence_module is not None:
        evidence_module.append_claim_outcome(
            args.evidence,
            claim.claim_id,
            passed,
            claim.command or "",
            output_digest,
            out_dir=None,
        )
    print(json.dumps({"recorded": str(path), **record}, sort_keys=True))
    return OK


def cmd_summary(args: argparse.Namespace) -> int:
    report = Path(args.file)
    claims = parse_claims(report)
    outcomes = load_outcomes(report)
    claim_ids = {claim.claim_id for claim in claims}
    recorded_ids = {outcome.get("claim_id") for outcome in outcomes if outcome.get("claim_id") in claim_ids}
    summary = {
        "sampled": len(claims),
        "recorded": len(recorded_ids),
        "outcomes": len(outcomes),
        "passed": sum(1 for outcome in outcomes if outcome.get("passed") is True),
        "failed": sum(1 for outcome in outcomes if outcome.get("passed") is False),
        "unrecorded": len(claim_ids - recorded_ids),
        "uncited": sum(1 for claim in claims if claim.status == "UNCITED"),
    }
    print(json.dumps(summary, sort_keys=True))
    latest = {outcome.get("claim_id"): outcome for outcome in outcomes}
    for claim in claims:
        outcome = latest.get(claim.claim_id)
        state = "unrecorded"
        if outcome:
            state = "passed" if outcome.get("passed") else "failed"
        print(json.dumps({**claim_json(claim), "outcome": state}, sort_keys=True))
    return OK


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("sample")
    p.add_argument("file")
    p.add_argument("--n", type=int, required=True)
    p.add_argument("--seed")
    p.add_argument("--tags", help="comma-separated tag list, for example R,X")
    p.set_defaults(fn=cmd_sample)
    p = sub.add_parser("record")
    p.add_argument("file")
    p.add_argument("--claim", required=True)
    verdict = p.add_mutually_exclusive_group(required=True)
    verdict.add_argument("--passed", action="store_true")
    verdict.add_argument("--failed", action="store_true")
    p.add_argument("--output")
    p.add_argument("--evidence", help="evidence record digest to append to")
    p.set_defaults(fn=cmd_record)
    p = sub.add_parser("summary")
    p.add_argument("file")
    p.set_defaults(fn=cmd_summary)
    args = parser.parse_args(argv)
    try:
        return args.fn(args)
    except WorkerError as error:
        print(f"claims: {error}", file=sys.stderr)
        return ERROR


if __name__ == "__main__":
    sys.exit(main())
