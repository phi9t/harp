# Legacy gate repair: independent review iteration 1

Read and repair base: `d4273dce28cd9754342dba0c1df5a46996a08d84`.
Original spec base: `c0a2fc0370f18ff28009dbe9b62a59916de0e582`.
Scope: round-zero Standards and Spec P1 findings in the legacy gate receipt.
Status: implementation verified; independent re-review and root acceptance remain pending.

The legacy gate treated porcelain status plus HEAD equality as proof of clean tracked bytes.
An assume-unchanged or skip-worktree flag could hide a changed tracked file from porcelain status.
The gate then checked those changed bytes while landing the original Candidate.

## Red evidence

The public `land.py` regressions use real disposable Git repositories and a configured override owner.
Both index-flag variants returned 0 against the frozen repair-base implementation, where the tests require 5.
The tests also require an invalid-subject receipt, retained changed bytes and an unchanged mainline.

Mutation between the gate and recovery was reproduced separately against the frozen repair-base journal validator.
Both a late hidden index flag and a replacement checkout recovered with exit 0, where the tests require 64.
The new gate runner was retained during this comparison so that the tests had bound, live workspaces to mutate.
The baseline file substitutions occurred only in this isolated repair worktree and were restored immediately.

## Change

Version-2 legacy receipts bind the check path, worktree Git directory and exact Candidate.
They record registration identity, HEAD equality, index flags, clean status and process-group quiescence.
Hidden flags, a changed workspace identity, an external worktree lock or a live recorded descendant refuse acceptance.
The journal validates those facts again from the live workspace before landing or recovery.
Receipts without the new affirmative facts refuse instead of inheriting an unsafe default.

Journal-bound gate workspaces remain available through the locked landing or recovery check.
Failed or invalid gate artifacts remain available for inspection.
Successful workspaces are removed only after fresh identity, cleanliness, lock and liveness checks.
Cleanup does not unlock worktrees or recursively remove a path after Git refuses removal.

The new `before_legacy_local` process-death failpoint makes the replay boundary testable before any local ref moves.
The public shim also exports `temporary_worktree` explicitly for the retained `runpy.run_path` namespace contract.
That wrapper remains lazy and retains the forbidden-TMPDIR check before Git execution.

## Green evidence

- Six new public CLI regressions pass: both flag variants, late flags, replacement identity,
  preservation of an external lock and refusal while a recorded gate descendant remains alive.
- All 31 historical landing tests pass unchanged.
- All 15 existing legacy integration tests pass unchanged.
- All seven `tools/test_temp_root.py` tests pass unchanged.
- `py_compile` and `git diff --check` pass; modified Python lines remain within 120 columns.

The broad repository gate belongs to the root integration run and was not repeated here.
No synchronized policy, ticket record, observed checkout, real remote or shared branch was modified.
This is repair iteration 1 of the three-iteration review limit; passing tests do not replace independent acceptance.
