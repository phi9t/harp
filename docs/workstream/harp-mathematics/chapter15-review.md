# Chapter 15 holomorphic functional calculus record

Date: 2026-09-11. Fifth chapter of Package 3 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md).

**Review status: independently reviewed before landing**, the fourth chapter
under the amended process.

## What the chapter is

The deepest chapter of Part III. It builds `f(A)` for an arbitrary matrix: a
parametrized boundary integral, its continuity in the matrix argument, its
agreement with the eigenvalue recipe on simply diagonalizable matrices, and a
density argument carrying that agreement to matrices with no diagonalization at
all. The running family `A_{λ,α}` with `α ≠ 0` is the case in point — the recipe
is not inaccurate there, it is undefined.

Four of the six rows had been registered as `checkpoint` with no underlying
declaration: CFT-15-001 and CFT-15-004 alias definitions and become `definition`
rows; CFT-15-003 and CFT-15-006 alias theorems in the nested namespace
`PositivePeriodicRadialData.OrientedRadialConvexBoundary` and become
`reexported-proof` rows.

The scope gap is again wide. The chapter is titled for complex differentiability
and develops none of it — no difference quotient, no Cauchy–Riemann, no
"holomorphic implies analytic", no Cauchy estimate. Cauchy's theorem itself enters
through a maintained declaration the chapter names rather than proves.

## Independent review findings

**Five blocking defects.** Both reviewers independently found the first two.

1. **The normalization arithmetic was stated wrong, twice.** The chapter said the
   kernel's `(2π)⁻¹` and Cauchy's `period·i` "cancel exactly". The period is `2π`,
   so those factors give `i`. The missing `−i` comes from the orientation identity
   `tangent = i·normal·speed`, which is the only thing making the contour
   "oriented" in the sense the ledger claimed. The compiled lemma is
   `(−i(2π)⁻¹)·(2π i) = 1`.

   Worth recording precisely: **the proof paragraph had it right**, quoting
   `k = −i/(2π)` from the provider, and I had opened and verified
   `cauchyNormalization_mul_period_I` earlier in the same session. Two other
   passages stated the same fact without the `−i`, and nothing compared them.
   Applying the "open the source" check once and not propagating it to every
   passage asserting the same thing.

2. **CFT-15-001's hypothesis ledger claimed typeclasses the declaration lacks** —
   a compact parameter space and a finite measure — contradicted by the normalized
   type printed twelve lines below and by the chapter's own E01 solution.

3. **"Part V consumes CFT-15-006", asserted three times, is false.** No Part 05
   declaration references it. Part V's Chapter 28 takes `SimpleDiagonalization` as
   a hypothesis, which is exactly the assumption this chapter's last card removes.
   The consumers are `CrouzeixConjecture.HolomorphicOuterLimit` and, through it,
   Part VI's Chapter 32.

4. **A false converse about the numerical range.** The chapter said that outside
   the numerical-range set "the resolvent has a pole on the contour and the
   integrand is undefined". Both halves wrong: the spectrum is contained in the
   numerical range strictly, so `B = [[0,3],[0,0]]` with `Ω` the unit disk fails
   the condition while its resolvent exists on the whole circle; and the resolvent
   is Mathlib's total inverse, so the integrand always exists. What fails outside
   is meaning, not definedness.

5. **Convexity of `V` presented as necessary.** It is used — the Cauchy step needs
   a primitive — but the statement does not need it, since the closure is compact
   and convex and a thickening inside `V` would serve. The Chapter 13 lesson
   recurring in a new place.

Also repaired: a five-step `calc` described as four in three places; an
`integral_congr_ae` narration that understated what the provider proves; an
openness ledger that missed both that CFT-15-006 uses it and that it is already
bundled in the boundary structure, making the stated boundary case vacuous; two ML
paragraphs saying "diagonalizable" where the cards say "simply diagonalizable";
"a specific exact construction" for a `Classical.choose`; two ML diagnostics
stating heuristics as entailments; an unlabelled worked instance; a `functionEval`
notation hiding a data dependence on the diagonalization witness; a
registry-correction note saying two rows changed when four did; and a dangling
forward pointer in already-pushed Chapter 14, which promised this chapter would
develop complex differentiability "proper".

## The process failure, and what was done about it

Two of the five blocking defects — the broken forward link and the wrong Part —
were failures to apply checks written into the plan hours earlier, in this same
session. Not knowledge gaps. That is the second consecutive chapter where a known,
written-down check was not applied, after Chapter 14's registry rot.

The response is `scripts/check_chapter_prose.py`, run before dispatching
reviewers. It fails on unresolved cross-references, lists registry rows left
untouched while the prose was rewritten, and prints every sentence naming a formal
mode beside the registry's actual tally.

Two earlier versions were built and discarded, and the reasons are recorded in the
script because they are the same reasons that make checks worthless:

- A "Chapter NN does X" check comparing the phrase to the index title by word
  overlap produced **sixteen false positives on this chapter and caught none of
  its real defects**.
- A mode-claim check that parsed quantities matched **two of four** real
  phrasings, missing the exact sentence that was Chapter 12's blocking defect.

The shipped version surfaces rather than adjudicates. It then reported a defect in
pushed Chapter 12 that was its own false positive — reading the matrix
`[[0,-1],[1,0]]` as a wikilink — fixed by stripping code spans and re-verified.
Each version was checked by reintroducing a real defect and confirming failure; it
is clean across all eleven completed chapters.

## Verification

`mise run crouzeix-textbook-publication` reports "matches canonical inputs".
Counts moved to 69 `proved-here`, 19 `checkpoint`, 11 `definition`, 162 exact rows
and 162 solved exercises, on a 496-declaration receipt. The prerequisite rows were
verified rather than rewritten — they were already sound, unlike Chapter 14's — so
the roster digest did not move. 305 tests pass.
