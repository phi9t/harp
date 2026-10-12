#!/usr/bin/env python3
"""Deprecated compatibility CLI; use tools/collab/collab.py for new Attempts."""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import json
import os
import re
import signal
import threading
import subprocess
import sys
import tempfile
from pathlib import Path

from common import Refused, digest, failpoint
from legacy_journal import LegacyJournal
from landing_protocol import classify_push_failure, gate_workspace_status, process_group_alive
from state_store import StoreError

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "agents"))
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
RECORDS_INCOMPLETE = 12
SLUG = re.compile(r"^[a-z0-9][a-z0-9-]*$")


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
            journal = LegacyJournal(common_git_dir(repo))
            if journal.store.load_config() is not None:
                with journal.store.lock("lane"):
                    yield
            else:
                yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def git_result(repo: str | Path, *args: str, env: dict | None = None) -> subprocess.CompletedProcess[str]:
    environment = git_env()
    environment.update(env or {})
    return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True, env=environment)


def rev(repo: Path, ref: str) -> str:
    out = git(repo, "rev-parse", "--verify", "-q", ref + "^{commit}", check=False).strip()
    if not out:
        raise WorkerError(f"ref {ref!r} not found")
    return out


def is_base_branch(target: str, base: str | None) -> bool:
    return bool(base) and target.removeprefix("refs/heads/") == base.removeprefix("refs/heads/")


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
def temporary_worktree(repo: Path, ref: str, prefix: str, detach: bool, *, keep: bool = False):
    parent = temp_parent()
    path = Path(tempfile.mkdtemp(prefix=prefix, dir=str(parent)))
    added = False
    git_directory = None
    tip = None
    try:
        args = ["worktree", "add", "-q"]
        if detach:
            args.append("--detach")
        git(repo, *args, str(path), ref)
        added = True
        git_directory = git(path, "rev-parse", "--absolute-git-dir").strip()
        tip = rev(path, "HEAD")
        yield path
    finally:
        if added and not keep:
            status = gate_workspace_status(repo, path, tip, git_directory)
            if status["identity"] and status["same_head"] and status["clean"]:
                git(repo, "worktree", "remove", str(path), check=False)
        elif not added and path.exists():
            try:
                path.rmdir()
            except OSError:
                pass


def merge_commits(repo: Path, base_ref: str, tip: str) -> list[str]:
    return git(repo, "rev-list", "--merges", f"{base_ref}..{tip}").splitlines()


def run_gates(
    repo: Path, tip: str, gates: list[str], *, journal: LegacyJournal | None = None,
    operation: dict | None = None, timeout_seconds: float | None = None,
) -> bool:
    retained = journal is not None and operation is not None
    with temporary_worktree(repo, tip, ".land-gate-", detach=True, keep=True) as checkout:
        git_directory = git(checkout, "rev-parse", "--absolute-git-dir").strip()
        if retained:
            operation["check_path"] = str(checkout)
            operation["check_git_dir"] = git_directory
            journal.event(operation, "gate_workspace", check_path=str(checkout), check_git_dir=git_directory)
        log = journal.home / (operation["id"] + "-gate.log") if journal and operation else None
        outcome, exit_code = "passed", 0
        process_groups = []
        for gate in gates:
            env = git_env()
            env["LAND_TIP"] = tip
            process = subprocess.Popen(
                ["sh", "-c", gate], cwd=checkout, env=env,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, start_new_session=True,
            )
            process_groups.append(process.pid)
            errors = []

            def stream():
                try:
                    with log.open("ab") if log else contextlib.nullcontext() as output:
                        while chunk := process.stdout.read1(65536):
                            if output:
                                output.write(chunk)
                                output.flush()
                            print(chunk.decode(errors="replace"), end="", file=sys.stderr, flush=True)
                        if output:
                            os.fsync(output.fileno())
                except BaseException as error:
                    errors.append(error)

            reader = threading.Thread(target=stream, daemon=True)
            reader.start()
            try:
                exit_code = process.wait(timeout=timeout_seconds)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                exit_code = process.wait()
                outcome = "timeout"
            reader.join(timeout=2)
            if reader.is_alive():
                os.killpg(process.pid, signal.SIGKILL)
                reader.join(timeout=2)
                outcome = "failed"
            if process.stdout:
                process.stdout.close()
            if errors:
                raise WorkerError(f"Gate output could not be recorded: {errors[0]}")
            if outcome == "timeout" or exit_code:
                if outcome != "timeout":
                    outcome = "failed"
                break
        workspace = gate_workspace_status(repo, checkout, tip, git_directory)
        quiescent = not any(process_group_alive(identity) for identity in process_groups)
        if not workspace["clean"] or not workspace["same_head"] or not workspace["identity"]:
            outcome = "invalid_subject"
        elif outcome == "passed" and not quiescent:
            outcome = "invalid_subject"
        if journal and operation:
            if not log.exists():
                log.touch()
            journal.event(
                operation, "gate_receipt", subject=tip, base=operation["local_before"],
                version=2, check_path=str(checkout), check_git_dir=git_directory,
                argv=[["sh", "-c", gate] for gate in gates], outcome=outcome,
                exit_code=exit_code, timeout_seconds=timeout_seconds, log=str(log),
                log_sha256=digest(log.read_bytes()), quiescent=quiescent, process_groups=process_groups,
                **workspace,
            )
        elif outcome == "passed":
            git(repo, "worktree", "remove", str(checkout), check=False)
        return outcome == "passed"


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


