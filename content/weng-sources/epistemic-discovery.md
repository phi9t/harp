---
source_id: EPISTEMIC-DISCOVERY
title: Epistemic Uncertainty for Test-Time Discovery
weng_locator: reference-26
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2605.11328
captured_path: evidence/weng/text/epistemic-discovery.txt
publication_state: "arXiv:2605.11328v1 preprint; peer-reviewed venue not verified"
evidence_state: card-complete
edited_object_family: model-self-play-or-weight-adaptation
claim_ceiling: Sections 2–3 and Limitations uncertainty-guided LoRA-ensemble test-time training, bounded CP26 maximum-reward and family-entropy result, single-seed verifier and ensemble-cost limits; no independent reproduction
lesson_ids: 0007,0010
card_path: content/weng-sources/epistemic-discovery.md
canonical_route: content/weng/07-evolutionary-search.md
---

# Preserving Disagreement During Test-Time Search

## Problem

Captured lines 112–141 argue that single-policy reward training favors familiar low-variance mutations. Solution families collapse, and one network's entropy cannot separate missing knowledge from intrinsic ambiguity.

## Core mechanism

UG-TTT retains TTT-Discover's PUCT selection from a history buffer and verifier-scored executable-code rollouts. Lines 193–247 place five trainable rank-16 Low-Rank Adaptation, or LoRA, adapters over frozen Qwen3-8B base weights. Lines 248–350 update adapter weights with a mutual-information exploration bonus and use nuclear-norm maximization to preserve distinct adapter subspaces.

## Reported evidence

This endpoint measures uncertainty alongside discovery reward. On CP26 at the matched 384-versus-384 rollout budget, Table 1 at lines 503–535 reports maximum reward, or Rmax, of 2.6302 for the single-adapter baseline and 2.6359 for UG-TTT; higher is better. The latter separately matches the published TTT-Discover ceiling reached with 25,600 rollouts. Final family entropy is 0.70 versus 1.23 bits. Higher entropy means a broader distribution over correct-rollout families assigned by an ordered hand-written regular-expression list, not an objective family count. Harp did not reproduce these results.

## Key limitation

Lines 725–733 cover one seed, four verifier-based problems, one model, and roughly fivefold adapter overhead. Family labels use hand-written regular expressions; streaming and uncertainty settings needed problem-specific calibration.

## Why Weng cites it

Weng lists this paper as reference 26 but does not explicitly connect it to the diversity-collapse paragraph at lines 328–338. Harp’s source-derived interpretation is that UG-TTT tests epistemic uncertainty as a mitigation for test-time discovery collapse.
