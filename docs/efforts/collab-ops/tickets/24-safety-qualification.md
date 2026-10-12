# 24 safety-qualification: Qualify the follow-up against real Git and independent reviews

Type: task
Status: claimed
Owner: trae
Blocked by: 
Claimed: 2026-10-12T18:02:04Z expires 2026-10-13T18:02:04Z

Triage: ready-for-agent

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [ ] Convert all nine retained architecture reproductions to public CLI behavioral regressions and pass the additional acceptance scenarios in architecture-review.md.
- [ ] Run affected toolbox/shim suites, Collab suite, lint and one full default gate after focused checks pass.
- [ ] Fresh Standards and Spec reviews plus independent branch defect review have no unresolved blockers before declaring Collab Ops accepted.

## Comments
- 2026-10-12T17:57:00Z trae: unblocked after integration branch merged ticket 20/21/23 mutation-recovery branch, ticket 22 liveness branch and ticket 25 archive compatibility branch with focused affected suites passing.
- 2026-10-12T18:02:04Z trae: claimed until 2026-10-13T18:02:04Z
