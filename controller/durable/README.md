# Insula-local durable execution

This Python component runs short replayable workflows and supervised executable
activities. SQLite stores operational state. Git and the existing Rust controller
retain ticket, Candidate, acceptance and landing authority.

The runtime re-enters each workflow from its beginning. Completed operations
return stored results. It rejects changes to operation order, keys, kinds and
specifications. It does not save Python stacks, Rust futures or training memory.
Workflow functions must put external effects in activities.

## Local reference execution

Run commands from the Corenius repository root on Linux with Python 3.11 or newer.
No third-party Python packages are required.

```bash
python3 tools/temp_root.py -- python3 -m controller.durable \
  --state "$PWD/.scratch/durable-demo/state" submit example \
  --workflow pipeline --args '{"value":3}'
python3 tools/temp_root.py -- python3 -m controller.durable \
  --state "$PWD/.scratch/durable-demo/state" run --max-active 2
python3 tools/temp_root.py -- python3 -m controller.durable \
  --state "$PWD/.scratch/durable-demo/state" status example
```

The result contains `sum: 181`. Direct execution uses trusted process groups and
does not place the payload in an Insula. Use the launcher below for that boundary.

Normal operation rejects SQLite versions outside the fixed release lines
specified by the supplied design: 3.51.3+, 3.50.7+, or 3.44.6+ on their respective
branches. For disposable demonstrations on older libraries, add
`--allow-unpatched-sqlite` before the subcommand. Keep that override out of
important persistent state. See [validation](../../docs/efforts/insula-durable/VALIDATION.md).

## Admit an Insula execution

The launcher uses `tools/insula/capsule.py`. It freezes the Python code in the
existing content-addressed tree store. The execution file binds the full capsule
record, code digest, state path and backend. Its identity is local admission
evidence; it is not a signed rootfs attestation.

Prepare an existing capsule request, or use an existing capsule digest:

```bash
python3 tools/temp_root.py -- python3 -m tools.insula.durable prepare \
  --capsule path/to/request.json \
  --state "$PWD/.scratch/durable-insula/state" \
  --execution "$PWD/.scratch/durable-insula/execution.json" \
  --backend process-group
```

Use the same execution file for submission, execution and recovery:

```bash
python3 tools/temp_root.py -- python3 -m tools.insula.durable exec \
  --execution "$PWD/.scratch/durable-insula/execution.json" -- \
  submit example --workflow pipeline --args '{"value":3}'
python3 tools/temp_root.py -- python3 -m tools.insula.durable exec \
  --execution "$PWD/.scratch/durable-insula/execution.json" -- run --max-active 2
```

Each activity gets readonly code at `/workspace`, its own writable attempt
directory at `/home/bench/run`, and private temporary storage at
`/run/corenius-tmp`. The controller and guardian stay outside the payload
namespace. No additional keeper or tmux server is created. A caller may run the
foreground controller under an existing lifecycle owner.

Use a suitable local filesystem for state. Keep its absolute location stable.
The database WAL can contain committed state; copying only `runtime.sqlite`
is not a supported live backup.

## Compare process ownership backends

| Backend | Payload assumptions | Cleanup evidence | Admission |
| --- | --- | --- | --- |
| `process-group` | Trusted children remain in their original group | Guardian observes no live group members | Linux `/proc` |
| `cgroup` | Payload runs inside the admitted Insula without cgroup controls | `cgroup.kill`, then `cgroup.events` reports `populated=0` | Explicit writable delegated cgroup v2 root |

For cgroups, prepare a separate execution file and state directory:

```bash
python3 tools/temp_root.py -- python3 -m tools.insula.durable prepare \
  --capsule path/to/request.json \
  --state "$PWD/.scratch/durable-cgroup/state" \
  --execution "$PWD/.scratch/durable-cgroup/execution.json" \
  --backend cgroup --cgroup-root /sys/fs/cgroup/your-delegated-root
```

Missing delegation refuses admission. There is no automatic fallback. The
guardian places the child in an attempt-bound subgroup before executing the
Insula launcher. Cgroup controls remain outside the payload. Recovery verifies
the recorded cgroup identity before cleanup. If cleanup cannot be proved, global
dispatch stops. CPU and GPU slots are logical admission accounting; they do not
provide CPU quotas or per-device GPU isolation.

## Workflow and activity APIs

```python
from controller.durable import Activity, Context

def workflow(ctx: Context, args: dict):
    result = ctx.activity("compute", Activity(
        argv=("/usr/bin/my-activity",), input=args,
        recovery="retry_safe", max_attempts=3,
    ))
    ctx.sleep("cooldown", 1)
    approval = ctx.wait_signal("approval")
    return {"result": result, "approval": approval}

REGISTRY = {"example": ("example-v1", workflow)}
```

Pass `--registry package.module` before the CLI subcommand. For Insula execution,
include that import root with `prepare --workflow-source DIRECTORY`. It must not
replace the runtime's `controller` or `tools` packages. Activities use absolute
executables and JSON stdin. They publish bounded JSON to `result.json` in the
request's `attempt_dir`. Large artifacts belong in domain-owned files.

`ctx.parallel([(key, Activity(...)), ...])` returns results in declaration order.
Timers retain their original deadlines. Signals are immutable one-shot keyed
values. Resubmitting a run ID deduplicates identical immutable inputs and rejects
conflicting inputs.

## Recovery and operator decisions

The controller persists claim and release intent before the guardian starts an
activity. The guardian retains the inherited controller lock until cleanup.
A durable receipt can be recovered after controller death without rerunning the
payload. Only the current attempt can publish an authoritative result.

`Activity` defaults to `recovery="manual"`. A started interrupted manual activity
remains `UNCERTAIN`, even after cancellation or sibling failure. `retry_safe` is
an application promise that repetition is safe or that the destination enforces
the stable idempotency key. The runtime does not guarantee exactly-once external
effects.

Inspect `status`, `events` and `check` before reconciliation. `ack-cleanup`
requires evidence that old processes are gone. `resolve RUN KEY retry|fail|succeed`
requires evidence about external effects; `succeed` also takes `--result JSON`.
These commands record decisions and launch no work. A later `run` performs
execution. Evidence strings are trusted operator assertions.

## Tests and code map

```bash
python3 tools/temp_root.py -- python3 -m unittest \
  controller.durable.runtime_test controller.durable.insula_test \
  controller.durable.cgroup_test -v
```

Set `CORENIUS_TEST_CGROUP_ROOT` to an explicitly delegated root to enable kernel
cgroup tests. They skip when delegation is absent; skips do not establish cgroup
correctness. Tests on an older SQLite library use disposable state only.

For a result that requires both backends and refuses skips, use the
[qualification command](../../tools/agents/VALIDATION.md#qualify-both-durable-backends).
It supports a dedicated systemd user scope and checks production SQLite admission.

- `engine.py`: operation replay, SQLite state, controller and receipt acceptance.
- `guard.py`: gated launch, process ownership and durable completion receipt.
- `cgroup.py`: delegated attempt identity and kernel cleanup evidence.
- `insula.py`: code snapshot, execution binding and activity projections.
- `cli.py`, `workflows.py`, `demo_activity.py`: operator CLI and examples.
- `common.py`: bounded canonical JSON and filesystem publication.

The supplied reference archive has SHA-256
`30f55eef3f5aa10b84df0c9b8d31aa0ec28300dc08da7acfd53ccd8be88f180b`.
Corenius adaptation and observed validation are recorded in the durable effort.
DBOS remains an alternative orchestration owner; no DBOS adapter is implemented.
