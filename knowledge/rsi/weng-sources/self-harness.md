---
source_id: SELF-HARNESS
title: "Self-Harness: Harnesses That Improve Themselves"
weng_locator: reference-17
section_id: self-improving-harnesses
primary_url: https://arxiv.org/abs/2606.09498
captured_path: evidence/weng/text/self-harness.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: harness-code-search
claim_ceiling: Inspected §§3–4 fixed-model protocol, 64-case held-in/held-out setup, promotion rule, author-reported results, and stated limits; no independent reproduction
lesson_ids: 0006,0010
card_path: knowledge/rsi/weng-sources/self-harness.md
canonical_route: knowledge/rsi/systems/self-harness.md
---

# Turning Trace Weaknesses into Harness Edits

## Problem

Captured lines 33–67 argue that harnesses are model-specific and expensive to tune manually. Self-Harness asks whether the same fixed model can improve its surrounding execution protocol.

## Core mechanism

Weakness Mining clusters verifier-grounded trace failures. The same model proposes diverse, minimal harness edits. Regression tests on held-in and held-out splits accept or reject each edit, then merge accepted candidates. Model weights and evaluator stay fixed.

## Reported evidence

On Terminal-Bench-2.0, the abstract reports held-out pass rates rising from 40.5% to 61.9% for MiniMax M2.5, 23.8% to 38.1% for Qwen3.5-35B-A3B, and 42.9% to 57.1% for GLM-5. Harp did not reproduce these author measurements.

## Key limitation

The source uses a fixed 64-case subset of the 89-task benchmark, partitioned held-in and held-out, with Pass (%) averaging two attempts per candidate. Harp's boundary is that repeated held-out promotion can select to that split. Weng separately warns that permissions and evaluators must remain outside.

## Why Weng cites it

Weng lines 217–238 uses Self-Harness for a propose-evaluate-accept loop over bounded edits, then warns that editable OS surfaces break abstraction boundaries unless permissions and evaluators remain outside.
