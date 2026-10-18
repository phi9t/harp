# Reduce RSI research to a brief article survey

Status: done
Owner: Codex
Branch: codex/harp-corenius-consolidation
Parent: [consolidation intent](../spec.md)

## What to build

Preserve a brief, source-linked survey of research articles previously read.
Remove RSI article-related material from the consolidated product, including
RSI deep dives, source cards, claim ledgers and graphs, article captures and
RSI-specific supporting code. Preserve Crouzeix and other math/proof work, SICP
and other educational material. Delete the Atlas as subsequently requested; retain the headless machinery required
by those educational and proof consumers. Extract useful operational documentation before
retiring the research prose that currently houses it.

## Acceptance criteria

- The retained survey describes prior reading, rather than implying new source
  verification or independently reproduced research results.
- Previously uninspected bibliography entries are not presented as read.
- The final product has no build, test, UI or documentation dependency on retired
  article material, and no replacement research archive under a new name.
- Runtime, recovery, permission and controller behavior remains protected.
- Unrelated primary-checkout changes remain untouched.
- Research-only validation and frozen corpus-count contracts are retired with
  the behavior they protect; applicable surviving contracts still pass.

## Blocked by

No blocker to writing the retained article survey or mapping article consumers.
The owner explicitly chose to preserve math/proofs (especially Crouzeix), SICP
and other educational material. Remove only RSI-related content. The owner subsequently requested deleting Atlas entirely instead of installing
frontend prerequisites. Keep the educational and proof content and its headless
consumers. No broader content retirement is authorized.

## Verification

Inspected knowledge/rsi, the structured source registry and source cards, product
contract, corpus compiler paths, source verifier, CLI tests and topic packets.
The source registry differentiates inspected articles from uninspected captures
and implementation snapshots; preserve that distinction in the survey.

## Comments

The owner resolved broader scope on 2026-10-18: education and mathematical proof
work remain. Removing Markdown alone would leave broken consumers. Remove
RSI-only contracts while preserving the shared consumers required by retained
material. Do not classify SICP by its current knowledge/rsi path; move it to an
appropriate retained home before removing that RSI directory.

## Retention boundaries

- Keep: Crouzeix proof and textbook, mathematical foundations, autodiff geometry,
  training-dynamics proofs, SICP, other educational materials, and agentic
  engineering practice. Preserve their required evidence and publication paths.
- Remove: the RSI-oriented article library, deconstructions, duplicated source
  cards, claim ledgers, research agendas and article-specific artifacts/tools.
- Condense: previously inspected RSI articles into the brief survey.
- Inspect before removal: knowledge/rsi/sicp; runtime documentation embedded in
  RSI concepts/chapters; mixed evidence bundles such as evidence/weng; shared
  corpus/source/search/publication code; tests that protect both retained and retired
  contracts. Classify by actual responsibility, not directory name alone.

Document verification: run the changed-surface gate for this scope correction.
The isolated candidate now retires the article-only source/evidence and Atlas
while preserving the educational and proof material. Originals remain in Git.

## Candidate implementation and checks

Atlas retirement is b156e903; educational corpus retirement is c45f0367.
The retained survey is knowledge/article_survey.md. All 23 SICP documents are
registered under knowledge/sicp; implementation deep dives moved into agentic
engineering. The corpus is harp-knowledge/v1 with 131 documents and 11 routes.
Research-only registries, article graphs, diagnostics, lesson projections,
article implementation snapshots and Meta-Harness reproduction tools are removed.
All mathematical and educational evidence groups were independently reviewed
and remain intact. The five retained generic source-manifest tests and the
Crouzeix source suite passed together (120 tests). Corpus freshness, search
refresh/status/query, Obsidian assets and 15 writing-style structural tests pass.

The owner authorized the standalone PDF packages/lockfile and pinned Lean cache
restoration. Both are complete. The full candidate gate passed at 838c1ca5,
including 28 proof-goal, 25 PDF renderer, 452 library, 14 CLI and 295 textbook
cases. Actual PDF export and bundle checks pass. Atlas and the RSI article product
are retired in the verified candidate; primary-work reconciliation/local landing
continue under the operational migration ticket.
