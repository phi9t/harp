---
source_id: ABSOLUTE-ZERO
title: "Absolute Zero: Reinforced Self-play Reasoning with Zero Data"
weng_locator: reference-4
section_id: system-being-improved
primary_url: https://arxiv.org/abs/2505.03335
captured_path: evidence/weng/text/absolute-zero.txt
publication_state: arXiv:2505.03335v3 preprint; peer-reviewed venue not verified
evidence_state: card-complete
edited_object_family: model-self-play-or-weight-adaptation
claim_ceiling: Sections 2–4 and 6 proposer–solver code-reasoning weight-training mechanism, author-reported benchmark results, verifier constraints, and safety caveats; no independent reproduction
lesson_ids: 0001,0010
card_path: content/weng-sources/absolute-zero.md
canonical_route: content/weng/01-system-being-improved.md
---

# Learning What Problems to Practice

## Problem

Captured lines 7–23 and 39–67 ask how verifiable-reward training can escape human-curated question and answer sets. The goal is a model that generates learnable tasks without external examples while retaining grounded verification.

## Core mechanism

The Absolute Zero Reasoner uses one model as proposer and solver. Captured lines 124–266 show it proposing deduction, abduction, and induction code tasks; Python validates tasks and answers; reinforcement learning updates shared weights from learnability and correctness rewards. Lines 528–586 restrict tasks to filtered, approximately deterministic programs.

## Reported evidence

Under greedy decoding, lines 614–709 report the Qwen2.5-7B-Coder base checkpoint at 40.2 versus the Absolute Zero Reasoner-trained checkpoint at 50.4, +10.2 points overall. The metric equally averages code scores on HumanEval+, MBPP+, and LiveCodeBench v1–5 with math accuracy on AIME 2024/2025, AMC 2023, MATH500, Minerva, and OlympiadBench; lines 728–754 bound model scale. Harp did not reproduce these author results.

## Key limitation

Captured lines 528–567 require runnable deterministic Python and a package filter, not open-world verification. Lines 1025–1028 say safe management is unresolved; lines 829–834 report a concerning Llama-3.1-8B output.

## Why Weng cites it

Weng’s introduction at evidence/weng/text/weng-harness.txt lines 24–30 treats self-play and weight updates as adjacent to recursive self-improvement but outside the harness focus. Absolute Zero is not harness-evolution evidence.
