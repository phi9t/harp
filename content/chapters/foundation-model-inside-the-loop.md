---
id: rsi-foundation-model-inside-the-loop
kind: concept
title: Foundation model inside the loop
summary: How architecture, weights, training, inference, and deployment constrain an RSI candidate.
primary_parent: modeling
additional_parents:
  - mlsys
related:
  - kind: wrapped-by
    target: rsi-harness-engineering
  - kind: co-adapts-with
    target: rsi-joint-harness-weight-adaptation
attachments:
  - content/recursive_language_models_compositional_generalization.md
  - content/source_registry.md
claims: []
human_review: null
---

# Foundation model inside the loop

## Model state is not the whole agent

A foundation model maps a tokenized observation and decoding state to a distribution over next tokens or actions. Its weights retain statistical regularities learned during training. Its architecture fixes important constraints such as context representation, attention pattern, expert routing, multimodal encoders, output heads, and numerical execution. The deployed agent adds a harness that selects observations, constructs context, exposes tools, interprets outputs, and persists state.

For an RSI experiment, separate:

`model behavior = f(W, A, T, x, d)`

`agent behavior = H(f(W, A, T, context_H(s)), tools_H, memory_H)`

`W` is the parameter tensor collection, `A` is architecture, `T` is tokenization and observation encoding, `x` is model-visible input, and `d` is decoding configuration. `H` is the harness and `s` is the larger environment state. A model-weight update changes `W`; a harness update can change the distribution of `x`, available actions, and consequences without touching `W`.

## How weights change

### Pretraining

Pretraining minimizes a predictive loss over a broad corpus. For autoregressive text:

`L_pre(W) = −Eₓ[∑ᵢ log p_W(xᵢ | x₍<ᵢ₎)]`

The loss teaches representations and conditional behavior but does not directly specify a deployed tool loop. Architecture and data determine which regularities can be represented and which inputs the model sees.

### Post-training and supervised adaptation

Supervised fine-tuning fits demonstrations from a target interaction format. Distillation fits a student to teacher targets. Both can internalize procedures that were previously supplied as prompts or traces, but they also inherit dataset coverage and teacher errors.

### Reinforcement learning

Reinforcement learning changes weights to increase expected return:

`J(W) = E_τ∼π_W,H[R(τ)]`

The trajectory distribution `τ` depends on both model policy `π_W` and harness `H`. A reward measured in one harness may not transfer when a new harness changes observations, tool schemas, context length, or action timing.

### Deployment and inference

Quantization, expert placement, cache policy, batching, speculative decoding, and tool latency affect which behavior is reachable under a real budget. A weight checkpoint is not a complete behavioral identity. The model revision, tokenizer, generation settings, runtime kernels, and harness configuration belong in an experiment receipt.

<details>
<summary>Original sources for this mechanism</summary>

- Recursive Language Models, §§2–4 and Figure 2, defines a fixed recursive inference harness around a root model; the [RLM mechanism anchor](../recursive_language_models_compositional_generalization.md) retains the paper-to-code comparison: [arXiv:2512.24601v3](https://arxiv.org/abs/2512.24601).
- SIA, §§2–4, and Continual Harness, §§3–4, describe bounded experiments that route experience into harness or weight updates: [arXiv:2605.27276v2](https://arxiv.org/abs/2605.27276), [arXiv:2605.09998v1](https://arxiv.org/abs/2605.09998).

</details>

## Architecture constrains the loop

A harness cannot assume that every model has the same context limit, tool-call
syntax, modality encoder, routing behavior, or inference cost. The experiment
must therefore bind each observation, action, and measurement to a named model
revision, tokenizer, runtime, and decoding policy. These are experimental
controls, not evidence that the model is self-improving.

### Multimodal state and actions

A multimodal model changes the loop when observations include images, audio, video, rendered interfaces, or continuous sensor features. Let `oₜ = (textₜ, imageₜ, audioₜ, stateₜ)` and let encoder `g_T` map that observation into model tokens or latent features. The harness controls capture timing, resolution, crop, compression, frame selection, and action serialization.

An apparent model regression may come from changed observation encoding rather than changed weights. Conversely, a stronger visual encoder may be useless if the harness drops the pixels or sends stale screenshots. A multimodal RSI experiment must version the encoder, preprocessing, observation schedule, and action interface.

## Runtime data flow

1. The environment produces state `sₜ`.
2. The harness selects and encodes observation `oₜ`.
3. The tokenizer or modality encoders produce model input.
4. The model samples text, structured calls, or actions under decoding policy `d`.
5. The harness validates and executes allowed actions.
6. Results update environment state and persistent artifacts.
7. The evaluator observes protected outcomes that the candidate cannot rewrite.

The model only learns from this loop when trajectories enter a training dataset or an online update. Context alone changes inference behavior but not weights.

## Failure modes and tradeoffs

- **Weights-harness attribution error.** A benchmark change may come from prompt, tools, retrieval, or decoding rather than `W`.
- **Architecture mismatch.** A harness designed for one tokenizer, context layout, or tool grammar can degrade another model.
- **Training-serving skew.** Training trajectories use action formats or tools absent at deployment.
- **Context masking.** Larger context can bury relevant state or increase distraction and latency.
- **Multimodal aliasing.** Cropping, temporal sampling, or lossy encoding removes task-critical information.
- **Reward misspecification.** RL improves the scored behavior while damaging calibration, exploration, or safety.
- **Checkpoint incompleteness.** Weights without tokenizer, config, runtime, and generation settings do not reproduce behavior.
- **Cost confounding.** A larger or slower model may win only because the budget is not matched.

## What would weaken the mechanism

The model-harness account weakens if controlled ablations show that model behavior is invariant to the claimed architecture, training, or harness difference. For a proposed weight update, compare frozen-harness parent and child checkpoints. For a proposed harness update, compare frozen weights. For a joint update, use the four-way experiment described in [joint harness and model-weight adaptation](joint-harness-weight-adaptation.md).

## Open technical questions

- Which harness abstractions transfer across model families without hiding model-specific constraints?
- How should an archive identify hosted models whose exact weights and runtime are unavailable?
- Can multimodal observation policies be optimized without teaching the model shortcuts in the capture process?
- How much of a useful harness procedure should be internalized into weights before flexibility is lost?

<details>
<summary>Reference records and operational metadata</summary>

- RSI source identities and claim limits are in [the source registry](../source_registry.md).
- This chapter deliberately stops at model/harness experimental controls; it
  does not claim a model-specific implementation study that Harp does not
  carry.

</details>
