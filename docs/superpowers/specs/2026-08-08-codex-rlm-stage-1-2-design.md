# Codex-RLM stage 1+2 design

**Status:** Approved for implementation planning
**Date:** 2026-08-08
**Repository:** `~/workspace/harp`
**Codex source:** `~/workspace/codex`

## 1. Objective

Build a thin, end-to-end Codex-RLM vertical slice in Rust.

The system will:

1. represent a large task as external artifacts and bounded model-visible metadata;
2. ask a root Codex thread to generate a schema-constrained `TaskGraph`;
3. validate the complete graph before executing any child work;
4. launch fresh, depth-one Codex children programmatically;
5. retain detailed child output in artifacts while returning only typed result envelopes;
6. execute the same graph through process and in-process Codex adapters;
7. survive a `harp` restart and resume from durable task state;
8. evaluate the result with deterministic checks and a protected semantic rubric; and
9. record enough operational data to compare the adapters through repeated use rather than choosing one in advance.

The first workload analyzes `~/workspace/codex` and produces an evidence-linked architecture report covering App Server, multi-agent execution, persistence, token accounting, workspace controls, and their implications for an RLM-style recursive harness.

This milestone implements RLM state and depth-one recursive execution. It does not implement recursive grandchildren, root-policy training, or an outer RSI mutation loop.

## 2. Design principles

### 2.1 RLM is an information-flow architecture

Large task-specific data remains in external artifacts. The root sees:

- the task specification;
- corpus metadata;
- artifact handles;
- graph status;
- bounded result summaries; and
- remaining budget.

The root does not automatically receive:

- the complete corpus;
- child transcripts;
- large command output;
- sibling results; or
- complete intermediate artifacts.

### 2.2 Planning and execution are separate

The root generates a `TaskGraph`. It does not receive App Server credentials or execute arbitrary orchestration code.

`harp` owns:

- graph validation;
- scheduling;
- runtime calls;
- retries;
- cancellation;
- budget enforcement;
- persistence;
- artifact publication; and
- evaluation.

### 2.3 Fresh children are the default

Every analysis child starts through App Server `thread/start` with a projected context. No child inherits the root transcript.

History forking is outside this milestone. A later continuation of the same interrupted child may use `thread/resume`, because that operation preserves one task's own durable context rather than contaminating a new sibling.

### 2.4 Durable state precedes side effects

Before `harp` performs a Codex or filesystem side effect, it commits durable intent to SQLite. After the effect, it records the observed external identity and outcome.

External effects remain at-least-once. Accepted task results are exactly-once.

### 2.5 Adapter choice remains empirical

The process and in-process adapters implement one behavior-oriented contract. The benchmark records correctness, latency, resource use, failures, and operational complexity. The design does not declare either adapter the long-term winner.

### 2.6 Disk use is a hard constraint

Normal `harp` development must not compile Codex. Rust builds must not emit DWARF or ordinary debug artifacts. The heavyweight in-process experiment is explicit, isolated, and disposable.

## 3. Scope

### 3.1 Included

- Rust workspace for the core harness;
- external task manifests and content-addressed artifacts;
- schema-constrained root planning;
- validated depth-one `TaskGraph` execution;
- fresh child Codex threads;
- typed `ResultEnvelope` outputs;
- process App Server adapter;
- separately built in-process App Server adapter;
- global and per-task budgets;
- resumable task execution after process failure;
- deterministic and semantic evaluation;
- adapter conformance tests;
- failure-injection tests; and
- release-oriented, no-DWARF build tasks.

### 3.2 Deferred

- grandchildren or arbitrary recursion depth;
- arbitrary model-generated Python execution;
- a general workflow engine;
- Temporal dependency or Temporal cluster;
- complete event-history replay;
- workflow version markers and Continue-As-New;
- distributed workers;
- worktree-based code modification;
- root-policy fine-tuning;
- DGM-style harness mutation;
- hidden production benchmarks; and
- modifications to Codex core.

## 4. Repository and build architecture

```text
harp/
├── Cargo.toml
├── Cargo.lock
├── .cargo/
│   └── config.toml
├── crates/
│   ├── harp-contracts/
│   ├── harp-runtime/
│   ├── harp-app-server-process/
│   ├── harp-engine/
│   ├── harp-artifacts/
│   ├── harp-eval/
│   └── harp-cli/
├── benchmarks/
│   └── codex-architecture/
├── experiments/
│   └── codex-inprocess/
├── docs/
│   └── superpowers/
└── mise.toml
```

