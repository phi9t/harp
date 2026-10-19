# 01 wt-core: worktree lifecycle (F5)

Type: feature
Status: resolved
Owner: lead
Blocked by:
Claimed:

## Description
Build `tools/agents/wt.py` with `new`, `status`, `finish`, `gc` (contract: `milestone-b/contracts/wt-dispatch.md`; you also write the `_load_plugins` hook that loads `land`).
One writer per tree is a rule; nothing creates, lists or removes agent worktrees consistently, and trial branches (`claude/*`, `worker/*`, `traecli/*`) accumulated by hand.

## Interface
- `wt.py new --agent A --topic T [--base B] [--repo R]`: creates branch `A/T` from B (default: mainline) and a sibling worktree directory `<repo>-<A>-<T>`; records owner metadata
  (agent, topic, base, created time) in a file in that worktree's git dir; prints JSON with path, branch, base sha. Refuses an existing branch or directory.
- `wt.py status [--json] [--repo R]`: one row per worktree of the repo: path, branch, ahead/behind versus mainline, dirty file count, owner (metadata or branch prefix), age.
- `wt.py finish BRANCH [--repo R]`: only when BRANCH is merged into mainline (ancestor) and the worktree is clean; removes the worktree and deletes the branch. **Must work when the current directory is inside the worktree being removed** (change directory to the main worktree before removing; the known failure being fixed is that removing the current directory's worktree fails or leaves the shell in a deleted directory).
- `wt.py gc [--apply] [--days N]`: lists worktrees that are merged, or abandoned (clean, no commits ahead of mainline, older than N days, default 7); removes them only with `--apply`.
- Usage errors exit 64; refusals exit 1 with a one-line reason.

## Acceptance
- [x] Tests on throwaway repos cover new (branch, directory, metadata, refusal of existing branch and directory).
- [x] Tests cover status: ahead/behind, dirty count, owner, age, JSON shape.
- [x] Test covers finish from inside the worktree it removes (run the command with that worktree as the working directory of a subprocess).
- [x] Tests cover finish refusing an unmerged branch and a dirty worktree.
- [x] Tests cover gc as a dry run by default and --apply removing only merged or abandoned trees, never a dirty one or one with commits ahead.
- [x] Tests cover the plugin hook: a missing land module is ignored, a land module with a broken import raises, a fake land module with add_parser/run registers a land subcommand.
- [x] tools/agents/checks.d/10-wt.sh runs the suite and exits non-zero on failure.

## Comments
- 2026-09-30T08:24:30Z lead: claimed until 2026-10-01T08:24:30Z
- 2026-09-30T08:24:37Z lead: resolved
