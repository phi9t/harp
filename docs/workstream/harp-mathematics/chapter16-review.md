# Chapter 16 holomorphic functional calculus record

Date: 2026-09-12. Sixth chapter of Package 3 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md).

**Review status: independently reviewed before landing**, the fifth chapter under
the amended process. Two reviewers, non-overlapping briefs: mathematical content
against the provider sources, and contract/disclosure against the plan's
obligations.

## What the chapter is

The holomorphic functional calculus on matrices. Its one structural fact, which
the withdrawn sketch had backwards, is the order of definition and theorem:
`holomorphicMatrixEval A f` is **defined** as `limUnder atTop (simpleSpectrumHolomorphicEval A f)`
— the limit of the eigenvalue recipe along perturbed matrices that do diagonalize —
and the Cauchy integral is a theorem *about* that limit, not its definition. The
sketch asserted that the formalization "does not define `f(A)` by choosing
eigenvectors", which is the opposite of what `SimpleDiagonalization.functionEval`
does.

The sketch's mathematical content was the classical consequence list — maximum
modulus, identity theorem, open mapping, residues — none of which is compiled and
none of which any of the six cards states. It was replaced.

The chapter's organizing question is which cards need holomorphy and why.
CFT-16-003 and CFT-16-004 need none; CFT-16-005 and CFT-16-006 do. The reason is
the proof route rather than the statements: the two algebraic laws close with
`tendsto_nhds_unique`, which needs the sequences to converge, and convergence is
what Chapter 15's contour theory supplies. CFT-16-004 escapes because it closes
with `Filter.map_congr` — equality of pushforward filters — and CFT-16-003 escapes
because `Polynomial.continuous_aeval` gives convergence on the whole matrix space.

That `limUnder` depends only on the pushforward filter was checked against Mathlib
rather than assumed: `limUnder f g = lim (f.map g)` and
`lim f = Classical.epsilon fun x => f ≤ 𝓝 x` (`Mathlib/Topology/Defs/Filter.lean:247-253`).
So locality genuinely survives totalization — it holds even when neither side
converges, and both sides are then the same unspecified value.

## Independent review findings

**Eight blocking, fifteen minor across the two reviewers.** Both independently
found the false forward-dependency claim, which the author had also caught before
dispatch.

Blocking, mathematical:

1. *The rationals are not dense in `\mathbb{C}`.* The chapter used the indicator of
   `\mathbb{Q}` twice as a "badly behaved" witness. `\mathbb{Q}` lies in `\mathbb{R}`,
   so that indicator vanishes on a neighborhood of every non-real point and is
   holomorphic off the real line — the opposite of the intended property. Replaced
   throughout with the Gaussian rationals `\mathbb{Q} + i\mathbb{Q}`.

2. *Asserting divergence while warning against assertion.* Two passages said the
   totalized value "is not zero, not the identity" and that the sequence "has no
   reason to converge, and `limUnder` has returned junk" — asserting exactly what
   they declared illegitimate, and plausibly false: if no eigenvalue of any chosen
   approximant is a Gaussian rational, every term is the conjugate of the zero
   diagonal matrix, the sequence is constantly zero, and the limit exists. Since
   `simpleSpectrumApproximation` is a `Classical.choose`, the chapter cannot know
   which case obtains. Rewritten to claim only that the chapter determines nothing.

3. *The chapter refused a commutation that its own card proves.* It said "nothing
   here licenses reordering" of `f(A)g(A)`. But multiplication in `\mathbb{C}` is
   commutative, so `fun z => f z * g z` and `fun z => g z * f z` are the same
   function, and applying CFT-16-006 to each orientation gives
   `f(A)g(A) = g(A)f(A)` under exactly its own hypotheses. The fixed factor order
   is an artifact of the formal statement, not a restriction. Corrected to say that
   no *declaration* is the commutation identity, which is the true weaker claim.

4. *The ML bridge contradicted CFT-16-004's own ledger*, saying locality carries
   "nothing at all" while the ledger says openness of `U` is used essentially.

Blocking, specification:

5. *CFT-16-001 was falsely disclosed as having no nameable provider.* It shipped as
   a `checkpoint` with `underlying_declaration: null`, and the prose said the
   definition "has no proof to name". But `CrouzeixConjecture` is a maintained
   prefix and Chapter 15's CFT-15-001 and CFT-15-004 are the same alias shape
   carrying `formal_mode: definition` with a non-null underlying. The consequence
   was concrete: `CrouzeixConjecture.holomorphicMatrixEval`, the declaration the
   entire chapter is about, was **absent from the compiled receipt**. Promoted to
   `definition` / `exact` / `reconstructible` with the provider named; the receipt
   went from 502 to 503 declarations.

