# Ticket 10 admission kernel implementation handoff

Status: repair round 1/3 candidate; awaiting independent re-review and primary integration. No ticket resolution, commits, pushes or deployment performed.
Source commit: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Worktree: `/data02/home/philip.yang/workspace/corenius-kernel-admission`, branch `codex/kernel-admission`.
Canonical SPEC, ticket10, approved-baseline decision and bounded execution brief read in the reference-design worktree on 2026-10-01.

## Candidate scope and interface

New isolated crate `crates/team-control-plane`, its explicit migration, PostgreSQL fixture suite and `tools/agents/checks.d/70-team-control-plane.sh`. Root Cargo configuration stays unchanged.

Public Rust methods: `migrate(dsn: &str) -> Result<()>`,
`Broker::connect(dsn: &str, recovery_epoch: Uuid) -> Result<Broker>`,
`bootstrap_owner(&self) -> Result<String>`,
`rotate_epoch(&self, bearer: &str, new_epoch: Uuid) -> Result<String>`,
`execute(&self, bearer: &str, envelope: CommandEnvelope) -> Result<CommandResult>`,
`snapshot(&self, bearer: &str, query: SnapshotQuery) -> Result<Snapshot>`.
`token_hash` computes SHA256 for separately generated bearer grants. `http::router`
provides the authenticated HTTP facade.

Command variants: CreateMission, CreateTask, SetDependencies, AdmitTask,
GrantCredential, AssignAttempt. Envelope fields: schema_version, key,
expected_revision, expected_generation, recovery_epoch, command. Result fields:
schema_version, disposition, revision, generation, recovery_epoch, attempt_id,
reservation_id, operation_id. Read queries: Mission, Task, Command. HTTP routes:
POST /v1/commands; GET /v1/snapshots/{mission|task}/{id};
GET /v1/commands/{principal}/{family}/{key}. CLI: migrate, bootstrap,
rotate-epoch, serve; environment supplies DSN and external epoch.

Authenticated owner binds finite mission policy and approved Git reference,
creates immutable typed task specs and explicitly approves an exact spec digest.
Scoped leads can assign; workers cannot. Grants store only hashes and DB-time
expiry. Checks for active deployment epoch, credential expiry/generation and
current role/scope happen before returning idempotent responses. Explicit epoch
rotation invalidates old brokers/grants and forbids epoch reuse from retained
history. Bootstrap cannot reactivate an existing deployment.

Admission transaction persists cumulative attempt/episode reservations,
capacity, sole attempt owner/generation, prepared operation, append-only event,
outbox and recorded result atomically. Rejected domain commands also have a
retained disposition; correcting a rejected command requires a new key.
Administrative fixture-trigger failures roll back every mutation including
budget/capacity, attempt, reservation, operation, task, event, outbox and command.
No external operation executes in ticket10.

## Initial candidate verification evidence

Raw logs and micro-plan are gitignored under `.scratch/team-control-plane/`.
No credentials or DSNs appear in this report or deliberately captured logs.

- Initial public-seam behavioral red: `red.log`,
  `admission_is_durable_atomic_and_idempotent` failed at owner mission creation
  with TemporarilyUnavailable against the compile scaffold. Underlying Cargo
  returned 101; an outer shell used `tail` after capturing the log and returned
  0, not the fixture runner. Final runner propagates Cargo failures.
- Targeted red regressions: `red-cache-scope.log` (cached wider-authority replay),
  `red-rejected.log` (unrecorded rejection became an admission),
  `red-transport.log` (remote NoTls configuration was not rejected), and
  `red-command-read.log` (task-scoped cached command read crossed its scope).
  Each named regression failed with Cargo101 before the associated fix.
- Fresh final `bash crates/team-control-plane/tests/run-postgres.sh`:
  12 integration tests passed, 0 failed/ignored, real PG; `green-final.log`.
