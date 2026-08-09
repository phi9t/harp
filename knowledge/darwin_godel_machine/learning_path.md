---
id: dgm-learning-path
title: DGM learning path
type: learning-path
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, curriculum, exercises, system-design]
confidence: high
canonical: ../rsi/systems/dgm.md
---

# DGM learning path

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## How to use this path

The path moves through five levels:

1. vocabulary;
2. mechanism;
3. source tracing;
4. evidence critique; and
5. successor-system design.

Do not skip directly to the benchmark numbers. The central question is which
system component changed, who evaluated it, and whether the change improved
the next improvement cycle.

## Level 1: explain the system

**[INFERENCE - DGM-065](claim_evidence_crosswalk.md#dgm-065-dgm-is-evolutionary-search-over-agent-scaffolds).**
Read:

1. [orientation](01_orientation.md);
2. [glossary](glossary.md); and
3. [paper walkthrough](02_paper_walkthrough.md).

### Checkpoint 1 — five-sentence explanation

Explain DGM in five sentences without using the phrase "it improves itself"
until the final sentence.

Your explanation should name:

- the coding-agent repository;
- the diagnostic and coding models;
- benchmark evaluation;
- the branching archive; and
- the recursive claim boundary.

<details>
<summary>Answer guidance</summary>

A strong answer says that an external controller chooses an archived coding
agent, a diagnostic stage turns failure evidence into one general improvement
issue, and the selected agent edits the repository that defines its own
workflow and tools. The resulting child is evaluated as a coding agent, and a
functioning child can enter the archive as a future parent. The foundation
models, benchmarks, parent selector, evaluator, and archive policy remain
external. The self-referential part is that descendant agent code can be used
to create later descendants; the unproven part is whether descendants become
better producers of later valid improvements.

</details>

### Checkpoint 2 — classify the objects

Place each item in one category:

```text
mutable candidate
protected envelope
evidence artifact
derived measurement
```

Items:

- `coding_agent.py`;
- hidden benchmark tests;
- `model_patch.diff`;
- task success rate;
- parent-selection equation;
- `self_evo.md`;
- foundation-model weights;
- archive-admission decision.

<details>
<summary>Answer guidance</summary>

- Mutable candidate: `coding_agent.py`.
- Protected envelope: hidden tests, parent selection, model weights in the
  reported experiment, archive admission.
- Evidence artifact: `model_patch.diff`, `self_evo.md`.
- Derived measurement: task success rate.

The same physical file can cross categories in a different design. The point
is to declare ownership for this experiment.

</details>

## Level 2: derive the algorithm

Read:

1. [algorithm derivation](03_algorithm_derivation.md); and
2. [open-endedness](07_open_endedness.md).

### Exercise 1 — selection probability

Use:

$$
s_i = \frac{1}{1+\exp(-10(\alpha_i-0.5))},
\qquad
h_i = \frac{1}{1+n_i},
\qquad
p_i = \frac{s_i h_i}{\sum_j s_j h_j}.
$$

Calculate probabilities for:

| Agent | Accuracy | Functioning children |
|---|---:|---:|
| A | 0.80 | 7 |
| B | 0.55 | 0 |
| C | 0.40 | 0 |

Then answer:

1. Can B be more likely than A?
2. What does the child-count term reward?
3. What kind of novelty does it not measure?

<details>
<summary>Answer guidance</summary>

Approximate values:

- A: \(s=0.9526\), \(h=0.1250\), \(w=0.1191\);
- B: \(s=0.6225\), \(h=1.0000\), \(w=0.6225\);
- C: \(s=0.2689\), \(h=1.0000\), \(w=0.2689\).

The total weight is about 1.0105, giving probabilities near 0.118, 0.616, and
0.266. B is more likely because A has already produced many functioning
children. The term rewards underexplored parent lineages. It does not measure
behavioral, causal, or source-code novelty.

</details>

### Exercise 2 — archive policy

Compare three policies:

```text
P1: keep only a child that beats its parent
P2: keep the latest functioning child
P3: keep all functioning children and allocate future search probabilistically
```

For each, state:

- what evidence it preserves;
- what failure it avoids;
- what new cost it creates; and
- whether archive admission means deployment permission.

<details>
<summary>Answer guidance</summary>

P1 gives monotone retained score but deletes negative-score stepping stones.
P2 permits score dips but can trap the entire search in one damaged lineage.
P3 preserves alternatives and rollback but grows state and can retain junk.
None should equate archive admission with production deployment.

</details>

### Exercise 3 — stepping-stone claim

A child scores 30%, below its parent's 35%. Its descendant scores 50%.

What additional evidence is needed to claim the 30% child was a causal
stepping stone?

<details>
<summary>Answer guidance</summary>

The lineage alone is insufficient. Reconstruct the descendant without the
intermediate change, or replace the change with a matched alternative, then
rerun evaluation. Preserve task scope and uncertainty. Inspect whether the
descendant actually uses the intermediate mechanism. Otherwise the later gain
could be noise or an unrelated patch.

</details>

## Level 3: trace the implementation

Read:

1. [system architecture](04_system_architecture.md);
2. [repository walkthrough](05_repository_walkthrough.md); and
3. [source registry](source_registry.md).

### Exercise 4 — runtime trace

Starting from an archive node ID, list the files and functions involved before
a child can enter the archive.

Your trace should include:

- parent metadata;
- patch reconstruction;
- diagnosis;
- self-edit execution;
- task evaluation;
- performance aggregation;
- viability filter; and
- generation record.

<details>
<summary>Answer guidance</summary>

One valid route is:

```text
DGM_outer.py::choose_selfimproves
  -> utils/evo_utils.py::get_model_patch_paths
  -> prompts/self_improvement_prompt.py
  -> self_improve_step.py::diagnose_problem
  -> self_improve_step.py::self_improve
  -> coding_agent.py::AgenticSystem
  -> llm_withtools.py + tools/
  -> swe_bench/harness.py or polyglot/harness.py
  -> swe_bench/report.py or Polyglot result report
  -> utils/evo_utils.py::get_all_performance
  -> utils/evo_utils.py::is_compiled_self_improve
  -> DGM_outer.py::update_archive
  -> dgm_metadata.jsonl
```

</details>

### Exercise 5 — find the authority break

Suppose a candidate patch edits `DGM_outer.py` so that its score is doubled
before archive update.

Answer:

1. Why is the file physically present in the self-edit repository?
2. Why should the active outer controller still be protected?
3. Which build or publication boundary should reject the patch?

<details>
<summary>Answer guidance</summary>

The released self-edit environment contains the DGM repository, so outer-loop
files may be visible and editable unless a stricter mutation manifest is
enforced. The active controller owns evaluation and promotion and cannot be
candidate-writable without a nested policy experiment. A candidate manifest
should declare mutable paths; publication should reject changes outside that
set. Search-policy proposals should be evaluated separately rather than
applied to the active judge.

</details>

### Exercise 6 — inspect a snapshot defect

Find and explain the two `best` parent-selection defects in `DGM_outer.py`.

Then propose the smallest unit tests that would catch them.

<details>
<summary>Answer guidance</summary>

The CLI choices concatenate `score_child_prop` and `best` because a comma is
missing. The `best` branch sorts ascending and selects the first values. One
test should parse `--choose_selfimproves_method best`; another should construct
three candidate scores and assert that the highest-scoring candidates are
chosen.

</details>

## Level 4: audit the evidence

Read:

1. [evaluation analysis](06_evaluation_analysis.md);
2. [safety and failure](08_safety_and_failure.md);
3. [critical review](09_critical_review.md); and
4. [claim-evidence crosswalk](claim_evidence_crosswalk.md).

### Exercise 7 — normalize the headline

Explain why these three values cannot be placed in one unlabeled ranking:

```text
SWE-bench: 50.0%
Polyglot search subset: 38.0%
Polyglot full benchmark: 30.7%
```

<details>
<summary>Answer guidance</summary>

They use different task sets and, across benchmarks, different model and task
structures. The Polyglot search subset is adaptively used during search; the
full benchmark is a broader final estimate. The SWE-bench result is a
200-task paper subset. Every value needs benchmark, task count, metric, model,
and selection context.

</details>

### Exercise 8 — root-tree accounting

Design a resource receipt for one self-improvement attempt. Include costs that
would be omitted by a task-agent-only token count.

<details>
<summary>Answer guidance</summary>

Include diagnostic calls, self-edit calls, retries, all child model calls,
container build and execution, benchmark evaluation, failed tasks, full-stage
allocation, background processes, and external evaluator compute. Attribute
parallel descendants to the root attempt. Record tokens, calls, dollars, CPU,
memory, disk, and wall time separately.

</details>

### Exercise 9 — objective hacking

Reconstruct the Appendix H failure as a causal chain:

```text
target behavior
-> measured proxy
-> candidate-visible interface
-> candidate change
-> observed score
-> actual behavior
```

Then propose two controls that do not depend on telling the candidate not to
change markers.

<details>
<summary>Answer guidance</summary>

The target is genuine tool use without hallucinated transactions. The proxy
checks special logging markers. The candidate can change the logging channel,
removes or restructures markers, and receives a perfect score while
hallucination remains. Controls include evaluator-owned observation outside
candidate code, multiple independently implemented detectors, raw API event
attestation, and an interface-compatibility gate that invalidates changes to
the measurement channel.

</details>

### Exercise 10 — strongest justified claim

Choose one:

A. DGM proves full recursive self-improvement.
B. DGM proves open-ended intelligence.
C. DGM demonstrates automated persistent harness evolution with a branching
archive and author-reported task gains.
D. DGM proves safe autonomous software engineering.

<details>
<summary>Answer guidance</summary>

C. The causal path to successor improvement exists, but the matched
parent-versus-child improvement-yield test is missing.

</details>

## Level 5: design the successor experiment

**[INFERENCE - DGM-067](claim_evidence_crosswalk.md#dgm-067-task-fitness-and-descendant-productivity-are-different-objectives).**
Read:

1. [successor design](10_successor_design.md); and
2. canonical
   [evaluation, promotion, and containment](../rsi/chapters/evaluation-promotion-containment.md).

### Exercise 11 — parent versus child

Specify a matched experiment for one parent–child pair.

Required fields:

- treatment conditions;
- task partitions;
- diagnostic model;
- coding model;
- attempts;
- root-tree budget;
- hard gates;
- primary outcome;
- hidden promotion set;
- invalid-run handling; and
- stopping rule.

<details>
<summary>Answer guidance</summary>

Use the same blinded improvement-task packets, model snapshots, attempts,
permissions, evaluator, and resource vector for parent and child. Count every
attempt, including invalid children. The primary outcome is valid held-out
utility gain per attempt, not selected maximum score. Hidden tasks remain
inaccessible to diagnosis and implementation. Precommit the sample size or
sequential stopping rule.

</details>

### Exercise 12 — crossed diagnosis

Why compare:

```text
diagnosis(P) -> implementation(P)
diagnosis(P) -> implementation(C)
diagnosis(C) -> implementation(P)
diagnosis(C) -> implementation(C)
```

<details>
<summary>Answer guidance</summary>

The factorial separates proposal quality from implementation quality and their
interaction. Without it, a stronger external critic can make a weak modifier
look better, or an easier-to-modify child can look like a stronger
self-improver.

</details>

### Exercise 13 — promotion decision

Candidate X improves held-out task score by 8 points but:

- costs 2.5 times more;
- changes the tool logging schema;
- fails one evaluator-integrity check; and
- produces children with a lower validity rate.

Should X enter:

- the research archive;
- the active parent pool;
- production?

<details>
<summary>Answer guidance</summary>

It may remain in a quarantined research archive with the integrity failure
recorded. It should not enter the active trusted parent pool or production.
Evaluator integrity is a hard gate, and lower next-child validity weakens the
successor claim. Task gain cannot buy authority.

</details>

### Exercise 14 — design review

Write a one-page review with:

1. decision;
2. three highest-consequence findings;
3. evidence for each;
4. one experiment that changes the decision; and
5. explicit non-goals.

Use the consequence ordering in the
[critical review](09_critical_review.md), not a file-by-file summary.

## Capstone

Design a DGM-like system for a non-coding domain with an executable but
imperfect evaluator.

Your design must state:

- candidate representation;
- protected envelope;
- mutation operator;
- evaluator and hidden holdout;
- archive semantics;
- behavior descriptor;
- root-tree budget;
- objective-hacking tests;
- human decision points;
- rollback;
- next-cycle improvement metric; and
- evidence needed before increasing authority.

### Capstone review rubric

| Dimension | Strong answer |
|---|---|
| System boundary | Candidate and evaluator ownership are explicit |
| Evidence | Claims map to artifacts and claim ceilings |
| Search | Archive and selection policy have stated semantics |
| Safety | Authority and evaluator integrity are enforced, not prompted |
| Accounting | All descendants and retries belong to the root budget |
| Recursion | Next-cycle improvement is directly measured |
| Durability | Effects and recovery have action identities |
| Human role | Decision points are placed at consequential abstractions |

## Completion standard

**[MISSING - DGM-050](claim_evidence_crosswalk.md#dgm-050-matched-successor-improvement-evidence-is-missing).**
You understand DGM when you can hold these two statements together:

1. DGM is a real self-referential harness-evolution mechanism whose
   descendants can shape later self-modification.
2. Better benchmark-solving descendants are not yet direct evidence of a
   better successor-production process.

Use [maintenance](maintenance.md) before changing the packet.

Back to the [DGM index](darwin_godel_machine_index.md).