### 4.1 Main workspace

The main workspace contains the engine, contracts, artifact store, evaluator, process adapter, CLI, and benchmark.

It has no dependency on Codex Rust crates. It communicates with an App Server binary built from the sibling checkout.

### 4.2 In-process experiment

`experiments/codex-inprocess/` is an independent Cargo build universe and is excluded from the main workspace. Its manifest contains its own empty `[workspace]` root and path-depends on:

```text
../../../codex/codex-rs/app-server-client
```

This resolves to `~/workspace/codex/codex-rs/app-server-client` in the agreed checkout layout.

The in-process package reuses the same `harp-engine` and runtime contracts. It must not implement a second scheduler or different task semantics.

### 4.3 Codex provenance

Codex is never copied, vendored, or committed into `harp`.

Every build and benchmark records:

- canonical Codex checkout path;
- Codex commit SHA;
- dirty or clean status;
- a digest of the dirty diff when applicable;
- App Server protocol/schema fingerprint;
- adapter kind;
- model configuration; and
- `harp` commit SHA.

Experiments from different Codex source states are not treated as equivalent samples.

### 4.4 Small-build policy

All `harp` Cargo profiles, including `dev`, `test`, and the optimized profile used by tasks, explicitly set:

```toml
debug = 0
split-debuginfo = "off"
incremental = false
strip = "symbols"
```

Routine build and test tasks use an optimized size-oriented profile. They do not use Cargo's default artifact behavior implicitly.

The in-process package defines the same no-DWARF profiles because the root package's profiles govern dependency compilation. Codex's own release profile is not reused: the surveyed Codex release profile retains line tables and unstripped symbols, which violates this project's disk constraint.

Build requirements:

- one configured `CARGO_TARGET_DIR` per build universe;
- no per-command target directory proliferation;
- no checked-in `target/`;
- no DWARF sidecars;
- no incremental compilation cache;
- preflight free-space check before heavyweight builds;
- explicit `mise run build-inprocess` for the Codex-linked build;
- copy only the stripped executable and a provenance manifest to a commit-addressed cache;
- remove heavyweight intermediate artifacts after a successful packaged build; and
- report target-directory size before and after every heavyweight task.

Normal `harp` tests must remain runnable without resolving or compiling Codex crates.

## 5. Core components

### 5.1 `harp-contracts`

Owns stable serializable types:

- `ArtifactRef`;
- `TaskManifest`;
- `TaskGraph`;
- `TaskNode`;
- `Budget`;
- `ThreadSpec`;
- `TurnSpec`;
- `RuntimeEvent`;
- `ResultEnvelope`;
- `EvaluationRecord`; and
- durable run, task, attempt, and operation states.

This crate contains no scheduler, filesystem, SQLite, or Codex transport logic.

### 5.2 `harp-runtime`

Defines the behavior-oriented `CodexRuntime` interface and normalized error types.

The contract includes:

```rust
trait CodexRuntime {
    fn start_thread(
        &self,
        spec: ThreadSpec,
    ) -> impl Future<Output = Result<ThreadHandle, RuntimeError>> + Send;

    fn start_turn(
        &self,
        thread: &ThreadHandle,
        spec: TurnSpec,
    ) -> impl Future<Output = Result<TurnHandle, RuntimeError>> + Send;

    fn read_thread(
        &self,
        thread: &ThreadHandle,
    ) -> impl Future<Output = Result<ThreadSnapshot, RuntimeError>> + Send;

    fn resume_thread(
        &self,
        thread: &ThreadHandle,
    ) -> impl Future<Output = Result<ThreadHandle, RuntimeError>> + Send;

    fn events(
        &self,
        turn: &TurnHandle,
    ) -> impl Stream<Item = Result<RuntimeEvent, RuntimeError>> + Send;

    fn interrupt(
        &self,
        turn: &TurnHandle,
    ) -> impl Future<Output = Result<(), RuntimeError>> + Send;

    fn shutdown_thread(
        &self,
        thread: ThreadHandle,
    ) -> impl Future<Output = Result<(), RuntimeError>> + Send;
}
```

