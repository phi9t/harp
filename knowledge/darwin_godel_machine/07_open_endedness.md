---
id: dgm-open-endedness
title: DGM open-endedness
type: deep-dive
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, open-endedness, stepping-stones, diversity]
confidence: high
canonical: ../rsi/systems/dgm.md
---

# DGM open-endedness

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## Learning outcomes

After this chapter, you should be able to:

- compare latest-only, greedy, and branching archive search;
- explain why a lower-scoring node can be a stepping stone;
- separate lineage diversity from behavioral diversity;
- identify the limits of DGM's child-count exploration term; and
- specify stronger archive and open-endedness measurements.

## The design question

Why keep an agent that scores worse than its parent?

A linear optimizer treats a lower score as evidence to discard the candidate.
DGM treats it as one observation about one current objective. A weaker node may
contain a tool, representation, or workflow change that becomes useful after a
later modification.

This is the stepping-stone argument.

## Hill climbing

A simple hill climber maintains one incumbent:

```text
parent = current_best
child = mutate(parent)
if score(child) > score(parent):
    current_best = child
```

Advantages:

- simple state;
- cheap selection;
- monotone retained score; and
- easy attribution.

Failure mode:

- a useful intermediate change is discarded if it lowers immediate score.

Examples include a refactor that initially breaks a heuristic, a new tool that
needs a later prompt change, or a context mechanism that costs tokens before a
later retrieval policy exploits it.

## Latest-lineage search

The paper's no-open-ended-exploration baseline is not strict hill climbing. It
continues from the latest version that retains basic code-editing function.

```text
parent = latest_functioning_agent
child = mutate(parent)
if functioning(child):
    parent = child
```

This allows temporary score regressions but maintains only one path. A harmful
yet functioning edit can make all later work start from a degraded interface.
Earlier alternatives disappear from the active search.

## Greedy archive search

DGM Greedy preserves the archive but always branches from the best-scoring
node.

This preserves rollback and historical variants while concentrating new
compute on one observed optimum. It can still miss:

- low-scoring prerequisites;
- alternative implementations of the same capability;
- lineages whose value emerges under later tasks; and
- underexplored nodes whose score estimate is noisy.

## DGM archive search

Full DGM:

1. keeps functioning children;
2. leaves older nodes selectable;
3. weights selection by score;
4. discounts parents that already have many functioning children; and
5. permits non-best nodes to receive further mutations.

