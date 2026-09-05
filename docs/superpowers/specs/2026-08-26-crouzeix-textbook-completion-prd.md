# Crouzeix textbook completion PRD

**Status:** Approved

**Date:** 2026-08-26

**Branch:** `codex/crouzeix-textbook`

**Audience:** ML research scientists with undergraduate mathematics and
PhD-level research experience

## Purpose

Finish the 35-chapter Crouzeix textbook as a self-contained proof education,
starting with structural linear algebra and ending with reconstructible
constant-two proofs. The book keeps its reader-facing order. Authoring follows
proof risk: finish the load-bearing terminal arguments first, then deepen their
prerequisites.

This PRD is the execution delta for the approved designs in
`2026-08-23-crouzeix-textbook-design.md` and
`2026-08-23-crouzeix-textbook-deepening-design.md`. It records the current
verified state, the unfinished work, the three-route correction, and the exit
tests for the remaining program.

## Current state

The branch has a complete 35-chapter route, 210 stable CFT rows, 210 exercises,
version-two theorem and exercise contracts, compiler-produced Lean receipts,
and working publication gates.

The terminal mathematics is stronger than the textbook exposition:

- Jin's maintained terminal declaration compiles from the Harp-owned
  holomorphic route:
  [`CrouzeixConjecture.crouzeixConjecture`](../../../formalization/lean/Crouzeix/Jin/Terminal.lean#L41).
- The Lorist--Schwenninger terminal declaration compiles from its maintained
  dilation route:
  [`CrouzeixConjecture.loristSchwenningerMainTheorem`](../../../formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean#L24).
- Harp's finite-horizon terminal declaration compiles from its maintained
  atomic-dilation route:
  [`CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem`](../../../formalization/lean/Crouzeix/Harp/MainTheorem.lean#L27).
- The route scans found no `sorry`, `admit`, or project-specific axiom
  declaration in these maintained proof directories.
- The route and full-library gates have passed under the pinned Lean and
  Mathlib toolchain.

Those facts establish local kernel verification. They do not establish that
the current prose teaches the proofs. At this branch point the publication
contracts report:

- 2 reconstructible proof rows;
- 206 summary proof rows;
- 2 rows where a prose proof does not apply;
- 2 exact Lean correspondence rows;
- 68 checkpoint rows;
- 140 unmapped rows;
- 8 exercises with distinct checked Lean solutions;
- 202 exercises without checked Lean solutions.

Chapter 33 items CFT-33-001 and CFT-33-002 define the present quality sample.
The next work starts at CFT-33-003.

## Product outcome

A reader who begins with undergraduate mathematics and research-level ML
experience can read the book in order and do the following:

1. Reason about linear maps without confusing them with chosen matrices.
2. Use duality, adjoints, positive operators, singular values, functional
   calculus, numerical ranges, spectral sets, compressions, and dilations.
3. Reconstruct the Jin and Lorist--Schwenninger proofs from displayed
   equations and cited prerequisites.
4. Understand Harp's finite-horizon proof as a Harp-derived construction,
   with its relationship to the source-derived routes stated exactly.
5. Follow every formal theorem to a compiling Lean declaration and its
   substantive provider.
6. Explain which parts Lean verifies and which historical, pedagogical,
   numerical, and source-status claims remain outside the kernel theorem.

## Writing contract

The book remains text-only. It may use equations, tables, code blocks, and
textual dependency lists. It does not require diagrams or external interactive
material.

The mathematical voice follows the structural habits associated with Peter
Lax's linear algebra writing:

- introduce the object before choosing coordinates;
- state the problem that forces a definition;
- expose the governing proof idea before calculation;
- perform the calculation without skipping its load-bearing steps;
- return to the invariant meaning after the calculation;
- use small examples and counterexamples to test each boundary.

Every analogy to ML has four labeled parts:

1. the mathematical object and proposed ML counterpart;
2. the part of the correspondence that is exact;
3. the part that needs extra assumptions or does not transfer;
4. a calculation or diagnostic the reader can perform.

An analogy cannot replace a proof or inherit the proof's evidence status.

## Theorem-slice contract

Each CFT item is finished as one vertical theorem slice. A complete slice has:

1. a stable item ID and unique prose anchor;
2. motivation tied to a later proof use;
3. definitions and a notation map;
4. an exact statement with all hypotheses and quantifiers;
5. a hypothesis ledger that says what each assumption buys;
6. a proof roadmap;
7. a line-by-line proof of every nontrivial transition;
8. a worked low-dimensional or scalar instance;
9. a boundary case, failed strengthening, or counterexample;
10. labeled historical context with source and review status;
11. the four-field ML analogy;
12. pedagogical prerequisites;
13. a Lean correspondence block;
14. exercises, hints, and complete solutions;
15. focused prose, contract, Lean, and publication tests.

The Lean correspondence block records:

- formal mode;
- public declaration name;
- substantive provider when the public name is a reexport;
- readable type or a hypothesis-to-parameter map;
- repository-root code link to the compiler-reported declaration;
- source position and normalized type fingerprint;
- direct maintained kernel dependencies;
- recorded axioms and material assumptions;
- verification target and most recent receipt identity.

Allowed formal modes remain `proved-here`, `reexported-proof`, `definition`,
`checkpoint`, and `informal`. Only the first two can establish exact theorem
correspondence. A checkpoint may orient the reader but cannot count as the
proof of a displayed theorem.

## Exercise contract

Each chapter keeps six exercises, normally covering retrieval, calculation,
proof reconstruction, a boundary case, Lean work, and ML transfer. The precise
mix may change when the mathematics needs a different progression.

Every formal solution must:

- compile as a theorem;
- have a stable chapter-scoped declaration;
- answer the prose prompt;
- have a statement fingerprint different from every public theorem card in
  the chapter;
- include a source link and compiler receipt;
- avoid becoming a renamed checkpoint or direct alias.

Prose-only historical or interpretive exercises may use a truthful
not-applicable Lean status. The book must still provide a complete written
solution. The target is meaningful support for all 210 exercises, not a fake
Lean theorem for a question that has no formal proposition.

## Delivery order

The published book remains Chapters 1 through 35. Work proceeds in the order
below because mistakes near the terminal proofs have the largest downstream
cost.

### Wave 1: finish Chapter 33

Complete the Lorist--Schwenninger perturbation workshop.

#### CFT-33-003

Derive the inverse-power weighted telescope and source Equation (3). Display
the finite sum, each shifted index, the terminal term, its limit, and the
geometric-series evaluation. The Lean proof and exercise must expose the same
finite-to-infinite bridge used in the prose.

#### CFT-33-004

Display source Equation (4), combine it with Equation (3), and derive the
checked scalar inequality. Preserve the exact hypothesis boundary: this card
does not assume the displacement scalar is nonnegative unless the matched
Lean theorem does.

#### CFT-33-005

Use nonnegativity at the correct step and prove the contradiction for
`kappa > 2`. State the equality and boundary cases. Explain why a numerical
recurrence experiment can test intuition but cannot prove the infinite family.

#### CFT-33-006

Assemble the operator theorem. Map every abstract field to the provider data,
show where norm attainment enters, and identify the exact theorem that turns
the scalar endpoint into `norm T <= 2`.

Wave 1 exits when all six Chapter 33 cards are reconstructible and exact, all
six exercises have truthful complete solutions, and both the textbook and LS
targets pass.

### Wave 2: Jin, Chapters 30 through 32

Upgrade all 18 theorem slices. The prose must display:

- matrix-valued positive-real and Herglotz objects;
- sampling at the eigenvalues and at the origin;
- the correction or completion object with dimensions stated;
- expanded sample and origin quadratic forms;
- the cancellation identities term by term;
- weighted Gram matrices and every congruence;
- removal of the positive tail;
- extraction of the operator-norm estimate;
- simple-spectrum and outer-domain limit passages;
- polynomial and rational consequences at their exact scope.

The ML material compares exact positive certificates with kernel and feature
Gram matrices. Finite sampled PSD residuals are diagnostics only.

Wave 2 exits when all 18 cards and all 18 exercises pass exact correspondence,
the Jin target compiles in provider isolation, and a mathematical reader can
reproduce the route without the source manuscript.

### Wave 3: LS realization and three-route comparison, Chapters 34 and 35

Chapter 34 constructs and checks the concrete Lorist--Schwenninger data:

- boundary measure space and coordinate multiplier;
- embedding, adjoint compression, and projection;
- first and higher moment compression;
- multiplier power identities;
- multiplier contractivity;
- instantiation of Chapter 33;
- removal of normalization.

Chapter 35 replaces the approved design's obsolete two-route wording with a
three-route comparison:

1. Jin's source-derived positive-completion route.
2. Lorist--Schwenninger's source-faithful dilation route.
3. Harp's derived finite-horizon atomic-dilation route.

The three routes lead to the same finite-matrix constant-two conclusion, but
their provenance is not the same. Harp is not presented as a third historical
source. The book states its reuse of common and LS-inspired mathematics. The
compiled kernel graph, not prose similarity, determines whether a terminal
provider imports another maintained provider namespace.

The six Chapter 35 cards cover:

- the normalized terminal statements and their three provider declarations;
- finite-matrix polynomial consequences;
- rational consequences and pole-free hypotheses;
- finite-dimensional to Hilbert-space transport boundaries;
- two-spectral-set packaging;
- a theorem-level comparison of hypotheses, objects, decisive mechanisms,
  limits, kernel dependencies, source status, and amplification boundaries.

A Chapter 35 card has one primary textbook declaration. It may link additional
provider declarations as supporting evidence. The contract must distinguish
the declaration that proves the displayed card from declarations included for
comparison.

Wave 3 exits when all 12 cards and all 12 exercises are complete, the Jin, LS,
and Harp targets compile separately, and the comparison graph introduces no
false proof dependency.

### Wave 4: common machinery, Chapters 25 through 29

Upgrade all 30 theorem slices covering smooth outer domains, boundary
parametrization, Cauchy and double-layer maps, positive boundary density, the
full power family, the `1 + sqrt 2` obstruction, normalization, approximation,
and sharpness.

Every route hypothesis must point back to a displayed common-trunk theorem.
The pedagogical graph branches only after the common material. The kernel
graph remains compiler-derived.

Wave 4 exits when all 30 cards and 30 exercises are complete and both terminal
source-derived routes still compile in isolation.

### Wave 5: analysis and operator theory, Chapters 13 through 24

Upgrade all 72 theorem slices covering:

- completeness, compactness, and convergence;
- operator series and continuity of inversion;
- complex differentiation and contour integration;
- Cauchy theory and its required consequences;
- polynomial, rational, and holomorphic functional calculus;
- positive-real analytic functions;
- normal and nonnormal operators;
- numerical ranges and their geometry;
- spectral sets and amplification boundaries;
- positive and completely positive maps;
- compression and dilation;
- Gramians, Lyapunov equations, Schur complements, and ordered inequalities.

Use the running matrix

```text
A_(lambda,alpha) = [[lambda, alpha], [0, lambda]]
```

for exact calculations of spectra, singular values, numerical ranges,
resolvents, polynomial evaluations, transient growth, and finite-horizon
Gramians.

Wave 5 exits when Chapters 25 through 35 have no unstated analytic or operator
prerequisite.

### Wave 6: Lax-style foundations, Chapters 1 through 12

Upgrade all 72 theorem slices covering:

- mathematical objects and representations;
- vector spaces, subspaces, bases, and quotients;
- kernels, images, rank-nullity, and invariant subspaces;
- coordinates, duality, and similarity;
- determinants, trace, and the required exterior algebra;
- eigenvalues, minimal polynomials, and generalized eigenspaces;
- inner products, projections, and adjoints;
- positive operators and Gram geometry;
- operator norms, singular values, and polar decomposition;
- multilinear maps and tensors;
- differentiation as linear approximation;
- the scoped differential-forms and Stokes material used later.

The opening chapters must be comfortable for a mathematically mature ML
reader whose proof memory is uneven. They cannot assume functional analysis or
Lean experience. Fast-recall paths may shorten review, but they cannot skip a
later dependency.

Wave 6 exits when a reader can start at Chapter 1 and reach Chapter 13 without
an external prerequisite text.

### Wave 7: whole-book integration

After all mathematical waves:

- normalize notation, naming, and theorem references;
- remove generic filler and duplicated explanations;
- repair chapter transitions;
- propagate the running nonnormal example where it teaches real mathematics;
- add reader routes for foundations, common machinery, Jin, LS, and Harp;
- validate every historical claim against the source registry and ADR-0001;
- resolve every theorem and exercise code link against compiler output;
- regenerate contracts, ledgers, corpus JSON, Atlas HTML, and receipts as one
  valid publication set;
- perform an independent mathematical-reader review;
- perform an independent Lean-correspondence review;
- run the complete repository gate.

## Publication order for each slice

Each slice lands transactionally in this order:

1. canonical prose and source records;
2. theorem and exercise contracts;
3. textbook Lean declarations and exercise proofs;
4. compiled correspondence receipt;
5. generated reader ledgers;
6. corpus and Atlas artifacts;
7. `docs/import-receipt.md`, refreshed last;
8. focused and full verification evidence.

A prose edit alone cannot promote a correspondence status. Only a fresh
compiler receipt can do that. A failed publication attempt preserves the last
valid sibling set.

## Issue decomposition

The implementation issue map uses vertical slices, not one issue for all
prose followed by another issue for all Lean.

An independently grabbable issue contains:

- one proof mechanism or at most one tightly coupled group of CFT cards;
- its chapter prose;
- local Lean declarations and exercises;
- contract rows and dependency updates;
- focused tests;
- generated publication changes only when the slice freezes;
- exact acceptance commands and expected changed paths.

No two concurrently executable issues may edit the same canonical chapter,
contract rows, or generated output set. Generated outputs therefore belong to
one integration issue per wave unless the issues run serially.

Each wave has these issue classes:

1. contract and RED-test freeze;
2. one or more theorem-slice implementation issues;
3. chapter or wave integration;
4. mathematical quality review;
5. Lean and publication verification;
6. repair issue when review finds a real defect.

Issue acceptance criteria state observable behavior and exact commands. They
do not use placeholders such as "improve prose" or "add tests."

## Verification

Focused work uses the narrow chapter tests and the appropriate provider
target. Before a slice or wave freezes, run:

```sh
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
```

Run one or more of these when the slice touches a provider:

```sh
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
scripts/check_lean_library.sh CrouzeixHarp
```

Each frozen wave also runs:

```sh
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
```

Before expensive Lean runs, verify the pinned toolchain, warm cache link,
required dependency artifacts, worktree-local mise trust, and exact target.
Do not run `lake update` or hydrate shared dependencies during proof work.

## Review gates

Two reviews are required for every wave.

The mathematical review checks:

- whether the prose proves the displayed statement;
- whether any algebra, limit, or domain hypothesis is hidden;
- whether examples and exercises test the actual mechanism;
- whether history and ML analogies respect their stated boundaries;
- whether an expert can reconstruct the argument without consulting the source
  manuscript.

The formal review checks:

- statement equality or the documented translation map;
- source link and position;
- formal mode and substantive provider;
- declaration kind and type fingerprint;
- direct kernel dependencies and provider isolation;
- axioms and assumptions;
- distinct exercise statements;
- fresh focused and aggregate receipts.

A review finding blocks promotion of the affected slice. It does not demote a
separate verified provider theorem unless the finding concerns that theorem.

## Whole-book acceptance criteria

The textbook is complete only when:

- all 35 chapters are active;
- every proof-bearing CFT row is reconstructible;
- every displayed formal theorem is exact or has a truthful not-applicable
  classification;
- every theorem card carries usable Lean declaration and provider links;
- every formal exercise has a distinct compiled solution;
- every prose-only exercise has a complete written solution and truthful Lean
  status;
- the pedagogical dependency graph is acyclic and branches truthfully;
- compiler-derived kernel graphs preserve provider boundaries;
- Jin, LS, and Harp proof provenance is stated without conflation;
- historical claims satisfy ADR-0001;
- all ML analogies use the four-field boundary contract;
- all source links, code positions, fingerprints, routes, ledgers, corpus data,
  and Atlas artifacts are current;
- the Jin, LS, Harp, textbook, and full repository gates pass;
- independent mathematical and formal reviews have no unresolved blocking
  findings.

The final status must keep four claims separate:

1. the terminal Lean theorem compiles;
2. the prose reconstructs the theorem;
3. the prose statement corresponds exactly to the Lean statement;
4. external publication or independent peer review has the recorded status.

No one claim implies the other three.

## Non-goals

- Proving a stronger Crouzeix theorem than the maintained terminal statements.
- Treating Harp as a third historical source proof.
- Claiming complete boundedness where only scalar spectral-set control is
  proved.
- Formalizing an ML training system merely to justify an analogy.
- Using numerical evidence as proof of an exact infinite family.
- Adding a dependency on another local checkout.
- Replacing source or review status with a successful local Lean build.
- Publishing or pushing the feature branch without explicit owner direction.

## Main risks

### Proof prose drifts from Lean

The compiler receipt binds names, types, positions, dependencies, and axioms.
The mathematical review separately checks that the displayed derivation proves
that type. Neither check substitutes for the other.

### Exercise formalization becomes ceremonial

The solution statement must differ from chapter checkpoints and answer the
prompt. Reviewers inspect the exercise as a learning task, not only as a
compiling declaration.

### Three-route comparison distorts provenance

Chapter 35 labels Jin and LS as source-derived routes and Harp as a derived
route. It reports kernel dependencies from receipts and never infers them from
similar prose.

### Foundations become encyclopedic

Every foundational theorem must support a later proof, the running example, or
a stated reader outcome. Material that does none of these moves to optional
context or leaves the book.

### Generated files create false parallelism

Canonical slices may be developed independently only when their write sets do
not overlap. One integration issue serializes shared contract and generated
publication changes per wave.

## Decisions fixed by this PRD

- Keep the reader-facing order from linear algebra to terminal proofs.
- Author in proof-risk order from Chapter 33 back to the foundations.
- Use vertical theorem slices as the unit of completion.
- Keep the book text-only.
- Write for frontier ML researchers without reducing the mathematical depth.
- Attach compiler-validated Lean information to every formal theorem.
- Use clearly labeled motivation, history, and ML analogies.
- Compare Jin, LS, and Harp in Chapter 35 with truthful provenance.
- Keep the work on `codex/crouzeix-textbook` until the owner chooses a landing
  action.
