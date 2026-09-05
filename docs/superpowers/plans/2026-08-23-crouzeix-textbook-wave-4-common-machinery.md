# Crouzeix Textbook Wave 4 Common Machinery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild Chapters 25–29 as the common analytic trunk used by the Jin, Lorist--Schwenninger, and Harp routes: outer convex domains, double-layer positivity, the earlier constant barrier, complete power families, normalization, approximation, and sharpness.

**Architecture:** Upgrade five chapters in dependency order while preserving CFT-25..29 IDs and public Lean names. Each chapter proves its six claims, adds six distinct solution declarations, and advances the running nonnormal 2×2 example. Chapter 29 branches pedagogically to Jin, LS, and Harp without importing a terminal provider.

**Tech Stack:** Markdown, Lean 4.32.1, existing `CrouzeixConjecture` common modules, Rust publisher, JSON v2 contracts, mise.

---

## Inventory and files

| Chapter | Canonical Markdown | CFT roles |
|---:|---|---|
| 25 | `knowledge/crouzeix_textbook/part_05_crouzeix_machinery/25_convex_boundaries_and_cauchy_layers.md` | convex projection, variational property, nonexpansiveness, outer radius, convergence, smooth outer-domain data |
| 26 | `knowledge/crouzeix_textbook/part_05_crouzeix_machinery/26_double_layer_map.md` | support point, resolvent invertibility, support positivity, double-layer resolvent, congruence, density positivity |
| 27 | `knowledge/crouzeix_textbook/part_05_crouzeix_machinery/27_one_plus_sqrt_two_barrier.md` | √2 facts, scalar quadratic barrier, triangle kernel, positive-map norm kernel |
| 28 | `knowledge/crouzeix_textbook/part_05_crouzeix_machinery/28_complete_power_family.md` | power Cauchy formula, mass one, contractive boundary functions, Cayley companions, positive completion, norm two |
| 29 | `knowledge/crouzeix_textbook/part_06_constant_two_routes/29_crouzeix_problem_and_sharpness.md` | maximum modulus, Crouzeix statement, main theorem, Jordan extremizer, attainment, least constant |

Modify matching Lean files `Part05/Chapter25.lean` through `Chapter28.lean` and
`Part06/Chapter29.lean`, both v2 contracts, and generated publication outputs.

## Task 1: Freeze the common-trunk contract

- [ ] Add RED tests for 30 unique theorem anchors, 30 distinct chapter exercise
  solution declarations, complete theorem cards, and exact code locators.
- [ ] Assert that CFT-25..29 form a pedagogical DAG and that both CFT-30 and
  CFT-33 begin from appropriate CFT-29/common nodes rather than each other.
- [ ] Assert the maintained common Lean closure imports neither terminal
  provider namespace.
- [ ] Commit as `test(crouzeix-textbook): freeze common Crouzeix machinery`.

## Task 2: Deepen Chapter 25, convex boundaries and Cauchy layers

- [ ] CFT-25-001 defines nearest-point projection onto a nonempty closed convex
  set and distinguishes existence from uniqueness.
- [ ] CFT-25-002 proves the variational inequality by differentiating the
  squared-distance function along a convex segment, including the endpoint
  argument.
- [ ] CFT-25-003 proves nonexpansiveness by writing both variational
  inequalities, adding them, and applying Cauchy--Schwarz.
- [ ] CFT-25-004 and 005 define the parallel outer radius and prove its decay,
  with compactness/uniformity and Hausdorff-distance assumptions explicit.
- [ ] CFT-25-006 constructs the smooth positively oriented outer boundary data
  and states exactly which regularity the Cauchy integral later consumes.
- [ ] Compute projection and outer-domain geometry for
  `A_{λ,α}`'s elliptical numerical range.
- [ ] Add `Exercises.Chapter25.exercise_01_solution` through
  `exercise_06_solution`; the Lean proof exercise reconstructs
  nonexpansiveness from the two variational inequalities.
- [ ] Verify `chapter_25_` tests and `mise run lean-crouzeix-textbook`; commit
  as `docs(crouzeix-textbook): deepen convex outer approximation`.

## Task 3: Deepen Chapter 26, the double-layer map

- [ ] CFT-26-001 chooses a supporting boundary point and normal, deriving the
  scalar half-plane separation from numerical-range containment.
- [ ] CFT-26-002 proves support resolvent invertibility: assume a kernel vector,
  pair with it, and contradict strict separation. Treat the boundary limit
  separately from an exterior point.
