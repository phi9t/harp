# Harness layer versus core intelligence

## What Weng claims

**CLAIM — [WENG-HARNESS], “Harness Layer vs Core Intelligence?”** Near-term
self-improvement may occur in the harness around frozen model weights. Some
useful external procedures may later be internalized into model behavior,
while interfaces to tools and the environment remain external.

[Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#harness-layer-vs-core-intelligence).

## Mechanism

**INFERENCE.** Treat the model–harness pair as a compatibility contract.
External procedures can elicit, route, or extend model behavior without
changing `Wₜ`. Internalization changes `Wₜ` so that a behavior previously
dependent on an artifact or procedure survives when that support is removed.
The clean test runs the model with and without the procedure under matched
tasks and budgets.

RLM is useful here as a boundary example: recursive subcalls and externalized
context can create strong length and strategy generalization without changing
the underlying weights or demonstrating a self-improving successor loop.

The [Codex continuity companion](../codex_state_continuity_and_compaction.md)
adds a second boundary example: changing only whether a harness retains opaque
reasoning and semantically compacts old observations can materially change
deployed performance with the same model weights.

## Hidden assumption

**INFERENCE.** Better harness artifacts do not guarantee the consuming model
will discover, invoke, or follow them. Activation, adherence, and outcome are
separate capabilities. A harness can outrun its model and make the composite
less reliable.

## Demonstrated versus proposed

**EVIDENCE — [HARNESS-DISENTANGLE].** The inspected study separates harness
updating from harness benefit and reports activation and adherence failures.

**EVIDENCE — [RLM-PAPER].** The inspected RLM work supports harness-level
inductive bias and compositional generalization claims at fixed weights.

**MISSING.** Neither result proves that harness gains become weight-level
intelligence or that the composite recursively improves its improvement
operator.

## What would weaken this interpretation

If procedure ablation leaves behavior unchanged, the procedure was not
causally load-bearing. If a model reliably exploits every newly evolved
harness without compatibility adaptation, the proposed activation/adherence
boundary would be less important in that setting.

## Reader checkpoint

Design a four-cell evaluation that separates a new model from a new harness.

<details>
<summary>Check your answer</summary>

Compare `W_old × H_old`, `W_old × H_new`, `W_new × H_old`, and
`W_new × H_new` on the same held-out tasks and root-tree budget. Measure tool or
skill discovery, activation, sustained adherence, task outcome, and integrity.
Promote the co-versioned pair only after external checks pass.

</details>
