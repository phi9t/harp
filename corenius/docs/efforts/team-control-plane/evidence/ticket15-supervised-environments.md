# Ticket 15 supervised environments — repair 3 candidate

Source is frozen for independent Standards and Spec re-review; this report does
not claim acceptance. No commit, push, model call, production mutation or mainline
change was performed. Read base `963f58b31e518a8162dfe719e386ef9d9843321f`;
implementation content baseline remains the exact accepted10+11+12 25-file
`ticket12-accepted-manifest.json`. The initial rejected candidate and its proof
are retained separately in [round0](ticket15/round0/report.md); the repair1 frozen
package and report are preserved in [round1](ticket15/round1/report.md).
Repair2 frozen package/report remain in [round2](ticket15/round2/report.md). Canonical ticket15,
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

## Final repair 3 incarnation authority

Repair2's external cached live-identity proof could outlive the original namespace
and its sole monitor pin, then select unrelated members under a reused inode. The
controlled observation-boundary regression exercises the extracted production
decision: prior behavior signals one replacement member, expected zero (red101);
current behavior signals none and preserves uncertainty. This is deterministic
observation substitution, not a reproduced adversarial kernel race. Earlier
actual namespace reuse and all red/green logs remain retained.

The monitor verifies original namespace PID1 through NSpid ending1 while workload
is blocked. The persisted ProcessIdentity now includes namespace_pid; older
records without it have no PID1 certificate. Once set, init_process is immutable.
Observer frame is checked at launch and every reconciliation: actual procfs,
/proc/self/stat PID equal getpid, same recorded controller namespace and boot.
Foreign/unproven frames cannot classify or signal original host PIDs.

External reconciliation first checks certified original PID1 finalization using
independently fallible host stat. Verified reaping/PID replacement under that
frame and boot/start fence, or correctly fenced default whole-process pidfd
readiness, proves original-group finalization. Namespace-link disappearance,
generic alive=false, live S/D, leader Z alone, malformed data and unreadable
observations do not establish completion. Controlled tests cover those negatives,
missing certificate, boot/frame mismatch and host PID replacement.

Otherwise external reconciliation obtains its own namespace FD from a current
live recorded identity. It holds that descriptor through inventory, classification
and every pidfd signal. Monitor reconciliation passes its actual gated-launch FD
explicitly, checked against the immutable recorded namespace; external callers
cannot borrow a controller-liveness inference. If no finalization or pin can be
established, a live controller allows deferral without inventory/Stop/signals;
otherwise the execution is quarantined with capacity retained.

The new real public broker/PG/capsule fixture verifies persisted NSpid1 and
startticks, binds a default whole-process pidfd while the owned init is live,
kills the controller, observes POLLIN and subsequent original stat absence,
checks heartbeat writes remain stopped over60ms, then performs publicStop/release.
The strict existing32-capsule, nested setsid and controller/launcher-loss fixtures
remain unchanged. The independently retained gated bwrap diagnosis additionally
verified no workload effects before unblock and corroborated the same teardown.

The finalization inference follows upstream [v5.15 namespace teardown](https://raw.githubusercontent.com/torvalds/linux/v5.15/kernel/pid_namespace.c),
[v5.15 exit ordering](https://raw.githubusercontent.com/torvalds/linux/v5.15/kernel/exit.c)
and [default whole-process pidfd semantics](https://man7.org/linux/man-pages/man2/pidfd_open.2.html).
The parent-integrated `research/pid1-finalization-contract.md` and
`evidence/pid1-finalization-observations.json` document its source hashes and
actual bounded probe. Those observations were read, not rerun here. Host vendor
Linux5.15.152.bsk.9-amd64 is not attested identical to upstreamv5.15; the actual
fixture corroborates this host only. Kernel task finalization does not establish
completion of previously submitted asynchronous external service effects.

## Fresh checks and frozen handoff

[Final gate record](ticket15/final-gate.json) records complete raw-log hashes and
commands. Controlled observation/classification tests4passed; focused real
PG/OS20passed8.14s; formatting0, clippy all-targets warningsdenied0 and fullgate0.
Final fullgate supervisor20passed8.01s and terminal `check: all green`. Initial
clippy rejected a test-only cloned-reference lint; it was corrected and fresh
format/clippy/gate run on the final source. No source edits followed these checks.

[Accepted12-relative full patch](ticket15/baseline-relative.patch) SHA-256:
`c97c23028ae0851deb8a2fbb579fa330f4cc1aa29c2b100c480c12ed55b618ae`.
[Repair2-relative final repair patch](ticket15/repair3.patch) SHA-256:
`67a7db310a03edac03a4655cd1b69c6864c52e1308e227042a88876afac4e1e2`.
[Source manifest](ticket15/source-manifest.json) SHA-256:
`78971e5090475be2b3b7843f3d9ecad797c35d1df5a71b021efe1f917896af9b`.

Eight total source paths remain declared. Repair3 changes only supervisor.rs,
migration0005.sql and supervisor tests; shared Cargo/lib/main/storage glue
remains unchanged from round0. Parent integrates Temporal0004 before0005.
Round0,round1,round2 remain separately frozen. This is the final authorized third
repair; further blocking findings require architectural review/attention, not an
unannounced fourth patch. No acceptance or independent-review pass claim.

## Limits

Observed host bubblewrap is 0.8.0; cgroup v2 is not delegated writable. No cgroup
CPU/memory enforcement, multi-host, external runtime or production adoption claim
is made. The supplied Git store and local broker/monitor operator are trusted.
Unavailable identity, persistent unknown namespace membership and interrupted
Git preparation remain fail-closed quarantine with capacity held; this slice does
not invent complete preparation membership or automatic release for such cases.
Kernel recovery/owner review remains necessary. Both independent review axes and
explicit acceptance remain outstanding.
