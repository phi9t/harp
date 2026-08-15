# Training Dynamics Packet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a first-class, source-bounded Training Dynamics packet and a pinned Lean 4/mathlib companion that machine-checks its deterministic quadratic and momentum theorem spine.

**Architecture:** Keep reader-facing material under `knowledge/training_dynamics/` and Lean sources under the independent root project `formalization/training_dynamics/`. Harp’s Rust corpus compiler owns registration and the Atlas export remains derived; Lean is invoked only through the pinned Lake project. Exact controlled-model results link to compiled Lean declarations, while empirical diagnostics remain prose claims. The delivered packet is self-contained: it contains no local external-checkout name, path, link, or runtime/build/test dependency. Its pinned public Lean/mathlib dependencies are declared in the Lake manifest.

**Tech Stack:** Harp Rust, Markdown/pulldown-cmark, Atlas React/TypeScript/Vitest, Lean 4 `v4.32.1`, mathlib `v4.32.1`, Lake.

---

## File structure

- Create: `knowledge/training_dynamics/{training_dynamics_index,01_quadratic_gradient_descent,02_momentum_and_acceleration,03_stochastic_gradients,04_diagnostics_and_transfer,glossary,source_registry,claim_evidence_ledger}.md` — advanced reader packet.
- Create: `formalization/training_dynamics/{lean-toolchain,lakefile.toml,TrainingDynamics.lean,TrainingDynamics/Quadratic.lean,TrainingDynamics/Momentum.lean,TrainingDynamics/Stochastic.lean,README.md}` — pinned Lean project and proof-bearing modules.
- Create: `scripts/check_training_dynamics_lean.sh` — fail-closed wrapper that runs the packet’s Lake build.
- Modify: `crates/harp/src/corpus/{mod.rs,render.rs,tests.rs}`, `crates/harp/src/search.rs`, `crates/harp/tests/cli.rs` — packet roster, reader route, safe math scope, search, contracts.
- Modify: `atlas/src/{app/AtlasApp.tsx,app/ReaderApp.test.tsx,app/math.test.ts,app/routes.test.ts,content/types.ts,content/canonical.test.ts}` — route type, navigation, reader, math, and fixture coverage.
- Modify: `mise.toml`, `README.md`, `docs/product-contract.md`, `docs/import-receipt.md` — Lean verification task, product boundary, and final receipt.
- Derived: `atlas/src/content/generated/corpus.json`, `atlas/dist/harp-atlas.html`, `atlas/dist/harp-atlas.receipt.json`.

### Task 1: Establish a self-contained source boundary

**Files:**
- Create: all eight documents under `knowledge/training_dynamics/`
- Create: `formalization/training_dynamics/README.md`
- Modify: `crates/harp/src/corpus/tests.rs`

- [ ] **Step 1: Inventory the input concepts without importing artifacts**

Make an internal implementation checklist of the useful concepts needed for the
four modules: quadratic recurrences, momentum state, stochastic-gradient
expectation/noise, and diagnostics. Do not copy prose, source files, captures,
or code into Harp. For each concept, either (a) provide original exposition and
a public primary-source locator when it is an external factual claim, or (b)
state it as a clearly scoped Harp derivation under explicit assumptions.

- [ ] **Step 2: Write the self-containment contract test**

Add a focused corpus test that scans the Training Dynamics packet and Lean
project source files. It must reject absolute local paths (`/Users/`),
`file://` links, and any source-registry or claim-ledger locator that does not
resolve to packet-local prose or a public source identity.

- [ ] **Step 3: Verify the contract begins red**

Run: `cargo test -p harp compiles_the_training_dynamics_route_and_auxiliary_documents --lib`

Expected: FAIL because the packet and proof project do not yet exist.

- [ ] **Step 4: Apply the source boundary while authoring the packet**

Use only Harp-authored explanations, problems, solutions, and Lean code.
Record public primary sources in `source_registry.md`; record only their stable
bibliographic locators in `claim_evidence_ledger.md`. Do not record a local
input, machine path, private URL, or byte digest outside Harp. Define every
term required by the four-module route in the packet itself or link only to
the already-landed Mathematical Foundations packet.

- [ ] **Step 5: Run the self-containment audit**