- `cargo fmt --manifest-path crates/team-control-plane/Cargo.toml -- --check`
  passes. `cargo clippy --manifest-path crates/team-control-plane/Cargo.toml
  --all-targets --locked -- -D warnings` passes; `clippy-final.log`.
- Fresh full `sh tools/agents/check.sh` returned 0 and `check: all green`;
  final gate includes all 12 real-PG tests, 114 Python tests and 71 existing
  root Rust tests. Existing Python worker tests emitted ResourceWarnings but
  none failed. `repo-gate-final.log` retains the final full result.
- An intermediate full gate failed only the new PostgreSQL fixture with
  SQLSTATE57P03 while the server was starting. `repo-gate-readiness-red.log`
  preserves that failed gate. Pinned image entrypoint inspection confirmed its
  temporary init server runs with `listen_addresses=''` (line310); readiness
  had checked its Unix socket. The runner now checks explicit container-local
  TCP (`pg_isready -h 127.0.0.1 -p 5432`), waiting for the actual published
  server. Fresh full gate after that correction passes.

Named integration tests: admission_is_durable_atomic_and_idempotent;
authority_expiry_scopes_generations_and_epoch_rotation;
dependencies_validation_and_immutable_approval;
distinct_assignments_capacity_and_cumulative_budget;
rollback_at_every_admission_write_stage;
http_auth_forged_fields_and_reconnect;
a_cached_command_never_restores_wider_authority;
rejected_command_is_durable_and_cannot_be_reused;
token_expires_while_waiting_for_the_transaction_lock;
schema_and_transport_fail_closed; command_snapshots_respect_task_scopes;
episode_budget_and_competing_capacity_are_independent_limits.

Fixtures use PostgreSQL17.6 Debian17.6-2.pgdg12+1, amd64, pinned manifest
`postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3`.
Containers have unique names, tmpfs database data and loopback ephemeral ports.
Readiness/waits are bounded; EXIT cleanup stops/removes the container, including
failed red tests. After the final gate, `docker ps -a --filter name=corenius-kernel-pg` returned no containers; the entrypoint inspection container was also auto-removed.

Dependency pins: tokio-postgres0.7.18, Tokio1.52.3, axum0.8.8. Locked serde1.0.229,
serde_json1.0.151, uuid1.26.1, sha2 0.10.9; Rust1.95. Lockfile SHA256:
`02e4c2a68d544ec16ee5541fba1fba188d07938353a82ee1c6e1ef1cf38af7f1`.

## Limits and dependent-ticket handoff

Global transaction advisory locking is deliberate initial single-host
serialization, not a throughput claim. DB clients open independent connections;
no production pool exists. NoTls permits only loopback TCP/local Unix sockets;
CLI serves loopback. Remote TLS/auth deployment is outside current support.
Bootstrap and migration require trusted local administrator database authority.
Credential/token/DSN values are excluded from error surfaces and Broker Debug;
bootstrap/rotation intentionally emit the one-time new token for secure capture.

Task specs retain immutable content-addressed dependency/spec history and one
current approved version. Dependency changes before admission create a new spec
digest; other specification amendments, runtime episode starts, release/cancel/repair/completion and
accepted-dependency transitions require later tickets/migrations. Dependencies
prevent assignment while unresolved; this crate has no command that can falsely
mark their outputs accepted. One episode is reserved per admitted attempt;
max_repairs/deadlines are retained policy for subsequent lifecycle work. No
runtime is called and no reservation is automatically released.

Untested here: PostgreSQL server crash/WAL durability or backup/restore,
privileged-role separation against arbitrary direct SQL, production TLS,
production pool/load behaviour, artifact acceptance, gateway transport fencing,
Temporal replay, supervisor shutdown or live provider recovery. External epoch
identity must remain outside restored DB state; operators must establish a fresh
identity before resuming after restore. This ticket does not establish ticket24
restore safety or infer stopped external effects from a restored database.

Independent review and primary integration remain required. Passing tests do not
resolve architecture/policy or authorize commits/pushes/merge.

