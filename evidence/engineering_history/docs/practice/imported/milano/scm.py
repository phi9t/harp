"""Git workflow for concurrent agents: one worktree per task, linear mainline.

The Insula candidate is a git worktree's *staged index*, so agents must never
share a worktree. Each task gets its own worktree and branch (``<agent>/<topic>``)
created from ``origin/diziet/mainline``; integration rebases, gates the exact
commit that will be pushed, fast-forwards mainline, and pushes, retrying if
another agent integrated first. Mainline never receives merge commits or
non-fast-forward updates (enforced again by the pre-push hook).

Host-side and standard-library only::

    python3 -m milano.scm new --agent claude --topic m4-orchestrator
    python3 -m milano.scm status
    python3 -m milano.scm integrate claude/m4-orchestrator --gate repo --gate rtest
    python3 -m milano.scm finish claude/m4-orchestrator
"""

from __future__ import annotations

import argparse
import fcntl
import os
import re
import subprocess
import sys
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator, Sequence

MAINLINE = "diziet/mainline"
REMOTE = "origin"
ZERO = "0" * 40
BRANCH_RE = re.compile(r"^[a-z0-9][a-z0-9-]*/[a-z0-9][a-z0-9._-]*$")


class ScmError(RuntimeError):
    pass


def git(repo: Path, *args: str, check: bool = True) -> str:
    completed = subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True, text=True, check=False
    )
    if check and completed.returncode != 0:
        raise ScmError(f"git {' '.join(args)} failed: {completed.stderr.strip()}")
    return completed.stdout.strip()


def _is_ancestor(repo: Path, ancestor: str, descendant: str) -> bool:
    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", ancestor, descendant],
        capture_output=True,
        check=False,
    ).returncode == 0


def validate_branch(branch: str) -> None:
    if branch == MAINLINE or not BRANCH_RE.fullmatch(branch):
        raise ScmError(f"task branches are named <agent>/<topic> in lowercase, got {branch!r}")


@dataclass(frozen=True)
class Worktree:
    path: Path
    branch: str | None
    head: str


def worktrees(repo: Path) -> list[Worktree]:
    entries: list[Worktree] = []
    path = head = branch = None
    for line in git(repo, "worktree", "list", "--porcelain").splitlines() + [""]:
        if line.startswith("worktree "):
            path = Path(line.split(" ", 1)[1])
        elif line.startswith("HEAD "):
            head = line.split(" ", 1)[1]
        elif line.startswith("branch "):
            branch = line.split(" ", 1)[1].removeprefix("refs/heads/")
        elif not line and path is not None:
            entries.append(Worktree(path, branch, head or ""))
            path = head = branch = None
    return entries


def worktree_for(repo: Path, branch: str) -> Worktree | None:
    return next((entry for entry in worktrees(repo) if entry.branch == branch), None)


