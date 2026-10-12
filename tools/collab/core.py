"""Collab lifecycle facade composed from narrow modules."""
from __future__ import annotations

import uuid
from pathlib import Path

from check_runner import CheckRunner
from common import Refused, failpoint, output_path, valid_branch_component, valid_id
from landing import LandingManager
from liveness import liveness_blockers, normalize_worker_binding
from policy import (
    protect_candidate_paths,
    validate_check_argv,
    validate_check_timeout,
    validate_protected_paths,
    validate_override_owners,
)
from state_store import StateStore
from sweep import SweepManager
from ticket_source import TicketSource, corenius_ticket_id
from ticket_resolution import TicketResolution
from workspace import RepositoryWorkspace, run_git


class Collab:
    def __init__(self, repo: str | Path):
        self.invoked = output_path(repo)
        common = run_git(self.invoked, "rev-parse", "--path-format=absolute", "--git-common-dir").strip()
        self.common = output_path(common)
        self.home = self.common / "collab"
        self.db_path = self.home / "state.sqlite3"
        self.store = StateStore(self.home, self.db_path)
        self.config = self.store.load_config()
        self.workspace = RepositoryWorkspace(self.invoked, self.common, self.store, self.config)
        self.tickets = TicketSource(self.workspace)
        self.checks = CheckRunner(self.store, self.workspace, self.tickets, self.require_config)
        self.landing = LandingManager(self.store, self.workspace, self.tickets, self.require_config)
        self.sweeper = SweepManager(self.store, self.workspace, self.require_config)

    def require_config(self) -> dict:
        if self.config is None:
            raise Refused("Run init first; never delete state.sqlite3 to reset active work")
        return self.config

    def init(
        self,
        trunk: str,
        check_argv: list[str],
        workspace_root: str | None = None,
        protected_paths: list[str] | None = None,
        check_timeout_seconds: float | None = None,
        mainline_override_owners: list[str] | None = None,
    ) -> dict:
        if self.db_path.exists():
            raise Refused("Already initialized or partial initialization; inspect, do not overwrite")
        check_argv = validate_check_argv(check_argv)
        protected_paths = validate_protected_paths(protected_paths)
        check_timeout_seconds = validate_check_timeout(check_timeout_seconds)
        mainline_override_owners = validate_override_owners(mainline_override_owners)
        root = Path(self.workspace.git("rev-parse", "--show-toplevel", cwd=self.invoked).strip())
        trunk_ref = "refs/heads/" + trunk
        self.workspace.git("check-ref-format", trunk_ref, cwd=root)
        if self.workspace.git("symbolic-ref", "-q", "HEAD", cwd=root).strip() != trunk_ref:
            raise Refused("Initialize from the intended trunk checkout")
        self.workspace.git("rev-parse", "--verify", trunk_ref + "^{commit}", cwd=root)
        self.workspace.clean(root)
        if workspace_root:
            work = output_path(workspace_root)
        else:
            work = root.parent / (root.name + "-collab-worktrees")
        if work == root or root in work.parents:
            raise Refused("Workspaces must be outside the primary checkout")
        config = {
            "schema": 1,
            "root": str(root),
            "trunk_ref": trunk_ref,
            "workspace_root": str(work),
            "check_argv": check_argv,
            "protected_paths": protected_paths,
            "check_timeout_seconds": check_timeout_seconds,
            "mainline_override_owners": mainline_override_owners,
        }
        self.store.initialize(config)
        self.config = config
        self.workspace.set_config(config)
        return config

    def new(self, tid: str, title: str, deps: list[str]) -> dict:
        self.require_config()
        valid_id(tid)
        deps = [valid_id(item) for item in deps]
        if len(deps) != len(set(deps)) or tid in deps:
            raise Refused("Duplicate or self dependency")
        with self.store.lock("meta"):
            self.workspace.primary()
            directory = Path(self.require_config()["root"]) / ".tickets"
            if directory.is_symlink():
                raise Refused("Ticket directory cannot be a symlink")
            directory.mkdir(exist_ok=True)
            path = directory / (tid + ".md")
            text = (
                f"---\nid: {tid}\nstatus: open\ndeps: [{', '.join(deps)}]\n---\n\n"
                f"# {title}\n\n"
                "## Outcome\nDescribe the outcome and why it matters.\n\n"
                "## Invariants\nList actual constraints, not implementation guesses.\n\n"
                "## Acceptance\nDescribe observable evidence.\n\n"
                "## Discretion\n"
                "You own the engineering strategy. Escalate changes to acceptance or authority.\n"
            )
            with path.open("x") as stream:
                stream.write(text)
        return {
            "path": str(path),
            "next": "Edit and explicitly git-add/commit this ticket on trunk before claiming",
        }

    def attempt_branch(self, owner: str, ticket_label: str, aid: str) -> str:
        agent = valid_branch_component("owner", owner)
        topic = valid_branch_component("topic", f"collab-{ticket_label}-{aid}")
        return f"{agent}/{topic}"

    def open_attempt(
        self,
        tid: str,
        owner: str,
        effort: str | None = None,
        ttl_hours: float = 24.0,
        worker_pid: int | str | None = None,
        worker_run_dir: str | None = None,
    ) -> dict:
        self.require_config()
        if not owner.strip():
            raise Refused("Owner required")
        if ttl_hours <= 0:
            raise Refused("--ttl-hours must be positive")
        worker = normalize_worker_binding(worker_pid, worker_run_dir)
        if effort:
            return self.open_corenius_attempt(tid, owner.strip(), effort, ttl_hours, worker)
        return self.open_standalone_attempt(tid, owner.strip(), worker)

    def open_standalone_attempt(self, tid: str, owner: str, worker: dict | None = None) -> dict:
        worker = worker or normalize_worker_binding(None, None)
        with self.store.lock("meta"):
            base = self.workspace.trunk()
            meta, _ = self.tickets.ticket(tid, base)
            if meta["status"] == "closed":
                raise Refused("Ticket is closed")
            self.tickets.deps_ok(tid, base)
            aid = "a-" + uuid.uuid4().hex[:16]
            branch = self.attempt_branch(owner, tid, aid)
            path = Path(self.require_config()["workspace_root"]) / aid
            with self.store.tx() as con:
                self.store.insert_attempt(con, aid, tid, owner, base, meta["sha256"], path, branch, **worker)
                self.store.event(con, aid, "allocating", ticket=tid, owner=owner, base=base, **worker)
            failpoint("after_claim_record")
            self.workspace.create_attempt_worktree(self.store.row(aid))
            failpoint("after_worktree_add")
            self.store.change(aid, "working")
        return self.store.row(aid)

    def open_corenius_attempt(
        self,
        tid: str,
        owner: str,
        effort: str,
        ttl_hours: float,
        worker: dict | None = None,
    ) -> dict:
        worker = worker or normalize_worker_binding(None, None)
        source_id = corenius_ticket_id(effort, tid)
        with self.store.lock("meta"):
            self.workspace.primary()
            self.tickets.require_corenius_ready(effort, tid)
            base = self.workspace.trunk()
            meta, _ = self.tickets.ticket(source_id, base)
            aid = "a-" + uuid.uuid4().hex[:16]
            branch = self.attempt_branch(owner, f"{effort}-{tid}", aid)
            path = Path(self.require_config()["workspace_root"]) / aid
            with self.store.tx() as con:
                self.store.insert_attempt(con, aid, source_id, owner, base, meta["sha256"], path, branch, **worker)
                self.store.event(con, aid, "allocating", ticket=source_id, owner=owner, base=base, **worker)
            failpoint("after_claim_record")
            claimed, _ = self.tickets.claim_corenius(effort, tid, owner, ttl_hours)
            failpoint("after_tracker_claim")
            self.store.update_claim_basis(aid, self.workspace.trunk(), claimed["sha256"])
            self.workspace.create_attempt_worktree(self.store.row(aid))
            failpoint("after_worktree_add")
            self.store.change(aid, "working")
        return self.store.row(aid)

    def submit(self, aid: str) -> dict:
        with self.store.lock("meta"):
            attempt = self.store.row(aid)
            if attempt["state"] not in ("working", "submitted", "check_failed", "verified"):
                raise Refused("Attempt cannot submit in its current state")
            path = self.workspace.validate_attempt(attempt)
            self.workspace.clean(path)
            assigned_branch = "refs/heads/" + attempt["branch"]
            if self.workspace.git("symbolic-ref", "-q", "HEAD", cwd=path).strip() != assigned_branch:
                raise Refused("Worker changed its assigned branch")
            base = self.workspace.trunk()
            candidate = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
            if base != attempt["base"]:
                raise Refused("Trunk moved; stop/abandon, claim fresh, and cherry-pick only the prior contribution")
            self.tickets.same_contract(attempt, base)
            if self.workspace.git("merge-base", base, candidate).strip() != base:
                raise Refused(
                    "Candidate diverges from its admitted base; stop/abandon, claim fresh, cherry-pick contribution"
                )
            if base == candidate:
                raise Refused("Empty contribution")
            if self.workspace.git("rev-list", "--merges", f"{base}..{candidate}").strip():
                raise Refused("Use a linear private branch for this prototype")
            changed = self.workspace.git("diff", "--name-only", "--no-renames", "-z", base, candidate).split("\0")
            if not any(changed):
                raise Refused("Empty contribution: candidate has no file changes")
            protect_candidate_paths(changed, self.require_config().get("protected_paths"))
            self.workspace.git("update-ref", f"refs/collab/candidates/{aid}/{candidate}", candidate)
            self.store.change(aid, "submitted", base=base, candidate=candidate, integration=None)
        return self.store.row(aid)

    def check(self, aid: str) -> dict:
        return self.checks.check(aid)

    def land(self, aid: str, owner_mainline_override: str | None = None,
             review: dict | None = None, effort: str | None = None,
             expected_tip: str | None = None, no_push: bool = False) -> dict:
        attempt = self.store.row(aid)
        resolver = TicketResolution(self.store, self.workspace, self.tickets, self.require_config)
        resolver.prepare_review(attempt, review)
        export_effort = attempt["ticket"].split(":", 1)[0] if ":" in attempt["ticket"] else effort
        if export_effort:
            valid_id(export_effort)
            entry = f"docs/efforts/{export_effort}/EFFORT.md"
            if not self.workspace.git("ls-tree", attempt["base"], "--", entry).startswith("100644 blob "):
                raise Refused("Override effort must be a committed regular effort record")
        if owner_mainline_override is not None:
            if owner_mainline_override not in self.require_config().get("mainline_override_owners", []):
                raise Refused("Mainline override owner is not configured")
            with self.store.tx() as con:
                self.store.event(
                    con, aid, "mainline_override", owner=owner_mainline_override,
                    subject=attempt["integration"], ticket=attempt["ticket"], effort=export_effort,
                )
        return self.landing.land(aid, expected_tip, no_push)

    def recover(self, aid: str, confirmed: bool) -> dict:
        return self.landing.recover(aid, confirmed)

    def abandon(self, aid: str, confirmed: bool) -> dict:
        return self.landing.abandon(aid, confirmed)

    def gc(self, aid: str, confirmed: bool) -> dict:
        return self.landing.gc(aid, confirmed)

    def sweep(
        self, apply: bool = False, archive_dirty: bool = False, include_foreign: bool = False,
        retention_days: float = 30,
    ) -> dict:
        return self.sweeper.sweep(apply, archive_dirty, include_foreign, retention_days)

    def restore_archive(self, archive: str, destination: str) -> dict:
        from archives import restore_archive

        return restore_archive(archive, destination)

    def inspect(self, aid: str) -> dict:
        self.require_config()
        attempt = self.store.row(aid)
        blockers = []
        blockers.extend(liveness_blockers(attempt))
        blockers.extend(self.store.busy_locks())
        blockers.extend(self.workspace.attempt_blockers(attempt))
        return {
            "attempt": aid,
            "state": attempt["state"],
            "blockers": blockers,
            "blocked": bool(blockers),
        }

    def status(self) -> dict:
        self.require_config()
        base = self.workspace.trunk()
        paths = self.workspace.git("ls-tree", "-r", "--name-only", base, "--", ".tickets/").splitlines()
        attempts = self.store.attempts()
        active = {
            attempt["ticket"]: attempt
            for attempt in attempts
            if attempt["state"] not in ("landed", "abandoned")
        }
        tickets = []
        for path in paths:
            if path.count("/") != 1 or not path.endswith(".md"):
                continue
            tid = Path(path).stem
            try:
                meta, _ = self.tickets.ticket(tid, base)
                blocker = None
                try:
                    self.tickets.deps_ok(tid, base)
                except Refused as exc:
                    blocker = str(exc)
                attempt = active.get(tid)
                tickets.append(
                    {
                        **meta,
                        "active_attempt": attempt["id"] if attempt else None,
                        "execution": attempt["state"] if attempt else None,
                        "ready": meta["status"] != "closed" and not blocker and not attempt,
                        "blocker": blocker,
                    }
                )
            except Refused as exc:
                tickets.append({"id": tid, "error": str(exc), "ready": False})
        return {"trunk": base, "tickets": tickets, "attempts": attempts}

    def backup(self, destination: str) -> dict:
        self.require_config()
        return self.store.backup(destination)
