# Chapter 4 duality-core implementation record

Date: 2026-09-07. Implementer status: source frozen, pending independent
specification and quality review and coordinated publication verification.
This is a bounded local record, not whole-chapter acceptance or a compiler
publication receipt. CFT-04-004 through CFT-04-006 remain summary previews.

Final disposition (2026-09-07): the independent specification and quality
reviews recorded below both passed, and the duality core was accepted locally
as part of the approved foundations core; see
[foundations-review.md](foundations-review.md) for the coordinated
publication, presentation and full-gate evidence. The implementer status
above records the state at freeze time and is kept unchanged. Kata `q8h9`
remains open until the CFT-04-004 through CFT-04-006 exposition obligations
are completed in Chapters 5 and 6; the six statement correspondences are
already exact.

## Ownership and source identity

Worktree: `codex/harp-mathematics-spec`, based on
`4a72888fd9b09d86e958da35074e86c08615a257`, with previously reviewed Chapters
1–3 and shared integration changes already present. This subtask changed
only Chapter 4 Lean, its canonical Markdown, and this review record. No
shared JSON, exporter, generated correspondence, other chapter, issue, commit,
or push was changed by this subtask.

Frozen SHA-256 values:

- Lean: `a7cf1d6687fe385d15eabb859d31f69c22bbc0ac262821fba9bfbefd24c838dc`.
- Prose: `3c54667c9697f5d67d4490a649aa7beef951e6db02b9547e7e306aa7232ac532`.

Final prose after the accepted terminology clarification:
`efab6893b5bcd0f438c03c9d8d5ab298ec7b70614dab503451d3684d92e1db32`.
The Lean source is unchanged.

All six public CFT names and types are preserved. In particular,
`coordinate_change_conjugacy := @change_basis_action` and
`duality_similarity_preserves_charpoly := @similarity_preserves_charpoly`
remain definition aliases. The proposition-definition linter is disabled
locally at those two retained declarations, rather than changing their
compatibility surface. Provider modes must be determined by the integration
owner's compiler pipeline.

## Mathematical implementation

The opening measurement on `redundantPredictor` pulls coefficients `(a,b)`
back to `(a,b,a+b)` and annihilates the imported `nullDirection=(-1,-1,1)`.
The compiled `redundant_predictor_pullback` proves both statements using
Chapter 2's actual definitions.

CFT-04-001 constructs linear pullback and evaluates it; CFT-04-002 proves
composition reversal with two extensionality steps; CFT-04-003 fixes
`x_new=S x_old` and reconstructs the action calculation. Its exact statement
uses separate matrices `S,R` and the left-inverse equation `R*S=1`, matching
the existing public type. Invertible changes specialize to `R=S⁻¹`.

Dual-basis existence, evaluation, independence, and spanning are explained.
`dual_basis_exists` uses `Module.Basis.dualBasis` and
`Module.Basis.dualBasis_apply_self`. `dual_basis_expansion` uses
`Module.Basis.sum_dual_apply_smul_coord`. `pullback_basis_coefficients`
derives transpose coefficients by expanding the image of each primal basis
vector before applying the output functional. It is valid over any field,
including complex scalars without conjugation.

The annihilator proof extends a basis of U, tests vanishing on its first
block, then proves that the remaining dual coordinates span the annihilator
and are independent by evaluation. Named support is separate from the
dimension provider:

- `dualityExtendedBasis`: the `Module.Basis.sumExtend` construction for a
  linearly independent family over a field.
- `annihilator_of_basis_span`: vanishing on the first primal block is
  equivalent to annihilating its span.
- `annihilator_expansion`: a functional vanishing on the first block is the
  sum of its remaining dual-coordinate terms.
- `annihilator_coefficients_unique`: evaluating a zero sum of remaining
  coordinate covectors forces every coefficient to vanish.
- `annihilator_dimension`: the numerical identity uses
  `Subspace.finrank_add_finrank_dualAnnihilator_eq`, not an attribution of
  the whole basis argument to a rank-nullity alias.

The coordinate construction requires no metric. The cumulative family
`A_{λ,α}` has primal action `(λx+αy,λy)` and algebraic-dual coefficient
action `(λa,αa+λb)`. The prose separately chooses the standard Hermitian
metric and derives the conjugate-transpose adjoint by expanding its defining
pairing. The specialization λ=i, α=1 exhibits the conjugation difference.
The general compiled transpose theorem supports the family calculation;
no generic adjoint or autodifferentiation correctness theorem is claimed.

For the real metric with Gram matrix diag(2,1), `weightedMetric_symmetric`,
`weightedMetric_linear_right`, and `weightedMetric_positive` prove its local
properties. E05 proves positivity, the exact linear-loss increment at every
base point, universal gradient pairing, and the inequality between gradient
coordinates `(1/2,2)` and differential coefficients `(1,2)`. The prose calls
the differential a covector and the gradient its metric-dependent vector
representative. It makes no unchecked complex-array API claim.

CFT-04-004 defines the meaning of the characteristic polynomial as det(tI-A).
CFT-04-005 and 006 explain determinant and trace. All three have exact
statements and explicitly labeled summary proofs with prerequisites owned
by Chapters 5 and 6. They do not support the duality core and do not claim
that this completes the whole chapter.

## Hypotheses and providers