Run the focused corpus test and:

```sh
rg -n -i '/Users/|file://' knowledge/training_dynamics formalization/training_dynamics scripts/check_training_dynamics_lean.sh
```

Expected: the test passes and the scan reports no matches.

### Task 2: Establish the pinned Lean project

**Files:**
- Create: `formalization/training_dynamics/lean-toolchain`
- Create: `formalization/training_dynamics/lakefile.toml`
- Create: `formalization/training_dynamics/TrainingDynamics.lean`
- Create: `formalization/training_dynamics/README.md`
- Create: `scripts/check_training_dynamics_lean.sh`
- Modify: `mise.toml`

- [ ] **Step 1: Write the missing-toolchain failure test**

Add `crates/harp/tests/training_dynamics_lean.rs` that executes the wrapper with
`PATH` containing no `lake`, and asserts its stderr contains
`Training Dynamics Lean toolchain is unavailable` and its status is nonzero.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p harp --test training_dynamics_lean missing_lake_is_actionable`

Expected: FAIL because the wrapper does not exist.

- [ ] **Step 3: Add the pinned project and wrapper**

Create `lean-toolchain` with exactly:

```text
leanprover/lean4:v4.32.1
```

Create `lakefile.toml` with exactly this dependency declaration:

```toml
name = "training_dynamics"
defaultTargets = ["TrainingDynamics"]

[[require]]
name = "mathlib"
git = "https://github.com/leanprover-community/mathlib4.git"
rev = "v4.32.1"
```

Create `TrainingDynamics.lean`:

```lean
import TrainingDynamics.Quadratic
import TrainingDynamics.Momentum
import TrainingDynamics.Stochastic
```

Create the wrapper with an explicit local project root and no fallback toolchain:

```sh
#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
project="$root/formalization/training_dynamics"
if ! command -v lake >/dev/null 2>&1; then
  echo "Training Dynamics Lean toolchain is unavailable; install elan, then run lake build in $project." >&2
  exit 1
fi
cd "$project"
exec lake build
```

Make it executable. Add `verify-lean` to `mise.toml` with
`run = 'scripts/check_training_dynamics_lean.sh'`, and make `verify` invoke it
after `verify-rust` and before `verify-repository`. The README must state that
`elan` is an external bootstrap prerequisite and that `lean-toolchain` and the
Lake manifest pin all build inputs after bootstrap.

- [ ] **Step 4: Re-run the failure test**

Run: `cargo test -p harp --test training_dynamics_lean missing_lake_is_actionable`

Expected: PASS.

- [ ] **Step 5: Bootstrap and lock dependencies**

Install Lean through elan only after user approval for the download, run
`lake update`, `lake exe cache get`, and commit the generated
`formalization/training_dynamics/lake-manifest.json`. Then run:

```sh
cd formalization/training_dynamics
lake build
```

Expected: `Build completed successfully.`

- [ ] **Step 6: Commit**

```sh
git add formalization/training_dynamics scripts/check_training_dynamics_lean.sh mise.toml crates/harp/tests/training_dynamics_lean.rs
git commit -m "build: add pinned training dynamics Lean project"
```

### Task 3: Formalize deterministic quadratic gradient descent

**Files:**
- Create: `formalization/training_dynamics/TrainingDynamics/Quadratic.lean`
- Modify: `formalization/training_dynamics/TrainingDynamics.lean`
- Modify: `scripts/check_training_dynamics_lean.sh`
- Modify: `crates/harp/tests/training_dynamics_lean.rs`

- [ ] **Step 1: Write failing theorem examples**

Create the module with theorem statements for the error recurrence and the
one-step contraction identity:

```lean
theorem gd_error (a η x xStar : ℝ)
    (update : ℝ → ℝ) (hupdate : ∀ y, update y = y - η * a * (y - xStar)) :
    update x - xStar = (1 - η * a) * (x - xStar) := by
  sorry

theorem gd_energy_step (a η e : ℝ) :
    ((1 - η * a) * e)^2 = (1 - η * a)^2 * e^2 := by
  sorry
