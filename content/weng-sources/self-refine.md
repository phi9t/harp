---
source_id: SELF-REFINE
title: "Self-Refine: Iterative Refinement with Self-Feedback"
weng_locator: reference-14
section_id: workflow-design-and-search
primary_url: https://arxiv.org/abs/2303.17651
captured_path: evidence/weng/text/self-refine.txt
publication_state: "NeurIPS 2023 reported by anchor; official status not verified"
evidence_state: card-complete
edited_object_family: within-episode-output-refinement
claim_ceiling: Iterative feedback/refinement algorithm, author-reported task results, and stated limitations; no persistent cross-episode or RSI claim
lesson_ids: 0005,0010
card_path: content/weng-sources/self-refine.md
canonical_route: content/weng/05-workflow-design-and-search.md
---

# Refining One Output with Self-Feedback

## Problem

Captured lines 55–103 ask whether a capable language model can improve a draft without supervised refinement training, external reward models, or human feedback.

## Core mechanism

The same frozen model alternates natural-language feedback and refinement, retains prior outputs and feedback in context, and stops on a task rule or after at most four iterations. It changes the current output only; no weights or persistent cross-episode procedure are updated.

## Reported evidence

Across seven tasks, Table 1 combines task-specific metrics with GPT-4 proxy preference. Blind human A/B evaluation is reported separately on output subsets. One bounded endpoint is GPT-4 code optimization rising from 27.3% to 36.0%, an 8.7-point gain. Harp did not reproduce these author-reported results.

## Key limitation

Lines 493–505 require sufficient few-shot or instruction-following ability, test proprietary models, and cover English-only datasets. Math gains are near zero when the model cannot detect its own errors; preference judges are task-dependent proxies.

## Why Weng cites it

Weng lines 166–169 mentions Self-Refine only as ADAS's two-step proposal check. Harp's interpretation is that it supplies a bounded within-output refinement primitive, not persistent improvement or RSI.
