#!/usr/bin/env python3
"""Land an exact candidate commit through gates without rewriting it."""

from __future__ import annotations

import argparse
import contextlib
from dataclasses import dataclass
import fcntl
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from worker import WorkerError, git, git_env, main_worktree, now, temp_parent  # noqa: E402

(
    OK,
    OVERLAP,
    MERGE_COMMIT,
    GATE_FAILED,
    STALE_CANDIDATE,
    ACCEPTANCE_FAILED,
    MAINLINE_REFUSED,
    NO_GATE_REFUSED,
    LOCAL_FF_FAILED,
    RECORD_BRANCH_REQUIRED,
    USAGE,
) = 0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 64
LOST_RACE = STALE_CANDIDATE
SLUG = re.compile(r"^[a-z0-9][a-z0-9-]*$")


@dataclass(frozen=True)
class WorktreeRecord:
    path: Path
    branch: str | None


@dataclass(frozen=True)
class EffortRecord:
    effort: str
    worktree: Path
    branch: str | None
    path: Path
    metadata: dict[str, str | None]
    text: str


@dataclass(frozen=True)
class EffortRecordProblem:
    effort: str
    path: Path
    message: str


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
    return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True, env=git_env())


def rev(repo: Path, ref: str) -> str:
    out = git(repo, "rev-parse", "--verify", "-q", ref + "^{commit}", check=False).strip()
    if not out:
        raise WorkerError(f"ref {ref!r} not found")
    return out


def normalize_branch(branch: str | None) -> str | None:
    return branch.removeprefix("refs/heads/") if branch else None


def parse_effort_metadata(text: str) -> dict[str, str | None]:
    out = {"base": None, "integration_branch": None, "record_branch": None}
    for key, field in (
        ("Base", "base"),
        ("Integration branch", "integration_branch"),
        ("Record branch", "record_branch"),
    ):
        match = re.search(rf"^{re.escape(key)}:\s*(\S.*)?$", text, re.M)
        if match:
            value = (match.group(1) or "").strip()
            out[field] = value or None
    return out


def repository_worktrees(repo: Path) -> list[WorktreeRecord]:
    root = repo.resolve()
    entries: list[WorktreeRecord] = []
    current: dict[str, str] = {}
    for line in git(repo, "worktree", "list", "--porcelain").splitlines() + [""]:
        if not line:
            if current.get("path"):
                entries.append(WorktreeRecord(Path(current["path"]).resolve(), normalize_branch(current.get("branch"))))
            current = {}
        elif line.startswith("worktree "):
            current["path"] = line[9:]
        elif line.startswith("branch "):
            current["branch"] = line[7:]
    if root not in [entry.path for entry in entries]:
        entries.append(WorktreeRecord(root, normalize_branch(git(root, "symbolic-ref", "-q", "--short", "HEAD", check=False).strip())))
    entries.sort(key=lambda entry: (entry.path != root, str(entry.path)))
    return entries


def scan_effort_records(repo: Path, effort: str | None = None) -> tuple[dict[str, list[EffortRecord]], list[EffortRecordProblem]]:
    if effort is not None and not SLUG.match(effort):
        raise WorkerError(f"bad effort slug {effort!r}: use lowercase letters, digits, hyphens")
    candidates: dict[str, list[EffortRecord]] = {}
    problems: list[EffortRecordProblem] = []
    for worktree in repository_worktrees(repo):
        efforts = worktree.path / "docs" / "efforts"
        if effort is not None:
            paths = [efforts / effort / "EFFORT.md"]
        elif efforts.is_dir():
            paths = [path / "EFFORT.md" for path in sorted(efforts.iterdir()) if path.is_dir()]
        else:
            paths = []
        for path in paths:
            if not path.is_file():
                continue
            try:
                text = path.read_text()
            except OSError as error:
                problems.append(EffortRecordProblem(path.parent.name, path, f"could not read effort record: {error}"))
                continue
            record = EffortRecord(
                effort=path.parent.name,
                worktree=worktree.path,
                branch=worktree.branch,
                path=path,
                metadata=parse_effort_metadata(text),
                text=text,
            )
            candidates.setdefault(record.effort, []).append(record)
    return candidates, problems


def effort_record_candidates(repo: Path, effort: str | None = None) -> dict[str, list[EffortRecord]]:
    candidates, problems = scan_effort_records(repo, effort)
    relevant = [problem for problem in problems if effort is None or problem.effort == effort]
    if relevant:
        raise WorkerError("; ".join(f"{problem.path}: {problem.message}" for problem in relevant))
    return candidates


