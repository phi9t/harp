# Agentic eval/apply: an effectful evaluator for bounded work

## Thesis and analogy boundary

**CLAIM — SICP evaluator.** In the metacircular evaluator, `eval` interprets
an expression in an environment, while `apply` invokes a procedure value with
already-evaluated argument values. A compound procedure carries its defining
environment; applying it extends that environment with parameter bindings and
evaluates its body there (`SICP-2E-UF`, §§4.1–4.1.4, printed pp. 492–520 / PDF
pp. 520–548). The [Seminar 09 treatment](course/seminars/09-eval-apply-and-executable-semantics.md)
and [evaluator deep dive](metacircular_evaluator_deep_dive.md) give the book's
fuller semantic account.

**INFERENCE — runtime thesis.** A bounded agent runtime is usefully designed as
a *partially observable, effectful evaluator*: it interprets admitted work under
current policy and a selected context projection, proposes or interprets
actions, invokes only resolved capabilities, and incorporates new observations
before deciding what to do next. The useful transfer is a discipline of
separating representations, authority, and consumers. It is not a claim of
semantic equivalence.

An LLM is not literally `eval`: it is a probabilistic producer of model events,
not SICP's expression-and-environment mapping. Tool execution is not literally
Scheme `apply`: it is policy-mediated work against an external, changing world.
Nor do Pi, Hermes, and Codex share a formal architecture. They are pinned,
separate implementation observations, summarized in the [Pi](../pi_harness_deep_dive.md),
[Hermes](../hermes_harness_deep_dive.md), and
[Codex](../codex_harness_deep_dive.md) deep dives; the [capstone dossier](course/capstone/agent_harness_architecture_dossier.md)
keeps their evidence and the course synthesis distinct.

**INFERENCE — ownership boundary.** This essay is a semantic bridge from
Seminar 09's evaluator to questions raised by effectful agent work. The
[agent-harness architecture dossier](course/capstone/agent_harness_architecture_dossier.md)
is the maintained place for the full proposed state vocabulary, control
sequence, and research hypotheses. Nothing here defines an API, schema,
database record, state machine, or replacement for that capstone; the
lower-case pseudocode terms below name semantic obligations only.

**MISSING — formal correspondence.** The repository supplies no formal
semantics proving that an agent loop is equivalent to SICP's evaluator, no
common execution trace across the three implementations, and no basis for
promoting this vocabulary into their shared API.

The engineering payoff is narrower and practical: when an agent changes code,
starts a job, or delegates work, an operator should be able to answer *which
observation justified the action, which policy admitted it, what effect was
attempted, and what later observation supports the outcome?* A final natural
language answer cannot substitute for those records.

## Mutual recursion over work and capabilities

The following is a semantic sketch, not an implementation prescription. It
shows only the mutual recursion and the point where a selected context needs
fresh evidence. Its terms are not required type, method, record, or state
names.

```text
agentic_eval(instruction, context):
  projection = select a context projection from retained evidence
  if the projection needs fresh evidence:
    observation or unresolved = agentic_apply(ContextAcquisitionRead, [], context)
    if unresolved: return it
    projection = extend the projection with the observation
  interpret instruction under the selected projection
  if it is a call, resolve/evaluate what is necessary and return
    agentic_apply(resolved_capability, values, context)
  if it is a control form, evaluate only according to its form-specific rule

agentic_apply(resolved_capability, values, context):
  primitive/read capability -> invoke under runtime policy and return an
    observation or unresolved outcome
  compound workflow -> evaluate its body under extended context and return
    agentic_eval(body, extended_context)
```

**CLAIM — SICP boundary.** The book's ordinary application evaluates operator
and operand expressions before `apply`; compound application evaluates a body
in the saved environment extended with formal-to-actual bindings (`SICP-2E-UF`,
§4.1.1 and §4.1.4, printed pp. 497–500 and 518–520 / PDF pp. 525–528 and
546–548).

