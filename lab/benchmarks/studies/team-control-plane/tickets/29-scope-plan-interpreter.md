# 29 scope-plan-interpreter: durable per-scope declarative coordination

Type: task
Status: open
Owner:
Blocked by: 13, 26, 27, 31
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement an independently addressed Temporal workflow per ExecutionScope under
a stable versioned Plan IR interpreter. PostgreSQL remains authoritative;
signals are hints and Activities recheck current grants. Extend the accepted13
bridge without reclassifying its v1 evidence. Follow
[v0.2 reconciliation](../v02-reconciliation.md).

Use short ensure/inspect/stop Activities, durable waits/timers and bounded
service anti-entropy. No model polling or long job held in an Activity.
Sequence/cursor handling preserves wakes during refresh/drain/wait; retain v1
timer safety until the replacement is demonstrated. Kata hierarchy does not
imply parent-child Temporal cancellation/lifetime.

## Interface contract

Workflow identity pins scope and workflow generation; broker Activities return
intent/plan/interpreter versions, current epochs, outstanding operations and
durable waits. Plan steps have stable identities and operation bindings.
Amendments take effect at recorded safe points; in-flight effects retain old
plan receipts, completed steps cannot be reinterpreted or duplicated.
CAN carries versions, cursor, wait IDs and stable step/operation identities,
then rereads canonical state. Retain compatible interpreter versions/history.

## Exclusions

No eval of generated code, direct workflow I/O, authoritative replayed task
status, blanket child-workflow hierarchy or automatic uncertain-effect retry.

## Acceptance

- [ ] Real SDK+PG RED/GREEN: independent scope workflow crash/replay; repeated hints/start replies do not duplicate jobs or authority.
- [ ] Wake during refresh, drain and wait-entry survives; lost hints recover on bounded timer/anti-entropy (F03/F04).
- [ ] Revocation/intent amendment between read and Activity blocks fresh effect; pre-granted operation remains attributable, unknown ensure inspects rather than relaunches.
- [ ] Plan amendment, join/repair/loop bounds and stable step IDs preserve completed/in-flight semantics and cumulative budgets.
- [ ] Actual CAN/replay retains versions/cursors/waits; incompatible interpreter/history promotion rejects before trusted dispatch (F27).
- [ ] Telemetry flood coalesces hints and keeps control lanes responsive; healthy silent job remains running based on independent observations (F28/F29).
- [ ] Independent Standards/Spec, exact histories/profile/source freeze, applicable repo gate and branch review accepted.

## Comments

- 2026-10-03T17:30:00Z lead: The pasted changed=False illustration is not an implementation. Current500ms v1 timer bounds missed-hint delay; new proof is required before longer sleeps.
