---
id: rsi-harness-search
kind: concept
title: Searching for better harnesses
summary: Context, workflow, code, population, archive, mutation, selection, diversity, and open-ended search over agent systems.
primary_parent: modeling
additional_parents:
  - mlsys
related:
  - kind: optimizes
    target: rsi-harness-engineering
  - kind: evaluated-by
    target: rsi-evaluation-promotion-containment
attachments:
  - content/source_registry.md
  - content/claim_evidence_ledger.md
  - content/context_engineering_deep_dive.md
  - knowledge/darwinx/darwinx_index.md
claims: []
human_review: null
---

# Searching for better harnesses

## Search object and objective

Harness search treats agent configuration or code as an optimization variable. A candidate may contain instructions, context policy, retrieval, tool schemas, control flow, delegation, memory, verification, and recovery logic. The evaluator executes candidates on development and held-out tasks, then an external selector retains valid variants.

Let `h ∈ H` be a harness candidate, `T` a task distribution, and `b` a matched root-tree budget. A simple objective is:

`F(h) = E_t∼T[S(h,t)] − λ Cost(h,t) − μ Risk(h,t)`

`S` is task outcome after hard correctness gates, `Cost` includes all descendant work, and `Risk` measures integrity or authority violations. In practice, correctness and evaluator integrity should be constraints rather than terms that a high task score can buy away.

## Search dimensions

- **Context optimization** changes retained records, ordering, summaries, and selection policies.
- **Workflow search** changes a graph of model calls, tools, branches, and aggregators.
- **Harness-code search** edits executable implementation and tests.
- **Population search** evaluates several variants per generation.
- **Archive search** samples parents from a lineage rather than only the current best.
- **Open-ended search** rewards novelty or diversity so one local optimum does not dominate.

Mutation operators should match the representation. Text edits fit prompts and skills. Graph rewrites fit workflows. Typed code edits fit harness implementation. Cross-representation mutation is powerful but harder to attribute.

## Population and archive algorithm

1. Select one or more parents from the valid archive.
2. Provide bounded traces and failure classes, not held-out answers.
3. Generate mutations with declared target, changed paths, and predicted effect.
4. Reject candidates that violate schema or mutation boundaries.
5. Run development checks, then protected held-out and integrity evaluation.
6. Record every result, including invalid and failed variants.
7. Add valid candidates to an archive indexed by score and behavioral descriptors.
8. Select future parents using quality, diversity, and cost.
9. Stop at the precommitted budget or when integrity fails.

Quality-diversity methods keep elites in behavior cells rather than collapsing the archive to one scalar winner. A descriptor might encode tool strategy, decomposition depth, cost profile, or failure coverage. The descriptor must not leak protected task identity.

<details>
<summary>Original sources for this mechanism</summary>

