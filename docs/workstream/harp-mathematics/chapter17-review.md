# Chapter 17 rational functional calculus record

Date: 2026-09-12. Seventh chapter of Package 3 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md).

**Review status: independently reviewed before landing**, the sixth chapter under
the amended process. Two reviewers, non-overlapping briefs: mathematical content
against the provider sources and Mathlib, and contract/disclosure against the plan's
obligations.

## What the chapter is

The rational functional calculus: a rational function's pole set through Mathlib's
canonical reduced denominator, its finiteness, the openness of its complement, the
pole-freeness predicate, and scalar and matrix evaluation. Four of the six cards are
definitions; two are one-line theorems.

The withdrawn sketch was title-driven for the third chapter running. It promised the
Jordan-block calculus, the resolvent identity, the spectral mapping theorem and
operators on infinite-dimensional spaces, none of which is compiled, and it defined
`f(A)` by the contour integral — the definitional contradiction with Chapter 16 that
Chapter 16's reviewers had flagged. The Lean defines the rational calculus
algebraically, `p(A) q(A)^{-1}` through polynomial evaluation, and never touches the
holomorphic limit.

Two theorems the chapter cannot be understood without are not among its six cards.
`polynomialEval_denom_isUnit_of_rationalPoleFreeOn_numericalRange` makes the matrix
evaluation meaningful, by spectral mapping and the containment of the spectrum in
the numerical range — a forward reference to CFT-20-006, disclosed as such. And
`holomorphicMatrixEval_rational` identifies this calculus with Chapter 16's on
pole-free rational functions; it is proved from CFT-16-003, CFT-16-004 and
CFT-16-006 and is where those laws are actually consumed. Both are narrated at length.

Finding that bridge theorem corrected Chapter 16, which had said reconciling the two
calculi was "outstanding work" and that Chapter 17 defines `f(A)` by a contour
integral — true of the Chapter 17 *sketch*, false of the Lean. Four passages of
Chapter 16 were rewritten in this change set and were covered by the specification
review.

## Contract changes

CFT-17-001, CFT-17-004, CFT-17-005 and CFT-17-006 shipped as checkpoints with no
underlying declaration. All four alias `def`s in the maintained `CrouzeixConjecture`
namespace and are now `definition` rows naming their providers; CFT-17-005 and
CFT-17-006 also had `kind: theorem` corrected to `definition`. The receipt went from
503 to 513 declarations: six exercises and four newly named providers.

Two prerequisite edges were removed by reading the definitions: CFT-17-004 → CFT-13-001
(the maximum modulus on a compact set plays no part in `Disjoint (poleSet) s`) and
CFT-17-006 → CFT-16-001 (`rationalMatrixEval` never unfolds `holomorphicMatrixEval`;
the calculi meet in the bridge theorem, not in the definition). The chapter's
cross-chapter set is now `{6}`, down from `{6, 13, 16}`. The roster digest moved from
`99d8dc09…` to `317cb1a7…`, verified: the previous contract reproduces the old pin and
exactly the two intended rows differ. The specification reviewer was asked whether a
Chapter 16 edge should be kept for pedagogy and advised against it: it would
reintroduce the false dependency the removal fixed, and the Previous/Next links carry
the reading order.

Exercise `skills` were set to the cards each solution invokes — E02 to CFT-17-006 and
E03 to CFT-17-003 and CFT-17-004 — replacing the mechanical `E0k → CFT-17-00k` mapping.

## Independent review findings

**Two blocking, twelve minor across the two reviewers.**

1. *The wrong law.* The opening problem said the bridge theorem is where Chapter 16's
   "additivity, multiplicativity and locality" are consumed. The bridge uses
   `holomorphicMatrixEval_mul`, `holomorphicMatrixEval_congr_on_neighborhood` and
   `holomorphicMatrixEval_polynomial`; the additive law never appears, and the chapter
   stated the correct triple in two other places, contradicting itself. Corrected.

2. *A false topological claim.* CFT-17-003's boundary case said the complement of a
   finite set "is not in general" connected, then exhibited it being connected.
   `\mathbb{C}` minus a finite set is always path-connected. Rewritten to say so and
   that nothing here uses it.

Minor, all fixed: the `rfl` that unfolds scalar evaluation is the *first* step of the
bridge's closing calculation, not the last (asserted twice); Example 3 wrote the
denominator matrix of `1/z` at a singular `A` as `0` rather than `A`; Example 1 called
the resolvent "the one instance" of a closed form when every polynomial and every
pole-free rational function has one; "it does not" was asserted where "it need not"
holds, since `q(A)` is singular exactly when `q` vanishes on the spectrum, and the
packet's spectrum-level theorem is stated through nonvanishing rather than the
pole-freeness predicate; "the only place in the maintained packet" locality is invoked
overlooked the textbook's own CFT-16-E04; the bridge uses both forms of pole-freeness,
not only containment; CFT-06-004 was credited with placing the inverse factor in the
algebra generated by `A`, which needs the uncompiled Cayley–Hamilton remark;
CFT-17-E04 claimed the engine and bridge rely on monotonicity when neither invokes it;
the E02 disclosure undercounted its steps; E03 lacked a disclosure though it is a bare
triple of maintained lemmas; CFT-21-005 was described as a route card when it is the
Part IV Hilbert-space spectral-set statement; and the forward reference to CFT-20-006
now notes that the card's correspondence is not yet accepted.

## What the reviewers confirmed

Receipt fidelity exact on all twelve prose blocks and both JSON contracts. All ten
subsections on all six cards. Every mode/kind/status combination admitted by the
contract. Both edge removals verified from the provider definitions and imports. The
running example's reduced presentation (`1` over `X - C λ`), the totalization claims
for scalar division and the matrix inverse, the uniqueness-of-reduced-form sketch, the
engine narration line by line, and the bridge narration step by step — including that
it never uses the engine and obtains invertibility as a right inverse through
`Matrix.inv_eq_right_inv`. Every named Lean lemma exists and does what is claimed. The
four Chapter 16 corrections are accurate and no stale sentence about Chapter 17 remains.

## Verification

Final landing read also narrowed three introductory claims: the algebraic formula
does not prescribe explicitly forming an inverse numerically; failure of the
numerical-range pole-free condition does not force a singular denominator; and
exclusion from the numerical range is sufficient, not necessary, for the resolvent
to exist. These corrections preserve the statements and proofs reviewed above.

A further independent landing review found the same overstatement in the ML bridge
and invalid reverse implications in the diagnostics. The repaired text distinguishes
near-pole conditioning from pole finiteness, large gain from pole proximity, and a
valid zero output from singular-denominator totalization. It also makes explicit
that neither totalized result type certifies validity and that exact matrix
semantics supply no floating-point error bound. The initial full gate was stopped
after these findings so the final gate could run against the corrected candidate.

`harp crouzeix-textbook check` reports "matches canonical inputs" against a fresh
513-declaration receipt. Counts moved to 69 `proved-here`, 117 `reexported-proof`,
14 `checkpoint`, 16 `definition`, 174 exact rows and 174 solved exercises.
