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

A result can also change because serving conversion, runtime configuration,
harness policy, or evaluation changed. Record those factors separately from a
weight update before crediting adaptation.

## Pretraining

Pretraining minimizes a predictive loss over a broad corpus and writes statistical regularities into model weights. In an RSI experiment it is usually the most expensive editable component and the hardest to attribute because changes in data, optimizer, architecture, and compute interact. A successor claim needs a reproducible training recipe and matched compute, not only a better final checkpoint.

Tokenizer and observation contracts, architecture, and checkpoint conversion
are separate variables. A new training corpus or architecture is not equivalent
to a harness-only change.

## Post-training

Post-training adapts a pretrained model to instructions, preferences, tools, or target domains. Supervised fine-tuning learns from labeled trajectories; preference optimization changes relative likelihoods using ranked outputs. These methods can internalize a procedure, but gains may remain narrow to the post-training distribution.

If a tool schema, action encoding, or context policy changes, collect and
evaluate demonstrations under that new contract. Do not attribute an interface
mismatch to weights by default.

## Distillation

Distillation trains a student on teacher outputs, logits, traces, or selected demonstrations. It can move a slow harness procedure into weights and reduce inference cost. The key test compares the student with and without the external procedure on held-out tasks, including cases where teacher traces are unavailable.

## Reinforcement learning

Reinforcement learning updates a policy from rewards assigned to sampled trajectories. In a harnessed agent, the action space includes text and tool calls, while the harness determines which states and actions are visible. Reward quality, exploration, credit assignment, and evaluator independence determine whether the update learns the intended behavior or exploits the measurement.

Freeze the training reward within a run, keep the promotion evaluator separate,
and record the serving interface. Changing a scorer or serving harness with
frozen weights can change measured behavior without changing the learned policy.

## Multimodal observation interface

A multimodal interface changes the state available to the policy through image, audio, video, spatial, or other encoders and changes actions through modality-specific decoders or tools. An RSI loop must version tokenization, encoders, context packing, and action schemas because changing any of them shifts the deployed state-action distribution even when core model weights do not change.

If learned encoder tensors change, record a weight update. If only capture
timing, crop, frame selection, context packing, tool availability, or request
schemas change, record a harness update and evaluate with the model frozen.

<details>
<summary>Original sources for this mechanism</summary>

- [[knowledge/rsi/chapters/foundation-model-inside-the-loop#How weights change|Foundation model inside the loop]] connects each adaptation mechanism to candidate state and evaluation.
- Continual Harness, §§3–4, provides a concrete DAgger-style joint harness and weight adaptation process: [arXiv:2605.09998v1](https://arxiv.org/abs/2605.09998).
- SIA describes alternating harness and weight updates and reports the resulting confounds: [arXiv:2605.27276v2](https://arxiv.org/abs/2605.27276).

</details>
