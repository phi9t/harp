# Chapters 5-10 independent review record

Date: 2026-09-09. Closes the deferred independent review carried by Kata
`a2ry`, `9hxf`, `3ya3`, `tdga`, `wstr` and `0wc6` for Package 2 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md).

**Review status: independent.** Four reviewers ran with no access to
`docs/workstream/harp-mathematics/`, so none of them could anchor on the author
self-reviews recorded in [Chapters 5 and 6](chapter05-06-review.md),
[Chapter 7](chapter07-review.md), [Chapter 8](chapter08-review.md),
[Chapter 9](chapter09-review.md) and [Chapter 10](chapter10-review.md). Two
dimensions were reviewed over two chapter groups: mathematical correctness
(Chapters 5-7, 8-10) and prose-to-Lean correspondence (Chapters 5-7, 8-10).
Lean builds were barred during the pass because the shared Lake cache corrupts
under concurrent builds; every declaration was read as source.

This satisfies the review-authority section of the plan, which requires
obligation 9 to be discharged by a non-implementer.

## Why it was worth running

The self-reviews recorded in this directory passed all six chapters. The
independent pass found **five blocking defects** in prose that had already been
committed and pushed, and roughly thirty smaller ones. The self-review's blind
spot was systematic rather than incidental: it re-checked the claims the author
had just written against the author's own reading of the providers, so a claim
that was wrong in the same way twice survived both times.

## Blocking findings and their fixes

1. **Chapter 9, ML bridge — a false mathematical claim.** The non-transfer
   paragraph asserted that a numerically computed largest singular value is
   unreliable for a near-defective matrix. This is backwards. Singular values
   are perfectly conditioned: Weyl's inequality gives
   `|sigma_k(A+E) - sigma_k(A)| <= ||E||`. Defectiveness destroys *eigenvalue*
   conditioning, not singular-value conditioning, and the chapter's own example
   refutes the claim it was offered in support of -- for `A_{1,10}` the Gram
   matrix is `[[1,10],[10,101]]`, giving `sigma_max ~ 10.0990` and
   `sigma_min ~ 0.09902`, both stable. Confirmed numerically over 20000 random
   perturbations before the fix. Replaced with the correct statement, including
   the closed form `sigma = (sqrt(alpha^2+4) +/- alpha)/2`.

2. **Chapter 9 — singular values are the chapter's title subject and nothing
   about them is compiled.** No declaration in `Part02/Chapter09.lean` mentions
   a singular value, an eigenvalue or a spectrum, yet five passages asserted
   singular-value facts as established, and the synthesis listed
   "`||A||` is the largest singular value" as one of the four compiled interface
   facts. Identifying `||A||` with `sigma_max` needs the norm of a positive
   semidefinite matrix to equal its spectral radius, which this book has not
   built. Added an explicit uncompiled-vocabulary disclosure, demoted the
   identification out of the interface, and marked the remaining readings as
   glosses.

3. **Chapter 9 — the central thesis rested on an uncompiled step.** "Nilpotent,
   so every eigenvalue is zero and the spectral radius is 0" was cited to
   `shear_nilpotent`, which proves only `N^2 = 0`. Rather than label this, it is
   now compiled: `shear_charpoly` proves the characteristic polynomial is
   `X ^ 2` and `shear_spectrum_eq_zero` proves zero is the only eigenvalue,
   reading the characteristic polynomial through Chapter 6's root test. The
   chapter's counterexample is now checked end to end.

4. **Chapter 10 — uniqueness claimed, computation rule compiled.** The universal
   property was stated with "factors through a **unique** linear map" and cited
   to `bilinear_factors_through_tensor`, which states only
   `lift f (x ⊗ₜ y) = f x y` for an `f` already handed over in curried linear
   form. Neither existence nor uniqueness is checked anywhere in the chapter.
   Disclosed.

5. **Chapter 7, Cauchy-Schwarz — the displayed step discards the wrong term.**
   "Discarding the nonnegative **projection** term in Pythagoras" yields only
   `r.r <= x.x`, which gives nothing. The inequality follows by discarding the
   **residual**. The conclusion was right and the very next clause showed what
   was meant, but the displayed derivation did not follow. Corrected, with the
   Pythagoras identity written out.

