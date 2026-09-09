# Chapter 8 Gram-positivity record

Date: 2026-09-09. Second chapter of Package 2 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md),
on top of the Chapter 7 landing.

**Review status: author self-review only**, as for
[Chapters 5 and 6](chapter05-06-review.md) and
[Chapter 7](chapter07-review.md). Kata `tdga` stays open for the independent
specification and content-quality reviews its acceptance criteria require.

## What landed

Six cards moved to `exact` correspondence with `reconstructible` exposition and
six exercises gained distinct checked solutions. The prose went from 124 lines
to a full development.

Five of the six cards were already `reexported-proof` rows naming
`CrouzeixConjecture` providers, so only their prose and status changed.
CFT-08-001 was a `checkpoint` alias into `MathematicalFoundations` — the
namespace the receipt exporter does not treat as maintained, first hit in
Chapter 7 — and is now proved locally.

New support declarations: the real Gram quadratic form and its nonnegativity
proved from Chapter 7's pairing axioms, a compiled semidefinite-but-not-definite
witness, a compiled entrywise-positive-but-indefinite witness, the Gram matrix
unfolding, and the three square-root conclusions the Part VI completion
argument consumes.

## The honest reading of CFT-08-001

Its checked proof is `exact hPositive x hx` — the conclusion is the hypothesis
instantiated at the supplied vector, and the symmetry hypothesis is never used.
The card records the *shape* of the positive-definiteness interface the later
chapters expect; it proves no new inequality. The prose says exactly that
rather than dressing an instantiation up as an argument, and points the reader
at the Gram cards where positivity is actually derived.

This mattered for the local reproof too: the declaration keeps the unused
symmetry binder so the interface still matches, and the prose flags that the
binder is unused.

## Every proof narrates a checked derivation

- CFT-08-002's provider is the single library result
  `Matrix.posSemidef_conjTranspose_mul_self`. The prose gives the
  move-a-factor-across-the-pairing argument the library packages, and the real
  case is separately proved here so the two steps are visible.
- CFT-08-003's provider unfolds the Gram matrix and applies `IsUnit.star` then
  `IsUnit.mul`. Chapter 5's determinant criterion would give an alternative
  route; the maintained proof does not take it, and the prose follows the proof
  that exists rather than the one a reader might expect.
- CFT-08-004's provider composes CFT-08-002 and CFT-08-003 through
  `PosSemidef.posDef_iff_isUnit`. The prose says the equivalence carries the
  null-vector argument and the card assembles the inputs.
- CFT-08-005's provider is one application of
  `PosSemidef.conjTranspose_mul_mul_same`.
- CFT-08-006's provider discharges five structure fields using `CFC.sqrt`,
  `CFC.isUnit_sqrt_iff` and `Matrix.nonneg_iff_posSemidef`, and the prose lists
  them in that order.

Mechanically checked: all twelve card and exercise regions cite at least one
compiled declaration, and every Lean line link resolves to the declaration it
names.

## Three meanings of "positive", compiled

The plan's boundary obligation for this chapter is "PSD versus positive
definite versus entrywise positivity". Both separations are compiled rather
than asserted: a singular generator gives a Gram matrix that vanishes on a
nonzero vector, and a matrix with all entries positive has a strictly negative
quadratic form at `(1,-1)`. The chapter never uses the word "positive"
unqualified.

## What this does not establish

No independent review was performed. Uniqueness of the nonnegative square root
is true and is not claimed by CFT-08-006, which supplies existence with five
listed properties. The ML-bridge section is explicit that a floating-point
kernel matrix is routinely not positive semidefinite and that exact
invertibility says nothing about conditioning. Chapters 9–24 remain, with 96
unsolved exercises and 96 non-exact rows.
