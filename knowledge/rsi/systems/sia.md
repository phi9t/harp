---
id: rsi-system-sia
kind: concept
title: Self Improving AI
summary: A feedback agent that updates both task-agent scaffolds and LoRA weights, three-domain author-reported evaluation, ablation boundaries, model confounds, and joint-adaptation claim limits.
primary_parent: rsi-joint-harness-weight-adaptation
additional_parents:
  - rsi-evaluation-promotion-containment
related:
  - kind: compared-with
    target: rsi-system-continual-harness
attachments:
  - content/source_registry.md
  - content/diagnostics/cases/sia.json
claims: []
human_review: null
---

# SIA: routing updates into harnesses and weights

## Problem and RSI relevance

**EVIDENCE — [SIA], §§1–3.** SIA combines two update paths that are often
studied separately. A Feedback-Agent reads a task agent's scaffold,
trajectories, and metrics, then proposes a next scaffold. A weight-update path
trains task-specific LoRA parameters from task feedback.

The editable candidate spans `H_t` and `W_t`, so SIA is
`joint-harness-weight-adaptation`.

## Generation protocol

Each generation evaluates the current task agent, records its full trajectory
and error metrics, and asks the Feedback-Agent for an improvement report plus a
new scaffold. The broader pipeline can then update weights and run another
evaluation.

The evaluator, benchmark interface, base model, trainer, controller, and
generation budget remain external.

<details>
<summary>Original sources for this mechanism</summary>

- Vocabulary and agent decomposition: [SIA, §§2–3](https://arxiv.org/abs/2605.27276).
- Configurable improvement loop: [SIA, §5](https://arxiv.org/abs/2605.27276).
- Results and ablations: [SIA, §6](https://arxiv.org/abs/2605.27276).
- Lever interpretation: [SIA, §7](https://arxiv.org/abs/2605.27276).
- Limitations and future work: [SIA, §§8–9](https://arxiv.org/abs/2605.27276).
- Checked-in text: `evidence/weng/text/sia.txt`.

</details>

## Evaluation

**EVIDENCE — [SIA], Abstract and §6.** The paper evaluates law, kernel
optimization, and single-cell RNA denoising. It reports that harness plus weight
updates outperform harness-only iteration on all three selected tasks, including
the abstract's 20.4% improvement over the compared denoising result.

The experiments include baseline, harness-only, and harness-plus-weight
operating points. They do not by themselves establish that every gain is due to
the claimed lever rather than model choice, training budget, task-specific
grader behavior, or interaction between levers.

## Attribution and Goodhart risk

Joint adaptation creates a four-cell causal question:

1. baseline weights and baseline harness;
2. baseline weights and updated harness;
3. updated weights and baseline harness;
4. updated weights and updated harness.

Without matched cells, one cannot cleanly attribute gains or determine whether
the harness merely changes data collection for weight training.

## Failure modes and limits

- Different models or training budgets can confound lever comparisons.
- LoRA training can memorize narrow task feedback.
- Scaffold search can exploit benchmark extraction or graders.
- A shared Feedback-Agent and verifier create judge circularity.
- Three task domains do not establish general successor development.

Weng's synthesis specifically treats the current SIA evidence as confounded and
provisional. This Atlas preserves that caution.

## Claim ceiling

SIA supports an author-reported joint harness-and-weight adaptation mechanism.
It does not show that an externally accepted child becomes a better producer of
later accepted children under matched authority and resource conditions.

## Reading routes

- [Weng: joint harness and weight optimization](../weng/08-joint-harness-weight-optimization.md)
- [SIA versus Continual Harness lesson](../lessons/06-sia-vs-continual-harness.md)
- [Joint harness and model-weight adaptation](../chapters/joint-harness-weight-adaptation.md)
- [Original paper](https://arxiv.org/abs/2605.27276)