Candidate diff is reviewable with `git diff` (intent-to-add entries only;
no content commit). `git diff --check` passes. Patch and duplicate handoff:
`.scratch/team-control-plane/ticket10.patch` and
`.scratch/team-control-plane/ticket10-handoff.md`.

## Independent review repair round 1/3

All four reported defects were reproduced against the initial candidate before
repair. `round1-red.log` records 12 passing and these four failing real-PG tests:

- `missing_resource_rejection_stays_rejected_after_resource_appears`: owner
  assignment key K first returned NotFound; after the same task ID was created,
  approved and worker-granted, identical K allocated an attempt instead of
  returning its original rejection.
- `rejected_snapshot_retains_its_original_task_scope`: a broad lead's budget
  rejection for task A remained readable by a credential for the same principal
  scoped only to task B, because rejection scope was absent.
- `dependencies_change_spec_digest_and_retain_immutable_history`: changing A's
  dependency from B to C left the spec digest unchanged and erased its former
  dependency transition from the public read model.
- `event_sequences_have_typed_aggregate_namespaces`: a mission and task with
  the same UUID collided on event sequence 1 and returned TemporarilyUnavailable.

Repair changes, relative to the first handoff: `src/auth.rs`, `src/storage.rs`,
`src/domain.rs`, `src/validation.rs`, `src/admission.rs`, `migrations/0001.sql`,
`tests/kernel.rs`, crate `README.md`, and this evidence report. No dependency,
root Cargo or gate-shim change was needed during round 1.

Authorized owner commands targeting a missing task now reach domain rejection
persistence. Every applied/rejected command stores an explicit typed record
scope with owner/mission/task context. Missing-resource owner rejections retain
owner-only context permanently. Current command authority and retained scope
are both checked for cached execution; retained scope is also checked for
command snapshots. No missing scope is treated as a wildcard and no command
family prefix determines read authorization.

`TaskSpec` now includes a sorted dependency list (`dependencies`, optional/default
empty in JSON; explicit field in Rust literals). Dependency changes create
immutable content-addressed spec records and typed historical edge rows; the
current graph remains a reconstructible projection. Task reads retain current
spec, `spec_history` with prior digests/dependencies, and namespaced `events`.
Owner approval rejects a digest from before a dependency change. Event payloads
include the normalized command, result and exact current spec digest. A tighter
assertion requiring historical event-to-digest binding first failed in
`round1-red-event-binding.log`, then passed after explicit binding was added.
Identical content may reuse its original immutable spec record; each dependency
transition still has its own aggregate revision and append-only event.

Events now carry `aggregate_kind` (mission/task/credential), and uniqueness binds
kind, UUID and sequence. Entity tables retain their distinct identity namespaces.
Touched cumulative budget reads use named fields rather than anonymous positions.

Fresh focused suite: `bash crates/team-control-plane/tests/run-postgres.sh`
returned 0 with all 16 integration tests passing, 0 failed/ignored;
`round1-green.log`. `cargo fmt ... -- --check` and
`cargo clippy ... --all-targets --locked -- -D warnings` pass;
`round1-clippy.log`. The four red regressions and the strengthened event-binding
assertion now pass, including an explicit narrowed-credential cached execute
assertion for the retained rejection scope. Full `sh tools/agents/check.sh` returned 0 with `check: all green`,
including all 16 PostgreSQL tests, 114 Python tests and 71 existing root Rust
tests; `round1-repo-gate.log`. No assertion of independent review acceptance is
made by the implementer; primary owns re-review and ticket disposition.

After final verification, `docker ps -a --filter name=corenius-kernel-pg` returned
no containers. `git diff --check` passes. Fixed-base patch regenerated from
`963f58b` under `.scratch/team-control-plane/ticket10.patch`; SHA256 manifest in
`.scratch/team-control-plane/round1-source-manifest.sha256`. No commits/pushes,
extra agents or live model calls occurred. Current product source is frozen for
independent re-review; no unresolved-blocking acceptance claim is made.
