# 19 mutation-design: Approve shared mutation contract and bounded safety follow-up

Type: task
Status: claimed
Owner: trae
Blocked by: 
Claimed: 2026-10-12T17:35:52Z expires 2026-10-13T17:35:52Z

Triage: ready-for-agent

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md), [architectural evidence](../evidence/architecture-review.md) and [the approved mutation contract](../decisions/2026-10-12-mutation-contract.md). Classification: gated; concurrency, publication and destructive cleanup invariants. The current implementation request approves the bounded follow-up budget for tickets 20-25.

## Acceptance
- [x] Choose shared landing_protocol mutation interface and its exact lock order; record the all-managed-writers scope or explicitly narrower guarantee.
- [x] Approve liveness decision and three-effect recovery contracts in FOLLOWUP.md; preserve public CLI and compatibility semantics.
- [x] Record a fresh bounded implementation/review budget before starting remediation; this draft grants no extension.

## Comments
- 2026-10-12T17:35:52Z trae: claimed until 2026-10-13T17:35:52Z
- 2026-10-12T17:38:00Z trae: approved the shared `landing_protocol.py` mutation interface, lock order, all-managed-writers scope, liveness states, three-effect recovery contract and bounded ticket 20-25 repair/review budget in `decisions/2026-10-12-mutation-contract.md`.