The retained evaluation and composition theorems use a commutative semiring
with additive commutative monoids and modules, without finite dimension.
The coordinate-action and characteristic-polynomial statements use a
commutative ring and a finite index with decidable equality. The determinant
statement has the same ring/index conditions; trace only needs a commutative
semiring. The three preview providers are respectively
`Matrix.charpoly_units_conj`, `Matrix.det_units_conj`, and
`Matrix.trace_units_conj`.

Dual-basis support uses a field, finite basis indices, and decidable equality
where required by the selected coordinate representation. The extension
construction takes a linearly independent family. The annihilator spanning
and uniqueness lemmas take an extended sum-indexed basis with finite blocks;
the dimension formula additionally requires `Module.Finite 𝕜 V`. Exercises
use real spaces except E03, which quantifies over an arbitrary field and
the standard two-dimensional coordinate space.

## Stable exercise prompt revisions

All six existing IDs and anchors remain. The new solution names are distinct
theorem declarations in `CrouzeixTextbook.Part01.Exercises.Chapter04`.
Their types are mathematical statements, not aliases or repetitions of the
public CFT cards. Integration still must check their compiler fingerprints.

| ID | Old prompt | New mathematical verification |
| --- | --- | --- |
| E01 | Define the dual space without coordinates. | Retain the definition question; verify φ(x,y)=2x−3y on v+rw and compute its pullback 2x−5y under T(x,y)=(x+2y,3y). |
| E02 | Compute the pullback in the first example. | Prove equality of the pulled-back functional (4,−1) with the functional (4,5), universally. |
| E03 | Prove dualization reverses composition. | For any 2×2 matrix over a field and any output functional, derive transpose action by evaluation on every standard basis vector. |
| E04 | Construct the dual basis of the standard basis of Rⁿ. | For every (a,b,c), prove its functional annihilates span((1,1,0)) iff a+b=0, and describe the resulting plane. |
| E05 | Show that transposing in the same order is generally wrong. | Prove positivity for Gram diag(2,1), the exact increment of ℓ=x+2y, pairing of (1/2,2) with every perturbation, and its inequality to (1,2). |
| E06 | Check dual_map_composition in Lean and explain each type. | Calculate all three pullbacks for S(x,y)=(2x,x+y), T(p,q)=(p+2q,3q), and φ(p,q)=4p−q: composite and reversed order give 13x+5y, the other order gives 7x+11y. |

The prose's conceptual explanations are distinguished from the actual
conjuncts checked by each theorem. E03's matrix indices are zero-based in
Lean and one-based in the displayed mathematics. E06 does not incorrectly
claim the two different functionals differ at every input.

## RED, GREEN, and focused verification

Following the TDD workflow, six `#check` clients were added before any
solution implementation. A direct pinned Lean invocation exited 1 with
exactly the six intended unknown-identifier errors for exercise_01_solution
through exercise_06_solution (plus the two pre-existing alias-linter warnings).
These same six clients remain at the end of the source.

After implementing and repairing the new proofs, the final direct compiler
invocation exited 0, emitted all six expected complete exercise types, and
had no warnings or errors. The equivalent command below uses worktree-relative
cache paths and is run from `formalization/lean`:

```sh
HARP_LEAN_PATH=/tmp/harp-foundations-olean.ajHyrU
for pkg in .lake/packages/*; do
  HARP_LEAN_PATH="$HARP_LEAN_PATH:$pkg/.lake/build/lib/lean"
done
LEAN_PATH="$HARP_LEAN_PATH" \
  /Users/bytedance/.elan/toolchains/leanprover--lean4---v4.32.1/bin/lean \
  CrouzeixTextbook/Part01/Chapter04.lean
```

The compiler reported Lean 4.32.1, commit
`f054605aea4b840552cca2e725580bffd1e1b704`, arm64 macOS. The worktree's
`.lake` symlink was read and verified as the primary checkout's canonical
cache. Imports of reviewed Chapters 1–3 came from the integration owner's
private overlay; Mathlib imports came from absolute warm package paths.
No Lake command, cache hydration, download, update, or shared output write
was performed. This is ordinary local, nonhermetic compilation, not an
independent human review or hermetic proof-execution receipt.

`git diff --check` passed for the owned source files. A source scan found no
`sorry`, `admit`, or `axiom`; all eight existing level-two headings, six CFT
anchors, and six exercise anchors remain. Named local Lean links include
verified declaration line locators. The canonical narrative was replaced
with `apply_patch`; the tool rejected one combined delete/add, requiring
separate operations and briefly leaving the file absent before restoration.
The source is present and frozen now.

The parent owns independent specification/quality review, compiler metadata,
web/PDF publication checks, integrated contracts, and the full repository
gate. This source-level success does not bypass those remaining requirements.

## Independent acceptance

The specification reviewer read the full pair, checked all six cards and
exercises and 24 named Lean links, then independently compiled the pinned
source successfully without warnings or errors. A separate quality reviewer
checked the full mathematical development and formal correspondence and
returned PASS with no critical or important findings.

The quality reviewer suggested clarifying “matrix unit” to avoid confusion
with elementary matrices. All three previews now say “invertible matrix” and
identify it as a unit in the matrix ring. The same reviewer verified that
reversing only those wording changes reproduces the previously reviewed
prose hash, checked the final hash and unchanged Lean source, and preserved
the PASS. The actual PDF renderer also passed the revised chapter's formula
and six named-solution reference check. Global publication is still a
coordinator-owned gate.
