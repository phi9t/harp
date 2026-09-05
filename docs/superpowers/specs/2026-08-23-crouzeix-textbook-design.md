# From linear maps to numerical ranges textbook design

**Status:** Approved in conversation; awaiting written-spec review

**Date:** 2026-08-23

**Audience:** Frontier-lab MTS researchers with undergraduate mathematics and
PhD-level machine-learning research experience

**Canonical prose root:** `knowledge/crouzeix_textbook/`

**Lean root:** `formalization/lean/CrouzeixTextbook/`

## 1. Objective

Create a self-contained mathematical textbook that develops the reader from
structural undergraduate linear algebra through the analytic and
operator-theoretic machinery needed to understand the constant-two Crouzeix
result.
The exposition takes its structural cues from Peter Lax's *Linear Algebra and
Its Applications*, uses the calculus and multilinear-geometry lineage of
Michael Spivak's *Calculus on Manifolds*, and draws on the other registered
Mathematical Foundations and Autodiff Geometry sources. All prose, examples,
figures, and exercises are newly authored.

The book is written for researchers whose working intuition comes from modern
machine learning, including large-language-model training, vision,
optimization, autodiff, representation geometry, and numerical computation.
Those applications motivate and interpret the mathematics without setting its
claim ceiling. The mathematical development remains rigorous and is allowed to
go substantially deeper than normal ML prerequisite material.

The book ends with both source-backed 2026 candidate proof routes:

1. Jin's positive-real completion, sampled-kernel cancellation, and
   weighted-Gramian route; and
2. Lorist--Schwenninger's compressed contraction-power,
   commuting-perturbation, and scalar-recurrence route.

The book must preserve the existing Crouzeix packet's status language. It is a
teaching and formalization artifact, not a peer-review certificate or a claim
that external publication and independent review are complete.

## 2. Reader contract

### 2.1 Assumed background

The reader is expected to have:

- an undergraduate mathematics education;
- familiarity with matrices, derivatives, integrals, and elementary proofs;
- research-level ML experience; and
- no required prior exposure to functional analysis, operator theory,
  differential geometry, complex analysis, or Lean 4.

Each advanced topic begins from its motivating problem and definitions. The
book does not treat a term as a prerequisite merely because it is conventional
among operator theorists.

### 2.2 Completion outcomes

A successful reader can:

1. distinguish a linear transformation from its coordinate matrix;
2. reason fluently with duals, adjoints, Gram matrices, singular values,
   positive order, and polar decompositions;
3. use multilinear derivatives, differential forms, pullbacks, and Stokes'
   theorem in their natural domains;
4. prove and apply the finite-dimensional analytic facts used by the
   Crouzeix arguments;
5. explain holomorphic functional calculus, numerical ranges, spectral sets,
   positive maps, kernels, compressions, and dilations;
6. derive the symmetrized double-layer identity and locate the information
   loss behind the earlier `1 + sqrt 2` estimate;
7. reconstruct both constant-two proof architectures without relying on a
   memorized summary;
8. identify every load-bearing hypothesis, order-sensitive operation, and
   limiting argument in the two routes;
9. read, run, and extend the accompanying Lean 4 development; and
10. explain what the formalization verifies and what remains outside Lean's
    statement, source-model, and trust boundaries.

### 2.3 Pace

The normal chapter is designed for two to four focused hours including
exercises and Lean work. Chapters are divided into 20--40 minute sections.
Long proofs are staged through named lemmas and retrieval checkpoints rather
than presented as uninterrupted walls of symbols.

Parts II and III include fast-recall routes for readers who can demonstrate
mastery through their checkpoint problems. The fast route omits repetition,
not dependencies: no later theorem may use an undefined object or an
unrecorded result.

## 3. Pedagogical model

The book uses a layered, cumulative structure. It does not begin with the
Crouzeix statement and backfill disconnected prerequisites, and it does not
attempt encyclopedic coverage of either Lax or Spivak. It teaches a coherent
mathematical route broad enough to make the proof machinery natural.

Every chapter follows this sequence:

1. **Opening problem.** A concrete tension or question that creates the need
   for the chapter's structures.
2. **Conceptual model.** Coordinate-free meaning before calculation.
3. **Formal development.** Definitions, lemmas, theorems, and complete proofs.
4. **Worked examples.** Small exact examples followed by proof-relevant
   examples.
5. **ML bridge.** A bounded connection to an ML research object such as a
   Jacobian, Hessian, covariance operator, attention map, optimizer, kernel,
   or stability calculation.
6. **Lean translation.** The chapter's definitions, theorem statements,
   library choices, and proof patterns in Lean 4.
7. **Exercises.** Retrieval, calculation, written proof, counterexample, and
   Lean proof tasks.
8. **Synthesis.** Dependency summary, common errors, and downstream uses.

The ML bridge is explanatory background. It does not turn a controlled
mathematical theorem into a claim about arbitrary training systems. Empirical
or implementation claims require their own evidence and are labeled as such.

## 4. Book structure

### Part I -- Linear structure

#### Chapter 1: Mathematical objects and their representations

Sets, functions, proof patterns, linear transformations versus matrices,
coordinate maps, and the Lean distinction among data, propositions, and
proofs.

#### Chapter 2: Vector spaces and subspaces

Span, independence, bases, dimension, sums, intersections, and quotient
spaces.

#### Chapter 3: Linear maps and exact structure

Kernels, images, rank-nullity, invariant subspaces, compositions, inverses,
and commutative diagrams.

#### Chapter 4: Coordinates and duality

Change of basis, similarity, dual spaces, annihilators, transpose maps, and
coordinate-free reasoning.

#### Chapter 5: Determinants, trace, and exterior algebra

Alternating multilinear forms, determinants, orientation, trace, and their
invariance properties.

#### Chapter 6: Eigenvalues and polynomial algebra

Characteristic and minimal polynomials, diagonalization, generalized
eigenspaces, polynomial identities, and simple spectrum.

### Part II -- Geometry and calculus

#### Chapter 7: Inner-product spaces

Orthogonality, projection, Riesz representation, adjoints, and orthonormal
bases.

#### Chapter 8: Positive operators and Gram geometry

Positive-semidefinite order, Gram matrices, congruence, square roots, and
Cholesky factorization.

#### Chapter 9: Operator norms and singular values

Maximum stretch, singular vectors, polar decomposition, condition numbers,
and finite-dimensional norm attainment.

#### Chapter 10: Multilinear maps and tensors

Bilinear forms, tensor products, alternating forms, contractions, and
coordinate transformations.

#### Chapter 11: Differentiation as linear approximation

Fréchet derivatives, Jacobians, the chain rule, higher derivatives, and the
inverse and implicit function theorems in the finite-dimensional setting used
by the book.

#### Chapter 12: Differential forms and Stokes' theorem

Pullbacks, exterior derivatives, orientation, integration on parameterized
domains, and the generalized Stokes theorem. This chapter supplies the
Spivak-lineage geometric synthesis without claiming that manifold machinery
is itself a hidden prerequisite of Crouzeix.

### Part III -- Analysis and complex functions

#### Chapter 13: Metric and normed spaces

Completeness, compactness, continuity, boundedness, equivalence of norms in
finite dimensions, and the points where infinite-dimensional behavior
differs.

#### Chapter 14: Sequences and series of operators

Operator-norm convergence, geometric series, uniform convergence, exchanging
limits with bounded linear operations, and continuity of inversion.

#### Chapter 15: Complex differentiability

Holomorphic functions, power series, contour integration, Cauchy's theorem,
and Cauchy's formula.

#### Chapter 16: Consequences of Cauchy theory

Maximum modulus, open mapping, the identity theorem, residues, and the
approximation facts needed by later functional calculus.

#### Chapter 17: Functions of matrices and operators

Polynomial, rational, and holomorphic functional calculi; resolvents; spectral
mapping; and contour definitions.

#### Chapter 18: Positive-real analytic functions

