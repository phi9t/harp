---
id: rsi-harness-engineering
kind: concept
title: Harness engineering
summary: The agent loop, context, tools, persistence, delegation, verification, and recovery around a foundation model.
primary_parent: mlsys
additional_parents:
  - modeling
  - infrastructure
related:
  - kind: wraps
    target: rsi-foundation-model-inside-the-loop
  - kind: optimized-by
    target: rsi-harness-search
attachments:
  - content/pi_harness_deep_dive.md
  - content/hermes_harness_deep_dive.md
  - content/codex_harness_deep_dive.md
  - content/codex_state_continuity_and_compaction.md
  - content/rsi_harness_by_lil_log_deconstructed.md
claims: []
human_review: null
---

# Harness engineering

## The harness is executable policy

A harness turns a foundation model into an agent by deciding what the model sees, which actions it may request, how those actions run, what state persists, and when the loop stops. Behavior in weights and behavior in the harness can produce the same visible answer through different mechanisms. The distinction becomes critical when a system edits prompts, tools, workflows, or memory and calls the result self-improvement.

Represent one agent step as:

`cₜ = Context(H, sₜ, mₜ)`

`aₜ ∼ Model_W(cₜ, d)`

`rₜ = Dispatch(P, Tools, aₜ)`

`(sₜ₊₁, mₜ₊₁) = Persist(H, sₜ, mₜ, aₜ, rₜ)`

`H` is harness policy, `sₜ` is environment and session state, `mₜ` is persistent memory, `P` is permission policy, and `d` is decoding policy. The model proposes `aₜ`; ordinary code validates, dispatches, records, and recovers.

## Agent-loop control flow

1. Load the model revision, harness configuration, session history, and allowed capabilities.
2. Build model context from instructions, retrieved records, tool schemas, recent events, and compacted history.
3. Sample a response or structured action.
4. Parse the action at the trust boundary. Reject malformed, unknown, or forbidden requests.
5. Execute allowed actions through a runtime that enforces permissions and budgets.
6. Persist the proposed action before or with its side effect, then persist the result.
7. Decide whether to continue from model follow-up, queued input, a workflow transition, or a stop rule.
8. Compact model-visible context when needed while retaining full archive references.
9. Return control to an external evaluator for acceptance, not to the candidate's self-assessment.

The loop is an implementation mechanism, not evidence of improvement. A recursive claim needs an outer process that proposes harness variants, evaluates them independently, promotes one, and tests whether it improves later harness work.

<details>
<summary>Original sources for this mechanism</summary>