def effort_record_locations(records: list[EffortRecord]) -> str:
    return "; ".join(
        f"{record.path} (branch {record.branch or 'detached'}, "
        f"Integration branch {record.metadata.get('integration_branch') or 'missing'}, "
        f"Record branch {record.metadata.get('record_branch') or 'missing'})"
        for record in records
    )


def selected_by_metadata_branch(records: list[EffortRecord], field: str) -> list[EffortRecord]:
    return [
        record for record in records
        if record.metadata.get(field) and record.branch == normalize_branch(record.metadata.get(field))
    ]


def select_effort_record(repo: Path, effort: str, records: list[EffortRecord]) -> EffortRecord:
    if not records:
        searched = ", ".join(str(worktree.path) for worktree in repository_worktrees(repo))
        raise WorkerError(f"effort record docs/efforts/{effort}/EFFORT.md not found in {searched}")
    preferred = selected_by_metadata_branch(records, "record_branch")
    if len(preferred) == 1:
        return preferred[0]
    preferred = selected_by_metadata_branch(records, "integration_branch")
    if len(preferred) == 1:
        return preferred[0]
    local = [record for record in records if record.worktree == repo.resolve()]
    if local:
        return local[0]
    if len(records) == 1:
        return records[0]
    if all(not record.metadata.get("integration_branch") and not record.metadata.get("record_branch") for record in records):
        raise WorkerError(
            f"effort record for {effort!r} is not in this checkout; found only other worktree copies: "
            f"{effort_record_locations(records)}"
        )
    raise WorkerError(
        f"ambiguous effort record for {effort!r}: found multiple records and no single worktree is checked out "
        f"on the record's Record branch or Integration branch: {effort_record_locations(records)}"
    )


def resolve_effort_record(repo: Path, effort: str) -> EffortRecord:
    return select_effort_record(repo, effort, effort_record_candidates(repo, effort).get(effort, []))


def resolve_effort_records(repo: Path) -> list[EffortRecord]:
    records = effort_record_candidates(repo)
    return [select_effort_record(repo, effort, records[effort]) for effort in sorted(records)]


def effort_metadata(repo: Path, effort: str) -> dict[str, str | None]:
    return resolve_effort_record(repo, effort).metadata


def resolve_target_branch(args: argparse.Namespace, repo: Path) -> tuple[str, EffortRecord | None]:
    if not args.ticket:
        return args.mainline or "main", None
    record = resolve_effort_record(repo, args.effort)
    target = args.mainline or record.metadata.get("integration_branch")
    if not target:
        raise WorkerError(f"effort {args.effort!r} does not name an integration branch")
    return target, record


def is_base_branch(target: str, base: str | None) -> bool:
    return bool(base) and target.removeprefix("refs/heads/") == base.removeprefix("refs/heads/")


def record_mainline_override(checkout: Path, effort: str, ticket: str, candidate: str, target: str, owner: str) -> Path:
    path = checkout / "docs" / "efforts" / effort / "evidence" / "mainline-landing-overrides.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "schema": "corenius.mainline_landing_override.v1",
        "effort": effort,
        "ticket": ticket,
        "candidate": candidate,
        "target": target,
        "owner": owner,
        "recorded_at": now(),
    }
    with path.open("a") as handle:
        handle.write(json.dumps(record, sort_keys=True) + "\n")
    return path


def record_landing(checkout: Path, effort: str, ticket: str, candidate: str, target: str) -> Path:
    path = checkout / "docs" / "efforts" / effort / "evidence" / f"landing-{ticket}.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "schema": "corenius.ticket_landing.v1",
        "effort": effort,
        "ticket": ticket,
        "candidate": candidate,
        "target": target,
        "landed_at": now(),
    }
    with path.open("a") as handle:
        handle.write(json.dumps(record, sort_keys=True) + "\n")
    return path


def commit_effort_record_change(checkout: Path, message: str, paths: list[Path]) -> None:
    relative = [str(path.resolve().relative_to(checkout.resolve())) for path in paths]
    git(checkout, "add", *relative)
    if git(checkout, "diff", "--cached", "--name-only", "--", *relative).strip():
        git(checkout, "commit", "-q", "-m", message, "--", *relative)


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


def worktree_for_ref(repo: Path, ref: str) -> Path | None:
    current: dict[str, str] = {}
    for line in git(repo, "worktree", "list", "--porcelain").splitlines() + [""]:
        if not line:
            if current.get("path") and current.get("branch") == ref:
                return Path(current["path"])
            current = {}
        elif line.startswith("worktree "):
            current["path"] = line[9:]
        elif line.startswith("branch "):
            current["branch"] = line[7:]
    return None