The concrete Rust shape may use associated types where required to keep the trait object-safe or avoid allocation. The semantic contract above is normative.

### 5.3 `harp-engine`

Owns:

- graph validation;
- ready-node calculation;
- task leasing;
- concurrency control;
- durable attempt transitions;
- operation reconciliation;
- budget reservations and accounting;
- reducer scheduling;
- cancellation; and
- run completion.

The scheduler is deterministic over the committed graph and authoritative SQLite state. Model calls do not make hidden scheduler decisions.

### 5.4 `harp-artifacts`

Stores immutable content-addressed objects and run manifests.

```text
.harp/runs/<run-id>/
├── manifest.json
├── graph.json
├── events.jsonl
├── metrics.json
├── tasks/<task-id>/attempts/<attempt>/
└── objects/sha256/<prefix>/<digest>
```

SQLite stores metadata, graph state, leases, attempts, operations, budgets, and artifact references. Large content remains in immutable files.

Publication uses:

1. write to a temporary file in the destination filesystem;
2. flush and `fsync`;
3. calculate and verify the digest;
4. atomically rename into the content-addressed location; and
5. commit the metadata reference in SQLite.

Publishing the same bytes more than once is harmless.

### 5.5 `harp-eval`

Runs deterministic contract checks and invokes a protected semantic evaluator.

The evaluator receives the final report, evidence manifest, and rubric. It does not receive child reasoning traces.

## 6. Data contracts

### 6.1 Artifact reference

```rust
struct ArtifactRef {
    uri: String,
    sha256: String,
    media_type: String,
    size_bytes: u64,
    logical_schema: Option<String>,
    permitted_ranges: Option<Vec<String>>,
}
```

An `ArtifactRef` is immutable. A changed object receives a new digest and URI.

### 6.2 Task graph

Each task node declares:

- stable task ID;
- role;
- instruction;
- input artifact references;
- dependency IDs;
- workspace mode;
- model policy;
- permission profile;
- token, wall-time, and storage budgets;
- output schema;
- retry policy; and
- whether it is an analysis or reducer node.

The first milestone accepts only:

- one root planning operation;
- depth-one analysis children; and
- one terminal reducer.

Children cannot create new graph nodes.

### 6.3 Result envelope

```rust
struct ResultEnvelope {
    task_id: String,
    status: ResultStatus,
    answer_ref: Option<ArtifactRef>,
    evidence: Vec<ArtifactRef>,
    trace_ref: ArtifactRef,
    summary: String,
    metrics: BTreeMap<String, f64>,
    confidence: Option<f64>,
    failure_class: Option<String>,
}
```

`summary` has a strict byte and token limit. Full answers and evidence remain in artifacts.

## 7. Root planning and graph validation

### 7.1 Root input

The root receives:

- benchmark task;
- corpus manifest handle;
- repository metadata;
- allowed roles;
- allowed output schemas;
- global budget;
- graph limits; and
- the JSON Schema for `TaskGraph`.

The root does not receive the complete repository contents in its initial prompt.

### 7.2 Root output

App Server `turn/start` uses `output_schema` to constrain the final root response to `TaskGraph`.

`harp` validates the complete graph before launching children.

### 7.3 Validation requirements

A graph is accepted only if:

- node IDs are unique;
- dependencies reference existing nodes;
- dependencies are acyclic;
- every input is an approved artifact;
- requested roles and schemas are allowlisted;
- task count and concurrency fit host limits;
- aggregate requested budgets fit the remaining run budget;
- writable paths are disjoint;
- all analysis paths reach the reducer;
- the reducer depends on every required analysis result;
- prompts and projected artifacts fit size limits; and
- no task requests recursion beyond depth one.

An invalid plan returns compact machine-readable diagnostics for one repair attempt. A second invalid plan fails the run.

## 8. Child execution

### 8.1 Context projection

Each child receives:

- base and role instructions;
- one bounded task instruction;
- materialized approved inputs;
- artifact handles;
- private scratch path;
- output schema;
- task budget;
- checkpoint instructions; and
- stable task and operation IDs.

It does not receive:

- root transcript;
- sibling transcript;
- unrelated user history;
- complete child result collections; or
- control-plane credentials.

### 8.2 Workspace

Analysis children receive:

```text
shared read-only Codex corpus
+
private writable task scratch directory
```

The first milestone does not modify the Codex checkout.

### 8.3 Result flow

The child writes detailed output and evidence into its private scratch directory, publishes a schema-constrained `ResultEnvelope`, and returns only that bounded envelope through the runtime.

The reducer receives result references and bounded summaries. It does not receive complete child transcripts.

## 9. Runtime adapters

### 9.1 Process adapter

The process adapter:

- uses an App Server binary built from `~/workspace/codex`;
- speaks App Server v2 JSON-RPC;
- performs initialization and capability negotiation;
- starts fresh threads through `thread/start`;
- starts schema-constrained turns through `turn/start`;
- reads and resumes persisted threads for recovery;
- normalizes notifications into `RuntimeEvent`;
- preserves completion and token-usage events losslessly;
- captures bounded stderr as an artifact; and
- distinguishes process exit, protocol drift, malformed frames, lag, and server errors.

The first implementation may use a managed stdio App Server process. A `harp` restart starts a new App Server against the same Codex state directory and resumes persisted thread IDs. It does not claim that an old stdio stream can be reattached.

### 9.2 In-process adapter

The in-process adapter:

- uses the sibling Codex `codex-app-server-client` crate;
- converts typed `AppServerEvent` values into the same normalized stream;
- runs the same engine and benchmark;
- uses the same persistence and recovery transitions; and
- must not exploit internal Rust access to bypass the runtime contract.

After a `harp` crash, the embedded App Server no longer exists. Recovery starts a new in-process App Server instance against the same Codex state directory and resumes the persisted thread.

### 9.3 Conformance suite

Both adapters must pass one transport-independent suite:

- fresh threads contain only projected instructions;
- schema-constrained responses decode identically;
- completion and token events are lossless;
- interruption reaches a terminal state;
- approval requests are rejected according to policy;
- malformed output maps to equivalent normalized errors;
- shutdown leaves no active workload owned by the test;
- recovery can read or resume a persisted thread; and
- child result bodies do not automatically enter the root transcript.

## 10. Budgets

The host enforces:

- maximum graph nodes;
- maximum concurrent children;
- root token budget;
- per-child token budget;
- aggregate token budget;
- wall-time budget;
- writable storage budget;
- result-envelope size;
- maximum continuation turns; and
- evaluator budget.

Before launch, the scheduler reserves each child's maximum allocation. It reconciles reservations against App Server token-usage notifications.

An active turn is interrupted when a hard per-task or aggregate limit is crossed. Unused capacity is not silently duplicated across children.

Every attempt, including duplicate or indeterminate work, counts toward observed cost.

## 11. Resumable durable execution

This section is normative. An implementation that executes the graph successfully but cannot satisfy these restart and reconciliation requirements does not complete the milestone.

### 11.1 Adaptation from Temporal

`harp` borrows the smallest useful subset of Temporal's durable-execution model:

- deterministic orchestration is separate from side-effecting activities;
- durable intent is committed before an external effect;
- activities are at-least-once;
- activity output publication is idempotent;
- external identities are persisted for reconciliation;
- process restart reconstructs execution from durable state; and
- retry policy distinguishes transient infrastructure failures from semantic outcomes.

`harp` does not initially implement Temporal's complete Event History replay model. Materialized SQLite state is authoritative. An append-only event table exists for diagnosis and audit, not as the recovery source of truth.

Relevant Temporal concepts are documented in:

- <https://docs.temporal.io/workflows>
- <https://docs.temporal.io/workflow-execution>
- <https://docs.temporal.io/workflow-execution/event>
- <https://docs.temporal.io/tasks>

### 11.2 Recovery guarantee

The first milestone guarantees:

> After a `harp` process restart, the engine reconstructs the validated graph and budgets from SQLite, does not repeat durably completed tasks, reconnects to or resumes known Codex threads, reconciles known turns, and schedules only work that remains incomplete.

It additionally provides best-effort continuation within an interrupted Codex task by using:

- the task's persisted Codex thread;
- the task's private scratch directory;
- semantic checkpoints;
- accumulated evidence; and
- a bounded continuation turn.

It does not guarantee resumption at the exact model token, tool instruction, or CPU instruction where failure occurred.

### 11.3 Delivery semantics

The honest delivery model is:

- **Codex and filesystem activities:** at-least-once;
- **durable state transitions:** transactional;
- **artifact publication:** idempotent by content digest;
- **accepted task result:** exactly-once by compare-and-swap;
- **model compute:** may be duplicated in an ambiguous failure window.

No documentation or metric may describe the complete system as globally exactly-once.

### 11.4 Authoritative durable tables

SQLite contains at least:

```text
runs
task_graphs
tasks
attempts
operations
budget_reservations
artifacts
events
```

Required identities:

- `run_id`;
- `task_id`;
- `attempt`;
- `operation_id`;
- `operation_kind`;
- `thread_id`, when known;
- `turn_id`, when known; and
- accepted result artifact digest, when complete.

Every external operation has a stable idempotency key:

```text
(run_id, task_id, attempt, operation_kind)
```

### 11.5 Task and attempt states

Task state:

```text
Pending
Ready
Running
ResultPublished
Completed
Failed
Cancelled
```

Attempt state:

```text
Prepared
DispatchingThread
ThreadStarted
DispatchingTurn
TurnStarted
Reconciling
ResultPublished
Succeeded
Failed
Indeterminate
Cancelled
```

State transitions use transactions and compare-and-swap preconditions. A stale worker cannot overwrite a newer attempt or accepted result.

### 11.6 Persist-before-effect protocol

Every Codex activity follows this sequence:

1. claim the task with a lease;
2. reserve its budget;
3. create an operation row in `Prepared`;
4. commit;
5. change the operation and attempt to the relevant `Dispatching*` state;
6. commit;
7. perform the external App Server call;
8. persist the returned external ID immediately;
9. reconcile notifications and thread state;
10. publish result artifacts idempotently; and
11. atomically accept the result and complete the task.

The explicit `Dispatching*` state makes the ambiguity window visible. A crash in that state is never silently treated as though the external call did not occur.

### 11.7 Restart entry point

The CLI exposes:

```text
harp run <benchmark>
harp resume <run-id>
```

Startup also supports an explicit `--resume-incomplete` mode for unattended recovery.

`resume` performs:

1. open and integrity-check the run database;
2. verify graph and run-manifest digests;
3. load the pinned Codex and adapter provenance;
4. expire dead leases;
5. verify published artifacts;
6. reconcile every nonterminal attempt;
7. rebuild the ready queue from authoritative task states;
8. restore budget reservations and observed usage;
9. reconnect or restart the selected adapter; and
10. continue scheduling.

### 11.8 Recovery decision table

| Durable state | Required recovery action |
| --- | --- |
| `Pending` or `Ready` | Recompute dependency readiness and enqueue when eligible. |
| `Running` with an unexpired live lease | Do not steal it. Observe the active worker. |
| `Running` with an expired lease and no dispatched operation | Reclaim the task and continue the same attempt. |
| `Prepared` | The external call was not marked for dispatch; it is safe to dispatch. |
| `DispatchingThread` without `thread_id` | Mark the attempt `Indeterminate`; start a new attempt because the server may have created an unreachable thread. |
| `ThreadStarted` without a turn | Resume or read the known thread, then start the required turn. |
| `DispatchingTurn` without `turn_id` | Read the known thread and search for the unique operation marker before deciding to start another turn. |
| `TurnStarted` | Read the thread and reconcile the stored turn's status and output. |
| completed Codex turn without published result | Decode, validate, and publish the existing result. |
| interrupted or failed Codex turn with useful durable progress | Start one bounded continuation turn in the same thread. |
| `ResultPublished` | Verify the artifact, then transactionally accept it and complete the task. |
| `Completed` | Perform no model work. Rebuild dependent readiness only. |
| all required children `Completed` | Schedule the reducer exactly once if it is not already running or complete. |

### 11.9 Thread-start ambiguity

Current App Server `thread/start` does not expose a client-supplied idempotency key that `harp` can use to recover a response lost after the server created the thread.

If `harp` crashes in `DispatchingThread` before persisting `thread_id`:

1. mark the attempt `Indeterminate`;
2. preserve its budget reservation and observed cost record;
3. create a new attempt;
4. accept only the first valid result through the task's compare-and-swap completion;
5. report potential duplicate compute; and
6. clean up any discoverable orphan later.

This ambiguity must be visible in metrics and evaluation. The implementation must not pretend the first call never happened.