```

- [ ] **Step 2: Add and exercise a fail-closed proof-hole policy**

Extend the Lean wrapper to reject a Lean source file containing a `sorry`
token before invoking `lake build`, using only a scoped source-file scan under
`formalization/training_dynamics/`. Add an integration-test case that inserts a
temporary Lean fixture with `sorry`, invokes the wrapper, and asserts a
nonzero, actionable failure. Remove the fixture after the test.

Run: `scripts/check_training_dynamics_lean.sh`

Expected: FAIL while the theorem bodies contain `sorry`, even though Lean can
otherwise admit an axiom-backed declaration.

- [ ] **Step 3: Prove the recurrence algebraically**

Replace the proof holes with:

```lean
  rw [hupdate]
  ring
```

and:

```lean
  ring
```

Add a theorem that converts `0 < η * a` and `η * a < 2` into
`|1 - η * a| < 1` using `abs_lt.2` and `linarith`.

- [ ] **Step 4: Build the theorem module**

Run: `scripts/check_training_dynamics_lean.sh`

Expected: PASS with no declarations containing `sorry`.

- [ ] **Step 5: Commit**

```sh
git add formalization/training_dynamics/TrainingDynamics/Quadratic.lean scripts/check_training_dynamics_lean.sh crates/harp/tests/training_dynamics_lean.rs
git commit -m "feat: formalize quadratic gradient descent dynamics"
```

### Task 4: Formalize the scalar heavy-ball recurrence

**Files:**
- Create: `formalization/training_dynamics/TrainingDynamics/Momentum.lean`
- Test: `formalization/training_dynamics/TrainingDynamics/Momentum.lean`

- [ ] **Step 1: State the two-state update and its error recurrence**

Add failing declarations for scalar heavy-ball state:

```lean
def heavyBall (a η β x previous xStar : ℝ) : ℝ :=
  x - η * a * (x - xStar) + β * (x - previous)

theorem heavyBall_error (a η β x previous xStar : ℝ) :
    heavyBall a η β x previous xStar - xStar =
      (1 - η * a + β) * (x - xStar) - β * (previous - xStar) := by
  sorry
```

- [ ] **Step 2: Verify the theorem is not accepted with proof holes**

Run: `scripts/check_training_dynamics_lean.sh`

Expected: FAIL under the no-`sorry` check.

- [ ] **Step 3: Prove the update recurrence and one zero-momentum reduction**

Replace the theorem proof with `ring`. Add:

```lean
theorem heavyBall_zero_momentum (a η x previous xStar : ℝ) :
    heavyBall a η 0 x previous xStar = x - η * a * (x - xStar) := by
  simp [heavyBall]
```

- [ ] **Step 4: Build the module**

Run: `cd formalization/training_dynamics && lake build TrainingDynamics.Momentum`

Expected: PASS.

- [ ] **Step 5: Commit**

```sh
git add formalization/training_dynamics/TrainingDynamics/Momentum.lean
git commit -m "feat: formalize scalar momentum dynamics"
```

### Task 5: Add a finite-support stochastic-gradient theorem boundary

**Files:**
- Create: `formalization/training_dynamics/TrainingDynamics/Stochastic.lean`
- Test: `formalization/training_dynamics/TrainingDynamics/Stochastic.lean`

- [ ] **Step 1: State a finite two-outcome unbiased estimator theorem**

Use scalar perturbations `δ` and `-δ`, not measure theory:

```lean
def meanTwo (left right : ℝ) : ℝ := (left + right) / 2

theorem symmetric_noise_is_unbiased (gradient δ : ℝ) :
    meanTwo (gradient + δ) (gradient - δ) = gradient := by
  sorry
