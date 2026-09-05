# Crouzeix textbook deepening design

**Status:** Approved

**Date:** 2026-08-23

**Branch:** `codex/crouzeix-textbook`

## Summary

The existing Crouzeix textbook is a verified proof-route atlas. It has a clear
35-chapter route, 210 indexed exercises, 210 coverage rows, and compiled Jin
and Lorist--Schwenninger constant-two endpoints. It is not yet the intended
self-contained proof textbook for frontier machine-learning research
scientists whose research maturity is high but whose recent proof-based pure
mathematics may be uneven.

This design deepens the book through vertical theorem slices. Every slice lands
the mathematical statement, reconstructible proof, motivation, historical
context, ML analogy, exercises, Lean correspondence, dependency records, and
publication artifacts together. The verification model changes at the same
time so the repository checks correspondence rather than merely checking that
prose and Lean declarations exist in parallel.

The work is delivered as independently verifiable subprojects. Publication
integrity comes first, Chapter 33 establishes the reference slice, and the
remaining chapters follow in proof-dependency order.

## Baseline findings

The design responds to the following observed properties of the current
branch:

- Chapters 30--34 name load-bearing completion, cancellation, realization, and
  recurrence machinery without displaying enough equations to reconstruct
  either constant-two proof independently.
- Of the 210 coverage declarations, 173 are aliases to maintained declarations
  rather than textbook-local proofs.
- Of the 210 exercise rows, 204 reuse the chapter checkpoint declaration as the
  purported solution. Only Chapter 1 has six distinct local exercise solution
  declarations.
- The dependency data has 210 rows and 209 edges. It is almost a single chain
  and incorrectly makes the Lorist--Schwenninger route depend on the Jin route.
- The coverage contract accepts prose paths, anchors, Lean declarations, and
  verification targets as strings without resolving all of them against their
  claimed sources.
- `PublicTheorems.lean` repeats its declaration roster as both inert strings
  and `#check` commands.
- Python and Rust independently implement much of the textbook contract
  validation.
- Thirty-eight of the forty-four textbook documents have different registered
  corpus IDs and frontmatter IDs.
- Two textbook claim entries use `DIRECT OBSERVATION`, which conflicts with the
  evidence vocabulary established by ADR-0001.
- Parts I--V contain repeated generic prose and mostly isolated ML analogies.

The underlying Jin and Lorist--Schwenninger proof providers are substantive.
The Harp-derived finite-horizon provider is substantive as well. Their
terminal theorems are unconditional beyond their explicit typeclass and
analytic hypotheses. No project-specific axiom declaration was found. The
problem is textbook depth and correspondence, not terminal proof soundness.

## Audience

The primary audience is frontier ML research scientists with:

- undergraduate mathematical education;
- PhD-level or equivalent ML research experience;
- strong computational and experimental judgment;
- familiarity with representations, Jacobians, optimization, conditioning,
  and numerical work;
- potentially uneven recent experience with proof-based linear algebra,
  analysis, complex variables, and operator theory.

The book may go as deeply into mathematics as the proof requires. It must not
replace a proof with an ML analogy, assume that a numerical experiment proves
an exact statement, or treat research maturity as recent recall of pure
mathematics.

## Goals

1. Make every theorem-level claim independently reconstructible from the book.
2. Give every theorem-level claim exact, mechanically checked Lean
   correspondence.
3. Give every formal exercise a distinct checked solution.
4. Separate pedagogical prerequisites from Lean kernel dependencies.
5. Make motivation, historical context, and ML analogies useful and visibly
   distinct from formal claims.
6. Replace duplicated and stringly typed verification with one deep publication
   module.
7. Preserve stable CFT IDs, public Lean declaration names, source ceilings, and
   provider isolation.
8. Keep the book text-only, portable in Obsidian, and correctly rendered in the
   offline Atlas.

## Non-goals

- Claiming a new mathematical endpoint beyond the maintained Jin and
  Lorist--Schwenninger results.
- Claiming independent peer review, historical priority, or publication status
  beyond the recorded sources.
- Formalizing a broad theorem merely for atmosphere. A standard result is
  either linked to an exact Mathlib proof, proved locally at the required
  scope, or confined to clearly labeled context.
- Claiming that a mathematical operator theorem applies directly to finite
  precision training code without a separate error analysis.
- Adding runtime, build, source, or documentation dependencies on another local
  checkout.
