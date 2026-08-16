---
id: deepseek-harness-self-modification-and-dynamic-cordis
title: DeepSeek Harness decision 10 - Self-modification and dynamic Cordis
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, cordis, hmr, self-modification, dynamic-composition]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, CORDIS-PAPER, DSH-SMOKE]
---

# Decision 10: Self-modification and dynamic Cordis

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C015].** DSH's Cordis foundation treats dynamic
composition, config reconciliation, and hot module replacement as first-class
runtime design concerns. For Harp, the important reading is that DSH provides
mechanical affordances a self-modifying harness would need, but the captured
DSH evidence does not include an autonomous self-improvement loop.

## Evidence

**EVIDENCE - self-evolving harness motivation.** The Cordis paper explicitly
uses self-evolving agent harnesses as a motivating example, describing harnesses
that compose tools, permissions, sandboxing, session state, persistence,
context management, memory, subagents, and interfaces
([[evidence/cordis_paper/text/cordis-paper-v8.txt|Cordis paper text]]
([lines 188-207](../../evidence/cordis_paper/text/cordis-paper-v8.txt#L188))).

**EVIDENCE - coarse-grained workaround.** The paper argues that process
restart and container-level orchestration are too coarse for fine-grained
dynamic composition because they discard process-local state and cannot express
dependencies between components sharing an address space
([lines 209-233](../../evidence/cordis_paper/text/cordis-paper-v8.txt#L209)).

**EVIDENCE - HMR tutorial.** The DSH Cordis tutorial says stable config entry
IDs let the loader distinguish edits from remove/add cycles; HMR unloads an old
instance, unwinds effects, loads new code, and can diagnose PENDING plugins
with unsatisfied dependencies
([[evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/06-composition-and-hmr.md|composition and HMR]]
([lines 19-25](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/06-composition-and-hmr.md#L19),
[lines 50-60](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/06-composition-and-hmr.md#L50),
[lines 61-109](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/06-composition-and-hmr.md#L61))).

**EVIDENCE - formal lifecycle.** The paper formalizes components and fibers,
where a fiber has lifecycle state and transition rules move it toward a target
view derived from dependency satisfaction
([lines 1420-1542](../../evidence/cordis_paper/text/cordis-paper-v8.txt#L1420)).

## Why this matters

**INFERENCE - [DSH-C016].** Self-modification becomes less opaque when the
object being modified is a plugin row, a provider, a tool, a prompt section, or
a generated contract with an explicit lifecycle. DSH's Cordis substrate gives a
future harness optimizer something concrete to manipulate and something
concrete to roll back.

But dynamic composition is not autonomous improvement. A system still needs a
candidate generator, evaluator, archive, promotion policy, rollback protocol,
and safety boundary. DSH supplies many runtime affordances that such a system
would want; the pinned evidence does not show those affordances closed into an
RSI loop.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** The local smoke receipt checks source identity and
keyless static/demonstration surfaces. It does not run HMR, persist a
self-modified profile, or evaluate a candidate patch.

## Failure modes

**MISSING.** A stronger receipt would run a minimal Cordis HMR demo, record a
profile patch before/after dump, and demonstrate that a bad plugin can be
unloaded while dependents move to PENDING or recover. This packet stops at the
source-backed design reading.
