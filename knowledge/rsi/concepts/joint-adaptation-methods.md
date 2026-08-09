---
id: rsi-joint-adaptation-methods
kind: concept
title: Joint harness and weight adaptation methods
summary: Distribution shift, DAgger, supervised relabeling, and process rewards.
primary_parent: rsi-joint-harness-weight-adaptation
additional_parents: []
related: []
attachments:
  - content/chapters/joint-harness-weight-adaptation.md
claims: []
human_review: null
---

# Joint harness and weight adaptation methods

Changing a harness changes the trajectories used at inference. Joint adaptation trains model weights on that deployed distribution while preserving an external evaluator and promotion boundary.

## Model-harness distribution shift

Let `d_H(s,a)` be the occupancy distribution induced by harness `H` and the current model policy. Replacing `H₀` with `H₁` changes prompts, tool opportunities, recovery states, and action formats. Performance can fall even when `H₁` is better designed because the model has not learned the new distribution.

## DAgger

Dataset Aggregation alternates policy execution with expert relabeling of states the current policy actually visits. The aggregated dataset reduces compounding error caused by training only on expert trajectories. In a harnessed agent, the expert must label tool and recovery states introduced by the current harness.

## Supervised relabeling

Supervised relabeling turns selected trajectory states into target actions or reasoning traces and fine-tunes the model on them. It is easier to stabilize than online reinforcement learning but inherits teacher mistakes and may optimize imitation rather than the final outcome.

## Process rewards

Process rewards score intermediate actions, states, or steps rather than only terminal success. They can improve credit assignment and reject unsafe trajectories early. They also create more opportunities for reward hacking, so reward generation and integrity checks remain outside candidate write control.

<details>
<summary>Original sources for this mechanism</summary>

- [Joint harness and model-weight adaptation](../chapters/joint-harness-weight-adaptation.md#alternating-adaptation) gives the alternating update algorithm.
- Ross, Gordon, and Bagnell, "A Reduction of Imitation Learning and Structured Prediction to No-Regret Online Learning," Algorithm 3.1, introduces DAgger: [AISTATS 2011](https://proceedings.mlr.press/v15/ross11a.html).
- Continual Harness, §§3.2–3.4, applies teacher relabeling and process-reward scoring to evolving harness trajectories: [arXiv:2605.09998v1](https://arxiv.org/abs/2605.09998).

</details>
