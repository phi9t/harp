# Agent-harness architecture dossier: research questions from SICP

This dossier compares bounded, pinned source evidence from Pi, Hermes Agent,
and OpenAI Codex through concepts taught by SICP. It is an architecture reading,
not a claim of historical influence, semantic equivalence, benchmark parity, or
one approved implementation design.

## Evidence boundary

The columns below are deliberately different voices. A SICP claim says what the
book establishes. Pinned harness evidence says what a tracked public-source
snapshot shows. A course inference transfers a design question without
claiming historical influence or semantic identity. A research hypothesis is
testable future work, not a decision. Missing evidence remains `MISSING`.

| SICP claim | pinned harness evidence | course inference | research hypothesis | missing evidence |
|---|---|---|---|---|
| `CLAIM` — abstraction barriers separate a client's contract from representation choices. [§§2.1.1–2.1.2, printed pp. 113–121 / PDF pp. 141–149](../../../../../evidence/sicp/sicp.pdf#page=141) [Seminar 2](../seminars/02-data-abstraction-and-immutable-representation.md#agent-harness-bridge) | `EVIDENCE` — pinned Codex keeps model-visible specifications separate from registry dispatch; pinned Hermes allows at most one external provider, rejects reserved core tool names, and ignores already-routed duplicate tool names. [Codex router](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289) [Hermes memory registry](../../../../../evidence/implementations/hermes/snapshot/agent/memory_manager.py#L364-L464) | `INFERENCE` — visibility, normalized identity, resolution, and authority should be separate records. | `HYPOTHESIS` — that separation can remain usable without excessive ceremony. | `MISSING` — end-to-end proof that every tool path in any compared harness enforces all four boundaries. |
| `CLAIM` — data-directed dispatch resolves behavior through explicit operation/type keys. [§2.4.3, printed pp. 242–248 / PDF pp. 270–276](../../../../../evidence/sicp/sicp.pdf#page=270) [Seminar 4](../seminars/04-symbols-sets-compression-and-generic-dispatch.md#agent-harness-bridge) | `EVIDENCE` — Codex constructs a canonical tool name before registry dispatch; Hermes repairs and validates a name before execution. [Codex router](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289) [Hermes conversation loop](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L5700-L6165) | `INFERENCE` — discovery and normalization can be additive while execution remains runtime-controlled. | `HYPOTHESIS` — a small capability resolver can expose these stages in experiment receipts. | `MISSING` — a common capability identity or authorization vocabulary across Pi, Hermes, and Codex. |
| `CLAIM` — retained state and environments make history, identity, and scope operational. [§§3.1.1 and 3.2.3, printed pp. 297–305 and 330–337 / PDF pp. 325–333 and 358–365](../../../../../evidence/sicp/sicp.pdf#page=325) [Seminar 5](../seminars/05-state-identity-and-environments.md#agent-harness-bridge) | `EVIDENCE` — Pi constructs turn state, Codex records model-visible step state, and Hermes has provider-mediated memory lifecycle operations. [Pi turn state](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/agent-harness.ts#L395-L429) [Codex turn](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L328) [Hermes memory](../../../../../evidence/implementations/hermes/snapshot/agent/memory_manager.py#L364-L735) | `INFERENCE` — turn snapshot, session history, cross-session memory, and external world state are different contracts. | `HYPOTHESIS` — an event log plus derived projection is enough to reproduce small-agent context experiments. | `MISSING` — a verified physical-durability and replay contract for all cited session/history paths. |
| `CLAIM` — mutation makes ordering and identity part of the result. [§3.3.4, printed pp. 369–386 / PDF pp. 397–414](../../../../../evidence/sicp/sicp.pdf#page=397) [Seminar 6](../seminars/06-mutation-simulation-and-constraints.md#agent-harness-bridge) | `EVIDENCE` — Hermes gates a configured database path on a pre-tool flush; Pi advances its in-memory leaf only after appending the corresponding session entry. [Hermes gate](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L6058-L6104) [Pi session](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L333-L515) | `INFERENCE` — proposal, durable admission, effect attempt, and receipt are distinct states. | `HYPOTHESIS` — freezing effect and uncertain-outcome semantics is necessary before comparing harness policies. | `MISSING` — proof that every external effect is idempotent, transactional, or reconcilable. |
| `CLAIM` — concurrent correctness depends on controlled interleavings and resource-acquisition discipline. [§§3.4.1–3.4.2, printed pp. 403–426 / PDF pp. 431–454](../../../../../evidence/sicp/sicp.pdf#page=431) [Seminar 7](../seminars/07-concurrency-serialization-and-interleavings.md#agent-harness-bridge) | `EVIDENCE` — the cited Codex paths expose tree scope, reservations, parent metadata, and targeted interrupt; Pi and Hermes examples bound fan-out. [Codex control](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control.rs#L90-L453) [Pi subagents](../../../../../evidence/implementations/pi/snapshot/packages/coding-agent/examples/extensions/subagent/README.md#L91-L175) [Hermes delegation](../../../../../evidence/implementations/hermes/snapshot/tools/delegate_tool.py#L2778-L2985) | `INFERENCE` — capacity, cancellation policy, lineage, and budgets need separately owned invariants. | `HYPOTHESIS` — matched budgets and explicit lineage make policy comparisons more credible. | `MISSING` — a shared durable-lineage guarantee, fairness contract, or recursive-cancellation guarantee. |
| `CLAIM` — streams separate production, demand, and retention; memoization is another policy. [§§3.5.1–3.5.2, printed pp. 430–453 / PDF pp. 458–481](../../../../../evidence/sicp/sicp.pdf#page=458) [Seminar 8](../seminars/08-streams-delay-and-infinite-processes.md#agent-harness-bridge) | `EVIDENCE` — Pi emits incremental model events and projects selected stored history; Codex makes compaction an explicit turn transition. [Pi loop](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) [Pi session projection](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L61-L149) [Codex turn](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L418-L457) | `INFERENCE` — production, consumption, retention, and context replacement require different policies. | `HYPOTHESIS` — compaction can replace a model projection while canonical audit events remain addressable. | `MISSING` — evidence that any compacted summary is lossless or sufficient as an effect receipt. |
| `CLAIM` — `eval` interprets expression data in an environment; `apply` invokes a procedure value on argument values. [§§4.1–4.1.1, printed pp. 492–498 / PDF pp. 520–526](../../../../../evidence/sicp/sicp.pdf#page=520) [Seminar 9](../seminars/09-eval-apply-and-executable-semantics.md#agent-harness-bridge) | `EVIDENCE` — Codex response-item handling precedes canonical tool-name resolution and registry invocation. [Codex turn](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L520) [Codex router](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289) | `INFERENCE` — structured intent and capability invocation should remain distinguishable; an LLM is not `eval`, and a tool runtime is not Scheme `apply`. | `HYPOTHESIS` — explicit interpreting and dispatching states improve fault localization. | `MISSING` — a formal semantics proving equivalence between a harness loop and SICP's evaluator; no such equivalence is claimed. |
| `CLAIM` — lazy demand and nondeterministic search require explicit forcing, choice, continuation, and rollback policies. [§§4.2.1–4.3.3, printed pp. 542–592 / PDF pp. 570–620](../../../../../evidence/sicp/sicp.pdf#page=570) [Seminar 10](../seminars/10-lazy-evaluation-and-nondeterministic-search.md#agent-harness-bridge) | `EVIDENCE` — Pi records parent-linked session entries and selected-path projection; Codex materializes a filtered fork context with parent identity. [Pi session](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L333-L515) [Codex fork](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control/spawn.rs#L570-L826) | `INFERENCE` — shared prefixes and attributable lineage do not define search order, scoring, merging, or effect rollback. | `HYPOTHESIS` — explicit branch lineage and budgets permit fairer candidate-search experiments. | `MISSING` — verified isolation or compensation for external effects across candidate branches. |
| `CLAIM` — explicit-control machines expose live state, transfers, storage obligations, and terminal paths. [§§5.4–5.5.1, printed pp. 741–778 / PDF pp. 769–806](../../../../../evidence/sicp/sicp.pdf#page=769) [Seminar 12](../seminars/12-machines-storage-control-and-compilation.md#agent-harness-bridge) | `EVIDENCE` — pinned Pi, Hermes, and Codex each expose concrete turn-loop branches. [Pi loop](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) [Hermes loop](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L1084-L1325) [Codex turn](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L520) | `INFERENCE` — a research harness can name intermediate obligations without pretending those names are shared by existing systems. | `HYPOTHESIS` — explicit transitions improve diagnosis relative to one opaque recursive loop. | `MISSING` — a cross-system state-transition trace collected under one workload and instrumentation contract. |

The Pi, Hermes, and Codex identities used here are fixed by the tracked
[harness revision ledger](../../sources/harness_revisions.tsv), and the local
checkouts are verified by the tracked
[materializer](../../sources/materialize_harness_sources.sh). Three tracked
repository-local harness deep dives are useful explanatory navigation:

- [Pi harness deep dive](../../../pi_harness_deep_dive.md)
- [Hermes harness deep dive](../../../hermes_harness_deep_dive.md)
- [Codex harness deep dive](../../../codex_harness_deep_dive.md)

Their tracked [promotion and pin manifest](../../../harness_deep_dive_manifest.tsv)
records each document's promoted digest, repository identity, declared
revision, and exact-match status. They remain explanatory supplements rather
than pin authority: every `EVIDENCE` cell below links the materialized source or
a tracked public-source snapshot directly.

## Minimal conceptual state model

These names are conceptual components. They are not Rust modules, structs,
traits, public API names, database tables, or promises that any implementation will adopt
this vocabulary.

The complete conceptual vocabulary is `TurnInput`, `TurnSnapshot`,
`ModelRequest/ModelEvent`, `StructuredAction`, `VisibleToolSpec`,
`ResolvedCapability`, `ActionIntent/ActionReceipt`,
`SessionEvent/ContextProjection`, `ChildLineage`, `Budget`, and
`TerminalOutcome`.

| Concept | Representation | Creator authority | Immutability or mutation status | Downstream consumer |
|---|---|---|---|---|
| `TurnInput` | An admitted user, operator, or queued control input with identity and provenance. | The session admission boundary; model text cannot create an admitted external input. | Immutable after admission; correction creates another input. | `TurnSnapshot` construction and the ready-to-snapshotting transition. |
| `TurnSnapshot` | One model-facing capture of selected messages, active configuration, visible tools, resource context, and projection identity. | The harness context builder under operator policy. | Immutable for the sampling step; a later step gets a new snapshot. | `ModelRequest`, compaction attribution, and replay comparison. |
| `ModelRequest` / `ModelEvent` | A fixed request plus append-only incremental response/lifecycle observations. | The sampler creates the request record; the provider adapter records events it actually observes. | Request immutable; events immutable and append-only. Partial assembly is a projection, not event mutation. | The interpreter, live UI/observer, usage accounting, and terminal decision. |
| `StructuredAction` | A parsed, schema-checked action proposal derived from one or more model events. | The protocol interpreter; the model proposes content but cannot attest successful parsing. | Immutable; repair or normalization creates another attributable representation. | Capability resolution and dispatch policy. |
| `VisibleToolSpec` | A description and input schema included in a particular model-facing snapshot. | The tool-discovery and visibility policy. | Immutable within its snapshot; the visible set may differ in a later snapshot. | `ModelRequest` construction and `StructuredAction` validation. |
| `ResolvedCapability` | A runtime-owned resolution result binding normalized identity to a candidate executor and policy context. | The capability registry/resolver, not the model and not the visible schema. | Immutable for one dispatch decision; revocation or policy change requires re-resolution. | The authorization/admission boundary that may create `ActionIntent`. |
| `ActionIntent` / `ActionReceipt` | A durable admitted effect identity and a later outcome record bound to that exact identity. | Runtime policy creates intent before dispatch; the effect adapter/verifier creates a receipt from observed outcome evidence. | Both immutable, append-only records. Absence of a receipt is ambiguity, not a negative receipt. | Executor, recovery/reconciliation, audit, and terminal-state logic. |
| `SessionEvent` / `ContextProjection` | Canonical append-only session facts and a bounded model-visible derivation over named source ranges. | Event owners append only events they are authorized to attest; the context projector derives a view. | Events immutable. A projection is immutable once identified but replaceable by a newer projection; replacement never deletes audit history. | Snapshot construction, continuation, compaction, replay, and inspection. |
| `ChildLineage` | A durable edge binding a child identity to its parent, spawn action, inheritance mode, and shared-prefix range. | The scheduler records it before child work becomes runnable. | Immutable; later status is recorded as events rather than editing ancestry. | Scheduler, cancellation, budget accounting, result attribution, and branch comparison. |
| `Budget` | Operator-defined limits plus attributable consumption events and a derived remaining balance. | The operator/policy plane defines and delegates limits; workers may report consumption but may not enlarge their allocation. | Limit grants are immutable; consumption is append-only; remaining balance is derived. | Sampler, dispatcher, scheduler, compactor, and terminal decision. |
| `TerminalOutcome` | A receipt-backed claim of completed, cancelled, or failed, including unresolved ambiguity where relevant. | The harness terminal-state reducer under operator policy, using recorded evidence. | Immutable; later contradictory evidence opens a separate adjudication/recovery episode linked to the terminal turn. It never mutates or re-enters that terminal turn. | User/operator reporting, experiment scoring, replay, and child aggregation. |

The purity boundary is representational, not a claim that the whole loop is
mathematically pure. Parsing `ModelEvent` values into a `StructuredAction`,
normalizing identities, projecting context from `SessionEvent` values, and
reducing recorded events into state can be deterministic transformations of
explicit inputs. Sampling a model, reading time, persisting an event, resolving
ambient credentials, invoking a tool, observing the external world, cancelling
a process, and verifying an effect cross effectful boundaries. A pure
transformation may propose an `ActionIntent`; only an effect-owning runtime can
durably admit it and dispatch the resolved capability.

This model adopts the distinctions developed in [Seminar 5](../seminars/05-state-identity-and-environments.md),
[Seminar 6](../seminars/06-mutation-simulation-and-constraints.md),
[Seminar 8](../seminars/08-streams-delay-and-infinite-processes.md), and
[Seminar 9](../seminars/09-eval-apply-and-executable-semantics.md). It does not
claim that the compared harnesses use these component names.

## Control sequence and effect boundaries

The exact proposed research state sequence is:

```text
ready → snapshotting → sampling → interpreting
interpreting → dispatching → awaiting-effect → recording → sampling
sampling | interpreting → compacting → sampling
any nonterminal → recovering | cancelled | failed
sampling → completed only when no action or queued input remains
recovering → recording | sampling | cancelled | failed
```

`ready` means a `TurnInput` is admitted. `snapshotting` fixes the
`TurnSnapshot`, including its `VisibleToolSpec` set. `sampling` owns one
`ModelRequest` and records `ModelEvent` values. `interpreting` derives a
`StructuredAction` or concludes that no action was proposed. `dispatching`
normalizes the action, obtains a `ResolvedCapability`, applies policy, and
durably records `ActionIntent`. `awaiting-effect` has crossed the invocation
boundary but lacks a conclusive receipt. `recording` binds an `ActionReceipt`
and resulting `SessionEvent` values before another sample can depend on them.
`compacting` creates a new `ContextProjection` over a named source range.
`recovering` reconciles recorded intent, observed world state, and receipts; it
is not a synonym for retry.

The plan-required `sampling → completed` edge is compact transition notation,
not a bypass around `interpreting`. It is enabled only by a recorded interpreter
result that classifies the completed response as **no action** and by an atomic
observation that the admitted-input queue is empty. Expanded, its authorization
path is `sampling → interpreting(record no-action) → completed`; the compact
edge names the externally visible turn transition. A stop token or friendly
sentence alone is not sufficient. Whether the no-action classification and
empty-queue observation should be one atomic terminal-admission record or two
linked records remains an open research question.

Recovery has only four bounded outgoing outcomes:

- `recovering → recording` when reconciliation obtains a verified receipt or
  world-state observation that can be bound to the existing `ActionIntent`;
- `recovering → sampling` when recorded reconciliation proves that no effect
  occurred or that continuation is safe without re-dispatching the intent;
- `recovering → cancelled` when operator policy terminates the episode and the
  terminal record preserves any still-ambiguous effect; or
- `recovering → failed` when reconciliation cannot establish a safe continuation
  or an operator-owned budget is exhausted.

There is deliberately no `recovering → dispatching` or
`recovering → awaiting-effect` edge: recovery cannot blind-retry an admitted
non-idempotent action. Whether a verified idempotency contract, a successful
compensation receipt, or a newly admitted replacement intent may authorize a
later dispatch is an open research question; it is not decided by this state
model.

### State invariants

1. Visible specifications do not grant authority: `VisibleToolSpec` is not a
   `ResolvedCapability`, and neither is sufficient without the current policy
   decision.
2. Durable intent precedes effect: an exact `ActionIntent` must be recorded
   before a non-read-only capability is invoked.
3. A receipt is not inferred from model text: only the effect adapter or an
   explicit verifier may create `ActionReceipt` evidence.
4. Child lineage is explicit: `ChildLineage` exists before child work becomes
   runnable and binds results, failures, cancellations, and charges to a parent.
5. Budgets are operator-owned: a model, tool, or child may consume or decline
   an allocation but cannot mint or enlarge one.
6. Compaction replaces a context projection, not audit history:
   `ContextProjection` may supersede a model-visible view while the source
   `SessionEvent` and effect-receipt history remain independently addressable.
7. Recovery cannot silently duplicate non-idempotent effects: an intent without
   a conclusive receipt enters reconciliation, not automatic replay.

These invariants are research requirements inferred from the course, especially
[Seminars 6–10](../seminars/06-mutation-simulation-and-constraints.md) and the
[explicit-control synthesis in Seminar 12](../seminars/12-machines-storage-control-and-compilation.md).
They are not claims that one existing harness already satisfies the entire set.

## Bounded Pi, Hermes, and Codex comparison

Every cell is scoped to the cited course evidence. `EVIDENCE` means the locator
shows the stated narrow mechanism. `INFERENCE` means the course derives a useful
category from that mechanism. `MISSING` means the bounded corpus does not
establish the requested property. In the `promotion` row, promotion means a
runtime-owned move from merely proposed or visible material to admitted
execution authority; it does not mean model deployment or product rollout.

| Concern | Pi | Hermes | Codex |
|---|---|---|---|
| Turn snapshot | `EVIDENCE` — `createTurnState` assembles messages, resources, model settings, tool context, active tools, and prompt. [Pi](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/agent-harness.ts#L395-L429) | `EVIDENCE` — `run_conversation` initializes per-turn context, flags, and counters. [Hermes](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L1084-L1325) | `EVIDENCE` — `run_turn` captures step context and exact model-visible state. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L328) |
| Loop | `EVIDENCE` — nested model, tool-continuation, steering, and follow-up loops have distinct exits. [Pi](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) | `EVIDENCE` — iteration-budget loop initializes state and checks interrupt/budget exits. [Hermes](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L1084-L1325) | `EVIDENCE` — turn loop samples, compacts, continues, aborts, errors, or completes. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L520) |
| Tool boundary | `EVIDENCE` — the loop extracts complete calls, executes them, and appends results as separate steps. [Pi](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) | `EVIDENCE` — name repair, valid-name and JSON gates precede `_execute_tool_calls`. [Hermes](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L5700-L6165) | `EVIDENCE` — model-visible specs, canonical name construction, registry resolution, and invocation are distinct. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289) |
| Memory | `MISSING` — stored session entries and projections do not establish cross-session semantic memory. [Pi limit](../seminars/05-state-identity-and-environments.md#agent-harness-bridge) | `EVIDENCE` — providers prefetch context and receive completed-turn synchronization under a session identity. [Hermes](../../../../../evidence/implementations/hermes/snapshot/agent/memory_manager.py#L364-L735) | `MISSING` — the cited history/model-visible-state path does not establish cross-session memory. [Codex limit](../seminars/05-state-identity-and-environments.md#agent-harness-bridge) |
| Compaction | `EVIDENCE` — a compaction entry projects a summary plus retained tail for a selected path. [Pi](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L61-L149) | `MISSING` — counters and flags in the cited turn-loop range do not establish replacement semantics. [Hermes limit](../seminars/12-machines-storage-control-and-compilation.md#agent-harness-bridge) | `EVIDENCE` — `run_auto_compact` is an explicit mid-turn rollover under a context limit. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L418-L457) |
| Delegation | `EVIDENCE` — the cited extension bounds parallel tasks/processes and aggregates child results or failures. [Pi](../../../../../evidence/implementations/pi/snapshot/packages/coding-agent/examples/extensions/subagent/README.md#L91-L175) | `EVIDENCE` — delegation checks depth and concurrent-child count, joins background batches, and distinguishes pause from interrupt. [Hermes](../../../../../evidence/implementations/hermes/snapshot/tools/delegate_tool.py#L2778-L2985) | `EVIDENCE` — shared tree control reserves capacity and carries parent/depth/path metadata. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control.rs#L90-L453) |
| Persistence | `EVIDENCE` — append reaches the configured store before the in-memory leaf advances; durability varies by store. [Pi](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L333-L515) | `EVIDENCE` — the active session-database path flushes before tool execution; disabled/no-database paths differ. [Hermes gate](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L6058-L6104) [Hermes flush](../../../../../evidence/implementations/hermes/snapshot/run_agent.py#L1919-L1962) | `MISSING` — recording conversation items in the cited turn path does not prove physical durability. [Codex limit](../seminars/05-state-identity-and-environments.md#agent-harness-bridge) |
| Permissions | `MISSING` — the cited Pi ranges do not establish a runtime permission/sandbox authority model. [Course limit](../seminars/12-machines-storage-control-and-compilation.md#limits-of-the-analogy) | `MISSING` — valid tool names and spawn limits do not establish effect authorization or OS mediation. [Course limit](../seminars/04-symbols-sets-compression-and-generic-dispatch.md#limits-of-the-analogy) | `EVIDENCE` — runtime sandbox selection depends on permission profile and platform/runtime constraints. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/sandboxing/src/manager.rs#L264-L380) |
| Recovery | `INFERENCE` — error and aborted stop reasons are distinct exits, but effect reconciliation is not shown. [Pi](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) | `INFERENCE` — a failed pre-effect flush prevents tool execution, but unknown post-effect reconciliation is not shown. [Hermes](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L6058-L6104) | `INFERENCE` — aborted and error branches are explicit, but the cited turn loop does not prove non-idempotent effect reconciliation. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L520) |
| Evaluator | `INFERENCE` — response events are interpreted into calls, but Pi's loop is not SICP `eval`. [Pi](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) | `INFERENCE` — normalization/validation interprets structured requests, but is not an expression evaluator. [Hermes](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L5700-L6165) | `INFERENCE` — response-item interpretation and capability invocation form a useful boundary model, not Scheme `eval`/`apply`. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L520) [router](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289) |
| Promotion | `MISSING` — the cited Pi evidence does not show promotion from visible spec to runtime authority. [Seminar 2 boundary](../seminars/02-data-abstraction-and-immutable-representation.md#agent-harness-bridge) | `INFERENCE` — reserved/duplicate-name rejection and valid-name gating constrain routing, but do not prove authority promotion. [Hermes registry](../../../../../evidence/implementations/hermes/snapshot/agent/memory_manager.py#L364-L455) [Hermes gate](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L5700-L6165) | `EVIDENCE` — visible specs remain separate from registry-held runtime resolution and dispatch. [Codex](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289) |

The table is intentionally sparse. In particular, `MISSING` does not mean a
feature is absent from a project; it means this course's exact pinned ranges do
not justify the stronger claim.

## Research hypotheses, not decisions

1. Whether an append-only session event log plus derived context projection is
   sufficient for reproducible small-agent experiments.
2. Whether tool visibility and runtime capability resolution can remain
   separate without excessive ceremony.
3. Whether explicit state transitions improve fault diagnosis relative to a
   single recursive loop.
4. Whether branch lineage and matched budgets make harness-policy comparisons
   more credible.
5. Which effect and recovery semantics must be frozen before comparative
   experiments are meaningful.

Each hypothesis needs a falsification plan. For example, the first fails if two
runs with the same canonical events, projection algorithm, pinned model/runtime
attestations, and operator policy cannot reconstruct equivalent requests. The
fourth fails if lineage and budget matching still leave uncontrolled external
state or scheduler effects large enough to dominate the comparison. These are
examples of tests to design later, not implementation requirements silently
promoted into decisions.

## Explicit exclusions

This dossier does not specify:

- Rust modules, crates, traits, enums, or public API names;
- CLI syntax or operator commands;
- a persistence schema, database, serialization format, or migration plan;
- provider integration, credentials, transport, or model-specific adapters;
- an implementation schedule, staffing plan, or delivery milestones.

It also approves no implementation. Choosing any of the above requires the separate
design cycle stated at the opening.

## Limits and missing evidence

- `MISSING` — no common workload has been run through Pi, Hermes, and Codex
  with matched models, tools, external state, budgets, and event
  instrumentation. The comparison is architectural reading, not a benchmark.
- `MISSING` — the course does not prove that the cited Pi store, Hermes session database, and Codex conversation history
  share durability,
  transaction, replay, or retention semantics.
- `MISSING` — none of the bounded excerpts establishes exactly-once execution
  for arbitrary filesystem, subprocess, network, credential, or human-visible
  effects.
- `MISSING` — cancellation propagation, capacity fairness, and durable child
  lineage are not established uniformly across the compared harnesses.
- `MISSING` — compaction quality, semantic loss, and replay fidelity have not
  been measured under a course-owned evaluation contract.
- `INFERENCE` — the conceptual components and state sequence are a synthesis of
  [Seminars 2, 4–10, and 12](../sicp_course_guide.md#agent-harness-design), not a
  taxonomy asserted by SICP or by any one harness.

The analogy limit is fundamental: SICP's evaluator defines semantics for a
small programming language. A coding-agent harness coordinates probabilistic
model output, runtime policy, external capabilities, partial failures, and
durable evidence. An LLM is not literally `eval`; a tool executor is not
literally Scheme `apply`; a context summary is not a stream promise; and a
cancelled branch has not thereby rolled back its effects.

## Navigation

Return to [Seminar 12 — Machines, storage, explicit control, and compilation](../seminars/12-machines-storage-control-and-compilation.md)
for the state-machine derivation, or return to the
[SICP course guide](../sicp_course_guide.md) for sequential and concept-oriented
routes. Future implementation work starts with a separate design cycle; it does not
continue by treating this research brief as an approved architecture.
