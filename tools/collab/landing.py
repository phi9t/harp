"""Landing, recovery, abandonment, and terminal cleanup for collab Attempts."""
from __future__ import annotations

from pathlib import Path

from common import Refused, digest, failpoint
from state_store import UNRESOLVED_LANDING


class LandingManager:
    def __init__(self, store, workspace, tickets, config):
        self.store = store
        self.workspace = workspace
        self.tickets = tickets
        self.config = config

    def finish_landing(self, attempt: dict) -> None:
        ref = f"refs/collab/landed/{attempt['id']}"
        old = self.workspace.git("rev-parse", "--verify", ref, check=False).strip()
        if old and old != attempt["integration"]:
            raise Refused("Conflicting landing receipt ref")
        if not old:
            self.workspace.git("update-ref", ref, attempt["integration"], "")
        self.store.change(attempt["id"], "landed")

    def require_receipt(self, attempt: dict) -> None:
        receipt = attempt["check_receipt"]
        expected = {
            "subject": attempt["integration"],
            "base": attempt["base"],
            "candidate": attempt["candidate"],
            "check_path": attempt["check_path"],
            "log": attempt["log"],
            "trunk": attempt["base"],
            "argv": self.config()["check_argv"],
            "exit_code": 0,
            "clean": True,
            "same_head": True,
            "identity": True,
            "passed": True,
        }
        if not receipt or any(receipt.get(key) != value for key, value in expected.items()):
            raise Refused("Missing/mismatched durable check receipt; submit/check again")
        if digest(Path(attempt["log"]).read_bytes()) != receipt.get("log_sha256"):
            raise Refused("Check log changed; durable receipt no longer matches")

    def land(self, aid: str) -> dict:
        # This explicit human operator action is the manual review/approval gate.
        with self.store.lock("lane"), self.store.lock("meta"):
            self.store.no_uncertain_landing()
            attempt = self.store.row(aid)
            if attempt["state"] != "verified":
                raise Refused("Exact integration candidate has not passed checking")
            self.require_receipt(attempt)
            self.workspace.primary()
            if self.workspace.trunk() != attempt["base"]:
                raise Refused("Trunk moved; stop/abandon, claim fresh, cherry-pick contribution, submit/check again")
            self.tickets.same_contract(attempt, attempt["base"])
            if self.workspace.git("rev-parse", attempt["integration"] + "^").strip() != attempt["base"]:
                raise Refused("Integration candidate must have the expected parent")
            self.store.change(aid, "landing_prepared")
            failpoint("before_git_land")
            self.store.change(aid, "landing_started")
            failpoint("after_land_started")
            # Unlike update-ref on a checked-out branch, this safely updates index/files.
            self.workspace.git(
                "merge",
                "--ff-only",
                "--no-edit",
                "--no-overwrite-ignore",
                attempt["integration"],
                env={"GIT_REFLOG_ACTION": f"collab-land:{aid}"},
            )
            failpoint("after_git_land")
            if self.workspace.trunk() != attempt["integration"]:
                raise Refused("Unexpected target after merge; reconcile")
            self.finish_landing(attempt)
        return self.store.row(aid)

    def recover(self, aid: str, confirmed: bool) -> dict:
        if not confirmed:
            raise Refused("Requires --confirm-stopped: inspect old CLI/Git/check processes first")
        with self.store.lock("lane"), self.store.lock("meta"):
            attempt = self.store.row(aid)
            if attempt["state"] in UNRESOLVED_LANDING:
                self._recover_landing(attempt, aid)
            elif attempt["state"] == "allocating":
                self._recover_allocation(attempt, aid)
            elif attempt["state"] == "checking":
                self.store.change(aid, "check_failed")
            else:
                return {
                    "attempt": aid,
                    "state": attempt["state"],
                    "action": "none; resume existing work, do not relaunch automatically",
                }
        return self.store.row(aid)

    def _recover_landing(self, attempt: dict, aid: str) -> None:
        self.workspace.primary()
        self.require_receipt(attempt)
        log = self.workspace.git("reflog", "show", "--format=%H%x00%gs", self.config()["trunk_ref"])
        entries = log.splitlines()
        attributed = any(
            line.startswith(attempt["integration"] + "\0collab-land:" + aid + ":")
            and index + 1 < len(entries)
            and entries[index + 1].split("\0", 1)[0] == attempt["base"]
            for index, line in enumerate(entries)
        )
        target = self.workspace.trunk()
        landed_after_start = (
            attempt["state"] != "landing_prepared"
            and attributed
            and self.workspace.git("merge-base", attempt["integration"], target).strip() == attempt["integration"]
        )
        if landed_after_start:
            self.finish_landing(attempt)
        elif attempt["state"] == "landing_prepared" and target == attempt["base"] and not attributed:
            self.store.change(aid, "verified")
        else:
            raise Refused("Ambiguous landing; preserve state and inspect manually")

    def _recover_allocation(self, attempt: dict, aid: str) -> None:
        path = Path(attempt["path"])
        if not path.exists():
            # No deletion or new execution. Claim stays reserved pending operator choice.
            raise Refused("Workspace not present; explicitly abandon before a new claim")
        self.workspace.validate_attempt(attempt, expected_head=attempt["base"])
        self.store.change(aid, "working")

    def abandon(self, aid: str, confirmed: bool) -> dict:
        if not confirmed:
            raise Refused("Requires --confirm-stopped; this command does not stop processes")
        with self.store.lock("lane"), self.store.lock("meta"):
            attempt = self.store.row(aid)
            if attempt["state"] in (*UNRESOLVED_LANDING, "landed", "abandoned"):
                raise Refused("Cannot abandon this state; reconcile landing first")
            self.store.change(aid, "abandoned")
        return self.store.row(aid)

    def gc(self, aid: str, confirmed: bool) -> dict:
        if not confirmed:
            raise Refused("Requires --confirm-stopped; no process supervision is provided")
        with self.store.lock("lane"), self.store.lock("meta"):
            attempt = self.store.row(aid)
            if attempt["state"] not in ("landed", "abandoned"):
                raise Refused("Only terminal attempts may be cleaned")
            removed = []
            paths = []
            for name in ("path", "check_path"):
                if not attempt[name]:
                    continue
                path = Path(attempt[name])
                if not path.exists():
                    continue
                if name == "check_path" and not attempt["integration"]:
                    raise Refused("Workspace identity unknown for incomplete check; preserve it manually")
                expected_head = attempt["integration"] if name == "check_path" else attempt["candidate"]
                self.workspace.validate_attempt(attempt, name, expected_head)
                self.workspace.clean(path, include_ignored=True)
                paths.append(path)
            # Validate every workspace before unlocking or removing any of them.
            for path in paths:
                self.workspace.git("worktree", "unlock", str(path))
                try:
                    self.workspace.git("worktree", "remove", str(path))
                except Refused:
                    self.workspace.git("worktree", "lock", "--reason", f"collab:{aid}", str(path), check=False)
                    raise
                removed.append(str(path))
            # Keep branches and sealed refs: no hidden destructive pruning.
            with self.store.tx() as con:
                self.store.event(con, aid, "cleanup", removed=removed)
        return {
            "removed": removed,
            "retained": "branches, candidate/check refs, logs, database history",
        }
