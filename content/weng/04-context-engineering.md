# Context engineering: from artifacts to mechanisms

## What Weng claims

**CLAIM — [WENG-HARNESS], “Context Engineering.”** ACE evolves an itemized
context playbook from rollout feedback. MCE separates context artifacts from
the skills that construct and update them. Meta-Harness moves outward again by
optimizing the code that stores, retrieves, and presents information.

[Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#context-engineering).

[Open the full context-engineering deep dive](../context_engineering_deep_dive.md)
for the ACE artifact lifecycle, MCE bi-level skill evolution, Meta-Harness
code-space search, runtime context stack, cost model, implementation contracts,
and causal ablations.

The deep dive has two reading branches:

- **learning context:** how ACE, MCE, and Meta-Harness expand the mutable object
  from artifacts to learning skills to executable harness code; and
- **running with context:** how retention, retrieval, rendering, activation,
  compaction, replay, and world-state refresh preserve or lose useful state.

[Open the Codex state-continuity companion](../codex_state_continuity_and_compaction.md)
for a source-pinned implementation of the runtime branch. Codex is not a fourth
system in Weng's comparison; it shows why learned context still needs typed
history, coherent compaction, and durable replay.

## Mechanism

**INFERENCE.** The optimization ladder changes the edited object:

1. an artifact such as a prompt, memory item, or playbook bullet;
2. a context-construction skill that chooses and transforms artifacts; and
3. harness code that defines the complete context lifecycle.

This is a progression from changing `Dₜ` toward changing `Hₜ` and `Rₜ`.
Provenance must identify which layer changed, because identical final prompts
can arise from materially different update mechanisms.

The final prompt is also too late to localize many failures. A context item can
exist but remain ineligible, unselected, badly rendered, never activated, or
abandoned during execution. Measure these boundaries separately.

## Hidden assumption

**INFERENCE.** The evaluator must distinguish useful compression from context
collapse, memorization, leakage, or a larger effective budget. The mechanism
also assumes that stored lessons remain valid when tasks and model behavior
shift.

## Demonstrated versus proposed

**EVIDENCE — [ACE], [MCE], [META-HARNESS].** The inspected sources support the
named context representations, bi-level optimization, outer-loop mechanisms,
and author-reported results within their experimental settings.

**MISSING.** This packet contains no independent reproduction and no proof that
context-mechanism evolution yields a generally better future improver.

## What would weaken this interpretation

Held-out performance that disappears when context length and model calls are
matched, or a provenance audit showing the mechanism retrieved evaluation
answers, would weaken the improvement claim. Repeated rediscovery of discarded
lessons would weaken the durability claim.

## Reader checkpoint

Place an ACE playbook bullet, an MCE skill, and Meta-Harness optimizer code in
`Cₜ = (Wₜ, Hₜ, Dₜ, Rₜ)`.

<details>
<summary>Check your answer</summary>

The playbook bullet is primarily `Dₜ`. The context-construction and update
skill belongs mainly to `Hₜ`, with retrospective proposal logic in `Rₜ`.
Meta-Harness optimizer code changes the mechanism that proposes or selects
harness behavior and therefore spans `Hₜ` and `Rₜ`. The manifest should declare
the exact mutable surface instead of relying on these broad labels alone.

</details>
