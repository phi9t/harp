# Crouzeix Worktree Retirement Plan 001

## Claim Ceiling

This document records preservation, cleanup, and recovery decisions for
proof-related worktrees. It is not a proof of Crouzeix's conjecture, not a live
run receipt, and not approval to remove branches or worktrees without the
pre-cleanup checks listed here.

## Owner Decisions

| Decision | Status |
|-|-|
| Preserve `.teaching/crouzeix-lean-validation/` from `crouzeix-formal-validation` | Done: copied into `docs/workstream/crouzeix-proof-reproduction/teaching/crouzeix-lean-validation/` |
| Capture proof-agent amendments `005` through `012` | Done: summarized in `agent-trace-index.md` and `proof-agent-amendment-capture-001.md` |
| Remove raw amendment files after verified capture | Done: removed from old `crouzeix-proof-reproduction` worktree after capture commit |
| Recover `crouzeix-rsi-workflow-design` | Required: use separate recovery inventory and fresh integration worktree |
| Remove stale CPFR worker branch refs | Not approved yet; ask after commit classification |

## Required Preservation

| Source | Content | Capture target | Removal after capture |
|-|-|-|-|
| `crouzeix-formal-validation/.teaching/crouzeix-lean-validation/` | Teaching packet | `docs/workstream/crouzeix-proof-reproduction/teaching/crouzeix-lean-validation/` | Do not remove source worktree until preserved files are landed on `master` |
| `crouzeix-proof-reproduction/proof-agent-amendment-005..012.md` | Amendment evidence | `docs/workstream/crouzeix-proof-reproduction/agent-trace-index.md` and `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-capture-001.md` | Raw amendment files removed from old proof-reproduction worktree after capture commit |

## Approved Removals

No worktree removal is approved yet. The following worktrees are candidates for
retirement after their preservation and final status checks pass:

| Worktree | Branch | Head | Contained in master | Scratch disposition | Proposed action |
|-|-|-|-|-|-|
| `crouzeix-formal-validation` | `codex/crouzeix-formal-validation` | `1d70062ba047` | yes | `.teaching/` captured, but not yet landed on `master` | remove worktree after capture lands on `master` and final check passes |
| `crouzeix-conjecture-deep-dive` | `integrate/crouzeix-conjecture` | `127c0e0474ee` | yes | no known untracked scratch | remove worktree after final check passes |
| `math-foundations-advanced-linear` | `codex/math-foundations-advanced-linear` | `378e004ab01f` | yes | no known untracked scratch | remove worktree after final check passes |
| `math-foundations-formalization-design` | `codex/math-foundations-formalization-design` | `378e004ab01f` | yes | no known untracked scratch | remove worktree after final check passes |

## Branch Ref Removal

Branch ref removal is not approved yet. The CPFR worker branch heads are
represented in `feat/crouzeix-proof-reproduction`, but the branch refs should
remain until the CPFR integration lane is landed and the owner explicitly
approves branch deletion.

Candidate branch refs for later decision:

- `work/cpfr-011`
- `work/cpfr-012`
- `work/cpfr-013`
- `work/cpfr-014`
- `work/cpfr-015`
- `work/cpfr-016`
- `work/cpfr-019`
- `work/cpfr-020`
- `work/cpfr-021`
- `work/cpfr-r-evaluator`
- `work/cpfr-r-review`

## Quarantined

`crouzeix-rsi-workflow-design` remains quarantined for direct merge or
verification. It is included in the landing party only through the recovery path
below: inspect, classify, and copy reviewed recoverable content into a fresh
integration worktree. Do not run verification from the quarantined worktree.

## Recovery Required

| Worktree | Reason | Recovery path |
|-|-|-|
| `crouzeix-rsi-workflow-design` | Head contained in `master`, but local index is effectively emptied with staged deletes and an untracked replacement tree | Write `crouzeix-rsi-workflow-design-recovery-001.md`; copy only reviewed recoverable content into a fresh RSI integration worktree |

## Not Removed

The following are explicitly not removed by this plan:

- `feat/crouzeix-proof-reproduction`
- `codex/crouzeix-rsi-implementation`
- `codex/crouzeix-rsi-workflow-spec`
- `codex/crouzeix-rsi-workflow-design`
- detached `<tmp>/harp-cpfr*` verification worktrees
- any ignored `.runs/` evidence
- any branch ref

## Pre-Cleanup Checks

Before removing any worktree:

```sh
git -C <worktree> status --short --untracked-files=all
git -C <harp-root> merge-base --is-ancestor <head-sha> master
ps -axo pid,ppid,pgid,etime,command | rg '<worktree>|lake|cargo|pnpm|mise'
```

Expected:

- contained worktree heads return ancestor status `0`;
- no process uses the worktree;
- no untracked scratch remains unless it has been deliberately preserved or
  explicitly discarded by the owner.

Before removing raw amendment evidence:

```sh
git -C <planning-worktree> log --oneline -- docs/workstream/crouzeix-proof-reproduction/agent-trace-index.md docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-capture-001.md
git -C <old-proof-worktree> status --short -- docs/workstream/crouzeix-proof-reproduction
```

Expected:

- durable capture commits exist;
- old proof worktree shows only the intended raw amendment deletions under the
  amendment paths.
