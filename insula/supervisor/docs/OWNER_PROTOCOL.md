# Authoritative Insula owner protocol

The production supervisor never spawns, signals, enters namespaces for, or reaps
a service directly. `ExecutionOwner` in `src/owner.rs` is the integration
boundary. `OwnerRpc` in `src/owner_rpc.rs` implements that interface over an
authenticated local Unix socket. The existing Insula owner must implement the
server side using its canonical launch, namespace-entry, containment, and cleanup
machinery.

The repository includes a **test-only** owner under `tests/support/`. It exercises
the contract with real Tokio child processes and Linux pidfds. It is not an
Insula implementation and is never built into the production CLI. Its explicit
`direct_process` boundary proves that one owned foreground child has exited and
been reaped. It does not prove that arbitrary descendants, tmux panes, cgroups,
sub-Insulas, or a root environment are empty.

## Wire format

Each frame consists of a four-byte unsigned big-endian length followed by that
many UTF-8 JSON bytes. Empty frames and frames larger than 1,048,576 bytes are
rejected before payload allocation. Every request and response uses protocol
version `1` and an independent UUID `request_id`; the response must echo both.
The request ID correlates one transport exchange. Lifecycle operation IDs inside
`LaunchRequest` and `StopRequest` identify durable, idempotent operations.
One user-requested restart has one supervisor operation ID and distinct durable
owner operation IDs for its stop and launch phases. Both phase IDs are persisted
before effects. They must not reuse one owner ID with two different payloads.

The public Rust types are `RpcRequest`, `OwnerRequest`, `RpcResponse`,
`OwnerResponse`, and `OwnerReply`. `read_frame` and `write_frame` are shared framing
helpers. A request is shaped as:

```json
{
  "version": 1,
  "request_id": "e3199048-6476-45d2-a140-8472b4060152",
  "request": {
    "method": "capabilities",
    "root_insula_id": "root"
  }
}
```

A success contains `response.status = "success"` and a typed `value`. An error
contains `response.status = "error"` and a serialized `Error`. Error messages and
probe codes must be sanitized machine descriptions. Raw stderr, HTTP bodies,
command output, resolved environment values, and bearer credentials must never
appear in these messages.

The client requires an absolute socket path, an explicit expected peer UID, and
a request timeout between greater than zero and 120 seconds. Each connection is
authenticated with Unix peer credentials. The configured socket and all its
ancestors must be protected by the owner from workload writes: UID checking
does not make a workload-writable socket path trustworthy. The server must
authenticate the client UID and bind accepted controller identities to that
authenticated caller.

The timeout covers connecting, admission to the client's bounded request pool,
writing a complete frame, and receiving a complete response. Partial progress
does not extend it. There are separate pools for control exchanges and probes,
so stalled probes do not consume the control pool.

## Operations

| Method | Request content | Successful reply |
| --- | --- | --- |
| `capabilities` | Root Insula ID | `capabilities` |
| `claim` | Root Insula ID and controller UUID | `claimed` |
| `validate` | Ownership token | `acknowledged` |
| `observe` | Ownership, service key, optional generation | `observation` |
| `launch` | Ownership and `LaunchRequest` | `observation` |
| `stop` | Ownership and `StopRequest` | `observation` |
| `probe` | Ownership and `ProbeRequest` | `probe` |
| `release` | Ownership token | `acknowledged` |

Capabilities identify the canonical owner authority directory and report
controller/generation fencing, durable launch reservations, service-boundary
cleanup, probe/log limits, and verified survival after controller exit.
Unsupported capabilities are explicit. In particular, an owner may not claim
service-boundary cleanup because it can kill a numeric PID or a process group.

## Controller ownership

`claim` is a special, long-lived connection. The owner grants one epoch for the
root authority, sends its `Ownership`, and leaves the connection open without
further application frames. All other requests use independent connections and
carry that ownership token. The token contains root identity, owner identity,
controller UUID, and a monotonic epoch. It is not a bearer secret: the owner also
checks the authenticated caller and its live claim.

The owner must enforce these properties:

1. Claims for the same canonical root are exclusive across every controller,
   socket alias, and supervisor state directory.
2. A claim connection's EOF, error, unexpected bytes, or explicit release fences
   the epoch. The owner completes this transition before granting a new epoch.
3. Every actual launch, graceful stop, forced cleanup, or namespace mutation
   validates the epoch under the same gate that admits a replacement claim.
4. A delayed request validates at its effect, after delays and await points.
   Checking only at request arrival is insufficient.
5. An external graceful-stop helper must have finished or been canceled and
   reaped before a conflicting epoch or generation can make its actions stale.
6. Releasing an already-fenced old token never releases a newer controller.

The supervisor additionally holds a kernel file lock in the owner-reported,
verified canonical authority directory. The lock is not selected by the
supervisor's state-directory option. This local lock supplements the owner gate;
it cannot substitute for fencing inside the owner at the effect.

