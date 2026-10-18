# RFC 0001 implementation analysis: a broker above replaceable runtimes

Status: proposed implementation path, not an approved build specification.
Date: 2026-10-01.
Commit inspected: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Source: [preserved owner proposal](0001-durable-team-control-plane.md).
Decisions/frontier: [wayfinding map](../efforts/team-control-plane/map.md).

## Recommendation

Build a separate, supervised Corenius broker with PostgreSQL team-domain
authority, Temporal workflow control, a gateway owning Trae runtime mutations,
an artifact store and CLI/MCP access. Keep the lead replaceable. Start on one
host with one lead, two implementing workers and an independent verifier.
The present Python toolbox supplies useful contracts and fixtures, not the
new authoritative scheduler.

Prefer a Rust broker and gateway beside the existing Rust application. Evaluate
the supported Rust Temporal SDK with a pinned restart/replay spike before
locking the workflow-worker language. Python is a credible fallback for the
Temporal worker, not an automatic second orchestration layer. Rust is no longer
excluded on the assumption that Temporal offers only an experimental internal
Core SDK: the current [official Rust guide](https://docs.temporal.io/develop/rust)
documents a public SDK. Language selection remains a decision ticket.

Use a PostgreSQL-only fake-runtime slice to validate transactions first, then
add Temporal before declaring the durable orchestration milestone complete.
This sequencing does not replace the owner's requested Temporal architecture.
No new lead/worker runtime or deployable service was built in this investigation.

## What exists, and what can be reused

| Existing asset | Useful input | Missing for the proposed control plane |
| --- | --- | --- |
| Rust ACP provider | Model transport, streamed output, process observations | Creates a new child/session per prompt; no persistent thread binding, command journal or turn reconciliation |
| Rust turn-scoped runner/TUI | Agent turn behavior and terminal presentation | TUI/lead must become clients of broker state, not owners of team lifetime |
| Python worker supervisor | Process/boundary verdicts, recorded launch inputs | No team-domain transactions, durable mailbox, ownership generation or complete cancellation guarantee |
| Effort dehydrate/rehydrate | Durable human context and pointer drift checks | Auto-commits; not a broker lease or external-execution recovery protocol |
| Worktrees and gated landing | Owned candidate workspaces and exact-tip safety scenarios | Worktrees share Git data; no-push landing still moves local mainline; authorization belongs outside the tool |
| Insula capsule builder | Read-only binds, allowed writable paths and PID containment | Does not establish task identity, runtime deduplication or remote workload ownership |
| Benchmark/tmux harness | Controlled faults, visible runs and cost/time observations | Terminal bytes are not domain events or completion authority |
| Durable-execution design note | Trace/artifact separation and replay questions | A proposal, not evidence that effect-idempotent replay is implemented |

Local evidence: [runtime capability report](../efforts/team-control-plane/research/trae-runtime-capabilities.md),
source locations and inspected commit are recorded there. The previous
[reference-workflow design](../efforts/agentic-engineering/reference-design.md)
and reviewed draft spec retain their safety/evidence value. Their assumption
that a lead owns operational dispatch is superseded only if this RFC is adopted.
Do not implement two competing durable ledgers.

## Actual Trae interfaces and unresolved guarantees

Installed TraeCLI: `0.207.1(internal edition)`. Safe schema generation found:

- Thread start/resume/read and turn start/steer/interrupt contracts.
- `turn/steer` has an expected turn ID precondition.
- Thread goals and an experimental native workflow surface.
- No start idempotency key on thread/turn start. User-message correlation is
  not a documented promise of durable duplicate suppression.
- No explicit runtime boot/incarnation field in the inspected core schemas.
- No verified durable notification cursor or complete external-effect history.

These are available schema contracts, not tested runtime guarantees. Trae's
experimental workflow start/revise endpoints do expose idempotency keys, but
their behavior is untested and does not extend to ordinary thread/turn starts.
The current wrappers do not explicitly disable shared-daemon behavior.
Consequently owning their shell PID must not be confused with owning the
runtime that executes their work.

Proposed first real adapter: an explicitly supervised private Trae app-server
with broker-owned transport and storage binding. Keep native goals disabled
unless continuation is explicitly delegated. Use an exec-process adapter only
as a simpler transitional mode with narrower recovery claims. ACP alone does
not meet saved-thread/reconnection requirements without new provider support.

## Ownership and module interfaces

| Module | Owns | Public interface | Does not own |
| --- | --- | --- | --- |
| Team domain | Missions, task revisions, acceptance, permissions, budgets and generations | Validated idempotent commands and generated mission views | Retry clocks or native conversation history |
| PostgreSQL store | Durable domain rows, journal, events, outbox and projection cursors | Atomic command transaction; claim/version checks | Runtime-start exactly-once effects |
| Workflow worker | Deterministic waits, timers and bounded lifecycle orchestration | Mission/task workflows invoking broker Activities | A second authoritative task-status table |
| Runtime gateway | Runtime bindings, observed capabilities and operation reconciliation | Inspect, bind, start episode, observe, steer, interrupt, reconcile | Plan authority or acceptance of its own output |
| Execution supervisor | Attempt workspaces, process/job handles and containment | Allocate, inspect, cancel, checkpoint and quarantine | Team role authority |
| Artifact store | Immutable candidate/evidence bytes and manifests | Publish/read by content identity and version | Granting acceptance because a hash matches |
| CLI/MCP/TUI | Human/agent commands and observed views | Same authenticated broker commands | Starting turns behind the gateway |

Logical modules may share a binary or process. Run broker/gateway supervision
outside the lead and outside the TUI. Runtime storage identity and broker-issued
gateway epoch are separate from provider thread IDs. A gateway epoch fences
broker messages; it does not prove that a superseded app-server or tool process
has stopped.

## Domain and storage shape

Model Mission, Task, TaskRevision, Attempt, Principal/WorkerIdentity,
RuntimeBinding, Episode/Turn, Artifact, Verification, Decision and Approval
separately. Preserve Attempt-to-workspace and Episode-to-runtime relationships;
do not make a logical worker persona the permanent owner of one dirty tree.
Keep the existing Effort concept as a planning/export bridge rather than
silently renaming it Mission. The terminology mapping is a decision.

Suggested tables, subject to schema review:

- Missions, principals, role grants and mission budget policies.
- Tasks, immutable task revisions, dependency edges and ownership generations.
- Attempts, budget reservations/usage and execution environment handles.
- Runtime deployments/storage bindings, gateway epochs and episode bindings.
- Commands and normalized argument hashes; operation journal and external observations.
- Messages, presentation records, acknowledgements, replies and question resolutions.
- Decisions, approvals, artifacts/manifests and exact-candidate verifications.
- Append-only domain events, transactional outbox, workflow bindings and projection cursors.

Namespace command deduplication by authenticated principal and operation.
Reusing a key with different normalized arguments is a conflict, not a repeat.
Use revision preconditions and unique constraints to prevent two accepted
owners of the same task generation. Preserve event sequence per aggregate.
Database roles separate domain access from Temporal's internal persistence;
never mutate Temporal's internal tables as the integration mechanism.

## One task through the system

1. A lead proposes a versioned task; the owner/delegation policy authorizes it.
2. The broker transaction validates role, expected revision and dependencies,
   reserves budget/ownership, creates an Attempt and operation intent, and
   appends a domain event plus outbox row. It commits before runtime I/O.
3. A relay durably delivers the command reference to the corresponding Temporal
   workflow. Wakeup delivery and business-command application remain separate.
4. An Activity calls the broker/gateway with the same stable operation identity.
   The gateway checks current ownership and records dispatch intent before I/O.
5. The runtime reply/observation binds a provider thread and turn to the Attempt.
   Loss of the reply produces outcome_unknown and a reconciliation requirement.
6. The worker submits a versioned candidate under its scoped identity/generation.
7. The verifier checks the exact candidate, task revision, test revision and
   environment. A clean process verdict alone does not permit acceptance.
8. The lead evaluates open engineering questions; the broker applies the
   accepted policy transactionally. The human authorizes integration under the
current canonical policy; workers hold no protected-branch credentials.

Dispatch requires its own exclusive ownership protocol, not only a pre-I/O
fence check. A durable dispatch owner and per-binding operation serialization
prevent concurrent Activities/gateways issuing the same start. Losing that
owner after a possible external call enters uncertainty; lease expiry alone
does not grant permission to resend. Reassignment must await reconciliation or
use a quarantined old attempt and an isolated new workspace. Precise transitions
and the handoff race are open decision work, not solved by this sketch.

PostgreSQL transactions protect intent and domain transitions, not provider
effects. [Transactions](https://www.postgresql.org/docs/current/tutorial-transactions.html)
provide the local atomic boundary. Notifications can wake listeners, but durable
work is re-read from tables; [NOTIFY](https://www.postgresql.org/docs/current/sql-notify.html)
is not the command ledger. This is an architectural recommendation grounded in
those mechanisms, not a database-supplied exactly-once execution guarantee.

## Temporal integration without duplicate authority

Workflows refer to mission/task IDs and durable command/event cursors. Activities
perform database, runtime, Git, model and environment I/O. Configure retry
classes explicitly: deduplicated domain transitions can retry; ambiguous
external starts cannot be blindly retried. A repeated start Activity consults
the journal and reconciles or suspends. No default retry policy may silently
duplicate model work. [Temporal deterministic constraints](https://docs.temporal.io/workflow-definition#deterministic-constraints),
[Activity execution](https://docs.temporal.io/activity-execution).

The task workflow requests admission, waits for runtime observations, handles
questions/approvals, requests verification and follows the declared repair
budget. Mission workflows request lead episodes for meaningful events and
handle escalation/deadlines. Domain status always comes from broker
transactions; Workflow progress may be a projection, not a second truth.

Workflow wakeups carry command references; duplicate wakeups are safe because
command application is deduplicated. A missed wakeup is recovered by polling
the durable command/event cursor. High-volume token/terminal traces go to
artifact storage. Continue-As-New carries only IDs, cursors and needed durable
control state, while preserving pending command processing; test concurrent
wakeups during rollover. Workflow code/version migration needs replay tests.

The PostgreSQL/Temporal bridge needs an explicit protocol before implementation:
stable Workflow ID and concurrent-start handling, outbox-delivery acknowledgement,
the durable processed-command cursor, wakeups arriving after terminal completion,
and pending commands during Continue-As-New. Database acceptance, transport
delivery and business application are distinct acknowledgements. These are
questions for the bridge decision ticket, not assumed properties of a signal.

## Messaging, tools and authentication

Expose authenticated team tools through MCP or a small broker API, backed by
the same command handler used by the CLI. Derive sender identity from a scoped
credential; body fields cannot grant a lead role. Use attempt/generation-bound
worker credentials with limited task, mailbox and artifact rights. A localhost
socket or shared UID alone does not distinguish a worker from a lead.

Keep broker/provider credentials out of worker writable paths and logs. Broker
access is intentional network access, separate from the model-provider route;
do not describe the agent capsule as offline when it can use these endpoints.
Database revision/fence checks constrain commands, while execution containment
constrains filesystem/process effects. Neither replaces the other.

Delivery states persist separately: recorded, presented to a specific episode,
acknowledged and resolved. Acknowledgement does not resolve a question. Failed
steering returns delivery to pending or records uncertain delivery without
claiming the model saw it. Use message IDs in a subsequent episode to support
explicit acknowledgements/deduplication. Cancellation uses the adapter/supervisor,
not mailbox prose. Yield records a durable wait and ends a bounded episode.

Current MCP includes an optional Tasks extension; it can expose broker-owned
task handles where supported. It does not replace team authority, ownership
fencing or the persistence implementation. Pin the actual supported protocol
and extension. [Tasks extension](https://modelcontextprotocol.io/extensions/tasks/overview).
A2A remains a future external-agent boundary; its [specification](https://a2a-protocol.org/latest/specification/)
defines task/message/artifact exchange, not this broker's acceptance policy.

## Unknown effects, cancellation and recovery

Record command/operation state independently of task state. Proposed operation
states are prepared, dispatched, confirmed, outcome_unknown, reconciling and
terminal. Proposed Attempt state also includes waiting-input, cancel-requested,
stopping, stopped and quarantined. Exact transition rules are a specification
deliverable; do not force unknown effects into ordinary failed/retryable.

If a start acknowledgement is lost, inspect the known binding, persisted
history and execution handle. Absence of evidence is not evidence of no start.
If correlation cannot prove the outcome, quarantine the operation and retain
budget. A replacement worker gets a new attempt workspace; the old generation
can submit audit material but cannot become authoritative.

Treat lost thread creation separately from lost turn start. A lost thread/start
reply may leave no known thread ID to read or resume; private-store inspection
or correlation is only a proposed recovery technique until measured. Turn/start
normally has a known thread binding but still needs reliable operation correlation.
If either lookup cannot establish the outcome, quarantine rather than inventing
a recoverable binding. Generations fence broker acceptance and gated shared
operations; they do not revoke an old process's existing filesystem or credential
access. Shared-resource enforcement needs an explicit execution/integration design.

Cancellation has three distinct observations: request recorded, runtime
interruption acknowledged, and owned tool/workload effects stopped or explicitly
detached. Temporal Activity cancellation is cooperative and cannot by itself
kill those processes. Leases likewise cannot stop an old shell. Explicitly
address external Execution handles and determine whether a workload is
cancelled, adopted or left running by policy.

Startup reconciliation runs before dispatch: inspect active/unknown operations,
gateway/runtime bindings, surviving environments and retained budgets. Rebuild
lead context from accepted state, current observations, unresolved decisions
and next actions. Operator takeover pauses broker continuation and is recorded
as an authenticated command; a native TUI is not an invisible second scheduler.

Artifact publication writes and verifies immutable bytes before publishing
their manifest reference; partial publication must not create accepted dangling
references. Acceptance binds candidate, integration base and test environment.
Moving the base invalidates any verification that does not cover the new
integration candidate. Backups/restore must cover domain data, artifacts and
Temporal histories coherently, with explicit recovery-point limitations. A
restore does not justify reissuing unresolved external operations.

## Phased implementation and observable exit criteria

| Phase | Vertical slice | Exit evidence |
| --- | --- | --- |
| Capability discovery | Pin actual Trae schema and probe contracts using private fixtures | Start/steer/interrupt/restart behavior and ambiguity limits measured; no assumed parity |
| Transactional kernel | One task, authenticated command, admission/fence, budget, outbox and fake adapter | Duplicate commands, concurrent owners, crash before/after dispatch and stale reports handled without accepted-state loss |
| Temporal lifecycle | The same task uses a workflow, broker Activities, bounded retry, yield/approval and cancellation | Workflow-worker restart, replay, pending-wakeup recovery and Continue-As-New pass with exact domain transition counts |
| Supervised Trae execution | Private gateway/runtime and isolated attempt workspace | Known execution adopted after detach/restart; unknown start never blindly duplicated; effects inspected after interruption |
| Replaceable lead and mailbox | Lead proposals, worker questions, exact-candidate verifier, reconstructed lead context | Kill lead while work runs; replacement reconstructs accepted plan and pending questions without old conversation |
| Acceptance and operator view | Human-controlled integration path, CLI/tmux projection and exact-candidate evidence | Changed base forces new verification; old generation and unauthorized operations rejected; view detach leaves execution intact |

Start with one authenticated assignment flowing through a real PostgreSQL
transaction to an outbox and a controllable fake runtime. The next slice adds
Temporal to that exact path. A thread-list dashboard or chat mesh is not the
first slice. Add the real Trae adapter only after its safe probe contract is
reviewed; add placement through Armada after local recovery is proven.

First durable orchestration acceptance includes PostgreSQL + Temporal + the
gateway contract. Fake adapters prove broker invariants; live adapter acceptance
is explicitly separate. No PostgreSQL-only demo is described as the completed
architecture.

Keep Markdown specs/tickets and human integration authoritative under current
policy while this RFC is under discovery. A broker-authority migration and an
integration service that performs merges require explicit versioned policy
decisions. Approval for one integration operation is not approval to change
who performs integration.

## Required recovery matrix

| Injected failure | Required invariant / observable result | Proof needed |
| --- | --- | --- |
| All views close | Broker/gateway ownership remains; reattachment starts no turn | Fake fixture, then real Trae/view detach |
| Lead conversation disappears | Mission/task authority unchanged; new lead receives effective decisions and pending questions | Fake context reconstruction, then fresh Codex/Claude sessions |
| Broker dies after domain commit | Outbox intent survives; next dispatcher applies command once at domain level | Real PostgreSQL + fake runtime |
| External start succeeds, reply lost | Outcome becomes unknown; lookup/adopt or quarantine; no blind retry | Fake fault injector, then bounded actual Trae probe |
| Activity worker dies after external effect | Journal checked on retry; external effect not assumed transactional | Real Temporal/PostgreSQL + fake runtime, then actual Trae |
| Runtime server replaced | Binding/storage identity and epoch reconciled; surviving tool effects inspected before resume | Fake lifecycle model, then actual private Trae server/effects |
| Old worker returns after reassignment | Audit retained; stale-generation acceptance rejected; shared effects fenced only through enforcing services | Database fake-worker fixture plus real containment/shared-resource checks |
| Message resent or steer races completion | Message retained with correct presentation/ack state; no fabricated receipt | Fake message fixture, then actual Trae steering race |
| Task blocks on a question | Durable wait; no repeated model polling; answer wakes an eligible episode | Temporal fake-runtime fixture, then live bounded agent turns |
| Cancellation and timeout overlap | Idempotent stop/reconciliation; budget stays reserved while effects are uncertain | Fake runtime and actual harmless namespace-contained processes |
| Continue-As-New races a wakeup | Pending durable commands remain processable; domain state is not duplicated | Real Temporal/PostgreSQL fixture |
| Integration base changes | Prior candidate evidence cannot accept a different integration tip | Real disposable Git repos; no shared branch movement |
| Database/artifact restore | Missing or stale evidence is visible; unknown effects remain quarantined | Real private backup/restore fixture, then supervised-runtime reconciliation |

## Decisions still needed before a build spec

1. Lock the service/worker language after a pinned Temporal SDK feasibility spike.
2. Choose the first runtime adapter/probe budget and minimum reconciliation
   capability required for live dispatch.
3. Resolve broker-domain authority versus the current Markdown tracker floor,
   and human integration versus a narrowly authorized integration service.
4. Approve first-slice scope/test seams and deployment/lifecycle assumptions.

The map owns these decisions and their blockers. Detailed migrations, production
operations, bootstrap import of existing threads and the observer UI follow as
the frontier clears. Do not publish implementation tickets as ready while
these prerequisites remain open.
