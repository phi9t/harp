---
id: rsi-durable-improvement-workflows
kind: concept
title: Workflows that persist across interruptions
summary: Durable plan-execute-observe-improve loops with checkpoints, retries, idempotency, and code-enforced steps.
primary_parent: mlsys
additional_parents:
  - infrastructure
related:
  - kind: executes
    target: rsi-harness-engineering
  - kind: supports
    target: rsi-automated-research
attachments:
  - content/source_registry.md
  - content/rsi_harness_by_lil_log_deconstructed.md
claims: []
human_review: null
---

# Workflows that persist across interruptions

## The technical problem

Long improvement loops outlive one model context, process, machine, reservation, or human approval window. A durable workflow preserves enough state to resume without repeating completed side effects, losing measurements, or allowing two processes to advance the same run.

The workflow owns control state. The model may choose a hypothesis or propose code, but ordinary code enforces required steps, budgets, timeouts, and promotion rules.

Represent run state as `S = (phase, attempt, inputs, outputs, pending, owner, history)`. A transition `δ(S, event) → S'` is valid only if the event matches the current phase, required inputs exist, and the caller holds the current writer token. Persist `S'` before acknowledging completion.

## Plan-execute-observe-improve

1. **Plan.** The model proposes a bounded change and an expected measurable effect.
2. **Validate.** Code checks editable paths, requested resources, and required metadata.
3. **Execute.** A worker applies the change and starts the experiment.
4. **Wait.** The workflow records an external job identifier and sleeps or polls without occupying model context.
5. **Observe.** A collector records metrics, artifacts, failures, and resource use.
6. **Decide.** Fixed gates keep, reject, or request another bounded proposal.
7. **Checkpoint.** The workflow seals the branch and the next legal transition.
8. **Repeat.** A later worker resumes from persisted state.

Autoresearch is a compact example: edit one training program, run a fixed-duration experiment, read one metric, keep or discard the change, and repeat. Its strength is the narrow mutation surface and objective. It becomes durable only when run state, jobs, metrics, and decisions survive process loss.

<details>
<summary>Original sources for this mechanism</summary>

- Karpathy Autoresearch, pinned `README.md` and `program.md` at commit `228791fb499afffb54b46200aca536f79142f117`, defines the edit, train, measure, and keep-or-discard loop and its bounded files: [upstream tree](https://github.com/karpathy/autoresearch/tree/228791fb499afffb54b46200aca536f79142f117).
- [Weng reader companion, durable execution section](../rsi_harness_by_lil_log_deconstructed.md). It compares the source loop with pinned Pi, Hermes, and Codex persistence and lineage mechanisms.
- [Harness engineering](harness-engineering.md) defines persistence, tool authority, and recovery boundaries used here.

</details>

## Retry and duplicate-action prevention

A retry is safe only when the operation is idempotent or reconciled. Give each effect an action key:

`k = hash(run_id, phase, logical_action, input_digest)`

Before executing, the worker asks the authoritative target whether `k` already completed. If yes, it records the existing result. If no, it executes once and stores the target's operation identifier. A timeout is not proof of failure; reconcile before retry.

Separate delivery attempts from semantic attempts. A workflow engine may redeliver a task because a worker died. The domain state machine decides whether the experiment itself may be attempted again.

## Checkpoints and long waits

A useful checkpoint contains:

- workflow schema version and phase;
- immutable input and candidate digests;
- completed action keys and external operation IDs;
- result and artifact locators;
- active writer identity or lease;
- retry classification and remaining budget;
- next legal transitions.

Long waits should release compute and model context. Completion events, timers, and approvals wake the workflow. The archive keeps the ordered history needed to replay control decisions, while large experiment artifacts stay in referenced storage.

## Fixed steps enforced by code

Some steps should not be model choices:

- validate the candidate manifest;
- freeze evaluator and task split;
- run integrity checks before outcome scoring;
- collect descendant resource use;
- archive failures and rejected candidates;
- require external promotion authority;
- stop at the predeclared budget or generation limit.

The model can propose content within those steps. It cannot skip them because a prompt says they are mandatory. The state machine rejects an invalid transition.

## Failure modes and tradeoffs

- **Double execution.** A timeout triggers the same external effect twice.
- **Split brain.** Two workers hold apparent authority to advance one run.
- **Checkpoint-before-effect mismatch.** State says an action completed when the external system never accepted it.
- **Effect-before-checkpoint mismatch.** The effect completed but resume logic cannot discover it.
- **Schema drift.** New workers interpret old state differently.
- **Retry amplification.** Nested workflow, job, and model retries multiply spend.
- **Stale observation.** A delayed metric is attached to the wrong candidate.
- **Model-owned sequencing.** The model silently skips a required evaluator step.

Durability adds storage, orchestration, and migration work. Short deterministic tasks may not need a workflow service. Long waits, external jobs, approvals, and costly side effects usually do.

## What would weaken the mechanism

Crash injection should interrupt every boundary between state persistence and side effects. The workflow claim weakens if any injection creates an unreconciled duplicate, lost result, two active writers, or a state from which the legal next action is ambiguous. A replay that reaches a different control decision from the same recorded history also falsifies deterministic workflow logic.

## Open technical questions

- Which experiment actions can provide strong idempotency keys across service boundaries?
- How should workflow schema migration preserve old promotion semantics?
- When should a workflow retain raw events versus compacted summaries?
- How can a long-running workflow attest model and evaluator identities supplied by external services?

<details>
<summary>Reference records and operational metadata</summary>

- The captured public implementations establish bounded persistence and lineage mechanisms, not one complete durable RSI runner.
- The pinned Autoresearch source identity and claim ceiling are in [the source registry](../source_registry.md).
- A production run receipt must name the actual workflow engine, worker revision, schema, and external job identities.

</details>
