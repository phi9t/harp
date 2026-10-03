# Specification: durable team control plane

Status: baseline approved; implementation in progress
Source commit read: Corenius `963f58b31e518a8162dfe719e386ef9d9843321f`
Scope: first single-host durable kernel, then a measured Trae integration
Owner input: [preserved RFC](../../rfcs/0001-durable-team-control-plane.md)
Evidence: [implementation investigation](../../rfcs/0001-durable-team-control-plane-implementation.md), [runtime research](research/trae-runtime-capabilities.md), [durability research](research/durability-sources.md), [Kata assessment](research/kata-assessment.md), [jj assessment](research/jj-assessment.md)

## Goal

Build a persistent engineering team whose lead and workers are replaceable:
assignments, decisions, questions, budgets and evidence survive loss of a
conversation, terminal or service, while uncertain execution is reconciled
before retry and completed work is accepted against an exact candidate.

Corenius should supply the executable reference and operating guidance for
Codex/Claude leads collaborating with TraeCLI workers. The control plane is
external to those reasoning processes. Success means safe recovery with
inspectable evidence, including a precise stopped state where recovery cannot
be established; it does not mean keeping every conversation alive.

## Approved implementation baseline

The owner approved the recommended baseline. See the
[decision record](decisions/20261001-approved-baseline.md). Live-probe budgets
and measured guarantees remain separately gated; no repository policy is changed.

1. Rust broker and supervised gateway; Rust Temporal workflow worker subject
   to a pinned SDK build/replay feasibility gate. If that gate fails, keep the
   same broker contracts and use a Python Temporal worker. Do not maintain two
   production workflow implementations.
2. PostgreSQL owns admitted execution state and accepted specification bindings.
   Git remains the approved planning/specification input for this repository.
   Importing a document does not approve it. Human integration authority remains
   in force. Replacing the Markdown tracker requires a separate policy change.
3. Deliver a PostgreSQL + Temporal kernel against a fault-injectable runtime
   before live Trae recovery tests. PostgreSQL-only preparation is not acceptance
   of the durable orchestration architecture.
4. Evaluate Kata as a planning ledger/operator interface before building an issue
   UI. It is optional to the first kernel and cannot be a second execution
   authority. Do not migrate current tickets or install hooks during this work.
5. Broker owns episode continuation. Native runtime goals are disabled for
   broker-managed bindings until an exclusive delegation contract is proven.
6. Use a VCS workspace interface with Git worktrees first; evaluate jj as an
   optional backend in fixture repositories. Evidence binds exact commits/trees,
   never mutable change IDs alone. No shared checkout conversion.
7. Start single-host and single active gateway for each runtime store. A gateway
   failover suspends dispatch until the previous controller and executions are
   reconciled. Availability does not justify launching an ambiguous operation.

## Public seams and ownership

| Seam | Responsibility | Must not own |
| --- | --- | --- |
| Broker command API | Authorization, revisions, state transitions, journal, budgets, outbox | Provider lifecycle hidden inside a model conversation |
| Workflow service | Durable waits, deadlines, bounded retries, reconciliation and verification scheduling | Independent authoritative task status |
| Runtime gateway | Capability profile, exclusive runtime control, thread/turn RPCs and observations | Task acceptance or new engineering scope |
| Environment supervisor | Attempt workspace, independently addressed jobs, shutdown and inventory | Inferred completion from terminal text |
| Artifact service | Immutable manifests, digest checking, publication and evidence lookup | Mutable accepted outputs |
| Planning adapter | Import approved Git/Kata revisions and publish execution projections | Starting turns or accepting candidates on issue closure |
| Lead/worker/verifier tools | Scoped proposals, messages, checkpoints, submissions and reports | Arbitrary app-server mutations |
| CLI/TUI | Read models and authenticated operator commands | A second scheduler or hidden takeover |

HTTP/JSON is the broker transport; MCP is an agent-facing facade over the same
commands. Version every external schema. Clients submit an idempotency key,
expected aggregate revision and structured arguments; principal identity comes
from authentication. Responses distinguish persisted, applied, rejected and
pending external outcome. Reads return aggregate revision and snapshot time.

Errors have stable codes: unauthorized, stale_revision, stale_generation,
idempotency_conflict, budget_exhausted, dependency_unmet, capability_missing,
outcome_unknown, artifact_missing and temporarily_unavailable. Retrying a
transport request does not imply retrying its external operation.

