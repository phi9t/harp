# Crouzeix Multi-Worktree Landing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Consolidate proof-related Harp worktrees into current, reviewable landing lanes without losing unique Crouzeix work or making proof claims.

**Architecture:** The work is split into inventory, retirement, CPFR proof-reproduction integration, Crouzeix RSI/hermetic-runtime integration, and final landing. Subagents perform read-only inventories and bounded reviews; the main agent owns integration, staging, receipt refresh, verification, and any local fast-forward.

**Tech Stack:** Git worktrees, shell, Python 3 standard library, existing Harp CPFR Python tools, Rust/Cargo, Lean wrapper scripts, `mise run verify`.

---

## File Structure

- Create `docs/workstream/crouzeix-proof-reproduction/worktree-inventory-001.md`: durable inventory of proof-related worktrees, commit maps, dirty state, and disposition.
- Create `docs/workstream/crouzeix-proof-reproduction/retrospective-001.md`: retrospective over the proof-reproduction lineage and next repair gate.
- Optional create `docs/workstream/crouzeix-proof-reproduction/agent-trace-index.md`: only if proof-agent amendment files are accepted as durable evidence.
- Optional create `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-005.md` through `proof-agent-amendment-012.md`: only after review decides they are durable evidence, not raw notes.
- Modify `docs/workstream/crouzeix-proof-reproduction/tracker.org`: record worktree disposition, CPFR-R014 repair gate, and any terminal/canceled state updates required by the accepted lane.
- Modify `docs/import-receipt.md`: refresh the payload digest last for each landed lane.
- Modify or create RSI/hermetic proof-task files in a separate required lane
  after Lane 2 is settled. Individual RSI commits may be excluded only with an
  explicit duplicate, obsolete, unsafe, or blocked classification.

## Invariant Map

- Intent authority: `tracker.org`, approved specs, and owner instructions. Worktree branch names and subagent reports are evidence, not intent authority.
- Execution boundary: every integration lane runs in a fresh worktree based on current `master`. Existing stale worktrees are read-only evidence sources until a specific patch is selected.
- Verification boundary: focused validators and `mise run verify` decide whether a lane can land. A subagent review or successful focused test cannot waive the release gate.
- Evidence boundary: ignored `.runs/` content, raw traces, and proof-agent amendments are not durable project guidance until summarized, reviewed, and committed.
- Human-control boundary: branch deletion, worktree removal, scratch deletion, push, release, and proof-claim escalation require explicit owner approval.
- CPFR-070..081 preservation: FormalTarget, formal receipts, Jin/LS validation, sealed reconstruction, blind-frontier binding, and Lean wrapper gates must remain passing after any lane.

### Task 1: Build Read-Only Commit And Worktree Inventory

**Files:**
- Create: `docs/workstream/crouzeix-proof-reproduction/worktree-inventory-001.md`

- [ ] **Step 1: Spawn read-only inventory subagents**

Run three subagents with non-overlapping scopes:

```text
Subagent A: inspect /Users/bytedance/workspace/harp/.worktrees/crouzeix-proof-reproduction, the CPFR worker branches, and detached /private/tmp/harp-cpfr* worktrees. Read only. Report commit maps, dirty files, unique artifacts, and likely duplicate commits.

Subagent B: inspect /Users/bytedance/workspace/harp/.worktrees/crouzeix-rsi-implementation and /Users/bytedance/workspace/harp/.worktrees/crouzeix-rsi-workflow-spec. Read only. Compare their scope to landed CPFR-070..081 and report unique files, risks, and verification commands.

Subagent C: inspect already-contained and quarantined worktrees: crouzeix-formal-validation, crouzeix-conjecture-deep-dive, math-foundations-advanced-linear, math-foundations-formalization-design, crouzeix-rsi-workflow-design. Read only. Report untracked scratch and safe/unsafe retirement recommendations.
```

Expected: subagents do not edit, stage, delete, rebase, or commit.

- [ ] **Step 2: Generate the inventory skeleton**

Create `docs/workstream/crouzeix-proof-reproduction/worktree-inventory-001.md` with this structure:

```markdown
# Crouzeix Proof Worktree Inventory 001

## Baseline

- Primary checkout: `/Users/bytedance/workspace/harp`
- Baseline commit: `$(git -C /Users/bytedance/workspace/harp rev-parse HEAD)`
- Inventory date: `YYYY-MM-DD`
- No proof claim: this document classifies worktree state only.

## Disposition Summary

| Worktree | Branch | Head | master...HEAD | Status | Disposition |
|-|-|-|-|-|-|

## Already Contained

## Primary CPFR Proof-Reproduction Lane

## CPFR Worker Branches

## Detached Verification Worktrees

## Crouzeix RSI / Hermetic Runtime Lane

## Quarantined Worktrees

## Open Owner Decisions
```

- [ ] **Step 3: Fill the inventory from command evidence**

Run:

```sh
git worktree list --porcelain
```

For every proof-related worktree, record:

```sh
git -C <worktree> status --short --branch --untracked-files=all
git -C <worktree> rev-list --left-right --count master...HEAD
git -C <worktree> log --oneline --max-count=8 master..HEAD
git -C <worktree> diff --name-status master...HEAD
```

Expected: no mutation; `worktree-inventory-001.md` contains exact branch/head/disposition data.

- [ ] **Step 4: Cross-check duplicate and contained branches**

Run:

```sh
git -C /Users/bytedance/workspace/harp branch --contains <head-sha>
git -C /Users/bytedance/workspace/harp cherry -v master <branch>
git -C /Users/bytedance/workspace/harp range-diff master...<branch> master...feat/crouzeix-proof-reproduction
```

Use `range-diff` only for branches intended to compare against the main proof branch. Record whether each worker branch is represented by the main branch, unique, obsolete, or unknown.

- [ ] **Step 5: Verify and commit the inventory**

Run:

```sh
git diff --check -- docs/workstream/crouzeix-proof-reproduction/worktree-inventory-001.md
```

Then commit:

```sh
git add docs/workstream/crouzeix-proof-reproduction/worktree-inventory-001.md
git commit -m "docs(crouzeix): inventory proof worktrees" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 2: Decide Retirement And Quarantine Actions

**Files:**
- Modify: `docs/workstream/crouzeix-proof-reproduction/worktree-inventory-001.md`
- Optional create: `docs/workstream/crouzeix-proof-reproduction/worktree-retirement-plan-001.md`

- [ ] **Step 1: Ask for owner decisions on scratch preservation**

Ask for explicit decisions on:

```text
1. Preserve or discard `.teaching/` from `crouzeix-formal-validation`?
2. Preserve or discard untracked files from contained math/Crouzeix worktrees, if any?
3. Remove only worktrees, or remove branches too after containment is proven?
4. Keep `crouzeix-rsi-workflow-design` quarantined in place, or create a separate recovery inventory?
```

Expected: no cleanup until owner answers.

- [ ] **Step 2: Record approved retirement plan**

If any cleanup is approved, create or update:

```markdown
# Crouzeix Worktree Retirement Plan 001

## Approved Removals

| Worktree | Branch | Head | Contained in master | Scratch disposition | Action |
|-|-|-|-|-|-|

## Quarantined

## Not Removed
```

- [ ] **Step 3: Run pre-cleanup safety checks**

For each approved worktree:

```sh
git -C <worktree> status --short --untracked-files=all
git -C /Users/bytedance/workspace/harp merge-base --is-ancestor <head-sha> master
ps -axo pid,ppid,pgid,etime,command | rg '<worktree>|lake|cargo|pnpm|mise'
```

Expected: contained branches show ancestor status 0; no process uses the worktree.

- [ ] **Step 4: Commit the retirement plan before deletion**

Run:

```sh
git add docs/workstream/crouzeix-proof-reproduction/worktree-inventory-001.md docs/workstream/crouzeix-proof-reproduction/worktree-retirement-plan-001.md
git commit -m "docs(crouzeix): plan proof worktree retirement" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 3: Reconcile CPFR Proof-Reproduction Lane

**Files:**
- Modify or create files selected from:
  - `docs/workstream/crouzeix-proof-reproduction/tracker.org`
  - `docs/workstream/crouzeix-proof-reproduction/design.md`
  - `docs/workstream/crouzeix-proof-reproduction/retrospective-001.md`
  - `docs/workstream/crouzeix-proof-reproduction/agent-trace-index.md`
  - `docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-005.md` through `proof-agent-amendment-012.md`
  - `labs/crouzeix_proof_reproduction/**`
  - `docs/import-receipt.md`

- [ ] **Step 1: Create fresh CPFR integration worktree**

Run from primary checkout:

```sh
git worktree add .worktrees/crouzeix-proof-reproduction-integration -b integrate/crouzeix-proof-reproduction-current master
```

