# 19 jj-evaluation: Evaluate optional jj workspace backend

Type: task
Status: open
Owner:
Blocked by: 11, 15, 16
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Evaluate pinned jj in disposable fixture repositories behind the workspace interface, compare checkpoints/recovery/integration against Git, and record adopt/defer. Preserve one writer per tree and shared-store mutation authority.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Workspace operations are create, inspect, checkpoint, freeze candidate, prepare integration and release. Manifest adds jj operation/change IDs for navigation and exact commit/tree identities for evidence. Snapshot authority is explicit because status commands may mutate.

## Exclusions

No shared checkout conversion, required kernel dependency or unrestricted cross-attempt rewrites.

## Acceptance

- [ ] Establish a failing public behavioral fixture for the required outcome.
- [ ] Implement the smallest slice and record fresh passing evidence.
- [ ] Rewrite after verification; create divergent operations/stale workspaces; crash before working-copy update; add ignored/private files; export conflicts; run gate/evidence/GC and fast-forward integration fixtures. Retained immutable candidate proof stays correct; changed content requires new proof.
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