## Correspondence defects fixed

Both correspondence reviewers found the same recurring failure: a displayed
argument that reads as the proof, cited to a provider that runs a different
one, with no disclosure. Obligation 8 requires the disclosure in exactly these
cases. Fixed in CFT-06-004 and CFT-06-005 (both are single applications of
`Polynomial.aeval_mem_adjoin_singleton` and `Algebra.adjoin_mem_exists_aeval`,
not the subalgebra arguments displayed), CFT-10-002 (two rewrites, not the
component computation), and CFT-08-E04 (the iterated congruence is never
stated).

`CFT-07-006` was mis-attributed to `norm_nonneg`; the declaration is a local
proof. Worse, its second `calc` step was `by simp`, and `norm_nonneg` is itself
a simp lemma, so the compiled artifact was probably a restatement of the card
rather than a derivation of it. Rewritten to run the displayed argument
genuinely -- `norm_sub_le` at `y = x`, then `sub_self`, `norm_zero`, `linarith`
-- so prose and Lean now agree.

Unlabeled uncompiled claims were labeled in Chapter 6 (the `p(J)` derivative
formula, the running family's characteristic and minimal polynomials, complex
diagonalizability of `planeRotation`, the `dim <A> <= n` corollary) and
Chapter 10 (the pure-tensor generalization, which is a statement about `2x2`
real arrays and does not mention `TensorProduct`; "most elements" is a
generalization from one witness).

## Mathematical overstatements fixed

- Chapter 5: "strictly weaker in hypothesis" was false. For square matrices over
  a commutative ring, `RS = I` and "`S` is a unit" are *equivalent*, and the
  book compiles the implication itself as `left_inverse_is_two_sided`. Also
  "the vanishing of that scalar detects singularity" is a field statement; over
  a ring the criterion is that `det` be a unit, as the chapter's own next card
  says.
- Chapter 6: nonemptiness of the index set is a formalization hypothesis, not a
  mathematical necessity -- over an empty index type `B = A` already works. And
  the claim that the density argument needs eigenvector annihilation is false;
  it runs entirely through the resultant of the characteristic polynomial with
  its derivative.
- Chapter 8: the condition number `~40` of the running Gram matrix was
  attributed to the vectors being "far from parallel". The angle is about
  `53 degrees`; the driver is the `5:1` length disparity. Equalizing the lengths
  and holding the angle fixed drops the condition number to `4`.
- Three "unless / only when" claims (Chapters 8, 9, 10) asserted that two
  matrices agree only in a special case, when they agree *for every argument*
  only in that case but can coincide pointwise otherwise.

## Defects found outside the reviewers' reports

Chapter 7 forward-referenced subspace orthogonalization to "the orthogonalization
that Chapter 9 supplies". No chapter in the book develops orthogonalization;
Chapter 9 is operator norms. Rewritten to say the book does not develop it.

Chapter 7's opening claim that "Part I never measured anything" is contradicted
by CFT-06-006, which states a density result in the operator norm. Both sides
now name the forward reference.

## Contract consequences

Two support declarations were added to `Part02/Chapter09.lean` and one proof in
`Part02/Chapter07.lean` was rewritten. Neither changes a card or exercise type,
so the receipt still carries 459 declarations and 541909 bytes. Twelve exercise
line pins moved (Chapter 7 by `-1`, Chapter 9 by `+15`) and the receipt digest
changed even though its length did not -- every shifted line number kept its
digit count, which is a reminder that the byte count is not an identity.

`mise run crouzeix-textbook-publication` reports "matches canonical inputs";
`crouzeix_textbook` (296 tests) and `foundations_contract` (9 tests) pass.

## What this says about the process

The plan's acceptance obligation 8 was written after three provider-fidelity
defects were found by hand. It caught the right class of defect -- every
correspondence finding above is an obligation 8 violation -- but the author
applying it to their own work did not find them. The obligation needs the
independent reviewer the plan's review-authority section already requires; it
is not self-applicable. Recording that as the operative lesson for
Chapters 11-24.
