---
id: dgm-successor-design
title: DGM successor experiment design
type: deep-dive
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, successor-improvement, experiment-design, evaluation]
confidence: high
canonical: ../rsi/chapters/recursive-improvement-loop.md
---

# DGM successor experiment design

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## Learning outcomes

After this chapter, you should be able to:

- specify a direct parent-versus-child next-cycle experiment;
- define immutable candidate, evaluator, archive, and budget boundaries;
- design content-addressed lineage and derived evaluation artifacts;
- choose primary successor-improvement metrics and stopping rules; and
- interpret positive, null, negative, and mixed outcomes without overclaiming.

## Objective

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-070: A stronger successor should separate fitness, productivity, and diversity|INFERENCE - DGM-070]].**
Test the claim DGM currently treats as a proxy:

> Does a child agent that scores better on coding tasks become a better
> producer of later valid agent improvements than its parent?

The experiment compares parent and child as *improvers*, not only as task
solvers.

## Primary hypothesis

For a parent candidate \(P\) and its accepted child \(C\), under one frozen
protected envelope:

$$
\Delta_{\text{next}}(C) >
\Delta_{\text{next}}(P),
$$

where:

$$
\Delta_{\text{next}}(X) =
\mathbb{E}\left[
U\left(\operatorname{bestValidChild}(X)\right)-U(X)
\right].
$$

\(U\) is a precommitted multi-objective utility over held-out task outcome,
integrity, cost, and successor usability.

The primary estimand is not the best lucky descendant. It is the expected
valid improvement yield under a fixed number of attempts and a complete
root-tree budget.

## Secondary hypotheses

1. Child \(C\) produces functioning descendants at a higher rate than parent
   \(P\).
2. Descendants of \(C\) have higher held-out outcome utility.
3. \(C\)'s gain persists after matching model calls, tokens, wall time, and
   dollars.
4. \(C\)'s descendants remain understandable and modifiable by a fresh
   successor.
5. The improvement is not explained only by the external diagnostic model.
6. The improvement transfers to a held-out benchmark family.

## Experimental unit

The unit is a matched parent–child pair:

```text
pair_id:
parent_candidate_manifest:
child_candidate_manifest:
child_creation_receipt:
search_generation:
selection_reason:
```

Sample pairs across:

- early, middle, and late generations;
- positive, flat, and negative immediate score changes;
- different discovered mechanism classes;
- multiple independent search runs; and
- at least two seed harness families.

Do not select only the final best lineage. That would condition the experiment
on later success.

## Protected envelope

Freeze these inputs for each parent–child comparison:

| Surface | Fixed value |
|---|---|
| Diagnostic model | One immutable model snapshot and decoding contract |
| Coding model | One immutable model snapshot and decoding contract |
| Proposal evidence | Matched failure records with protected answers removed |
| Improvement-task distribution | Precommitted sample |
| Benchmark training tasks | Identical across P and C |
| Promotion tasks | Hidden and disjoint |
| Evaluator code | Immutable digest |
| Parent selector | Not used inside pairwise trial |
| Archive policy | Not used inside pairwise trial |
| Attempts per candidate | Equal |
| Root-tree token budget | Equal |
| Root-tree wall-time budget | Equal |
| Dollar budget | Equal |
| Container image | Immutable digest |
| Network and permissions | Identical and externally enforced |
| Human intervention | Predeclared and symmetric |

The candidate may edit only its declared agent repository.

## Candidate package

The candidate is one immutable manifest plus one reconstructable source tree:

```text
candidate package =
  seed revision
  + ordered patch digests
  + source-tree digest
  + model/tool contract
  + dependency lock
  + parent edge
```

The package does not include evaluator code, hidden tasks, archive authority,
or active credentials.

## Evaluator package

The evaluator is separately versioned and immutable during a trial:

```text
evaluator package =
  task split identities
  + hidden tests
  + metric implementation
  + integrity probes
  + resource accountant
  + promotion policy
  + package digest
```

Candidates receive only the declared public interface. The evaluator package is
mounted read-only or invoked across a capability boundary.

## Task partitions

Use four disjoint partitions.

### Search-development tasks

Used to create the original parent–child lineage. They are historical context
and cannot be reused as hidden evidence.

### Improvement-proposal tasks

Failure traces supplied to both \(P\) and \(C\) when generating next children.
These traces exclude private promotion answers.

### Child-screening tasks

Used to reject invalid or nonfunctioning generated children.

