#!/usr/bin/env python3
"""Record and validate lead acceptance decisions for exact Candidates."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import claims  # noqa: E402
import land as land_tool  # noqa: E402
import tracker  # noqa: E402
from worker import WorkerError, git, git_proc, now  # noqa: E402

OK, ERROR = 0, 64
PASSED, FAILED = "passed", "failed"
AXES = ("standards", "spec")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
ID_RE = re.compile(r"^\d{2}$")
NAME_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.-]*$")


def repo_root(repo: str | Path) -> Path:
    return Path(git(repo, "rev-parse", "--show-toplevel").strip())


def effort_record_root(root: Path, effort: str) -> Path:
    try:
        return land_tool.resolve_effort_record(root, effort).worktree
    except WorkerError:
        if (root / "docs" / "efforts" / effort).is_dir():
            return root
        raise


def effort_dir(root: Path, effort: str) -> Path:
    path = root / "docs" / "efforts" / effort
    if not path.is_dir():
        raise WorkerError(f"effort {effort!r} not found")
    return path


def ticket_path(root: Path, effort: str, ticket: str) -> Path:
    if not ID_RE.match(ticket):
        raise WorkerError(f"bad ticket id {ticket!r}: expected NN")
    matches = sorted((effort_dir(root, effort) / "tickets").glob(f"{ticket}-*.md"))
    if not matches:
        raise WorkerError(f"ticket {ticket} not found in effort {effort}")
    return matches[0]


def digest_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(1024 * 1024)
            if not chunk:
                break
            hasher.update(chunk)
    return hasher.hexdigest()


def effort_has_record_branch(root: Path, effort: str) -> bool:
    path = effort_dir(root, effort) / "EFFORT.md"
    return bool(land_tool.parse_effort_metadata(path.read_text()).get("record_branch"))


def worker_report_relpath(effort: str, ticket: str) -> str:
    return f"docs/efforts/{effort}/evidence/{ticket}-report.md"


def imported_report_relpath(effort: str, ticket: str, round_number: int) -> str:
    return f"docs/efforts/{effort}/evidence/{ticket}-report-round-{round_number}.md"


def next_report_round(root: Path, effort: str, ticket: str) -> int:
    evidence = effort_dir(root, effort) / "evidence"
    rounds = []
    for path in evidence.glob(f"{ticket}-report-round-*.md") if evidence.is_dir() else []:
        match = re.match(rf"^{re.escape(ticket)}-report-round-(\d+)\.md$", path.name)
        if match:
            rounds.append(int(match.group(1)))
    return max(rounds, default=0) + 1


def import_worker_report(root: Path, effort: str, ticket: str, candidate: str) -> tuple[dict, Path] | tuple[None, None]:
    if not effort_has_record_branch(root, effort):
        return None, None
    source = worker_report_relpath(effort, ticket)
    result = git_proc(root, "show", f"{candidate}:{source}")
    if result.returncode != 0:
        return None, None
    round_number = next_report_round(root, effort, ticket)
    dest_rel = imported_report_relpath(effort, ticket, round_number)
    dest = root / dest_rel
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(result.stdout)
    return (
        {
            "path": dest_rel,
            "source_path": source,
            "source_commit": candidate,
            "round": round_number,
            "sha256": digest_file(dest),
        },
        dest,
    )


def copy_claim_outcomes(source_report: Path, imported_report: Path) -> None:
    source_path = claims.outcome_path(source_report)
    if not source_path.exists():
        return
    dest_path = claims.outcome_path(imported_report, create=True)
    if source_path.resolve() == dest_path.resolve():
        return
    records = []
    for line_no, line in enumerate(source_path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as error:
            raise WorkerError(f"{source_path}:{line_no}: bad claim outcome JSON: {error}") from error
        record["report"] = str(imported_report.resolve())
        records.append(json.dumps(record, sort_keys=True))
    if records:
        dest_path.write_text("\n".join(records) + "\n")


def record_path(root: Path, effort: str, ticket: str, *, create: bool = False) -> Path:
    ticket_path(root, effort, ticket)
    evidence = effort_dir(root, effort) / "evidence"
    if create:
        evidence.mkdir(parents=True, exist_ok=True)
    return evidence / f"acceptance-{ticket}.jsonl"


def rework_record_path(root: Path, effort: str, ticket: str, *, create: bool = False) -> Path:
    ticket_path(root, effort, ticket)
    evidence = effort_dir(root, effort) / "evidence"
    if create:
        evidence.mkdir(parents=True, exist_ok=True)
    return evidence / f"rework-{ticket}.jsonl"


def parse_result(value: str, label: str) -> str:
    normalized = value.strip().lower()
    if normalized not in {PASSED, FAILED}:
        raise WorkerError(f"{label} must be passed or failed")
    return normalized


def parse_check(value: str) -> tuple[str, str]:
    if "=" not in value:
        raise WorkerError("--check must be NAME=passed or NAME=failed")
    name, result = value.split("=", 1)
    name = name.strip()
    if not NAME_RE.match(name):
        raise WorkerError(f"bad check name {name!r}")
    return name, parse_result(result, f"check {name}")


def parse_required(values: list[str]) -> list[str]:
    required = []
    seen = set()
    for value in values:
        name = value.strip()
        if not NAME_RE.match(name):
            raise WorkerError(f"bad required check name {name!r}")
        if name not in seen:
            seen.add(name)
            required.append(name)
    return required


def claim_summary(report: Path) -> dict:
    parsed = claims.parse_claims(report)
    outcomes = claims.load_outcomes(report)
    claim_ids = {claim.claim_id for claim in parsed}
    recorded_ids = {outcome.get("claim_id") for outcome in outcomes if outcome.get("claim_id") in claim_ids}
    return {
        "claims": len(parsed),
        "recorded": len(recorded_ids),
        "outcomes": len(outcomes),
        "passed": sum(1 for outcome in outcomes if outcome.get("passed") is True),
        "failed": sum(1 for outcome in outcomes if outcome.get("passed") is False),
        "unrecorded": len(claim_ids - recorded_ids),
    }


def stored_path(path: Path, root: Path) -> str:
    resolved = path.resolve()
    try:
        return str(resolved.relative_to(root.resolve()))
    except ValueError:
        return str(resolved)


def validate_acceptance(record: dict, effort: str, ticket: str, candidate: str) -> tuple[bool, str]:
    if record.get("schema") != "corenius.acceptance.v1":
        return False, "unsupported acceptance schema"
    if record.get("effort") != effort:
        return False, "record effort mismatch"
    if record.get("ticket") != ticket:
        return False, "record ticket mismatch"
    if record.get("candidate") != candidate:
        return False, "record names another commit"
    axes = record.get("axes")
    if not isinstance(axes, dict):
        return False, "review axes missing"
    for axis in AXES:
        if axis not in axes:
            return False, f"missing review axis: {axis}"
        if axes.get(axis) != PASSED:
            return False, f"review axis {axis} did not pass"
    checks = record.get("required_checks")
    if not isinstance(checks, list) or not checks:
        return False, "required checks missing"
    for check in checks:
        if not isinstance(check, dict) or not check.get("name"):
            return False, "malformed required check"
        if check.get("result") not in {PASSED, FAILED}:
            return False, f"check {check.get('name')} has invalid result"
    failing = [check["name"] for check in checks if check["result"] == FAILED]
    if failing and not record.get("owner_waiver"):
        return False, f"failing required check without owner waiver: {', '.join(failing)}"
    if failing:
        waiver = record.get("owner_waiver")
        if not isinstance(waiver, dict) or not waiver.get("owner") or not waiver.get("reason"):
            return False, "owner waiver is incomplete"
    if record.get("accepted") is not True:
        return False, "record is not accepted"
    return True, ""


def load_records(root: Path, effort: str, ticket: str) -> list[dict]:
    path = record_path(root, effort, ticket)
    if not path.exists():
        return []
    return parse_record_lines(path.read_text(), path)


def parse_record_lines(text: str, path: Path) -> list[dict]:
    out = []
    for line_no, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise WorkerError(f"{path}:{line_no}: bad acceptance JSON: {error}") from error
    return out


def acceptance_for_records(records: list[dict], effort: str, ticket: str, candidate: str) -> tuple[dict | None, str]:
    matching = [record for record in records if record.get("candidate") == candidate]
    if not matching:
        if records:
            return None, "record names another commit"
        return None, "acceptance record missing"
    record = matching[-1]
    ok, reason = validate_acceptance(record, effort, ticket, candidate)
    return (record if ok else None), reason


def acceptance_for_candidate(root: Path, effort: str, ticket: str, candidate: str) -> tuple[dict | None, str]:
    return acceptance_for_records(load_records(root, effort, ticket), effort, ticket, candidate)


def require_for_landing(root: Path, effort: str, ticket: str, candidate: str) -> None:
    if not SHA_RE.match(candidate):
        raise WorkerError(f"bad candidate {candidate!r}: expected 40 hex characters")
    record_root = effort_record_root(root, effort)
    _, reason = acceptance_for_candidate(record_root, effort, ticket, candidate)
    if reason:
        raise WorkerError(reason)


def require_committed_for_landing(repo: Path, record_branch: str, effort: str, ticket: str, candidate: str) -> None:
    if not SHA_RE.match(candidate):
        raise WorkerError(f"bad candidate {candidate!r}: expected 40 hex characters")
    rel = f"docs/efforts/{effort}/evidence/acceptance-{ticket}.jsonl"
    result = git_proc(repo, "show", f"{record_branch}:{rel}")
    if result.returncode != 0:
        raise WorkerError("acceptance record missing")
    records = parse_record_lines(result.stdout, Path(rel))
    _, reason = acceptance_for_records(records, effort, ticket, candidate)
    if reason:
        raise WorkerError(reason)


def build_record(args: argparse.Namespace, root: Path) -> dict:
    ticket_path(root, args.effort, args.ticket)
    if not SHA_RE.match(args.candidate):
        raise WorkerError(f"bad candidate {args.candidate!r}: expected 40 hex characters")
    if not args.reviewer.strip():
        raise WorkerError("reviewer is required")
    axes = {}
    if args.standards:
        axes["standards"] = parse_result(args.standards, "standards")
    if args.spec:
        axes["spec"] = parse_result(args.spec, "spec")
    for axis in AXES:
        if axis not in axes:
            raise WorkerError(f"missing review axis: {axis}")
    checks = dict(parse_check(item) for item in args.check)
    required = parse_required(args.required_check)
    if not required:
        raise WorkerError("at least one required check must be named")
    for name in required:
        if name not in checks:
            raise WorkerError(f"missing required check: {name}")
    failing = sorted(name for name, result in checks.items() if result == FAILED)
    waiver = None
    if failing:
        if args.waiver_owner or args.waiver_reason:
            if not args.waiver_owner or not args.waiver_reason:
                raise WorkerError("owner waiver requires both owner and reason")
            waiver = {"owner": args.waiver_owner, "reason": args.waiver_reason}
    elif args.waiver_owner or args.waiver_reason:
        raise WorkerError("owner waiver requires a failing check")
    accepted = all(axes[axis] == PASSED for axis in AXES) and (not failing or waiver is not None)
    worker_report = None
    imported_report = None
    if accepted:
        worker_report, imported_report = import_worker_report(root, args.effort, args.ticket, args.candidate)
    claim_sampling = None
    if args.claims_report:
        report = Path(args.claims_report)
        if not report.is_file():
            raise WorkerError(f"claims report {report} not found")
        if imported_report is not None:
            copy_claim_outcomes(report, imported_report)
            report = imported_report
        outcome = claims.outcome_path(report)
        claim_sampling = {
            "report": stored_path(report, root),
            "outcomes": stored_path(outcome, root),
            "summary": claim_summary(report),
        }
    record = {
        "schema": "corenius.acceptance.v1",
        "effort": args.effort,
        "ticket": args.ticket,
        "candidate": args.candidate,
        "reviewer": args.reviewer.strip(),
        "reviewer_context": args.reviewer_context,
        "axes": axes,
        "required_checks": [{"name": name, "result": checks[name]} for name in sorted(checks)],
        "owner_waiver": waiver,
        "claim_sampling": claim_sampling,
        "accepted": accepted,
        "recorded_at": now(),
    }
    if worker_report is not None:
        record["worker_report"] = worker_report
    return record


def build_review_record(args: argparse.Namespace, root: Path) -> dict:
    ticket_path(root, args.effort, args.ticket)
    if not SHA_RE.match(args.candidate):
        raise WorkerError(f"bad candidate {args.candidate!r}: expected 40 hex characters")
    if not args.reviewer.strip():
        raise WorkerError("reviewer is required")
    findings = Path(args.findings_file)
    if not findings.is_file():
        raise WorkerError(f"findings file {findings} not found")
    axes = {}
    if args.standards:
        axes["standards"] = parse_result(args.standards, "standards")
    if args.spec:
        axes["spec"] = parse_result(args.spec, "spec")
    for axis in AXES:
        if axis not in axes:
            raise WorkerError(f"missing review axis: {axis}")
    accepted = all(axes[axis] == PASSED for axis in AXES)
    records = load_records(root, args.effort, args.ticket)
    round_number = sum(1 for record in records if record.get("accepted") is False) + 1
    return {
        "schema": "corenius.acceptance.v1",
        "effort": args.effort,
        "ticket": args.ticket,
        "candidate": args.candidate,
        "reviewer": args.reviewer.strip(),
        "reviewer_context": args.reviewer_context,
        "axes": axes,
        "required_checks": [],
        "owner_waiver": None,
        "claim_sampling": None,
        "accepted": accepted,
        "round": round_number,
        "findings": {
            "path": stored_path(findings, root),
            "sha256": digest_file(findings),
        },
        "recorded_at": now(),
    }


def cmd_record(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def cmd_review(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def load_rework_records(root: Path, effort: str, ticket: str) -> list[dict]:
    path = rework_record_path(root, effort, ticket)
    if not path.exists():
        return []
    records = []
    for line_no, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        try:
            records.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise WorkerError(f"{path}:{line_no}: bad rework JSON: {error}") from error
    return records


def rework_rounds_used(root: Path, effort: str, ticket: str) -> int:
    return sum(1 for record in load_rework_records(root, effort, ticket) if record.get("counts_toward_cap"))


def exhaust_review_rework(root: Path, effort: str, ticket_id: str, actor: str) -> None:
    ticket, _ = tracker.find_ticket(root, effort, ticket_id)
    tracker.write_triage_comment(
        ticket,
        "ready-for-human",
        actor,
        "review-driven rework cap exhausted after 3 round(s)",
        tracker.current_time(),
    )


def ensure_rework_round_available(root: Path, effort: str, ticket: str, kind: str, actor: str) -> int:
    ticket_path(root, effort, ticket)
    used = rework_rounds_used(root, effort, ticket)
    if kind == "review" and used >= 3:
        exhaust_review_rework(root, effort, ticket, actor)
        raise WorkerError("review-driven rework cap exhausted")
    return used + (1 if kind == "review" else 0)


def build_rework_record(
    root: Path,
    effort: str,
    ticket: str,
    *,
    candidate: str,
    kind: str,
    findings: Path,
    actor: str,
    extra: dict | None = None,
) -> dict:
    if not SHA_RE.match(candidate):
        raise WorkerError(f"bad candidate {candidate!r}: expected 40 hex characters")
    if not findings.is_file():
        raise WorkerError(f"findings file {findings} not found")
    round_number = ensure_rework_round_available(root, effort, ticket, kind, actor)
    record = {
        "schema": "corenius.rework.v1",
        "effort": effort,
        "ticket": ticket,
        "candidate": candidate,
        "kind": kind,
        "round": round_number,
        "counts_toward_cap": kind == "review",
        "findings": {
            "path": stored_path(findings, root),
            "sha256": digest_file(findings),
        },
        "requested_by": actor,
        "recorded_at": now(),
    }
    if extra:
        record.update(extra)
    return record


def append_rework_record(root: Path, effort: str, ticket: str, record: dict) -> Path:
    path = rework_record_path(root, effort, ticket, create=True)
    with path.open("a") as handle:
        handle.write(json.dumps(record, sort_keys=True) + "\n")
    return path


def record_rework_round(
    root: Path,
    effort: str,
    ticket: str,
    *,
    candidate: str,
    kind: str,
    findings: Path,
    actor: str,
    extra: dict | None = None,
) -> tuple[dict, Path]:
    record = build_rework_record(
        root,
        effort,
        ticket,
        candidate=candidate,
        kind=kind,
        findings=findings,
        actor=actor,
        extra=extra,
    )
    return record, append_rework_record(root, effort, ticket, record)


def cmd_request_rework(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def latest_accepted(records: list[dict]) -> dict[str, dict]:
    latest = {}
    for record in records:
        if record.get("accepted") is True and record.get("ticket"):
            latest[record["ticket"]] = record
    return latest


def tickets_reviewed_by_other_context(records: list[dict]) -> list[str]:
    return sorted({record["ticket"] for record in records if record.get("ticket") and record.get("reviewer_context") == "other"})


def cmd_summary(args: argparse.Namespace) -> int:
    raise WorkerError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--repo", default=".")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser(
        "record",
        help="write acceptance to the effort record branch before landing",
        description=(
            "Write an acceptance record to the effort record branch before landing. Ticket landings need "
            "effort records on a record branch; run effort.py move-records <effort> first."
        ),
    )
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--candidate", required=True)
    p.add_argument("--reviewer", required=True)
    p.add_argument("--reviewer-context", choices=("lead", "other"), required=True)
    p.add_argument("--standards", choices=(PASSED, FAILED))
    p.add_argument("--spec", choices=(PASSED, FAILED))
    p.add_argument("--required-check", action="append", default=[])
    p.add_argument("--check", action="append", default=[])
    p.add_argument("--waiver-owner")
    p.add_argument("--waiver-reason")
    p.add_argument("--claims-report")
    p.set_defaults(fn=cmd_record)
    p = sub.add_parser("review")
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--candidate", required=True)
    p.add_argument("--reviewer", required=True)
    p.add_argument("--reviewer-context", choices=("lead", "other"), required=True)
    p.add_argument("--standards", choices=(PASSED, FAILED))
    p.add_argument("--spec", choices=(PASSED, FAILED))
    p.add_argument("--findings-file", required=True)
    p.set_defaults(fn=cmd_review)
    p = sub.add_parser("request-rework")
    p.add_argument("effort")
    p.add_argument("ticket")
    p.add_argument("--candidate", required=True)
    p.add_argument("--kind", choices=("review", "stale-base"), default="review")
    p.add_argument("--findings-file", required=True)
    p.add_argument("--actor", required=True)
    p.set_defaults(fn=cmd_request_rework)
    p = sub.add_parser("summary")
    p.add_argument("effort")
    p.set_defaults(fn=cmd_summary)


def main(argv: list[str] | None = None) -> int:
    print('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.', file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
