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

- **Classify a proposed improvement loop.** Start with [[knowledge/rsi/chapters/recursive-improvement-loop|what makes an improvement loop recursive]], then
  use [[knowledge/rsi/concepts/system-state-and-notation|system state and notation]] and
  [[knowledge/rsi/chapters/evaluation-promotion-containment|evaluation, promotion, and containment]].
  Use the [[knowledge/rsi/lessons/01-self-refine-vs-persistence|comparison lessons]] when
  the boundary between reflection, persistence, search, and accepted lineage
  is unclear.
- **Design or inspect a practical harness.** Read [[knowledge/rsi/chapters/foundation-model-inside-the-loop|foundation model inside the loop]], [[knowledge/rsi/chapters/harness-engineering|harness engineering]], and [[knowledge/rsi/chapters/durable-improvement-workflows|durable improvement workflows]]. Continue to
  [[knowledge/rsi/chapters/procedure-internalization|procedure internalization]] when the
  question is whether an external procedure becomes a learned artifact or
  behavior.
- **Learn the systems and evaluator foundations.** Use the [[knowledge/rsi/sicp/sicp_index|SICP systems and evaluator course]] for the book-order route through state,
  evaluation, authority, control, storage, concurrency, and compilation. Its
  SICP claims remain sourced to the book; modern harness comparisons remain
  labeled inferences rather than claims about the source.
- **Compare search and adaptation mechanisms.** Start with [[knowledge/rsi/chapters/harness-search|harness search]], then choose [[knowledge/rsi/chapters/automated-research|automated research]] or [[knowledge/rsi/chapters/joint-harness-weight-adaptation|joint harness and model-weight adaptation]].
  Follow the matching [[knowledge/rsi/systems/system_readings_index|source-bound system reading]]
  before comparing reported results.
- **Audit promotion, safety, or claim strength.** Read [[knowledge/rsi/chapters/evaluation-promotion-containment|evaluation, promotion, and containment]] and
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

1. [[knowledge/rsi/chapters/recursive-improvement-loop|What makes an improvement loop recursive]].
2. [[knowledge/rsi/chapters/foundation-model-inside-the-loop|Foundation model inside the loop]].
3. [[knowledge/rsi/chapters/harness-engineering|Harness engineering]].
4. [[knowledge/rsi/chapters/durable-improvement-workflows|Workflows that persist across interruptions]].
5. [[knowledge/rsi/chapters/procedure-internalization|From external procedures to learned behavior]].
6. [[knowledge/rsi/chapters/harness-search|Searching for better harnesses]].
7. [[knowledge/rsi/chapters/automated-research|Automated research as an RSI component]].
8. [[knowledge/rsi/chapters/joint-harness-weight-adaptation|Joint harness and model-weight adaptation]].
9. [[knowledge/rsi/chapters/evaluation-promotion-containment|Evaluation, promotion, and containment]].

[[knowledge/rsi/concepts/system-state-and-notation|RSI system state and notation]] owns the
shared symbols. [[content/coverage-map.tsv|The closure coverage map]] assigns every
retained concept one primary explanatory home.

[[knowledge/rsi/systems/system_readings_index|Technical system readings]] map Weng's
argument to the inspected primary systems. All sixteen registry entries have
published canonical readings and six-part original-source routes.
[[knowledge/rsi/systems/aflow|AFlow]] is the first-class MCTS workflow reading.

[[knowledge/rsi/context_engineering_deep_dive|Context engineering]] provides the deeper
ACE → MCE → Meta-Harness route from learned artifacts to learned learning
procedures, then crosses that learning ladder with runtime selection,
activation, state continuity, compaction, and replay.

[[knowledge/meta_harness/meta_harness_deep_dive|Meta-Harness implementation deep dive]] separates
the paper, dated project page, pinned repositories, and one local TRAE proposal
iteration. Use it for implementation and evidence boundaries rather than
treating the cleaned public code as a reproduction.

[[knowledge/darwinx/darwinx_index|DarwinX population-selection deep dive]] audits
bounded-regression promotion, archive and steering state, specialist
recombination, benchmark accounting, and the missing equal-budget
single-lineage comparison. Use it when the question is how a harness search
retains measured gains rather than how it proposes an edit.

The [[knowledge/rsi/sicp/sicp_index|SICP systems and evaluator course]] is the nested
foundations reader for executable semantics and system design. It preserves
its twelve-seminar book order, exercise and dialogue contracts, evaluator
supplements, source ledger, and agent-harness architecture capstone under RSI
ownership without treating SICP as an RSI source.

[[knowledge/rsi/lessons/01-self-refine-vs-persistence|Six comparison lessons]] teach the
boundaries between output refinement, persistent adaptation, harness search,
accepted lineage, and joint harness-weight adaptation. Each lesson reveals the
same source-backed cases used by the deterministic diagnostic workbench.

