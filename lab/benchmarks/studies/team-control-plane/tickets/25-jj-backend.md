# 25 jj-backend: qualify the preferred worker VCS and immutable proof path

Type: task
Status: open
Owner:
Blocked by: 19, 27, 30, 31
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

The owner's RFC0002 direction makes jj the preferred worker VCS candidate,
subject to executable qualification. Use19's measured evaluation and implement
the qualified private-store backend behind the workspace interface. The older
defer recommendation remains historical and cannot by itself resolve this
ticket. A required capability that fails qualification needs an explicit owner
disposition; do not silently drop jj from the target or claim it accepted.
Canonical contracts: [SPEC.md](../SPEC.md) and
[v0.2 reconciliation](../v02-reconciliation.md).

## Interface contract

Create attempt, inspect, checkpoint, freeze candidate, prepare integration and
release use the scope-authorized, journaled workspace seam. Return
store/workspace/base identity, immutable commit/tree digest, conflict state,
allowed-path manifest and jj operation/change navigation IDs. No proof binds
only to a change ID. Use private repository stores and separate workspace
generations; join31's independent execution and30's protected Git gate.

## Exclusions

No shared checkout conversion, shared branch rewriting, new execution authority,
operation-log rollback of external effects or implied policy migration.

## Acceptance

- [ ] Pin selected jj version/profile and record measured19 limits plus explicit qualification disposition; no default adoption from recommendation alone.
- [ ] Public fixtures fail before implementation and pass afterward for stale/divergent/crash recovery, exact old/new proof after rewrite, exclusions, capsule paths and conflict-free export.
- [ ] Existing evidence/gate/GC and fast-forward handoff contracts pass against the actual backend.
- [ ] Backup and upgrade preserve referenced exact candidates and required jj metadata; interrupted restore remains dispatch-disabled until reconciled.
- [ ] Poisoned worker build caches/provenance cannot satisfy trusted verification; retained original candidate bundles remain independently verifiable (F16/F23).
- [ ] Assembled32/live demonstrations wait for this ticket; unsupported capability is an explicit decision blocker rather than silent Git-only completion.
- [ ] Independent Standards and Spec reviews and immutable evidence handoff complete.

## Comments

- 2026-10-01T05:50:05Z lead: Added after independent plan review to separate optional evaluation from backend implementation and restore acceptance. Not execution authority.
- 2026-10-03T17:36:01Z lead: RFC0002 supersedes optional/deferred target and blockers; original contract remains preserved in Git history and evaluation evidence. No shared VCS conversion.
