#!/usr/bin/env python3
"""Land a branch through gates without moving local mainline first."""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from worker import WorkerError, git, main_worktree, now  # noqa: E402

OK, OVERLAP, MERGE_COMMIT, GATE_FAILED, LOST_RACE, USAGE = 0, 3, 4, 5, 6, 64


def repo_root(repo: str | Path) -> Path:
    return Path(git(repo, "rev-parse", "--show-toplevel").strip())


def common_git_dir(repo: str | Path) -> Path:
    return Path(git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").strip())


@contextlib.contextmanager
def land_lock(repo: Path):
    path = common_git_dir(repo) / "land.lock"
    with path.open("a+") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        handle.seek(0)
        handle.truncate()
        handle.write(f"{os.getpid()} {now()}\n")
        handle.flush()
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def git_result(repo: str | Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)


def local_branch_ref(repo: Path, branch: str) -> str:
    candidates = [branch] if branch.startswith("refs/heads/") else [f"refs/heads/{branch}", branch]
    for candidate in candidates:
        full = git(repo, "rev-parse", "--symbolic-full-name", "--verify", "-q", candidate, check=False).strip()
        if full.startswith("refs/heads/"):
            return full
    raise WorkerError(f"branch {branch!r} not found")


def rev(repo: Path, ref: str) -> str:
    out = git(repo, "rev-parse", "--verify", "-q", ref + "^{commit}", check=False).strip()
    if not out:
        raise WorkerError(f"ref {ref!r} not found")
    return out


def worktree_branches(repo: Path) -> dict[str, Path]:
    found: dict[str, Path] = {}
    current: Path | None = None
    for line in git(repo, "worktree", "list", "--porcelain").splitlines() + [""]:
        if not line:
            current = None
        elif line.startswith("worktree "):
            current = Path(line[9:])
        elif line.startswith("branch ") and current is not None:
            found[line[7:]] = current
    return found


def parse_status_paths(text: str) -> set[str]:
    paths: set[str] = set()
    for line in text.splitlines():
        if len(line) < 4:
            continue
        raw = line[3:].strip().strip('"')
        if " -> " in raw:
            old, new = raw.split(" -> ", 1)
            paths.add(old.strip('"'))
            paths.add(new.strip('"'))
        else:
            paths.add(raw)
    return paths


def tracked_overlap(repo: Path, mainline: str, branch: str) -> list[str]:
    main = main_worktree(repo)
    dirty = parse_status_paths(git(main, "status", "--porcelain", "--untracked-files=no"))
    changed = set(git(repo, "diff", "--name-only", f"{mainline}...{branch}").splitlines())
    return sorted(dirty & changed)


def remote_exists(repo: Path, remote: str) -> bool:
    return remote in git(repo, "remote").splitlines()


def fetch_mainline(repo: Path, remote: str, mainline: str) -> str:
    ref = f"refs/remotes/{remote}/{mainline}"
    git(repo, "fetch", "-q", remote, f"+refs/heads/{mainline}:{ref}")
    return ref


def is_ancestor(repo: Path, older: str, newer: str) -> bool:
    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", older, newer],
        capture_output=True,
    ).returncode == 0


@contextlib.contextmanager
def temporary_worktree(repo: Path, ref: str, prefix: str, detach: bool):
    parent = main_worktree(repo).parent
    path = Path(tempfile.mkdtemp(prefix=prefix, dir=str(parent)))
    added = False
    try:
        args = ["worktree", "add", "-q"]
        if detach:
            args.append("--detach")
        git(repo, *args, str(path), ref)
        added = True
        yield path
    finally:
        if added:
            git(repo, "worktree", "remove", "--force", str(path), check=False)
        if path.exists():
            shutil.rmtree(path)


def rebase_branch(repo: Path, branch_ref: str, base_ref: str) -> str:
    branch_name = branch_ref.removeprefix("refs/heads/")
    branch_tree = worktree_branches(repo).get(branch_ref)
    if branch_tree is not None:
        try:
            git(branch_tree, "rebase", "-q", "--rebase-merges", base_ref)
        except WorkerError:
            git(branch_tree, "rebase", "--abort", check=False)
            raise
        return rev(repo, branch_ref)

    with temporary_worktree(repo, branch_name, ".land-rebase-", detach=False) as worktree:
        try:
            git(worktree, "rebase", "-q", "--rebase-merges", base_ref)
        except WorkerError:
            git(worktree, "rebase", "--abort", check=False)
            raise
    return rev(repo, branch_ref)


def merge_commits(repo: Path, base_ref: str, tip: str) -> list[str]:
    return git(repo, "rev-list", "--merges", f"{base_ref}..{tip}").splitlines()