## Domain records and constraints

Use opaque stable IDs, database time for admission/expiry, per-aggregate
monotonic revisions, immutable specification revisions and append-only audit
events. Mutable projections are reconstructible from retained canonical records.

| Record | Required information and invariant |
| --- | --- |
| Mission | Objective, approved input digest, acceptance policy, budget policy, status, revision; amendments require explicit authorization |
| Task specification | Mission, objective, input artifacts, dependencies, allowed paths/capabilities, acceptance criteria, deliverables, revision digest; admitted revisions immutable |
| Task execution | Current admitted specification, lifecycle status, ownership generation, accepted candidate; at most one authoritative attempt per task |
| Attempt | Task/specification revision, worker identity, generation, workspace/environment, status, budget reservations, checkpoint; replacement creates a new attempt |
| Worker identity | Role, authenticated principal and mailbox; independent of thread/task identity |
| Runtime binding | Runtime kind/version, deployment, thread-store identity, endpoint, gateway epoch, thread ID if known, environment; a thread ID alone is insufficient |
| Episode | Attempt, bounded input digest, runtime turn ID if known, deadline and usage; no simultaneous episodes on one binding |
| Operation | Stable operation ID, kind, argument digest, target, authority generation, dispatch owner, intent/outcome/evidence; persisted before I/O |
| Command | Principal, idempotency key, argument digest, expected revision, disposition and recorded result |
| Message | Sender derived from principal, recipients, task/revision context, kind, reply-to, attention, delivery/acknowledgment/resolution records |
| Decision/approval | Principal, rationale, exact scope and normalized argument digest, expiry and consumption; changed arguments require new approval |
| Execution | Supervisor handle, target/environment, process/job identity, lifecycle, logs and effects; may outlive agent episode |
| Artifact | Content digest, immutable manifest, storage location, producing attempt, retention and integrity state |
| Verification | Candidate and base digests, specification/test-suite revisions, environment, commands, exit results/logs and verifier principal |
| Budget reservation | Mission/task/attempt counters and costs, allocation, consumed/unknown/released amounts; unknown usage stays reserved |
| Workflow registry | Aggregate, workflow generation, stable Workflow ID, observed Run ID, accepted command cursor and lifecycle |
| Event/outbox | Aggregate sequence, transaction ID, event version and durable delivery attempts; delivery is not application |
| Planning binding | Source kind/project/issue ID or Git path, immutable source digest and imported revision; no authority inferred from mutable labels |

Enforce uniqueness of command keys scoped to principal and operation family;
reuse with different arguments fails. Use transactionally checked generations
and revision preconditions for mutations. Dependencies cannot cross missions
without explicit references/authority; reject cycles on admission. A new task
revision supersedes scheduling inputs, not evidence already retained for audit.

Enforce unique authoritative control of the physical thread identity
(runtime deployment, thread-store identity, thread ID), not just binding-record
ID. Multiple historical/view bindings may refer to it, but only one current
control owner and active episode may exist. Adoption of a previously unknown
thread acquires this unique ownership transactionally; conflicting adoption
quarantines the binding and cannot start or steer a turn. Store identity must
be established before dispatch; endpoint aliases do not create new identities.

Do not rely on a generic metadata field as the only representation of attempts,
external operations, approvals or budgets. Those require typed constraints.

## Commands and permissions

| Command family | Caller and effect |
| --- | --- |
| mission.create/amend/pause/resume | Owner; creates or changes approved intent and cumulative limits |
| task.propose/plan.propose | Lead; records a proposal, not automatic authority to execute |
| task.admit/assign | Authorized lead or owner under mission policy; checks dependencies, specification and capacity, allocates attempt/generation/reservations |
| attempt.cancel/reconcile | Owner/controller; desired cancellation or explicit evidence-backed recovery decision |
| message.send/ack/resolve | Scoped participants; acknowledge only an actually presented message; resolution identifies answer or decision |
| checkpoint.record/attempt.yield | Current attempt; records evidence and dependency, ends current episode without accepting task |
| result.submit | Current implementer; immutable candidate submission, not task acceptance |
| verification.submit | Assigned independent verifier; records exact evidence, cannot edit implementer candidate |
| task.accept/repair | Authorized acceptance policy; checks current candidate, specification, reports and unresolved risks |
| approval.request/grant/consume | Scoped requester/owner/executor; grants bind exact operation arguments |
| integration.prepare | Authorized integration role; produces a gated candidate and handoff |
| integration.execute | Separate human authorization under current policy; no worker direct mainline mutation |
| operator.takeover/release | Owner; stops automatic dispatch and establishes exclusive controller transfer |

