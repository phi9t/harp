# Reduce RSI research to a brief article survey

Status: in-progress
Owner: Codex
Branch: codex/harp-corenius-consolidation
Parent: [consolidation intent](../spec.md)

## What to build

Preserve a brief, source-linked survey of research articles previously read.
Remove RSI article-related material from the consolidated product, including
RSI deep dives, source cards, claim ledgers and graphs, article captures and
RSI-specific supporting code. Preserve Crouzeix and other math/proof work, SICP
and other educational material. Keep the Atlas and shared machinery required
by those retained consumers. Extract useful operational documentation before
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
and other educational material. Remove only RSI-related content. The Atlas must
continue supporting retained content. No broader retirement is authorized.

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
  corpus/source/search/Atlas code; tests that protect both retained and retired
  contracts. Classify by actual responsibility, not directory name alone.

Document verification: run the changed-surface gate for this scope correction.
No source, evidence, proof or educational file has been deleted.
