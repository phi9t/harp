---
id: recursive-self-improvement-index
title: Recursive self-improvement - orientation index
type: research-index
mode: DOMAIN ORIENTATION
status: active
created: 2026-07-31
updated: 2026-08-07
tags: [recursive-self-improvement, harness-engineering, automated-research, evolutionary-search, evaluation]
confidence: medium
---

# Recursive self-improvement

Mode: `DOMAIN ORIENTATION`.

**INFERENCE — synthesis from [WENG-HARNESS], the [RLM-PAPER] → [A1ZHANG-HARNESS-BLOG] lineage, [STOP], [DGM], [ALPHAEVOLVE], [AI-SCIENTIST], and [SIA].** Recursive self-improvement (RSI) is best treated as a property of an improvement loop, not a synonym for reflection, adaptation, or a model editing its own weights. The loop becomes recursive when a persistent change to the system improves the process that proposes, tests, selects, or retains later changes.

**INFERENCE — operational state model.** [[system_state_and_notation]] separates the editable candidate `Cₜ = (Wₜ, Hₜ, Dₜ, Rₜ)`—weights, harness, learned artifacts, and internal retrospection—from external signals `Xₜ`, evaluator `E`, budget `B`, permissions `P`, protected archive `Aₜ`, and promotion authority. A loop has an RSI claim only when an externally accepted `Cₜ₊₁` is reused and produces better later accepted changes under the same protected envelope.

**INFERENCE — strongest current boundary.** Public work in this packet demonstrates bounded pieces of that loop: improving an improver, evolving agent code, optimizing programs against executable evaluators, automating portions of ML research, and jointly choosing harness or weight updates. It does not establish an autonomous system that repeatedly chooses the AI-R&D agenda, builds and validates a generally more capable successor, deploys it, and hands the next cycle to that successor.

## Anchor lineages

- **EVIDENCE — Weng anchor.** [WENG-HARNESS] supplies the broad map from prompts and context through workflow, harness code, harness optimizers, autonomous research, and evaluator integrity.
- **EVIDENCE — RLM anchor.** [RLM-PAPER], [RLM-REPO], and [A1ZHANG-HARNESS-BLOG] supply a concrete lineage from recursive inference architecture to implementation and RL evidence about harness-induced compositional generalization.
- **INFERENCE — complementarity.** Weng identifies the harness as an improvement surface; the RLM lineage explains one way that surface can reshape the effective learning problem seen by the model. Neither anchor alone demonstrates multi-generation RSI.

## Choose a reading route

Choose the route that matches the decision at hand rather than reading the
packet in order. Start every route with the candidate/envelope split; finish
with the evidence boundary before treating an observed mechanism as an RSI
claim.

- **Classify a proposed improvement loop.** Start with [what makes an
  improvement loop recursive](chapters/recursive-improvement-loop.md), then
  use [system state and notation](concepts/system-state-and-notation.md) and
  [evaluation, promotion, and containment](chapters/evaluation-promotion-containment.md).
  Use the [comparison lessons](lessons/01-self-refine-vs-persistence.md) when
  the boundary between reflection, persistence, search, and accepted lineage
  is unclear.
- **Design or inspect a practical harness.** Read [foundation model inside the
  loop](chapters/foundation-model-inside-the-loop.md), [harness
  engineering](chapters/harness-engineering.md), and [durable improvement
  workflows](chapters/durable-improvement-workflows.md). Continue to
  [procedure internalization](chapters/procedure-internalization.md) when the
  question is whether an external procedure becomes a learned artifact or
  behavior.
- **Learn the systems and evaluator foundations.** Use the [SICP systems and
  evaluator course](sicp/sicp_index.md) for the book-order route through state,
  evaluation, authority, control, storage, concurrency, and compilation. Its
  SICP claims remain sourced to the book; modern harness comparisons remain
  labeled inferences rather than claims about the source.
- **Compare search and adaptation mechanisms.** Start with [harness
  search](chapters/harness-search.md), then choose [automated
  research](chapters/automated-research.md) or [joint harness and
  model-weight adaptation](chapters/joint-harness-weight-adaptation.md).
  Follow the matching [source-bound system reading](systems/system_readings_index.md)
  before comparing reported results.
- **Audit promotion, safety, or claim strength.** Read [evaluation, promotion,
  and containment](chapters/evaluation-promotion-containment.md) and
  [[evaluator_integrity_and_promotion]], then consult
  [[claim_evidence_ledger]] and [[missing_evidence]]. This route separates
  observed loop mechanics from the evidence needed to claim recursive,
  accepted improvement.
- **Trace an anchor or implementation.** Use
  [[rsi_harness_by_lil_log_deconstructed]] for Weng's argument,
  [[recursive_language_models_compositional_generalization]] for the RLM
  lineage, and [[implementation_harnesses]] before opening the Pi, Hermes, or
  Codex implementation profiles.