def fast_forward_ref(
    repo: Path, ref: str, expected: str, tip: str, checkout: Path | None, action: str | None = None,
) -> tuple[bool, str]:
    if rev(repo, ref) != expected or ref_is_stale(repo, expected, tip):
        return False, "stale"
    if checkout is not None:
        result = git_result(
            checkout, "merge", "--ff-only", "--no-overwrite-ignore", "-q", tip,
            env={"GIT_REFLOG_ACTION": action} if action else None,
        )
        if result.returncode != 0:
            return False, result.stderr.strip() or result.stdout.strip() or "target checkout cannot fast-forward"
        return True, ""
    args = ["update-ref"]
    if action:
        args.extend(["-m", action])
    result = git_result(repo, *args, ref, tip, expected)
    if result.returncode != 0:
        return False, result.stderr.strip() or result.stdout.strip() or "target ref cannot fast-forward"
    return True, ""


def finish_records(journal: LegacyJournal, operation: dict) -> int:
    try:
        if operation.get("ticket"):
            import effort_records
            effort_records.finish_landing_records(operation)
        journal.change(operation, "landed")
        release_gate_workspace(journal, operation)
        return OK
    except Exception as error:
        try:
            journal.change(operation, "landed_records_incomplete", reason=str(error), repair="retry --recover-landing")
        except Exception as record_error:
            # The attributed local Git movement and the preceding local_started
            # record still let recovery reconcile a failed record write.
            print(f"partial-success record could not be completed: {record_error}", file=sys.stderr)
        print(f"landed {operation['tip']}, records incomplete: {error}; recover {operation['id']}", file=sys.stderr)
        return RECORDS_INCOMPLETE


def release_gate_workspace(journal: LegacyJournal, operation: dict) -> None:
    try:
        journal.require_gate_receipt(operation)
        checkout = Path(operation["check_path"])
        git(operation["repo"], "worktree", "remove", "--force", str(checkout))
    except (WorkerError, Refused, OSError) as error:
        print(f"gate workspace retained for inspection: {error}", file=sys.stderr)


def require_legacy_candidate(journal: LegacyJournal, candidate_ref: str, tip: str) -> None:
    if journal.store.load_config() is None:
        return
    for attempt in journal.store.attempts():
        if attempt["state"] in {"landed", "abandoned"}:
            continue
        registered = candidate_ref.removeprefix("refs/heads/") == attempt["branch"]
        registered = registered or tip in {attempt.get("candidate"), attempt.get("integration")}
        path = Path(attempt["path"])
        if path.exists():
            head = git(path, "rev-parse", "--verify", "HEAD", check=False).strip()
            registered = registered or (head != attempt["base"] and head == tip)
        if registered:
            raise Refused(f"Candidate belongs to Attempt {attempt['id']}; use the collab lifecycle")


