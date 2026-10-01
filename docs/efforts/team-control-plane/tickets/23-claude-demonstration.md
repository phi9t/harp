# 23 claude-demonstration: Claude lead end-to-end reference demonstration

Type: task
Status: open
Owner:
Blocked by: 11, 14, 21
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Repeat the reference mission with Claude lead from the same baseline and limits, using independent workspaces and evidence. Compare contract adherence and recovery paths, not unsupported speed claims.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Uses identical broker tool/domain contracts and acceptance criteria as the Codex run; separate run IDs, artifacts, budget and replacement context.

## Exclusions

No reuse of Codex results as Claude behavioral proof or unapproved live budget.

## Acceptance

- [ ] Establish a failing public behavioral fixture for the required outcome.
- [ ] Implement the smallest slice and record fresh passing evidence.
- [ ] Lose Claude lead/view, recover pending questions and attempts, exercise the same supported fault class and gate the integration candidate. Any lead-specific contract gap is recorded and repaired within the bounded policy.
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