The client keeps the claim stream alive independently of probes and lifecycle
calls. If that stream closes, it rejects further requests locally. A transport
failure in another exchange does not create a new claim or acquire a newer
epoch implicitly.

## Launch and crash reconciliation

`launch` carries a stable service key, configuration, generation UUID, and
operation UUID. The operation ID binds the complete request. Reusing it with
different arguments is a conflict. Repeating the same admitted launch cannot
create another process, including after that process exits or a replacement
generation has run.

The owner records admission durably before spawning. Its record must distinguish
an unissued launch from a launch that might already have executed. An owner
implementation can use a process-start barrier and durable receipt to tighten
this window. Otherwise, a crash in the admission/spawn/receipt gap is `Unknown`.
It is never evidence that retrying a fresh launch is safe.

The supervisor persists its own intent before calling the owner, then reconciles
using the owner record and live process/endpoint identity. The three observation
outcomes are:

| Observation | Required evidence |
| --- | --- |
| `alive` | An authoritative process identity for the exact generation, verified against the live instance |
| `gone` | Required boundary is empty **and** no admitted old launch or helper can subsequently create it |
| `unknown` | Either proof is missing, ownership is uncertain, or the target is inaccessible |

`gone` carries both `boundary_empty` and `launch_resolved`. Both must be true
before the supervisor permits replacement. A saved PID, an absent socket, or a
single empty process listing does not establish those conditions.

If a mutation times out, the connection breaks, or the response framing cannot
be trusted, `OwnerRpc` returns `Unknown`. The owner may have completed the effect.
The caller must reconcile or retry the **same** operation. A well-formed owner
error is returned as that definite error. Read-only transport failures return
`Unavailable` and do not imply that a service is unhealthy.

The supervisor records the controller epoch alongside an unfinished transition.
If a stop was interrupted in the same epoch, observing the target alive does not
prove that the pending stop cannot still take effect. That transition remains
unknown. A newly acquired epoch can adopt a verified survivor only after the
owner has fenced all old-epoch effects according to the claim contract.

## Stops and generation checks

Every stop includes the target generation, operation ID, and declared policy.
The owner validates the policy against its service record, verifies ownership,
and compares the current generation before any effect. A stale request must not
change desired state, send a signal, invoke a graceful helper, or authorize a
new generation. It returns `StaleGeneration`.

An idempotent replay of a previously completed stop returns its recorded
outcome. It does not reinterpret the old request as a stop of the current
generation. Forced cleanup is conditional on the declaration, the deadline,
the still-valid ownership epoch, and the still-current target generation.

The owner is the only child-reaping authority. A supervisor does not install a
competing `waitpid(-1)` loop. Linux implementations should retain verified
pidfds for processes they own. Reopening a handle after recovery requires live
identity verification through the owner; SQLite cannot reconstruct that handle.

## Probe and tmux integration

Probes carry the target generation. The owner enters the target Insula, resolves
environment references, applies time/output/concurrency limits, and cleans up
probe children. The optional local probe helpers are called **inside** the
owner's namespace context. They are not a second supervisor launch path.

The tmux adapter starts an empty foreground server with explicit arguments:

```text
/absolute/path/to/tmux -D -f /dev/null -S /private/path/tmux.sock
```

There is no command argument after `-D`. No session, pane, training command, hook,
Codex turn, or benchmark is part of automatically replayed bootstrap. Clients
and probes use `-N` to prevent implicit server creation. Identity checks bind the
private socket to the expected live server; pane status and session expectations
are reported separately from server health.

Controller crash, owner crash, workload sub-Insula destruction, and root
destruction are distinct events. Survival depends on the actual bwrap
`--die-with-parent`, PID-namespace-init, parent-death, containment, and owner
lifetime settings. Those settings must be verified in the existing runtime
before advertising `survives_controller_exit` for real Insulas.

## Test fixture guarantees

The fixture owns Tokio `Child` values and opens pidfds before reaping, so test
signals never use an unverified saved PID. It keeps services alive while a
controller disconnects, since the fixture owner remains alive. It persists
reservation receipts and operation fingerprints, and a restarted fixture treats
unresolved receipts as `Unknown` rather than reconstructing child handles.
Stdout and stderr are discarded, which gives a strict zero-byte log bound.
Admitted stop operations run in owner-owned tasks, so canceling an in-process
caller's wait cannot release a graceful helper's gate before that helper has
finished or been killed and reaped.

`tests/owner_rpc.rs` exercises peer UID checks, exclusive claims, independent
probe/control connections, duplicate launches, stale stop requests, historical
operation retries, timed-out launch reconciliation, malformed mutation responses,
and an old delayed launch arriving after claim EOF and a replacement epoch.
It also checks owner reload with persisted live receipts, cancellation during a
graceful helper, and a forced-stop deadline arriving after its epoch was fenced.
The main acceptance tests add supervisor persistence and real CLI crash cases.

Passing these tests establishes the library and transport behavior against this
owner contract. It does not certify uninspected Insula namespace lifetime or
whole-service containment.
