# Crouzeix Multi-Worktree Landing Design

## Status

Ready for review.

## Objective

Consolidate all proof-related Harp worktrees into a small number of current,
reviewable landing lanes. The outcome should preserve unique Crouzeix proof
reproduction work, retire worktrees already contained in `master`, quarantine
unsafe or corrupt worktrees, and leave the repository with explicit next
implementation gates.

This is a retrospective integration design. It is not a live proof attempt, not
a provider run, and not a Crouzeix proof claim.

## Agentic Engineering Model

The landing follows the Harp agentic-engineering rule: scale execution, not
authority.

- Intent lives in the CPFR tracker, specs, and owner instructions.
- Reasoning scaffolds include this spec, future plans, and subagent reports.
  They guide execution but are not durable authority by themselves.
- Execution happens in isolated worktrees with bounded file ownership.
- Verification has its own budget: focused tests, tracker validators, review
  agents, repository verification, and `mise run verify`.
- Observability comes from worktree inventories, commit maps, receipts, run
  ledgers, and evidence manifests.
- Human control remains explicit for merge, push, release, destructive cleanup,
  and authority-widening decisions.

Subagents may inventory, compare, implement disjoint patches, and review. They
must not own final integration, branch deletion, destructive cleanup, proof
claims, or release-gate waivers.

## Current Baseline

Primary checkout:

- Path: `/Users/bytedance/workspace/harp`
- Branch: `master`
- Commit: `1d70062ba047d98cc7ea5e4d5b968d5d7d4d8bcf`
- State during inventory: clean except for the untracked continuation prompt
  `docs/superpowers/plans/2026-08-16-crouzeix-end-to-end-continuation-prompt.md`

The already-landed formal-validation lane includes CPFR-070 through CPFR-081
and the focused Lean gate repairs. Those commits are baseline invariants. Any
landing strategy that regresses FormalTarget, formal receipts, Jin/LS
validation, sealed reconstruction, blind-frontier binding, or the two Lean
wrapper gates must stop and repair the regression first.

## Worktree Inventory

### Already Landed Or Contained

These worktrees do not need code landing because their heads are contained in
current `master` or equal to it:

- `/Users/bytedance/workspace/harp/.worktrees/crouzeix-formal-validation`
  - Branch: `codex/crouzeix-formal-validation`
  - Status: same commit as `master`
  - Note: preserves untracked `.teaching/crouzeix-lean-validation/...` scratch.
- `/Users/bytedance/workspace/harp/.worktrees/crouzeix-conjecture-deep-dive`
  - Branch: `integrate/crouzeix-conjecture`
  - Status: contained in `master`
- `/Users/bytedance/workspace/harp/.worktrees/math-foundations-advanced-linear`
  - Branch: `codex/math-foundations-advanced-linear`
  - Status: contained in `master`
- `/Users/bytedance/workspace/harp/.worktrees/math-foundations-formalization-design`
  - Branch: `codex/math-foundations-formalization-design`
  - Status: contained in `master`

Landing action: none. Cleanup action requires explicit owner approval after any
untracked scratch is either preserved or intentionally discarded.

### Main Proof-Reproduction Source

- `/Users/bytedance/workspace/harp/.worktrees/crouzeix-proof-reproduction`
  - Branch: `feat/crouzeix-proof-reproduction`
  - Inventory state: `101` commits behind current `master`, `56` commits ahead
  - Dirty tracked files: `docs/import-receipt.md`,
    `docs/workstream/crouzeix-proof-reproduction/tracker.org`
  - Untracked files: `proof-agent-amendment-005.md` through
    `proof-agent-amendment-012.md`
  - Top committed topics include legacy ticket exceptions, expert-frontier
    preparation, CPFR-032 readiness blockers, and expert-frontier readiness
    repairs.

Landing action: this is the primary Crouzeix proof-reproduction integration
source. Reconcile it onto current `master` in a fresh integration worktree.
Do not merge it directly while it is dirty and behind.

