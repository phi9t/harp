# Crouzeix Textbook Wave 1 Chapter 33 Reference Workshop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild Chapter 33 as the reference-quality, source-reconstructible workshop for the Lorist--Schwenninger perturbation lemma, with exact equations, distinct exercises, and compiler-verified Lean correspondence.

**Architecture:** Treat the six existing CFT-33 items as one vertical proof chain. The prose derives the recurrence and scalar contradiction line by line; the textbook Lean module exposes exact local statements or truthful reexports to the substantive LS provider; the Wave 0 publisher binds every card and exercise to its compiled receipt.

**Tech Stack:** Markdown, Lean 4.32.1, Mathlib 4.32.1, existing `Crouzeix.LoristSchwenninger` provider, Rust publication tests, mise.

## Readiness reconciliation (2026-08-25)

The following corrections are part of the executable plan:

- Task 1 stages the Chapter 33 prose together with its tests, source registry,
  and `coverage.json` kind correction; otherwise its theorem-card and notation
  work would be omitted from the commit.
- Tasks 2 and 4 run both `lean-crouzeix-textbook` and
  `lean-crouzeix-ls`: the former checks the public textbook wrappers and
  exercise namespace, while the latter checks the provider route.
- The supporting provider for source Equation (4) is
  `CrouzeixConjecture.LoristSchwenninger.DilationData.displacementSq_le` in
  `formalization/lean/Crouzeix/LoristSchwenninger/DisplacementUpper.lean`.
- Published counts advance only after a freshly compiled Lean receipt passes;
  prose or authored contract edits alone do not change exact-correspondence or
  checked-exercise counts.
- Exercise declarations use the full namespace
  `CrouzeixTextbook.Part06.Exercises.Chapter33`.
- The pinned source graph is the locator authority. Older packet prose that
  describes the LS terminal route as blocked is only dated status evidence and
  must be explicitly superseded by current local verification, never silently
  treated as current status.
- Chapter 33's focused test names are introduced by Task 1. Every documented
  filter must be checked with `cargo test ... -- --list` and must select a
  nonzero test set before its result counts as a gate.

## Task 1 quality-review reconciliation (2026-08-25)

The Task 1 candidate is repaired before Task 2 begins:

- The active textbook claim ledger, not the dated LS source-graph status
  fields, owns the current local-verification statement. It records the
  compiled provider theorem separately from source publication and peer-review
  status, while leaving the raw source graph unchanged.
- Chapter 33's labeled `SOURCE CLAIM` and `EVIDENCE` blocks route to stable
  `CFT-CL-*` claim IDs so the labels are auditable rather than free text.
- CFT-33-004 mirrors `scalar_combined_inequality_of_recurrence_bounds`: it does
  not assume `b ≥ 0`. Nonnegativity enters only at CFT-33-005 through
  `scalar_endpoint_le_two`.
- Regression tests parse the exact retired anchor `{#the-six-item-spine}` and
  structured claim-ledger fields/routes. They do not freeze incidental English
  phrasing that later proof-deepening tasks are expected to improve.

## Task 1 status-boundary reconciliation (2026-08-25)

Before Task 2, the active status page must route its current LS verification
claim through `CFT-CL-003` and its pinned source attribution through
`CFT-CL-006`. The dated source graph remains a locator and historical-status
authority, not the authority for the current local build. Regression coverage
must validate those structured routes, fields, and authority boundaries; it
must not reject particular obsolete English sentences by blacklist.

## Task 1 exercise-kind reconciliation (2026-08-25)

A fresh Wave 1 compiler receipt classifies the existing Chapter 1 exercise
solutions E01 through E05 as `direct-alias`, while the truthful v2 exercise
contract deliberately accepts only compiler-classified `theorem` solutions.
Before Task 2 publishes its first new checked exercises, convert exactly those
five baseline declarations into genuine theorem proofs under their stable
names. Preserve Chapter 1's mathematical intent, leave E06 unchanged because
it already classifies as `theorem`, and update its five receipt rows only from
a fresh compile. This is a Task 1 integration prerequisite, not Chapter 33
mathematical scope; it must land in a separate commit before Task 2.

