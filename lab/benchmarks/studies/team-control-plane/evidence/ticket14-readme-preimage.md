# Admission kernel

Standalone Rust1.95 crate for ticket10. PostgreSQL is the only state store.
Schema/HTTP version1. No runtime launches, workflow service or artifact acceptance.

`Broker::connect(dsn, external_recovery_epoch)` validates schema readiness;
`migrate(dsn)` is a separate privileged operation. `bootstrap_owner()` initializes
an empty deployment and returns a random bearer once; only its SHA256 hash is
stored. `rotate_epoch(owner_bearer, fresh_external_epoch)` explicitly invalidates
old grants and returns a new owner bearer once. Epochs cannot be reused while
retained epoch history exists. External control identity must remain outside the
restored database; restore operators must supply a fresh identity and reconcile
external effects before any later gateway dispatch. This crate performs no dispatch.

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
no command to claim dependency completion.

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
propagate. Database tests fail explicitly when no fixture DSN is supplied;
there is no ignored or in-memory fallback. Direct administrative SQL inside
fixtures installs constraint-trigger failures; no production test callbacks exist.

This kernel proves database admission contracts, not persistent-use readiness,
PostgreSQL backup/restore, production role separation, runtime starts, Temporal
replay, gateway fencing, supervisor shutdown or live-provider recovery.

## Mailbox and replaceable lead (ticket14 candidate)

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

Ticket16 must bind actual provider/supervisor adapters and role-neutral physical
planning/verifier execution. Execution references here are explicitly unresolved,
not validated Supervisor handles or stopped-process/refund evidence. No bearer
enters workflow input/history, no second scheduler is introduced, and no live
provider, cross-host exactly-once or failover guarantee is asserted.
