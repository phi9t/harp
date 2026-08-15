---
id: rsi-system-self-harness
kind: concept
title: Self-Harness
summary: Model-specific weakness mining, minimal harness proposals, held-in and held-out validation, conservative acceptance, and fixed-model claim limits.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-evaluation-promotion-containment
related:
  - kind: compared-with
    target: rsi-system-stop
attachments:
  - content/diagnostics/cases/self-harness.json
claims: []
human_review: null
---

# Self-Harness: turning execution weaknesses into harness edits

## Mechanism

**EVIDENCE — [SELF-HARNESS], §§3–4.** Self-Harness runs a fixed model and
current harness, mines recurring weakness patterns from execution traces,
proposes several diverse but minimal edits tied to those patterns, and validates
proposals on held-in and held-out tasks. Accepted edits seed the next iteration.

The same model family participates in task execution and harness improvement,
but evaluator and acceptance logic remain external.

<details>
<summary>Original sources for this mechanism</summary>

- Three-stage loop: [Self-Harness, §3](https://arxiv.org/abs/2606.09498).
- Validation and results: [Self-Harness, §4](https://arxiv.org/abs/2606.09498).
- Checked-in text: `evidence/weng/text/self-harness.txt`.

</details>

## Evaluation and limits

The preprint reports Terminal-Bench 2.0 pass-rate gains from 40.5% to 61.9%,
23.8% to 38.1%, and 42.9% to 57.1% across three models. Qualitative analysis
reports executable structural changes rather than prompt length alone.

The results are model- and benchmark-specific and not independently reproduced.
Repeated held-out use can still overfit, and accepted harness changes may not be
activated or followed by the target model.

## Claim ceiling

Self-Harness is `harness-improvement` with stronger regression gating than
ungated search. It does not measure whether an accepted harness improves the
next harness-improvement cycle.

## Reading routes

- [[knowledge/rsi/weng/06-self-improving-harnesses|Weng: self-improving harnesses]]
- [[knowledge/rsi/lessons/04-stop-vs-self-harness-ahe|STOP versus Self-Harness and AHE lesson]]
- [[knowledge/rsi/chapters/evaluation-promotion-containment|Evaluation and promotion]]
- [Original paper](https://arxiv.org/abs/2606.09498)
