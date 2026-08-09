---
source_id: GEPA
title: "GEPA: Reflective Prompt Evolution Can Outperform Reinforcement Learning"
weng_locator: reference-19
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2507.19457
captured_path: evidence/weng/text/gepa.txt
publication_state: "ICLR 2026 oral reported by captured paper; official venue record not independently verified"
evidence_state: card-complete
edited_object_family: evolutionary-program-or-population-search
claim_ceiling: Sections 2–4 and Appendix D–E reflective compound-system prompt evolution, author-reported benchmark and rollout results, merge behavior, and evaluator-dependent limits; no independent reproduction
lesson_ids: 0007,0010
card_path: knowledge/rsi/weng-sources/gepa.md
canonical_route: knowledge/rsi/weng/07-evolutionary-search.md
---

# Reflecting on Traces to Evolve System Prompts

## Problem

Captured lines 99–138 ask whether natural-language execution traces and evaluator feedback can adapt compound language-model systems with fewer rollouts than scalar-reward reinforcement learning.

## Core mechanism

Lines 231–245 keep model weights fixed and evolve module prompts. Genetic-Pareto (GEPA) runs a candidate on a minibatch, reflects over module inputs, outputs, reasoning, score, and textual evaluator feedback, then tests a revised prompt. Lines 380–418 retain prompts that lead on at least one validation instance and sample the Pareto population. Appendix D lines 1252–1258 merges complementary prompt lineages as system-aware crossover.

## Reported evidence

Table 2 at lines 521–538 reports GPT-4.1 Mini across six benchmarks. GEPA+Merge reaches 66.36 aggregate score versus 58.67 for MIPROv2, a comparison prompt optimizer that jointly tunes instructions and demonstrations; plain GEPA reaches 65.22. Appendix E lines 1398–1403 and 1442–1454 says MIPROv2 used 2,270–6,926 rollouts by benchmark and GEPA was capped to each matched budget, with usage differing by at most 10.15%. The totals include validation selection. Harp did not reproduce these author results.

## Key limitation

The method depends on useful execution traces, feedback functions, and validation selection. Lines 722–737 show merge can hurt Qwen3-8B under fixed hyperparameters. Results optimize benchmark-specific prompts in fixed systems and models.

## Why Weng cites it

Weng lines 259–264 uses GEPA to connect reflection-based prompting with evolutionary search over trial-and-error trajectories.
