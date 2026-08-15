# Mathematical Foundations Formalization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a self-contained, pinned Lean companion and theorem/problem map for all six original Mathematical Foundations modules.

**Architecture:** `formalization/mathematical_foundations/` is an independent Lake project with six small namespaces imported only after each module is complete. The Markdown formalization map is the sole reader-facing contract for which of the 48 original problems are direct theorems, applications, or prose-only; it links only to compiled declarations.

**Tech Stack:** Lean 4 and pinned mathlib, Lake, Harp Rust corpus tests, Markdown/pulldown-cmark, Atlas derived corpus/export.

---

## File structure

- Create: `formalization/mathematical_foundations/{lean-toolchain,lakefile.toml,lake-manifest.json,MathematicalFoundations.lean,README.md}`.
- Create: `formalization/mathematical_foundations/MathematicalFoundations/{Linear,Orthogonality,Probability,BayesInformation,LinearModels,Optimization}.lean`.
- Create: `scripts/check_mathematical_foundations_lean.sh` and `crates/harp/tests/mathematical_foundations_lean.rs`.
- Create: `knowledge/mathematical_foundations/formalization_map.md`.
- Modify: `.gitignore`, `mise.toml`, `crates/harp/src/corpus/{mod.rs,tests.rs}`, `crates/harp/tests/cli.rs`, `README.md`, `docs/product-contract.md`, `docs/import-receipt.md`.
- Derived: `atlas/src/content/generated/corpus.json`, `atlas/dist/harp-atlas.html`, `atlas/dist/harp-atlas.receipt.json`.

### Task 1: Establish the isolated, pinned Lean project

**Files:** project root files, wrapper, Rust integration test, `.gitignore`, `mise.toml`.

- [ ] **Step 1: Write red wrapper tests**

Create `crates/harp/tests/mathematical_foundations_lean.rs`. Test that the wrapper with a PATH containing no `lake` exits nonzero and writes `Mathematical Foundations Lean toolchain is unavailable`. Add a second test that supplies a temporary project via exact `--project-for-test <directory>` arguments, places a `.lean` source containing `sorry`, and asserts rejection before a fake `lake` executable can run. Add a third test proving an ambient project-root environment variable is ignored by normal invocation.

- [ ] **Step 2: Run the red test**

Run: `cargo test -p harp --test mathematical_foundations_lean -- --test-threads=1`

Expected: FAIL because the wrapper and project do not exist.

- [ ] **Step 3: Add the project and wrapper**

Set `lean-toolchain` to `leanprover/lean4:v4.32.1`. Set `lakefile.toml` project name to `mathematical_foundations`, default target to `MathematicalFoundations`, add `[[lean_lib]] name = "MathematicalFoundations"`, and require mathlib at public tag `v4.32.1`. The root module initially imports no unfinished namespace. The wrapper defaults unconditionally to its script-relative project root, supports only the exact test-only argument shape, prunes `.lake`, documents an intentional all-occurrences proof-hole text ban, and invokes `lake build` only after its checks pass. Add `/formalization/mathematical_foundations/.lake/` to `.gitignore` and `verify-mathematical-foundations-lean` to `mise.toml` before repository verification.

- [ ] **Step 4: Run the green wrapper tests**

Run: `cargo test -p harp --test mathematical_foundations_lean -- --test-threads=1`

Expected: PASS.

- [ ] **Step 5: Acquire and lock public inputs**

Before download, obtain official versioned release metadata and its published digest. Download a versioned Lean installer artifact, verify its digest, and install only under a task-scoped temporary `ELAN_HOME`; do not use a mutable installer pipe or a global home directory. With `ELAN_HOME` and `PATH` explicit on every command, run `lake update`, `lake exe cache get`, and `lake build`; commit only `lake-manifest.json`, never `.lake`, caches, or binaries.

- [ ] **Step 6: Commit the green bootstrap slice**

```sh
git add .gitignore mise.toml formalization/mathematical_foundations scripts/check_mathematical_foundations_lean.sh crates/harp/tests/mathematical_foundations_lean.rs
git commit -m "build: add mathematical foundations Lean project"
```

### Task 2: Formalize Module 1 — linear spaces and maps

**Files:** `MathematicalFoundations/Linear.lean`, root import.