A future App Server idempotency key or client-supplied thread ID could close this gap without changing the engine's higher-level semantics.

### 11.10 Turn-start reconciliation

Once `thread_id` is durable, turn start is more recoverable.

Every turn input includes a unique non-secret operation marker derived from `operation_id`. Only one active turn is permitted per child thread.

If the `turn/start` response is lost:

1. call `thread/read`;
2. inspect persisted turns and their inputs for the operation marker;
3. if the turn exists, persist its `turn_id` and reconcile it;
4. if it does not exist, start the turn; and
5. if the state cannot be determined safely, mark the attempt `Indeterminate` rather than launching unbounded duplicates.

### 11.11 Mid-turn failure

There are three recovery cases.

#### Case A: `harp` failed but a reconnectable App Server and turn remain alive

The adapter reconnects, reads the stored `thread_id` and `turn_id`, and continues consuming or polling the turn. No new model turn is created.

The initial managed-stdio process adapter does not promise this case because its stream cannot be reattached after the parent dies. A future socket-managed process adapter may support it.

#### Case B: App Server or the embedded runtime also failed

`harp` starts a new App Server against the same Codex state directory, resumes the stored thread, and reads its persisted turns.

- If the target turn completed, use its existing final output.
- If it is terminal but incomplete, start a bounded continuation turn in the same thread.
- If it is still recorded as in progress but no live runtime owns it, reconcile it as interrupted before continuation.

The continuation prompt instructs the child to:

- inspect its existing thread history;
- inspect its durable scratch directory;
- continue from the latest valid checkpoint;
- avoid repeating completed analysis;
- retain existing evidence;
- satisfy the original output schema; and
- publish a final result envelope.

#### Case C: neither the thread nor its external identity can be recovered

The attempt becomes `Indeterminate` or `Failed`, depending on evidence. A new attempt starts with the same immutable task input and remaining host-approved budget.

### 11.12 Semantic checkpoint contract

Every child receives a persistent scratch directory:

```text
.harp/runs/<run-id>/tasks/<task-id>/attempts/<attempt>/
├── checkpoint.json
├── evidence.jsonl
├── notes.md
└── result.json
```

The child must checkpoint after each planned analysis unit and before final reduction.

`checkpoint.json` contains:

```json
{
  "schemaVersion": 1,
  "taskId": "analyze-app-server",
  "attempt": 1,
  "phase": "analyzing_modules",
  "completedUnits": ["app-server-protocol", "app-server-client"],
  "pendingUnits": ["app-server", "rollout-trace"],
  "evidenceCount": 14,
  "updatedAt": "2026-08-08T00:00:00Z"
}
```

`harp` provides a narrow checkpoint command that validates the schema and atomically replaces `checkpoint.json`. Children do not write the authoritative SQLite task state directly.

`evidence.jsonl` is append-only. Each record identifies the source path, content digest, bounded location, claim, and collection time. A continuation turn reuses valid evidence rather than rediscovering it.

These are semantic checkpoints. They do not serialize model hidden state.

### 11.13 Exactly-once result acceptance

Result acceptance occurs in one SQLite transaction:

1. verify the result artifact exists and matches its digest;
2. verify its schema and task identity;
3. verify evidence references;
4. reconcile actual token and storage usage;
5. compare-and-swap `tasks.accepted_attempt` from `NULL` to the attempt ID;
6. mark the accepted attempt `Succeeded`;
7. mark competing attempts terminal without accepting their results;
8. mark the task `Completed`;
9. release the lease;
10. make newly satisfied dependents `Ready`; and
11. commit.

If the process crashes before commit, recovery repeats the transaction. If it crashes after commit, the task is already complete and no model work is repeated.

### 11.14 Reducer recovery

The reducer is an ordinary durable task with dependencies on all required child tasks.

It becomes `Ready` only after every required predecessor has a durably accepted result. Scheduling uses the same compare-and-swap and lease mechanism as analysis tasks.

A crash between the final child completion and reducer scheduling is recovered by recomputing dependency readiness from SQLite. No special event delivery is required.

### 11.15 Cancellation

Cancellation is durable:

1. mark the run or task `CancellationRequested`;
2. persist the request;
3. issue `turn/interrupt` when a known turn is active;
4. reconcile terminal state;
5. mark the attempt and task `Cancelled`; and
6. retain all artifacts and cost records.

