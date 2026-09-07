# Foundations 2 through 4 implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development. Review each chapter for specification and quality before the next chapter.

**Goal:** Supply reconstructible proofs and 18 distinct Lean exercise solutions for the approved foundations slice.

**Architecture:** Preserve existing chapter identities, cards and contracts. Add local supporting declarations and exercises, then publish exact correspondence through the existing compiler.

**Tech stack:** Markdown, Lean 4.32.1, Rust, Atlas, PDF.

Specification: ../specs/2026-09-07-harp-foundations-02-04-design.md.
Baseline: 2e5d57da339d83b90bb631e0b1939958bd7369a6.
Shared files are edited only by the integration owner.

## Task 0: Prerequisite disposition

- [x] Inspect Kata 8mq3 and m4qy. Both remain open; m4qy records a source-only increment with publication pending.
- [ ] Resolve actual Chapter 1 acceptance evidence before beginning Chapter 2. Do not close either issue or remove dependencies merely because six exercise declarations exist.
- [ ] If resolution requires expanding the approved scope, report the exact missing acceptance and request direction after finishing independent Harp work.

## Task 1: Chapter 2

Files: knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces.md;
formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean.

- [ ] Add compile clients for six exercise_YY_solution statements in namespace CrouzeixTextbook.Part01.Exercises.Chapter02; observe missing declarations.
- [ ] Prove concrete subspace closure, span of (1,1),(1,-1), concrete span inclusion, diagonal quotient factorization, axes-union failure, and equal-output/null-direction equivalence.
- [ ] Expand all six retained cards and their proofs. Supply span construction, basis existence/uniqueness, basis transport, direct-sum existence versus uniqueness, quotient operations/kernel/factorization.
- [ ] Use T(x,y,z)=(x+z,y+z), whose kernel is span((-1,-1,1)). Explain real/complex specialization of more general Lean providers.
- [ ] Run lake env lean CrouzeixTextbook/Part01/Chapter02.lean; review statements, written solutions and all source links separately for specification and quality.

## Task 2: Chapter 3

Files: knowledge/crouzeix_textbook/part_01_linear_structure/03_linear_maps_and_exact_structure.md;
formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean.

- [ ] Add failing compile clients for Chapter03 exercise_01_solution through exercise_06_solution.
- [ ] Prove the running-map zero fiber, explicit kernel/image bases, quotient-image inverse, injectivity of a chosen restriction, strict composition range, and preservation of powers.
- [ ] Write closure calculations and basis-extension rank-nullity, including independence and spanning of image basis. Prove the first isomorphism separately with its own Lean support.
- [ ] Keep CFT-03-005 a definition; verify invariant restriction evaluation and linearity. Distinguish inclusion from equality.
- [ ] Run lake env lean CrouzeixTextbook/Part01/Chapter03.lean and both independent reviews.

## Task 3: Chapter 4 duality core

Files: knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality.md;
formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean.

- [ ] Add failing compile clients for Chapter04 exercise_01_solution through exercise_06_solution.
- [ ] Prove concrete pullback, (4,-1) pulled back to (4,5), basis-evaluation transpose, annihilator of span((1,1,0)), metric diag(2,1) gradient pairing, and two-map reversed pullback order.
- [ ] Expand core cards 001 through 003. Prove dual-basis/annihilator support and the dimension formula with basis extension.
- [ ] Fix x_new=S x_old throughout, so A_new=S A_old S^-1. Use algebraic dual T^vee and metric adjoint T^dagger distinctly.
- [ ] Keep cards 004 through 006 as exact-statement forward references with summary proofs owned by Chapters 5 and 6. Never recycle their IDs.
- [ ] Run lake env lean CrouzeixTextbook/Part01/Chapter04.lean and both independent reviews.

## Task 4: Contracts and publication

Files: content/crouzeix_textbook/coverage.json and exercises.json;
generated Correspondence.lean and existing ledgers; crates/harp/tests/crouzeix_textbook.rs;
atlas/src/app/ChapterReader.test.tsx and generated corpus/export.

- [ ] Add failing tests for retained identities, exact correspondence, 18 nonalias solution statements, missing assumptions, stale hashes/positions, and acyclic prerequisites.
- [ ] Set truthful provider modes and proof scope; preserve definition not-applicable and three forward-reference summary rows.
- [ ] Recompute expected frozen-baseline counts: exact 90, reconstructible 85, summary 129, not-applicable 2, solved exercises 96 of 216.
- [ ] Generate compiler metadata through the maintained pipeline and run focused tests.
- [ ] Join the program-wide full repository, browser and PDF acceptance. No full Chapter 4 or whole-book completion claim.
- [ ] Commit only after the full gate; preserve existing issue blockers and no push.