**INFERENCE — controlled extension.** In an agent runtime,
`ContextAcquisitionRead` is an apply-like capability invocation whose
conclusive outcome may support an observation; it is not an already trusted
value. The selected projection is a deterministic transformation of retained
evidence, whereas freshness acquisition crosses the effect boundary and cannot
be hidden inside projection. Compound apply likewise returns to evaluation
under extended context. Neither branch makes a model-produced action
authoritative, and neither treats a returned report as truth without
provenance.

Once this sketch meets external effects, its elided questions need explicit
runtime semantics: how current policy admits work, how intent and outcome
evidence remain durable, how pending or ambiguous effects are reconciled, how
terminal work is recognized, and how child work is scoped. The
[capstone dossier](course/capstone/agent_harness_architecture_dossier.md)
owns the maintained proposed architecture, records, control sequence, and
research hypotheses for those consequences.

The recursion matters because a conclusive `ContextAcquisitionRead` changes the
next decision. The evaluator does not merely execute a precomputed plan: it
applies a permitted `ContextAcquisitionRead`, receives a verified observation,
deterministically projects retained events, and evaluates again. Effects make
the difference more severe: an incomplete receipt after a write is not a normal
return value and must enter recovery rather than become an optimistic premise
for the next recursive step.

## Context is a view, not the world

An agent prompt is a view, not the runtime's whole state. The semantic bridge
only needs six distinctions:

- a snapshot says what one sampling step was shown, including a model-visible
  policy description; it is not dispatch authority;
- retained event history supports audit after the prompt has omitted or
  summarized details;
- a context projection is a deterministic, provenance-bearing derivation over
  retained events, not a world read;
- a visible schema, resolved capability, and current authorization are
  different things, so revocation between sampling and dispatch remains
  observable;
- budgets and parent-child lineage constrain what work is attributable to an
  episode; and
- files, processes, services, and remote objects remain re-observable world
  state rather than facts made current by a transcript.

Freshness or retrieval first crosses the effect boundary through an applied,
resolved, authorized `ContextAcquisitionRead` and yields a receipt-backed
observation. Only then may the evaluator deterministically project retained
events for the next snapshot. The resulting projection should identify its
sources, transformation, omissions, and freshness boundary, but this essay does
not prescribe a context object or persistence layout.

For the maintained conceptual state vocabulary, exact control sequence, and
research questions about these distinctions, use the
[agent-harness architecture dossier](course/capstone/agent_harness_architecture_dossier.md).
This essay retains only the distinctions needed to read the evaluator analogy.

**EVIDENCE — pinned implementation observations.** The Pi deep dive records a
turn-scoped state assembled before execution and a parent-linked session
projection; the Hermes deep dive records a provider-mediated memory boundary
and bounded delegated children; and the Codex deep dive records turn-scoped
state, persisted rollout items, routed capabilities, parent-child lineage, and
host-enforced sandbox selection. These observations are source-specific, not a
shared schema.

**INFERENCE — materialization rule.** Reject an eagerly, fully materialized
"agent context" object as the authority for all later decisions. Instead,
materialize a demand-driven projection that carries provenance: source event
identities or ranges, projection algorithm/version, omitted or summarized
regions, retrieval time, and any re-observation receipt. This preserves a
question that a monolithic prompt erases: *did the model see a fact, did the
runtime observe it, or is it now stale?*

**MISSING — sufficiency.** No cited source establishes that a compacted
projection is lossless, that all event stores have identical physical-durability
semantics, or that replaying a logical history reconstructs external processes
and remote side effects.

## The capability pipeline

The semantic effect path is exactly:

```text
model events -> structured action -> normalized identity -> resolved capability -> action intent -> effect attempt -> action receipt -> new observation
```

This is causal vocabulary, not a required API or state-machine sequence. Each
arrow asks a distinct question:

1. **Model events → structured action:** Did the protocol interpreter obtain a
   complete, schema-valid request, and from which event IDs?
2. **Structured action → normalized identity:** Which stable name/version or
   capability key does the request denote after aliases and repairs are made
   explicit?
3. **Normalized identity → resolved capability:** Which runtime-owned executor
   and *current runtime policy revision* currently match that key? Resolution
   occurs after sampling rather than against the snapshot's model-visible policy
   description, so revocation or policy change between sampling and dispatch is
   observable. It can fail because the capability is absent, revoked, or
   ambiguous.
4. **Resolved capability → action intent:** Did authorization admit this exact
   operation, arguments, idempotency key, lineage, budget, and current policy
   revision before effectful dispatch?
5. **Action intent → effect attempt:** Did the adapter begin the requested
   operation? Beginning is not the same as completion.
6. **Effect attempt → action receipt:** What evidence did the adapter or a
   verifier actually record: output digest, operation ID, status, error, or
   unresolved outcome?
7. **Action receipt → new observation:** Is the receipt conclusive success or
   conclusive failure, and if so which fact may the next evaluation consume
   with its time, source, and confidence boundary? An acknowledgement/pending
   receipt awaits completion; an ambiguous outcome enters reconciliation rather
   than producing an ordinary recursive premise.

**EVIDENCE — pinned implementation observation.** The Codex deep dive records
that its cited router keeps model-visible specifications, structured calls,
normalization into an invocation shape, and runtime registry dispatch separate.
The Hermes deep dive records a configured session-database path that appends the
assistant tool-call turn before side-effecting tool execution. Those are
specific implementation paths, not evidence that every capability in either
system has this complete pipeline.

**INFERENCE — authority boundary.** A model event can propose an action;
parsing can make it well formed; a visible schema can make it understandable;
and a normalized identity can make it resolvable. None of those stages grants
authority. Authority appears only at the runtime policy decision that admits an
intent for a resolved capability. At that point the runtime rechecks current
policy and can reject a capability revoked after sampling. Its retained evidence
must let a reviewer distinguish the snapshot's model-visible policy from the
policy effective at dispatch. The capstone owns any fuller record vocabulary or
control sequence needed to express that distinction.

## Effects, receipts, and recovery

The primitive/compound split acquires a new consequence once application can
change the world: an attempted effect is not automatically a value. A
receipt can establish completion or conclusive failure; an acknowledgement says
only that work was accepted or remains pending; ambiguity says too little to
continue normally. Thus only a conclusive, receipt-backed observation can be a
normal premise for a later `agentic_eval`.

This is why current authorization, persistence, idempotency, asynchronous
completion, and post-crash reconciliation belong to the semantic problem. A
read is often repeatable, a content-addressed write may be conditionally
repeatable, and an irreversible mutation may instead require an operation
identity or compensation. A runtime cannot infer that choice from a tool name.
If it cannot tell whether an admitted mutation happened, recovery begins from
the retained intent and retained evidence rather than pretending a retry is an
ordinary recursive call. The [capstone dossier](course/capstone/agent_harness_architecture_dossier.md)
owns the detailed proposed records, transitions, and invariants for making
those distinctions operational.

**INFERENCE — recovery rule.** An ambiguous non-idempotent mutation must enter
reconciliation rather than count as an ordinary retryable failure. A later
retry has different meaning only when capability-specific evidence—such as a
verified absent effect, idempotency material, compensation, or a newly admitted
replacement intent—establishes why duplication cannot occur.

**MISSING — universal effect guarantee.** The cited corpus does not establish
exactly-once execution, durable reconciliation, or compensation for arbitrary
filesystem, subprocess, network, credential, or human-visible effects.

## Compound procedures and delegation

A compound procedure need not be a language closure. In a runtime it can be a
skill with declared resources, a workflow with steps, or a subagent task with a
bounded contract. The transfer from SICP is structural: preserve the body,
captured context, formal inputs, and call-time bindings separately enough to
ask what work inherited its meaning from the caller.

