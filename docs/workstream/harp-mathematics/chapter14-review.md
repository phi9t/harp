# Chapter 14 matrix power series record

Date: 2026-09-11. Fourth chapter of Package 3 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md).

**Review status: independently reviewed before landing**, the third chapter under
the amended process. Two reviewers, neither with access to this directory,
neither permitted to build Lean.

## What the chapter is

Four of the six cards were already `reexported-proof` rows over
`CrouzeixConjecture.MatrixPowerSeries`; CFT-14-001 and CFT-14-004 alias
*definitions* and had been mis-registered as `checkpoint`, the same artifact
corrected in Chapter 13. They become the book's eighth and ninth `definition`
rows. The Lean work was six exercise solutions.

The scope gap is again wide. The chapter is titled for sequences and series of
operators and compiles one case: matrix-valued power series in a complex
variable. Not the Neumann series, not `(I-T)^{-1}`, not Banach-space absolute
convergence, not the spectral radius formula, not continuity of inversion,
nothing at `|z| = 1`. The sketch displayed four theorems and set six exercises on
those topics; four exercises are withdrawn, and two survive — the finite geometric
identity as E02, which is pure ring algebra and genuinely compilable, and the
radius reading as E06.

## Independent review findings

**One blocking defect, from the mathematical reviewer, and two from the
correspondence reviewer.** The rate fell sharply from Chapter 13's six, and the
reason is worth recording.

Chapter 13's six were all necessity-versus-use claims about hypotheses. This
chapter is dense with exactly those claims — a coefficient bound that is
sufficient but not necessary, a `C < 0` vacuity argument, an open-disk hypothesis
that genuinely cannot be dropped — and the reviewer **confirmed every one of them
correct**, checking the `a_m = m\cdot I` and `a_m = m!\cdot I` witnesses against
Mathlib's definition of `radius` directly. The Chapter 13 lesson transferred.

The blocking defect was a different kind of error: a Mathlib fact stated
backwards. The chapter claimed `AnalyticOnNhd` is *stronger* than pointwise
analyticity and that the two coincide on open sets. `AnalyticOnNhd` is *defined*
as `∀ x ∈ s, AnalyticAt 𝕜 f x` — it is the pointwise notion. The weaker one is
`AnalyticOn`, built from `AnalyticWithinAt`, and Mathlib's own docstring says so;
the two agree exactly on open sets. Corrected against the source.

The two correspondence blockers were both registry rot: `exercises.json` `skills`
and `coverage.json` `pedagogical_prerequisites` were byte-identical to the
pre-rewrite sketch, while the prose had been written fresh. Four of six skills
rows named the wrong card. **This is the same defect Chapter 13's review found,
and I did not check for it here despite knowing about it** — a process failure,
not a knowledge gap.

## A distinction the reviewers conflated, and the resolution

The correspondence reviewer proposed aligning `pedagogical_prerequisites` with the
Lean dependency graph. That is the wrong fix, and taking it in Chapter 13 was a
partial mistake.

The two are different objects by design. In the reviewed wave-4 chapters the prose
*Pedagogical prerequisites* section is narrative — "the proof uses continuous
functions on compact sets, infima, convex segments" — and never enumerates card
identifiers, while the registry field holds a curated card graph whose
cross-chapter edges are pinned by `reviewed_chapter_policy` in
`crouzeix_textbook.rs`. Chapter 14's policy is `{6, 9, 11}`; rewriting the graph
to match the Lean would have silently changed it.

The resolution taken here: correct the registry to a defensible pedagogical graph
*within* the reviewed policy — the cross-chapter edges to Chapters 6, 9 and 11 are
preserved and verified — and add a paragraph to the Lean-translation section
saying explicitly that the two graphs are recorded separately and differ on
purpose, with CFT-14-001 as the example, since it is pedagogically preceded by
Chapter 6 and Chapter 9 while its Lean type depends on neither.

## Defects found outside the reviewers' reports

Before dispatching them I checked my own riskiest claims and found one: the
running example asserts that submultiplicativity gives `‖T^{m}‖ \le 1` for
`‖T‖ \le 1`, which does not reach `m = 0`, where `T^{0} = I` and the bound comes
from the identity's induced norm. The bound holds; the justification did not cover
the degree the chapter elsewhere singles out. The mathematical reviewer found the
same gap independently, which is a useful calibration: self-checking catches some
things, and the three chapters of evidence say it does not catch enough of them.

## A latent projector bug this chapter surfaced

The published claim ledger has been asserting "216 theorem rows: 69
`proved-here`, 115 `reexported-proof`, 23 `checkpoint`, and **six** `definition`"
— parts summing to 213. The projector had the word "six" hardcoded in its
replacement string. It was true while there were exactly six definition rows and
went false when Chapter 13 added the seventh, silently, because nothing compared
the parts against the total.

Fixed to derive the count, with a regression test that renders the ledger and
reads the number back. The test was verified by temporarily reverting the
projector: it fails against the hardcoded word and passes against the fix. A first
attempt tested `tally` instead and would not have caught the defect — adjacency to
a bug is not coverage of it.

## Verification

`mise run crouzeix-textbook-publication` reports "matches canonical inputs".
Counts moved to 69 `proved-here`, 23 `checkpoint`, 9 `definition`, 156 exact rows
and 156 solved exercises, on a 486-declaration receipt. The roster digest moved
with the four corrected prerequisite rows and was repinned only after confirming
the previous contract still reproduces the previous pin. The correspondence
reviewer independently recomputed all twelve type hashes from the quoted
normalized types and found them exact.