Worker credentials bind mission, task, attempt, ownership generation,
capabilities, deployment recovery epoch and expiry. Every mutation and dispatch
grant, including workflow/gateway grants, carries the current recovery epoch.
Recheck epoch, generation and expiry inside the domain transaction. Verifier grants
are distinct. Runtime provider credentials stay with the gateway. Localhost and
shared Unix UID are not authenticated role boundaries. Read scopes include
artifact/message permissions; no agent automatically reads all mission secrets.

## Task, attempt and episode transitions

Task states: proposed → ready → running → submitted → verifying → accepted.
Running can become blocked or awaiting_approval; failed verification becomes
repair_required. Cancelled and superseded are explicit terminal dispositions.
A dependency answer may unblock a task but does not extend budget or grant new
scope. Acceptance is terminal for a specification/candidate pair; later work is
an amendment or new task, not an invisible reopening of accepted evidence.

Attempt states: reserved → preparing → runnable → executing → yielded or
submitted; failed, cancelled, superseded and quarantined remain distinct.
Episode states: admitted → dispatching → active → completed; outcome_unknown
and cancellation_requested are nonterminal until reconciled. Runtime completion
only closes an episode. It does not close a task.

Cancellation first commits desired state and revokes future dispatch authority,
then asks the gateway/supervisor to stop. Cancellation is complete only after
runtime and environment evidence establish stopped effects. Deadline expiry
uses this same shutdown path. A failed stop enters quarantine, holds conflicting
capacity and prohibits workspace reuse. A superseded attempt can retain its
late report but cannot update authoritative completion.

## Admission and external-operation protocol

An admission transaction locks the relevant task/budget/capacity records,
validates revision, role, dependency and acceptance policy, increments ownership
generation where required, creates attempt/episode and operation intent,
reserves budget, appends events and writes outbox entries. Commit before sending
any RPC. No launch-then-register path is permitted.

Operation states are prepared → dispatching → confirmed, failed_known or
outcome_unknown. Reconciliation may establish confirmed, failed_known or
quarantined. Record target, request digest and authority before dispatch.

A single supervised gateway owns the live control connection for a given
runtime store. Its serialized operation queue obtains the durable dispatch
claim before sending. Duplicate deliveries return the same operation record.
An operation in dispatching after controller failure is uncertain even if no
reply was persisted. Never resend a non-idempotent start solely because its
claim lease expired or its Activity retried.

Database fencing protects broker writes, not the moment a provider receives an
RPC. On gateway ownership loss, close/fence its transport and stop new dispatch;
a replacement remains reconciliation-only until previous control is confirmed
inactive. Where that cannot be confirmed, quarantine the binding. Multi-host
active failover needs a separately proven transport/execution fencing mechanism;
it is outside the first slice. A check followed by network I/O is not an
exactly-once protocol.

Reconcile in this order: operation journal, runtime/store identity, authoritative
provider observations, environment/job inventory and immutable effect records.
Evidence must establish either the original execution to adopt, or absence of
an effect sufficient to permit a new operation. Silence, an empty lossy history
or missing acknowledgment cannot prove no effect.

Thread creation with a lost reply may lack any thread ID. Preserve that unknown
binding; do not create another thread automatically. Turn start against a known
thread is a different uncertainty case. If native deduplication is proven for
an exact runtime capability, retry only using its documented stable key and
scope. JSON-RPC request IDs and broker journal IDs alone do not provide this.

## PostgreSQL–Temporal bridge

PostgreSQL is authoritative for applied commands, transitions and external
operation results. Temporal owns durable control flow and timers. Workflow
code never directly reads the database, calls a model, launches Git commands
or invokes runtime RPCs; Activities call broker/gateway services.

Use a stable Workflow ID derived from aggregate ID and workflow generation.
Create the registry generation and wakeup outbox entry in one broker transaction.
A bridge worker starts or signals that generation. Duplicate start acknowledgments
are reconciled by exact Workflow ID, not by creating another ID. A workflow
Activity reads durable pending commands; signals contain IDs as wakeup hints.
Periodic durable timers also request pending work, so lost hints do not strand
commands. This is database scanning by services, not agent polling turns.

