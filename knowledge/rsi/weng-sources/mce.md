---
source_id: MCE
title: Meta Context Engineering via Agentic Skill Evolution
weng_locator: reference-8
section_id: context-engineering
primary_url: https://arxiv.org/abs/2601.21557
captured_path: evidence/weng/text/mce.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: context-management-mechanism
claim_ceiling: Bi-level skill/context evolution mechanism, fixed meta-agent and base-level setup, author-reported aggregate results, evaluation subsets, and limitations; no matched successor-producer test or demonstrated RSI; no independent reproduction
lesson_ids: 0004,0010
card_path: knowledge/rsi/weng-sources/mce.md
canonical_route: knowledge/rsi/systems/mce.md
---

# Evolving How Context Is Built

## Problem

Captured lines 103–116 argue that context systems still depend on handcrafted schemas and update workflows. MCE asks whether the context-engineering procedure itself can be searched.

## Core mechanism

An outer meta-agent performs agentic crossover over prior skills, executions, and metrics. An inner base agent executes each skill to build context as files and code from rollouts. The skill and context artifact co-evolve; model weights stay unchanged.

## Reported evidence

The authors report a 5.6–53.8% relative-improvement range over compared state-of-the-art context-engineering methods and a 16.9% mean aggregate. This is their comparison aggregate, not a simple arithmetic average over five domain scores. Harp did not reproduce it.

## Key limitation

The source fixes MiniMax M2.1 as its default meta-agent and uses budget-constrained data subsets. Harp's boundary is that it does not test whether an accepted skill becomes a better producer of later accepted skills under matched conditions, so it does not demonstrate RSI.

## Why Weng cites it

Weng lines 117–136 uses MCE to separate the mechanism for managing context from the resulting context, moving the editable object from an artifact toward the procedure that creates it.

The complete paper-derived protocol, including Appendix A–E contracts and the
explicit no-local-reproduction boundary, is maintained at
knowledge/harness_benchmarks/harness_benchmark_field_guide.md.
