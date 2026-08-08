---
id: recursive-self-improvement-codex-state-continuity
title: Codex state continuity and compaction
type: technical-deep-dive
mode: TECHNICAL DEEP DIVE
status: active
created: 2026-08-02
updated: 2026-08-02
tags: [recursive-self-improvement, codex, arc-agi-3, responses-api, state-continuity, compaction]
confidence: medium
---

# Codex state continuity: preserving the agent's working representation

Mode: `TECHNICAL DEEP DIVE`.

> This companion extends [[codex_harness_deep_dive]]. The earlier document
> focuses on lineage, capabilities, permissions, and containment. This one
> focuses on active working-state continuity, compaction, replay, and the
> separate cross-thread memory pipeline.

## Anchor and provenance

**EVIDENCE — pinned code.** The implementation reading is pinned to
`openai/codex@1e85ca099e4265bf89f4016772d299816e231bb3` and
`arcprize/arc-agi-3-benchmarking@86d72170ce3155551712a9fafd290bab471d6eee`.
Both clean source trees were inspected locally; no upstream code was executed.

**EVIDENCE — official compaction guide.** The OpenAI compaction guide was
retrieved on 2026-08-02 through its redirect to
<https://developers.openai.com/api/docs/guides/compaction>. The 353,044-byte
response had SHA-256
`272289a464456803422a33ff5091dc3a860196dc0de3a00d2550cb0dad3eb3ee`.
Claim-level extraction is preserved in
`sources/codex-state-continuity-claims.tsv`.

**MISSING — blocked pages.** The OpenAI ARC result page and Responses feature
page returned HTTP 403 to direct acquisition in this environment. Their
quantitative and product claims below come from the user-provided packet and
are labeled `CLAIM`, not independently fetched `EVIDENCE`.

## Core thesis

1. **CLAIM — [OPENAI-ARC-ARTICLE].** OpenAI reports that the official ARC-AGI-3
   harness with GPT-5.6 Sol at maximum reasoning scored 13.3%, while a
   Responses-based harness with retained reasoning and compaction scored 38.3%
   and used roughly sixfold fewer output tokens. This is a state-continuity
   result, not a new ARC-solving algorithm.
2. **INFERENCE.** The two settings address different losses. Retained reasoning
   avoids reconstructing a latent task model after every action. Compaction
   avoids deleting early observations solely because they are old.
3. **INFERENCE.** The primary bottleneck is integrity of agent working state
   over long horizons. Repeated reconstruction inflates decode work; evidence
   loss creates policy regression and repeated exploration.
4. **EVIDENCE — [ARC-AGI3-BENCH].** The public server-state path reproduces the
   article's core mechanism, but it is much smaller than the complete Codex
   harness.
5. **EVIDENCE — [CODEX-REPO].** Codex keeps several distinct state layers:
   opaque reasoning items, typed thread history, recomputed world state,
   durable rollout checkpoints, and asynchronously consolidated cross-thread
   memory.
6. **INFERENCE.** `previous_response_id` is a lineage pointer and wire-delta
   optimization. It is not semantic memory, a vector store, or proof of a
   reusable server KV cache.
7. **INFERENCE.** A harness should preserve the representation on which the
   model was trained to continue. Discarding opaque reasoning or trained prompt
   order can make the same weights behave like a weaker deployed system.
8. **MISSING.** The fastest causal falsifier is a matched 2×2 ablation:
   reasoning retention on/off × compaction on/off.

## Mental model

### Thread, turn, and sampling step

A **thread** is the durable lineage. A **turn** is one user request. A
**sampling step** is one model invocation inside a turn. Tool calls create new
evidence and can require several sampling steps before a final answer.

The ARC loop has the same shape:

```text
observe frame -> reason -> take action -> observe next frame
```

The decisive question is which state crosses each arrow.

### State decomposition

Represent the step state as:

`Xₜ = (Hₜ, Wₜ, Bₜ, Rₜ, Mₜ, Eₜ)`

- `Hₜ`: ordered model-visible `ResponseItem` history;
- `Wₜ`: current world state, including model, cwd, permissions, instructions,
  tools, plugins, and collaboration state;
- `Bₜ`: persisted world-state baseline used for diffs;
- `Rₜ`: durable rollout, compaction checkpoints, and replay metadata;
- `Mₜ`: optional consolidated cross-thread memory; and
- `Eₜ`: live external execution state such as processes or remote sessions.

The prompt is conceptually:

`Pₜ = I_base + render(Wₜ | Bₜ) + normalize(Hₜ) + Uₜ`

where `Uₜ` is the new user or tool input. `Rₜ` can reconstruct logical model
state; it does not automatically recreate `Eₜ`.

## ARC baseline: two independent losses

### Visible transcript without latent working state

**EVIDENCE — [ARC-AGI3-BENCH],
`benchmarking/agent.py:32-49,547-583`.** The ordinary agent appends a rendered
frame and a visible assistant action/reasoning message to a Python list.

**INFERENCE.** Retaining the selected action does not preserve the model's
opaque belief state: object hypotheses, rejected explanations, long-range
plan, and uncertainty frontier. The next request may pay output tokens to
derive them again.

### Chronological trimming without semantic importance

**EVIDENCE — [ARC-AGI3-BENCH],
`benchmarking/agent.py:622-641,707-728`.** Once the estimated context exceeds
the configured limit, the harness removes the oldest user message and adjacent
assistant response while preserving the system prompt and current turn. The
public configuration repeatedly uses a 175,000-token limit, and the estimator
uses roughly one rendered-grid character per token.

**INFERENCE.** The trimming policy cannot distinguish a decisive game rule from
boilerplate. It can delete failed hypotheses worth retaining, the transition
that unlocked a later level, or the evidence that justified the current plan.

## Optimized ARC path

**EVIDENCE — [ARC-AGI3-BENCH],
`benchmarking/agent.py:67-75,435-454,547-583`.** The first request sends the
system prompt and frame. Later requests send only pending observations plus the
last successful `previous_response_id`. Pending observations are cleared only
after a successful response ID is installed. The local conversation remains a
recording mirror, and local trimming is disabled.

**EVIDENCE — [ARC-AGI3-BENCH],
`benchmarking/runtime_adapters.py:69-114`.** The adapter defaults `store=true`
and translates `compact_threshold` into a `context_management` compaction
request.

The state transition is therefore closer to:

`Sₜ₊₁ = Responses(Sₜ, Oₜ₊₁; retain reasoning, compact at threshold)`

than:

`Sₜ₊₁ = newest chronological tail of visible messages`.

## Opaque reasoning is not displayed chain of thought

**EVIDENCE — [CODEX-REPO], `codex-rs/core/src/client.rs:860-929`.** Codex asks
the Responses API to include `reasoning.encrypted_content`.

The protocol separates:

- a human-facing reasoning summary;
- opaque encrypted reasoning state for continuation; and
- visible assistant messages, tool calls, commentary, and final answers.

**INFERENCE.** The client should round-trip opaque state without decrypting,
summarizing, or exposing it. Preserving a model-usable continuation object is a
different operation from storing readable chain of thought.

## Canonical typed history

**EVIDENCE — [CODEX-REPO],
`codex-rs/core/src/context_manager/history.rs:39-208`.** `ContextManager`
contains oldest-to-newest `ResponseItem` history, a history version, token
accounting, reference context, and world-state baseline. Prompt construction
normalizes the typed sequence, strips unsupported media, truncates tool output
by policy, and preserves function-call/output invariants.

The transcript is protocol state, not an append-only text blob:

```text
developer context
user request
reasoning item
function calls
tool outputs
assistant message
compaction item
```

Removing arbitrary entries can orphan a call, attach an output to the wrong
call ID, or leave a context diff without the baseline it was computed against.

## Logical request versus wire request

**EVIDENCE — [CODEX-REPO], `codex-rs/core/src/client.rs:1612-1680`.** The
WebSocket path sends `previous_response_id` and an incremental suffix only when
the logical request is a valid extension and non-input request properties still
match. Otherwise it sends the full logical input. Tracing retains the logical
request even when the wire payload is a delta.

```text
Canonical truth: local typed transcript
Transport optimization: previous_response_id + suffix
Recovery: resend full logical transcript
```

**MISSING.** Client source does not establish whether the service keeps live KV
state, reconstructs stored items, hits a prefix cache, or combines those
mechanisms. Measure cached input tokens and time-to-first-token rather than
inferring KV reuse from the API shape.

## World state is recomputed truth

**EVIDENCE — [CODEX-REPO],
`codex-rs/core/src/context_manager/history.rs:92-113,190-208` and
`codex-rs/core/src/context/world_state/`.** Codex builds typed sections for
current runtime state and renders a full snapshot or a merge-patch-style diff
against a baseline. Replacing history clears that baseline.

