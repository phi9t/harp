# Ticket 15 supervised environments — repair 1 candidate

Source is frozen for independent Standards and Spec re-review; this report does
not claim acceptance. No commit, push, model call, production mutation or mainline
change was performed. Read base `963f58b31e518a8162dfe719e386ef9d9843321f`;
implementation content baseline remains the exact accepted10+11+12 25-file
`ticket12-accepted-manifest.json`. The initial rejected candidate and its proof
are retained separately in [round0](ticket15/round0/report.md). Canonical ticket15,
SPEC and the approved Rust/PostgreSQL/Git/local-isolation baseline govern this
slice. Session microplan/debug hypotheses remain non-authoritative scratch.
Context reuse follows the parent's documented thread-limit exception; no agent
was spawned.

## Resulting behavior and public seam

`supervisor::Supervisor` exposes typed creation, scoped creation, start, inspection,
cancellation, reconciliation, history, retained logs and release. Every public
read/mutation checks current credential expiry, deployment epoch and scope.
Worker authority additionally requires the assigned `attempts.worker_id`, not
merely a matching task/generation. Verifier credentials cannot mutate execution.
A current Owner can inspect/stop/reconcile/delete historical executions after
generation or epoch supersession. This recovery permission cannot relaunch them;
stale Workers remain rejected.

The immutable manifest pins Git store/base, workspace path, attempt/task/spec,
generation/epoch, admitted write paths and their file/directory kinds, and an
independent log path. Existing kinds come from the exact Git tree. Missing paths
require an explicit `WriteScope { path, kind }`; kind guesses from extensions,
unadmitted paths, mismatches and symlinks are rejected. Before creation, tracked
Git ancestors and every existing filesystem component are checked. Actual
preparation uses descriptor-relative `openat(O_NOFOLLOW)`/`mkdirat`, including
missing parent creation. The capsule mounts only the admitted leaf file or
directory writable; preparing a missing file never grants its parent writable.
Each attempt receives independent Git metadata at the exact admitted base.

PostgreSQL registration commits before attempt filesystem creation. A durable
`Preparing` intent additionally commits before Git clone/checkout spawn. Unknown
preparation is quarantined with a retained reason, reservation and workspace;
a creator's death or DB lock release is not evidence that Git descendants stopped.
Only waited/completed preparation can become Ready. Preparation, launch and
deletion reservations serialize on the broker's advisory transaction lock.

The independent local monitor commits its controller claim before spawning
bubblewrap. Its second launch transaction rechecks the current deployment epoch,
attempt/task generation, deadline and desired cancellation before launch files
or a blocked namespace are created. Deadline is checked again before eligibility.
Bubblewrap blocks on a pipe until namespace-init identity, boot ID/start ticks,
launcher and initial membership commit. A repeated monitor cannot relaunch the
same execution. The trusted local monitor CLI uses environment credentials;
the workload receives neither DSN nor bearer material.

Bubblewrap isolates user/PID/network/IPC/UTS namespaces, drops capabilities,
disables nested user namespaces, exposes the workspace/Git metadata read-only,
and binds only exact admitted write scopes. Shared Git storage is absent.
Cancellation commits desired state before signaling. Deadline, generation/epoch
change and controller loss share the same stop path. Namespace inventory includes
nested sessions/background children; checked pidfds avoid signaling reused PIDs.
Kernel `NSpid` excludes host-namespace processes without relying on ancestry.
`ENOENT` and `ESRCH` during `/proc` reads mean a task disappeared; permission failures
and unproven live membership still quarantine. Reasons contain no credentials.
Immutable observations retain membership; logs remain outside workspace deletion.

Only proven complete stop plus completed deletion refunds concurrent capacity;
cumulative attempt/episode counts remain consumed. Deletion commits its
reservation before filesystem deletion and can be resumed. Neither quarantine
nor a surviving/un-inventoried preparation can release/refund/reuse its workspace.

A reusable, nonmutating supervisor schema guard validates marker existence and
exact supported version before `open` filesystem effects and monitor effects,
and at later control/launch boundaries. Migration0005 remains additive and atomic,
with its independent version marker; core schema v1 was not changed.

## Behavioral repair proof

