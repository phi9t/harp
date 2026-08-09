# SIA versus Continual Harness

## Observe

SIA routes between harness and weight updates with a controller. Continual
Harness uses online states, teacher relabeling, process rewards, and policy
updates that propagate without resetting the environment.

## Predict

Identify which parts of `W_t` and `H_t` change, how data is produced, who owns
the evaluator, and which confounds prevent a recursive claim.

## Compare

Both systems are `joint-harness-weight-adaptation`. SIA uses controller-selected
actions; Continual Harness emphasizes on-policy co-learning. Neither source
establishes an accepted successor that improves later successor production.

## Explain

[Joint harness and model-weight adaptation](../chapters/joint-harness-weight-adaptation.md)
separates weight change, harness change, data-policy change, and model-harness
distribution shift.

## Missing fact

A stronger claim needs matched parent-child generations, protected promotion,
complete resource accounting, and positive next-cycle gain on fresh tasks.

## Transfer

Fork [SIA](../../../content/diagnostics/cases/sia.json) or
[Continual Harness](../../../content/diagnostics/cases/continual-harness.json), then vary the
editable components and evidence status.
