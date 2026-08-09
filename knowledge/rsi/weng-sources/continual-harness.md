---
source_id: CONTINUAL-HARNESS
title: "Continual Harness: Online Adaptation for Self-Improving Foundation Agents"
weng_locator: reference-38
section_id: joint-harness-weight-optimization
primary_url: https://arxiv.org/abs/2605.09998
captured_path: evidence/weng/text/continual-harness.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: joint-harness-and-weight-adaptation
claim_ceiling: Inspected §§2–4 and Appendix D reset-free harness-refinement cost and completion cells, DAgger-style teacher relabeling, process-reward scoring, author-reported co-learning results, game, teacher, and evaluator limits, and unresolved 31B-versus-26B initial-policy identity; no independent reproduction
lesson_ids: 0008,0010
card_path: knowledge/rsi/weng-sources/continual-harness.md
canonical_route: knowledge/rsi/systems/continual-harness.md
---

# Co-Learning a Harness and Policy without Resets

## Problem

The paper asks whether a foundation agent can refine its prompt, subagents, skills, and memory during one continuing episode, then train a policy on trajectories produced under that changing harness.

## Core mechanism

An outer refiner performs reset-free create, read, update, and delete operations on harness state. For co-learning, low-reward windows are scored by a process reward model, relabeled by a stronger teacher, and used for soft supervised policy updates. Both harness and model weights change.

## Reported evidence

For individual 24-hour Emerald seeds, the paper's Pro cell uses Gemini 3.1 Pro. Its per-cell medians put from-scratch HCH at 100% of 31 milestones for $130, versus Hmin at 98% for $215. Figure 7 plots only five advancing Red co-learning runs, with gains of +2, +3, +3, +4, and +5. Harp did not reproduce this.

## Key limitation

The source says teacher dependence, game scope, and reset-free comparison remain limits. Its appendix calls 31B the viable Red initial policy but later says 26B, an unresolved discrepancy. Harp's boundary is that teacher, process-reward, and evaluator choices confound causal attribution.

## Why Weng cites it

Weng line 311 uses Continual Harness as a long-horizon example that combines harness updating with policy learning from a stronger teacher's labels on low-reward trajectories.
