# Crouzeix Textbook Wave 2 Jin Route Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Chapters 30–32 a complete reconstruction of the Jin positive-real-completion route, from the completion predicate through correction cancellation and the constant-two endpoint.

**Architecture:** Upgrade 18 preserved CFT rows in dependency order. Chapter 30 states the exact completion problem and Gramian bridge, Chapter 31 performs the sampled-kernel cancellation algebra, and Chapter 32 extracts the norm bound and carries it through rational, holomorphic, and polynomial limit passages. Each chapter receives six distinct Lean exercise solutions and truthful proof provenance.

**Tech Stack:** Markdown, Lean 4.32.1, existing `CrouzeixConjecture` Jin modules, Rust publisher, JSON v2 contracts, mise.

---

## Chapter inventory

| Chapter | CFT range | Core declarations |
|---:|---|---|
| 30 | CFT-30-001..006 | `positive_real_completion`, `positive_real_completion_statement`, `completion_gramian_four`, `completion_gramian_two`, `completion_gramian_difference`, `completion_gramian_source_positive` |
| 31 | CFT-31-001..006 | `completion_kernel_model`, `completion_kernel_at_zero`, `sample_origin_quadratic_identity`, `correction_sampling_identity`, `correction_sampling_cancels`, `kernel_positivity_implies_X` |
| 32 | CFT-32-001..006 | `completion_implies_norm_two`, `jin_polynomial_constant_two`, `jin_rational_spectral_set`, `jin_rational_constant_two`, `holomorphic_constant_two`, `polynomial_from_holomorphic` |

Read the corresponding provider modules under `formalization/lean/CrouzeixConjecture/`
and the pinned Jin evidence. Do not import any Lorist--Schwenninger module into
the Chapter 30–32 closure.

## Task 1: Freeze the Jin branch contract

- [ ] Add RED tests requiring unique CFT anchors, 18 complete theorem cards,
  distinct Chapter 30–32 solution declarations, exact Jin source locators, and
  a pedagogical branch from the common CFT-29 trunk with no CFT-33/34 node.
- [ ] Add a route test that compiles `CrouzeixJin` and rejects both direct and
  transitive imports from `Crouzeix.LoristSchwenninger`.
- [ ] Run and require RED:

```sh
cargo test -p harp --test crouzeix_textbook jin_route_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
```

- [ ] Commit the tests as `test(crouzeix-textbook): freeze the Jin teaching route`.

## Task 2: Rebuild Chapter 30's exact completion interface

- [ ] For CFT-30-001, display `IsPositiveRealCompletion B T H` with all four
  conjuncts: analyticity on the unit disk, `H 0=I`, positive real part, and
  the generated-adjoint-algebra defect condition.
- [ ] For CFT-30-002, quantify `B,T,H,hB,lambda`; state the shared-basis
  diagonalization, `|lambda_i|≤1`, completion hypothesis, and conclusion
  `‖T‖≤2`. Explain why simple spectrum belongs to the auxiliary `B`, not
  necessarily to `T`.
- [ ] For CFT-30-003..005, define the matrices `P`, `R`, and `X`, display the
  congruences to Gramian-four, Gramian-two, and their difference, and expand a
  2×2 instance by hand.
- [ ] For CFT-30-006, show every implication from source positivity through
  congruence and ordered-matrix subtraction. State the invertibility and
  Hermitian hypotheses at the line where each is used.
- [ ] Replace the six aliases in `Chapter30.lean` with exact local theorems
  where the textbook statement needs a bridge; otherwise mark exact reexports
  and link to `CompletionStatement.lean` or `CompletionGramianBridge.lean`.
- [ ] Add `Exercises.Chapter30.exercise_01_solution` through
  `exercise_06_solution`, including a copyable starter for unpacking the
  completion conjunction and a Lean congruence proof.
- [ ] Verify and commit:

```sh
cargo test -p harp --test crouzeix_textbook chapter_30_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
git add knowledge/crouzeix_textbook/part_06_constant_two_routes/30_jin_positive_real_completion.md \
  formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean content/crouzeix_textbook
git commit -m "docs(crouzeix-textbook): reconstruct Jin completion data"
```

## Task 3: Expose every vector and block in Chapter 31

- [ ] For CFT-31-001, write
  `K(z)=G(I-zΛ)⁻¹+D(z)G` and its entrywise formula. Define the origin sample,
  eigenvalue samples `conj(λᵢ)/2`, sparse vectors `uᵢeᵢ`, and compensating
  vector `v=-G⁻¹Pu` before using them.