Expected: new branch starts at current `master`.

- [ ] **Step 2: Spawn CPFR lane audit subagents**

Subagents:

```text
Audit 1: compare feat/crouzeix-proof-reproduction against master. Identify coherent commit groups and conflicts with CPFR-070..081.

Audit 2: compare CPFR worker branches against feat/crouzeix-proof-reproduction. Identify unique commits absent from the main branch.

Audit 3: inspect proof-agent amendment files 005..012 and detached verification worktrees. Decide which content is durable evidence, which should be summarized, and which should remain local.
```

Expected: read-only reports with path lists and recommended commit groups.

- [ ] **Step 3: Build the CPFR commit map**

Create a local scratch file outside the repo or a temporary ignored file. Record:

```text
commit_sha | source_branch | topic | represented_in_main_branch | represented_in_master | action
```

Do not commit the scratch map unless it becomes the polished inventory document.

- [ ] **Step 4: Apply the first coherent CPFR patch group**

Use cherry-pick or manual patch application only for one coherent group at a time. Start with the smallest group that unblocks future proof-reproduction state, usually legacy ticket exception or CPFR-031 readiness records.

Run before applying:

```sh
git -C .worktrees/crouzeix-proof-reproduction-integration status --short --branch
```

Then apply:

```sh
git -C .worktrees/crouzeix-proof-reproduction-integration cherry-pick <commit-sha>
```

If conflicts occur, resolve by preserving current `master` CPFR-070..081 contracts and manually porting only the intended older behavior.

- [ ] **Step 5: Run focused tests for the patch group**

Choose commands based on touched files. For tracker-only patches:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
git diff --check
```

For lab code patches:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v
git diff --check
```

Expected: focused checks pass before committing.

- [ ] **Step 6: Commit each CPFR patch group**

```sh
git add <explicit paths>
git commit -m "feat(crouzeix): <specific CPFR concern>" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Repeat Steps 4 through 6 for every accepted CPFR group.

- [ ] **Step 7: Write the retrospective and next repair gate**

Create `docs/workstream/crouzeix-proof-reproduction/retrospective-001.md` with:

```markdown
# Crouzeix Proof Reproduction Retrospective 001

## Claim Ceiling

No Crouzeix proof is claimed. This retrospective records control-plane,
provider, review, and evidence state.

## Timeline

## Worktree Evidence

## Landed CPFR Work

## Deferred Or Quarantined Work

## Current Live Frontier State

## CPFR-R014 Repair Gate

## Verification Evidence
```

If proof-agent amendments are accepted, create `agent-trace-index.md` and commit the reviewed amendment docs. If not, summarize their durable conclusions in the retrospective and leave raw amendment files untracked.

- [ ] **Step 8: Refresh receipt and run CPFR lane gate**

Run:

```sh
cargo run -q -p harp -- repository verify
```

If it reports a stale digest, update only the digest line in `docs/import-receipt.md`, then rerun:

```sh
cargo run -q -p harp -- repository verify
PYTHONDONTWRITEBYTECODE=1 python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
git diff --check
```

Commit:

```sh
git add docs/import-receipt.md docs/workstream/crouzeix-proof-reproduction/retrospective-001.md docs/workstream/crouzeix-proof-reproduction/agent-trace-index.md docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "docs(crouzeix): record proof reproduction retrospective" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 4: Land Crouzeix RSI / Hermetic Runtime Lane

**Files:**
- Potentially create or modify:
  - `docs/superpowers/specs/2026-08-15-crouzeix-rsi-workflow-design.md`
  - `docs/superpowers/plans/2026-08-15-crouzeix-rsi-hermetic-reproduction.md`
  - `labs/**`
  - `evidence/**`
  - `crates/harp/src/sources.rs`
  - `crates/harp/tests/cli.rs`
  - `docs/import-receipt.md`

- [ ] **Step 1: Create fresh RSI integration worktree**

Run:

```sh
git worktree add .worktrees/crouzeix-rsi-integration -b integrate/crouzeix-rsi-current master
```

- [ ] **Step 2: Spawn RSI audit subagents**

Subagents:

```text
Audit 1: inspect codex/crouzeix-rsi-workflow-spec. Determine whether the two docs are still accurate after CPFR-070..081.

Audit 2: inspect codex/crouzeix-rsi-implementation. Identify unique commits, touched paths, verification commands, overlap with FormalTarget, and risks.

Audit 3: inspect evidence/source verifier changes for rights, LFS, source-count, and repository-verify impacts.
```

