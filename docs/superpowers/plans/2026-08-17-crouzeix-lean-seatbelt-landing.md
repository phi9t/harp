# Crouzeix Lean Seatbelt Landing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the reviewed Lean-suite infrastructure onto current `master` without losing existing formalization work, then prepare CPFR-L007 as the required macOS Seatbelt gate before real Lean validation.

**Architecture:** Use a clean integration branch from `master`, cherry-pick only the reviewed Lean-suite commits, and preserve existing `master` formalization material on conflicts. After landing, implement CPFR-L007 as a separate Seatbelt-backed execution gate using the global `macos-seatbelt-sandbox` skill and the Codex policy-compiler reference.

**Tech Stack:** Git worktrees/cherry-pick, Python `unittest`, Rust/Cargo, `mise`, macOS `/usr/bin/sandbox-exec`, Apple Seatbelt SBPL, existing Harp Crouzeix lab validators.

---

## Invariant Map

- **No full feature-branch merge:** never merge or rebase all of `feat/crouzeix-proof-reproduction` into `master`.
- **Preserve current `master`:** `formalization/mathematical_foundations`, existing Lean evidence, existing scripts, and current Crouzeix formal-validation records win unless the reviewed Lean-suite surface explicitly owns the path.
- **No push:** all landing is local.
- **No real Lean before CPFR-L007:** `formalization/mathematical_foundations` may be inspected but not called hermetic or run as a proof validation until the Seatbelt gate passes.
- **No CPFR-032/033:** this plan does not resume proof search or DGM.
- **No silent sandbox fallback:** if Seatbelt is unavailable, CPFR-L007 reports `blocked`.
- **Untracked primary prompt preserved:** `docs/superpowers/plans/2026-08-16-crouzeix-end-to-end-continuation-prompt.md` in the primary checkout is not deleted or staged by this plan.

## File Map

- Create temporary integration worktree: `.worktrees/crouzeix-lean-suite-integration`.
- Modify through cherry-picks:
  - `docs/superpowers/specs/2026-08-16-lean-hermetic-proof-suite-design.md`
  - `docs/superpowers/plans/2026-08-16-lean-hermetic-proof-suite.md`
  - `docs/superpowers/specs/2026-08-16-crouzeix-lean-seatbelt-landing-design.md`
  - `docs/workstream/crouzeix-proof-reproduction/tracker.org`
  - `labs/crouzeix_proof_reproduction/lean_suite.py`
  - `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`
  - `labs/crouzeix_proof_reproduction/lean_suite_fixtures/`
  - `labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json`
  - `docs/import-receipt.md`
- Future CPFR-L007 implementation files, selected after the integration branch lands:
  - likely `labs/crouzeix_proof_reproduction/seatbelt_runner.py` or a focused section in `lean_suite.py`;
  - likely `labs/crouzeix_proof_reproduction/tests/test_seatbelt_runner.py`;
  - tracker entries and receipt/schema updates as needed.

## Task 1: Create Clean Integration Branch

**Files:**
- No tracked file edits expected.
- Worktree path: `.worktrees/crouzeix-lean-suite-integration`

- [ ] **Step 1: Verify source and target state**

Run from the primary Harp checkout:

```bash
git status --short --branch
git worktree list
git rev-parse master feat/crouzeix-proof-reproduction
```

Expected:

- primary checkout is on `master`;
- only known untracked prompt may be present;
- `feat/crouzeix-proof-reproduction` exists;
- old feature worktree still exists.

- [ ] **Step 2: Create integration branch from current master**

Run:

```bash
git worktree add .worktrees/crouzeix-lean-suite-integration -b integrate/crouzeix-lean-seatbelt master
```

Expected: new worktree created at `.worktrees/crouzeix-lean-suite-integration`.

- [ ] **Step 3: Record baseline refs**

Run from `.worktrees/crouzeix-lean-suite-integration`:

```bash
git rev-parse HEAD
git status --short --branch
test -d formalization/mathematical_foundations
```

Expected: branch is `integrate/crouzeix-lean-seatbelt`, worktree is clean, and `formalization/mathematical_foundations` exists.

