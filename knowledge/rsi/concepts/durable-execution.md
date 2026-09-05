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

The detailed controller contract is in [[knowledge/rsi/chapters/durable-improvement-workflows|workflows that persist across interruptions]]. This page keeps the terms that recur in other RSI chapters.

## Idempotency

An operation is idempotent when repeating the same logical request leaves the system in the same valid state as one successful execution. Durable workflows use stable operation keys, compare-and-swap writes, and recorded completion results. Merely retrying a non-idempotent effect can create duplicate jobs, charges, messages, or promotions.

An action key identifies one intended semantic effect across delivery attempts.
An operation identifier names the receiver-side operation after acceptance. A
local action key does not create exactly-once external execution. Recovery must
query the authoritative receiver and retain an action as pending when status is
unknown.

## Writer fencing

One controller with an unexpired lease and current fencing token may advance a
run. Compare-and-swap protects durable state writes, but not an expired
controller request that reaches a receiver after ownership changes. The
receiver, or a controller-owned dispatcher, must validate the fencing token.
Otherwise do not reassign ownership while an effect remains ambiguous.

## Checkpoints

A checkpoint records enough control and experiment state to resume from a known boundary. It names completed steps, pending work, artifact identities, budgets, and the workflow schema that interprets the record. A useful checkpoint is written before relinquishing execution and verified before later code trusts it.

## Harp's executor and scheduler

Harp's implementation uses one durable executor. Dynamic Workflow is an
authored Rust representation that compiles into `TaskGraph`; it is not a second
scheduler. Sequence creates dependency chains, parallel branches share their
incoming dependencies, and pipelines give each item its own stage chain.

```text
Authored workflow
       |
       v
Compile and validate TaskGraph
       |
       v
Engine: schedule dependency-ready activities
       |                         ^
       v                         |
Provider activity ------> recorded result and state
                                 |
                                 v
                         resume or reduce
```

The engine records state before effects, validates result envelopes, publishes
artifacts, and resumes incomplete runs. Recovery, budget accounting, retries,
and accepted-result accounting belong to this layer. Context Control selects
provider-backed workflows above it; it does not add another durable scheduler.

These are implementation boundaries, not a guarantee of exactly-once external
effects or mathematical correctness. The durable graph currently supports
Codex; its Trae adapter is not implemented. The separate provider wrapper
supports both Codex and Trae CLI. See the
[workflow compilation decision](https://github.com/phi9t/harp/blob/master/docs/adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md)
and [agent workflow guide](https://github.com/phi9t/harp/blob/master/docs/agent-workflows.md) for the contract and commands.

<details>
<summary>Original sources for this mechanism</summary>

- [[knowledge/rsi/chapters/durable-improvement-workflows#Retry and duplicate-action prevention|Workflows that persist across interruptions]] gives the retry and checkpoint state machine.
- Karpathy Autoresearch's pinned `program.md` and `README.md` define its fixed experiment loop at commit `228791fb499afffb54b46200aca536f79142f117`.

</details>
