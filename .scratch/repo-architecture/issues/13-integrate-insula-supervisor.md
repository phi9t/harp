# 13: Integrate the supplied Insula service supervisor

Status: done
Owner: supervisor-migration
Parent: [Repository architecture](../spec.md)
Source and boundaries: [supervisor integration](../../../docs/workstream/repo-architecture/supervisor-integration.md)

## What to build

Acquire the supplied archive at its recorded SHA-256 into insula/supervisor/,
preserving source, lockfile provenance, declarations, tests, reported evidence
and licensing status. Reuse its service state machine and Unix owner client.
Implement the ExecutionOwner boundary through the actual authoritative Insula
launch/stop/namespace machinery without introducing another launcher or reaper.
Keep service policy separate from the single agent/workload control plane.

## Acceptance criteria

- [x] Imported tests are colocated with owning source modules, with native integration targets configured explicitly; preserve original archive provenance and test behavior.
- [x] Source/archive and adaptations have exact provenance. Reconcile workspace SQLite linkage/toolchain constraints and rerun tests against the chosen lock without silently changing pins.
- [x] The real owner implements all eight interface methods for its advertised capability subset, exclusive root claims, effect-time epoch/generation fencing, durable launch and endpoint reservations, and reconciliation observations. Unsupported operations refuse admission; a test fixture never supplies production authority.
- [x] Durable activity admission and service claims share the authoritative execution boundary without state-directory or socket aliases bypassing exclusivity. No competing namespace keeper or reaping loop is introduced.
- [x] Private service-state SQLite records remain service-owned; workflow attempts, candidate results and approvals remain durable-owned. Neither database writes or mirrors the other's authority.
- [x] Verify actual target namespace entry, service/probe descendant cleanup, secret resolution, bounded log sinks and workload/controller access isolation. Unsupported stronger capabilities refuse admission.
- [x] Reconnect, terminal replacement and workflow recovery remain separate. A completed tmux-pane counter stays at one after controller recovery and explicit empty-server replacement. Agent turns, benchmarks and training are never implicit bootstrap replays.
- [x] Reproduce supported duplicate-start, stale-generation, claim-loss, lost-reply/unknown-outcome, owner-crash recovery, bounded logs and competing-owner cases through the real owner. Preserve richer policy/probe/budget fixture tests without claiming they qualify unsupported owner capabilities.
- [x] Qualify actual bwrap/PID-namespace lifetime and both controller-crash and owner-crash cases. Advertise keep-services survival only when that specific boundary is demonstrated.
- [x] No daemon, hook or persistent service is installed as an integration side effect. Preserve explicit operator lifecycle controls and sanitized public evidence.

## Blocked by

- [04-extract-insula](04-extract-insula.md)

Source acquisition, dependency inspection and standalone test reproduction can
precede the extracted owner adapter. The missing real-owner capability is an
integration task, not a reason to reimplement the service supervisor.

## Verification

The supplied report records 54 test executions, clippy and fmt; these have not
been rerun here. Preserve that evidence with its direct-process fixture limits.
Rerun the library/transport/daemon tests, then the owner contract plus real
namespace/cgroup cleanup and tmux/Emacs service scenarios. Record exact source,
lock, owner and rootfs identities. The initial acquisition did not include production owner integration; the fresh
evidence below and final root-workspace qualification supersede that state.

## Acquisition and supported owner delivery

Imported the exact 24-member supplied archive under `insula/supervisor/`, with
original ZIP, original lock and a member-to-import hash manifest retained under
`provenance/`. The library/CLI remain the supplied implementation. Integration
tests now colocate beside their source modules with explicit Cargo targets and
no automatic test-binary discovery. Updated rusqlite to exactly 0.39.0, using
libsqlite3-sys 0.37.0. The archive's dual-license declaration remains; separate
license texts were absent and are not invented.

Added the actual Linux `insula.owner.service` owner protocol and shared
`insula.owner.process.launch` gate. The owner uses the same identity-checked
cgroup and capsule mechanisms as durable activity execution. It reserves service
launches durably, fences controller epochs and generations at effects, takes a
root-inode lock against simultaneous owner aliases, bounds log sinks, and proves
whole-service cgroup emptiness. Retained state is required after owner crash;
restart cleans retained groups without PID adoption or bootstrap replay. The
service journal never writes workflow state.

Supported admission is deliberately narrower than the full owner protocol:
foreground/empty tmux services, default bench identity, process probes, terminate
stop, service cgroup cleanup and stop_services lifetime. Secrets, other probe
kinds, privileged identities, Insula destruction and keep_services lifetime are
Unsupported. These refusal boundaries are explicit in `INTEGRATION.md` and
`insula/owner/README.md`. The supplied fixture owner remains test-only.

### Fresh evidence

- Linux Rust suite: 54 executions passed, zero skipped; clippy with warnings
  denied and cargo fmt passed. Rust 1.95.0, rusqlite 0.39.0 and private tmux 3.4.
- Native real-owner suite: nine tests passed, zero skipped, through actual
  cgroup v2 and capsule PID/mount/network namespaces. Includes namespace access
  limits, fork/setsid descendant cleanup, duplicate launch, stale generation,
  claim EOF fencing, lost reply reconciliation, root alias exclusion, owner
  SIGKILL recovery without bootstrap replay, bounded logs and capability refusals.
- The Rust supervisor binary successfully served a real request through OwnerRpc
  to this owner. An actual namespace tmux server ran one pane counter; reconnect
  and explicit empty replacement left that counter at one.
- Private qualification rootfs was copied from an existing admitted closure and
  completed with tmux 3.4/library/UTF-8 locale dependencies. Shared rootfs bytes
  were not changed. No persistent service was installed.
- `provenance/qualification/record.json` records exact source, standalone lock,
  rootfs manifest and release hashes; adjacent logs retain passing output.
- Python black/ruff and diff whitespace checks passed. The baseline changed-path
  verifier refuses new `insula/owner/` paths until the migration's selector update;
  parent integration must run the combined root-workspace gate.

### Remaining qualification boundary

The advertised owner subset is the existing-code migration contract. Richer owner
capabilities and their full real-adapter matrix are tracked in
[the service-owner follow-up](14-extend-service-owner-capabilities.md). The imported fixture suite covers rich supervisor
policy and probe cases; it does not qualify those capabilities for this owner.
Non-process probes, secret resolution and controller-independent survival remain
explicitly unsupported. Shared durable guardian integration and root-workspace
linkage/verification are parent integration responsibilities. Do not represent
the nine native cases as proof of the unimplemented stronger capabilities.

Post-import boundary review also found and closed generation reuse across service
keys. The owner now reserves each physical generation payload and rejects both
cross-service aliases and retired generation reuse before launching. Native
regression verifies the original process identity remains unchanged, operation-ID
collisions refuse, and replay of a completed original launch observes Gone
without replaying its bootstrap. Final native result is nine passed, zero skipped.

## Completion

Implemented and verified in body commit `c32c6f0f4a2de73e8067be0361121bb841ab4622`. The complete-candidate
changed-surface plan passes, including the bounded verification repairs recorded in
[the repository qualification](../../../evidence/provenance/repository-qualification.json).
Native runtime, benchmark and supervisor receipts bind the composed runtime bytes;
real Codex one/two/four-worker runs and all eight explicitly approved landings are
recorded separately in [Codex qualification](../../../evidence/provenance/codex-qualification.json).
Retained workloads and proof/PDF consumers passed their relevant checks.

This closes the advertised existing-code capability subset only. Secret resolution,
non-process probes and controller-independent service survival remain unsupported
and tracked in [ticket 14](14-extend-service-owner-capabilities.md).
