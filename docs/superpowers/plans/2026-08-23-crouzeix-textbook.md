# Crouzeix Foundations Textbook Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and verify a 35-chapter, self-contained mathematical textbook with Lean 4 support from structural linear algebra through both constant-two Crouzeix proof routes.

**Architecture:** The book is one vertically integrated Harp knowledge packet. Canonical Markdown, structured coverage contracts, chapter-scoped Lean modules, corpus registration, and Atlas rendering advance together in six dependency-ordered parts; a part is published only when its prose, exercises, formal statements, and focused gates agree.

**Tech Stack:** Markdown with Obsidian wikilinks and math extensions, Rust contract tests and corpus compiler, JSON coverage contracts, Lean 4.32.1 with pinned Mathlib, TypeScript/Vitest Atlas tests, mise task runner.

---

## Implementation constraints

- Work only in an isolated worktree on `codex/crouzeix-textbook` or a descendant
  integration branch.
- Preserve the primary checkout's unrelated untracked files.
- Do not install Git LFS, Lean, Mathlib, Rust, Node, or pnpm dependencies as
  part of a chapter task.
- Before Lean work, verify that `formalization/lean/.lake` is either the
  primary checkout's canonical cache or a symlink to it. Never run
  `lake update`, `lake --try-cache exe cache get Mathlib`, or
  `mise run lean-cache` without explicit owner approval.
- Use newly authored prose and exercises. Do not copy Lax, Spivak, Bishop, or
  captured Crouzeix source text.
- Keep technical prose under `knowledge/crouzeix_textbook/`; keep structured
  contracts under `content/crouzeix_textbook/`; do not add Markdown below
  `content/`.
- Use exact `CFT-<chapter>-<sequence>` identifiers. Once a published ID lands,
  do not reuse it.
- A chapter may land as `draft` with `open-formalization` rows only on a
  clearly labeled draft branch. A published part must have no mathematical
  open rows.
- Every commit below stages explicit paths. Never use `git add .` or
  `git add -A`.
- Refresh `docs/import-receipt.md` only after all other tracked payload for a
  landing batch has settled.

## File structure

### Canonical book packet

```text
knowledge/crouzeix_textbook/
├── crouzeix_textbook_index.md
├── reading_guide.md
├── notation_and_glossary.md
├── theorem_dependency_map.md
├── exercise_index.md
├── lean_coverage_ledger.md
├── source_registry.md
├── claim_evidence_ledger.md
├── status_and_scope.md
├── part_01_linear_structure/
│   ├── 01_objects_and_representations.md
│   ├── 02_vector_spaces_and_subspaces.md
│   ├── 03_linear_maps_and_exact_structure.md
│   ├── 04_coordinates_and_duality.md
│   ├── 05_determinants_trace_and_exterior_algebra.md
│   └── 06_eigenvalues_and_polynomial_algebra.md
├── part_02_geometry_and_calculus/
│   ├── 07_inner_product_spaces.md
│   ├── 08_positive_operators_and_gram_geometry.md
│   ├── 09_operator_norms_and_singular_values.md
│   ├── 10_multilinear_maps_and_tensors.md
│   ├── 11_differentiation_as_linear_approximation.md
│   └── 12_differential_forms_and_stokes.md
├── part_03_analysis_and_complex_functions/
│   ├── 13_metric_and_normed_spaces.md
│   ├── 14_sequences_and_series_of_operators.md
│   ├── 15_complex_differentiability.md
│   ├── 16_consequences_of_cauchy_theory.md
│   ├── 17_functions_of_matrices_and_operators.md
│   └── 18_positive_real_analytic_functions.md
├── part_04_operator_theory/
│   ├── 19_normality_and_nonnormality.md
│   ├── 20_numerical_range.md
│   ├── 21_spectral_sets.md
│   ├── 22_positive_and_completely_positive_maps.md
│   ├── 23_compression_and_dilation.md
│   └── 24_gramians_and_ordered_matrix_inequalities.md
├── part_05_crouzeix_machinery/
│   ├── 25_convex_boundaries_and_cauchy_layers.md
│   ├── 26_double_layer_map.md
│   ├── 27_one_plus_sqrt_two_barrier.md
│   └── 28_complete_power_family.md
└── part_06_constant_two_routes/
    ├── 29_crouzeix_problem_and_sharpness.md
    ├── 30_jin_positive_real_completion.md
    ├── 31_jin_correction_cancellation.md
    ├── 32_jin_constant_two_endpoint.md
    ├── 33_lorist_schwenninger_perturbation_lemma.md
    ├── 34_lorist_schwenninger_realization.md
    └── 35_comparison_verification_and_boundaries.md
```

### Structured contracts

```text
content/crouzeix_textbook/
├── coverage.json
├── exercises.json
└── theorem_dependencies.json
```

### Lean library

```text
formalization/lean/CrouzeixTextbook.lean
formalization/lean/CrouzeixTextbook/PublicTheorems.lean
formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean
...
formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean
```

