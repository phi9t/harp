---
id: deepseek-harness-events-effects-and-reversible-lifecycle
title: DeepSeek Harness decision 03 - Events, effects, and reversible lifecycle
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, cordis, events, effects, lifecycle]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, CORDIS-PAPER, DSH-SMOKE]
---

# Decision 03: Events, effects, and reversible lifecycle

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C004], [DSH-C005].** DSH chooses Cordis effects and typed
events as the main extension mechanism. Registrations install behavior into a
context and return disposal paths; events provide typed observation,
interception, parallel fanout, or serial coordination without importing the
concrete loop.

## Evidence

**EVIDENCE - Cordis primer.** The DSH primer says a plugin claims services
through stable `ctx.<key>` names, declares dependencies with `inject`, and uses
typed events with `emit`, `waterfall`, `parallel`, or `serial` dispatch modes
([[evidence/implementations/deepseek_harness/snapshot/docs/cordis-primer.md|primer]]
([lines 7-14](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-primer.md#L7),
[lines 15-34](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-primer.md#L15))).

**EVIDENCE - lifecycle tutorial.** The lifecycle tutorial says a Cordis plugin
can unload by config edit, hot reload, explicit disposal, or loss of a required
service; registrations through Cordis APIs are effects and are undone when the
owning plugin unloads
([[evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/02-lifecycle-and-effects.md|lifecycle tutorial]]
([lines 5-6](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/02-lifecycle-and-effects.md#L5),
[lines 84-94](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/02-lifecycle-and-effects.md#L84))).

**EVIDENCE - architecture domains.** DSH distinguishes durable session events,
live agent events, and capability events as separate extension domains
([[evidence/implementations/deepseek_harness/snapshot/docs/architecture.md|architecture]]
([lines 53-61](../../evidence/implementations/deepseek_harness/snapshot/docs/architecture.md#L53))).

**EVIDENCE - formal basis.** The Cordis paper models an effect as a context
transformation paired with an inverse, then tracks inverses so the environment
can be recovered on removal
([[evidence/cordis_paper/text/cordis-paper-v8.txt|Cordis paper text]]
([lines 388-395](../../evidence/cordis_paper/text/cordis-paper-v8.txt#L388),
[lines 548-600](../../evidence/cordis_paper/text/cordis-paper-v8.txt#L548))).

## Why this matters

**INFERENCE.** In an agent harness, the extension mechanism is also the
authority mechanism. A tool policy listener, a prompt section, a persistence
writer, and a UI projection should not all patch a central loop. DSH's event
domains let a plugin attach to the boundary it actually owns: durable facts
go to the session log; live request interception goes to `agent/*`; capability
policy goes to the capability seam.

The cost is that dispatch modes become part of the public contract. A
waterfall listener can veto or replace a result; an emit listener should not
be treated as a decision point. This is a real design burden, but the burden
is visible in generated docs rather than hidden in callback convention.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** Keyless local inspection can confirm that the docs
and package scripts expose event catalogs and generated Cordis surface checks.
It does not prove every generated event catalog entry matches runtime behavior.

## Failure modes

**MISSING.** Harp has not run a plugin unload stress test. The evidence supports
the design decision and API contract, not a full proof that every first-party
plugin's cleanup is complete under asynchronous teardown.
