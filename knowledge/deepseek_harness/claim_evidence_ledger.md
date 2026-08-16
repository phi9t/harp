---
id: deepseek-harness-claim-evidence-ledger
title: DeepSeek Harness claim evidence ledger
type: claim-ledger
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, claims, evidence, cordis]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, CORDIS-PAPER, DSH-SMOKE]
---

# DeepSeek Harness claim evidence ledger

Mode: `CLAIM LEDGER`.

## DSH-C001: DSH explicitly uses Cordis and an everything-is-a-plugin architecture

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `evidence/implementations/deepseek_harness/snapshot/README.md`, lines 5-11; `docs/architecture.md`, lines 9-14
- Claim: DSH describes itself as an open-source DeepSeek AI agent harness powered by Cordis, with every product part modeled as a plugin and no privileged core to patch.
- Reproduction status: source snapshot captured; local smoke verifies commit identity.

## DSH-C002: Cordis frames dynamic composition as temporal plus spatial composability

- Class: `SOURCE CLAIM`
- Source: `CORDIS-PAPER`
- Locator: `evidence/cordis_paper/text/cordis-paper-v8.txt`, lines 13-27, 132-148, and 238-266
- Claim: The Cordis paper identifies temporal composability as reversible side-effect management and spatial composability as structured dependency management, then implements the ideas in Cordis.
- Reproduction status: paper PDF captured and text extracted; mathematical metatheory not independently checked.

## DSH-C003: DSH boot composition is ordered profile, bundle, and patch layering

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/architecture.md`, lines 15-37
- Claim: DSH boot state is built from ordered profile bundles, profile patch, home patch, and optional command-line patch overlays; rows can be replaced by id.
- Reproduction status: source docs captured; local smoke inspects package scripts and examples rather than booting a full UI profile.

## DSH-C004: Cordis registrations are reversible effects tied to plugin lifecycle

- Class: `SOURCE CLAIM`
- Sources: `DEEPSEEK-HARNESS`, `CORDIS-PAPER`
- Locator: `docs/cordis-primer.md`, lines 7-14 and 40-44; `docs/cordis-tutorial/02-lifecycle-and-effects.md`, lines 5-6 and 84-94; Cordis paper lines 388-395 and 548-600
- Claim: Cordis registers services, listeners, prompt sections, tools, and other contributions as effects with disposers that unwind on plugin unload.
- Reproduction status: docs and paper captured; local smoke does not prove every disposer in the product tree is correct.

## DSH-C005: DSH separates durable session events, live agent events, and capability events

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/architecture.md`, lines 53-61 and 63-90; `docs/cordis-primer.md`, lines 15-34
- Claim: DSH uses durable session events for replayable facts, live `agent/*` events for in-flight coordination, and capability events for policy/adapters at service seams.
- Reproduction status: docs and generated catalogs captured.

## DSH-C006: A DSH turn is one or more model steps over an append-only session log

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/architecture.md`, lines 63-96; `packages/core/agent-loop/src/agent.ts`, lines 245-399
- Claim: The default loop opens a turn, claims inbox input, runs pre-step, appends entered user messages, derives model history from the log, streams assistant chunks, appends assistant messages, dispatches tool calls, and closes step/turn boundaries.
- Reproduction status: source docs and source code captured; no live model run executed.

## DSH-C007: Model-visible DSH context must be logged and reconstructable

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/architecture.md`, lines 92-96; `docs/subsystems/session.md`, lines 1-6 and 152-180
- Claim: DSH treats the session log as the source of model-visible history, and request headers/system/tool schema state are logged so request reconstruction is possible.
- Reproduction status: docs captured; local smoke can inspect invariants but does not replay an authenticated conversation.

## DSH-C008: A DSH capability seam has service definition, provider, and consumer roles

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/architecture.md`, lines 98-103; `docs/capability-seams.md`, lines 4-7 and 74-155
- Claim: DSH models swappable capabilities as service definitions plus provider implementations plus consumers; provider swaps can change product behavior without loop forks.
- Reproduction status: generated capability graph captured; local smoke inspects package surfaces.

## DSH-C009: DSH tool definitions separate model-visible schema from host execution metadata

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/subsystems/tools.md`, lines 9-12 and 96-151
- Claim: DSH registered tools contain model-facing schema fields plus host-only execution, output, timeout, concurrency, and presentation fields; the registry allowlists what reaches the model.
- Reproduction status: docs and source captured; keyless tutorial exercises the tool pipeline shape.

