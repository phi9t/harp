---
id: rsi-harness-components
kind: concept
title: Harness components
summary: Context, retrieval, memory, tools, permissions, filesystem state, subagents, verification, and recovery.
primary_parent: rsi-harness-engineering
additional_parents: []
related: []
attachments:
  - content/chapters/harness-engineering.md
  - content/context_engineering_deep_dive.md
claims: []
human_review: null
---

# Harness components

A harness is executable policy around a model. These components determine what the model observes, which effects it may request, what persists, and how the system detects failure.

## Context construction

Context construction selects instructions, history, retrieved records, tool schemas, and current state, then orders them inside the model's context limit. It is a policy because inclusion, omission, and ordering change behavior. A comparison must hold the underlying model fixed and record the exact assembled context.

The [context-engineering deep dive](../context_engineering_deep_dive.md)
separates learned context artifacts from the procedures that retrieve, update,
and validate them, traces ACE, MCE, and Meta-Harness as progressively broader
mutable surfaces, and separates availability, selection, rendering, activation,
adherence, and outcome.

The [Codex state-continuity companion](../codex_state_continuity_and_compaction.md)
covers the runtime side: typed active history, opaque reasoning continuation,
world-state refresh, atomic compaction, replay, and cross-thread memory.

## Retrieval

Retrieval maps a query and corpus state to a bounded set of records. Its index, query rewriting, filters, ranking, and freshness policy affect both recall and prompt cost. Retrieval quality should be measured against task-relevant evidence, not only whether some text was returned.

## Memory

Memory persists observations or conclusions across episodes. A reliable memory record keeps source identity, conditions, uncertainty, and replacement rules so a later model can distinguish evidence from summary. Write access and expiry matter as much as retrieval.

## Tools and permissions

Tool schemas translate model output into typed requests. Ordinary code validates arguments and enforces the permission policy before effects occur. A prompt saying "do not delete files" is not a capability boundary; the runtime must withhold or constrain the operation.

## Filesystem state

Filesystem state gives the agent durable artifacts, checkpoints, and code, but introduces races, symlink attacks, partial writes, and stale reads. Safe harnesses bind reads and writes to a held directory, publish atomically, and record the revision or digest consumed by evaluation.

## Subagents

Subagents split work into concurrent or specialized contexts. Delegation improves throughput only when write ownership, result contracts, and integration checks are explicit. Total descendant calls and compute belong to the root-tree budget.

## Verification and recovery

Verification observes the real artifact through tests, static checks, or runtime probes before the harness claims success. Recovery resumes from durable state, distinguishes retryable from terminal failures, and avoids repeating completed side effects.

<details>
<summary>Original sources for this mechanism</summary>

- [Harness engineering](../chapters/harness-engineering.md#agent-loop-control-flow) defines the control loop and component boundaries.
- Pi, Hermes, and Codex implementation locators are maintained in the [implementation comparison](../implementation_harnesses.md) and pinned source records.
- Lilian Weng, \"Harness Engineering for Self-Improvement,\" sections \"Context engineering\" and \"Agent harness\": [2026-07-04](https://lilianweng.github.io/posts/2026-07-04-harness/).

</details>
