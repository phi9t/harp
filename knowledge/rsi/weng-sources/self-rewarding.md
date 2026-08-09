---
source_id: SELF-REWARDING
title: Self-Rewarding Language Models
weng_locator: reference-5
section_id: system-being-improved
primary_url: https://arxiv.org/abs/2401.10020
captured_path: evidence/weng/text/self-rewarding.txt
publication_state: arXiv:2401.10020v3 preprint; peer-reviewed venue not verified
evidence_state: card-complete
edited_object_family: model-self-play-or-weight-adaptation
claim_ceiling: Sections 2–3 and 6 self-instruction and self-judging iterative direct preference optimization mechanism, author-reported instruction and reward-model results, and stated limitations; no independent reproduction
lesson_ids: 0001,0010
card_path: knowledge/rsi/weng-sources/self-rewarding.md
canonical_route: knowledge/rsi/weng/01-system-being-improved.md
---

# A Model That Supplies Its Next Preferences

## Problem

Captured lines 12–50 argue that human preferences and a frozen reward model limit alignment. The paper asks whether one instruction-following model can also improve the reward signal for its next iteration.

## Core mechanism

Captured lines 129–203 have the model generate responses, judge them, form preference pairs, and train the next model with direct preference optimization. Initialization uses 3,200 human-authored Open Assistant first-turn examples. For evaluation fine-tuning, the supervised fine-tuning baseline generates justifications and scores retained when rankings agree with human rankings, at lines 204–226. A fixed Llama 2 Chat 70B generates prompts; the trained model generates and judges responses, at 264–274.

## Reported evidence

For Llama 2 70B, lines 227–251, 299–304, and 369–390 report M1 versus M3 GPT-4-judged AlpacaEval 2.0 win rate against GPT-4 Turbo at 9.94 versus 20.44 percent over 805 prompts. Lines 455–464 report M1 versus M3 MT-Bench overall score at 6.78 versus 7.25 of 10. On held-out Open Assistant evaluation pairs, lines 496–510 report pairwise accuracy against human rankings at 78.7 percent for M1 versus 81.7 percent for M3. Harp did not reproduce these author results.

## Key limitation

Captured lines 632–655 limit evidence to three iterations and one 70B setting, with length growth, possible reward hacking, incomplete safety evaluation, and model judges used for rewards and evaluation. Lines 483–493 add a small author-run human check.

## Why Weng cites it

Weng’s introduction at evidence/weng/text/weng-harness.txt lines 24–30 places synthetic feedback and weight improvement beside recursive self-improvement but outside the harness focus. This is not a harness example.