- [ ] **Step 1: Write failing theorem statements**

Add declarations for linear-map zero preservation, additivity/scalar action, kernel closure, and finite-coordinate reconstruction. Use `LinearMap`, `Fin n → ℝ`, and explicit hypotheses rather than an unformalized basis abstraction where mathlib already exposes a suitable API. Begin one theorem with `sorry` and verify the wrapper rejects it.

- [ ] **Step 2: Prove the algebraic spine**

Replace proof holes using the appropriate `LinearMap.map_zero`, `map_add`, `map_smul`, extensionality, and `simp`/`ring` lemmas. Export only theorem names prefixed `linear_` and include module documentation stating all results are finite-dimensional or algebraic as appropriate.

- [ ] **Step 3: Build and commit**

Run: `ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan PATH=/private/tmp/harp-mathematical-foundations-elan/bin:$PATH scripts/check_mathematical_foundations_lean.sh`

Expected: root build PASS.

```sh
git add formalization/mathematical_foundations/MathematicalFoundations/Linear.lean formalization/mathematical_foundations/MathematicalFoundations.lean
git commit -m "feat: formalize linear spaces and maps"
```

### Task 3: Formalize Module 2 — orthogonality and spectra

**Files:** `MathematicalFoundations/Orthogonality.lean`, root import.

- [ ] **Step 1: State red orthogonality theorems**

Add a finite-real-vector projection identity, an orthogonal-residual inner-product identity, and a positive-definite quadratic positivity theorem with nonzero-vector and symmetry assumptions made explicit. Verify a temporary proof hole is rejected by the wrapper.

- [ ] **Step 2: Prove and bound the scope**

Use mathlib inner-product, norm, and matrix lemmas; do not state an eigenvalue or decomposition theorem unless its exact hypotheses and existing mathlib declaration are confirmed by a compiling proof. Document any spectral result as finite-dimensional real symmetric only.

- [ ] **Step 3: Build and commit**

Run the canonical wrapper with the pinned environment; expect PASS.

```sh
git add formalization/mathematical_foundations/MathematicalFoundations/Orthogonality.lean formalization/mathematical_foundations/MathematicalFoundations.lean
git commit -m "feat: formalize orthogonality foundations"
```

### Task 4: Formalize Modules 3 and 4 — finite probability, Bayes, and information

**Files:** `Probability.lean`, `BayesInformation.lean`, root import.

- [ ] **Step 1: Add finite-support definitions and red tests**

Define expectation only over explicit finite functions/probability mass functions supported by mathlib. State finite linearity, covariance expansion, Bayes' rule with nonzero evidence, and a finite KL nonnegativity/identity theorem only after checking available APIs. Confirm wrapper rejection for an injected proof hole.

- [ ] **Step 2: Implement exact identities**

Prove elementary finite-sum identities with `Finset.sum`, `ring`, and existing probability lemmas. If a desired Gaussian, entropy, or KL statement would require unsupported measure-theory infrastructure, omit it from Lean and reserve it for a prose-only map row with the explicit reason.

- [ ] **Step 3: Build and commit each green module**

Run the canonical wrapper after each namespace is imported; expect PASS.

```sh
git add formalization/mathematical_foundations/MathematicalFoundations/Probability.lean formalization/mathematical_foundations/MathematicalFoundations/BayesInformation.lean formalization/mathematical_foundations/MathematicalFoundations.lean
git commit -m "feat: formalize finite probability and Bayes identities"
```

### Task 5: Formalize Modules 5 and 6 — linear models and optimization

**Files:** `LinearModels.lean`, `Optimization.lean`, root import.

- [ ] **Step 1: State red controlled-model results**

For linear models, state normal-equation and ridge objective identities under explicit matrix/vector dimensions. For optimization, state scalar or finite-coordinate quadratic gradient-descent and stationary-iteration recurrences with explicit contraction hypotheses. Verify the wrapper rejects a proof hole in each new source file.

- [ ] **Step 2: Prove identities and separate claims**

Use `ring`, matrix algebra, and imported Module 1/2 results for exact identities. State convergence only from a proved contraction or positive-definiteness condition; classify logistic regression behavior, numerical conditioning estimates, and experimental choices as prose-only where they cannot be discharged in the chosen Lean boundary.

- [ ] **Step 3: Build and commit**

