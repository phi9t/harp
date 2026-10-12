"""Landing, recovery, abandonment, and terminal cleanup for collab Attempts."""
from __future__ import annotations

from pathlib import Path

from common import Refused, digest, failpoint
from liveness import require_worker_stopped
from state_store import StoreError, UNRESOLVED_LANDING
from ticket_resolution import TicketResolution


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
        TicketResolution(self.store, self.workspace, self.tickets, self.config).finish(attempt)
        self.store.change(attempt["id"], "landed")

    def require_receipt(self, attempt: dict) -> None:
        receipt = attempt["check_receipt"]
        config = self.config()
        expected = {
            "version": 2,
            "subject": attempt["integration"],
            "base": attempt["base"],
            "candidate": attempt["candidate"],
            "check_path": attempt["check_path"],
            "log": attempt["log"],
            "trunk": attempt["base"],
            "argv": config["check_argv"],
            "outcome": "passed",
            "exit_code": 0,
            "timeout_seconds": config.get("check_timeout_seconds"),
            "clean": True,
            "same_head": True,
            "identity": True,
            "passed": True,
        }
        if not receipt or any(receipt.get(key) != value for key, value in expected.items()):
            raise Refused("Missing/mismatched durable check receipt; submit/check again")
        if digest(Path(attempt["log"]).read_bytes()) != receipt.get("log_sha256"):
            raise Refused("Check log changed; durable receipt no longer matches")

    def land(self, aid: str, expected_tip: str | None = None, no_push: bool = False) -> dict:
        # This explicit human operator action is the manual review/approval gate.
        with self.store.lock("lane"), self.store.lock("meta"):
            self.store.no_uncertain_landing()
            attempt = self.store.row(aid)
            if attempt["state"] != "verified":
                raise Refused("Exact integration candidate has not passed checking")
            self._preflight(attempt, expected_tip)
            destination = None if no_push else self._push_destination()
            self.store.change(aid, "landing_prepared")
            failpoint("before_git_land")
            if destination:
                self.store.change_with_event(
                    aid,
                    "push_intended",
                    event_kind="push_intent",
                    event_details=self._push_details(attempt, destination),
                )
                failpoint("after_push_intent")
                self._push_checked_tip(aid, attempt, destination)
                self.store.change_with_event(
                    aid,
                    "push_completed",
                    event_kind="push_completed",
                    event_details=self._push_details(attempt, destination),
                )
                failpoint("after_git_push")
            self.store.change(aid, "landing_started")
            failpoint("after_land_started")
            self._fast_forward_primary(aid, attempt)
            result = self._finish_after_local_landing(attempt)
        return result

    def _preflight(self, attempt: dict, expected_tip: str | None) -> None:
        self.require_receipt(attempt)
        self.workspace.primary_identity()
        config = self.config()
        root = Path(config["root"])
        if self.workspace.git("symbolic-ref", "-q", "HEAD", cwd=root, check=False).strip() != config["trunk_ref"]:
            raise Refused("Primary checkout must be on configured trunk; no automatic switching")
        if self.workspace.trunk() != attempt["base"]:
            raise Refused("Trunk moved; stop/abandon, claim fresh, cherry-pick contribution, submit/check again")
        self.tickets.same_contract(attempt, attempt["base"])
        if self.workspace.git("rev-parse", attempt["integration"] + "^").strip() != attempt["base"]:
            raise Refused("Integration candidate must have the expected parent")
        if self.workspace.git("merge-base", attempt["base"], attempt["integration"]).strip() != attempt["base"]:
            raise Refused("Local fast-forward is not possible")
        self._require_expected_candidate_tip(attempt, expected_tip)
        self._refuse_operator_overlaps(root, attempt)
        self.workspace.clean(root)

    def _require_expected_candidate_tip(self, attempt: dict, expected_tip: str | None) -> None:
        if expected_tip:
            resolved = self.workspace.git(
                "rev-parse",
                "--verify",
                expected_tip + "^{commit}",
                check=False,
            ).strip()
            if not resolved:
                raise Refused("Expected Candidate tip does not name a commit")
            if resolved != attempt["candidate"]:
                raise Refused("Expected Candidate tip does not match the sealed Candidate")
        branch_tip = self.workspace.git(
            "rev-parse",
            "--verify",
            "refs/heads/" + attempt["branch"] + "^{commit}",
            check=False,
        ).strip()
        if not branch_tip:
            raise Refused("Candidate branch is missing; submit/check again")
        if branch_tip != attempt["candidate"]:
            raise Refused("Candidate branch moved since submit; submit/check again")

    def _changed_paths(self, attempt: dict) -> list[str]:
        raw = self.workspace.git(
            "diff",
            "--name-only",
            "--no-renames",
            "-z",
            attempt["base"],
            attempt["integration"],
        )
        return [path for path in raw.split("\0") if path]

    def _refuse_operator_overlaps(self, root: Path, attempt: dict) -> None:
        changed = self._changed_paths(attempt)
        if not changed:
            return
        status = self.workspace.git(
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignored",
            cwd=root,
        )
        blockers = []
        for entry in (item for item in status.split("\0") if item):
            if len(entry) < 4 or entry[:2] not in ("??", "!!"):
                continue
            path = entry[3:].rstrip("/")
            if any(self._paths_overlap(path, candidate) for candidate in changed):
                blockers.append(path)
        if blockers:
            preview = ", ".join(sorted(blockers)[:5])
            raise Refused("Untracked or ignored operator path would be overwritten by landing: " + preview)

    @staticmethod
    def _paths_overlap(left: str, right: str) -> bool:
        left = left.rstrip("/")
        right = right.rstrip("/")
        return left == right or left.startswith(right + "/") or right.startswith(left + "/")

    def _push_destination(self) -> dict | None:
        trunk_ref = self.config()["trunk_ref"]
        if not trunk_ref.startswith("refs/heads/"):
            raise Refused("Configured trunk must be a local branch")
        branch = trunk_ref[len("refs/heads/"):]
        remote = self.workspace.git("config", "--get", f"branch.{branch}.remote", check=False).strip()
        merge = self.workspace.git("config", "--get", f"branch.{branch}.merge", check=False).strip()
        if not remote or not merge or remote == ".":
            raise Refused("Configured trunk has no upstream; use --no-push for local-only landing")
        return {"remote": remote, "ref": merge}

    def _push_details(self, attempt: dict, destination: dict) -> dict:
        return {
            "base": attempt["base"],
            "candidate": attempt["candidate"],
            "integration": attempt["integration"],
            "remote": destination["remote"],
            "ref": destination["ref"],
        }

    def _push_checked_tip(self, aid: str, attempt: dict, destination: dict) -> None:
        result = self.workspace.git_result(
            "push",
            "--porcelain",
            destination["remote"],
            f"{attempt['integration']}:{destination['ref']}",
        )
        if result.returncode == 0:
            return
        output = self._push_output(result)
        cause = self._push_failure_cause(output)
        self.store.change_with_event(
            aid,
            "verified",
            event_kind="push_failed",
            event_details={**self._push_details(attempt, destination), "cause": cause, "output": output},
        )
        raise Refused(f"Push failed ({cause}): {output}")

    @staticmethod
    def _push_output(result) -> str:
        output = "\n".join(part.strip() for part in (result.stdout, result.stderr) if part.strip())
        return output[-1200:] if output else "git push exited non-zero without output"

    @staticmethod
    def _push_failure_cause(output: str) -> str:
        lower = output.lower()
        network = (
            "could not resolve hostname",
            "name or service not known",
            "network is unreachable",
            "connection timed out",
            "connection refused",
            "unable to access",
            "failed to connect",
        )
        if any(marker in lower for marker in network):
            return "network"
        if "non-fast-forward" in lower or "fetch first" in lower or "stale info" in lower:
            return "non-fast-forward"
        authentication = (
            "permission denied",
            "authentication failed",
            "could not read from remote repository",
            "publickey",
        )
        if any(marker in lower for marker in authentication):
            return "authentication"
        return "unknown"

    def _fast_forward_primary(self, aid: str, attempt: dict) -> None:
        root = Path(self.config()["root"])
        self._refuse_operator_overlaps(root, attempt)
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

    def _finish_after_local_landing(self, attempt: dict) -> dict:
        try:
            self.finish_landing(attempt)
            return self.store.row(attempt["id"])
        except (OSError, Refused, StoreError) as exc:
            message = "landed, records incomplete: " + str(exc)
            try:
                self.store.records_incomplete(attempt["id"], message)
            except (OSError, Refused, StoreError):
                pass
            result = dict(attempt)
            try:
                result = self.store.row(attempt["id"])
            except (OSError, Refused, StoreError):
                pass
            result.update({"state": "landed_records_incomplete", "landed": True, "records_error": message})
            return result

    def recover(self, aid: str, confirmed: bool) -> dict:
        if not confirmed:
            raise Refused("Requires --confirm-stopped: inspect old CLI/Git/check processes first")
        require_worker_stopped(self.store.row(aid))
        with self.store.lock("check"), self.store.lock("lane"), self.store.lock("meta"):
            attempt = self.store.row(aid)
            require_worker_stopped(attempt)
            if attempt["state"] in UNRESOLVED_LANDING:
                recovered = self._recover_landing(attempt, aid)
                if recovered:
                    return recovered
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

    def _recover_landing(self, attempt: dict, aid: str) -> dict | None:
        self.workspace.primary()
        self.require_receipt(attempt)
        if attempt["state"] in ("push_intended", "push_completed") or self.store.latest_event(aid, "push_intent"):
            return self._recover_pushed_landing(attempt, aid)
        if attempt["state"] == "landed_records_incomplete":
            return self._recover_landed_records(attempt)
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
        return None

    def _recover_pushed_landing(self, attempt: dict, aid: str) -> dict | None:
        destination = self.store.latest_event(aid, "push_intent") or self._push_destination()
        if not destination:
            raise Refused("Cannot recover pushed landing without a recorded remote destination")
        remote_tip = self._remote_tip(destination)
        target = self.workspace.trunk()
        if self._integration_is_landed(attempt, target):
            return self._finish_after_local_landing(attempt)
        if remote_tip == attempt["integration"] and target == attempt["base"]:
            self.store.change(aid, "landing_started")
            self._fast_forward_primary(aid, attempt)
            return self._finish_after_local_landing(attempt)
        if attempt["state"] == "push_intended" and target == attempt["base"] and remote_tip in (None, attempt["base"]):
            self.store.change(aid, "verified")
            return None
        raise Refused("Ambiguous pushed landing; preserve state and inspect manually")

    def _recover_landed_records(self, attempt: dict) -> dict | None:
        target = self.workspace.trunk()
        if not self._integration_is_landed(attempt, target):
            raise Refused("Landing records are incomplete, but the Integration commit is not on trunk")
        return self._finish_after_local_landing(attempt)

    def _remote_tip(self, destination: dict) -> str | None:
        result = self.workspace.git_result("ls-remote", destination["remote"], destination["ref"])
        if result.returncode:
            output = self._push_output(result)
            raise Refused("Cannot inspect pushed remote during recovery: " + output)
        for line in result.stdout.splitlines():
            parts = line.split()
            if len(parts) == 2 and parts[1] == destination["ref"]:
                return parts[0]
        return None

    def _integration_is_landed(self, attempt: dict, target: str) -> bool:
        if target == attempt["integration"]:
            return True
        ancestor = self.workspace.git("merge-base", attempt["integration"], target, check=False).strip()
        return ancestor == attempt["integration"]

    def _recover_allocation(self, attempt: dict, aid: str) -> None:
        path = Path(attempt["path"])
        if not path.exists():
            if ":" not in attempt["ticket"]:
                # No deletion or new execution. Claim stays reserved pending operator choice.
                raise Refused("Workspace not present; explicitly abandon before a new claim")
            effort, ticket_id = attempt["ticket"].split(":", 1)
            self.workspace.primary()
            current = self.workspace.trunk()
            if current == attempt["base"]:
                meta, _ = self.tickets.ticket(attempt["ticket"], current)
                if meta["status"] != "open":
                    raise Refused("Tracker claim no longer matches this Attempt")
                claimed, _ = self.tickets.claim_corenius(effort, ticket_id, attempt["owner"], 24.0)
                self.store.update_claim_basis(aid, self.workspace.trunk(), claimed["sha256"])
                attempt = self.store.row(aid)
            else:
                self._recover_missing_corenius_allocation_claim(attempt, current)
                attempt = self.store.row(aid)
            if self.workspace.trunk() != attempt["base"]:
                raise Refused("Trunk moved after tracker claim; explicitly abandon before a new claim")
            self.tickets.same_contract(attempt, attempt["base"])
            self.workspace.create_attempt_worktree(attempt)
        self.workspace.validate_attempt(attempt, expected_head=attempt["base"])
        self.store.change(aid, "working")

    def _recover_missing_corenius_allocation_claim(self, attempt: dict, current: str) -> None:
        effort, ticket_id = attempt["ticket"].split(":", 1)
        meta, _ = self.tickets.ticket(attempt["ticket"], current)
        changed = self.workspace.git("diff", "--name-only", attempt["base"], current).splitlines()
        if changed != [meta["path"]]:
            raise Refused("Trunk moved after tracker claim; explicitly abandon before a new claim")
        if meta["status"] != "claimed" or meta.get("owner") != attempt["owner"]:
            raise Refused("Tracker claim no longer matches this Attempt")
        self.tickets.corenius_blockers_ok(effort, ticket_id, current)
        self.store.update_claim_basis(attempt["id"], current, meta["sha256"])

    def abandon(self, aid: str, confirmed: bool) -> dict:
        if not confirmed:
            raise Refused("Requires --confirm-stopped; this command does not stop processes")
        require_worker_stopped(self.store.row(aid))
        with self.store.lock("lane"), self.store.lock("meta"):
            attempt = self.store.row(aid)
            require_worker_stopped(attempt)
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
