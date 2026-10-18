#!/usr/bin/env python3
"""Lead-side worker launch for tracker tickets.

The command refuses tickets that are not launchable, claims a whole launch batch
before cutting any worker branch, then delegates process setup to worker.py.
"""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import os
import re
import shlex
import subprocess
import sys
import time
from datetime import timedelta
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import land  # noqa: E402
import acceptance  # noqa: E402
import tracker  # noqa: E402
import worker  # noqa: E402
from landing_protocol import guard_initialized_trunk_mutation  # noqa: E402
from worker import WorkerError  # noqa: E402

OK, REFUSED, USAGE = 0, 1, 64
INSTRUCTION_PATHS = ("AGENTS.md", "CONSTITUTION.md", "docs/agents/agentic-engineering.md")
CONTEXT_RE = re.compile(r"\bContext\s+(\d+%)\s+(?:left|remaining)\b", re.I)

STANDING_WORKER_BRIEF = """## Worker Operating Contract

Precedence:
- [standing] The tracked ticket, spec and decisions are the source of user intent.
- [standing] Ticket-specific permissions and prohibitions replace conflicting standing defaults.
- [standing] Lines in this section are standing rules; lines under "Ticket" come from the ticket source named above.

Read first, in this order:
1. [standing] AGENTS.md and CONSTITUTION.md.
2. [standing] docs/agents/agentic-engineering.md; this is isolated execution of the gated path, so intent, design, critique and specification are done.
3. [standing] The effort SPEC.md and every file in the effort decisions directory.
4. [ticket] The ticket file. Its acceptance items are what must be made true.
5. [ticket] The code named by the ticket's Declared paths, and tests beside that code, for style.

Boundaries:
- [ticket] Change files only inside this worker workspace and under the ticket's Declared paths, plus the ticket report path.
- [ticket] If the ticket cannot be done without touching another path, do the minimum and list each such path with the reason in the report.
- [standing] Never touch the main checkout, the integration worktree, the record worktree, another worker workspace, or another repository.
- [standing] Commit on the worker branch only. Do not push, merge, reset, force-move, delete, switch shared branches, or rewrite history below the recorded base.
- [ticket] Do not edit your ticket file itself unless the ticket explicitly says to.
- [standing] Do not launch real model workers or spend model budget unless the ticket explicitly authorizes it.

Method:
- [standing] Work test-first at the seams the spec names: write the failing test, see it fail, then make it pass.
- [ticket] Run the narrow suites and final gate named by the ticket. Report failures honestly, including failures believed to be pre-existing.
- [standing] Match surrounding code style; use the standard library unless the repository already provides a dependency.
- [standing] This repository is used in a multi-checkout layout: the mainline checkout, the integration worktree, the record worktree when effort records live on a record branch, and private worker workspaces. Test branch, effort-record, ticket and evidence behavior in that layout when those are touched.

Report and hand-off:
- [standing] Write the ticket report with these sections: Built, Acceptance, Claims, Outside declared paths, For later tickets, Not checked.
- [standing] Finish with one marker commit whose subject is exactly the ready marker named above. It may contain only the report.
- [standing] The tree must be clean after it. The marker commit is the done signal.
- [standing] After making the marker, do not amend, reset, rebase, rewrite, or otherwise move the handed-over commit.
- [standing] If the lead asks you to hold the marker, do not create the marker until released.
- [standing] If the lead reports that the integration branch moved, re-integrate only when asked, produce a new candidate on the moved base, rerun the gate, and finish with a new marker commit.
- [standing] If blocked on a decision or unavailable access, write the question in the report, commit with the exact needs-info marker, and stop.
"""


class Refusal(Exception):
    pass


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(repo: str | Path, *args: str, check: bool = True) -> str:
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True, env=worker.git_env())
    if check and result.returncode != 0:
        raise WorkerError(f"git {' '.join(args)} failed in {repo}: {result.stderr.strip() or result.stdout.strip()}")
    return result.stdout


def git_bytes(repo: str | Path, *args: str) -> bytes:
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, env=worker.git_env())
    if result.returncode != 0:
        detail = (result.stderr or result.stdout).decode("utf-8", "replace").strip()
        raise WorkerError(f"git {' '.join(args)} failed in {repo}: {detail}")
    return result.stdout


def repo_root(repo: str | Path) -> Path:
    return Path(git(repo, "rev-parse", "--show-toplevel").strip()).resolve()


def integration_worktree(repo: Path, integration: str) -> Path:
    checkout = land.worktree_for_ref(repo, land.local_mainline_ref(integration))
    if checkout is None:
        raise Refusal(f"integration branch {integration} is not checked out in any worktree")
    return checkout.resolve()


def resolve_integration(repo: Path, effort: str, explicit: str | None) -> str:
    if explicit:
        return explicit
    record = land.resolve_effort_record(repo, effort)
    integration = record.metadata.get("integration_branch")
    if not integration:
        raise Refusal(f"effort {effort} does not name an integration branch")
    return integration


def effort_record_root(repo: Path, effort: str) -> Path:
    return land.resolve_effort_record(repo, effort).worktree


def current_branch(repo: Path) -> str:
    return git(repo, "branch", "--show-current").strip()


def rev(repo: Path, ref: str) -> str:
    value = git(repo, "rev-parse", "--verify", f"{ref}^{{commit}}", check=False).strip()
    if not value:
        raise WorkerError(f"ref {ref!r} not found")
    return value


def clean_worktree(repo: Path) -> bool:
    return not git(repo, "status", "--porcelain", "--untracked-files=all").strip()


def source_at(repo: Path, commit: str, path: str) -> tuple[dict, str]:
    data = git_bytes(repo, "show", f"{commit}:{path}")
    return {"path": path, "commit": commit, "sha256": sha256(data), "size": len(data)}, data.decode("utf-8", "replace")


def launch_access_profile(args: argparse.Namespace) -> dict:
    return {
        "network": args.network,
        "command_timeout_seconds": int(args.command_timeout),
        "external_add_paths": [str(Path(path).resolve()) for path in (args.external_add_path or [])],
        "external_protect_paths": [str(Path(path).resolve()) for path in (args.external_protect_path or [])],
    }


def ticket_relpath(effort: str, ticket: tracker.Ticket) -> str:
    return f"docs/efforts/{effort}/tickets/{ticket.id}-{ticket.slug}.md"


def declared_paths(ticket: tracker.Ticket, effort: str) -> list[str]:
    start, end = tracker.section_bounds(ticket.lines, "## Declared paths")
    paths: list[str] = []
    if start is not None:
        for line in ticket.lines[start + 1 : end or len(ticket.lines)]:
            stripped = line.strip()
            if not stripped.startswith("- "):
                continue
            value = stripped[2:].strip()
            if value.startswith("`") and "`" in value[1:]:
                value = value[1:].split("`", 1)[0]
            if value:
                paths.append(value)
    report = f"docs/efforts/{effort}/evidence/{ticket.id}-report.md"
    if report not in paths:
        paths.append(report)
    return paths


def report_path(effort: str, ticket_id: str) -> str:
    return f"docs/efforts/{effort}/evidence/{ticket_id}-report.md"


def without_worker_report(effort: str, ticket_id: str, paths: list[str] | set[str]) -> list[str]:
    report = report_path(effort, ticket_id)
    return sorted(path for path in paths if path.rstrip("/") != report)


def ticket_declared_entries(effort: str, tickets: list[tracker.Ticket]) -> list[dict]:
    return [
        {
            "ticket": ticket.id,
            "declared_paths": without_worker_report(effort, ticket.id, declared_paths(ticket, effort)),
            "run": None,
        }
        for ticket in tickets
    ]


def run_declared_paths(run: dict) -> list[str]:
    launch = run.get("lead_launch", {})
    effort = launch.get("effort", "")
    ticket_id = launch.get("ticket", "")
    return without_worker_report(effort, ticket_id, list(run.get("allowed", [])))


