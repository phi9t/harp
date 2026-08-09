---
source_id: HYPERAGENTS
title: Hyperagents
weng_locator: reference-24
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2603.19461
captured_path: evidence/weng/text/hyperagents.txt
publication_state: "arXiv:2603.19461v1 preprint; peer-reviewed venue not verified"
evidence_state: card-complete
edited_object_family: harness-code-search
claim_ceiling: Sections 3–7 and Appendices C–D editable task-and-meta-agent program, archive search, bounded held-out paper-review agent-quality result, fixed outer-loop and evaluator limits, and safety discussion; no independent reproduction
lesson_ids: 0007,0010
card_path: knowledge/rsi/weng-sources/hyperagents.md
canonical_route: knowledge/rsi/weng/07-evolutionary-search.md
---

# Making the Improvement Procedure Editable

## Problem

Captured lines 22–37 and 64–76 argue that Darwin Gödel Machine, or DGM, uses fixed instruction generation and relies on coding skill aligning with self-modification skill, an assumption unlikely to hold across domains.

## Core mechanism

Lines 252–300 combine a task agent and meta agent in one editable program. DGM-Hyperagents branches selected parents, lets them rewrite task and meta logic, evaluates children, and retains them in an archive. The foundation model stays frozen at lines 305–316.

## Reported evidence

This endpoint measures agent quality. Across five repeated 100-iteration runs, lines 398–403 and 432–463 report median held-out paper-review accuracy rising from 0 to 0.710, with a 95% bootstrap interval of 0.590–0.750; the open static reviewer scores 0.630. Validation selects agents before evaluation on 100 test papers, as lines 1595–1603 and 1681–1688 specify. Harp did not reproduce these results.

## Key limitation

Evidence covers four domains with expensive foundation-model calls and empirical task proxies. Lines 704–743 bound safety to sandboxing, resource and time limits, restricted internet, predefined evaluations, and human oversight; the authors warn safeguards may strain as capability grows. The task distribution, parent selection, and evaluation protocol remain fixed. This does not prove progress on every computable task or safe acceleration.

## Why Weng cites it

Weng lines 285–293 says this DGM follow-up introduces a meta-agent that controls how existing task agents are modified to create new ones. The primary paper, not Weng’s prose, makes the combined program editable and calls modification of its improvement procedure metacognitive self-modification.
