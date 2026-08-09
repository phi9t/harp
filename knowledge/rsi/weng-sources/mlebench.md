---
source_id: MLEBENCH
title: "MLE-bench: Evaluating Machine Learning Agents on Machine Learning Engineering"
weng_locator: reference-32
section_id: future-challenges
primary_url: https://arxiv.org/abs/2410.07095
captured_path: evidence/weng/text/mlebench.txt
publication_state: "arXiv:2410.07095v6; captured paper header says published as an ICLR 2025 conference paper; official status not independently verified"
evidence_state: card-complete
edited_object_family: evaluation-benchmark
claim_ceiling: Sections 2–3 and 6 offline Kaggle benchmark contract, private-leaderboard medal metric, author-reported o1-preview with AIDE 16-seed mean-and-SEM endpoint, public-versus-private leaderboard discrepancy, human-comparator caveats, and contamination, split, scope, and resource limits; no independent reproduction
lesson_ids: 0009,0010
card_path: knowledge/rsi/weng-sources/mlebench.md
canonical_route: knowledge/rsi/weng/09-future-challenges.md
---

# Measuring Offline Kaggle Engineering

## Problem

Captured lines 37–47 ask whether agents can complete end-to-end machine-learning engineering rather than isolated coding tasks.

## Core mechanism

Lines 103–169 define 75 scored offline Kaggle competitions plus seven development tasks, with descriptions, data, local graders, and leaderboard snapshots. Most unavailable test sets are reconstructed holdouts. A score-free validation server checks submission validity.

## Reported evidence

Across 75 Kaggle ML-engineering competitions spanning natural language processing, computer vision, and signal processing, the headline metric is the percentage of attempts earning any medal, bronze or above; higher is better. Table 2 at lines 248–286 reports o1-preview with purpose-built AIDE at 16.9% ± 1.1, mean ± one SEM over 16 seeds. Each attempt had 24 hours, one A10 GPU, 36 vCPUs, 440 GB RAM, and 4095 GiB storage. Harp did not reproduce these results.

## Key limitation

Lines 285–291 and 513–545 warn that not all chosen competitions originally awarded medals, datasets and grading changed, and agents use newer technology than historical competitors. Public-data contamination and reconstructed holdouts are comparability risks. The authors found no systematic contamination effect for GPT-4o but make no guarantee for future models. One seed costs 1,800 GPU-hours and 142.5 million tokens. The benchmark evaluates core ML-engineering competencies, not the full open-ended ML-research workflow or RSI.

## Why Weng cites it

Weng lines 389–394 says "public leaderboards." The primary experiment explicitly compares medals against Private leaderboard snapshots at lines 157–169. Harp follows the primary experiment and keeps Weng's broader wording as an attribution discrepancy.
