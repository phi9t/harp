# Chapter 11 differentiation record

Date: 2026-09-10. First chapter of Package 3 of
[the remaining-work plan](../../superpowers/plans/2026-09-08-textbook-remaining-work.md),
landed on top of the Chapters 5-10 independent review.

**Review status: independently reviewed 2026-09-10; blocking findings repaired,
three Lean-side items deferred.** See *Independent review* at the end of this
record. The section immediately below is the original author self-review, kept
because the comparison is the point: it passed a chapter that two independent
reviewers then found five blocking defects in.

**Original status line: author self-review only; obligation 8 unverified.** Under the
plan's amended *Review authority* section this chapter is not reported as
meeting acceptance obligation 8 until a non-implementer has checked it. Kata
`raf4` (Chapter 11) stays open for that pass. The
[Chapters 5-10 independent review](chapters05-10-independent-review.md) is the
reason for the new wording: a self-review is not evidence for obligation 8.

## The finding that shaped the chapter

Chapter 11's five aliased cards pointed at `AutodiffGeometry` declarations, and
**four of the five were proved by `rfl`**. That module *defines* `jvp` to be
`matVec`, `vjp` to be `matVec` of the transpose, and `hvp` to be `matVec`, so
`jvp_eq_matVec`, `vjp_eq_transpose_matVec`, `hvp_eq_matVec` and
`grad_arg_eq_component` were definitional unfoldings. Its own docstring says it
"does not model JAX tracing, floating-point execution, pytrees, or analytic
differentiability".

So the chapter's compiled layer asserted a naming convention while its prose
promised Fréchet derivatives, the chain rule and implicit differentiation. The
namespace wall - `AutodiffGeometry` is not maintained, so none of the five could
carry a checked provider - forced local proofs anyway; the `rfl` finding is what
determined *what* to prove.

## What was built

The Fréchet layer is new and carries the analytic content:
`frechet_derivative_unique`, `continuous_linear_map_hasFDerivAt` and
`frechet_chain_rule`, stated over arbitrary real normed spaces and delegating to
Mathlib's `HasFDerivAt.unique`, `ContinuousLinearMap.hasFDerivAt` and
`HasFDerivAt.comp`. `jacobianAction` bridges to coordinates through
`LinearMap.toContinuousLinearMap`, which is where finite-dimensionality enters:
boundedness is a theorem there, not a hypothesis.

Genuinely new coordinate content, as opposed to renamed Chapter 10 results:

- **CFT-11-005's uniqueness clause.** The transpose is the *only* matrix
  satisfying the pairing identity. Proved by instantiating at `Pi.single` pairs
  and reading off entries. This is the difference between "reverse mode computes
  something adjoint-like" and "reverse mode computes the adjoint", and Chapter 10
  does not have it.
- **`symmetric_cross_terms` behind CFT-11-004.** Symmetry collapses the two cross
  terms of a quadratic form's expansion, giving gradient `2Hx` and Hessian-vector
  product `2Hv`. Symmetry is used in exactly one clause and the boundary
  paragraph gives the counterexample when it is dropped.
- **`chain_rule_in_coordinates_is_matrix_product`.** The composite of two matrix
  actions has derivative the action of the product. This is the theorem the
  chapter exists to prove, and it is what ties the Fréchet chain rule to
  CFT-11-006's algebraic kernel.
- **The `coneCusp` boundary.** `x^3/(x^2+y^2)` extended by zero is homogeneous of
  degree one, so every directional derivative at the origin exists and equals
  `coneCusp v`; but `coneCusp(1,1) = 1/2` while `coneCusp(1,0) + coneCusp(0,1) = 1`,
  so the directional derivative is not additive and no Fréchet derivative exists.
  All three steps compile, including the non-existence: the proof composes
  through `ContinuousLinearMap.smulRight` for the line, uses `HasFDerivAt.unique`,
  and finishes with `map_add`.

## Disclosures made under obligation 8

- CFT-11-002 and CFT-11-003 each carry one theorem and one `rfl` clause. The
  cards say which is which rather than presenting the packaging convention as a
  result. This is the same defect the old `AutodiffGeometry` re-exports had; the
  difference is that it is now stated.
- CFT-11-003's first clause **is** CFT-10-002 and CFT-11-005's first clause **is**
  CFT-10-001. Both cards cite Chapter 10 and say explicitly that no second proof
  is offered.
- CFT-11-006's checked proof is the single library application
  `Matrix.mulVec_mulVec`. The displayed sum-interchange argument is named as the
  content of that library result, not as a compiled proof.
- The chapter withdraws the earlier sketch's promise of implicit differentiation
  rather than leaving it standing, and says so in the Lean-translation section.
  Clairaut symmetry is likewise named as not compiled.

## Structural walls hit

- **Unmaintained namespace**, as in Chapters 7, 8, 9: all five aliases had to
  become local proofs.
- **`Deriv.Comp` is not reachable from the fixture's import set.**
  `HasFDerivAt.comp_hasDerivAt` lives in `Mathlib.Analysis.Calculus.Deriv.Comp`,
  which `Deriv.Mul` does not import. Rather than widen
  `write_fake_all_mathlib_artifacts`, the non-existence proof routes through
  `HasFDerivAt.comp` plus `ContinuousLinearMap.smulRight` and stays inside the
  already-allowed imports.
- **A module-instance diamond on `ℝ`.** Converting the composite to `HasDerivAt`
  produced `RCLike.toInnerProductSpaceReal.toModule` where the goal wanted
  `Semiring.toModule`. Working entirely in `HasFDerivAt` and comparing the two
  bounded maps avoids it.

