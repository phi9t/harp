"""Tracker-owned closure in an attributed post-landing bookkeeping commit."""
from __future__ import annotations

import json
import os
from pathlib import Path
import re
import stat
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

    def prepare(self, attempt: dict) -> None:
        self.prepare_paths(attempt)
        self.prepare_identity(attempt)

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

    def prepare_paths(self, attempt: dict) -> None:
        source = split_corenius_ticket(attempt["ticket"])
        override = self.latest(attempt["id"], "mainline_override")
        effort = source[0] if source else override.get("effort") if override else None
        if not effort:
            return
        names = [f"acceptance-{source[1]}.jsonl", f"landing-{source[1]}.jsonl"] if source else []
        if override:
            names.append("mainline-landing-overrides.jsonl")
        for name in names:
            parts = ("docs", "efforts", effort, "evidence", name)
            for size in range(1, len(parts) + 1):
                relative = "/".join(parts[:size])
                entry = self.workspace.git("ls-tree", attempt["integration"], "--", relative).strip()
                if not entry:
                    continue
                mode = entry.split(" ", 1)[0]
                allowed = ("100644", "100755") if size == len(parts) else ("040000",)
                if mode not in allowed:
                    raise Refused(f"Bookkeeping path is symlinked or non-regular: {relative}")

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
        self.prepare_paths(attempt)
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
        if path.parent.resolve() != path.parent:
            raise Refused("Bookkeeping workspace parent identity changed")
        root = Path(self.config()["root"])
        self.workspace.git("clone", "--no-local", "--single-branch", str(root), str(path), cwd=root)
        self.workspace.git("remote", "remove", "origin", cwd=path)
        if self.workspace.git("rev-parse", "HEAD", cwd=path).strip() != attempt["integration"]:
            raise Refused("Record workspace base changed during capture")
        effort = source[0] if source else export_effort
        evidence = path / "docs" / "efforts" / effort / "evidence"
        if source:
            self._write_acceptance(attempt, source, review, evidence, path)
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
            }, path)
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

    def _write_acceptance(self, attempt: dict, source: tuple, review: dict, evidence: Path, root: Path) -> None:
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
        self._append(evidence / f"acceptance-{source[1]}.jsonl", record, root)
        self._append(evidence / f"landing-{source[1]}.jsonl", {
            "schema": "corenius.ticket_landing.v1", "effort": source[0], "ticket": source[1],
            "candidate": attempt["integration"], "attempt": attempt["id"], "recorded_at": now(),
        }, root)

    @staticmethod
    def _append(path: Path, record: dict, root: Path) -> None:
        if root.resolve() != root or not path.is_relative_to(root):
            raise Refused("Bookkeeping output must remain inside its original workspace")
        directory = None
        descriptor = None
        try:
            directory = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            parts = path.relative_to(root).parts
            if any(part in (".", "..") for part in parts):
                raise Refused("Bookkeeping output contains an invalid path component")
            for part in parts[:-1]:
                try:
                    os.mkdir(part, mode=0o755, dir_fd=directory)
                except FileExistsError:
                    pass
                next_directory = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=directory)
                os.close(directory)
                directory = next_directory
            flags = os.O_WRONLY | os.O_APPEND | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK
            descriptor = os.open(parts[-1], flags, 0o644, dir_fd=directory)
            info = os.fstat(descriptor)
            if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
                raise Refused("Bookkeeping evidence must be an unshared regular file")
            stream = os.fdopen(descriptor, "a", encoding="utf-8")
            descriptor = None
            with stream:
                stream.write(json.dumps(record, sort_keys=True) + "\n")
                stream.flush()
                os.fsync(stream.fileno())
        except OSError as exc:
            raise Refused(f"Bookkeeping path refused: {path}: {exc}") from exc
        finally:
            if descriptor is not None:
                os.close(descriptor)
            if directory is not None:
                os.close(directory)
