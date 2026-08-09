---
source_id: ACE
title: "Agentic Context Engineering: Evolving Contexts for Self-Improving Language Models"
weng_locator: reference-7
section_id: context-engineering
primary_url: https://iclr.cc/virtual/2026/poster/10008343
captured_path: evidence/weng/text/ace.txt
publication_state: ICLR 2026 official poster
evidence_state: card-complete
edited_object_family: context-artifact
claim_ceiling: Official-paper mechanism, reported results, cost analysis, and limitations; no independent reproduction
lesson_ids: 0004,0010
card_path: content/weng-sources/ace.md
canonical_route: content/systems/ace.md
---

# Evolving a Structured Context Playbook

## Problem

Captured lines 104–115 describe brevity bias and context collapse: repeated prompt rewriting can remove rare but useful details. ACE seeks post-training adaptation without changing model weights.

## Core mechanism

ACE stores identified playbook bullets rather than one prompt blob. A Generator produces trajectories, a Reflector extracts lessons from successes and failures, and a Curator incrementally adds, merges, refines, or removes items. The edited object is persistent context, not the model or update procedure.

## Reported evidence

The authors' 10.6% AppWorld aggregate is the arithmetic mean of three stated comparator summaries: 12.3% over ICL, 11.9% over GEPA, and 7.6% over Dynamic Cheatsheet. The displayed split metrics do not transparently derive the 12.3% or 11.9% summaries. Harp did not reproduce these results.

## Key limitation

Lines 402–425 say unreliable feedback can pollute context and that useful adaptation depends on a capable Reflector. Rich playbooks can also hide stale rules, leakage, duplicate advice, or unmatched token budgets.

## Why Weng cites it

Weng lines 107–117 uses ACE to show structured context as an optimization target, explain how itemized updates resist full-prompt collapse, and contrast its handcrafted workflow with MCE.