def declared_path_overlaps(entries: list[dict]) -> list[dict]:
    findings = []
    for left, right in itertools.combinations(sorted(entries, key=lambda e: e["ticket"]), 2):
        common = sorted(
            {
                path
                for path in left["declared_paths"]
                if worker.allowed_path(path, right["declared_paths"])
            }
            | {
                path
                for path in right["declared_paths"]
                if worker.allowed_path(path, left["declared_paths"])
            }
        )
        if not common:
            continue
        tickets = [left["ticket"], right["ticket"]]
        findings.append({
            "severity": "NOTE",
            "kind": "declared-path-overlap",
            "tickets": tickets,
            "paths": common,
            "detail": f"tickets {tickets[0]} and {tickets[1]} both declare: {', '.join(common)}",
        })
    return findings


def ticket_problem(problems: list[tracker.Problem], ticket: tracker.Ticket, text: str) -> tracker.Problem | None:
    for problem in problems:
        if problem.path == ticket.path and text in problem.message:
            return problem
    return None


def load_launch_ticket(root: Path, effort: str, ticket_id: str) -> tuple[tracker.Ticket, dict[str, tracker.Ticket]]:
    if not tracker.ID_RE.match(ticket_id):
        raise WorkerError(f"bad ticket id {ticket_id!r}: expected NN")
    _, tickets, problems = tracker.load_tickets(root, effort)
    by_id = tracker.tickets_by_id(tickets)
    ticket = by_id.get(ticket_id)
    if ticket is None:
        raise WorkerError(f"ticket {ticket_id} not found in effort {effort}")
    bad_blocker = ticket_problem(problems, ticket, "bad blocker id")
    if bad_blocker:
        raise Refusal(f"ticket {ticket.id} has unparseable blocker entry: {bad_blocker.message}")
    unknown_blocker = ticket_problem(problems, ticket, "unknown blocker")
    if unknown_blocker:
        raise Refusal(f"ticket {ticket.id} has unresolved blocker entry: {unknown_blocker.message}")
    target_other = [
        problem for problem in problems
        if problem.path == ticket.path and "bad blocker id" not in problem.message and "unknown blocker" not in problem.message
    ]
    tracker.ensure_clean(target_other)
    return ticket, by_id


def launch_findings(root: Path, effort: str, target: str) -> list[dict]:
    tickets, findings, problems = tracker.load_later_findings(root, effort)
    by_id = tracker.tickets_by_id(tickets)
    if target not in by_id:
        raise WorkerError(f"ticket {target} not found in effort {effort}")
    relevant = [
        problem for problem in problems
        if problem.path == by_id[target].path or "later-ticket" in problem.message
    ]
    tracker.ensure_clean(relevant)
    return tracker.collect_findings_addressed_to(root, findings, target)


def validate_launchable(root: Path, effort: str, ticket: tracker.Ticket, by_id: dict[str, tracker.Ticket]) -> None:
    if ticket.status != "open":
        raise Refusal(f"ticket {ticket.id} status is {ticket.status or 'missing'}, not open")
    unresolved = [
        blocker for blocker in ticket.blocked_by
        if by_id[blocker].status != "resolved" or not tracker.has_accepted_landed_candidate(root, effort, by_id[blocker])
    ]
    if unresolved:
        raise Refusal(f"ticket {ticket.id} has unresolved blocker(s): {', '.join(unresolved)}")
    if ticket.triage != "ready-for-agent":
        raise Refusal(f"ticket {ticket.id} triage is {ticket.triage or 'missing'}, not ready-for-agent")


def active_lead_runs(effort: str) -> list[dict]:
    active = []
    for run in all_lead_runs(effort):
        if worker_slot_held(run):
            active.append(run)
    return active


def all_lead_runs(effort: str) -> list[dict]:
    runs = []
    for path in sorted((worker.runs_root() / "workers").glob("*/run.json")):
        try:
            run = json.loads(path.read_text())
        except (OSError, ValueError):
            continue
        if run.get("lead_launch", {}).get("effort") != effort:
            continue
        try:
            worker.update_session_id_from_events(run)
        except (OSError, ValueError, KeyError):
            pass
        runs.append(run)
    return runs


def worker_slot_held(run: dict) -> bool:
    if run.get("retained_for_rework"):
        return True
    alive, _ = worker.process_state(run)
    return alive


def changed_paths(run: dict) -> list[str]:
    worktree = Path(run["worktree"])
    paths = set(worker.porcelain_paths(git(worktree, "status", "--porcelain", "--untracked-files=all", check=False)))
    base = run.get("base_sha")
    if base:
        paths.update(git(worktree, "diff", "--name-only", f"{base}..HEAD", check=False).splitlines())
    return sorted(paths)


def is_ancestor(repo: Path, older: str, newer: str) -> bool:
    env = os.environ.copy()
    env["GIT_OPTIONAL_LOCKS"] = "0"
    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", older, newer],
        capture_output=True,
        env=worker.git_env(),
    ).returncode == 0


def active_overlap_report(repo: Path, effort: str, integration: str | None, active: list[dict] | None = None) -> dict:
    active_runs = active_lead_runs(effort) if active is None else active
    integration_tip = rev(repo, integration) if integration else None
    entries = []
    for run in sorted(active_runs, key=lambda r: r.get("lead_launch", {}).get("ticket", "")):
        launch = run.get("lead_launch", {})
        ticket_id = launch.get("ticket", "")
        declared = run_declared_paths(run)
        changed = without_worker_report(effort, ticket_id, changed_paths(run))
        base = launch.get("base") or run.get("base_sha")
        behind = bool(integration_tip and base and base != integration_tip and is_ancestor(repo, base, integration_tip))
        entries.append({
            "ticket": ticket_id,
            "run_dir": run.get("run_dir"),
            "worktree": run.get("worktree"),
            "branch": run.get("branch"),
            "base": base,
            "changed_paths": changed,
            "declared_paths": declared,
            "alive": True,
            "behind_integration": behind,
        })

    findings = declared_path_overlaps(entries)
    by_ticket = {entry["ticket"]: entry for entry in entries}
    for entry in entries:
        for path in entry["changed_paths"]:
            if not worker.allowed_path(path, entry["declared_paths"]):
                findings.append({
                    "severity": "VIOLATION",
                    "kind": "changed-outside-declared",
                    "ticket": entry["ticket"],
                    "path": path,
                    "detail": f"ticket {entry['ticket']} changed outside its declared paths: {path}",
                })
    for left, right in itertools.permutations(entries, 2):
        for path in left["changed_paths"]:
            if worker.allowed_path(path, left["declared_paths"]):
                continue
            if worker.allowed_path(path, right["declared_paths"]):
                tickets = [left["ticket"], right["ticket"]]
                findings.append({
                    "severity": "VIOLATION",
                    "kind": "changed-path-declared-by-other",
                    "tickets": tickets,
                    "path": path,
                    "detail": f"ticket {tickets[0]} changed {path}, declared by running ticket {tickets[1]}",
                })
    changed_by_path: dict[str, list[str]] = {}
    for entry in entries:
        for path in entry["changed_paths"]:
            changed_by_path.setdefault(path, []).append(entry["ticket"])
    common_changed_paths = []
    for path, tickets in sorted(changed_by_path.items()):
        if len(tickets) < 2:
            continue
        sorted_tickets = sorted(tickets)
        common_changed_paths.append({"tickets": sorted_tickets, "path": path})
        if any(worker.allowed_path(path, by_ticket[ticket]["declared_paths"]) for ticket in tickets):
            continue
        findings.append({
            "severity": "VIOLATION",
            "kind": "common-undeclared-changed-path",
            "tickets": sorted_tickets,
            "path": path,
            "detail": f"running tickets {' and '.join(sorted_tickets)} both changed undeclared path {path}",
        })
    return {
        "effort": effort,
        "integration": integration,
        "integration_tip": integration_tip,
        "active": entries,
        "common_changed_paths": common_changed_paths,
        "findings": findings,
    }