**INFERENCE — compound outcome semantics.** A synchronous workflow extends
captured context and evaluates its body in the caller's episode. Whether that
body completes, fails, awaits work, or enters recovery, the parent retains a
receipt linking invocation to the workflow's status; only a conclusive
receipt-backed observation can become a new parent premise. Delegation differs:
the child is an independently scheduled process, not merely an extended
environment. That shift raises semantic questions absent from a Scheme closure:
which context is inherited or isolated, where ancestry and budget live, what
cancellation means after effects have begun, and whether a child report is
evidence or merely assertion. A report remains an observation for the parent to
interpret, not an automatically trusted value. The
[capstone dossier](course/capstone/agent_harness_architecture_dossier.md)
maintains the proposed lineage, budget, cancellation, and child-status
contracts.

**EVIDENCE — pinned implementation observations.** The Pi deep dive describes
an example subagent extension using isolated processes and contexts with bounded
output and abort propagation. The Hermes deep dive describes isolated children,
configured depth/concurrency/iteration bounds, inherited toolsets, and
aggregation. The Codex deep dive describes a root-thread tree with shared
control state, budget/reservation mechanics, persisted spawn edges, and
configurable history inheritance. These observations differ in scope and do
not establish common cancellation, trust, or durability guarantees.

## Execution is not benchmark evaluation

`agentic_eval` in this essay names the *execution-side interpreter* that turns
admitted work plus observations into the next controlled action. It is not the
benchmark, evaluator, judge, or selector infrastructure sometimes called
"agentic evaluation."

Those are different control-plane roles:

- an **execution evaluator** interprets the current work episode;
- a **benchmark/evaluator** measures an outcome against a task and metric;
- a **selector** compares candidates or outcomes and decides promotion; and
- a **policy owner** defines budgets, permissions, task splits, and the
  acceptance rule.

**INFERENCE — separation of concerns.** A runtime can correctly execute a
repair yet fail to establish that the repair is better than alternatives.
Conversely, a benchmark may score an artifact without possessing authority to
run a privileged tool. Keep these roles outside each other's mutable context
where independence matters.

**MISSING — common evaluator.** The inspected Pi, Hermes, and Codex material
does not provide one shared immutable benchmark, matched resource accounting,
or selector that proves improvement across their implementations.

## No-action and terminal control

No action is not a model sentence such as “done.” It is one explicit
interpreter control meaning alongside “structured action” and “interpreter
failure.” The interpreter retains a no-action classification only after it has
accounted for the complete model event sequence under the active protocol.

**INFERENCE — terminal semantic.** Completion means more than the absence of a
tool call: it relates a retained no-action interpretation to the admitted-input
queue at one decision boundary. The queue must be atomically empty for the
relation to denote a terminal result; if input has arrived, the evaluator has
another work item rather than a terminal value. A stop token or friendly final
answer does not settle that relation. The capstone owns the detailed terminal
and queue transition design.

## Bounded code-repair trace

Consider a repair episode constrained to one file, one approved test command,
a finite token/tool budget, and no network or deployment capability. The goal
is to repair a parser bug, not to create an unrestricted coding agent.

1. `agentic_eval(repair_work, context_0)` freezes `snapshot_0`: task identity,
   allowed file, repository revision, visible read/write/test schemas, budget,
   and policy. It samples model events that request a symbol search.
2. The interpreter derives `search(symbol="parse_header")`. The registry
   resolves it to `ContextAcquisitionRead(search)`. `agentic_apply` records the
   attempt and conclusive receipt, then returns an observation naming matched
   paths and source digests.
3. `agentic_eval(repair_work, context_1)` receives that observation rather than
   relying on the model's recollection. It applies a second
   `ContextAcquisitionRead(source)` to inspect the target function and its
   focused tests. The receipt records the exact file revision that was observed.
