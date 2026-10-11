# 01 cli: Implement the local Git-ticket coordination CLI

Type: feature
Status: claimed
Owner: collab-implementer
Blocked by: 
Claimed: 2026-10-11T21:20:43Z expires 2026-10-12T21:20:43Z

Triage: ready-for-agent

## Acceptance
- [x] Implement SPEC.md commands and invariants at tools/collab/.
- [x] Supply fresh CLI integration proof, recovery/concurrency regressions and safe temp handling.
- [x] Add operator documentation, root navigation and checks.d registration.
- [x] Reconcile independent Standards, Spec and defect review; report full-gate limits.

## Scope
Use the provided prototype; preserve existing controller and policy.

## Specification
Canonical requirements: ../SPEC.md. No blocking predecessors. Design and public-seam gates are complete.
Gated classification and skill fallback: ../EFFORT.md.

## Authority
Only this isolated worktree may be edited. No commits/push/PR/merge or live activation.
Disposable Git repos may be committed and landed by behavioral tests.

## Comments
- 2026-10-11T21:20:43Z collab-implementer: claimed until 2026-10-12T21:20:43Z

- 2026-10-11T22:22:14Z collab-implementer: acceptance evidence in ../evidence/implementation.md, ../evidence/review.md and ../evidence/verification.md. All four criteria are satisfied; final48 CLI tests and independent reviews pass, initial report-placement lint repaired and fresh global lint passes. No Corenius commit/push/PR/merge or live adoption.

- 2026-10-11T22:22:14Z collab-implementer: tracker resolve refused because there is no accepted, landed Candidate. Implementation/review/verification are complete; ticket lifecycle remains claimed pending human-authorized commit and landing. No acceptance or landing receipt was fabricated.
