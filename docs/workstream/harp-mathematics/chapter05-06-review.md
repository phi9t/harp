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
- Receipt identity is now `c4395c13…`, 512174 bytes, 435 declarations: 216
  public cards, 108 distinct solutions, 111 underlying providers.

## Defects the author found and fixed

Three, all in prose, none in the compiled statements:

1. The Chapter 5 multiplicativity proof asserted alternation on the *rows* of
   `B`. That is false as stated: equal rows of `B` do not give equal rows of
   `AB`. The argument was rerun on columns, where column `j` of `AB` is `A`
   applied to column `j` of `B`, and the transpose duality was moved into the
   conceptual model so the column form is available before it is used.
2. The Chapter 5 E01 solution mis-stated the two two-by-two determinants.
3. The Chapter 6 CFT-06-002 and CFT-06-003 statements listed the eigenvalue
   family, the change-of-basis unit, and the conjugation identity, but omitted
   that `SimpleDiagonalization` also carries injectivity of the eigenvalues.
   That understated the hypothesis while claiming exact correspondence. Both
   statements now name it, and both record that neither the displayed
   derivation nor the maintained provider consumes it.

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
