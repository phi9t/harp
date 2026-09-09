# Chapters 5 and 6 algebra-package record

Date: 2026-09-08. Package 1 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md),
implemented on local `master` on top of `d55d611d`.

**Review status: author self-review only.** The independent specification and
content-quality reviews that Chapters 1–4 received have not been run for this
package, and no delegation was authorized for it. Kata `a2ry` and `9hxf`
therefore stay open. Everything below records what the author checked and what
the machine checked; nothing here is a second opinion.

## What landed

Twelve cards moved from `unmapped` or `checkpoint` to `exact` correspondence
with `reconstructible` exposition, and twelve exercises gained distinct checked
solutions. Both chapters were rewritten from roughly 140-line summaries to the
accepted foundations standard.

Chapter 5 adds eleven support declarations: one-row additivity and homogeneity,
alternation, antisymmetry, the Leibniz expansion, transpose invariance, the
unit and field invertibility criteria, the exact two-by-two expansion
`det(I + tA) = 1 + t·tr A + t²·det A`, the top-degree exterior action, trace
linearity, and a compiled witness that trace is not multiplicative. It also
computes both invariants for the cumulative running family.

Chapter 6 adds fourteen: the characteristic-root spectrum test,
Cayley–Hamilton, minimal-polynomial divisibility, degree reduction modulo the
characteristic polynomial, commutativity of the generated algebra, eigenvector
transport through powers and through arbitrary polynomials, two-sidedness of a
one-sided matrix inverse, conjugation transport for powers and for polynomial
actions, diagonal evaluation over an arbitrary field, Lagrange interpolation
at distinct nodes, and the two boundary examples.

The public card names, anchors, and statements are unchanged. Chapter 6's six
cards are now declared `reexported-proof` with their maintained
`CrouzeixConjecture` provider named, which is what they always were; the
previous `checkpoint` mode left the alias target unexamined by the validator.

## Machine evidence

- `lake --try-cache build CrouzeixTextbook.Part01.Chapter05` and `…Chapter06`
  each complete with no errors and no warnings under `leanprover/lean4:v4.32.1`.
- `scripts/check_lean_library.sh CrouzeixTextbook` builds 3722 jobs and reports
  `outcome=passed`. Two consecutive receipt emissions are byte-identical.
- The receipt classifies all twelve new exercise solutions as `theorem`, not
  `direct-alias`, so the protected-target check in `ExportReceipt.lean` accepts
  them. All six Chapter 6 cards classify as `direct-alias` with exactly one
  maintained direct dependency, matching their declared underlying provider.
- All 108 solution type fingerprints are pairwise distinct and none collides
  with a public card fingerprint.
- `harp crouzeix-textbook check` passes, then
  `mise run crouzeix-textbook-publication` publishes and re-checks six ledgers.
- `cargo test -p harp --test crouzeix_textbook --test foundations_contract
  --test foundations_narrative`: 296 + 9 + 14 passing, 0 failing.
- Receipt identity is now `c367dd6e…`, 513498 bytes, 435 declarations: 216
  public cards, 108 distinct solutions, 111 underlying providers.

## Defects found and fixed

### First pass: statement correspondence

1. The Chapter 5 multiplicativity proof asserted alternation on the *rows* of
   `B`. That is false as stated: equal rows of `B` do not give equal rows of
   `AB`. The argument was rerun on columns, where column `j` of `AB` is `A`
   applied to column `j` of `B`.
2. The Chapter 5 E01 solution mis-stated the two two-by-two determinants.
3. CFT-06-002 and CFT-06-003 omitted that `SimpleDiagonalization` carries
   injectivity of the eigenvalues, understating the hypothesis while claiming
   exact correspondence.

### Second pass: prose proofs against the actual Lean proofs

The requirement is that every displayed proof narrates a checked derivation
and that the reader is told which one. Auditing each card against the proof
term its provider actually uses produced five more repairs.

4. **CFT-06-006 described the wrong argument.** The prose said the
   *discriminant* of the characteristic polynomial along the perturbation
   segment is a nonzero polynomial in the parameter. The maintained proof uses
   the **resultant** of the characteristic polynomial and its derivative, and
   passes through separability and coprimality rather than a discriminant. It
   also uses one specific perturbation target — the diagonal matrix with
   entries `0, 1, …, n-1` — not an arbitrary distinct diagonal, and it takes
   `δ = ε / (‖Δ - A‖ + 1)` where the `+1` is what keeps the bound usable when
   `A = Δ`. The proof was rewritten to those three stages and now names
   `simpleSpectrumBadPolynomial_ne_zero`,
   `exists_small_simpleSpectrumParameter`, and
   `hasDistinctEigenvalues_of_badPolynomial_eval_ne_zero`.
