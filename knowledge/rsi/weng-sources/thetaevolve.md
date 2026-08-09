---
source_id: THETAEVOLVE
title: "ThetaEvolve: Test-time Learning on Open Problems"
weng_locator: reference-22
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2511.23473
captured_path: evidence/weng/text/thetaevolve.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: evolutionary-program-or-population-search
claim_ceiling: Sections 2–4, 6, and Appendix A/E AlphaEvolve-style dynamic program environment, optional reward-shaped weight training, author-reported matched results, and evaluator caveats; no independent reproduction
lesson_ids: 0007,0010
card_path: knowledge/rsi/weng-sources/thetaevolve.md
canonical_route: knowledge/rsi/weng/07-evolutionary-search.md
---

# Training Inside a Growing Program Database

## Problem

Captured lines 18–50 and 201–220 ask whether an AlphaEvolve-style search can teach a smaller model to improve programs, rather than leaving all learning in an inference-only archive.

## Core mechanism

Lines 279–369 describe a dynamic, verifiable environment: sample parent programs from a large database, ask one model for batches of code diffs, evaluate children, and return them to the database. Lines 370–441 add invalid-output penalties and optional task-shaped rewards. Unlike frozen evolutionary search, reinforcement learning can update model weights while the database adapts.

## Reported evidence

Table 9 at lines 2127–2139 reports three-seed ProRL-1.5B-v2 CirclePacking-T results. The task maximizes summed radii of 26 circles in a unit square under OpenEvolve’s 1e-6 overlap and boundary tolerance, so higher is better. After 200 steps, with 512 new programs per step, the RL arm averages 2.3498 with best 2.5225; matched inference-only search averages 2.0265 with best 2.1343. Even 600 inference-only steps average 2.0991 with best 2.2491. Harp did not reproduce these author results.

## Key limitation

Reward shaping requires task-specific verifier bounds, and lines 1081–1097 show settings do not transfer cleanly across models. Figure 5 at lines 989–1024 ties gains to a 10,000-program database and large test compute. Appendix A lines 1535–1567 warns that one autocorrelation evaluator measures a different formula from prior reported bounds.

## Why Weng cites it

Weng lines 279–283 cites ThetaEvolve as evolutionary search combined with in-context learning and weight-updating reinforcement learning.
