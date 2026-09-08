# Chapters 2 through 4 foundations specification

Status: approved for implementation, subject to the recorded prerequisite checks. Parent: [mathematics program](2026-09-07-harp-mathematics-program-design.md).

## Scope and file ownership

Modify the three existing canonical chapters:

- `knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces.md`
- `knowledge/crouzeix_textbook/part_01_linear_structure/03_linear_maps_and_exact_structure.md`
- `knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality.md`

Update their matching Lean modules under
`formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean`, `Chapter03.lean`,
and `Chapter04.lean`. Extend the existing tests in
`crates/harp/tests/crouzeix_textbook.rs` and the actual reader-rendering tests
in `atlas/src/app/ChapterReader.test.tsx` as needed.

The integration owner updates `content/crouzeix_textbook/coverage.json`,
`exercises.json`, compiler-generated `Correspondence.lean`, current ledgers,
book navigation, status/claim counts, and generated Atlas artifacts together.
Keep the 36 chapters and 216 existing coverage and exercise IDs. Do not
repurpose an old theorem ID for a different statement. No duplicate coverage
manifest or manual compiler receipt is introduced.

## Reading experience

Each chapter starts with a concrete mathematical problem, develops the
coordinate-free objects, proves the central result, works through coordinates,
and returns to its invariant meaning. Aim for enough detail to reconstruct
proofs without Lean; word count is not acceptance. Avoid repetitive generic
motivation paragraphs. Use definitions and lemmas only when they serve the
chapter argument or an identified later use.

Introduce a redundant feature map T(x,y,z)=(x+z,y+z). Its null direction is
span((-1,-1,1)). Reuse it for subspaces, quotient classes, kernel/range,
rank-nullity, and pulling back output measurements. Distinguish equality of
linear predictors from claims about a nonlinear neural-network parameterization.

Use a second example for a nonnormal operator when coordinate changes arise.
Keep examples exact over real or complex scalars. Computational notation is a
translation of the mathematics, not the starting definition.

## Chapter 2: spaces, subspaces, and quotients

Retain the six CFT-02 statements and expand their proof context:

1. Span minimality: construct finite linear combinations and prove closure and
   leastness. Relate this construction to the intersection definition.
2. Intersections: prove subspace closure, then the membership equivalence.
   Show explicitly why a union of two distinct coordinate axes fails.
3. Basis coordinates: prove existence and uniqueness separately; explain
   finitely supported coordinates before specializing to finite index sets.
4. Dimension invariance: explain basis transport and where finite-dimensional
   interpretation is required. Lean's `finrank` statement can be more general
   than the elementary finite-dimensional explanation.
5. Direct sums: separate uniqueness from existence. Trivial intersection gives
   uniqueness; spanning V by the two subspaces is needed for every vector to
   admit a decomposition.
6. Quotients: prove equivalence-relation properties, well-defined addition and
   scalar multiplication, projection surjectivity, and the projection kernel.
   State and prove the factorization property used in Chapter 3.

Do not turn the small projection-surjectivity declaration into a claim that
it formalizes the entire quotient construction. Link each supporting result
to its own compiled declaration or identified Mathlib provider.

## Chapter 3: maps and exact structure

Retain all six CFT-03 identities; CFT-03-005 stays a definition.

- Show kernel and image closure before their membership characterizations.
- Prove rank-nullity through a basis of the kernel extended to the domain:
  images of the added basis vectors span the image and are independent.
  Every independence and spanning step must be written out.
- Derive injectivity iff zero kernel from differences of vectors.
- Construct the invariant restriction and verify its linearity and evaluation
  equation. Invariance is a hypothesis, not a property of every subspace.
- Explain composition range inclusion and distinguish it from equality.
- Construct V/ker(T) -> range(T), prove well-definedness and bijectivity, and
  show how it organizes rank-nullity without assuming a canonical complement.

Supporting first-isomorphism and basis-extension claims need their own Lean
links; the rank-nullity checkpoint alone is not their correspondence.

## Chapter 4: coordinates and duality

Use T^vee for the algebraic dual map and T^dagger for an inner-product adjoint.
Reserve V^* for the dual space. Explain transpose relative to dual bases;
conjugate transpose is an adjoint under an explicitly chosen Hermitian metric.

Fully develop CFT-04-001, 002, and 003: pullback evaluation, reversal of
composition, and coordinate conjugacy. Also develop dual bases, annihilators,
and the basis-extension proof of their dimension formula, with supporting
formal declarations.

Fix a convention explicitly: if x_new = S x_old, then
A_new = S A_old S^(-1). Do not change direction halfway through a calculation.
For a real output covector represented by coefficient column a, its pulled-back
column is A^T a. Derive the complex algebraic-dual analogue without confusing
it with the Hermitian adjoint.