def recover_landing(repo: Path, journal: LegacyJournal, identity: str) -> int:
    from landing_protocol import preflight
    with land_lock(repo):
        operation = journal.read(identity)
        if operation["repo"] != str(repo.resolve()):
            raise Refused("Compatibility landing repository identity changed")
        tip = operation["tip"]
        target_ref = operation["target_ref"]
        local_before = operation["local_before"]
        state = operation["state"]
        if state == "landed":
            print(tip)
            return OK
        if state not in {"prepared", "push_started", "pushed", "local_started", "landed_records_incomplete"}:
            raise Refused("This compatibility landing has no interrupted Git phase to recover")
        journal.require_gate_receipt(operation)
        current = rev(repo, target_ref)
        if current not in {local_before, tip}:
            raise Refused("Target moved beyond the bound landing; preserve and inspect")
        if operation["pushing"]:
            observed = remote_head(repo, operation["remote"], operation["target_branch"])
            no_push_observed = (
                observed == operation["remote_before"]
                and current == local_before
                and state in {"prepared", "push_started"}
            )
            if no_push_observed:
                journal.change(operation, "rolled_back")
                print(json.dumps({"landing": identity, "action": "no push observed; no refs changed"}))
                return OK
            if observed != tip:
                raise Refused("Remote push outcome is unknown or moved; preserve and inspect")
        if current == local_before and current != tip:
            checkout = worktree_for_ref(repo, target_ref)
            expected_checkout = operation["target_checkout"]
            if (str(checkout.resolve()) if checkout else None) != expected_checkout:
                raise Refused("Target checkout identity changed; preserve and inspect")
            preflight(repo, target_ref, local_before, tip, checkout)
            journal.change(operation, "local_started")
            forwarded, reason = fast_forward_ref(
                repo, target_ref, local_before, tip, checkout, f"legacy-land:{identity}",
            )
            if not forwarded:
                raise Refused(f"Recovery local fast-forward refused: {reason}")
        elif current == tip and local_before != tip:
            entries = git(repo, "reflog", "show", "--format=%H%x00%gs", target_ref).splitlines()
            attributed = any(
                entry.startswith(tip + "\0legacy-land:" + identity)
                and index + 1 < len(entries)
                and entries[index + 1].split("\0", 1)[0] == local_before
                for index, entry in enumerate(entries)
            )
            if not attributed:
                raise Refused("Local movement is not attributed to this landing; preserve and inspect")
        code = finish_records(journal, operation)
        print(tip)
        return code


def land(args: argparse.Namespace) -> int:
    repo = repo_root(args.repo or ".")
    journal = LegacyJournal(common_git_dir(repo))
    if getattr(args, "recover_landing", None):
        return recover_landing(repo, journal, args.recover_landing)
    if getattr(args, "landing_status", None):
        print(json.dumps(journal.read(args.landing_status), sort_keys=True))
        return OK
    if not args.branch:
        raise WorkerError("a Candidate is required")
    gates = list(args.gate or [])
    from policy import validate_check_timeout
    timeout = validate_check_timeout(getattr(args, "gate_timeout", None))
    if bool(args.effort) != bool(args.ticket):
        raise WorkerError("--effort and --ticket must be supplied together")

    with land_lock(repo):
        journal.require_no_pending()
        tip = rev(repo, args.branch)
        require_legacy_candidate(journal, args.branch, tip)
        if getattr(args, "expected_tip", None) and tip != args.expected_tip:
            print(f"expected tip {args.expected_tip}; Candidate now names {tip}", file=sys.stderr)
            return STALE_CANDIDATE
        if args.ticket:
            from effort_catalog import resolve_target_branch
            target_branch, effort_record = resolve_target_branch(args, repo)
        else:
            target_branch, effort_record = args.mainline or "main", None
        target_branch = target_branch.removeprefix("refs/heads/")
        effort_meta = (
            effort_record.metadata if effort_record is not None else {"base": "main", "integration_branch": None}
        )
        base_branch = effort_meta.get("base") or "main"
        protected_target = is_base_branch(target_branch, base_branch) or (
            effort_record is None and target_branch.removeprefix("refs/heads/") == "main"
        )
        if protected_target and not args.owner_mainline_override:
            print(
                f"owner mainline override is required to land on {target_branch}",
                file=sys.stderr,
            )
            return MAINLINE_REFUSED
        if args.owner_mainline_override and not protected_target:
            raise WorkerError(
                "--owner-mainline-override applies only when landing to the effort base branch or standalone main"
            )
        if args.owner_mainline_override and not args.owner_mainline_override.strip():
            raise WorkerError("--owner-mainline-override must name the owner")
        if args.owner_mainline_override:
            from landing_protocol import allowed_override_owners
            if args.owner_mainline_override.strip() not in allowed_override_owners(repo):
                print("mainline override owner is not configured in collab.overrideOwners", file=sys.stderr)
                return MAINLINE_REFUSED
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
        has_remote = remote_exists(repo, args.remote)
        pushing = has_remote and not args.no_push
        local_before = rev(repo, target_ref)
        from landing_protocol import PreflightRefused, preflight
        try:
            preflight(repo, target_ref, local_before, tip, target_checkout)
        except PreflightRefused as error:
            print(str(error), file=sys.stderr)
            for path in error.paths:
                print(path, file=sys.stderr)
            if error.kind == "stale":
                print_stale(target_branch, local_before, tip)
            return {"overlap": OVERLAP, "stale": STALE_CANDIDATE, "local": LOCAL_FF_FAILED}[error.kind]
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
        operation = journal.begin(
            repo=str(repo.resolve()), tip=tip, candidate_ref=args.branch,
            target_ref=target_ref, target_branch=target_branch, local_before=local_before,
            target_checkout=str(target_checkout.resolve()) if target_checkout else None,
            pushing=pushing, remote=args.remote, remote_before=target_before,
            effort=args.effort, ticket=args.ticket,
            record_checkout=str(effort_record.worktree) if effort_record else None,
            record_branch=record_branch if args.ticket else None,
            override_owner=args.owner_mainline_override,
        )
    if not run_gates(repo, tip, gates, journal=journal, operation=operation, timeout_seconds=timeout):
        journal.change(operation, "gate_failed")
        print(f"gate failed for {tip}", file=sys.stderr)
        return GATE_FAILED
    with land_lock(repo):
        journal.require_no_pending()
        require_legacy_candidate(journal, args.branch, tip)
        journal.require_gate_receipt(operation)
        if args.owner_mainline_override and args.owner_mainline_override.strip() not in allowed_override_owners(repo):
            print("mainline override owner is no longer configured", file=sys.stderr)
            return MAINLINE_REFUSED
        if rev(repo, args.branch) != tip:
            print(f"expected tip {tip}; Candidate moved during gates", file=sys.stderr)
            return STALE_CANDIDATE
        try:
            preflight(repo, target_ref, local_before, tip, target_checkout)
        except PreflightRefused as error:
            print(str(error), file=sys.stderr)
            for path in error.paths:
                print(path, file=sys.stderr)
            if error.kind == "stale":
                print_stale(target_branch, rev(repo, target_ref), tip)
            return {"overlap": OVERLAP, "stale": STALE_CANDIDATE, "local": LOCAL_FF_FAILED}[error.kind]
        if args.ticket:
            try:
                acceptance.require_committed_for_landing(repo, record_branch, args.effort, args.ticket, tip)
            except WorkerError as error:
                print(f"acceptance refused: {error}", file=sys.stderr)
                return ACCEPTANCE_FAILED
        journal.change(operation, "prepared")
        if pushing:
            journal.change(operation, "push_started")
            pushed = push_tip(repo, args.remote, tip, target_branch)
            if pushed.returncode != 0:
                reason = classify_push_failure(pushed)
                print(reason, file=sys.stderr)
                try:
                    observed = remote_head(repo, args.remote, target_branch)
                except WorkerError:
                    observed = None
                if observed == target_before:
                    journal.change(operation, "push_failed", reason=reason)
                elif observed == tip:
                    journal.change(operation, "pushed", reason=reason)
                else:
                    journal.event(operation, "push_uncertain", reason=reason, observed=observed)
                return STALE_CANDIDATE
            journal.change(operation, "pushed")
            failpoint("after_legacy_push")
        journal.change(operation, "local_started")
        failpoint("before_legacy_local")
        forwarded, reason = fast_forward_ref(
            repo, target_ref, local_before, tip, target_checkout, f"legacy-land:{operation['id']}",
        )
        if not forwarded:
            if reason == "stale":
                print_stale(target_branch, local_before, tip)
                return STALE_CANDIDATE
            print(f"local fast-forward failed for {target_branch}: {reason}", file=sys.stderr)
            return LOCAL_FF_FAILED
        failpoint("after_legacy_local")
        code = finish_records(journal, operation)
        print(tip)
        return code


