# 06 dispatch-reconciliation: define exclusive dispatch and uncertain-effect recovery

Type: task
Status: resolved
Owner:
Blocked by: 05
Claimed:
Label: wayfinder:grilling
Parent: ../map.md

## Question

How do durable dispatch ownership, per-binding serialization and external
operation observations prevent concurrent Activities/gateways issuing duplicate
starts? Define loss of dispatcher ownership, separate thread/turn unknown
outcomes, adoption/quarantine, and when reassignment may proceed. A journal
intent or JSON-RPC request ID alone cannot establish safe duplicate suppression.

## Acceptance

- [x] Define precise operation states, revision/fence checks and dispatch ownership transitions.
- [x] Specify retryable versus non-retryable/unknown errors and budget treatment after crashes.
- [x] Define fault-injection verdicts for competing dispatch, reply loss and stale ownership.

## Comments

## Answer

The owner approved the recommended baseline on 2026-10-01T07:20:06Z. The reviewed
[SPEC.md](../SPEC.md) supplies this kernel design contract;
[decision](../decisions/20261001-approved-baseline.md) records scope, public
test seam, evidence and live-probe limitations. Resolution is design/admission
authority, not implementation or real provider behavioral acceptance.

Kernel-only decision: removed premature live-probe dependency04. Live provider
capability and effect guarantees remain gated by04/20/21.
