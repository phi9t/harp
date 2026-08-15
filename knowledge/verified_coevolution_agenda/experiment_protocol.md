---
id: verified-coevolution-experiment-protocol
title: Verified coevolution agenda - experiment protocol
type: protocol
mode: PRE-REGISTERED PROTOCOL
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [recursive-self-improvement, experiments, coevolution, verification]
confidence: medium
---

# Verified coevolution agenda experiment protocol

Mode: `PRE-REGISTERED PROTOCOL`.

This protocol turns the agenda into falsifiable studies. It does not report
results. Every study must freeze evaluator identity and promotion authority,
count full root-tree resources, archive rejected branches, reserve delayed
holdouts, and record rollback targets before optimization starts. See
[VCA-017](claim_evidence_ledger.md#vca-017-experiment-protocol-boundary).

## Common controls

- Use the same base model, starting harness, task distribution, evaluator, and
  promotion authority across all arms unless the arm definition explicitly
  changes one of them.
- Account for search rollout compute, training compute, evaluator calls,
  verifier calls, failed branches, rejected branches, repair attempts,
  distillation runs, and human review.
- Archive every candidate, rejection reason, evaluator input, evaluator output,
  accepted artifact, rollback target, and delayed-holdout result.
- Keep the proposer adaptive only where the thrust defines adaptive pressure.
- Report endpoint score, held-out transfer, worst-case regression, calibration,
  evaluator disagreement, diversity, and cumulative-risk metrics.

## Thrust A: Model-harness coevolution

**Question.** Does alternating or joint model-harness adaptation beat the better
single-surface baseline after equal resource accounting?

| Arm | Model weights | Harness | Promotion |
|---|---|---|---|
| Harness-only | Frozen | Optimized | Frozen evaluator and authority |
| Weights-only | Optimized | Frozen | Frozen evaluator and authority |
| Alternating | Optimized every other phase | Optimized in the alternate phase | Frozen evaluator and authority |
| Joint | Candidate may include both weight and harness changes | Candidate may include both weight and harness changes | Frozen evaluator and authority |

Primary outcome: delayed-holdout utility after full root-tree resource
normalization. Secondary outcomes: transfer to new task families, rollback
success, evaluator disagreement, and component-level attribution.

## Thrust B: Search-to-skill distillation

**Question.** Which search artifact best internalizes reusable capability into a
model while preserving auditability?

Compare distillation from:

| Condition | Distillation input | Expected failure mode |
|---|---|---|
| Raw traces | Complete successful search traces | Learns incidental scaffolding and hidden evaluator cues. |
| Stripped traces | Success traces with search metadata removed | Loses causal recovery information. |
| Reusable skills | Curated skill abstractions | Over-compresses context and misses edge cases. |
| Failure-recovery pairs | Failed branch, repair, and successful continuation | May learn recovery rituals rather than task skill. |

Measure reduced-search performance, search-node reduction, transfer to unseen
environments, ability to act without original harness hints, and auditability of
the learned procedure.

## Thrust C: Recursive-depth stability

**Question.** How many accepted generations can improve held-out capability
without crossing a risk or regression threshold?

Run 20 to 50 generations. Report every generation, not only the best endpoint.
Track:

- held-out gain;
- worst-case regression;
- calibration;
- evaluator disagreement;
- output and lineage diversity;
- cumulative risk;
- distance from external data anchors; and
- rollback compatibility.

Define stable recursive depth as the maximum depth where each accepted
generation improves the registered target while risk remains below the
predeclared threshold and delayed holdouts do not regress beyond tolerance.

## Thrust D: Archive diversity and comparative evolution

**Question.** Do archive retention and comparative or recombined evolution
produce better descendants than a single greedy lineage?

Compare:

| Arm | Lineage policy | Required measurement |
|---|---|---|
| Single lineage | Keep only the current best candidate | Endpoint score and regressions. |
| Archive retention | Preserve diverse accepted and near-miss candidates | Recovery from local optima and diversity. |
| Comparative/recombined evolution | Compare lineages and create hybrid candidates | Descendant productivity and transfer. |

Descendant productivity is required: measure whether an intermediate candidate
is good at producing better descendants, not only whether it scores well now.

## Thrust E: Adaptive-proposer gate comparison

**Question.** Which gates remain useful when the proposer receives partial
feedback and adapts to the gate?

Compare:

- learned classifier;
- ensemble classifier;
- behavioral regression suite;
- executable verifier;
- symbolic verifier;
- certified parameter region; and
- hybrid gate.

Measure accepted utility, false acceptance, false rejection, coverage,
verification cost, evaluator disagreement, and cumulative risk. When a gate has
a certified region, separate encoded-property preservation from semantic
alignment. When a learned gate performs well, test whether the proposer can
learn to exploit its features.

## Reporting format

Every report must include:

- immutable task and evaluator identity;
- full resource ledger;
- accepted and rejected branch archive digest;
- rollback target and rollback test result;
- delayed-holdout schedule and result;
- per-generation metric table;
- claim ceiling for each conclusion; and
- explicit failures and missing evidence.
