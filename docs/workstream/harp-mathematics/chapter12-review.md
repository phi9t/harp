# Chapter 12 oriented-boundary record

Date: 2026-09-10. Second chapter of Package 3 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md).
Completes the foundations wave: Chapters 1-12 now all carry exact Lean
correspondence and checked exercise solutions.

**Review status: independently reviewed before landing.** This is the first
chapter written under the amended process — the reviewers ran on the finished
chapter *before* it was committed, rather than after it was pushed. Two
reviewers, one on mathematical correctness and one on prose-to-Lean
correspondence, neither with access to this directory, neither permitted to
build Lean. Kata `r60f` is closed by this record.

## Why the chapter is short on Lean and long on disclosure

Five of the six cards were already `reexported-proof` rows over genuine
multi-step `CrouzeixConjecture` proofs, so no card needed reproving — a contrast
with Chapter 11, where four of five providers turned out to be `rfl`. The work
was six exercise solutions, the prose, and one structural repair.

The repair: **CFT-12-001 could not be promoted to a re-export.** Its provider
`complexRealInner_eq_re_mul_conj` is literally `:= Complex.inner z w`, an exact
eta-alias, which the receipt classifies as `direct-alias`; the validator requires
a `reexported-proof` row's underlying declaration to be theorem-kind. That is why
the card had been registered as a bare `checkpoint` with no underlying
declaration, and the first attempt to promote it failed closed. It is now
`proved-here`, stating the real-coordinate form
`⟪z,w⟫_ℝ = z.re·w.re + z.im·w.im` and deriving it in three tactics — which is
also the form the card's title names.

The chapter's scope gap is larger than any so far and is disclosed at the top
rather than buried: **Stokes' theorem is not compiled in this book**, nor the
exterior derivative, pullback, the wedge product, or triangulation cancellation.
What is compiled is the planar orientation infrastructure Part V consumes. The
earlier sketch promised five things the compiled layer does not reach; all five
are withdrawn by name.

## Independent review findings

Three blocking defects, and the two reviewers found the first two independently
of each other.

1. **The chapter claimed all six cards were re-exports.** Written before
   CFT-12-001 was changed to `proved-here` and not updated afterwards; three
   separate sentences carried it, each contradicting CFT-12-001's own card.

2. **CFT-12-003's orthogonality step cited the wrong identity, and its minus sign
   came from nowhere.** The prose attributed the step to the "left-hand"
   quarter-turn identity `⟪iz,w⟫_ℝ = ω(z,w)`, which carries no sign and does not
   even pattern-match the term being converted. The provider actually uses
   `complexRealInner_I_mul_right : ⟪z, iw⟫_ℝ = -ω(z,w)` — a *third* identity that
   the chapter never stated, and the source of the only minus sign in its central
   computation. A reader following the citation literally would derive the
   opposite sign. The error propagated to CFT-12-002's boundary case and to E01's
   solution, both of which claimed the radial argument used each recorded identity
   once; the left-hand one is used by no proof in the chapter. The card's own
   receipt line had named the right lemma all along, which is how both reviewers
   caught it.

3. **The claim that the conjugation convention drives the chapter's signs is
   false.** Three passages said reversing which argument is conjugated would
   reverse every orientation conclusion. It would not:
   `Re(w z̄) = Re(z w̄)` identically, verified over ten thousand random pairs, and
   the card's own coordinate form is visibly symmetric. The area form comes from
   `Complex.orientation` and is not derived from the inner-product convention at
   all. What would reverse the conclusions is reversing the orientation.

Also repaired: a broken navigation link to a Chapter 13 path that does not exist;
"all six hypotheses are used" restated as used-by-this-proof, with the two
unit-norm hypotheses identified as unnecessary for CFT-12-003 though necessary
for CFT-12-004; "exactly `C^1`" for a map that is smooth; a `C^1` diagnostic
pointing at the inverse chart's blow-up rather than the compiled forward map; a
false claim that no higher-dimensional rotation satisfies the quarter-turn
identity; the radial slice's unit speed wrongly said to "match" CFT-12-004's
`‖v‖`; an absolute value the provider never forms; the two-variable polar
derivative asserted without an uncompiled label; a third provider module omitted
from the inventory; and six exercise `skills` rows in the registry that had been
carried over mechanically from the withdrawn sketch and disagreed with the
prose's own cross-references in all six cases.

## What the reviewers verified as correct

Both recomputed the sign conventions from Mathlib source rather than taking them
on trust: `Complex.inner`, `Complex.areaForm w z = (conj w * z).im`, and all
three quarter-turn identities including the unstated third. The `ring`
rearrangement at the heart of CFT-12-003 was expanded by hand and matches the
provider exactly. Every worked instance was recomputed — the circle example, the
`ω(1,1) = 0` degenerate case (confirmed to be a genuine rather than vacuous
counterexample), the polar derivative, the exact radial increment. All twelve
line links land on their declarations, and all twelve statements match their Lean
types in quantifier structure, hypothesis count and order, and conjunct count.

## Verification

`mise run crouzeix-textbook-publication` reports "matches canonical inputs".
Contract counts moved to 69 `proved-here`, 26 `checkpoint`, 144 exact rows and
144 solved exercises, with a 471-declaration receipt. The foundations test was
restated from a pending-boundary assertion to a completion invariant: all 72
Chapters 1-12 cards exact, all 72 exercises solved, with the four deliberate
forward-reference previews and two definition rows carved out by name.

## Process note

This is the first chapter where the independent pass ran before anything was
pushed, and it worked as intended: three blocking defects, including a sign error
in the chapter's central computation, were fixed with nothing untrue ever
reaching the repository. The two previous passes each found five blocking defects
in prose that had already been pushed.