- Adding required diagrams, images, or other non-textual teaching assets.

## Guiding method: vertical theorem slices

A theorem slice is the smallest landable teaching and verification increment.
It includes:

1. a stable theorem ID and exact prose anchor;
2. the mathematical statement and complete proof;
3. hypothesis explanations and a boundary example;
4. labeled motivation, history, and ML transfer material;
5. exercises, hints, and complete solutions;
6. the Lean declaration, proof mode, type, code locator, fingerprint, and
   assumptions;
7. pedagogical and kernel dependency records;
8. regenerated ledgers, corpus data, and Atlas output;
9. focused and aggregate verification evidence.

A chapter is upgraded only when all its theorem slices pass. The whole book is
not correspondence-complete until every chapter is upgraded.

## Architecture

### Canonical ownership

- `knowledge/crouzeix_textbook/` remains the authority for textbook prose.
- `content/crouzeix_textbook/` remains the authority for structured theorem and
  exercise contracts.
- `formalization/lean/CrouzeixTextbook/` remains the textbook Lean namespace.
- Generated reader ledgers, Lean correspondence sources, corpus JSON, and Atlas
  output remain derived.

### Deep publication module

One Rust publication module owns the textbook contract semantics. Its interface
has read-only `check` behavior and atomic `write` behavior. Focused tooling may
expose preparation and receipt-verification phases, but callers do not
reimplement validation rules.

The module owns:

- contract parsing and exact schemas;
- frontmatter discovery and canonical identity;
- theorem and exercise reference resolution;
- pedagogical graph validation;
- source and evidence-class validation;
- generation of Lean correspondence checks;
- validation of Lean correspondence receipts;
- generation of reader ledgers and compatibility route metadata;
- all-or-nothing publication of sibling outputs.

The existing Python renderer becomes a thin adapter during migration and is
deleted when no caller requires it. Rust integration tests exercise the real
publication interface rather than duplicate its schema implementation.

### Status model

Publication, proof exposition, formal correspondence, and external review are
independent:

- `publication_status`: `draft` or `active`;
- `prose_proof_status`: `summary`, `reconstructible`, or `not-applicable`;
- `lean_correspondence_status`: `unmapped`, `checkpoint`, `exact`, or
  `not-applicable`;
- `review_status`: a source-bound value that does not imply peer review.

The current chapters remain active but initially become correspondence-
incomplete. A chapter reaches exact correspondence only when all theorem and
formal exercise rows pass. Whole-book `complete` correspondence returns only
after all 35 chapters pass.

## Canonical data model

### Theorem contract

`coverage.json` migrates to a version-two exact theorem schema. Each row owns:

- `item_id`;
- chapter number and mathematical kind;
- prose path and exact anchor;
- source IDs;
- pedagogical prerequisites;
- formal mode;
- Lean declaration data when applicable;
- proof, correspondence, and review statuses.

The Lean declaration object contains:

- public declaration name;
- formal mode;
- underlying declaration for a reexport;
- source path;
- compiler-reported source position;
- verification target;
- normalized type fingerprint;
- explicit assumption and axiom receipt.

Allowed formal modes are:

- `proved-here`: a substantive local theorem proof;
- `reexported-proof`: an exact stable name for a substantive maintained proof;
- `definition`: a mathematical definition rather than a theorem proof;
- `checkpoint`: a declaration used for orientation or supporting machinery but
  not claimed as the exact proof of the displayed result;
- `informal`: motivation, history, analogy, or other non-formal material.

A `proved-here` declaration may use prior lemmas but may not be a reducible
alias to one existing declaration. A `reexported-proof` row must link to its
underlying proof and expose its exact type. A checkpoint cannot satisfy a
theorem-proof requirement.

### Exercise contract

Each exercise row owns:

- stable exercise ID and chapter;
- exact prose anchor;
- kind and difficulty;
- theorem skills or CFT items exercised;
- optional Lean starter text locator;
- distinct solution declaration, source locator, normalized type fingerprint,
  and verification target for formal exercises.

A formal exercise solution may use the theorem being studied, but it cannot be
represented by merely naming the same checkpoint declaration. Every formal
exercise has a distinct compiled solution declaration.

### Dependency contracts

The duplicated `theorem_dependencies.json` is removed.

Pedagogical prerequisites are authored in theorem rows. They answer: “What
must the reader understand before this proof?” The reader-facing route and
chapter navigation use this graph.

