# Crouzeix Textbook Wave 3 Lorist--Schwenninger Route Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the Lorist--Schwenninger branch in Chapter 34 and make Chapter 35 a truthful theorem-by-theorem comparison of Jin, Lorist--Schwenninger, and Harp.

**Architecture:** Chapter 34 constructs the concrete boundary dilation that satisfies Chapter 33's abstract data and proves the norm bound. Chapter 35 derives finite-matrix, rational, Hilbert-space, and spectral-set consequences, then compares the three verified terminal providers. Jin and Lorist--Schwenninger remain the two source-derived routes. Harp is labeled as a derived finite-horizon route. Compiler receipts, rather than prose similarity, determine provider imports.

**Tech Stack:** Markdown, Lean 4.32.1, existing `Crouzeix.LoristSchwenninger` and Jin terminal modules, Rust publisher, JSON v2 contracts, mise.

---

## Theorem inventory

| CFT range | Public declarations |
|---|---|
| CFT-34-001..006 | `boundary_dilation_data`, `boundary_compression_first_moment`, `boundary_compression_power_moments`, `boundary_multiplier_powers`, `boundary_multiplier_contractive`, `realization_norm_two` |
| CFT-35-001..006 | Preserve `lorist_schwenninger_main`, `lorist_schwenninger_finite_matrix`, `lorist_schwenninger_rational`, `lorist_schwenninger_hilbert`, `lorist_schwenninger_two_spectral_set`, and `jin_final_comparator`; add a checked three-provider terminal bundle and Harp consequence links without deleting the stable names. |

## Task 1: Freeze concrete-realization and comparison contracts

- [ ] Add RED tests for 12 unique CFT anchors, complete theorem cards, six
  distinct solutions per chapter, exact LS source locators, and compiler-valid
  code links.
- [ ] Assert the pedagogical graph sends Chapter 34 to CFT-33-006 and common
  machinery, never to CFT-30..32. Chapter 35 may reference Jin, LS, and Harp
  terminal theorems only in comparison rows.
- [ ] Assert the focused Jin, LS, and Harp kernel graphs match their compiled
  receipts. Reject an undocumented provider import. Do not infer independence
  from names or prose.
- [ ] Run focused tests, require RED, and commit as
  `test(crouzeix-textbook): freeze the LS realization route`.

## Task 2: Construct CFT-34-001 explicitly

- [ ] Define the boundary measure space, boundary coordinate multiplier,
  embedding, adjoint compression, projection, perturbation, and uniform bound.
  State all measurability, boundedness, finite-dimensionality, and Cauchy
  moment hypotheses before packaging `DilationData`.
- [ ] Show field by field how
  `LoristSchwenninger.dilationDataOfParametricPolynomial` satisfies Chapter
  33's abstract structure. Include a finite atomic boundary model as a worked
  text-only calculation.
- [ ] Preserve `CrouzeixTextbook.Part06.boundary_dilation_data`; use
  `reexported-proof` only if its exact type matches the displayed construction.
- [ ] Add distinct exercise solutions 01 and 02 for checking the isometry and
  compression fields.

## Task 3: Prove moment compression, CFT-34-002 and 003

- [ ] Expand the first-moment calculation under the boundary integral. Identify
  the Cauchy moment that survives and why the other modes vanish.
- [ ] Prove the power-moment identity by induction. Display the induction base,
  multiplier-product step, use of `bcfMulL_pow`, and compression equality.
- [ ] Explain exactly why a first-moment identity alone is insufficient for the
  perturbation lemma and where all positive powers are used.
- [ ] Add distinct solutions 03 and 04, one for a scalar Fourier moment and one
  for the Lean induction bridge.
- [ ] Verify:

```sh
cargo test -p harp --test crouzeix_textbook chapter_34_moments_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
```

## Task 4: Prove multiplier and realization bounds, CFT-34-004..006

- [ ] Derive `(M_h)^k=M_{h^k}` pointwise and explain equality of bounded
  continuous-function multipliers with continuous linear maps.