## DSH-C010: DSH tool execution is an evented policy pipeline

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/subsystems/tools.md`, lines 170-172 and 212-220; `packages/core/agent-loop/src/tool-calls.ts`, lines 40-100 and 112-245
- Claim: DSH materializes a tool execution, runs pre-execute policy, monotonic guards, around-dispatch wrappers, post-execute replacement, final content, and result emission, while the loop schedules tool calls with ordering and exclusivity rules.
- Reproduction status: docs and source captured; no untrusted tool run or provider call claimed.

## DSH-C011: DSH resolves one per-call sandbox policy for file and shell enforcement

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/subsystems/sandbox.md`, lines 9-30 and 41-80; `packages/sandbox/sandbox-policy/src/index.ts`, lines 1-16 and 126-151; `packages/fs/fs-sandbox/src/index.ts`, lines 1-30 and 115-148; `packages/shell/bash-sandbox/src/index.ts`, lines 1-8 and 88-114
- Claim: DSH uses a shared `ctx.sandboxPolicy` service to resolve file-effect mode and workspace root per call, while filesystem and shell providers enforce through their own surfaces.
- Reproduction status: source captured; Harp does not claim cross-platform sandbox enforcement was proven.

## DSH-C012: DSH subagents are a named provider seam with one-shot and continuable modes

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/subsystems/subagent.md`, lines 5-14, 35-43, 114-145, and 192-244
- Claim: DSH registers multiple subagent providers behind `ctx.subagents`; one-shot requests are capability-checked before start, while continuable children are durable sessions with activation ownership, direct-parent authorization, and cold-resume behavior.
- Reproduction status: docs captured; no external subagent provider run claimed.

## DSH-C013: DSH persistence is a capability seam over one canonical session event type

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/subsystems/persistence.md`, lines 1-19, 41-95, and 231-237
- Claim: DSH persists the same `SessionEvent` log through interchangeable backends, records metadata beside the log, batches flushes, and repairs interrupted cold sessions without inventing a parallel persisted event schema.
- Reproduction status: docs captured; local smoke does not simulate crash recovery.

## DSH-C014: DSH runtime packaging separates host, client, gateway, SDK, and generated contract faces

- Class: `SOURCE CLAIM`
- Source: `DEEPSEEK-HARNESS`
- Locator: `docs/api-gateway.md`, lines 7-17, 80-99, 119-130, and 158-164; root `package.json` scripts inspected in local smoke
- Claim: DSH separates Remote method declaration, Host generation, Client generation, gateway dispatch, connection transport, and SDK/runtime package faces rather than treating UI calls as informal JSON endpoints.
- Reproduction status: docs and package scripts captured; local smoke does not run a browser client.

## DSH-C015: Cordis HMR and dynamic composition are first-class design paths

- Class: `SOURCE CLAIM`
- Sources: `DEEPSEEK-HARNESS`, `CORDIS-PAPER`
- Locator: `docs/cordis-tutorial/06-composition-and-hmr.md`, lines 19-25, 50-60, and 61-109; Cordis paper lines 1420-1542
- Claim: Cordis config rows have stable identity, plugins can be disabled/reloaded, HMR unloads old effects and loads new code, and the paper formalizes fibers and target views for lifecycle transitions.
- Reproduction status: docs and paper captured; local smoke does not run a long-lived HMR process.

## DSH-C016: DSH is an inspectable harness substrate, not evidence of autonomous RSI

- Class: `INFERENCE`
- Sources: `DEEPSEEK-HARNESS`, `CORDIS-PAPER`, `DSH-SMOKE`
- Claim: DSH's design choices make candidate harness composition, capability replacement, replay, and policy interposition easier to inspect than a monolithic loop, but the captured evidence does not show an evaluator, archive, promotion controller, or held-out recursive improvement result.
- Evidence that would weaken: a DSH source or release receipt showing built-in candidate generation, evaluation, and promotion over harness changes.
- Evidence that would falsify: a pinned DSH experiment proving autonomous harness self-improvement under a reproducible evaluator.
