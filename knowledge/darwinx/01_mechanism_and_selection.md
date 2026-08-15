---
id: darwinx-mechanism-and-selection
title: DarwinX mechanism and selection
type: technical-deep-dive
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [darwinx, population-search, selection, recombination]
confidence: high
---

# DarwinX mechanism and selection

DarwinX is an outer loop for harness search. The model proposes and runs work,
but its weights do not change. Prompts, memory, tools, control flow, skills,
and agent-loop code do.

**[EVIDENCE - DX-001](claim_evidence_ledger.md#dx-001-darwinx-edits-the-harness-while-holding-model-weights-fixed).**
The paper splits the editable harness into a skill layer and a code layer. The
skill layer contains prompts, memory, and distilled knowledge. The code layer
contains tools, control flow, and the agent loop. This is broad harness
editing, but it does not include the selector, benchmark, evaluator, model
weights, or deployment authority.

## System state

A useful reconstruction of one archive node is:

```text
v = (
  complete harness snapshot,
  edit relative to parent,
  parent identity,
  per-task avg@k estimates,
  trial evidence,
  distilled lessons,
  solved-task signature,
  steering status
)
```

**[EVIDENCE - DX-002](claim_evidence_ledger.md#dx-002-the-archive-stores-executable-lineages-plus-evidence).**
The paper explicitly gives the first six fields. Solved-task signatures appear
in the inheritance rules. Steering status follows from the separate
confirmation gate. That last distinction matters.

An archive node can exist without being allowed to direct search:

```text
proposed
  -> evaluated
  -> archived as evidence
  -> provisionally promoted
  -> preservation-probed and confirmed
  -> steering eligible
```

The paper sometimes uses "kept," "retained," "promoted," and "may become an
ancestor" near one another. They should not be collapsed. DarwinX wants failed
and globally weak variants as historical evidence, but only confirmed variants
belong in the trusted steering set.

## Preserve-and-extend fitness

For a child `c`, parent `p`, and task `t`, the paper defines:

```text
Δ_t = p_hat_t(c) - p_hat_t(p)
g(c) = sum_t Δ_t
R(c) = sum_t max(-Δ_t, 0)
```

The fitness gate is:

```text
g(c) > 0
R(c) <= δ
```

**[EVIDENCE - DX-003](claim_evidence_ledger.md#dx-003-the-fitness-gate-permits-bounded-measured-regression).**
`g` measures net estimated improvement. `R` measures the total estimated
downside without letting gains cancel losses. A candidate needs positive net
gain and regression mass below the tolerance.

This is not strict monotonicity.

If one task gains `0.4` and another loses `0.1`, then:

```text
g = 0.3
R = 0.1
```

The child is eligible when `δ >= 0.1`, even though one measured capability
regressed. Finite `avg@k` adds another problem: a task can appear preserved
because the sample missed a failure mode.

**[INFERENCE - DX-030](claim_evidence_ledger.md#dx-030-darwinx-offers-no-formal-capability-monotonicity-guarantee).**
DarwinX implements a noisy bounded-regression policy. It does not prove that a
child preserves every behavior its parent had.

## Two-speed selection

The reasoned verifier reads the child's aggregate gain, regression mass, trial
evidence, and shared population memory. It returns `promote` or `revert`. A
promoted child still needs a higher-fidelity retest and a preservation probe
before it can steer later search.

**[EVIDENCE - DX-004](claim_evidence_ledger.md#dx-004-exploration-admission-and-steering-promotion-use-different-evidence).**
This separates two decisions:

| Decision | Bias | Purpose |
|---|---|---|
| Exploratory promotion | Higher recall | Keep search moving when evidence is noisy |
| Steering confirmation | Higher precision | Prevent lucky variants from redirecting later search |

The split is sensible. An `avg@5` full-suite test for every speculative edit
would spend most of the budget rejecting weak ideas. A pass@1-only search would
let noise compound. DarwinX screens cheaply, then spends more evidence on
variants that may direct future work.

The paper does not expose the reasoned verifier's prompt, complete model
configuration, or calibration data. Its judgment is therefore part of the
unreproduced optimizer, not an independently audited authority.

## Parent selection

Each child accumulates lineage gain:

```text
G(c) = G(parent(c)) + g(c)
```

The next parent comes from an exploit/broaden mixture:

```text
p* ~ (1 - β) point_mass(argmax_{v in S} G(v))
     + β Broaden(P)
```

`S` is the confirmed steering set. `P` is the wider population.

**[EVIDENCE - DX-005](claim_evidence_ledger.md#dx-005-parent-selection-mixes-cumulative-gain-exploitation-with-broadening).**
The paper argues that cumulative gain is more comparable than raw score because
variants may be screened on different task subsets.

That argument is incomplete. Cumulative gain still adds estimates produced on
different subsets, at different stages, and with different uncertainty. It can
double-count correlated improvements. A hard subset may produce a small
measured gain that matters more than a large gain on an easy subset, but `G`
has no explicit difficulty or variance adjustment.

The preservation probes reduce the damage from a bad ranking. They do not make
`G` an unbiased estimate of general capability.

## Learning signals

DarwinX uses three evidence modes:

| Signal | Available evidence | Intended use |
|---|---|---|
| Failure-derived | Failed target-agent trajectory | Localize a missing capability |
| Teacher-derived | Successful reference-solver trajectory | Distill a procedure when the target has no success |
| Self-derived | Passing and failing target-agent samples | Find behavior that makes success reliable |

**[EVIDENCE - DX-007](claim_evidence_ledger.md#dx-007-mutation-uses-failure-teacher-and-self-contrast-evidence).**
All three produce harness edits. None trains the target model.

The phrase "no gold solutions" has a narrow meaning. The loop does not consume
benchmark reference answers. It can still consume successful teacher
demonstrations. Mutation is guided by diagnosis and examples, not blind genetic
variation.

The paper partitions tasks into:

- reliable solves;
- variance-band tasks with both passing and failing samples; and
- walls with no successful target-agent rollout.

Failure evidence is the default. Self-contrast augments variance-band tasks.
Teacher trajectories augment walls.

## Shared memory

**[EVIDENCE - DX-008](claim_evidence_ledger.md#dx-008-shared-memory-aggregates-cross-task-failure-themes).**
After evaluation, a classifier labels failures such as setup timeout,
wrong output, and tool error. An aggregator updates shared memory:

```text
K_{g+1} = Agg(K_g, worked, regressed, themes)
```

Both proposer and reasoned verifier read `K`.

This is how DarwinX attempts to turn repeated failures into reusable edits.
For example, if setup cost dominates timeouts, the proposer can add a setup
procedure instead of patching one benchmark item.

The design introduces another unmeasured component. Theme classification,
aggregation, retention, and prompt placement can all change proposal quality.
The paper gives an interface and examples, not enough detail to reproduce the
memory policy.

## Population roles

DarwinX classifies variants by solved-task sets:

| Class | Relation to parent | Inheritance status |
|---|---|---|
| Improver | `S(c)` strictly contains `S(p)` | Eligible |
| Neutral | `S(c) = S(p)` | Eligible |
| Stepping stone | `S(c)` is a strict subset of `S(p)` | Lessons only |
| Archived tradeoff | Gains and loses tasks | Lessons only |
| Specialist | Solves a task no sibling solves | Potential merge source if preservation holds |

This table reveals a tension in the prose. The broad archive keeps every
evaluated variant as evidence. Recombination uses the narrower set that
preserves inherited solves. A globally weaker specialist can contribute only
if it remains eligible under the inheritance rule or its useful edit can be
recovered through another candidate.

## Recombination

For variants with a common ancestor `H_0`, the paper writes:

```text
H = H_0
    + Δ_code
    + Δ_skill
    + Δ_prompt
    + Δ_tool
```

The merged child survives only if:

```text
S(child) contains union_i S(v_i)
```

**[EVIDENCE - DX-006](claim_evidence_ledger.md#dx-006-recombination-uses-a-stricter-solved-set-rule-than-the-fitness-gate).**
This union rule is stronger than the ordinary tolerance-based fitness gate.
It prevents a high average score from buying away one source variant's unique
win.

The rule is expensive. As the union grows, every merge needs a broader
regression suite. It also leaves key engineering questions unanswered:

- How are overlapping code edits merged?
- What happens when two skills encode incompatible policies?
- How is a common ancestor chosen when lineages have multiple shared points?
- How many source variants may be merged?
- Does the merge proposer see protected task identities or only behavior
  descriptors?
- How are flaky solved sets treated?

These are implementation questions, not cosmetic details. They determine
whether recombination means additive inheritance or another unconstrained LLM
rewrite.

## The protected envelope

The paper's causal claim is cleanest when written as:

```text
frozen:
  model weights
  benchmark tasks and verifiers
  selector and archive authority
  proposer/analyzer/verifier orchestration
  evaluation policy

editable:
  prompts and memory
  skills and distilled notes
  tools
  control flow
  agent-loop source code
```

The fixed model makes performance deltas attributable to the full harness
change. It does not attribute them to any one DarwinX operator.

**[MISSING - DX-010](claim_evidence_ledger.md#dx-010-core-optimizer-details-needed-for-reproduction-are-absent).**
The paper leaves `β`, `δ`, model settings, prompts, subset allocation, and merge
conflict resolution unspecified. No public optimizer implementation was found
in the bounded search.

## Mechanism verdict

The paper defines a coherent selection architecture:

```text
guided proposal
  -> cheap measured screen
  -> bounded-downside judgment
  -> archive retention
  -> high-fidelity confirmation
  -> steering eligibility
  -> optional specialist merge
```

Its strongest design idea is not mutation. Frontier agents already propose
harness edits. The contribution is a policy for retaining alternatives and
deciding which measured changes may influence later search.

The central unresolved issue is attribution. The complete system changes
proposal context, memory, selection, inference effort, archive structure, and
sometimes action policy. The experiments do not isolate those parts.

Continue with the [evaluation audit](02_evaluation_audit.md) or return to the
[DarwinX index](darwinx_index.md).
