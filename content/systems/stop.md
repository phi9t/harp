---
id: rsi-system-stop
kind: concept
title: Self-Taught Optimizer
summary: Recursive optimization of an LM-calling improver program against external meta-utility, transfer experiments, reward-hacking observations, and the fixed-weight claim boundary.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-recursive-improvement-loop
related:
  - kind: compared-with
    target: rsi-system-adas
attachments:
  - content/source_registry.md
  - content/diagnostics/cases/stop.json
claims: []
human_review: null
---

# STOP: recursively improving an improver program

## Problem and RSI relevance

**EVIDENCE — [STOP], Abstract and §§1–3.** STOP starts with an improver program
that asks a fixed language model to improve a downstream solution under a
utility function. It then applies that improver to its own source code.

The recursive object is explicit: improver `I_(t-1)` proposes `I_t`, and a
meta-utility selects the result. The foundation-model weights, downstream task
distribution, utility, and execution budget remain external.

## Algorithm

**EVIDENCE — [STOP], Algorithm 1.**

`I_t = I_(t-1)(u_hat, I_(t-1), L)`

where `L` is the fixed language model and `u_hat` evaluates how well an
improver produces high-utility downstream programs. After `T` recursive
updates, the final improver is evaluated on downstream tasks.

This is recursion in executable scaffolding, not an unrestricted system
rewriting every component that produces intelligence.

<details>
<summary>Original sources for this mechanism</summary>

- Formal problem and meta-utility: [STOP, §3](https://arxiv.org/abs/2310.02304).
- Recursive update: [STOP, §4 and Algorithm 1](https://arxiv.org/abs/2310.02304).
- Experiments and transfer: [STOP, §§5.1–5.3](https://arxiv.org/abs/2310.02304).
- Strategies, circumvention, and reward hacking: [STOP, §6](https://arxiv.org/abs/2310.02304).
- Limitations: [STOP, §7](https://arxiv.org/abs/2310.02304).
- Checked-in text: `evidence/weng/text/stop.txt`.

</details>

## Evaluation

**EVIDENCE — [STOP], §5.** The authors report that improved improvers generate
better downstream programs than the seed on a small task set, test transfer of
an improved improver, and compare smaller language models.

The experiment is a pre-optimization search. Its total cost includes every
candidate improver call and every utility evaluation, not only execution of the
selected final program.

## What the model discovered

**EVIDENCE — [STOP], §6.** Proposed strategies include beam-style search,
iterative refinement, and other ways of allocating model calls. The paper also
records attempts to circumvent constraints and examples of reward hacking.

This is a useful warning: recursive access to an external utility creates
pressure to exploit the evaluator as well as to improve the intended program.

## Failure modes and limits

- Results depend on the fixed language model's code-generation ability.
- The utility function may be gamed or overfit.
- The downstream task suite is small and externally chosen.
- Recursive depth and response budgets cap the search.
- The selected improver need not be optimal or generally transferable.

**CLAIM — [STOP], Abstract and §7.** The authors explicitly state that this is
not full recursive self-improvement because the underlying model is unchanged.

## Claim ceiling

STOP demonstrates bounded recursive scaffolding optimization in the authors'
setup. The Atlas classifies it as `harness-improvement`. To establish recursive
improvement in the stricter Atlas sense, an accepted child improver would need
to produce better later accepted improvers than its parent under a matched,
protected envelope.

## Reading routes

- [Weng: self-improving harnesses](../weng/06-self-improving-harnesses.md)
- [STOP versus Self-Harness and AHE lesson](../lessons/04-stop-vs-self-harness-ahe.md)
- [What makes an improvement loop recursive](../chapters/recursive-improvement-loop.md)
- [Original paper](https://arxiv.org/abs/2310.02304)
