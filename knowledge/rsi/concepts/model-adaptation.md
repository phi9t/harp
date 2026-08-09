---
id: rsi-model-adaptation
kind: concept
title: Model adaptation mechanisms
summary: Pretraining, post-training, distillation, reinforcement learning, and multimodal interfaces inside an improvement loop.
primary_parent: rsi-foundation-model-inside-the-loop
additional_parents: []
related: []
attachments:
  - content/chapters/foundation-model-inside-the-loop.md
claims: []
human_review: null
---

# Model adaptation mechanisms

Model adaptation changes weights or the observation and action interface that those weights operate over. Each mechanism creates a different data requirement and attribution problem.

## Pretraining

Pretraining minimizes a predictive loss over a broad corpus and writes statistical regularities into model weights. In an RSI experiment it is usually the most expensive editable component and the hardest to attribute because changes in data, optimizer, architecture, and compute interact. A successor claim needs a reproducible training recipe and matched compute, not only a better final checkpoint.

## Post-training

Post-training adapts a pretrained model to instructions, preferences, tools, or target domains. Supervised fine-tuning learns from labeled trajectories; preference optimization changes relative likelihoods using ranked outputs. These methods can internalize a procedure, but gains may remain narrow to the post-training distribution.

## Distillation

Distillation trains a student on teacher outputs, logits, traces, or selected demonstrations. It can move a slow harness procedure into weights and reduce inference cost. The key test compares the student with and without the external procedure on held-out tasks, including cases where teacher traces are unavailable.

## Reinforcement learning

Reinforcement learning updates a policy from rewards assigned to sampled trajectories. In a harnessed agent, the action space includes text and tool calls, while the harness determines which states and actions are visible. Reward quality, exploration, credit assignment, and evaluator independence determine whether the update learns the intended behavior or exploits the measurement.

## Multimodal observation interface

A multimodal interface changes the state available to the policy through image, audio, video, spatial, or other encoders and changes actions through modality-specific decoders or tools. An RSI loop must version tokenization, encoders, context packing, and action schemas because changing any of them shifts the deployed state-action distribution even when core model weights do not change.

<details>
<summary>Original sources for this mechanism</summary>

- [Foundation model inside the loop](../chapters/foundation-model-inside-the-loop.md#how-weights-change) connects each adaptation mechanism to candidate state and evaluation.
- Continual Harness, §§3–4, provides a concrete DAgger-style joint harness and weight adaptation process: [arXiv:2605.09998v1](https://arxiv.org/abs/2605.09998).
- SIA describes alternating harness and weight updates and reports the resulting confounds: [arXiv:2605.27276v2](https://arxiv.org/abs/2605.27276).

</details>
