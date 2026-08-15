---
id: rsi-system-aflow
kind: concept
title: AFlow MCTS over agentic workflows
summary: Code-represented agentic workflows, MCTS selection and expansion, execution feedback, author-reported evaluation, cost boundaries, and the evidence required for a recursive claim.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-evaluation-promotion-containment
related:
  - kind: compared-with
    target: rsi-harness-search
attachments:
  - content/source_registry.md
  - content/claim_evidence_ledger.md
  - content/systems/system_readings_index.md
claims: []
human_review: null
---

# AFlow: MCTS over agentic workflows

Mode: `TECHNICAL DEEP DIVE`.

> Read this after Weng's
> [workflow-design section](https://lilianweng.github.io/posts/2026-07-04-harness/#workflow-design).
> AFlow is useful because it makes workflow design an executable search
> problem. Its evidence supports bounded workflow optimization under the
> paper's evaluator. It does not establish recursive self-improvement.

## Problem and RSI relevance

**EVIDENCE — [AFLOW], §§3.1–3.2.** AFlow defines a workflow as LLM-invoking
nodes connected by executable edges. Each node can vary model, prompt,
temperature, and output format. The paper chooses code for edges because code
can express sequence, branching, loops, parallelism, and data flow.

**EVIDENCE — [AFLOW], §3.2.** The reported experiments narrow the nominal
search space. Model, temperature, and output format are fixed while the
optimizer changes prompts and code-represented control flow. Reusable operators
package common structures such as generation, formatting, review and revision,
ensembling, testing, and program execution.

**INFERENCE.** In RSI notation, AFlow edits part of harness state `Hₜ`. The
model weights `Wₜ`, task distribution, evaluator `E`, search budget `B`, and
promotion rule remain outside the candidate workflow. This makes AFlow a
concrete harness-search system, but a successful search run is not yet a
recursive improvement loop.

## Editable object and protected envelope

The candidate is a complete executable workflow:

`W = (P₁, …, Pₙ, E, O₁, …, Oₙ)`

where:

- `Pᵢ` is a prompt used by an LLM-invoking node;
- `E` is code that carries values and controls execution between nodes; and
- `Oᵢ` is an optional reusable operator.

**EVIDENCE — [AFLOW], Appendix A.3.** The starting workflow is a Python class
whose call method is intentionally unimplemented. The optimizer fills that
method with node and operator calls.

**EVIDENCE — [AFLOW], Appendix A.4.** The operator library includes
contextual and code generation, formatting, review, revision, ensemble, test,
and programmer operations. A generic custom operation remains available when
no predefined operator fits.

**INFERENCE.** The protected envelope should include the benchmark split,
evaluator implementation, model identities, API settings, root-tree budget,
and test records. If candidate code can modify these objects, the reported
score no longer isolates workflow quality.

## Workflow representation

AFlow uses two levels of “node,” which are easy to confuse:

1. A workflow contains LLM-invoking nodes.
2. An MCTS tree node represents one complete workflow.

Code is the edge representation. Therefore a branch in workflow code is not
the same thing as a child in the search tree. The former controls one
candidate's execution. The latter records a proposed modification and its
evaluation history.

**EVIDENCE — [AFLOW], §4.** Each search-tree entry retains the workflow,
score, parent modification, and experience propagated from evaluated
descendants. The paper argues that this tree preserves successful and failed
search experience better than a linear list of prior workflows.

## Algorithm

### Initialization

**EVIDENCE — [AFLOW], §4 and Appendix A.6.** AFlow splits the selected data
20:80 with random seed 42. It executes the blank workflow five times on the
validation portion, then keeps high-variance instances as the final validation
set. Appendix A.6 calls the 80% portion “training,” while §4 and §5.1 call it
the test set. This reading follows the main evaluation text and records the
appendix inconsistency rather than resolving it.

### Selection

The search samples a parent from the best `k` candidates with a mixture of
uniform and score-weighted probability:

`P(i) = λ/n + (1 − λ) exp(α(sᵢ − sₘₐₓ)) / ∑ⱼ exp(α(sⱼ − sₘₐₓ))`

`sᵢ` is a workflow score. The uniform term preserves exploration while the
softmax term favors higher-scoring parents.

**MISSING — parameter inconsistency.** The main text assigns `α = 0.4` and
`λ = 0.2`; Appendix A.6 assigns `λ = 0.4` and `α = 0.2`. The inspected paper
text does not reconcile the two.

### Expansion

**EVIDENCE — [AFLOW], §4 and Appendix A.1.** An optimizer LLM receives the
selected workflow, its ancestor experience, execution feedback, and the
operator set. It emits one code or prompt modification and a new complete
workflow. The prompt permits adding, changing, or deleting nodes and prompts,
but caps graph complexity at ten.

### Execution and evaluation

**EVIDENCE — [AFLOW], §4.** AFlow executes each candidate five times on the
validation set and records the mean, standard deviation, and cost. Multiple
runs reduce selection noise, but they multiply search-time model calls.

### Experience backpropagation

The search records:

- candidate performance;
- the modification from the parent; and
- whether the modification improved on that parent.

That experience propagates to the parent and becomes context for later
expansions. Scores also enter the global candidate record used for selection.
This is experience backpropagation, not gradient backpropagation through model
weights.

### Stopping and selection

**EVIDENCE — [AFLOW], Appendix A.6.** The reported configuration uses 20
rounds, top `k = 3`, five executions per candidate, and early stopping when the
top-k set remains unchanged for five rounds. The output is the highest-scoring
validation workflow found before the budget or stopping condition.

<details>
<summary>Original sources for this mechanism</summary>

- Problem formulation and search-space definition: [AFlow, §§3.1–3.2](https://openreview.net/forum?id=z5uVAKwmjf).
- Search loop and Algorithm 1: [AFlow, §4](https://openreview.net/forum?id=z5uVAKwmjf).
- Workflow template and operators: [AFlow, Appendices A.3–A.4](https://openreview.net/forum?id=z5uVAKwmjf).
- Detailed MCTS configuration: [AFlow, Appendix A.6](https://openreview.net/forum?id=z5uVAKwmjf).
- Checked-in text capture: `evidence/weng/text/aflow.txt`.

</details>

## Evaluation

### Setup

**EVIDENCE — [AFLOW], §5.1.** The paper evaluates six datasets:
HotpotQA and DROP with F1, HumanEval and MBPP with pass@1, and GSM8K plus a
level-five MATH subset with solve rate. It uses Claude 3.5 Sonnet as the
optimizer and tests several execution models. The main Table 1 comparison runs
all methods with GPT-4o-mini on the divided test sets and reports the mean of
three runs.

The baselines include direct invocation, chain of thought, self-consistency,
MedPrompt, MultiPersona Debate, Self-Refine, and ADAS. AFlow receives 20 search
rounds. The compared ADAS configuration receives 30.

### Main reported result

**EVIDENCE — [AFLOW], Table 1.** The AFlow row reports an 80.3 average across
the six metrics. The authors describe this as a 5.7% relative improvement over
the best compared manual-workflow average and a 19.5% relative improvement over
ADAS. The corresponding raw average gaps in the table are 4.3 and 13.1
percentage points. Because the columns use different metrics and datasets,
80.3 is an arithmetic summary of benchmark-specific percentages, not one
common probability of success.

### Transfer

**EVIDENCE — [AFLOW], Table 2.** HumanEval workflows searched with
GPT-4o-mini or DeepSeek-V2.5 are executed with four models. Most AFlow cells
exceed direct invocation, but the DeepSeek-searched workflow scores 90.8 with
GPT-4o-mini versus 94.7 for the GPT-4o-mini-searched workflow. The authors use
this as evidence that workflows transfer while still remaining model-specific.

**INFERENCE.** This is cross-model execution transfer on one benchmark, not
evidence that one workflow is universally model-agnostic.

### Cost

**EVIDENCE — [AFLOW], Figure 4 and Appendix D.** The paper plots HumanEval
test-execution cost, not total search cost. A workflow searched with
GPT-4o-mini and executed with DeepSeek reaches 93.9 pass@1 at $0.0291 in the
reported table, while direct GPT-4o reaches 93.89 at $0.6371. The paper
describes this as parity at 4.55% of the inference cost in dollars.

**INFERENCE.** This comparison supports deployment-time cost efficiency for a
selected workflow. It does not show end-to-end break-even after optimizer calls,
five validation executions per candidate, failed branches, and all 20 search
rounds are charged.

### Operator ablation and search trace

**EVIDENCE — [AFLOW], Figure 5.** On GSM8K, predefined operators speed search
and improve the best curve. The no-operator run still reports 93.1 and creates
an ensemble-like structure through custom nodes.

**EVIDENCE — [AFLOW], Figure 6.** The highlighted GSM8K path adds an ensemble
operator, adds programmer-based verification, and then modifies formatting and
reasoning prompts. The full tree also retains failed branches, including a
review node and an over-specific rephrasing attempt.

**INFERENCE.** The ablation shows that human-designed operators are useful
search priors, while custom code retains some discovery capacity. It does not
separate the effect of MCTS from the optimizer model, validation-instance
selection, or the larger number of candidate executions.

## Failure modes and limits

**EVIDENCE — scope stated by [AFLOW], §3.2 and Appendix F.** The main method
targets reasoning tasks with numerical evaluators. Appendix F proposes
LLM-as-judge evaluation for open-ended tasks, adding judge reliability and
human-preference dependence rather than removing the evaluator problem.

**INFERENCE — repeated-validation overfitting.** Twenty rounds repeatedly
select against a small, high-variance validation subset. A candidate may learn
formatting or instance-specific regularities that do not transfer.

**INFERENCE — evaluator exploitation.** Appendix C shows that formatting
changes can alter scores. This may be legitimate task compliance, but it also
shows how directly search can adapt to scorer behavior.

**INFERENCE — resource mismatch.** Comparing final workflow execution alone
does not match root-tree compute. A fair search comparison must count optimizer
calls, all candidate executions, retries, and failed workflows.

**INFERENCE — unsafe executable search.** The workflow representation includes
generated code and a programmer operator that executes code. A production
implementation needs sandboxing, timeouts, bounded permissions, immutable
evaluators, and durable operation records.

**MISSING.** The inspected paper does not provide an independent reproduction,
a delayed hidden benchmark after repeated search, a matched total-search-cost
comparison, or a dedicated limitations section.

## Claim ceiling

The strongest supported statement is:

> **EVIDENCE.** In the authors' six-benchmark setup, an optimizer LLM plus an
> MCTS variant searched code-represented workflows that achieved the reported
> Table 1 scores and yielded deployment-time cost/performance tradeoffs on
> HumanEval.

Do not strengthen that to any of these claims:

- AFlow independently reproduces across providers or implementations.
- MCTS alone caused the gain.
- The workflow transfers to arbitrary tasks or models.
- Search is cheaper end to end than manual design or stronger-model inference.
- AFlow improved its own workflow-improvement procedure.
- AFlow demonstrated recursive self-improvement.

## What would establish a recursive result

A later generation would need to persist a change to the workflow improver,
not merely to the task workflow. Under a matched external envelope, the changed
improver would then need to:

1. propose or select better future workflow modifications;
2. reach stronger protected-test outcomes at the same root-tree budget;
3. reproduce that advantage across fresh task families and seeds;
4. preserve evaluator, archive, and permission integrity; and
5. become the accepted parent for another measured generation.

Without that successor-production test, AFlow remains a strong example of
bounded harness search.

## Reading routes

- [[knowledge/rsi/weng/05-workflow-design-and-search|Weng: workflow design becomes a search problem]]
- [[knowledge/rsi/chapters/harness-search|Searching for better harnesses]]
- [[knowledge/rsi/chapters/evaluation-promotion-containment|Evaluation, promotion, and containment]]
- [Original paper](https://openreview.net/forum?id=z5uVAKwmjf)
- Checked-in paper text: `evidence/weng/text/aflow.txt`
