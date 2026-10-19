# Ticket12 gateway-journal implementation handoff

Candidate only; acceptance and independent Standards/Spec review remain root-owned.
Read baseline commit: `963f58b31e518a8162dfe719e386ef9d9843321f` plus the
accepted ticket10 manifest. Implementation worktree: `corenius-codex-gateway-journal`,
branch `codex/gateway-journal`. No commit, push, PR, production change or model call.

## Implemented contract

Public Rust `Gateway::bind/prepare/dispatch/read/reconcile` and `RuntimeAdapter`
inspect/execute/observe support create, recover, start, steer and cancel. Intent
references the exact admitted operation, attempt, current task generation and
recovery epoch. Stable-ID duplicates read the same entry; changed arguments and
alternate create/start IDs for one admission conflict. Claims commit before I/O.
Immutable typed evidence accompanies prepared, dispatching, confirmed,
failed_known, outcome_unknown and quarantined states.

Physical ownership uses runtime kind + deployment + thread-store identity, with
thread uniqueness enforced in PostgreSQL. Full runtime/version/deployment/store/
endpoint/environment identity is checked against the adapter. Endpoint aliases
cannot acquire another physical owner. A local queue serializes dispatch and
reconciliation; a persistent store owner never expires. A replacement cannot
resume transport or infer absence while old control may survive: it quarantines.
There is deliberately no supervisor-confirmed takeover mechanism in this slice.

The additive 0003 migration has its own checked version and executes atomically
with core migration under the existing advisory lock. Admission v1 schema and
operation intents are unchanged. Owner credentials authorize writes; scoped
credentials can read their admitted operation. Every command/read validates
expiry, active recovery epoch, attempt/admission epoch and distinct current task,
attempt and operation generations. Workflow/gateway grants await ticket13.

## Behavioral evidence

`tests/gateway.rs` drives real disposable loopback PostgreSQL and a deterministic
Python subprocess external-effect inventory (zero models). Actual gateway test
processes are stopped at a flushed test boundary, then killed with SIGKILL;
`ExitStatus::signal()==Some(9)` is asserted. The controller and runtime inventory
are different processes.

| Killed controller boundary | Surviving journal | External effects |
| --- | --- | --- |
| Before intent | no operation | 0 |
| After committed intent | prepared | 0 |
| After dispatch claim, before runtime I/O | dispatching | 0 |
| Known-thread start effect/reply generated, before durable outcome | dispatching, thread known | 1 |
| After outcome commit | confirmed | 1 |
| Thread-create effect/reply generated, acknowledgment lost | dispatching, no thread ID | 1 |

Every nonconfirmed replacement remains explicitly quarantined; duplicate delivery
never adds an effect. Additional abandoned async dispatch-stack tests cover
before/after effects and same-controller authoritative reconciliation. They are
separate from the actual process-kill proof. Concurrent duplicate dispatch,
racing adoption, physical-thread alias, retained old controller, empty lossy
history, known failure, named missing capabilities, immutable evidence mutation,
expired grants, stale task generation and recovery-epoch rotation are exercised.
Migration rerun is idempotent. Existing admission kernel suite remains green.

Visible behavioral red: retained-controller regression initially returned
`failed_known` from an authoritative empty inventory when old control was not
confirmed dead; the test required quarantine and failed with that exact mismatch.
Green follows the owner-check/quarantine repair. Initial public-seam red was a
missing-module compile failure; fixture worker-registration and child driver
scheduling mistakes were corrected and are not claimed as behavioral reds.
The test uses two admitted attempts (capacity2, max8 episodes, max3 repairs,
30-second child boundary deadline); subprocess fixtures make zero model calls.

## Fresh checks and reproducibility

- `bash crates/team-control-plane/tests/run-postgres.sh --test gateway`: exit0,
  2 tests, including the OS-process child helper and the integrated fault matrix.
- `cargo fmt --manifest-path crates/team-control-plane/Cargo.toml --check`: exit0.
- `cargo clippy --manifest-path crates/team-control-plane/Cargo.toml --all-targets --locked -- -D warnings`: exit0.
- `sh tools/agents/check.sh`: exit0, `check: all green`; gateway and all16 existing
  kernel tests pass, repository Python gates and root Rust suites pass.

Raw logs and baseline-relative patch/hashes are under
`.scratch/team-control-plane/gateway12/`. The manifest enumerates exact changed
files relative to the accepted seed; source is frozen for independent review.