Cayley transforms, harmonic real parts, scalar Herglotz theory, positive
matrix-valued kernels, and the finite-sampling form used later.

### Part IV -- Finite-dimensional operator theory

#### Chapter 19: Normality and nonnormality

Unitary diagonalization, transient amplification, conditioning of eigenbases,
pseudospectral intuition, and ML-relevant stability examples.

#### Chapter 20: The numerical range

Definition, convexity, spectrum inclusion, affine behavior, compressions,
supporting lines, and `2 x 2` geometry.

#### Chapter 21: Spectral sets

Scalar control versus operator control, rational inequalities, amplification,
and the distinction between spectral and completely bounded spectral sets.

#### Chapter 22: Positive and completely positive maps

Matrix order, positivity, unitality, Kadison-type inequalities, matrix levels,
and the difference between positivity and complete positivity.

#### Chapter 23: Compression and dilation

Isometries, contractions, compressions, unitary dilations, Stinespring
representations, and power families.

#### Chapter 24: Gramians and ordered matrix inequalities

Weighted Gramians, Lyapunov equations, Schur complements, congruence,
anticommutators, and noncommutative order.

### Part V -- Machinery behind Crouzeix

#### Chapter 25: Convex boundaries and Cauchy layers

Smooth outer domains, boundary parametrization, normal vectors, Cauchy
transforms, and approximation by fixed outer domains.

#### Chapter 26: The double-layer map

Positive boundary density, unitality, the companion transform, and the
symmetrized identity

```text
2 Phi(f) = f(A) + alpha(f)(A)*.
```

#### Chapter 27: Why the old estimate stops at `1 + sqrt 2`

The Crouzeix--Palencia argument, the independent companion estimate, and the
structural loss caused by discarding the coupling.

#### Chapter 28: Preserving the complete power family

Powers `f^n`, generating functions, Cayley packaging, commuting perturbations,
and the carefully bounded comparison between the two candidate proofs.

### Part VI -- The constant-two routes

#### Chapter 29: The Crouzeix problem and sharpness

Precise polynomial, rational, and spectral-set formulations; the nilpotent
sharpness example; and current evidence and publication boundaries.

#### Chapter 30: Jin I -- constructing the positive-real completion

Outer domains, simple-spectrum approximation, auxiliary eigenbases, the
Cayley family, and the adjoint-algebra defect.

#### Chapter 31: Jin II -- canceling the correction

Herglotz-kernel sampling, repeated samples, the origin vector, exact
diagonal-defect cancellation, and the ordered pre-Gramian inequality.

#### Chapter 32: Jin III -- extracting the constant two

Balancing by the Gram square root, the two weighted Gramians, the
anticommutator argument, polar-unitary transfer, and the correctly ordered
limits.

#### Chapter 33: Lorist--Schwenninger I -- the perturbation lemma

Compressed contraction powers, uniformly bounded commuting perturbations,
top singular vectors, completion of squares, and the scalar recurrence.

#### Chapter 34: Lorist--Schwenninger II -- realization and contradiction

Terminal boundedness, the contradiction above two, the double-layer
realization, and the abstract uniform-algebra variant.

#### Chapter 35: Comparison, verification, and open boundaries

Shared power families without false equivalence, completely bounded
limitations, Lean theorem graphs, axiom and source-model boundaries, and
remaining formalization work.

## 5. Canonical artifact architecture

The book is a registered Harp knowledge packet. Canonical technical prose
lives under `knowledge/`; structured coverage data lives under `content/`;
Lean source lives under the shared formalization root.

```text
knowledge/crouzeix_textbook/
├── crouzeix_textbook_index.md
├── reading_guide.md
├── notation_and_glossary.md
├── theorem_dependency_map.md
├── exercise_index.md
├── lean_coverage_ledger.md
├── source_registry.md
├── claim_evidence_ledger.md
├── status_and_scope.md
├── part_01_linear_structure/
│   ├── 01_objects_and_representations.md
│   └── ...
├── part_02_geometry_and_calculus/
├── part_03_analysis_and_complex_functions/
├── part_04_operator_theory/
├── part_05_crouzeix_machinery/
└── part_06_constant_two_routes/

content/crouzeix_textbook/
├── coverage.json
├── exercises.json
└── theorem_dependencies.json

formalization/lean/
├── CrouzeixTextbook.lean
└── CrouzeixTextbook/
    ├── Part01/
    ├── Part02/
    ├── Part03/
    ├── Part04/
    ├── Part05/
    └── Part06/
```

