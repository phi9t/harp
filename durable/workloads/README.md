# Admitted workloads

The workload adapter uses the same Store, Activity journal, guardian and
InsulaExecution as coding work. It adds no scheduler, controller, retry database
or service authority. Rootfs preparation freezes the runtime and native bytes;
workload admission freezes committed source plus a committed manifest and owner
oracle files. Reusing a run identity with different inputs refuses.

A manifest declares literal command argv, independent checks, bounded artifact
paths, timeout and optional native file/executable requirements. Workload code
belongs under `lab/`. `{source}` and `{output}` expand to an isolated committed
checkout and attempt-owned artifact directory. `/oracle/<path>` check arguments
refer to immutable bytes frozen from `oracle_files` at the task revision. Checker
qualification reuses the intent policy: pinned system executables, isolated
Python and frozen scripts or owner-admitted inline code. It never uses a mutable
workload checker as its acceptance authority.

Successful results bind source commit, task revision, manifest hash, execution
contract, attempt identity, check outcomes and artifact hashes. Retrieval is
limited to declared regular files,16MiB each and64MiB total. Output tails retain
at most64KiB per command. No background service owns these files: the existing
guardian proves descendant cleanup before the runtime accepts the result.

Workloads use manual recovery with one attempt. A transport failure does not
repeat an effect. Cancellation can prove cgroup cleanup while leaving a started
effect uncertain; that run remains blocked for explicit reconciliation. Proven
cleanup permits capacity reuse by unrelated work under the same runtime.