## Integration glue and limitations

Only shared glue: `lib.rs` exports `gateway`; `storage.rs` exposes crate-private
DSN/active-epoch/generation helpers and calls `gateway::migrate` inside the existing
migration transaction. Root must combine this with artifact11's analogous 0002
migration/export; no domain, authorization or HTTP refactor was performed.

No live Trae guarantee, native runtime deduplication, exactly-once external RPC,
multi-host active failover, supervisor-confirmed transport takeover or Temporal
integration is claimed. The native-deduplication profile field is informational;
this implementation never retries a dispatched RPC. Provider observations are
trusted adapter assertions validated against exact runtime identity; only the
fixture's complete persisted inventory is proven here. Journal grants and current
owner-only mutation authority are intentionally restricted until ticket13.

## Independent Spec repair round1 of3

Finding: creation confirmation and authoritative reconciliation accepted empty or
oversized thread IDs even though binding admission rejected them. Visible public
real-PG RED: malformed creation reply returned `confirmed`; expected
`quarantined` (`malformed-identity-red.log`, assertion mismatch). The shared
`valid_thread_id` predicate now applies to bind and all confirmation paths.
Malformed physical identities quarantine without adopting a thread or inferring
absence. Public regressions cover empty and 1025-byte replies, then empty and
1025-byte authoritative observations; binding remains unknown and subsequent
start intent rejects `invalid_argument`. The external effect inventory remains
exactly one. The complete fixture runs twice in isolated schemas, retaining all
six SIGKILL boundary types in each run.

Fresh repair1 checks: focused real-PG tests exit0 (2 tests, two integrated matrix
runs), fmt exit0, clippy all-targets `-D warnings` exit0, full repository gate
exit0 (`check: all green`). Logs: `repair1-focused.log`, `repair1-clippy.log`,
`repair1-gate.log` under the existing scratch directory. Updated frozen source
manifest/accepted10-relative patch: `changed-files-repair1.json` and
`changes-repair1.patch`. Only `src/gateway.rs`, `tests/gateway.rs` and this report
changed from the initial candidate. Independent re-review remains pending;
this is not ticket acceptance.

## Independent Standards repair round2 of3

The original external fixture keyed create-new effect files by operation ID, so
it implicitly deduplicated repeated RPCs despite the capability profile denying
native deduplication. Its helper also converted unexpected subprocess failures
to an unknown reply, and deliberate reply dropping concealed those failures.
Those weaknesses limited the original effect-count evidence; fresh repair2
evidence replaces that weaker proof. No production duplicate-dispatch bug was
observed, and production gateway code is unchanged from repair1.

Meaningful public-fixture RED: two direct calls with the same operation ID
produced one effect, while the test required two (`nonidempotent-red.log`).
Separate RED: a missing fixture directory made the subprocess fail, but DropReply
hid that unexpected failure (`subprocess-failure-red.log`). The external process
now assigns a new invocation/effect ID on every mutation, fsyncs each invocation
record and effect plus their directory, and never deduplicates by operation ID.
The fixture self-test proves two calls create two effects and two invocation
records; multiple-effect observation is inconclusive. Unexpected launch, status
and JSON errors now fail the fixture explicitly. A deliberate lost reply occurs
only after a successful external invocation/effect. Known rejection is an actual
recorded subprocess invocation with zero effects.

The real-PG integrated matrix now asserts durable mutation-invocation counts as
well as effect counts after duplicate dispatch, authoritative reconciliation,
retained/competing control, malformed identity and every actual SIGKILL boundary.
Duplicate and crash recovery paths retain zero/one invocations and zero/one effects
according to their boundary, even though a repeated RPC would create another
effect. Two direct fixture calls intentionally produce two effects only in the
separate fixture self-test.

Fresh checks: focused real-PG command exit0, four tests; fmt exit0; clippy all
targets `-D warnings` exit0; full gate exit0 (`check: all green`). Logs:
`repair2-focused.log`, `repair2-clippy.log`, `repair2-gate.log`. Updated full
accepted10-relative patch/manifest: `changes-repair2.patch` and
`changed-files-repair2.json`; delta from frozen repair1: `repair2-relative.patch`.
Changed from repair1: tests/gateway.rs, tests/support/gateway_fault.rs,
tests/fixtures/fault_runtime.py and this report. Source frozen for independent
re-review; no acceptance claim.
