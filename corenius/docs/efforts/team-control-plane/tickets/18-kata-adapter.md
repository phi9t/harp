# 18 kata-adapter: Optional approved-planning and projection adapter

Type: task
Status: resolved
Owner: codex-lead
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

- [x] Accepted evaluation17 records the explicitly approved deferred decision.
- [x] Kernel16 prerequisite is accepted and landed; retain Git planning boundary.
- [x] No Kata adapter or adoption is claimed; limitations and future adoption obligations are recorded.

## Deferred scope

The following original adoption-only criteria are not applicable to the selected deferred path:

- Public failing behavioral fixture and smallest implementation slice with passing evidence.
- Kata outage/lost acknowledgement, duplicate projection, admitted-content edit and unverified-close fault probes for a production adapter.
- Standards/Spec review and artifact/run handoff for that adapter implementation.

Evaluation17's actual probes and independent reviews remain evidence for deferral, not proof that a production adapter exists.

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
- 2026-10-03T07:10:41Z codex-lead: claimed until 2026-10-04T07:10:41Z


## Answer

Resolved through the explicit deferred branch. Kernel16 is accepted and landed at `264756edd550225449202b6f20ac4b560808d14e`; ticket17 evaluation/independent reviews and the [approved deferral decision](../decisions/20261001-kata-deferred-first-kernel.md) establish that Kata is optional and deferred. Preserve Git-approved planning and PostgreSQL execution authority. No Kata adapter, migration, hooks or runtime deployment was implemented or adopted. Original adoption-only behavioral criteria above are explicitly not applicable to this conditional deferral; no passing implementation claim is made. Future adoption requires a new explicit decision and the adapter fault proofs.
- 2026-10-03T07:11:04Z codex-lead: resolved
