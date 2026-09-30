#!/usr/bin/env python3
"""Create, inspect and retire agent-owned git worktrees.

Commands
  new      create an agent/topic branch and sibling worktree
  status   list worktrees, ownership, age, dirty count and mainline divergence
  finish   remove a clean worktree whose branch is merged into mainline
  gc       dry-run or remove clean merged/abandoned worktrees

Exit codes: 0 ok, 1 refusal, 64 usage/tool error. Standard library only; the
shared git helpers live in worker.py.
"""

from __future__ import annotations

import argparse
import importlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from worker import WorkerError, git, main_worktree, now  # noqa: E402

OK, REFUSED, USAGE = 0, 1, 64
MAINLINE = "main"
MAINLINE_CANDIDATES = (MAINLINE, "mainline", "ultron/mainline", "master")
METADATA = "agent-worktree.json"
COMPONENT = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


class Refusal(Exception):
    pass


class UsageError(Exception):
    pass


class WtParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        self.print_usage(sys.stderr)
        raise UsageError(f"{self.prog}: error: {message}")


def repo_root(repo: str | None) -> Path:
    return main_worktree(repo or ".").resolve()


def rev(repo: str | Path, ref: str) -> str | None:
    out = git(repo, "rev-parse", "--verify", "-q", ref + "^{commit}", check=False).strip()
    return out or None


def branch_exists(repo: str | Path, branch: str) -> bool:
    return rev(repo, f"refs/heads/{branch}") is not None


def mainline_ref(repo: str | Path) -> str:
    for candidate in MAINLINE_CANDIDATES:
        if rev(repo, candidate):
            return candidate
    return MAINLINE


def is_ancestor(repo: str | Path, older: str, newer: str) -> bool:
    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", older, newer],
        capture_output=True,
    ).returncode == 0


def git_dir(worktree: str | Path) -> Path:
    return Path(git(worktree, "rev-parse", "--path-format=absolute", "--git-dir").strip())


def metadata_path(worktree: str | Path) -> Path:
    return git_dir(worktree) / METADATA


def read_metadata(worktree: str | Path) -> dict:
    path = metadata_path(worktree)
    if not path.exists():
        return {}
    try:
        data = json.loads(path.read_text())
    except (OSError, ValueError):
        return {}
    return data if isinstance(data, dict) else {}


def write_metadata(worktree: Path, data: dict) -> None:
    metadata_path(worktree).write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def validate_component(name: str, value: str) -> str:
    if not COMPONENT.match(value):
        raise WorkerError(f"bad {name} {value!r}: use letters, digits, dots, underscores or hyphens")
    return value


def worktrees(repo: str | Path) -> list[dict]:
    rows: list[dict] = []
    current: dict = {}
    for line in git(repo, "worktree", "list", "--porcelain").splitlines() + [""]:
        if not line:
            if current.get("path"):
                rows.append(current)
            current = {}
        elif line.startswith("worktree "):
            current["path"] = line[9:]
        elif line.startswith("HEAD "):
            current["head"] = line[5:]
        elif line.startswith("branch "):
            current["branch"] = line[7:].removeprefix("refs/heads/")
        elif line == "detached":
            current["detached"] = True
    return rows


def dirty_count(worktree: str | Path) -> int:
    return len(git(worktree, "status", "--porcelain", "-uall").splitlines())


def ahead_behind(repo: str | Path, ref: str, mainline: str = MAINLINE) -> tuple[int | None, int | None]:
    if not rev(repo, mainline) or not rev(repo, ref):
        return None, None
    out = git(repo, "rev-list", "--left-right", "--count", f"{mainline}...{ref}", check=False).strip()
    parts = out.split()
    if len(parts) != 2:
        return None, None
    behind, ahead = (int(parts[0]), int(parts[1]))
    return ahead, behind


