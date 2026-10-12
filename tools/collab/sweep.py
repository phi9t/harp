"""Safe manual cleanup for completed collab Attempts."""
from __future__ import annotations

from common import Refused


TERMINAL_STATES = ("landed", "abandoned")


class SweepManager:
    def __init__(self, store, workspace, config):
        self.store = store
        self.workspace = workspace
        self.config = config

    def sweep(self, apply: bool = False) -> dict:
        self.config()
        with self.store.lock("lane"), self.store.lock("meta"):
            trunk = self.workspace.trunk()
            worktrees = self.worktrees_by_path()
            candidates = []
            skipped = []
            attempt_branches = set()
            for attempt in self.store.attempts():
                attempt_branches.add(attempt["branch"])
                item = self.classify_attempt(attempt, trunk, worktrees)
                if item["action"] == "skip":
                    skipped.append(item)
                else:
                    candidates.append(item)
            skipped.extend(self.foreign_unmerged_branches(trunk, attempt_branches))
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

    def classify_attempt(self, attempt: dict, trunk: str, worktrees: dict[str, dict]) -> dict:
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
        if record and record["locked"]:
            reason = "worktree is locked"
            if record["lock_reason"]:
                reason += f": {record['lock_reason']}"
            return {**item, "action": "skip", "reason": reason}
        try:
            path = self.workspace.validate_attempt(attempt)
            self.workspace.clean(path, include_ignored=True)
            head = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
        except Refused as exc:
            return {**item, "action": "skip", "reason": str(exc)}
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
        head = item["head"]
        path = self.workspace.validate_attempt(item)
        self.workspace.clean(path, include_ignored=True)
        current = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
        if current != head:
            raise Refused(f"Sweep candidate moved before removal: {item['attempt']}")
        if item["action"] == "archive_then_remove_worktree_and_branch":
            archive_ref = item["archive_ref"]
            old = self.workspace.git("rev-parse", "--verify", archive_ref, check=False).strip()
            if old and old != head:
                raise Refused(f"Archive ref already points elsewhere: {archive_ref}")
            if not old:
                self.workspace.git("update-ref", archive_ref, head, "")
            archived.append({"branch": item["branch"], "head": head, "ref": archive_ref})
        self.workspace.git("worktree", "remove", item["path"])
        self.workspace.git("update-ref", "-d", "refs/heads/" + item["branch"], head)
        removed.append(
            {
                "attempt": item["attempt"],
                "branch": item["branch"],
                "head": head,
                "path": item["path"],
            }
        )

    def is_ancestor(self, older: str, newer: str) -> bool:
        return self.workspace.git("merge-base", older, newer, check=False).strip() == older

    def foreign_unmerged_branches(self, trunk: str, attempt_branches: set[str]) -> list[dict]:
        trunk_branch = self.config()["trunk_ref"].removeprefix("refs/heads/")
        rows = self.workspace.git(
            "for-each-ref",
            "--format=%(refname:short)%00%(objectname)%00%(worktreepath)",
            "refs/heads",
        ).splitlines()
        skipped = []
        for row in rows:
            branch, head, worktree = (row.split("\0") + ["", ""])[:3]
            if branch == trunk_branch or branch in attempt_branches:
                continue
            if self.is_ancestor(head, trunk):
                continue
            skipped.append(
                {
                    "kind": "foreign_branch",
                    "action": "skip",
                    "branch": branch,
                    "owner": self.branch_owner(branch),
                    "head": head,
                    "path": worktree,
                    "reason": "foreign branch has unmerged commits",
                }
            )
        return skipped

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