An apply-command Activity submits a stable transition ID and expected revision.
The broker commits the transition, command disposition and processing cursor
atomically. A retried Activity receives its recorded result. Cursor advancement
requires all earlier commands to have a recorded disposition, including rejected
commands; reject gaps rather than losing them behind a high-water mark.

Workflow input includes aggregate ID, generation and cursor; Continue-As-New
carries these and durable wait references. Read canonical state again through
an Activity after rollover. Old-generation Activities fail mutation preconditions.
A late command for a terminal workflow causes the bridge to inspect domain
state: return terminal rejection, or allocate a new workflow generation only
for an authorized reopening/new revision. Do not blindly restart a terminal
workflow. Commands during rollover remain in the DB until applied.

External dispatch Activities may be retried to query the same operation, but
must not repeat an uncertain provider start. Set explicit Activity timeouts,
retry classifications and cancellation behavior. Heartbeats carry operation
references, not assumed atomic rollback. Version workflow code and test replay
against retained histories before deployment. Keep tokens, terminal streams and
large logs out of histories; retain artifact references and milestone events.

## Messaging, yield and lead replacement

Store messages before delivery; use separate persisted, presented, acknowledged
and resolved records. Presentation references an episode and input digest. An
explicit acknowledgment comes from the addressed principal; delivery retries
may repeat presentation without manufacturing acknowledgments.

Idle workers receive bounded inbox batches in a new episode. Active steering
uses the exact expected turn ID only when proven supported. If that turn ends
before delivery, leave the message pending for the next episode. Urgent stop
uses cancellation, not a conversational request. Peer questions cannot revise
accepted interfaces or transfer task ownership.

Yield records pending dependency/question IDs, a checkpoint and active external
Execution handles, then ends the episode. Answers wake the workflow. Schedule
a replacement episode only under current scope and remaining cumulative budget.

A replacement lead receives a generated mission view: approved objective,
accepted plan/specifications, running and uncertain attempts, decisions,
questions requiring judgment, candidate evidence, constraints and budget.
The old conversation is optional evidence. Every accepted lead decision must
already have a canonical record. Lead loss does not reassign workers, refund
budget or erase unanswered questions. Coalesce events to avoid a lead turn for
every token/message. Lead planning and verifier turns consume the same mission
budget accounting infrastructure as worker turns.

## Environments and shared effects

Each attempt gets a separate workspace at an exact base commit. Record workspace
identity before launch. A worktree is collaboration isolation, not a security
boundary: permitted writes need a supervised capsule or stronger repository
isolation when shared Git data must be protected.

Environment supervisors allocate stable Execution handles before workloads
start, retain logs and process/job membership independently of the agent, and
can inspect/cancel workloads after lead/runtime restart. A launcher PID alone
is insufficient. Prove complete membership and shutdown across capsule/PID
namespace boundaries. Unreachable or unclassified survivors prevent workspace
reuse and conflicting launches.

Shared mutations use a service that transactionally checks current generation
and exact approval: publishing accepted artifacts, protected integration and
resource allocation. Arbitrary old shell writes are not fenced by a database
lease. Attempt-local output from a stale executor may be preserved, never silently
promoted to accepted output. Garbage collection must require authoritative
inactive status and a locked deletion reservation; creating workspaces before
registration cannot create a GC race window.

## Artifacts, verification and acceptance

Publish bytes to immutable storage, verify digest and finalize manifest before
creating a broker reference eligible for acceptance. A crash before DB reference
leaves an unreferenced object for delayed collection; a missing/corrupt referenced
object blocks acceptance. For single-host storage, atomic publication includes
file/directory durability, not only a successful rename.

Submission binds task/specification revision, attempt generation, candidate
commit or snapshot, base commit, deliverables and risks. Verification binds the
same candidate/specification plus test-suite revision, environment identity,
commands, exit results and retained logs. The verifier obtains candidate bytes
from the artifact/workspace binding, not just the implementer's summary.

Acceptance requires all mandatory checks to cover the current exact candidate,
resolved required decisions and an authorized acceptance action. Integration
base changes invalidate base-sensitive proof. Construct and gate the actual
integration candidate before handoff. Passing worker checks and CLEAN statuses
cannot substitute for this step. Current human mainline integration policy
remains effective; integration.execute is not enabled by this draft.

