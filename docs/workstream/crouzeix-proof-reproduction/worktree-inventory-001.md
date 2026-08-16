# Crouzeix Proof Worktree Inventory 001

## Baseline

- Primary checkout: `/Users/bytedance/workspace/harp`
- Baseline commit: `1d70062ba047d98cc7ea5e4d5b968d5d7d4d8bcf`
- Inventory date: `2026-08-16`
- No proof claim: this document classifies worktree state only.

## Disposition Summary

| Worktree | Branch | Head | `master...HEAD` | Status | Disposition |
|-|-|-|-|-|-|
| `/Users/bytedance/workspace/harp/.worktrees/crouzeix-formal-validation` | `codex/crouzeix-formal-validation` | `1d70062ba047` | `0 0` | tracked-clean; untracked `.teaching/` scratch | already landed; preserve scratch decision before removal |
| `/Users/bytedance/workspace/harp/.worktrees/crouzeix-conjecture-deep-dive` | `integrate/crouzeix-conjecture` | `127c0e0474ee` | `136 0` | tracked-clean | contained in `master`; candidate for retirement |
| `/Users/bytedance/workspace/harp/.worktrees/math-foundations-advanced-linear` | `codex/math-foundations-advanced-linear` | `378e004ab01f` | `58 0` | tracked-clean | contained in `master`; candidate for retirement |
| `/Users/bytedance/workspace/harp/.worktrees/math-foundations-formalization-design` | `codex/math-foundations-formalization-design` | `378e004ab01f` | `58 0` | tracked-clean | contained in `master`; candidate for retirement |
| `/Users/bytedance/workspace/harp/.worktrees/crouzeix-proof-reproduction` | `feat/crouzeix-proof-reproduction` | `71d46df88576` | `101 56` | dirty tracked docs plus untracked proof-agent amendments | primary CPFR proof-reproduction integration source |
| `/Users/bytedance/workspace/harp/.worktrees/crouzeix-retrospective-landing-spec` | `docs/crouzeix-retrospective-landing-spec` | `71d46df88576` | `101 56` | one untracked retrospective spec draft | sidecar evidence; do not land as independent code lineage |
| `/Users/bytedance/workspace/harp/.worktrees/crouzeix-rsi-implementation` | `codex/crouzeix-rsi-implementation` | `0c0b729edfb6` | `101 28` | tracked-clean | required RSI/hermetic-runtime landing lane |
| `/Users/bytedance/workspace/harp/.worktrees/crouzeix-rsi-workflow-spec` | `codex/crouzeix-rsi-workflow-spec` | `bc88c9221cf4` | `132 2` | tracked-clean | docs-only subset of RSI implementation lane; do not land independently |
| `/Users/bytedance/workspace/harp/.worktrees/crouzeix-rsi-workflow-design` | `codex/crouzeix-rsi-workflow-design` | `530e0edeb1f2` | `132 0` | broad tracked deletions plus many untracked files | quarantined; separate recovery only |

## Already Contained

These worktrees have heads contained in current `master` or equal to current
`master`.

### `crouzeix-formal-validation`

- Path: `/Users/bytedance/workspace/harp/.worktrees/crouzeix-formal-validation`
- Branch: `codex/crouzeix-formal-validation`
- Head: `1d70062ba047d98cc7ea5e4d5b968d5d7d4d8bcf`
- Current role: CPFR-070 through CPFR-081 and Lean gate repair lane already
  landed.
- Local scratch: untracked `.teaching/crouzeix-lean-validation/...`.
- Disposition: no code landing needed. Remove only after owner decides whether
  to preserve `.teaching/`.

### `crouzeix-conjecture-deep-dive`

- Path: `/Users/bytedance/workspace/harp/.worktrees/crouzeix-conjecture-deep-dive`
- Branch: `integrate/crouzeix-conjecture`
- Head: `127c0e0474ee3cc534ee7ad4f2ccf38b681e65c5`
- Status: tracked-clean; head contained in `master`.
- Disposition: candidate for retirement after final untracked scratch check.

### Mathematical-foundations worktrees

- Paths:
  - `/Users/bytedance/workspace/harp/.worktrees/math-foundations-advanced-linear`
  - `/Users/bytedance/workspace/harp/.worktrees/math-foundations-formalization-design`
