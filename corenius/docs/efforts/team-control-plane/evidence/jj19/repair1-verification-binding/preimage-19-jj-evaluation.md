# 19 jj-evaluation: Evaluate optional jj workspace backend

Type: task
Status: claimed
Owner: codex-jj19
Blocked by: 11, 15, 16
Claimed: 2026-10-03T07:06:27Z expires 2026-10-04T07:06:27Z
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

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest evaluation slice and record fresh passing evidence.
- [x] Rewrite after verification; create divergent operations/stale workspaces; crash before working-copy update; add ignored/private files; export conflicts; run gate/evidence/GC and fast-forward integration fixtures. Retained immutable candidate proof stays correct; changed content requires new proof.
- [ ] Standards and Spec reviews have no unresolved blocking finding.
- [x] Record artifact/run IDs, limitations and handoff to dependent tickets.

## Execution sequence

1. Rehydrate approved intent, check blockers, claim this ticket and allocate an isolated worktree.
2. Derive session-local exact files/commands from the interfaces; approve no new architectural authority implicitly.
3. Run the acceptance fixture red, implement, then run it green; retain meaningful red/green evidence.
4. Run focused checks, applicable repository gate and independent review; at most three repair/review rounds.
5. Update durable evidence and status only when requirements are proven. Commit/push need explicit authority; human integration remains separate.

## Comments

- 2026-10-01T05:48:16Z lead: Detailed planning task created at the owner's request. No execution or architectural decision is implied.
- 2026-10-03T07:06:27Z codex-jj19: claimed until 2026-10-04T07:06:27Z
- 2026-10-03T07:06:28Z codex-jj19: Gated evaluation execution authorized after accepted kernel16 exact evidence-bearing base `264756edd550225449202b6f20ac4b560808d14e`. Solewriter branch `codex/jj19-evaluation-author`; approved seams are the six workspace operations and five canonical handoff behavior slices. This ticket evaluates only: no ticket25 backend implementation, shared checkout conversion, real mainline update, provider invocation, or inherited private configuration. Matt routing owner is not exposed by this harness; repository crosswalk applies directly with research/TDD support. Root retains independent Standards/Spec, branch verification, and integration authority. At most three bounded repair iterations.
- 2026-10-03T07:27:24Z codex-jj19: Five jj slices each have an actual behavioral falsification (exit 1) and passing real disposable execution (exit 0); a sixth real Git baseline compares status, checkpoint, immutable candidate retention and repository/checkout reconciliation. Existing wt/land/evidence suites passed 12/8/15 tests; candidate fast-forward gate, evidence tamper rejection and GC used exported jj commit/tree. Evaluation report recommends defer, with exact IDs, source pin, fault/cleanup and explicit limits. Ticket remains claimed pending independent Standards/Spec, branch verification and owner decision; ticket25 is not authorized. No repair/review round has yet occurred.
