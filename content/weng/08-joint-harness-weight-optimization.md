# Joint harness and weight optimization

## What Weng claims

**CLAIM — [WENG-HARNESS], “Joint Optimization with Model Weights.”** SIA
routes feedback into harness changes or model-parameter updates. Continual
Harness combines online harness adaptation with policy learning from
teacher-labeled low-reward trajectories.

[Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#joint-optimization-with-model-weights).

## Mechanism

**INFERENCE.** Joint adaptation changes more than one causal surface:
`Hₜ` shapes the states, tools, and feedback observed by `Wₜ`, while model
updates alter how later harness instructions are activated and followed.
Teacher relabeling on learner-visited states resembles DAgger's response to
sequential distribution shift.

Use alternating interventions before joint search: freeze `W` while changing
`H`, then freeze `H` while changing `W`. Crossed evaluations reveal whether a
gain is attributable to harness, weights, their interaction, or extra data and
compute.

## Hidden assumption

**INFERENCE.** The feedback agent, teacher, data collection, training budget,
and evaluator are assumed independent enough to support attribution. Without
matched baselines, a stronger teacher or larger update budget can masquerade as
a superior improvement policy.

## Demonstrated versus proposed

**EVIDENCE — [SIA], [CONTINUAL-HARNESS].** The inspected sources support the
named routing, teacher relabeling, process-reward, state-propagation, and
author-reported co-learning mechanisms.

**MISSING.** SIA's model choices and baselines leave confounds. Neither source
independently demonstrates a general recursive successor loop under fixed
external authority.

## What would weaken this interpretation

If alternating ablations show all gains come from a stronger teacher, extra
training, or one component alone, the joint-policy claim weakens. If learned
weights exploit evaluator leakage or make the new harness less usable, the
composite must not promote.

## Reader checkpoint

Name the minimum ablations for a joint `W` and `H` improvement claim.

<details>
<summary>Check your answer</summary>

Compare old and new weights under old and new harnesses, plus no-update,
harness-only, weight-only, and resource-matched controls. Keep evaluator,
permissions, task partitions, teacher identity, and root-tree budget fixed.
Report both immediate outcome and later-cycle improvement behavior.

</details>
