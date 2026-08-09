---
id: dgm-knowledge-index
title: Darwin Gödel Machine knowledge index
type: index
status: active
created: 2026-08-08
updated: 2026-08-08
tags: [darwin-godel-machine, self-improvement, open-ended-search, coding-agents]
confidence: high
canonical: ../../content/systems/dgm.md
---

# Darwin Gödel Machine

> This file is a learning projection. Canonical claims live under `content/`;
> primary-source captures and pinned implementation files live under
> `evidence/`.

This packet teaches the Darwin Gödel Machine as an engineered system. It covers
the paper's argument, the released code, the selection algorithm, the benchmark
design, the safety case, and the evidence needed for a stronger recursive
self-improvement claim.

The shortest accurate description is:

> DGM keeps a branching archive of coding-agent implementations. A selected
> parent receives a general improvement task derived from its benchmark
> failures, edits its own agent repository, and produces a child. An external
> controller evaluates the child and may retain it as a future parent.

That is a real harness-improvement loop. It is not yet a matched demonstration
that improved children become better producers of later improved children.

## Choose a route

### Expert route

Use this route for a design review or architecture assessment:

1. [System architecture](04_system_architecture.md)
2. [Repository walkthrough](05_repository_walkthrough.md)
3. [Evaluation analysis](06_evaluation_analysis.md)
4. [Critical review](09_critical_review.md)
5. [Successor design](10_successor_design.md)

Expected reading time: 90 to 150 minutes.

### Guided route

Use this route to learn the mechanism from first principles:

1. [Orientation](01_orientation.md)
2. [Paper walkthrough](02_paper_walkthrough.md)
3. [Algorithm derivation](03_algorithm_derivation.md)
4. [Open-endedness](07_open_endedness.md)
5. [System architecture](04_system_architecture.md)
6. [Repository walkthrough](05_repository_walkthrough.md)
7. [Evaluation analysis](06_evaluation_analysis.md)
8. [Safety and failure](08_safety_and_failure.md)
9. [Critical review](09_critical_review.md)
10. [Successor design](10_successor_design.md)
11. [Learning path](learning_path.md)

Expected reading time: 4 to 7 hours, including exercises.

## Route by question

| Question | Start here | Continue with |
|---|---|---|
| What is DGM actually changing? | [Orientation](01_orientation.md) | [System architecture](04_system_architecture.md) |
| How does parent selection work? | [Algorithm derivation](03_algorithm_derivation.md) | [Open-endedness](07_open_endedness.md) |
| Why retain worse agents? | [Open-endedness](07_open_endedness.md) | [Evaluation analysis](06_evaluation_analysis.md) |
| How does the released code implement the paper? | [Repository walkthrough](05_repository_walkthrough.md) | [System architecture](04_system_architecture.md) |
| What do the benchmark numbers include? | [Evaluation analysis](06_evaluation_analysis.md) | [Claim crosswalk](claim_evidence_crosswalk.md) |
| Does DGM demonstrate recursive self-improvement? | [Critical review](09_critical_review.md) | [Successor design](10_successor_design.md) |
| What could go wrong? | [Safety and failure](08_safety_and_failure.md) | [Critical review](09_critical_review.md) |
| How would I build a stronger experiment? | [Successor design](10_successor_design.md) | [Learning path](learning_path.md) |

## Packet map

### Foundations

- [Orientation](01_orientation.md) establishes the vocabulary and claim ceiling.
- [Paper walkthrough](02_paper_walkthrough.md) reconstructs the paper's causal
  argument and identifies the appendices that carry important evidence.
- [Glossary](glossary.md) owns the packet's terms and symbols.

### Mechanism

- [Algorithm derivation](03_algorithm_derivation.md) derives parent selection,
  archive admission, and the baseline algorithms.
- [System architecture](04_system_architecture.md) separates mutable candidate
  code from the protected outer loop.
- [Open-endedness](07_open_endedness.md) explains stepping stones, search
  diversity, and why `keep_all` matters.

### Implementation

- [Repository walkthrough](05_repository_walkthrough.md) traces the released
  implementation at commit
  `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.

### Evidence and judgment

- [Evaluation analysis](06_evaluation_analysis.md) accounts for tasks, models,
  metrics, costs, ablations, transfer, and reproduction status.
- [Safety and failure](08_safety_and_failure.md) examines containment,
  evaluator gaming, and objective hacking.
- [Critical review](09_critical_review.md) gives a consequence-ordered MTS
  assessment.
- [Claim-evidence crosswalk](claim_evidence_crosswalk.md) maps material claims
  to their sources and caveats.

### Design and teaching

- [Successor design](10_successor_design.md) specifies a more rigorous DGM-like
  experiment.
- [Learning path](learning_path.md) moves from explanation to experiment
  design.
- [Maintenance](maintenance.md) records update and verification rules.
- [Source registry](source_registry.md) states what each source can prove.

## Source boundary

The packet relies on three evidence classes:

1. **Canonical Harp synthesis.** The main entry is
   [the canonical DGM system article](../../content/systems/dgm.md).
2. **Primary paper evidence.** The checked-in paper text is
   [the DGM capture](../../evidence/weng/text/dgm.txt), corresponding to
   arXiv `2505.22954v3`.
3. **Pinned implementation evidence.** The narrow source snapshot begins at
   [the DGM implementation evidence root](../../evidence/implementations/dgm/snapshot/README.md).

The paper supports method and author-reported result claims. The source
snapshot supports present-day implementation claims at its pinned commit.
Neither proves independent reproduction.

## Current technical judgment

DGM makes three useful advances:

- it treats the coding-agent implementation as the edited object;
- it lets accepted descendants participate in later self-modification; and
- it preserves multiple lineages instead of replacing one incumbent.

The main unresolved claim is causal. Better benchmark-solving ability is used
as a proxy for better future self-improvement ability. The paper does not
directly compare a parent and child on their ability to produce later accepted
children under a matched protected envelope.

## Packet status

- Paper: arXiv `2505.22954v3`, published as an ICLR 2026 conference paper in
  the captured source.
- Repository snapshot:
  `a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.
- Independent benchmark reproduction: not performed.
- Packet authority: learning projection only.

Use [maintenance](maintenance.md) before updating any result, implementation
claim, or source identity.
