"""Carry a stopped Attempt's linear contribution onto a fresh trunk basis."""
from pathlib import Path

from common import Refused
from state_store import UNRESOLVED_LANDING


class RefreshManager:
    def __init__(self, store, workspace, tickets, open_fresh, require_stopped):
        self.store = store
        self.workspace = workspace
        self.tickets = tickets
        self.open_fresh = open_fresh
        self.require_stopped = require_stopped

    def refresh(self, aid: str, confirmed: bool) -> dict:
        if not confirmed:
            raise Refused("Requires --confirm-stopped; refresh does not stop workers")
        with self.store.lock("check"), self.store.lock("lane"), self.store.lock("meta"):
            self.store.no_uncertain_landing()
            old = self.store.row(aid)
            self.require_stopped(old)
            if old["state"] in (*UNRESOLVED_LANDING, "allocating", "checking", "landed", "abandoned"):
                raise Refused("Cannot refresh this state; recover or inspect it first")
            self.workspace.primary()
            path = self.workspace.validate_attempt(old)
            self.workspace.clean(path)
            head = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
            if self.workspace.git("merge-base", old["base"], head).strip() != old["base"]:
                raise Refused("Contribution no longer descends from the admitted base")
            if self.workspace.git("rev-list", "--merges", f"{old['base']}..{head}").strip():
                raise Refused("Refresh requires a linear contribution; preserve and inspect merge commits")
            trunk = self.workspace.trunk()
            commits = self.workspace.git(
                "rev-list", "--reverse", f"{old['base']}..{head}", "--not", trunk
            ).splitlines()
            self.tickets.require_refresh_claim(old, trunk)
            with self.store.tx() as con:
                self.store.set_state_with_event(con, aid, "abandoned")
                self.store.event(con, aid, "refresh_requested", trunk=trunk, head=head, commits=commits)
            # Keep admission and picks under the same operation locks. Transactions
            # remain short; no DB connection stays open across a cherry-pick.
            fresh = self.open_fresh(old, commits)
            new_path = Path(fresh["path"])
            for commit in commits:
                try:
                    self.workspace.git("cherry-pick", "--", commit, cwd=new_path)
                except Refused as exc:
                    # Never abort or resolve a failed pick: preserve index, sequencer,
                    # conflict markers and the already-applied prefix for the worker.
                    with self.store.tx() as con:
                        self.store.event(con, fresh["id"], "refresh_conflict", previous=aid,
                                         commit=commit, error=str(exc))
                    raise Refused(
                        f"Refresh stopped at {commit}; new Attempt {fresh['id']} at {new_path} retains "
                        f"the conflict and unapplied suffix. Original Attempt {aid} is preserved. {exc}"
                    ) from exc
            with self.store.tx() as con:
                self.store.event(con, fresh["id"], "refreshed", previous=aid, commits=commits)
        return self.store.row(fresh["id"])
