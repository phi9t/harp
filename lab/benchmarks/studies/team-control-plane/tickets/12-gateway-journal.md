# 12 gateway-journal: Exclusive gateway dispatch and uncertain-effect reconciliation

Type: task
Status: resolved
Owner: codex-gateway
Blocked by: 10
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement capability-profiled runtime adapter and deterministic fault runtime. Persist operation intent before I/O; serialize gateway control and enforce physical-thread ownership across binding aliases. Distinguish thread creation without a known ID from turn start on a known thread.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Adapter supports inspect capabilities, create/recover binding, start episode, observe, steer, cancel and reconcile. Outcomes are confirmed, failed_known, outcome_unknown or quarantined with evidence. Non-idempotent operation retries query the same journal entry.

## Exclusions

No claim of live Trae semantics, multi-host active failover or exactly-once external RPC.

## Acceptance

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Drop start replies, crash at every intent/dispatch/reply boundary, deliver duplicates, retain an old controller, alias the same thread and race adoption. One effect or explicit quarantine; lease expiry never permits blind restart. Capability absence returns a named limitation.
- [x] Standards and Spec reviews have no unresolved blocking finding.
- [x] Record artifact/run IDs, limitations and handoff to dependent tickets.

## Execution sequence

1. Rehydrate approved intent, check blockers, claim this ticket and allocate an isolated worktree.
2. Derive session-local exact files/commands from the interfaces; approve no new architectural authority implicitly.
3. Run the acceptance fixture red, implement, then run it green; retain meaningful red/green evidence.
4. Run focused checks, applicable repository gate and independent review; at most three repair/review rounds.
5. Update durable evidence and status only when requirements are proven. Commit/push need explicit authority; human integration remains separate.

## Comments

- 2026-10-01T05:48:16Z lead: Detailed planning task created at the owner's request. No execution or architectural decision is implied.
- 2026-10-01T08:08:22Z codex-gateway: claimed until 2026-10-02T08:08:22Z

## Accepted proof

Two worker repair rounds passed independent Standards/Spec reviews. A third
bounded integration-glue correction fixed the migration transaction signature
and passed separate review. Fresh joint root gate exit0:10artifact/Git,4gateway,
16admission realPG,114Python,71Rust tests; fmt/clippy alltargets-Dwarnings pass.
ActualsixOS SIGKILLboundaries and per-invocation nonidempotent effects/counts
prove deterministic single-host no-blind-relaunch or explicitquarantine.
Sharedmigrations run0001->0002->locked0003; mergedmanifest retained.
No liveprovider/native-dedup/transporttakeover/Temporal/multihost/powerlossclaim.
- 2026-10-01T09:11:15Z codex-gateway: resolved
