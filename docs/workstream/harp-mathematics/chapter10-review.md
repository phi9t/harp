# Chapter 10 multilinear-and-tensor record

Date: 2026-09-09. Fourth chapter of Package 2 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md),
and the first landed under the amended plan's obligation 8.

**Review status: author self-review only.** Kata `0wc6` stays open for the
independent reviews, per the plan's *Review authority* section.

**Superseded 2026-09-09:** an independent review of Chapters 5-10 has since
run and is recorded in
[the Chapters 5-10 independent review](chapters05-10-independent-review.md).
It found defects this self-review missed, including blocking ones; read the two
records together. The Kata issue named above is closed by that record.


## The roster finding

**Half of this chapter's indexed cards are renamed Chapter 5 cards.** CFT-10-003,
CFT-10-004 and CFT-10-005 re-export `determinant_multiplicative`,
`determinant_diagonal` and `trace_cyclic`, and carry type fingerprints identical
to CFT-05-001, CFT-05-002 and CFT-05-004 respectively.

This differs from the Chapter 9 duplication in one respect that matters: these
three name their `underlying_declaration`, so the validator checks the alias
identity rather than accepting an unexamined checkpoint. The duplication is
declared, not hidden by the machinery.

It is now also declared to the reader. The chapter opens its formal development
with a disclosure saying which three cards restate Part I, why they are indexed
again — the multilinear reading is what the chapter teaches — and that the
chapter's own mathematical content is in the support declarations. The honest
summary, which the prose states, is that half this roster restates Part I.

## The unmaintained-namespace wall, a fourth time

CFT-10-001 and CFT-10-002 were checkpoint aliases into `AutodiffGeometry`,
another namespace the receipt exporter does not treat as maintained. Both are now
proved locally. CFT-10-001's local proof deliberately reproduces the retired
provider's own argument — expand both pairings into double sums, interchange with
`Finset.sum_comm`, commute the products — so the prose narrates a derivation that
is checked rather than one that used to be checked somewhere else.

## The eta-alias protection, a fourth time

E04 first elaborated to an exact eta alias of the support lemma and was rejected.
The fix improved the exercise: its task says *derive* the transformation law, so
the solution now performs the derivation rather than delegating to a lemma that
already contains it.

## New content beyond the cards

The tensor product enters through its universal property — every bilinear map
factors uniquely, computing on pure tensors — over an arbitrary commutative ring
with no basis and no finiteness. Against it the chapter sets the reshape: a
bijection of index sets and nothing more.

The distinction is compiled rather than asserted. **The identity matrix is an
array of the product shape and is not a pure tensor**, because every array
`x i * y j` has vanishing two-by-two determinant while `det I = 1`. The proof
uses Chapter 5's determinant to detect rank, which is the first time the book
uses an earlier chapter's invariant as a diagnostic rather than as a result.

Contraction is defined so that the types carry which axis is summed, and
contracting an outer product over the shared axis is proved to be matrix
multiplication. Basis dependence is congruence, connecting to Chapter 8's
stability result, and congruence is separated from similarity by a compiled
witness: the running shear is not orthogonal, so its transpose is not its
inverse.

## What this does not establish

No independent review. The chapter states the universal property and its
computation rule but does not develop the tensor product's construction or
prove uniqueness of the factorization. `contractMiddle` is defined here for the
shape-checking point and is not a general tensor-contraction API. The ML-bridge
section is explicit that a reshape carries no algebra, that broadcasting is not
multilinearity, and that naming an axis does not make a transformation law hold.
