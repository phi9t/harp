# 11 artifact-acceptance: Immutable candidate publication and verified acceptance

Type: task
Status: resolved
Owner: codex-artifacts
Blocked by: 10
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Build immutable artifact publication and submission/verification/acceptance transactions. Bind candidate commit/tree, base, specification, test-suite and environment identities. Gate integration preparation against the actual resulting candidate in fixture repositories.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Publish returns a verified immutable manifest; submit records a candidate; verification records exact evidence; accept checks current candidate and policy. Accepted artifact references cannot point at incomplete objects.

## Exclusions

No production push/merge or acceptance based on CLEAN, issue close or prose.

## Acceptance

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Inject failures before publication finalization and before DB reference. Missing/corrupt content blocks acceptance. Old specification/candidate/base proof fails. Unreferenced objects can be collected after a grace period; referenced evidence cannot. A changed integration base triggers a new candidate/check.
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
- 2026-10-01T08:08:22Z codex-artifacts: claimed until 2026-10-02T08:08:22Z

## Accepted proof

Repair1 passed independent Standards and Spec review. Eight exact reviewed
source files integrated into primary isolated checkout, then fresh root gate
exit0: ten artifact/Git +16admission realPG,114Python,71Rust tests.
DistinctVerifierdenial, publicmigrationrolegrant and actualGitbundle/checklogs
throughPGacceptance proven. No productionintegration/Temporal/liveruntime or
powerloss guarantee. Source/proof manifest and executionledger retained.
- 2026-10-01T08:55:03Z codex-artifacts: resolved
