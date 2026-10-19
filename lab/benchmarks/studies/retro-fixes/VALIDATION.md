# Retrospective repairs: validation

Worktree: `/data02/home/philip.yang/workspace/corenius-retro-fixes`.
Branch: `codex/retro-fixes`.
Base: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`. No new commits.

## Delivered behavior

- Complete WIP snapshots include committed, staged, unstaged and untracked
  content, including binary files, without changing the source index or refs.
  The shared code-review skill now uses this artifact for both review axes.
- The gate supports suite selection, time limits, process-group cleanup and
  JSON results. Private short temporary roots prevent Unix socket overflow.
  Escaped sessions remain outside process-group cleanup; use the durable
  cgroup backend for workloads that daemonize.
- Hosted PR CI invokes the existing gate. A separate manual workflow qualifies
  real delegated cgroups and production SQLite admission without skipped tests.
- Effort snapshot records local dirty state without committing. Ticket handoff
  records implementation awaiting acceptance without resolving dependencies
  or granting landing permission.

## Native durable qualification

[qualification.json](evidence/qualification.json) records **73 passed, zero
skips**, in 22.308 seconds. This run includes process-group recovery, real
Insula boundaries and delegated cgroup cleanup of detached descendants. SQLite
3.51.3 passed the production admission check without an override.

The qualifier ran in an owned delegated user-systemd scope. A live cancellation
probe returned 143, marked its report incomplete and left its exact unit unloaded.
[Provenance](evidence/qualification-provenance.md) records the private SQLite
build, verified official source checksum and command. No system SQLite was changed.

Both backends passed their intended tests. Process groups are suitable for the
restricted child-tree contract; cgroups provide kernel cleanup for descendants
that detach. Passing cleanup tests does not establish exactly-once external
side effects; manual-effect uncertainty still requires explicit resolution.

## Repository gate

The initial default-suite run used an explicit 900-second per-suite budget.
[gate.json](evidence/gate.json) records 25 passed entries and one controller
suite timeout after 900 seconds; all selected entries ran. Some default entries
are opt-in dispatchers and can return success without running their optional
service tests. This report is retained as failed, not rewritten as green.

The isolated [controller retry](evidence/controller-retry.json) passed all 138
tests with zero failures in 582.647 seconds. Every default
gate entry has a passing result across the initial run and this retry; this is
not a claim that the initial full run passed.

The final default budget is 1800 seconds. Controller tests retain the two-thread
local default and permit `CHECK_CONTROLLER_THREADS`; hosted CI selects four.
The isolated controller retry selects eight. Root Cargo tests remain single-threaded.

## Focused checks and review

- [Final focused gate](evidence/final-focused.json): 15 WIP snapshot tests and
  six qualifier contract tests passed after all respective repairs.
- Final gate dispatcher tests: 10 passed; process cleanup runner tests: four passed.
- Temporary-root tests: 13 passed. Effort tests: 22 passed. Tracker tests:
  32 passed. Acceptance tests: 18 passed. Lead-launch tests: 48 passed.
- Lead-driven harness: 41 passed, including tmux and socket-path behavior.
- Policy sync tests: 24 passed. Workflow actionlint 1.7.7 passed.
- [Independent reviews](evidence/reviews.md): Standards, Spec and focused defect
  reviews cleared all findings after repair. Review artifact 4 captures the final
  code tree `d088f4961c3c9347622b204f51e8b29c6650a1fd` outside the source worktree.
  Later effort evidence and handoff metadata do not change reviewed code.

## Limits and handoff

The Git repository has no configured origin. Workflow files were validated
locally; hosted execution, the trusted runner group and environment protections
have not been activated or verified remotely. See the setup requirements in
[the validation guide](../../../tools/agents/VALIDATION.md).

Tickets 01–04 are handed off as implemented-awaiting-acceptance with hashed
validation evidence. Their status remains claimed; dependencies remain blocked
until the existing accepted-and-landed rules are met. STATE records a
noncommitting snapshot of the dirty worktrees and verification evidence.

The main checkout and original durable worktree are unchanged. All carried-forward
source hashes still match [their provenance](evidence/carried-forward.json).
The implementation remains uncommitted and local-only; no commit, push, PR or
landing was performed. Run-log hashes are retained in [logs.json](evidence/logs.json).