def claim_batch(root: Path, effort: str, tickets: list[tracker.Ticket], owner_prefix: str, ttl_hours: float) -> None:
    with guard_initialized_trunk_mutation(root):
        stamp = tracker.current_time()
        expires = stamp + timedelta(hours=ttl_hours)
        claim_text = f"{tracker.format_utc(stamp)} expires {tracker.format_utc(expires)}"
        for ticket in tickets:
            tracker.write_transition(
                ticket,
                {"Status": "claimed", "Owner": f"{owner_prefix}-{ticket.id}", "Claimed": claim_text},
                f"{owner_prefix}-{ticket.id}",
                f"claimed until {tracker.format_utc(expires)}",
                stamp,
            )
        git(root, "add", *[ticket_relpath(effort, ticket) for ticket in tickets])
        subject = f"Claim ticket {tickets[0].id} for worker" if len(tickets) == 1 else f"Claim tickets {', '.join(t.id for t in tickets)} for workers"
        git(root, "commit", "-q", "-m", subject)


def release_unlaunched_claims(root: Path, effort: str, tickets: list[tracker.Ticket], launched: list[dict], actor: str) -> None:
    with guard_initialized_trunk_mutation(root):
        launched_tickets = {item.get("lead_launch", {}).get("ticket") for item in launched}
        _, current_tickets, problems = tracker.load_tickets(root, effort)
        tracker.ensure_clean(problems)
        by_id = tracker.tickets_by_id(current_tickets)
        to_release = [by_id[ticket.id] for ticket in tickets if ticket.id not in launched_tickets]
        if not to_release:
            return
        stamp = tracker.current_time()
        for ticket in to_release:
            tracker.write_transition(
                ticket,
                {"Status": "open", "Owner": "", "Claimed": ""},
                actor,
                "released after worker launch failed before a run started",
                stamp,
            )
        git(root, "add", *[ticket_relpath(effort, ticket) for ticket in to_release])
        subject = (
            f"Release ticket {to_release[0].id} after failed launch"
            if len(to_release) == 1
            else f"Release tickets {', '.join(t.id for t in to_release)} after failed launch"
        )
        git(root, "commit", "-q", "-m", subject)


def launch_sources(
    record_root: Path,
    code_root: Path,
    effort: str,
    ticket: tracker.Ticket,
    record_base: str,
    code_base: str,
) -> tuple[dict, dict[str, str]]:
    ticket_source, ticket_text = source_at(record_root, record_base, ticket_relpath(effort, ticket))
    spec_source, spec_text = source_at(record_root, record_base, f"docs/efforts/{effort}/SPEC.md")
    instruction_sources = []
    instruction_texts = {}
    for path in INSTRUCTION_PATHS:
        source, text = source_at(code_root, code_base, path)
        instruction_sources.append(source)
        instruction_texts[path] = text
    return (
        {"ticket": ticket_source, "spec": spec_source, "worker_instructions": instruction_sources},
        {"ticket": ticket_text, "spec": spec_text, **instruction_texts},
    )


def brief_findings(findings: list[dict]) -> str:
    if not findings:
        return ""
    lines = ["## Findings From Earlier Tickets"]
    for finding in findings:
        sources = ", ".join(source["label"] for source in finding["sources"])
        lines.append(f"- from {sources}: {finding['text']}")
    return "\n\n" + "\n".join(lines) + "\n"


TICKET_RULE_RE = re.compile(
    r"\b("
    r"you may|you must|must not|do not|do NOT|never|allowed|not allowed|authorized|not authorized|"
    r"permission|prohibition|forbidden|network is|network access"
    r")\b",
    re.I,
)


def ticket_specific_rules(ticket_text: str) -> list[str]:
    rules: list[str] = []
    for raw in ticket_text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or ":" in line[:20]:
            continue
        if line.startswith("- "):
            line = line[2:].strip()
        if not TICKET_RULE_RE.search(line):
            continue
        rules.append(line)
    return rules


def ticket_rules_section(ticket_text: str) -> str:
    lines = [
        "## Ticket-Specific Permissions and Prohibitions",
        "These [ticket] lines replace conflicting [standing] defaults for this ticket.",
    ]
    rules = ticket_specific_rules(ticket_text)
    if not rules:
        lines.append("- [ticket] none found; standing defaults apply.")
    else:
        lines.extend(f"- [ticket] {rule}" for rule in rules)
    return "\n".join(lines)


def build_brief(
    effort: str,
    ticket: tracker.Ticket,
    integration: str,
    base: str,
    sources: dict,
    texts: dict[str, str],
    findings: list[dict],
) -> str:
    source_lines = [
        f"- ticket: {sources['ticket']['path']} @ {sources['ticket']['commit']} sha256={sources['ticket']['sha256']}",
        f"- spec: {sources['spec']['path']} @ {sources['spec']['commit']} sha256={sources['spec']['sha256']}",
    ]
    for source in sources["worker_instructions"]:
        source_lines.append(f"- instruction: {source['path']} @ {source['commit']} sha256={source['sha256']}")
    instructions = "\n\n".join(f"## {path}\n{text.rstrip()}" for path, text in texts.items() if path in INSTRUCTION_PATHS)
    return (
        f"# Lead launch brief\n\n"
        f"Effort: {effort}\n"
        f"Ticket: {ticket.id}-{ticket.slug}\n"
        f"Integration branch: {integration}\n"
        f"Base commit: {base}\n"
        f"Ready marker: ticket {ticket.id}: ready for review\n\n"
        "## Pinned sources\n"
        + "\n".join(source_lines)
        + "\n\n"
        + STANDING_WORKER_BRIEF.rstrip()
        + "\n\n"
        + ticket_rules_section(texts["ticket"])
        + "\n\n"
        "## Standing worker instructions\n"
        + instructions
        + "\n\n"
        "## Spec\n"
        + texts["spec"].rstrip()
        + brief_findings(findings)
        + "\n\n"
        "## Ticket\n"
        + texts["ticket"].rstrip()
        + "\n"
    )


def tmux_window_name(ticket: tracker.Ticket) -> str:
    return f"{ticket.id}-{ticket.slug}"[:32]


def run_tmux(*args: str) -> tuple[subprocess.CompletedProcess | None, str]:
    try:
        return subprocess.run(["tmux", *args], capture_output=True, text=True), ""
    except OSError as exc:
        return None, str(exc)


def ensure_tmux_available() -> None:
    proc, error = run_tmux("-V")
    if proc is None:
        raise Refusal(f"interactive launch requires tmux: {error}")
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout).strip() or "tmux check failed"
        raise Refusal(f"interactive launch requires tmux: {detail}")


def tmux_window_record(session: str | None, window_name: str, cwd: Path, command: str) -> dict:
    if not session:
        return {"available": False, "session": None, "window": window_name, "reason": "not requested"}
    base = {"available": False, "session": session, "window": window_name, "command": command}
    has, error = run_tmux("has-session", "-t", session)
    if has is None:
        return {**base, "error": error}
    if has.returncode != 0:
        created, error = run_tmux(
            "new-session", "-d", "-s", session, "-n", "overview", "-c", str(cwd), "while :; do sleep 3600; done"
        )
        if created is None:
            return {**base, "error": error}
        if created.returncode != 0:
            return {**base, "error": (created.stderr or created.stdout or has.stderr or has.stdout).strip()}
    new, error = run_tmux("new-window", "-d", "-t", f"{session}:", "-n", window_name, "-c", str(cwd), command)
    if new is None:
        return {**base, "error": error}
    if new.returncode != 0:
        return {**base, "error": (new.stderr or new.stdout).strip()}
    return {**base, "available": True, "target": f"{session}:{window_name}"}


def watch_command(run_dir: Path, title: str) -> str:
    stream = Path(__file__).resolve().parents[2] / "bench" / "tmuxh" / "stream.py"
    return shlex.join([sys.executable, str(stream), "--follow", str(run_dir / "events.jsonl"), "--title", title])


