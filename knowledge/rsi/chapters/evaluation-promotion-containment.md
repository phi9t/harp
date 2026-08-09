---
id: rsi-evaluation-promotion-containment
kind: concept
title: Evaluation, promotion, and containment
summary: Held-out evaluation, independent gates, multi-objective selection, budgets, permissions, lineage, rollback, and human authority.
primary_parent: mlsys
additional_parents:
  - infrastructure
  - modeling
related:
  - kind: governs
    target: rsi-recursive-improvement-loop
  - kind: contains
    target: rsi-harness-search
attachments:
  - content/rsi_harness_by_lil_log_deconstructed.md
  - content/source_registry.md
  - content/pi_harness_deep_dive.md
  - content/hermes_harness_deep_dive.md
  - content/codex_harness_deep_dive.md
  - content/evaluator_integrity_and_promotion.md
claims: []
human_review: null
---

# Evaluation, promotion, and containment

## Improvement needs external authority

An editable candidate may inspect traces, diagnose failures, and argue for a change. It cannot be the sole authority that decides whether the change succeeded, whether the resource comparison was fair, or whether the child becomes the next parent. Evaluation, promotion, and containment form the protected envelope around the improvement loop.

Use three independent hard gates:

`G_outcome = held_out_gain ∧ no_required_regression`

`G_integrity = immutable_evaluator ∧ complete_budget ∧ intact_lineage ∧ policy_compliance`

`G_authority = approved_actor ∧ valid_receipt ∧ rollback_ready`

Promotion requires `G_outcome ∧ G_integrity ∧ G_authority`. Task performance cannot compensate for evaluator tampering or unauthorized deployment.

## Held-out and multi-objective evaluation

Development tasks guide proposals. Held-out tasks estimate generalization. Delayed or out-of-distribution tasks detect repeated benchmark adaptation. The candidate must not read held-out inputs, answers, judge prompts, or score aggregation before submission.

For valid candidates, keep a score vector:

`J(C) = (S_task, Q_correct, M_maintain, U_successor, C_resource, I_risk)`

`S_task` is held-out outcome, `Q_correct` is invariant and regression quality, `M_maintain` is maintenance cost, `U_successor` is usefulness to a fresh later improver, `C_resource` is complete root-tree cost, and `I_risk` is observed integrity or authority risk. Select from a Pareto frontier or a predeclared lexicographic order. Do not hide regressions in one scalar.

## Evaluation and promotion flow

1. Freeze candidate, task split, evaluator, judge, runtime, permission, and budget revisions.
2. Validate the candidate's declared mutation boundary.
3. Run integrity probes before exposing protected outcome tasks.
4. Execute parent and child under matched root-tree resources.
5. Preserve all attempts, children, failures, timeouts, and cancellations.
6. Compute the score vector from evaluator-owned artifacts.
7. Apply outcome and integrity gates independently.
8. Ask an authorized human or service to approve promotion.
9. Seal parent, child, score, gate, and actor identities in the lineage archive.
10. Deploy through a reversible mechanism and verify the final state.
11. Roll back automatically or manually when post-promotion checks fail.

<details>
<summary>Original sources for this mechanism</summary>

