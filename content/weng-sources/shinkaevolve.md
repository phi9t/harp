---
source_id: SHINKAEVOLVE
title: "ShinkaEvolve: Towards Open-Ended And Sample-Efficient Program Evolution"
weng_locator: reference-21
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2509.19349
captured_path: evidence/weng/text/shinkaevolve.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: evolutionary-program-or-population-search
claim_ceiling: Sections 3–6 and Appendix B LLM-guided island program evolution, bounded AIME 2024 scaffold-search result, 2023/2025 transfer and contamination caveat, ablations, and stated limitations; no independent reproduction
lesson_ids: 0007,0010
card_path: content/weng-sources/shinkaevolve.md
canonical_route: content/weng/07-evolutionary-search.md
---

# Spending Program Evaluations on Novel Branches

## Problem

Captured lines 34–55 argue that language-model program evolution wastes evaluations through weak parent selection, repeated code proposals, and static model choice.

## Core mechanism

Sections 3.1–3.3 at lines 90–238 maintain island archives and sample parents by fitness and offspring count. Language models propose diffs, full rewrites, or crossovers. Embedding similarity plus a novelty judge rejects near-duplicates. World feedback updates a bandit-selected language-model ensemble, while a meta-scratchpad turns successful programs into future mutation guidance.

## Reported evidence

Figure 6 and Appendix B at lines 430–516 and 1245–1264 report a 75-generation scaffold search for a GPT-4.1-nano solver on all 30 American Invitational Mathematics Examination 2024 problems. Each candidate received three full-set runs and at most ten model calls per problem. The evolved scaffold scores 34.4% accuracy versus 32.2% for Majority@5 and 24.4% for the base agent. Harp did not reproduce these author results.

## Key limitation

Section 6 at lines 820–839 says configurations and exploration controls are fixed, task objectives need human design, and the method requires numerical evaluators. The AIME score is on the search task; lines 444–449 flag possible contamination and test transfer separately. This does not establish open-ended discovery.

## Why Weng cites it

Weng lines 279–283 highlights parent sampling, novelty rejection, and the meta-scratchpad as sample-efficiency improvements to solution-program evolution.