### Retrospective Sidecar

- `/Users/bytedance/workspace/harp/.worktrees/crouzeix-retrospective-landing-spec`
  - Branch: `docs/crouzeix-retrospective-landing-spec`
  - Head: same as `feat/crouzeix-proof-reproduction`
  - Untracked file:
    `docs/superpowers/specs/2026-08-16-crouzeix-multi-worktree-retrospective-landing-design.md`

Landing action: treat as a design sidecar. Reuse useful ideas in this spec and
later retrospective docs, but do not land this branch as an independent code
lineage.

### CPFR Worker Branches

The CPFR worker worktrees are tracked-clean but all predate the current master
by 101 commits. They appear to be incremental contributors to the older
`feat/crouzeix-proof-reproduction` stack:

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

Landing action: do not land these one by one. Use them as evidence sources for
a commit map. A commit or patch from one of these branches may be landed only if
it is absent from both current `master` and the primary
`feat/crouzeix-proof-reproduction` branch, still relevant, and independently
verifiable.

### Detached Verification Worktrees

These detached worktrees are dirty only in `docs/import-receipt.md` and the
Crouzeix tracker:

- `/private/tmp/harp-cpfr017-verify`
- `/private/tmp/harp-cpfr018-closeout-verify`
- `/private/tmp/harp-cpfr018-findings-verify`

Landing action: do not land directly. Extract unique tracker evidence or
findings only after comparing against the primary proof branch. Then retire
with owner approval.

### Crouzeix RSI / Hermetic Proof-Task Lane

- `/Users/bytedance/workspace/harp/.worktrees/crouzeix-rsi-implementation`
  - Branch: `codex/crouzeix-rsi-implementation`
  - Inventory state: `101` behind, `28` ahead
  - Scope: Lean proof-task materialization, hermetic runtime publication,
    evidence, labs, source verifier changes, and docs.
- `/Users/bytedance/workspace/harp/.worktrees/crouzeix-rsi-workflow-spec`
  - Branch: `codex/crouzeix-rsi-workflow-spec`
  - Inventory state: `132` behind, `2` ahead
  - Scope: design/spec docs for Crouzeix hermetic reproduction.

Landing action: keep this as a separate integration lane from CPFR
proof-reproduction. It overlaps conceptually with Lean/formal infrastructure
but has broader source/evidence/runtime changes. It requires its own commit map,
spec review, and verification.

### Quarantined Worktree

- `/Users/bytedance/workspace/harp/.worktrees/crouzeix-rsi-workflow-design`
  - Branch: `codex/crouzeix-rsi-workflow-design`
  - Status: head contained in `master`, but working tree shows broad deletions
    and hundreds of untracked files.

Landing action: quarantine. Do not land or clean it in the main consolidation.
Only a separate recovery task should classify whether it contains intentional
scratch that needs preservation.

## Landing Strategy

### Lane 1: Retire Already-Contained Worktrees

Purpose: remove noise without losing work.

Steps:

1. Reconfirm head containment and status.
2. Inventory untracked files.
3. Ask the owner whether to preserve or discard untracked scratch.
4. Remove only approved worktrees.

Verification:

- `git merge-base --is-ancestor <worktree-head> master`
- `git status --short --untracked-files=all`
- No branch deletion or worktree removal without explicit approval.

### Lane 2: CPFR Proof-Reproduction Integration

Purpose: salvage and land the coherent CPFR proof-reproduction stack.

Steps:

1. Create a fresh integration worktree from current `master`.
2. Build a commit map across:
   - `feat/crouzeix-proof-reproduction`
   - CPFR worker branches
   - detached CPFR verification worktrees
   - the retrospective sidecar
3. Classify each commit or artifact:
   - already in `master`
   - represented by primary proof branch
   - unique and still relevant
   - obsolete
   - unsafe or unverifiable
