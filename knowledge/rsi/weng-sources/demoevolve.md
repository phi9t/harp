---
source_id: DEMOEVOLVE
title: "DemoEvolve: Overcoming Sparse Feedback in Agentic Harness Evolution with Demonstrations"
weng_locator: reference-39
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2605.24539
captured_path: evidence/weng/text/demoevolve.txt
publication_state: "arXiv:2605.24539v1 preprint; peer-reviewed venue not verified"
evidence_state: card-complete
edited_object_family: harness-code-search
claim_ceiling: Sections 3–4, 6, and Appendix C demonstration-guided frozen-model harness evolution, bounded fixed-seed Balatro result, causal-audit warning, and task-coverage limits; no independent reproduction
lesson_ids: 0007,0010
card_path: knowledge/rsi/weng-sources/demoevolve.md
canonical_route: knowledge/rsi/weng/07-evolutionary-search.md
---

# Giving Harness Search Diagnostic Examples

## Problem

Captured lines 88–102 ask how a coding proposer can assign credit to a harness edit when long episodes end in sparse, noisy rewards and trajectory variance can favor inactive code.

## Core mechanism

Lines 148–223 keep the task model frozen while a coding proposer evolves executable harnesses from a filesystem archive. Lines 224–287 compare archive-only self-rollouts, added external text, and competent human trajectories in the same observation-action format. Demonstrations help diagnose missing state and action mechanisms; they do not fine-tune the task model.

## Reported evidence

This endpoint measures demonstration-guided sparse-feedback improvement. Table 1 at lines 365–410 reports DemoEvolve completing 12 of 15 fixed-seed Balatro rollouts versus 6 of 15 for archive-only Meta-Harness, with capped mean final round 22.00 versus 18.13. Appendix C lines 924–959 uses three development and two held-out seeds, three final rollouts per seed, a frozen GPT-5.4-low task model, the same proposer and selection budget, and nine development-seed human trajectories. Harp did not reproduce these results.

## Key limitation

Lines 534–548 call TextArena a lightweight positive control and evaluate demonstrations mainly on one game. Lines 938–947 describe the fixed-seed low-sample protocol as an engineering comparison, not a variance-free estimate of average Balatro skill. The selected Meta-Harness hook appeared in 0 of 517 inspected model requests, so it could not explain the apparent gain; the source attributes selection to rollout noise.

## Why Weng cites it

Weng lines 279–283 cites DemoEvolve for adding human demonstrations to the self-rollout archive as reference experience for sparse-feedback harness diagnosis and editing.
