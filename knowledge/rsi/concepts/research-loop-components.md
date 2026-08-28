---
id: rsi-research-loop-components
kind: concept
title: Research-loop components
summary: Hypothesis generation, experiment design, negative evidence, and research memory.
primary_parent: rsi-automated-research
additional_parents: []
related: []
attachments:
  - content/chapters/automated-research.md
claims: []
human_review: null
---

# Research-loop components

The owning [[knowledge/rsi/chapters/automated-research|automated-research chapter]]
defines the result record, validity boundary, and external promotion rule. This
page names the components that record must connect.

Automated research is more than running code. It must choose informative interventions, preserve failed branches, and update beliefs without rewriting the evidence that justifies them.

## Hypothesis generation

Hypothesis generation proposes explanations that make different observable predictions. Useful hypotheses identify a mechanism, competing account, and discriminating result. Generating many plausible sentences without prediction differences does not improve experiment selection.

## Experiment design

Experiment design selects the smallest intervention that separates hypotheses under a recorded metric, control, stopping rule, and budget. It must isolate the variable under study and predeclare invalidation conditions. Otherwise implementation drift and post hoc metric choice make the result uninterpretable.

## Negative results

A negative result records that a valid experiment did not show the predicted effect. The record must distinguish a falsified hypothesis from an invalid run or an underpowered comparison. Keeping negative evidence reduces repeated work and exposes which research strategies consume budget without information.

Use compound status: valid-positive, valid-negative, valid-inconclusive, and
invalid-or-interrupted. Invalid and interrupted work is evidence about the
execution path, not evidence for or against the hypothesis.

## Research memory

Research memory links questions, hypotheses, code and data revisions, raw outputs, analysis, uncertainty, and next experiments. Summaries help retrieval, but immutable result locators remain necessary because later agents must be able to challenge a conclusion or recover conditions omitted by compression.

It must also retain controls, seed policy, environment, integrity status, and
competing explanations. The next research choice should point back to that
record rather than treating a conclusion-only summary as authoritative.

<details>
<summary>Original sources for this mechanism</summary>

- [[knowledge/rsi/chapters/automated-research#End-to-end control flow|Automated research as an RSI component]] defines the twelve-step research loop.
- AI Scientist, Methods and "Limitations," describes its ideation-to-paper workflow and reported failure modes: [Nature 651, 914–919](https://www.nature.com/articles/s41586-026-10265-5).
- PaperBench and RE-Bench define replication and time-budgeted research evaluations: [PMLR 267, Starace et al.](https://proceedings.mlr.press/v267/starace25a.html), [PMLR 267, Wijk et al.](https://proceedings.mlr.press/v267/wijk25a.html).

</details>
