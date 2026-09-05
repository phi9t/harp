# Chapter 36: The Harp finite-horizon proof

Status: chapter design approved in conversation; written specification awaiting review.
Kata: `fq4m`. Base: `6f26d6e9`. Branch: `codex/textbook-harp-proof`.

## Outcome and scope

Add a full proof workshop to the existing textbook. A reader with undergraduate
mathematics and research-level ML experience should be able to reconstruct the
finite-horizon argument, identify every hypothesis, and follow each formal
claim into Lean. The chapter must explain the calculations, not merely name
the lemmas that perform them.

Use the title **The Harp finite-horizon proof**, document ID
`cft-chapter-36-harp-finite-horizon-proof`, and canonical path
`knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof.md`.
Add `formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean`.
Keep all existing chapter numbers, document IDs, and anchors stable.
Chapter 35 gains a next-chapter link and a forward reference from its comparison.
Chapter 36 ends with a return to that comparison and the book index.

The alternatives considered were expanding Chapter 35 and adding an appendix.
Chapter 35 already covers terminal consequences and comparison at length.
A separate chapter gives the construction enough space without renumbering
the book or making Harp an ancillary appendix.

This is exposition of a local derived formalization. Do not describe Harp as
an independent historical manuscript, claim priority, or imply that its proof
has no shared dependencies with Lorist–Schwenninger. Do not change certified
route providers merely to simplify exposition. An actual discrepancy between
prose and a provider is a review finding to resolve explicitly.

## Reader experience

Lead with mathematics. Use square-bracket matrices, distinguish operators
from their coordinate representations, and state the complex inner-product
convention before expanding adjoints. Define each symbol before use.

Start with the question: how can a different finite model at every horizon
prove one operator inequality? Introduce the finite-dimensional objects first,
then use them to explain the abstract Lean records. Supply a short prerequisite
recap of positive matrices, adjoints, isometric compression, boundary moments,
and geometric series, with links to the full earlier developments.

Each major section contains a purpose, statement, hypothesis ledger, proof
roadmap, complete calculation, boundary case, and Lean correspondence.
Label Motivation, Historical context, and ML analogy separately. An analogy
must say what transfers exactly and what does not. In particular, exact
positive cubature is not an approximate numerical quadrature certificate.
No PyTorch or JAX dependency is introduced; array notation is explanatory.

## Six proof sections and coverage records

Reserve `CFT-36-001` through `CFT-36-006`. Each record has one exact primary
statement; supporting lemmas have explicit source links and hypothesis maps.
Do not label a multi-lemma discussion exact merely because its endpoint has
a Lean theorem. Existing providers may be reexported, with that mode stated.
New pedagogical deductions must have actual proof bodies.

### 001. Positive cubature for a finite list of moments

Explain the real finite-dimensional observable containing all complex matrix
moments for powers zero through `N+1`. Prove continuity and explain why
integration gives a point in the appropriate convex hull. Develop the finite
convex combination argument, deletion of zero weights, and rescaling from a
probability measure to a finite nonzero measure. Make positivity, total mass,
and simultaneous preservation of every coordinate explicit.

Use `CrouzeixConjecture.Harp.exists_positive_matrix_moment_cubature` as the
primary provider in `FiniteAtomicDilation.lean`. Its prerequisites are in
`PositiveCubature.lean` and `FiniteMeasureCubature.lean`.
Present the provider's cardinality bound as real dimension plus one. Any
simplification of that dimension into a matrix-size formula needs a separate
checked derivation; do not silently improve the bound.

### 002. Turn the moments into a finite dilation

Construct the positive atomic density, its square roots, the finite
counting-measure L2 space, the embedding, and the multiplication operator.
Display the exact mass-two identity and the normalization making the
embedding isometric. Derive the contraction and compressed moment identities,
including all factors of two and adjoints. Explain why powers through `N+1`
are sufficient for `N` adjacent recurrence steps.

The primary statement is existence of a
`CrouzeixConjecture.Harp.FiniteAtomicL2DilationWitness core N`, constructed by
`finiteAtomicL2DilationWitness` in `FiniteAtomicL2Dilation.lean`.
Link `finiteAtomicDoubleLayer_mass_eq_two_one`,
`finiteAtomic_compression_eq_of_moments`, and `finrank_countL2` for the
supporting calculations. Name the finite-dimensionality assumptions explicitly.

### 003. Derive the finite recurrence

