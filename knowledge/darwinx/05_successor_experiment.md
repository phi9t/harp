---
id: darwinx-successor-experiment
title: DarwinX successor experiment
type: research-design
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [darwinx, ablation, experiment-design, fixed-budget]
confidence: medium
---

# DarwinX successor experiment

**[[knowledge/darwinx/claim_evidence_ledger#DX-033: A fixed-budget factorial study is the next causal test|INFERENCE - DX-033]].**
The next useful DarwinX experiment should isolate selection operators under one
fixed root-tree budget. Another larger leaderboard run would show scale, not
causality.

## Primary question

Does population structure improve held-out harness capability per unit of total
search compute after controlling for regression probes, historical evidence,
and candidate generation?

The estimand is:

```text
expected protected held-out capability
per root-tree rollout token
```

Root-tree accounting includes every descendant attempt, rejected candidate,
confirmation rollout, teacher trajectory, proposer call, verifier call, retry,
and final evaluation.

## Arms

| Arm | Regression gate | Archive | Recombination | Full-history proposer |
|---|---:|---:|---:|---:|
| Greedy | No | No | No | No |
| Preserve-only | Yes | No | No | No |
| Archive | Yes | Yes | No | No |
| DarwinX-selection | Yes | Yes | Yes | No |
| Full system | Yes | Yes | Yes | Yes |

### Greedy

Maintain one incumbent. Accept a child when its development objective improves.
Do not protect old solves beyond the scalar objective.

This arm measures ordinary iterative harness optimization.

### Preserve-only

Maintain one incumbent. Apply the same regression mass, reasoned-verifier
contract, confirmation schedule, and protected probes as the DarwinX arms.

This isolates the gate from the population.

### Archive

Retain alternative confirmed lineages and use the same parent-selection policy.
Do not merge edits.

This isolates archive retention and broadening.

### DarwinX-selection

Add specialist detection and recombination. Keep proposal context compressed to
the same summary available in the first four arms.

This measures population plus merge without full-history diagnosis.

### Full system

Expose source, scores, traces, and rejected candidates through a bounded
filesystem interface. Keep selection identical to the DarwinX-selection arm.

This measures whether richer historical diagnosis adds value.

## Fixed components

Every arm must share:

- one frozen base model and model snapshot;
- one initial harness;
- one candidate edit schema;
- the same proposer model and sampling settings;
- the same task environments and verifier revisions;
- the same development and held-out splits;
- the same tool and network policy;
- the same candidate timeout;
- the same maximum edit size;
- the same total root-tree rollout-token budget;
- the same wall-clock and concurrency ceiling;
- the same infrastructure retry policy; and
- the same final promotion suite.

Do not let the population arm generate more proposals simply because it has
more parents. Candidate count is part of the fixed search budget.

## Data split

Use four disjoint evaluation roles:

| Split | Visible to proposer | Visible to selector | Purpose |
|---|---:|---:|---|
| Development | Yes, through bounded traces | Yes | Candidate generation and cheap screens |
| Protected promotion | No raw task or answer bytes | Yes, score only | Regression and deployment promotion |
| Delayed held-out | No | No until search ends | Primary generalization outcome |
| Integrity and cost canaries | Policy only | Binary or typed outcome | Detect evaluator abuse and resource regressions |

The protected promotion suite should contain:

- tasks solved by the parent;
- tasks unique to each specialist;
- security and authority probes;
- latency and token limits; and
- consumer-interface checks such as import, schema, or tool registration.

The delayed held-out set must remain inaccessible to candidate and selector
processes until every run has stopped.

## Search budget

Precommit one budget in at least three units:

```text
total model input and output tokens
total task-environment seconds
total wall-clock time at fixed concurrency
```

Dollar cost can be a reported conversion. It should not be the only budget
because provider prices can change during the study.

Each arm gets the same:

- proposer-token allowance;
- task-rollout-token allowance;
- teacher-solver allowance;
- confirmation allowance; and
- final-evaluation allowance.

If an arm does not use one category, it cannot silently reallocate those tokens
unless the reallocation rule was preregistered for every arm.

## Candidate protocol

Each candidate must declare:

```json
{
  "target_capability": "stable identifier",
  "parent": "archive node or incumbent",
  "changed_paths": ["relative/path"],
  "edit_kind": ["prompt", "skill", "tool", "control_flow", "code"],
  "predicted_improvements": ["task family or behavior descriptor"],
  "protected_behaviors": ["probe identifiers"],
  "expected_cost_change": {
    "tokens": "signed estimate",
    "latency": "signed estimate"
  }
}
```

The controller, not the proposer, validates:

- exact write paths;
- syntax and schema;
- import or registration behavior;
- symlink and traversal safety;
- mutation-size limit; and
- absence of protected task identifiers.

Invalid candidates consume proposal budget. Otherwise an arm can appear better
by ignoring producer-side failures.

## Sequential evidence allocation

Fixed `avg@3` and `avg@5` are simple but wasteful. Use a preregistered racing
policy:

1. run one sample on the target development tasks and a small protected probe;
2. stop candidates with impossible positive gain under a conservative bound;
3. allocate more samples to candidates near the decision boundary;
4. run the full protected suite only for deployment candidates; and
5. reserve a fixed final sample count for cross-arm comparison.

A Beta-Binomial model is acceptable for one binary task measured repeatedly.
It is not sufficient for the aggregate by itself because task outcomes are
correlated.

Use one of:

- a hierarchical model with task-family random effects;
- a block bootstrap over preregistered task clusters; or
- task-level posterior estimates followed by a conservative union bound for
  protected regressions.

Simulate the decision rule before the main experiment. Measure false promotion,
false rejection, and sample use under realistic task correlations.

## Promotion rule

For candidate `c` and parent `p`, define:

```text
P(net development gain > 0 | data) >= 0.95
P(protected regression mass > δ | data) <= 0.05
```

Deployment promotion also requires:

```text
no integrity canary failure
no consumer-contract failure
cost and latency inside preregistered limits
```

Archive admission and deployment promotion remain separate. A candidate may
stay as research evidence without being executable in later rounds.

## Diversity descriptors

Do not use task identity as the only diversity measure. It makes the archive a
benchmark lookup table.

Record behavior descriptors such as:

- tool strategy;
- verification depth;
- retry pattern;
- decomposition depth;
- context-retention policy;
- action-policy class;
- latency and token profile; and
- failure category repaired.

A descriptor enters parent selection only after showing repeatable measurement.
Store raw traces so later audits can check whether two syntactically different
edits produce the same behavior.

## Recombination protocol

A merge proposal must declare:

- common ancestor;
- source variants;
- unique protected wins from each source;
- conflicting paths;
- conflict-resolution method; and
- expected combined cost.

Test merges in stages:

1. consumer and syntax checks;
2. unique-win probes for each source variant;
3. parent solved-set probes;
4. integrity and cost canaries; and
5. delayed held-out evaluation only after search ends.

Track three merge outcomes separately:

```text
syntactic merge success
behavioral union success
held-out benefit
```

Conflating them makes recombination look better than it is.

## Outcomes

### Primary

```text
protected delayed-held-out pass rate
at the fixed root-tree rollout-token budget
```

### Secondary

- area under best-confirmed-held-out-proxy versus search-token curve;
- protected regression mass;
- integrity failure rate;
- total and marginal inference cost;
- candidate validity rate;
- accepted edits per million search tokens;
- archive behavioral coverage;
- merge union-success rate;
- final harness token and latency overhead;
- repeatability across random seeds; and
- performance after a base-model swap.

Do not use the delayed held-out set to compute best-so-far curves during search.
Use the protected promotion proxy and report its correlation with held-out only
after all runs finish.

## Replication

One evolutionary run is not enough. Use several independent seeds for:

- proposer sampling;
- rollout stochasticity;
- task order;
- infrastructure placement; and
- archive parent sampling.

Analyze seed as a random effect. Report all runs, not only the best archive.

If budget prevents five arms with several seeds, reduce task count or mutation
space before reducing replication. A single expensive run cannot separate an
algorithm from luck.

## Predeclared failure modes

The study fails to answer its primary question if:

- any arm reads delayed held-out data during search;
- arms receive different proposer models or token budgets;
- invalid candidate attempts are excluded from cost;
- infrastructure retries differ by arm;
- final harnesses use different inference limits;
- the selector can inspect protected answers;
- merges receive an extra evaluation budget;
- only best seeds are reported; or
- task revisions differ across runs.

## Required artifacts

Publish:

- source revision for every harness and controller;
- exact model identities and sampling settings;
- task, verifier, and environment revisions;
- split manifests with delayed-held-out encryption or access receipts;
- candidate diffs;
- proposer, analyzer, and verifier prompts;
- per-trial outcomes and typed failure classes;
- token, latency, retry, and cost ledger;
- archive graph and parent-selection decisions;
- sequential-test state at every decision;
- merge conflict and union-test receipts;
- final harness snapshots; and
- one command that recomputes every table.

Redact secrets, not evidence structure.

## Decision criteria

Population search earns its complexity if the Archive or DarwinX-selection arm:

1. improves delayed held-out performance over Preserve-only;
2. does so at the same root-tree budget;
3. does not increase protected regression or integrity failures;
4. repeats across seeds; and
5. produces useful behavioral diversity rather than duplicate edit histories.

Recombination earns a separate claim only if DarwinX-selection exceeds Archive
and the positive difference traces to merges that pass both behavioral-union
and held-out tests.

Full-history proposal earns a separate claim only if Full system exceeds
DarwinX-selection without protected leakage.

## After the study

If population search wins, repeat the experiment after:

- swapping the base model;
- delaying held-out evaluation in time;
- changing the verifier implementation; and
- removing the top discovered skill family.

These tests distinguish durable procedures from one model's quirks and one
benchmark's evaluator.

Return to the [[knowledge/darwinx/03_critical_review|critical review]] or the
[[knowledge/darwinx/darwinx_index|DarwinX index]].