## Budgets and progress

An admitted mission must have an explicit finite budget policy: concurrency,
episode deadlines, maximum episode/attempt/repair counts and optional measured
cost limits. No implicit unlimited goal continuation. Reserve before dispatch;
include lead/verifier/replacement episodes and supervised resource allocations.
Release only known unused reservations. Unknown spend stays reserved until
reconciled or the owner records an explicit adjustment with rationale.

Use separate timestamps for controller heartbeat, runtime/model/tool activity
and meaningful evidence. Repeated failures without new evidence trigger bounded
repair or lead review, not indefinite continuation. Legitimate long jobs use
Execution handles and deadlines; silence alone is not failure. Capacity occupied
by an uncertain live execution stays occupied until safely resolved.

## Kata integration contract

Kata is a candidate implementation of the planning ledger and human issue UI,
backed by its own PostgreSQL schema. Broker execution tables remain separate;
sharing a PostgreSQL server does not create a distributed transaction between
independent APIs. Rust consumes its service API rather than assuming Go embedding.

First adapter mode imports an explicitly approved immutable issue/specification
snapshot and records its digest/source identity. Mutable issue title/body,
metadata, claim or close status cannot change an admitted task. Subsequent source
changes create a proposal requiring admission. Broker events project attempt,
attention and verification summaries back to Kata through an outbox with stable
mapping IDs and deduplication; failed projections cannot block broker recovery.
Projection commands must not loop back as new task admissions.

A Kata claim can request broker assignment; it is not an execution lease. A Kata
close is a human/agent completion claim; it cannot bypass broker verification.
If direct issue mutations are permitted, label the planning versus accepted
execution state clearly. Before making Kata authoritative for accepted task
specifications, prove revision capture, authorization, transactional stale-worker
fencing, closure policy and event replay against a pinned release. Its documented
host transaction fences are relevant, but they do not establish provider RPC or
filesystem fencing. Current repository tickets stay canonical until policy
migration is explicitly accepted.

The evaluation must compare reuse savings (CLI/TUI, relationships, search, MCP,
attention and scoped credentials) against adapter, synchronization and operating
cost. Keep Kata out of the kernel critical path if it cannot enforce the chosen
authority boundary. No custom issue UI until that evaluation is recorded.

## Workspace backend and jj integration

The workspace service exposes create_attempt, inspect, checkpoint,
freeze_candidate, prepare_integration and release. Its returned manifest binds
backend and repository-store identity, workspace identity, base commit, exact
candidate commit/tree digest, captured paths and conflict state. With jj, add
operation ID and change ID for recovery/navigation; these do not replace the
immutable evidence digest. Journal repository mutations before execution and
reconcile interrupted operations rather than assuming rollback.

Keep the initial Git backend compatible with existing lifecycle, evidence and
landing tools. Evaluate a pinned jj backend in fixture repositories. Each
attempt still has one writer and its own workspace. Because jj workspaces share
repository storage, workers must not gain unrestricted authority to rewrite
other attempts; use isolated stores or a mediated VCS service for that boundary.
A mutating VCS service needs explicit snapshot authority under current policy;
commands such as status may snapshot with jj and cannot be assumed read-only.

freeze_candidate stops conflicting writes or materializes an immutable tree,
rejects unresolved content/bookmark/change ambiguity, validates allowed paths
and ignored/private-file exclusions, and returns the exact immutable candidate.
The verifier executes against that candidate in its own environment. Rewriting
or changing ancestry yields a new candidate; retained verification cannot follow
a mutable change ID. Repository operation recovery does not roll back broker
state, external jobs or remote effects. Protected landing remains human-approved
and fast-forward-only regardless of local VCS.

Before adoption, prove stale workspace repair, divergent-operation handling,
crash recovery, candidate preservation after rewrite, Git export, capsule paths,
existing gate/evidence/GC contracts and protected landing. jj is not a kernel
prerequisite and no shared repository conversion is part of this specification.

## Operations and restore

Initial deployment is one broker, workflow worker, supervised gateway,
PostgreSQL, Temporal and immutable artifact storage. Process supervision is
independent of TUI attachment. Explicit readiness checks verify schema versions,
workflow compatibility, artifact access and runtime capability profile. A failed
check disables new admissions/dispatch while preserving read/recovery access.
Separate migration and runtime credentials. Start with one known deployment;
do not make unmeasured high-availability claims.

