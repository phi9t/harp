---
id: darwinx-knowledge-index
title: DarwinX knowledge index
type: index
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [darwinx, harness-search, population-search, recursive-self-improvement]
confidence: high
---

# DarwinX

DarwinX is a population selection method for harness engineering. It keeps
alternative harness lineages, applies a bounded-regression gate, confirms
promising variants at higher fidelity, and attempts to merge specialists. The
model weights remain fixed in the reported experiments.

## Executive assessment

**[INFERENCE - DX-024](claim_evidence_ledger.md#dx-024-the-reported-studies-support-durable-harness-capability).**
The paper gives strong system-level evidence that harness changes can store
procedural competence. A frozen model gains better task completion,
verification, persistence checking, tool grounding, and acceptance-contract
behavior.

**[MISSING - DX-026](claim_evidence_ledger.md#dx-026-the-paper-does-not-isolate-darwinxs-population-operators).**
The paper does not isolate the archive, parent selector, regression gate,
recombination operator, or inference effort under an equal search budget.

The shortest accurate verdict is:

> Important harness-engineering result. Incomplete attribution to population
> search and recombination.

## Claim ceiling

The primary source is the CC BY 4.0 arXiv v1 paper. Harp captured its PDF,
semantic HTML, metadata, API response, and deterministic text extraction.

Harp did not:

- run DarwinX;
- reproduce any benchmark;
- inspect a DarwinX optimizer implementation;
- verify the proprietary Monet harness;
- audit raw trajectories or archive state; or
- reconstruct the total evolution cost.

**[MISSING - DX-009](claim_evidence_ledger.md#dx-009-a-public-darwinx-optimizer-release-was-not-located).**
The bounded 2026-08-14 public search found no official DarwinX optimizer
repository or project page. The paper's GitHub link points to BrowserCode, the
browser action runtime.

The user's original review is preserved verbatim as
[provisional input](../../evidence/darwinx/artifacts/supplied_review.md). It is
not evidence for paper claims. The
[critical review](03_critical_review.md#correction-matrix-for-the-supplied-review)
records each maintained correction.

## System decomposition

**[EVIDENCE - DX-001](claim_evidence_ledger.md#dx-001-darwinx-edits-the-harness-while-holding-model-weights-fixed).**

```text
guided mutation
  failure trajectory
  teacher trajectory
  self-contrast
        |
        v
candidate harness edit
  prompt / memory / skill / tool / control flow / code
        |
        v
cheap task screen
        |
        v
net gain + regression mass + reasoned verifier
        |
        v
archive retention
        |
        v
high-fidelity confirmation + preservation probe
        |
        v
steering eligibility
        |
        +---- optional specialist recombination
```

Three distinctions prevent overclaiming:

1. Archive membership is not steering authority.
2. `R <= δ` permits nonzero measured regression.
3. A merge must preserve the union of source wins, a stricter rule than the
   ordinary fitness gate.

## Reported results

| Benchmark | Reported result | Audit boundary |
|---|---|---|
| Terminal-Bench 2.1 | 75.5% to 83.2% on frozen GPT-5.5 | Same 89 tasks drive evolution and reporting; evolved harness spends more tokens on newly solved tasks |
| TerminalWorld | 25/41 to 28/41 on frozen Opus 4.8 | Held-out tasks, but one solve separates merged harness from strongest specialist |
| WebArena-Infinity | 43.5% to 93.0% audit-clean on frozen GPT-5.5 | Strong same-model browser harness is 86.1%; action policy broadens; every merge is reverted |
| SWE-bench Verified | TB2.1 harness reaches 421/500 | One-way transfer; no unevolved Monet row |

Open the [evaluation audit](02_evaluation_audit.md) for task counts, sampling,
inference-compute, action-policy, validity, and statistical boundaries.

## What survived audit

The supplied review's main judgment holds:

- DarwinX's important empirical result is durable harness capability.
- The population and recombination attribution is preliminary.
- Verification-before-finalization is the recurring procedural pattern.
- WAI is a strong full-system result and a weak recombination result.
- TerminalWorld is the only positive merge case and remains suggestive.
- The complete search economics are missing.

The audit tightened five points:

- preserve-and-extend is bounded regression, not strict monotonicity;
- weak variants remain in the broad archive, but not all remain inheritance
  eligible;
- TerminalWorld reports two different McNemar comparisons, `p=0.45` and
  `p=1.0`;
- WAI supports the complete system result, not the regression gate by itself;
  and
- "synthetic-to-real" includes intent, reward-source, and partial application
  transfer rather than a completely new environment.

## Reading routes

### Expert route

1. [Critical review](03_critical_review.md)
2. [Evaluation audit](02_evaluation_audit.md)
3. [Successor experiment](05_successor_experiment.md)
4. [Claim ledger](claim_evidence_ledger.md)

Expected reading time: 45 to 75 minutes.

### Mechanism route

1. [Mechanism and selection](01_mechanism_and_selection.md)
2. [Evaluation audit](02_evaluation_audit.md)
3. [Critical review](03_critical_review.md)

Expected reading time: 60 to 90 minutes.

### RSI route

1. [Comparative synthesis](04_comparative_synthesis.md)
2. [Successor experiment](05_successor_experiment.md)
3. [Harp harness-search chapter](../rsi/chapters/harness-search.md)

Expected reading time: 45 to 70 minutes.

## Route by question

| Question | Start here |
|---|---|
| What exactly changes? | [Mechanism and selection](01_mechanism_and_selection.md#system-state) |
| What does preserve-and-extend guarantee? | [Preserve-and-extend fitness](01_mechanism_and_selection.md#preserve-and-extend-fitness) |
| How does parent selection work? | [Parent selection](01_mechanism_and_selection.md#parent-selection) |
| Does recombination matter? | [Where the causal argument breaks](03_critical_review.md#where-the-causal-argument-breaks) |
| Is the gain only more compute? | [TB2.1 inference compute](02_evaluation_audit.md#inference-compute) |
| Is WAI a fair browser comparison? | [WebArena-Infinity](02_evaluation_audit.md#webarena-infinity) |
| What is reproducible? | [Reproducibility](03_critical_review.md#reproducibility) |
| How does DarwinX compare with DGM or Meta-Harness? | [Comparative synthesis](04_comparative_synthesis.md) |
| What experiment would settle the mechanism claim? | [Successor experiment](05_successor_experiment.md) |

## Packet map

- [Mechanism and selection](01_mechanism_and_selection.md) reconstructs archive
  state, gates, parent selection, signals, memory, and recombination.
- [Evaluation audit](02_evaluation_audit.md) owns all benchmark accounting.
- [Critical review](03_critical_review.md) ranks causal and reproducibility
  gaps and audits the supplied review.
- [Comparative synthesis](04_comparative_synthesis.md) compares Meta-Harness,
  DGM, HarnessX, and DarwinX under one contract.
- [Successor experiment](05_successor_experiment.md) specifies the equal-budget
  factorial study.
- [Claim ledger](claim_evidence_ledger.md) owns material claims.
- [Source registry](source_registry.md) fixes source identity and claim ceiling.
- [Maintenance](maintenance.md) defines update and verification rules.

## Source boundary

Primary local evidence:

- [DarwinX v1 PDF](../../evidence/darwinx/artifacts/darwinx-2608.07545v1.pdf)
- [DarwinX v1 text](../../evidence/darwinx/text/darwinx-2608.07545v1.txt)
- [HarnessX v1 PDF](../../evidence/darwinx/artifacts/harnessx-2606.14249v1.pdf)
- [Evidence provenance](../../evidence/darwinx/PROVENANCE.md)

Existing Harp comparisons:

- [Meta-Harness deep dive](../meta_harness/meta_harness_deep_dive.md)
- [DGM packet](../darwin_godel_machine/darwin_godel_machine_index.md)
- [Harness benchmark field guide](../harness_benchmarks/harness_benchmark_field_guide.md)

## Verdict

DarwinX puts the hard question in the right place. The bottleneck is no longer
whether an agent can invent a harness edit. It is whether the system can decide
which measured edit is real, what it damages, whether an alternative lineage is
worth keeping, and whether two edits can coexist.

The paper shows that the full loop can produce a much better harness. The next
study must show which selection machinery earned that result.
