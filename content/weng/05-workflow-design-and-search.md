# Workflow design becomes a search problem

## What Weng claims

**CLAIM — [WENG-HARNESS], “Workflow Design.”** Expert-authored systems such as
AI Scientist coordinate research stages directly. ADAS searches agent designs
with a meta-agent. AFlow represents workflows as executable graphs and uses
Monte Carlo tree search to generate and evaluate workflow candidates.

[Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#workflow-design).

## Mechanism

**INFERENCE.** Workflow search requires four distinct contracts:

- a representation that can express model actions and deterministic logic;
- a mutation or proposal operator over that representation;
- an evaluator that executes candidates under matched tasks and budgets; and
- an archive or tree policy that decides which candidates receive more search.

AFlow's MCTS organizes selection, expansion, evaluation, and backpropagation
over workflow candidates. ADAS relies more directly on a meta-agent's proposed
agent code. Neither search procedure is itself the task solver; each searches
for a solver configuration.

## Hidden assumption

**INFERENCE.** The workflow evaluator is assumed to measure a reusable
mechanism rather than benchmark-specific overfitting, stochastic luck, or
additional model calls. Search can efficiently optimize the wrong objective.

## Demonstrated versus proposed

**EVIDENCE — [AI-SCIENTIST], [ADAS], [AFLOW].** The inspected sources support
the named pipeline and search mechanisms and their author-reported evaluations.
AFlow's claim ceiling includes its official-paper MCTS mechanism and reported
results, not independent reproduction or RSI efficacy.

**MISSING.** Better searched workflows do not establish that the accepted
workflow is a better searcher of its own successor.

## What would weaken this interpretation

If gains disappear under equal model-call cost, held-out task families, or a
fresh evaluator, the workflow may be a benchmark-specific allocation of extra
compute. If MCTS statistics cannot be replayed from archived evaluations, the
attribution chain is incomplete.

## Reader checkpoint

For AFlow, identify the search state, expansion operator, reward observation,
and claim ceiling.

<details>
<summary>Check your answer</summary>

The state is an executable workflow graph plus its search history. Expansion
uses an LLM to modify workflow code. Evaluation runs the candidate on task
instances, and backpropagation updates search statistics from observed scores.
The supported claim is bounded workflow-search improvement reported by the
authors; it is not independent replication and not a demonstration of
recursive self-improvement.

</details>