def default_worktree_root(repo: Path) -> Path:
    common = Path(git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir"))
    main = common.parent
    return Path(os.environ.get("MILANO_WORKTREE_ROOT", main.parent / f"{main.name}.worktrees"))


def new_worktree(repo: Path, agent: str, topic: str, root: Path | None = None) -> Worktree:
    branch = f"{agent}/{topic}"
    validate_branch(branch)
    git(repo, "fetch", "--quiet", REMOTE, MAINLINE)
    path = (root or default_worktree_root(repo)) / f"{agent}-{topic}"
    if path.exists():
        raise ScmError(f"{path} already exists")
    git(repo, "worktree", "add", "--quiet", "-b", branch, str(path), f"{REMOTE}/{MAINLINE}")
    git(path, "branch", "--quiet", "--unset-upstream", check=False)
    return Worktree(path, branch, git(path, "rev-parse", "HEAD"))


def merge_commits(repo: Path, base: str, tip: str) -> list[str]:
    output = git(repo, "rev-list", "--merges", f"{base}..{tip}")
    return output.split() if output else []


def check_push(repo: Path, remote_ref: str, local_sha: str, remote_sha: str) -> list[str]:
    """Why a push to `remote_ref` must be refused (empty list: allowed)."""
    if remote_ref != f"refs/heads/{MAINLINE}":
        return []
    if local_sha == ZERO:
        return [f"{MAINLINE} cannot be deleted"]
    problems = []
    if remote_sha != ZERO:
        if not _is_ancestor(repo, remote_sha, local_sha):
            problems.append(f"{MAINLINE} only moves by fast-forward (rebase first)")
        merges = merge_commits(repo, remote_sha, local_sha)
    else:
        merges = git(repo, "rev-list", "--merges", local_sha).split()
    if merges:
        problems.append(f"{MAINLINE} is linear; merge commits: {', '.join(m[:12] for m in merges)}")
    return problems


@contextmanager
def integration_lock(repo: Path) -> Iterator[None]:
    common = Path(git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir"))
    with open(common / "milano-integrate.lock", "w") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        yield


def _gate(worktree: Path, revision: str, gate: str) -> None:
    command = [
        str(worktree / "scripts" / "run_local_insula.sh"),
        "--rev",
        revision,
        "--",
        "scripts/rootfs/run_pipeline.sh",
        *gate.split(),
    ]
    print(f"scm: gate {gate!r} on {revision[:12]}", file=sys.stderr, flush=True)
    if subprocess.run(command, cwd=worktree, check=False).returncode != 0:
        raise ScmError(f"gate {gate!r} failed on {revision[:12]}")


def integrate(
    repo: Path,
    branch: str,
    gates: Sequence[str] = ("repo",),
    push: bool = True,
    attempts: int = 3,
    run_gate=_gate,
) -> str:
    """Rebase `branch` onto origin mainline, gate its tip, fast-forward, push."""
    validate_branch(branch)
    task = worktree_for(repo, branch)
    if task is None:
        raise ScmError(f"{branch} is not checked out in any worktree")
    if git(task.path, "status", "--porcelain", "--untracked-files=no"):
        raise ScmError(f"{task.path} has uncommitted changes; commit or stash them first")
    with integration_lock(repo):
        for attempt in range(1, attempts + 1):
            git(repo, "fetch", "--quiet", REMOTE, MAINLINE)
            upstream = f"{REMOTE}/{MAINLINE}"
            rebased = subprocess.run(
                ["git", "-C", str(task.path), "rebase", "--quiet", upstream],
                capture_output=True,
                text=True,
                check=False,
            )
            if rebased.returncode != 0:
                git(task.path, "rebase", "--abort", check=False)
                raise ScmError(f"{branch} does not rebase cleanly onto {upstream}; resolve and retry")
            tip = git(task.path, "rev-parse", "HEAD")
            base = git(repo, "rev-parse", upstream)
            if tip == base:
                raise ScmError(f"{branch} has no commits beyond {upstream}")
            merges = merge_commits(repo, base, tip)
            if merges:
                raise ScmError(f"{branch} contains merge commits: {', '.join(m[:12] for m in merges)}")
            for gate in gates:
                run_gate(task.path, tip, gate)
            _fast_forward_mainline(repo, tip)
            if not push:
                return tip
            pushed = subprocess.run(
                ["git", "-C", str(repo), "push", "--quiet", REMOTE, f"{tip}:refs/heads/{MAINLINE}"],
                capture_output=True,
                text=True,
                check=False,
            )
            if pushed.returncode == 0:
                return tip
            print(f"scm: push rejected (attempt {attempt}): {pushed.stderr.strip()}", file=sys.stderr)
        raise ScmError(f"could not integrate {branch} after {attempts} attempts")


def _fast_forward_mainline(repo: Path, tip: str) -> None:
    current = git(repo, "rev-parse", "--verify", "--quiet", f"refs/heads/{MAINLINE}", check=False)
    if current and not _is_ancestor(repo, current, tip):
        raise ScmError(f"local {MAINLINE} has commits not on the rebased branch; sync it first")
    holder = worktree_for(repo, MAINLINE)
    if holder is not None:
        if git(holder.path, "status", "--porcelain", "--untracked-files=no"):
            raise ScmError(f"{holder.path} ({MAINLINE}) has uncommitted changes")
        git(holder.path, "merge", "--quiet", "--ff-only", tip)
    else:
        git(repo, "update-ref", f"refs/heads/{MAINLINE}", tip, *([current] if current else []))


def _main_worktree(repo: Path) -> Path:
    common = Path(git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir"))
    return common.parent


def finish(repo: Path, branch: str) -> None:
    validate_branch(branch)
    # Work from the main worktree: `repo` may be the very worktree we remove.
    repo = _main_worktree(repo)
    git(repo, "fetch", "--quiet", REMOTE, MAINLINE)
    if not _is_ancestor(repo, branch, f"{REMOTE}/{MAINLINE}"):
        raise ScmError(f"{branch} is not on {REMOTE}/{MAINLINE}; integrate it first")
    task = worktree_for(repo, branch)
    if task is not None:
        git(repo, "worktree", "remove", str(task.path))
    git(repo, "branch", "--quiet", "-D", branch)


def status(repo: Path) -> list[str]:
    git(repo, "fetch", "--quiet", REMOTE, MAINLINE, check=False)
    lines = []
    for entry in worktrees(repo):
        branch = entry.branch or "(detached)"
        counts = git(
            repo, "rev-list", "--left-right", "--count", f"{REMOTE}/{MAINLINE}...{entry.head}", check=False
        ).split()
        behind, ahead = counts if len(counts) == 2 else ("?", "?")
        dirty = "dirty" if git(entry.path, "status", "--porcelain", check=False) else "clean"
        lines.append(f"{branch:<40} +{ahead}/-{behind} {dirty:<5} {entry.path}")
    return lines


# Repository-local settings every clone should have (idempotent; worktrees
# share them). The SSH key is the one GitLab accepts for this account.
LOCAL_CONFIG = {
    "core.hooksPath": ".githooks",
    "core.sshCommand": "ssh -i ~/.ssh/byted_code_devx -o IdentitiesOnly=yes",
    "pull.rebase": "true",
    "push.default": "current",
}


def setup(repo: Path) -> list[str]:
    applied = []
    for key, value in LOCAL_CONFIG.items():
        if git(repo, "config", "--local", "--get", key, check=False) != value:
            git(repo, "config", "--local", key, value)
            applied.append(f"{key}={value}")
    return applied


def pre_push_hook(repo: Path, stdin: str) -> int:
    problems = []
    for line in stdin.splitlines():
        fields = line.split()
        if len(fields) == 4:
            _local_ref, local_sha, remote_ref, remote_sha = fields
            problems += check_push(repo, remote_ref, local_sha, remote_sha)
    for problem in problems:
        print(f"pre-push: {problem}", file=sys.stderr)
    return 1 if problems else 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    commands = parser.add_subparsers(dest="command", required=True)
    new = commands.add_parser("new", help="create a worktree and <agent>/<topic> branch from origin mainline")
    new.add_argument("--agent", required=True)
    new.add_argument("--topic", required=True)
    new.add_argument("--root", type=Path)
    commands.add_parser("status", help="worktrees, branches, ahead/behind origin mainline")
    integrate_parser = commands.add_parser("integrate", help="rebase, gate the exact tip, fast-forward, push")
    integrate_parser.add_argument("branch")
    integrate_parser.add_argument("--gate", action="append", help="pipeline mode (repeatable; default: repo)")
    integrate_parser.add_argument("--no-push", action="store_true")
    finish_parser = commands.add_parser("finish", help="remove an integrated task's worktree and branch")
    finish_parser.add_argument("branch")
    commands.add_parser("setup", help="apply repository-local git settings (hooks, push key, rebase)")
    commands.add_parser("pre-push-hook")
    args = parser.parse_args(argv)
    repo = args.repo.resolve()
    try:
        if args.command == "new":
            created = new_worktree(repo, args.agent, args.topic, args.root)
            print(f"{created.branch}\t{created.path}")
        elif args.command == "status":
            print("\n".join(status(repo)))
        elif args.command == "integrate":
            tip = integrate(repo, args.branch, gates=args.gate or ["repo"], push=not args.no_push)
            print(f"{MAINLINE} -> {tip}")
        elif args.command == "finish":
            finish(repo, args.branch)
        elif args.command == "setup":
            print("\n".join(setup(repo)) or "already configured")
        elif args.command == "pre-push-hook":
            return pre_push_hook(repo, sys.stdin.read())
    except ScmError as error:
        print(f"scm: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