Each `ChapterNN.lean` owns chapter definitions, public bridge theorems, worked
examples, and a `CrouzeixTextbook.Exercises.ChapterNN` namespace containing
checked reference solutions. `PublicTheorems.lean` imports the published
chapter modules and exposes a machine-readable `List String` plus `#check`
lines for every public declaration.

### Verification and presentation files

```text
crates/harp/tests/crouzeix_textbook.rs
crates/harp/tests/lean_library.rs
crates/harp/src/corpus/mod.rs
crates/harp/src/corpus/render.rs
crates/harp/src/corpus/tests.rs
scripts/render_crouzeix_textbook_ledgers.py
scripts/check_lean_library.sh
formalization/lean/lakefile.toml
mise.toml
atlas/src/content/canonical.test.ts
atlas/src/app/math.test.ts
atlas/src/content/generated/corpus.json
atlas/dist/harp-atlas.html
docs/import-receipt.md
```

## Chapter contract

Every chapter Markdown file has the standard Harp frontmatter plus these exact
top-level sections in order:

```markdown
## Opening problem
## Conceptual model
## Formal development
## Worked examples
## ML bridge
## Lean translation
## Exercises
## Synthesis and forward dependencies
```

The frontmatter uses:

```yaml
---
id: cft-chapter-NN-<slug>
title: <chapter title>
type: textbook-chapter
status: draft
created: 2026-08-23
updated: 2026-08-23
tags: [crouzeix-textbook, <part-tag>, mathematics, lean]
confidence: high
canonical: <filename>.md
chapter: NN
part: N
---
```

Every chapter includes at least:

- one opening problem;
- four numbered definitions or theorems;
- two complete worked examples;
- one bounded ML bridge;
- one copyable Lean declaration and explanation;
- six exercises: one retrieval, one calculation, two written proofs, one
  counterexample or boundary problem, and one Lean proof;
- checked Lean reference solutions for every proof-oriented exercise; and
- previous, next, part, and book-index links.

The minimum is a floor, not a target. Proof-heavy final chapters will exceed
it.

## Structured contract shapes

`content/crouzeix_textbook/coverage.json` begins with:

```json
{
  "schema_version": "crouzeix-textbook-coverage/v1",
  "book_status": "draft",
  "items": [
    {
      "item_id": "CFT-01-001",
      "chapter": 1,
      "kind": "definition",
      "prose_path": "knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md",
      "heading_id": "linear-transformation",
      "lean_declaration": "CrouzeixTextbook.Part01.linearTransformationCoordinateAction",
      "formal_status": "compiled-lean",
      "source_ids": ["LAX-2007"],
      "depends_on": [],
      "verification_target": "CrouzeixTextbook"
    }
  ]
}
```

`exercises.json` records `exercise_id`, `chapter`, `kind`, `difficulty`,
`prose_path`, `lean_declaration`, and `solution_declaration`.
`theorem_dependencies.json` records `item_id` and an ordered `depends_on`
array. Tests reject unknown fields, duplicate IDs, missing targets, cycles,
and mismatches between the three files.

## Task 1: Add fail-closed textbook contract tests

**Files:**
- Create: `crates/harp/tests/crouzeix_textbook.rs`
- Create: `scripts/render_crouzeix_textbook_ledgers.py`
- Create: `content/crouzeix_textbook/coverage.json`
- Create: `content/crouzeix_textbook/exercises.json`
- Create: `content/crouzeix_textbook/theorem_dependencies.json`

- [ ] **Step 1: Write the failing packet-roster and schema tests**

Add exact constants for the 35 chapter paths above, the nine support-document
paths, the eight required chapter headings, the four supported formal
statuses, and the three JSON schema versions. Parse all JSON through
`serde_json::Value`, reject unknown object keys, and assert that
`book_status=complete` forbids mathematical `open-formalization` rows.
Add a temporary-directory test for a deterministic ledger renderer. It must
accept `--contracts-root`, `--packet-root`, and exactly one of `--write` or
`--check`; sort by numeric chapter and stable ID; render the coverage ledger,
theorem dependency map, and exercise index; reject symlinked output targets;
and make `--check` fail with a named stale or missing output.

The first test entry points are:

```rust
#[test]
fn textbook_packet_has_exact_support_and_chapter_roster() { /* exact arrays */ }

#[test]
fn textbook_contracts_are_strict_and_cross_referenced() { /* strict keys */ }

#[test]
fn complete_textbook_has_no_open_mathematical_items() { /* completion gate */ }

#[test]
fn textbook_reader_ledgers_are_deterministic_and_current() { /* temp fixture */ }
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run:

```sh
cargo test --profile test-small -p harp --test crouzeix_textbook -- --test-threads=1
```

Expected: compilation succeeds; the roster test fails because
`knowledge/crouzeix_textbook/` and the three contracts do not exist, and the
renderer test fails because the script does not exist.

- [ ] **Step 3: Add strict draft contract fixtures**

Create the three JSON files with the exact schema versions, `book_status` set
to `draft`, empty item/exercise/dependency arrays, and no unknown fields. The
tests must allow empty arrays only while the packet root is absent; Task 3
removes that bootstrap exception when Chapter 1 lands.

Implement the renderer with the Python standard library only. Parse and
validate the same required top-level fields before rendering; write through a
temporary sibling followed by `os.replace`; and refuse any destination whose
existing path is a symlink. The temporary-directory Rust test exercises
`--write`, a passing `--check`, stale-content detection, stable ordering, and
symlink rejection.

- [ ] **Step 4: Run focused tests**

Run the command from Step 2.

Expected: all textbook contract tests pass with the explicit bootstrap
exception and no other skipped assertion.

- [ ] **Step 5: Commit**

```sh
git add crates/harp/tests/crouzeix_textbook.rs \
  scripts/render_crouzeix_textbook_ledgers.py \
  content/crouzeix_textbook/coverage.json \
  content/crouzeix_textbook/exercises.json \
  content/crouzeix_textbook/theorem_dependencies.json