5. **CFT-06-003 attributed the wrong step.** The prose derived
   `χ_B = χ_D` from Chapter 5's determinant similarity invariance. The provider
   uses `Matrix.charpoly_units_conj` directly, which is CFT-01-005's provider.
   The proof was rewritten as the provider's three named steps, and
   CFT-06-003's pedagogical prerequisites were corrected from `[CFT-06-002]` to
   `[CFT-01-005, CFT-06-002]`. That change moves the pinned 216-row roster
   digest, which was repinned.
6. **CFT-05-001's displayed proof had no compiled counterpart.** The book
   proves multiplicativity from the rank-one characterization; the provider
   `Matrix.det_mul` expands Leibniz sums instead, so nothing checked the
   displayed argument. `determinant_multiplicative_via_alternating` was added
   to check it. It carries a caveat that is now stated in the chapter: the
   library lemma it leans on, `Module.Basis.det_comp`, is itself proved from
   `Matrix.det_mul`, so the compiled version is a reformulation and not a
   logically independent second proof. Claiming otherwise would have been the
   more serious error.
7. **Three more cards displayed arguments their providers do not run.**
   `Matrix.det_units_conj` commutes factors under the determinant with
   `det_mul_right_comm` instead of forming three scalars; `Matrix.trace_mul_comm`
   goes through transposes instead of the double sum; `Matrix.trace_units_conj`
   uses the three-factor `trace_mul_cycle`. Each card now points at the local
   declaration that does check the displayed argument — E03, E04, E06
   respectively — and states the provider's different route.
8. **Two generalizations were stated as though checked.** The `n`-dimensional
   form of `det(I + tA) = 1 + t·tr A + …` and the bound
   `dim⟨A⟩ ≤ deg m_A` are both true and neither is compiled here; the compiled
   statements are the dimension-two identity and the bound by `deg χ_A`. Both
   are now labeled. The Chapter 5 E02 solution also narrated a permutation
   argument its checked proof does not run, and now says what the proof does.

### Second pass: exercise strength

9. Chapter 6 E02 asked for the polynomial action `p(J)` but checked only the
   matrix power `J³`. It now states both and derives the first from the second
   through `map_pow`.
10. Chapter 6 E03 and E05 were pure repackagings of same-file support lemmas.
    E03 now also derives that an eigenvector is annihilated when the polynomial
    vanishes at its eigenvalue; E05 now also identifies the residual data
    explicitly as `(X - 2)(J) = N`, checked entrywise.

Every Lean line link in Part I was checked mechanically against the
declaration it names; all resolve.

## Statement-by-statement correspondence check

Each of the twelve card statements was compared against the normalized type in
the fresh receipt, including the typeclass hypotheses. The chapter's boundary
paragraphs state exactly the classes the declaration carries: `CommRing` for
the three determinant cards, `AddCommMonoid` with `CommMagma` for trace
cyclicity, `CommSemiring` for trace similarity, `AddCommMonoid` alone for the
diagonal trace, and the fixed complex scalars for all six Chapter 6 cards.
Defect 3 above was found by that comparison.

## The four previews

`CFT-01-005`, `CFT-04-004`, `CFT-04-005`, and `CFT-04-006` stay `summary`.
Their derivations now exist in Chapters 5 and 6, and each preview links to the
owning card. Importing those proofs backwards would make Chapter 4 depend on
Chapter 5, so they remain labeled forward references outside the
completed-proof count. `chapters_01_through_06_publish_exact_cards_with_explicit_proof_boundaries`
pins that decision.

## Incidental assurance change

`ExportReceipt.lean`'s protected-target list covered Chapters 1–4 and 25–35 but
not Chapter 36, so Chapter 36's six accepted solutions could have become eta
aliases without failing the build. The twelve new solutions and those six were
added together; all eighteen pass the `theorem`-kind requirement. This is
outside Package 1's scope and is recorded here rather than left silent.

## What this package does not establish

Compiler acceptance establishes the recorded Lean declarations in the pinned
environment and nothing beyond it. No independent mathematical or
correspondence review was performed. The ML-bridge sections are labeled
illustrations, not proved claims about any implementation. Chapters 7–24 remain
untouched, with 108 unsolved exercises and 108 non-exact rows; see
[textbook-backlog-inventory.md](textbook-backlog-inventory.md).
