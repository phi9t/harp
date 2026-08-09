---
id: rsi-procedure-representations
kind: concept
title: Procedure representations
summary: Prompts, skills, scripts, workflows, and model behavior as distinct storage and execution mechanisms.
primary_parent: rsi-procedure-internalization
additional_parents: []
related: []
attachments:
  - content/chapters/procedure-internalization.md
claims: []
human_review: null
---

# Procedure representations

The same useful practice can live at several layers. The representation determines who can edit it, whether execution is guaranteed, and how broadly it generalizes.

## Prompt procedure

A prompt procedure describes behavior in model-readable text. It is cheap to edit and inspect but consumes context and depends on instruction following. Its effect can disappear under conflicting context or distribution shift.

## Skill procedure

A skill packages instructions, references, and scripts behind an explicit trigger. It can load domain guidance only when needed and can be versioned independently from the model. Trigger errors and stale references are its main failure modes.

## Script procedure

A script encodes deterministic transformations or checks in executable code. It gives stronger repeatability than prose for fixed steps, but it handles only inputs anticipated by its interface and requires ordinary software maintenance.

## Workflow procedure

A workflow orders activities, records state transitions, and controls retries, waits, and approvals. It is the right representation when the useful practice concerns sequencing and durability rather than one transformation.

## Model behavior procedure

A learned procedure is encoded in weights and invoked by ordinary inference. It saves context and may generalize beyond explicit rules, but it is harder to inspect, patch, and guarantee. The four-way model-with-or-without-procedure test separates weight learning from dependence on an external procedure.

<details>
<summary>Original sources for this mechanism</summary>

- [From external procedures to learned behavior](../chapters/procedure-internalization.md#the-four-way-test) defines the controlled comparison.
- The RLM paper, §§2–4, describes recursive decomposition supplied by an inference harness: [arXiv:2512.24601v3](https://arxiv.org/abs/2512.24601).
- \"Language model harnesses are compositional generalizers,\" sections \"Post-training setup\" and \"Results,\" reports the model-plus-harness comparison: [2026-07-20 capture](https://alexzhang13.github.io/blog/2026/rlm-harness-generalization/).

</details>
