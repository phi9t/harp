# 13: Integrate the supplied Insula service supervisor

Status: ready-for-agent
Owner: unassigned
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

- [ ] Imported tests are colocated with owning source modules, with native integration targets configured explicitly; preserve original archive provenance and test behavior.
- [ ] Source/archive and adaptations have exact provenance. Reconcile workspace SQLite linkage/toolchain constraints and rerun tests against the chosen lock without silently changing pins.
- [ ] The real owner implements all eight interface methods, exclusive root claims, effect-time epoch/generation fencing, durable launch and endpoint reservations, and complete observation semantics. A test fixture never supplies production authority.
- [ ] Durable activity admission and service claims share the authoritative execution boundary without state-directory or socket aliases bypassing exclusivity. No competing namespace keeper or reaping loop is introduced.
- [ ] Private service-state SQLite records remain service-owned; workflow attempts, candidate results and approvals remain durable-owned. Neither database writes or mirrors the other's authority.
- [ ] Verify actual target namespace entry, service/probe descendant cleanup, secret resolution, bounded log sinks and workload/controller access isolation. Unsupported stronger capabilities refuse admission.
- [ ] Reconnect, terminal replacement and workflow recovery remain separate. A completed tmux-pane counter stays at one after controller recovery and explicit empty-server replacement. Agent turns, benchmarks and training are never implicit bootstrap replays.
- [ ] Reproduce duplicate-start, stale-generation, claim-loss, delayed-effect, interrupted-stop/restart, unknown-outcome, probe/budget and competing-controller cases through the real owner.
- [ ] Qualify actual bwrap/PID-namespace lifetime and both controller-crash and owner-crash cases. Advertise keep-services survival only when that specific boundary is demonstrated.
- [ ] No daemon, hook or persistent service is installed as an integration side effect. Preserve explicit operator lifecycle controls and sanitized public evidence.

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
lock, owner and rootfs identities. No production owner integration, crate import
or native qualification has been performed yet.
