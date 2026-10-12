"""Shared stateless safety checks for the collab and compatibility landing routes."""
from __future__ import annotations

from pathlib import Path
import os
import subprocess

from common import Refused
from state_store import StateStore
from workspace import run_git


class PreflightRefused(Refused):
    def __init__(self, kind: str, reason: str, paths: list[str] | None = None):
        super().__init__(reason)
        self.kind = kind
        self.paths = paths or []


def hidden_index_entries(checkout: Path) -> list[dict]:
    entries = run_git(checkout, "ls-files", "-v", "-z").split("\0")
    hidden = []
    for entry in entries:
        if entry and (entry[0].islower() or entry[0] == "S"):
            flag = "skip-worktree" if entry[0].upper() == "S" else "assume-unchanged"
            hidden.append({"path": entry[2:], "flag": flag})
    return hidden


def process_group_alive(identity: int) -> bool:
    try:
        os.killpg(identity, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def gate_workspace_status(repo: Path, checkout: Path, tip: str, git_directory: str) -> dict:
    status = {"identity": False, "same_head": False, "clean": False, "hidden_index": []}
    try:
        expected_common = run_git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").strip()
        actual_common = run_git(checkout, "rev-parse", "--path-format=absolute", "--git-common-dir").strip()
        actual_git = run_git(checkout, "rev-parse", "--absolute-git-dir").strip()
        top = run_git(checkout, "rev-parse", "--show-toplevel").strip()
        head = run_git(checkout, "rev-parse", "HEAD").strip()
        branch = run_git(checkout, "symbolic-ref", "-q", "HEAD", check=False).strip()
        records = run_git(repo, "worktree", "list", "--porcelain", "-z").split("\0\0")
        registration = next(
            (record.split("\0") for record in records if record.startswith("worktree " + str(checkout) + "\0")),
            [],
        )
        status["identity"] = (
            checkout.resolve() == checkout
            and top == str(checkout)
            and expected_common == actual_common
            and actual_git == git_directory
            and not branch
            and "detached" in registration
            and "HEAD " + head in registration
            and not any(entry == "locked" or entry.startswith("locked ") for entry in registration)
        )
        status["same_head"] = head == tip
        status["hidden_index"] = hidden_index_entries(checkout)
        visible = run_git(checkout, "status", "--porcelain=v1", "-z", "--untracked-files=all")
        status["clean"] = not visible and not status["hidden_index"]
    except (Refused, OSError) as error:
        status["reason"] = str(error)
    return status


def collision_paths(repo: Path, checkout: Path | None, base: str, tip: str) -> list[str]:
    if checkout is None:
        return []
    changed = run_git(repo, "diff", "--name-only", "-z", base, tip).split("\0")
    local = set()
    for args in (
        ("diff", "--name-only", "-z"),
        ("diff", "--cached", "--name-only", "-z"),
        ("ls-files", "--others", "--exclude-standard", "-z"),
        ("ls-files", "--others", "--ignored", "--exclude-standard", "-z"),
    ):
        local.update(run_git(checkout, *args).split("\0"))
    return sorted(
        path for path in local if path and any(
            other and (path == other or path.startswith(other + "/") or other.startswith(path + "/"))
            for other in changed
        )
    )


def preflight(repo: Path, target_ref: str, expected_base: str, tip: str, checkout: Path | None) -> None:
    current = run_git(repo, "rev-parse", "--verify", target_ref + "^{commit}").strip()
    if current != expected_base or run_git(repo, "merge-base", expected_base, tip).strip() != expected_base:
        raise PreflightRefused("stale", "Target moved or the Candidate cannot fast-forward from the target")
    overlap = collision_paths(repo, checkout, expected_base, tip)
    if overlap:
        raise PreflightRefused("overlap", "target checkout has local files overlapping the Candidate", overlap)
    if checkout is None:
        return
    expected_common = run_git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").strip()
    actual_common = run_git(checkout, "rev-parse", "--path-format=absolute", "--git-common-dir").strip()
    actual_top = run_git(checkout, "rev-parse", "--show-toplevel").strip()
    if (
        checkout.resolve() != checkout
        or actual_common != expected_common
        or actual_top != str(checkout)
    ):
        raise PreflightRefused("local", "Target checkout repository identity changed")
    if run_git(checkout, "symbolic-ref", "-q", "HEAD", check=False).strip() != target_ref:
        raise PreflightRefused("local", "Target checkout is no longer on the intended branch")
    if run_git(checkout, "rev-parse", "HEAD").strip() != expected_base:
        raise PreflightRefused("stale", "Target checkout HEAD moved")
    if hidden_index_entries(checkout):
        raise PreflightRefused("local", "Target checkout has hidden index entries; preserve and inspect them")
    # An existing staged change makes Git's fast-forward index update unsafe even
    # when its path is outside the Candidate diff. Preserve it before publishing.
    if run_git(checkout, "diff", "--cached", "--name-only", "-z"):
        raise PreflightRefused("local", "Target checkout has staged changes; preserve them before landing")
    for marker in ("MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply", "sequencer"):
        path = Path(run_git(checkout, "rev-parse", "--path-format=absolute", "--git-path", marker).strip())
        if path.exists():
            raise PreflightRefused("local", f"Target checkout has an in-progress Git operation: {marker}")


def allowed_override_owners(repo: Path) -> list[str]:
    common = Path(run_git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").strip())
    home = common / "collab"
    store = StateStore(home, home / "state.sqlite3")
    config = store.load_config()
    if config is not None:
        owners = config.get("mainline_override_owners", [])
        if not isinstance(owners, list) or not all(isinstance(owner, str) and owner.strip() for owner in owners):
            raise Refused("Invalid configured mainline override owners")
        return owners
    # Read repository-owned config, never a Candidate's files or environment values.
    return run_git(repo, "config", "--local", "--get-all", "collab.overrideOwners", check=False).splitlines()


def classify_push_failure(result: subprocess.CompletedProcess) -> str:
    reason = (result.stderr or result.stdout or "push failed without diagnostics").strip()
    lower = reason.lower()
    if any(word in lower for word in ("non-fast-forward", "fetch first", "stale info")):
        category = "non-fast-forward"
    elif any(word in lower for word in ("authentication", "permission denied", "could not read username")):
        category = "authentication"
    elif any(word in lower for word in ("could not resolve", "connection", "network", "unable to access")):
        category = "network"
    else:
        category = "remote rejection or transport error"
    return f"push failed ({category}): {reason}"
