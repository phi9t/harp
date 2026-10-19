# Durable framework

`durable` is Harp's single project-scoped agent execution framework. Git-managed
Markdown is intent authority; SQLite journals execution, attempts, receipts and
operator decisions. See the [operator guide](../docs/durable-runtime.md) for host
prerequisites and the [ownership ADR](../docs/adr/0001-lightweight-durable-runtime-ownership.md)
for architectural context.

Ownership follows the package boundaries:

- [intent/](intent/) reads committed tasks, dependencies and acceptance policy,
  then freezes their bytes and hashes. The journal is not an editable backlog.
- [collaboration/](collaboration/) binds frozen records to owner-authorized
  `allowed_paths`; worker output cannot redefine its assignment.
- [agents/](agents/) admits coding work and adapts providers to isolated workers.
  Start with Codex; Trae is an optional retained adapter, and deterministic
  fixtures support qualification. Provider sessions are not automatically resumed.
- [workflows/](workflows/) defines the admitted coding and verification graph.
- [runtime/](runtime/) owns capacity, attempts, cancellation and recovery;
  [state/](state/) owns the SQLite journal. Insula supplies containment and
  delegated cgroup supervision; runner supplies SSH transport where used.
- [git/](git/) captures proposals, runs independent verification and implements
  exact-candidate approval and landing. A successful worker produces a proposal,
  not merge authority.

Commands below follow `python3 -m durable --help` and subcommand help. Global
`--state` and `--execution` options precede the subcommand. Run from this
repository; all paths below are fictional examples.

```sh
python3 -m durable --help
python3 -m durable --state /example/project-runtime/state prepare --help
python3 -m durable --state /example/project-runtime/state admit --help
```

Prepare an immutable execution profile on Linux with private owned local state,
a native rootfs, provider binaries and an already-authorized writable cgroup v2
delegation. Python 3.11+, supported SQLite (at least 3.51.3), Git and the rootfs
closure described in the operator guide are prerequisites. Do not install services
or acquire delegation implicitly.

```sh
state=/example/project-runtime/state
execution=/example/project-runtime/execution-v1.json
python3 -m durable --state "$state" --execution "$execution" prepare \
  --rootfs /example/native-rootfs --cgroup-root /sys/fs/cgroup/example-delegation \
  --network host --providers-bin /example/provider-binaries \
  --codex-auth /example/private/codex-auth.json
```

Preparation pins runtime source, rootfs and provider bytes. After source, rootfs,
provider or delegation changes, prepare a fresh immutable profile at a new path.
Never edit or rebind old profiles or attempts. Retained runs keep their original
execution identity. Before using fresh work with retained state, reconcile active
attempts, cleanup blocks and controller ownership. Keep owner capabilities outside
state, source, rootfs and every worker projection; expose only selected provider
authentication, never an entire home or configuration directory.

Commit the task Markdown and acceptance JSON before admission. Acceptance must
include explicit `allowed_paths` and owner-controlled checks. For example:

```json
{
  "allowed_paths": ["example/module.py"],
  "oracle_files": [".scratch/example/check.py"],
  "checks": [["/usr/bin/python3", "-I", "/oracle/.scratch/example/check.py"]]
}
```

The owner freezes oracle bytes separately; `/oracle` checks do not come from the
candidate. Use isolated Python (`-I`) for checks. Candidate edits cannot replace
the frozen parent, dependencies, scope or verifier. Admission reads committed
revisions and binds the base, workflow, provider and execution contract. Run IDs
are idempotency keys; conflicting reuse is refused.

```sh
python3 -m durable --state "$state" --execution "$execution" admit example-001 \
  --repository /example/repository --task .scratch/example/task.md \
  --acceptance .scratch/example/acceptance.json \
  --base HEAD --task-revision HEAD --provider codex
python3 -m durable --state "$state" --execution "$execution" start \
  --capacity 1 --until-idle --timeout 1800
python3 -m durable --state "$state" --execution "$execution" status example-001
```

Start and stop are explicit and project-scoped. `start` runs in the foreground;
there is no installed always-on controller. In another terminal, request draining
of this state's current instance with:

```sh
python3 -m durable --state "$state" --execution "$execution" stop
```

A second controller for the same state is refused. Stop targets an instance token,
not a stale PID; inspect status to confirm cleanup. Qualify Codex at capacity one,
then two, then four, inspecting isolation, receipts and cleanup at each stage.
The CLI accepts capacities 1–4; capacity becomes reusable only after proven cleanup.

Capture and independent verification are separate from execution:

```sh
python3 -m durable --state "$state" --execution "$execution" capture example-001
python3 -m durable --state "$state" --execution "$execution" verify example-001
python3 -m durable --state "$state" --execution "$execution" start \
  --capacity 1 --until-idle --timeout 1800
```

Inspect the returned verification run ID with `status`; inspect retained logs,
`events` and `check` for execution history and SQLite integrity. Verification
binds the captured base, commit, tree, checks and execution identity.

Only the human owner, after reviewing that exact candidate and evidence, may run
`owner-init --capability PATH`, then `approve RUN_ID --verification VERIFY_RUN_ID
--destination PATH --capability PATH`, then `land APPROVAL_ID --capability PATH`,
with the same global state/execution options. Approval must be explicit and
exact-candidate-bound before landing. Agents never impersonate approval or receive
the private owner capability. A dirty or advanced destination is refused; a changed
base requires a fresh integration candidate, verification and human approval.

For ambiguous recovery, inspect status, retained receipts, logs and owned cgroup
evidence first. A valid durable receipt replays without rerunning work; a started
uncertain effect is not automatically retried. `cancel RUN_ID` requests cancellation
but does not prove cleanup. Use `ack-cleanup ATTEMPT_ID --evidence TEXT` only with
actual cleanup proof, and `resolve RUN_ID KEY {retry,fail,succeed} --evidence TEXT
[--result JSON]` for each uncertain effect after deciding its semantics. Cleanup
and effect resolution are separate decisions; neither command launches work.
Missing cleanup proof blocks capacity reuse. Explicitly `start` only after
reconciliation. Interrupted landing likewise reconciles durable intent and exact
Git identities before another mutation; never force or blindly repeat effects.
