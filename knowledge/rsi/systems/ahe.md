---
id: rsi-system-ahe
kind: concept
title: Agentic Harness Engineering
summary: File-level component observability, trace-grounded evolution, prediction contracts, transfer evidence, component ablations, and coding-harness claim limits.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-evaluation-promotion-containment
related:
  - kind: compared-with
    target: rsi-system-self-harness
attachments:
  - content/diagnostics/cases/ahe.json
claims: []
human_review: null
---

# AHE: observability-driven harness evolution

## Mechanism

**EVIDENCE — [AHE], §§2–4.** Agentic Harness Engineering exposes editable
harness components as files, records execution traces, and asks an evolution
agent to tie each bounded edit to a predicted effect. Evaluation then checks
the effect, turning edits into falsifiable file-level contracts.

Editable components include prompts, tools, middleware, skills, and long-term
memory. Evaluator files, runs, model configuration, and protected controls stay
outside the editable workspace.

<details>
<summary>Original sources for this mechanism</summary>

- Component and observability design: [AHE, §§2–3](https://arxiv.org/abs/2604.25850).
- Evolution loop and safeguards: [AHE, §4](https://arxiv.org/abs/2604.25850).
- Transfer and ablations: [AHE, experiments](https://arxiv.org/abs/2604.25850).
- Checked-in text: `evidence/weng/text/ahe.txt`.

</details>

## Evaluation and limits

The preprint reports Terminal-Bench 2 pass@1 rising from 69.7% to 77.0%, a
frozen-harness transfer result on SWE-bench Verified, and ablations attributing
gain to tools, middleware, and long-term memory rather than prompt-only edits.

The source is not independently reproduced. A changed harness can still exploit
traces, verifiers, or presentation unless capability boundaries enforce the
declared read/write split.

## Claim ceiling

AHE supports author-reported `harness-improvement` with unusually strong
observability and attribution. It does not measure next-cycle improvement
production by accepted children.

## Reading routes

- [[knowledge/rsi/weng/06-self-improving-harnesses|Weng: self-improving harnesses]]
- [[knowledge/rsi/lessons/04-stop-vs-self-harness-ahe|STOP versus Self-Harness and AHE lesson]]
- [[knowledge/rsi/chapters/evaluation-promotion-containment|Evaluation and promotion]]
- [Original paper](https://arxiv.org/abs/2604.25850)