## The prose-size wall, and why a limit moved

Registering Chapter 11 as complete pushed the textbook Markdown tree past
`MAX_PACKET_MARKDOWN_BYTES`, then 1 MiB. This is not a Chapter 11 problem. The
structured card format that Chapters 25-36 already use runs 38-75 KiB per
chapter; Chapter 11 is 49 KiB, squarely in that range. Thirteen chapters (12-24)
are still 5-7 KiB sketches. The finished 36-chapter tree is therefore about
1.8 MiB, and **no amount of trimming makes the book fit in 1 MiB**.

The constant is a defensive bound on how much prose the validator will read, not
a correctness invariant, so it was raised to 4 MiB with the arithmetic recorded
at the definition site, and the one test that pins the diagnostic text was
updated to match. This is flagged here because changing a validator limit to let
content land is exactly the kind of change that should not pass silently.

## Verification

`mise run crouzeix-textbook-publication` reports "matches canonical inputs".
All twelve new declarations compile as `theorem` in the receipt, not
`direct-alias`, so the eta-alias protection is satisfied by genuine derivations.
The receipt carries 465 declarations, up six for the new exercise solutions.
Contract counts moved to 68 `proved-here`, 27 `checkpoint`, 138 exact rows and
138 solved exercises; the count projector reprojected all five derived surfaces.


## Independent review

Two reviewers, one on mathematical correctness and one on prose-to-Lean
correspondence, neither with access to this directory, neither permitted to build
Lean. They found **five blocking defects** and about twenty smaller ones in prose
landed the same day, after the self-review above had passed it. That is the same
defect rate the Chapters 5-10 pass found, and it is the second consecutive
confirmation that obligation 8 is not self-applicable.

### Blocking, and their repairs

1. **The forward-mode cost model was stated backwards.** The ML bridge said
   forward mode is attractive "only when inputs outnumber outputs". For an
   `m x n` Jacobian, forward mode costs one pass per *input* and reverse mode one
   pass per *output*, so forward mode wins when outputs outnumber inputs. The
   chapter contradicted itself two sections later, where the scalar-loss argument
   is stated correctly. Rewritten to give the pass-count model explicitly.

2. **CFT-11-004 described a quadratic form no declaration mentions.** Both
   reviewers reached this independently. The Statement, Purpose and Proof all
   spoke about `q(x) = (Hx) . x` — its expansion, its remainder, its gradient,
   its Hessian — while the compiled theorem is three coordinate identities that
   never mention `q`. `quadraticForm` is defined in the Lean source and used by
   no theorem in the repository. The card now states what is compiled, labels the
   expansion, the remainder estimate and the gradient conclusion as uncompiled,
   and says the definition carries no content.

3. **CFT-11-004's third clause is an undisclosed `rfl`.** Its checked proof is
   `jacobianAction_apply`, which is `rfl`, plus `Matrix.smul_mulVec` — the exact
   pattern CFT-11-002 and CFT-11-003 disclose. CFT-11-004 presented it as a
   result and its provider line described the wrong clause. Now disclosed.

4. **The non-symmetry counterexample had the cross terms swapped and compared
   against the wrong target.** At `H = [[0,1],[0,0]]`, `x = e_0`, `w = e_1` the
   terms are `0` and `1`, and the failing comparison is `1 != 2*0 = 0` — not
   "the cross terms are 1 and 0, whose sum is not 2*1". Restated at `x = e_1`,
   `w = e_0`, where the original sentence becomes true verbatim.

5. **A worked example credited CFT-11-004 with a remainder estimate.** "The
   quadratic remainder is `o(||w||)`. This is CFT-11-004 at `H = I`." That card
   has no remainder content. Corrected, and the chapter's not-compiled list no
   longer implies the quadratic form's derivative is compiled.

Also repaired: "doubles the shear" for a scaling by `2*lambda`; a diagnostic
inferring non-differentiability from a disagreement its own non-transfer calls
uninformative; a diagnostic describing a probability-zero event; the adjoint
formula under a changed pairing, which needs two pairings for a rectangular
matrix; a false claim about which of `AB`, `BA` typechecks; "six items were
re-exports" where five were; the chapter-level re-use count, three not two; three
attributions of the coordinate chain rule to cards the checked proof does not
call; an unverifiable claim about how Mathlib proves `HasFDerivAt.unique`; the
E05 narration, which called homogeneity a conclusion it is not; and an affine
counterexample to "nonlinear implies varying derivative".

### Deferred to Lean, blocked

Three items need a Lean edit and therefore a rebuild, which is impossible while
the toolchain cache is missing:

- remove the unused `quadraticForm` definition;
- fix the `hessian_vector_action` docstring, which says "one linear solve" where
  it means one matrix-vector product, and repeats the gradient overclaim;
- move the docstring on `jacobianAction_comp`, which describes
  `chain_rule_in_coordinates_is_matrix_product` instead.

Any of these shifts line numbers, so the contracts and receipt must be refreshed
in the same change.

### Environment note

The reviewers worked without Mathlib sources: `formalization/lean/.lake/packages`
is a dangling symlink because `~/.cache/harp/lean` was cleared mid-session, along
with the Rust target cache and the atlas `node_modules`. One correspondence
finding — whether Mathlib's `HasFDerivAt.unique` runs the scaling argument the
prose displays — could not be settled from source and was resolved
conservatively, by dropping the claim about the library's internal proof.
