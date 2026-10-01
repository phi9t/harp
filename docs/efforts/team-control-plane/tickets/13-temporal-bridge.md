# 13 temporal-bridge: Durable mission/task workflows and database command bridge

Type: task
Status: resolved
Owner: codex-temporal
Blocked by: 10, 12
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement deterministic mission/task workflows with Activities for external work. Register stable aggregate/generation Workflow IDs transactionally, relay outbox hints, process durable command dispositions and contiguous cursors, and use durable timers for lost hints.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Workflow generation and cursor survive Continue-As-New. Activities use stable transition/operation IDs. Terminal wakeups reject or allocate an authorized new generation. DB transitions remain authoritative; histories contain milestones/artifact references.

## Exclusions

No DB/model/RPC/Git I/O in Workflow code and no second task status authority.

## Acceptance

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Replay retained histories; repeat Activities; lose/duplicate signals; insert cursor gaps; send commands across rollover and after terminal completion. No lost command or repeated start. Old-generation Activities fail. SDK feasibility must pass before choosing production worker language.
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
- 2026-10-01T09:22:53Z codex-temporal: claimed until 2026-10-02T09:22:53Z

- 2026-10-01T10:11:12Z lead: Independent two-axis review completed; consolidated repair1 assigned in sole-writer tree. Acceptance remains unchecked and exact initial evidence preserved.
- 2026-10-01T10:44:50Z codex-temporal: resolved
