"""Safe manual cleanup for completed collab Attempts."""
from __future__ import annotations

from contextlib import nullcontext
from pathlib import Path

from archives import archive_worktree, clear_archived_changes, require_archived_source
from common import Refused
from retention import Retention
from liveness import require_worker_stopped, worker_liveness


TERMINAL_STATES = ("landed", "abandoned")


class SweepManager:
    def __init__(self, store, workspace, config):
        self.store = store
        self.workspace = workspace
        self.config = config
        self.retention = Retention(store, workspace, config)

    def sweep(
        self, apply: bool = False, archive_dirty: bool = False, include_foreign: bool = False,
        retention_days: float = 30,
    ) -> dict:
        self.config()
        execution_lock = self.store.lock("check") if apply else nullcontext()
        with execution_lock, self.store.lock("lane"), self.store.lock("meta"):
            trunk = self.workspace.trunk()
            worktrees = self.worktrees_by_path()
            candidates = []
            skipped = []
            attempt_branches = set()
            for attempt in self.store.attempts():
                attempt_branches.add(attempt["branch"])
                item = self.classify_attempt(attempt, trunk, worktrees, archive_dirty)
                if item["action"] == "skip":
                    skipped.append(item)
                else:
                    candidates.append(item)
            for item in self.foreign_branches(trunk, attempt_branches, include_foreign):
                (skipped if item["action"] == "skip" else candidates).append(item)
            old_candidates, old_skipped = self.retention.classify(retention_days, self.store.attempts(), worktrees)
            candidates.extend(old_candidates)
            skipped.extend(old_skipped)
            removed = []
            archived = []
            if apply:
                for item in candidates:
                    self.apply_candidate(item, removed, archived)
        return {
            "applied": apply,
            "candidates": candidates,
            "skipped": skipped,
            "removed": removed,
            "archived": archived,
        }

    def classify_attempt(
        self, attempt: dict, trunk: str, worktrees: dict[str, dict], archive_dirty: bool = False,
    ) -> dict:
        item = {
            "kind": "attempt",
            "attempt": attempt["id"],
            "ticket": attempt["ticket"],
            "state": attempt["state"],
            "owner": attempt["owner"],
            "path": attempt["path"],
            "branch": attempt["branch"],
        }
        if attempt["state"] not in TERMINAL_STATES:
            return {
                **item,
                "action": "skip",
                "reason": f"Attempt is {attempt['state']}; fresh or in-progress Attempts are never swept",
            }
        record = worktrees.get(attempt["path"])
        if record and record["locked"] and record["lock_reason"] != f"collab:{attempt['id']}":
            reason = "worktree is locked"
            if record["lock_reason"]:
                reason += f": {record['lock_reason']}"
            return {**item, "action": "skip", "reason": reason}
        item["unlock"] = bool(record and record["locked"])
        try:
            self.require_stopped(attempt)
            path = self.workspace.validate_attempt(attempt)
            dirty = False
            try:
                self.workspace.clean(path, include_ignored=True)
            except Refused:
                if not archive_dirty:
                    raise
                dirty = True
            head = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
        except Refused as exc:
            return {**item, "action": "skip", "reason": str(exc)}
        if dirty:
            return {**item, "action": "archive_dirty_then_remove_worktree_and_branch", "head": head,
                    "dirty": True, "reason": "terminal dirty Attempt; verified archive before removal"}
        merged = self.is_ancestor(head, trunk)
        if merged:
            return {
                **item,
                "action": "remove_worktree_and_branch",
                "head": head,
                "merged": True,
                "reason": "terminal Attempt has a clean merged branch",
            }
        archive_ref = f"refs/collab/archive/{attempt['id']}/{head}"
        return {
            **item,
            "action": "archive_then_remove_worktree_and_branch",
            "archive_ref": archive_ref,
            "head": head,
            "merged": False,
            "reason": "terminal Attempt has a clean unmerged branch; archive before removal",
        }

    def apply_candidate(self, item: dict, removed: list[dict], archived: list[dict]) -> None:
        if item["kind"] not in ("attempt", "foreign_branch"):
            self.retention.apply(item, removed)
            return
        head = item["head"]
        if item["kind"] == "foreign_branch":
            self.apply_foreign_branch(item, removed, archived)
            return
        path = self.require_removal_owner(item)
        if not item.get("dirty"):
            self.workspace.clean(path, include_ignored=True)
        current = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
        if current != head:
            raise Refused(f"Sweep candidate moved before removal: {item['attempt']}")
        if item.get("dirty"):
            record = archive_worktree(self.workspace, self.store.home, item)
            archived.append(record)
            self.require_removal_owner(item)
            require_archived_source(self.workspace, self.store.home, record, item)
            self.require_removal_owner(item)
            clear_archived_changes(self.workspace, path)
        if item["action"] == "archive_then_remove_worktree_and_branch":
            archive_ref = item["archive_ref"]
            old = self.workspace.git("rev-parse", "--verify", archive_ref, check=False).strip()
            if old and old != head:
                raise Refused(f"Archive ref already points elsewhere: {archive_ref}")
            if not old:
                self.workspace.git("update-ref", archive_ref, head, "")
            archived.append({"branch": item["branch"], "head": head, "ref": archive_ref})
        self.require_removal_owner(item)
        if item.get("unlock"):
            self.workspace.git("worktree", "unlock", item["path"])
        try:
            self.workspace.git("worktree", "remove", item["path"])
        except Refused:
            if item.get("unlock"):
                self.workspace.git(
                    "worktree", "lock", "--reason", f"collab:{item['attempt']}", item["path"], check=False,
                )
            raise
        self.workspace.git("update-ref", "-d", "refs/heads/" + item["branch"], head)
        removed.append(
            {
                "attempt": item["attempt"],
                "branch": item["branch"],
                "head": head,
                "path": item["path"],
            }
        )

    def require_removal_owner(self, item: dict) -> Path:
        """Re-read owner and Git registration immediately before destructive cleanup."""
        attempt = self.store.row(item["attempt"])
        if attempt["state"] not in TERMINAL_STATES or any(
            attempt[name] != item[name] for name in ("owner", "path", "branch")
        ):
            raise Refused("Sweep Attempt ownership changed; retaining worktree")
        self.require_stopped(attempt)
        registration = self.worktrees_by_path().get(item["path"])
        if registration is None:
            raise Refused("Worktree registration changed; retaining worktree")
        if registration["locked"] and registration["lock_reason"] != f"collab:{item['attempt']}":
            raise Refused("Worktree is locked; retaining worktree: " + registration["lock_reason"])
        return self.workspace.validate_attempt(attempt, expected_head=item["head"])

    def require_stopped(self, attempt: dict) -> None:
        require_worker_stopped(attempt)
        status = worker_liveness(attempt)
        if any(source.get("state") == "unknown" for source in status["sources"]):
            raise Refused("Worker liveness is unknown; retaining Attempt artifacts")

    def is_ancestor(self, older: str, newer: str) -> bool:
        return self.workspace.git("merge-base", older, newer, check=False).strip() == older

    def foreign_branches(self, trunk: str, attempt_branches: set[str], include_foreign: bool) -> list[dict]:
        trunk_branch = self.config()["trunk_ref"].removeprefix("refs/heads/")
        rows = self.workspace.git(
            "for-each-ref", "--format=%(refname:short)%00%(objectname)%00%(worktreepath)", "refs/heads",
        ).splitlines()
        result = []
        for row in rows:
            branch, head, worktree = (row.split("\0") + ["", ""])[:3]
            if branch == trunk_branch or branch in attempt_branches:
                continue
            merged = self.is_ancestor(head, trunk)
            item = {"kind": "foreign_branch", "branch": branch, "owner": self.branch_owner(branch),
                    "head": head, "path": worktree, "merged": merged}
            if worktree:
                result.append({**item, "action": "skip", "reason": "foreign checkout has unknown worker identity"})
            elif not merged and not include_foreign:
                result.append({**item, "action": "skip", "reason": "foreign branch has unmerged commits"})
            else:
                result.append({**item, "action": "remove_branch" if merged else "archive_then_remove_branch",
                               "reason": "fully merged foreign branch" if merged else "explicit foreign archive",
                               "archive_ref": "refs/collab/archive/foreign/" + head})
        return result

    def apply_foreign_branch(self, item: dict, removed: list[dict], archived: list[dict]) -> None:
        if any(record["branch"] == "refs/heads/" + item["branch"] for record in self.worktrees_by_path().values()):
            raise Refused("Foreign branch acquired a checkout; retaining it")
        current = self.workspace.git("rev-parse", "--verify", "refs/heads/" + item["branch"]).strip()
        if current != item["head"]:
            raise Refused("Foreign branch moved; retaining it")
        if not item["merged"]:
            ref = item["archive_ref"]
            old = self.workspace.git("rev-parse", "--verify", ref, check=False).strip()
            if old and old != current:
                raise Refused("Archive ref moved; retaining branch")
            if not old:
                self.workspace.git("update-ref", ref, current, "")
            archived.append({"branch": item["branch"], "head": current, "ref": ref})
        self.workspace.git("update-ref", "-d", "refs/heads/" + item["branch"], current)
        removed.append({"branch": item["branch"], "head": current})

    def branch_owner(self, branch: str) -> str:
        return branch.split("/", 1)[0] if "/" in branch else "unknown"

    def worktrees_by_path(self) -> dict[str, dict]:
        records = self.workspace.git("worktree", "list", "--porcelain", "-z").split("\0\0")
        result = {}
        for record in records:
            if not record:
                continue
            fields = {}
            for entry in record.split("\0"):
                key, _, value = entry.partition(" ")
                fields.setdefault(key, []).append(value)
            path = fields["worktree"][0]
            locked = "locked" in fields
            result[path] = {
                "path": path,
                "head": fields.get("HEAD", [""])[0],
                "branch": fields.get("branch", [""])[0],
                "locked": locked,
                "lock_reason": fields.get("locked", [""])[0] if locked else "",
            }
        return result
