# 19 jj-evaluation: Evaluate optional jj workspace backend

Type: task
Status: resolved
Owner: corenius-jj19
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

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Rewrite after verification; create divergent operations/stale workspaces; crash before working-copy update; add ignored/private files; export conflicts; run gate/evidence/GC and fast-forward integration fixtures. Retained immutable candidate proof stays correct; changed content requires new proof.
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
- 2026-10-03T16:56:53Z corenius-jj19: claimed until 2026-10-04T16:56:53Z

- 2026-10-03T16:56:53Z lead: Original frozen evaluation rehashed; Standards PASS, Spec Important verification-order gap. Acceptance withheld. Isolated repair1/3 assigned; see ../evidence/jj19-conformance-round0.json and ../decisions/20261003-jj19-verification-repair.md.
- 2026-10-03T17:36:01Z lead: Repair1 resolves original verification-order/proof-reuse finding. Both axes then required handoff-only retention repair2; bytecode is excluded from authoritative durable inventory while earlier metadata remains historical. Fresh Spec and Standards PASS, all112 selected public paths and six reviewed source/input records verified and transferred to the root. Exact frozen ticket/SPEC inputs are archived under evidence/jj19/repair1-verification-binding; current canonical contracts are not overwritten. Root gate/branch review remain pending. Evidence: ../evidence/jj19-repair2-root-integration.json. RFC0002 supersedes optional/deferred target; evaluation acceptance is distinct from preferred-backend25 qualification.
- 2026-10-03T18:06:00Z lead: Wholebranch630 rejects private-file validation limited to current tree while bundles include reachable history. Root confirmed; acceptance/landing withheld despite gate82804 PASS. Final bounded repair3/3 assigned in fresh isolated history-export tree atd819a94. Original100 evidence records are now retained alongside repair1/2 metadata, with212 total mapped records hash-checked. See ../decisions/20261003-jj19-history-export-repair630.md and ../evidence/jj19-branch630-rejection.json.
- 2026-10-03T18:38:11Z lead: Repair3 frozen fixture0223e87add1595353f2e6d787ab275b0ea1df6c1666436e2f9094872027e65bf has fresh independent Spec PASS and Standards PASS; one duplication heuristic is nonblocking. Root verified/transferred276 bound paths, preserved213 prior evidence files and current canonical ticket history, and integrated exact three reviewed source/report postimages. Frozen author ticket remains at its indexed source-snapshot path. Fresh imported regression and repository gate are running; branch review/acceptance/landing remain pending. See ../evidence/jj19/repair3-history-export/root-integration.json and root-conformance/.
- 2026-10-03T19:00:51Z lead: Both imported actual regressions and required rootgate40945 pass exit0. Wholebranch631 accepts264756e..94aa15f with terminal0/no issues. Evaluation accepted within the recorded path/type/native-export limitations; source postimages remain exact. Detached exact-tip gate/FF landing follows. RFC0002's preferred jj backend still requires separate25 qualification; no v2/live/restore proof is implied. Exact acceptance: ../evidence/jj19/repair3-root-tests/accepted-evaluation.json.
- 2026-10-03T19:00:51Z corenius-jj19: resolved
