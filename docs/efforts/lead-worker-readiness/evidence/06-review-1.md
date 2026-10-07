# Lead review 1 of ticket 06: one change now; wait for my word to re-integrate

Reviewed Candidate: 4d08d2d. The three overlap cases, the read-only `overlap` command with the
behind-the-tip column, the two-worker scenario with a stale redo, the refusal of a protected path that
contains the repository or the run storage, and lock-free probes are right and stay.

## 1. The "unaccounted change" finding does not know how real workers edit files (blocking)

`unaccounted_workspace_findings` raises a VIOLATION for an allowed dirty path unless some recorded shell
command names it. Real runtimes edit files with an edit tool, not a shell command. I ran one real Codex
worker through your launcher and supervisor (codex-cli 0.160.0, a task that creates one file and commits
it). The stream contains, besides the command events you already handle, one item of this shape for the
edit (path shortened by me, everything else verbatim):

    {"type": "item.completed", "item": {"id": "item_N", "type": "file_change", "changes": [{"path": "<worktree>/result.txt", "kind": "add"}], "status": "completed"}}

and an `item.started` twin before it. In that run the file was later named by a `git add result.txt`
command, so your finding stayed quiet by luck; a worker that runs `git add -A`, or a script that
generates files, would be reported as VIOLATED for doing its job.

Required:
- The runtime adapter (ticket 12's, in `worker.py`) also yields the paths of `file_change` items; a dirty
  path named by a file-change event is accounted for, exactly like one named by a command.
- A dirty allowed path that no command and no file-change event accounts for is a NOTE while the worker
  is running (a command can write files it does not name) and becomes a VIOLATION only in the case you
  built for the lead's rule: it appears after the Candidate was pinned.
- Tests with a fake runtime emitting the real shape above: an edit through `file_change` then
  `git add -A` is not a finding; a path changed after the pinned Candidate with no event still is.

## 2. Something for your report, not for you to fix

The same real run ended not ready, and correctly so: the worker's `git add` failed with
`fatal: Unable to create '<main repo>/.git/worktrees/<name>/index.lock': Read-only file system`. Under the
default `workspace-write` sandbox a worker in a git worktree cannot commit at all, because the worktree's
git metadata lives in the main repository's `.git`, outside the writable root. Every test so far used fake
binaries, so nobody saw it. Add under `## For later tickets`:
`- to 14: under the workspace-write sandbox a worker in a git worktree cannot commit (index.lock under the main repository's .git is read-only); found by a real Codex run on 2026-10-05. The live trial cannot use sandboxed worktree workers until a ticket fixes this.`
Do not try to fix it in this ticket.

## Protocol

New commits on top of 4d08d2d; no rewriting. Update the report. When item 1 is done and your gate passes,
commit and then STOP without the marker commit: ticket 16 lands before you and changes
`tools/agents/lead_launch.py` and the harness module. Wait for my message telling you to re-integrate; the
`ticket 06: ready for review` commit comes after that rebase.