def run_worker_launch(args: argparse.Namespace, ticket: tracker.Ticket, base: str, brief: str, sources: dict, findings: list[dict]) -> dict:
    profile = launch_access_profile(args)
    try:
        tmpdir = worker.temporary_directory(prefix=f"corenius-launch-{ticket.id}-")
    except OSError as error:
        raise WorkerError(f"could not create launch workspace for ticket {ticket.id}: {error}") from error
    with tmpdir as tmp:
        task = Path(tmp) / "brief.md"
        try:
            task.write_text(brief)
        except OSError as error:
            raise WorkerError(f"could not write launch brief for ticket {ticket.id}: {error}") from error
        cmd = [
            sys.executable,
            str(Path(__file__).resolve().with_name("worker.py")),
            "launch",
            "--task",
            str(task),
            "--name",
            args.name or f"{args.effort}-{ticket.id}-{ticket.slug}",
            "--repo",
            str(args.repo_root),
            "--base",
            base,
            "--runtime",
            args.runtime,
            "--handoff-mode",
            "marker",
            "--handoff-marker",
            f"ticket {ticket.id}: ready for review",
            "--model",
            args.model,
            "--effort",
            args.reasoning_effort,
            "--sandbox",
            args.sandbox,
            "--network",
            profile["network"],
            "--command-timeout",
            str(profile["command_timeout_seconds"]),
            "--cap",
            str(args.worker_cap),
            "--stall",
            str(args.stall),
        ]
        if args.interactive:
            cmd += ["--interactive", "--tmux-session", args.tmux_session, "--tmux-window", tmux_window_name(ticket)]
        if args.keep_temp:
            cmd.append("--keep-temp")
        for path in declared_paths(ticket, args.effort):
            cmd += ["--allow", path]
        for path in profile["external_add_paths"]:
            cmd += ["--external-add-path", path]
        for path in profile["external_protect_paths"]:
            cmd += ["--external-protect-path", path]
        for path in args.protect:
            cmd += ["--protect", path]
        proc = subprocess.run(cmd, capture_output=True, text=True, env=worker.git_env())
    if proc.returncode != 0:
        raise WorkerError(proc.stderr.strip() or proc.stdout.strip())
    launched = json.loads(proc.stdout)
    run_path = Path(launched["run_dir"]) / "run.json"
    run = json.loads(run_path.read_text())
    run["lead_launch"] = {
        "effort": args.effort,
        "ticket": ticket.id,
        "integration": args.integration,
        "base": base,
        "runtime": args.runtime,
        "interactive": bool(args.interactive),
        "parallel_cap": args.parallel_cap,
        "access_profile": profile,
        "sources": sources,
        "findings": findings,
    }
    if args.interactive:
        run["lead_launch"]["tmux_window"] = run.get("tmux_window") or {
            "available": False,
            "session": args.tmux_session,
            "window": tmux_window_name(ticket),
            "reason": "interactive launch did not report tmux target",
        }
    else:
        run["lead_launch"]["tmux_window"] = tmux_window_record(
            args.tmux_session,
            tmux_window_name(ticket),
            Path(run["worktree"]),
            watch_command(Path(run["run_dir"]), f"{ticket.id}-{ticket.slug}"),
        )
    run_path.write_text(json.dumps(run, indent=2) + "\n")
    return launched


def include_batch_branch_baselines(launched: list[dict]) -> None:
    runs = []
    heads: dict[str, str] = {}
    for item in launched:
        run_path = Path(item["run_dir"]) / "run.json"
        run = json.loads(run_path.read_text())
        runs.append((run_path, run))
        heads[run["branch"]] = rev(Path(run["worktree"]), run["branch"])
    for run_path, run in runs:
        baseline = dict(run.get("branch_baselines", {}))
        peers = []
        for branch, head in heads.items():
            if branch != run["branch"]:
                baseline[branch] = head
                peers.append(branch)
        run["branch_baselines"] = baseline
        run["peer_worker_branches"] = sorted(peers)
        run_path.write_text(json.dumps(run, indent=2) + "\n")


def cmd_launch(args: argparse.Namespace) -> int:
    repo = repo_root(args.repo_root)
    args.integration = resolve_integration(repo, args.effort, args.integration)
    if args.interactive:
        if not args.tmux_session:
            raise Refusal("interactive launch requires --tmux-session")
        ensure_tmux_available()
    code_root = integration_worktree(repo, args.integration)
    record_root = land.resolve_effort_record(code_root, args.effort).worktree
    args.repo_root = code_root
    if not clean_worktree(code_root):
        raise Refusal("integration worktree has uncommitted changes")
    if record_root != code_root and not clean_worktree(record_root):
        raise Refusal("effort record worktree has uncommitted changes")
    tickets = []
    by_id = {}
    for ticket_id in args.tickets:
        ticket, by_id = load_launch_ticket(record_root, args.effort, ticket_id)
        validate_launchable(record_root, args.effort, ticket, by_id)
        tickets.append(ticket)
    active = active_lead_runs(args.effort)
    if len(active) + len(tickets) > args.parallel_cap:
        raise Refusal(f"parallel cap exceeded: {len(active)} active worker(s), {len(tickets)} requested, cap {args.parallel_cap}")
    launch_overlap = {
        "effort": args.effort,
        "integration": args.integration,
        "findings": declared_path_overlaps(
            [
                *[
                    {
                        "ticket": run.get("lead_launch", {}).get("ticket", ""),
                        "declared_paths": run_declared_paths(run),
                        "run": run,
                    }
                    for run in active
                ],
                *ticket_declared_entries(args.effort, tickets),
            ]
        ),
    }
    if record_root == code_root:
        claim_batch(record_root, args.effort, tickets, args.owner_prefix, args.ttl_hours)
        base = rev(code_root, args.integration)
        record_base = base
    else:
        base = rev(code_root, args.integration)
        claim_batch(record_root, args.effort, tickets, args.owner_prefix, args.ttl_hours)
        record_base = rev(record_root, "HEAD")
    launched = []
    try:
        for ticket in tickets:
            # Re-read after the claim commit so the pinned ticket source is the exact text the worker receives.
            refreshed, _ = load_launch_ticket(record_root, args.effort, ticket.id)
            sources, texts = launch_sources(record_root, code_root, args.effort, refreshed, record_base, base)
            findings = launch_findings(record_root, args.effort, refreshed.id)
            brief = build_brief(args.effort, refreshed, args.integration, base, sources, texts, findings)
            launched.append(run_worker_launch(args, refreshed, base, brief, sources, findings))
    except Exception:
        release_unlaunched_claims(record_root, args.effort, tickets, launched, args.owner_prefix)
        raise
    include_batch_branch_baselines(launched)
    output = launched[0] if len(launched) == 1 else {"launched": launched}
    output["overlap"] = launch_overlap
    print(json.dumps(output, indent=None))
    return OK


def cmd_overlap(args: argparse.Namespace) -> int:
    repo = repo_root(args.repo_root)
    integration = resolve_integration(repo, args.effort, args.integration)
    print(json.dumps(active_overlap_report(repo, args.effort, integration), sort_keys=True))
    return OK


def cmd_question(args: argparse.Namespace) -> int:
    root = repo_root(args.repo_root)
    code = tracker.main([
        "--repo",
        str(root),
        "question",
        args.effort,
        args.ticket,
        "--owner",
        args.owner,
        "--question",
        args.question,
    ])
    return code


def rework_message(run: dict, findings: Path, digest: str, kind: str) -> str:
    candidate = (run.get("candidate") or {}).get("commit", "<unknown>")
    lead = run.get("lead_launch") or {}
    ticket = lead.get("ticket", "<unknown>")
    if kind == "stale-base":
        opening = "The integration branch moved; create a new Candidate on the moved base."
    else:
        opening = "The review found issues; create a new Candidate that addresses the findings."
    marker = run.get("handoff_marker")
    marker_text = f"\nFinish with a `{marker}` marker commit." if marker else ""
    return (
        f"{opening}\n"
        f"Ticket: {ticket}\n"
        f"Previous Candidate: {candidate}\n"
        f"Findings file: {findings.resolve()}\n"
        f"Findings sha256: {digest}\n"
        f"{marker_text}\n"
    )


def matching_rejected_review(root: Path, run: dict, digest: str) -> dict | None:
    lead = run.get("lead_launch") or {}
    effort = lead.get("effort")
    ticket = lead.get("ticket")
    candidate = (run.get("candidate") or {}).get("commit")
    if not (effort and ticket and candidate):
        return None
    effort_root = land.resolve_effort_record(root, effort).worktree
    for record in reversed(acceptance.load_records(effort_root, effort, ticket)):
        if record.get("candidate") != candidate or record.get("accepted") is not False:
            continue
        findings = record.get("findings") or {}
        if findings.get("sha256") == digest:
            return record
    return None


