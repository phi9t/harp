# 21 trae-adapter: Live Trae adapter with supervised recovery

Type: task
Status: open
Owner:
Blocked by: 12, 15, 16, 20
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement only measured Trae capabilities behind the existing runtime seam. Persist versioned bindings, supervise gateway connections, disable competing native continuation, and apply recover-or-quarantine protocols to live effects.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Runtime events update observed episode/Execution state through idempotent broker commands. Safe steering targets expected turn ID; lost delivery stays pending. Unsupported inspection disables unsafe automatic recovery.

## Exclusions

No blind retries, arbitrary endpoint load balancing or fake-runtime results labeled live proof.

## Acceptance

- [ ] Establish a failing public behavioral fixture for the required outcome.
- [ ] Implement the smallest slice and record fresh passing evidence.
- [ ] Repeat kernel loss/duplication/cancel/alias tests against live Trae in bounded fixtures. Restart app-server, broker and gateway independently; reconcile surviving jobs. Report each tested recovery path and unsupported limitation.
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