## Task 2: Cherry-Pick Reviewed Lean-Suite Commits

**Files:**
- Cherry-picked files listed in the File Map.
- Preserve all current `master` formalization and evidence files unless directly owned by the reviewed Lean-suite commits.

- [ ] **Step 1: Cherry-pick commits in order**

Run from `.worktrees/crouzeix-lean-suite-integration`:

```bash
git cherry-pick \
  750891d \
  8fb1139 \
  5750bc5 \
  d8c7fe7 \
  95556ac \
  b09e782 \
  d363fdb \
  6c6e849 \
  e4a1718 \
  5733c58 \
  795c9dd \
  34978ca \
  bd5e9f6 \
  2f791b7 \
  9d12b5d
```

Expected: either all commits apply, or Git stops at conflicts.

- [ ] **Step 2: Resolve conflicts conservatively**

If conflicts occur:

1. Inspect each conflicted path with `git status --short`.
2. Keep current `master` versions for:
   - `formalization/mathematical_foundations`;
   - existing Lean evidence and scripts;
   - existing formal-validation material;
   - existing source/evidence manifests unless the conflict is only the import receipt digest.
3. Keep cherry-picked versions for:
   - `labs/crouzeix_proof_reproduction/lean_suite.py`;
   - `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`;
   - `labs/crouzeix_proof_reproduction/lean_suite_fixtures/`;
   - `labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json`;
   - Lean-suite design/plan docs.
4. Stage explicit resolved paths.
5. Continue with:

```bash
git cherry-pick --continue
```

Expected: each conflict resolution preserves `master` formalization material and keeps the reviewed Lean-suite surface.

- [ ] **Step 3: Verify no accidental formalization deletion**

Run:

```bash
test -d formalization/mathematical_foundations
find formalization/mathematical_foundations -maxdepth 3 -type f -print | sort
git diff --name-status master..HEAD -- formalization/mathematical_foundations
```

Expected:

- the directory exists;
- files include `MathematicalFoundations.lean`, `lakefile.toml`, `lake-manifest.json`, and `lean-toolchain`;
- the diff is empty unless an explicitly reviewed Lean-suite commit owns the change. Empty is preferred.

## Task 3: Verify Integration Branch

**Files:**
- `docs/import-receipt.md` may need final digest refresh.

- [ ] **Step 1: Run focused Lean-suite checks**

Run:

```bash
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
python3 -m json.tool labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json
python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
git diff --check
```

Expected: all commands exit 0.

- [ ] **Step 2: Refresh repository receipt if needed**

Run:

```bash
cargo run -q -p harp -- repository verify
```

If stale digest is reported, patch only the digest line in `docs/import-receipt.md`, then run again:

```bash
cargo run -q -p harp -- repository verify
```

Expected: `repository verified: 521 import rows`.

- [ ] **Step 3: Run full gate**

Run:

```bash
mise run verify
```

Expected: exits 0. Non-fatal `mise` cache permission warnings may be recorded, but build/test/repository stages must pass.

- [ ] **Step 4: Commit final receipt refresh if needed**

If `docs/import-receipt.md` changed after cherry-picks:

```bash
git add docs/import-receipt.md
git commit -m "docs(crouzeix): refresh lean suite integration receipt" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Expected: focused receipt-only commit, or no commit if receipt already matched.

## Task 4: Fast-Forward Master And Retire Old Worktree

**Files:**
- No tracked file edits expected.

- [ ] **Step 1: Verify fast-forward is possible**

Run from the primary Harp checkout:

```bash
git merge-base --is-ancestor master integrate/crouzeix-lean-seatbelt
git status --short --branch
```

Expected: merge-base command exits 0; primary checkout still has only the known untracked prompt or is clean.

- [ ] **Step 2: Fast-forward local master**

Run:

```bash
git merge --ff-only integrate/crouzeix-lean-seatbelt
```

Expected: local `master` advances to the integration branch. No push.

- [ ] **Step 3: Verify landed master**

Run:

```bash
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
cargo run -q -p harp -- repository verify
test -d formalization/mathematical_foundations
git status --short --branch
```

Expected: tests and repository verify pass; formalization directory exists; primary worktree has only the known untracked prompt or is clean.

- [ ] **Step 4: Remove old feature worktree and branch**

Run:

```bash
git worktree remove .worktrees/crouzeix-proof-reproduction
git branch -d feat/crouzeix-proof-reproduction
```

If worktree removal fails due to permission bits from test fixtures, run:

```bash
chmod -R u+w .worktrees/crouzeix-proof-reproduction
git worktree remove .worktrees/crouzeix-proof-reproduction
```

Expected: old feature worktree removed and local feature branch deleted.

- [ ] **Step 5: Keep or remove integration branch**

If `integrate/crouzeix-lean-seatbelt` is now merged:

```bash
git branch -d integrate/crouzeix-lean-seatbelt
git worktree remove .worktrees/crouzeix-lean-suite-integration
```

Expected: integration branch/worktree removed after master is verified.

## Task 5: Design CPFR-L007 Seatbelt Implementation Plan

**Files:**
- Create or modify future plan only after landing:
  - `docs/superpowers/plans/2026-08-17-crouzeix-seatbelt-lean-execution.md`
- Future code paths to be selected in that plan.

- [ ] **Step 1: Confirm global skill is installed**

Run:

```bash
test -f ~/.agents/skills/macos-seatbelt-sandbox/SKILL.md
sed -n '1,120p' ~/.agents/skills/macos-seatbelt-sandbox/SKILL.md
```

Expected: skill exists and names `/usr/bin/sandbox-exec`, Codex Seatbelt references, and CPFR-L007 guidance.

- [ ] **Step 2: Inspect Codex reference files**

Run from the primary Harp checkout, with the sibling Codex checkout available:

```bash
sed -n '1,260p' ../codex/codex-rs/sandboxing/src/seatbelt.rs
sed -n '1,220p' ../codex/codex-rs/sandboxing/src/seatbelt_base_policy.sbpl
sed -n '1,220p' ../codex/codex-rs/sandboxing/src/seatbelt_network_policy.sbpl
```

Expected: references are available. If the Codex checkout is missing, record a blocker; do not guess.

- [ ] **Step 3: Write CPFR-L007 implementation plan**

The CPFR-L007 plan must include:

- typed policy model;
- `/usr/bin/sandbox-exec` pinning;
- SBPL generation with `-D` parameters;
- read roots and write roots;
- network denial;
- metadata protection for `.git`, `.agents`, `.codex`, `.trae`;
- fake malicious tool tests;
- blocked result when Seatbelt is unavailable;
- real `formalization/mathematical_foundations` run only after fake tests pass.

Expected: CPFR-L007 remains planned, not implemented, until the plan is reviewed.

## Task 6: First Real Lean Validation After CPFR-L007

**Files:**
- No file edits until CPFR-L007 exists.

- [ ] **Step 1: Verify CPFR-L007 is terminal**

Run:

```bash
python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
rg -n "CPFR-L007|Seatbelt|mathematical_foundations" docs/workstream/crouzeix-proof-reproduction/tracker.org
```

Expected: CPFR-L007 is implemented/reviewed and not blocked.

- [ ] **Step 2: Run real Lean under Seatbelt only**

Use the CPFR-L007 command, not the old unsandboxed script directly.

Expected:

- `passed` if Seatbelted Lean/Lake build succeeds and receipts validate;
- `failed` if Lean/Lake executes and proof/build fails;
- `blocked` if toolchain, cache, Seatbelt, or permission setup is unavailable.

## Plan Self-Review

- **Spec coverage:** Tasks 1-4 cover clean landing and old worktree retirement; Task 5 covers CPFR-L007 planning using Codex Seatbelt; Task 6 covers the gated real Lean validation boundary.
- **Scope:** The plan does not merge the full divergent branch, does not run real Lean before CPFR-L007, does not push, and does not resume CPFR-032/033.
- **Conflict policy:** Current `master` formalization material is preserved by default.
- **Type consistency:** CPFR-L003 remains declared-write-surface only; CPFR-L007 owns OS Seatbelt containment.
- **Verification:** Integration branch must pass focused tests, tracker validation, repository verify, and `mise run verify` before `master` moves.
