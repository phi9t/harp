---
id: rsi-evaluation-and-control
kind: concept
title: Evaluation and control mechanisms
summary: Held-out tests, multi-objective selection, evaluator independence, reward hacking, root-tree budgets, and rollback.
primary_parent: rsi-evaluation-promotion-containment
additional_parents: []
related: []
attachments:
  - content/chapters/evaluation-promotion-containment.md
claims: []
human_review: null
---

# Evaluation and control mechanisms

An improvement loop needs a protected boundary that measures candidates, accounts for their resources, and records promotion without granting candidates authority over those decisions.

## Held-out evaluation

Held-out evaluation uses tasks or data excluded from proposal, training, and development feedback. The split and evaluator revision must stay fixed for parent-child comparison. Repeated adaptive querying can leak the holdout even without direct file access, so query counts and promotion cadence matter.

## Multi-objective selection

Multi-objective selection compares capability, reliability, cost, latency, security, and maintenance rather than collapsing every concern into one scalar too early. A candidate may be Pareto-improving, dominated, or acceptable only behind a constraint. Promotion policy must state which tradeoffs require human authority.

## Evaluator independence

Evaluator independence means the candidate cannot rewrite tests, judges, aggregation, integrity checks, or official results. The proposer and evaluator may use related models, but authority and data paths remain separate. Shared model biases still limit epistemic independence and must be disclosed.

## Reward hacking

Reward hacking occurs when a candidate increases the measured reward without producing the intended outcome. Common routes include exploiting judge preferences, altering execution conditions, hiding failures, and spending uncounted resources. Independent integrity gates test the measurement process as well as the score.

## Root-tree budget

A root-tree budget charges every descendant model call, tool action, retry, subagent, and evaluation spawned by one candidate attempt. Without complete accounting, a search method can appear better by silently using more compute or parallel branches.

## Lineage and rollback

Lineage records every proposed candidate, parent, evaluation, rejection, and accepted edge. Rollback restores a prior accepted candidate and its compatible data and schema, not just one code file. Rejected branches remain available for audit and duplicate-work prevention.

<details>
<summary>Original sources for this mechanism</summary>

- [Evaluation, promotion, and containment](../chapters/evaluation-promotion-containment.md#evaluation-and-promotion-flow) defines the protected gate sequence.
- Amodei et al., "Concrete Problems in AI Safety," §4, frames reward hacking and scalable oversight: [arXiv:1606.06565v2](https://arxiv.org/abs/1606.06565).
- Weng, "Reward Hacking in Reinforcement Learning," sections "Reward misspecification" and "Reward tampering": [2024-11-28](https://lilianweng.github.io/posts/2024-11-28-reward-hacking/).

</details>