def parse_created(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    except ValueError:
        return None


def age_seconds(metadata: dict) -> int | None:
    created = parse_created(metadata.get("created"))
    if created is None:
        return None
    return max(0, int((datetime.now(timezone.utc) - created).total_seconds()))


def format_age(seconds: int | None) -> str:
    if seconds is None:
        return "-"
    if seconds < 60:
        return f"{seconds}s"
    if seconds < 3600:
        return f"{seconds // 60}m"
    if seconds < 86400:
        return f"{seconds // 3600}h"
    return f"{seconds // 86400}d"


def owner_for(branch: str, metadata: dict) -> str:
    owner = metadata.get("agent")
    if isinstance(owner, str) and owner:
        return owner
    if "/" in branch:
        return branch.split("/", 1)[0]
    return branch or "-"


def status_rows(repo: Path, mainline: str = MAINLINE) -> list[dict]:
    rows = []
    for item in worktrees(repo):
        path = Path(item["path"])
        branch = item.get("branch") or f"detached:{item.get('head', '')[:12]}"
        metadata = read_metadata(path)
        ahead, behind = ahead_behind(repo, branch if item.get("branch") else item.get("head", ""), mainline)
        rows.append(
            {
                "path": str(path),
                "branch": branch,
                "ahead": ahead,
                "behind": behind,
                "dirty": dirty_count(path),
                "owner": owner_for(branch, metadata),
                "age": age_seconds(metadata),
                "created": metadata.get("created"),
            }
        )
    return rows


def cmd_new(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    agent = validate_component("agent", args.agent)
    topic = validate_component("topic", args.topic)
    branch = f"{agent}/{topic}"
    target = root.parent / f"{root.name}-{agent}-{topic}"
    if branch_exists(root, branch):
        raise Refusal(f"branch {branch} already exists")
    if target.exists():
        raise Refusal(f"{target} already exists")
    base = args.base or mainline_ref(root)
    base_sha = rev(root, base)
    if base_sha is None:
        raise WorkerError(f"base {base!r} not found")
    git(root, "worktree", "add", "-q", "-b", branch, str(target), base)
    write_metadata(
        target,
        {
            "agent": agent,
            "topic": topic,
            "branch": branch,
            "base": base,
            "base_sha": base_sha,
            "created": now(),
        },
    )
    print(json.dumps({"path": str(target), "branch": branch, "base_sha": base_sha}, sort_keys=True))
    return OK


def cmd_status(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    rows = status_rows(root, mainline_ref(root))
    if args.json:
        print(json.dumps(rows, indent=2, sort_keys=True))
    else:
        print("path\tbranch\tahead\tbehind\tdirty\towner\tage")
        for row in rows:
            print(
                "{path}\t{branch}\t{ahead}\t{behind}\t{dirty}\t{owner}\t{age}".format(
                    **{
                        **row,
                        "ahead": "-" if row["ahead"] is None else row["ahead"],
                        "behind": "-" if row["behind"] is None else row["behind"],
                        "age": format_age(row["age"]),
                    }
                )
            )
    return OK


def worktree_for_branch(repo: Path, branch: str) -> Path | None:
    for item in worktrees(repo):
        if item.get("branch") == branch:
            return Path(item["path"])
    return None


def remove_worktree_and_branch(repo: Path, path: Path, branch: str) -> None:
    previous = Path.cwd()
    os.chdir(repo)
    try:
        git(repo, "worktree", "remove", str(path))
        if branch_exists(repo, branch):
            git(repo, "branch", "-D", branch)
    finally:
        try:
            os.chdir(previous)
        except FileNotFoundError:
            os.chdir(repo)


def cmd_finish(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    mainline = mainline_ref(root)
    branch = args.branch
    if branch == mainline:
        raise Refusal(f"refusing to finish mainline branch {mainline}")
    if not branch_exists(root, branch):
        raise Refusal(f"branch {branch} does not exist")
    target = worktree_for_branch(root, branch)
    if target is None:
        raise Refusal(f"no worktree found for {branch}")
    if target.resolve() == root.resolve():
        raise Refusal("refusing to remove the main worktree")
    if dirty_count(target):
        raise Refusal(f"worktree {target} is dirty")
    if not is_ancestor(root, branch, mainline):
        raise Refusal(f"{branch} is not merged into {mainline}")
    remove_worktree_and_branch(root, target, branch)
    print(json.dumps({"removed": str(target), "branch": branch}, sort_keys=True))
    return OK


def gc_candidates(repo: Path, days: int, mainline: str) -> list[dict]:
    older_than = days * 86400
    candidates = []
    for item in worktrees(repo):
        path = Path(item["path"])
        branch = item.get("branch")
        if path.resolve() == repo.resolve() or not branch or branch == mainline:
            continue
        dirty = dirty_count(path)
        if dirty:
            continue
        ahead, behind = ahead_behind(repo, branch, mainline)
        if ahead is None:
            continue
        metadata = read_metadata(path)
        age = age_seconds(metadata)
        abandoned = age is not None and age >= older_than and ahead == 0
        merged = is_ancestor(repo, branch, mainline)
        if not (merged or abandoned):
            continue
        candidates.append(
            {
                "path": str(path),
                "branch": branch,
                "ahead": ahead,
                "behind": behind,
                "dirty": dirty,
                "age": age,
                "reason": "abandoned" if abandoned else "merged",
            }
        )
    return candidates


def cmd_gc(args: argparse.Namespace) -> int:
    if args.days < 0:
        raise WorkerError("--days must be non-negative")
    root = repo_root(args.repo)
    rows = gc_candidates(root, args.days, mainline_ref(root))
    for row in rows:
        if args.apply:
            remove_worktree_and_branch(root, Path(row["path"]), row["branch"])
        row["removed"] = bool(args.apply)
        print(json.dumps(row, sort_keys=True))
    return OK


def _load_plugins(subparsers: argparse._SubParsersAction) -> None:
    try:
        land = importlib.import_module("land")
    except ModuleNotFoundError as exc:
        if exc.name == "land":
            return
        raise
    land.add_parser(subparsers)


def build_parser() -> WtParser:
    parser = WtParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True, parser_class=WtParser)

    p = sub.add_parser("new")
    p.add_argument("--agent", required=True)
    p.add_argument("--topic", required=True)
    p.add_argument("--base")
    p.add_argument("--repo", default=".")
    p.set_defaults(func=cmd_new)

    p = sub.add_parser("status")
    p.add_argument("--json", action="store_true")
    p.add_argument("--repo", default=".")
    p.set_defaults(func=cmd_status)

    p = sub.add_parser("finish")
    p.add_argument("branch")
    p.add_argument("--repo", default=".")
    p.set_defaults(func=cmd_finish)

    p = sub.add_parser("gc")
    p.add_argument("--apply", action="store_true")
    p.add_argument("--days", type=int, default=7)
    p.add_argument("--repo", default=".")
    p.set_defaults(func=cmd_gc)

    _load_plugins(sub)
    return parser


def main(argv: list[str] | None = None) -> int:
    try:
        args = build_parser().parse_args(argv)
        return args.func(args)
    except UsageError as error:
        print(error, file=sys.stderr)
        return USAGE
    except Refusal as error:
        print(f"wt: {error}", file=sys.stderr)
        return REFUSED
    except WorkerError as error:
        print(f"wt: {error}", file=sys.stderr)
        return USAGE


if __name__ == "__main__":
    raise SystemExit(main())
