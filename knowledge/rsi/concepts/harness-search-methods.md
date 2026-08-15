---
id: rsi-harness-search-methods
kind: concept
title: Harness search methods
summary: Context, workflow, code, population, quality-diversity, and open-ended search over agent systems.
primary_parent: rsi-harness-search
additional_parents: []
related: []
attachments:
  - content/chapters/harness-search.md
claims: []
human_review: null
---

# Harness search methods

Harness search chooses which part of agent policy is editable and which process proposes and selects candidates. The search method determines cost, diversity, and how easily evaluator leakage can enter.

## Context optimization

Context optimization edits instructions, examples, retrieval policy, or memory without changing control code. It is fast to evaluate but sensitive to model version and task sampling. Held-out tasks and prompt-length accounting prevent memorization and extra context from appearing as general improvement.

## Workflow search

Workflow search changes the graph of model calls, tools, branches, and aggregators. Candidate workflows need typed execution semantics so malformed graphs fail before evaluation. Search cost includes every node executed, not only the final answer.

## Harness code search

Harness-code search can alter loops, tool adapters, recovery logic, and evaluation plumbing. Its expressive power also creates the largest integrity risk. Candidates must run in isolation and must not modify tests, held-out data, budget accounting, or promotion code.

## Population search

Population search keeps several candidates and creates variants by mutation, recombination, or model-generated edits. A population can preserve alternatives that greedy hill climbing would discard, but it multiplies evaluation cost and requires lineage-aware deduplication.

## Quality diversity

Quality-diversity methods archive candidates by both performance and behavioral descriptors. A candidate enters or replaces a cell only when it improves quality for that region. Descriptor choice controls what diversity survives; a weak descriptor can preserve cosmetic variants rather than distinct capabilities.

## Open-ended search

Open-ended search allows objectives, niches, or candidate structures to expand over time instead of optimizing one fixed target. It can discover unexpected methods, but evaluation becomes harder as tasks and candidates co-evolve. Protected safety and capability tests must remain comparable even when the search frontier changes.

<details>
<summary>Original sources for this mechanism</summary>

- [[knowledge/rsi/chapters/harness-search#Population and archive algorithm|Searching for better harnesses]] gives the archive algorithm and matched-budget objective.
- Pugh et al., "Quality Diversity: A New Frontier for Evolutionary Computation," taxonomy and §§2–3: [DOI:10.3389/frobt.2016.00040](https://doi.org/10.3389/frobt.2016.00040).
- AlphaEvolve, §§2–3, and FunSearch, Methods, provide program-search examples with executable evaluators: [arXiv:2506.13131v1](https://arxiv.org/abs/2506.13131), [Nature 625](https://www.nature.com/articles/s41586-023-06924-6).

</details>