- [ ] For CFT-31-002, prove `D(0)=0` from `K(0)=G` and show the corresponding
  Lean normalization rather than asserting it informally.
- [ ] For CFT-31-003, write the complete block quadratic form for the resolvent
  contribution: sample/sample gives `4R-2P`; sample/origin and origin/sample
  give `G+R`; origin/origin gives `2G`. Expand the sums and dimensions.
- [ ] For CFT-31-004 and 005, display the unknown diagonal contribution
  coordinate by coordinate and substitute `Gv+Pu=0`; show why both the term
  and its adjoint vanish. Include a scalar two-sample worked case.
- [ ] For CFT-31-006, pass from positive sampled kernel to the block-matrix
  quadratic inequality, remove the canceled term, and isolate the matrix `X`.
  State explicitly whether positivity is pointwise, sampled-kernel, or matrix
  positive semidefiniteness at each stage.
- [ ] Add exact textbook wrappers plus
  `Exercises.Chapter31.exercise_01_solution` through `exercise_06_solution`.
  At least one formal exercise proves sparse-vector column selection and one
  proves `completionUnknownHalfContribution_eq_zero`; neither may invoke the
  final `kernel_positivity_implies_X` theorem.
- [ ] Verify and commit:

```sh
cargo test -p harp --test crouzeix_textbook chapter_31_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
git add knowledge/crouzeix_textbook/part_06_constant_two_routes/31_jin_correction_cancellation.md \
  formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean content/crouzeix_textbook
git commit -m "docs(crouzeix-textbook): expand Jin kernel cancellation"
```

## Task 4: Reconstruct the Chapter 32 norm extraction

- [ ] For CFT-32-001, begin with the positive matrix inequality from Chapter
  31, apply the Gramian congruences from Chapter 30, remove the positive tail,
  and derive the quadratic estimate that yields `‖T‖≤2`. Show the choice of
  test vector and every square-root/invertibility transfer.
- [ ] For CFT-32-002, instantiate the completion theorem for the normalized
  polynomial problem and map each hypothesis to the earlier construction.
- [ ] For CFT-32-003 and 004, define pole-free rational evaluation, prove the
  spectral-set formulation, and show the polynomial/rational normalization
  rather than treating them as synonyms.
- [ ] For CFT-32-005 and 006, display both limit passages: outer-domain or
  simple-spectrum approximation to holomorphic functions, then polynomial
  recovery. State uniform convergence, spectrum containment, continuity of
  functional calculus, and norm-limit arguments explicitly.
- [ ] Add `Exercises.Chapter32.exercise_01_solution` through
  `exercise_06_solution`. The Lean exercise must prove one norm-extraction
  bridge with its hypotheses visible, not simply invoke `jinFinalCrouzeixConjecture`.
- [ ] Verify and commit:

```sh
cargo test -p harp --test crouzeix_textbook chapter_32_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
git add knowledge/crouzeix_textbook/part_06_constant_two_routes/32_jin_constant_two_endpoint.md \
  formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean content/crouzeix_textbook
git commit -m "docs(crouzeix-textbook): complete the Jin endpoint"
```

## Task 5: Add cumulative history, ML transfer, and route review

- [ ] In each chapter add labeled historical records with exact Jin artifact
  locators and separate publication, review, and Harp reproduction states.
- [ ] Continue the running matrix `A_{λ,α}`: Chapter 30 formulates an auxiliary
  completion, Chapter 31 computes its sampled blocks for the nilpotent case,
  and Chapter 32 recovers the sharp factor two.
- [ ] Use the four-field ML interface. Map `G` to feature geometry and sampled
  kernel positivity to a finite certificate; explicitly deny direct transfer
  to noisy empirical kernels or floating-point PSD checks; include a concrete
  eigenvalue-sampling and smallest-eigenvalue diagnostic.
- [ ] Have an independent editorial pass reproduce all displayed algebra from
  the chapter alone and trace each Lean link to both public and underlying code.

## Task 6: Publish and freeze Wave 2

```sh
cargo test -p harp --test crouzeix_textbook jin_route_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
```

Regenerate all publication outputs together, refresh the import receipt last,
and use explicit commits grouped as prose, Lean/contracts, generated outputs,
and repository receipt. Wave 2 exits only when all 18 rows are
reconstructible/exact and the Jin kernel graph remains independent of LS.