Introduce the horizon-independent target `T`, commuting perturbations `E_k`,
and their uniform norm bound `B`. Then introduce `K_N`, `V_N`, and `Q_N`.
State the exact compressed-adjoint identity from `FiniteHorizonDilationData`.
For a unit top singular vector `x`, define `kappa = norm T`, the recurrence
scalar `m_k`, and the squared displacement `b_N` using the source definitions.

Expand the adjacent-power defect and its factorization. Explain each use of
commutation, adjoints, contraction, and the completed-square estimate.
Derive the finite weighted inequality, retaining its terminal term:

```text
kappa^(-N) m_(N+1)
  + sum_(i=0)^(N-1) kappa^(-(i+1)) [-b_N/(kappa^2-kappa)] <= m_1.
```

The primary provider is
`FiniteHorizonDilationData.equation_three_finite_lower_bound` in
`FiniteHorizonOperatorRecurrence.lean`, under namespace
`CrouzeixConjecture.Harp`. Include the `N=0` boundary and the precise
`1 <= k <= N` range for adjacent steps. If discussing Lean's auxiliary
extension of the finite scalar sequence, distinguish it from an extension
of the actual dilation.

### 004. Remove the horizon and obtain constant two

The crucial quantifiers are a fixed core and a witness for every `N`.
The spaces and nodes may depend on `N`. No compatibility, nesting, uniform
dimension bound, or common infinite atomic dilation is assumed.

Handle `kappa <= 1` first. For `kappa > 1`, derive

```text
C = 2 kappa^2 - kappa m_1 - kappa^3,
0 <= b_N <= C,
|m_k| <= B (2+B).
```

Explain why replacing `b_N` by `C` preserves the inequality direction.
Prove that the bounded terminal term tends to zero and compute the geometric
sum limit. Obtain `-C/[kappa (kappa-1)^2] <= m_1`.
Use the horizon-zero witness to justify `C >= 0`. Display the scalar
combination giving

```text
C [1 - 1/(kappa-1)^2] <= kappa^2 (2-kappa).
```

For `kappa > 2`, the left side is nonnegative and the right side is negative.
Show the signs and every denominator condition instead of hiding this step
behind a tactic name.

Primary provider: `norm_target_le_two_of_finiteHorizonDilationData` in
`FiniteHorizonPerturbation.lean`. Supporting providers include
`finite_weighted_inequalities_to_limit_lower_bound` in
`FiniteHorizonRecurrence.lean`, the norm-attainment lemma, and the reused
LS scalar endpoint. Attribute those shared lemmas explicitly.

### 005. Recover the arbitrary-matrix polynomial bound

Follow `CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem` in
`MainTheorem.lean`. Start with a fixed smooth outer domain and a
simple-spectrum approximating matrix. Define the maximum `M` on the closed
outer domain. Handle `M=0` by eigenvalue evaluation and diagonalization;
for `M>0`, normalize the polynomial by `M`, apply the finite-horizon theorem,
and rescale. Explain where the polynomial Cauchy formula is supplied.

Keep the limits in order: first the simple-spectrum approximation index
with the domain fixed, then the shrinking outer domain. Display the
continuity and compactness statements used at both steps. The conclusion is
`norm (p(A)) <= 2 max_(z in W(A)) |p(z)|` for arbitrary finite complex
matrices under the provider's nonempty-index convention. No normality or
diagonalizability assumption remains. Match `n : Type` in the provider;
do not silently generalize its universe.

### 006. Consequences and what the proof does not supply

Use the Harp provider with the existing neutral rational and Hilbert-space
adapters. Explain the finite Krylov compression and the pole-free condition,
with precise references to Chapter 35 for the full shared adapter proofs.
Restate the distinction between a numerical range and its closure for a
bounded operator on a Hilbert space.

Primary provider:
`CrouzeixConjecture.harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet`
in `Consequences.lean`. Also link the finite-matrix, rational, and polynomial
Hilbert-space providers there. Do not claim complete boundedness,
constructive node-finding, numerical stability of cubature, or bounds for
nonlinear/time-varying ML systems.

## Exercises

Add six exercises, `CFT-36-E01` through `CFT-36-E06`, with worked prose and
distinct Lean solutions in `Chapter36.lean`:

1. Project one preserved moment from a simultaneous finite cubature identity.
2. Derive an isometry identity from the normalized positive atomic mass.
3. Telescope a finite weighted recurrence and retain its terminal term.
4. Derive the horizon-independent displacement substitution with the correct sign.
5. Undo positive polynomial normalization; discuss the zero case separately.
6. Assemble the finite-to-limit scalar endpoint from supplied inequalities,
   without invoking a terminal Crouzeix theorem.

