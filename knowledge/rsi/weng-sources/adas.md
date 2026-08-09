---
source_id: ADAS
title: Automated Design of Agentic Systems
weng_locator: reference-13
section_id: workflow-design-and-search
primary_url: https://proceedings.iclr.cc/paper_files/paper/2025/file/36b7acf6f6010652b3f2a433774a66fe-Paper-Conference.pdf
captured_path: evidence/weng/text/adas.txt
publication_state: ICLR 2025 official proceedings
evidence_state: card-complete
edited_object_family: workflow-or-agent-program-search
claim_ceiling: Inspected official-paper mechanism, held-out results, subset and run setup, transfer, cost, safety, and limitations; author-reported; no independent reproduction
lesson_ids: 0005,0010
card_path: knowledge/rsi/weng-sources/adas.md
canonical_route: knowledge/rsi/systems/adas.md
---

# Searching Complete Agent Programs

## Problem

Captured lines 134–165 frame agent design as a search problem broader than prompt tuning. The editable object is executable agent code, while foundation models, evaluators, datasets, and budgets remain fixed.

## Core mechanism

Meta Agent Search initializes an archive, asks a meta-agent to describe and implement a novel agent, self-refines the proposal, evaluates it on development tasks, and stores code plus metrics. Later proposals see the growing archive.

## Reported evidence

The authors report independent domain searches at 79.4 F1 on DROP and 53.4% accuracy on MGSM, versus listed hand-designed baselines at 65.8 and 39.0. Cross-domain transfer evaluates MGSM agents on held-out domains. Cross-model transfer instead re-evaluates on ARC the top three agents selected by GPT-3.5 ARC test accuracy. Harp did not reproduce this.

## Key limitation

The source uses GPT-4 as meta-agent, GPT-3.5 as executor, finite validation/test subsets, 25 or 30 search iterations, and reports about $500 per ARC run or $300 per reasoning-domain run. It optimizes performance only and warns generated code can be destructive.

## Why Weng cites it

Weng lines 164–174 uses ADAS to make workflow design an optimization problem over agent code and an archive, not merely manual orchestration.
