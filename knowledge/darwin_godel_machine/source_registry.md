---
id: dgm-source-registry
title: DGM packet source registry
type: source-registry
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, sources, provenance]
confidence: high
canonical: ../../content/sources/source_registry.tsv
---

# DGM packet source registry

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## Source classes

| ID | Class | Location | What it can prove | What it cannot prove |
|---|---|---|---|---|
| DGM | Primary paper | [[evidence/weng/text/dgm.txt|Captured text]] and [arXiv](https://arxiv.org/abs/2505.22954) | Paper method, author-reported experiments, appendices, limitations | Independent reproduction or released-code correctness |
| DGM-REPO | Pinned implementation | [[evidence/implementations/dgm/snapshot/README|Snapshot root]] | Source behavior at commit `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2` | Historical experiment identity, runtime safety, benchmark reproduction |
| HARP-DGM | Canonical synthesis | [[knowledge/rsi/systems/dgm|DGM system article]] | Harp's maintained claim boundary and paper/code reconciliation | New primary evidence |
| HYPERAGENTS | Primary paper | [[evidence/weng/text/hyperagents.txt|Captured text]] and [arXiv](https://arxiv.org/abs/2603.19461) | DGM-H method, author-reported task and transfer experiments, stated outer-loop boundary | Independent reproduction, implementation behavior, or a complete editable-envelope result |
| HARP-RSI | Canonical framework | [[knowledge/rsi/chapters/recursive-improvement-loop|Recursive loop]], [[knowledge/rsi/concepts/system-state-and-notation|state notation]], [[knowledge/rsi/chapters/evaluation-promotion-containment|evaluation]] | Candidate/envelope vocabulary and RSI classification rules | DGM-specific empirical results |
| GODEL-MACHINE | Historical source record | [[content/sources/source_registry.tsv|Harp source registry]] | Identity and theoretical lineage at the admitted claim ceiling | Full proof details beyond the captured record |
| QD-2016 | Quality-diversity source | [[content/sources/source_registry.tsv|Harp source registry]] | Quality-diversity taxonomy admitted by Harp | That DGM implements a full QD algorithm |
| FUNSEARCH | Program-search comparison | [[content/sources/source_registry.tsv|Harp source registry]] | Predecessor mechanism and executable-evaluator comparison | Self-referential coding-agent improvement |
| ALPHAEVOLVE | Comparison system | [[knowledge/rsi/systems/alphaevolve|Canonical article]] | Program evolution against executable evaluators | DGM's agent-lineage mechanism |
| ADAS | Comparison system | [[knowledge/rsi/systems/adas|Canonical article]] | Fixed meta-agent search over agent programs | DGM's evolving modifier claim |
| STOP | Comparison system | [[knowledge/rsi/systems/stop|Canonical article]] | Recursive improver-program optimization | DGM's branching archive result |

## DGM: ICLR 2026 paper

- Class: `primary paper`
- Title: Darwin Gödel Machine: Open-Ended Evolution of Self-Improving Agents
- Artifact: [[evidence/weng/text/dgm.txt|Captured paper text]]
- Public page: [ICLR 2026 poster](https://iclr.cc/virtual/2026/poster/10007327)
- Stability: `pinned`
- Immutable identity: `arXiv:2505.22954v3`
- Digest record: [[evidence/weng/receipts/dgm.tsv|DGM acquisition receipt]]
- Publication status: The captured v3 text identifies the work as an ICLR 2026
  conference paper.
- Semantic locators: abstract, numbered sections, figures, tables, algorithms,
  and appendices.
- Can prove: Paper wording, method, reported experiments, limitations, cost
  estimates, model assignments, and author-described safety measures.
- Cannot prove: Independent reproduction, behavior of the released source
  snapshot, or the historical code revision used for every experiment.

## DGM-REPO: pinned implementation

- Class: `pinned implementation`
- Title: Released DGM implementation snapshot
- Artifact: [[evidence/implementations/dgm/snapshot/README|Snapshot entrypoint]]
- Stability: `pinned`
- Immutable identity: `git:a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`
- Revision record: [[evidence/implementations/dgm/REVISION|REVISION]]
- Remote record: [[evidence/implementations/dgm/REMOTE|REMOTE]]
- License record: [[evidence/implementations/dgm/LICENSE_STATUS|LICENSE_STATUS]]
- Semantic locators: captured file path plus physical line anchor.
- Can prove: Source behavior of the captured files at the pinned revision.
- Cannot prove: Historical experiment identity, runtime behavior not executed
  here, benchmark reproduction, or files intentionally omitted from the narrow
  snapshot.

## HYPERAGENTS: DGM-H successor paper

- Class: `primary paper`
- Title: Hyperagents
- Artifact: [[evidence/weng/text/hyperagents.txt|Captured paper text]]
- Public page: [arXiv abstract](https://arxiv.org/abs/2603.19461)
- Stability: `pinned`
- Immutable identity: `arXiv:2603.19461v1`
- Digest record: [[evidence/weng/receipts/hyperagents.tsv|Hyperagents acquisition receipt]]
- Publication status: The captured version is a preprint; a peer-reviewed venue
  has not been verified in this packet.
- Semantic locators: abstract, §3–§6, Appendix E.5, and the conclusion.
- Can prove: The paper's DGM-H construction, its editable task-plus-meta-agent
  definition, author-reported task/transfer outcomes, and stated limits of the
  main outer loop.
- Cannot prove: Behavior of the unpinned upstream repository, independent
  reproduction, fully editable evaluation and selection, or a matched
  successor-productivity comparison.

## HARP-DGM: canonical synthesis

- Class: `canonical Harp synthesis`
- Title: DGM system article
- Artifact: [[knowledge/rsi/systems/dgm|Canonical DGM article]]
- Stability: `repository versioned`
- Immutable identity: The Harp commit containing the cited statement.
- Semantic locators: article heading.
- Can prove: Harp's maintained terminology, reconciliations, and claim ceiling.
- Cannot prove: New facts about the paper, source behavior, or runtime results.

## HARP-RSI: canonical evaluation framework

- Class: `canonical Harp framework`
- Title: Recursive-improvement and evaluation framework
- Artifact: [[knowledge/rsi/chapters/recursive-improvement-loop|Recursive improvement loop]]
- Stability: `repository versioned`
- Immutable identity: The Harp commit containing the cited framework.
- Semantic locators: chapter or concept heading.
- Can prove: Harp's definitions for candidate state, protected envelope,
  successor improvement, root-tree accounting, and promotion.
- Cannot prove: DGM-specific implementation behavior or empirical outcomes.

## Immutable identities

### Paper

- arXiv ID: `2505.22954`
- captured version: `v3`
- capture digest and registry status:
  [[content/sources/source_registry.tsv|canonical source registry]]
- paper text:
  [[evidence/weng/text/dgm.txt|evidence/weng/text/dgm.txt]]

### Repository

- remote: `https://github.com/jennyzzt/dgm.git`
- revision: `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`
- revision receipt:
  [[evidence/implementations/dgm/REVISION|evidence/implementations/dgm/REVISION]]
- file digest manifest:
  [[evidence/implementations/manifest.tsv|implementation manifest]]
- upstream license:
  [[evidence/implementations/dgm/LICENSE|DGM LICENSE]]

## Primary paper locators

| Topic | Locator |
|---|---|
| Main motivation and headline results | Abstract, §1 |
| Related work | §2, Appendix B |
| Self-referential agent and archive | §3 |
| Experiment setup | §4.1 |
| Benchmarks and staged evaluation | §4.2 |
| Main baselines | §4.3 |
| Results and transfer | §4.4 |
| Safety | §5, Ethics Statement, Appendix I |
| Limitations | §6 |
| Parent selection | Appendix C.2 |
| Diagnostic prompt | Appendix C.3 |
| Pseudocode | Appendix C.4 |
| Models and temperature | Appendix D.1 |
| Cost and task subsets | Appendix E |
| Best-agent lineages | Appendix F |
| Different implementations of one feature | Appendix G |
| Hallucination and objective hacking | Appendix H |
| Future work | Appendix J |

## Implementation locators

| Topic | Snapshot locator |
|---|---|
| Archive initialization and parent selection | [[evidence/implementations/dgm/snapshot/DGM_outer.py|DGM_outer.py]] |
| Self-modification container and evaluation handoff | [[evidence/implementations/dgm/snapshot/self_improve_step.py|self_improve_step.py]] |
| Seed SWE-bench coding agent | [[evidence/implementations/dgm/snapshot/coding_agent.py|coding_agent.py]] |
| Seed Polyglot coding agent | [[evidence/implementations/dgm/snapshot/coding_agent_polyglot.py|coding_agent_polyglot.py]] |
| Model and tool loop | [[evidence/implementations/dgm/snapshot/llm_withtools.py|llm_withtools.py]] |
| Improvement diagnosis prompt | [[evidence/implementations/dgm/snapshot/prompts/self_improvement_prompt.py|self_improvement_prompt.py]] |
| Viability and patch lineage | [[evidence/implementations/dgm/snapshot/utils/evo_utils.py|evo_utils.py]] |
| Seed editor | [[evidence/implementations/dgm/snapshot/tools/edit.py|edit.py]] |
| Seed shell | [[evidence/implementations/dgm/snapshot/tools/bash.py|bash.py]] |
| SWE-bench execution | [[evidence/implementations/dgm/snapshot/swe_bench/harness.py|swe_bench/harness.py]] |
| Test-edit filtering and reports | [[evidence/implementations/dgm/snapshot/swe_bench/report.py|swe_bench/report.py]] |
| Polyglot execution | [[evidence/implementations/dgm/snapshot/polyglot/harness.py|polyglot/harness.py]] |

## Evidence rules for this packet

1. A benchmark number must cite DGM and state its task scope.
2. A code-behavior claim must cite DGM-REPO and the pinned revision.
3. A statement about what the result means for RSI must be labeled
   `INFERENCE` unless the source directly measures it.
4. Missing runtime evidence must be labeled `MISSING`.
5. Paper claims and released-code behavior may differ. Record both rather than
   silently choosing one.
6. The packet does not treat its own prose as evidence.
7. Main prose links to the exact heading-based entry in
   [[knowledge/darwin_godel_machine/claim_evidence_crosswalk|the claim ledger]]; the ledger then links to the
   underlying paper or source artifact.

## Reproduction status

Harp has not:

- rerun the 80-iteration searches;
- replayed the external experiment-log archive;
- reproduced the USD cost estimates;
- verified API model snapshots;
- independently scored the final agents; or
- audited Docker isolation under hostile generated code.

Back to the [[knowledge/darwin_godel_machine/darwin_godel_machine_index|DGM index]].
