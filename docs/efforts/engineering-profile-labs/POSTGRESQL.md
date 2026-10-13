# Lab 1: PostgreSQL as the transactional authority

Source read: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.
At that revision, the executable assertions are in
`crates/team-control-plane/tests/kernel.rs`.
After the semantic layout migration, the active relocated path is
`experiments/team_control_plane/tests/kernel.rs`.
The implementation anchors are `src/storage.rs`, `src/admission.rs`,
`src/auth.rs`, and `migrations/0001.sql` under that component.

## Learning model

An Attempt is admitted through a command, not through a worker's local memory.
The command uses a principal, operation family and key as its durable identity.
Its expected revision protects a proposed transition against concurrent changes.
Its current credential and generation protect authority against replacement.

The fixture uses two real PostgreSQL connections where concurrency matters.
A transaction-scoped advisory lock serializes broker mutations.
This implementation is deliberately coarse: passing the lab does not prove
scalable claim throughput or justify this lock strategy for another system.

Read `storage::Broker::execute` and `execute_in_transaction` in this order:

1. Begin a transaction and acquire `pg_advisory_xact_lock(784092110)`.
2. Check schema, active recovery epoch, credential expiry and command scope.
3. Look up the principal/family/key receipt and compare its argument digest.
4. For a new command, open `SAVEPOINT command_mutation`.
5. Apply the admission and insert its durable result in the same transaction.
6. Commit an applied result or a journaled domain rejection.
7. Roll back infrastructure, authorization and incompatible-schema failures.

Do not confuse these two outcomes. A domain rejection can be a durable receipt.
A database write failure must leave the attempted admission uncommitted.

## Execute the baseline

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/run.py --lab postgres --timeout-seconds 1200
```

The fixture starts a digest-pinned, disposable PostgreSQL container.
It binds an ephemeral port on `127.0.0.1` and stores data in container tmpfs.
The fixture's `migrate` calls affect only that database. Most cases own random
schemas. No DSN from a user database is accepted by the lab runner.

## Exercise A: one command, two concurrent callers

Read `admission_is_durable_atomic_and_idempotent`.
The fixture creates a Mission, Task, admitted specification and worker grant.
It sends the same `AssignAttempt` through two Broker instances with
`tokio::join!`.

Predict the outcome before inspecting the assertions:

| Observation | Required result |
| --- | --- |
| Both command replies | Equal durable `CommandResult` |
| `attempt_count` | 1 |
| `reservation_count` | 1 |
| `operation_count` | 1 |
| `outbox_count` | 1 |
| Same key, changed worker argument | `IdempotencyConflict` |
| Forged credential | `Unauthorized` |

Explain why retrying the same identity is different from creating another claim.
The receipt is committed with the admission. The second caller reads that
receipt after the first transaction releases the lock.

This proves one bounded database admission despite duplicate callers.
It does not prove one physical RPC effect at an external provider.

## Exercise B: distinct commands compete on one revision

Read `distinct_assignments_capacity_and_cumulative_budget`.
Two callers use different keys against the same Task at revision 2.
Exactly one succeeds; the other receives `StaleRevision`.

The same case then separates resource limits:

| Condition | Rejection |
| --- | --- |
| Mission capacity 1 already occupied | `CapacityExhausted` |
| Mission maximum attempts 1 already consumed | `BudgetExhausted` |
| Same Task revision already advanced | `StaleRevision` |

Explain why a new key cannot make an obsolete proposed state current.
Then locate the revision check and `FOR UPDATE` Task read in `admission.rs`.
Capacity measures current occupancy. Attempt and Episode budgets record
cumulative admissions. Their counts are not interchangeable.

These commands are transactional admission claims. The lab does not demonstrate
a generic lease queue, `SKIP LOCKED` scheduler or distributed claim service.

## Exercise C: rejected receipts stay rejected

Read `rejected_command_is_durable_and_cannot_be_reused`.
The command initially uses revision 1 while the Task is at revision 2.
It receives `StaleRevision`. The test changes only `expected_revision` to 2
and retries through another connection with the same identity.
The retry still receives the original rejection.

Then the test changes the command's Task and worker under that identity.
The reply is `IdempotencyConflict`.

The argument digest covers the normalized command body. It does not make
`expected_revision` an escape hatch from a recorded receipt.
A caller that intends a new transition needs a new command identity and current
authority. It cannot rewrite an earlier rejection.

Also inspect `missing_resource_rejection_stays_rejected_after_resource_appears`.
The later existence of a resource does not convert an old rejected identity
into a newly admitted command.

## Exercise D: atomic rollback at every write stage

Read `rollback_at_every_admission_write_stage`.
A PostgreSQL trigger raises a real write error at each table below:

```text
missions attempts reservations operations tasks events outbox commands
```

For every injected failure, the test requires `TemporarilyUnavailable`.
The Task remains ready at revision 2 and generation 0.
Attempt, reservation, operation and outbox counts stay zero.
Mission capacity and cumulative counters stay unconsumed.
The fixture drops each trigger before the next injection.

Explain why outbox insertion and command-receipt insertion belong to the same
transaction as the admission. A half-written outbox cannot be treated as
evidence that a complete admission occurred.

These are statement/transaction failures. They do not prove power-loss
durability, WAL recovery, disk-full handling, replication or backup restore.

## Exercise E: fence old authority

Read `authority_expiry_scopes_generations_and_epoch_rotation`,
`a_cached_command_never_restores_wider_authority`, and
`token_expires_while_waiting_for_the_transaction_lock`.

The cases distinguish four checks:

| Check | What the native test changes |
| --- | --- |
| Scope | A Lead credential tries to read another Mission or Task |
| Generation | A command supplies an obsolete/incorrect generation |
| Expiry | A grant expires, including while its caller waits for the lock |
| Recovery epoch | Explicit rotation invalidates all prior grants |

The lock-wait case grants a token for one second, holds the advisory lock,
starts the command, waits 1200 milliseconds, then releases the lock.
The command must fail `Unauthorized`.
Read `auth::authenticate`: expiry uses `clock_timestamp()` after lock
acquisition. A permission checked only before waiting would be stale.

The cached-command case retries through a narrower grant for the same principal.
It fails authorization before replaying the old result. A receipt must not
restore the caller's earlier privileges.

This is database admission fencing. It cannot revoke an already dispatched RPC
or prove that an external process has stopped. Temporal's dispatch boundary is
covered separately.

## Rerun one native case

For focused exploration, select one exact test and keep the private target:

```bash
python3 tools/temp_root.py -- env CARGO_TARGET_DIR="$PWD/.scratch/engineering-profile-labs/cargo-target" bash experiments/team_control_plane/tests/run-postgres.sh --offline --test kernel rejected_command_is_durable_and_cannot_be_reused -- --exact --test-threads=1
```

This focused command provides libtest output; the full runner additionally
indexes source inputs and checks retained cleanup evidence.
Do not run the test binary directly against a shared database.

## Completion questions

Explain all three differences: duplicate identity versus distinct claim;
revision versus authority generation; durable domain rejection versus failed
transaction. Use the native assertions above to support each answer.
Then read the evidence summary before moving to Temporal.
