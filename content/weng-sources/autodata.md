---
source_id: AUTODATA
title: "Autodata: An agentic data scientist to create high quality synthetic data"
weng_locator: reference-12
section_id: workflow-design-and-search
primary_url: https://arxiv.org/abs/2606.25996
captured_path: evidence/weng/text/autodata.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: automated-research-system
claim_ceiling: Sections 2–3 and 6 Agentic Self-Instruct workflow, author-reported controlled training-data results, strong-solver boundary, and stated limitations; no independent reproduction
lesson_ids: 0005,0010
card_path: content/weng-sources/autodata.md
canonical_route: content/weng/05-workflow-design-and-search.md
---

# Searching for Useful Training Questions

## Problem

Captured lines 28–65 ask how synthetic-data generation can control task quality and difficulty instead of relying on one-pass prompting or post-hoc filtering.

## Core mechanism

Agentic Self-Instruct at lines 83–118 has a main data-scientist agent orchestrate a challenger, weak solver, strong solver, and judge. Solver outputs and judge feedback update the input prompt or recipe sent to the challenger, which generates another example. Accepted examples train weak Qwen3.5-4B; strong Qwen3.5-397B-A17B stays fixed.

## Reported evidence

Table 2 at lines 156–166 reports training-step-200 results on the 100-prompt Agentic half of a paired 200-prompt evaluation. Mean@3 is the average Kimi-K2.6 rubric score across three sampled answers on the table’s 0–1 scale, where higher is better. Qwen3.5-4B reaches 0.632 after training on 1,300 Agentic examples, versus 0.500 after equal-budget chain-of-thought Self-Instruct training and 0.366 without additional training. Lines 238–247 define the two 100-prompt halves; the strong solver stays fixed. Harp did not reproduce these author results.

## Key limitation

Lines 640–655 call for broader tasks, models, and dataset-level analysis. The authors observed agents weakening the weak-solver prompt and questions overfitting paper-specific numbers. The reported loop improves data and the weak model; it does not improve the strong solver.

## Why Weng cites it

Weng lines 158–160 presents Autodata as iterative workflow design, but calls its fixed strong solver an indirect-distillation boundary on recursive self-improvement.
