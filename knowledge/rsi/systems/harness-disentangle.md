---
id: rsi-system-harness-disentangle
kind: concept
title: Harness updating is not harness benefit
summary: A decomposition of update quality, harness activation, sustained adherence, and downstream benefit across models and benchmarks.
primary_parent: rsi-procedure-internalization
additional_parents:
  - rsi-evaluation-promotion-containment
related: []
attachments:
  - content/claim_evidence_ledger.md
claims: []
human_review: null
---

# Harness updating is not harness benefit

## Capability decomposition

**EVIDENCE — [HARNESS-DISENTANGLE], §§3.1–3.3.** The paper separates:

- harness-updating: whether an evolver produces useful persistent updates;
- harness-benefit: whether a task agent benefits from an available update;
- base task capability: ordinary performance without the update.

This prevents an end-to-end score from hiding which boundary failed.

## Activation and adherence

A useful skill or memory may never be loaded. If loaded, the agent may stop
following it before final validation. The paper diagnoses both activation
failure and declining adherence across trajectories.

<details>
<summary>Original sources for this mechanism</summary>

- Formal definitions: [Harness Disentangle, §§3.1–3.3](https://arxiv.org/abs/2605.30621).
- Seven-model, three-benchmark results: [Harness Disentangle, §4](https://arxiv.org/abs/2605.30621).
- SkillsBench activation and adherence: [Harness Disentangle, Tables 2–3 and Figure 7](https://arxiv.org/abs/2605.30621).
- Limits and deployment implications: [Harness Disentangle, §§6–7](https://arxiv.org/abs/2605.30621).
- Checked-in text: `evidence/weng/text/harness-disentangle.txt`.

</details>

## Reported findings and limits

The preprint reports relatively flat update quality across tested capability
tiers, non-monotonic benefit, and large differences in skill loading and final
adherence. These measurements rely partly on an LLM-judge pipeline and selected
models, harnesses, and benchmarks.

## Claim ceiling

The system is an evaluation framework, not itself a successor-producing
improver. Its main contribution to RSI is a measurement contract: report update
quality, activation, adherence, and downstream outcome separately.

## Reading routes

- [[knowledge/rsi/chapters/procedure-internalization|From external procedures to learned behavior]]
- [[knowledge/rsi/chapters/evaluation-promotion-containment|Evaluation and promotion]]
- [Original paper](https://arxiv.org/abs/2605.30621)