Run the canonical wrapper; expect all six namespaces and the root target to PASS.

```sh
git add formalization/mathematical_foundations/MathematicalFoundations/LinearModels.lean formalization/mathematical_foundations/MathematicalFoundations/Optimization.lean formalization/mathematical_foundations/MathematicalFoundations.lean
git commit -m "feat: formalize linear models and optimization"
```

### Task 6: Publish the 48-problem formalization map and enforce it

**Files:** `knowledge/mathematical_foundations/formalization_map.md`, `crates/harp/src/corpus/tests.rs`, `crates/harp/src/corpus/mod.rs`, `crates/harp/tests/cli.rs`.

- [ ] **Step 1: Write the failing corpus contract**

Add a corpus test requiring `math-foundations-formalization-map` as an auxiliary document. It must parse all `MF-01-01` through `MF-06-08` identifiers, reject duplicates/missing rows, require exactly one of `Direct theorem`, `Corollary/application`, or `Prose-only`, require a `Lean:` identifier for the first two statuses, and reject one for prose-only rows. It must scan the map and formalization sources for `/Users/` and `file://`.

- [ ] **Step 2: Run the red test**

Run: `cargo test -p harp compiles_the_mathematical_foundations_formalization_map --lib`

Expected: FAIL because the map and registration are absent.

- [ ] **Step 3: Add the map and registry entry**

Create 48 rows in module/problem order. Use direct theorem only when the declaration exists and builds; use corollary/application only with named compiled theorems; use prose-only with an exact limitation. Register the map as an auxiliary Mathematics document without changing the reader-route roster.

- [ ] **Step 4: Run green map and corpus tests**

Run:

```sh
cargo test -p harp compiles_the_mathematical_foundations_formalization_map --lib
cargo test -p harp corpus::tests --lib -- --test-threads=1
```

Expected: PASS.

- [ ] **Step 5: Commit**

```sh
git add knowledge/mathematical_foundations/formalization_map.md crates/harp/src/corpus/mod.rs crates/harp/src/corpus/tests.rs crates/harp/tests/cli.rs
git commit -m "docs: map mathematical foundations formalization"
```

### Task 7: Regenerate, document, and verify the integrated release

**Files:** `README.md`, `docs/product-contract.md`, `docs/import-receipt.md`, derived Atlas files.

- [ ] **Step 1: Document the exact boundary**

Describe the Lean companion as formalization of original Harp statements, list its six namespaces, link the map, and state that prose-only problems have no proof claim. Update canonical document expectations only from observed compiler output.

- [ ] **Step 2: Regenerate derived outputs**

Run:

```sh
cargo run -p harp -- check
cargo run -p harp -- build
cargo run -p harp -- search refresh
cd atlas && corepack pnpm run test:export
```

Expected: canonical corpus and offline export regenerate successfully.

- [ ] **Step 3: Refresh receipt after all payload is staged**

Stage only the named formalization, canonical prose, code, docs, and derived outputs. Run `cargo run -p harp -- repository verify`; copy its reported expected digest into `docs/import-receipt.md`; rerun until repository verification passes.

- [ ] **Step 4: Run release gates on the integration commit**

Run:

```sh
ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan PATH=/private/tmp/harp-mathematical-foundations-elan/bin:$PATH scripts/check_mathematical_foundations_lean.sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test mathematical_foundations_lean -- --test-threads=1
cd atlas && corepack pnpm run test
cd .. && mise run verify
```

Expected: every command exits successfully; record the aggregate exit status and the exact integration commit.

- [ ] **Step 5: Commit**

```sh
git add README.md docs/product-contract.md docs/import-receipt.md atlas/src/content/generated/corpus.json atlas/dist/harp-atlas.html atlas/dist/harp-atlas.receipt.json crates/harp/tests/cli.rs
git commit -m "docs: publish mathematical foundations formalization"
```

## Self-review

- Spec coverage: Tasks 1–5 establish the pinned proof project and all six namespaces in the requested order; Task 6 maps each of the 48 original problems; Task 7 publishes and verifies the integrated release.
- No placeholder tasks: every code-changing task names its files, test command, expected result, and green commit boundary.
- Consistency: project name, wrapper name, six namespace names, map statuses, and the no-local-external-checkout boundary are identical throughout.
