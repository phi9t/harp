---
id: dgm-glossary
title: DGM glossary
type: concept
status: active
created: 2026-08-08
updated: 2026-08-08
tags: [darwin-godel-machine, glossary, notation]
confidence: high
canonical: ../../content/concepts/system-state-and-notation.md
---

# DGM glossary

> This file is a learning projection. Canonical claims live under `content/`;
> primary-source captures and pinned implementation files live under
> `evidence/`.

## Agent

A code repository plus foundation-model calls that can inspect, edit, and
execute code. In DGM, the mutable agent is the harness implementation around
frozen model weights.

Common misunderstanding: "agent" does not mean model weights alone.

Canonical source: [harness components](../../content/concepts/harness-components.md).

## Archive

The externally maintained collection of viable agent versions and metadata. In
DGM, nodes are reconstructed from parent-linked patches.

Common misunderstanding: archive membership does not imply improvement over
the parent.

Canonical source: [harness search methods](../../content/concepts/harness-search-methods.md).

## Candidate

The mutable object under evaluation. In DGM, it is the coding-agent
implementation assembled from its ancestor patch lineage.

Common misunderstanding: the candidate does not include the protected
benchmark, parent selector, or promotion authority.

Canonical source: [system state and notation](../../content/concepts/system-state-and-notation.md).

## Child

An agent version created when a parent edits its own implementation. In DGM, a
child is represented by a new patch plus parent metadata.

Common misunderstanding: every generated child is not automatically valid or
archived.

Canonical source: [recursive improvement loop](../../content/chapters/recursive-improvement-loop.md).

## Coding benchmark

A set of repository-editing tasks with an evaluation procedure. DGM uses
SWE-bench Verified and Polyglot as empirical fitness signals.

Common misunderstanding: benchmark accuracy does not directly measure future
self-improvement ability.

Canonical source: [evaluation and control](../../content/concepts/evaluation-and-control.md).

## Diagnostic model

The separate model that reads failure evidence and proposes one general
agent-improvement issue. The released code uses `o1-2024-12-17`.

Common misunderstanding: the coding agent does not independently formulate
every improvement task in the released implementation.

Canonical source: [DGM system article](../../content/systems/dgm.md).

## DGM

Darwin Gödel Machine, the complete system combining self-referential coding
agent edits, empirical evaluation, and branching archive search.

Common misunderstanding: DGM is not only the mutable coding-agent repository;
it also includes a fixed outer search and evaluation envelope.

Canonical source: [DGM system article](../../content/systems/dgm.md).

## Eligibility set

At iteration \(t\), the paper defines:

$$
E_t = \{a_i^t \in A_t : \alpha_i < 1\}
$$

It excludes agents already scoring perfectly.

Common misunderstanding: eligibility means selectable as a parent, not
authorized for deployment.

Canonical source: [DGM system article](../../content/systems/dgm.md).

## Empirical validation

Using observed benchmark performance and viability checks instead of a formal
proof that a rewrite improves expected utility. DGM uses this replacement for
Gödel Machine proof search.

Common misunderstanding: empirical validation is bounded by the evaluator and
does not prove universal improvement.

Canonical source: [evaluation and control](../../content/concepts/evaluation-and-control.md).

## Foundation model

The pretrained model invoked by the agent. The reported DGM experiments keep
model weights fixed.

Common misunderstanding: an evolved DGM agent is not a newly trained
foundation model.

Canonical source: [model adaptation](../../content/concepts/model-adaptation.md).

## Functioning child count

\(n_i\), the number of viable code-editing children already produced by agent
\(i\). It reduces that parent's future selection weight.

Common misunderstanding: failed attempts and behavioral novelty are not
represented directly by this count.

Canonical source: [DGM system article](../../content/systems/dgm.md).

## Gödel Machine

A theoretical self-referential system that applies a rewrite after proving the
rewrite improves expected utility.

Common misunderstanding: DGM does not supply that proof obligation; it uses
empirical benchmark selection.

Canonical source: [Harp source registry](../../content/sources/source_registry.tsv).

## Harness improvement

A persistent change to prompts, tools, control flow, context handling, or other
agent code that improves downstream behavior. DGM searches this surface while
keeping model weights fixed.

Common misunderstanding: harness improvement is not automatically successor
improvement.

Canonical source: [harness search](../../content/chapters/harness-search.md).

## Hill climbing

Search that continues from one incumbent, usually the best or latest candidate.
DGM's archive differs because older viable nodes remain selectable.

Common misunderstanding: DGM's no-open-ended baseline follows the latest
functioning node and is not identical to strict monotone hill climbing.

Canonical source: [harness search methods](../../content/concepts/harness-search-methods.md).

## Lineage

The parent-child chain of agent patches. The released implementation follows
`parent_commit` metadata and applies `model_patch.diff` files from root to leaf.

Common misunderstanding: a lineage records ancestry, not proof that every
ancestor causally enabled the final score.

