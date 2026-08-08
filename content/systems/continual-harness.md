---
id: rsi-system-continual-harness
kind: concept
title: Continual Harness
summary: Reset-free online harness refinement, evolving prompts agents skills and memory, teacher relabeling, process rewards, policy updates, and task-bounded co-learning evidence.
primary_parent: rsi-joint-harness-weight-adaptation
additional_parents:
  - rsi-procedure-internalization
related:
  - kind: compared-with
    target: rsi-system-sia
attachments:
  - content/source_registry.md
  - content/diagnostics/cases/continual-harness.json
claims: []
human_review: null
---

# Continual Harness: reset-free model-harness co-learning

## Problem and RSI relevance

**EVIDENCE — [CONTINUAL-HARNESS], §§2–3.** Continual Harness starts from a
minimal embodied-agent interface and updates harness state during one ongoing
episode. Harness state includes the system prompt, subagents, skills, and
memory. The agent does not reset after each refinement.

The full method later adds model updates from trajectories collected under the
evolving harness, creating joint adaptation of `H_t`, `D_t`, and `W_t`.

## Two-loop architecture

The inner loop uses the current harness to choose environment actions. Every
fixed number of steps after warm-up, an outer refiner reads the trajectory and
applies CRUD edits to prompt, agents, skills, and memory:

`H_(t+1) = H_t plus delta_t`

The reset-free design preserves consequences and learned state across
refinement cycles, which differs from prompt optimizers that restart complete
episodes.

## Model-harness co-learning

**EVIDENCE — [CONTINUAL-HARNESS], §3.3.** After harness warm-up, trajectories
are scored, low-reward behavior is relabeled by a stronger teacher, process
rewards are computed, and an open-source student is updated through soft
supervised fine-tuning. The changed model then acts under the changed harness,
producing new on-policy states.

<details>
<summary>Original sources for this mechanism</summary>

- Harness decomposition: [Continual Harness, §2.2](https://arxiv.org/abs/2605.09998).
- Reset-free refinement: [Continual Harness, §§3.1–3.2](https://arxiv.org/abs/2605.09998).
- Co-learning loop: [Continual Harness, §3.3](https://arxiv.org/abs/2605.09998).
- Results and skill analysis: [Continual Harness, §§4.2–4.6](https://arxiv.org/abs/2605.09998).
- Discussion and limits: [Continual Harness, §6](https://arxiv.org/abs/2605.09998).
- Checked-in text: `evidence/weng/text/continual-harness.txt`.

</details>

## Evaluation

**EVIDENCE — [CONTINUAL-HARNESS], §4.** The paper evaluates Pokémon Red and
Emerald across several frontier models, compares a minimal harness with a
hand-engineered expert harness, studies model-capability dependence, and then
transfers the refined harness to open-source students for online co-learning.

The authors report lower button-press cost, recovery of a majority of the gap
to the expert harness, capability-dependent gains, and measurable skill
movement toward an oracle.

These are author-reported embodied-agent experiments. The environment,
teacher, reward model, process-reward design, and model-training pipeline remain
external.

## Failure modes and limits

- A stronger teacher can supply the decisive capability.
- Process rewards can encode evaluator bias.
- Reset-free trajectories make interventions dependent and hard to attribute.
- Model and harness changes shift each other's data distribution.
- Long episodes make full root-tree accounting expensive.
- Results in game environments may not transfer to coding or research agents.

## Claim ceiling

Continual Harness supports `joint-harness-weight-adaptation` in a reset-free
online loop. It does not provide independent reproduction or a matched
parent-child test of later improvement production, so it does not establish
successor or recursive improvement in this Atlas.

## Reading routes

- [Weng: joint harness and weight optimization](../weng/08-joint-harness-weight-optimization.md)
- [SIA versus Continual Harness lesson](../lessons/06-sia-vs-continual-harness.md)
- [Joint harness and model-weight adaptation](../chapters/joint-harness-weight-adaptation.md)
- [Original paper](https://arxiv.org/abs/2605.09998)