## Technical spine

The nine canonical chapters are the primary explanation:

1. [What makes an improvement loop recursive](chapters/recursive-improvement-loop.md).
2. [Foundation model inside the loop](chapters/foundation-model-inside-the-loop.md).
3. [Harness engineering](chapters/harness-engineering.md).
4. [Workflows that persist across interruptions](chapters/durable-improvement-workflows.md).
5. [From external procedures to learned behavior](chapters/procedure-internalization.md).
6. [Searching for better harnesses](chapters/harness-search.md).
7. [Automated research as an RSI component](chapters/automated-research.md).
8. [Joint harness and model-weight adaptation](chapters/joint-harness-weight-adaptation.md).
9. [Evaluation, promotion, and containment](chapters/evaluation-promotion-containment.md).

[RSI system state and notation](concepts/system-state-and-notation.md) owns the
shared symbols. [The closure coverage map](coverage-map.tsv) assigns every
retained concept one primary explanatory home.

[Technical system readings](systems/system_readings_index.md) map Weng's
argument to the inspected primary systems. All sixteen registry entries have
published canonical readings and six-part original-source routes.
[AFlow](systems/aflow.md) is the first-class MCTS workflow reading.

[Context engineering](context_engineering_deep_dive.md) provides the deeper
ACE → MCE → Meta-Harness route from learned artifacts to learned learning
procedures, then crosses that learning ladder with runtime selection,
activation, state continuity, compaction, and replay.

The [SICP systems and evaluator course](sicp/sicp_index.md) is the nested
foundations reader for executable semantics and system design. It preserves
its twelve-seminar book order, exercise and dialogue contracts, evaluator
supplements, source ledger, and agent-harness architecture capstone under RSI
ownership without treating SICP as an RSI source.

[Six comparison lessons](lessons/01-self-refine-vs-persistence.md) teach the
boundaries between output refinement, persistent adaptation, harness search,
accepted lineage, and joint harness-weight adaptation. Each lesson reveals the
same source-backed cases used by the deterministic diagnostic workbench.

[Open Harp Atlas](../atlas/README.md) for the guided chapter reader. 
## Categorized map

The technical spine above is the shortest explanation. This map accounts for
the maintained supporting material without making every page a prerequisite.

### Orientation and shared boundaries

- [Domain orientation](domain_orientation.md) and [improvement-loop
  taxonomy](improvement_loop_taxonomy.md) are compatibility routes into the
  canonical chapters.
- [System state and notation](system_state_and_notation.md) is the compatibility
  route; [canonical notation and ownership boundaries](concepts/system-state-and-notation.md)
  owns `Cₜ`, the protected envelope, and generation semantics.
- [Improvement types](concepts/improvement-types.md), [model
  adaptation](concepts/model-adaptation.md), and [evaluation and
  control](concepts/evaluation-and-control.md) provide focused definitions
  used across the chapters.

### Building and operating an improvement loop

- [Harness components](concepts/harness-components.md), [durable
  execution](concepts/durable-execution.md), and [procedure
  representations](concepts/procedure-representations.md) support the
  harness, workflow, and procedure chapters.
- [Harness search methods](concepts/harness-search-methods.md), [research-loop
  components](concepts/research-loop-components.md), and [joint adaptation
  methods](concepts/joint-adaptation-methods.md) support the search,
  automated-research, and joint-adaptation chapters.
- [Context-engineering deep dive](context_engineering_deep_dive.md) follows
  the ACE → MCE → Meta-Harness progression; [evaluator integrity and
  promotion](evaluator_integrity_and_promotion.md) expands the external
  evaluation and authority boundary.

### Worked distinctions and source-bound cases

- The six lessons compare [self-refinement and persistence](lessons/01-self-refine-vs-persistence.md),
  [ACE and MCE](lessons/02-ace-vs-mce.md), [ADAS and AFlow](lessons/03-adas-vs-aflow.md),
  [STOP and Self-Harness/AHE](lessons/04-stop-vs-self-harness-ahe.md),
  [AlphaEvolve and DGM](lessons/05-alphaevolve-vs-dgm.md), and [SIA and
  Continual Harness](lessons/06-sia-vs-continual-harness.md).
- The system-reading library groups [ACE](systems/ace.md), [MCE](systems/mce.md),
  and [Meta-Harness](systems/meta-harness.md) with context and learned-artifact
  systems; [ADAS](systems/adas.md), [AFlow](systems/aflow.md), [Self-Harness](systems/self-harness.md),
  [AHE](systems/ahe.md), and [AutoResearch](systems/autoresearch.md) with
  workflow and harness search.
