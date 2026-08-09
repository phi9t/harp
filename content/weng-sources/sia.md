---
source_id: SIA
title: "SIA: Self Improving AI with Harness & Weight Updates"
weng_locator: reference-27
section_id: joint-harness-weight-optimization
primary_url: https://arxiv.org/abs/2605.27276
captured_path: evidence/weng/text/sia.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: joint-harness-and-weight-adaptation
claim_ceiling: Inspected §§1–6 joint harness-and-weight mechanism, three-domain author-reported results, ablations, setup, and confounds; no independent reproduction
lesson_ids: 0008,0010
card_path: content/weng-sources/sia.md
canonical_route: content/systems/sia.md
---

# Routing Updates into Harnesses and Weights

## Problem

Captured lines 20–40 argue that harness search and test-time weight training usually improve separate objects. SIA combines both within one task loop.

## Core mechanism

A Meta-Agent creates an initial scaffold. A task-specific gpt-oss-120b agent acts. A Claude Sonnet 4.6 Feedback-Agent reads trajectories and chooses either a harness rewrite or a rank-32 Low-Rank Adaptation weight update. The verifier and controller remain external.

## Reported evidence

Across law, kernel optimization, and single-cell RNA denoising, the abstract reports harness-plus-weight updates beating harness-only variants. One endpoint is denoising mse norm at 0.289 versus prior state of the art at 0.240, reported as 20.4% higher. Harp did not reproduce these author results.

## Key limitation

Weng says model asymmetry and weak baselines make the evidence provisional. The source states that both levers optimize one fixed verifier and calls this coupled Goodhart risk. Harp's boundary is that three domains do not establish general successor development.

## Why Weng cites it

Weng lines 301–309 calls SIA an early joint-update attempt but explicitly labels the evidence provisional because the task agent is weaker than its meta and feedback agents and baselines are hard to compare.