Each exercise must ask for a genuine deduction. Calling the exercise's parent
checkpoint as its entire solution does not qualify. Supporting lemmas may be
reused. Numerical diagnostics and historical discussion are labeled prose,
not falsely marked Lean-proved exercises.

## Formal correspondence and dependency discipline

For every primary theorem and exercise, record its full declaration name,
actual source locator, statement, hypotheses, correspondence mode, compiler
type hash, and axiom/dependency information using the existing exporter.
Generate these values from the pinned compiler; never invent receipt hashes.
Links must reach the actual declaration in both the hosted reader and PDF.

The intended terminal and construction providers are reexports unless the
textbook adds a genuinely different statement. Describe a new wrapper honestly.
Use `proved-here` for actual new deductions, not aliases renamed for coverage.

Chapter 36 may import Harp construction and terminal modules and their existing
LS support. It must not import Jin or LS terminal providers as substitutes.
Do not import Chapter 35 into Chapter 36 merely for prose navigation.
Keep Chapters 1–29 free of new terminal dependencies. Distinguish pedagogical
reading links from actual Lean proof dependencies in the dependency map.

## Publication integration

Extend the registered packet, chapter roster, coverage, exercises, source
registry, reading guide, index, notation, and dependency map consistently.
The intended roster is 36 chapters, 216 theorem records, and 216 exercises.
Recompute exactness and solution totals from the resulting contracts rather
than inserting anticipated values into prose.

Audit current 35-chapter and 210-record assumptions in Rust, Python, Lean
generation, fixtures, Atlas, and PDF tooling. Change live roster expectations;
preserve intentionally historical fixture counts and immutable evidence.
Add regression tests for the new chapter, not a blanket text replacement.
Keep malformed or duplicate chapter identities rejected.

Regenerate the compiler correspondence, managed ledgers, corpus JSON, and
static Atlas export through their existing owners. If aggregate proof evidence
requires a refresh, publish a new evidence generation; never rewrite captured
bytes or historical manifests. Preserve all existing website routes.

Build the whole-book PDF and a standalone Chapter 36 PDF using the existing
typography. Check equation wrapping, page breaks, theorem headings, navigation,
source links, and embedded source snapshots. No UI redesign or new rendering
framework is part of this change.

## Verification and completion criteria

Review each proof section against its provider immediately after drafting it.
Check factors, adjoints, indices, strict inequalities, and quantifier order.
Review the completed chapter as a reader, checking that no decisive step is
replaced by a theorem name alone. Record defects and their resolutions.

Preflight the pinned Lean toolchain and canonical cache before execution.
Do not hydrate dependencies. Run narrow Harp/textbook targets while iterating,
then the full Lean and repository gates on the frozen candidate. Local compile
results must not be described as hermetic execution without the required
Seatbelt enforcement and receipts.

Acceptance requires:

- Six primary statements and six distinct exercises compile with resolved links.
- No new project axiom, `sorry`, or substituted terminal provider.
- The manuscript reproduces the finite recurrence and both terminal limits.
- The six new correspondence records accurately describe their proof modes.
- Existing chapter URLs and the three-route comparison remain valid.
- Website and both PDFs include the same canonical Chapter 36 text.
- Focused contract, corpus, reader, and PDF checks pass.
- `mise run verify` passes before a commit or landing.

Use explicit staging and preserve unrelated changes. No push, deployment,
proof-cache maintenance, or mainline merge is authorized by this specification.

## Workflow and review record

- [x] Inspect the existing textbook and Harp providers.
- [x] Confirm audience and mathematical style from the conversation.
- [x] Compare chapter expansion, appendix, and dedicated chapter approaches.
- [x] Obtain approval for the dedicated chapter design.
- [x] Write the specification in an isolated worktree.
- [x] Self-review scope, provider names, quantifiers, and publication requirements.
- [ ] Obtain review of this written specification.
- [ ] Write the implementation plan after that review.
- [ ] Implement, review each section, verify, and commit the candidate.

Visual design exploration is unnecessary for this content addition. Existing
PDF and website presentation conventions remain in force.
This specification is not a proof-verification receipt. No Lean build or full
repository gate has been run for it. Keep it uncommitted until the required
pre-commit gate has run; do not infer fresh verification from the base commit.