**[SOURCE CLAIM - DGM-034](claim_evidence_crosswalk.md#dgm-034-paper-reports-two-score-dips-in-the-best-lineage).**
The paper's SWE-bench lineage contains two reported score dips on the ancestry
of the final best agent. That observation supports the narrow stepping-stone
claim:

> at least one high-scoring descendant arose through ancestors that would not
> have been retained by monotone score-only replacement.

It does not prove that every retained low-scoring node is useful or that the
same archive policy is optimal.

## Why `keep_all` matters

**[EVIDENCE - DGM-009](claim_evidence_crosswalk.md#dgm-009-archive-admission-does-not-require-score-improvement).**
The released controller defaults to adding every child that passes the
functioning-agent predicate.

`keep_all` separates two decisions:

- **viability:** can this child still operate as a code-editing agent?
- **value:** is this lineage worth further exploration?

The parent selector, not archive admission, allocates future search effort
based on current score and child count.

This choice prevents immediate score from deleting a possible stepping stone.
It also creates costs:

- archive metadata grows;
- parent selection scans more nodes;
- more lineages require provenance and storage;
- retrospective analysis becomes harder; and
- low-value candidates can consume search probability.

## Performance and underexploration

The selection weight is:

$$
w_i =
\frac{1}{1+\exp(-10(\alpha_i-0.5))}
\cdot
\frac{1}{1+n_i}.
$$

The first term biases toward current task performance. The second biases
toward lineages with fewer functioning children.

This is an exploration-exploitation policy:

- exploit candidates likely to solve tasks;
- explore candidates that have not produced many descendants.

The policy does not know *why* two agents differ.

**[INFERENCE - DGM-065](claim_evidence_crosswalk.md#dgm-065-dgm-is-evolutionary-search-over-agent-scaffolds).**
The exact mechanism is archive-based, non-greedy program search. It is
open-ended only in the bounded sense that old executable lineages remain
selectable and branching can continue under fixed tasks and control policy.

## Lineage diversity versus behavioral diversity

### Lineage diversity

Two nodes have different ancestor chains or patches.

DGM records this directly.

### Syntactic diversity

Two nodes have different source text or patch embeddings.

The reported parent selector does not use this measure.

### Behavioral diversity

Two nodes solve different task subsets, choose different tools, use different
plans, fail in different ways, or expose different cost/safety profiles.

The reported parent selector does not use an explicit behavior descriptor or
novelty distance.

### Causal diversity

Two nodes implement genuinely different mechanisms rather than superficial
variants.

This requires code and trace analysis. Appendix G gives one useful example:
two attempts at finer-grained editing produce materially different task
scores. One target functionality can contain multiple implementations with
different consequences.

The archive guarantees lineage plurality, not behavioral or causal diversity.

## Relation to quality-diversity

Quality-diversity methods aim to illuminate a space of high-quality solutions
across explicit behavioral niches.

A typical QD archive needs:

- a behavior descriptor;
- a niche or distance function;
- a quality measure within each niche; and
- an archive update policy that preserves coverage.

DGM has:

- a population of lineages;
- one primary task-performance score;
- a child-count exploration bonus; and
- permissive retention of functioning nodes.

It resembles QD in preserving alternatives, but it does not instantiate a full
behavior-space archive. Calling its child-count term "novelty" should not hide
that distinction.

Canonical context:
[harness search methods](../rsi/concepts/harness-search-methods.md).

## Relation to admitted program-search systems

[AlphaEvolve](../rsi/systems/alphaevolve.md) and
[FunSearch](../../content/sources/source_registry.tsv) provide Harp's admitted
program-search comparison. They retain executable solution programs and use
external evaluators to guide later proposals.

DGM differs in the edited object:

- AlphaEvolve and FunSearch primarily evolve solution programs for a declared
  problem;
- DGM evolves the coding-agent implementation used to produce later task and
  self-modification patches.

All three depend on externally fixed evaluators and model supply. DGM adds a
parent-linked agent archive and self-referential code path, but its released
selector still lacks an explicit behavioral descriptor.

## Why archive growth is not progress

An archive can grow while:

- average task score falls;
- all nodes implement near-identical behavior;
- child validity declines;
- cost rises;
- evaluator gaming spreads;
- no node transfers to held-out tasks; or
- later descendants become harder to understand and modify.

Therefore report at least:

- archive size;
- unique functioning nodes;
- task-score distribution with task scope;
- lineage depth and branching;
- behavior coverage;
- candidate cost;
- integrity failures;
- valid-child rate; and
- next-cycle improvement yield.

Archive size alone measures retained search state.

## Temporary regressions

A temporary score dip can mean several different things:

1. **Useful stepping stone.** The child introduces a prerequisite used later.
2. **Noisy evaluation.** True quality did not decline; the small subset score
   fluctuated.
3. **Tradeoff.** One capability improved while the benchmark aggregate fell.
4. **Damage.** The child is simply worse but later receives an unrelated fix.
5. **Selection artifact.** Many branches make one apparently redemptive
   lineage likely to appear.

The paper's lineage supports non-monotone ancestry. It does not alone identify
which explanation holds for every dip.

A causal stepping-stone test would ablate the intermediate change from the
descendant and rerun matched evaluation.

## Stepping-stone receipt

For ancestor \(A\), descendant \(D\), and intermediate change \(m\), record:

```text
ancestor:
intermediate_patch:
immediate_score_and_scope:
descendant_patch:
descendant_score_and_scope:
descendant_without_intermediate_patch:
mechanism_hypothesis:
reproduction_runs:
```

The key comparison is not only:

```text
score(D) > score(A)
```

but:

```text
score(D) > score(D with m removed or replaced)
```

under a valid counterfactual reconstruction.

## Search-policy confounds

### Unequal evaluation precision

**[INFERENCE - DGM-069](claim_evidence_crosswalk.md#dgm-069-staged-point-estimates-create-winners-curse-risk).**
Nodes evaluated on 10, 60, and 200 tasks can coexist. Parent selection can use
scores with different uncertainty.

### Child count is outcome-dependent

Only functioning children count. A parent that receives many failed mutation
attempts can remain "underexplored" according to the functioning-child count.

### Model and task dependence

A lineage can look diverse under one model and collapse under another. Model
transfer results partially test this.

### Privileged diagnosis

Private tests and evaluator logs can influence the proposed harness change.
Search may discover broadly useful mechanisms, but it does not operate under a
fully hidden evaluator.

### Permissive archive update

`keep_all` retains stepping stones and junk. Search quality depends on later
selection pressure and finite compute.

## What would make DGM more open-ended?

"More open-ended" should mean more than a larger archive or longer run.

Possible improvements include:

### Explicit behavior descriptors

Track tool-use strategies, solved-task clusters, language coverage, error
recovery, context cost, and safety properties. Preserve high-quality nodes
across those niches.

### Learned but protected descriptors

Use an external representation to cluster behavior while preventing the
candidate from rewriting the descriptor to appear novel.

### Dynamic task generation

Co-evolve tasks or environments that expose new capability frontiers. This
requires protection against trivial task generation and collusion between
proposer and solver.

### Multi-objective archive

Retain Pareto-efficient candidates across quality, cost, robustness,
maintainability, safety, and successor usability rather than one score.

### Search-policy evolution

Allow candidate proposals for parent selection and archive policy, but evaluate
those policies in a nested protected experiment. The current paper explicitly
leaves this as future work.

### Cross-generation improvement metrics

**[INFERENCE - DGM-067](claim_evidence_crosswalk.md#dgm-067-task-fitness-and-descendant-productivity-are-different-objectives).**
Select partly on the quality and diversity of children a node produces, not
only its task score.

This is the most direct bridge from harness evolution to successor
improvement.

## When should a node be deleted?

Permanent retention is not free. A safe compaction policy can remove or cold
store a node only when:

- its patch and metadata remain content-addressed;
- descendants can still be reconstructed;
- no unique behavior or failure evidence is lost;
- rejected and invalid outcomes remain represented in aggregate;
- the deletion policy is external to candidates; and
- rollback and audit requirements are preserved.

Archive compaction is an evidence-management problem, not only a storage
problem.

## Open-endedness claim ladder

| Claim | Evidence needed | DGM status |
|---|---|---|
| Multiple lineages are retained | Parent-linked archive | Supported |
| Non-best nodes remain selectable | Positive selection probability | Supported by paper mechanism |
| A final lineage contains score dips | Lineage scores | Author-reported |
| A lower-scoring node causally enabled a later gain | Counterfactual removal or replacement | Not established |
| Archive preserves behavioral diversity | Behavior descriptors and coverage | Not established |
| Search creates unbounded novelty | Expanding domains and sustained novelty measures | Not established |
| Improvement process self-accelerates | Matched next-cycle improvement yield | Not established |

## Design-review judgment

**[INFERENCE - DGM-070](claim_evidence_crosswalk.md#dgm-070-a-stronger-successor-should-separate-fitness-productivity-and-diversity).**
DGM's archive is a meaningful advance over one-incumbent harness search. It
preserves recoverable alternatives and creates a direct route through
non-monotone lineages.

Its "open-ended" mechanism is nevertheless bounded by:

- fixed benchmarks;
- fixed model supply;
- fixed diagnostic process;
- fixed parent selection;
- fixed archive admission;
- fixed execution authority; and
- one main task-performance objective.

The precise description is:

> branching, non-monotone empirical search over self-modifying coding-agent
> implementations.

That is enough to study stepping stones. It is not yet evidence of indefinitely
expanding behavioral diversity or autonomous improvement of the search process.

Continue with [safety and failure](08_safety_and_failure.md), design a stronger
experiment in [successor design](10_successor_design.md).

Back to the [DGM index](darwin_godel_machine_index.md).