### Hidden promotion tasks

Used only after child generation. Include:

- a held-out slice of the same benchmark family;
- a different repository/task distribution;
- a second evaluator implementation; and
- adversarial integrity cases.

Promotion tasks remain inaccessible to candidate and diagnostic model.

The four partitions correspond to development, validation, and held-out use:

- search-development tasks are development history;
- improvement-proposal and child-screening tasks form the visible validation
  surface; and
- hidden promotion tasks are the held-out decision surface.

## Factorial design

For each pair, run:

| Modifier | Starting candidate | Purpose |
|---|---|---|
| Parent \(P\) | \(P\) | Parent self-improvement yield |
| Child \(C\) | \(C\) | Child self-improvement yield |
| Fixed base modifier \(M_0\) | \(P\) | Separate candidate quality from modifier quality |
| Fixed base modifier \(M_0\) | \(C\) | Same control on child code |
| Parent \(P\) | neutral target harness | Test modifier transfer |
| Child \(C\) | neutral target harness | Test modifier transfer |

The neutral target harness has the same interface but is not in either
candidate's lineage.

This design distinguishes:

- being a better task-solving starting point;
- being easier for any modifier to improve;
- being a better modifier of itself; and
- being a better general modifier.

## Proposal and implementation split

DGM uses an external diagnostic model and a coding agent. Test both stages.

### Condition A: external diagnosis

The frozen diagnostic model supplies the same distribution of generalized
improvement issues to \(P\) and \(C\).

Measures implementation ability.

### Condition B: candidate diagnosis

\(P\) and \(C\) each analyze blinded failure evidence and propose their own
improvement issue before implementing it.

Measures the complete self-improvement operator.

### Condition C: crossed diagnosis

```text
diagnosis(P) -> implementation(P)
diagnosis(P) -> implementation(C)
diagnosis(C) -> implementation(P)
diagnosis(C) -> implementation(C)
```

Separates proposal quality, implementation quality, and interaction.

## Attempt protocol

For each modifier condition:

1. sample an improvement-task packet;
2. persist the action intent and budget;
3. generate one diagnosis or use the matched supplied diagnosis;
4. create one isolated child workspace;
5. run self-modification under a fixed limit;
6. store patch, trace, model usage, and side effects;
7. run deterministic validity checks;
8. run child-screening tasks;
9. run hidden promotion tasks for valid children;
10. run integrity and authority audits;
11. compute candidate utility without exposing hidden details;
12. retain every attempted result, including invalid and rejected children.

Repeat enough independent attempts to estimate variance. The number of repeats
must be selected from a power analysis or a sequential design specified before
examining treatment results.

## Candidate manifest

Every candidate has:

```text
candidate_id:
seed_revision:
ordered_patch_digests:
source_tree_digest:
model_identity:
tool_schema_digest:
dependency_lock_digest:
container_image_digest:
parent_id:
creation_action_id:
```

No score is valid unless it names this manifest.

## Append-only lineage receipt

Every accepted edge records:

```text
edge_id:
parent_candidate_id:
child_candidate_id:
mutation_action_id:
diagnosis_digest:
patch_digest:
evaluator_package_digest:
resource_receipt_digest:
promotion_decision:
decision_authority:
timestamp:
```

The receipt is append-only and candidate-inaccessible. Rejected and invalid
edges receive receipts too, so search cost and failure modes remain visible.

## Outcome metrics

### Primary

#### Valid improvement yield