def append_rework_delivery(
    run: dict,
    *,
    kind: str,
    findings: Path,
    digest: str,
    receipt: dict,
    shared_record: dict | None = None,
    shared_record_path: Path | None = None,
    review: dict | None = None,
) -> dict:
    candidate = run.get("candidate")
    if not candidate:
        raise WorkerError("cannot request rework before a Candidate is pinned")
    history = worker.ensure_candidate_history(run)
    used = sum(1 for delivery in run.get("rework_deliveries", []) if delivery.get("counts_toward_cap"))
    acknowledged = receipt.get("acknowledged") is True
    if shared_record:
        round_number = shared_record["round"]
        counts_toward_cap = shared_record["counts_toward_cap"]
    else:
        counts_toward_cap = kind == "review" and acknowledged
        round_number = used + 1 if counts_toward_cap else None if kind == "review" else used
    delivery = {
        "id": f"{kind}-{len(run.get('rework_deliveries', [])) + 1}",
        "kind": kind,
        "round": round_number,
        "counts_toward_cap": counts_toward_cap,
        "run_dir": str(Path(run["run_dir"])),
        "candidate": candidate["commit"],
        "findings_file": str(findings.resolve()),
        "findings_sha256": digest,
        "delivered_at": worker.now(),
        "receipt": receipt,
    }
    if shared_record_path:
        delivery["effort_rework_record"] = {"path": str(shared_record_path), "round": round_number}
    run.setdefault("rework_deliveries", []).append(delivery)
    if review is not None:
        history[-1]["review"] = review
    elif acknowledged and kind == "review":
        history[-1]["review"] = {
            "delivery_id": delivery["id"],
            "findings_file": delivery["findings_file"],
            "findings_sha256": digest,
            "delivered_at": delivery["delivered_at"],
        }
    with (Path(run["run_dir"]) / "rework.jsonl").open("a") as handle:
        handle.write(json.dumps(delivery, sort_keys=True) + "\n")
    worker.save_run(run)
    return delivery


def rework_record_context(root: Path, run: dict) -> tuple[Path, str, str]:
    lead = run.get("lead_launch") or {}
    effort = lead.get("effort")
    ticket = lead.get("ticket")
    if not (effort and ticket):
        raise WorkerError("run does not carry lead_launch effort and ticket metadata")
    return land.resolve_effort_record(root, effort).worktree, effort, ticket


def ensure_rework_available(root: Path, run: dict, kind: str) -> None:
    record_root, effort, ticket = rework_record_context(root, run)
    try:
        acceptance.ensure_rework_round_available(record_root, effort, ticket, kind, "lead")
    except WorkerError as error:
        if "review-driven rework cap exhausted" in str(error):
            raise Refusal(str(error)) from error
        raise


def record_lead_rework(
    root: Path,
    run: dict,
    *,
    kind: str,
    findings: Path,
    receipt: dict,
) -> tuple[dict, Path]:
    record_root, effort, ticket = rework_record_context(root, run)
    return acceptance.record_rework_round(
        record_root,
        effort,
        ticket,
        candidate=run["candidate"]["commit"],
        kind=kind,
        findings=findings,
        actor="lead",
        extra={
            "run_dir": str(Path(run["run_dir"])),
            "receipt": receipt,
            "source": "lead_launch.py rework",
        },
    )


def fresh_rework_brief(run: dict, findings: Path, digest: str, kind: str) -> str:
    return (
        "# Rework brief\n\n"
        + rework_message(run, findings, digest, kind).strip()
        + "\n\n"
        "## Earlier worker brief\n"
        + Path(run["run_dir"], "prompt.md").read_text().rstrip()
        + "\n"
    )


def launch_fresh_rework(args: argparse.Namespace, run: dict, findings: Path, digest: str) -> dict:
    candidate = (run.get("candidate") or {}).get("commit")
    if not candidate:
        raise WorkerError("cannot launch fresh rework before a Candidate is pinned")
    worker.import_candidate_to_shared_repo(run, run["candidate"])
    worker.save_run(run)
    base = candidate
    integration = (run.get("lead_launch") or {}).get("integration")
    if args.kind == "stale-base" and integration:
        base = rev(args.repo_root, integration)
    profile = worker.access_profile(run)
    deliveries = len(run.get("rework_deliveries", [])) + 1
    with worker.temporary_directory(prefix="corenius-rework-") as tmp:
        task = Path(tmp) / "brief.md"
        task.write_text(fresh_rework_brief(run, findings, digest, args.kind))
        cmd = [
            sys.executable,
            str(Path(__file__).resolve().with_name("worker.py")),
            "launch",
            "--task",
            str(task),
            "--name",
            f"{run['name']}-rework-{deliveries}",
            "--repo",
            str(args.repo_root),
            "--base",
            base,
            "--runtime",
            run.get("runtime", "traecli"),
            "--model",
            run.get("model", "GPT-5.5"),
            "--effort",
            run.get("effort", "xhigh"),
            "--sandbox",
            worker.run_sandbox_mode(run),
            "--network",
            profile["network"],
            "--command-timeout",
            str(profile["command_timeout_seconds"]),
            "--cap",
            str(run.get("cap_seconds", 2700)),
            "--stall",
            str(run.get("stall_seconds", 300)),
        ]
        if (run.get("temp") or {}).get("keep"):
            cmd.append("--keep-temp")
        if run.get("capsule"):
            cmd += ["--capsule", str(run["capsule"])]
        if run.get("handoff_mode"):
            cmd += ["--handoff-mode", run["handoff_mode"]]
        if run.get("handoff_marker"):
            cmd += ["--handoff-marker", run["handoff_marker"]]
        if profile.get("container_builds"):
            cmd.append("--allow-container-builds")
        if profile.get("shared_cache_writes"):
            cmd.append("--allow-shared-cache-writes")
        for path in run.get("allowed", []):
            cmd += ["--allow", path]
        for path in profile.get("external_add_paths", []):
            cmd += ["--external-add-path", path]
        for path in (run.get("external_state") or {}).get("protected_paths", []):
            cmd += ["--external-protect-path", path]
        shared_repo = str(Path(run.get("shared_repo", "")).resolve()) if run.get("shared_repo") else None
        for entry in run.get("protected", []):
            path = str(Path(entry["path"]).resolve())
            if shared_repo and path == shared_repo:
                continue
            cmd += ["--protect", path]
        proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        raise WorkerError(proc.stderr.strip() or proc.stdout.strip())
    launched = json.loads(proc.stdout)
    fresh_path = Path(launched["run_dir"]) / "run.json"
    fresh = json.loads(fresh_path.read_text())
    fresh["lead_launch"] = {
        **(run.get("lead_launch") or {}),
        "rework_of": str(Path(run["run_dir"])),
        "previous_candidate": candidate,
        "rework_base": base,
        "findings_file": str(findings.resolve()),
        "findings_sha256": digest,
    }
    fresh_path.write_text(json.dumps(fresh, indent=2) + "\n")
    return launched


