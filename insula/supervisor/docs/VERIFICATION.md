# Verification and acceptance report

Verified on **2026-10-19** against the source in this package.

## Recorded result

| Gate | Result |
| --- | --- |
| `cargo test --offline --locked --all-targets` | **54 passed, 0 failed, 0 ignored** |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Passed; no warnings |
| `cargo fmt --check` | Passed |
| Both example declarations through the CLI parser and validator | Accepted; the intentionally absent control endpoint returned `unknown`, without launching anything |

The test count is the number of executions Cargo reports. There are 53 distinct
tests: the shared fixture's socket-cleanup regression runs in both integration
binaries that import it. No missing-prerequisite skips are hidden in the total.
The full Cargo test output is retained in [TEST_OUTPUT.txt](TEST_OUTPUT.txt).

The environment was Linux, Rust 1.99.1, Cargo 1.99.1, and tmux 3.4, with the
checked-in Cargo.lock. The package declares a Rust 1.89 minimum; that minimum
toolchain was not separately tested. SQLite is built from the bundled dependency.
The native tests used real Unix sockets, Linux pidfds, Tokio child processes,
and actual daemon processes. Tests required an execution context permitting
local sockets and process signals.

## Test groups

| Integration binary | Executions | What it exercises |
| --- | ---: | --- |
| `tests/supervisor.rs` | 29 | State transitions, persistent intent/outcomes, idempotency, ownership and generation fences, automatic recovery, cancellation and adversarial owner observations |
| `tests/owner_rpc.rs` | 11 | Real Unix transport, authenticated/exclusive owner claims, durable owner reservations, process effects, delayed requests and owner reload ambiguity |
| `tests/probes.rs` | 10 | Real process/command/socket/HTTP probes, output/capacity bounds, timeout/cancellation reaping and actual tmux interaction |
| `tests/daemon.rs` | 4 | Actual CLI and daemon processes, competing controllers, SIGKILL recovery, delayed launches, forced stop and tmux workload non-replay |

## Acceptance mapping

| Requested behavior | Evidence and observed result |
| --- | --- |
| Start tmux, detach and reconnect to the same live session | Real tmux test attaches/detaches twice and verifies the same pane PID. The daemon test verifies the same server PID and generation after controller recovery. |
| Duplicate starts create exactly one server | Concurrent same-operation and distinct-operation starts converge on one owner launch. Repeated real CLI starts keep one tmux server. |
| Failed pane does not restart an unrelated workload | An actual pane exits with code 7; the server remains healthy and pane exit is a separate observation. Completed pane work remains at one execution after controller recovery and an explicit server restart. |
| Hung probe cannot wedge the supervisor | Real command and HTTP probes time out. Direct probe children are reaped, concurrency permits recover, queue deadlines prevent late spawns, and lifecycle control uses independent capacity. |
| Old-generation probe cannot affect a replacement | Tests retain an in-flight old probe while replacing the service. Its late reply cannot change the replacement's readiness, health, or recovery action. |
| Crash loops stop at the budget | Automatic replacement attempts stop at the configured lifetime budget. The budget survives controller recovery and is neither consumed nor reset by manual restart. |
| Controller recovery adopts a survivor | A real daemon is killed with SIGKILL while tmux stays alive under the fixture owner. A new controller adopts the same generation and PID without another launch. |
| Crashes around persistence and launch are explicit | Injected crashes cover intent-before-launch, launch-before-outcome, and stop-before-outcome. Lost owner replies reconcile one instance. An actual SIGKILL during delayed launch fences the old request; the historical operation remains unknown and only a fresh explicit operation can launch. |
| Graceful stop and force respect the cleanup boundary | A real foreground process ignores TERM and is killed after its deadline through the owner. It is reaped; an unrelated process remains alive. The fixture proves only the declared direct-process boundary. |
| Service recovery does not replay completed work | A completed tmux pane increments a counter once. Controller adoption and explicit server replacement leave the count unchanged; the replacement server has no restored sessions. Default recovery also leaves a completed foreground command stopped. |
| Concurrent controllers cannot both control a service | A competing daemon with a different state directory is rejected. Independent tests check the canonical kernel lock, the exclusive owner claim, claim EOF fencing, and delayed old-epoch effects. |
| Every destructive request targets a generation | Real and adversarial tests reject stale stops/restarts before effects. Historical operation retries return historical outcomes. A forced-stop deadline from a fenced epoch cannot kill an adopted survivor. |

Additional regressions cover physical tmux endpoint collisions, authenticated
stale-socket cleanup, a full control-socket backlog, reused generations with a
changed kernel process identity, unavailable probe mechanisms, output redaction,
and a live process whose interrupted stop remains unresolved in the same epoch.

## Reproduce

From the project directory, with a Rust toolchain and tmux available:

```sh
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

For a private tmux installation, set `INSULA_TEST_TMUX` to its absolute executable
path and optionally `INSULA_TEST_TMUX_LIB` to its dynamic library directory.
The recorded run additionally used `CARGO_INCREMENTAL=0` and `--offline` with
dependencies already fetched. Neither changes the contract being tested.

## What this result establishes

The reusable supervisor, daemon/CLI, production owner RPC client and test-only
owner agree on the lifecycle contract. Actual operating-system tests accompany
the deterministic state-machine and crash-window tests. There is no alternative
production service launcher or competing child-reaping loop in the supervisor.

The existing Insula owner implementation was unavailable in the accessible
source, so these results **do not establish integration with that owner**.
The native fixture owns real foreground children but does not create Insulas or
certify arbitrary descendants, cgroups, PID namespaces or tmux pane containment.
Its surviving-owner/controller-crash test is not evidence that a real bwrap
configuration preserves services.

Before deployment, the actual owner must implement the eight ExecutionOwner
methods using its authoritative machinery, fence effects and physical endpoints,
and advertise only verified capabilities. Run this contract suite through that
implementation and verify its real bwrap/PID-namespace lifetime, service and probe
cleanup boundaries, workload access isolation, secret resolution, and bounded
log sink/retention. The fixture discards logs; production log rotation is an owner
responsibility and was not exercised here.

Owner-process crashes with persisted receipts deliberately produce `unknown`
when live handles or complete effect evidence are unavailable. Neither this
suite nor SQLite proves recovery of unrecorded command results or reconstructs a
live process handle from a saved PID.