git commit -m "test(crouzeix): define textbook coverage contracts"
```

## Task 2: Add the focused Lean library and gate

**Files:**
- Modify: `formalization/lean/lakefile.toml`
- Modify: `scripts/check_lean_library.sh`
- Modify: `mise.toml`
- Modify: `crates/harp/tests/lean_library.rs`
- Create: `formalization/lean/CrouzeixTextbook.lean`
- Create: `formalization/lean/CrouzeixTextbook/PublicTheorems.lean`
- Create: `formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean`

- [ ] **Step 1: Write the failing wrapper test**

Add a test beside `shared_wrapper_builds_crouzeix_focused_target`:

```rust
#[test]
fn shared_wrapper_builds_crouzeix_textbook_target() {
    let (_fake_bin, path) =
        fake_lake_bin("#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$LAKE_ARGS\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixTextbook")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixTextbook")
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "--try-cache build CrouzeixTextbook\n"
    );
}
```

- [ ] **Step 2: Run the test and verify it fails**

```sh
cargo test --profile test-small -p harp --test lean_library \
  shared_wrapper_builds_crouzeix_textbook_target -- --nocapture
```

Expected: failure with `Unknown Lean library target: CrouzeixTextbook`.

- [ ] **Step 3: Wire the new target**

Add `CrouzeixTextbook` to `defaultTargets` and add:

```toml
[[lean_lib]]
name = "CrouzeixTextbook"
```

Extend the wrapper usage, target case, `all` source roster, and `all` proof-
hole `admit` policy. Add `mise run lean-crouzeix-textbook` and include it in
`lean-timing`.

- [ ] **Step 4: Add a proof-hole-free bootstrap module**

Use:

```lean
import Mathlib.LinearAlgebra.Basic

namespace CrouzeixTextbook.Part01

/-- CFT-01-001: a bootstrap fact used to verify the textbook target. -/
theorem linear_map_preserves_zero
    {𝕜 V : Type*} [Semiring 𝕜] [AddCommMonoid V] [Module 𝕜 V]
    (T : V →ₗ[𝕜] V) : T 0 = 0 := by
  exact T.map_zero

end CrouzeixTextbook.Part01
```

`PublicTheorems.lean` imports Chapter 1, defines a one-entry string manifest,
and `#check`s the theorem. The root imports `PublicTheorems`.

- [ ] **Step 5: Run wrapper regressions**

```sh
cargo test --profile test-small -p harp --test lean_library -- --test-threads=1
```

Expected: all wrapper tests pass.

- [ ] **Step 6: Preflight and run the real focused Lean target**

Before invoking Lake, verify the toolchain and cache without writes:

```sh
test "$(cat formalization/lean/lean-toolchain)" = "leanprover/lean4:v4.32.1"
test -e formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean/Mathlib/LinearAlgebra/Basic.olean
```

Then run:

```sh
mise run lean-crouzeix-textbook
```

Expected: `[lean] target=CrouzeixTextbook` and `[lean] outcome=passed`. If the
cache preflight fails, stop with a cache-precondition blocker; do not hydrate.

- [ ] **Step 7: Commit**

```sh
git add formalization/lean/lakefile.toml \
  formalization/lean/CrouzeixTextbook.lean \
  formalization/lean/CrouzeixTextbook/PublicTheorems.lean \
  formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean \
  scripts/check_lean_library.sh mise.toml crates/harp/tests/lean_library.rs
git commit -m "feat(crouzeix): add textbook Lean target"
```

## Task 3: Publish the packet shell and Chapter 1

**Files:**
- Create all nine support documents under `knowledge/crouzeix_textbook/`
- Create: `knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md`
- Modify the three files under `content/crouzeix_textbook/`
- Modify: `crates/harp/tests/crouzeix_textbook.rs`
- Modify: `crates/harp/src/corpus/mod.rs`
- Modify: `crates/harp/src/corpus/render.rs`
- Modify: `crates/harp/src/corpus/tests.rs`
- Modify: `atlas/src/content/canonical.test.ts`
- Modify: `atlas/src/app/math.test.ts`

- [ ] **Step 1: Remove the bootstrap exception and verify failure**

