# Durable team control plane

Standalone Rust1.95 crate. PostgreSQL owns admitted state and evidence; Temporal
owns durable waits and workflow execution. HTTP schemas are version1. The
admission contract below remains the foundation for artifacts, mailbox, gateway
and supervised execution. Ticket16 assembles these with a model-free runtime;
its implementation and fixture results do not establish live Trae guarantees.

`Broker::connect(dsn, external_recovery_epoch)` validates schema readiness;
`migrate(dsn)` is a separate privileged operation. `bootstrap_owner()` initializes
an empty deployment and returns a random bearer once; only its SHA256 hash is
stored. `rotate_epoch(owner_bearer, fresh_external_epoch)` explicitly invalidates
old grants and returns a new owner bearer once. Epochs cannot be reused while
retained epoch history exists. External control identity must remain outside the
restored database; restore operators must supply a fresh identity and reconcile
external effects before gateway dispatch. A complete restore drill remains separate work.

`execute(bearer, CommandEnvelope)` derives actor identity from credentials and
checks database clock expiry, deployment epoch and grant generation before
idempotency lookup. Current command role/scope is checked before cached results.
Duplicates under the same principal/family/key return the durable applied result
or rejection. Changed typed command arguments conflict. Expected revision is a
first-application precondition; correcting a rejected command needs a new key.
Dependency lists are sorted before hashing. Every applied/rejected journal record
retains an explicit owner/mission/task scope checked on both cached execution and
snapshot reads; missing-resource owner rejections keep owner-only context. Typed specs include dependencies and have immutable content-addressed history.
Changing dependencies creates another immutable spec record/digest plus retained
historical typed dependency edges; current edges are a reconstructible projection.
Task read models expose current spec, spec_history and namespaced events. Owner
approval binds their SHA256 digest. `snapshot` reads are also authenticated and
scoped. Snapshot timestamps use database time.

Commands: CreateMission, CreateTask, SetDependencies, AdmitTask,
GrantCredential, AssignAttempt. GrantCredential takes a pre-generated bearer hash
so no command/result/journal persists plaintext bearer. Generate secrets with a
cryptographic random source; this crate's bootstrap uses two UUIDv4 values.
Workers cannot self-assign. An owner approves specs; scoped leads may assign ready
work. Worker grants bind mission/task/next generation and are usable only once
that generation is current. A dependent task remains nonassignable until a later
ticket supplies the accepted-dependency lifecycle; this admission-only crate has
no admission command to claim dependency completion; ArtifactService owns acceptance.

Assignment reserves one cumulative attempt and one episode, capacity, an attempt
and prepared operation, event and outbox in one transaction. Reservations remain
held: there is no release, repair or runtime completion in ticket10. A global
transaction advisory lock gives consistent single-host serialization across all
broker commands/reads; independent connections are used for each call. This
favours kernel correctness over throughput. Five-second lock and ten-second
statement bounds turn contention into temporarily_unavailable, never a duplicate
admission. Production scale/pooling requires separately reviewed changes.

HTTP: POST `/v1/commands`, GET `/v1/snapshots/mission/{id}`,
GET `/v1/snapshots/task/{id}`, GET `/v1/commands/{principal}/{family}/{key}`.
Every route requires `Authorization: Bearer ...`. Unknown JSON fields reject
forged role/sender data. Errors return stable codes without database errors,
tokens or DSNs. Broker Debug excludes DSN. Read models expose retained spec,
attempt, reservation and prepared-operation records, not credential hashes.

The CLI reads `TEAM_CONTROL_PLANE_DSN` and
`TEAM_CONTROL_PLANE_RECOVERY_EPOCH` from its environment. Run with
`cargo run --manifest-path crates/team-control-plane/Cargo.toml -- migrate`,
then `-- bootstrap`, then `-- serve 127.0.0.1:8080`. Bootstrap intentionally writes
the new bearer only to stdout for secure capture by the local administrator;
do not put that output in public logs. `-- rotate-epoch NEW_UUID` additionally
reads `TEAM_CONTROL_PLANE_OWNER_TOKEN`. CLI migration/bootstrap/rotation are
privileged local administration, not exposed through unauthenticated HTTP.
The CLI binds loopback only; the client accepts only loopback TCP or local Unix
socket PostgreSQL DSNs with NoTls. Remote PostgreSQL/TLS and remote HTTP deployment
are not supported. Do not expose this loopback fixture server to public networks.

