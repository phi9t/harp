# Trusted Cauchy–Schwarz fixtures implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a human-reviewed exact finite-real Cauchy–Schwarz reference and
type-level regression controls to the existing mathematical-foundations gate.

**Architecture:** Add two small Lean modules to the existing library. Keep the
statement independent of the reference proof, and write controls against the
literal intended proposition. This supplies trusted positive fixtures for W2;
it does not execute untrusted candidates or implement an acceptance service.

**Tech Stack:** Existing pinned Lean 4.32.1/Mathlib warm environment, existing
`MathematicalFoundations` library, existing `mise run lean-foundations` gate.

---

Code below is proposed and has not been compiled during planning. Stop if the
current checkout differs materially from the inspected interfaces; reconcile
before execution. The library theorem signature was inspected in the existing
Mathlib cache. No claim is made that this is a new proof of Cauchy–Schwarz.

## Files and responsibility

| Action | Exact repository-relative path | Responsibility |
|---|---|---|
| Create | `formalization/lean/MathematicalFoundations/TextbookCauchySchwarz.lean` | Fixed proposition and library-reuse reference |
| Create | `formalization/lean/MathematicalFoundations/TextbookCauchySchwarzControls.lean` | Literal full-target and edge-case compile controls |
| Modify | `formalization/lean/MathematicalFoundations.lean` | Import controls so the existing gate checks both new files |
| Modify | `crates/harp/tests/lean_library.rs` | Include the new direct Mathlib import in the all-library fake cache |
| Modify | `crates/harp/tests/mathematical_foundations_lean.rs` | Include the same import in both foundations fake-cache rosters |

These paths are relative to the isolated execution worktree, not a dependency
on another checkout. Use the primary checkout only for the approved machine-local
Lean cache relationship. Do not change the toolchain, Lake manifest, policy,
route checker, or curriculum schema in this slice.

## Task 1: establish execution preconditions

- [ ] Inspect current worktree instructions, `git status --short`,
  `formalization/lean/lean-toolchain`, `formalization/lean/lakefile.toml`,
  `scripts/harp_xdg_env.sh`, and the `lean-env`/`lean-foundations` mise tasks.
  Reconcile the planning base `e5b45c22` with current primary `2e5d57da` or its
  successor. Preserve all unrelated work.
- [ ] Inspect the cache link with `readlink formalization/lean/.lake` and check
  that it points to the verified primary cache under the repository's ancestry
  rules. Check worktree-local mise trust before invoking a task. Do not create
  or replace an unexpected cache link without resolving its current owner.
- [ ] Run `mise run lean-env`. Expected: exit zero after read-only environment
  preflight, with no Lake build or fetch. Failure blocks execution. Do not run
  cache-maintenance tasks to repair it implicitly.
- [ ] Confirm the imported Mathlib modules' `.olean` files exist in the warm
  dependency tree. Confirm the actual `Finset.sum_mul_sq_le_sq_mul_sq` signature
  matches its use below. Run source only in the confinement required by the
  applicable execution skill. This slice contains trusted hand-authored code.

## Task 2: write the regression controls first

- [ ] Create
  `formalization/lean/MathematicalFoundations/TextbookCauchySchwarzControls.lean`
  with the following complete content:

```lean
import MathematicalFoundations.TextbookCauchySchwarz

open scoped BigOperators

namespace TextbookBench.Controls

-- This spells out the target independently of the named proposition.
example : ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2) :=
  TextbookBench.cauchySchwarzReference

-- The public reference must remain applicable at dimension zero.
example (x y : Fin 0 → ℝ) :
    (∑ i, x i * y i) ^ 2 ≤
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2) :=
  TextbookBench.cauchySchwarzReference 0 x y

-- Zero vectors require no side condition.
example (n : ℕ) (y : Fin n → ℝ) :
    (∑ i, (0 : ℝ) * y i) ^ 2 ≤
      (∑ _i : Fin n, (0 : ℝ) ^ 2) * (∑ i, y i ^ 2) :=
  TextbookBench.cauchySchwarzReference n (fun _ => 0) y

-- An independent simplification control for the empty sum convention.
example (x y : Fin 0 → ℝ) :
    (∑ i, x i * y i) ^ 2 =
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2) := by
  simp

end TextbookBench.Controls
```

- [ ] Append this one import to
  `formalization/lean/MathematicalFoundations.lean`, preserving every existing
  import:

```lean
import MathematicalFoundations.TextbookCauchySchwarzControls
```

- [ ] Run `mise run lean-foundations`. Expected: nonzero exit because
  `MathematicalFoundations.TextbookCauchySchwarz` does not exist yet. An unrelated
  infrastructure error is not the intended red test. Do not commit this
  intermediate failing state.