Change the packet contract test so a present packet requires all nine support
documents, Chapter 1, nonempty coverage, six exercises, and exact navigation.

Run:

```sh
cargo test --profile test-small -p harp --test crouzeix_textbook -- --test-threads=1
```

Expected: failure naming the first missing support document.

- [ ] **Step 2: Write the support documents**

The index states the audience, 35-chapter route, draft status, source ceiling,
and links to all support documents. The reading guide defines normal and fast-
recall routes. The glossary begins with set, function, vector space, linear
map, coordinate vector, basis, proposition, theorem, and proof. The dependency
map contains Chapter 1 nodes. Generate the dependency map, exercise index, and
coverage ledger from the three contracts with:

```sh
python3 scripts/render_crouzeix_textbook_ledgers.py \
  --contracts-root content/crouzeix_textbook \
  --packet-root knowledge/crouzeix_textbook \
  --write
```

The status document marks only Chapter 1 as active. Source and claim files use
the credible-docs templates and register `LAX-2007`, `BISHOP-2006`,
Lean/Mathlib, and the existing Crouzeix packet as bounded source routes.

- [ ] **Step 3: Write Chapter 1**

Chapter 1 proves and exercises these items:

- `CFT-01-001`: a linear map is an object independent of coordinates;
- `CFT-01-002`: matrix columns are images of basis vectors;
- `CFT-01-003`: coordinate action is matrix multiplication;
- `CFT-01-004`: change of basis produces similarity;
- `CFT-01-005`: similarity preserves the characteristic polynomial;
- `CFT-01-006`: a nonunitary basis may change the operator norm.

The worked nonnormal example uses
`S = [[1,1],[0,1]]`, `Λ = diag(0,1)`, and
`T = S Λ S⁻¹ = [[0,1],[0,1]]`, showing spectral radius `1` but norm `sqrt 2`.
The ML bridge explains coordinate changes in representation spaces without
claiming arbitrary neural representations are linear or invertible.

- [ ] **Step 4: Complete Chapter 1 Lean and exercises**

Replace the bootstrap theorem-only file with declarations supporting the six
items and six exercises. Reuse `MathematicalFoundations.Linear` where exact;
prove bridge statements locally where the textbook interface differs. Update
`PublicTheorems.lean`, coverage, exercise, and dependency contracts together.

- [ ] **Step 5: Register the route and math rendering**

Add reader route:

```rust
(
    "crouzeix-textbook",
    "Crouzeix textbook",
    "knowledge/crouzeix_textbook/crouzeix_textbook_index.md",
),
```

Register the ten existing packet documents in `AUXILIARY_DOCUMENTS`, extend
`is_mathematics_packet` to `knowledge/crouzeix_textbook/`, and add corpus tests
for the route, exact current document count, math rendering, and link validity.

- [ ] **Step 6: Run focused gates**

```sh
cargo test --profile test-small -p harp --test crouzeix_textbook -- --test-threads=1
cargo test --profile test-small -p harp corpus::tests --lib -- --test-threads=1
mise run lean-crouzeix-textbook
python3 scripts/migrate_obsidian_links.py --check
```

Expected: all commands pass.

- [ ] **Step 7: Regenerate and test Atlas**

```sh
cargo run --quiet -p harp -- build
cd atlas && corepack pnpm run test
```

Expected: corpus generation succeeds and Atlas tests pass with all registered
textbook formulas rendered without `.math-error`.

- [ ] **Step 8: Commit**

Stage the nine support documents, Chapter 1, the three structured contracts,
the Chapter 1 Lean files, focused tests, corpus/Atlas registration, and the two
derived Atlas files explicitly. Commit:

```sh
git commit -m "feat(crouzeix): publish textbook foundation"
```

## Task 4: Complete Part I -- linear structure

**Files:**
- Create Chapters 2--6 at the exact Part I paths in the file structure
- Create: `formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean`
- Create: `formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean`
- Create: `formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean`
- Create: `formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean`
- Create: `formalization/lean/CrouzeixTextbook/Part01/Chapter06.lean`
- Modify support, contract, corpus, test, and generated files from Task 3

- [ ] **Step 1: Add failing roster and coverage expectations for Chapters 2--6**

Require all six Part I chapters, 36 exercises, navigation closure, nonempty
Lean declarations for mathematical items, and a Part I dependency subgraph
ending at simple-spectrum polynomial action.

- [ ] **Step 2: Write Chapters 2--6 and their exact theorem spines**

The public mathematical spines are:

- Chapter 2: span minimality, subspace intersections, basis coordinate
  uniqueness, dimension invariance, direct sums, and quotient projection.
- Chapter 3: kernels and ranges are subspaces, rank-nullity, inverse
  equivalences, invariant-subspace restriction, and composition rank bounds.
- Chapter 4: dual basis evaluation, annihilator dimension, transpose
  functoriality, coordinate-change conjugacy, and similarity invariants.
- Chapter 5: alternating characterization of determinant, determinant
  multiplicativity, determinant under similarity, trace cyclicity, trace under
  similarity, and exterior-power interpretation.
