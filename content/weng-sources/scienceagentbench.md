---
source_id: SCIENCEAGENTBENCH
title: "ScienceAgentBench: Toward Rigorous Assessment of Language Agents for Data-Driven Scientific Discovery"
weng_locator: reference-33
section_id: future-challenges
primary_url: https://arxiv.org/abs/2410.05080
captured_path: evidence/weng/text/scienceagentbench.txt
publication_state: "ICLR 2025 reported by anchor; captured arXiv:2410.05080v3 paper header agrees; official status not independently verified"
evidence_state: card-complete
edited_object_family: evaluation-benchmark
claim_ceiling: Sections 2–4, Ethics, Limitations, and Appendix E.1 102-task scientific Python-program benchmark, expert-and-annotator validation, selected-best-versus-mean success-rate accounting, Claude-3.5-Sonnet self-debug and o1-preview endpoints and costs, taxonomy discrepancy, contamination mitigations, and deployment limits; no independent reproduction
lesson_ids: 0009,0010
card_path: content/weng-sources/scienceagentbench.md
canonical_route: content/weng/09-future-challenges.md
---

# Testing Scientific Code Before End-to-End Claims

## Problem

Lines 94–101 say end-to-end paper evaluation alone is insufficient to support end-to-end automation claims; assess individual tasks before bold claims.

## Core mechanism

From task instructions, data, and optional expert-provided knowledge, an agent emits one self-contained Python program. Lines 102–118 and 218–279 derive 102 tasks from 44 papers. Nine annotators adapt and rerun code; nine experts validate realism and rubrics. Point removal and re-splitting mitigate contamination and shortcuts without guarantees. Lines 521–563 scope a two-rater rubric study to 102 Claude-3.5-Sonnet + self-debug programs generated with expert knowledge.

## Reported evidence

Across 102 paper-derived tasks in bioinformatics, computational chemistry, geographical information science, and psychology and cognitive neuroscience, SR is the percentage meeting every output criterion; execution failure scores zero. Table 3 selects each task’s best of three independent runs lexicographically by maximum SR, then maximum VER, then maximum CBS, then minimum cost. Claude self-debug reports 32.4%/$0.057 without and 34.3%/$0.061 with expert knowledge; o1-preview + self-debug reports 42.2%/$0.636 without. Appendix E.1 instead gives mean (SD) SRs: Claude 22.9 (2.0)/27.8 (2.0), o1-preview 27.1 (1.2)/27.8 (1.7), without/with knowledge. Harp did not reproduce these results.

## Key limitation

Limitations lines 1161–1196 cover public-data Python programs, four domains, ten-minute tasks, and imperfect metrics. Ethics lines 577–601 say agents lack laboratory access; chemistry or bioinformatics deployment still needs human controls over labs, reagents, and equipment.

## Why Weng cites it

Weng lines 379–381 labels the domains "math, chemistry, biology, geography." The primary taxonomy is Bioinformatics, Computational Chemistry, Geographical Information Science, and Psychology & Cognitive Neuroscience; Weng substitutes math for psychology/cognitive neuroscience.