Kernel dependencies are derived from compiled Lean declarations. They answer:
“Which named declarations occur in the checked proof body?” A Lean exporter
walks declaration types and bodies, records direct constants in maintained
namespaces, and emits a machine-readable receipt. The publication module
validates this receipt and produces a generated kernel graph.

The common trunk and the Jin, Lorist--Schwenninger, and Harp provider branches
must be visible in both graphs at their appropriate abstraction levels. The
compiler-derived graph records provider imports. The prose does not infer
independence merely because two arguments look different.

## Chapter interface

Every chapter uses this textual structure.

### Motivation

A concrete mathematical problem explains why the chapter exists. It includes a
short prerequisite diagnostic and states which later proof step uses the
result. Motivation is labeled and does not claim formal authority.

### Definitions and notation

Every new object receives:

- a precise definition;
- a small example and non-example;
- its relationship to earlier notation;
- its Lean representation when formalized.

### Theorem cards

Every theorem-level claim has a stable CFT heading and:

- purpose;
- exact hypotheses and quantified statement;
- a hypothesis ledger explaining what each assumption buys;
- a proof roadmap;
- a complete proof showing every nontrivial transition;
- a counterexample or boundary case for a tempting stronger claim;
- pedagogical prerequisites;
- a Lean correspondence block.

The Lean block contains:

- formal mode;
- public declaration name;
- exact readable type or hypothesis-to-parameter map;
- clickable source link with a compiler-validated line locator;
- link to the substantive underlying proof for a reexport;
- normalized type fingerprint;
- direct kernel dependencies;
- classical, noncomputable, finite-dimensional, completeness, analyticity, and
  other material assumptions.

### Worked proof workshops

Every load-bearing mechanism has a symbolic derivation and a low-dimensional
worked instance. A mathematically mature reader must be able to reproduce the
argument without consulting the source paper. Routine algebra may be assigned
as an exercise only when a complete solution is included.

### Exercises

Each chapter progresses through:

1. retrieval;
2. calculation;
3. proof reconstruction;
4. boundary or counterexample analysis;
5. Lean formalization;
6. ML transfer.

The prose provides a copyable Lean starter, graduated hints, and a complete
solution. Compiled solutions live in chapter-scoped exercise namespaces.

## Labeled context

### Historical context

Every material historical statement records:

- evidence class from ADR-0001;
- exact source identity;
- stable locator;
- publication and review status;
- reproduction status where relevant;
- caveat against unsupported priority or acceptance claims.

Inspectable artifact facts use `EVIDENCE`. Author-reported results and
interpretations use `SOURCE CLAIM`. Documentation reconciliation uses
`INFERENCE`. Reproduction remains a separate field.

### ML analogy

Every ML analogy uses four explicit fields:

1. mathematical object and proposed ML counterpart;
2. what transfers exactly;
3. what does not transfer without additional assumptions;
4. a concrete diagnostic, calculation, or experiment.

The analogies form a cumulative through-line rather than unrelated metaphors.
They never change a theorem statement or inherit its evidence class.

## Running nonnormal example

The book uses

\[
A_{\lambda,\alpha}
=
\begin{bmatrix}
\lambda & \alpha\\
0 & \lambda
\end{bmatrix}
\]

as its running example. This family supports hand calculations for:

- object versus representation;
- eigenvalues and defective eigenspaces;
- singular values and operator norm;
- transient amplification and resolvents;
- holomorphic and polynomial functional calculus;
- numerical range geometry;
- Crouzeix normalization and sharpness.

The normalized nilpotent instance supplies the constant-two extremizer. ML
sections may interpret the family as a local Jacobian or directed feature
coupling, subject to the approved non-transfer caveats.

## Identity and compatibility

Frontmatter `id` becomes the canonical document identity. CFT item IDs,
exercise IDs, and public Lean names remain unchanged.

The publisher discovers the textbook packet from an allowlisted root, validates
regular Markdown files and frontmatter, and emits compatibility mappings for
current corpus concept IDs. Atlas and Obsidian links continue to resolve during
the migration. Once compatibility coverage is verified, hard-coded per-file
corpus registration is removed.

The publisher rejects a canonical ID that collides with another document,
route, concept, theorem, or compatibility alias.

## Data flow