- Chapter 6: polynomial action on eigenvectors, minimal-polynomial
  annihilation, diagonalizable functional calculus, spectral mapping for
  polynomials, simple-spectrum generated algebra, and density of simple-
  spectrum matrices.

Each chapter includes the six required exercise kinds and at least one example
that becomes a named dependency of Part VI.

- [ ] **Step 3: Implement and compile Part I Lean**

Add Chapter 2--6 modules, checked exercise solutions, and public manifest
entries. Use existing Mathematical Foundations declarations only at exact
interfaces. Add bridge theorems for textbook notation rather than importing
the old packet's prose assumptions.

- [ ] **Step 4: Close Part I coverage and presentation**

Update the three JSON contracts, glossary, dependency map, exercise index,
coverage ledger, status document, auxiliary-document registry, corpus tests,
and Atlas formula-count tests. Mark Chapters 1--6 `active`; keep later chapters
absent rather than implying completion.

- [ ] **Step 5: Run Part I gates**

```sh
cargo test --profile test-small -p harp --test crouzeix_textbook -- --test-threads=1
cargo test --profile test-small -p harp corpus::tests --lib -- --test-threads=1
mise run lean-crouzeix-textbook
python3 scripts/migrate_obsidian_links.py --check
cd atlas && corepack pnpm run test
```

Expected: all pass and no Part I mathematical row is open.

- [ ] **Step 6: Commit**

Stage exact Part I, Lean, contract, support, registration, test, and generated
paths. Commit:

```sh
git commit -m "feat(crouzeix): teach linear structure"
```

## Task 5: Complete Part II -- geometry and calculus

**Files:**
- Create Chapters 7--12 at the exact Part II paths
- Create Lean modules `Part02/Chapter07.lean` through `Part02/Chapter12.lean`
- Modify the shared support, contract, manifest, corpus, test, and generated files

- [ ] **Step 1: Add failing Part II roster and dependency tests**

Require six new chapters, exercises `CFT-07-E01` through `CFT-12-E06`, and a
dependency path from inner products through pullbacks and Stokes.

- [ ] **Step 2: Write Chapters 7--12**

The public theorem spines are:

- Chapter 7: Cauchy--Schwarz, orthogonal decomposition, projection
  characterization, finite-dimensional Riesz representation, adjoint
  existence/uniqueness, and unitary norm preservation.
- Chapter 8: Gram positivity, congruence preservation, positive square-root
  existence/uniqueness, kernel of a positive quadratic form, Cholesky on the
  supported finite-dimensional domain, and the metric identity
  `‖S c‖² = c* (S* S) c`.
- Chapter 9: operator-norm characterization, top singular-vector attainment,
  `‖T‖² = ‖T* T‖`, singular-value decomposition, polar decomposition, and
  condition-number bounds.
- Chapter 10: universal bilinear factorization, tensor-coordinate change,
  alternating projection, wedge antisymmetry, determinant as top exterior
  action, and contraction identities.
- Chapter 11: uniqueness of the Fréchet derivative, chain rule, derivative of
  a bounded linear map, product derivative, inverse-function derivative, and
  implicit differentiation in the stated finite-dimensional form.
- Chapter 12: pullback functoriality, exterior derivative linearity,
  `d ∘ d = 0` on the supported smooth forms, change of variables, boundary
  orientation, and the selected generalized Stokes statement.

- [ ] **Step 3: Implement Lean and classify the Spivak boundary**

Compile all mathematical statements. Where Mathlib's manifold API would force
an artificial abstraction, state and prove the exact finite-dimensional chart
version used by the book. Mention the broader manifold result only as
source-bounded historical context, not as a numbered textbook theorem or
coverage item. Do not convert the content-gated Spivak outline into theorem
text.

- [ ] **Step 4: Close Part II ledgers and run gates**

Update every shared ledger and registry, then run the same focused commands as
Task 4. Add `mise run lean-autodiff` as a compatibility gate because Part II
reuses Autodiff Geometry interfaces.

Expected: all pass and Parts I--II have no mathematical open rows.

- [ ] **Step 5: Commit**

```sh
git commit -m "feat(crouzeix): teach geometry and calculus"
```

## Task 6: Complete Part III -- analysis and complex functions

**Files:**
- Create Chapters 13--18 at the exact Part III paths
- Create Lean modules `Part03/Chapter13.lean` through `Part03/Chapter18.lean`
- Modify the shared support, contract, manifest, corpus, test, and generated files

- [ ] **Step 1: Add failing Part III contract tests**

Require six chapters, 36 exercises, complex-analysis source rows, and a
dependency path from completeness to the positive Herglotz kernel.

- [ ] **Step 2: Write Chapters 13--18**

The theorem spines are:

- Chapter 13: equivalent finite-dimensional norms, compact unit sphere,
  continuous image of compact sets, completeness, boundedness of linear maps,
  and finite versus infinite-dimensional closure distinctions.
- Chapter 14: Banach-space geometric series, Neumann inverse, uniform limit of
  continuous maps, termwise bounded-linear transport, norm convergence of
  weighted Gramians, and continuity of inversion.