The exact chapter filenames use zero-padded chapter numbers and stable slugs.
Every chapter links to the previous and next chapter, its part index, the book
index, and any referenced canonical Crouzeix packet documents.

The existing Atlas corpus is the presentation layer. Implementation adds the
book to the corpus and regenerates
`atlas/src/content/generated/corpus.json` and `atlas/dist/harp-atlas.html`
together. No second prose authority or bespoke runtime renderer is introduced.

## 6. Stable identifiers and coverage

Every formal book item receives a stable identifier:

```text
CFT-<chapter>-<sequence>
```

Examples:

- `CFT-08-014` -- a theorem in Chapter 8;
- `CFT-20-006` -- a worked numerical-range example; and
- `CFT-31-009` -- the correction-cancellation lemma.

Identifiers are never reused after publication. A corrected item retains its
identifier and records the revision in the claim and coverage ledgers.

`content/crouzeix_textbook/coverage.json` is the machine-readable authority
for coverage. Each row records:

- item ID;
- chapter;
- item kind;
- prose locator;
- Lean declaration when applicable;
- formal status;
- exercise or example links;
- source-lineage IDs;
- downstream dependents; and
- verification target.

Formal status is exactly one of:

- `compiled-lean`;
- `executable-check`;
- `non-formal-prose`; or
- `open-formalization`.

The final book may contain `open-formalization` rows while visibly in draft.
The completion gate rejects every mathematical definition, lemma, theorem,
worked derivation, and proof exercise that remains open.

Historical remarks, pedagogical analogies, empirical observations, and reader
instructions are `non-formal-prose`. This status cannot be used to avoid
formalizing a mathematical claim.

## 7. Lean architecture

### 7.1 Shared project and dependencies

The book adds a `CrouzeixTextbook` library to the existing shared Lean project.
It uses the repository's pinned Lean 4.32.1 toolchain and existing Mathlib
revision. It adds no runtime or source dependency on another checkout.

Proof iteration follows the repository's cache discipline:

- verify the toolchain and warm primary `.lake` cache before invoking Lake;
- symlink an isolated feature worktree to the verified primary cache;
- never run `lake update`, Mathlib cache hydration, or global toolchain setup;
- use a narrow module or part target while iterating; and
- run the full library only after a candidate part is frozen.

Missing dependency cache state is a typed blocker, not permission to download
or regenerate dependencies.

### 7.2 Chapter-facing modules

Each chapter owns one public Lean module with the same conceptual boundary as
the prose chapter. Large chapters may split private definitions and helper
lemmas into sibling modules, but the public chapter import remains small and
stable.

Public theorem names describe mathematical content rather than page or section
numbers. Each public declaration receives a source comment containing its CFT
identifier. The coverage generator validates the one-to-one mapping.

The book may reuse existing declarations from:

- `MathematicalFoundations`;
- `AutodiffGeometry`;
- `CrouzeixConjecture`; and
- `Crouzeix` proof-route modules.

Reuse requires exact statement compatibility. A textbook-facing bridge lemma
may restate an existing theorem at a clearer interface, but it may not silently
strengthen, weaken, or rename a gap as a result.

### 7.3 Exercises and solutions

Every proof-oriented exercise has:

- a precise mathematical statement in the chapter;
- a Lean theorem statement;
- a checked reference proof; and
- a difficulty and prerequisite classification.

Canonical Lean files remain proof-hole free. The reader-facing chapter shows a
copyable theorem signature and proof scaffold. The repository stores the
compiled reference solution in a solution namespace. Optional scratch files
may be generated locally for practice, but generated files are ignored and are
not evidence of completion.

