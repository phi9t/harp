# 12 salvage-prune-mainline: Salvage pending work and prune Corenius branches/worktrees to a pristine mainline

Type: task
Status: claimed
Owner: codex-salvage-prune
Blocked by: 
Claimed: 2026-10-07T09:14:21Z expires 2026-10-08T09:14:21Z

Triage: needs-triage

## Acceptance
- [ ] Identify and land all currently landable work with independent reviews and required gates.
- [ ] Preserve every remaining unique commit, pending file, useful ignored artifact and recovery mapping before retirement.
- [ ] Retire recovered inactive worktrees and prune eligible branches without losing live-owner work.
- [ ] Preserve mainline private/untracked artifacts outside the checkout and verify pristine main status.
- [ ] Audit current refs/worktrees and recovery coverage against the complete objective.

## Scope and authority

User explicitly requests all landable work, fixes for encountered issues, a pristine
mainline and salvage/pruning of all branches/worktrees. This extends Ticket11’s
reviewed queue into repository cleanup. Gated path: broad destructive-data scope;
preservation and exact-target verification must precede retirement. User’s explicit
pruning instruction authorizes recovered inactive checkout/ref retirement. Never
force-delete unresolved work or mutate live-owner trees. Keep main and policy-required
ultron/mainline; record any other unavoidable active ownership separately.

## Progress

See ../evidence/salvage-prune-20261007-progress.json. Fresh initial inventory124
worktrees/134 branches; all history bundled and verified. Four clean, merged,
artifact-free inactive trees retired, with three non-force merged branch deletions.
120 remaining trees received stable pending-file snapshots (5,498 entries).
Ignored scratch and live-owner work remain preserved in place, requiring further
salvage and retirement. Main currently has its transcript and nested worktree
untracked; pristine completion is not yet proved.

## Comments
- 2026-10-07T09:14:21Z codex-salvage-prune: claimed until 2026-10-08T09:14:21Z
