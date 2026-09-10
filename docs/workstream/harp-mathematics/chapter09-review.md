# Chapter 9 operator-norm record

Date: 2026-09-09. Third chapter of Package 2 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md),
on top of the Chapter 8 landing.

**Review status: author self-review only**, as for
[Chapters 5 and 6](chapter05-06-review.md), [Chapter 7](chapter07-review.md)
and [Chapter 8](chapter08-review.md). Kata `wstr` stays open for the
independent specification and content-quality reviews.

**Superseded 2026-09-09:** an independent review of Chapters 5-10 has since
run and is recorded in
[the Chapters 5-10 independent review](chapters05-10-independent-review.md).
It found defects this self-review missed, including blocking ones; read the two
records together. The Kata issue named above is closed by that record.


## What landed

Six cards moved to `exact` correspondence with `reconstructible` exposition and
six exercises gained distinct checked solutions. The prose went from 128 lines
to a full development.

Five cards were already `reexported-proof` rows naming `CrouzeixConjecture`
providers. CFT-09-006 was a `checkpoint` alias of `norm_add_le` and is now
proved locally; a bare alias of a non-maintained library lemma classifies as
`direct-alias` and cannot carry a local-proof claim, the same wall met in
Chapters 7 and 8.

## A duplication found in the card roster

**CFT-09-001 and CFT-07-005 re-export the same provider.** Both name
`CrouzeixConjecture.matrix_norm_eq_euclidean_operator_norm`, so both carry the
same type fingerprint: two indexed cards, one theorem. This is not new
mathematics in Chapter 9.

The card identities are frozen by the contract, so the roster was not adjusted.
Instead Chapter 9's boundary paragraph states the duplication plainly and gives
the reason the card is indexed again — Chapter 9 is where the identity is used,
since every estimate in the chapter is stated for the matrix norm and proved
through the operator norm. The backlog inventory records it too.

## Every proof narrates a checked derivation

- CFT-09-001's provider is `rfl`; the prose says the two sides denote the same
  number by definition.
- CFT-09-002's provider rewrites the unitary-membership criterion, distributes
  the conjugate transpose with the self-adjointness field of the square-root
  data, regroups with `noncomm_ring`, substitutes the squaring field and cancels
  with the two inverse fields. The displayed calculation follows that order.
- CFT-09-003's provider substitutes the unitary conjugation and strips the two
  unitary factors with `CStarRing.norm_mem_unitary_mul` and
  `norm_mul_mem_unitary`; both are recorded here as a support declaration so the
  reader can see the step named.
- CFT-09-004's provider is two rewrites by CFT-09-001 followed by CFT-09-003,
  and the prose says exactly that.
- CFT-09-005's provider rewrites by CFT-09-001 and delegates to the
  operator-level statement; the prose says the positivity-to-bound step lives in
  the maintained positivity development, not in this card.
- CFT-09-006's provider is `norm_add_le`.

## New content beyond the cards

The two halves of the least-upper-bound interface are proved separately
(`matrix_norm_mulVec_bound`, `matrix_norm_le_of_bound`), because later chapters
use them in opposite directions. Submultiplicativity, the C\*-identity, both
unitary invariances and the null-vector characterization of singularity are
recorded as named support declarations.

The chapter's central boundary result is quantitative rather than qualitative.
For the nilpotent shear `N_α = !![0, α; 0, 0]` the Gram matrix is computed
explicitly as `diagonal ![0, conj α * α]`, its induced norm is read off with
`Matrix.l2_opNorm_diagonal`, and the C\*-identity then gives
**`‖N_α‖ = ‖α‖` exactly**, while nilpotency and a vanishing determinant give
spectral radius zero. A lower bound would have shown the spectrum is not an
upper bound for the norm; the equality shows the gap is exactly the parameter
the spectrum discards. That is the compiled justification for the rest of the
book's refusal to bound `‖p(A)‖` by `p` on the spectrum.

Mechanically checked: all twelve card and exercise regions cite at least one
compiled declaration, and every Lean line link resolves.

## What this does not establish

No independent review was performed. The chapter does not claim uniqueness of
the polar decomposition, only that the given construction yields a unitary
factor. The ML-bridge section states plainly that a composition bound built from
spectral radii is not a bound at all, and that a numerically computed largest
singular value is unreliable for a near-defective matrix — the running family is
exactly that case. Chapters 10–24 remain, with 90 unsolved exercises and 90
non-exact rows.
