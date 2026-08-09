---
source_id: AFLOW
title: "AFlow: Automating Agentic Workflow Generation"
weng_locator: reference-15
section_id: workflow-design-and-search
primary_url: https://openreview.net/forum?id=z5uVAKwmjf
captured_path: evidence/weng/text/aflow.txt
publication_state: ICLR 2025 official-paper PDF identified
evidence_state: card-complete
edited_object_family: workflow-or-agent-program-search
claim_ceiling: Official-paper MCTS workflow-search mechanism and author-reported results; no independent reproduction or RSI efficacy claim
lesson_ids: 0005,0010
card_path: content/weng-sources/aflow.md
canonical_route: content/systems/aflow.md
---

# Searching Code-Represented Workflows

## Problem

Captured lines 176–187 of Weng and the paper's method ask how to automate search over prompts, control flow, and reusable operators rather than hand-design one workflow.

## Core mechanism

AFlow represents LLM actions as nodes and executable logic as edges. Monte Carlo tree search selects a complete workflow, an optimizer model proposes code changes from scores and tree history, an executor runs them, and the tree retains results. Model identities and evaluation stay external.

## Reported evidence

Final Table 1 averages three GPT-4o-mini test executions and reports a heterogeneous arithmetic mean of 80.3 versus ADAS at 67.2; it is not a common success probability. During search, each candidate is executed five times on a filtered validation subset for up to 20 rounds. Harp did not reproduce the results.

## Key limitation

The main text calls the 80% partition the test set, while Appendix Algorithm 1 calls it training. The optimizer and executor can differ, and the source reports test-execution cost rather than a matched total search cost. Harp's boundary is that 80.3 does not prove RSI efficacy.

## Why Weng cites it

Weng lines 176–189 uses AFlow to show workflow optimization as MCTS over executable candidates, then compares its reported QA, code, and math results with manual workflows and ADAS.