- [ ] **Step 3: Land the docs-only RSI spec**

Cherry-pick or manually copy the docs/spec files first. If the docs contain
assumptions superseded by CPFR-070 through CPFR-081, patch those assumptions in
the integration worktree rather than silently dropping the docs.

Verify:

```sh
git diff --check
cargo run -q -p harp -- repository verify
```

Refresh receipt if needed and commit:

```sh
git add docs/superpowers/specs/2026-08-15-crouzeix-rsi-workflow-design.md docs/superpowers/plans/2026-08-15-crouzeix-rsi-hermetic-reproduction.md docs/import-receipt.md
git commit -m "docs(crouzeix): preserve hermetic RSI workflow design" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

- [ ] **Step 4: Split the runtime implementation into required review groups**

Build a commit map for `codex/crouzeix-rsi-implementation` and classify every
ahead commit. Land all unique relevant work. Exclude a commit only when it is:
duplicate of current `master`, obsolete after CPFR-070 through CPFR-081, unsafe,
or blocked by a concrete verification failure.

Split accepted work by concern:

1. Lean task materialization tests and code.
2. Runtime publication hardening.
3. Evidence/source verifier registration.
4. Receipt refresh.

Run focused tests after each group and `mise run verify` before landing to `master`.

- [ ] **Step 5: Record every excluded RSI commit**

For every rejected or deferred RSI commit, add a row to
`docs/workstream/crouzeix-proof-reproduction/worktree-inventory-001.md`:

```markdown
| Commit | Source branch | Classification | Reason | Future owner |
|-|-|-|-|-|
```

Allowed classifications are `duplicate`, `obsolete`, `unsafe`, and `blocked`.
Do not use vague classifications such as `later` or `maybe`.

- [ ] **Step 6: Verify and commit the RSI lane**

Run:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v
cargo test -p harp --test cli
cargo run -q -p harp -- repository verify
git diff --check
```

If `repository verify` reports a stale digest, update only the digest line in
`docs/import-receipt.md`, then rerun it.

Commit accepted groups with focused messages, for example:

```sh
git add <explicit RSI paths>
git commit -m "feat(crouzeix): land hermetic RSI proof task runtime" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 5: Final Review And Local Landing

**Files:**
- Modify: `docs/import-receipt.md`
- Modify: tracker and inventory docs if final status changes

- [ ] **Step 1: Spawn final review subagents**

Subagents:

```text
Spec reviewer: compare landed integration branches against the approved multi-worktree landing design.

Standards reviewer: inspect diffs for path safety, stale receipt risk, proof overclaiming, broad generated churn, and unrelated changes.

Verification reviewer: run read-only status, diff, and command-summary checks; do not edit or stage.
```

- [ ] **Step 2: Run full release gate for each integration lane**

For each lane ready to land:

```sh
mise run verify
git status --short --branch --untracked-files=all
git diff --check
```

Expected: release gate exits 0; worktree has only intentional tracked changes already committed and approved local scratch.

- [ ] **Step 3: Fast-forward master one lane at a time**

From primary checkout:

```sh
git status --short --branch --untracked-files=all
git merge-base --is-ancestor master <integration-branch>
git merge --ff-only <integration-branch>
```

Expected: primary `master` advances by fast-forward only.

- [ ] **Step 4: Post-landing status audit**

Run:

```sh
git status --short --branch --untracked-files=all
git rev-list --left-right --count master...<integration-branch>
git log --oneline --decorate --max-count=12
```

Expected: `master` clean except owner-approved untracked scratch; landed branch has `0 0` divergence.

- [ ] **Step 5: Do not clean up worktrees until approved**

Prepare a cleanup proposal listing:

```text
worktree | branch | contained_in_master | untracked_scratch | proposed_action
```

Ask owner for explicit approval before any `git worktree remove`, branch delete, or scratch deletion.

## Plan Self-Review

- Spec coverage: covers all worktree groups, lane separation, required RSI landing, subagent use, verification gates, no-push policy, quarantine, and no-proof-claim boundary.
- Placeholder scan: no placeholder markers; optional steps are guarded by explicit owner decisions or audit outcomes.
- Scope check: CPFR proof-reproduction and Crouzeix RSI are intentionally separate required lanes to avoid one oversized merge while still landing RSI work.
- Authority check: subagents inventory and review; main agent owns integration and owner approves destructive cleanup.
