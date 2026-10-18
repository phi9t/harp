# Ticket 15 supervised environments candidate

This is an unaccepted, frozen implementation candidate for separate Standards and
Spec review. No commit, push, model call, production mutation or mainline change
was performed. Read baseline `963f58b31e518a8162dfe719e386ef9d9843321f`; implementation
content baseline is the exact 25-file `ticket12-accepted-manifest.json`, verified
before constructing the patch. Canonical ticket15/SPEC and the approved Rust,
PostgreSQL, Git and local isolation baseline govern this slice. Session microplan
is non-authoritative scratch. Context reuse follows the parent's documented
thread-limit exception; no agent was spawned.

## Behavior and public seam

`supervisor::Supervisor` exposes typed `create_attempt`, `start`, `inspect`,
`cancel`, `reconcile`, `history`, `logs` and `release` operations. Every public read
and mutation checks live credential expiry, deployment epoch, task scope and
current generation; verifier credentials cannot mutate execution state. An
immutable PostgreSQL workspace manifest pins store, exact Git base, path,
attempt, task, spec digest, generation, epoch, allowed write directories and the
independent log path. Each attempt receives its own Git clone with independent
metadata; checkout uses the admitted exact commit. Attempt uniqueness prevents
workspace reuse.

The workspace/execution record commits before attempt filesystem creation.
Preparation, launch and deletion reservations serialize on the broker's advisory
transaction lock. A failed preparation never becomes launchable. The monitor
claims its durable identity before spawning bubblewrap; a repeated monitor
cannot launch the same record. Bubblewrap blocks on a pipe until its namespace
init PID, boot ID, start ticks, launcher identity and initial membership commit.
Only then does the workload receive the launch signal.

The monitor is a separate local process, detached from the initiating caller.
The trusted local CLI `supervisor-monitor` uses an environment DSN, never a DSN
argument or error message. The workload receives neither the DSN nor bearer
credentials. Bubblewrap creates a dedicated user/PID/network/IPC/UTS namespace,
drops capabilities, disables further user namespaces, exposes the Git workspace
read-only, and mounts only admitted writable directories. Shared Git storage is
absent from the capsule. Git metadata and supervisor logs are not writable by
the workload.

Cancellation persists desired state before signaling. Deadline expiry, current
generation/epoch changes and controller loss converge through the same stop
logic. Host PID namespace inventory uses kernel namespace identity and `NSpid`
to exclude host-namespace processes without relying on ancestry or sessions.
PID/start-tick/boot/namespace checks around `pidfd_open` prevent signaling a reused
PID. Nested `setsid` and background processes remain in the capsule namespace.
Live membership snapshots are retained as immutable observations; logs live
outside workspace deletion. Missing or ambiguous identity/membership yields
quarantine and holds capacity. Only proven stop plus completed deletion refunds
a concurrent slot; cumulative attempt and episode admission remain consumed.
Deletion commits its reservation before touching the filesystem and is resumable.

## Retained behavioral evidence

[Behavioral proof](behavioral-proof.txt) retains the genuine missing-
implementation red (two public PostgreSQL behaviors fail after successful
admission/reservation), the fresh ten-test green, clippy terminal output and full
gate summaries. The initial fixture authentication setup error was corrected
before the behavioral red and is not presented as product evidence.

The model-free real Git/PostgreSQL/Linux matrix exercises:

- Durable registration before filesystem creation; interrupted creation cannot launch.
- Actual subprocess SIGKILL after registration and after Git creation, with durable recovery.
- Actual initiating-agent SIGKILL while a separately supervised workload remains inspectable and cancellable.
- Real nested `setsid`/background children and read-only Git metadata/shared store protection.
- Explicit cancellation, one-second deadline, monitor SIGKILL and actual bwrap launcher SIGKILL; complete namespace stop is checked.
- Launch racing committed deletion reservation; released workspaces cannot restart or be recreated for the same attempt.
- Current generation fencing also stops an already running namespace; expiry and invalid credentials reject reads.
- Interrupted starting identity quarantines workspace/capacity and rejects release/relaunch.
- Retained logs remain readable after the workspace is deleted; immutable history includes nested membership.

Subprocess helpers require their actual SIGKILL terminal signal and do not swallow
unexpected subprocess failures. No daemon/provider/model was substituted for the
real capsule process. Fixtures use bounded deadlines and disposable loopback PG;
passing fixtures remove their own workspace roots and stop all namespace members.

## Fresh checks and source freeze

[Final gate record](final-gate.json) records commands, UTC time, terminal
exit codes and retained complete-log digests. Focused tests: 10 passed, exit 0.
Formatting: exit 0. Clippy all targets with `-D warnings`: exit 0. Full repository
`tools/agents/check.sh`: exit 0, terminal `check: all green`. Accepted artifact,
verifier and gateway suites remain green in that full gate.

[Baseline-relative patch](baseline-relative.patch) SHA-256:
`bb8660c906106c00416617c8ea0cc25922e933cea1b51fe36842900407d562d3`.
[Source manifest](source-manifest.json) freezes eight changed source paths.
Shared glue is only the library export, authoritative core migration call, local
monitor CLI command and direct locked libc dependency. New implementation,
migration and tests are `src/supervisor.rs`, `migrations/0005.sql` and
`tests/supervisor.rs`. No core v1 schema version, accepted authority behavior or
artifact/gateway source was rewritten. Migration 0005 owns its independent schema
marker; parent integrates Temporal 0004 before it.

## Limits

Observed host bubblewrap version is 0.8.0. The cgroup v2 hierarchy is not delegated
writable, so this candidate makes no CPU/memory/cgroup enforcement claim. Proof
is local Linux PID-namespace supervision, not multi-host or external runtime
control. Unavailable nested namespace links, kernel identity or PostgreSQL evidence
remain fail-closed quarantine. Writable scope currently admits directories and
rejects symlinks/files; it does not grant extra paths to compensate. The supplied
Git store and local broker/monitor operator are trusted, as in the approved
baseline. Quarantined workspaces/capacity need deliberate recovery and are never
silently reused. Independent reviews and owner acceptance remain outstanding.
