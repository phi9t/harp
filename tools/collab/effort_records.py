"""Optional evidence finalization for existing efforts with separate record branches.

Core collab landing and in-tree effort records do not depend on this extension.
"""
from __future__ import annotations

import json
from pathlib import Path

from landing_protocol import guard_initialized_trunk_mutation
from worker import WorkerError, git, now


def record_mainline_override(
    checkout: Path, effort: str, ticket: str, candidate: str, target: str, owner: str,
    *, landing_id: str | None = None,
) -> Path:
    path = checkout / "docs" / "efforts" / effort / "evidence" / "mainline-landing-overrides.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "schema": "corenius.mainline_landing_override.v1",
        "effort": effort,
        "ticket": ticket,
        "candidate": candidate,
        "target": target,
        "owner": owner,
        "recorded_at": now(),
    }
    if landing_id:
        record["landing_id"] = landing_id
    with path.open("a") as handle:
        handle.write(json.dumps(record, sort_keys=True) + "\n")
    return path


def record_landing(
    checkout: Path, effort: str, ticket: str, candidate: str, target: str, *, landing_id: str | None = None,
) -> Path:
    path = checkout / "docs" / "efforts" / effort / "evidence" / f"landing-{ticket}.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "schema": "corenius.ticket_landing.v1",
        "effort": effort,
        "ticket": ticket,
        "candidate": candidate,
        "target": target,
        "landed_at": now(),
    }
    if landing_id:
        record["landing_id"] = landing_id
    with path.open("a") as handle:
        handle.write(json.dumps(record, sort_keys=True) + "\n")
    return path


def commit_effort_record_change(
    checkout: Path, message: str, paths: list[Path], *, except_legacy: str | None = None,
) -> None:
    with guard_initialized_trunk_mutation(checkout, except_legacy=except_legacy):
        relative = [str(path.resolve().relative_to(checkout.resolve())) for path in paths]
        git(checkout, "add", *relative)
        if git(checkout, "diff", "--cached", "--name-only", "--", *relative).strip():
            git(checkout, "commit", "-q", "-m", message, "--", *relative)


def finish_landing_records(operation: dict) -> None:
    """Repair only record outputs bound by an exact compatibility landing journal."""
    from legacy_landing import common_git_dir, local_mainline_ref, worktree_for_ref
    repo = Path(operation["repo"])
    checkout = Path(operation["record_checkout"])
    branch = operation["record_branch"]
    if (
        worktree_for_ref(repo, local_mainline_ref(branch)) != checkout
        or common_git_dir(checkout) != common_git_dir(repo)
        or checkout.resolve() != checkout
    ):
        raise WorkerError("Effort record checkout identity changed; preserve and inspect")
    effort, ticket = operation["effort"], operation["ticket"]
    tip, target = operation["tip"], operation["target_branch"]
    path = checkout / "docs" / "efforts" / effort / "evidence" / f"landing-{ticket}.jsonl"
    identity = operation["id"]
    _append_once(
        path, identity, lambda: record_landing(checkout, effort, ticket, tip, target, landing_id=identity),
    )
    commit_effort_record_change(checkout, f"Record landing for ticket {ticket}", [path], except_legacy=identity)
    if operation.get("override_owner"):
        path = checkout / "docs" / "efforts" / effort / "evidence" / "mainline-landing-overrides.jsonl"
        _append_once(
            path, identity,
            lambda: record_mainline_override(
                checkout, effort, ticket, tip, target, operation["override_owner"], landing_id=identity,
            ),
        )
        commit_effort_record_change(
            checkout, f"Record mainline override for ticket {ticket}", [path], except_legacy=identity,
        )


def _append_once(path: Path, identity: str, append) -> None:
    records = [json.loads(line) for line in path.read_text().splitlines() if line] if path.exists() else []
    if not any(record.get("landing_id") == identity for record in records):
        append()
