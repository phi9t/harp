---
id: crouzeix-textbook-claim-evidence-ledger
title: Crouzeix foundations textbook claim-evidence ledger
type: claim-ledger
status: active
created: 2026-08-23
updated: 2026-09-07
tags: [crouzeix-textbook, claims, evidence, provenance]
confidence: high
canonical: claim_evidence_ledger.md
---

# Crouzeix foundations textbook claim-evidence ledger

Back to the [[knowledge/crouzeix_textbook/crouzeix_textbook_index|book index]].

Routine derivations are proved in their chapters and indexed by CFT IDs. This
ledger records material source, verification, and publication claims.

**Contract snapshot.**

| Metric | Count |
| --- | ---: |
| theorem rows | 216 |
| summary prose rows | 101 |
| reconstructible prose rows | 113 |
| exact correspondence rows | 120 |
| unmapped correspondence rows | 61 |
| solved exercises | 120 |
| unresolved exercises | 96 |

## CFT-CL-001: Matrices represent linear maps after bases are chosen {#cft-cl-001}

- Class: `EVIDENCE`
- Statement: A linear transformation is defined independently of coordinates;
  choosing bases represents it by a matrix, and composition becomes matrix
  multiplication.
- Source: [[knowledge/crouzeix_textbook/source_registry#lax-2007|LAX-2007]]
- Locator: Chapter III, “Linear Mappings,” printed pages 19--31, as recorded by
  the canonical Mathematical Foundations source route.
- Scope: finite-dimensional linear algebra.
- Reproduction: Chapter 1 derives the matrix-column and coordinate-action
  formulas independently.
- Confidence: `high`
- Confidence basis: standard source orientation plus compiled local proofs.
- Caveat: arbitrary arrays between arbitrary sets are not thereby linear maps.
- Mode: `paraphrase`
- Source stability: `local-only-pinned-record`

## CFT-CL-002: The truthful contract has a bounded compiled Lean receipt {#cft-cl-002}

- Class: `EVIDENCE`
- Statement: The version-two contract has 216 theorem rows: 60 are
  `proved-here`, 115 are `reexported-proof`, 35 are `checkpoint`, and six are
  `definition`. A fresh 447-row Lean receipt checks the declarations required
  by that contract under Lean 4.32.1. The contract has 120 distinct exercise
  solutions; 96 exercises remain correspondence-incomplete. The theorem
  correspondence axis records 120 exact rows, 35 checkpoints, and 61 unmapped
  rows.
- Source: [[knowledge/crouzeix_textbook/source_registry#mathlib-4-32-1|MATHLIB-4.32.1]]
- Locator: `formalization/lean/CrouzeixTextbook/Correspondence.lean`.
- Scope: exact contract declarations and exercise solutions in the local
  source tree; it does not upgrade prose correspondence.
- Reproduction: run `mise run crouzeix-textbook-publication` with the canonical
  warm cache. The task compiles a fresh external receipt, publishes the
  Rust-owned ledger generation, and then checks it without another write.
- Confidence: `high`
- Confidence basis: local compilation and proof-hole scan.
- Caveat: compilation does not establish prose-to-model correspondence without
  review.
- Mode: `direct-observation`
- Source stability: `revision-bound`

## CFT-CL-003: Constant-two routes retain verification boundaries {#cft-cl-003}

- Class: `EVIDENCE`
- Statement: Chapters 30--35 teach two source-backed proof routes. The
  maintained Jin terminal compiles. The checked LS terminal provider is
  `CrouzeixConjecture.loristSchwenningerMainTheorem`; its fixed-domain kernel
  includes
  `CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two`.
  The checked finite-horizon provider
  `CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem` is Harp-derived and
  not source-derived.
- Source: [[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]]
- Locator: `formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean`,
  `formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean`,
  `formalization/lean/Crouzeix/Harp/MainTheorem.lean`, and
  [[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|Lorist--Schwenninger source graph]].
- Scope: the exact maintained LS terminal and fixed-domain provider, plus the
  separate Harp-derived finite-horizon provider, compiled in this repository.
  The source graph remains the authority for pinned LS source locators, not
  current local build status. It supplies no source attribution for the Harp
  theorem.
- Reproduction: run `mise run lean-crouzeix-ls`, `mise run lean-crouzeix-harp`,
  and `mise run lean-crouzeix-textbook` with the canonical warm cache.
- Confidence: `high`
- Confidence basis: current local kernel checking plus pinned source identity.
- Caveat: local verification establishes neither upstream publication status
  nor independent peer review, does not convert summary prose into an exact
  correspondence claim, and does not turn the Harp-derived theorem into a
  source-derived Jin or Lorist--Schwenninger result.
- Mode: `direct-observation`
- Source stability: `revision-bound`

## CFT-CL-004: Jin endpoint compiles {#cft-cl-004}

- Class: `EVIDENCE`
- Statement: the bounded Jin terminal receipt records successful Lean checking
  and axiom audit for `CrouzeixConjecture.crouzeixConjecture`.
- Source: [[knowledge/crouzeix_textbook/source_registry#jin-v4-audited|JIN-V4-AUDITED]]
- Locator: [[labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-terminal-crouzeix/attempt-001/receipt.json|Jin terminal receipt]].
- Scope: the exact declaration and bounded checks named by that receipt.
- Reproduction: run `mise run lean-crouzeix-textbook` and
  `mise run lean-crouzeix-jin`.
- Caveat: local kernel checking is not independent peer review.
- Mode: `direct-observation`
- Source stability: `revision-bound`

## CFT-CL-005: Lorist--Schwenninger equation-one intermediate proof slice compiles {#cft-cl-005}

- Class: `EVIDENCE`
- Statement: the bounded Lorist--Schwenninger receipt records successful Lean
  checking of the intermediate declaration
  `CrouzeixConjecture.LoristSchwenninger.perturbation_mul_target_power_norm_le`,
  its aggregate build, and its allowed-axiom audit.
- Source: [[knowledge/crouzeix_textbook/source_registry#ls-arxiv-v1|LS-ARXIV-V1]]
- Locator: [[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/proof-slices/ls-equation-one-terminal-bound/attempt-001/receipt.json|Lorist--Schwenninger equation-one intermediate receipt]] and [[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|Lorist--Schwenninger source graph]].
- Scope: only the exact intermediate declaration and bounded checks named by
  that dated receipt. Current provider status is recorded separately by
  [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]].
- Reproduction: run `mise run lean-crouzeix-textbook` and
  `mise run lean-crouzeix-ls`.
- Caveat: this intermediate receipt alone does not establish the maintained
  endpoint or independent peer review; it must not override a fresh aggregate
  build or be used as a publication-status claim.
- Mode: `direct-observation`
- Source stability: `revision-bound`

## CFT-CL-006: Lorist--Schwenninger state the perturbation lemma {#cft-cl-006}

- Class: `SOURCE CLAIM`
- Statement: Lorist and Schwenninger state that a uniformly bounded commuting
  perturbation family satisfying their compressed-power identity forces the
  finite-dimensional target operator to have norm at most two.
- Source: [[knowledge/crouzeix_textbook/source_registry#ls-arxiv-v1|LS-ARXIV-V1]]
- Locator:
  [[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|Pinned source graph]],
  `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json#L1`.
- Scope: Lemma 1 at
  `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99`.
- Reproduction: compare the pinned locator and source digest with the
  maintained Chapter 33 notation and theorem-card statements.
- Confidence: `high`
- Confidence basis: pinned primary-source identity and exact source locator.
- Caveat: the source claim does not establish local compilation, publication
  beyond the pinned arXiv version, independent peer review, or priority.
- Mode: `paraphrase`
- Source stability: `pinned`
