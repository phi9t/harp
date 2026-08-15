---
id: darwinx-source-registry
title: DarwinX source registry
type: source-registry
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [darwinx, harness-search, sources, provenance]
confidence: high
---

# DarwinX source registry

This registry fixes source identity and claim ceilings for the
[DarwinX packet](darwinx_index.md). Quantitative results remain author reports
unless an entry states that Harp reproduced them.

## Source classes

| ID | Class | Identity | Can support | Cannot support |
|---|---|---|---|---|
| DARWINX-PAPER | Primary paper | `arXiv:2608.07545v1` | Paper method, reported experiments, appendices, stated limitations | Independent reproduction or unreleased implementation behavior |
| DARWINX-SUPPLIED | User-supplied provisional analysis | Captured 2026-08-14 | What the supplied review argued and which questions it raised | DarwinX method, results, or implementation |
| DARWINX-RELEASE-SEARCH | Dated Harp inspection | 2026-08-14 | Which first-party links and search results Harp inspected | Proof that no private or later release exists |
| HARNESSX-PAPER | Primary comparison paper | `arXiv:2606.14249v1` | HarnessX's reported typed composition, AEGIS loop, isolation, co-evolution, and experiments | Independent reproduction or current implementation behavior |
| HARP-META-HARNESS | Maintained Harp synthesis plus pinned evidence | Repository versioned | Meta-Harness paper, site, source, and local-interface boundaries already audited by Harp | New DarwinX evidence |
| HARP-DGM | Maintained Harp synthesis plus pinned evidence | Repository versioned | DGM paper/source reconciliation and Harp's claim ceiling | New DarwinX evidence |
| HARP-RSI | Maintained Harp framework | Repository versioned | Harp's harness-search, promotion, root-tree accounting, and successor terminology | DarwinX-specific empirical facts |

## DARWINX-PAPER: primary paper

- Class: `primary paper`
- Title: *DarwinX: Evolving Agent Harnesses Through Natural Selection*
- Artifact: [Captured PDF](../../evidence/darwinx/artifacts/darwinx-2608.07545v1.pdf)
- Text sidecar: [Extracted text](../../evidence/darwinx/text/darwinx-2608.07545v1.txt)
- Semantic HTML: [Captured arXiv HTML](../../evidence/darwinx/metadata/darwinx-2608.07545v1-paper.html)
- Metadata: [Captured abstract page](../../evidence/darwinx/metadata/darwinx-2608.07545v1-arxiv.html)
- Stability: `pinned`
- Immutable identity: `arXiv:2608.07545v1`
- Digest record: [DarwinX manifest](../../evidence/darwinx/manifest.tsv)
- License: `CC BY 4.0`
- Semantic locators: numbered sections, tables, figures, equations, and appendices.
- Can prove: Paper wording, algorithm, author-reported experiment design and
  outcomes, disclosed action-validity policy, and stated limitations.
- Cannot prove: Independent reproduction, behavior of a DarwinX optimizer
  implementation, historical Monet source identity, or unpublished run
  accounting.

The text sidecar supplies stable line anchors. The PDF remains authoritative
when extraction damages equations or table layout.

## DARWINX-SUPPLIED: provisional review

- Class: `user-supplied provisional analysis`
- Title: *DarwinX: Evolving Agent Harnesses Through Natural Selection*
- Artifact: [Verbatim supplied review](../../evidence/darwinx/artifacts/supplied_review.md)
- Stability: `pinned local input`
- Immutable identity:
  `sha256:d9ce2907d352c15601fef192f732642c6c97a2ce30af51ea49d96136637eb972`
- Digest record: [DarwinX manifest](../../evidence/darwinx/manifest.tsv)
- Can prove: The exact analysis, proposed interpretation, and experiment agenda
  supplied for this task.
- Cannot prove: Any claim about the DarwinX paper, system, benchmarks, code, or
  authors.

The maintained packet audits this source. It does not cite the supplied review
as corroboration for paper claims.

## DARWINX-RELEASE-SEARCH: dated public-artifact inspection

