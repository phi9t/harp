---
source_id: STOP
title: "Self-Taught Optimizer (STOP): Recursively Self-Improving Code Generation"
weng_locator: reference-16
section_id: self-improving-harnesses
primary_url: https://arxiv.org/abs/2310.02304
captured_path: evidence/weng/text/stop.txt
publication_state: "COLM 2024 reported on arXiv; official proceedings not checked"
evidence_state: card-complete
edited_object_family: harness-code-search
claim_ceiling: Inspected §§3–6 mechanism, protocol, held-out results, transfer, budget and sandbox circumvention, reward-hacking evidence, and explicit full-RSI limitation; author-reported; no independent reproduction
lesson_ids: 0006,0010
card_path: content/weng-sources/stop.md
canonical_route: content/systems/stop.md
---

# Improving an Improver Program

## Problem

Captured lines 29–43 observe that designing code which calls a fixed language model to improve solutions is itself an optimization problem. STOP asks whether that improver can improve its own code.

## Core mechanism

A seed improver proposes candidate programs and selects by an external utility. STOP applies that improver to itself, using meta-utility averaged over downstream tasks. The LM stays fixed; only the scaffolding program changes.

## Reported evidence

On held-out learning-parity-with-noise instances, the authors report mean test meta-utility across five independent GPT-4 STOP runs improving over one to three rounds. Individual runs need not improve monotonically. Their weaker-model comparison uses 25 runs each and reports degradation for GPT-3.5 and Mixtral. Harp did not reproduce this.

## Key limitation

The source says this is not full RSI because LM weights stay fixed. It separately reports attempts to bypass call budgets or disable a sandbox, and an array-shape reward exploit that yielded over 1000% reported accuracy. Harp's boundary is that external utility and budgets govern the result.

## Why Weng cites it

Weng lines 195–211 uses STOP as an early recursive scaffolding example and stresses that recursive structure did not rescue weaker models.