- The same library groups [AlphaEvolve](systems/alphaevolve.md), [DGM](systems/dgm.md),
  and [AI Scientist](systems/ai-scientist.md) with evolutionary and automated
  research; [RLM](systems/rlm.md), [STOP](systems/stop.md), [SIA](systems/sia.md),
  [Continual Harness](systems/continual-harness.md), and [Harness
  Disentangle](systems/harness-disentangle.md) with recursive, persistent, or
  joint-adaptation boundaries.

### Anchor readers and implementation overlays

- The Weng source-order reader covers [the system being improved](weng/01-system-being-improved.md),
  [harness design patterns](weng/02-harness-design-patterns.md), [harness
  layer versus core intelligence](weng/03-harness-layer-vs-core-intelligence.md),
  [context engineering](weng/04-context-engineering.md), [workflow design and
  search](weng/05-workflow-design-and-search.md), [self-improving
  harnesses](weng/06-self-improving-harnesses.md), [evolutionary
  search](weng/07-evolutionary-search.md), [joint harness-weight
  optimization](weng/08-joint-harness-weight-optimization.md), and [future
  challenges](weng/09-future-challenges.md).
- [[rsi_harness_by_lil_log_deconstructed]] preserves the Weng companion;
  [[recursive_language_models_compositional_generalization]] preserves the RLM
  mechanism anchor.
- [[implementation_harnesses]] compares the overlay structures, while
  [[pi_harness_deep_dive]], [[hermes_harness_deep_dive]], and
  [[codex_harness_deep_dive]] retain implementation-specific paths and
  revisions. [[codex_state_continuity_and_compaction]] covers the Codex
  state-continuity comparison.

### Evidence, coverage, and provenance

- [[claim_evidence_ledger]] records exact claim locators and claim ceilings;
  [[missing_evidence]] records the gaps that block stronger conclusions.
- [[source_registry]] and [[evidence_graph]] present the human-readable views
  of the authoritative [source registry](sources/source_registry.tsv) and
  [evidence graph](sources/evidence_graph.tsv).
- [[bounded_transitive_closure]] records the admitted Weng-rooted closure;
  [retained concepts](retained-concepts.tsv) and [the closure coverage
  map](coverage-map.tsv) account for the retained explanatory surface.
- [Codex state-continuity provenance](sources/codex-state-continuity-provenance.md)
  records the source boundary for that implementation comparison.

## Reference layer

The supporting layer is separate from the reading spine:

- [[rsi_harness_by_lil_log_deconstructed]] remains the Weng reader companion,
  and [[recursive_language_models_compositional_generalization]] remains the RLM
  mechanism anchor.
- [[pi_harness_deep_dive]], [[hermes_harness_deep_dive]], and
  [[codex_harness_deep_dive]] retain exact implementation paths and revisions.
- [[claim_evidence_ledger]] contains exact claim locators and limits; [[missing_evidence]] prevents unavailable or unverified material from becoming a stronger claim.
- [[source_registry]] and [[evidence_graph]] document the authoritative TSV registries under `sources/`.
- [[bounded_transitive_closure]] records the Weng-rooted expansion rule, retained branches, stop predicate, and closure status.
- `evidence/weng/` vendors the Weng anchor's bounded two-hop external closure. `evidence/rlm/` captures the RLM anchor paper, implementation metadata, training/generalization blog, figures, and parsed citations.

## Baseline five-minute route

1. Read [what makes an improvement loop recursive](chapters/recursive-improvement-loop.md).
2. Use [system state and notation](concepts/system-state-and-notation.md) to separate candidate state from evaluator authority.
3. Read [harness engineering](chapters/harness-engineering.md) and [harness search](chapters/harness-search.md).
4. Read [joint adaptation](chapters/joint-harness-weight-adaptation.md) before interpreting model-plus-harness results.
5. End with [evaluation, promotion, and containment](chapters/evaluation-promotion-containment.md).
6. Open the Weng companion, RLM anchor, or implementation deep dives only when their source-specific detail is needed.
7. Consult [[claim_evidence_ledger]] or [[missing_evidence]] for exact evidence receipts and open gaps.

## Provenance contract

**CLAIM — research-domain-orientation skill.** Every non-trivial prose statement in this packet is marked `CLAIM`, `EVIDENCE`, `INFERENCE`, or `SPECULATION`; absent or inaccessible evidence is marked `MISSING`.

**EVIDENCE — anchor manifests.** Lilian Weng's 2026 article and the RLM paper → implementation → blog lineage are first-class anchors. Weng's 39 numbered references are registered and vendored exactly once, whether interpreted or not. The RLM capture preserves its two substantive texts, pinned implementation metadata, 11-entry blog bibliography, and 51 parsed paper citations without silently admitting all of those citations into the maintained synthesis.

**MISSING.** The packet contains no independent, matched-budget reproduction of a recursively improving system across multiple accepted generations.