Some facts should be recomputed rather than remembered:

- current directory and environment;
- model and model-specific instructions;
- permissions and approval policy;
- `AGENTS.md`;
- tools, plugins, apps, and collaboration mode; and
- active multi-agent state.

This prevents stale transcript claims from outranking current runtime truth.

## Compaction is an atomic state transition

Compaction changes both active history and its context baseline:

`(Hₜ, Bₜ) -> (H'ₜ, B'ₜ)`.

### Local textual compaction

**EVIDENCE — [CODEX-REPO], `codex-rs/core/src/compact.rs:55-84,348-393`.**
The local path asks a model for a summary, retains up to roughly 20,000 tokens
of recent real user messages, installs replacement history, updates or clears
context baselines, and warns that repeated compactions can reduce accuracy.

This path is lossy: old reasoning, calls, outputs, and actions survive only if
the textual summary preserves their relevant meaning.

### Remote compact endpoint

**EVIDENCE — [OPENAI-COMPACTION-DOC].** The official guide describes an opaque
encrypted compaction item that carries prior state and reasoning in a smaller
window. The complete returned window should be passed forward; a
`previous_response_id`-managed chain should not be manually pruned.

### Remote compaction v2

**EVIDENCE — [CODEX-REPO],
`codex-rs/core/src/compact_remote_v2.rs:56-62,442-514`.** At the pinned
revision, Codex retains selected user/developer/system messages and bounded
agent messages under a 64,000-token retained-message budget, caps an individual
retained agent message at 10,000 tokens, preserves input images, excludes final
answers, and appends exactly one opaque compaction output.

### Durable installation

**EVIDENCE — [CODEX-REPO], `codex-rs/core/src/compact.rs:76-84`.** The
replacement history and compaction metadata are persisted together after item
IDs are assigned, so the live model state and durable checkpoint have the same
authoritative history.

## Resume and replay

**EVIDENCE — [CODEX-REPO],
`codex-rs/core/src/session/rollout_reconstruction.rs:112-187,286-294`.**
Reconstruction scans newest-to-oldest until it finds the newest surviving
replacement-history checkpoint and required metadata. It then replays only the
surviving suffix forward.

`H_resume = H_latest-checkpoint + replay(surviving tail)`.

Older events before the selected replacement checkpoint cannot modify the
rebuilt model history. External side effects remain a separate problem:
processes, deployments, payments, and remote writes need idempotency keys,
operation IDs, or reconciliation.

## Cross-thread memory is orthogonal

**EVIDENCE — [CODEX-REPO],
`codex-rs/memories/write/src/phase1.rs:50-108,149-220`.** Phase one leases
eligible rollout jobs and extracts a detailed raw memory, compact routing
summary, and optional slug with bounded concurrency and retries.

**EVIDENCE — [CODEX-REPO],
`codex-rs/memories/write/src/phase2.rs:46-220`.** Phase two claims a global
lock, syncs selected memories into a git-backed workspace, writes the workspace
diff, and launches a constrained consolidation agent.

The resulting hierarchy is:

| Layer | Scope | Role |
|---|---|---|
| Encrypted reasoning | Active response lineage | Opaque within-thread cognitive continuity |
| `ResponseItem` history | Active thread | Ordered episodic and tool-protocol state |
| Compaction checkpoint | Context epoch | Bounded continuation state |
| Rollout | Durable thread | Replay, fork, rollback, and checkpoint source |
| World state | Current runtime | Recomputed configuration and environment truth |
| Memories workspace | Cross-thread | Selectively retrieved reusable knowledge |

No one object should be labeled simply “agent memory.”

## Performance model

For sampling step `s`, let:

- `Cₛ` be logical input tokens and `C_cached,ₛ` cached input tokens;
- `Mₛ` be serialized wire bytes;
- `Rₛ` be newly generated reasoning tokens;
- `Oₛ` be visible output and tool-call tokens;
- `Dₛ,ⱼ` be tool latency; and
- `T_compact,c` be compaction latency.

A practical decomposition is:

`T_total ≈ Σₛ[T_wire(Mₛ) + T_prefill(Cₛ, C_cached,ₛ) + T_decode(Rₛ + Oₛ) + T_tools,ₛ + T_harness,ₛ] + Σc T_compact,c`.

For a parallelizable tool batch, `T_tools,ₛ ≈ maxⱼ Dₛ,ⱼ`; for serial calls it is
approximately the sum.

