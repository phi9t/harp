# ADAS versus AFlow

## Observe

ADAS uses a meta-agent and archive to synthesize agent programs. AFlow treats
complete executable workflows as MCTS search nodes and expands them with an
optimizer model.

## Predict

Name the search state, parent-selection policy, expansion operator, evaluator,
and total resource boundary for each system.

## Compare

Both systems are `harness-improvement`. Their method families differ:
archive-based agent-program search for ADAS and MCTS workflow search for AFlow.
Search organization does not change the primary classification.

## Explain

The full [AFlow reading](../systems/aflow.md) distinguishes workflow nodes from
MCTS tree nodes and explains why final-workflow cost is not total search cost.
[Harness search methods](../concepts/harness-search-methods.md) owns the wider
search taxonomy.

## Missing fact

Neither system shows that a selected workflow becomes a better producer of
later selected workflows under a matched root-tree budget.

## Transfer

Fork [ADAS](../../../content/diagnostics/cases/adas.json) or
[AFlow](../../../content/diagnostics/cases/aflow.json), then change evaluator ownership or
root-tree accounting.