## Task 2 proof-bridge review reconciliation (2026-08-25)

The first Task 2 review found that CFT-33-001 jumped directly to the provider's
rewritten inner-product form. The reconstructible proof must instead begin at
the exact `recurrenceScalar` definition
`Re ⟪x, (E_n T^n)x⟫` and display the complete bridge to
`Re ⟪E_n x, (T*)^n x⟫`: commute `E_n` with `T^n`, transfer the power through
the inner product by adjunction, and use real-part symmetry. Repeat the bridge
at `n + 1` before the adjacent-power recurrence step. Regression coverage must
check these mathematical transformations and both indices without freezing
incidental connective prose or an arbitrary count of the word “commutation.”

## Task 2 publication-and-exercise review reconciliation (2026-08-25)

The second Task 2 review adds two landing conditions:

- Publication-facing prose and generated ledgers must project the canonical
  contracts after CFT-33-001/002 become exact and Chapter 33 E01/E02 become
  solved. The canonical status pages and Chapter 35 baseline must report the
  contract-derived totals; the corpus JSON and Atlas HTML/receipt regenerate
  together, and `docs/import-receipt.md` refreshes only after the complete
  repository payload passes verification.
- Compiler kind alone is insufficient evidence that an exercise is a distinct
  learning task. A registered solution must compile as a theorem and its
  normalized statement fingerprint must differ from every public theorem
  checkpoint in the same chapter. Reaudit Chapter 1 E01–E06 and Chapter 33
  E01/E02 under that rule. Repair Chapter 1 E03 and E05 so their stable solution
  names answer their prompts with substantive coordinate-action and
  similarity/unitary-boundary statements, then accept metadata only from a
  fresh compiler receipt.

Regression tests derive counts and chapter relationships from structured
contracts and compiler receipts; they do not freeze explanatory sentences.

## Task 2 Atlas-link review reconciliation (2026-08-25)

The next review found that the Atlas renderer does not classify
`formalization/...` wikilinks as vault-root paths even though the canonical
Obsidian resolver does. A Chapter 33 link can therefore be resolved relative
to its `knowledge/...` directory and published with a duplicated
`knowledge/.../formalization/...` href.

The repair must make the rendered-corpus path policy agree with the Obsidian
resolver for the existing `formalization` root, preferably through one shared
minimal predicate. A focused behavioral regression must assert the exact
root-qualified Chapter 33 href and reject the duplicated relative form. After
the canonical code and test pass, regenerate corpus JSON, Atlas HTML, and Atlas
receipt as one derived set; refresh the import receipt only after the complete
gate reports the new payload digest.

---

## Files and theorem inventory

**Modify:**

- `knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma.md`
- `formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean`
- `content/crouzeix_textbook/coverage.json`
- `content/crouzeix_textbook/exercises.json`
- generated textbook ledgers, corpus JSON, and Atlas HTML

**Read as proof authorities:**

- `formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean`
- `formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean`
- `formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean`
- the pinned LS source and exact locators registered in the evidence packet

| CFT ID | Stable public Lean name | Required mathematical role |
|---|---|---|
| CFT-33-001 | `CrouzeixTextbook.Part06.recurrence_difference_identity` | Exact adjacent-power recurrence. |
| CFT-33-002 | `CrouzeixTextbook.Part06.recurrence_difference_lower_bound` | Completed-square lower bound. |
| CFT-33-003 | `CrouzeixTextbook.Part06.equation_three_lower_bound` | Inverse-power telescoping and source Equation (3). |
| CFT-33-004 | `CrouzeixTextbook.Part06.scalar_combined_inequality` | Combine Equations (3) and (4). |
| CFT-33-005 | `CrouzeixTextbook.Part06.scalar_endpoint_two` | Scalar contradiction when κ > 2. |
| CFT-33-006 | `CrouzeixTextbook.Part06.perturbation_lemma` | Assemble the operator theorem ‖T‖ ≤ 2. |

## Task 1: Freeze mathematical and source correspondence