4. Apply only unique, relevant changes as focused commits by concern.
5. Preserve proof-agent amendments as reachable docs only if they pass review
   as durable evidence rather than raw trace dumps.
6. Refresh `docs/import-receipt.md` last.
7. Run focused validators and the strongest practical release gate.

Expected commit groups:

1. legacy-ticket exception and validation repair, if still absent from master;
2. expert-frontier preparation and readiness blocker record, if still useful;
3. proof-agent amendment/trace index/retrospective docs;
4. CPFR-R014 repair-gate specification;
5. receipt refresh and verification evidence.

Non-goals:

- No live provider calls.
- No CPFR-033 or downstream live frontier execution.
- No mutation of ignored `.runs/` evidence.
- No direct merge of a dirty branch.

### Lane 3: Crouzeix RSI / Hermetic Runtime Integration

Purpose: decide whether the hermetic proof-task runtime belongs in the current
Harp proof infrastructure.

Steps:

1. Create a separate fresh integration worktree from current `master`.
2. Inventory the two RSI branches commit by commit.
3. Compare against landed CPFR-070 through CPFR-081 formal-validation
   interfaces.
4. Land docs-only workflow spec first if still accurate.
5. Land runtime/source-verifier work only if it is still relevant and does not
   duplicate or weaken FormalTarget boundaries.

Expected commit groups:

1. hermetic reproduction design/spec docs;
2. Lean proof-task materialization tests and implementation;
3. source verifier/evidence registration updates;
4. receipt refresh and release gate.

Non-goals:

- No re-opening CPFR-070 through CPFR-081 contracts without a new ticket.
- No fake Lean or provider artifacts.
- No broad adoption of the quarantined workflow-design worktree.

## Subagent Use

Every lane must use subagents, but the main agent owns integration.

Recommended subagents:

- Inventory subagent: read-only, builds commit/artifact map for one lane.
- Spec-review subagent: checks tracker and design compliance.
- Standards-review subagent: checks path safety, receipt discipline,
  verification adequacy, and proof-claim ceilings.
- Implementation subagents: only for disjoint write sets, never for the main
  integration branch choreography.

Subagents must not stage, commit, delete worktrees, rebase branches, or clean
untracked scratch unless explicitly assigned after owner approval.

## Verification Gates

For every integration lane:

- `git status --short --branch --untracked-files=all`
- `git diff --check`
- explicit staged-path audit
- tracker validator when `tracker.org` changes
- focused Python/Rust/Lean tests for touched code
- `cargo run -q -p harp -- repository verify` after receipt refresh

Before landing a lane to `master`:

- run `mise run verify`, unless the owner explicitly accepts a narrower gate;
- ensure every new commit has exactly one
  `Co-authored-by: TRAE CLI <noreply@bytedance.com>` trailer;
- fast-forward local `master` only after the gate passes;
- do not push.

## Acceptance Criteria

The multi-worktree landing is complete when:

- every proof-related worktree is classified as landed, retired, integrated,
  deferred, or quarantined;
- all unique relevant CPFR proof-reproduction work is either landed or has a
  documented blocker;
- all unique relevant Crouzeix RSI/hermetic-runtime work is either landed or
  explicitly deferred;
- stale detached verification worktrees have no unique unpreserved evidence;
- current `master` is clean;
- `docs/import-receipt.md` matches the tracked payload;
- final verification evidence is recorded; and
- no Crouzeix proof claim is made.

## Open Decisions For Review

1. Whether to preserve the untracked `.teaching/` scratch from
   `crouzeix-formal-validation` before retiring that worktree.
2. Whether proof-agent amendment files `005` through `012` are durable evidence
   to land, or local execution notes to summarize and leave untracked.
3. Whether the Crouzeix RSI implementation lane is part of the current CPFR
   proof-reproduction consolidation or a separate future workstream.
4. Whether final cleanup should remove stale CPFR worker branches after their
   commits are classified, or only remove worktrees and keep branches.
