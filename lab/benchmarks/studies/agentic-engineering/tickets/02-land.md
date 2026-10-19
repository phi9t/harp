# 02 land: landing with gates (F6)

Type: feature
Status: resolved
Owner: lead
Blocked by:
Claimed:

## Description
Build `tools/agents/land.py` (contract: `milestone-b/contracts/wt-dispatch.md`). Landing by hand moved branch refs without a gate and risks contaminating other checkouts. The fix recorded for `milano.scm integrate` is the required order: rebase, gate the **exact tip**, push **first**, and only then move local mainline.

## Interface
`land.py BRANCH [--gate CMD]... [--no-push] [--repo PATH] [--remote origin] [--mainline main] [--attempts 3]`; also plugs into `wt.py land` (`add_parser`, `run`).
Algorithm, under a lock in the repository's common git dir (a second concurrent land waits or fails clearly):
1. Refuse (exit 3, list the paths) if the main checkout has tracked changes overlapping `git diff --name-only mainline...BRANCH`.
2. Fetch the remote if there is one. Rebase BRANCH onto the remote mainline (or local mainline when there is no remote).
3. Refuse (exit 4) if the rebased range contains merge commits (linear history only).
4. Check the exact tip out in a clean temporary checkout and run every gate there (cwd = that checkout, env `LAND_TIP=<sha>`). Any gate failure exits 5 and leaves local mainline and the remote untouched.
5. If pushing: `git push <remote> <tip>:<mainline>` **before** touching local mainline. A rejected (non-fast-forward) push means a lost race: fetch and go back to step 2 (gates re-run on the new tip), up to `--attempts` times, then exit 6.
6. Only after the push succeeds (or with `--no-push` / no remote): fast-forward local mainline to the tip (work with mainline checked out in the main worktree, or not checked out anywhere), print the landed sha on stdout, exit 0.
Never leave the main checkout on a different branch, never leave a stranded local commit that the remote does not have.

## Acceptance
- [x] Test: a failing gate leaves local mainline and the remote exactly as they were.
- [x] Test: lost race. A gate command pushes a competing commit to the bare remote; land rebases again, re-runs the gates on the new tip, and local mainline ends at the pushed tip with no stranded commit.
- [x] Test: overlap with a tracked change in the main checkout is refused with exit 3 and the changed paths; untracked files do not block.
- [x] Test: a branch containing a merge commit is refused with exit 4; the landed history stays linear.
- [x] Test: push-first ordering, proven by a remote pre-receive hook that rejects the first push: local mainline must not have moved before the retry succeeds.
- [x] Test: no remote or --no-push fast-forwards local mainline only; the landed sha is printed.
- [x] Tests cover the standalone CLI and the add_parser/run plug-in interface.
- [x] tools/agents/checks.d/20-land.sh runs the suite and exits non-zero on failure.

## Comments
- 2026-09-30T08:24:30Z lead: claimed until 2026-10-01T08:24:30Z
- 2026-09-30T08:24:37Z lead: resolved
