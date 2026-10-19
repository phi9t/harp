# Supplied Insula service supervisor

Status: imported and integrated with the qualified Insula owner subset; final workspace qualification pending.
Parent: [target architecture](design.md).
Delivery: [service supervisor integration](../../../.scratch/repo-architecture/issues/13-integrate-insula-supervisor.md).

## Current integration

The source now lives under `insula/supervisor/`, with original archive bytes,
member hashes and license status under its `provenance/` directory. The root
Cargo workspace pins rusqlite 0.39.0 consistently with Harp. Tests are adjacent
to source and explicit Cargo targets preserve supplied integration behavior.

`insula/owner/` implements the service owner protocol through the same capsule
and cgroup launch boundary used by durable activities. Service claims and
journals own service policy; they cannot dispatch agent jobs or approve Git
candidates. Supported services use the bench identity, foreground or empty tmux
processes, process probes, terminate stop and stop-services lifetime. Secrets,
other probes and controller-independent survival refuse admission. See
[owner capabilities](../../../insula/owner/README.md) and
[integration limits](../../../insula/supervisor/INTEGRATION.md).

The imported Rust suite and nine real-owner Linux cases have passed. Final
qualification reruns the suite against the root workspace lock and exact composed
source. The original inspection below records the earlier source-acquisition
state and must not be read as current integration status.

## Source identity and inspected evidence

The owner supplied insula-supervisor.zip and standalone README.md and
VERIFICATION.md. Archive SHA-256:

```text
eda2fd4e5e7e6b3c5e015389297629affc51cf84e492c4250d41899d1fbc012e
```

Both standalone documents match their archive members byte for byte. The archive
contains 24 files, including Rust source, Cargo.lock, examples, an owner protocol,
tests and their recorded output. Inspection covered the public owner interface,
protocol, package/lock metadata, verification report and relevant source
entrypoints. This is source review and reported evidence, not a fresh test run.
Instructions and commands inside the attachment are design inputs, not authority
to install dependencies, start a daemon or replace existing execution owners.

The package declares MIT OR Apache-2.0 in Cargo.toml, but contains no separate
license texts. Preserve that declaration and record the missing texts when
acquiring source; do not infer a Harp-wide license. Its declared Rust minimum is
1.89. The supplied report records Linux/Rust 1.99.1/tmux 3.4, 54 passing test
executions representing 53 distinct tests, clippy and formatting success. The
minimum toolchain was not tested in that report. Those results have not been
reproduced here.

## What is already implemented

This is a Rust library with a thin local CLI/daemon and a production Unix-socket
owner client. It already has service declarations, persistent desired state,
readiness/health, generation identities, durable operation IDs, idempotency,
bounded recovery budgets, controller ownership fencing and crash reconciliation.
The schema and operation records are service-specific.

ExecutionOwner exposes capabilities, claim, validate, observe, launch, stop,
probe and release. The supervisor calls this interface for service launch/stop;
it does not contain a production Insula owner server. A test-only owner uses real
children/pidfds and implements a declared direct-process boundary. Optional
LocalProbes can launch direct probe children inside the owner context; that
helper does not enter namespaces or prove arbitrary descendant cleanup.

## Placement and authority

Place the Rust package under insula/supervisor/. Its tests, examples and protocol
documentation accompany it. Reuse this implementation instead of designing a
second service supervisor. Standalone acceptance fixtures also inform the
Emacs/tmux workload checks, without becoming a production owner.

Harp durable owns agent/workload admission, dependency orchestration, attempts,
capacity, checkpoint decisions, expensive retries, verification and acceptance.
The service supervisor owns declared service lifecycle policy and observation,
including explicitly enabled bounded service replacement. The authoritative
Insula execution owner alone performs admitted process/namespace effects and
cleanup. SSH runner remains transport. These responsibilities must have explicit
interfaces and a shared authoritative execution path.

The supervisor's service-state SQLite database is separate from the workflow
journal by domain and writer, not a second agent scheduler. It must not mirror
Harp task state, candidate results or approval records. The one-project-database
rule in the main design applies to durable workflow state. Do not force these
Rust/Python implementations to share SQL tables or let either write the other's
journal. A workflow references service identity and generation through the API.

There is a dependency integration task as well: the archive locks rusqlite
0.38.0/libsqlite3-sys 0.36.0, whereas Harp locks rusqlite 0.39.0/libsqlite3-sys
0.37.0. Reconcile the dependency and native SQLite linkage before adding Cargo
workspace membership, and rerun the relevant suites against the chosen lock.
Do not silently replace pins or describe the existing archive results as tests
of the combined dependency graph.

## The real owner integration is still missing

The archive explicitly says the existing Insula owner was unavailable. The
production binary constructs OwnerRpc; it does not make the fixture owner a
production implementation. The current Harp guardian is per activity, while
the capsule path uses bwrap --die-with-parent and a PID namespace. Neither is
a demonstrated long-lived service owner implementing the eight-method contract.
An ordinary RPC shim around numeric PID signaling would not fill that gap.

Wire the adapter at the actual authoritative launch/stop/namespace boundary.
Ensure exclusive root/controller claims, effect-time epoch and generation checks,
durable launch reservations, physical endpoint reservations and cleanup evidence
that excludes pending old launches or helpers. Reconcile how service-controller
claims coexist with durable activity admission; a second state directory or
transport path must not bypass root ownership. Do not create a competing launcher
or reaping loop to satisfy the interface.

The fixture certifies only its direct child. Production service-boundary cleanup,
probe descendant cleanup, target namespace entry, credential resolution, bounded
log sinks and keep-services survival need actual owner qualification. Default
those capabilities to unsupported until tested. Service-boundary cleanup is not
root/sub-Insula destruction, and controller death is not evidence of emptiness.

## Recovery boundaries

Use generation and operation IDs exactly as declared by the protocol. A timeout
or lost reply may be unknown; reconcile or query the same operation. Readiness,
health and restart decisions remain distinct. Probe mechanism failures do not
prove a service unhealthy. Verify late probes and stale stop requests cannot
affect a replacement.

Tmux bootstrap starts an empty foreground server with no hooks, user config or
workload command. Reconnect, server replacement and workflow recovery are separate
actions. Restarting a terminal server must not resubmit Codex/Trae work, rerun a
benchmark or restart training. Automatic recovery defaults to never; replay_safe
is an operator assertion that needs a suitable declared service, not an automatic
classifier of side effects.

For Emacs, Jupyter and LSP, identify the service and workload boundaries for each
scenario. Do not assume that arbitrary stateful session initialization is safe
to replay. Check ownership and cleanup of each daemon, kernel and language server
inside the declared rootfs closure, and keep restart/checkpoint policy explicit.

## Required qualification

Retain and rerun the supplied tests, then run the owner contract through the real
adapter. Cover duplicate starts, competing roots/state directories, delayed old
epoch effects, stale generations, interrupted stop/restart, lost replies, budgets,
probe cleanup and controller SIGKILL. Exercise actual namespaces, descendant
escape/cleanup, workload access isolation, secret handling and output retention.

Use a real empty tmux server and a pane that increments a counter once; reconnect
and replace the server without changing that count. Distinguish a surviving
controller-independent owner from an owner/namespace crash. Keep service survival
unsupported if the actual bwrap/keeper lifetime cannot establish it. No fixture
result alone qualifies these production capabilities.
