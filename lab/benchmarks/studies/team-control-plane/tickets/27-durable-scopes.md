# 27 durable-scopes: scope-owned jobs and conserved authority

Type: task
Status: open
Owner:
Blocked by: 15, 26
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement ExecutionScope ownership independently from Attempt provenance,
controller, lead seat and workspace. Extend existing role separation; preserve
v1 lead_generation and its accepted evidence. Follow the
[v0.2 contracts](../v02-reconciliation.md).
Jobs, unresolved effects, allocations and usage survive attempt/lead replacement.

Explicitly separate ScopeEpoch, RoleEpoch, WorkspaceGeneration,
RuntimeIncarnation, intent/plan revision and deployment recovery epoch.
Multiple jobs per scope/attempt require no one-job-per-attempt uniqueness.
At most one authoritative workspace writer remains mandatory.
Parent/child allocation transfers conserve capacity atomically under races;
unknown usage and cancelled-but-running jobs remain committed. Separate each
cumulative dimension (unspent/reserved/unknown/consumed) from reusable concurrency
capacity (free/reserved/active/unknown). Child envelopes include their subtree
commitments; no child consumption is counted again at its parent.

## Interface contract

Versioned scope admit/read/revoke/reconcile and child-allocation commands extend
existing broker transaction/journal seams. Grants and effect receipts expose
applicable independent axes and preconditions. Execution records name owning
scope and optional original role/Episode/Attempt provenance separately; replace
controller without reattribution. Legitimate legacy NULL Attempt records stay
NULL, with explicit admitted role/mission scope ownership rather than invented
historical attempts.
Scope cancellation distinguishes future-effect revocation, previously granted
effects and independently confirmed termination or quarantine.

## Exclusions

No live provider, scope workflow interpreter or remote orchestration service.
No guessed ownership backfill, refund on timeout, or replacement writer while
an old writer's effects remain unresolved.

## Acceptance

- [ ] Real PG RED/GREEN: role replacement preserves scope/job attribution and budget; stale role or scope grants fail independently.
- [ ] Concurrent child allocations conserve parent+children+consumed/unknown commitments; retries and provider/goal/attempt replacement cannot reset or double-charge allocation (F13–F15).
- [ ] Multiple jobs remain inspectable through attempt loss; creating provenance is immutable and current controller transfer is explicit.
- [ ] Cancellation versus effect grant is serialized; pre-granted effects retain receipt; unreachable stop holds capacity and conflicting writer admission (F09–F11).
- [ ] Useful late cancelled result is retained in quarantine; reuse requires a new current-authority command and exact evidence (F12).
- [ ] Legacy migration with affected admission closed preserves manifests/accepted subjects, including task-free lead jobs with original NULL Attempt; ambiguous backfill quarantines and prohibits dispatch. GC preserves unresolved-operation dependencies (F25).
- [ ] Independent conformance and branch review accept exact migration/source freeze with focused PG and applicable supervisor checks.

## Comments

- 2026-10-03T17:30:00Z lead: Additive scope migration follows26; accepted v1 role/task separation is reused, not relabeled as absent or as complete v0.2 proof.
