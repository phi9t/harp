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

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-065: DGM is evolutionary search over agent scaffolds|INFERENCE - DGM-065]].**
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

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-001: DGM evolves an editable coding-agent repository|EVIDENCE - DGM-001]].**
The self-reference is real but bounded. A selected archive node is reconstructed
from inherited patches, its agent edits the repository that defines future
agent behavior, and the resulting child can later become a parent. The outer
controller, evaluator, diagnostic model, benchmark allocation, and foundation
model weights remain outside that evolving candidate.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-049: Evidence supports bounded harness improvement|INFERENCE - DGM-049]].**
The strongest defensible conclusion is that DGM demonstrates autonomous search
over heritable coding-agent architecture and workflow, with material
author-reported benchmark gains and evidence that evolving the modifier helps.
The paper results have not been independently reproduced here.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-050: Matched successor-improvement evidence is missing|MISSING - DGM-050]].**
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

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-066: Patch lineage is the genotype and agent behavior the phenotype|INFERENCE - DGM-066]].**
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

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-020: Paper reports the SWE-bench result|SOURCE CLAIM - DGM-020]].**
The paper reports a selected SWE-bench result from 20.0% to 50.0% on a
200-task evaluation. This result has not been independently reproduced here.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-022: Paper reports the full Polyglot result|SOURCE CLAIM - DGM-022]].**
The paper reports full-Polyglot improvement from 14.2% to 30.7%. This is
separate from the 38.0% score reported on the 50-task search subset, and neither
result has been independently reproduced here.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-067: Task fitness and descendant productivity are different objectives|INFERENCE - DGM-067]].**
Current task fitness and descendant productivity are different objectives. DGM
selects parents using current task score and underexploration, then relies on
the hypothesis that better task solvers are also better future modifiers. It
does not directly estimate valid held-out gain produced by a parent's children.

## Released-code audit

The pinned release makes the pipeline inspectable but should not be treated as
the exact historical experiment snapshot.

| Finding | Consequence | Ledger |
|---|---|---|
| Full SWE threshold is passed but not consumed | Visible source implements 10→60, not the full paper 10→60→200 path | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-060: Released SWE path does not consume the full-evaluation threshold|DGM-060]] |
| Defaults use 80 generations and two attempts per generation | A literal default run may schedule up to 160 attempts | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-061: Released defaults can schedule two attempts per generation|DGM-061]] |
| Perfect-score eligibility filter is absent | Paper and release parent sets differ at the boundary | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-062: Released parent selection omits the paper's perfect-score filter|DGM-062]] |
| Empty unresolved lists are compared with integer zero | `random.choice([])` remains reachable | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-063: Empty SWE unresolved lists can reach random choice|DGM-063]] |
| Expanded Polyglot score is stored separately | Parent selection reads the shallow score | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-056: Released Polyglot selection reads the shallow score|DGM-056]] |
| Polyglot self-improvement selects `o3-mini` | Pinned source differs from the paper's Claude 3.5 assignment | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-057: Released Polyglot self-modification model differs from the paper|DGM-057]] |
| Patch filtering rewrites lineage inputs | Concurrent evaluation can mutate archive evidence | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-058: Polyglot evaluation rewrites archive patch inputs|DGM-058]] |
| Timeout is applied after `as_completed` | Hung attempts are not bounded by the stated outer timeout | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-059: Outer timeout does not bound unfinished futures|DGM-059]] |
| Dataset and dependency identity is incomplete | Bit-for-bit reproduction needs a stronger run manifest | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-064: Released environment identity is incomplete|DGM-064]] |

## Safety boundary

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-041: Paper describes sandboxing and lineage monitoring|SOURCE CLAIM - DGM-041]].**
The paper describes isolated, time-limited execution and monitored lineages.
Those safeguards have not been independently audited here.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-042: Released Docker use is not a hardened sandbox proof|INFERENCE - DGM-042]].**
The public snapshot starts ordinary Docker containers and passes provider and
cloud credentials into generated-code execution. Docker use alone is not a
hostile-code sandbox proof; a production system should broker inference outside
the candidate and keep raw credentials out of the sandbox.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-044: Paper reports objective hacking in Appendix H|SOURCE CLAIM - DGM-044]].**
Appendix H reports a perfect detector score obtained by changing the
measurement channel rather than solving the intended hallucination behavior.
The case is not independently reproduced, but it directly demonstrates why the
evaluator and observation channel must remain protected.