[[atlas/README|Open Harp Atlas]] for the guided chapter reader.
## Categorized map

The technical spine above is the shortest explanation. This map accounts for
the maintained supporting material without making every page a prerequisite.

### Orientation and shared boundaries

- [[knowledge/rsi/domain_orientation|Domain orientation]] and [[knowledge/rsi/improvement_loop_taxonomy|improvement-loop taxonomy]] are compatibility routes into the
  canonical chapters.
- [[knowledge/rsi/system_state_and_notation|System state and notation]] is the compatibility
  route; [[knowledge/rsi/concepts/system-state-and-notation|canonical notation and ownership boundaries]]
  owns `Cₜ`, the protected envelope, and generation semantics.
- [[knowledge/rsi/concepts/improvement-types|Improvement types]], [[knowledge/rsi/concepts/model-adaptation|model adaptation]], and [[knowledge/rsi/concepts/evaluation-and-control|evaluation and control]] provide focused definitions
  used across the chapters.

### Building and operating an improvement loop

- [[knowledge/rsi/concepts/harness-components|Harness components]], [[knowledge/rsi/concepts/durable-execution|durable execution]], and [[knowledge/rsi/concepts/procedure-representations|procedure representations]] support the
  harness, workflow, and procedure chapters.
- [[knowledge/rsi/concepts/harness-search-methods|Harness search methods]], [[knowledge/rsi/concepts/research-loop-components|research-loop components]], and [[knowledge/rsi/concepts/joint-adaptation-methods|joint adaptation methods]] support the search,
  automated-research, and joint-adaptation chapters.
- [[knowledge/rsi/context_engineering_deep_dive|Context-engineering deep dive]] follows
  the ACE → MCE → Meta-Harness progression; [[knowledge/rsi/evaluator_integrity_and_promotion|evaluator integrity and promotion]] expands the external
  evaluation and authority boundary.

### Worked distinctions and source-bound cases

- The six lessons compare [[knowledge/rsi/lessons/01-self-refine-vs-persistence|self-refinement and persistence]],
  [[knowledge/rsi/lessons/02-ace-vs-mce|ACE and MCE]], [[knowledge/rsi/lessons/03-adas-vs-aflow|ADAS and AFlow]],
  [[knowledge/rsi/lessons/04-stop-vs-self-harness-ahe|STOP and Self-Harness/AHE]],
  [[knowledge/rsi/lessons/05-alphaevolve-vs-dgm|AlphaEvolve and DGM]], and [[knowledge/rsi/lessons/06-sia-vs-continual-harness|SIA and Continual Harness]].
- The system-reading library groups [[knowledge/rsi/systems/ace|ACE]], [[knowledge/rsi/systems/mce|MCE]],
  and [[knowledge/rsi/systems/meta-harness|Meta-Harness]] with context and learned-artifact
  systems; [[knowledge/rsi/systems/adas|ADAS]], [[knowledge/rsi/systems/aflow|AFlow]], [[knowledge/rsi/systems/self-harness|Self-Harness]],
  [[knowledge/rsi/systems/ahe|AHE]], and [[knowledge/rsi/systems/autoresearch|AutoResearch]] with
  workflow and harness search.
- The same library groups [[knowledge/rsi/systems/alphaevolve|AlphaEvolve]], [[knowledge/rsi/systems/dgm|DGM]],
  and [[knowledge/rsi/systems/ai-scientist|AI Scientist]] with evolutionary and automated
  research; [[knowledge/rsi/systems/rlm|RLM]], [[knowledge/rsi/systems/stop|STOP]], [[knowledge/rsi/systems/sia|SIA]],
  [[knowledge/rsi/systems/continual-harness|Continual Harness]], and [[knowledge/rsi/systems/harness-disentangle|Harness Disentangle]] with recursive, persistent, or
  joint-adaptation boundaries.

### Anchor readers and implementation overlays

- The Weng source-order reader covers [[knowledge/rsi/weng/01-system-being-improved|the system being improved]],
  [[knowledge/rsi/weng/02-harness-design-patterns|harness design patterns]], [[knowledge/rsi/weng/03-harness-layer-vs-core-intelligence|harness layer versus core intelligence]],
  [[knowledge/rsi/weng/04-context-engineering|context engineering]], [[knowledge/rsi/weng/05-workflow-design-and-search|workflow design and search]], [[knowledge/rsi/weng/06-self-improving-harnesses|self-improving harnesses]], [[knowledge/rsi/weng/07-evolutionary-search|evolutionary search]], [[knowledge/rsi/weng/08-joint-harness-weight-optimization|joint harness-weight optimization]], and [[knowledge/rsi/weng/09-future-challenges|future challenges]].