## Verification

Docker daemon access, Rust1.95/Cargo and access to the pinned PostgreSQL image are
required. Run `bash crates/team-control-plane/tests/run-postgres.sh` from the repo,
or `tools/agents/check.sh` which invokes the dedicated checks.d shim. The runner
uses a uniquely named auto-removed container, loopback ephemeral port and tmpfs;
readiness is bounded, the container is stopped on exit, and cargo failures
propagate. Independent test fixtures default to two concurrent libtest cases
because each owns several database clients and SDK workers on the shared
100-connection PostgreSQL server; set `RUST_TEST_THREADS` explicitly to override.
Tests retain their own concurrent physical workloads and capacity races. When
Temporal evidence is requested, PostgreSQL logs and sampled connection counts
are retained before container cleanup.
Database tests fail explicitly when no fixture DSN is supplied;
there is no ignored or in-memory fallback. Direct administrative SQL inside
fixtures installs constraint-trigger failures; no production test callbacks exist.

`bash crates/team-control-plane/tests/run-temporal.sh --offline` additionally
starts owned loopback Temporal and runs the complete PostgreSQL/Temporal suite,
including actual supervisor capsules, SDK worker restart and retained-history
replay. Persistent-use readiness, backup/restore, production role separation
and live-provider recovery require separate evidence.

## Mailbox and replaceable lead

`MailboxService` is the durable command/read seam. HTTP exposes
`POST /v1/mailbox/commands`, scoped messages/records and bounded mission context;
`POST /mcp` implements the pinned MCP 2025-11-25 Streamable HTTP JSON transport.
Initialize, initialized notification and the returned session ID precede tool or
resource requests. Both facades authenticate the caller directly on every
request. `TEAM_CONTROL_PLANE_ARTIFACT_ROOT` must be an absolute owned directory
when configuring the executable's ArtifactService tools and routes.

An Owner replaces the mission Lead using its expected Lead generation. Old Lead
credentials are fenced before command replay or reads; Worker/Verifier task
authority remains separate. Message persistence, presentation into an exact
input/episode, explicit recipient acknowledgment and question resolution are
independent immutable records. Sending an exact registered artifact reference
grants only its addressed recipients scoped access to those bytes. It does not
grant peer task/history access and does not require accepting the source task.

CaptureInput retains a bounded canonical packet with approved constraints and
unacknowledged messages. AdmitEpisode checks its content digest, caller, role,
current generation/epoch and event frontier. Previously unprocessed batches can
advance without buying empty polling turns. Legacy digest-only admissions are
explicitly `caller_asserted_logical`; they cannot bind a physical runtime turn.
Planning and independent candidate verification episodes spend the same finite
mission budget without acquiring Worker execution authority.

Owner/controller BindRuntimeTurn requires a confirmed gateway Start, the exact
captured packet and adapter-proven native turn identity. PrepareDelivery retains
the message, current episode, workflow incarnation and optional exact canonical
decision/consumed approval before dispatch. The gateway checks these in its
serialized durable claim after acquiring the runtime queue. Physical steering
requires an adapter's atomic expected-turn capability: a completed turn leaves
pending input for a later bounded episode; lost replies remain uncertain and
are not retransmitted. These capabilities are proven only by the disposable
non-idempotent subprocess fixture, not by a live provider. An already claimed
in-flight RPC can finish after authority closure.

Yield atomically records a checkpoint, typed questions/task dependencies,
explicit unresolved Execution references and durable waits. Accepted13's existing
Activity reads, outbox, timer and Continue-As-New recover attention after missed
signals. Task acceptance remains the ArtifactService transition; mailbox notices
never accept tasks. Approval records bind normalized arguments, an executor,
epoch/generation, expiry and one consumption; they record permission, not an
external Git/publication effect.

## Assembled model-free execution and operators

EpisodeGateway journals role-neutral planning, implementation and verification
operations before physical dispatch. Supervisor allocates private Git workspaces,
delivers exact captured input through a sealed descriptor and records independent
exit, log and namespace-stop evidence. A verifier materializes the immutable
current candidate and its authorized integration base; allowed paths still derive
from the admitted Task input. Changing candidate/base invalidates acceptance
evidence. An already allocated historical snapshot may execute, but its report
cannot authorize the changed current candidate.

