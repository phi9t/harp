# Keep local workflow execution under the controller

The user approved the supplied Python durable-execution design and asked to try
both process-group and cgroup activity ownership. Place the execution-local
runtime in `controller/durable/`, consistent with ADR-0001's ownership boundary.
Keep its Insula operational entry point in `tools/insula/durable.py`.

SQLite records workflow execution, ordered operations, attempts, signals and
recovery evidence. The existing Rust controller retains Git/ticket, Candidate,
verification, acceptance and landing authority. This component creates no
second keeper, independent publication policy or distributed scheduler.

Retain Python orchestration and direct executable activities. A Rust port would
replace a runnable semantic reference before the local contract is qualified.
DBOS is an alternative orchestration implementation, not a wrapper scheduler;
adopting it later requires the same activity ownership and fault evidence.

The admitted launcher freezes code through the existing Insula tree store and
binds the full capsule record, code, persistent state location and backend.
Controllers and guardians remain outside each activity's bubblewrap namespace.
Each activity gets readonly code, its own writable attempt directory and private
temporary storage. This allows cgroup management without exposing migration
controls to the payload. Existing lifecycle services may own the foreground
controller; this component starts none.

Process groups support trusted activities that do not leave their group. Cgroup
v2 requires explicit delegation and kernel cleanup evidence. Missing cgroup
capability refuses execution. A deployment must qualify its selected backend;
successful process-group tests cannot establish cgroup correctness.
