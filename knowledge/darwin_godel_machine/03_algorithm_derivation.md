---
id: dgm-algorithm-derivation
title: DGM algorithm derivation
type: derivation
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, algorithm, parent-selection, archive-search]
confidence: high
canonical: ../rsi/systems/dgm.md
---

# DGM algorithm derivation

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## Learning outcomes

After this chapter, you should be able to:

- derive the parent-selection probability from archive metadata;
- explain why score and underexploration enter as separate factors;
- distinguish an eligible parent, a functioning child, and a promoted system;
- trace one generation from selection through staged evaluation;
- compare the two paper baselines with the full algorithm; and
- identify which parts of the search policy remain fixed.

Primary paper route: [[evidence/weng/text/dgm.txt|DGM Appendix C]].
Released-code route:
[[evidence/implementations/dgm/snapshot/DGM_outer.py|DGM_outer.py]] and
[[evidence/implementations/dgm/snapshot/utils/evo_utils.py|evo_utils.py]].

## State of the search

At iteration \(t\), let the archive be:

$$
A_t = \{a_1^t, a_2^t, \ldots, a_N^t\}.
$$

Each node needs at least:

- an identity;
- a parent identity;
- a patch lineage;
- a benchmark score \(\alpha_i\);
- counts or lists of resolved, unresolved, and empty-patch tasks;
- the number \(n_i\) of functioning children it has already produced; and
- enough metadata to reconstruct and evaluate the candidate.

The archive is not one source tree copied \(N\) times. In the released
implementation, a node is materialized by following `parent_commit` pointers
to `initial` and applying ancestor `model_patch.diff` files from oldest to
newest.

## Eligibility

The paper defines the parent-eligible set as:

$$
E_t = \{a_i^t \in A_t : \alpha_i < 1\}.
$$

A perfect-scoring agent is excluded because the current benchmark gives no
task-performance reason to modify it. This does not mean a perfect scorer
cannot be improved in maintainability, cost, safety, or transfer. It means the
paper's scalar parent-selection rule does not represent those objectives.

The released code builds candidates from archive nodes whose metadata can be
loaded. It does not contain a separate explicit `score < 1` filter in the
captured `choose_selfimproves` path. The equation is therefore the paper
contract; the snapshot is evidence for the particular released implementation.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-062: Released parent selection omits the paper's perfect-score filter|EVIDENCE - DGM-062]].**
This is a paper-to-code mismatch, not evidence that the historical experiment
used the released candidate set.

## Performance term

For candidate score $\alpha_i$

$$
s_i =
\frac{1}
{1 + \exp(-\lambda(\alpha_i - \alpha_0))}.
$$

The reported settings are:

$$
\lambda = 10, \qquad \alpha_0 = 0.5.
$$

The sigmoid has two effects:

1. it keeps every non-perfect eligible candidate at positive weight; and
2. it emphasizes score differences near the 0.5 midpoint while compressing
   differences near 0 and 1.

This is not a calibrated probability that the candidate is good. It is a
search heuristic that turns benchmark accuracy into a bounded weight.

## Under-exploration term

Let \(n_i\) be the number of functioning children already attributed to agent
\(i\). The paper calls:

$$
h_i = \frac{1}{1+n_i}
$$

a novelty bonus.

The name needs a caveat. \(h_i\) rewards low offspring count; it does not
measure behavioral novelty, code distance, capability diversity, or coverage
of a behavior descriptor. A parent with four admitted children receives one
fifth of the underexploration weight of a parent with none.

## Combined selection probability

The unnormalized weight is:

$$
w_i = s_i h_i.
$$

Normalize over eligible parents:

$$
p_i = \frac{w_i}{\sum_{a_j \in E_t} w_j}.
$$

The controller draws \(k\) parents with replacement:

$$
\{parent_1,\ldots,parent_k\}
\sim \operatorname{Categorical}(\{p_i\}_{a_i \in E_t}).
$$

The paper uses \(k=2\) for SWE-bench and \(k=4\) for Polyglot. Sampling with
replacement means one parent can receive more than one self-modification
attempt in the same generation.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-005: Parent selection combines score and underexploration|EVIDENCE - DGM-005]].**
The released default implements the same sigmoid-score times inverse-child-count
weight, but the quantity called novelty is lineage underexploration rather than
behavioral distance.

## Worked example

Consider four eligible agents:

| Agent | Accuracy \(\alpha_i\) | Functioning children \(n_i\) |
|---|---:|---:|
| A | 0.70 | 4 |
| B | 0.60 | 0 |
| C | 0.45 | 0 |
| D | 0.30 | 0 |

Using \(\lambda=10\) and \(\alpha_0=0.5\):

| Agent | \(s_i\) | \(h_i\) | \(w_i=s_i h_i\) | \(p_i\) |
|---|---:|---:|---:|---:|
| A | 0.8808 | 0.2000 | 0.1762 | 0.1255 |
| B | 0.7311 | 1.0000 | 0.7311 | 0.5207 |
| C | 0.3775 | 1.0000 | 0.3775 | 0.2689 |
| D | 0.1192 | 1.0000 | 0.1192 | 0.0849 |
| **Total** |  |  | **1.4040** | **1.0000** |

Three observations matter:

1. A has the highest task score but has already produced four functioning
   children, so B is more likely to be selected.
2. C remains meaningfully selectable despite scoring below the midpoint.
3. D has low probability, not zero probability. It can still serve as a
   stepping stone.

