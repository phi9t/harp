---
id: rsi-system-ace
kind: concept
title: Agentic Context Engineering
summary: Structured evolving context playbooks, Generator Reflector and Curator roles, grow-and-refine updates, cost evidence, and context-artifact claim limits.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-harness-engineering
related:
  - kind: compared-with
    target: rsi-system-mce
attachments:
  - content/context_engineering_deep_dive.md
  - content/diagnostics/cases/ace.json
claims: []
human_review: null
---

# ACE: evolving structured context artifacts

## Problem

**EVIDENCE — [ACE], §§2.2–3.** Existing context adaptation can suffer brevity
bias and context collapse: repeated monolithic rewriting shortens a useful
context and erases rare details.

## Mechanism

ACE treats context as an itemized playbook. A Generator proposes candidate
lessons, a Reflector extracts lessons from trajectories and validation signals,
and a Curator inserts, merges, refines, or removes identified bullets.

Stable item identity enables incremental `grow-and-refine` updates instead of
rewriting the whole context. It also makes provenance and conflict tracking
possible, though those guarantees still require implementation discipline.

<details>
<summary>Original sources for this mechanism</summary>

- Failure model and roles: [ACE, §§2–3](https://iclr.cc/virtual/2026/poster/10008343).
- Evaluation, cost, and ablations: [ACE, §4 and Appendix A](https://iclr.cc/virtual/2026/poster/10008343).
- Checked-in text: `evidence/weng/text/ace.txt`.

</details>

## Evaluation and limits

**EVIDENCE — [ACE], §4.** The paper reports gains on agent and domain-specific
benchmarks, offline prompt optimization, online memory adaptation, and lower
rollout cost than compared methods. It also reports component ablations.

These are author-reported official-paper results, not an independent
reproduction. More context can hide stale rules, duplicate advice, leakage, or
unmatched token budgets.

## Claim ceiling

ACE primarily changes persistent artifacts `D_t`, so the Atlas classifies it
as `persistent-adaptation`. It does not show that the procedure learning those
artifacts becomes a better producer of later learning procedures.

## Reading routes

- [[knowledge/rsi/context_engineering_deep_dive|Context engineering deep dive]]
- [[knowledge/rsi/lessons/02-ace-vs-mce|ACE versus MCE lesson]]
- [Original paper](https://iclr.cc/virtual/2026/poster/10008343)
