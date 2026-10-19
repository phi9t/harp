# Harp integration

This crate is the supplied service supervisor, acquired without replacing its
state machine. `provenance/original.zip` contains all 24 original files.
`provenance/manifest.json` records every member's SHA-256 and imported location;
`provenance/Cargo.lock.supplied` retains its original dependency graph. The
archive's README and verification report describe upstream results, not Harp
qualification. No root license is added. Cargo declares MIT OR Apache-2.0;
the supplied archive contains neither license text.

Harp changes the rusqlite requirement from 0.38 to exactly 0.39.0 to share the
workspace's single libsqlite3-sys 0.37 native linkage. The supplied lock is
provenance only; the root workspace lock is authoritative after integration.
Tests now sit beside source: `src/supervisor_tests.rs`, `src/owner_rpc_tests.rs`,
`src/probes_tests.rs` and `src/bin/daemon_tests.rs`. Explicit `[[test]]` paths
preserve their independent integration crates. Automatic bin discovery is off,
so the adjacent daemon test is never compiled as a production binary.
`src/fixture_owner.rs` remains test-only. The tmux test waits for an available
exit status as well as the dead-pane marker; its exit-code contract is retained.

## Real owner

`insula.owner.service` is the explicit Linux owner process. `OwnerRpc` reaches
its private Unix socket using protocol v1. Both this owner and the durable
activity guardian use `insula.owner.process.launch` and the identity-checked
`insula.cgroup.Cgroup` implementation. The supervisor does not launch or reap
production services itself. No daemon or machine service is installed.

Only foreground services and empty tmux bootstrap are admitted. Each generation
runs in the admitted capsule's PID/network/mount namespaces and its own cgroup.
The readonly workspace and private service projection are the only workload
surfaces. Control sockets, journals, cgroup FDs and logs stay outside the capsule.
The owner persists launch reservations before spawning, fences epochs at effects,
and proves cgroup emptiness before reporting Gone. On owner restart it cleans
persisted cgroups; it never adopts stored numeric PIDs or automatically repeats
a service bootstrap. Claim EOF fences the old controller and cleans its services.

Only process probes, terminate stop, same host bench UID/GID and two bounded
stdout/stderr logs are currently supported. Secret references, arbitrary command,
HTTP, Unix-socket and tmux probes, privileged namespace identities, Insula-level
cleanup and keep-services lifetime are refused. `survives_controller_exit` is
false. A tmux service uses process readiness/health and an endpoint below
`/home/bench/service/`; replacing its empty server never resubmits pane work.
These limits apply even though the library protocol supports richer owners.

See [owner operation](../owner/README.md) for configuration and exact native
qualification. The service SQLite journal contains service operations and claims;
it neither writes nor mirrors durable workflow attempts, task intent or Git
acceptance records.