def cmd_rework(args: argparse.Namespace) -> int:
    root = repo_root(args.repo_root)
    args.repo_root = root
    run = worker.load_run(args.run_dir)
    findings = Path(args.findings_file)
    if not findings.is_file():
        raise WorkerError(f"findings file {findings} not found")
    digest = worker.digest_file(findings)
    session_id = worker.update_session_id_from_events(run)
    message = rework_message(run, findings, digest, args.kind)
    ensure_rework_available(root, run, args.kind)
    target = pane_target(run)
    if run.get("interactive") and target:
        refreshed = worker.refresh_integration_branch(run) if args.kind == "stale-base" else None
        terminal_message = worker.findings_message(run, findings, digest, args.kind)
        receipt = worker.terminal_delivery_receipt(run, target, None, terminal_message, findings, args.ack_timeout)
        receipt["refreshed_integration"] = refreshed
        run = worker.load_run(args.run_dir)
        review = matching_rejected_review(root, run, digest)
        shared_record, shared_path = record_lead_rework(root, run, kind=args.kind, findings=findings, receipt=receipt)
        delivery = append_rework_delivery(
            run,
            kind=args.kind,
            findings=findings,
            digest=digest,
            receipt=receipt,
            shared_record=shared_record,
            shared_record_path=shared_path,
            review=review,
        )
        print(json.dumps({"mode": "terminal", "delivery": delivery, "terminal": receipt}, sort_keys=True))
        return OK
    if session_id:
        proc = subprocess.run(
            [
                sys.executable,
                str(Path(__file__).resolve().with_name("worker.py")),
                "resume",
                str(Path(run["run_dir"])),
                "--message",
                message,
                "--detach",
            ],
            capture_output=True,
            text=True,
        )
        if proc.returncode not in (worker.RUNNING, worker.CLEAN):
            raise WorkerError(proc.stderr.strip() or proc.stdout.strip())
        receipt = json.loads(proc.stdout)
        run = worker.load_run(args.run_dir)
        receipt.setdefault("events_offset", 0)
        receipt.setdefault("head_before", (run.get("candidate") or {}).get("commit"))
        shared_record, shared_path = record_lead_rework(root, run, kind=args.kind, findings=findings, receipt=receipt)
        review = matching_rejected_review(root, run, digest)
        delivery = append_rework_delivery(
            run,
            kind=args.kind,
            findings=findings,
            digest=digest,
            receipt=receipt,
            shared_record=shared_record,
            shared_record_path=shared_path,
            review=review,
        )
        print(json.dumps({"mode": "resumed", "delivery": delivery, "resume": receipt}, sort_keys=True))
        return OK
    launched = launch_fresh_rework(args, run, findings, digest)
    receipt = {
        "transport": "fresh-worker",
        "reason": "no session id observed in events.jsonl",
        "fresh_run_dir": launched["run_dir"],
        "fresh_worktree": launched["worktree"],
        "acknowledged": False,
    }
    shared_record, shared_path = record_lead_rework(root, run, kind=args.kind, findings=findings, receipt=receipt)
    review = matching_rejected_review(root, run, digest)
    delivery = append_rework_delivery(
        run,
        kind=args.kind,
        findings=findings,
        digest=digest,
        receipt=receipt,
        shared_record=shared_record,
        shared_record_path=shared_path,
        review=review,
    )
    print(json.dumps({
        "mode": "fresh-worker",
        "reason": receipt["reason"],
        "fresh_run_dir": launched["run_dir"],
        "fresh_worktree": launched["worktree"],
        "delivery": delivery,
    }, sort_keys=True))
    return OK


def latest_run_by_ticket(effort: str) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for run in all_lead_runs(effort):
        ticket = run.get("lead_launch", {}).get("ticket")
        if not ticket:
            continue
        current = out.get(ticket)
        if current is None or float(run.get("started_epoch", 0)) >= float(current.get("started_epoch", 0)):
            out[ticket] = run
    return out


def clean_status(path: Path) -> str:
    return worker.git(path, "status", "--porcelain", "--untracked-files=all", check=False)


def reviewed_candidate(run: dict) -> str | None:
    for entry in reversed(run.get("candidates", [])):
        candidate = entry.get("candidate") or {}
        if entry.get("review") and candidate.get("commit"):
            return candidate["commit"]
    candidate = run.get("candidate") or {}
    return candidate.get("commit")


def diff_names(repo: Path, left: str | None, right: str = "HEAD") -> list[str]:
    if not left:
        return []
    text = worker.git(repo, "diff", "--name-only", f"{left}..{right}", check=False)
    return [line for line in text.splitlines() if line.strip()]


def outside_allowed(paths: list[str], allowed: list[str]) -> list[str]:
    return [path for path in paths if not worker.allowed_path(path, allowed)]


def worker_integration_tip(run: dict, integration: str) -> str | None:
    worktree = Path(run["worktree"])
    return (
        worker.ref_commit(worktree, integration)
        or (run.get("branch_baselines") or {}).get(integration)
        or (run.get("shared_branch_baselines") or {}).get(integration)
        or run.get("base_sha")
    )


def run_process_label(run: dict) -> str:
    if run.get("retained_for_rework"):
        return "retained"
    alive, _ = worker.process_state(run)
    return "running" if alive else "stopped"


def latest_verdict(run: dict) -> dict | None:
    path = Path(run["run_dir"]) / "verdict.json"
    if not path.is_file():
        return None
    try:
        verdict = json.loads(path.read_text())
    except (OSError, ValueError):
        return None
    return verdict if isinstance(verdict, dict) else None


def run_worker_state(run: dict, ticket: tracker.Ticket | None = None, pane_text: str | None = None) -> str:
    if ticket and ticket.status == "resolved":
        return "landed"
    worktree = Path(run["worktree"])
    if not worktree.exists():
        return "stopped"
    alive, exit_code = worker.process_state(run)
    if pane_text is None and alive:
        pane_text = tmux_pane_text(pane_target(run))
    if alive and pane_blocked_prompt(pane_text):
        return "blocked-on-prompt"
    verdict = latest_verdict(run)
    if verdict and verdict.get("ready_for_review") is False:
        if verdict.get("verdict") == "RUNNING":
            return "running"
        return "not-ready"
    status = clean_status(worktree)
    subject = worker.git(worktree, "log", "-1", "--format=%s", "HEAD", check=False).strip()
    marker = run.get("handoff_marker")
    needs_info = marker and subject == marker.replace("ready for review", "needs-info")
    if needs_info and not status.strip():
        return "needs-info"
    pending = worker.pending_delivery(run)
    head = worker.git(worktree, "rev-parse", "HEAD", check=False).strip()
    if pending:
        if head and head != (run.get("candidate") or {}).get("commit") and not status.strip() and subject != marker:
            return "waiting for re-integration"
        return "rework"
    if marker and subject == marker and not status.strip():
        if session_busy(run, alive, pane_text):
            return "finishing"
        reviewed = reviewed_candidate(run)
        return "rework" if reviewed == head and (run.get("candidates") or [{}])[-1].get("review") else "ready"
    reviewed = reviewed_candidate(run)
    if (
        reviewed
        and head
        and head != reviewed
        and not status.strip()
        and any(entry.get("review") for entry in run.get("candidates", []))
    ):
        return "waiting for re-integration"
    if alive:
        return "running"
    return "stopped"


def tmux_pane_text(target: str | None) -> str | None:
    if not target:
        return None
    proc, error = run_tmux("capture-pane", "-p", "-t", target)
    if proc is None or proc.returncode != 0:
        return None
    return proc.stdout


def pane_target(run: dict) -> str | None:
    lead = run.get("lead_launch") or {}
    window = lead.get("tmux_window") or run.get("tmux_window") or {}
    if not window.get("available"):
        return None
    return window.get("pane_id") or window.get("target")


def pane_blocked_prompt(text: str | None) -> dict | None:
    if not text:
        return None
    lines = [line.rstrip() for line in text.splitlines()]
    compact = [line.strip() for line in lines if line.strip()]
    lower = "\n".join(compact).lower()
    waits_for_key = (
        "enter continue" in lower
        or "esc quit" in lower
        or "press enter" in lower
        or "press any key" in lower
    )
    if "do you trust the contents of this directory" in lower and waits_for_key:
        return {"kind": "folder-trust", "text": "\n".join(compact[-8:])}
    approval_markers = ("approve", "approval", "allow", "deny")
    if waits_for_key and any(marker in lower for marker in approval_markers):
        return {"kind": "runtime-approval", "text": "\n".join(compact[-8:])}
    return None


def pane_is_busy(run: dict, text: str | None) -> bool:
    lower = (text or "").lower()
    runtime = run.get("runtime", "traecli")
    if runtime == "traecli":
        # Real TraeCLI interactive panes show "esc to interrupt" while a turn is running; idle panes keep
        # the input box and footer visible without that marker.
        return "esc to interrupt" in lower
    if runtime == "codex":
        return "esc to interrupt" in lower or "ctrl-c to interrupt" in lower
    if "idle after marker" in lower or "(idle)" in lower:
        return False
    busy_markers = ("esc to interrupt", "ctrl-c to interrupt", "running")
    return any(marker in lower for marker in busy_markers)


def session_busy(run: dict, alive: bool, pane: str | None = None) -> bool:
    if not alive:
        return False
    if not (run.get("interactive") or (run.get("lead_launch") or {}).get("interactive")):
        return True
    if pane is None:
        pane = tmux_pane_text(pane_target(run))
    return pane_is_busy(run, pane)