def add_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("branch", metavar="CANDIDATE", nargs="?")
    parser.add_argument("--recover-landing", metavar="ID", help="reconcile a recorded compatibility landing")
    parser.add_argument("--landing-status", metavar="ID", help="show a recorded compatibility landing")
    parser.add_argument("--expected-tip", help="pin the exact accepted Candidate commit")
    parser.add_argument("--gate", action="append", default=[], help="gate command to run with sh -c; repeatable")
    parser.add_argument("--gate-timeout", type=float, help="positive gate timeout in seconds")
    parser.add_argument("--allow-no-gate", action="store_true", help="explicitly allow landing without a gate command")
    parser.add_argument("--no-push", action="store_true")
    parser.add_argument("--repo", default=".")
    parser.add_argument("--remote", default="origin")
    parser.add_argument("--mainline")
    parser.add_argument(
        "--owner-mainline-override", help="configured owner authorizing an effort ticket to land on its base branch",
    )
    parser.add_argument("--effort", help="effort containing the acceptance record")
    parser.add_argument(
        "--ticket",
        help=(
            "ticket id whose acceptance record must already be committed on the effort record branch; "
            "ticket landings need effort records on a record branch. Run effort.py move-records <effort> first"
        ),
    )


def add_parser(subparsers) -> argparse.ArgumentParser:
    parser = subparsers.add_parser("land", description=__doc__)
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
    except (WorkerError, Refused, StoreError, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return USAGE


if __name__ == "__main__":
    raise SystemExit(main())
