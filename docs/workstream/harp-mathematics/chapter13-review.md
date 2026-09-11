# Chapter 13 maximum-modulus record

Date: 2026-09-10. Third chapter of Package 3 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md),
and the first chapter of Part III.

**Review status: independently reviewed before landing**, the second chapter
written under the amended process. Two reviewers, one on mathematical correctness
and one on prose-to-Lean correspondence, neither with access to this directory,
neither permitted to build Lean. Kata issue for Chapter 13 is closed by this
record.

## What the chapter is, and is not

Five of the six cards were already `reexported-proof` rows over genuine
multi-step `CrouzeixConjecture` proofs — none `rfl`, none a single library
application — so the Lean work was six exercise solutions plus one registration
repair. CFT-13-001 aliases a *definition*, `maxFunctionModulusOnSet`, and had been
mis-registered as a `checkpoint`; it is now a `definition` row, the seventh in the
book.

The scope gap is the widest so far. The chapter is titled for metric and normed
spaces and develops none of them: no Cauchy sequences, no completeness, no norm
equivalence, no continuity of finite-dimensional linear maps, no sphere
compactness, no infinite-dimensional counterexample. What it compiles is one
functional — the maximum modulus over a set — and five of its properties. The
earlier sketch set five of those six topics as exercises; all five are withdrawn
and the sixth, a reading of monotonicity, survives as E06.

## Independent review findings

**Six blocking defects.** Every one was a claim about *which hypothesis is
load-bearing and why* — a category the previous two chapters barely exercised,
and one where I was wrong repeatedly and in the same direction: asserting that a
hypothesis was necessary when it was merely used by the chosen proof.

1. **"A supremum bounds its set unconditionally" is false in this setting.** I
   used it to call CFT-13-003's compactness redundant. Mathlib's real supremum
   returns `0` on a set unbounded above (`Real.sSup_of_not_bddAbove`), so the card
   is *false* without boundedness: take `s = ℝ` in `ℂ`, `f` the identity, `z = 1`;
   then `M(s,f) = 0` and `1 ≤ 0` fails. Compactness is load-bearing for the
   statement, not just the proof.

2. **Nonnegativity needs no hypotheses at all.** I wrote that a supremum of
   nonnegative numbers is nonnegative "only when the set is nonempty" — false, and
   self-refuting in the same sentence. Mathlib packages the general fact as
   `Real.sSup_nonneg`, covering empty and unbounded sets through the same junk
   value. `0 ≤ M(s,f)` holds for every `s` and `f`; CFT-13-004's three hypotheses
   are inherited from its route through attainment.

3. **Monotonicity does not need both maxima attained.** I claimed compactness on
   both sets is what the card "actually needs". Three of its six hypotheses are
   removable: compactness of `t` with continuity on it bounds the image, and
   `csSup_le_csSup` finishes. I had also called monotonicity of a supremum
   "immediate from the definition", which is false in a conditionally complete
   order.

4. **CFT-13-006's ambient set.** I claimed that without a common compact ambient
   set the modulus of continuity could degrade. That counterfactual cannot occur:
   the union of the `s k` is automatically compact under the remaining
   hypotheses. What the ambient set genuinely supplies is a single domain on which
   `f` is assumed continuous, since per-`k` continuity would not glue.

5. **The opening claimed finiteness is compiled.** It is not — nothing establishes
   that the image is bounded above — and the same sentence dropped two properties
   that *are* compiled.

6. **An uncompiled iff in an "Exact transfer" slot.** CFT-13-004's ML analogy
   claimed the maximum is zero exactly when the function vanishes identically.
   True, and compiled nowhere.

Also repaired: a provider narration naming the wrong closing tactic; a diagnostic
whose second disjunct is impossible; an ML-bridge claim that compactness does the
work "in every card"; a boundary case asserting non-convergence where a decreasing
family always converges, merely to the wrong limit; a conceptual-model sentence
implying the approximating family is nested; a worked instance ill-formed at
`k = 0`; a claim that the named declaration "is the definition itself" when it is
an alias; three source-boundary lines omitting the packet that owns the providers;
an orphaned exercise cross-reference; and an undisclosed exercise that conjoins
two public cards' exact types.

## Registry repairs

The reviewers found the registry rows disagreeing with both the prose and the
Lean dependency graph. Fixed for this chapter: `CFT-13-004` was `kind:
"definition"` for a theorem; four `pedagogical_prerequisites` lists disagreed with
the providers' actual calls; five exercise `skills` lists were mechanically
`E0k → CFT-13-00k` and contradicted the prose, including one that pointed at the
card the prose explicitly says no exercise restates.

**Known book-wide artifact, not fixed here:** seventeen rows carry `kind:
"definition"` with a non-definition `formal_mode`, almost all with `-001` or
`-004` item numbers, which looks like a mechanical scaffolding bug. Sixteen of
them belong to Chapters 14-24, which are still sketches; they should be corrected
as each chapter is completed rather than in a sweep that would touch unfinished
contracts. The same applies to the prose/registry prerequisite divergence, which
one reviewer measured at 67 of 90 sections book-wide.

## Verification

`mise run crouzeix-textbook-publication` reports "matches canonical inputs".
Counts moved to 69 `proved-here`, 25 `checkpoint`, 7 `definition`, 150 exact rows
and 150 solved exercises, with a 478-declaration receipt. All twelve line links,
all twelve type hashes, and all twelve statements were verified against the Lean
by the correspondence reviewer.

## Process note

Third pass, third confirmation. The defect rate has not fallen — six blocking this
time against three for Chapter 12 — but the character has shifted: Chapter 13's
failures were all in one category, necessity-versus-use claims about hypotheses,
which is exactly the kind of claim a careful-sounding chapter invites and a
self-review will wave through. Worth watching for in Chapters 14-24, where the
same ledger format is used throughout.
