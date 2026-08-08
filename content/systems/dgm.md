---
id: rsi-system-dgm
kind: concept
title: Darwin Gödel Machine
summary: Self-modifying coding-agent repositories, benchmark-tested descendants, branching archive search, reported coding gains, frozen foundation models, and the missing next-cycle comparison.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-recursive-improvement-loop
  - rsi-evaluation-promotion-containment
related:
  - kind: specializes
    target: rsi-harness-search
attachments:
  - content/source_registry.md
  - content/diagnostics/cases/dgm.json
claims: []
human_review: null
---

# DGM: a branching lineage of editable coding agents

## Problem and RSI relevance

**EVIDENCE — [DGM], §§1–3.** Darwin Gödel Machine represents a coding agent as
an editable repository powered by frozen pretrained foundation models. Parent
agents inspect their evaluation logs, modify their own code, and create child
agents that enter a branching archive after execution and benchmark checks.

This reaches the recursive boundary more directly than solution-program search:
the edited repository includes code used for later self-modification.

## Algorithm and archive

1. Select a parent from the archive using performance and exploration pressure.
2. Let the parent inspect its own code and benchmark logs.
3. Propose and implement one agent change.
4. Execute safety and validity checks.
5. Evaluate the child on a coding benchmark.
6. Add viable children and their lineage to the archive.
7. Repeat while preserving older stepping stones.

The archive avoids single-incumbent replacement and allows an older branch to
become useful later. Selection still depends on externally chosen benchmarks
and archive policy.

<details>
<summary>Original sources for this mechanism</summary>

- Self-modifying agent and archive: [DGM, §3](https://arxiv.org/abs/2505.22954).
- Experiment setup and benchmarks: [DGM, §§4.1–4.3](https://arxiv.org/abs/2505.22954).
- Results and ablations: [DGM, §4.4](https://arxiv.org/abs/2505.22954).
- Safety and limitations: [DGM, §§5–6](https://arxiv.org/abs/2505.22954).
- Checked-in text: `evidence/weng/text/dgm.txt`.

</details>

## Evaluation

**EVIDENCE — [DGM], Abstract and §4.4.** The paper reports improvement from
20.0% to 50.0% on SWE-bench and from 14.2% to 30.7% on Polyglot. Ablations
compare self-improvement with generating agents without self-modification and
test the value of open-ended exploration and retaining the archive.

These are author-reported benchmark results. The model, task suites, time
limits, sandbox, and selection policy remain fixed external components.

## The disputed recursive step

**CLAIM — [DGM], §1.** The paper argues that better coding-benchmark
performance indicates better ability to modify and improve the coding-agent
repository.

**INFERENCE.** That proxy is plausible but not the matched next-cycle test.
Benchmark gain and later improvement-production gain can diverge. The stricter
experiment would compare parent and child as proposers under the same model,
task distribution, permissions, evaluator, and root-tree budget.

## Failure modes and limits

- Benchmark overfitting can masquerade as self-improvement ability.
- Frozen foundation models bound what repository edits can discover.
- Generated code requires sandboxing and bounded network/tool authority.
- Archive growth raises evaluation cost and retrospective selection risk.
- Human-chosen tasks and promotion policy define what counts as progress.

The paper's safety section describes sandboxed, time-limited experiments and
the conclusion names frozen models and benchmark dependence as limitations.

## Claim ceiling

DGM supports `harness-improvement` with an accepted branching agent lineage.
It does not establish `successor-improvement` or
`recursive-improvement-demonstrated` in this Atlas because the source does not
measure whether accepted children produce better later accepted children under
a matched protected envelope.

## Reading routes

- [Weng: evolutionary search](../weng/07-evolutionary-search.md)
- [AlphaEvolve versus DGM lesson](../lessons/05-alphaevolve-vs-dgm.md)
- [Evaluation, promotion, and containment](../chapters/evaluation-promotion-containment.md)
- [Original paper](https://arxiv.org/abs/2505.22954)
