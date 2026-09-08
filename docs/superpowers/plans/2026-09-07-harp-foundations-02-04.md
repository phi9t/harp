# Foundations 2 through 4 implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development. Review each chapter for specification and quality before the next chapter.

**Goal:** Supply reconstructible proofs and 18 distinct Lean exercise solutions for the approved foundations slice.

**Architecture:** Preserve existing chapter identities, cards and contracts. Add local supporting declarations and exercises, then publish exact correspondence through the existing compiler.

**Tech stack:** Markdown, Lean 4.32.1, Rust, Atlas, PDF.

Specification: ../specs/2026-09-07-harp-foundations-02-04-design.md.
Baseline: 2e5d57da339d83b90bb631e0b1939958bd7369a6.
Shared files are edited only by the integration owner.

## Autonomous continuation

The user authorized prerequisite checks and repairs on 2026-09-07 with
"finish up all the work end-to-end autonomously". Continue in the existing
isolated feature worktree. No merge, push, deployment or external outreach.
Accept the foundations contract and Chapter 1 locally before developing
Chapters 2, 3 and the Chapter 4 duality core. Review each slice for specification
and quality. Regenerate compiler metadata, web exports and the three existing
PDF editions, then run the full repository gate and final integration review.

The old Wave 6 issue chain includes the unfinished analysis-wave publication.
Preserve that historical blocker and distinguish local prerequisite acceptance
for this package from closing the broader Wave 6 or whole-book issues. The
contract freezes the 72-row foundations target; it must not claim that all
72 exercise solutions already exist. Chapters 5–24 remain outside this package.

Contract review found two bounded parser defects: child theorem headings were
truncated, and adjacent bold title/statement labels were not recognized. Both
repairs belong to the existing editorial-fixture acceptance task. They do not
add a runtime parser or broaden the writing scope. Freeze this helper after
its positive and negative fixtures pass, then resume chapter delivery.

Publication integration subsequently exposed a separate format mismatch in
the existing Rust Markdown validator: promoting Chapter 1 to reconstructible
caused `generate_correspondence` to reject its narrative cards because only
ten child-heading workshop subsections were recognized. The approved
foundations format uses explicit bold statement/proof/provider labels and
chapter-level motivation, history and ML discussion. This is not a failed
Lean proof or permission to waive proof exposition.

Task 4a is a bounded prerequisite to publication: extend the existing Markdown
owner to recognize the approved narrative format, with parser-backed positive
and mutation fixtures. Preserve the existing workshop checks and reject empty
or missing statements, proofs and formal correspondence; code, quoted text,
and the next card cannot satisfy a missing field. No new public manifest,
duplicate production validator, theorem promotion, or chapter-specific bypass.
The integration owner reviews this repair separately before publication.
Files are `crates/harp/src/crouzeix_textbook/markdown.rs`, a focused integration
test, and its review record. Mathematical chapter delivery may continue while
this independent publication repair is implemented.

## Task 0: Prerequisite disposition

- [x] Inspect Kata 8mq3 and m4qy. Both remain open; m4qy records a source-only increment with publication pending.
- [x] Resolve actual Chapter 1 core acceptance before beginning Chapter 2. The repaired basis proofs and norm witness passed separate specification and quality reviews and narrow compilation. Full publication remains a final integration gate; no legacy issue or dependency was removed.
- [x] Obtain authority for the prerequisite repairs. The user authorized autonomous end-to-end completion; the local contract distinguishes the accepted core from the original wider issue conditions.

## Task 1: Chapter 2

Files: knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces.md;
formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean.

- [x] Add compile clients for six exercise_YY_solution statements in namespace CrouzeixTextbook.Part01.Exercises.Chapter02; observe missing declarations.
- [x] Prove concrete subspace closure, span of (1,1),(1,-1), concrete span inclusion, diagonal quotient factorization, axes-union failure, and equal-output/null-direction equivalence.
- [x] Expand all six retained cards and their proofs. Supply span construction, basis existence/uniqueness, basis transport, direct-sum existence versus uniqueness, quotient operations/kernel/factorization.
- [x] Use T(x,y,z)=(x+z,y+z), whose kernel is span((-1,-1,1)). Explain real/complex specialization of more general Lean providers.
- [x] Run the pinned Lean compiler and narrow Lake build for Chapter02; review statements, written solutions and all source links separately for specification and quality. Both independent reviews passed; see chapter02-review.md.

## Task 2: Chapter 3

Files: knowledge/crouzeix_textbook/part_01_linear_structure/03_linear_maps_and_exact_structure.md;
formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean.

- [x] Add failing compile clients for Chapter03 exercise_01_solution through exercise_06_solution.
- [x] Prove the running-map zero fiber, explicit kernel/image bases, quotient-image inverse, injectivity of a chosen restriction, strict composition range, and preservation of powers.
- [x] Write closure calculations and basis-extension rank-nullity, including independence and spanning of image basis. Prove the first isomorphism separately with its own Lean support.
- [x] Keep CFT-03-005 a definition; verify invariant restriction evaluation and linearity. Distinguish inclusion from equality.
- [x] Run pinned direct Lean on Chapter03 and both independent reviews. The spec review required explicit kernel/range results for powers zero and at least two; the repaired source compiled and both reviewers passed. See chapter03-review.md.

## Task 3: Chapter 4 duality core

Files: knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality.md;
formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean.

- [x] Add failing compile clients for Chapter04 exercise_01_solution through exercise_06_solution.
- [x] Prove concrete pullback, (4,-1) pulled back to (4,5), basis-evaluation transpose, annihilator of span((1,1,0)), metric diag(2,1) gradient pairing, and two-map reversed pullback order.
- [x] Expand core cards 001 through 003. Prove dual-basis/annihilator support and the dimension formula with basis extension.
- [x] Fix x_new=S x_old throughout, so A_new=S A_old S^-1. Use algebraic dual T^vee and metric adjoint T^dagger distinctly.
- [x] Keep cards 004 through 006 as exact-statement forward references with summary proofs owned by Chapters 5 and 6. Never recycle their IDs.
- [x] Run pinned direct Lean on Chapter04 and both independent reviews. All passed; the same quality reviewer accepted the terminology clarification identifying invertible matrices as units in the matrix ring. See chapter04-review.md.

## Task 4: Contracts and publication

Files: content/crouzeix_textbook/coverage.json and exercises.json;
generated Correspondence.lean and existing ledgers; crates/harp/tests/crouzeix_textbook.rs;
atlas/src/app/ChapterReader.test.tsx and generated corpus/export.

- [x] Add failing tests for retained identities, exact correspondence, 18 nonalias solution statements, missing assumptions, stale hashes/positions, and acyclic prerequisites. New compiler-backed and scoped tests passed; existing mutation fixtures passed in the full suite.
- [x] Set truthful provider modes and proof scope; preserve definition not-applicable and three forward-reference summary rows.
- [x] Recompute counts from the integrated contracts and compiler receipt. Actual totals are exact 96, reconstructible 89, summary 125, not-applicable 2, and solved exercises 96 of 216. The original 90/85/129 estimate omitted the authorized Chapter 1 repair. Correspondence has 45 checkpoints and 75 unmapped rows.
- [x] Generate compiler metadata through the maintained pipeline and run focused tests. The official receipt has 419 declarations; publisher and read-only checks passed.
- [x] Join the program-wide full repository, browser and PDF acceptance. All passed; see foundations-review.md. No full Chapter 4 or whole-book completion claim.
- [x] Prepare the local feature-branch commit only after the full gate; preserve existing issue blockers and no push.