def run_gates(repo: Path, tip: str, gates: list[str]) -> bool:
    with temporary_worktree(repo, tip, ".land-gate-", detach=True) as checkout:
        for gate in gates:
            env = os.environ.copy()
            env["LAND_TIP"] = tip
            result = subprocess.run(["sh", "-c", gate], cwd=checkout, env=env, capture_output=True, text=True)
            if result.stdout:
                print(result.stdout, end="", file=sys.stderr)
            if result.stderr:
                print(result.stderr, end="", file=sys.stderr)
            if result.returncode != 0:
                return False
    return True


def push_tip(repo: Path, remote: str, tip: str, mainline: str) -> subprocess.CompletedProcess[str]:
    return git_result(repo, "push", "-q", remote, f"{tip}:refs/heads/{mainline}")


def restore_branch(repo: Path, branch_ref: str, original: str) -> None:
    branch_tree = worktree_branches(repo).get(branch_ref)
    if branch_tree is not None:
        git(branch_tree, "rebase", "--abort", check=False)
        git(branch_tree, "reset", "--hard", "-q", original, check=False)
    else:
        git(repo, "update-ref", branch_ref, original, check=False)


def fast_forward_mainline(repo: Path, mainline: str, tip: str) -> None:
    mainline_ref = f"refs/heads/{mainline}"
    current = rev(repo, mainline_ref)
    if not is_ancestor(repo, current, tip):
        raise WorkerError(f"{mainline} cannot fast-forward to {tip}")
    mainline_tree = worktree_branches(repo).get(mainline_ref)
    if mainline_tree is not None:
        git(mainline_tree, "merge", "--ff-only", "-q", tip)
    else:
        git(repo, "update-ref", mainline_ref, tip)


def land(args: argparse.Namespace) -> int:
    repo = repo_root(args.repo or ".")
    gates = list(args.gate or [])
    attempts = int(args.attempts)
    if attempts < 1:
        raise WorkerError("--attempts must be at least 1")

    with land_lock(repo):
        branch_ref = local_branch_ref(repo, args.branch)
        original = rev(repo, branch_ref)
        overlap = tracked_overlap(repo, args.mainline, branch_ref)
        if overlap:
            print("tracked changes in the main checkout overlap the branch:", file=sys.stderr)
            for path in overlap:
                print(path, file=sys.stderr)
            return OVERLAP

        has_remote = remote_exists(repo, args.remote)
        pushing = has_remote and not args.no_push
        last_push_error = ""
        for attempt in range(1, attempts + 1):
            base_ref = fetch_mainline(repo, args.remote, args.mainline) if has_remote else f"refs/heads/{args.mainline}"
            tip = rebase_branch(repo, branch_ref, base_ref)
            merges = merge_commits(repo, base_ref, tip)
            if merges:
                restore_branch(repo, branch_ref, original)
                print("merge commits are not allowed in the landed range:", file=sys.stderr)
                for merge in merges:
                    print(merge, file=sys.stderr)
                return MERGE_COMMIT
            if not run_gates(repo, tip, gates):
                restore_branch(repo, branch_ref, original)
                print(f"gate failed for {tip}", file=sys.stderr)
                return GATE_FAILED
            if pushing:
                pushed = push_tip(repo, args.remote, tip, args.mainline)
                if pushed.returncode != 0:
                    last_push_error = pushed.stderr.strip()
                    if attempt == attempts:
                        restore_branch(repo, branch_ref, original)
                        if last_push_error:
                            print(last_push_error, file=sys.stderr)
                        return LOST_RACE
                    continue
            fast_forward_mainline(repo, args.mainline, tip)
            print(tip)
            return OK

    return LOST_RACE


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("branch", metavar="BRANCH")
    parser.add_argument("--gate", action="append", default=[], help="gate command to run with sh -c; repeatable")
    parser.add_argument("--no-push", action="store_true")
    parser.add_argument("--repo", default=".")
    parser.add_argument("--remote", default="origin")
    parser.add_argument("--mainline", default="main")
    parser.add_argument("--attempts", type=int, default=3)


def add_parser(subparsers) -> argparse.ArgumentParser:
    parser = subparsers.add_parser("land")
    add_arguments(parser)
    parser.set_defaults(func=run)
    return parser


def run(args: argparse.Namespace) -> int:
    return land(args)


class ArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        raise WorkerError(message)


def main(argv: list[str] | None = None) -> int:
    parser = ArgumentParser(description=__doc__)
    add_arguments(parser)
    try:
        return run(parser.parse_args(argv))
    except WorkerError as error:
        print(f"error: {error}", file=sys.stderr)
        return USAGE


if __name__ == "__main__":
    raise SystemExit(main())