Canonical source: [evaluation and control](../../content/concepts/evaluation-and-control.md).

## Novelty bonus

The paper's name for:

$$
h_i = \frac{1}{1+n_i}
$$

It is an underexploration bonus based on child count. It is not a behavioral
novelty metric.

Common misunderstanding: two behaviorally identical agents can receive
different bonuses because their child counts differ.

Canonical source: [harness search methods](../../content/concepts/harness-search-methods.md).

## Objective hacking

Optimizing the measured objective while defeating the intended goal. In the
paper's hallucination case, a candidate removes markers used by the detector
and receives a perfect score without solving the underlying problem.

Common misunderstanding: hiding evaluator code is insufficient when the
candidate can alter the measured observation channel.

Canonical source: [evaluation and control](../../content/concepts/evaluation-and-control.md).

## Open-ended exploration

Search intended to keep generating useful new stepping stones rather than
converging quickly to one fixed target. DGM implements a bounded form through
archive retention and non-zero parent selection.

Common misunderstanding: archive growth alone does not establish expanding
behavioral diversity or unbounded novelty.

Canonical source: [harness search methods](../../content/concepts/harness-search-methods.md).

## Parent

An archived agent selected to produce a child through self-modification.

Common misunderstanding: the selected parent does not own the benchmark,
archive policy, or diagnostic model.

Canonical source: [recursive improvement loop](../../content/chapters/recursive-improvement-loop.md).

## Performance

\(\alpha_i\), the benchmark accuracy associated with agent \(i\) in parent
selection.

Common misunderstanding: scores from 10, 60, and 200 tasks do not have equal
precision.

Canonical source: [evaluation and control](../../content/concepts/evaluation-and-control.md).

## Protected envelope

The evaluator-owned state that candidates cannot modify. For DGM this should
include tasks, scoring, budgets, permissions, official lineage, and promotion
authority.

Common misunderstanding: a prompt telling the candidate not to edit something
does not create a protected boundary.

Canonical source: [system state and notation](../../content/concepts/system-state-and-notation.md).

## Recursive self-improvement

An accepted persistent change improves the process that produces later
accepted changes. Better task scores alone do not establish this stronger
property.

Common misunderstanding: repeated self-editing is not sufficient without
evidence that the improvement operator improves.

Canonical source: [recursive improvement loop](../../content/chapters/recursive-improvement-loop.md).

## Self-modification

The selected coding agent edits its own agent repository rather than only the
downstream task repository.

Common misunderstanding: the released system does not let the candidate
legitimately rewrite every outer-loop component.

Canonical source: [DGM system article](../../content/systems/dgm.md).

## Sigmoid-scaled performance

$$
s_i = \frac{1}{1+\exp(-\lambda(\alpha_i-\alpha_0))}
$$

The paper uses \(\lambda=10\) and \(\alpha_0=0.5\).

Common misunderstanding: \(s_i\) is a search weight, not a calibrated
probability of candidate correctness.

Canonical source: [DGM system article](../../content/systems/dgm.md).

## Staged evaluation

Allocating more benchmark tasks to candidates that pass earlier, cheaper
stages. DGM uses 10-task, 60-task, and, for selected SWE-bench agents, 200-task
stages.

Common misunderstanding: staged scores cannot be compared without retaining
task count and selection context.

Canonical source: [evaluation and control](../../content/concepts/evaluation-and-control.md).

## Stepping stone

A candidate that is useful as an ancestor even if its immediate benchmark
score is not the best available score.

Common misunderstanding: non-monotone ancestry alone does not prove causal
stepping-stone value.

Canonical source: [harness search methods](../../content/concepts/harness-search-methods.md).

## Successor improvement

A child produces better later candidates than its parent under the same
protected evaluation and budget conditions.

Common misunderstanding: a child solving more benchmark tasks does not by
itself establish successor improvement.

Canonical source: [improvement types](../../content/concepts/improvement-types.md).

## Unnormalized selection weight

$$
w_i = s_i h_i
$$

It combines sigmoid-scaled performance with the child-count bonus.

Common misunderstanding: \(w_i\) is not comparable across archives until it is
normalized over the current eligible set.

Canonical source: [DGM system article](../../content/systems/dgm.md).

## Valid child

In the paper, a child that compiles and retains code-editing functionality. In
the released code, viability requires evaluation metadata, at least one
non-empty submitted patch among resolved and unresolved tasks, and enough
evaluated tasks.

Common misunderstanding: valid means evaluable and functioning, not better,
safe, or production-ready.

Canonical source: [DGM system article](../../content/systems/dgm.md).

## Selection probability

$$
p_i = \frac{w_i}{\sum_j w_j}
$$

Parents are sampled with replacement from this categorical distribution.

Common misunderstanding: low-scoring eligible parents can retain nonzero
probability.

Canonical source: [DGM system article](../../content/systems/dgm.md).

Back to the [DGM index](darwin_godel_machine_index.md).