- [ ] CFT-26-003 derives positivity of the Hermitian support-resolvent density
  by an explicit congruence and quadratic-form calculation.
- [ ] CFT-26-004 defines the double-layer resolvent map with orientation and
  normalization; show its scalar Cauchy provenance.
- [ ] CFT-26-005 writes the matrix congruence in full and checks adjoints and
  scalar factors.
- [ ] CFT-26-006 integrates pointwise positivity to positive semidefiniteness;
  justify integrability and interchange with finite sums.
- [ ] Work out the nilpotent 2×2 resolvent and one boundary density explicitly.
- [ ] Add six distinct Chapter 26 solution theorems, including a resolvent
  invertibility exercise and a PSD-congruence exercise.
- [ ] Verify and commit as `docs(crouzeix-textbook): derive double-layer positivity`.

## Task 4: Explain the `1+√2` obstruction in Chapter 27

- [ ] Prove CFT-27-001..003 from the real square-root specification, including
  the nonnegativity branch that rules out `-√2`.
- [ ] For CFT-27-004, start from `x²≤2x+1`, factor
  `(x-1-√2)(x-1+√2)`, and audit every sign to conclude `x≤1+√2`.
- [ ] For CFT-27-005 and 006, derive the triangle/positive-map estimate that
  creates the quadratic inequality. Identify the discarded correlation term
  that prevents the argument from reaching two.
- [ ] Give a scalar equality case and the nonnormal `A_{0,2}` comparison. The
  ML analogy maps the triangle relaxation to a loose independent-component
  bound, states the exact inequality, denies claims about trained-network
  tightness, and asks readers to measure the discarded cross term.
- [ ] Add six distinct solutions. The Lean exercise proves the barrier theorem
  from its scalar hypotheses without invoking `one_plus_sqrt_two_barrier`.
- [ ] Verify and commit as `docs(crouzeix-textbook): expose the sqrt-two barrier`.

## Task 5: Build the complete power family in Chapter 28

- [ ] CFT-28-001 states the power-indexed Cauchy formula with domain,
  orientation, and analyticity hypotheses.
- [ ] CFT-28-002 proves mass one by applying the scalar Cauchy formula to the
  constant function and tracks normalization.
- [ ] CFT-28-003 defines the normalized boundary function and proves its sup
  norm is at most one.
- [ ] CFT-28-004 constructs every Cayley companion/power, not a single
  function, and proves the algebraic identity used by positivity.
- [ ] CFT-28-005 assembles pointwise positivity into a positive-real completion
  while preserving the complete power family.
- [ ] CFT-28-006 shows why simultaneous control of all powers removes the
  `1+√2` loss and yields the norm-two kernel. Cross-reference but do not depend
  on either terminal provider.
- [ ] Add six distinct solutions, one of which formalizes the mass-one
  normalization and one the final scalar norm extraction.
- [ ] Verify and commit as `docs(crouzeix-textbook): complete the power family`.

## Task 6: State, normalize, and prove sharpness in Chapter 29

- [ ] CFT-29-001 defines the maximum polynomial modulus on the closed numerical
  range and proves existence via compactness.
- [ ] CFT-29-002 states the polynomial Crouzeix bound with quantifiers and the
  zero-maximum edge case.
- [ ] CFT-29-003 states the exact main theorem surface shared by the providers;
  distinguish finite-matrix, rational spectral-set, and Hilbert-space forms.
- [ ] For CFT-29-004 and 005, compute the numerical range of
  `J=[[0,2],[0,0]]`, show it is the unit disk, evaluate `p(z)=z`, and calculate
  `‖p(J)‖=2` while `max_{W(J)}|p|=1`.
- [ ] CFT-29-006 proves the universal constant is at least two and combines
  this lower bound only with a separately identified terminal upper bound.
- [ ] Add six distinct solutions; include a Lean 2×2 norm/sharpness theorem
  that exposes the matrix calculation rather than aliasing the terminal result.
- [ ] Add historical context for Crouzeix's conjecture, the two source-derived
  proof routes, and Harp's derived finite-horizon route
  with exact dates/sources but no unsupported priority or peer-review claim.
- [ ] Verify and commit as `docs(crouzeix-textbook): prove normalization and sharpness`.

## Task 7: Publish and freeze Wave 4

```sh
cargo test -p harp --test crouzeix_textbook common_machinery_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
```

Regenerate all publisher outputs, perform an editorial proof reconstruction,
refresh `docs/import-receipt.md` last, and commit it separately. Wave 4 exits
only when 30 rows are reconstructible/exact and the common trunk genuinely
feeds all three terminal branches.
