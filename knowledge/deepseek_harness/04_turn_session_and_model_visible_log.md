---
id: deepseek-harness-turn-session-and-model-visible-log
title: DeepSeek Harness decision 04 - Turn, session, and model-visible log
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, sessions, event-log, turns, replay]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, DSH-SMOKE]
---

# Decision 04: Turn, session, and model-visible log

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C006], [DSH-C007].** DSH chooses an event-sourced session
log as the model-visible state boundary. A turn is one or more model steps, and
anything that reaches a model request must be reconstructable from the session
log rather than held only in live process memory.

## Evidence

**EVIDENCE - turn flow.** The architecture guide defines a step as one model
request plus the tools it calls, and a turn as zero or more steps. The text
lists the durable sequence from `turn/start` through user messages, request,
assistant chunks, tool calls/results, `step/end`, `agent/turn-stopping`, and
`turn/end`
([[evidence/implementations/deepseek_harness/snapshot/docs/architecture.md|architecture]]
([lines 63-90](../../evidence/implementations/deepseek_harness/snapshot/docs/architecture.md#L63))).

**EVIDENCE - session source of truth.** The sessions doc states that a session
is an append-only log of typed events, and LLM message history is derived from
the log rather than stored separately
([[evidence/implementations/deepseek_harness/snapshot/docs/subsystems/session.md|session docs]]
([lines 1-6](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/session.md#L1))).

**EVIDENCE - request reconstruction.** The same session doc says the
`request/header` event records call config, rendered system prompt, and tool
schemas as logged request state, while route capacity stays separate in
`request/context`
([lines 152-180](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/session.md#L152)).

**EVIDENCE - loop source.** `ReactLoopAgent` opens a turn, claims input,
dispatches `agent/pre-step`, appends entered `user/message` events, streams
assistant chunks, appends an assembled assistant message, executes tool calls,
and closes step/turn boundaries
([[evidence/implementations/deepseek_harness/snapshot/packages/core/agent-loop/src/agent.ts|agent loop]]
([lines 245-330](../../evidence/implementations/deepseek_harness/snapshot/packages/core/agent-loop/src/agent.ts#L245),
[lines 332-399](../../evidence/implementations/deepseek_harness/snapshot/packages/core/agent-loop/src/agent.ts#L332))).

## Why this matters

**INFERENCE.** Event sourcing turns a harness run into an object an evaluator
can inspect: what input entered, what request envelope was used, what the model
streamed, which tool calls were made, which results returned, and how the turn
closed. That matters for harness evolution because a candidate cannot be
judged only by its final answer if it may change tools, prompts, policy, or
state-management behavior.

The design also narrows a class of replay bugs. If a plugin wants to show the
model new context, it needs a logged channel; a hidden live mutation is not a
valid model-visible input. That raises implementation cost but creates an
audit boundary.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** The smoke receipt confirms the pinned source and
keyless static inspection. It does not run a provider request or compare a
recorded session replay against a live model output.

## Failure modes

**MISSING.** The current evidence does not include a full end-to-end run that
persists and reloads a session across process restart. The design supports
that path; this packet does not claim it reproduced it.