def tracked_overlap(repo: Path, checkout: Path | None, mainline: str, branch: str) -> list[str]:
    if checkout is None:
        return []
    dirty = parse_status_paths(git(checkout, "status", "--porcelain", "--untracked-files=no"))
    changed = set(git(repo, "diff", "--name-only", f"{mainline}...{branch}").splitlines())
    return sorted(dirty & changed)


def untracked_overlap(repo: Path, checkout: Path | None, mainline: str, branch: str) -> list[str]:
    if checkout is None:
        return []
    untracked = set(parse_status_paths(git(checkout, "status", "--porcelain", "--untracked-files=all")))
    tracked_dirty = parse_status_paths(git(checkout, "status", "--porcelain", "--untracked-files=no"))
    untracked -= tracked_dirty
    changed = set(git(repo, "diff", "--name-only", f"{mainline}...{branch}").splitlines())
    return sorted(untracked & changed)


def remote_exists(repo: Path, remote: str) -> bool:
    return remote in git(repo, "remote").splitlines()


def is_ancestor(repo: Path, older: str, newer: str) -> bool:
    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", older, newer],
        capture_output=True,
        env=git_env(),
    ).returncode == 0


@contextlib.contextmanager
def temporary_worktree(repo: Path, ref: str, prefix: str, detach: bool):
    parent = temp_parent()
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


def merge_commits(repo: Path, base_ref: str, tip: str) -> list[str]:
    return git(repo, "rev-list", "--merges", f"{base_ref}..{tip}").splitlines()


def run_gates(repo: Path, tip: str, gates: list[str]) -> bool:
    with temporary_worktree(repo, tip, ".land-gate-", detach=True) as checkout:
        for gate in gates:
            env = os.environ.copy()
            for name in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR"):
                env.pop(name, None)
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


def local_mainline_ref(mainline: str) -> str:
    return mainline if mainline.startswith("refs/heads/") else f"refs/heads/{mainline}"


def remote_head(repo: Path, remote: str, mainline: str) -> str:
    result = git_result(repo, "ls-remote", "--heads", remote, mainline)
    if result.returncode != 0:
        raise WorkerError(result.stderr.strip() or f"remote {remote!r} could not be read")
    for line in result.stdout.splitlines():
        parts = line.split()
        if len(parts) == 2 and parts[1] == f"refs/heads/{mainline}":
            return parts[0]
    raise WorkerError(f"remote branch {remote}/{mainline} not found")


def ref_is_stale(repo: Path, base: str, tip: str) -> bool:
    return not is_ancestor(repo, base, tip)


def print_stale(mainline: str, base: str, tip: str) -> None:
    print(
        f"{mainline} is at {base}; {tip} is stale and a new Candidate on the new base is required",
        file=sys.stderr,
    )


def fast_forward_ref(repo: Path, ref: str, expected: str, tip: str, checkout: Path | None) -> tuple[bool, str]:
    if rev(repo, ref) != expected or ref_is_stale(repo, expected, tip):
        return False, "stale"
    if checkout is not None:
        result = git_result(checkout, "merge", "--ff-only", "-q", tip)
        if result.returncode != 0:
            return False, result.stderr.strip() or result.stdout.strip() or "target checkout cannot fast-forward"
        return True, ""
    result = git_result(repo, "update-ref", ref, tip, expected)
    if result.returncode != 0:
        return False, result.stderr.strip() or result.stdout.strip() or "target ref cannot fast-forward"
    return True, ""


