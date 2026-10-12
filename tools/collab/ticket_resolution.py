"""Tracker-owned closure in an attributed post-landing bookkeeping commit."""
from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess
import sys
import uuid

from common import Refused, child_environment, failpoint, now
from ticket_source import TRACKER, split_corenius_ticket


class TicketResolution:
    def __init__(self, store, workspace, tickets, config):
        self.store = store
        self.workspace = workspace
        self.tickets = tickets
        self.config = config

    def latest(self, aid: str, kind: str) -> dict | None:
        events = self.store.row(aid).get("events", [])
        return next((event["details"] for event in reversed(events) if event["kind"] == kind), None)

    def prepare_identity(self, attempt: dict) -> dict | None:
        source = split_corenius_ticket(attempt["ticket"])
        override = self.latest(attempt["id"], "mainline_override")
        if not source and not (override and override.get("effort")):
            return None
        existing = self.latest(attempt["id"], "records_identity")
        if existing:
            return existing["environment"]
        environment = {}
        for role in ("AUTHOR", "COMMITTER"):
            identity = self.workspace.git("var", f"GIT_{role}_IDENT").strip()
            match = re.fullmatch(r"(.+) <([^<>]+)> \d+ [+-]\d{4}", identity)
            if not match:
                raise Refused("Bookkeeping commits require a valid repository commit identity")
            environment[f"GIT_{role}_NAME"] = match[1]
            environment[f"GIT_{role}_EMAIL"] = match[2]
        with self.store.tx() as con:
            self.store.event(con, attempt["id"], "records_identity", environment=environment,
                             subject=attempt["integration"])
        return environment

    def prepare_review(self, attempt: dict, review: object) -> None:
        source = split_corenius_ticket(attempt["ticket"])
        if not source:
            return
        if not isinstance(review, dict):
            raise Refused("Corenius landing requires --review-json with reviewer, both axes, and criteria")
        _, raw = self.tickets.ticket(attempt["ticket"], attempt["base"])
        acceptance = raw.decode().split("## Acceptance", 1)[1].split("\n## ", 1)[0]
        count = len(re.findall(r"(?m)^- \[[ xX]\] ", acceptance))
        criteria = review.get("criteria")
        valid = (
            isinstance(review.get("reviewer"), str)
            and bool(review["reviewer"].strip())
            and review.get("standards") == "passed"
            and review.get("spec") == "passed"
            and isinstance(criteria, list)
            and all(type(item) is int for item in criteria)
            and sorted(criteria) == list(range(1, count + 1))
            and count > 0
        )
        if not valid:
            raise Refused("Review must pass both axes and explicitly cover every acceptance criterion")
        with self.store.tx() as con:
            self.store.event(
                con, attempt["id"], "ticket_review", review=review,
                subject=attempt["integration"], ticket_hash=attempt["ticket_hash"],
            )

    def finish(self, attempt: dict) -> None:
        source = split_corenius_ticket(attempt["ticket"])
        override = self.latest(attempt["id"], "mainline_override")
        export_effort = override.get("effort") if override else None
        if not source and not export_effort:
            return
        identity = self.prepare_identity(attempt)
        receipt_ref = f"refs/collab/records/{attempt['id']}"
        existing = self.workspace.git("rev-parse", "--verify", receipt_ref, check=False).strip()
        prepared = self.latest(attempt["id"], "records_prepared")
        if existing:
            if not prepared or prepared.get("commit") != existing:
                raise Refused("Unattributed or conflicting ticket-record receipt")
            self._land_records(attempt, existing)
            return
        if self.workspace.trunk() != attempt["integration"]:
            raise Refused("Trunk moved before tracker records; preserve and inspect the pending operation")
        self.workspace.primary()
        if source:
            self.tickets.same_contract(attempt, attempt["integration"])
            review = self.latest(attempt["id"], "ticket_review")
            if not review or review["subject"] != attempt["integration"]:
                raise Refused("Missing review bound to the landed Integration commit")
        else:
            review = None
        path = Path(self.config()["workspace_root"]) / f"records-{attempt['id']}-{uuid.uuid4().hex[:8]}"
        root = Path(self.config()["root"])
        self.workspace.git("clone", "--no-local", "--single-branch", str(root), str(path), cwd=root)
        self.workspace.git("remote", "remove", "origin", cwd=path)
        if self.workspace.git("rev-parse", "HEAD", cwd=path).strip() != attempt["integration"]:
            raise Refused("Record workspace base changed during capture")
        effort = source[0] if source else export_effort
        evidence = path / "docs" / "efforts" / effort / "evidence"
        evidence.mkdir(parents=True, exist_ok=True)
        if source:
            self._write_acceptance(attempt, source, review, evidence)
            failpoint("before_tracker_resolution")
            command = [sys.executable, str(TRACKER), "--repo", str(path), "resolve", source[0], source[1],
                       "--owner", attempt["owner"], "--mark-reviewed"]
            result = subprocess.run(command, cwd=path, env=child_environment(), text=True, capture_output=True)
            if result.returncode:
                raise Refused(result.stderr.strip() or result.stdout.strip() or "Tracker resolution failed")
        if override:
            self._append(evidence / "mainline-landing-overrides.jsonl", {
                "schema": "corenius.mainline_landing_override.v1",
                "effort": effort, "ticket": source[1] if source else attempt["ticket"],
                "candidate": attempt["integration"], "target": self.config()["trunk_ref"],
                "owner": override["owner"], "recorded_at": now(),
            })
        self.workspace.git("add", "--", f"docs/efforts/{effort}/", cwd=path)
        message = f"Record acceptance for {attempt['ticket']}\n\nCollab-Records: {attempt['id']}"
        self.workspace.git("commit", "-m", message, cwd=path, env=identity)
        commit = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
        self.workspace.git("fetch", "--no-tags", str(path), commit)
        with self.store.tx() as con:
            self.store.event(con, attempt["id"], "records_prepared", commit=commit, path=str(path),
                             predecessor=attempt["integration"])
        self.workspace.git("update-ref", receipt_ref, commit, "")
        failpoint("before_records_land")
        self._land_records(attempt, commit)

    def _land_records(self, attempt: dict, commit: str) -> None:
        if self.workspace.git("rev-parse", commit + "^").strip() != attempt["integration"]:
            raise Refused("Record commit has an unexpected predecessor")
        target = self.workspace.trunk()
        if target == attempt["integration"]:
            self.workspace.primary()
            self.workspace.git("merge", "--ff-only", "--no-edit", "--no-overwrite-ignore", commit,
                               env={"GIT_REFLOG_ACTION": f"collab-records:{attempt['id']}"})
        elif self.workspace.git("merge-base", commit, target, check=False).strip() != commit:
            raise Refused("Trunk conflicts with pending tracker record commit")
        failpoint("after_records_land")
        with self.store.tx() as con:
            self.store.event(con, attempt["id"], "records_completed", commit=commit)

    def _write_acceptance(self, attempt: dict, source: tuple, review: dict, evidence: Path) -> None:
        record = {
            "schema": "corenius.acceptance.v1", "effort": source[0], "ticket": source[1],
            "candidate": attempt["integration"], "accepted": True,
            "axes": {"standards": "passed", "spec": "passed"},
            "reviewer": review["review"]["reviewer"],
            "reviewer_context": review["review"].get("reviewer_context", "lead"),
            "required_checks": [{"name": "collab_check", "result": "passed"}],
            "check_receipt": attempt["check_receipt"], "operator_review": review["review"],
            "recorded_at": now(), "owner_waiver": None,
        }
        self._append(evidence / f"acceptance-{source[1]}.jsonl", record)
        self._append(evidence / f"landing-{source[1]}.jsonl", {
            "schema": "corenius.ticket_landing.v1", "effort": source[0], "ticket": source[1],
            "candidate": attempt["integration"], "attempt": attempt["id"], "recorded_at": now(),
        })

    @staticmethod
    def _append(path: Path, record: dict) -> None:
        with path.open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(record, sort_keys=True) + "\n")
