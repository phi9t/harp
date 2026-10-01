# 18 kata-adapter: Optional approved-planning and projection adapter

Type: task
Status: open
Owner:
Blocked by: 16, 17
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

If adopted, import explicitly approved immutable Kata snapshots and project broker execution/attention summaries using durable outbox mapping IDs. If deferred, resolve this conditional ticket with the documented decision and keep the Git adapter.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Source edits propose new task revisions. Claims request assignment; closes record completion claims. Projection acknowledgment differs from broker transition application. Deduplication and origin tags prevent feedback admission loops.

## Exclusions

No shared-table coupling, distributed-transaction claim or custom issue UI.

## Acceptance

- [ ] Establish a failing public behavioral fixture for the required outcome.
- [ ] Implement the smallest slice and record fresh passing evidence.
- [ ] Lose Kata availability and acknowledgments, duplicate projection commands, edit admitted issue content and close an unverified issue. Broker recovery continues; no implicit assignment, acceptance or new revision.
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

## Deferred-path disposition recorded

Ticket17 recommends and records [defer within the approved first-kernel baseline](../decisions/20261001-kata-deferred-first-kernel.md).
Retain the Git planning boundary; no Kata adapter or adoption work is requested.
This ticket remains open pending kernel16, then resolves explicitly through its
conditional deferred path; adoption-only behavioral boxes are not implementation claims.
