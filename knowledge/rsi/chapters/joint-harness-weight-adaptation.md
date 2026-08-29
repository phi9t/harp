---
id: rsi-joint-harness-weight-adaptation
kind: concept
title: Joint harness and model-weight adaptation
summary: Distribution shift, DAgger, relabeling, reinforcement learning, process rewards, distillation, RLMs, Continual Harness, and SIA.
primary_parent: modeling
additional_parents:
  - mlsys
related:
  - kind: combines
    target: rsi-foundation-model-inside-the-loop
  - kind: combines
    target: rsi-harness-engineering
attachments:
  - content/source_registry.md
  - content/rsi_harness_by_lil_log_deconstructed.md
  - content/recursive_language_models_compositional_generalization.md
claims: []
human_review: null
---

# Joint harness and model-weight adaptation

## The coupling problem

A harness determines the observations, actions, tools, and trajectories available to a model. Changing the harness changes the deployed data distribution. A model trained under old prompts, tool schemas, context layouts, or recovery behavior may become less capable when the harness improves. Conversely, a harness optimized around current model weaknesses can become obsolete after weight updates.

Let `d_H(s,a)` be the state-action occupancy distribution induced by harness `H` and model policy. A harness update from `H₀` to `H₁` creates distribution shift:

`D_shift = D_KL(d_H₁ || d_H₀)`

`D_KL` is the Kullback-Leibler divergence from the old occupancy distribution
to the new one. The divergence is conceptual unless the state and action spaces
admit a density estimate. Operationally, measure changes in observation types,
tool calls, context positions, error states, and trajectory lengths. Large
shift means offline examples from `H₀` may not train reliable behavior under
`H₁`.

## Alternating adaptation

One practical algorithm alternates harness and weight updates:

1. Run model `Wₜ` in harness `Hₜ` and collect trajectories.
2. Identify failures attributable to context, tools, workflow, or model policy.
3. Propose harness candidates and evaluate them with frozen `Wₜ`.
4. Promote `Hₜ₊₁` only through protected gates.
5. Run `Wₜ` in `Hₜ₊₁` to collect on-policy states, including new failure states.
6. Obtain teacher relabels, demonstrations, process rewards, or scalar returns.
7. Train candidate weights `Wₜ₊₁`.
8. Evaluate all four model-harness combinations.
9. Promote a pair only if gains survive held-out tasks, integrity checks, and cost normalization.

The four combinations are `(Wₜ,Hₜ)`, `(Wₜ,Hₜ₊₁)`, `(Wₜ₊₁,Hₜ)`, and `(Wₜ₊₁,Hₜ₊₁)`. They separate harness benefit, weight benefit, complementarity, and incompatibility.

## DAgger and supervised relabeling

DAgger addresses covariate shift by collecting states from the current policy and asking an expert for the preferred action. At iteration `i`, aggregate:

`Dᵢ = Dᵢ₋₁ ∪ {(s, a*) | s ∼ d_πᵢ, a* ∼ expert(s)}`

Then train `πᵢ₊₁` on `Dᵢ`. For a tool-using agent, states include context, tool results, filesystem status, and prior errors. The expert may be a stronger model, a verified planner, human correction, or an executable oracle for narrow actions.

Supervised relabeling is attractive because corrected actions are inspectable. It can still teach the expert's mistakes and may not cover long-term credit.

## Reinforcement learning and process rewards

Reinforcement learning optimizes expected return under the deployed harness. Outcome rewards score final success. Process rewards score intermediate decisions such as decomposition, tool selection, invariant preservation, or recovery.

Process rewards improve credit assignment but enlarge the evaluator. If the candidate can influence reward-model inputs or annotations, reward hacking moves inside the trajectory. Keep reward computation and protected traces outside candidate write authority.

Distillation can compress a stronger model or expensive procedure into a cheaper policy. It should be evaluated with the same four-way procedure test described in [[knowledge/rsi/chapters/procedure-internalization|procedure internalization]].

<details>
<summary>Original sources for this mechanism</summary>

