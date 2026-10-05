# SQLite controller and MAC baseline

This checkpoint records the implementation landed in the repository. It does not
accept the complete product baseline. Source read: SQLite candidate483f1bc,
MAC/operatorabe8715, execution evidencec498c38; original maina21316f.

## What is implemented

The local controller uses SQLite and a Unix socket. It authenticates commands,
checks expected revisions, deduplicates retries and records task, attempt,
message, question, operation and evidence state. Two global worker slots limit
concurrent admitted attempts. Runtime effects are journaled for reconciliation.

Codex adapters bind explicit server/thread/store identities. Supervised attempt
processes use private environments. Kata supplies the visible work ledger and
versioned task input; the controller owns execution authority. Git supplies
isolated attempt repositories, exact candidate snapshots, independent verifier
materializations, acceptance and conditional fast-forward landing.

The MAC tools provide a read-only terminal dashboard, controller/Kata/workspace
views and a tmux client manager. They read explicit public exports and retain
source times and digests. The manager preserves historical clients and can
reference already-open scoped observers without replacing them. Missing or stale
bindings cannot launch duplicate clients. The retained Codex monitor records
bounded public-action batches, coverage/cursors and context-efficiency evidence.

## Code map

| Path | Responsibility |
| --- | --- |
| crates/sqlite-controller/src/service.rs, query.rs, schema.sql | Authenticated command transactions, SQLite state and scoped queries |
| crates/sqlite-controller/src/runtime.rs, supervisor.rs | Attempt admission, process ownership and uncertain-effect reconciliation |
| crates/sqlite-controller/src/codex.rs, native_ws.rs, native_ui.rs | Native thread/turn bindings and selected-model delivery |
| crates/sqlite-controller/src/kata.rs | Source admission and visible Kata synchronization |
| crates/sqlite-controller/src/git.rs, git_workspace.rs | Candidate, verification and landing lifecycle |
| crates/sqlite-controller/src/control.rs, view.rs, monitor.rs | Lead controls, public projections and advisory monitoring |
| tools/agents/sqlite_baseline.py, sqlite_responses_relay.py | Disposable real-runtime trial and host relay |
| tools/agents/mac_dashboard.py, mac_state_view.py, mac_tmux.py | Operator TUI, state views and preserved native clients |

## Current demonstrated scope

Tickets01–04 are accepted within their recorded scope. Ticket05 retains the
open test-fixture timing finding. Ticket06 and the assembled baseline are not
qualified. The old c8403c trial proved one solo candidate, verification,
acceptance and physical fast-forward. It did not prove two useful concurrent
worker episodes or stale-base reintegration.

The retained replacement84eb5a runs source6f074c3. Its lead admission failed
with serverOverloaded before any worker attempt. One same-relay gpt-6.1-sol probe
succeeded. Candidate483f1bc preserves explicit model selection; independent
review precedes a coordinator-admitted fresh live run. All prior trial services,
views and evidence remain preserved in corenius-team. WorkerA/B panes are status
views; they are not proof of native worker clients or active execution.

## Use and verification

Build/test the controller with:

```sh
cargo test --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml
cargo build --locked --offline --manifest-path crates/sqlite-controller/Cargo.toml
```

Its README specifies private storage and authenticated socket requests. Start a
fresh trial only through the coordinator-owned runner/admission path. Attach to
the existing operator session with `tmux attach -t corenius-team`. Public export
paths and live authority belong to the coordinator's CURRENT projection; a Git
snapshot of that projection is a dated observation.

The richer PostgreSQL/Temporal roadmap and rejected variants remain historical
source/evidence. They are not enabled by this landing. Git is the current
workspace implementation; preferred jj support and root-Insula deployment still
need their dedicated qualification. No repair count or approval is reset.