- [ ] **Step 1: Add RED prose-card and source-locator tests**

Require six unique `cft-33-00N` anchors, the ten theorem-card fields from Wave
0, exact source locators for Equations (3) and (4), and no generic
`the-six-item-spine` anchor. Require the historical block to distinguish the
source's statement, the Harp reproduction, and the textbook exposition.

```sh
cargo test -p harp --test crouzeix_textbook chapter_33_prose_ -- --test-threads=1
```

Expected RED: all six rows currently share the generic spine and the chapter
does not display the exact recurrence scalar or inequalities.

- [ ] **Step 2: Record the exact notation ledger**

Before prose expansion, define in the chapter: the spaces `E,K`, target `T`,
isometry/compression data, perturbations `E_n`, norm `κ=‖T‖`, norm-attaining
unit vector `x`, singular-vector equation `T*Tx=κ²x`, displacement
`d=Q*VTx-κVx`, `b=‖d‖²`, and

```text
m_n = Re ⟪E_n x, (T*)^n x⟫.
```

Map each symbol to the exact Lean field or definition, especially
`DilationData.perturbation`, `recurrenceScalar`, `displacement`,
`displacementSq`, and `firstPerturbationMoment`.

- [ ] **Step 3: Commit the source/notation test surface**

```sh
git add crates/harp/tests/crouzeix_textbook.rs \
  knowledge/crouzeix_textbook/source_registry.md \
  knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma.md \
  content/crouzeix_textbook/coverage.json
git commit -m "test(crouzeix-textbook): freeze chapter 33 proof correspondence"
```

## Task 2: Write CFT-33-001 and CFT-33-002 in full

- [ ] **Step 1: Derive the adjacent-power identity**

Show every rewrite from
`m_n` and `m_{n+1}` through commutation, adjoint transfer, the identity
`(T*)^(n+1)Tx=κ²(T*)^n x`, and the definition of the adjacent-power defect.
End at the exact checked equality:

```text
κ m_n - m_{n+1}
= (κ²-κ) ‖(T*)^n x‖²
  - Re ⟪adjacentPowerDefect κ n x, (T*)^n x⟫.
```

- [ ] **Step 2: Complete the square explicitly**

Introduce `y=(T*)^n x` and `d=displacement κ x`. Expand the relevant squared
norm, show why `κ²-κ>0` follows from `κ>1`, and derive

```text
-‖d‖²/(κ²-κ) ≤ κ m_n-m_{n+1}.
```

Do not replace the algebra by “complete the square.” Include a one-dimensional
scalar check and explain equality conditions.

- [ ] **Step 3: Add exact Lean wrappers and exercise solutions**

Preserve the public names. Use `theorem ... := by` when the textbook proof
adds a local bridge; otherwise use `reexported-proof` and expose the provider
declaration in the Lean block. Add distinct solutions
`Exercises.Chapter33.exercise_01_solution` and
`exercise_02_solution`; neither may be definitionally the public checkpoint.

- [ ] **Step 4: Verify and commit**

```sh
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
cargo test -p harp --test crouzeix_textbook chapter_33_cft_001_002 -- --test-threads=1
git add knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma.md \
  formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean \
  content/crouzeix_textbook/coverage.json content/crouzeix_textbook/exercises.json
git commit -m "docs(crouzeix-textbook): derive the LS recurrence"
```

## Task 3: Prove source Equation (3) by inverse-power telescoping

- [ ] **Step 1: Add a RED test for the missing finite telescoping display**

Require the chapter to state the finite inequality before taking a limit, name
the uniform bound on `|m_n|`, justify every division by κ, and identify the
geometric series. Require CFT-33-003 to depend pedagogically on 001 and 002,
not on any Jin item.

- [ ] **Step 2: Write the finite recurrence proof**

From `r_n≤κm_n-m_{n+1}`, divide by `κ^(n+1)`, sum from `n=1` to `N`, show the
telescoping endpoints, bound the terminal term by
`M/κ^(N+1)`, and prove it tends to zero. Evaluate the remaining geometric sum
and the factorization `κ²-κ=κ(κ-1)` to obtain

