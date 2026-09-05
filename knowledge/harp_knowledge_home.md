---
id: harp-knowledge-home
title: Harp research atlas
type: research-index
status: active
tags: [harp, knowledge, navigation, obsidian]
confidence: high
---

# How can agents improve, and how can we verify it?

Harp connects research on recursive self-improvement with durable agent
execution and mathematical proof studies. Read the mechanisms, inspect the
implementation, and follow each claim to its evidence and limits.

## RSI research

Study what changes in an improvement loop, how a candidate is evaluated, and
whether accepted changes improve later attempts. Persistent adaptation alone
does not establish recursive self-improvement.

- [[knowledge/rsi/chapters/recursive-improvement-loop|Start with the improvement loop]]
  Distinguish reflection, persistent changes, and accepted improvement across generations.
- [Compare research systems](https://github.com/phi9t/harp/blob/master/knowledge/rsi/systems/system_readings_index.md)
  Follow source-bound readings of harness search, automated research, and joint model-harness adaptation.
- [[knowledge/rsi/context_engineering_deep_dive|Explore context engineering]]
  Trace learned artifacts, runtime context selection, and state continuity.
- [[knowledge/darwinx/darwinx_index|Study population-based harness evolution]]
  Inspect selection, retained gains, and missing comparisons.
- [[knowledge/verified_coevolution_agenda/verified_coevolution_agenda|Read the coevolution research agenda]]
  Separate proposed research from established results.

## Durable execution

Follow how Harp turns authored workflows into a durable task graph. Scheduling,
recorded results, and recovery make a run inspectable across interruptions;
they do not establish that the agent's answer is correct.

- [[knowledge/rsi/concepts/durable-execution#Harp's executor and scheduler|Understand Harp's executor]]
  Connect workflow compilation, dependency scheduling, and recorded state.
- [[knowledge/rsi/chapters/durable-improvement-workflows|Read the durability model]]
  Work through checkpoints, retries, ambiguous effects, and promotion authority.
- [[knowledge/agentic_engineering/agentic_engineering_index|Study agentic engineering]]
  Keep intent, execution, verification, and human control separate.

The implementation guide is [Agent workflows](https://github.com/phi9t/harp/blob/master/docs/agent-workflows.md)
  It covers
the durable Codex task graph and the separate Codex or Trae provider wrapper.
The Atlas's Workstreams view is an offline reference snapshot, not a live
monitor or a control panel for running jobs.

## Mathematics and Lean

Read mathematical arguments alongside their formalization boundaries. Harp
includes Crouzeix proof studies and Lean companions for mathematical
foundations. A theorem's exact statement, assumptions, and verification record
matter more than a page-level completion label.

- [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Explore the Crouzeix proof routes]]
  Compare the Jin and Lorist-Schwenninger arguments and Harp's local route.
- [[knowledge/crouzeix_conjecture/09_status_and_critical_assessment|Check the proof status and limits]]
  Harp records three local route certifications. These do not establish upstream clean-room builds, author endorsement, or external peer review.
- [[knowledge/mathematical_foundations/mathematical_foundations_index|Learn the mathematical foundations]]
  Follow finite-domain Lean companions and the map from exercises to theorems, applications, or prose-only explanations.
- [[knowledge/autodiff_geometry/autodiff_geometry_index|Study automatic differentiation and geometry]]
  Connect mathematical exposition with its bounded formal companion.

## Evaluation and evidence

Inspect the evaluator, source, and reproduction record behind a result. A
navigation link is not evidence, and a successful local run does not by itself
establish a general capability gain.

- [[knowledge/harness_benchmarks/harness_benchmark_field_guide|Understand the benchmark landscape]]
  Compare task definitions, access policies, verifier disclosure, and what each result can establish.
- [Trace research claims](https://github.com/phi9t/harp/blob/master/knowledge/rsi/claim_evidence_ledger.md)
  Find the supporting route and the allowed strength of each claim.
- [[knowledge/rsi/source_registry|Inspect the source registry]]
  Check primary sources and capture boundaries.
- [Read the evidence gaps](https://github.com/phi9t/harp/blob/master/knowledge/rsi/missing_evidence.md)
  Identify what Harp does not possess or establish.

## Further study

The [SICP systems and evaluator course](https://github.com/phi9t/harp/blob/master/knowledge/rsi/sicp/sicp_index.md) develops
the foundations of state, evaluation, authority, concurrency, and compilation.
[Implementation studies](https://github.com/phi9t/harp/blob/master/knowledge/rsi/implementation_harnesses.md) compare
concrete agent architectures. The full Atlas library also includes
[[knowledge/meta_harness/meta_harness_deep_dive|Meta-Harness]],
[Darwin Gödel Machine](https://github.com/phi9t/harp/blob/master/knowledge/darwin_godel_machine/darwin_godel_machine_index.md),
and source-specific research packets.

## Reading and provenance

The Atlas reads a compiled snapshot of maintained Markdown. Article metadata
describes the document; it is not an independent proof or benchmark verdict.
Source sections and exact Markdown receipts remain available within each
reading. Captured evidence and repository files are not bundled into the
standalone HTML; use their repository locations when inspecting original bytes.

For local reading, open the repository root as an Obsidian vault. Use the
[[knowledge/obsidian/harp_knowledge.base|Knowledge Base]] or
[[knowledge/obsidian/harp_knowledge_map.canvas|Knowledge Canvas]] to navigate.
Maintained prose lives in `knowledge/`, captured bytes in `evidence/`, and
machine-readable contracts in `content/`.