- Weng, future challenges and evaluator boundary, [reader companion](../rsi_harness_by_lil_log_deconstructed.md). It identifies weak evaluators, reward hacking, long-term cost, negative results, diversity, and human roles.
- Weng, "Reward Hacking in Reinforcement Learning," sections "Reward misspecification" and "Reward tampering," and Amodei et al., "Concrete Problems in AI Safety," §4, establish proxy exploitation and evaluator-bias risks: [Lil'Log 2024-11-28](https://lilianweng.github.io/posts/2024-11-28-reward-hacking/), [arXiv:1606.06565v2](https://arxiv.org/abs/1606.06565).
- Self-Harness, §§3–4; Agentic Harness Engineering, §§2–4; and Darwin Gödel Machine, §§2–3, describe regression gates, observability-driven edits, sandboxing, and lineage with their stated limits: [arXiv:2606.09498v1](https://arxiv.org/abs/2606.09498), [arXiv:2604.25850v4](https://arxiv.org/abs/2604.25850), [arXiv:2505.22954v3](https://arxiv.org/abs/2505.22954).
- [Pi](../pi_harness_deep_dive.md), [Hermes](../hermes_harness_deep_dive.md), and [Codex](../codex_harness_deep_dive.md) provide pinned implementation examples for sessions, budgets, capability routing, and sandboxing.

</details>

## Budget accounting

Count the complete descendant tree. A root receipt includes model calls, tokens, cache use, wall time, concurrency, subprocesses, accelerators, external services, and money for every child and retry. Record both allowed budget and observed spend.

Parent and child comparisons need the same budget contract. If one candidate spends less and achieves the same outcome, cost is a valid gain. If one silently spawns more agents, the task score is confounded.

## Permissions and containment

Containment starts with capability design:

- explicit readable and writable paths;
- separate evaluator credentials and storage;
- network allowlists or network denial;
- tool registry with runtime permission checks;
- process, concurrency, and time limits;
- no candidate access to promotion credentials;
- fresh candidate environments;
- final-state verification after all actions.

Prompt instructions are not containment. The host or service boundary must reject a forbidden action even when the model strongly requests it.

## Lineage and rollback

The protected archive stores parent digest, candidate diff, model and harness revisions, task/evaluator identity, raw result locators, score vector, complete resource receipt, gate decisions, actor identity, and rollback target. Rejected and invalid branches stay in the archive.

Rollback is part of promotion readiness. A promoted child must identify the last known good parent and the mechanism that restores it. For irreversible model or data changes, promotion needs a shadow, canary, or versioned indirection rather than destructive replacement.

## Worked implementation examples

### Pi

Pi's parent-linked session entries and turn snapshots are useful lineage inputs. They become official archive evidence only when an external controller seals candidate, evaluator, and promotion receipts.

### Hermes

Hermes exposes iteration budgets, persistent memory, skill archives, and curator reports. Those controls support reversible adaptation, but curator judgment does not replace a held-out evaluator.

### Codex

Codex separates model-visible tool specifications from registered runtimes, shares root-tree control state, persists rollout lineage, and resolves platform sandboxing. It is a containment example, not proof of autonomous safe self-evolution.

### Durable workflow boundary

A durable workflow engine can persist timers, retries, signals, and step
history outside a model session. It should deliver evaluator-approved tasks,
not take over semantic retry or promotion decisions. Harp carries no source
that establishes one deployed workflow engine as the implementation of this
full promotion contract.

## Reward hacking and evaluator independence

Optimization will search the evaluator's weaknesses. Common attacks include modifying tests, leaking held-out labels, changing score normalization, manipulating model judges, omitting child costs, deleting failed branches, escalating tool authority, and damaging required output after a local check passes.

Use adversarial probes that attempt each attack. Missing write audits, incomplete descendant accounting, or unavailable archive linkage should fail closed. Evaluator independence is architectural, not a claim that a second model is unbiased.

## Long-term maintenance and human decisions

An accepted change can create future debt through opaque prompts, brittle memory, extra dependencies, slower execution, or harder diagnosis. Test successor usability by giving a fresh agent a bounded maintenance or failure-diagnosis task on parent and child under matched conditions.

Humans remain responsible for values that the experiment cannot reduce to a stable executable metric, for high-impact deployment, and for resolving evidence conflicts. The receipt should show which decisions were automatic and which person or role approved the irreversible boundary.

## Failure modes and tradeoffs

- **Held-out exhaustion.** Repeated selection turns the test set into training data.
- **Judge correlation.** Several evaluators share the same blind spot.
- **Budget omission.** Descendant or external-service work is not counted.
- **Policy bypass.** A hidden tool or credential path escapes the declared registry.
- **Archive rewrite.** Candidates can delete failures or alter ancestry.
- **Unauthorized promotion.** Passing checks automatically grants deployment authority.
- **Rollback theater.** A target is named but restoration has never been exercised.
- **Maintenance blindness.** Immediate score hides future debugging and migration cost.
- **Human rubber stamp.** Approval receives only a scalar score and no boundary evidence.

Stronger gates reduce false promotion but increase cost and may reject exploratory work. Use cheap development loops inside the candidate boundary and reserve protected evaluation for promotion candidates.

## What would weaken the mechanism

Containment claims weaken if adversarial tests cross the boundary, if repeated evaluations reveal held-out leakage, if resource reconciliation does not close, or if rollback cannot restore the verified parent. Promotion evidence weakens when independent reruns or delayed tasks reverse the result.

## Open technical questions

- How can held-out evaluation remain useful during long-running open-ended search?
- Which evaluator ensembles reduce shared blind spots rather than duplicating them?
- How should a promotion controller compare successor usability and immediate task gain?
- What cryptographic or service-level guarantees are practical for lineage archives?

<details>
<summary>Reference records and operational metadata</summary>

- [The evaluator deep dive](../evaluator_integrity_and_promotion.md) retains the exact source locators and the original Pi-first experiment design.
- Full source identities and claim ceilings are in [the source registry](../source_registry.md).
- No current source proves containment against a candidate that can redesign its evaluator, weight-update pipeline, and deployment authority.

</details>