- [[knowledge/rsi/pi_harness_deep_dive|Pi implementation deep dive]], pinned at commit `4488ad55c18f07ae89a489096c90de8667b3adfb`. It establishes a compact loop, hookable context and tools, branchable session state, and compaction.
- [[knowledge/rsi/hermes_harness_deep_dive|Hermes implementation deep dive]], pinned at commit `e444d165807f489b5c1ab8e4a612c8d09c2e67a2`. It establishes persistent memory, skills, curation, bounded delegation, and iteration accounting.
- [[knowledge/rsi/codex_harness_deep_dive|Codex implementation deep dive]], pinned at commit `1e85ca099e4265bf89f4016772d299816e231bb3`. It establishes rollout state, routed capabilities, compaction, multi-agent lineage, permissions, and sandbox selection.
- [[knowledge/rsi/codex_state_continuity_and_compaction|Codex state-continuity companion]] separates the ARC-AGI-3 harness result from Codex's richer typed history, world-state, checkpoint, replay, and cross-thread memory architecture.
- Weng, "Harness Engineering for Self-Improvement," sections "Agent harness" and "Harness design patterns," supplies the broad harness decomposition and improvement argument; the [[knowledge/rsi/rsi_harness_by_lil_log_deconstructed|reader companion]] preserves commentary separately: [Lil'Log 2026-07-04](https://lilianweng.github.io/posts/2026-07-04-harness/).

</details>

## Context construction and retrieval

Context policy chooses which evidence enters a finite model window and in which order. It includes system instructions, conversation state, retrieved documents, tool definitions, environment observations, branch summaries, and error traces. A context transform can improve behavior without changing weights by making relevant variables easier to attend to.

Retrieval has three separate jobs:

- **Discovery** finds candidate records.
- **Selection** chooses records within the budget.
- **Presentation** formats them for the model.

Evaluation must vary these independently. A better retriever can appear worse if presentation is noisy; a prompt can appear better because it received privileged evaluation records.

Memory is persisted context with an admission and retirement policy. Raw transcripts, compact summaries, declarative facts, skills, and executable scripts have different failure modes. A memory store needs provenance, scope, expiration, conflict handling, and a way to show which stored item influenced an action.

## Tools, permissions, and filesystem state

Tool visibility and runtime authority are different. The model may see a tool specification, but a trusted registry must map that specification to an implementation and permission profile. Every action should cross one validation point that checks:

- tool identity and schema;
- path, network, credential, and process scope;
- approval requirements;
- resource budget;
- idempotency or duplicate-action key;
- persistence and audit requirements.

Filesystem state is useful because it can hold plans, code, tests, checkpoints, artifacts, and receipts across context resets. It is also dangerous because stale files, hidden state, generated outputs, and broad write access can leak evaluation data or bypass promotion. A harness-improvement experiment should expose a declared candidate directory and mount evaluator assets separately as read-only or inaccessible.

## Subagents and concurrency

Subagents create additional proposal capacity and isolated contexts. They do not automatically create independent evidence because they may share the same model, prompt bias, tools, and evaluator. The root run must account for every descendant's calls, tokens, wall time, processes, and external spend.

Concurrent agents should not write the same branch, archive row, or state object. Assign separate workspaces and merge through an explicit controller. A shared append path needs one writer token or a transactional store. Waiting and cancellation must preserve child status so abandoned work does not vanish from the receipt.

## Verification and recovery

Verification observes the final artifact through an authority the candidate cannot rewrite. It includes tests, static analysis, invariant checks, runtime probes, and human review. The harness may run development checks, but the promotion controller reruns fresh checks after all candidate actions finish.

Recovery needs durable state:

- completed step and output identity;
- pending action and idempotency key;
- attempt number and retry policy;
- active child operations;
- checkpoint and rollback target;
- immutable evaluator and permission revisions.

After interruption, the controller reconstructs state from the archive, not from the model's memory of what happened. Side effects that cannot be retried safely must expose an external operation identifier or a reconciliation read.

## Worked implementation examples

### Pi

Pi keeps the loop small and makes prompt, tools, hooks, context transforms, compaction, and branch state visible. That makes it a good harness-search testbed. Its extension power also means candidate extensions cannot own the evaluator.

### Hermes

Hermes puts memory, skills, curation, sessions, and delegation into durable stores. It is a useful reference for persistent adaptation. Usage and curator decisions are not held-out fitness measurements.

### Codex

Codex separates model-visible tool specifications from runtime registration, persists rollouts, and manages thread ancestry and sandbox policy. It is a useful containment reference. The larger state space makes controlled attribution harder.

## Failure modes and tradeoffs

- **Prompt-policy ambiguity.** Critical behavior exists only in prose and cannot be tested as a state transition.
- **Context contamination.** Held-out answers or evaluator details enter retrieval or memory.
- **Compaction loss.** A summary drops constraints or changes the meaning of prior evidence.
- **Tool confusion.** A visible tool name resolves to the wrong runtime or permission profile.
- **Side-effect ambiguity.** A crash occurs after an effect but before the archive records it.
- **Budget laundering.** Child work or retries escape root accounting.
- **Shared-state races.** Parallel agents overwrite files or promotion state.
- **Self-verification.** The same mutable candidate both produces and ratifies its score.
- **Recovery drift.** Resume logic reruns completed work or skips an unfinished action.

More harness structure improves control but increases code, latency, and interaction complexity. A smaller harness is easier to audit but may lack durable recovery or fine-grained authority. The right design depends on the experiment's risk and horizon.

## What would weaken the mechanism

Harness attribution weakens if frozen-weight ablations show no reliable behavior difference, if improvements disappear with fresh tasks, or if the effect is explained by more context or compute. A claimed recovery mechanism fails if injected crashes produce duplicate effects, lost lineage, or an ambiguous final state.

## Open technical questions

- Which harness components should be model-editable, and which should remain ordinary fixed code?
- Can context policies be optimized without leaking held-out structure?
- How should a memory curator measure long-term usefulness and deletion cost?
- Which recovery invariants remain valid when actions span external services?

<details>
<summary>Reference records and operational metadata</summary>

- Exact source paths and code revisions remain in the three implementation deep dives and [[knowledge/rsi/source_registry|source registry]].
- The older [[knowledge/rsi/implementation_harnesses|implementation comparison]] remains as a concise overlay and links here for the canonical explanation.
- No inspected harness ships the complete external multi-generation evaluator required for an RSI result.

</details>
