# Chapter 7 inner-product-core record

Date: 2026-09-09. First chapter of Package 2 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md),
implemented on local `master` on top of `5b870460`.

**Review status: author self-review only**, as for
[Chapters 5 and 6](chapter05-06-review.md). Kata `3ya3` stays open for the
independent specification and content-quality reviews its acceptance criteria
require.

## What landed

Six cards moved from `checkpoint` or `unmapped` to `exact` correspondence with
`reconstructible` exposition, and six exercises gained distinct checked
solutions. The prose went from a 130-line sketch whose six card headings had no
bodies at all to a full development.

Three cards were `checkpoint` aliases into the maintained foundations library.
`MathematicalFoundations` is not one of the receipt's maintained namespaces, so
a re-export could not name its provider and the validator could not check the
alias target. CFT-07-001, CFT-07-002 and CFT-07-006 are therefore now proved
locally, which is also what lets Part II own its own development. CFT-07-003,
CFT-07-004 and CFT-07-005 remain re-exports of `CrouzeixConjecture` theorems,
which the validator does check.

New support declarations: the four pairing axioms, Pythagoras for the
projection splitting, best approximation on a line, the degenerate zero
direction, and the complex boundary that transpose and conjugate transpose
differ. Cauchy–Schwarz and its equality case are restated in the chapter's
notation and discharged by the maintained laboratory's quadratic and
proportionality theorems; that reuse is labeled in the prose rather than
presented as a new proof.

## Every proof narrates a checked derivation

Each card's displayed proof was written against the proof term its provider
actually runs, following the discipline established for Chapters 5 and 6.

- CFT-07-001's checked proof is the single `abel` step, and the prose says the
  identity is abelian-group cancellation that needs no hypothesis on `u`.
- CFT-07-002's checked proof derives the nonzero denominator from
  `dotProduct_self_eq_zero`, rewrites with `sub_dotProduct` and
  `smul_dotProduct`, then clears the denominator; the prose runs those steps in
  that order and says where the hypothesis is consumed.
- CFT-07-003's provider is `EuclideanSpace.inner_eq_star_dotProduct` followed
  by `dotProduct_comm`, and the prose describes exactly that convention
  bookkeeping rather than a computation.
- CFT-07-004's provider is a single `map_star` application; the prose says the
  work sits in the star-algebra structure of `euclideanOperator`, not in the
  card.
- CFT-07-005's provider is `rfl`. The prose says there is nothing to compute
  because the matrix norm is defined as the operator norm, and adds why the
  card is still worth stating: later chapters bound `‖p(A)‖` and the reader is
  entitled to know it is neither the Frobenius norm nor an entrywise maximum.
- CFT-07-006's provider is `norm_nonneg`; the prose gives the two-line
  derivation from the triangle inequality that the library packages.

Mechanically checked: all twelve card and exercise regions cite at least one
compiled declaration, and every Lean line link in the chapter resolves to the
declaration it names.

## Defect found while landing

Two of the three locally proved cards first elaborated to exact eta aliases of
non-maintained library lemmas, so the receipt classified them as `direct-alias`
and the validator rejected them as `proved-here`. That is the protection
working as intended: a card claiming a local proof must not be a renamed import.
Both were rewritten to genuine derivations — the splitting through a named local
step, and nonnegativity through the triangle-inequality argument the prose
describes.

## What this does not establish

No independent mathematical or correspondence review was performed. The
Cauchy–Schwarz results are reused, not reproved here, and the laboratory's own
controls are what establish that its routes are independent of each other.
Chapters 8–24 remain, with 102 unsolved exercises and 102 non-exact rows; see
[textbook-backlog-inventory.md](textbook-backlog-inventory.md).
