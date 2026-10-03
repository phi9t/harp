# 16 kernel-proof: End-to-end durable kernel and operator CLI

Type: task
Status: claimed
Owner: codex-kernel
Blocked by: 11, 13, 14, 15
Claimed: 2026-10-02T16:49:45Z expires 2026-10-03T16:49:45Z
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

- [ ] Establish a failing public behavioral fixture for the required outcome.
- [ ] Implement the smallest slice and record fresh passing evidence.
- [ ] Close every view, restart broker/workflow worker/gateway, exhaust budgets, present unknown usage, corrupt evidence and change integration base. Outcomes meet the public SPEC failure matrix. No duplicate effects or false acceptance; uncertainty is visible and holds capacity.
- [ ] Standards and Spec reviews have no unresolved blocking finding.
- [ ] Record artifact/run IDs, limitations and handoff to dependent tickets.

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
