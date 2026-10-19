# 31 independent-execution: qualified scope-owned ExecutionContract

Type: task
Status: open
Owner:
Blocked by: 15, 27
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement Corenius's independently testable Armada/Insula execution seam and
qualify the selected provider profile. A stable Execution is owned by scope;
original creating role/Episode/Attempt is immutable provenance and controller
is replaceable. Creating Attempt is optional for role and legacy executions.
Ensure/inspect/stop and inventory outlive lead/runtime/workflow loss. Follow
[v0.2 reconciliation](../v02-reconciliation.md).

Pin target/Insula/provider identities, recipe requirements and capability
observations. Journal launch before I/O; qualify deduplication or discovery,
independent inventory/orphan coverage, resource ceilings, logs/checkpoint
integrity and cancellation versus physical termination. Unknown stays held.
The seam is usable without29's interpreter, avoiding a dependency cycle.

## Interface contract

ExecutionContract returns stable handle, scope, optional original role/Episode/Attempt, target,
Insula, capability-profile digest, operation identity and lifecycle/evidence.
Public ensure/inspect/stop read or act only under current broker grant and
resource reservation. Inventory reports coverage, observations and unknowns;
stop-request receipt is distinct from confirmed termination.
Preserve multiple jobs and original attribution across controller changes.

## Exclusions

Corenius files only; no Armada/Insula sibling edits or production/GPU placement
implied. No launcher-PID-only inventory, local namespace evidence promoted to
remote guarantees, uncontrolled credentials or paid provider launch here.

## Acceptance

- [ ] Fixed-version local physical fixture RED/GREEN: job survives initiator/lead loss, remains independently addressed, and scope replacement does not relaunch it (F05/F07).
- [ ] Lost ensure response adopts exact discovered effect or quarantines; retries never blindly start a second job.
- [ ] Nested/setsid descendants, missing sandbox capability and incomplete inventory stop admission or retain precise quarantine with capacity held (F11/F22).
- [ ] Current controller can inspect prior job without rewriting provenance; stale controller cannot mutate; provider/goal changes preserve usage and unknown reservations.
- [ ] Cancel receipt and confirmed shutdown are separate; bounded silent healthy job is not failed from missing chat activity (F29).
- [ ] Pin and qualify any selected real Armada/Insula API path; unavailable/unsupported capabilities are explicit blockers, not fixture-only product claims.
- [ ] Independent conformance/branch review accepts exact execution/source/profile evidence; retained logs/checkpoints and orphan observations survive restart.

## Comments

- 2026-10-03T17:30:00Z lead: Reuse accepted15 supervisor seam, then qualify extension. Implementation remains confined to Corenius.