- [[rsi_harness_by_lil_log_deconstructed]] preserves the Weng companion;
  [[recursive_language_models_compositional_generalization]] preserves the RLM
  mechanism anchor.
- [[implementation_harnesses]] compares the overlay structures, while
  [[pi_harness_deep_dive]], [[hermes_harness_deep_dive]], and
  [[codex_harness_deep_dive]], and [[deepseek_harness_deep_dive]] retain
  implementation-specific paths and revisions.
  [[codex_state_continuity_and_compaction]] covers the Codex state-continuity
  comparison.

### Evidence, coverage, and provenance

- [[claim_evidence_ledger]] records exact claim locators and claim ceilings;
  [[missing_evidence]] records the gaps that block stronger conclusions.
- [[source_registry]] and [[evidence_graph]] present the human-readable views
  of the authoritative [[content/sources/source_registry.tsv|source registry]] and
  [[content/sources/evidence_graph.tsv|evidence graph]].
- [[bounded_transitive_closure]] records the admitted Weng-rooted closure;
  [[content/retained-concepts.tsv|retained concepts]] and [[content/coverage-map.tsv|the closure coverage map]] account for the retained explanatory surface.
- [[knowledge/rsi/sources/codex-state-continuity-provenance|Codex state-continuity provenance]]
  records the source boundary for that implementation comparison.

## Reference layer

The supporting layer is separate from the reading spine:

- [[rsi_harness_by_lil_log_deconstructed]] remains the Weng reader companion,
  and [[recursive_language_models_compositional_generalization]] remains the RLM
  mechanism anchor.
- [[pi_harness_deep_dive]], [[hermes_harness_deep_dive]],
  [[codex_harness_deep_dive]], and [[deepseek_harness_deep_dive]] retain exact
  implementation paths and revisions.
- [[claim_evidence_ledger]] contains exact claim locators and limits; [[missing_evidence]] prevents unavailable or unverified material from becoming a stronger claim.
- [[source_registry]] and [[evidence_graph]] document the authoritative TSV registries under `sources/`.
- The [[knowledge/harness_benchmarks/harness_benchmark_field_guide|harness benchmark field guide]]
  compares evaluator boundaries and preserves the complete MCE experiment protocol.
- The [[knowledge/darwinx/darwinx_index|DarwinX packet]] separates reported
  frozen-model harness gains from causal evidence for population search,
  bounded-regression selection, and recombination.
- The [[knowledge/evaluator_integrity/evaluator_integrity_benchmark_suite|evaluator-integrity benchmark suite]]
  compares GDPval, DeepSWE, FrontierCode 1.1, and SWE-bench Verified without
  collapsing their different access, contamination, and verifier boundaries
  into a capability ranking.
- [[bounded_transitive_closure]] records the Weng-rooted expansion rule, retained branches, stop predicate, and closure status.
- `evidence/weng/` vendors the Weng anchor's bounded two-hop external closure.
  `evidence/rlm/` captures the RLM paper, training/generalization blog, figures,
  and parsed citations. Narrow full and minimal code snapshots live under
  `evidence/implementations/`.

## Baseline five-minute route

1. Read [[knowledge/rsi/chapters/recursive-improvement-loop|what makes an improvement loop recursive]].
2. Use [[knowledge/rsi/concepts/system-state-and-notation|system state and notation]] to separate candidate state from evaluator authority.
3. Read [[knowledge/rsi/chapters/harness-engineering|harness engineering]] and [[knowledge/rsi/chapters/harness-search|harness search]].
4. Read [[knowledge/rsi/chapters/joint-harness-weight-adaptation|joint adaptation]] before interpreting model-plus-harness results.
5. End with [[knowledge/rsi/chapters/evaluation-promotion-containment|evaluation, promotion, and containment]].
6. Open the Weng companion, RLM anchor, or implementation deep dives only when their source-specific detail is needed.
7. Consult [[claim_evidence_ledger]] or [[missing_evidence]] for exact evidence receipts and open gaps.

## Provenance contract

**CLAIM — research-domain-orientation skill.** Every non-trivial prose statement in this packet is marked `CLAIM`, `EVIDENCE`, `INFERENCE`, or `SPECULATION`; absent or inaccessible evidence is marked `MISSING`.

**EVIDENCE — anchor manifests.** Lilian Weng's 2026 article and the RLM paper → implementation → blog lineage are first-class anchors. Weng's 39 numbered references are registered and vendored exactly once, whether interpreted or not. The RLM capture preserves its two substantive texts, narrow pinned full and minimal code snapshots, 11-entry blog bibliography, and 51 parsed paper citations without silently admitting all of those citations into the maintained synthesis.

**MISSING.** The packet contains no independent, matched-budget reproduction of a recursively improving system across multiple accepted generations.
