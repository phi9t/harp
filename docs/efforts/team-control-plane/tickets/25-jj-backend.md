# 25 jj-backend: optional accepted jj workspace implementation

Type: task
Status: open
Owner:
Blocked by: 11, 15, 16, 19
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

If ticket 19 records adoption, implement the approved jj backend behind the
workspace interface and exercise actual candidate, environment and recovery
contracts before live use or rollout. If jj is deferred, resolve this conditional
ticket with that explicit exclusion and retain Git as the accepted backend.
Canonical contract: [SPEC.md](../SPEC.md).

## Interface contract

Create attempt, inspect, checkpoint, freeze candidate, prepare integration and
release use the same broker-authorized, journaled workspace seam as Git. Return
store/workspace/base identity, immutable commit/tree digest, conflict state,
allowed-path manifest and jj operation/change navigation IDs. No proof binds
only to a change ID. Keep mutation ownership separate across attempts.

## Exclusions

No shared checkout conversion, shared branch rewriting, new execution authority,
operation-log rollback of external effects or implied policy migration.

## Acceptance

- [ ] Adoption/defer disposition explicitly recorded from ticket 19.
- [ ] If adopted, public fixtures fail before implementation and pass afterward for stale/divergent/crash recovery, exact proof after rewrite, exclusions, capsule paths and conflict-free export.
- [ ] Existing evidence/gate/GC and fast-forward handoff contracts pass against the actual backend.
- [ ] Backup and upgrade preserve referenced exact candidates and required jj metadata; interrupted restore remains dispatch-disabled until reconciled.
- [ ] Live demonstrations using jj wait for this ticket; deferred resolution explicitly forbids claiming jj behavioral proof.
- [ ] Independent Standards and Spec reviews and immutable evidence handoff complete.

## Comments

- 2026-10-01T05:50:05Z lead: Added after independent plan review to separate optional evaluation from backend implementation and restore acceptance. Not execution authority.
