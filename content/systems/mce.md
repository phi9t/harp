---
id: rsi-system-mce
kind: concept
title: Meta Context Engineering
summary: Bi-level evolution of context-engineering skills and context artifacts through agentic crossover, execution histories, and task evaluation.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-harness-engineering
related:
  - kind: compared-with
    target: rsi-system-ace
attachments:
  - content/context_engineering_deep_dive.md
  - content/diagnostics/cases/mce.json
claims: []
human_review: null
---

# MCE: learning the context-learning procedure

## Problem

ACE and related context methods still use human-designed update workflows and
schemas. MCE asks whether the context-engineering skill itself can be evolved.

## Bi-level mechanism

**EVIDENCE — [MCE], §3.** An outer meta-agent reads prior skills, executions,
and evaluations, then performs agentic crossover to propose a new
context-engineering skill. An inner base agent executes that skill to construct
and update context artifacts from task rollouts.

The editable object therefore includes executable instructions and code for
learning context, not only the final context text.

<details>
<summary>Original sources for this mechanism</summary>

- Skill representation and bi-level objective: [MCE, §§2–3.1](https://arxiv.org/abs/2601.21557).
- Agentic crossover and base-level optimization: [MCE, §§3.2–3.4](https://arxiv.org/abs/2601.21557).
- Evaluation and limitations: [MCE, §4](https://arxiv.org/abs/2601.21557).
- Checked-in text: `evidence/weng/text/mce.txt`.

</details>

## Evaluation and limits

The paper reports results across five domains, comparisons with no-skill and
fixed-skill baselines, and average improvement over compared agentic context
methods. The source is a preprint and this packet contains no independent
reproduction.

Skill histories can overfit finite tasks, accumulate opaque code, or improve
only because they spend more calls and context. A matched comparison must count
outer and inner work.

## Claim ceiling

MCE changes `H_t`, `R_t`, and the artifacts they produce, so the Atlas
classifies it as `harness-improvement`. The source does not measure whether an
accepted skill becomes better at producing later accepted skills under a
matched envelope.

## Reading routes

- [Context engineering deep dive](../context_engineering_deep_dive.md)
- [ACE versus MCE lesson](../lessons/02-ace-vs-mce.md)
- [Original paper](https://arxiv.org/abs/2601.21557)