Back up domain DB, workflow state/configuration and referenced artifacts with a
recorded consistent recovery boundary. A restore starts with dispatch disabled,
a new recovery epoch and a reconciliation inventory. Allocate the recovery
epoch from control state outside the restored backup (or a new unguessable
deployment identifier), persist it before access reopens, and invalidate all
pre-restore credentials, grants and control connections. Reissue credentials
only after ownership reconciliation. Inventory all supervised environments,
provider stores and controllers, including executions absent from the restored
journal because they began after the backup. Such orphans consume conflicting
capacity and remain quarantined until stopped or explicitly adopted with
evidence. Unknown inventory coverage keeps the affected target closed.
Database rollback does
not roll back live external effects; every restored uncertain/active operation
must be reconciled before dispatch resumes. Restore drills must include a job
that survived the restored database boundary. Retention rules distinguish
redacted reference evidence from raw private traces and prevent collection of
referenced candidates/checkpoints. Production RPO/RTO and remote placement are
subsequent owner decisions, not inferred guarantees.

## Observable acceptance boundary

The public test seam is the command API, durable read model, gateway observation
contract and independent supervisor inventory. Tests inspect these and exact
artifacts, not implementation-private flags or terminal keywords.

| Scenario | Required observable result | Proof stage |
| --- | --- | --- |
| Duplicate command/Activity/outbox delivery | One admission, reservation and external-operation intent; same recorded response | Kernel |
| Concurrent binding/adoption aliases for one physical thread | Unique control owner; second owner cannot start/steer | Kernel, then live |
| Concurrent assignment or stale revision | One admitted owner; explicit loser rejection | Kernel |
| Crash after DB intent / before RPC | Safe recovery or unknown state; no blind launch | Kernel, then live |
| Provider starts but reply is lost | Adopt proven execution or quarantine; no repeated start | Kernel, then live |
| Thread creation reply lost with no ID | Unknown binding retained; no automatic new thread | Kernel, then live |
| Old controller survives lease expiry | New dispatch blocked until old control stopped/reconciled | Kernel, then live |
| Workflow replay / rollover / terminal wakeup | Pending commands applied once or rejected explicitly; no cursor gaps | Temporal kernel |
| Message races turn completion | Message remains pending or has recorded presentation; no synthetic ack | Kernel, then live |
| Lead/thread/TUI loss | Plan/questions/attempts recover from records; TUI closure does not stop owned work | Kernel, then live |
| Deadline/cancel through nested capsule | All effects stopped or quarantine with survivor evidence | Actual supervisor |
| Old attempt submits or edits its workspace | Authoritative submission rejected; artifacts isolated; no shared landing | Kernel and actual environment |
| DB/artifact publication crash | No accepted missing artifact; orphan is safely collectible | Kernel |
| Candidate/base/spec changes | Prior report cannot accept changed integration candidate | Kernel and fixture Git |
| Budget crash / unknown spend | Reservation retained; no budget bypass via replacement | Kernel |
| Kata edit/claim/close and projection loop | Proposal/projection only; no implicit launch/acceptance or duplicate binding | Optional adapter |
| jj rewrite/stale workspace/divergent operation | Immutable candidate remains identifiable; no stale proof or cross-attempt rewrite accepted | Optional VCS adapter |
| Restore with pre-restore credential and surviving post-backup job | Old grant rejected by recovery epoch; unrecorded job quarantined/adopted; no duplicate job | Restore drill |

A fake runtime proves broker protocol behavior, not Trae semantics. Live proof
must pin runtime version/schema, record capability observations and failures,
and preserve redacted run evidence. An unsupported observation path is a named
limitation and safe quarantine, not a claimed successful recovery. Demonstrate
both Codex and Claude lead replacement against the same controlled baseline
before calling the repository a multi-runtime engineering reference.

## Build sequence and exit

Use the [proposed implementation breakdown](implementation-breakdown.md).
Kernel acceptance requires real PostgreSQL transactions, real Temporal replay
and lifecycle tests, and a deterministic fault runtime. Live acceptance adds
actual provider and capsule behavior. Product rollout requires owner-approved
policy/configuration and measured restore/upgrade behavior. These are separate
milestones; a simulator or written spec cannot satisfy later ones.
