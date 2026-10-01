# 08 shared-effect-enforcement: fence effects beyond database submissions

Type: task
Status: resolved
Owner:
Blocked by: 05
Claimed:
Label: wayfinder:grilling
Parent: ../map.md

## Question

Which execution handles, isolated attempt workspaces, scoped credentials and
privileged shared-resource operations are required before stale workers can
be safely replaced? A revoked broker generation does not stop an old process
editing files or using credentials. Define actual enforcement for integration,
artifact acceptance and retained external jobs.

## Acceptance

- [x] Separate logical fencing, process containment and shared-service authorization guarantees.
- [x] Define stop/adopt/quarantine evidence and workspace reuse preconditions.
- [x] Specify real harmless subprocess/namespace tests and non-destructive integration fixtures.

## Comments

## Answer

The owner approved the recommended baseline on 2026-10-01T07:20:06Z. The reviewed
[SPEC.md](../SPEC.md) supplies this kernel design contract;
[decision](../decisions/20261001-approved-baseline.md) records scope, public
test seam, evidence and live-probe limitations. Resolution is design/admission
authority, not implementation or real provider behavioral acceptance.

Kernel-only decision: removed premature live-probe dependency04. Live provider
capability and effect guarantees remain gated by04/20/21.