1. Load versioned theorem and exercise contracts.
2. Discover and validate chapter frontmatter and navigation.
3. Validate identities, prose anchors, source IDs, evidence classes, and the
   pedagogical DAG.
4. Generate a Lean correspondence source containing every applicable theorem
   and exercise declaration.
5. Compile the correspondence source with the pinned Lean project.
6. Export normalized declaration types, compiler source positions, direct
   kernel dependencies, and axioms to a bounded temporary receipt.
7. Validate the receipt against the contracts.
8. Prepare reader ledgers, compatibility routes, corpus registration data, and
   other generated outputs.
9. Replace sibling outputs only after all preparation and validation succeeds.
10. Regenerate corpus and Atlas artifacts together.

The Lean receipt is bounded, schema-validated, and untracked. Canonical
contracts store stable fingerprints and source paths, not machine-specific
absolute paths.

## Failure model

Validation fails closed with a typed diagnostic containing:

- theorem, exercise, chapter, or document identity;
- offending field;
- expected and observed values;
- canonical path and source position when available;
- stable error code.

The publisher rejects:

- duplicate or mismatched identities;
- missing prose anchors or Lean declarations;
- stale source positions or type fingerprints;
- unresolved or invalid source IDs and evidence classes;
- cycles or unknown nodes in either dependency graph;
- cross-provider dependencies;
- proof modes inconsistent with declaration bodies;
- formal exercises without distinct checked solutions;
- new or unrecorded axioms;
- symlinked, hardlinked, escaped, or non-regular inputs and outputs;
- stale derived files;
- partial sibling publication.

A failed write preserves the previous valid publication set. The implementation
prepares every output in held temporary storage, validates the complete set,
and then publishes with the repository's existing symlink-safe and atomic-file
discipline.

## Verification strategy

### Contract tests

Mutation tests cover every field, missing and unknown fields, duplicate IDs,
bad source references, invalid state transitions, dependency cycles, identity
collisions, and incompatible proof modes.

### Prose correspondence tests

Every theorem row resolves to exactly one prose anchor. The theorem card must
contain its statement, proof, hypothesis ledger, Lean block, and labeled
context. Code links must resolve to the compiler-reported declaration.

Tests enforce structure and correspondence, not subjective word counts. An
editorial review remains necessary for mathematical readability.

### Lean correspondence tests

The generated correspondence source compiles in the `CrouzeixTextbook` target.
The exporter receipt must match declaration names, normalized types, source
positions, proof modes, dependencies, and axioms. Forbidden proof placeholders
remain rejected.

### Route tests

The pedagogical graph has one common trunk and three terminal branches. Jin and
Lorist--Schwenninger are the two source-derived routes. Harp is a derived
finite-horizon route. The three focused targets compile separately. The
aggregate textbook target may import all three only in the comparison chapter.

### Publication tests

Corpus and Atlas consume canonical frontmatter identities, validate all
compatibility routes, render every formula and code link, and reject stale
generated artifacts. Claim-ledger validation applies ADR-0001 uniformly to the
textbook packet.

### Release gate

Focused tests run during each slice. `mise run lean-crouzeix-textbook` and the
appropriate provider-isolated target run before a slice freezes. `mise run
verify` runs on every frozen delivery wave and before any local landing.

Lean uses the approved warm cache. The work never runs `lake update`, cache
hydration, or dependency maintenance during proof iteration.

## Delivery sequence

### Wave 0: publication integrity

- introduce version-two schemas and status fields;
- implement the deep publication module;
- correct ADR-0001 claim classes;
- adopt frontmatter identity with compatibility mappings;
- resolve every current prose and Lean reference;
- generate the public Lean manifest and correspondence checks;
- remove duplicated dependency data and validators after replacement coverage;
- publish truthful incomplete correspondence statuses.

### Wave 1: Chapter 33 reference slice

Write the exact:

- dilation data;
- recurrence scalar;
- adjacent-power identity;
- completed-square lower bound;
- telescoping inverse-power argument;
- source Equations (3) and (4);
- scalar contradiction at two;
- exercise and Lean correspondence set.

This chapter defines the quality bar for every later slice.

### Wave 2: Jin route, Chapters 30--32

Display and prove the completion predicate, sampled kernel matrices, correction
vectors, expanded quadratic forms, cancellation, Gramian congruences,
positive-tail removal, norm extraction, and both limit passages.

