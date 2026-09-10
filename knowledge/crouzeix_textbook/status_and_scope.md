---
id: crouzeix-textbook-status-and-scope
title: Crouzeix foundations textbook status and scope
type: status
status: active
created: 2026-08-23
updated: 2026-09-09
tags: [crouzeix-textbook, status, scope, verification]
confidence: high
canonical: status_and_scope.md
---

# Crouzeix foundations textbook status and scope

Back to the [[knowledge/crouzeix_textbook/crouzeix_textbook_index|book index]].

## Current state

- Publication status: all 36 chapters are `active`.
- Proof-exposition status: 125 coverage rows are now `reconstructible`; 89 remain `summary` and two other
  definition rows are `not-applicable`.
- Whole-book exact Lean correspondence: `incomplete`.
- Coverage rows: 216, comprising 63 `proved-here`, 115
  `reexported-proof`, 32 `checkpoint`, and 6 `definition` rows.
- Distinct proofs behind those rows: 208. 8 cards restate
  a theorem another card already indexes; they share its provider. Run
  `mise run textbook-counts` with `--audit` to list them.
- Exact-correspondence rows: 132. The other rows remain `checkpoint` or
  `unmapped` until prose and Lean are reviewed together.
- Indexed exercises: 216.
- Distinct checked exercise solutions: six each in Chapters 1, 2, 3, 4, 5, 6, 7, 8, 9,
  10, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, and 36. The other 84 exercise rows
  have no claimed formal solution yet.
- Active Lean target: `CrouzeixTextbook`.

## Contract snapshot

| Metric | Count |
| --- | ---: |
| theorem rows | 216 |
| summary prose rows | 89 |
| reconstructible prose rows | 125 |
| exact correspondence rows | 132 |
| unmapped correspondence rows | 52 |
| solved exercises | 132 |
| unresolved exercises | 84 |

## What the current route provides

Chapters 1–10 now provide a reviewed core, with 60 exact statement
correspondences and 60 distinct Lean exercise solutions. The Chapter 1
characteristic-polynomial preview and Chapter 4's three scalar-invariant
previews deliberately retain summary proofs and are now labelled forward
references to their owning derivations in Chapters 5 and 6; keeping them as
previews is what keeps the teaching order acyclic, and they stay outside the
completed-proof count. The two definition cards have no theorem-proof
obligation. This acceptance does not complete the foundations wave or the book:
Chapters 11–24 still hold 84 unsolved exercises and 84 rows whose
correspondence is `checkpoint` or `unmapped`.

Parts I--III rebuild structural linear algebra, Euclidean geometry, analysis,
complex differentiability, and functional calculus. Parts IV--V develop
numerical ranges, spectral sets, positivity, dilation, Gramians, convex outer
boundaries, and the complete double-layer power family. Part VI presents the
Jin and Lorist--Schwenninger constant-two routes as independent branches after
their common Chapter 29 trunk.

At the current comparison phase, those chapters are a source-registered teaching
route, not yet a claim that every proof can be reconstructed from the prose.
The maintained Jin route has a bounded compiled terminal receipt. The checked
Lorist--Schwenninger terminal provider is
`CrouzeixConjecture.loristSchwenningerMainTheorem` in
`formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean`; its
fixed-domain kernel uses
`CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two`.
The separately checked finite-horizon terminal is
`CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem` in
`formalization/lean/Crouzeix/Harp/MainTheorem.lean`. That theorem is
Harp-derived and not source-derived: its local compilation is evidence only
for the maintained formal provider and does not assign it to Jin or
Lorist--Schwenninger. These boundaries are recorded by
[[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]].
The pinned theorem attribution is recorded separately by
[[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]].
The maintained [[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|source graph]]
remains the locator and dated historical-status authority; its older node
statuses do not override a fresh aggregate build. Neither record establishes
publication beyond its pinned source version or independent peer review.

## Lorist--Schwenninger verification boundary:

- Checked terminal provider: `CrouzeixConjecture.loristSchwenningerMainTheorem`
- Terminal source: `formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean`
- Fixed-domain provider: `CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two`
- Current verification: [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]]
- Pinned source claim: [[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]]
- Source-graph authority: `locator-and-dated-historical-status-only`
- Publication status: `not-established-by-local-compilation`
- Peer-review status: `not-established-by-local-compilation`

## Harp finite-horizon verification boundary:

- Checked terminal provider: `CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem`
- Terminal source: `formalization/lean/Crouzeix/Harp/MainTheorem.lean`
- Provenance class: `Harp-derived; not source-derived`
- Current verification: `mise run lean-crouzeix-harp` and the aggregate
  `mise run lean-crouzeix-textbook`
- Mathematical scope: the maintained finite-horizon theorem statement named
  by the provider; it is not a substitute attribution for the Jin or
  Lorist--Schwenninger routes.
- Publication status: `not-applicable-to-a-Harp-derived-local-route`
- Peer-review status: `not-established-by-local-compilation`

## Formalization boundary

The [[knowledge/crouzeix_textbook/lean_coverage_ledger|Lean coverage ledger]]
is a maintained reader projection of the structured contracts. The Rust-owned
immutable publication ledgers are generated from those contracts and a fresh
compiled receipt. The migration metadata was validated against this durable
compiler receipt identity:

- Compiler receipt SHA-256:
  `17d2adad0c68de5a8d237abcd23fd0813185e4cda399b16d8b8c6e2584e4a8e7`
- Compiler receipt bytes: `541909`
- Compiler receipt declarations: `459`

Those 459 unique declarations comprise 216 public coverage declarations,
132 distinct exercise solutions, and the 111 additional underlying proof
providers needed by `reexported-proof` rows. The receipt SHA-256 hashes the
exact serialized output accepted by the Rust validator. It is not the
six-ledger publication generation: that separate digest hashes the rendered
publication ledgers and is selected by the runtime `current.json` pointer.

Compiler acceptance establishes the recorded Lean declarations in the pinned
environment. It does not establish that a summary paragraph exactly states the
Lean type. For that reason a declaration can be present in the baseline while
its correspondence status remains `checkpoint` or `unmapped`.

## Source and copyright boundary

All textbook prose and exercises are newly authored. Lax and Bishop are
local-only orientation records; the Spivak repository is structure-parsed but
content-gated; JAX sources support API notation only; recent Crouzeix sources
retain the publication and verification ceilings in the maintained proof
packet. See the [[knowledge/crouzeix_textbook/source_registry|source registry]]
and [[knowledge/crouzeix_textbook/claim_evidence_ledger|claim ledger]].

## Completion gate

Whole-book exact correspondence is restored only after every content wave has
made its theorem proofs reconstructible, supplied distinct checked formal
exercise solutions where claimed, validated the Jin and
Lorist--Schwenninger boundaries, and published fresh ledgers, corpus data, and
Atlas artifacts. The final reviewed candidate must pass `mise run verify`.

Even then, compilation certifies formal derivations only in the pinned
environment. It does not establish independent peer review, historical
priority, numerical implementation stability, or empirical transfer of an ML
analogy.
