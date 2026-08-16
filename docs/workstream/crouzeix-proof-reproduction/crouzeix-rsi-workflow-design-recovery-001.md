# Crouzeix RSI Workflow Design Recovery 001

## Claim Ceiling

This document classifies the corrupt `crouzeix-rsi-workflow-design` worktree.
It is not a proof claim, not a Lean verification result, and not a source of
runtime authority.

## Worktree State

- Worktree: `<harp-root>/.worktrees/crouzeix-rsi-workflow-design`
- Branch: `codex/crouzeix-rsi-workflow-design`
- Head: `530e0edeb1f22eade7888552df7f5106aec48276`
- Relationship to current `master`: head is already contained in `master`
- Staged deletes reported by read-only inventory: `1156`
- Untracked files reported by read-only inventory: `356`
- Present untracked files are reappeared paths from the staged-delete set.
- Verification from this worktree is invalid because the index and working tree
  do not represent a coherent checkout.

## Duplicate Content

The clean branch `codex/crouzeix-rsi-workflow-spec` preserves the intended
workflow-design content as two reviewable commits:

- `99e376a docs: design Crouzeix RSI workflow`
- `bc88c92 docs: plan Crouzeix hermetic reproduction`

The broader clean branch `codex/crouzeix-rsi-implementation` carries the
workflow/spec lineage plus the implementation lane that must still land.

Read-only comparison found that the corrupt worktree's physical untracked files
match the `codex/crouzeix-rsi-workflow-design` branch content and are not a
better recovery source than the clean RSI branches.

## Recoverable Content

No content should be copied directly from the corrupt worktree during the main
landing.

Recoverable RSI work is instead sourced from:

| Source branch | Recovery role |
|-|-|
| `codex/crouzeix-rsi-workflow-spec` | Clean docs/spec source for the Crouzeix RSI workflow design and hermetic reproduction plan |
| `codex/crouzeix-rsi-implementation` | Required RSI/hermetic-runtime implementation lane, including capsule work, Lean proof-engineering evidence, benchmark harness work, and source verifier updates |

## Rejected Content

| Path or group | Classification | Reason |
|-|-|-|
| All staged deletes in `crouzeix-rsi-workflow-design` | obsolete | The branch head is already contained in `master`; staged deletes are an incoherent local index state, not desired deletion work. |
| All untracked reappeared files in `crouzeix-rsi-workflow-design` | duplicate | The files are represented by the branch content and clean RSI branches; direct copying from the corrupt checkout is unnecessary and riskier. |
| Generated Atlas outputs in `crouzeix-rsi-workflow-design` | unsafe | Derived artifacts must regenerate from canonical inputs and should not be recovered from a corrupt checkout. |
| Repository root files in `crouzeix-rsi-workflow-design` | duplicate or obsolete | Current `master` and clean RSI branches are better authority for these files. |

## Recovery Actions

1. Keep `crouzeix-rsi-workflow-design` quarantined.
2. Do not merge, reset, clean, verify from, or copy broad trees from that
   worktree.
3. Land RSI workflow-design documents from `codex/crouzeix-rsi-workflow-spec`
   or from the matching commits in `codex/crouzeix-rsi-implementation`.
4. Land RSI implementation work from `codex/crouzeix-rsi-implementation` in
   focused, verified commits.
5. After the RSI lane lands, ask the owner before removing the corrupt worktree
   or deleting its branch ref.

## Verification Evidence

Read-only recovery audits used:

- `git status --short --branch`
- `git diff --cached --stat`
- `git ls-files --others --exclude-standard`
- `git branch --contains`
- `git diff --stat master...<branch>`
- blob equality comparisons against `master`,
  `codex/crouzeix-rsi-workflow-spec`, and
  `codex/crouzeix-rsi-implementation`

No files were recovered directly from `crouzeix-rsi-workflow-design`.
