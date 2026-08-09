---
id: rsi-improvement-types
kind: concept
title: Improvement types and recursion boundaries
summary: Task gains, persistent adaptation, harness changes, and accepted successor improvement.
primary_parent: rsi-recursive-improvement-loop
additional_parents: []
related: []
attachments:
  - content/chapters/recursive-improvement-loop.md
claims: []
human_review: null
---

# Improvement types and recursion boundaries

These four outcomes differ in what changes, how long the change persists, and what an experiment must measure. Treating them as synonyms makes a one-run task gain look like evidence that the system became a better improver.

## Task improvement

Task improvement raises an externally measured score for one artifact or episode while leaving the process that produced it unchanged. More samples, critique and revision, test-time search, or a larger execution budget can all improve an answer. The required comparison holds the evaluator and total budget fixed. Persistence and better future proposal quality are not implied.

## Persistent adaptation

Persistent adaptation changes state that later episodes read, such as memory, demonstrations, retrieved notes, or a skill file. A valid experiment must show that the stored change survives the episode boundary and causes later behavior to differ. Persistence alone is not improvement: stale or contaminated memory can reduce performance.

## Harness improvement

Harness improvement changes executable policy around the model, including context assembly, tool schemas, workflow control, verification, or recovery. The model weights may remain fixed. The claim needs matched-model evaluation against the prior harness and must account for added calls, tools, latency, and human repair.

## Successor improvement

Successor improvement begins when external promotion authority accepts a child candidate and records `Cₜ → Cₜ₊₁`. Evidence for recursion then compares how parent and child produce later valid candidates under the same proposal protocol, evaluator, permissions, and root-tree budget. A better child score without a better next-cycle improvement process is immediate gain, not recursive gain.

<details>
<summary>Original sources for this mechanism</summary>

- [What makes an improvement loop recursive](../chapters/recursive-improvement-loop.md#the-technical-problem) defines the five-level boundary and matched-envelope recursion test.
- STOP, §6 "Limitations," states that its fixed base model does not demonstrate full recursive self-improvement: [arXiv:2310.02304v3](https://arxiv.org/abs/2310.02304).
- Darwin Gödel Machine, §§2–3, defines an empirically selected archive of agent variants while retaining a frozen foundation model: [arXiv:2505.22954v3](https://arxiv.org/abs/2505.22954).

</details>
