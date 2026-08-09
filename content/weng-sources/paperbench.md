---
source_id: PAPERBENCH
title: "PaperBench: Evaluating AI's Ability to Replicate AI Research"
weng_locator: reference-30
section_id: future-challenges
primary_url: https://proceedings.mlr.press/v267/starace25a.html
captured_path: evidence/weng/text/paperbench.txt
publication_state: ICML 2025 official proceedings
evidence_state: card-complete
edited_object_family: evaluation-benchmark
claim_ceiling: Official §§2–5 and relevant appendix evidence for the 20-paper, 8,316-rubric benchmark, agent and human run accounting, JudgeEval, reported results, and limits; no independent reproduction
lesson_ids: 0009,0010
card_path: content/weng-sources/paperbench.md
canonical_route: content/weng/09-future-challenges.md
---

# Measuring Full Paper Replication

## Problem

Captured lines 15–42 ask whether coding agents can understand and replicate empirical contributions from research papers, rather than merely write plausible manuscripts.

## Core mechanism

PaperBench contains 20 ICML 2024 Spotlight or Oral papers and 8,316 weighted leaf criteria co-developed with authors. Agents build codebases and run experiments; an LLM judge scores code, execution, and result matching. This is a benchmark, not an improving system.

## Reported evidence

The authors report Claude 3.5 Sonnet with BasicAgent at 21.0% across 20 papers, using three runs per paper capped at 12 hours each. On the same three-paper subset, human best@3 reached 41.4% after 48 active hours and an extended 36-hour o1 IterativeAgent reached 26.6%; these are not time-matched. Separately, Table 4 reports 26.0% ±0.3 as the full-PaperBench extended-limit average. Harp did not reproduce this.

## Key limitation

The human result covers three papers and different time accounting. JudgeEval's 0.83 is macro-averaged F1 across papers, and the source says the judge is below expert accuracy and nondeterministic. Harp's boundary is that replication does not measure novel discovery.

## Why Weng cites it

Weng lines 369–373 lists PaperBench as a future-challenges benchmark for research replication and notes that the roughly 21% best tested agent remained below ML PhDs.
