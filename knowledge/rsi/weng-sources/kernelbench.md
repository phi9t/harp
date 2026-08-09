---
source_id: KERNELBENCH
title: "KernelBench: Can LLMs Write Efficient GPU Kernels?"
weng_locator: reference-35
section_id: future-challenges
primary_url: https://arxiv.org/abs/2502.10517
captured_path: evidence/weng/text/kernelbench.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: evaluation-benchmark
claim_ceiling: Inspected §§3–4 250-task benchmark contract, fast_p metric, one-shot table, feedback experiments, and hardware and configuration limits; no independent reproduction
lesson_ids: 0009,0010
card_path: knowledge/rsi/weng-sources/kernelbench.md
canonical_route: knowledge/rsi/weng/09-future-challenges.md
---

# Evaluating Correct and Faster GPU Kernels

## Problem

Captured lines 34–55 ask whether language models can reproduce a kernel engineer's compiler, profiler, hardware, and optimization workflow.

## Core mechanism

KernelBench gives reference PyTorch code for 250 tasks and asks a model for an optimized replacement. Five random inputs test correctness; repeated timing measures speed. fast_p is the fraction of all tasks whose kernel is functionally correct and whose runtime speedup is strictly greater than p over the baseline.

## Reported evidence

In the one-shot NVIDIA L40S baseline, DeepSeek R1 records fast_1 of 12%, 36%, and 2% on Levels 1, 2, and 3 against PyTorch Eager. OpenAI o1 records 10%, 24%, and 12%. Higher is better. Harp did not reproduce these author results.

## Key limitation

Correctness, speed, and rankings depend on shapes, precision, GPU, attempts, feedback, and baseline configuration. Hardware transfer varies. Kernel generation is a narrow automatically verifiable engineering task, not general research ability.

## Why Weng cites it

Weng lines 395–397 only lists KernelBench as correctness-and-speed evaluation. Calling kernel generation an AI-for-AI engineering sub-loop is Harp's interpretation, not Weng's stated rationale.
