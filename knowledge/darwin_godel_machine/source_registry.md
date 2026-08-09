---
id: dgm-source-registry
title: DGM packet source registry
type: source-registry
status: active
created: 2026-08-08
updated: 2026-08-08
tags: [darwin-godel-machine, sources, provenance]
confidence: high
canonical: ../../content/sources/source_registry.tsv
---

# DGM packet source registry

> This file is a learning projection. Canonical claims live under `content/`;
> primary-source captures and pinned implementation files live under
> `evidence/`.

## Source classes

| ID | Class | Location | What it can prove | What it cannot prove |
|---|---|---|---|---|
| DGM | Primary paper | [Captured text](../../evidence/weng/text/dgm.txt) and [arXiv](https://arxiv.org/abs/2505.22954) | Paper method, author-reported experiments, appendices, limitations | Independent reproduction or released-code correctness |
| DGM-REPO | Pinned implementation | [Snapshot root](../../evidence/implementations/dgm/snapshot/README.md) | Source behavior at commit `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2` | Historical experiment identity, runtime safety, benchmark reproduction |
| HARP-DGM | Canonical synthesis | [DGM system article](../../content/systems/dgm.md) | Harp's maintained claim boundary and paper/code reconciliation | New primary evidence |
| HARP-RSI | Canonical framework | [Recursive loop](../../content/chapters/recursive-improvement-loop.md), [state notation](../../content/concepts/system-state-and-notation.md), [evaluation](../../content/chapters/evaluation-promotion-containment.md) | Candidate/envelope vocabulary and RSI classification rules | DGM-specific empirical results |
| GODEL-MACHINE | Historical source record | [Harp source registry](../../content/sources/source_registry.tsv) | Identity and theoretical lineage at the admitted claim ceiling | Full proof details beyond the captured record |
| QD-2016 | Quality-diversity source | [Harp source registry](../../content/sources/source_registry.tsv) | Quality-diversity taxonomy admitted by Harp | That DGM implements a full QD algorithm |
| FUNSEARCH | Program-search comparison | [Harp source registry](../../content/sources/source_registry.tsv) | Predecessor mechanism and executable-evaluator comparison | Self-referential coding-agent improvement |
| ALPHAEVOLVE | Comparison system | [Canonical article](../../content/systems/alphaevolve.md) | Program evolution against executable evaluators | DGM's agent-lineage mechanism |
| ADAS | Comparison system | [Canonical article](../../content/systems/adas.md) | Fixed meta-agent search over agent programs | DGM's evolving modifier claim |
| STOP | Comparison system | [Canonical article](../../content/systems/stop.md) | Recursive improver-program optimization | DGM's branching archive result |

## Immutable identities

### Paper

- arXiv ID: `2505.22954`
- captured version: `v3`
- capture digest and registry status:
  [canonical source registry](../../content/sources/source_registry.tsv)
- paper text:
  [evidence/weng/text/dgm.txt](../../evidence/weng/text/dgm.txt)

### Repository

- remote: `https://github.com/jennyzzt/dgm.git`
- revision: `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`
- revision receipt:
  [evidence/implementations/dgm/REVISION](../../evidence/implementations/dgm/REVISION)
- file digest manifest:
  [implementation manifest](../../evidence/implementations/manifest.tsv)
- upstream license:
  [DGM LICENSE](../../evidence/implementations/dgm/LICENSE)

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
| Archive initialization and parent selection | [DGM_outer.py](../../evidence/implementations/dgm/snapshot/DGM_outer.py) |
| Self-modification container and evaluation handoff | [self_improve_step.py](../../evidence/implementations/dgm/snapshot/self_improve_step.py) |
| Seed SWE-bench coding agent | [coding_agent.py](../../evidence/implementations/dgm/snapshot/coding_agent.py) |
| Seed Polyglot coding agent | [coding_agent_polyglot.py](../../evidence/implementations/dgm/snapshot/coding_agent_polyglot.py) |
| Model and tool loop | [llm_withtools.py](../../evidence/implementations/dgm/snapshot/llm_withtools.py) |
| Improvement diagnosis prompt | [self_improvement_prompt.py](../../evidence/implementations/dgm/snapshot/prompts/self_improvement_prompt.py) |
| Viability and patch lineage | [evo_utils.py](../../evidence/implementations/dgm/snapshot/utils/evo_utils.py) |
| Seed editor | [edit.py](../../evidence/implementations/dgm/snapshot/tools/edit.py) |
| Seed shell | [bash.py](../../evidence/implementations/dgm/snapshot/tools/bash.py) |
| SWE-bench execution | [swe_bench/harness.py](../../evidence/implementations/dgm/snapshot/swe_bench/harness.py) |
| Test-edit filtering and reports | [swe_bench/report.py](../../evidence/implementations/dgm/snapshot/swe_bench/report.py) |
| Polyglot execution | [polyglot/harness.py](../../evidence/implementations/dgm/snapshot/polyglot/harness.py) |

## Evidence rules for this packet

1. A benchmark number must cite DGM and state its task scope.
2. A code-behavior claim must cite DGM-REPO and the pinned revision.
3. A statement about what the result means for RSI must be labeled
   `INFERENCE` unless the source directly measures it.
4. Missing runtime evidence must be labeled `MISSING`.
5. Paper claims and released-code behavior may differ. Record both rather than
   silently choosing one.
6. The packet does not treat its own prose as evidence.

## Reproduction status

Harp has not:

- rerun the 80-iteration searches;
- replayed the external experiment-log archive;
- reproduced the USD cost estimates;
- verified API model snapshots;
- independently scored the final agents; or
- audited Docker isolation under hostile generated code.

Back to the [DGM index](darwin_godel_machine_index.md).
