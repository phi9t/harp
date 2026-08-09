---
source_id: LEARNING-DISCOVER
title: Learning to Discover at Test Time
weng_locator: reference-25
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2601.16175
captured_path: evidence/weng/text/learning-discover.txt
publication_state: "arXiv:2601.16175v2 preprint; peer-reviewed venue not verified"
evidence_state: card-complete
edited_object_family: model-self-play-or-weight-adaptation
claim_ceiling: Sections 2–4 and 6 test-time weight training with archive reuse, bounded GPUMode TriMul discovery result, continuous-verifier and problem-specific compute limits; no independent reproduction
lesson_ids: 0007,0010
card_path: knowledge/rsi/weng-sources/learning-discover.md
canonical_route: knowledge/rsi/weng/07-evolutionary-search.md
---

# Training a Model on One Open Problem

## Problem

Captured lines 65–91 argue that frozen inference search can reuse prior attempts in prompts but cannot internalize experience specific to one open problem.

## Core mechanism

Algorithm 1 at lines 175–207 samples a prior program or solution, generates and verifies a child, appends it to an archive, and updates model weights at test time. Lines 231–283 use an adaptive entropic objective to favor exceptional rewards and Predictor plus Upper Confidence Bound applied to Trees, or PUCT, to balance promising and underexplored archive states.

## Reported evidence

This endpoint measures problem discovery. Table 4 at lines 490–548 reports one gpt-oss-120b model trained for 50 steps with 512 rollouts per step. Its submitted TriMul H100 kernel runs in 1161.2 microseconds versus 1371.1 for the top human entry and 5390.3 for best-of-25,600. Lines 1550–1555 say the authors selected the 20 best training-verifier kernels, retimed each three times on target hardware, and submitted the smallest-average candidate. The official leaderboard owns the reported H100 runtime. Harp did not reproduce it.

## Key limitation

The method trains separately for one problem and seeks one maximum, not transferable policy quality. Lines 877–880 require continuous verifiable rewards; lines 278–291 use substantial per-problem sampling. A better verifier score does not guarantee genuine novelty.

## Why Weng cites it

Weng lists this paper as reference 25 but gives no explicit prose rationale. Harp’s source-derived interpretation is that it joins archive-based problem-state selection with problem-specific test-time weight updates rather than frozen-model mutation alone.