- Chapter 15: Cauchy--Riemann characterization where used, contour integral
  linearity, Cauchy theorem on the supported domains, Cauchy formula, power-
  series analyticity, and derivative estimates.
- Chapter 16: maximum modulus, identity theorem, open mapping, residue
  calculation, outer-neighborhood maximum convergence, and the exact
  approximation result used in Chapter 17.
- Chapter 17: polynomial and rational functional calculus identities,
  resolvent identity, contour functional calculus, spectral mapping on the
  supported finite-dimensional domain, commutation, and continuity under a
  fixed contour.
- Chapter 18: scalar Cayley positive-real property, analytic matrix-valued
  Cayley family, Herglotz-kernel positivity, repeated-sample admissibility,
  finite sampled quadratic nonnegativity, and origin-sample augmentation.

- [ ] **Step 3: Implement Lean without broadening the claim ceiling**

Reuse existing CrouzeixConjecture analytic modules when their statements
match. Record every broader textbook theorem not supplied by the current
Mathlib surface as an implementation obligation and prove it before publishing
Part III; do not downgrade it to prose-only.

- [ ] **Step 4: Close ledgers, run focused gates, and commit**

Run Task 4's gates plus `mise run lean-crouzeix` to ensure the new bridge
imports do not break the maintained proof surface.

Commit:

```sh
git commit -m "feat(crouzeix): teach analysis and complex functions"
```

## Task 7: Complete Part IV -- finite-dimensional operator theory

**Files:**
- Create Chapters 19--24 at the exact Part IV paths
- Create Lean modules `Part04/Chapter19.lean` through `Part04/Chapter24.lean`
- Modify the shared support, contract, manifest, corpus, test, and generated files

- [ ] **Step 1: Add failing Part IV contract tests**

Require six chapters, 36 exercises, and dependency paths from nonnormality to
numerical range, spectral sets, complete positivity, dilation, and Gramians.

- [ ] **Step 2: Write Chapters 19--24**

The theorem spines are:

- Chapter 19: unitary diagonalization of normal matrices, norm equals spectral
  radius for normal matrices, the explicit nonnormal amplification example,
  eigenbasis-conditioning bound, transient power growth example, and
  pseudospectral resolvent interpretation.
- Chapter 20: numerical-range compactness, affine covariance, spectrum
  inclusion, compression monotonicity, Toeplitz--Hausdorff convexity in the
  finite-dimensional form, and the exact `2 x 2` disk calculation.
- Chapter 21: polynomial/rational spectral-set definitions, monotonicity in
  the constant, algebra-homomorphism interpretation, scalar versus matrix
  amplification, complete-boundary counterexample contract, and rational-to-
  polynomial specialization.
- Chapter 22: positive-map star preservation, unital positivity bounds,
  complete-positivity matrix levels, Kadison inequality under its exact
  hypotheses, matrix-amplification bookkeeping, and the source-model boundary.
- Chapter 23: compression norm bound, isometry identities, contraction power
  bounds, Stinespring compression formula, dilation moment identities, and
  commuting power-family consequences.
- Chapter 24: weighted-Gramian positivity, convergence, Lyapunov recurrence,
  congruence preservation, Schur-complement criterion, anticommutator
  eigenvector evaluation, and factor-order counterexamples.

- [ ] **Step 3: Implement Lean, close ledgers, run gates, and commit**

Run Task 4's gates plus both `mise run lean-crouzeix` and
`mise run lean-crouzeix-ls`. Verify no theorem silently assumes normality or
commutativity. Commit:

```sh
git commit -m "feat(crouzeix): teach finite-dimensional operator theory"
```

## Task 8: Complete Part V -- Crouzeix machinery

**Files:**
- Create Chapters 25--28 at the exact Part V paths
- Create Lean modules `Part05/Chapter25.lean` through `Part05/Chapter28.lean`
- Modify the shared support, contract, manifest, corpus, test, and generated files

- [ ] **Step 1: Add failing Part V contract tests**

Require four chapters, 24 exercises, exact claim-ledger routes for the double-
layer and Crouzeix--Palencia claims, and two distinct outgoing dependency edges
from the complete-power-family chapter to the Jin and LS routes.

- [ ] **Step 2: Write Chapters 25--28**

The theorem spines are:

- Chapter 25: convex outer-neighborhood containment, fixed-boundary
  parametrization, positive double-layer density prerequisites, Cauchy layer
  evaluation, fixed-contour continuity, and ordered outer-limit statements.
- Chapter 26: positivity and unitality of the supported double-layer map,
  companion-transform antilinearity, star relation, symmetrized identity, power
  compatibility, and Stinespring boundary embedding.
- Chapter 27: the exact Crouzeix--Palencia coupled estimate, independent
  companion norm bound, scalar optimization yielding `1 + sqrt 2`, and a
  counterexample showing why arbitrary decoupling cannot recover two.
- Chapter 28: Cayley generating series, transport through a bounded positive
  map, explicit `f^n` perturbation family, commutation, uniform boundedness,
  and the bounded shared-power-family comparison without asserting theorem
  equivalence.