Calculation exercises use exact arithmetic where possible. Floating-point or
external-library examples are correspondence checks, not substitutes for the
exact theorem.

### 7.4 Axiom and proof-hole boundary

No completed textbook theorem may depend on:

- `sorry` or `admit`;
- an unregistered custom axiom;
- unsafe proof construction;
- a native-decision shortcut that exceeds the repository policy; or
- an imported theorem absent from the declared dependency surface.

Public capstone theorems receive machine-captured axiom output. The trust
boundary explicitly includes Lean's kernel, Mathlib, the compiler/runtime,
formal definitions, and the correspondence between the formal statement and
the prose theorem.

## 8. Source and copyright boundary

The textbook uses the following source roles:

- **Lax 2007:** structural linear-algebra sequence, vocabulary, and theorem
  orientation from the user-supplied local reference scan;
- **Spivak, Calculus on Manifolds:** curriculum structure and notation lineage
  through the registered, content-gated repository evidence;
- **Bishop 2006:** ML-oriented examples and modelling vocabulary from the
  user-supplied local reference scan;
- **JAX documentation and source:** immutable autodiff notation and API
  locators;
- **SICM and FDG:** rights-cleared geometric notation and selected examples
  after claim-specific review;
- **Mathlib and Lean documentation:** formal definitions, theorem interfaces,
  and proof-engineering guidance; and
- **Crouzeix primary sources:** theorem history, double-layer prerequisites,
  the `1 + sqrt 2` result, the two constant-two routes, and formalization
  status.

The book does not reproduce source prose, figures, textbook exercises,
solutions, scans, or gated source bytes. It contains newly authored
exposition, examples, diagrams, and exercises. All material source claims use
the project's evidence classes and clickable claim routes.

The Spivak source remains `structure-parsed-content-gated`. Its outline can
guide chapter organization, but direct theorem wording or proof reproduction
requires a rights-cleared locator. Conventional mathematics may be proved
independently and supported by Mathlib or other lawful primary sources.

No repository-wide license is added.

## 9. Claim discipline

The main chapters teach without drowning the reader in provenance metadata.
Material source, historical, publication-status, implementation, and
formalization claims link to the packet claim ledger.

Claims use the repository's exact classes:

- `EVIDENCE` for bounded observations of inspected artifacts;
- `SOURCE CLAIM` for an author's reported result or interpretation;
- `INFERENCE` for Harp's synthesis; and
- `MISSING` for evidence required by a stronger conclusion.

Conventional definitions and independently supplied mathematical derivations
do not require a claim block in every paragraph. The source registry still
records their lineage and applicability. A theorem attributed to a named
author, a publication-status statement, or a claim that Lean verifies a route
is material and requires an exact ledger entry.

## 10. Verification architecture

### 10.1 Structural checks

Repository tests verify:

- the exact 35-chapter roster;
- required frontmatter and stable IDs;
- root-qualified internal wikilinks;
- previous/next/part/index navigation;
- glossary terms introduced before required use;
- no Markdown under `content/`;
- no source paths that expose local-only book files;
- exercise and solution pairing; and
- valid source, claim, and coverage ledgers.

### 10.2 Coverage checks

The coverage verifier fails when:

- two items share an ID;
- a mathematical item lacks a Lean declaration or remains open at completion;
- a declared Lean theorem is absent from the compiled declaration inventory;
- a prose item claims proof without a compatible compiled statement;
- an exercise lacks a checked solution;
- a dependency points forward without an explicit preview contract; or
- a final Crouzeix theorem is disconnected from the prerequisite graph.

### 10.3 Lean checks

Each part has a narrow check route. A frozen part also runs the full
`CrouzeixTextbook` library. Checks include:

- compilation under the pinned toolchain and warm cache;
- prohibited-token scanning;
- public-declaration inventory generation;
- axiom audits for capstone theorems; and
- statement-digest checks for the final Crouzeix endpoints.

### 10.4 Content and presentation checks

Corpus and Atlas checks verify that:

- every chapter is discoverable and rendered;
- equations survive Markdown-to-HTML rendering;
- internal links resolve;
- the book index and guided routes appear in search;
- generated corpus and Atlas HTML are refreshed together; and
- no derived artifact is hand-edited or left stale.

### 10.5 Pedagogical review

Each part receives a content review independent of Lean compilation. The
review checks:

- definitions precede use;
- examples exercise the stated concept rather than a nearby one;
- proofs expose their load-bearing steps;
- ML bridges remain accurate and bounded;
- retrieval questions test storage strength rather than recognition alone;
- difficulty rises gradually; and
- the route to Crouzeix remains visible without premature jargon.

Compilation proves neither readability nor source correspondence, so those
review results remain separate evidence.

## 11. Implementation strategy

Implementation is incremental by part, not by artifact type. Each part lands
as a vertically complete slice containing:

- canonical chapters;
- source and claim updates;
- coverage rows;
- Lean modules and exercise solutions;
- focused tests;
- corpus registration; and
- regenerated derived artifacts when the part is published.

The dependency order is:

1. shared textbook infrastructure, schemas, index, glossary, and Chapter 1;
2. Part I;
3. Part II;
4. Part III;
5. Part IV;
6. Part V;
7. Part VI;
8. whole-book pedagogical, formal, source, and release audit.

No later part is presented as complete while an earlier mathematical
dependency remains open. Draft chapters may exist ahead of the formalization,
but the index and coverage ledger must label them `draft` and expose their open
rows.

Feature work occurs in isolated worktrees. Parallel work is allowed only when
chapter and Lean write sets do not overlap and each slice has an independently
verifiable boundary. The main integrator owns identifiers, cross-part links,
coverage reconciliation, generated artifacts, and final gates.

## 12. Completion criteria

The textbook objective is complete only when current repository evidence
establishes all of the following:

1. all 35 canonical chapters exist and pass the chapter-template and link
   contracts;
2. the book is self-contained for the stated reader baseline;
3. every chapter contains conceptual development, complete proofs, worked
   examples, an ML bridge, Lean guidance, exercises, and synthesis;
4. every mathematical definition, theorem, worked derivation, and proof
   exercise maps to compiled Lean or an exact executable check as appropriate;
5. every proof exercise has a checked reference solution;
6. no mathematical coverage row remains `open-formalization`;
7. all public Lean modules compile without prohibited proof holes;
8. capstone axiom audits and endpoint statement digests pass;
9. both Jin and Lorist--Schwenninger routes are reconstructed with their
   hypotheses, order-sensitive steps, reductions, and limitations intact;
10. source, copyright, claim, and publication-status boundaries are explicit
    and validated;
11. corpus and Atlas artifacts are regenerated together and pass their tests;
12. the complete theorem dependency graph reaches both constant-two endpoints;
13. independent content review finds no definitions used before introduction,
    unproved mathematical assertions, or misleading ML generalizations; and
14. `mise run verify` passes on the exact final integration commit.

Partial chapter, prose-only, or narrow-test completion cannot support a claim
that the full textbook is finished.

## 13. Non-goals

- Reproducing Lax, Spivak, Bishop, or any other source textbook.
- Providing comprehensive coverage of all linear algebra, differential
  geometry, complex analysis, or operator theory.
- Treating ML analogies as empirical validation.
- Claiming that Lean verifies the intended informal mathematics without a
  source-model audit.
- Claiming peer review, publication, or the completely bounded Crouzeix
  conclusion.
- Adding a repository-wide license or an external local-checkout dependency.
- Hydrating or updating common Lean dependencies during proof iteration.

## 14. Design rationale

The proof-backchained alternative was rejected because it reaches Crouzeix
quickly at the cost of a fragmented treatment of calculus, geometry, and
operator structure. The encyclopedic alternative was rejected because it
dilutes the target, grows beyond a reviewable formalization surface, and makes
measured pacing harder.

The selected layered design is broad enough to make the proof mechanisms feel
earned, narrow enough to preserve a visible endpoint, and structured so prose,
formal proof, exercises, source evidence, and presentation can advance
together.