def tip_rewritten(repo: Path, anchor: str | None, head: str | None) -> bool:
    if not anchor or not head or anchor == head:
        return False
    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", anchor, head],
        capture_output=True,
    ).returncode != 0


def monitor_ticket_rows(root: Path, effort: str) -> list[dict]:
    _, tickets, problems = tracker.load_tickets(root, effort)
    tracker.ensure_clean(problems)
    runs = latest_run_by_ticket(effort)
    rows = []
    for ticket in sorted(tickets, key=lambda item: item.id):
        run = runs.get(ticket.id)
        row = {
            "ticket": ticket.id,
            "slug": ticket.slug,
            "tracker_status": ticket.status,
            "triage": ticket.triage,
            "worker_state": "landed" if ticket.status == "resolved" else "-",
            "ahead": 0,
            "branch_contains_integration": None,
            "dirty_count": 0,
            "last_commit_age": "",
            "waits_for": ticket.blocked_by,
            "candidate_tip_changed": False,
            "tip_rewritten_since_pinned_or_reviewed": False,
            "outside_declared_count": 0,
            "diff_since_reviewed_candidate_size": 0,
            "diff_since_reviewed_candidate": [],
            "outside_declared_paths": [],
            "blocked_prompt": None,
        }
        if run:
            worktree = Path(run["worktree"])
            pane_text = tmux_pane_text(pane_target(run))
            row["worker_state"] = run_worker_state(run, ticket, pane_text)
            prompt = pane_blocked_prompt(pane_text)
            if prompt:
                row["blocked_prompt"] = prompt["text"]
            if worktree.exists():
                integration = run.get("lead_launch", {}).get("integration")
                if integration:
                    integration_tip = worker_integration_tip(run, integration)
                    if integration_tip:
                        row["ahead"] = int(worker.git(worktree, "rev-list", "--count", f"{integration_tip}..HEAD", check=False).strip() or "0")
                        row["branch_contains_integration"] = (
                            subprocess.run(["git", "-C", str(worktree), "merge-base", "--is-ancestor", integration_tip, "HEAD"],
                                           capture_output=True, env=worker.git_env()).returncode == 0
                        )
                row["dirty_count"] = len(clean_status(worktree).splitlines())
                row["last_commit_age"] = worker.git(worktree, "log", "-1", "--format=%cr", "HEAD", check=False).strip()
                reviewed = reviewed_candidate(run)
                changed = diff_names(worktree, reviewed)
                row["diff_since_reviewed_candidate"] = changed
                row["outside_declared_paths"] = outside_allowed(changed, run.get("allowed", []))
                row["diff_since_reviewed_candidate_size"] = len(changed)
                row["outside_declared_count"] = len(row["outside_declared_paths"])
                candidate = (run.get("candidate") or {}).get("commit")
                head = worker.git(worktree, "rev-parse", "HEAD", check=False).strip()
                row["candidate_tip_changed"] = bool(candidate and head and candidate != head)
                row["tip_rewritten_since_pinned_or_reviewed"] = tip_rewritten(worktree, reviewed or candidate, head)
        rows.append(row)
    return rows


def cmd_monitor_tickets(args: argparse.Namespace) -> int:
    root = repo_root(args.repo_root)
    resolve_integration(root, args.effort, args.integration)
    record_root = effort_record_root(root, args.effort)
    while True:
        rows = monitor_ticket_rows(record_root, args.effort)
        if args.json:
            print(json.dumps({"effort": args.effort, "tickets": rows}, sort_keys=True))
            return OK
        print(f"{args.effort} tickets")
        for row in rows:
            print(
                f"{row['ticket']} {row['worker_state']} ahead={row['ahead']} dirty={row['dirty_count']} "
                f"outside={row['outside_declared_count']} diff={row['diff_since_reviewed_candidate_size']} "
                f"waits={','.join(row['waits_for']) or '-'} age={row['last_commit_age'] or '-'}"
            )
        if not args.interval:
            return OK
        time.sleep(args.interval)


def parse_pane_activity(text: str | None) -> tuple[str | None, str | None]:
    if not text:
        return None, None
    activity = None
    context = None
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    for line in lines:
        lower = line.lower()
        if lower.startswith("current activity:"):
            activity = line.split(":", 1)[1].strip()
        elif lower.startswith("context remaining:"):
            context = line.split(":", 1)[1].strip()
        elif (match := CONTEXT_RE.search(line)):
            context = match.group(1)
        elif "esc to interrupt" in lower or "ctrl-c to interrupt" in lower:
            activity = line
    if activity is None and lines:
        for line in reversed(lines):
            lower = line.lower()
            if lower.startswith("context remaining:") or CONTEXT_RE.search(line):
                continue
            activity = line
            break
    return activity, context


def last_delivery(run: dict) -> dict | None:
    deliveries = run.get("rework_deliveries") or []
    if not deliveries:
        return None
    delivery = deliveries[-1]
    receipt = delivery.get("receipt") or {}
    return {
        "id": delivery.get("id"),
        "kind": delivery.get("kind"),
        "candidate": delivery.get("candidate"),
        "delivered_at": delivery.get("delivered_at"),
        "sent_at": receipt.get("sent_at") or delivery.get("delivered_at"),
        "acknowledged": worker.delivery_acknowledged(delivery),
        "receipt": receipt,
    }


def monitor_worker_rows(effort: str) -> list[dict]:
    rows = []
    for run in all_lead_runs(effort):
        alive, exit_code = worker.process_state(run)
        pane_text = tmux_pane_text(pane_target(run))
        activity, context = parse_pane_activity(pane_text)
        state = run_worker_state(run, pane_text=pane_text)
        rows.append({
            "ticket": run.get("lead_launch", {}).get("ticket"),
            "run_dir": run.get("run_dir"),
            "worktree": run.get("worktree"),
            "state": state,
            "process": {
                "pid": run.get("pid"),
                "alive": alive,
                "exit_code": exit_code,
                "state": "running" if alive else "stopped",
                "uptime_seconds": max(0, int(time.time() - float(run.get("started_epoch", time.time())))),
            },
            "current_activity": activity,
            "context_remaining": context,
            "pane_target": pane_target(run),
            "disk_usage": worker.disk_usage(run),
            "last_delivery": last_delivery(run),
        })
    return sorted(rows, key=lambda row: (row.get("ticket") or "", row.get("run_dir") or ""))


def cmd_monitor_workers(args: argparse.Namespace) -> int:
    effort_record_root(repo_root(args.repo_root), args.effort)
    while True:
        rows = monitor_worker_rows(args.effort)
        if args.json:
            print(json.dumps({"effort": args.effort, "workers": rows}, sort_keys=True))
            return OK
        print(f"{args.effort} workers")
        for row in rows:
            delivery = row["last_delivery"]["id"] if row["last_delivery"] else "-"
            print(
                f"{row['ticket'] or '-'} {row['state']} pid={row['process']['pid'] or '-'} "
                f"up={row['process']['uptime_seconds']}s context={row['context_remaining'] or '-'} "
                f"delivery={delivery}"
            )
            if row["current_activity"]:
                print(f"  {row['current_activity']}")
        if not args.interval:
            return OK
        time.sleep(args.interval)
    return OK


def accounting_rows(effort: str) -> list[dict]:
    rows = []
    for run in all_lead_runs(effort):
        state = run_process_label(run)
        if state == "stopped":
            worker.cleanup_run_temp(run)
        rows.append({
            "ticket": run.get("lead_launch", {}).get("ticket"),
            "run_dir": run["run_dir"],
            "worktree": run["worktree"],
            "pid": run.get("pid"),
            "process_state": state,
            "slot_held": state in {"running", "retained"},
            "retained_for_rework": run.get("retained_for_rework"),
            "disk_usage": worker.disk_usage(run),
        })
    return sorted(rows, key=lambda row: (row.get("ticket") or "", row["run_dir"]))


