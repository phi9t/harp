---
source_id: WENG-REWARD
title: Reward Hacking in Reinforcement Learning
weng_locator: body-link-reward-hacking
section_id: self-improving-harnesses
primary_url: https://lilianweng.github.io/posts/2024-11-28-reward-hacking/
captured_path: evidence/weng/text/weng-reward.txt
publication_state: Lil'Log 2024-11-28
evidence_state: card-complete
edited_object_family: safety-or-reward-hacking-framing
claim_ceiling: Secondary reward-hacking and evaluator-bias synthesis
lesson_ids: 0006,0010
card_path: content/weng-sources/weng-reward.md
canonical_route: content/weng/06-self-improving-harnesses.md
---

# Treating the Evaluator as a Proxy

## Problem

Weng's secondary synthesis asks why optimizing a measurable reward can produce behavior that satisfies the proxy while violating the intended goal.

## Core mechanism

The article organizes reward misspecification, proxy exploitation, evaluator bias, and reward tampering. In harness evolution, tests, judge models, and benchmark scores are imperfect oracles. Selection pressure can target their gaps rather than the desired capability.

## Reported evidence

This card reports a secondary taxonomy, not one primary experiment. The article synthesizes examples from other studies, but Harp did not independently inspect every cited study for this card. It therefore supports safety framing and audit questions, not a pooled empirical result.

## Key limitation

Goodhart-style labels do not identify which exploit will occur or quantify risk for a new system. A judge can be useful while still biased. Held-out tests, trace audits, protected evaluators, and human review reduce exposure but do not make the proxy identical to the goal.

## Why Weng cites it

Weng lines 338–340 directly links reward hacking to self-improving harnesses and argues that evaluators and permission controls should sit outside the editable loop.