A restart observes the persisted cancellation request and does not resume semantic work.

### 11.16 Retry policy

Automatic retries are limited to explicitly transient infrastructure failures such as:

- App Server unavailable before request dispatch;
- temporary transport failure with a safely reconcilable external identity;
- bounded event-stream interruption; or
- temporary artifact-store I/O failure before publication.

The following are semantic outcomes and are not silently retried:

- invalid root graph after the repair attempt;
- output-schema violation;
- unsupported claim or missing required evidence;
- child-declared inability to complete;
- hard budget exhaustion; or
- evaluator rejection.

Continuation of an interrupted turn is not counted as an invisible retry. It is a recorded operation with its own token and wall-time usage.

### 11.17 Leases

Task leases prevent two healthy workers from intentionally executing the same attempt.

A lease contains:

- worker identity;
- acquisition time;
- expiry time;
- last renewal; and
- operation state.

Lease expiry alone does not prove a Codex activity is dead. Recovery must reconcile known `thread_id` and `turn_id` before creating a new attempt.

The first milestone does not require a separate heartbeat stream. Lease renewal and App Server events provide liveness observations. A general heartbeat subsystem is deferred until real workloads demonstrate the need.

### 11.18 Failure-injection matrix

The implementation must test restart at these boundaries:

| Failure point | Expected result after `harp resume` |
| --- | --- |
| before task claim commits | Task remains ready and runs once. |
| after task claim, before operation preparation | Expired lease is reclaimed. |
| after `Prepared`, before `DispatchingThread` | Thread start is safely dispatched. |
| after `DispatchingThread`, before `thread_id` persists | Attempt becomes `Indeterminate`; a bounded new attempt may run. |
| after `thread_id`, before turn start | Existing thread is resumed and receives one turn. |
| after `DispatchingTurn`, before `turn_id` persists | `thread/read` and the operation marker reconcile the existing turn. |
| during streamed turn | Existing completed output is used, or one continuation turn resumes from checkpoints. |
| after artifact write, before atomic publication | Temporary file is ignored or cleaned; publication retries safely. |
| after artifact publication, before SQLite metadata commit | Digest lookup recovers or republishes the same object. |
| after `ResultPublished`, before task completion | Acceptance transaction completes without a new model call. |
| after final child completion, before reducer readiness | Dependency scan marks the reducer ready exactly once. |
| during reducer execution | Reducer follows the same reconciliation and continuation rules. |
| after evaluation artifact publication, before run completion | Existing evaluation is accepted without rerunning the evaluator. |

For every test, the resumed run must preserve:

- graph digest;
- accepted task result digests;
- task and attempt lineage;
- budget accounting;
- evidence integrity; and
- adapter provenance.

### 11.19 Durable-execution acceptance criteria

The durability requirement passes only when:

- killing and restarting `harp` does not repeat completed child tasks;
- a known completed Codex turn is recovered without a new semantic turn;
- an interrupted known thread continues from durable thread and file state;
- ambiguous thread creation is marked `Indeterminate` and measured;
- only one attempt can become the accepted result;
- published artifacts remain valid across restart;
- aggregate budget accounting includes all attempts;
- reducer readiness is reconstructed after restart;
- cancellation remains effective after restart; and
- both adapters pass the same recovery suite.

## 12. Benchmark

### 12.1 Workload

Analyze `~/workspace/codex` and answer:

> How do App Server, multi-agent execution, persistence, token accounting, and workspace controls support or constrain an RLM-style recursive harness?

The report must distinguish:

- current Codex capability;
- capability present internally but not exposed through the chosen boundary;
- functionality implemented by `harp`; and
- proposed future Codex changes.

Every material claim must link to evidence in the pinned Codex snapshot.

### 12.2 Fair adapter comparison

The first comparison:

1. generates and validates one root `TaskGraph`;
2. persists it as an immutable artifact;
3. executes the same graph through the process adapter;
4. executes the same graph through the in-process adapter; and
5. compares normalized outcomes and operational metrics.

Later experiments may repeat the complete planning and execution path independently for each adapter.

## 13. Hybrid evaluation

