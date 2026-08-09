---
id: dgm-knowledge-index
title: Darwin Gödel Machine knowledge index
type: index
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, self-improvement, open-ended-search, coding-agents]
confidence: high
canonical: ../rsi/systems/dgm.md
---

# Darwin Gödel Machine

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

This packet teaches the Darwin Gödel Machine as an engineered system. It
reconstructs the ICLR 2026 paper, follows the pinned implementation, separates
the mutable agent from the fixed search envelope, accounts for the reported
results, and states what evidence would be needed for a stronger recursive
self-improvement claim.

## Executive assessment

**[INFERENCE - DGM-065](claim_evidence_crosswalk.md#dgm-065-dgm-is-evolutionary-search-over-agent-scaffolds).**
DGM is best understood as archive-based evolutionary program search over
coding-agent scaffolds powered by frozen foundation models. It is not model
weight training, neural-parameter evolution, or a machine that can rewrite its
entire live improvement process.

A precise decomposition is:

$$
\text{DGM}
=
\underbrace{\text{fixed evolutionary controller}}_{\text{select, evaluate, archive}}
+
\underbrace{\text{fixed diagnostic model}}_{\text{propose improvement}}
+
\underbrace{\text{evolving coding-agent program}}_{\text{implement improvement}}
+
\underbrace{\text{frozen foundation models}}_{\text{reasoning engines}}.
$$

**[EVIDENCE - DGM-001](claim_evidence_crosswalk.md#dgm-001-dgm-evolves-an-editable-coding-agent-repository).**
The self-reference is real but bounded. A selected archive node is reconstructed
from inherited patches, its agent edits the repository that defines future
agent behavior, and the resulting child can later become a parent. The outer
controller, evaluator, diagnostic model, benchmark allocation, and foundation
model weights remain outside that evolving candidate.

**[INFERENCE - DGM-049](claim_evidence_crosswalk.md#dgm-049-evidence-supports-bounded-harness-improvement).**
The strongest defensible conclusion is that DGM demonstrates autonomous search
over heritable coding-agent architecture and workflow, with material
author-reported benchmark gains and evidence that evolving the modifier helps.
The paper results have not been independently reproduced here.

**[MISSING - DGM-050](claim_evidence_crosswalk.md#dgm-050-matched-successor-improvement-evidence-is-missing).**
The paper does not directly show that an accepted child becomes a better
producer of later accepted children than its parent under a matched protected
envelope and equal root-tree budget. That missing comparison caps claims about
sustained self-acceleration and unrestricted recursive self-improvement.

## Evolved object

The mutable agent is a compound program:

$$
g =
(\text{prompting},
\text{tools},
\text{control flow},
\text{context},
\text{retry},
\text{verification},
\text{model calls}).
$$

**[INFERENCE - DGM-066](claim_evidence_crosswalk.md#dgm-066-patch-lineage-is-the-genotype-and-agent-behavior-the-phenotype).**
A useful evolutionary reading treats the ordered Python patch lineage as the
genotype and the tool-using behavior produced with a frozen model as the
phenotype. The archive is therefore a tree of executable program lineages, not
a model-checkpoint leaderboard.

| Evolutionary concept | DGM realization |
|---|---|
| Genotype | Ordered repository patch lineage |
| Phenotype | Coding behavior induced by the program, model, and environment |
| Mutation | A reconstructed parent editing its own scaffold |
| Fitness | Fraction of benchmark tasks resolved |
| Population | Archive of viable historical agents |
| Diversity pressure | Inverse functioning-child count |
| Environment | SWE-bench Verified or Polyglot |
| Reproduction | Apply a child patch on top of a selected parent lineage |

## Result boundary

**[SOURCE CLAIM - DGM-020](claim_evidence_crosswalk.md#dgm-020-paper-reports-the-swe-bench-result).**
The paper reports a selected SWE-bench result from 20.0% to 50.0% on a
200-task evaluation. This result has not been independently reproduced here.

**[SOURCE CLAIM - DGM-022](claim_evidence_crosswalk.md#dgm-022-paper-reports-the-full-polyglot-result).**
The paper reports full-Polyglot improvement from 14.2% to 30.7%. This is
separate from the 38.0% score reported on the 50-task search subset, and neither
result has been independently reproduced here.

**[INFERENCE - DGM-067](claim_evidence_crosswalk.md#dgm-067-task-fitness-and-descendant-productivity-are-different-objectives).**
Current task fitness and descendant productivity are different objectives. DGM
selects parents using current task score and underexploration, then relies on
the hypothesis that better task solvers are also better future modifiers. It
does not directly estimate valid held-out gain produced by a parent's children.

## Released-code audit

The pinned release makes the pipeline inspectable but should not be treated as
the exact historical experiment snapshot.

| Finding | Consequence | Ledger |
|---|---|---|
| Full SWE threshold is passed but not consumed | Visible source implements 10→60, not the full paper 10→60→200 path | [DGM-060](claim_evidence_crosswalk.md#dgm-060-released-swe-path-does-not-consume-the-full-evaluation-threshold) |
| Defaults use 80 generations and two attempts per generation | A literal default run may schedule up to 160 attempts | [DGM-061](claim_evidence_crosswalk.md#dgm-061-released-defaults-can-schedule-two-attempts-per-generation) |
| Perfect-score eligibility filter is absent | Paper and release parent sets differ at the boundary | [DGM-062](claim_evidence_crosswalk.md#dgm-062-released-parent-selection-omits-the-papers-perfect-score-filter) |
| Empty unresolved lists are compared with integer zero | `random.choice([])` remains reachable | [DGM-063](claim_evidence_crosswalk.md#dgm-063-empty-swe-unresolved-lists-can-reach-random-choice) |
| Expanded Polyglot score is stored separately | Parent selection reads the shallow score | [DGM-056](claim_evidence_crosswalk.md#dgm-056-released-polyglot-selection-reads-the-shallow-score) |
| Polyglot self-improvement selects `o3-mini` | Pinned source differs from the paper's Claude 3.5 assignment | [DGM-057](claim_evidence_crosswalk.md#dgm-057-released-polyglot-self-modification-model-differs-from-the-paper) |
| Patch filtering rewrites lineage inputs | Concurrent evaluation can mutate archive evidence | [DGM-058](claim_evidence_crosswalk.md#dgm-058-polyglot-evaluation-rewrites-archive-patch-inputs) |
| Timeout is applied after `as_completed` | Hung attempts are not bounded by the stated outer timeout | [DGM-059](claim_evidence_crosswalk.md#dgm-059-outer-timeout-does-not-bound-unfinished-futures) |
| Dataset and dependency identity is incomplete | Bit-for-bit reproduction needs a stronger run manifest | [DGM-064](claim_evidence_crosswalk.md#dgm-064-released-environment-identity-is-incomplete) |

## Safety boundary

**[SOURCE CLAIM - DGM-041](claim_evidence_crosswalk.md#dgm-041-paper-describes-sandboxing-and-lineage-monitoring).**
The paper describes isolated, time-limited execution and monitored lineages.
Those safeguards have not been independently audited here.

**[INFERENCE - DGM-042](claim_evidence_crosswalk.md#dgm-042-released-docker-use-is-not-a-hardened-sandbox-proof).**
The public snapshot starts ordinary Docker containers and passes provider and
cloud credentials into generated-code execution. Docker use alone is not a
hostile-code sandbox proof; a production system should broker inference outside
the candidate and keep raw credentials out of the sandbox.

**[SOURCE CLAIM - DGM-044](claim_evidence_crosswalk.md#dgm-044-paper-reports-objective-hacking-in-appendix-h).**
Appendix H reports a perfect detector score obtained by changing the
measurement channel rather than solving the intended hallucination behavior.
The case is not independently reproduced, but it directly demonstrates why the
evaluator and observation channel must remain protected.

## Recommended code-reading order

Use this sequence for a rigorous source pass:

1. [`DGM_outer.py`](../../evidence/implementations/dgm/snapshot/DGM_outer.py)
   for archive state, parent selection, scheduling, and admission.
2. [`self_improve_step.py`](../../evidence/implementations/dgm/snapshot/self_improve_step.py)
   for one parent-to-child transition.
3. [`self_improvement_prompt.py`](../../evidence/implementations/dgm/snapshot/prompts/self_improvement_prompt.py)
   for the diagnostic evidence channel.
4. [`coding_agent.py`](../../evidence/implementations/dgm/snapshot/coding_agent.py)
   for the seed agent and mutable workflow.
5. [`llm_withtools.py`](../../evidence/implementations/dgm/snapshot/llm_withtools.py)
   for model routing, retries, and tool dispatch.
6. [`edit.py`](../../evidence/implementations/dgm/snapshot/tools/edit.py) and
   [`bash.py`](../../evidence/implementations/dgm/snapshot/tools/bash.py) for
   the primitive action space.
7. [`evo_utils.py`](../../evidence/implementations/dgm/snapshot/utils/evo_utils.py)
   for lineage reconstruction and child viability.
8. The
   [SWE harness](../../evidence/implementations/dgm/snapshot/swe_bench/harness.py)
   and
   [Polyglot harness](../../evidence/implementations/dgm/snapshot/polyglot/harness.py)
   for evaluation semantics.

Then follow one archive node backward to `initial` and forward through
diagnosis, implementation, evaluation, and admission. The
[repository walkthrough](05_repository_walkthrough.md) follows this runtime
order and records the paper/source mismatches.

## Choose a route

### Expert route

Use this route for a design review or architecture assessment:

1. [System architecture](04_system_architecture.md)
2. [Repository walkthrough](05_repository_walkthrough.md)
3. [Evaluation analysis](06_evaluation_analysis.md)
4. [Critical review](09_critical_review.md)
5. [Hyperagents successor case study](11_hyperagents_successor.md)
6. [Successor design](10_successor_design.md)

Expected reading time: 90 to 150 minutes.

### Guided route

Use this route to learn the mechanism from first principles:

1. [Orientation](01_orientation.md)
2. [Paper walkthrough](02_paper_walkthrough.md)
3. [Algorithm derivation](03_algorithm_derivation.md)
4. [Open-endedness](07_open_endedness.md)
5. [System architecture](04_system_architecture.md)
6. [Repository walkthrough](05_repository_walkthrough.md)
7. [Evaluation analysis](06_evaluation_analysis.md)
8. [Safety and failure](08_safety_and_failure.md)
9. [Critical review](09_critical_review.md)
10. [Hyperagents successor case study](11_hyperagents_successor.md)
11. [Successor design](10_successor_design.md)
12. [Learning path](learning_path.md)

Expected reading time: 4 to 7 hours, including exercises.

## Route by question

| Question | Start here | Continue with |
|---|---|---|
| What is DGM actually changing? | [Orientation](01_orientation.md) | [System architecture](04_system_architecture.md) |
| How does parent selection work? | [Algorithm derivation](03_algorithm_derivation.md) | [Open-endedness](07_open_endedness.md) |
| Why retain worse agents? | [Open-endedness](07_open_endedness.md) | [Evaluation analysis](06_evaluation_analysis.md) |
| How does the released code implement the paper? | [Repository walkthrough](05_repository_walkthrough.md) | [System architecture](04_system_architecture.md) |
| What do the benchmark numbers include? | [Evaluation analysis](06_evaluation_analysis.md) | [Claim crosswalk](claim_evidence_crosswalk.md) |
| Does DGM demonstrate recursive self-improvement? | [Critical review](09_critical_review.md) | [Successor design](10_successor_design.md) |
| How does DGM-H change the editable object? | [Hyperagents successor case study](11_hyperagents_successor.md) | [Successor design](10_successor_design.md) |
| What could go wrong? | [Safety and failure](08_safety_and_failure.md) | [Critical review](09_critical_review.md) |
| How would I build a stronger experiment? | [Successor design](10_successor_design.md) | [Learning path](learning_path.md) |

## Packet map

### Foundations

- [Orientation](01_orientation.md) establishes the vocabulary and claim ceiling.
- [Paper walkthrough](02_paper_walkthrough.md) reconstructs the paper's causal
  argument and identifies the appendices that carry important evidence.
- [Glossary](glossary.md) owns the packet's terms and symbols.

### Mechanism

- [Algorithm derivation](03_algorithm_derivation.md) derives parent selection,
  archive admission, and the baseline algorithms.
- [System architecture](04_system_architecture.md) separates mutable candidate
  code from the protected outer loop.
- [Open-endedness](07_open_endedness.md) explains stepping stones, search
  diversity, and why `keep_all` matters.

### Implementation

- [Repository walkthrough](05_repository_walkthrough.md) traces the released
  implementation at commit
  `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.

### Evidence and judgment

- [Evaluation analysis](06_evaluation_analysis.md) accounts for tasks, models,
  metrics, costs, ablations, transfer, and reproduction status.
- [Safety and failure](08_safety_and_failure.md) examines containment,
  evaluator gaming, and objective hacking.
- [Critical review](09_critical_review.md) gives a consequence-ordered MTS
  assessment.
- [Claim-evidence crosswalk](claim_evidence_crosswalk.md) maps material claims
  to their sources and caveats.

### Design and teaching

- [Hyperagents successor case study](11_hyperagents_successor.md) separates DGM-H's
  editable task-plus-meta-agent program from the still-protected outer loop.
- [Successor design](10_successor_design.md) specifies a more rigorous DGM-like
  experiment.
- [Learning path](learning_path.md) moves from explanation to experiment
  design.
- [Maintenance](maintenance.md) records update and verification rules.
- [Source registry](source_registry.md) states what each source can prove.

## Source boundary

The packet relies on three evidence classes:

1. **Canonical Harp synthesis.** The main entry is
   [the canonical DGM system article](../rsi/systems/dgm.md).
2. **Primary paper evidence.** The checked-in paper text is
   [the DGM capture](../../evidence/weng/text/dgm.txt), corresponding to
   arXiv `2505.22954v3`.
3. **Pinned implementation evidence.** The narrow source snapshot begins at
   [the DGM implementation evidence root](../../evidence/implementations/dgm/snapshot/README.md).

The paper supports method and author-reported result claims. The source
snapshot supports present-day implementation claims at its pinned commit.
Neither proves independent reproduction.

## Current technical judgment

DGM makes three useful advances:

- it treats the coding-agent implementation as the edited object;
- it lets accepted descendants participate in later self-modification; and
- it preserves multiple lineages instead of replacing one incumbent.

The main unresolved claim is causal. Better benchmark-solving ability is used
as a proxy for better future self-improvement ability. The paper does not
directly compare a parent and child on their ability to produce later accepted
children under a matched protected envelope.

## Packet status

- Paper: arXiv `2505.22954v3`, published as an ICLR 2026 conference paper in
  the captured source.
- Repository snapshot:
  `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.
- Independent benchmark reproduction: not performed.
- Packet authority: maintained Harp technical packet.

Use [maintenance](maintenance.md) before updating any result, implementation
claim, or source identity.
