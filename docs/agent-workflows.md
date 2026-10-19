# Agent workflows

[Documentation](README.md) / Agent workflows

Use [durable coding work](durable-runtime.md) to run Codex or Trae through Harp.
The project-scoped Python + SQLite runtime owns admission, attempts, capacity,
cancellation and recovery. Each worker receives committed task and acceptance
inputs, an isolated source workspace and an admitted execution profile.

A completed worker produces a proposal. Independent verification binds the exact
candidate; human approval and checked-out local landing are separate operations.
The [operator guide](durable-runtime.md) covers preparation, execution, retained
logs, reconciliation, verification and landing on Linux.

Provider authentication is supplied explicitly during preparation. Starting a
live coding worker can contact its provider and incur charges. Offline repository
verification uses deterministic fixtures and never starts a live model session.

## Retired execution commands

| Previous command | Current disposition |
| --- | --- |
| `harp run --provider … --workflow … -- TASK` | Retired. Commit task Markdown and acceptance JSON, then use `harp durable admit` and `start`. |
| `harp rlm run` or `resume` | Retired. Task graphs are not automatically converted into coding tasks. |
| `harp rlm status` or `checkpoint` | Retired. Existing graph databases are retained without opening or migrating them. |
| `harp workflow run` or `resume` | Retired. Dynamic Workflow IR is not executed through the old scheduler. |
| `harp workflow status`, `inspect` or `explain` | Retired. These commands do not open historical databases or resume processes. |

Valid historical argument spellings remain recognizable so commands can return
an actionable retirement error. They do not forward provider flags, probe an
executable, read graph inputs, create state or start a worker. Old task-graph and
context-control state is not imported into the selected runtime. Preserve that
state for historical inspection; a new admission pins its own explicit inputs.

The old `harp run` byte-for-byte provider stdout contract ended with that
execution path. The selected runtime reports lifecycle data and retains provider
stdout/stderr in attempt logs; it does not emulate the old live stream wrapper.
The context-control `run` library entrypoint also refuses direct execution.

## Retained context tools

Capability diagnostics and immutable context releases remain available:

```sh
harp providers doctor --provider codex
harp providers doctor --provider trae
harp releases compile
harp --format json releases list
harp --format json releases inspect RELEASE_ID
```

Replace `RELEASE_ID` with an identifier from the list. The doctor commands perform
bounded local help/version probes. They do not establish that a provider is ready
inside a particular execution profile; preparation verifies that profile separately.

Releases retain content-addressed context descriptions and their schemas. They
do not admit work, grant permissions or start provider sessions. The historical
`.harp/context-control.json` policy and workflow names are not implicitly translated
into the selected runtime's execution contract.

Pure graph schemas, validation and decoding helpers remain useful library and
reference material. Their presence does not expose another supported scheduler.
Historical design records, including [ADR0002](adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md),
describe the earlier architecture.
