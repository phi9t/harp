---
id: dgm-evaluation-analysis
title: DGM evaluation analysis
type: deep-dive
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, evaluation, benchmarks, ablations, cost]
confidence: high
canonical: ../../content/systems/dgm.md
---

# DGM evaluation analysis

> This file is a learning projection. Canonical claims live under `content/`;
> primary-source captures and pinned implementation files live under
> `evidence/`.

## Learning outcomes

After this chapter, you should be able to:

- reconcile task scopes, metrics, model roles, and reported values;
- distinguish paper assignments from pinned-source behavior;
- identify adaptive-selection and winner's-curse risks;
- explain the released Polyglot shallow-score mismatch; and
- state which claims remain author-reported rather than reproduced.

## Claim boundary

**[EVIDENCE - DGM-071](claim_evidence_crosswalk.md#dgm-071-captured-paper-identifies-iclr-2026-publication).**
The pinned paper capture identifies the work as published at ICLR 2026.

All measurements below are author-reported in [DGM](../../evidence/weng/text/dgm.txt).
Harp has not rerun the 80-iteration searches, independently scored the final
agents, replayed the external experiment logs, reproduced the cost estimates,
or audited the historical model API environment.

Every table therefore uses:

```text
Reproduction status: not independently reproduced by Harp
```

unless stated otherwise.

## Benchmark shapes

### SWE-bench Verified

The paper uses the human-filtered SWE-bench Verified set. Tasks are Python
repository issues and may require coordinated edits across multiple files.

The search uses:

- 10 tasks for initial functionality;
- 50 additional tasks for a 60-task estimate; and
- 140 additional tasks for selected candidates, yielding a 200-task result.

The 60-task subset is based on SWE-bench-verified-mini. A candidate proceeds to
200 tasks when its 60-task success exceeds 40% and it ranks among the top two
archive performers at that point.

**[EVIDENCE - DGM-060](claim_evidence_crosswalk.md#dgm-060-released-swe-path-does-not-consume-the-full-evaluation-threshold).**
That is the paper protocol. The pinned release passes a full-evaluation
threshold and loads the `big` subset but never consumes either in the captured
SWE harness dispatch, so the visible source implements only the initial and
medium stages.

### Polyglot

Polyglot covers C++, Go, Java, JavaScript, Python, Rust, and other language
tasks. The tasks are generally smaller and often center on implementing one
solution file while inspecting surrounding code.

The search uses:

- 10 initial tasks; and
- 50 additional tasks after the threshold.

The final selected agent is also evaluated on the full benchmark.

### Metric boundary

The paper reports pass@1 for Polyglot: the agent does not receive ground-truth
test feedback and retry once. The public leaderboard setup referenced by the
paper uses pass@2, so leaderboard values are not directly interchangeable with
the paper's DGM values.

## Model assignments

| Role | SWE-bench search | Polyglot search | Source locator | Reproduction status |
|---|---|---|---|---|
| Self-modification coding model | Claude 3.5 Sonnet (New) | Claude 3.5 Sonnet (New) | DGM §4.1, Appendix D.1 | Not independently reproduced |
| Task-evaluation coding model | Claude 3.5 Sonnet (New) | `o3-mini` | DGM §4.1, Appendix D.1 | Not independently reproduced |
| Diagnostic model in released source | `o1-2024-12-17` | `o1-2024-12-17` | DGM Appendix C.3; DGM-REPO `self_improve_step.py` | Source-inspected, not run |
| Sampling temperature | 1.0 | 1.0 | DGM Appendix D.1 | Not independently reproduced |

The model assignment is part of the treatment. A candidate repository is not
the complete agent without the model that executes it.

### Paper/source model mismatch

The pinned Polyglot source does not implement the paper's self-modification
model row. `self_improve_step.py` swaps `coding_agent_polyglot.py` into the
common path and invokes it with `--self_improve`;
`coding_agent_polyglot.py` then selects `OPENAI_MODEL`, or `o3-mini`.

| Role | Paper assignment | Pinned-source behavior | Historical provenance |
|---|---|---|---|
| Polyglot self-modification | Claude 3.5 Sonnet (New) | `o3-mini` | Unresolved without the exact experiment revision |

Do not infer which implementation produced the paper results from the current
repository snapshot alone.

### Released Polyglot score mismatch

The pinned runner stores the initial Polyglot aggregate in
`overall_performance` and the expanded result in
`overall_performance_deep`. Parent selection and the viability predicate read
only `overall_performance`.

The paper reports a 50-task search score. The released code, as captured,
continues selecting parents from the shallow aggregate. This is a material
paper-to-code mismatch and prevents treating the snapshot as a faithful
reproduction recipe.

## Parallelism and iteration accounting

The paper says DGM runs for 80 iterations and describes one generated agent per
iteration, while also stating two self-modification attempts run in parallel
for SWE-bench and four for Polyglot. The released outer loop exposes
`selfimprove_size` and launches that many attempts per generation.

This terminology makes three counts worth preserving separately:

- outer generations;
- self-modification attempts;
- functioning children admitted to the archive.

The paper's headline "80 iterations" should not be read as exactly 80 model
calls or 80 benchmark evaluations.

## Headline task results

| Benchmark | Task scope | Task count | Metric | Base | Best DGM | Task-evaluation model | Source locator | Reproduction status |
|---|---|---:|---|---:|---:|---|---|---|
| SWE-bench Verified | Paper subset | 200 | Solved-task rate | 20.0% | 50.0% | Claude 3.5 Sonnet (New) | DGM Abstract, §§4.2–4.4 | Not independently reproduced |
| Polyglot | Search subset | 50 | pass@1 solved-task rate | 14.0% | 38.0% | `o3-mini` | DGM §§4.2–4.4 | Not independently reproduced |
| Polyglot | Full benchmark | Full benchmark | pass@1 solved-task rate | 14.2% | 30.7% | `o3-mini` | DGM Abstract, §4.4 | Not independently reproduced |

The Polyglot 38.0% and 30.7% values answer different questions. The former is
the selected search subset; the latter is the full-benchmark estimate.

## Ablations

| Method | Benchmark | Task count | Metric | Score | Evaluation model | What changes | Source locator | Reproduction status |
|---|---|---:|---|---:|---|---|---|---|
| DGM | SWE-bench Verified | 200 | Solved-task rate | 50.0% | Claude 3.5 Sonnet (New) | Full self-modifying archive search | DGM Appendix A.3, Table 1 | Not independently reproduced |
| DGM | Polyglot search subset | 50 | pass@1 solved-task rate | 38.0% | `o3-mini` | Full self-modifying archive search | DGM Appendix A.3, Table 1 | Not independently reproduced |
| No open-ended exploration | SWE-bench Verified | 200 | Solved-task rate | 23.0% | Claude 3.5 Sonnet (New) | Keep latest functioning lineage | DGM §4.3, Appendix A.1/A.3 | Not independently reproduced |
| No open-ended exploration | Polyglot search subset | 50 | pass@1 solved-task rate | 14.0% | `o3-mini` | Keep latest functioning lineage | DGM §4.3, Appendix A.1/A.3 | Not independently reproduced |
| No self-improve | SWE-bench Verified | 200 | Solved-task rate | 39.0% | Claude 3.5 Sonnet (New) | Fixed base modifier generates candidates | DGM §4.3, Appendix A.1/A.3 | Not independently reproduced |
| No self-improve | Polyglot search subset | 50 | pass@1 solved-task rate | 28.0% | `o3-mini` | Fixed base modifier generates candidates | DGM §4.3, Appendix A.1/A.3 | Not independently reproduced |
| Greedy parent | SWE-bench Verified | 200 | Solved-task rate | 39.7% | Claude 3.5 Sonnet (New) | Always branch from current best | DGM Appendix A.3 | Not independently reproduced |
| Greedy parent | Polyglot search subset | 50 | pass@1 solved-task rate | 30.0% | `o3-mini` | Always branch from current best | DGM Appendix A.3 | Not independently reproduced |

These comparisons support the value of the tested self-modifier inheritance and
archive-selection design in the reported setup. They do not establish that
every open-ended policy beats every greedy or population-search alternative.

## Functioning-child rate

| Method | Benchmark | Population | Metric | Result | Model setup | Source locator | Reproduction status |
|---|---|---|---|---:|---|---|---|
| DGM | SWE-bench Verified search | Generated descendants | Basic code-editing function rate | 51.3% | Paper SWE-bench search setup | DGM Appendix A.4, Table 2 | Not independently reproduced |
| No open-ended exploration | SWE-bench Verified search | Generated descendants | Basic code-editing function rate | 32.5% | Paper SWE-bench search setup | DGM Appendix A.4, Table 2 | Not independently reproduced |
| No self-improve | SWE-bench Verified search | Generated descendants | Basic code-editing function rate | 32.5% | Paper SWE-bench search setup | DGM Appendix A.4, Table 2 | Not independently reproduced |

This metric is closer to improvement-process health than downstream task score,
but it is still incomplete. It measures whether children remain basic code
editors, not the magnitude or quality of improvements they later produce.

## Stability

Appendix A.4 reports three Polyglot DGM runs with:

| Benchmark scope | Task count | Metric | Model | Mean | Standard deviation | Number of runs | Source locator | Reproduction status |
|---|---:|---|---|---:|---:|---:|---|---|
| Polyglot search setup | 50-task search subset | pass@1 accuracy | `o3-mini` evaluation | 40.7% | 2.3 percentage points | 3 | DGM Appendix A.4 | Not independently reproduced |

Three runs are useful but weak for characterizing a stochastic adaptive search.
The paper does not provide enough independent runs to estimate tail risk,
archive collapse frequency, or the probability of failing to beat the
baselines.

## Model transfer

### SWE-bench

The agent was searched with Claude 3.5 Sonnet and re-executed with other
models on the 200-task subset.

| Benchmark | Task count | Metric | Execution model | Base | DGM-discovered agent | Source locator | Reproduction status |
|---|---:|---|---|---:|---:|---|---|
| SWE-bench Verified paper subset | 200 | Solved-task rate | `o3-mini` | 23.0% | 33.0% | DGM §4.4, Figure 4 | Not independently reproduced |
| SWE-bench Verified paper subset | 200 | Solved-task rate | Claude 3.5 Sonnet (New) | 20.0% | 50.0% | DGM §4.4, Figure 4 | Not independently reproduced |
| SWE-bench Verified paper subset | 200 | Solved-task rate | Claude 3.7 Sonnet | 19.0% | 59.5% in prose; 59.0% in Figure 4 label | DGM §4.4, Figure 4 | Not independently reproduced; source inconsistency retained |

The prose/figure mismatch is small but material to exact reporting. This packet
does not choose one silently.

### Polyglot

The agent was searched with `o3-mini` and re-executed with other models on the
full benchmark.

| Benchmark | Task count | Metric | Execution model | Base | DGM-discovered agent | Source locator | Reproduction status |
|---|---|---|---|---:|---:|---|---|
| Full Polyglot | Full benchmark | pass@1 solved-task rate | `o3-mini` | 14.2% | 30.7% | DGM §4.4, Appendix A.2 | Not independently reproduced |
| Full Polyglot | Full benchmark | pass@1 solved-task rate | Claude 3.5 Sonnet (New) | 32.0% | 33.3% | DGM Appendix A.2 | Not independently reproduced |
| Full Polyglot | Full benchmark | pass@1 solved-task rate | Claude 3.7 Sonnet | 35.6% | 36.8% | DGM Appendix A.2 | Not independently reproduced |

The transfer lift is much smaller on the stronger Claude Polyglot baselines.
That is consistent with some evolved harness features being most useful to the
search-time model and task distribution.

## Cross-benchmark transfer

| Discovered on | Evaluated on | Task count | Metric | Evaluation model | Base | Transferred agent | Source locator | Reproduction status |
|---|---|---:|---|---|---:|---:|---|---|
| SWE-bench | Polyglot | Full benchmark | pass@1 solved-task rate | `o3-mini` | 14.2% | 28.9% | DGM §4.4, Figure 4 | Not independently reproduced |
| Polyglot | SWE-bench Verified | 200 | Solved-task rate | Claude 3.5 Sonnet (New) | 20.0% | 24.5% | DGM §4.4, Figure 4 | Not independently reproduced |

The paper calls the alternate benchmark held out from search. This is strong
evidence against a purely benchmark-local implementation. It does not rule out:

- common benchmark conventions;
- adaptation to the shared coding-agent task family;
- model-training contamination;
- selection on features broadly useful to these two benchmarks; or
- use of private SWE-bench evidence in the diagnostic stage.

## Cross-language transfer

Figure 4 reports Python and non-Python Polyglot slices for:

- the base agent;
- Aider;
- DGM searched on all languages; and
- DGM searched only on Python.

The plotted values are:

| Agent | Benchmark scope | Task count | Metric | Evaluation model | Python tasks | Non-Python tasks | Source locator | Reproduction status |
|---|---|---|---|---|---:|---:|---|---|
| Base agent | Full Polyglot language slices | Full benchmark | pass@1 solved-task rate | `o3-mini` | 8.8% | 15.2% | DGM §4.4, Figure 4 | Not independently reproduced |
| Aider | Full Polyglot language slices | Full benchmark | Paper-comparison solved-task rate | Paper comparison setup | 11.8% | 17.3% | DGM §4.4, Figure 4 | Not independently reproduced |
| DGM searched on all languages | Full Polyglot language slices | Full benchmark | pass@1 solved-task rate | `o3-mini` | 17.6% | 33.0% | DGM §4.4, Figure 4 | Not independently reproduced |
| DGM searched only on Python | Full Polyglot language slices | Full benchmark | pass@1 solved-task rate | `o3-mini` | 20.6% | 30.8% | DGM §4.4, Figure 4 | Not independently reproduced |

The Python-only search improves the Python slice more but transfers below the
all-language search on non-Python tasks. This supports partial cross-language
transfer, not language-invariant behavior.

## Handcrafted comparisons

### SWE-bench

Appendix E.4 reports:

- OpenHands + CodeAct v2.1 at 53.0% on the full SWE-bench Verified setting
  available at the paper's comparison date; and
- 51.0% on the same 200-task subset used for DGM.

DGM reports 50.0% on that 200-task subset. The comparison is close, but the
systems have different development histories, search budgets, and operational
contracts.

### Polyglot

The paper compares with Aider under task slices while explicitly changing the
metric from the leaderboard's pass@2 to pass@1. The plot provides slice-level
values rather than one directly interchangeable leaderboard number.

The correct conclusion is that the discovered agent is competitive in the
paper's selected setup, not that DGM universally dominates Aider.

## Cost accounting

| Item | Reported cost | Included boundary | Excluded or unclear | Source locator | Reproduction status |
|---|---:|---|---|---|---|
| One SWE-bench DGM run | About USD 22,000 | Paper's complete search run estimate | Engineering labor, infrastructure, failed setup, historical price drift | DGM Appendix E.1 | Not independently reproduced |
| One SWE-bench main baseline run | About USD 10,000 | Either no-self-improve or no-open-ended run | Same exclusions | DGM Appendix E.1 | Not independently reproduced |
| One 60-task SWE-bench evaluation | About USD 350 | Claude 3.5 Sonnet task inference estimate | Diagnostic and self-edit calls unless part of another subtotal | DGM Appendix E.1 | Not independently reproduced |
| One 60-task Polyglot evaluation | About USD 5 | `o3-mini` task inference estimate | Diagnostic and self-edit calls unless part of another subtotal | DGM Appendix E.1 | Not independently reproduced |

The paper notes that stronger discovered agents can cost more per inference,
but cost and performance are not strictly correlated.

A fair root-tree budget should include:

- diagnostic calls;
- self-modification calls;
- candidate retries;
- benchmark task calls;
- evaluator compute;
- container build and execution;
- failed candidates;
- full-evaluation allocation; and
- all parallel descendants.

Task accuracy alone does not normalize these costs.

## Elapsed time

The paper states one SWE-bench DGM run takes roughly two weeks. This is a
wall-clock report for the studied setup, not a hardware-normalized throughput
measure.

Parallel attempts and adaptive benchmark staging mean total compute and elapsed
time are different accounting boundaries.

## Selection and winner's-curse risks

### Adaptive evaluation

Candidates receive more tasks after strong small-subset results. The selected
best candidate is therefore conditioned on noisy early measurements.

### Many-candidate selection

Searching many descendants raises the expected maximum observed score even
when true candidate qualities change less. Final evaluation on a larger or
held-out task set reduces but does not eliminate this selection effect.

### Unequal score precision

Archive plots mix 10-, 60-, and 200-task SWE-bench nodes. Color differences do
not have equal statistical confidence.

### Stochastic model execution

Temperature 1.0 and limited repeated runs add task-level and search-level
variance. Polyglot's released harness rejects repeated evaluations.

### Feature-selection leakage

**[INFERENCE - DGM-068](claim_evidence_crosswalk.md#dgm-068-privileged-diagnosis-makes-the-benchmark-a-development-channel).**
The diagnostic planner can inspect private SWE-bench tests and result logs.
Even when the generated task solver never sees those tests, the search process
can select general harness features using privileged information.

### Statistical uncertainty

**[INFERENCE - DGM-069](claim_evidence_crosswalk.md#dgm-069-staged-point-estimates-create-winners-curse-risk).**
At a true success rate of 0.5, the binomial standard deviation of raw accuracy
is about 15.8 percentage points for 10 tasks, 6.5 points for 60 tasks, and 3.5
points for 200 tasks. Passing those unequal-precision point estimates through a
steep score selector creates winner's-curse risk. A successor should use
posterior sampling, confidence-aware racing, or another uncertainty-aware
allocation rule.

## Contamination boundary

The paper argues Polyglot is less likely than SWE-bench to appear in model
post-training data. This is a plausibility argument, not a contamination
audit.

The packet has no evidence that:

- model pretraining and post-training corpora exclude the benchmarks;
- model providers exposed immutable training-data attestations;
- benchmark task identifiers were hidden from all model stages; or
- diagnostic prompts could not transmit test-specific structure into the
  candidate.

Cross-benchmark and cross-language transfer are useful checks, but they do not
prove contamination absence.

## What the experiments test

The experiments provide evidence for:

- automated improvement of a deliberately simple coding-agent harness;
- persistence of useful changes through patch lineage;
- benefit of the tested self-improving modifier relative to a fixed modifier;
- benefit of the tested branching archive relative to linear and greedy
  alternatives;
- non-monotone stepping-stone lineages;
- partial transfer across models, benchmarks, and languages; and
- recurring production of functioning child agents.

## What the experiments do not test

They do not directly test whether:

- a child is a better modifier than its parent under matched conditions;
- the expected gain per self-improvement attempt increases across generations;
- the search becomes more compute-efficient;
- the outer archive and selection policy can safely improve itself;
- the system remains aligned under a hidden adversarial evaluator;
- the archive retains meaningful behavioral diversity;
- a result reproduces under current public model APIs; or
- the discovered agents improve long-term repository health.

## Minimal direct successor metric

For parent \(P\) and child \(C\), freeze the outer envelope and sample matched
self-improvement tasks:

$$
\Delta(P) =
\mathbb{E}[
  \operatorname{utility}(\operatorname{bestValidChild}(P))
  - \operatorname{utility}(P)
]
$$

and similarly for \(\Delta(C)\).

Evidence for successor improvement requires:

$$
\Delta(C) > \Delta(P)
$$

with uncertainty bounds and matched:

- model;
- diagnostic evidence;
- tasks;
- archive policy;
- evaluator;
- permissions;
- retries; and
- root-tree budget.

The [successor design](10_successor_design.md) develops this experiment.

## Evaluation judgment

**[INFERENCE - DGM-067](claim_evidence_crosswalk.md#dgm-067-task-fitness-and-descendant-productivity-are-different-objectives).**
The reported evidence is substantial for automated harness search and stronger
than a single best-score anecdote because it includes ablations, transfer,
lineage analysis, and limited repeated runs.

The central causal gap remains unchanged:

> better task-solving descendants are used as a proxy for better future
> self-improvers, but parent and child are not directly compared as producers
> of the next accepted generation.

Continue with [open-endedness](07_open_endedness.md), inspect
[safety and failure](08_safety_and_failure.md).

Back to the [DGM index](darwin_godel_machine_index.md).
