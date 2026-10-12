# 22 liveness-evidence: Retain workspaces with unusable worker evidence

Type: task
Status: claimed
Owner: trae
Blocked by: 
Claimed: 2026-10-12T17:38:46Z expires 2026-10-13T17:38:46Z

Triage: ready-for-agent

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [ ] Missing, malformed, non-mapping, nonpositive or invalid PID/session and malformed completion produce unknown with an explicit reason.
- [ ] Inspect, abandon, recover, refresh, GC, sweep and retention consume the same conservative decision and retain unknown workspaces/refs.
- [ ] Valid stopped workers permit cleanup; live sessions override completion; explicitly unbound confirmation still works.

## Comments
- 2026-10-12T17:39:00Z trae: unblocked by resolved ticket 19 decision record `decisions/2026-10-12-mutation-contract.md`.
- 2026-10-12T17:38:46Z trae: claimed until 2026-10-13T17:38:46Z
