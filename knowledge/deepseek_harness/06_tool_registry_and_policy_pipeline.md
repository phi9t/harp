---
id: deepseek-harness-tool-registry-and-policy-pipeline
title: DeepSeek Harness decision 06 - Tool registry and policy pipeline
type: technical-deep-dive
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, tools, schemas, policy, tool-execution]
confidence: medium
source_ids: [DEEPSEEK-HARNESS, DSH-SMOKE]
---

# Decision 06: Tool registry and policy pipeline

Mode: `DESIGN DECISION DEEP DIVE`.

## Decision

**SOURCE CLAIM - [DSH-C009], [DSH-C010].** DSH separates tool visibility from
tool authority. The model receives an allowlisted tool schema; host-only
execution metadata, policy decisions, concurrency metadata, presentation, and
canonical output validation stay on the host side.

## Evidence

**EVIDENCE - tool definition.** The tools doc states that a registered tool
contains model-facing schema fields plus mandatory output, execute function,
host-only scheduler metadata, optional final content, and UI presenters; the
registry's `schemas()` builds model-facing `ToolSchema[]` by explicit allowlist
([[evidence/implementations/deepseek_harness/snapshot/docs/subsystems/tools.md|tools docs]]
([lines 9-12](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/tools.md#L9))).

**EVIDENCE - schema discipline.** The same doc describes one schema DSL for
parameters and output values, exact-one unions, explicit object openness,
bounded TypeScript inference, runtime validation, and rejection of unsupported
raw JSON Schema keywords
([lines 96-151](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/tools.md#L96)).

**EVIDENCE - pipeline.** Tool execution materializes a `ToolExecution`, then
runs `tools/pre-execute`, monotonic guards, `tools/execute`,
`tools/post-execute`, optional `finalizeContent`, and `tools/result`
([lines 170-172](../../evidence/implementations/deepseek_harness/snapshot/docs/subsystems/tools.md#L170)).

**EVIDENCE - keyless tutorial.** The harness tutorial registers a `greet` tool,
executes it through `ctx.tools.execute()`, and observes `tools/result` without
calling a model
([[evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/07-into-the-harness.md|into the harness]]
([lines 5-49](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/07-into-the-harness.md#L5),
[lines 51-94](../../evidence/implementations/deepseek_harness/snapshot/docs/cordis-tutorial/07-into-the-harness.md#L51))).

## Why this matters

**INFERENCE.** Tool APIs are where agent harnesses most often collapse
interfaces: a JSON schema, a callable function, a UI card, a policy decision,
and a durable result get treated as the same object. DSH deliberately keeps
those roles distinct. That gives a reviewer several independent questions:
what did the model see, what did the host execute, what policy admitted it,
what canonical value was recorded, and what presentation was replayed?

For RSI-oriented harness work, this is evaluator-protective. A candidate can
be allowed to propose or use a tool without gaining authority to decide that
the tool should run. Policy listeners and monotonic guards remain outside the
model-visible schema.

## Verification reading

**EVIDENCE - [DSH-SMOKE].** The local smoke receipt focuses on keyless
mechanics and source surfaces. It can support the existence of the no-key
tutorial path, but not a claim that every production tool respects
cancellation, output schemas, or concurrency promises.

## Failure modes

**MISSING.** The current packet does not include a bad-args run, invalid-output
run, denial run, or parallel/exclusive scheduling run. Those would be needed
to verify the policy pipeline behavior empirically.