- [ ] **Step 3: Implement Lean and preserve source identity**

Bridge to existing source-mapped Crouzeix declarations. Cite pinned source IDs
in prose and ledgers, but never copy upstream proof text. Keep Jin, LS, and
Harp-local route dependencies distinguishable in the Lean import graph.

- [ ] **Step 4: Close ledgers, run gates, and commit**

Run Task 4's gates plus the three route-isolated checks:

```sh
mise run lean-crouzeix-jin
mise run lean-crouzeix-ls
mise run lean-crouzeix-harp
```

Commit:

```sh
git commit -m "feat(crouzeix): teach double-layer proof machinery"
```

## Task 9: Complete Part VI -- both constant-two routes

**Files:**
- Create Chapters 29--35 at the exact Part VI paths
- Create Lean modules `Part06/Chapter29.lean` through `Part06/Chapter35.lean`
- Modify the shared support, contract, manifest, corpus, test, and generated files

- [ ] **Step 1: Add failing endpoint and evidence-boundary tests**

Require seven chapters, 42 exercises, exact Jin and LS endpoint dependencies,
claim routes for publication/formalization status, and a test that rejects the
phrases `peer-reviewed proof`, `published proof`, or `completely bounded
Crouzeix theorem` unless they occur in an explicit negation or missing-evidence
entry.

- [ ] **Step 2: Write Chapters 29--32 -- statement and Jin route**

Required items are:

- sharp nilpotent example and lower bound two;
- fixed outer-domain normalization;
- simple-spectrum auxiliary approximation;
- same-basis target with repeated target values allowed;
- Cayley-family positive-real construction;
- diagonal adjoint-algebra defect;
- Herglotz sampling at conjugate target values divided by two;
- compensating origin vector `v = -G⁻¹ P u`;
- exact correction cancellation;
- ordered pre-Gramian inequality;
- balancing by the positive Gram square root;
- two weighted Gramians and positive difference;
- anticommutator eigenvector contradiction;
- first nonconstant term yielding norm two;
- polar-unitary norm transfer; and
- fixed-domain matrix limit followed by the outer-domain limit.

- [ ] **Step 3: Write Chapters 33--34 -- LS route**

Required items are:

- abstract perturbation interface;
- direct Equation 1 bound on `T^n` and `E_n T^n`;
- top singular-vector equation;
- adjacent-power recurrence using `E_n T = T E_n`;
- completion-of-squares lower bound;
- compression error independent of the power index;
- terminal term convergence for norm greater than one;
- first-power upper bound;
- scalar contradiction when the norm exceeds two;
- double-layer boundary isometry and multiplication contraction;
- realization `E_n = alpha(f^n)(A)`;
- boundedness and commutation; and
- abstract uniform-algebra and completely bounded boundaries.

- [ ] **Step 4: Write Chapter 35 and the final comparison**

Compare interfaces, not just conclusions. Include the shared power-family
inference, the lack of a theorem-level equivalence, the matrix-amplification
failure, the exact local Lean coverage, upstream build status, axiom boundary,
and remaining independent-review evidence.

- [ ] **Step 5: Implement the full Part VI Lean bridge**

Every mathematical item maps to an existing compiled route theorem or a new
textbook bridge theorem. The public manifest ends with checked declarations
for both constant-two endpoints and the bounded comparison theorem. Capture
axiom output and statement digests through repository-owned test fixtures; do
not hand-edit receipts.

- [ ] **Step 6: Close all chapter coverage and run route gates**

Set every chapter status to `active`. Keep `book_status=draft` until Task 10.
Run:

```sh
cargo test --profile test-small -p harp --test crouzeix_textbook -- --test-threads=1
mise run lean-crouzeix-textbook
mise run lean-crouzeix-jin
mise run lean-crouzeix-ls
mise run lean-crouzeix-harp
mise run lean-crouzeix
```

Expected: all pass and no chapter mathematical row remains open.

- [ ] **Step 7: Commit**

```sh
git commit -m "feat(crouzeix): teach both constant-two proof routes"
```

## Task 10: Prove whole-book completeness

**Files:**
- Modify all three files under `content/crouzeix_textbook/`
- Modify: `knowledge/crouzeix_textbook/status_and_scope.md`
- Modify: `knowledge/crouzeix_textbook/lean_coverage_ledger.md`
- Modify: `knowledge/crouzeix_textbook/theorem_dependency_map.md`
- Modify: `formalization/lean/CrouzeixTextbook/PublicTheorems.lean`
- Modify: `crates/harp/tests/crouzeix_textbook.rs`
- Modify: `atlas/src/content/generated/corpus.json`
- Modify: `atlas/dist/harp-atlas.html`
- Modify: `docs/import-receipt.md`

- [ ] **Step 1: Add the failing completion test**

Require `book_status=complete`, exactly 35 chapters, at least 210 exercises,
zero mathematical `open-formalization` rows, an acyclic dependency graph, two
reachable constant-two endpoints, complete Lean manifest correspondence, and
all support documents marked active.

Run:

```sh
cargo test --profile test-small -p harp --test crouzeix_textbook \
  complete_textbook_has_no_open_mathematical_items -- --nocapture
```

Expected: failure because the book remains `draft`.

- [ ] **Step 2: Perform the manual pedagogical audit**

Read all chapters in order and record the audit in `status_and_scope.md`.
Reject completion if any definition is used before introduction, a proof skips
a load-bearing step, an ML bridge overgeneralizes a mathematical result, an
exercise lacks a checked solution, or the difficulty jump exceeds the stated
reader contract without a bridge section.

- [ ] **Step 3: Reconcile machine ledgers**

Regenerate the reader coverage ledger, theorem dependency map, and exercise
index from the three JSON contracts with
`scripts/render_crouzeix_textbook_ledgers.py --write`. Run the same command
with `--check` and require byte-for-byte equality in the contract test. Set
`book_status=complete` only after zero open mathematical rows remain.

- [ ] **Step 4: Run focused content, Lean, and Atlas gates**

```sh
python3 scripts/migrate_obsidian_links.py --check
python3 scripts/render_crouzeix_textbook_ledgers.py \
  --contracts-root content/crouzeix_textbook \
  --packet-root knowledge/crouzeix_textbook \
  --check
cargo test --profile test-small -p harp --test crouzeix_textbook -- --test-threads=1
cargo test --profile test-small -p harp corpus::tests --lib -- --test-threads=1
cargo test --profile test-small -p harp --test lean_library -- --test-threads=1
mise run lean-crouzeix-textbook
mise run lean-crouzeix
cd atlas && corepack pnpm run test:export
```

Expected: all pass; Atlas export regenerates the two derived artifacts
together.

- [ ] **Step 5: Run source and repository verification**

```sh
cargo run --quiet -p harp -- sources verify
cargo run --quiet -p harp -- repository verify
```

Copy the exact repository digest into `docs/import-receipt.md`, then rerun
`repository verify`. Expected: pass with the refreshed digest.

- [ ] **Step 6: Run the full release gate**

Before the gate, verify disk, pinned toolchain, warm cache link, mise trust,
and exact Lean target without invoking Lake. Then run:

```sh
mise run verify
```

Expected: all Atlas, Rust, Lean, source, search, generated-cleanliness, LFS,
and repository checks pass on the exact candidate commit.

- [ ] **Step 7: Review final diff and commit**

```sh
git diff --check
git status --short
git diff --stat
```

Confirm the diff contains only coherent textbook, formalization, validation,
registration, generated, and receipt paths. Stage explicit paths and commit:

```sh
git commit -m "feat(crouzeix): complete foundations textbook"
```

## Task 11: Independent final review and local landing

**Files:**
- Review only; modify only exact paths required by accepted findings

- [ ] **Step 1: Run specification review**

Check every requirement in
`docs/superpowers/specs/2026-08-23-crouzeix-textbook-design.md` against current
files, coverage rows, Lean declarations, generated artifacts, and fresh gate
output. Record each requirement as proved, contradicted, or missing.

- [ ] **Step 2: Run standards review**

Review source boundaries, symlink safety, strict JSON parsing, theorem/claim
ceilings, proof-hole policy, axiom output, generated-file pairing, unrelated
work preservation, and explicit staging.

- [ ] **Step 3: Repair accepted findings with focused regressions**

For each accepted defect, add or tighten a failing test, observe failure,
apply the minimal correction, rerun the focused gate, and commit the repair by
concern. Do not waive a failing completion criterion.

- [ ] **Step 4: Run the definitive gate on the reviewed commit**

```sh
mise run verify
```

Expected: pass with no working-tree changes except ignored build output.

- [ ] **Step 5: Hand off for local landing**

Report the branch, commit series, complete gate evidence, residual source and
publication boundaries, and exact comparison against the 14 completion
criteria. Do not push. Local merge or fast-forward remains the project owner's
decision unless explicitly authorized.

## Spec-coverage map

| Design requirement | Implementation evidence |
|---|---|
| 35 self-contained chapters | Tasks 3--9; exact roster test in Tasks 1 and 10 |
| Undergraduate-math/PhD-ML audience | Chapter contract and Task 10 pedagogical audit |
| Measured, engaging pace | Section contract, six exercise modes, fast-recall route, Task 10 audit |
| Lax/Spivak/other source lineage | Support source registry and Tasks 3--8 source updates |
| Newly authored material | Source boundary tests and standards review |
| ML as background, mathematics primary | Required bounded ML bridge and pedagogical audit |
| Lean support throughout | Task 2 target, chapter modules in Tasks 3--9, Task 10 completion gate |
| Every mathematical part formalized | Strict coverage JSON and zero-open completion test |
| Both candidate proof routes | Task 9 chapters, endpoints, route gates, and dependency reachability |
| Claim and publication boundaries | Packet claim ledger, Task 9 language tests, Task 11 review |
| Corpus and Atlas integration | Tasks 3--10 corpus registration and paired generated artifacts |
| Full verification | Tasks 10--11 `mise run verify` on exact reviewed commit |
