---
source_id: HARNESS-DISENTANGLE
title: "Harness Updating Is Not Harness Benefit: Disentangling Evolution Capabilities in Self-Evolving LLM Agents"
weng_locator: reference-36
section_id: harness-layer-vs-core-intelligence
primary_url: https://arxiv.org/abs/2605.30621
captured_path: evidence/weng/text/harness-disentangle.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: evaluation-benchmark
claim_ceiling: Formal capability decomposition in §§3.1–3.3; author-reported seven-model, three-benchmark results and activation/adherence analysis in §4; fixed-weight scope, audit controls, and deployment limitations in §§6–7
lesson_ids: 0003,0006,0010
card_path: content/weng-sources/harness-disentangle.md
canonical_route: content/systems/harness-disentangle.md
---

# Separating Updating from Harness Benefit

## Problem

Captured lines 265–276 argue that one post-evolution score confounds base task skill, the evolver's update quality, and the task agent's ability to use an update.

## Core mechanism

The framework measures base capability, harness-updating gain across anchor agents, and harness-benefit across anchor evolvers. It then audits whether skills activate, whether agents follow them, and whether adherence persists to final validation. Model weights stay fixed; this is evaluation, not an improver.

## Reported evidence

Across seven models and three benchmarks, the paper reports at most a 3.1-point spread in update quality on any benchmark, while harness benefit is non-monotonic. On SkillsBench, Qwen3-32B adherence falls from 0.52 after loading to 0.13 at final validation; Opus 4.6 falls from 0.89 to 0.80. Harp did not reproduce these results.

## Key limitation

The source limits the study to fixed weights, seven selected models, three benchmarks, and LLM-judged adherence; it says the model grid is not exhaustive. Harp's boundary is that benchmark audit controls do not establish safe deployment of persistent updates.

## Why Weng cites it

Weng lines 213–215 uses the decomposition to argue that writing a useful harness and benefiting from it are distinct capabilities.
