#!/usr/bin/env python3
"""Lead/follower relay for two coding agents sharing one repository.

One agent (the *follower*) owns a working checkout and its long-running
execution environment. Another agent (the *lead*) works only in its own git
worktrees and integrates through a *landing branch* that the follower
fast-forwards into the shared *target branch* at its turn boundaries. Neither
agent edits the other's working tree. See docs/agents/cross-agent-collaboration.md.

Subcommands:

  watch         (lead) poll and print one line per state change: target branch
                moves, follower busy/idle transitions, tracker edits,
                acknowledgement edits, and tracked files deleted (or restored)
                in the follower's working tree.
  rebase        (lead) rebase the landing branch onto the target branch in the
                lead's worktree, stop on conflicts (never auto-resolve), then
                run the verify command.
  sync-tracker  (lead) copy the tracker directory into the follower checkout
                while it is still untracked there, only if the follower copy is
                unchanged since the lead's last sync.
  manifest      (lead) record the follower's superseded dirty paths with their
                content hashes, so `land` can prove it discards nothing else.
  (common)      --agent/--io-root pin every verify command's disk I/O: TMPDIR and
                $VASO_BAZEL_OB point under <io-root>/agents/<agent>/ on a real
                disk (tmpfs/ramfs roots are refused). Use "$VASO_BAZEL_OB" as the
                Bazel --output_base in verify commands.
  land          (follower) verify the working tree holds only manifest-listed
                superseded state, discard it, move the untracked tracker aside,
                `git merge --ff-only` the landing branch, check the tracker
                survived, and run the verify command.

Only the Python standard library and the git CLI are used.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path


class RelayError(Exception):
    """A precondition failed; nothing was changed after the failing check."""


def git(repo: Path, *args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        text=True,
    )
    if check and result.returncode != 0:
        raise RelayError(f"git {' '.join(args)} failed: {result.stderr.strip() or result.stdout.strip()}")
    return result.stdout.strip()


RAM_FILESYSTEMS = {"tmpfs", "ramfs", "devtmpfs"}


def fs_type(path: Path) -> str:
    """Filesystem type of the mount containing path (longest /proc/mounts match)."""
    target = path.resolve()
    while not target.exists():
        target = target.parent
    best, kind = "", "unknown"
    try:
        mounts = Path("/proc/mounts").read_text().splitlines()
    except OSError:
        return kind
    for line in mounts:
        fields = line.split()
        if len(fields) < 3:
            continue
        mount = fields[1].replace("\\040", " ")
        if (str(target) == mount or str(target).startswith(mount.rstrip("/") + "/")) and len(mount) > len(best):
            best, kind = mount, fields[2]
    return kind


def agent_io_env(io_root: Path, agent: str) -> dict[str, str]:
    """Create and return the pinned I/O environment for one agent."""
    kind = fs_type(io_root)
    if kind in RAM_FILESYSTEMS:
        raise RelayError(f"--io-root {io_root} is on {kind} (RAM-backed); use a disk-backed estate root")
    root = io_root / "agents" / agent
    (root / "tmp").mkdir(parents=True, exist_ok=True)
    (root / "bazel-ob").mkdir(parents=True, exist_ok=True)
    return {"TMPDIR": str(root / "tmp"), "VASO_BAZEL_OB": str(root / "bazel-ob"), "VASO_AGENT_IO_ROOT": str(root)}


VERIFY_DIRTIED_TREE = 5


def _tree_state(repo: Path) -> list[tuple[str, str]]:
    return sorted(dirty_entries(repo))


def run_verify(command: str, cwd: Path, args: argparse.Namespace) -> int:
    """Run the verify command; it must not write into the source tree.

    A verify is read-only with respect to the checkout: build outputs belong
    under the pinned I/O root. A verify that adds or changes files in the
    tree (for example a Bazel run whose --output_base expanded to "") fails
    with VERIFY_DIRTIED_TREE, whatever its exit code.
    """
    env = dict(os.environ)
    if getattr(args, "agent", None):
        if not args.io_root:
            raise RelayError("--agent needs --io-root (a disk-backed estate root, e.g. $VASO_ESTATE_ROOT)")
        env.update(agent_io_env(args.io_root, args.agent))
    before = _tree_state(cwd)
    rc = subprocess.run(command, shell=True, cwd=cwd, env=env).returncode
    after = _tree_state(cwd)
    if after != before:
        changed = sorted(set(after) - set(before))
        print(f"VERIFY_DIRTIED_TREE: the verify wrote into {cwd}; build outputs belong under the I/O root:", flush=True)
        for status, path in changed[:20]:
            print(f"  {status} {path}", flush=True)
        if len(changed) > 20:
            print(f"  … {len(changed) - 20} more", flush=True)
        return VERIFY_DIRTIED_TREE
    return rc


def is_ancestor(repo: Path, ancestor: str, descendant: str) -> bool:
    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", ancestor, descendant],
        capture_output=True,
    ).returncode == 0


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_digest(root: Path) -> dict[str, str]:
    """Relative path -> sha256 for every file under root ({} if absent)."""
    if not root.is_dir():
        return {}
    return {
        path.relative_to(root).as_posix(): file_sha256(path)
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


def dirty_entries(repo: Path) -> list[tuple[str, str]]:
    """(status, path) for every modified/untracked entry, untracked dirs expanded."""
    # -z output is unquoted and NUL-separated, and is not stripped: the status
    # column's leading space is significant (" M" vs "M ").
    result = subprocess.run(
        ["git", "-C", str(repo), "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RelayError(f"git status failed: {result.stderr.strip()}")
    records = result.stdout.split("\0")
    entries = []
    index = 0
    while index < len(records) and records[index]:
        record = records[index]
        status, path = record[:2], record[3:]
        entries.append((status, path))
        # A rename/copy record is followed by its source path.
        index += 2 if status[0] in "RC" else 1
    return entries


# --------------------------------------------------------------------------- watch


@dataclass(frozen=True)
class WatchState:
    tip: str
    busy: str  # "busy" | "idle" | "gone" | "unknown"
    tracker: str
    ack_lines: int
    deleted: tuple[str, ...] = ()


def _busy_state(busy_cmd: str | None, follower_pid: int | None) -> str:
    if follower_pid is not None:
        try:
            Path(f"/proc/{follower_pid}").stat()
        except FileNotFoundError:
            return "gone"
    if not busy_cmd:
        return "unknown"
    return "busy" if subprocess.run(busy_cmd, shell=True).returncode == 0 else "idle"


def _ack_lines(handoff: Path | None, heading: str) -> int:
    if handoff is None or not handoff.is_file():
        return -1
    text = handoff.read_text(encoding="utf-8", errors="replace")
    _, sep, tail = text.partition(heading)
    return sum(1 for line in tail.splitlines() if line.strip()) if sep else -1


def tracked_deletions(repo: Path) -> tuple[str, ...]:
    """Tracked files missing from the working tree or staged for deletion."""
    return tuple(sorted(path for status, path in dirty_entries(repo) if "D" in status))


def read_watch_state(args: argparse.Namespace) -> WatchState:
    tracker_digest = json.dumps(tree_digest(args.follower / args.tracker), sort_keys=True)
    return WatchState(
        tip=git(args.follower, "rev-parse", "--short", args.target, check=False) or "?",
        busy=_busy_state(args.busy_cmd, args.follower_pid),
        tracker=hashlib.sha256(tracker_digest.encode()).hexdigest()[:12],
        ack_lines=_ack_lines(args.handoff, args.ack_heading),
        deleted=tracked_deletions(args.follower),
    )


def diff_watch_states(old: WatchState, new: WatchState, subject: str = "") -> list[str]:
    events = []
    if new.tip != old.tip:
        events.append(f"TARGET_MOVED {old.tip} -> {new.tip}{': ' + subject if subject else ''}")
    if new.busy != old.busy:
        events.append(f"FOLLOWER {old.busy} -> {new.busy}")
    if new.tracker != old.tracker:
        events.append("TRACKER_CHANGED in follower checkout")
    if new.ack_lines != old.ack_lines:
        events.append(f"ACK_CHANGED lines={new.ack_lines}")
    for path in sorted(set(new.deleted) - set(old.deleted)):
        events.append(f"FOLLOWER_DELETED_TRACKED {path}")
    for path in sorted(set(old.deleted) - set(new.deleted)):
        events.append(f"FOLLOWER_RESTORED_TRACKED {path}")
    return events


IO_TMP_PATTERNS = ("vaso", "tmp", "pytest-of-", "bazel")
IO_THRESHOLD = 0.8


def _statvfs_used(path: Path, inodes: bool) -> float:
    st = os.statvfs(path)
    total, free = (st.f_files, st.f_ffree) if inodes else (st.f_blocks, st.f_bfree)
    return 0.0 if not total else 1 - free / total


def io_health(paths, inode_used=None, space_used=None) -> dict:
    inode_used = inode_used or (lambda p: _statvfs_used(p, True))
    space_used = space_used or (lambda p: _statvfs_used(p, False))
    user = os.getuid()
    state = {}
    for path in paths:
        entries = set()
        try:
            for child in Path(path).iterdir():
                try:
                    if child.lstat().st_uid == user and child.name.startswith(IO_TMP_PATTERNS):
                        entries.add(str(child))
                except OSError:
                    continue
        except OSError:
            pass
        state[str(path)] = {"inodes": inode_used(path), "space": space_used(path), "entries": entries}
    return state


def diff_io_health(old: dict, new: dict) -> list[str]:
    events = []
    for path, now in new.items():
        before = old.get(path, {"inodes": 0.0, "space": 0.0, "entries": set()})
        for key, label in (("inodes", "IO_INODES_HIGH"), ("space", "IO_SPACE_HIGH")):
            if now[key] >= IO_THRESHOLD and (before[key] < IO_THRESHOLD or int(now[key] * 100) // 5 != int(before[key] * 100) // 5):
                events.append(f"{label} {path} {int(now[key] * 100)}%")
        for entry in sorted(now["entries"] - before["entries"]):
            events.append(f"IO_NEW_TMP_ENTRY {entry}")
    return events


def cmd_watch(args: argparse.Namespace) -> int:
    state = read_watch_state(args)
    io_state = io_health(args.io_health) if args.io_health else {}
    print(f"WATCHING target={args.target} tip={state.tip} follower={state.busy}", flush=True)
    for path, now in io_state.items():
        print(f"IO_BASELINE {path} inodes={int(now['inodes'] * 100)}% space={int(now['space'] * 100)}% "
              f"user_entries={len(now['entries'])}", flush=True)
    for path in state.deleted:
        print(f"FOLLOWER_DELETED_TRACKED {path} (already deleted when the watch started)", flush=True)
    deadline = time.monotonic() + args.max_seconds if args.max_seconds else None
    while deadline is None or time.monotonic() < deadline:
        time.sleep(args.interval)
        new = read_watch_state(args)
        subject = git(args.follower, "log", "-1", "--format=%s", args.target, check=False) if new.tip != state.tip else ""
        events = diff_watch_states(state, new, subject)
        if args.io_health:
            new_io = io_health(args.io_health)
            events += diff_io_health(io_state, new_io)
            io_state = new_io
        for event in events:
            print(f"{event} [{time.strftime('%H:%M:%S', time.gmtime())}Z]", flush=True)
        state = new
        if state.busy == "gone":
            print("FOLLOWER_EXITED", flush=True)
            return 0
    return 0


# -------------------------------------------------------------------------- rebase


def cmd_rebase(args: argparse.Namespace) -> int:
    """Rebase a detached copy of the landing branch, verify it, then move the ref.

    The follower may fast-forward to the landing branch at any moment, so the
    branch ref must only ever point at a verified commit. The rebase and the
    verify run on a detached HEAD; the branch moves only when both succeed.
    """
    repo = args.lead_worktree
    if git(repo, "status", "--porcelain"):
        raise RelayError(f"{repo} has uncommitted changes; commit them before rebasing")
    branch = git(repo, "rev-parse", "--abbrev-ref", "HEAD")
    if branch == "HEAD":
        raise RelayError(f"{repo} is on a detached HEAD; check out the landing branch first")
    target = git(repo, "rev-parse", args.target)
    before = git(repo, "rev-parse", "HEAD")
    if is_ancestor(repo, target, "HEAD"):
        print(f"UP_TO_DATE {branch} already contains {args.target}", flush=True)
    else:
        git(repo, "checkout", "-q", "--detach")
        result = subprocess.run(["git", "-C", str(repo), "rebase", target], capture_output=True, text=True)
        if result.returncode != 0:
            conflicts = git(repo, "diff", "--name-only", "--diff-filter=U", check=False)
            print(
                f"REBASE_CONFLICT on a detached HEAD; {branch} is unchanged. Resolve, "
                f"`git rebase --continue`, rerun verify, then `git checkout -B {branch}`:",
                flush=True,
            )
            for path in conflicts.splitlines():
                print(f"  {path}", flush=True)
            return 3
        print(f"REBASED (detached) onto {args.target} -> {git(repo, 'rev-parse', '--short', 'HEAD')}", flush=True)
    if args.verify:
        verdict = run_verify(args.verify, repo, args)
        if verdict != 0:
            candidate = git(repo, "rev-parse", "--short", "HEAD")
            git(repo, "checkout", "-q", branch)
            print(
                f"VERIFY_FAILED rc={verdict} on {candidate}; landing branch unchanged at "
                f"{git(repo, 'rev-parse', '--short', branch)} (candidate kept as {candidate})",
                flush=True,
            )
            return verdict if verdict == VERIFY_DIRTIED_TREE else 4
        print("VERIFY_PASSED", flush=True)
    if git(repo, "rev-parse", "HEAD") != before or git(repo, "rev-parse", "--abbrev-ref", "HEAD") == "HEAD":
        git(repo, "checkout", "-q", "-B", branch)
        print(f"MOVED {branch} -> {git(repo, 'rev-parse', '--short', branch)}", flush=True)
    return 0


# -------------------------------------------------------------------- sync-tracker


def cmd_sync_tracker(args: argparse.Namespace) -> int:
    rel = Path(args.tracker)
    source = args.lead_worktree / rel
    dest = args.follower / rel
    state_file: Path = args.state
    if git(args.follower, "ls-files", "--", str(rel)):
        print(f"TRACKED {rel} is tracked in the follower checkout; it travels by landing now, not by sync")
        return 0
    current = tree_digest(dest)
    recorded = json.loads(state_file.read_text()) if state_file.is_file() else None
    if recorded is not None and current != recorded:
        missing = "missing (follower may be mid-landing)" if not current else "edited by someone else"
        raise RelayError(f"follower copy of {rel} is {missing}; not overwriting. Diff and merge by hand.")
    if recorded is None and current:
        raise RelayError(f"no recorded state for {rel} but the follower has a copy; refusing first overwrite")
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(source, dest)
    state_file.parent.mkdir(parents=True, exist_ok=True)
    state_file.write_text(json.dumps(tree_digest(dest), sort_keys=True, indent=1))
    print(f"SYNCED {rel} ({len(tree_digest(dest))} files)", flush=True)
    return 0


# ------------------------------------------------------------------------ manifest


def cmd_manifest(args: argparse.Namespace) -> int:
    entries = []
    for status, path in dirty_entries(args.follower):
        if path.startswith(args.tracker.rstrip("/") + "/") or path in args.ignore:
            continue
        if not any(path == p or path.startswith(p.rstrip("/") + "/") for p in args.superseded):
            continue
        full = args.follower / path
        entries.append({"path": path, "status": status, "sha256": file_sha256(full) if full.is_file() else None})
    missing = [p for p in args.superseded if not any(e["path"] == p or e["path"].startswith(p.rstrip("/") + "/") for e in entries)]
    if missing:
        raise RelayError(f"superseded paths are not dirty in the follower checkout: {', '.join(missing)}")
    manifest = {"landing_branch": args.landing, "superseded": entries, "ignore": sorted(args.ignore)}
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"MANIFEST {args.out}: {len(entries)} superseded entries", flush=True)
    return 0


# ---------------------------------------------------------------------------- land


def cmd_land(args: argparse.Namespace) -> int:
    repo: Path = args.follower
    manifest = json.loads(args.manifest.read_text()) if args.manifest else {"superseded": [], "ignore": []}
    landing = args.landing or manifest.get("landing_branch")
    if not landing:
        raise RelayError("no landing branch given (use --landing or a manifest)")
    tracker_rel = args.tracker.rstrip("/") if args.tracker else None
    superseded = {entry["path"]: entry for entry in manifest.get("superseded", [])}
    ignore = set(manifest.get("ignore", [])) | set(args.ignore)

    # 1. Every dirty tracked entry must be superseded (with matching content);
    #    commit your own work first. Untracked files are left alone unless the
    #    landing branch would create the same path (or they are superseded, or
    #    the untracked tracker, which is moved aside and re-checked).
    landing_paths = set(git(repo, "ls-tree", "-r", "--name-only", landing).splitlines())
    unexpected = []
    for status, path in dirty_entries(repo):
        if path in ignore or any(path.startswith(i.rstrip("/") + "/") for i in ignore):
            continue
        if tracker_rel and status == "??" and path.startswith(tracker_rel + "/"):
            continue
        entry = superseded.get(path)
        if entry is None:
            if status == "??" and path not in landing_paths:
                continue
            reason = " (the landing branch creates this path)" if status == "??" else ""
            unexpected.append(f"{status} {path}{reason}")
            continue
        full = repo / path
        actual = file_sha256(full) if full.is_file() else None
        if actual != entry["sha256"]:
            unexpected.append(f"{status} {path} (content changed since the manifest was recorded)")
    if unexpected:
        raise RelayError(
            "working tree has changes the landing would discard or that block it; commit or ask the lead:\n  "
            + "\n  ".join(unexpected)
        )
    if not is_ancestor(repo, "HEAD", landing):
        raise RelayError(f"{landing} does not contain HEAD; ask the lead to rebase it (never merge non-ff)")
    if args.dry_run:
        print(f"DRY_RUN ok: would discard {len(superseded)} superseded entries and fast-forward to {landing}", flush=True)
        return 0

    # 2. Discard superseded state (already proven identical to the manifest).
    for path, entry in superseded.items():
        if entry["status"] == "??":
            (repo / path).unlink(missing_ok=True)
        else:
            git(repo, "restore", "--source=HEAD", "--staged", "--worktree", "--", path)

    # 3. Move the untracked tracker aside so the fast-forward can create it.
    premerge = None
    if tracker_rel and (repo / tracker_rel).is_dir() and not git(repo, "ls-files", "--", tracker_rel):
        premerge = args.premerge_dir
        if premerge.exists():
            raise RelayError(f"{premerge} already exists; remove it after checking it")
        shutil.move(str(repo / tracker_rel), str(premerge))

    # 4. Fast-forward only.
    before = git(repo, "rev-parse", "--short", "HEAD")
    git(repo, "merge", "--ff-only", landing)
    after = git(repo, "rev-parse", "--short", "HEAD")
    print(f"LANDED {before} -> {after}", flush=True)

    # 5. Tracker carry-over check.
    if premerge is not None:
        if tree_digest(premerge) == tree_digest(repo / tracker_rel):
            shutil.rmtree(premerge)
            print("TRACKER identical after landing; pre-merge copy removed", flush=True)
        else:
            print(f"TRACKER_DIFFERS: re-apply your edits from {premerge} (diff -r), then remove it", flush=True)

    # 6. Verify, with disk I/O pinned under the agent's I/O root.
    if args.verify:
        verdict = run_verify(args.verify, repo, args)
        print("VERIFY_PASSED" if verdict == 0 else f"VERIFY_FAILED rc={verdict}", flush=True)
        if verdict == VERIFY_DIRTIED_TREE:
            return verdict
        return 0 if verdict == 0 else 4
    return 0


# -------------------------------------------------------------------------- main


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)

    watch = sub.add_parser("watch", help="print one line per follower/branch state change")
    watch.add_argument("--follower", type=Path, required=True, help="follower checkout")
    watch.add_argument("--target", required=True, help="shared target branch")
    watch.add_argument("--tracker", required=True, help="tracker dir, relative to the repo root")
    watch.add_argument("--handoff", type=Path, help="handoff file whose acknowledgement section is watched")
    watch.add_argument("--ack-heading", default="## Acknowledgement")
    watch.add_argument("--follower-pid", type=int, help="follower process; its exit ends the watch")
    watch.add_argument("--busy-cmd", help="shell command exiting 0 while the follower is mid-turn")
    watch.add_argument("--interval", type=float, default=20.0)
    watch.add_argument("--max-seconds", type=float, default=0, help="0 = until the follower exits")
    watch.add_argument("--io-health", type=Path, nargs="*", default=[],
                       help="shared scratch dirs to watch (e.g. /tmp /var/tmp): usage >= 80%% and new "
                            "vaso/tmp/pytest/bazel entries owned by you")
    watch.set_defaults(func=cmd_watch)

    rebase = sub.add_parser("rebase", help="rebase the landing branch onto the target, then verify")
    rebase.add_argument("--lead-worktree", type=Path, required=True)
    rebase.add_argument("--target", required=True)
    rebase.add_argument("--verify", help="shell command run in the lead worktree after rebasing")
    rebase.add_argument("--agent", help="agent name for pinned verify I/O (see --io-root)")
    rebase.add_argument("--io-root", type=Path, help="disk-backed estate root for pinned verify I/O")
    rebase.set_defaults(func=cmd_rebase)

    sync = sub.add_parser("sync-tracker", help="guarded copy of the tracker into the follower checkout")
    sync.add_argument("--lead-worktree", type=Path, required=True)
    sync.add_argument("--follower", type=Path, required=True)
    sync.add_argument("--tracker", required=True)
    sync.add_argument("--state", type=Path, required=True, help="lead-private file recording the last sync")
    sync.set_defaults(func=cmd_sync_tracker)

    manifest = sub.add_parser("manifest", help="record superseded follower state for a safe landing")
    manifest.add_argument("--follower", type=Path, required=True)
    manifest.add_argument("--landing", required=True)
    manifest.add_argument("--tracker", required=True)
    manifest.add_argument("--superseded", nargs="+", required=True, help="paths (files or dir prefixes)")
    manifest.add_argument("--ignore", nargs="*", default=[], help="untracked paths landing leaves alone")
    manifest.add_argument("--out", type=Path, required=True)
    manifest.set_defaults(func=cmd_manifest)

    land = sub.add_parser("land", help="(follower) fast-forward the landing branch safely")
    land.add_argument("--follower", type=Path, default=Path("."))
    land.add_argument("--landing", help="landing branch (default: from the manifest)")
    land.add_argument("--manifest", type=Path, help="lead-recorded superseded state")
    land.add_argument("--tracker", help="tracker dir, relative to the repo root")
    land.add_argument("--ignore", nargs="*", default=[], help="additional untracked paths to leave alone")
    land.add_argument("--premerge-dir", type=Path, default=Path.home() / ".relay-tracker-premerge")
    land.add_argument("--verify", help="shell command run in the follower checkout after landing")
    land.add_argument("--agent", help="agent name for pinned verify I/O (see --io-root)")
    land.add_argument("--io-root", type=Path, help="disk-backed estate root for pinned verify I/O")
    land.add_argument("--dry-run", action="store_true")
    land.set_defaults(func=cmd_land)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    for name in ("follower", "lead_worktree"):
        if getattr(args, name, None) is not None:
            setattr(args, name, getattr(args, name).resolve())
    try:
        return args.func(args)
    except RelayError as exc:
        print(f"REFUSED: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