The benchmark uses the approved hybrid evaluator. Deterministic checks enforce
the execution contract. A protected semantic rubric evaluates the usefulness
and correctness of the evidence-linked architecture report.

### 13.1 Deterministic checks

The evaluator verifies:

- every required node is terminal;
- contexts contain only projected inputs;
- envelopes satisfy their schemas;
- evidence paths are within the pinned corpus;
- evidence hashes match;
- lineage and artifact digests are valid;
- budgets and concurrency limits were respected;
- writable directories were disjoint;
- no child body polluted the root transcript;
- restart recovery passes the failure-injection matrix;
- normalized adapter results are equivalent where required;
- no DWARF or ordinary debug artifacts were generated; and
- configured build and run storage limits were respected.

### 13.2 Semantic rubric

A protected evaluator scores:

- architectural correctness;
- evidence coverage;
- correct separation of existing and proposed capability;
- identification of real integration gaps;
- implementation usefulness;
- unsupported-claim rate;
- redundancy; and
- clarity.

The evaluator prompt, score thresholds, and credentials remain outside root and child task state.

### 13.3 Adapter metrics

Each run records:

- startup latency;
- total latency;
- root and child token usage;
- peak concurrency;
- event lag;
- protocol and decoding failures;
- restart and reconciliation outcomes;
- indeterminate attempts;
- duplicated model compute;
- peak and final disk use;
- build time;
- packaged artifact size;
- operator steps; and
- normalized task reward.

## 14. Error model

The engine distinguishes:

- invalid root plan;
- adapter startup failure;
- protocol incompatibility;
- transport failure;
- child execution failure;
- output-schema violation;
- budget exhaustion;
- timeout;
- cancellation;
- artifact-integrity failure;
- indeterminate external effect;
- recovery reconciliation failure; and
- evaluator failure.

Errors preserve their causal chain. The engine does not convert a conflict, integrity failure, or ambiguous side effect into generic task failure.

## 15. Security and isolation

- Root and child models receive no raw App Server control credentials.
- The host chooses permission profiles and corpus roots.
- Analysis children read the Codex corpus but cannot modify it.
- Each child writes only to its private scratch directory.
- Artifact handles are resolved by the host against an allowlist.
- Result references are verified before acceptance.
- The evaluator is outside the mutable task boundary.
- Generated graph content cannot increase permissions, budgets, recursion depth, or writable roots.

## 16. Milestone acceptance

The Stage 1+2 slice is complete when:

- the root generates a valid `TaskGraph`;
- the engine executes it through both adapters;
- both adapters pass the conformance suite;
- both adapters use fresh projected child contexts;
- both produce schema-valid, evidence-linked reports;
- deterministic checks pass;
- semantic evaluation reaches the configured threshold;
- `harp resume` satisfies the durable-execution acceptance criteria;
- failure-injection tests cover every required boundary;
- the process adapter is usable without compiling Codex into the main workspace;
- the in-process adapter remains an explicit isolated build;
- no Codex source is vendored;
- no build emits DWARF;
- heavyweight build artifacts are cleaned after packaging; and
- comparison output records enough evidence to guide later adapter selection.

## 17. Implementation sequence

1. Create the no-DWARF Rust workspace, build profiles, disk preflight, and provenance commands.
2. Implement contracts and schema fixtures.
3. Implement content-addressed artifacts and SQLite durable state.
4. Implement the deterministic scheduler and graph validator.
5. Implement resumable task, attempt, operation, and result-acceptance transitions.
6. Implement the process adapter and conformance harness.
7. Implement root planning and depth-one child execution.
8. Implement checkpoint tooling and recovery reconciliation.
9. Implement deterministic evaluation and failure injection.
10. Implement the benchmark and semantic evaluator boundary.
11. Implement the isolated in-process adapter.
12. Run the same graph through both adapters and publish the comparison bundle.

## 18. Source grounding

The design was checked against the local Codex checkout at commit `cd934c8bcb`.

Relevant current Codex surfaces include:

- `codex-rs/app-server-protocol/src/protocol/v2/thread.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/turn.rs`
- `codex-rs/app-server-client/`
- `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs`
- `codex-rs/core/src/tools/handlers/multi_agents_common.rs`
- `codex-rs/core/src/agent/control.rs`

The implementation must re-check these surfaces against the actual Codex commit used for each build. This design does not freeze or vendor that source revision.
