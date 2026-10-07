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
