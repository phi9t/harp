# 24 restore-rollout: Upgrade, restore and persistent-use handoff

Type: task
Status: open
Owner:
Blocked by: 16, 18, 21, 22, 23, 25
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement readiness, schema/version migration, workflow replay checks, consistent backups, referenced-artifact retention and dispatch-disabled restore. Allocate recovery epoch outside restored state, revoke pre-restore grants and reconcile all inventory, including post-backup orphans.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Restore returns a recovery inventory and explicit authorization checkpoint before dispatch. Unknown inventory coverage keeps target closed. Migration/runtime credentials are separate; optional adopted backends have their own compatible backup/upgrade evidence.

## Exclusions

No production deployment, automatic merge, unmeasured RPO/RTO or active multi-host failover.

## Acceptance

- [ ] Establish a failing public behavioral fixture for the required outcome.
- [ ] Implement the smallest slice and record fresh passing evidence.
- [ ] Restore an older DB while a post-backup job and old credential survive: credential rejected, orphan accounted for, no duplicate launch. Test missing artifacts, incompatible workflow/schema, interrupted upgrade and rollback. Complete focused/required repo checks and independent branch review.
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
