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

The [[knowledge/rsi/chapters/harness-engineering|harness-engineering chapter]]
owns the end-to-end request and dispatch contract. This page names its
components without treating model-visible interfaces as authority.

## Context construction

Context construction selects instructions, history, retrieved records, tool schemas, and current state, then orders them inside the model's context limit. It is a policy because inclusion, omission, and ordering change behavior. A comparison must hold the underlying model fixed and record the exact assembled context.

Protected instructions, approvals, unresolved actions, and tool schemas need
reserved context budget. An editable harness can select and order content only
inside declared slots. If protected state does not fit, the controller must fail
closed rather than omit it or convert it into an unverified summary.

The [[knowledge/rsi/context_engineering_deep_dive|context-engineering deep dive]]
separates learned context artifacts from the procedures that retrieve, update,
and validate them, traces ACE, MCE, and Meta-Harness as progressively broader
mutable surfaces, and separates availability, selection, rendering, activation,
adherence, and outcome.

The [[knowledge/rsi/codex_state_continuity_and_compaction|Codex state-continuity companion]]
covers the runtime side: typed active history, opaque reasoning continuation,
world-state refresh, atomic compaction, replay, and cross-thread memory.

## Retrieval

Retrieval maps a query and corpus state to a bounded set of records. Its index, query rewriting, filters, ranking, and freshness policy affect both recall and prompt cost. Retrieval quality should be measured against task-relevant evidence, not only whether some text was returned.

## Memory

Memory persists observations or conclusions across episodes. A reliable memory record keeps source identity, conditions, uncertainty, and replacement rules so a later model can distinguish evidence from summary. Write access and expiry matter as much as retrieval.

## Tools and permissions

Tool schemas translate model output into typed requests. Ordinary code validates arguments and enforces the permission policy before effects occur. A prompt saying "do not delete files" is not a capability boundary; the runtime must withhold or constrain the operation.

A model proposal becomes an action only after trusted code resolves the tool
implementation, applies external policy, reserves budget, assigns an action
key, and records intent. The request schema does not grant authority.

## Filesystem state

Filesystem state gives the agent durable artifacts, checkpoints, and code, but introduces races, symlink attacks, partial writes, and stale reads. Safe harnesses bind reads and writes to a held directory, publish atomically, and record the revision or digest consumed by evaluation.

## Subagents

Subagents split work into concurrent or specialized contexts. Delegation improves throughput only when write ownership, result contracts, and integration checks are explicit. Total descendant calls and compute belong to the root-tree budget.

## Verification and recovery

Verification observes the real artifact through tests, static checks, or runtime probes before the harness claims success. Recovery resumes from durable state, distinguishes retryable from terminal failures, and avoids repeating completed side effects.

Recovery starts from the pending intent and receiver observation, not from the
model recollection. It records unresolved state when the receiver cannot prove
completion or absence.

<details>
<summary>Original sources for this mechanism</summary>

- [[knowledge/rsi/chapters/harness-engineering#Agent-loop control flow|Harness engineering]] defines the control loop and component boundaries.
- Pi, Hermes, and Codex implementation locators are maintained in the [[knowledge/rsi/implementation_harnesses|implementation comparison]] and pinned source records.
- Lilian Weng, \"Harness Engineering for Self-Improvement,\" sections \"Context engineering\" and \"Agent harness\": [2026-07-04](https://lilianweng.github.io/posts/2026-07-04-harness/).

</details>
