# 20 mutation-fence: Fence every managed trunk mutation and pending operation

Type: task
Status: open
Owner:
Blocked by: 19
Claimed:

Triage: needs-triage

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [ ] Admission, allocation recovery and claim release refuse unrelated pending core or legacy publication before changing ticket bytes, refs or Attempt reservation.
- [ ] Held lane and surviving inherited Git-child locks fence mutation; exact operation recovery remains possible.
- [ ] Integrate the approved adjacent toolbox writers or document the approved restricted guarantee; preserve uninitialized legacy behavior.

## Comments