- ACE, §§2–4; MCE, §§2–3; and Meta-Harness, §§2–4, define context, skill, and end-to-end harness optimization variants: [ICLR 2026 poster 10008343](https://iclr.cc/virtual/2026/poster/10008343), [arXiv:2601.21557](https://arxiv.org/abs/2601.21557), [arXiv:2603.28052v1](https://arxiv.org/abs/2603.28052).
- STOP, §2; ADAS, §§2–3; AFlow, §§2–3; Self-Harness, §§3–4; and AHE, §§2–4, define scaffolding, agent-system, workflow, regression-gated, and observability-driven search: [arXiv:2310.02304v3](https://arxiv.org/abs/2310.02304), [ICLR 2025 paper](https://proceedings.iclr.cc/paper_files/paper/2025/file/36b7acf6f6010652b3f2a433774a66fe-Paper-Conference.pdf), [OpenReview z5uVAKwmjf](https://openreview.net/forum?id=z5uVAKwmjf), [arXiv:2606.09498v1](https://arxiv.org/abs/2606.09498), [arXiv:2604.25850v4](https://arxiv.org/abs/2604.25850).
- Darwin Gödel Machine, §§2–3, and Pugh et al., "Quality Diversity," §§2–3, define branching archives and diversity-preserving selection: [arXiv:2505.22954v3](https://arxiv.org/abs/2505.22954), [DOI:10.3389/frobt.2016.00040](https://doi.org/10.3389/frobt.2016.00040).
- DarwinX, §§2–9 and Appendices A–E, defines bounded-regression promotion, steering confirmation, specialist retention, and attempted cross-lineage recombination over harness variants: [arXiv:2608.07545v1](https://arxiv.org/abs/2608.07545v1). The [DarwinX packet](../../darwinx/darwinx_index.md) audits the paper's system-level gains separately from its unisolated population operators.
- FunSearch, Methods, and AlphaEvolve, §§2–3, define program mutation under executable evaluators: [Nature 625](https://www.nature.com/articles/s41586-023-06924-6), [arXiv:2506.13131v1](https://arxiv.org/abs/2506.13131).

</details>

## Worked examples

### ACE, MCE, and Meta-Harness

ACE evolves context artifacts. MCE adds a bi-level relation between skills and context use. Meta-Harness broadens the editable object toward context-management code and end-to-end harness behavior. These systems show that the search representation can move from text records toward executable policy. Their reported results do not establish independent multi-generation RSI.

The [context-engineering deep dive](../context_engineering_deep_dive.md)
provides the detailed state decomposition, algorithms, cost model, and matched
ablations behind this progression.

### STOP

STOP recursively improves a scaffolding program with fixed model weights. The recursive object is explicit and executable. Its utility function and downstream task set remain external, which makes the result bounded and interpretable.

### ADAS and AFlow

ADAS searches agent-system code. AFlow searches workflows using Monte Carlo tree search. Both automate system design around a model. Their claim depends on held-out evaluation and transfer beyond the search tasks.

### Self-Harness and AHE

Self-Harness mines weaknesses, proposes harness changes, and applies regression gates. AHE adds observability and asks edits to predict their effects. These features improve attribution, but the benchmark and evaluator still define the optimization target.

### DGM

DGM evolves coding-agent repositories and retains a branching archive rather than replacing one incumbent. This supports cumulative search and retrospective analysis. The foundation model and benchmark remain fixed external components.

### DarwinX

DarwinX keeps a tree of harness variants, screens children with net gain and
bounded regression mass, requires higher-fidelity confirmation before a node
may steer search, and attempts to merge specialists while preserving the union
of their wins. Its reported matched-model and held-out gains support durable
harness improvement. They do not isolate the archive, parent selector,
regression gate, merge operator, or inference effort under one fixed search
budget. See the [evidence-backed DarwinX packet](../../darwinx/darwinx_index.md).

## Failure modes and tradeoffs

- **Benchmark overfitting.** Repeated search exploits a finite held-out suite.
- **Evaluator hacking.** Candidates alter tests, logs, score parsing, or judge presentation.
- **Archive collapse.** Selection removes diverse stepping stones.
- **Mutation drift.** Large edits prevent causal attribution.
- **Activation failure.** A proposed prompt or skill exists but is never used.
- **Inherited debt.** Children accumulate opaque code and slow later search.
- **Population cost.** More variants increase the chance of a winner without improving the mutation policy.
- **Descriptor gaming.** Novelty metrics reward irrelevant behavior.
- **Cross-model brittleness.** A harness adapts to one model's quirks.

Small mutation spaces improve attribution but cap discovery. Open-ended spaces permit architectural change but increase evaluation and containment demands.

## What would weaken the mechanism

A search claim weakens if gains vanish on delayed tasks, if a random or resource-matched baseline finds the same improvement, if selected edits are not activated, or if repeated fresh runs do not reproduce the lineage. A recursive claim further requires evidence that an accepted harness improves later harness search, not merely the current benchmark score.

## Open technical questions

- Which behavior descriptors preserve useful diversity without encoding task identity?
- How should archives age candidates when tools and models change?
- Can mutation size be normalized across prompts, workflow graphs, and code?
- When does evaluator cost dominate proposer quality in open-ended search?

<details>
<summary>Reference records and operational metadata</summary>

- Full source metadata and result limits are in [the source registry](../source_registry.md) and [claim ledger](../claim_evidence_ledger.md).
- Uninspected systems remain identity-only worked examples in the coverage map.
- No source registry is expanded in this chapter's main flow.

</details>
