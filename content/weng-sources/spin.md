---
source_id: SPIN
title: Self-Play Fine-Tuning Converts Weak Language Models to Strong Language Models
weng_locator: reference-6
section_id: system-being-improved
primary_url: https://arxiv.org/abs/2401.01335
captured_path: evidence/weng/text/spin.txt
publication_state: ICML 2024 reported by anchor; official status not verified
evidence_state: card-complete
edited_object_family: model-self-play-or-weight-adaptation
claim_ceiling: Sections 3–7 and Appendix B iterative self-play fine-tuning mechanism, author-reported benchmark results, fixed-human-distribution ceiling, and compute scope; no independent reproduction
lesson_ids: 0001,0010
card_path: content/weng-sources/spin.md
canonical_route: content/weng/01-system-being-improved.md
---

# Learning Against the Previous Checkpoint

## Problem

Self-Play Fine-Tuning asks whether a supervised fine-tuning (SFT) model can improve without new human annotations or stronger-model feedback. Captured lines 8–44 and 76–112 target gains after continued SFT plateaus.

## Core mechanism

Captured lines 351–604 pair each human SFT response as preferred with the previous checkpoint's response as rejected. Preference-style training makes the current checkpoint distinguish them; it then generates opponents for the next iteration. Weights change while human SFT remains the positive distribution.

## Reported evidence

For Mistral-7B-derived zephyr-7b-sft-full with prompts sampled from Ultrachat200k, lines 728–746 say iteration 0 uses 50,000 synthetic examples. Iterations 1–3 each use 100,000 by combining the preceding 50,000 with 50,000 new examples. Lines 728–818 and 1523–1530 report the zephyr-7b-sft-full base at 58.14 versus SPIN iteration 3 at 63.16 on the six-task Open LLM Leaderboard average. Lines 1545–1553 report base MT-Bench at 5.94 versus SPIN iteration 2 at 6.78; leaderboard gains taper from 2.66 to 0.19 points. Harp did not reproduce these author results.

## Key limitation

Captured lines 904–916 state that a fixed human target distribution imposes a ceiling. Evidence covers one 7B family and SFT corpus; lines 792–818 show diminishing gains, and 1484–1496 report generation plus repeated training on eight A100 80GB GPUs.

## Why Weng cites it

Weng’s introduction at evidence/weng/text/weng-harness.txt lines 24–30 places model self-play and weight improvement beside recursive self-improvement but outside the harness focus. This is not a harness example.