- Ross, Gordon, and Bagnell, Algorithm 3.1, introduces DAgger's on-policy dataset aggregation: [AISTATS 2011](https://proceedings.mlr.press/v15/ross11a.html).
- Continual Harness, §§3.2–3.4 and §4, describes teacher relabeling, process rewards, reset-free state propagation, and author-reported co-learning experiments: [arXiv:2605.09998v1](https://arxiv.org/abs/2605.09998).
- SIA, §§2–4, describes a controller that chooses harness or weight updates in a bounded self-improvement experiment: [arXiv:2605.27276v2](https://arxiv.org/abs/2605.27276).
- Recursive Language Models, §§2–4, and "Language model harnesses are compositional generalizers," sections "Post-training setup" and "Results," describe training inside a fixed recursive harness and author-reported transfer effects; the [[knowledge/rsi/recursive_language_models_compositional_generalization|mechanism anchor]] retains the caveats: [arXiv:2512.24601v3](https://arxiv.org/abs/2512.24601), [author technical blog](https://alexzhang13.github.io/blog/2026/rlm-harness-generalization/).

</details>

## Worked examples

### Recursive language models

RLM training holds the recursive harness fixed and updates root-model weights. It demonstrates that harness-induced trajectories can change compositional transfer. It does not jointly search the harness, but it provides direct evidence that `H` changes the learning problem for `W`.

### EnvHarness

EnvHarness separates the two update surfaces across experiments. The main result
holds weights fixed and updates an external retrieved skill bank from shaped
environment rollouts. A separate single-seed GRPO experiment updates Qwen3-8B
weights using EnvHarness environments. Neither result evaluates all four
`(W,H)` combinations under a complete matched resource vector, so they motivate
the coupling problem without establishing robust joint adaptation.

### Continual Harness

Continual Harness collects experience under an evolving agent interaction, obtains teacher relabels and process rewards, and updates the model online. The mechanism addresses the gap between new harness states and old model behavior. Its current evidence is author-reported and task-bounded.

### SIA

SIA selects between harness and weight updates. This reaches a broader editable state than fixed-weight harness search. The attribution problem remains: a result from the pair does not show which update caused the gain unless all four combinations and matched controls are evaluated.

## Failure modes and tradeoffs

- **Harness-model co-overfitting.** The pair succeeds only together on the search tasks.
- **Catastrophic interference.** New tool behavior damages general language or prior tool skills.
- **Stale demonstrations.** Data from `H₀` trains actions invalid under `H₁`.
- **Teacher leakage.** A stronger teacher sees held-out data or evaluator details.
- **Process-reward hacking.** The model produces rewarded traces without correct outcomes.
- **Reset-free contamination.** Persistent environment state makes episodes non-comparable.
- **Attribution failure.** Joint promotion hides whether weights, harness, or extra compute helped.
- **Oscillation.** Harness updates compensate for model weaknesses that the next weight update removes.
- **Deployment mismatch.** Training uses tools, latencies, or permissions absent in production.

Alternating updates improve attribution but can be slow and myopic. Joint optimization can discover complementarity but requires factorial evaluation and stronger containment.

## What would weaken the mechanism

The coupling claim weakens if `Wₜ` performs equally under `Hₜ` and `Hₜ₊₁`, if occupancy and failure distributions do not change, or if frozen-factor ablations explain the full gain with one component. A co-adaptation claim fails if the pair advantage disappears on fresh tasks or if the model cannot operate under the production harness.

## Open technical questions

- How should one estimate useful state-action coverage for open-ended tool use?
- When should harness search pause so weight training can catch up?
- Can process rewards remain stable while the harness changes trace structure?
- How should rollback handle a pair when only one component causes a regression?

<details>
<summary>Reference records and operational metadata</summary>

- Source identities, access states, and result limits are in [[knowledge/rsi/source_registry|the source registry]].
- No independent reproduction in this packet closes the joint-adaptation claim.
- Model-specific architecture details remain in the linked model packets rather than being duplicated here.

</details>
