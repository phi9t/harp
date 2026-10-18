"""Conservative retention classification for collab-owned terminal artifacts."""
from __future__ import annotations

import math
from pathlib import Path
import re
import time

from common import Refused
from liveness import require_worker_stopped

TERMINAL = ("landed", "abandoned")


class Retention:
    def __init__(self, store, workspace, config):
        self.store = store
        self.workspace = workspace
        self.config = config

    def classify(self, days: float, attempts: list[dict], registrations: dict) -> tuple[list, list]:
        if not math.isfinite(days) or days < 0:
            raise Refused("--retention-days must be finite and non-negative")
        cutoff = time.time() - days * 86400
        candidates = []
        skipped = []
        by_id = {}
        for attempt in attempts:
            try:
                self.require_stopped(attempt)
            except Refused:
                continue
            by_id[attempt["id"]] = attempt
        for log in self.store.home.glob("check-*.log"):
            match = re.fullmatch(r"check-(a-[a-f0-9]{16})-([a-f0-9]{12})\.log", log.name)
            if not match or log.is_symlink() or not log.is_file():
                continue
            attempt = by_id.get(match[1])
            if not attempt or attempt["state"] not in TERMINAL:
                skipped.append({"kind": "check_log", "path": str(log), "action": "skip",
                                "reason": "active or unknown Attempt still needs this evidence"})
                continue
            if log.stat().st_mtime < cutoff:
                candidates.append({"kind": "check_log", "path": str(log), "action": "prune_log",
                                   "attempt": attempt["id"], "mtime": log.stat().st_mtime,
                                   "reason": "old terminal Attempt log"})
        rows = self.workspace.git("for-each-ref", "--format=%(refname)%00%(objectname)", "refs/collab").splitlines()
        check_refs = {}
        for row in rows:
            ref, head = row.split("\0")
            parts = ref.split("/")
            if len(parts) < 4 or parts[2] not in ("checks", "candidates", "landed"):
                continue
            attempt = by_id.get(parts[3])
            if not attempt or attempt["state"] not in TERMINAL:
                continue
            if parts[2] == "checks" and len(parts) == 5:
                check_refs[f"check-{parts[3]}-{parts[4]}"] = (attempt, head)
            file = self.workspace.common / ref
            # Packed refs have no individual age; retain them rather than guess from commit age.
            if file.is_file() and not file.is_symlink() and file.stat().st_mtime < cutoff:
                candidates.append({"kind": "collab_ref", "ref": ref, "head": head, "action": "prune_ref",
                                   "attempt": attempt["id"], "mtime": file.stat().st_mtime,
                                   "reason": "old terminal Attempt ref"})
        root = Path(self.config()["workspace_root"])
        for name, (attempt, head) in check_refs.items():
            path = root / name
            record = registrations.get(str(path))
            if not record:
                continue
            item = {"kind": "check_worktree", "path": str(path), "attempt": attempt["id"],
                    "head": head, "branch": attempt["branch"], "action": "skip"}
            if record["locked"]:
                skipped.append({**item, "reason": "check worktree is locked: " + record["lock_reason"]})
                continue
            if not path.exists():
                # Remove only this exact collab registration, never global worktree prune.
                registration = self.registration_dir(path)
                if registration and registration.stat().st_mtime < cutoff:
                    candidates.append({**item, "action": "prune_registration",
                                       "mtime": registration.stat().st_mtime,
                                       "reason": "old missing check workspace"})
                continue
            if path.is_symlink() or path.stat().st_mtime >= cutoff:
                continue
            try:
                self.workspace.validate_attempt({**attempt, "check_path": str(path)}, "check_path", head)
                self.workspace.clean(path, include_ignored=True)
            except Refused as exc:
                skipped.append({**item, "reason": str(exc)})
            else:
                candidates.append({**item, "action": "prune_worktree", "reason": "old terminal check workspace"})
        for path, record in registrations.items():
            candidate_path = Path(path)
            if candidate_path.parent == root and candidate_path.name.startswith("check-"):
                if candidate_path.name not in check_refs:
                    skipped.append({"kind": "check_worktree", "path": path, "action": "skip",
                                    "reason": "check workspace ownership or identity is unknown"})
        return candidates, skipped

    def registration_dir(self, path: Path) -> Path | None:
        root = self.workspace.common / "worktrees"
        if not root.is_dir() or root.is_symlink():
            return None
        for entry in root.iterdir():
            pointer = entry / "gitdir"
            if entry.is_symlink() or pointer.is_symlink() or not pointer.is_file():
                continue
            if pointer.read_text().strip() == str(path / ".git"):
                return entry
        return None

    def apply(self, item: dict, removed: list[dict]) -> None:
        attempt = next(attempt for attempt in self.store.attempts() if attempt["id"] == item["attempt"])
        self.require_stopped(attempt)
        if attempt["state"] not in TERMINAL:
            raise Refused("Retention Attempt became active; retaining artifacts")
        if item["action"] == "prune_log":
            path = Path(item["path"])
            if path.is_symlink() or path.stat().st_mtime != item["mtime"]:
                raise Refused("Log changed before pruning")
            path.unlink()
        elif item["action"] == "prune_ref":
            file = self.workspace.common / item["ref"]
            if file.is_symlink() or not file.is_file() or file.stat().st_mtime != item["mtime"]:
                raise Refused("Ref retention age changed; retaining ref")
            self.workspace.git("update-ref", "-d", item["ref"], item["head"])
        elif item["action"] == "prune_worktree":
            path = self.workspace.validate_attempt({**attempt, "check_path": item["path"]}, "check_path", item["head"])
            self.workspace.clean(path, include_ignored=True)
            self.workspace.git("worktree", "remove", str(path))
        elif item["action"] == "prune_registration":
            path = Path(item["path"])
            registration = self.registration_dir(path)
            if (path.exists() or path.is_symlink() or not registration
                    or registration.stat().st_mtime != item["mtime"]):
                raise Refused("Missing workspace identity changed; retaining registration")
            self.workspace.git("worktree", "remove", str(path))
        removed.append(item)

    def require_stopped(self, attempt: dict) -> None:
        require_worker_stopped(attempt)