- Class: `dated Harp inspection`
- Artifact: [Provenance record](../../evidence/darwinx/PROVENANCE.md)
- Stability: `dated observation`
- Observed: `2026-08-14 America/Los_Angeles`
- Scope: External links in the revision-pinned semantic HTML and web searches
  for the exact title with `GitHub` and with `Monet`.
- Can prove: The inspected paper links BrowserCode as its browser action
  runtime;
  the bounded search did not locate an official DarwinX optimizer repository
  or project page.
- Cannot prove: No private, unindexed, later, or differently named
  implementation exists.

## HARNESSX-PAPER: primary comparison paper

- Class: `primary paper`
- Title: *HarnessX: A Composable, Adaptive, and Evolvable Agent Harness Foundry*
- Artifact: [Captured PDF](../../evidence/darwinx/artifacts/harnessx-2606.14249v1.pdf)
- Text sidecar: [Extracted text](../../evidence/darwinx/text/harnessx-2606.14249v1.txt)
- Semantic HTML: [Captured arXiv HTML](../../evidence/darwinx/metadata/harnessx-2606.14249v1-paper.html)
- Metadata: [Captured abstract page](../../evidence/darwinx/metadata/harnessx-2606.14249v1-arxiv.html)
- Stability: `pinned`
- Immutable identity: `arXiv:2606.14249v1`
- Digest record: [DarwinX manifest](../../evidence/darwinx/manifest.tsv)
- License: `CC BY 4.0`
- Can prove: The paper's typed harness model, substitution algebra, AEGIS
  adaptation loop, variant-isolation design, cross-harness GRPO proposal, and
  author-reported experiments.
- Cannot prove: Independent reproduction, present source behavior, or that
  HarnessX and DarwinX were compared under a matched budget.

## HARP-META-HARNESS: maintained comparison

- Class: `maintained Harp synthesis and pinned evidence`
- Artifact: [Meta-Harness deep dive](../meta_harness/meta_harness_deep_dive.md)
- Canonical route: [Meta-Harness system note](../rsi/systems/meta-harness.md)
- Stability: `repository versioned`
- Underlying evidence: arXiv v1 paper, dated first-party site capture, pinned
  public repositories, and one bounded local proposal-interface experiment.
- Can prove: Harp's audited distinction between full-history proposal,
  external evaluation, Pareto retention, implementation behavior, and local
  interface findings.
- Cannot prove: DarwinX's mechanism or an equal-budget DarwinX comparison.

## HARP-DGM: maintained comparison

- Class: `maintained Harp synthesis and pinned evidence`
- Artifact: [DGM knowledge packet](../darwin_godel_machine/darwin_godel_machine_index.md)
- Source registry: [DGM source registry](../darwin_godel_machine/source_registry.md)
- Stability: `repository versioned`
- Underlying evidence: arXiv v3 paper and pinned implementation at
  `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.
- Can prove: Harp's audited distinction between an evolving coding-agent
  repository and the protected archive, evaluator, diagnostic model, and
  benchmark controller.
- Cannot prove: DarwinX's mechanism or an equal-budget DarwinX comparison.

## HARP-RSI: evaluation framework

- Class: `maintained Harp framework`
- Harness search: [Searching for better harnesses](../rsi/chapters/harness-search.md)
- Promotion: [Evaluation, promotion, and containment](../rsi/chapters/evaluation-promotion-containment.md)
- State notation: [System state and notation](../rsi/concepts/system-state-and-notation.md)
- Stability: `repository versioned`
- Can prove: Harp's definitions and design criteria for candidate state,
  protected evaluators, root-tree budget, regression constraints, promotion,
  and successor evidence.
- Cannot prove: DarwinX-specific method or benchmark results.

## Capture receipt

The complete acquisition record is
[`evidence/darwinx/capture_receipt.tsv`](../../evidence/darwinx/capture_receipt.tsv).
The bundle-level scope, license, and negative-search boundaries are in
[`evidence/darwinx/PROVENANCE.md`](../../evidence/darwinx/PROVENANCE.md).

Back to the [DarwinX index](darwinx_index.md).
