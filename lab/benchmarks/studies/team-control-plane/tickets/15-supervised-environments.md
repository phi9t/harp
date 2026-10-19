# 15 supervised-environments: Attempt workspaces and independent workload supervision

Type: task
Status: resolved
Owner: codex-supervisor
Blocked by: 10, 12
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Integrate Git workspace lifecycle with broker reservations and an independently supervised Execution registry. Protect allowed paths through actual capsule boundaries and retain complete workload inventory across agent/gateway loss. Fence GC before workspace creation/release/deletion races.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Workspace manifest contains store/path/base/attempt identity and write scope. Execution handle is registered before workload start. Inspect/cancel/reconcile cover nested process/session/namespace membership. Deadline uses the cancellation shutdown path.

## Exclusions

No assumption that a launcher PID, worktree or lease alone fences filesystem effects.

## Acceptance

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Timeout and cancel capsule workers with surviving child/background jobs; kill the launcher; crash during create/register; race GC and launch; attempt stale shared writes. Stopped membership is proven or workspace/capacity quarantined. Never reuse a directory with uncertain survivors.
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
- 2026-10-01T09:22:53Z codex-supervisor: claimed until 2026-10-02T09:22:53Z

- 2026-10-01T10:00:57Z lead: Initial candidate frozen; independent Spec found five blocking authority/path/launch/file-scope/operator-recovery gaps. [Review](../evidence/ticket15-review-initial.md). Await Standards before consolidated repair1; acceptance unchecked.

- 2026-10-01T10:11:12Z lead: Independent two-axis review completed; consolidated repair1 assigned in sole-writer tree. Acceptance remains unchecked and exact initial evidence preserved.

- 2026-10-01T10:53:02Z lead: Repair1 Spec passed; Standards blocked on silent liveness-observation errors. Root confirmed source. Consolidated repair2of3 assigned, no integration or acceptance.
- 2026-10-01T11:51:40Z codex-supervisor: resolved