```

- [ ] **Step 2: Confirm the proof hole is rejected**

Run: `scripts/check_training_dynamics_lean.sh`

Expected: FAIL under the no-`sorry` check.

- [ ] **Step 3: Prove the identity and document its boundary**

Replace the proof with `ring`. Add a module docstring stating that this proves
only a finite-support expectation identity and does not establish convergence
of general stochastic gradient descent.

- [ ] **Step 4: Build the module**

Run: `cd formalization/training_dynamics && lake build TrainingDynamics.Stochastic`

Expected: PASS.

- [ ] **Step 5: Commit**

```sh
git add formalization/training_dynamics/TrainingDynamics/Stochastic.lean
git commit -m "feat: formalize finite stochastic gradient symmetry"
```

### Task 6: Author the advanced reader packet

**Files:**
- Create: all eight documents under `knowledge/training_dynamics/`
- Modify: `knowledge/mathematical_foundations/mathematical_foundations_index.md`

- [ ] **Step 1: Add packet-contract tests**

In `crates/harp/src/corpus/tests.rs`, write a test asserting exactly eight
`training-dynamics-*` document IDs, four modules with six `TD-XX-YY` original
problem headings each, 24 unique solution anchors, no absolute local path or
external-checkout identifier, and exactly one backlink from every non-index
document.

- [ ] **Step 2: Run the contract test to verify it fails**

Run: `cargo test -p harp compiles_the_training_dynamics_route_and_auxiliary_documents --lib`

Expected: FAIL because the packet is absent.

- [ ] **Step 3: Write original packet content**

Write the eight documents described in the design. Make their coverage
explicit and self-sufficient:

- Module 1 derives scalar and spectral quadratic gradient-descent recurrences,
  step-size stability, and conditioning.
- Module 2 derives the scalar heavy-ball two-state recurrence, identifies the
  extra optimizer state, and separates a recurrence identity from a global
  convergence claim.
- Module 3 defines an estimator, its mean, covariance, minibatch averaging,
  and the difference between expectation statements and pathwise outcomes.
- Module 4 records the full executed-update state (parameters, optimizer,
  data/order, batch boundary, schedule, numerical policy, and metric), gives a
  falsifiable one-variable experiment template, and names the deferred
  boundary cases: adaptive methods, feature/parameterization change, coupled
  learners, nonstationarity, proxy transfer, and systems update equivalence.
- The index maps those boundary cases to named future follow-ons and states
  that they are orientation rather than covered theorem/benchmark claims.

Each module must use paired `$...$` and at least one `$$...$$` expression,
unique H3 solution anchors such as `{#td-01-01-solution}`, and a direct link to
the relevant Lean theorem name only after the declaration compiles. Add the
optional Training Dynamics link after Module 6 in the Mathematical Foundations
index.

- [ ] **Step 4: Run the packet contract test**

Run: `cargo test -p harp compiles_the_training_dynamics_route_and_auxiliary_documents --lib`

Expected: PASS.

- [ ] **Step 5: Commit**

```sh
git add knowledge/training_dynamics knowledge/mathematical_foundations/mathematical_foundations_index.md crates/harp/src/corpus/tests.rs
git commit -m "docs: add training dynamics learning packet"
```

### Task 7: Register corpus, search, and safe TeX support

**Files:**
- Modify: `crates/harp/src/corpus/mod.rs`
- Modify: `crates/harp/src/corpus/render.rs`
- Modify: `crates/harp/src/search.rs`
- Modify: `crates/harp/tests/cli.rs`

- [ ] **Step 1: Add failing route/search assertions**

Extend corpus tests to require reader route ID `training-dynamics`, label
`Training dynamics`, the exact eight auxiliary IDs, and math rendering for a
`knowledge/training_dynamics/` fixture. Extend the search CLI fixture with a
Training Dynamics phrase and expected packet path.

- [ ] **Step 2: Run focused tests to verify they fail**

Run:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test cli search_refresh_status_and_query_share_a_digest_receipt
```

Expected: FAIL because the packet is unregistered.

- [ ] **Step 3: Register the packet**

Increase `READER_ROUTES` and `AUXILIARY_DOCUMENTS` array lengths, add the
`training-dynamics` landing path and eight `training-dynamics-*` IDs, add the
search root, and extend the existing mathematics-packet predicate with
`knowledge/training_dynamics/`.

- [ ] **Step 4: Re-run focused tests**

Run the two commands from Step 2.

Expected: PASS.

- [ ] **Step 5: Commit**

```sh
git add crates/harp/src/corpus/mod.rs crates/harp/src/corpus/render.rs crates/harp/src/search.rs crates/harp/src/corpus/tests.rs crates/harp/tests/cli.rs
git commit -m "feat: register training dynamics packet"
```

### Task 8: Add Atlas navigation and math rendering coverage

**Files:**
- Modify: `atlas/src/content/types.ts`
- Modify: `atlas/src/content/canonical.test.ts`
- Modify: `atlas/src/app/AtlasApp.tsx`
- Modify: `atlas/src/app/routes.test.ts`
- Modify: `atlas/src/app/ReaderApp.test.tsx`
- Modify: `atlas/src/app/math.test.ts`

- [ ] **Step 1: Write failing Atlas tests**

Add `training-dynamics` to the valid corpus fixture with a matching synthetic
index document. Add a route parse/format assertion, click the Training dynamics
button in `ReaderApp.test.tsx`, and extend `math.test.ts` to select exactly
eight `knowledge/training_dynamics/` documents, require inline and display
math, MathML parity, and zero `.math-error` elements.

- [ ] **Step 2: Run the focused tests to verify they fail**

Run from `atlas/`:

```sh
corepack pnpm exec vitest run src/app/routes.test.ts src/app/ReaderApp.test.tsx src/app/math.test.ts src/content/canonical.test.ts
```

Expected: FAIL because the route type and navigation are absent.

- [ ] **Step 3: Implement the typed route and navigation**

Add the route ID to `ReaderRouteId` and `readerRouteIds`. Resolve the validated
reader route in `AtlasApp`, throw if it disappears, and add its navigation
button using the same `legacy` route shape as Math foundations and Crouzeix.

- [ ] **Step 4: Re-run focused Atlas tests and type check**

Run from `atlas/`:

```sh
corepack pnpm exec vitest run src/app/routes.test.ts src/app/ReaderApp.test.tsx src/app/math.test.ts src/content/canonical.test.ts
corepack pnpm exec tsc -b
```

Expected: PASS.

- [ ] **Step 5: Commit**

```sh
git add atlas/src/app atlas/src/content/types.ts atlas/src/content/canonical.test.ts
git commit -m "feat: add training dynamics Atlas route"
```

### Task 9: Generate, document, and release-verify

**Files:**
- Modify: `README.md`
- Modify: `docs/product-contract.md`
- Modify: `docs/import-receipt.md`
- Modify: derived Atlas corpus/export files

- [ ] **Step 1: Update product-boundary assertions**

Add the packet to the README and product contract as a local-source-bounded,
advanced route. State the packet’s exact document count, its separation from
the retained-concept/evidence counts, the Lean companion path, and the search
root. Update CLI canonical-document expectations only after the compiled count
is observed.

- [ ] **Step 2: Regenerate canonical outputs**

Run:

```sh
cargo run -p harp -- check
cargo run -p harp -- build
cargo run -p harp -- search refresh
cd atlas && corepack pnpm run test:export
```

Expected: the corpus and single-file Atlas export are rewritten from canonical
inputs, and the export test passes.

- [ ] **Step 3: Refresh the receipt after staging all payload files**

Stage only the named packet, formalization, code, docs, and generated files.
Run `cargo run -p harp -- repository verify`; copy its reported expected SHA-256
into `docs/import-receipt.md`; stage that one file; rerun the verifier until it
reports `repository verified: 521 import rows`.

- [ ] **Step 4: Run release gates**

Run:

```sh
scripts/check_training_dynamics_lean.sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test cli
cd atlas && corepack pnpm run test
cd .. && mise run verify
```

Expected: all gates pass. If the existing parallel process-control suite stalls,
record the interruption and run its affected test(s) serially before declaring
the packet verified.

- [ ] **Step 5: Commit**

```sh
git add README.md docs/product-contract.md docs/import-receipt.md atlas/src/content/generated/corpus.json atlas/dist/harp-atlas.html atlas/dist/harp-atlas.receipt.json crates/harp/tests/cli.rs
git commit -m "docs: publish training dynamics packet"
```

## Self-review

- Spec coverage: Task 1 establishes self-containment; Tasks 2–5 implement the
  pinned Lean companion and theorem boundary; Task 6 supplies the
  eight-document packet and 24 original solved problems; Tasks 7–8 cover
  compiler/search/Atlas integration; Task 9 covers generated outputs, product
  boundaries, receipt, and release gates.
- No placeholders: every task names files, a test, a command, and the exact
  behavior it must establish.
- Consistency: all routes use `training-dynamics`, all reader documents use
  `knowledge/training_dynamics/`, and the Lean project is always
  `formalization/training_dynamics/`.