4. The next evaluation proposes a minimal patch. The runtime resolves the
   write capability, checks the one-file allowlist and expected preimage digest,
   records an admitted write intent with an idempotency key, and then attempts
   the patch. Its receipt says whether the preimage matched and records the
   resulting content digest. The receipt is not inferred from the model saying
   "the fix is applied."
5. With the post-write observation in `context_3`, the evaluator applies the
   approved test capability. The test receipt contains command identity, exit
   status, bounded output digest, and execution environment facts. A failing
   receipt returns to another bounded `agentic_eval`/read/apply cycle; it does
   not authorize an unrestricted rewrite.
6. Completion is evaluated from new observations: the expected file digest,
   the admitted write receipt, the passing test receipt, a *recorded no-action*
   classification, an atomically empty admitted-input queue, and remaining
   budget. If a new input is already admitted, the runtime snapshots it and
   continues sampling instead. The final answer reports those observations and
   any limits. It is not itself the completion evidence.

The nesting is deliberate:

```text
agentic_eval(repair)
  -> agentic_apply(ContextAcquisitionRead(search))
  -> agentic_eval(repair with search observation)
       -> agentic_apply(ContextAcquisitionRead(source))
       -> agentic_eval(repair with source observation)
            -> agentic_apply(admitted write)
            -> agentic_eval(repair with write receipt)
                 -> agentic_apply(test)
                 -> agentic_eval(completion check with test observation)
```

If the process crashes after the write intent but before its receipt, the trace
does not continue with a blind patch retry. It re-observes the target's digest,
reconciles it with the stored intent, and either records the outcome or stops
with the ambiguity visible.

## Failure taxonomy

These are category errors about which representation is being treated as which
other representation, not model-quality diagnoses:

1. **Proposal as authority:** model output is mistaken for permission, or a
   visible schema is mistaken for an executable, authorized capability.
2. **Memory as world:** stale context is mistaken for present files, processes,
   credentials, or remote state.
3. **Uncertainty as failure:** an ambiguous mutation is retried as though a
   timeout proved non-execution.
4. **Loading as reading:** import or validation is treated as harmless even
   though parsing, initialization, dependencies, or hooks can execute work.
5. **Stopping as undoing:** cancellation of a child or subprocess is mistaken
   for rollback of effects already issued.

The [capstone dossier](course/capstone/agent_harness_architecture_dossier.md)
owns the concrete research architecture controls for these errors. The semantic
lesson here is simply that each mistake crosses a boundary without the evidence
the later boundary requires.

## Semantic review questions

Use four analytic questions to test whether a design preserves the bridge:

1. **What is being interpreted?** Can the design keep model events, structured
   action, capability resolution, authorization, and effect apart long enough
   to explain which one failed?
2. **What can count as the next premise?** Does it distinguish retained
   context, a fresh observation, an acknowledgement, a conclusive receipt, and
   unresolved ambiguity?
3. **Where does scope change?** When work becomes a workflow or child process,
   what is inherited, what becomes independently observable, and what does a
   returned report mean?
4. **Which evaluator is speaking?** Is execution-side `agentic_eval` kept
   separate from benchmark scoring, selection, and promotion?

The [capstone dossier](course/capstone/agent_harness_architecture_dossier.md)
contains the maintained full research architecture, detailed question set, and
hypotheses. If these four questions cannot be answered from retained evidence,
the evaluator analogy is not yet operationally meaningful.

## Navigation

[Seminar 09 — Eval/apply and executable semantics](course/seminars/09-eval-apply-and-executable-semantics.md)
owns the SICP evaluator semantics: expression data, environments, procedure
values, and application. This essay isolates the semantic bridge: what changes
when the apply-like boundary touches partial observations and external effects.
For the maintained full proposed state vocabulary, control sequence, bounded
implementation comparison, and research hypotheses, continue to the
[agent-harness architecture dossier](course/capstone/agent_harness_architecture_dossier.md).
