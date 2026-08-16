---
id: deepseek-harness-persistence-sdk-and-runtime-packaging
title: DeepSeek Harness decision 09 - Persistence, SDK, and runtime packaging
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, persistence, api-gateway, sdk, packaging]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, DSH-SMOKE]
---

# Decision 09: Persistence, SDK, and runtime packaging

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C013], [DSH-C014].** DSH packages durable session
storage, API gateway contracts, host/client build faces, and SDK/runtime entry
points as explicit seams. Persistence stores the canonical session event log;
API gateway code exposes selected unary Remote methods through generated
contracts rather than untyped endpoint convention.

## Evidence

**EVIDENCE - persistence seam.** The persistence doc says session persistence
is the durability seam for the event log and keeps the in-memory `SessionEvent`
as the single event vocabulary, with no parallel persisted event type
([[evidence/implementations/deepseek_harness/snapshot/docs/subsystems/persistence.md|persistence docs]]
([lines 1-8](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/persistence.md#L1))).

**EVIDENCE - checkpoint and recovery.** Persistence copies synchronous
`session/event` notifications into backend controllers, batches writes, uses
`session/flush` as a checkpoint, and repairs cold sessions with interrupted
turns without truncating committed events
([lines 9-19](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/persistence.md#L9)).

**EVIDENCE - metadata beside log.** `SessionHeader` stores format version,
session id, cwd, parent session, seed length, subagent origin, delegation
depth, and agent preset separately from conversation events
([lines 41-95](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/persistence.md#L41)).

**EVIDENCE - backend shape.** JSONL and SQLite backends implement the same
abstract `SessionPersistence`; SQLite maps one row per `SessionEvent` with no
parallel persisted schema
([lines 231-237](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/persistence.md#L231)).

**EVIDENCE - API gateway contract.** The API gateway doc says business
services mark selected methods with `@Remote` or `@RemoteScope`, unmarked
methods do not enter generated client types or runtime contributions, and
Host/Client contracts are generated through a strict build pipeline
([[evidence/implementations/deepseek_harness/snapshot/docs/api-gateway.md|API gateway]]
([lines 7-17](../../evidence/implementations/deepseek_harness/snapshot/docs/api-gateway.md#L7),
[lines 80-99](../../evidence/implementations/deepseek_harness/snapshot/docs/api-gateway.md#L80))).

## Why this matters

**INFERENCE.** Persistence and API packaging are where a harness either
becomes replayable software or collapses into UI state. DSH's persistence seam
keeps the session log canonical while allowing backend substitution. Its API
gateway keeps host object resolution, client method generation, transport, and
business dispatch separate.

For Harp, the lesson is that runtime packaging is part of harness design. A
candidate harness change that alters a Remote method, persistence header, SDK
entrypoint, or generated contract changes the observable system boundary, not
just internal code.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** The local receipt records package scripts and
source surfaces for build, API, SDK, and verification commands. It does not
execute the web client, publish packages, or validate a Python SDK round trip.

## Failure modes

**MISSING.** This packet does not include a crash-recovery test, a strict
Remote generation failure test, or an SDK invocation against a running host.
Those are needed for operational verification of the packaging design.