The CLI uses `TEAM_CONTROL_PLANE_TOKEN` for `status`, `inspect-execution`,
`cancel-execution`, `reconcile-execution`, `release-execution`,
`inspect-episode-operation`, `reconcile-episode-operation`,
`reconcile-gateway-usage` and `takeover-episode-store`. Execution commands also
require `TEAM_CONTROL_PLANE_WORKSPACE_ROOT`. Summary views are bounded and signal
overflow; authenticated exact lookups remain independent of controller ownership.

Unknown native effects and unknown usage are separate conditions. Stopped local
effects alone cannot refund unmeasured usage. Model-free zero requires the exact
original operation and input receipt from a registered singleton transport with
pinned native/workspace roots. Usage queries do not issue another Start or
transfer control. There is no Owner command to silently clear unknown usage.

Registered episode transport takeover requires independent complete native,
workspace and process inventory and refuses unfinished workloads. Legacy gateway
controller replacement remains unsupported: restart safely quarantines the
retained store and reservation rather than launching another execution. This is
a stopped recovery disposition, not successful legacy transfer. No bearer enters
workflow history, and no live provider, cross-host exactly-once or production
failover guarantee is asserted.

### Canonical attempt cancellation and control recovery

Public broker commands `cancel_attempt` and `reconcile_attempt` bind an exact
Task/Attempt, revision, generation, recovery epoch and idempotency key. Only the
Owner or that current Attempt's Worker controller may request cancellation.
Cancellation commits `desired_cancel` and `cancellation_requested` first and
revokes future episode admission, native/physical dispatch and result/acceptance
mutations. Existing physical work then follows the strict supervisor stop path.
Reconciliation records `cancelled` only after stopped effects are established;
uncertain effects or inventory record `quarantined`. Unknown usage retains its
reservation even when physical effects stop. Cancellation does not refund capacity
implicitly. A current Worker retains scoped execution inspection, logs, results,
history and bounded cancel/reconcile/release authority; cancellation fences
dispatch separately from credential and generation validity. Workspace `cancel` is a physical stop operation; use the canonical
Attempt command to revoke the Attempt's continuing authority.

`GET /v1/kernel/status` returns a database `snapshot_time`, the recovery epoch,
and each Mission's separate `control_revision` and `dispatch_paused` state.
`POST /v1/kernel/pause` accepts the current control revision, recovery epoch,
idempotency key and rationale. A fresh authenticated client can recover a lost
pause response from status or an authenticated exact Mission snapshot and resume
using that durable control revision;
Mission intent revisions are not control revisions.

Exact Mission snapshots expose `dispatch_paused` and `control_revision` even
when the Mission falls outside the bounded 32-row status page. Execution status
includes both episode-owned and legacy Attempt-owned workspaces. Legacy scope
comes from the Attempt and Task, with no invented episode identity; capacity
comes from the physical reservation or episode capacity owner.

Supervised verification evidence preserves raw output with exactly one field:
legacy `output` (a JSON string), or `output_bytes` containing
`{"encoding":"base64","data":"..."}`. Both fields, including null values,
are ambiguous and rejected. The decoded original bytes must match the trusted
monitor digest; command, candidate, exit status and authority bindings still apply.
Monitor finalization hashes every raw log byte incrementally with a fixed buffer.
Unreadable logs or incomplete stop evidence cannot establish releasable success.

An original unbound Attempt reservation and separately owned continuation slots
reconcile independently. Stopped continuation history does not consume the
original slot; unresolved effects, usage and execution checkpoints still hold
capacity, and cumulative admission budgets are never refunded.

Authenticated reconciliation/release can finalize a launched episode after its
original monitor crashes, including after epoch rotation under a current Owner. It requires an immutable exact launch/Stop observation,
fresh certified PID1 finalization, terminated monitor and launcher thread groups,
known effects/usage, and a stable securely opened retained log. Recovery records
`completion.kind=recovered_stop`, the original execution/monitor/epoch, and the
actual finalizer/current epoch/stop observation. Exit and signal remain unknown;
this result cannot pass verification or acceptance. Historical results without
`completion` retain original monitor-wait semantics. Result insertion is immutable
and preserves the stored winner across repeated or concurrent finalization.

Takeover securely classifies retained raw `.log` files without a size limit or
whole-file allocation. Native JSON receipts and bootstrap `.info` metadata retain
the 1 MiB bound; unsafe files, incomplete inventory and active work still refuse
transfer.