```text
-b/[κ(κ-1)²] ≤ Re ⟪x,E₁Tx⟫.
```

Include the exact hypothesis use: `κ>1` supplies positivity, nonzero
denominators, geometric convergence, and terminal decay.

- [ ] **Step 3: Add a distinct Lean reconstruction exercise**

Add `Exercises.Chapter33.exercise_03_solution` proving the finite scalar
telescoping lemma used by the theorem. The starter must expose the induction
and finite-sum goal rather than simply invoke `equation_three_lower_bound`.

- [ ] **Step 4: Verify and commit**

```sh
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
cargo test -p harp --test crouzeix_textbook chapter_33_cft_003 -- --test-threads=1
git add knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma.md \
  formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean \
  content/crouzeix_textbook
git commit -m "docs(crouzeix-textbook): prove the LS telescoping bound"
```

## Task 4: Display Equation (4) and finish the scalar contradiction

- [ ] **Step 1: Write the displacement upper-bound derivation**

Expand `b=‖Q*VTx-κVx‖²`, use the isometry/compression identities and the first
perturbation moment, and derive the exact upper bound

```text
b ≤ 2κ²-κm-κ³.
```

Label it source Equation (4), cite its exact source locator, and show where
unit norm, norm attainment, and the singular-vector equation enter.

- [ ] **Step 2: Prove CFT-33-004 and CFT-33-005 line by line**

Substitute Equation (3) into Equation (4), derive

```text
b(1-1/(κ-1)²) ≤ 2κ²-κ³,
```

then suppose `κ>2`. Prove the left factor is positive and the right side
`κ²(2-κ)` is negative. Use `b≥0` to obtain the contradiction. Treat `κ≤1`,
`1<κ≤2`, and `κ=2` explicitly as boundary cases.

- [ ] **Step 3: Add exercise solutions 04 and 05**

Make exercise 04 a denominator/sign audit and exercise 05 a Lean proof of the
real scalar endpoint from the combined inequality. Compile both as distinct
theorems.

- [ ] **Step 4: Verify and commit**

```sh
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
cargo test -p harp --test crouzeix_textbook chapter_33_cft_004_005 -- --test-threads=1
git add knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma.md \
  formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean content/crouzeix_textbook
git commit -m "docs(crouzeix-textbook): close the LS scalar contradiction"
```

## Task 5: Assemble CFT-33-006 and the teaching context

- [ ] **Step 1: Write the complete operator-level assembly**

Split on `‖T‖≤1`; in the other branch obtain a norm-attaining unit vector and
the adjoint singular-vector equation, instantiate Equations (3) and (4), and
invoke the scalar endpoint. Include a hypothesis ledger for finite
dimensionality, completeness, nontriviality, commutation, power identities,
and uniform perturbation bounds.

- [ ] **Step 2: Add labeled historical and ML blocks**

Historical context records `EVIDENCE` for inspected artifacts and `SOURCE
CLAIM` for author statements, with exact locators and no priority claim. The ML
analogy maps the bounded recurrence to stability analysis of a recurrent
linearization, states the exact scalar transfer, denies transfer to stochastic
or time-varying Jacobians without added bounds, and asks the reader to compute
the drift diagnostic for a fixed nonnormal 2×2 Jacobian.

- [ ] **Step 3: Complete exercises and Lean links**

Add `exercise_06_solution` for the assembly theorem. Ensure all six theorem
cards link to compiler-reported local code positions and, for reexports, to
the exact provider files. Set all six rows to reconstructible/exact only after
the receipt passes.

- [ ] **Step 4: Publish and freeze the wave**

```sh
cargo test -p harp --test crouzeix_textbook chapter_33_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
```

Regenerate the six ledgers, corpus JSON, and Atlas HTML through the Wave 0
publisher. Refresh `docs/import-receipt.md` last. Commit canonical content,
formalization/contracts, generated publication, and repository receipt as
separate explicit commits. Wave 1 exits only after an editorial read can
reconstruct the proof without consulting the LS paper.