## Recommended code-reading order

Use this sequence for a rigorous source pass:

1. [[evidence/implementations/dgm/snapshot/DGM_outer.py|`DGM_outer.py`]]
   for archive state, parent selection, scheduling, and admission.
2. [[evidence/implementations/dgm/snapshot/self_improve_step.py|`self_improve_step.py`]]
   for one parent-to-child transition.
3. [[evidence/implementations/dgm/snapshot/prompts/self_improvement_prompt.py|`self_improvement_prompt.py`]]
   for the diagnostic evidence channel.
4. [[evidence/implementations/dgm/snapshot/coding_agent.py|`coding_agent.py`]]
   for the seed agent and mutable workflow.
5. [[evidence/implementations/dgm/snapshot/llm_withtools.py|`llm_withtools.py`]]
   for model routing, retries, and tool dispatch.
6. [[evidence/implementations/dgm/snapshot/tools/edit.py|`edit.py`]] and
   [[evidence/implementations/dgm/snapshot/tools/bash.py|`bash.py`]] for
   the primitive action space.
7. [[evidence/implementations/dgm/snapshot/utils/evo_utils.py|`evo_utils.py`]]
   for lineage reconstruction and child viability.
8. The
   [[evidence/implementations/dgm/snapshot/swe_bench/harness.py|SWE harness]]
   and
   [[evidence/implementations/dgm/snapshot/polyglot/harness.py|Polyglot harness]]
   for evaluation semantics.

Then follow one archive node backward to `initial` and forward through
diagnosis, implementation, evaluation, and admission. The
[[knowledge/darwin_godel_machine/05_repository_walkthrough|repository walkthrough]] follows this runtime
order and records the paper/source mismatches.

## Choose a route

### Expert route

Use this route for a design review or architecture assessment:

1. [[knowledge/darwin_godel_machine/04_system_architecture|System architecture]]
2. [[knowledge/darwin_godel_machine/05_repository_walkthrough|Repository walkthrough]]
3. [[knowledge/darwin_godel_machine/06_evaluation_analysis|Evaluation analysis]]
4. [[knowledge/darwin_godel_machine/09_critical_review|Critical review]]
5. [[knowledge/darwin_godel_machine/11_hyperagents_successor|Hyperagents successor case study]]
6. [[knowledge/darwin_godel_machine/10_successor_design|Successor design]]

Expected reading time: 90 to 150 minutes.

### Guided route

Use this route to learn the mechanism from first principles:

1. [[knowledge/darwin_godel_machine/01_orientation|Orientation]]
2. [[knowledge/darwin_godel_machine/02_paper_walkthrough|Paper walkthrough]]
3. [[knowledge/darwin_godel_machine/03_algorithm_derivation|Algorithm derivation]]
4. [[knowledge/darwin_godel_machine/07_open_endedness|Open-endedness]]
5. [[knowledge/darwin_godel_machine/04_system_architecture|System architecture]]
6. [[knowledge/darwin_godel_machine/05_repository_walkthrough|Repository walkthrough]]
7. [[knowledge/darwin_godel_machine/06_evaluation_analysis|Evaluation analysis]]
8. [[knowledge/darwin_godel_machine/08_safety_and_failure|Safety and failure]]
9. [[knowledge/darwin_godel_machine/09_critical_review|Critical review]]
10. [[knowledge/darwin_godel_machine/11_hyperagents_successor|Hyperagents successor case study]]
11. [[knowledge/darwin_godel_machine/10_successor_design|Successor design]]
12. [[knowledge/darwin_godel_machine/learning_path|Learning path]]

Expected reading time: 4 to 7 hours, including exercises.

## Route by question