If two parents are drawn with replacement, B can be selected twice. The
expected number of selections for B is \(2 \times 0.5207 = 1.0414\), but any
particular generation may draw a different pair.

## One generation

The paper-level generation can be written as:

```text
input:
  archive A_t
  immutable benchmark and evaluator
  parent count k

1. Build eligible parent set E_t.
2. Compute score term s_i and child-count term h_i.
3. Sample k parents with replacement.
4. For each selected parent:
   a. choose failure evidence;
   b. diagnose one general improvement opportunity;
   c. reconstruct the parent from its patch lineage;
   d. ask the parent coding agent to modify its own repository;
   e. save the child patch and trace;
   f. evaluate the child in stages;
   g. test whether it remains a functioning code editor.
5. Add functioning children according to archive policy.
6. append generation metadata.
output:
  archive A_(t+1)
```

In the released source, steps 4b through 4f occur through
`self_improve_step.py`. The outer controller records both all attempted
children and the subset that passed the functioning-agent filter.

## Failure evidence and proposal

A selected parent does not simply receive "improve yourself." The controller
chooses:

- an unresolved task;
- an empty-patch failure;
- a stochasticity problem; or
- a context-length problem.

For ordinary SWE-bench failures, a separate diagnostic model receives the
agent trace, issue, proposed patch, private test patch, and evaluation log. It
returns one generalized improvement issue. The coding agent then edits its own
repository to implement that issue.

This division separates:

- **diagnosis:** infer a reusable harness defect from one failure; and
- **implementation:** modify the agent code.

It also introduces a confound: the improvement mechanism includes a fixed,
strong external diagnostic model and privileged evaluation evidence.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-068: Privileged diagnosis makes the benchmark a development channel|INFERENCE - DGM-068]].**
The benchmark subset therefore serves partly as an architecture-development
channel, even though the ordinary task-solving agent remains blind to private
tests.

## Staged evaluation

The search does not score every child on every task.

### SWE-bench

1. Run 10 tasks to test basic code-editing function.
2. If the child reaches the configured threshold, run 50 more tasks.
3. For selected high-scoring archive members, run 140 more tasks.
4. Report a 200-task score for those fully evaluated candidates.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-060: Released SWE path does not consume the full-evaluation threshold|EVIDENCE - DGM-060]].**
These four steps describe the paper. The captured release wires only the first
two evaluation stages; its full threshold and `big` subset are not consumed.

### Polyglot

1. Run 10 tasks.
2. If the child clears the threshold, run 50 additional tasks in the paper
   setup.
3. Evaluate the selected final agent on the full benchmark for the headline
   comparison.

Staging saves cost, but it creates unequal score precision. A 10-task result
has much higher variance than a 200-task result. Parent selection, archive
plots, and "best agent" language must therefore retain the evaluation scope.

## Archive admission

The default released-code archive update is `keep_all`, after a child passes
`is_compiled_self_improve`.

That function checks:

1. required performance metadata exists;
2. resolved plus unresolved task count is nonzero, so at least one submitted
   patch was not empty; and
3. the expected number of tasks was evaluated.

It does not require:

- child score greater than parent score;
- child score greater than the archive best;
- behavioral novelty;
- maintainability improvement;
- lower cost; or
- a successful next self-modification.

This permissiveness is deliberate for stepping-stone search. It also means
archive admission is not deployment promotion.

## Baseline algorithms

### Fixed modifier: no self-improving agents

A fixed base agent proposes changes to candidate agents. Descendants can
improve as task solvers, but the modifier itself does not inherit the improved
agent implementation.

This tests whether changing the modifier matters.

### Linear lineage: no open-ended exploration

The latest functioning child becomes the next parent. Earlier alternatives do
not remain selectable.

This tests whether branching retention matters.

### Greedy parent selection

Appendix A.3 compares the probabilistic parent selector with a version that
always branches from the best-performing node. This tests the value of
underexplored lineages, but not all possible archive or quality-diversity
policies.

### Full DGM

The selected parent is both:

- a task-solving agent; and
- the implementation used to make the next agent edit.

Multiple functioning lineages remain available.

## Fixed and mutable surfaces

| Surface | Mutable in reported DGM? | Consequence |
|---|---|---|
| Coding-agent prompts and tools | Yes | Later task solving and self-editing can change |
| Agent workflow and model adapter | Yes | Control flow and context use can change |
| Foundation-model weights | No | Core model capability remains externally supplied |
| Parent-selection equation | No | Search allocation does not self-improve |
| Diagnostic-model identity | No | Failure framing remains externally supplied |
| Benchmark tasks and scoring | No | Objective remains fixed |
| Archive-admission policy | No | What survives remains externally defined |
| Execution boundary | No | Candidate cannot legitimately redefine containment |

## What the algorithm establishes

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-050: Matched successor-improvement evidence is missing|MISSING - DGM-050]].**
The algorithm establishes a persistent, branching harness-search process in
which descendant code can influence later self-modification. It does not by
itself establish that the search becomes more efficient or effective over
generations.

The missing direct measurement is:

$$
\mathbb{E}[\text{valid gain produced by child as modifier}]
>
\mathbb{E}[\text{valid gain produced by parent as modifier}]
$$

under matched tasks, models, evaluator, authority, and root-tree budget.

Continue with [[knowledge/darwin_godel_machine/04_system_architecture|system architecture]], inspect
[[knowledge/darwin_godel_machine/07_open_endedness|open-endedness]].

Back to the [[knowledge/darwin_godel_machine/darwin_godel_machine_index|DGM index]].