- Head: `378e004ab01fef837490bc0702717047862944f7`
- Status: tracked-clean; head contained in `master`.
- Disposition: candidates for retirement after final untracked scratch check.

## Primary CPFR Proof-Reproduction Lane

The primary source is
`/Users/bytedance/workspace/harp/.worktrees/crouzeix-proof-reproduction` on
`feat/crouzeix-proof-reproduction`.

- Head: `71d46df885762eac23070783506bc08a567d47cf`
- Divergence from current `master`: `101 56`
- Dirty tracked paths:
  - `docs/import-receipt.md`
  - `docs/workstream/crouzeix-proof-reproduction/tracker.org`
- Untracked files:
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-005.md`
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-006.md`
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-007.md`
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-008.md`
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-009.md`
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-010.md`
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-011.md`
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-012.md`
  - `docs/workstream/crouzeix-proof-reproduction/worktree-inventory.md`
- Observed top commits:
  - `71d46df chore(crouzeix): block cpfr-032 after root recovery`
  - `3991a5a chore(crouzeix): record CPFR-032 readiness blocker`
  - `6277b65 fix(crouzeix): close expert frontier readiness gaps`
  - `343e6de feat(crouzeix): prepare expert frontier run`
  - `15d2ab8 fix(crouzeix): validate legacy ticket exceptions`

Disposition: reconcile onto current `master` in a fresh integration worktree.
Do not merge the dirty and behind branch directly.

Subagent finding: all requested CPFR worker branch heads and the three detached
verification commits are already ancestors of `feat/crouzeix-proof-reproduction`.
The preservation risk is therefore uncommitted/untracked material, not missing
worker-branch reachability.

## CPFR Worker Branches

All listed worker worktrees are tracked-clean but predate current `master`.
They should be compared against `feat/crouzeix-proof-reproduction` and current
`master`; do not land them one by one. Initial read-only audit found each worker
head already represented in `feat/crouzeix-proof-reproduction`.

| Worktree | Branch | Head | `master...HEAD` | Initial disposition |
|-|-|-|-|-|
| `cpfr-011-worker` | `work/cpfr-011` | `385cd64b54cc` | `101 6` | represented in primary CPFR branch |
| `cpfr-012-worker` | `work/cpfr-012` | `64b4ba9f758e` | `101 4` | represented in primary CPFR branch |
| `cpfr-013-worker` | `work/cpfr-013` | `2d41cc02858e` | `101 13` | represented in primary CPFR branch |
| `cpfr-014-worker` | `work/cpfr-014` | `9aedcf8c46d8` | `101 13` | represented in primary CPFR branch |
| `cpfr-015-worker` | `work/cpfr-015` | `41710dc1b900` | `101 21` | represented in primary CPFR branch |
| `cpfr-016-worker` | `work/cpfr-016` | `da516c0843ec` | `101 33` | represented in primary CPFR branch |
| `cpfr-019-worker` | `work/cpfr-019` | `2f7cdd2c37c3` | `101 23` | represented in primary CPFR branch |
| `cpfr-020-worker` | `work/cpfr-020` | `e1da4d66ac37` | `101 36` | represented in primary CPFR branch |
| `cpfr-021-worker` | `work/cpfr-021` | `7a8bf3952216` | `101 21` | represented in primary CPFR branch |
| `cpfr-r-evaluator-worker` | `work/cpfr-r-evaluator` | `77cf48e94b3b` | `101 42` | represented in primary CPFR branch |
| `cpfr-r-review-worker` | `work/cpfr-r-review` | `15735f16b507` | `101 41` | represented in primary CPFR branch |

## Detached Verification Worktrees

These worktrees are detached and dirty only in the import receipt and tracker.
They are evidence sources, not direct landing targets.

| Worktree | Head | `master...HEAD` | Dirty paths | Disposition |
|-|-|-|-|-|
| `/private/tmp/harp-cpfr017-verify` | `69145a609d09` | `101 38` | `docs/import-receipt.md`, `tracker.org` | commit represented in primary branch; compare dirty tracker edits only |
| `/private/tmp/harp-cpfr018-closeout-verify` | `270ae1a502ae` | `101 45` | `docs/import-receipt.md`, `tracker.org` | commit represented in primary branch; compare dirty tracker edits only |
| `/private/tmp/harp-cpfr018-findings-verify` | `253ef26e1abc` | `101 39` | `docs/import-receipt.md`, `tracker.org` | commit represented in primary branch; compare dirty tracker edits only |