def cmd_account(args: argparse.Namespace) -> int:
    effort_record_root(repo_root(args.repo_root), args.effort)
    rows = accounting_rows(args.effort)
    held = sum(1 for row in rows if row["slot_held"])
    report = {"effort": args.effort, "parallel_cap": args.parallel_cap, "available_slots": max(0, args.parallel_cap - held), "workers": rows}
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print(f"{args.effort} cap={args.parallel_cap} available={report['available_slots']}")
        for row in rows:
            print(f"{row['ticket'] or '-'} {row['process_state']} pid={row['pid'] or '-'} worker_repo={row['disk_usage']['worker_repo_bytes']}B")
    return OK


def cmd_retain_worker(args: argparse.Namespace) -> int:
    run = worker.load_run(args.run_dir)
    effort = (run.get("lead_launch") or {}).get("effort")
    if effort:
        effort_record_root(repo_root(args.repo_root), effort)
    run["retained_for_rework"] = {"reason": args.reason, "retained_at": worker.now()}
    worker.save_run(run)
    print(json.dumps({"run_dir": str(Path(args.run_dir)), "retained": True, "reason": args.reason}, sort_keys=True))
    return OK


def cmd_wait(args: argparse.Namespace) -> int:
    root = repo_root(args.repo_root)
    resolve_integration(root, args.effort, args.integration)
    record_root = effort_record_root(root, args.effort)
    running_states = {"running", "finishing", "rework"}
    deadline = time.time() + args.timeout
    states: dict[str, str] = {}
    triggered = None
    while True:
        rows = {row["ticket"]: row for row in monitor_ticket_rows(record_root, args.effort)}
        states = {ticket: rows.get(ticket, {"worker_state": "missing"})["worker_state"] for ticket in args.tickets}
        for ticket, state in states.items():
            if state not in running_states:
                triggered = {"ticket": ticket, "state": state, "reason": f"left running states: {state}"}
                prompt = rows.get(ticket, {}).get("blocked_prompt")
                if prompt:
                    triggered["prompt"] = prompt
                break
        if triggered or time.time() >= deadline:
            break
        time.sleep(args.interval)
    result = {"triggered": triggered is not None, "trigger": triggered, "states": states}
    print(json.dumps(result, sort_keys=True) if args.json else result)
    return OK if triggered else REFUSED


def monitor_loop_command(repo: Path, effort: str, kind: str, interval: float) -> str:
    command = [
        sys.executable,
        str(Path(__file__).resolve()),
        "--repo",
        str(repo),
        f"monitor-{kind}",
        effort,
        "--interval",
        f"{interval:g}",
    ]
    return shlex.join(command)


def cmd_open_session(args: argparse.Namespace) -> int:
    repo = repo_root(args.repo_root)
    effort_record_root(repo, args.effort)
    tickets = monitor_loop_command(repo, args.effort, "tickets", args.interval)
    workers = monitor_loop_command(repo, args.effort, "workers", args.interval)
    first, error = run_tmux("new-session", "-d", "-s", args.tmux_session, "-n", "tickets", "-c", str(repo), tickets)
    if first is None:
        raise WorkerError(error)
    if first.returncode != 0:
        raise WorkerError((first.stderr or first.stdout).strip())
    second, error = run_tmux("new-window", "-d", "-t", f"{args.tmux_session}:", "-n", "workers", "-c", str(repo), workers)
    if second is None:
        raise WorkerError(error)
    if second.returncode != 0:
        raise WorkerError((second.stderr or second.stdout).strip())
    print(json.dumps({"session": args.tmux_session, "windows": ["tickets", "workers"]}, sort_keys=True))
    return OK


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", dest="repo_root", default=".", help="repository root or integration worktree")
    sub = parser.add_subparsers(dest="cmd", required=True)
    launch = sub.add_parser("launch", epilog=worker.RUNTIME_HELP, formatter_class=argparse.RawDescriptionHelpFormatter)
    launch.add_argument("effort")
    launch.add_argument("tickets", nargs="+")
    launch.add_argument("--integration", help="override the integration branch named in the effort record")
    launch.add_argument("--parallel-cap", type=int, default=3)
    launch.add_argument("--owner-prefix", default="worker")
    launch.add_argument("--ttl-hours", type=float, default=48.0)
    launch.add_argument("--name", help="worker name for a single-ticket launch")
    launch.add_argument("--model", default="GPT-5.5")
    launch.add_argument("--reasoning-effort", default="xhigh")
    launch.add_argument("--runtime", choices=worker.RUNTIMES, default="traecli",
                        help="worker runtime to launch (default: traecli)")
    launch.add_argument("--sandbox", default="workspace-write")
    launch.add_argument("--network", choices=("on", "off"), default="on")
    launch.add_argument("--command-timeout", type=int, default=120)
    launch.add_argument("--external-add-path", action="append", default=[])
    launch.add_argument("--external-protect-path", action="append", default=[])
    launch.add_argument("--worker-cap", type=int, default=2700)
    launch.add_argument("--stall", type=int, default=300)
    launch.add_argument("--keep-temp", action="store_true", help="keep each worker's run temp directory after it stops")
    launch.add_argument("--protect", action="append", default=[])
    launch.add_argument("--interactive", action="store_true",
                        help="launch the runtime in an interactive tmux window instead of detached exec JSON mode")
    launch.add_argument(
        "--tmux-session",
        help=(
            "named tmux session. Non-interactive launches use it for a watch window; interactive launches "
            "require it and use the user's default tmux server unless TMUX_TMPDIR is set."
        ),
    )
    launch.set_defaults(fn=cmd_launch)
    question = sub.add_parser("question")
    question.add_argument("effort")
    question.add_argument("ticket")
    question.add_argument("--owner", required=True)
    question.add_argument("--question", required=True)
    question.set_defaults(fn=cmd_question)
    rework = sub.add_parser("rework")
    rework.add_argument("run_dir")
    rework.add_argument("--findings-file", required=True)
    rework.add_argument("--kind", choices=("review", "stale-base"), default="review")
    rework.add_argument("--ack-timeout", type=float, default=2.0,
                        help="seconds to wait for a worker-side acknowledgement after terminal delivery")
    rework.set_defaults(fn=cmd_rework)
    overlap = sub.add_parser("overlap", help="report read-only overlap state for running tickets in an effort")
    overlap.add_argument("effort")
    overlap.add_argument("--integration", help="override the integration branch named in the effort record")
    overlap.set_defaults(fn=cmd_overlap)
    monitor = sub.add_parser("monitor-tickets")
    monitor.add_argument("effort")
    monitor.add_argument("--integration")
    monitor.add_argument("--json", action="store_true")
    monitor.add_argument("--interval", type=float, default=0.0)
    monitor.set_defaults(fn=cmd_monitor_tickets)
    workers = sub.add_parser("monitor-workers")
    workers.add_argument("effort")
    workers.add_argument("--json", action="store_true")
    workers.add_argument("--interval", type=float, default=0.0)
    workers.set_defaults(fn=cmd_monitor_workers)
    wait = sub.add_parser("wait")
    wait.add_argument("effort")
    wait.add_argument("tickets", nargs="+")
    wait.add_argument("--integration")
    wait.add_argument("--timeout", type=float, default=3600.0)
    wait.add_argument("--interval", type=float, default=5.0)
    wait.add_argument("--json", action="store_true")
    wait.set_defaults(fn=cmd_wait)
    account = sub.add_parser("account")
    account.add_argument("effort")
    account.add_argument("--parallel-cap", type=int, default=3)
    account.add_argument("--json", action="store_true")
    account.set_defaults(fn=cmd_account)
    retain = sub.add_parser("retain-worker")
    retain.add_argument("run_dir")
    retain.add_argument("--reason", required=True)
    retain.set_defaults(fn=cmd_retain_worker)
    open_session = sub.add_parser("open-session")
    open_session.add_argument("effort")
    open_session.add_argument("--tmux-session", required=True)
    open_session.add_argument("--interval", type=float, default=20.0)
    open_session.set_defaults(fn=cmd_open_session)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        return args.fn(args)
    except Refusal as error:
        print(f"refused: {error}", file=sys.stderr)
        return REFUSED
    except WorkerError as error:
        print(f"lead-launch: {error}", file=sys.stderr)
        return USAGE


if __name__ == "__main__":
    raise SystemExit(main())
