---
id: deepseek-harness-capability-seams
title: DeepSeek Harness decision 05 - Capability seams
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, capability-seams, services, providers]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, DSH-SMOKE]
---

# Decision 05: Capability seams

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C008].** DSH treats a swappable capability as a seam
with three roles: a service definition, one or more service providers, and one
or more consumers. A package can combine roles, but the seam is the whole
contract, not merely a TypeScript interface.

## Evidence

**EVIDENCE - architecture definition.** The architecture guide defines a seam
as a swappable capability with service-definition, provider, and consumer
roles, and states that adding a capability means designing all three
([[evidence/implementations/deepseek_harness/snapshot/docs/architecture.md|architecture]]
([lines 98-103](../../evidence/implementations/deepseek_harness/snapshot/docs/architecture.md#L98))).

**EVIDENCE - generated graph.** The capability graph lists services such as
`ctx.llm`, `ctx.tools`, `ctx.sessions`, `ctx.shell`, `ctx.sandbox`,
`ctx.subagents`, `ctx.sessionPersistence`, and their providers/consumers
([[evidence/implementations/deepseek_harness/snapshot/docs/capability-seams.md|capability graph]]
([lines 4-7](../../evidence/implementations/deepseek_harness/snapshot/docs/capability-seams.md#L4),
[service list lines 74-155](../../evidence/implementations/deepseek_harness/snapshot/docs/capability-seams.md#L74))).

**EVIDENCE - extension map.** The architecture guide maps concrete goals to
extension mechanisms: register an LLM adapter on `ctx.llm`, model-facing
capability on `ctx.tools`, shell backend on `ctx.shell`, filesystem provider
or `fs/*` listener for filesystem access, and sandbox backend on `ctx.sandbox`
([lines 104-128](../../evidence/implementations/deepseek_harness/snapshot/docs/architecture.md#L104)).

## Why this matters

**INFERENCE.** Capability seams are the strongest practical expression of the
Cordis choice. They let DSH swap a filesystem, shell, process sandbox,
subagent provider, or persistence backend without rewriting the model loop.
For a harness-evolution system, that is the difference between changing
behavior by editing control flow and changing behavior by replacing a bounded
provider.

This separation also keeps authority visible. A model-facing tool can consume
a filesystem seam without owning the filesystem backend. A shell tool can
consume a sandboxed shell provider without knowing the platform runner. That
does not eliminate bugs, but it makes the review question sharper: did the
candidate change a service definition, a provider, or a consumer?

## Verification reading

**EVIDENCE - [DSH-SMOKE].** Local checks inspect package surfaces and generated
documentation. They do not instantiate every provider combination or prove
that provider swaps are behavior-preserving.

## Failure modes

**MISSING.** A future receipt should boot two profiles with different providers
for the same seam and compare tool-visible behavior. The current packet
supports the seam design, not provider-equivalence claims.
