# Agent workflows

[Documentation](README.md) / Agent workflows

Harp has two ways to run an installed agent CLI. Choose a provider workflow
for a single coding task with recorded context and output. Choose a durable
task graph when work needs persisted dependencies, results, and restart
recovery.

| Command | Purpose | Implemented live adapters |
| --- | --- | --- |
| `harp run` | Run a task with an immutable context release and record provider output | Codex CLI and Trae CLI |
| `harp rlm` | Execute a task graph with durable state and recovery | Codex CLI |

These are opt-in live runs. Install and authenticate the provider separately.
A run may contact its service, incur charges, and change files within the
provider's permissions. Harp does not supply model credentials or replace
provider security enforcement.

The examples assume you have [built the CLI](getting-started.md#build-the-cli)
and run it from the repository root. For brevity, they use `harp`; substitute
`.build/harp-target/size/harp` if it is not installed on PATH.

## Provider workflows

For Codex, inspect installed capabilities, then compile a local context release:

```sh
harp providers doctor --provider codex
harp releases compile
harp --format json releases list
```

For Trae CLI, use `harp providers doctor --provider trae` instead. The
unqualified `harp providers doctor` checks both providers and fails if either
is unavailable.

Inspect a release by replacing `RELEASE_ID` with an ID from the JSON list.
The plain-text list reports only a count:

```sh
harp --format json releases inspect RELEASE_ID
```

Start a live task only when you are ready to use the provider:

```sh
harp run --provider codex --workflow code_review -- review this change
harp run --provider trae --workflow auto -- fix the failing CI test
```

The task begins after `--`; that section is not a channel for native provider
arguments. The supported workflows are `auto`, `ci_repair`, `code_review`,
`dependency_update`, and `general_coding`.

Optional `--model`, `--profile`, `--sandbox`, and `--approval` values are
accepted only when the selected provider reports the required capability.
Native configuration, repository rules, and enforcement remain active. Harp
does not silently widen privileges, but explicit sandbox, approval, and profile
overrides may change the provider's native behavior.

### Output and records

During `harp run`, provider stdout requested in JSONL mode is copied
byte-for-byte. Malformed output is preserved. Harp lifecycle records and the
final episode ID go to stderr. This remains true with global `--format json`;
Harp does not wrap the live provider stream in a JSON envelope.

The pre-launch manifest is partial. It records the selected release, Harp's
context supplement, invocation identity, and pre-run repository state. It does
not reconstruct the provider's complete prompt, native skill selection, or
compaction state. Raw provider output and the completion receipt are separate
post-run records.

Private context-control state uses the first nonempty location:

1. `HARP_HOME`, used as the complete state-root path.
2. `${XDG_STATE_HOME}/harp`.
3. `${HOME}/.local/state/harp`.

Stateful commands fail if none is available. The state directory must be
privately owned and must not be symlinked or accessible by group or world.

### Repository restrictions

Optional `.harp/context-control.json` may disable Harp, restrict workflows,
pin a release, lower the context budget, or declare required verification
labels. It cannot inject prompts, select an executable, or broaden provider
authority.

Required verification labels are validated and included in the policy digest.
They do not yet execute verifiers or enforce an outcome gate. Automated
learning suggestions, evaluation, canarying, promotion, and rollback are also
deferred. See the [product contract](product-contract.md#local-context-control)
for the full boundary.

## Durable task graphs

`harp rlm` executes a JSON task graph through the durable engine. The
[task-graph schema](../benchmarks/codex-architecture/schemas/task-graph.json)
defines graph inputs; the
[result-envelope schema](../benchmarks/codex-architecture/schemas/result-envelope.json)
defines results.

Given a graph file, start a live Codex run:

```sh
harp rlm run --benchmark path/to/task-graph.json --runtime codex
```

Replace the placeholder with your graph path. Use
`--runtime-executable /path/to/codex` when Codex is not on PATH. The adapter
uses `codex exec --json`, supplies a per-activity output schema, and parses
bounded JSONL events.

State defaults to `.harp/rlm`:

| Path | Contents |
| --- | --- |
| `state.sqlite` | Runs, tasks, attempts, activities, leases, and recovery state |
| `artifacts/` | Content-addressed results, evidence, and checkpoints |
| `runtime-home/` | Supervised CLI process records and output spools |

Use the run ID returned by the CLI for status and recovery:

```sh
harp rlm checkpoint
harp rlm status RUN_ID
harp rlm resume RUN_ID
harp rlm resume --all-incomplete
```

Resume can observe an existing process or start work needed to finish the
graph. It is not a read-only status check. Each run pins graph policy,
projection policy, artifact-store identity, and runtime provenance. Resume
rejects mismatched runtime or artifact authority before observing processes
or starting new work.

The engine records process fingerprints and escalates cancellation through
SIGINT, SIGTERM, then SIGKILL when needed. The runtime options include `fake`
for deterministic tests. `traecli` is reserved but not implemented and fails
before semantic work starts. This is separate from the working Trae adapter
for `harp run`.

For the design rationale, read
[ADR 0002: dynamic workflows compile to durable task graphs](adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md).

## Proposer experiment and verification

The [Meta-Harness proposer lab](../labs/meta_harness_trae/README.md) is a separate,
opt-in experiment. It checks proposal shape and candidate interfaces. Its
recorded run does not establish a benchmark score, held-out evaluation, or
recursive self-improvement result.

`mise run verify` uses fake providers and deterministic fixtures. It never
launches a live Codex or Trae CLI task and does not require their credentials.
A passing offline gate does not verify a live provider session.
