# 16 kernel-proof: End-to-end durable kernel and operator CLI

Type: task
Status: resolved
Owner: codex-kernel
Blocked by: 11, 13, 14, 15
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Assemble real PostgreSQL and Temporal with the deterministic runtime into one assign/start/yield/resume/submit/verify/accept vertical proof. Add status, inspect, pause, cancel, reconcile and exclusive takeover commands, with views independent of execution ownership.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

CLI calls the same authenticated broker commands; status reports desired/observed state, unresolved operations, budgets and evidence. Takeover suspends dispatch and requires reconciled ownership transfer. Fixture configuration has explicit finite concurrency/episode/repair budgets.

## Exclusions

No claim that fake runtime proof establishes Trae detach/recovery behavior.

## Acceptance

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Close every view, restart broker/workflow worker/gateway, exhaust budgets, present unknown usage, corrupt evidence and change integration base. Outcomes meet the public SPEC failure matrix. No duplicate effects or false acceptance; uncertainty is visible and holds capacity.
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
- 2026-10-01T16:39:06Z codex-kernel: claimed until 2026-10-02T16:39:06Z
- 2026-10-02T16:49:45Z codex-kernel: claimed until 2026-10-03T16:49:45Z


## Answer

Accepted kernel behavior at source `fc0d9b147ab4244af5025314d622c1aebb98fc6d`: real PostgreSQL/Temporal vertical lifecycle, authenticated operator control, finite budgets, recovery/fencing, immutable verification provenance and safe episode reconciliation. Independent bounded Standards/Spec PASS, integrated repository gate12 session46372 exit0 and wholebranch617 session61562 exit0 (no issues) establish current candidate coverage. All48 committed source hashes match the reviewed freeze. Exact proofs, IDs and limitations are linked from [current handoff](../evidence/kernel16-handoff.md), [conformance](../evidence/kernel16-review611-conformance.json) and [quarantine disposition](../evidence/kernel16-review611-acceptance-disposition.json).

Historical failed1813 is retained quarantine, not green proof. Privileged namespace audit observes no current survivor/no permission gaps. Historical shutdown/DB accounting cannot be reconstructed; no adoption/deletion/refund/reuse is permitted on that evidence. Independent verifier adjudicated this as the SPEC-allowed precise unresolved outcome. Live Trae behavior, Kata/jj, independent leads and rollback restore remain dependent work, not kernel claims. Fast-forward landing awaits a fresh detached exact-tip gate.
- 2026-10-03T07:03:55Z codex-kernel: resolved
