---
id: deepseek-harness-subagents-and-continuable-children
title: DeepSeek Harness decision 08 - Subagents and continuable children
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, subagents, delegation, continuable-children]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, DSH-SMOKE]
---

# Decision 08: Subagents and continuable children

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C012].** DSH treats subagents as an optional capability
seam with named providers, not as a special case inside the core agent loop.
It supports both one-shot delegation and continuable child sessions with
activation ownership, direct-parent authorization, and cold-resume semantics.

## Evidence

**EVIDENCE - seam placement.** The subagent doc says the subagent seam is one
optional capability, not part of the agent loop, and multiple provider
implementations coexist in one context behind `ctx.subagents`
([[evidence/implementations/deepseek_harness/snapshot/docs/subsystems/subagent.md|subagent docs]]
([lines 5-10](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/subagent.md#L5))).

**EVIDENCE - start-time capabilities.** Provider descriptors advertise
start-time features such as output schema, depth limit, tool filter, and
persona. A request needing an unsupported capability is rejected before
delegation rather than silently degraded
([lines 11-33](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/subagent.md#L11)).

**EVIDENCE - one-shot request.** The one-shot request carries parent agent,
prompt, signal, optional schema, depth, tool filter, and persona; parent
supplies cwd, lineage, and delegation depth
([lines 35-99](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/subagent.md#L35)).

**EVIDENCE - continuable model.** The continuable path models one durable child
session with at most one live activation. Follow-up routing depends on
activation state; direct-parent authorization is required; interrupt cancels
the live turn while preserving the activation and parked FIFO queue
([lines 114-145](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/subagent.md#L114)).

## Why this matters

**INFERENCE.** Subagents introduce a second axis of harness evolution: not only
which tools a model can call, but which other agents it can ask to work. DSH's
design keeps this as a capability seam so providers can vary from in-process
children to external products while the parent harness reasons about one
contract.

The continuable design is especially important for long-running work. A child
is not a result-bearing wrapper around a single task; it is a durable session
that can accept later FIFO turns and survive process-local activation changes.
That makes delegation inspectable in the same session lineage language as the
main agent.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** The local receipt supports static inspection of
the subagent package and docs. It does not run a child provider, external ACP,
Codex, Claude Code, or DSH SDK delegation path.

## Failure modes

**MISSING.** Harp has not yet recorded a start-time unsupported-capability
denial, a continuable follow-up, a cold resume, or an unauthorized interrupt.
Those would be the empirical checks needed before treating the seam as
operationally verified.