## Task 3: add the protected-target reference

- [ ] Create
  `formalization/lean/MathematicalFoundations/TextbookCauchySchwarz.lean`
  with the following complete content:

```lean
import Mathlib.Data.Real.Basic
import Mathlib.Algebra.Order.BigOperators.Ring.Finset

open scoped BigOperators

namespace TextbookBench

/-- The fixed first workload, including dimension zero and zero vectors. -/
def CauchySchwarzTarget : Prop :=
  ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2)

/-- A library-reuse control, not evidence of theorem discovery. -/
theorem cauchySchwarzReference : CauchySchwarzTarget := by
  intro n x y
  simpa using Finset.sum_mul_sq_le_sq_mul_sq Finset.univ x y

end TextbookBench
```

- [ ] Run `mise run lean-foundations`. Expected: exit zero, with both new
  modules checked through the existing library root. If the signature differs,
  inspect the pinned source and repair only the application; do not weaken the
  target or add hypotheses to make it compile.
- [ ] Review the literal target control against the curriculum statement.
  Confirm the proposition contains no extra assumptions and that library reuse
  is labeled. Preserve the gate output as implementation evidence.

## Task 4: maintain wrapper-test cache fixtures

Execution uncovered this missing integration step. These fixtures test wrapper
selection and environment isolation; they do not stand in for real Lean proof
acceptance. All three affected rosters need the new direct import.

- [ ] In `crates/harp/tests/lean_library.rs`, add the following entry after
  `"Mathlib.Algebra.Module.Submodule.Ker",` in
  `write_fake_all_mathlib_artifacts`:

```rust
            "Mathlib.Algebra.Order.BigOperators.Ring.Finset",
```

- [ ] In `crates/harp/tests/mathematical_foundations_lean.rs`, add the following
  entry after `"Mathlib.Algebra.Module.Submodule.Ker",` in
  `ambient_project_override_is_ignored`:

```rust
            "Mathlib.Algebra.Order.BigOperators.Ring.Finset",
```

- [ ] In `crates/harp/tests/mathematical_foundations_lean.rs`, add the following
  entry after `"Mathlib.Algebra.Module.Submodule.Ker",` in
  `ambient_elan_toolchain_is_overridden_for_normal_builds`:

```rust
            "Mathlib.Algebra.Order.BigOperators.Ring.Finset",
```

- [ ] Run both affected suites together:

```sh
mise exec -- sh -c 'CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp --test lean_library --test mathematical_foundations_lean'
```

Expected: 27 shared-wrapper tests and 6 foundations-wrapper tests pass. The
observed failures before repair named the missing `Finset.olean` in the fake
cache. The canonical Mathlib artifact was present and real Lean compilation
had already passed. Do not change production cache policy or test assertions.

## Task 5: verify and land the coherent slice

- [ ] Run `git diff --check`. Expected: exit zero. Inspect the three explicit
  Lean paths and both Rust fixture paths, including untracked file contents,
  for accidental changes.
- [ ] Freeze the candidate and run `mise run verify`. Expected: the full
  repository gate succeeds, including the new modules through the existing
  mathematical-foundations import graph. If it fails, report the actual cause;
  focused proof compilation alone is not local-landing authorization.
- [ ] Stage only these paths and create a local commit after all required
  repository receipt/derived-file checks have passed:

```sh
git add formalization/lean/MathematicalFoundations.lean formalization/lean/MathematicalFoundations/TextbookCauchySchwarz.lean formalization/lean/MathematicalFoundations/TextbookCauchySchwarzControls.lean crates/harp/tests/lean_library.rs crates/harp/tests/mathematical_foundations_lean.rs
git commit -m "feat(lean): add trusted textbook Cauchy-Schwarz controls"
```

Expected: one coherent verified local commit. Do not push. If the repository
gate requires an import-receipt refresh or generated artifact changes, follow
the repository-owned workflow and explicitly review/stage those additional
paths; do not manufacture or copy stale digests.

## Explicitly deferred contracts and activation

This slice establishes only Q01's mathematical fixture, not Q01 verifier
acceptance. W2 activates protected-target comparison, alternate proof Q02,
weakened statements, hidden axioms, forged output, confinement, sealing,
receipts and replay. Do not add `sorry`-containing adversarial modules to the
ordinary library root. Store those fixtures in the qualified isolated verifier
suite when its execution boundary exists.

Before W2 coding, write its exact-code packet against the selected checker and
qualified environment. The contracts are already fixed in
[the verifier specification](../specs/2026-09-07-textbook-verifier-contracts.md).
Avoid treating these trusted compile controls as a security boundary or as
production qualification of the current compiler.