Correct the current sentence that calls a gradient naturally a covector:
the differential is the covector; a gradient is a vector representing that
covector after a metric is chosen. A derivative-as-linear-map and dual-map
calculation may motivate JVP/VJP. A full autodifferentiation correctness theorem
and complex-array API conventions are outside this chapter's formal claims.
Any concrete API example must be checked against pinned official documentation.

### Preserve forward references honestly

CFT-04-004, 005, and 006 retain their characteristic-polynomial, determinant,
and trace statements and exact Lean links. Keep them in a clearly labeled
forward-reference section whose proof prerequisites are Chapters 5 and 6.
Do not pretend they have self-contained proofs in the duality reading path.
Do not use these previews to prove the duality core or create dependency cycles.

This slice may establish exact statement correspondence for those three rows
while leaving `prose_proof_status = summary`. It does not close the complete
Chapter 4 issue until those proof obligations are resolved in their owning
foundations wave. A formula's meaning must still be explained in the preview;
an undefined characteristic polynomial is not useful orientation.

## Eighteen exercises and formal solutions

Keep six stable exercise IDs per chapter and record any prompt revision.
Use `CrouzeixTextbook.Part01.Exercises.ChapterXX.exercise_YY_solution`.
All 18 solutions are distinct theorem statements and compile. Retrieval
questions gain an explicit mathematical verification component; prose-only
commentary is not assigned a fake theorem proof.

Required mathematical tasks, ordered within each chapter:

| Chapter | E01 | E02 | E03 | E04 | E05 | E06 |
| --- | --- | --- | --- | --- | --- | --- |
| 2 | Verify zero membership and closure in a concrete subspace | Span of (1,1),(1,-1) over R | Prove a concrete span inclusion via combinations | Factor (x,y) -> x-y through the diagonal quotient | Prove the union of coordinate axes is not a subspace | Identify equal predictor outputs with a null-direction difference |
| 3 | Compute zero-output membership for the running feature map | Compute its image and kernel with explicit bases | Construct the quotient-to-image map and its inverse on classes | Prove injectivity of a chosen restriction | Exhibit strict composition-range inclusion | Prove restriction preserves powers on an invariant subspace |
| 4 | Verify a concrete linear functional and its pullback | Compute pullback of (4,-1) under T(x,y)=(x+2y,3y) | Derive the transpose action from basis evaluations | Compute the annihilator of span((1,1,0)) | Exhibit the distinction between differential coefficients and gradient under a nonidentity positive metric | Verify a two-map pullback calculation and its composition order |

For E05, use the concrete real metric with Gram matrix diag(2,1) and
ell(x,y)=x+2y. Its differential has coefficients (1,2), while its gradient
has coordinates (1/2,2). Prove that pairing this vector with any perturbation
under the chosen metric gives that differential. Define the metric locally,
without presuming a full Chapter 7 theorem. Each solution
has a statement fingerprint different from the chapter's public cards. Reusing
legitimate lemmas is allowed; aliasing a checkpoint or restating a parent card
is not an exercise solution. Mathlib support is labeled as support.

## Expected coverage delta and limits

For this foundations slice alone, assuming the frozen baseline and no concurrent
promotions: 18 additional exact correspondences consist of 17 theorem statements
and one definition, bringing the count from 72 to 90. Fourteen theorem proofs
become reconstructible, bringing that count from 71 to 85. Summaries decrease
from 143 to 129; two not-applicable exposition rows remain, including the
invariant-restriction definition. The three Chapter 4 previews remain summaries.

Eighteen additional solved exercises bring the count from 78 to 96 and leave
120 unresolved. Recompute these numbers from the contracts; do not force them
if the baseline changes. Formal-mode totals depend on honest provider
classification and are not quota targets. Extra unindexed support lemmas do
not inflate the 216-row coverage roster.

## Acceptance tests

- Every retained ID and anchor resolves to the same mathematical statement.
- All 18 registered rows have truthful exact correspondence, with definitions
  and forward references kept distinct from reconstructed theorem proofs.
- All 18 exercise statements match their written solutions and compile.
- Test fake solution reuse, declaration mismatch, missing assumptions, stale
  positions, and changed source hashes using temporary fixtures.
- Check the prerequisite graph for cycles and prevent Chapter 4 previews from
  becoming prerequisites of its own foundational arguments.
- A reader can solve the running feature-map example using only Chapters 1
  through the current chapter and explicitly supplied local definitions.
- Review actual web math and source links, then the rebuilt full PDF and
  preserved standalone editions. No terminal proof provider changes are needed.

Acceptance for the core slice does not imply that all of Chapter 4, all of
Part I, or the complete textbook is finished. Preserve the existing Kata
blockers until their actual acceptance conditions are satisfied.
