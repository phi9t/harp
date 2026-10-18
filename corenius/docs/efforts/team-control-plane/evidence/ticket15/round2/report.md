# Ticket 15 supervised environments — repair 2 candidate

Source is frozen for independent Standards and Spec re-review; this report does
not claim acceptance. No commit, push, model call, production mutation or mainline
change was performed. Read base `963f58b31e518a8162dfe719e386ef9d9843321f`;
implementation content baseline remains the exact accepted10+11+12 25-file
`ticket12-accepted-manifest.json`. The initial rejected candidate and its proof
are retained separately in [round0](ticket15/round0/report.md); the repair1 frozen
package and report are preserved in [round1](ticket15/round1/report.md). Canonical ticket15,
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

## Repair 2 authoritative observation and namespace incarnation

The actual liveness classifier now returns fallible observations. Identity/stat
EACCES, EIO and malformed state remain uncertainty; verified disappearance
(ENOENT/ESRCH), identity replacement and actual zombie state can establish absence.
Inventory propagates observation errors, every pidfd signaling check fails closed,
and reconciliation gathers fallible init/prior/controller observations before
an empty inventory can produce Stopped. A minimized injected-read regression
failed on the prior production classifier and passes after the fix. This proves
error classification under controlled reads, not an actual kernel permission race.

The unchanged strict concurrent fixture then exposed namespace inode reuse.
Diagnostic evidence records vanished init1733023 at start_ticks26777873 and a
newer live PID1733061 at26777881, NSpid `1733061 1`, under the same namespace
identifier. This newer PID1 proves a different incarnation. The monitor now
opens a namespace FD while init remains gated and verifies device/inode against
the current namespace path, inode against the recorded namespace and full init
identity before workload release. The FD stays held through complete inventory
and launcher reap. Temporary diagnostic code was removed; raw logs remain.

The pin prevents reuse during normal monitoring. No pin is asserted in the DB,
and external reconciliation does not infer a pin from controller liveness. After
monitor loss, the saved inode alone is not proof: unproven live membership remains
quarantine and is never adopted or signaled as belonging to the old execution.

## Fresh checks and frozen handoff

[Final gate record](ticket15/final-gate.json) retains exact commands, terminal exit
codes and full raw-log hashes, including both reds and namespace discriminator.
Focused real PG/OS suite:19passed8.45s. Classifier regression:1passed. Formatting,
clippy all targets with warnings denied, full `tools/agents/check.sh`:exit0. Full
gate supervisor19passed8.36s and terminal `check: all green`. No further source
changes or broad reruns followed these checks.

[Accepted12-relative full patch](ticket15/baseline-relative.patch) SHA-256:
`f3301a4537ead025449e95b5a971c3d511f13c5e0cc7e1812b2b07f66821290c`.
[Repair1-relative repair2 patch](ticket15/repair2.patch) SHA-256:
`7c8e2b71b96d80e8204def2489ca1a3092fc4f3b110b90b4f992ad97a5078d97`.
[Source manifest](ticket15/source-manifest.json) SHA-256:
`424c29750b73751a9877abb716fe6b370c4ec9c8c0332f5a20de4fadd9b6c866`.

Eight declared paths remain: Cargo.toml/Cargo.lock; lib.rs, main.rs, storage.rs;
new supervisor.rs, migration0005 and supervisor tests. Repair2 changes only
supervisor.rs; all shared glue, schema, fixtures and previous seven repairs remain
unchanged. Parent integrates Temporal0004 before supervisor0005. Source is frozen
for both independent reviews; no acceptance claim.

## Limits

Observed host bubblewrap is 0.8.0; cgroup v2 is not delegated writable. No cgroup
CPU/memory enforcement, multi-host, external runtime or production adoption claim
is made. The supplied Git store and local broker/monitor operator are trusted.
Unavailable identity, persistent unknown namespace membership and interrupted
Git preparation remain fail-closed quarantine with capacity held; this slice does
not invent complete preparation membership or automatic release for such cases.
Kernel recovery/owner review remains necessary. Both independent review axes and
explicit acceptance remain outstanding.
