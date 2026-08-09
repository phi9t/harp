---
id: rsi-durable-execution
kind: concept
title: Durable execution mechanisms
summary: Idempotency and checkpoints for long-running improvement workflows.
primary_parent: rsi-durable-improvement-workflows
additional_parents: []
related: []
attachments:
  - content/chapters/durable-improvement-workflows.md
claims: []
human_review: null
---

# Durable execution mechanisms

Improvement runs often outlive one process or model context. Durability requires ordinary code to make completed work observable and repeated execution safe.

## Idempotency

An operation is idempotent when repeating the same logical request leaves the system in the same valid state as one successful execution. Durable workflows use stable operation keys, compare-and-swap writes, and recorded completion results. Merely retrying a non-idempotent effect can create duplicate jobs, charges, messages, or promotions.

## Checkpoints

A checkpoint records enough control and experiment state to resume from a known boundary. It names completed steps, pending work, artifact identities, budgets, and the workflow schema that interprets the record. A useful checkpoint is written before relinquishing execution and verified before later code trusts it.

<details>
<summary>Original sources for this mechanism</summary>

- [Workflows that persist across interruptions](../chapters/durable-improvement-workflows.md#retry-and-duplicate-action-prevention) gives the retry and checkpoint state machine.
- Karpathy Autoresearch's pinned `program.md` and `README.md` define its fixed experiment loop at commit `228791fb499afffb54b46200aca536f79142f117`.

</details>
