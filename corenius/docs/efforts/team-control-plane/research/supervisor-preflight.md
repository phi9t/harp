# Supervisor preflight

Status: observed baseline; complete workload shutdown remains unproven
Read: 2026-10-01
Commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`

Observed host: bubblewrap 0.8.0, unified cgroup v2 controllers exposed
(cpuset/cpu/io/memory/hugetlb/pids), current session cgroup not writable by this
user, Python pidfd_open available. Availability is not delegated cgroup authority.
No cgroup directory or host containment policy was changed.

`python3 tools/insula/test_capsule.py` exited 0: 21 tests in 2.576s. It uses
actual disposable bubblewrap capsules and includes filesystem/env/network
isolation and a timeout return/record check. It emitted a ResourceWarning for
an unclosed stdout pipe in the streaming test. This result does not prove that
all detached descendants stop: the timeout test checks returned status and
record, not complete independently observed workload membership.

Inspected mechanisms: tools/insula/capsule.py uses start_new_session on its
launcher, bwrap --new-session/--unshare-pid/--die-with-parent, and launcher
killpg on timeout. tools/agents/worker.py adds an outer GNU timeout and its own
session. src/insula.rs kills the child on deadline and then joins output tasks.
src/supervisor.rs discovers descendants by a ps snapshot rooted at a PID.
These observations establish boundaries that ticket 15 must exercise, not
complete durable Execution handles or reliable inventory after reparenting.

Inference: cgroup-based enforcement would require explicit user-service
delegation or another supervised capability; it cannot be assumed available
because controllers exist. A first implementation can evaluate independently
supervised PID-namespace membership and pidfd-backed controller identity, but
must quarantine whenever complete membership/stop evidence is unavailable.
The architecture must not collapse this requirement into launcher-exit proof.

No product code, real model execution, shared worktree or production service
was changed. The existing lifecycle warnings are recorded, not silently fixed
as unrelated scope. Ticket 15 remains incomplete.
