# 23 local-record-recovery: Repair established local success while remote is offline

Type: task
Status: open
Owner:
Blocked by: 19
Claimed:

Triage: needs-triage

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [ ] After after_git_land, after_records_land or landed_records_incomplete, exact attributed local evidence permits only idempotent record/state repair without remote I/O.
- [ ] No duplicate acceptance record, repeated push or repeated Integration FF occurs.
- [ ] Offline before outstanding local FF remains uncertain; exact remote acknowledgment allows recovery; divergent evidence refuses.

## Comments