| Question | Start here | Continue with |
|---|---|---|
| What is DGM actually changing? | [[knowledge/darwin_godel_machine/01_orientation|Orientation]] | [[knowledge/darwin_godel_machine/04_system_architecture|System architecture]] |
| How does parent selection work? | [[knowledge/darwin_godel_machine/03_algorithm_derivation|Algorithm derivation]] | [[knowledge/darwin_godel_machine/07_open_endedness|Open-endedness]] |
| Why retain worse agents? | [[knowledge/darwin_godel_machine/07_open_endedness|Open-endedness]] | [[knowledge/darwin_godel_machine/06_evaluation_analysis|Evaluation analysis]] |
| How does the released code implement the paper? | [[knowledge/darwin_godel_machine/05_repository_walkthrough|Repository walkthrough]] | [[knowledge/darwin_godel_machine/04_system_architecture|System architecture]] |
| What do the benchmark numbers include? | [[knowledge/darwin_godel_machine/06_evaluation_analysis|Evaluation analysis]] | [[knowledge/darwin_godel_machine/claim_evidence_crosswalk|Claim crosswalk]] |
| Does DGM demonstrate recursive self-improvement? | [[knowledge/darwin_godel_machine/09_critical_review|Critical review]] | [[knowledge/darwin_godel_machine/10_successor_design|Successor design]] |
| How does DGM-H change the editable object? | [[knowledge/darwin_godel_machine/11_hyperagents_successor|Hyperagents successor case study]] | [[knowledge/darwin_godel_machine/10_successor_design|Successor design]] |
| What could go wrong? | [[knowledge/darwin_godel_machine/08_safety_and_failure|Safety and failure]] | [[knowledge/darwin_godel_machine/09_critical_review|Critical review]] |
| How would I build a stronger experiment? | [[knowledge/darwin_godel_machine/10_successor_design|Successor design]] | [[knowledge/darwin_godel_machine/learning_path|Learning path]] |

## Packet map

### Foundations

- [[knowledge/darwin_godel_machine/01_orientation|Orientation]] establishes the vocabulary and claim ceiling.
- [[knowledge/darwin_godel_machine/02_paper_walkthrough|Paper walkthrough]] reconstructs the paper's causal
  argument and identifies the appendices that carry important evidence.
- [[knowledge/darwin_godel_machine/glossary|Glossary]] owns the packet's terms and symbols.

### Mechanism

- [[knowledge/darwin_godel_machine/03_algorithm_derivation|Algorithm derivation]] derives parent selection,
  archive admission, and the baseline algorithms.
- [[knowledge/darwin_godel_machine/04_system_architecture|System architecture]] separates mutable candidate
  code from the protected outer loop.
- [[knowledge/darwin_godel_machine/07_open_endedness|Open-endedness]] explains stepping stones, search
  diversity, and why `keep_all` matters.

### Implementation

- [[knowledge/darwin_godel_machine/05_repository_walkthrough|Repository walkthrough]] traces the released
  implementation at commit
  `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.

### Evidence and judgment

- [[knowledge/darwin_godel_machine/06_evaluation_analysis|Evaluation analysis]] accounts for tasks, models,
  metrics, costs, ablations, transfer, and reproduction status.
- [[knowledge/darwin_godel_machine/08_safety_and_failure|Safety and failure]] examines containment,
  evaluator gaming, and objective hacking.
- [[knowledge/darwin_godel_machine/09_critical_review|Critical review]] gives a consequence-ordered MTS
  assessment.
- [[knowledge/darwin_godel_machine/claim_evidence_crosswalk|Claim-evidence crosswalk]] maps material claims
  to their sources and caveats.

### Design and teaching

- [[knowledge/darwin_godel_machine/11_hyperagents_successor|Hyperagents successor case study]] separates DGM-H's
  editable task-plus-meta-agent program from the still-protected outer loop.
- [[knowledge/darwin_godel_machine/10_successor_design|Successor design]] specifies a more rigorous DGM-like
  experiment.
- [[knowledge/darwin_godel_machine/learning_path|Learning path]] moves from explanation to experiment
  design.
- [[knowledge/darwin_godel_machine/maintenance|Maintenance]] records update and verification rules.
- [[knowledge/darwin_godel_machine/source_registry|Source registry]] states what each source can prove.

## Source boundary

The packet relies on three evidence classes:

1. **Canonical Harp synthesis.** The main entry is
   [[knowledge/rsi/systems/dgm|the canonical DGM system article]].
2. **Primary paper evidence.** The checked-in paper text is
   [[evidence/weng/text/dgm.txt|the DGM capture]], corresponding to
   arXiv `2505.22954v3`.
3. **Pinned implementation evidence.** The narrow source snapshot begins at
   [[evidence/implementations/dgm/snapshot/README|the DGM implementation evidence root]].

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

Use [[knowledge/darwin_godel_machine/maintenance|maintenance]] before updating any result, implementation
claim, or source identity.