Retained reasoning may carry more input state while reducing newly decoded
reasoning and the number of required actions:

`C_retain > C_drop`, `R_retain << R_drop`, and possibly
`steps_retain < steps_drop`.

With a valid server lineage, wire size can scale with the new suffix rather
than the full logical prompt. This does not imply that transformer prefill also
scales only with the suffix.

If compaction threshold is `H`, post-compaction context is `C₀`, and average
growth is `g` tokens per action, then actions between compactions are roughly
`K ≈ (H - C₀) / g`, with amortized overhead
`T_compact / K`. Low thresholds over-compact; high thresholds increase prefill
and overflow risk.

## Comparison boundaries

- **Stateless chat:** portable visible text, but no native opaque reasoning
  continuation.
- **Sliding window:** deterministic bounded size, but age substitutes for
  importance and can create abrupt evidence-loss cliffs.
- **Summary-only continuation:** simpler than Codex, but lacks typed item
  invariants, world-state baselines, exact replacement checkpoints, and replay.
- **Vector or RAG memory:** useful for cross-session retrieval, not as the
  primary representation of exact ordered calls, outputs, observations, and
  rejected hypotheses.
- **KV cache:** a serving optimization, not durable semantic state, crash
  recovery, or auditable replay.
- **Temporal-style durability:** makes external activities, retries, timers, and
  side effects first-class. Codex rollout durability primarily protects logical
  model-visible state.

## Experimental validation

### Priority 1: matched 2×2 causal ablation

Run reasoning retention off/on × rolling truncation/compaction under identical
games, prompts, model settings, seeds, action budgets, and retry policy.

Measure score, wins, output and reasoning tokens, actions, sampling steps,
wall-clock time, and cost. The expected signature is:

- retention primarily reduces re-derivation and output per action;
- compaction primarily protects late-horizon behavior; and
- the combined cell shows whether the interventions interact.

### Priority 2: early-rule retention

Show one decisive rule near the beginning of a partially observable synthetic
game. Measure recall and optimal-action probability as that observation crosses
the rolling-window boundary.

### Priority 3: re-derivation

Use a fixed observation stream that requires a multi-step latent plan. Compare
repeated hypotheses, repeated experiments, semantic similarity of reasoning
summaries, and output tokens per action.

### Priority 4: compaction and threshold sweeps

Compare head truncation, tail window, local summary, standalone remote compact,
and remote v2. Sweep thresholds and repeated compaction counts. Plant facts and
rejected hypotheses at different salience levels.

### Priority 5: wire versus logical state

Compare full logical requests with incremental server lineage. Measure request
bytes, serialization CPU, cached tokens, time-to-first-token, and fallback
rate. Do not infer service-side KV behavior from request shape.

### Priority 6: world-state and replay boundaries

Mutate cwd, instructions, permissions, model, and tools between turns. Kill the
process before and after response persistence, tool-output persistence, and
compaction installation. Compare normalized prompt hashes with a non-crashed
control and separately reconcile external side effects.

## Missing evidence

- **MISSING — contribution split.** Retention-only and compaction-only ARC
  scores are absent from the supplied article analysis.
- **MISSING — 7.8% versus 13.3%.** The supplied packet notes two baseline
  framings without enough run-population or aggregation detail to reconcile
  them.
- **MISSING — exact GPT-5.6 Sol run artifact.** The inspected public benchmark
  config does not identify the complete article run configuration.
- **MISSING — uncertainty.** Run counts, per-game results, and confidence
  intervals are not available in this packet.
- **MISSING — service semantics.** Encrypted reasoning representation and
  server-side KV or prefix-cache behavior remain provider-internal.
- **MISSING — semantic fidelity.** Compaction mechanics do not guarantee that a
  rare decisive fact or rejected hypothesis survives.
- **MISSING — cross-model portability.** Opaque checkpoints should not be
  assumed portable across incompatible model representations.
- **MISSING — external effects.** Logical rollout replay does not prove whether
  a non-idempotent external operation completed before a crash.

## RSI interpretation

**INFERENCE.** State continuity is a harness capability. It can make fixed
weights substantially more effective and make long-running improvement work
more efficient. It is not itself recursive improvement.

For RSI, continuity becomes foundational evidence only when it preserves the
state needed to propose, evaluate, reject, promote, and later improve a
successor under a protected envelope. The 2×2 ARC ablation tests immediate
harness causality. A separate later-generation experiment must still test
whether the accepted state-management policy improves future improvement work.