### Wave 3: Lorist--Schwenninger realization and three-route comparison, Chapters 34--35

Construct the boundary dilation, prove moment compression and multiplier
bounds, connect the realization to Chapter 33, and compare Jin,
Lorist--Schwenninger, and Harp theorem by theorem. Keep the two source-derived
routes distinct from Harp's derived finite-horizon route.

### Wave 4: common machinery, Chapters 25--29

Deepen outer convex domains, Cauchy and double-layer maps, the
`1 + sqrt 2` obstruction, complete power families, normalization,
approximation, and sharpness.

### Wave 5: analysis and operator theory, Chapters 13--24

Deepen convergence, complex analysis, functional calculus, positivity,
numerical ranges, spectral sets, dilation, and Gramians to the exact level
required by the terminal routes.

### Wave 6: linear structure and geometry, Chapters 1--12

Use Chapter 1 as the exposition standard. Replace repeated prose with
topic-specific motivation, full proofs, counterexamples, distinct formal
exercises, and the running example.

## Migration rules

1. Version-two contracts are introduced before version-one readers are
   removed.
2. Existing rows are converted mechanically, then marked with truthful
   correspondence status.
3. Compatibility mappings are generated and verified before corpus identities
   change.
4. Generated files are refreshed in the same commit as their canonical input.
5. Old validators and manifests are removed only after their replacement tests
   pass the deletion test: removing the old implementation must not reduce
   checked behavior.
6. Each delivery wave uses focused commits grouped by contract, formalization,
   prose, and generated publication concern.
7. Unrelated primary-checkout changes and other worktrees remain untouched.

## Whole-book completion contract

The book reaches exact whole-book completion only when:

- all 35 chapters are active and correspondence-exact;
- every theorem-level claim has an independently reconstructible proof;
- every theorem card has exact Lean correspondence or an explicit
  not-applicable classification;
- every formal exercise has a distinct compiled solution;
- pedagogical and kernel graphs are accurate and acyclic;
- the three terminal provider branches and their provenance are represented
  truthfully by the pedagogical and compiler-derived graphs;
- every historical claim satisfies ADR-0001;
- every ML analogy uses the four-field transfer interface;
- every source link, code locator, fingerprint, compatibility route, and
  generated ledger is current;
- both focused terminal targets, the textbook target, corpus, Atlas, and the
  full repository gate pass;
- an editorial review confirms suitability for the stated frontier ML research
  audience.

Compilation alone never implies prose readability, perfect model
correspondence, numerical implementation stability, historical priority, or
independent peer review. Those boundaries remain explicit in the book status
surface.

## Risks and mitigations

### Scope expansion

Deepening 35 chapters is a long-running program. Vertical slices and delivery
waves prevent an all-or-nothing rewrite. No wave begins by changing every
chapter.

### Formal interface distortion

Mathlib or existing Harp declarations could dictate awkward pedagogy. The book
states the natural mathematical theorem first, then proves a local exact
formulation or uses a clearly labeled reexport. Checkpoints do not masquerade
as proofs.

### Locator drift

Line numbers move as Lean files evolve. Compiler-reported source positions and
type fingerprints make drift detectable. Canonical paths remain portable and
never contain machine-specific roots.

### Generated-state complexity

Multiple generated artifacts can drift or partially update. The publication
module prepares and validates the complete sibling set before replacement, and
the release gate rejects stale outputs.

### Pedagogical uniformity

A rigid template can recreate boilerplate. The chapter interface fixes the
questions a chapter must answer, not its prose. Editorial review rejects
generic motivation or analogies that do not illuminate the chapter's actual
mathematics.

## Approved decisions

- Use vertical theorem slices.
- Preserve CFT IDs and public Lean names.
- Make frontmatter IDs canonical and preserve old routes through compatibility
  mappings.
- Require exact prose--Lean correspondence for every theorem-level claim.
- Include clearly labeled motivation, historical context, and ML analogies.
- Target frontier ML research scientists with mature research judgment and
  uneven proof recall.
- Show every nontrivial proof step; include complete solutions for assigned
  routine algebra.
- Permit exact reexports only with explicit proof provenance.
- Require distinct local solutions for formal exercises.
- Maintain separate pedagogical and kernel dependency graphs.
- Bind historical context to ADR-0001 evidence and review status.
- Use the four-field ML analogy interface.
- Keep the entire textbook text-only.
