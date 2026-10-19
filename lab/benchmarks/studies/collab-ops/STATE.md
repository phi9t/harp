# Collab Ops state

## Where we are

The 18-ticket implementation and follow-up draft are committed and landed locally
on mainline by fast-forward at snapshot 3b7b0b0. The user explicitly authorized
this snapshot landing and follow-up drafting on 2026-10-12. See the durable
`evidence/mainline-snapshot-landing.json` receipt. Final record commits can follow
that snapshot without changing its product source. Mainline
already contained the implementation through repair 2 plus two cleanup fixes;
those fixes were merged into the completion branch without conflicts.

The full default repository gate passed. Fresh affected checks passed at frozen
source `3c812cf`, including 169 Collab tests. Standards and Spec independently
passed; the independent branch defect gate still reports four blockers. The
read-only architecture review reproduced all four in nine CLI variants. See
`evidence/final-validation.json`, `evidence/review-round3.json`,
`evidence/branch-review-round3.json` and `evidence/architecture-review.md`.

The combined source `365876ed` ran 170 Collab tests with four archive/sweep
failures: the inherited mainline tar extraction filter is unsupported on this
host Python. See `evidence/mainline-combined-validation.json` and ticket 25.

This is an authorized mainline snapshot, not accepted completion. No original
pending tickets were resolved to hide the review result. Follow-up design and
implementation budget are not approved by the snapshot request.

## Next actions

Review `FOLLOWUP.md` and draft tickets 19–25. Settle the shared mutation interface,
all-managed-writers scope, liveness contract and recovery contract in ticket 19.
Authorize a new bounded repair/review budget before implementing tickets 20–23;
Fix the mainline archive/Python regression in ticket 25; qualify through ticket 24
before accepting Collab Ops.

## Open questions

Should the shared guard cover adjacent retained toolbox trunk writers? The draft
recommends including their small adapters; controller redesign is excluded.
Which preparation steps belong inside the shared mutation interface?

## Traps

Passing tests and Standards/Spec reviews do not clear the four reproduced branch
findings. Unknown run evidence currently permits unsafe cleanup. Admission can
fork trunk after a successful push and make recovery ambiguous. Empty cherry-pick
state is a predictable pre-push blocker. Offline record repair currently queries
the remote too early. Avoid relying on those paths until the follow-up is qualified.

The TraeCLI salvage worker was not independently qualified for landing. Its
supervisor false positives and limited archival scope remain recorded. Root
independently verified and removed five completed private checkout directories
and eight finished owned source worktrees. Archives, private Git directories,
run outputs and branch refs remain recoverable; foreign/shared work was preserved.

## Do not redo

Do not rerun the same exhausted repair cycle or resubmit unchanged findings.
Do not delete shared refs, retained archive packets or other agents' claims.
Do not treat the source snapshot as an acceptance certificate. Exact validation
logs and archive hashes are recorded in evidence; preserve their qualification.

<!-- effort:machine:start -->
```json
{
  "base_branch": "main",
  "base_commit": "3b7b0b0f4d188ad34e5598c704d5887d854d7ff5",
  "dehydrated": "2026-10-12T04:37:31Z",
  "effort": "collab-ops",
  "runs": [],
  "worktrees": [
    {
      "branch": "codex/collab-ops-complete",
      "dirty": [],
      "head": "3b7b0b0f4d188ad34e5598c704d5887d854d7ff5",
      "path": "/data02/home/philip.yang/workspace/corenius-codex-collab-ops-complete",
      "pushed": false
    }
  ]
}
```
<!-- effort:machine:end -->
