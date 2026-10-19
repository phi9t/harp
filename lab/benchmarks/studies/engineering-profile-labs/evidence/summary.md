# Fresh PostgreSQL and Temporal lab evidence

Recorded 2026-10-11T19:27:47.673544+00:00. Detailed learning content is integrated with
fresh native test evidence. Independent Standards and Spec review found no
remaining blocking findings in this database slice.
This is not Corenius production readiness or approval for Trondheim integration.

## Exact run and source

Source revision: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.
Private branch: `codex/engineering-postgres-temporal-labs`.
Scratch run: `/data02/home/philip.yang/.codex/worktrees/833c/corenius-labs/.scratch/engineering-profile-labs/20261011T192530Z-3138291`.
The final candidate contains documentation, a learning runner, and optional
PostgreSQL fixture identity/cleanup records. Product Rust and migrations are
unchanged. Input and artifact SHA-256 digests are in
[2026-10-11-run.json](2026-10-11-run.json).

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/run.py --lab all --timeout-seconds 1200
```

The runner invoked `run-postgres.sh --offline --test kernel -- --nocapture
--test-threads=1` and `run-temporal.sh --offline --test temporal -- --nocapture
--test-threads=1`. Both fixture scripts invoke `cargo test --locked`.
Cargo target: `/data02/home/philip.yang/.codex/worktrees/833c/corenius-labs/.scratch/engineering-profile-labs/cargo-target`.
Owned temporary root: `/data02/home/philip.yang/devx/tmp`.
Each suite had a 1200-second deadline. Neither reached it.

## Fresh results

| Gate | Result | Suite runtime | Fixture/build/cleanup runtime |
| --- | --- | --- | --- |
| PostgreSQL kernel | test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.36s | See native summary | 6.524 s |
| Temporal bridge/recovery | test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 18.97s | See native summary | 22.56 s |

The ignored Temporal case is the explicitly invoked worker subprocess entrypoint.
The parent starts it with `--exact temporal_worker_process --ignored`.
The suite's native recovery cases passed, so this ignored case is not a skipped
recovery scenario. A caught negative-test panic checks that an already-exited
worker cannot count as SIGKILL evidence.

The Temporal wrapper compilation is included in its fixture duration. The first
PostgreSQL warmup cold build took 66 seconds, then 3.58 seconds for the 16 tests.
The final run used only this clone's private target cache.

## PostgreSQL proof

- Duplicate concurrent command replies match and leave one Attempt, reservation,
  operation and outbox row.
- Distinct same-revision commands yield exactly one success and one
  `StaleRevision`; capacity and cumulative budget checks stay distinct.
- Trigger-injected failure at each of eight tables leaves no partial admission
  and no consumed Mission counters.
- A recorded rejection survives changed expected revision and a new connection.
  Changed command arguments under the same key conflict.
- Scoped, expired, wrong-generation and prior-epoch authority fail explicitly.
  A cached result cannot restore wider permissions.
- A token that expires during a transaction-lock wait is rejected afterward.

These are the native assertions documented in [POSTGRESQL.md](../POSTGRESQL.md).
They prove bounded database admission behavior, not one physical external effect.

## Temporal proof and concrete observations

Both native recovery tests use actual server/worker processes and SDK replay.
The first replays three fresh histories and three tracked v1 histories.
The mailbox case replays two fresh histories and the same three tracked inputs.
Thus 11 explicit SDK replay invocations passed, including five fresh histories.
The maximum actual Activity attempt in the retained invocation stream is
2.
The histories scan literal and decoded base64 payloads for fixture credentials.

Observed first task Run ID: `01a12c6d-b03f-7448-ae2d-5329c7105ffb`.
Observed successor: `b4e632cf-5aaf-4557-8932-45fdceebb9e1`, incarnation
2, initially `running`.
Terminal observation retains that successor and reports
`completed`.

| Fresh history | Events | Required native use |
| --- | --- | --- |
| `task-first.json` | 55 | SDK replay and source assertions |
| `task-terminal.json` | 33 | SDK replay and source assertions |
| `mission.json` | 12 | SDK replay and source assertions |
| `mailbox-first.json` | 52 | SDK replay and source assertions |
| `mailbox-terminal.json` | 26 | SDK replay and source assertions |

The tests additionally require cursor continuity, repeated-apply disposition,
actual SIGKILL before replacement, retained wait identity, lost-hint recovery,
real Continue-As-New, old-run observation rejection and terminal fencing.
The mailbox test waits for an actual unsatisfied Activity result before death,
then for an actual satisfied completion result after replacement. A database
registry write alone is not the history barrier.

See [TEMPORAL.md](../TEMPORAL.md) for the detailed procedure and distinctions.

## Runtime versions and pins

- rustc: `rustc 1.95.0 (59807616e 2026-04-14)`.
- cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`.
- temporal: `temporal version 1.7.0 (Server 1.31.0, UI 2.49.1)`.
- docker_server: `26.1.4`.
- PostgreSQL: `PG_MAJOR=17, PG_VERSION=17.6-2.pgdg12+1`.
- Image reference: `postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3`.
- Local image ID: `sha256:567d5d704a3e5a4864393b623a0d50eb1b5a4ad134cdac7cf19555cb2d8ffe9e`.
- Cargo.lock SHA-256: `acf6034494c031493bc0f2198ffac3a5c4ec3f5493c6e315a2cd83f31ff3a905`.
- Rust Temporal SDK/client/workflow/macros: exact 1.0.0; SDK Core resolves 0.9.0.

## Cleanup and failed launch

- postgres: owned container `corenius-kernel-pg-1791746731-3138423` removed: `true`.
- temporal: owned container `corenius-kernel-pg-1791746737-3141111` removed: `true`.
- Temporal fixture listener closed: `true`.
- PostgreSQL data lived in disposable container tmpfs. Histories, logs and the
  dev-server SQLite file remain only in owned gitignored scratch.
- No user database, source checkout, shared service or GPU was changed.

The earlier warmup was not accepted. Its 16 tests passed, but the shell wrapper
returned exit 2 after the wrapper was edited while Bash was running. The message
was `syntax error near unexpected token then`. The continuation read changing
source offsets. The owned container was absent before the next run.
No production semantics were changed to repair this operational mistake.
Two subsequent indexed runs passed with the wrapper unchanged during execution;
the final run above binds the formatted runner and final fixture hashes.
The warmup log digest and exit status remain in the machine record.

## Focused static and setup checks

Bash syntax checks passed for both fixture scripts. Ruff import/error checks and
format checks passed for the learning runner. Tracker lint and `git diff --check`
passed. Runner negative controls rejected deadline 0 and `TMPDIR=/tmp` with exit 2
before service setup. The documented history inspection command ran successfully.
The unrelated full repository gate was not run; this is a bounded parked-fixture
learning result. A clean-context reviewer checked the retained test counts,
source/artifact hashes, replay invocations, cleanup and proof limitations.

## What these runs do not prove

No production readiness, database HA, crash/power-loss storage recovery,
server restart, arbitrary workflow upgrade compatibility, large-scale scheduling,
real provider effects, native agent operation, or Insula execution was tested.
Offline Cargo dependency resolution does not establish host network isolation
or hermetic input closure. The deadline termination path was not fault-qualified.
Native runtime qualification is a separate companion lab.
