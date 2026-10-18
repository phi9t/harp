# 08: Approve and physically land a verified candidate

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Let the owner approve one exact verified candidate and land it locally through the durable runtime, preserving dirty or advanced destination state and reconciling uncertain Git effects.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Human approval is a distinct recorded decision bound to the candidate and required verification. Worker/lead role credentials cannot manufacture that decision or exercise the owner landing capability.
- [ ] Landing requires current approval, matching evidence and the expected destination base; candidate/input changes invalidate prior approval.
- [ ] A successful landing synchronizes destination branch, index and tracked worktree to the approved revision. Moving a ref alone is not reported as a completed checked-out landing.
- [ ] Dirty, substituted, advanced or otherwise conflicting destination state remains unchanged and is reported with an actionable reason. No reset, stash or overwrite is used to force progress.
- [ ] Concurrent landing requests serialize. Lost acknowledgments and crashes around Git mutation are reconciled using the exact expected/candidate identities before retry.
- [ ] A stale-base proposal needs a new integration candidate, fresh checks and new owner approval. The old approval cannot authorize silent rebasing or a different revision.
- [ ] Real Git/SQLite/process fixtures exercise success, refusal and uncertain recovery through the public CLI and inspect actual destination bytes and identities.

## Blocked by

- [06: Cancel, recover and reconcile interrupted work](06-recover-cancel-reconcile.md)
- [07: Capture and independently verify an exact candidate](07-verify-exact-candidate.md)

## Scope

Reuse useful imported Git contracts under the selected runtime; do not retain the Rust controller or Collab as a second execution authority.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.