- [ ] Prove `‖M_h‖≤1` from the boundary sup norm. State whether equality is
  needed and include a strict-contraction boundary case.
- [ ] Instantiate Chapter 33's perturbation lemma with the concrete dilation,
  identify its target as the normalized polynomial evaluation, and undo the
  normalization to obtain the displayed factor-two inequality.
- [ ] Add distinct solutions 05 and 06 for multiplier contractivity and the
  final theorem assembly.
- [ ] Add the four-field ML analogy: boundary multiplication ↔ diagonal action
  in a lifted feature space; exact compression identity transfers; learned or
  approximate features do not supply exact moments; calculate empirical
  moment residuals for `A_{λ,α}` as the diagnostic.
- [ ] Verify and commit Chapter 34:

```sh
cargo test -p harp --test crouzeix_textbook chapter_34_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
git add knowledge/crouzeix_textbook/part_06_constant_two_routes/34_lorist_schwenninger_realization.md \
  formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean content/crouzeix_textbook
git commit -m "docs(crouzeix-textbook): realize the LS boundary dilation"
```

## Task 5: Derive the Chapter 35 LS consequences

- [ ] CFT-35-001 states the normalized main theorem with every parameter and
  identifies Chapter 34 as its proof provider.
- [ ] CFT-35-002 specializes to finite matrices and records the finite-index
  equivalence explicitly.
- [ ] CFT-35-003 proves the rational spectral-set corollary, including the
  pole-free domain and normalization.
- [ ] CFT-35-004 explains finite-dimensional matrix representation inside a
  Hilbert space theorem without suggesting an unproved infinite-dimensional
  extension.
- [ ] CFT-35-005 packages the closed numerical range as a two-spectral set and
  spells out the equivalence between the set-valued statement and norm bound.
- [ ] Add exact wrappers or reexports and
  `Exercises.Chapter35.exercise_01_solution` through `exercise_05_solution`.

## Task 6: Build the theorem-by-theorem three-route comparison

- [ ] Add a textbook theorem that bundles the three provider proofs of the
  normalized finite-matrix statement. Its body must refer to the Jin, LS, and
  Harp terminal declarations so the compiler receipt records all three. Keep
  the six existing public declarations as compatibility names.
- [ ] For CFT-35-006, compare objects, hypotheses, common trunk, decisive
  mechanism, approximation step, terminal conclusion, provenance, and formal
  provider in a text table. Do not express comparison as a proof dependency.
- [ ] Keep `jin_final_comparator` as an exact reexport of the Jin endpoint and
  add explicit code links for the LS and Harp endpoints. Add
  `exercise_06_solution` proving equality of the normalized conclusion
  propositions or another bounded comparison lemma, not any terminal theorem.
- [ ] Historical context records the Jin and LS source identities and all
  three local reproduction states. It labels Harp as derived and makes no
  unsupported priority or acceptance claim.
- [ ] The ML section contrasts positive completion certificates, full boundary
  dilation certificates, and finite-horizon atomic certificates. It gives
  concrete PSD and moment-residual diagnostics and states that no finite
  residual test proves the exact infinite family.
- [ ] Verify and commit Chapter 35:

```sh
cargo test -p harp --test crouzeix_textbook chapter_35_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
git add knowledge/crouzeix_textbook/part_06_constant_two_routes/35_comparison_verification_and_boundaries.md \
  formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean content/crouzeix_textbook
git commit -m "docs(crouzeix-textbook): compare three verified constant-two routes"
```

## Task 7: Publish and freeze Wave 3

- [ ] Regenerate publisher outputs and review every theorem code link.
- [ ] Run:

```sh
cargo test -p harp --test crouzeix_textbook ls_route_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
scripts/check_lean_library.sh CrouzeixHarp
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
```

- [ ] Refresh `docs/import-receipt.md` last and commit it separately. Wave 3
  exits only when all 12 rows are reconstructible/exact, an editorial reader
  can assemble the LS route from Chapters 33 through 35, and compiler-derived
  receipts report the Jin, LS, and Harp provider relationships truthfully.
