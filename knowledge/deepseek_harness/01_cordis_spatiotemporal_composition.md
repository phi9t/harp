---
id: deepseek-harness-cordis-spatiotemporal-composition
title: DeepSeek Harness decision 01 - Cordis spatiotemporal composition
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, cordis, effects, coeffects, dynamic-composition]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, CORDIS-PAPER, DSH-SMOKE]
---

# Decision 01: Cordis spatiotemporal composition

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C001], [DSH-C002].** DSH chooses Cordis as the
composition substrate and describes the product as an agent harness where
everything is a plugin. The important design decision is not just "use a plugin
framework"; it is to make plugin lifecycle, dependency resolution, and effect
cleanup the foundation below model adapters, tools, sessions, sandbox policy,
and the agent loop.

## Evidence

**EVIDENCE - DSH README.** The DSH README says DSH is an open-source DeepSeek
AI agent harness and that it uses an architecture where "everything is a
plugin", powered by Cordis
([[evidence/implementations/deepseek_harness/snapshot/README.md|README]]
([lines 5-11](../../evidence/implementations/deepseek_harness/snapshot/README.md#L5))).

**EVIDENCE - DSH architecture.** The DSH architecture guide says plugins
contribute services, typed events, and reversible effects to a shared context;
model adapter, tool registry, session log, and agent loop are all plugins
([[evidence/implementations/deepseek_harness/snapshot/docs/architecture.md|architecture]]
([lines 9-14](../../evidence/implementations/deepseek_harness/snapshot/docs/architecture.md#L9))).

**EVIDENCE - Cordis paper.** The Cordis paper identifies temporal
composability as the ability to reverse a component's side effects upon
removal and spatial composability as the ability to declare and reactively
manage dependencies
([[evidence/cordis_paper/text/cordis-paper-v8.txt|Cordis paper text]]
([lines 13-27](../../evidence/cordis_paper/text/cordis-paper-v8.txt#L13),
[lines 132-148](../../evidence/cordis_paper/text/cordis-paper-v8.txt#L132))).

## Why this matters

**INFERENCE - [DSH-C016].** An agent harness is a bad place for ad hoc global
state. Tools, sandboxes, persistence, memory, subagents, and UI clients all
want to evolve independently, but they also need shared state, shared policy,
and a clean rollback story. Cordis gives DSH a vocabulary for those needs:
services for what a component reads, effects for what it contributes, and
fibers for the lifetime of one component instance.

For Harp's RSI model, this is useful because candidate harness changes can be
framed as composition changes instead of source edits to a monolithic loop.
That does not prove recursive self-improvement. It only lowers the mechanical
cost of making, isolating, unloading, and inspecting harness changes.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** The local smoke receipt verifies the pinned commit,
package metadata, script surface, and keyless docs/examples used by this
packet. It does not run a credentialed model adapter.

## Failure modes

**MISSING.** The packet does not prove that every DSH plugin is correctly
effect-scoped, that every disposer is complete, or that future DSH versions
preserve this architecture. Those would require deeper product tests across
HMR, provider unload, session persistence, and UI state.
