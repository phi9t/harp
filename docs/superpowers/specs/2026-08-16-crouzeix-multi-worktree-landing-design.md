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

- Path: `<harp-root>`
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

- `<harp-root>/.worktrees/crouzeix-formal-validation`
  - Branch: `codex/crouzeix-formal-validation`
  - Status: same commit as `master`
  - Note: preserves untracked `.teaching/crouzeix-lean-validation/...` scratch.
    Owner decision: preserve this teaching packet and include it in the landing
    party as a reviewed artifact rather than deleting it with the worktree.
- `<harp-root>/.worktrees/crouzeix-conjecture-deep-dive`
  - Branch: `integrate/crouzeix-conjecture`
  - Status: contained in `master`
- `<harp-root>/.worktrees/math-foundations-advanced-linear`
  - Branch: `codex/math-foundations-advanced-linear`
  - Status: contained in `master`
- `<harp-root>/.worktrees/math-foundations-formalization-design`
  - Branch: `codex/math-foundations-formalization-design`
  - Status: contained in `master`

Landing action: no code landing. Preserve the `.teaching/` scratch from
`crouzeix-formal-validation` as part of the landing party before retiring the
worktree.

### Main Proof-Reproduction Source

- `<harp-root>/.worktrees/crouzeix-proof-reproduction`
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

- `<harp-root>/.worktrees/crouzeix-retrospective-landing-spec`
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

- `<tmp>/harp-cpfr017-verify`
- `<tmp>/harp-cpfr018-closeout-verify`
- `<tmp>/harp-cpfr018-findings-verify`

Landing action: do not land directly. Extract unique tracker evidence or
findings only after comparing against the primary proof branch. Then retire
with owner approval.

### Crouzeix RSI / Hermetic Proof-Task Lane

- `<harp-root>/.worktrees/crouzeix-rsi-implementation`
  - Branch: `codex/crouzeix-rsi-implementation`
  - Inventory state: `101` behind, `28` ahead
  - Scope: Lean proof-task materialization, hermetic runtime publication,
    evidence, labs, source verifier changes, and docs.
- `<harp-root>/.worktrees/crouzeix-rsi-workflow-spec`
  - Branch: `codex/crouzeix-rsi-workflow-spec`
  - Inventory state: `132` behind, `2` ahead
  - Scope: design/spec docs for Crouzeix hermetic reproduction.

Landing action: land this lane as part of the multi-worktree consolidation.
Keep it separate from CPFR proof-reproduction because it has broader
source/evidence/runtime changes and can fail independently, but do not leave
the RSI work behind merely because it is a separate lane. Individual commits may
be deferred only when they are obsolete, duplicate CPFR-070 through CPFR-081,
or fail review/verification; every deferral must have a recorded blocker.

### Quarantined Worktree

- `<harp-root>/.worktrees/crouzeix-rsi-workflow-design`
  - Branch: `codex/crouzeix-rsi-workflow-design`
  - Status: head contained in `master`, but working tree shows broad deletions
    and hundreds of untracked files.

Landing action: recover this worktree as part of the landing party. It remains
unsafe as a direct merge source, but its local-only state must be inventoried
and any intentional design/proof work must be captured before cleanup. Do not
run verification from this tree; recover by copying reviewed content into a
fresh integration worktree.

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
5. Capture proof-agent amendments `005` through `012` into durable landing
   artifacts, then remove the raw untracked amendment files from the old
   worktree only after their captured content is verified.
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

Purpose: land the Crouzeix RSI/hermetic proof-task work that remains valid
against current `master`, while preserving CPFR-070 through CPFR-081 as the
formal-validation boundary.

Steps:

1. Create a separate fresh integration worktree from current `master`.
2. Inventory the two RSI branches commit by commit.
3. Compare against landed CPFR-070 through CPFR-081 formal-validation
   interfaces.
4. Land docs-only workflow spec first, updating it only to remove assumptions
   superseded by CPFR-070 through CPFR-081.
5. Land runtime/source-verifier work in focused commits when it remains
   relevant and does not duplicate or weaken FormalTarget boundaries.
6. Record any rejected or deferred RSI commit with a reason, verification
   evidence, and future ticket owner.

Expected commit groups:

1. hermetic reproduction design/spec docs;
2. Lean proof-task materialization tests and implementation;
3. source verifier/evidence registration updates;
4. receipt refresh and release gate.

Non-goals:

- No re-opening CPFR-070 through CPFR-081 contracts without a new ticket.
- No fake Lean or provider artifacts.
- No broad adoption of the quarantined workflow-design worktree.
- No silent deferral of RSI work; every excluded commit or artifact must be
  recorded as duplicate, obsolete, unsafe, or blocked.
- No direct merge from `crouzeix-rsi-workflow-design`; recovery happens through
  reviewed content copied into a fresh integration worktree.

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
- the `.teaching/crouzeix-lean-validation/` packet from
  `crouzeix-formal-validation` is preserved or landed as owner-approved
  teaching material;
- proof-agent amendments `005` through `012` are captured into durable landing
  artifacts and removed from the old proof-reproduction worktree after
  verification;
- all unique relevant CPFR proof-reproduction work is either landed or has a
  documented blocker;
- all unique relevant Crouzeix RSI/hermetic-runtime work is landed, with any
  rejected commit explicitly classified as duplicate, obsolete, unsafe, or
  blocked;
- stale detached verification worktrees have no unique unpreserved evidence;
- current `master` is clean;
- `docs/import-receipt.md` matches the tracked payload;
- final verification evidence is recorded; and
- no Crouzeix proof claim is made.

## Open Decisions For Review

1. Final placement for the preserved `.teaching/crouzeix-lean-validation/`
   teaching packet.
2. Exact capture format for proof-agent amendment files `005` through `012`
   before raw-file removal.
3. Whether final cleanup should remove stale CPFR worker branches after their
   commits are classified, or only remove worktrees and keep branches.