def land(args: argparse.Namespace) -> int:
    repo = repo_root(args.repo or ".")
    gates = list(args.gate or [])
    if bool(args.effort) != bool(args.ticket):
        raise WorkerError("--effort and --ticket must be supplied together")

    with land_lock(repo):
        tip = rev(repo, args.branch)
        target_branch, effort_record = resolve_target_branch(args, repo)
        effort_meta = effort_record.metadata if effort_record is not None else {"base": "main", "integration_branch": None}
        base_branch = effort_meta.get("base") or "main"
        protected_target = is_base_branch(target_branch, base_branch) or (effort_record is None and normalize_branch(target_branch) == "main")
        if protected_target and not args.owner_mainline_override:
            print(
                f"owner mainline override is required to land on {target_branch}",
                file=sys.stderr,
            )
            return MAINLINE_REFUSED
        if args.owner_mainline_override and not protected_target:
            raise WorkerError("--owner-mainline-override applies only when landing to the effort base branch or standalone main")
        if args.owner_mainline_override and not args.owner_mainline_override.strip():
            raise WorkerError("--owner-mainline-override must name the owner")
        if args.ticket and not effort_record.metadata.get("record_branch"):
            print(
                f"ticket landings need effort records on a record branch; "
                f"run effort.py move-records {args.effort}",
                file=sys.stderr,
            )
            return RECORD_BRANCH_REQUIRED
        if not gates and not args.allow_no_gate:
            print("no gate was supplied; pass --allow-no-gate to land without a gate", file=sys.stderr)
            return NO_GATE_REFUSED
        target_ref = local_mainline_ref(target_branch)
        target_checkout = worktree_for_ref(repo, target_ref)
        overlap = tracked_overlap(repo, target_checkout, target_branch, tip) + untracked_overlap(repo, target_checkout, target_branch, tip)
        if overlap:
            print("target checkout cannot fast-forward because local files overlap the branch:", file=sys.stderr)
            for path in overlap:
                print(path, file=sys.stderr)
            return OVERLAP

        has_remote = remote_exists(repo, args.remote)
        pushing = has_remote and not args.no_push
        local_before = rev(repo, target_ref)
        target_before = remote_head(repo, args.remote, target_branch) if pushing else local_before
        if ref_is_stale(repo, target_before, tip):
            print_stale(target_branch, target_before, tip)
            return STALE_CANDIDATE
        if pushing and ref_is_stale(repo, local_before, tip):
            print_stale(target_branch, local_before, tip)
            return STALE_CANDIDATE
        merges = merge_commits(repo, target_before, tip)
        if merges:
            print("merge commits are not allowed in the landed range:", file=sys.stderr)
            for merge in merges:
                print(merge, file=sys.stderr)
            return MERGE_COMMIT
        if args.ticket:
            import acceptance  # noqa: WPS433
            try:
                record_branch = effort_record.metadata["record_branch"]
                acceptance.require_committed_for_landing(repo, record_branch, args.effort, args.ticket, tip)
            except WorkerError as error:
                print(f"acceptance refused: {error}", file=sys.stderr)
                return ACCEPTANCE_FAILED
        if not run_gates(repo, tip, gates):
            print(f"gate failed for {tip}", file=sys.stderr)
            return GATE_FAILED
        if pushing:
            pushed = push_tip(repo, args.remote, tip, target_branch)
            if pushed.returncode != 0:
                if pushed.stderr.strip():
                    print(pushed.stderr.strip(), file=sys.stderr)
                print_stale(target_branch, target_before, tip)
                return STALE_CANDIDATE
        forwarded, reason = fast_forward_ref(repo, target_ref, local_before, tip, target_checkout)
        if not forwarded:
            if reason == "stale":
                print_stale(target_branch, local_before, tip)
                return STALE_CANDIDATE
            print(f"local fast-forward failed for {target_branch}: {reason}", file=sys.stderr)
            return LOCAL_FF_FAILED
        if args.ticket and effort_record.metadata.get("record_branch"):
            landing_path = record_landing(effort_record.worktree, args.effort, args.ticket, tip, target_branch)
            commit_effort_record_change(effort_record.worktree, f"Record landing for ticket {args.ticket}", [landing_path])
        if args.owner_mainline_override and effort_record is not None:
            override_path = record_mainline_override(
                effort_record.worktree,
                args.effort,
                args.ticket,
                tip,
                target_branch,
                args.owner_mainline_override.strip(),
            )
            if effort_record.metadata.get("record_branch"):
                commit_effort_record_change(
                    effort_record.worktree,
                    f"Record mainline override for ticket {args.ticket}",
                    [override_path],
                )
        print(tip)
        return OK


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("branch", metavar="CANDIDATE")
    parser.add_argument("--gate", action="append", default=[], help="gate command to run with sh -c; repeatable")
    parser.add_argument("--allow-no-gate", action="store_true", help="explicitly allow landing without a gate command")
    parser.add_argument("--no-push", action="store_true")
    parser.add_argument("--repo", default=".")
    parser.add_argument("--remote", default="origin")
    parser.add_argument("--mainline")
    parser.add_argument("--owner-mainline-override", help="owner name authorizing an effort ticket to land on its base branch")
    parser.add_argument("--effort", help="effort containing the acceptance record")
    parser.add_argument(
        "--ticket",
        help=(
            "ticket id whose acceptance record must already be committed on the effort record branch; "
            "ticket landings need effort records on a record branch. Run effort.py move-records <effort> first"
        ),
    )


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
