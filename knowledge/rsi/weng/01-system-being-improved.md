# What system is being improved?

## What Weng claims

**CLAIM — [WENG-HARNESS], introduction and coding-agent case study.** The
relevant system is not only a model checkpoint. It includes the deployment
harness that chooses context, tools, memory, control flow, subagents,
verification, and persistence. A model may also improve the training machinery
that produces a successor model.

[Read the original case study](https://lilianweng.github.io/posts/2026-07-04-harness/#case-study-coding-agent-harness).

## Mechanism

**INFERENCE.** Represent the editable candidate as `Cₜ = (Wₜ, Hₜ, Dₜ, Rₜ)`:
weights, harness policy, persistent learned artifacts, and the retrospective
policy that proposes changes. Keep evaluator `E`, budget `B`, permission
policy `P`, protected archive `Aₜ`, and promotion authority outside candidate
write control.

The distinction matters because equal weights can behave differently under
different harnesses, while equal harnesses can behave differently after a
weight update. A useful experiment therefore versions and evaluates the
composite rather than silently attributing every gain to the model.

## Hidden assumption

**INFERENCE.** Weng's framing assumes the deployed composite is the correct
unit of capability and that its components can be changed without losing
causal attribution. If every surrounding software improvement is labeled RSI,
ordinary engineering, task iteration, persistent adaptation, and recursive
improvement collapse into one category.

## Demonstrated versus proposed

**EVIDENCE.** Coding-agent harnesses demonstrate that tools, context policy,
verification, and persistence materially affect behavior with fixed weights.

**MISSING.** This does not show that an accepted composite is better at
producing its next accepted improvement. That successor-oriented test is the
additional evidence required for a recursive claim.

## What would weaken this interpretation

A crossed evaluation in which `W_old × H_new` and `W_new × H_old` explain no
meaningful behavior difference would weaken the claim that harness and weight
identity both matter. A later-cycle experiment in which the accepted child
does not improve the distribution of valid next candidates would weaken the
recursive interpretation while leaving an immediate task-gain claim intact.

## Reader checkpoint

For a coding agent that edits a data-curation script and launches fine-tuning,
name the deployment action, the training-pipeline change, the persistent
successor artifact, and the external actor that may promote it.

<details>
<summary>Check your answer</summary>

The edit and launch are deployment actions under `Hₜ`; the changed data or
training program alters the pipeline that produces `Wₜ₊₁`; the immutable model
checkpoint plus lineage receipt is the successor artifact; and an authority
outside the candidate must evaluate and promote the new composite. This is a
successor transition only after external acceptance, not merely because a new
run exists.

</details>
