# 23 local-record-recovery: Repair established local success while remote is offline

Type: task
Status: claimed
Owner: trae
Blocked by: 
Claimed: 2026-10-12T17:38:46Z expires 2026-10-13T17:38:46Z

Triage: ready-for-agent

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [x] After after_git_land, after_records_land or landed_records_incomplete, exact attributed local evidence permits only idempotent record/state repair without remote I/O.
- [x] No duplicate acceptance record, repeated push or repeated Integration FF occurs.
- [x] Offline before outstanding local FF remains uncertain; exact remote acknowledgment allows recovery; divergent evidence refuses.

## Comments
- 2026-10-12T17:39:00Z trae: unblocked by resolved ticket 19 decision record `decisions/2026-10-12-mutation-contract.md`.
- 2026-10-12T17:38:46Z trae: claimed until 2026-10-13T17:38:46Z
- 2026-10-12T17:49:00Z trae: recovery now checks exact attributed local publication before remote inspection and repairs records/state offline without duplicate acceptance records. Evidence: `LifecycleOpsTests.test_pushed_local_success_recovers_records_while_remote_offline` passed in focused run; affected Collab/legacy suite passed 46 tests.
