# 17 kata-evaluation: Evaluate Kata and select ledger integration

Type: task
Status: resolved
Owner: codex-kata
Blocked by: 05, 10
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Evaluate a pinned Kata release against approved revision capture, API authorization, claims, closure, events, restore and adapter costs. Decide adopt/defer before building a custom issue UI. Preserve current Markdown planning authority during the evaluation.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Assessment maps source project/issue ID and immutable approved digest to broker task revisions. Rust remote API capabilities must be measured separately from Go embedding hooks. Define projected versus authoritative fields.

## Exclusions

No automatic tracker migration, hook installation, new runtime authority or issue-close acceptance.

## Acceptance

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Claim races, direct close, stale scoped credentials, source edits, duplicate event replay and upgrade/restore retain one execution authority. Record supported and missing behavior with evidence. Adopt only if reuse outweighs bridge/operation cost without a mandatory fork.
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
- 2026-10-01T08:11:35Z codex-kata: claimed until 2026-10-02T08:11:35Z

## Answer

Evaluated pinned Kata through actual remote API, upgrade/restore, immutable
approval mapping, duplicate broker-event application and projection boundaries.
Both independent review axes passed repair2. Fresh narrow guard fixtures reject
unowned/mismatched targets and retain legitimate restore behavior. Exact files
and scoped evidence: [accepted manifest](../evidence/ticket17-accepted-manifest.json).
Disposition: [defer for the first kernel](../decisions/20261001-kata-deferred-first-kernel.md).
No production adapter, adoption, hooks, tracker migration or model run claimed.
- 2026-10-01T09:34:04Z codex-kata: resolved
