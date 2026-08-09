---
source_id: KARPATHY-AUTORESEARCH
title: Karpathy Autoresearch
weng_locator: body-link-workflow-automation
section_id: harness-design-patterns
primary_url: https://github.com/karpathy/autoresearch
captured_path: evidence/weng/text/karpathy-autoresearch.txt
publication_state: GitHub repository
evidence_state: card-complete
edited_object_family: implementation-case-study
claim_ceiling: Present-day README and program workflow behavior at the pinned commit; no execution, benchmark reproduction, or RSI efficacy claim
lesson_ids: 0001,0002,0010
card_path: content/weng-sources/karpathy-autoresearch.md
canonical_route: content/systems/autoresearch.md
---

# Bounding an Automated Training Loop

## Problem

The pinned README asks how an agent can run comparable model-training experiments overnight while keeping edits, evaluation, and decisions reviewable.

## Core mechanism

The agent edits only train.py. Each run trains candidate model weights for five minutes and reports validation bits per byte, where lower is better. The persistent search object is the training program: the agent commits an idea, runs it, logs the metric, keeps improvements, resets regressions, and repeats. Data, evaluator, time budget, and agenda stay human-defined.

## Reported evidence

The capture and pinned implementation specify roughly 12 experiments per hour, one editable training file, one scalar metric, and a tab-separated keep, discard, or crash log. These are workflow specifications, not measured efficacy. Harp did not execute the repository or reproduce a benchmark.

## Key limitation

Results depend on one machine and are not comparable across hardware. The source does not establish agenda autonomy, a better research procedure, or RSI. The human writes program.md and fixes the objective, dataset, metric, permissions, and acceptance rule.

## Why Weng cites it

Weng line 40 directly presents Autoresearch as a clean workflow-automation example for plan, execute, test, improve, and repeat. She does not claim it proves recursive improvement.