## Crouzeix RSI / Hermetic Runtime Lane

This lane is required to land as part of the consolidation, but it must remain a
separate integration lane from CPFR proof-reproduction.

### `crouzeix-rsi-workflow-spec`

- Branch: `codex/crouzeix-rsi-workflow-spec`
- Head: `bc88c9221cf431a5c0dd07f3fdc9e525ac56bb09`
- Divergence: `132 2`
- Changed paths:
  - `docs/superpowers/plans/2026-08-15-crouzeix-rsi-hermetic-reproduction.md`
  - `docs/superpowers/specs/2026-08-15-crouzeix-rsi-workflow-design.md`
- Disposition: land docs/spec content after updating assumptions superseded by
  CPFR-070 through CPFR-081. This branch is a docs-only subset or alternate
  copy of the implementation lane's early Crouzeix planning commits; do not land
  it independently if the implementation lane carries the same docs.

### `crouzeix-rsi-implementation`

- Branch: `codex/crouzeix-rsi-implementation`
- Head: `0c0b729edfb608bd9dac88555a29ea8b2928c321`
- Divergence: `101 28`
- Path families touched in branch diff:
  - `.gitattributes`
  - `.gitignore`
  - `crates/harp/src/sources.rs`
  - `crates/harp/tests/cli.rs`
  - `docs/superpowers/**`
  - `evidence/**`
  - `labs/**`
- Observed top commits:
  - `0c0b729 fix: reject Lean declaration modifier escapes`
  - `54dd116 fix: reject Lean attribute escape payloads`
  - `85c825d fix: reject Lean comment escape payloads`
  - `7e557ad feat: seal Lean proof task materialization`
  - `fda8bfc fix: publish runtime without overwrite races`
- Disposition: land all unique relevant work in focused RSI commits. Exclude
  only commits classified as duplicate, obsolete, unsafe, or blocked with
  evidence.
- Initial coherent groups:
  - Crouzeix RSI workflow/spec docs.
  - Capsule manifest and sealing.
  - Lean proof-engineering source/evidence capture.
  - Hermetic TRAE Lean benchmark/verifier docs.
  - Runtime/model/task materialization.
  - Lean escape hardening fixes.
- Initial conflict risks:
  - `.gitignore`
  - `crates/harp/src/sources.rs`
  - `crates/harp/tests/cli.rs`
  - `.gitattributes` PDF/LFS policy for Lean proof-engineering evidence.
- Initial semantic risk: this lane forked before CPFR-070 through CPFR-081, so
  capsule and runtime code must consume or bind to the current FormalTarget and
  v2 formal-receipt interfaces instead of introducing a parallel target or
  receipt authority.

## Quarantined Worktrees

### `crouzeix-rsi-workflow-design`

- Branch: `codex/crouzeix-rsi-workflow-design`
- Head: `530e0edeb1f22eade7888552df7f5106aec48276`
- Head is contained in current `master`.
- Working tree shows broad tracked deletions and hundreds of untracked files.
- Disposition: quarantine in place. Do not land, clean, or delete without a
  separate recovery inventory.
- Additional hazard: read-only audit found the worktree index effectively
  emptied, with 1156 staged deletes, 356 untracked files, and zero tracked index
  entries. Verification from this tree is invalid.

## Owner Decisions

1. Preserve `.teaching/crouzeix-lean-validation/` from
   `crouzeix-formal-validation` and include that teaching packet in the landing
   party.
2. Capture proof-agent amendments `005` through `012` into durable landing
   artifacts, then remove the old raw untracked files from the
   `crouzeix-proof-reproduction` worktree after capture verification.
3. Recover `crouzeix-rsi-workflow-design` into the landing through a separate
   recovery inventory and fresh integration worktree; do not merge directly from
   its corrupt/staged-deletion state.

## Remaining Owner Decision

1. After commit classification, decide whether final cleanup should remove stale
   CPFR worker branch refs or only remove worktrees while keeping branch refs.
