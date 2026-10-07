# 13 controller03-salvage-repair: Repair and verify frozen controller benchmark outcome evidence before landing

Type: bug
Status: claimed
Owner: codex-controller03-salvage
Blocked by:
Claimed: 2026-10-07T09:53:38Z expires 2026-10-08T09:53:38Z

Triage: needs-triage

## Acceptance
- [ ] Construct the exact reviewed five-file controller03 semantic delta over current main; preserve landed readiness APIs and behavior.
- [ ] Repair uncertain success cleanup, unverified manual-intervention task verdicts and resolved failure dispositions with causal regressions.
- [ ] Retire three default Kata paths into the stable content-addressed artifact directory while preserving overrides and required hashes.
- [ ] Run focused mixed legacy/controller/readiness checks on immutable candidate and pass independent Standards/Spec review.
- [ ] Publish verified landable candidate and preserve/prune its owned temporary tree after fast-forward landing.

## Intent, source and authority

The user’s active goal authorizes landing all landable work and fixing encountered
issues. This is a bounded repair of existing pending implementation against the
already approved controller-backed-bench Ticket03/SPEC, not a new feature design.
Gated path: process closure and result-evidence invariants; use existing test seam,
causal red/green proof and independent review. Base mainc213c8c; exact frozen
snapshot is controller03-frozen-review-64647782 in the verified salvage archive.
The source controller_mode.py SHA is01b0f9b81d787cc29b928e925ef3130148ebdc81b0e4d7b2a4fbb02acbbb3aaa.
Original Ticket03 live owner and checkout stay untouched. No model/live trial is
included. Separate candidate owns all repairs; no shared branch rebase or force.

## Findings and repair budget

Independent Standards and Spec reviews reject the frozen continuation: success
paths claim resolved effects/services stopped even when cleanup has exit=None;
manual-intervention classification invents task PASS before verification; some
failure stages ignore proved resolved cleanup. Whole worker results.py replacement
would erase current main readiness APIs. Earlier47-test log refers to different
source bytes and is not this candidate’s verification.
This is the next bounded repair iteration after the original owner’s repair1;
do not reset the three-round repair cap. Record and escalate non-convergence.

## Comments
- 2026-10-07T09:53:38Z codex-controller03-salvage: claimed until 2026-10-08T09:53:38Z

### Repair 2 candidate, 2026-10-07

The exact five-file delta was merged over the current readiness source. Only the result-schema constants conflicted;
readiness constants and APIs were preserved alongside controller v2 and legacy v1 validation. Success now uses observed
service and Attempt closure; pre-verifier manual pause retains UNKNOWN task truth; proven reconciliation resolves effects
independently of a failed task or orchestration. The three causal regressions failed before repair and passed afterward.
The mixed controller/legacy/readiness gate passed 94 tests in 828.259 seconds. The protected-verifier validator passed
41 tasks with no failures. This remains repair 2 of 3, following the original owner's repair 1.

Repository cleanup exposed a second bounded defect: three runtime/fixture defaults named a retired Kata worktree. The
current user goal and explicit lead delegation extend this repair to those three lookup sites, their necessary regression,
and the benchmark README. Defaults now select the same approved digest under the user's stable pinned-tools directory.
The controller and SQLite entry point use `CORENIUS_KATA_PINNED_BINARY`; the native fixture retains `KATA_PINNED_BINARY`.
Actual loaded default/override lookup and native fixture hash-guard behavior passed with all six SQLite-entry tests.
Final default runtime/fixture gates, evidence publication and independent Standards/Spec review are pending. Neither this
ticket nor controller-backed-bench Ticket03 is resolved by this candidate.
