# Ticket15 initial independent Spec review — unaccepted

Frozen package: `.scratch/team-control-plane/execution/review15`, 33 source/proof
files verified against accepted12 baseline and the worker's eight declared source
changes. Read base `963f58b31e518a8162dfe719e386ef9d9843321f` plus accepted12.
Independent Spec reviewer did not implement supervisor15. Reported green focused
and full checks do not discharge these five blockers:

1. `supervisor.rs:112–119,167–174`: Worker task/generation scope is not bound to
   attempts.worker_id, permitting another principal to control the attempt.
2. `supervisor.rs:236–242`: create_dir_all may follow a tracked symlink and write
   outside the clone before rejecting. Validate/no-follow components before any
   mutation; demonstrate an outside sentinel remains unchanged.
3. `supervisor.rs:610–621`: second launch transaction must recheck epoch,
   generation and deadline after controller claim, as well as cancellation. Forced
   changes between transactions must produce no workload effect.
4. `supervisor.rs:236–242,687–696`: directory-only mounts narrow admitted file
   scopes. Support actual file and directory paths, including explicit kind for
   missing paths; do not infer type from extensions or mount a parent writable.
5. `supervisor.rs:118–119`: current Owner must inspect/reconcile/cancel/release
   superseded executions for proven-stop cleanup/refund while stale workers stay
   fenced and relaunch remains prohibited. Direct SQL is not public recovery proof.

Registration/launch barrier, namespace inventory, pidfd stop and deletion
reservation provide useful partial proof. No cgroup/live/multi-host claim was
made. Standards review is pending; collect it before a single consolidated
repair1 brief. Three repair rounds maximum; none consumed for15 yet.

## Standards review completed

Sixth blocker: Supervisor::open creates directories before checking readiness,
then discards the selected supervisor schema version; run_monitor never checks
it. Use one nonmutating schema compatibility guard before any filesystem/process
effect in both startup seams. RealPG future-version and missing-marker fixtures
must reject without pollution or launch. Separate Standards otherwise found no
new blocker beyond Spec's five. Consolidated repair1 is now assigned; no
acceptance or scope narrowing.

## Preparation crash interval — seventh blocker

Primary inspection and separate Spec confirmation found registered state is
committed before synchronous Git preparation under an uncommitted lock. Creator
SIGKILL can leave Git children writing after the transaction/lock disappears.
Registered plus no capsule init then falsely becomes stopped and permits
deletion/refund. Initial AfterCreate fixtures kill only after preparation returns.
This confirmation is source evidence, not an experimental reproduction.

Author confirms the gap and includes durable Preparing-before-spawn state and
actual in-progress Git-child/creator-SIGKILL proof in consolidated repair1.
Unknown preparation must quarantine with capacity held and no directory reuse;
a database lock or dead creator PID alone cannot establish process shutdown.
No additional repair round or weakened contract was granted.