[Behavioral proof](ticket15/behavioral-proof.txt) retains the six genuinely failing
public regressions and the fresh green runs. Red failures were wrong-worker
creation, actual external write through a tracked ancestor symlink, rejection of
admitted files, stale generation launch after the durable controller claim,
Owner recovery rejection, and unsupported-schema filesystem pollution. New typed
scope/fault signatures were compile-shaped test seams before security changes.
The four deterministic launch races use the real PostgreSQL controller-claim
boundary, not delayed guesses.

The later confirmed preparation gap has a separate actual OS proof: a real
`/usr/bin/git` clone blocks opening its own trace FIFO, its initiating helper is
SIGKILLed, and Git demonstrably survives. Restoring the **prior unsafe Registered
classification** in the contemporary fault harness makes public reconciliation
falsely report Stopped while that Git process is alive (red). This was a targeted
classification regression, not a claim that the complete original frozen source
was rerun unchanged. Durable Preparing makes the same fixture quarantine and hold
capacity (green). No fake Git executable/provider/deduplication was used. The
fixture explicitly kills and checks its deliberately owned surviving Git process.

A short-command concurrency failure was held as unresolved despite a green full
gate. A 30-command serial diagnostic passed; an 8-way bounded concurrent loop
reproduced Linux `ESRCH` at round3, with a durable exact PID/errno reason. The first
root-cause fix classifies ESRCH as disappearance only at `/proc` observation
boundaries. The permanent strict regression executes 32 actual file-scope capsule
commands concurrently in bounded batches, requiring normal stop, exact admitted
writes, denied sibling/parent writes, retained logs and deletion. It passed alone
and with the original full focused matrix. The file-scope fixture was not weakened
to accept unexplained quarantine; throwaway debug loops/log markers were removed.

Fresh focused matrix: **19 tests passed**. It additionally retains real nested
setsid/background cancellation; deadlines; actual bwrap launcher, controller and
initiating-agent SIGKILL; real creation-boundary SIGKILL; deletion/launch races;
invalid/expired/wrong-principal credentials; public Owner recovery after generation
supersession and actual epoch rotation; exact existing/missing files and explicit
missing directory creation; missing-kind, mismatched-kind and scope-widening
rejection; and missing/future schema rejection with no pollution/launch.
Subprocess helpers require exact SIGKILL and reject unexpected prior exits.
Fixtures are disposable loopback PG, finite local Git/capsules and zero model calls.

## Fresh checks and frozen handoff

[Final gate record](ticket15/final-gate.json) records commands, UTC time, terminal
exit codes and retained complete-log hashes. Focused 19-test suite, formatting,
clippy all targets with warnings denied, and full `tools/agents/check.sh`: **exit0**.
Full gate ends `check: all green`, including accepted artifact/verifier/gateway
suites. No source change followed those final checks.

[Accepted12-relative full patch](ticket15/baseline-relative.patch) SHA-256:
`90b12ad8d0ac2737793b7170108f0bc1bcc77ba9fd369b28052ae18460f332d4`.
[Initial-candidate-relative repair patch](ticket15/repair1.patch) SHA-256:
`affa3b10036a52c4e7ddd5f7b8d941a4ffd2ec901fbb1d88fa3573c0dd997c20`.
[Source manifest](ticket15/source-manifest.json) SHA-256:
`5051e411f3a53a2428d414d11552c3c89f9f3bd88815d2671620333be92536b9`.

Eight changed source paths remain: Cargo.toml/Cargo.lock; lib.rs, main.rs,
storage.rs; new supervisor.rs, migration0005 and supervisor tests. Repair1 changes
only the three new supervisor-owned source paths. Shared glue is unchanged from
round0: module export, authoritative migration call, trusted local monitor CLI and
locked libc dependency. Accepted core authority/artifact/gateway source and core
v1 marker were preserved. Parent integrates Temporal0004 before supervisor0005.

## Limits

Observed host bubblewrap is 0.8.0; cgroup v2 is not delegated writable. No cgroup
CPU/memory enforcement, multi-host, external runtime or production adoption claim
is made. The supplied Git store and local broker/monitor operator are trusted.
Unavailable identity, persistent unknown namespace membership and interrupted
Git preparation remain fail-closed quarantine with capacity held; this slice does
not invent complete preparation membership or automatic release for such cases.
Kernel recovery/owner review remains necessary. Both independent review axes and
explicit acceptance remain outstanding.
