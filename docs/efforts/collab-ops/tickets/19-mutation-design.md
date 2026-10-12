# 19 mutation-design: Approve shared mutation contract and bounded safety follow-up

Type: task
Status: open
Owner:
Blocked by: 
Claimed:

Triage: needs-triage

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [ ] Choose shared landing_protocol mutation interface and its exact lock order; record the all-managed-writers scope or explicitly narrower guarantee.
- [ ] Approve liveness decision and three-effect recovery contracts in FOLLOWUP.md; preserve public CLI and compatibility semantics.
- [ ] Record a fresh bounded implementation/review budget before starting remediation; this draft grants no extension.

## Comments