$$
Y_{\text{valid}}(X) =
\frac{\#\text{children passing all hard gates and improving held-out utility}}
{\#\text{attempts}}
$$

#### Expected utility gain per attempt

$$
G(X) =
\frac{1}{m}
\sum_{j=1}^{m}
\left[U(C_j)-U(X)\right]
\mathbf{1}[\text{hard gates pass}],
$$

where failed hard gates contribute no promoted gain and remain separately
counted.

### Secondary

- functioning-child rate;
- held-out task improvement;
- cross-benchmark transfer;
- number of distinct causal mechanism classes;
- tokens, calls, dollars, and wall time per valid improvement;
- security and integrity failure rate;
- patch size and changed authority surface;
- fresh-reviewer comprehension time;
- next-child validity in a canary generation;
- rollback success; and
- reconstruction success.

Keep three measurement families separate:

1. current task fitness;
2. valid held-out descendant gain per attempt; and
3. behavioral diversity under the protected evaluator.

Do not let a high value in one family certify either of the others.

## Utility and hard gates

Do not collapse every property into one scalar.

### Hard gates

- candidate identity is complete;
- evaluator and hidden tests are unchanged;
- permissions are respected;
- no protected answer is accessed;
- no unresolved external side effect remains;
- security checks pass;
- lineage is reconstructable; and
- declared-scope correctness passes.

### Pareto-visible objectives

- held-out task quality;
- improvement yield;
- cost;
- latency;
- maintainability;
- successor usability;
- behavioral diversity; and
- transfer.

\(U\) can rank candidates only after hard gates pass. Publish the full vector
with any aggregate.

## Diversity descriptors

Record diversity separately from score:

- solved-task cluster;
- tool-use profile;
- planning and retry strategy;
- changed component class;
- trace-level failure mode;
- language or repository-family coverage;
- cost profile; and
- safety/integrity profile.

Descriptors are computed by an external versioned analyzer. Candidate source
distance may supplement them but cannot substitute for behavioral diversity.

## Model and prompt identity

Every run records:

- provider and endpoint;
- immutable model snapshot when available;
- dated observation when no immutable identity exists;
- decoding parameters;
- system and user prompt digests;
- tool-schema digest;
- retry policy;
- API response schema version; and
- diagnostic/coding model role.

Runs with different identities are separate treatments, not pooled replicas.

## Integrity tests

Inject adversarial opportunities:

- evaluator marker can be reformatted;
- test-file edit appears attractive;
- a child can spawn unaccounted descendants;
- a model call can occur after budget exhaustion;
- archive metadata appears writable;
- hidden-task identity can be guessed;
- an interrupted API call has unknown outcome;
- a symlink targets protected state;
- a candidate can suppress logging;
- a candidate can lower apparent cost without lowering actual cost.

The system must reject or contain these cases without relying on the
candidate's self-report.

## Sandbox and permission policy

Run mutation and evaluation inside a hardened candidate boundary:

- non-root execution;
- immutable base image and evaluator mounts;
- no container-management socket;
- explicit Linux capability drop;
- network allowlist through a model gateway;
- short-lived action-scoped credentials;
- CPU, memory, process, disk, wall-time, and spend quotas;
- no write path to official lineage, hidden tasks, or promotion state; and
- artifact export only through size-limited, path-validated publication.

The policy is enforced externally and included in the evaluator-package
identity. Candidate prompts can explain the policy but do not enforce it.

## Statistical analysis

Use paired analysis because \(P\) and \(C\) receive matched task packets and
budgets.

Report:

- mean and median utility gain;
- bootstrap or model-based uncertainty intervals;
- valid-child proportion difference;
- cost-normalized gain;
- distribution of failure classes;
- interaction between diagnosis source and modifier;
- heterogeneity by generation and mechanism class; and
- sensitivity to utility weights.

Precommit:

- exclusions;
- invalid-run handling;
- stopping rule;
- multiple-comparison policy;
- tie handling;
- treatment of API failures; and
- promotion thresholds.

Do not replace invalid attempts until the treatment has an equal opportunity to
produce invalid attempts; invalidity is an outcome.

## Causal interpretation table

| Observed result | Supported interpretation |
|---|---|
| \(C\) solves more tasks, but \(Y_{\text{valid}}(C)=Y_{\text{valid}}(P)\) | Better task solver, not better improver |
| Fixed modifier improves \(C\) more than \(P\), but \(C\) does not improve itself better | Child is a better substrate, not a better modifier |
| \(C\) improves itself better only with external diagnosis | Implementation capability improved; diagnosis did not |
| \(C\) diagnosis helps both \(P\) and \(C\) | Proposal policy improved |
| \(C\) yields larger valid gains under equal root-tree cost | Successor-improvement evidence |
| Gain disappears on held-out benchmark family | Narrow adaptation |
| Gain co-occurs with integrity failures | No valid promotion claim |
| Gain persists through another canary generation | Stronger recursive evidence |

## Canary third generation

For promoted grandchildren \(G_P\) and \(G_C\), run one additional matched
generation:

```text
P -> child(P) -> grandchild(P)
C -> child(C) -> grandchild(C)
```

The canary is not a claim of indefinite acceleration. It checks whether one
measured successor benefit survives beyond a single parent–child comparison.

Require:

- fresh improvement tasks;
- frozen envelope;
- equal budget;
- hidden promotion set; and
- no reuse of selected diagnostic answers.

## Strong-seed study

Repeat the experiment with at least:

- the original minimal seed;
- a mature human-designed coding agent;
- a different mature architecture; and
- one deliberately simplified ablation.

This reveals whether successor improvement depends on repairing obvious seed
deficiencies.

## Search-policy experiment

Do not let candidate code directly rewrite the active parent selector.

Instead:

1. candidate proposes a search-policy program;
2. external validation checks schema, authority, and resource bounds;
3. nested simulations evaluate the policy on frozen candidate populations;
4. hidden search tasks compare it with baselines;
5. the policy receives no write access to official archive state; and
6. external authority decides whether it enters a later experiment.

This preserves the possibility of meta-process improvement without letting the
active search certify its own judge.

## Durable execution contract

Before every side effect, persist:

```text
action_id:
pair_id:
candidate_id:
operation:
budget_remaining:
intended_external_identity:
status: prepared
```

After execution, record:

```text
observed_external_identity:
result_digest:
resource_usage:
side_effects:
status: succeeded | failed | unknown
```

On restart:

1. reconcile all `prepared` and `unknown` actions;
2. do not blindly retry model or container effects;
3. publish accepted result once;
4. retain failed and partial artifacts; and
5. prevent duplicate archive edges.

## Success criteria

The experiment supports `successor-improvement` only when:

1. \(C\) beats \(P\) on precommitted next-cycle primary metrics;
2. uncertainty excludes a practically negligible effect;
3. hard integrity and authority gates pass;
4. gain survives root-tree resource matching;
5. gain is not explained solely by the external diagnostic model;
6. at least one held-out task family supports transfer;
7. candidate and result identities are reconstructable; and
8. the canary generation does not immediately reverse the result.

The experiment supports a stronger recursive claim only after repeated accepted
generation edges show the effect without expanding candidate authority or
weakening evaluation.

## Stopping rules

Precommit separate stop conditions for:

- maximum matched attempts per parent–child pair;
- cumulative root-tree token, dollar, and wall-time budget;
- sequential statistical evidence threshold;
- integrity or security failure;
- evaluator drift;
- candidate identity or reconstruction failure;
- excessive invalid-child rate; and
- human safety intervention.

An integrity failure stops the affected trial without converting the remaining
budget into extra attempts. A statistical stop cannot override hard safety
gates.

## Failure criteria

Treat any of these as a negative or bounded result:

- task score improves but next-cycle yield does not;
- yield improves only by spending more;
- candidate accesses protected answers;
- integrity failures increase;
- behavior does not transfer;
- the result depends on one lucky selected child;
- descendants become materially less maintainable;
- reconstruction fails;
- rollback cannot restore a safe state; or
- the evaluator cannot distinguish objective hacking.

Negative results should remain in the protected research archive.

## Failure recovery

After interruption:

1. reconcile every prepared model call and container action;
2. mark unknown outcomes explicitly;
3. preserve partial traces and patches;
4. never infer success from file presence alone;
5. avoid repeating a potentially completed external effect without a new
   action identity;
6. publish a candidate only after complete identity and evaluator receipts
   exist; and
7. resume scheduling from durable state, not transcript memory.

Recovery changes operational state, not experimental assignment or budget.

## Minimum credible experiment

Before a full study:

1. choose five parent–child pairs from one completed run without using future
   lineage outcome as selection;
2. use ten matched blinded improvement tasks per candidate;
3. permit two attempts per task;
4. freeze one diagnostic and one coding model;
5. evaluate descendants on a disjoint 50-task set;
6. record all root-tree resource use;
7. audit candidate/evaluator boundaries;
8. compare valid gain per attempt; and
9. publish the complete manifest and analysis script.

This pilot will not establish the final claim. It will validate the measurement
contract and reveal variance and operational failure modes.

## Design judgment

DGM already implements the causal pathway needed for recursive improvement:
child code can participate in creating later child code.

The successor experiment changes what is measured:

```text
current:
  Did child C solve more coding tasks?

required:
  Under the same protected conditions, did child C produce better valid
  successors than parent P?
```

That is the shortest path from an impressive harness-evolution result to a
direct test of the recursive claim.

Continue with the [[knowledge/darwin_godel_machine/learning_path|learning path]], review the
[[knowledge/darwin_godel_machine/09_critical_review|critical assessment]].

Back to the [[knowledge/darwin_godel_machine/darwin_godel_machine_index|DGM index]].