6. *False forward dependency, asserted twice.* "Chapter 17 takes the four laws as
   an interface" is false: Chapter 17 never names this chapter, does not import its
   module, defines `f(A)` by the contour integral instead, and re-derives
   additivity and multiplicativity from nested contours. Only CFT-17-006 cites any
   Chapter 16 card, and it cites CFT-16-001. This is the same defect class as
   Chapter 15's "Part V consumes CFT-15-006". Replaced with the actual edges, and
   with the architectural disagreement stated as outstanding work rather than
   concealed.

7. *Rows never promoted, frontmatter claiming otherwise.* All six coverage rows were
   still `unmapped`/`checkpoint` while the prose frontmatter asserted
   `lean_exact_correspondences: 6`.

8. *Derived surfaces and pinned literals stale.* Handled as the normal projection
   step; see Verification.

Minor findings, all fixed: the CFT-16-002 disclosure named the double-layer module,
which is not in that chain (the chain is an exact Cauchy identity plus continuity,
with no estimate in it); the card quotes CFT-15-006 but listed only CFT-15-003 as a
prerequisite; CFT-16-E03 silently restated an existing maintained theorem
(`parametricBoundaryIntegral_eq_of_two_orientedRadialBoundaries`); the opening
problem dropped two hypotheses of CFT-15-003; "developed elsewhere" named no
chapter; `[Nonempty n]` was described as no hypothesis when the zero-by-zero case is
genuinely outside the definition; CFT-16-004's boundary case asserted a necessity the
chapter elsewhere refuses to assert; the claim that matrix-argument continuity is
special to polynomials (CFT-15-002 supplies it for holomorphic `f` too — what is
special is globality); a self-contradiction about which declarations the cards name;
`simpleSpectrumApproximation` called "not a constructed perturbation" when the
witness is explicitly `A + \eta(D - A)` and it is `Classical.choose` that hides it;
uneven obligation-8 labelling across exercises; and a ledger/exercise pair giving
opposite readings of the same hypothesis.

## What the reviewers confirmed

Receipt fidelity was exact on all twelve prose blocks — six cards and six exercises —
for normalized type, type SHA-256, direct dependencies, axioms, and source locators.
All six cards carry the ten required subsections. No row is an alias of an alias.
CFT-16-004's `kind` correction from `definition` to `theorem` is right.

Two deliberately planted premises in the review briefs were rejected, correctly: the
brief claimed the E02/E03 `skills` entries had been swapped and invited a check, and
the reviewer verified the current mapping is the correct one and warned against
reverting it; the brief also invited scrutiny of the CFT-16-001 prerequisite change,
which was confirmed against the definition's actual dependencies.

## Contract changes

CFT-16-004 `kind`: `definition` → `theorem`, matching its theorem-kind provider.
CFT-16-001 `formal_mode`: `checkpoint` → `definition`, with
`CrouzeixConjecture.holomorphicMatrixEval` named as the underlying declaration.
All six rows promoted to `exact` / `reconstructible`.
CFT-16-001 prerequisites `[CFT-06-004, CFT-15-001]` → `[CFT-06-006, CFT-15-004]`:
the definition is the limit of the eigenvalue recipe (CFT-15-004) along
approximations that exist by simple-spectrum density (CFT-06-006), and it does not
touch the contour integral — that it is *not* the integral is the chapter's point.
CFT-16-002 prerequisites gained CFT-15-006, the convergence result its proof quotes.
CFT-16-E02 and CFT-16-E03 `skills` swapped to the cards they actually use.

The roster digest moved from `caae3954…` to `99d8dc09…`. Verified rather than
repinned blindly: the pre-change contract reproduces the old pin exactly, and
exactly the two intended rows differ.

## Verification

`harp crouzeix-textbook check` reports "matches canonical inputs" against a fresh
503-declaration receipt. Counts moved to 69 `proved-here`, 117 `reexported-proof`,
18 `checkpoint`, 12 `definition`, 168 exact rows and 168 solved exercises. 296 tests
pass in the textbook suite and 9 in the foundations contract.
