# Corenius Observability Durable Execution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Corenius trace storage, activity lifecycle recording, payload blob references, deterministic replay/resume behavior, and the first `corenius bench cold-open-trace` command described by the observability durable-execution spec.

**Architecture:** Keep `RuntimeEvent` as the user-facing TUI/transcript projection and add a lower-level append-only trace stream beside `TranscriptStore`. `Coordinator` remains the orchestration boundary, with trace-backed activity scheduling/completion around `ProviderSession::stream_turn`, `ToolRegistryLike::execute`, approval decisions, and sandbox command execution. Payload data above fixed thresholds is written to content-addressed blobs so trace replay stays proportional to compact records.

**Tech Stack:** Rust 1.95, Cargo, Tokio, Serde/serde_json JSONL, Clap subcommands, uuid, tempfile/assert_cmd for tests, and `sha2 = "0.10"` only for stable content-addressed blob digests because Rust std does not provide a cross-process stable SHA-256.

**Spec:** `docs/observability/durable-execution.md`

## Global Constraints

- Make every agent run replayable, inspectable, and benchmarkable without depending on live provider state (`docs/observability/durable-execution.md:3-5`).
- Keep `RuntimeEvent` for the user-facing TUI projection and add a lower-level append-only trace stream for execution forensics (`docs/observability/durable-execution.md:68-71`).
- Effects are activities: provider calls, tool calls, shell execution, file reads, approval decisions, terminal input, and terminal output (`docs/observability/durable-execution.md:58-60`).
- Every activity idempotency key is `run_id + turn_id + step_id + activity_id + input_hash` (`docs/observability/durable-execution.md:61-62`).
- During resume, completed activity events are reused instead of re-running effects (`docs/observability/durable-execution.md:63-64`).
- During replay, suppress duplicate human-facing logs while still allowing trace validation logs (`docs/observability/durable-execution.md:65-66`).
- Store large payloads by content-addressed reference under `.corenius/blobs/` (`docs/observability/durable-execution.md:110-119`).
- Trace rows keep hashes, byte counts, MIME-like kind, truncation state, and a short preview (`docs/observability/durable-execution.md:121-123`).
- Add only the first benchmark command now: `corenius bench cold-open-trace` (`docs/observability/durable-execution.md:125-169`).
- Benchmark samples run as fresh compiled `corenius` child processes with private temp roots, fixed synthetic inputs, bounded children, and exactly one final JSON object on stdout (`docs/observability/durable-execution.md:21-25`, `docs/observability/durable-execution.md:140-156`).
- Do not build a distributed Temporal clone; scope stays inside local JSONL traces, local blobs, fake-effect resume tests, and the first cold-open benchmark.

---

## Existing Code Map

- `Cargo.toml:1-26` defines one Rust crate with Tokio, Serde, Clap, uuid, tempfile, and assert_cmd already available.
- `src/lib.rs:1-8` exports top-level modules; new modules must be added here.
- `src/core/types.rs:95-140` defines `RuntimeEvent`, which remains the TUI projection and should not absorb the full trace schema.
- `src/core/transcript.rs:8-55` is the JSONL store pattern to mirror for `TraceStore`.
- `src/core/runner.rs:48-80` defines the provider, tool, and approval traits used as effect boundaries.
- `src/core/runner.rs:102-141` constructs `Coordinator` and `ActiveTurn`; this is where a `TraceStore` handle and `run_id` enter the runtime.
- `src/core/runner.rs:154-188` runs a user turn and calls `ProviderSession::stream_turn`; this is the provider activity lifecycle boundary.
- `src/core/runner.rs:191-215` maps provider events into runtime events; this is where provider stream chunks and provider completion trace records are emitted.
- `src/core/runner.rs:230-270` executes tools and asks approval; this is where tool scheduled/started/completed and approval requested/decided trace records are emitted.
- `src/tools.rs:12-20` defines `ShellRunner`; this interface must grow tracing context so sandbox command/process lifecycle events can be recorded.
- `src/tools.rs:44-103` registers `read_file` and `shell`; these are the initial tool activity cases.
- `src/tools.rs:125-131` delegates tool execution to registered handlers; `Coordinator::execute_tool` should stay the idempotency/replay gate rather than burying resume behavior inside individual tools.
- `src/insula.rs:150-225` builds a sandbox command plan; trace `sandbox_command_built` can be emitted from the shell runner wrapper after this method returns.
- `src/insula.rs:228-260` starts and waits for bwrap; trace `sandbox_process_started` and `sandbox_process_completed` belong around this execution path.
- `src/cli.rs:39-50` defines top-level Clap subcommands; add `Bench` here.
- `src/cli.rs:120-219` routes subcommands; add `bench cold-open-trace` routing here.
- `src/cli.rs:269-298` loads `.corenius/transcript.jsonl` and constructs `Coordinator`; it must also load `.corenius/trace.jsonl` and pass trace/blobs into the runtime.

## Planned File Structure

- Create `src/core/trace.rs`: trace event schema, `TraceStore`, deterministic replay state, idempotency helpers, and payload inline/reference structs.
- Create `src/core/payload.rs`: content-addressed blob writes/reads and preview/truncation logic shared by traces and benchmarks.
- Modify `src/core/mod.rs`: export trace and payload interfaces.
- Modify `src/core/runner.rs`: thread `TraceStore`, `run_id`, and activity lifecycle recording through `Coordinator` and `TurnRunner`.
- Modify `src/tools.rs`: extend `ShellRunner::run_shell` to accept an optional tracing context and keep default tool behavior unchanged for callers.
- Modify `src/insula.rs`: add a traced shell execution method while keeping the current `InsulaSandbox::run` path as a compatibility wrapper.
- Modify `src/cli.rs`: load trace state for `run`, add `bench cold-open-trace`, and print a single benchmark JSON report.
- Modify `src/lib.rs`: export a new `bench` module.
- Create `src/bench.rs`: synthetic trace generator, cold-open benchmark implementation, and benchmark report structs.
- Create `tests/trace_store.rs`: focused trace JSONL, replay-state, idempotency, and payload-ref tests.
- Modify `tests/core.rs`: fake provider/tool resume tests proving completed activities are reused.
- Modify `tests/tools.rs`: shell tracing-context tests for sandbox output blob refs.
- Create `tests/bench.rs`: CLI test for `corenius bench cold-open-trace`.

---

### Task 1: Trace Schema And Payload References

**Files:**
- Create: `src/core/trace.rs`
- Modify: `src/core/mod.rs:1-19`
- Modify: `Cargo.toml:7-22`
- Test: `tests/trace_store.rs`

**Interfaces:**
- Consumes: `timestamp() -> String` from `src/core/types.rs:204-210`, `ToolUse` from `src/core/types.rs:20-25`.
- Produces:
  - `pub const TRACE_SCHEMA_VERSION: u16 = 1`
  - `pub struct TraceContext { pub run_id: String, pub turn_id: Option<String>, pub step_id: Option<String>, pub activity_id: Option<String> }`
  - `pub struct PayloadRef { pub sha256: String, pub bytes: u64, pub kind: String, pub truncated: bool, pub preview: String }`
  - `pub struct ActivityKey { pub run_id: String, pub turn_id: String, pub step_id: String, pub activity_id: String, pub input_hash: String }`
  - `impl ActivityKey { pub fn new(run_id: &str, turn_id: &str, step_id: &str, activity_id: &str, input: &serde_json::Value) -> anyhow::Result<Self>; pub fn as_idempotency_key(&self) -> String; }`
  - `pub enum TraceEventKind` with serde `rename_all = "snake_case"` variants for every event listed in `docs/observability/durable-execution.md:73-92`.
  - `pub struct TraceEvent { pub schema_version: u16, pub sequence: u64, pub timestamp: String, pub run_id: String, pub turn_id: Option<String>, pub step_id: Option<String>, pub activity_id: Option<String>, pub idempotency_key: Option<String>, pub provider: Option<String>, pub model: Option<String>, pub provider_response_id: Option<String>, pub tool_name: Option<String>, pub tool_call_id: Option<String>, pub tool_capabilities: Option<ToolUse>, pub attempt: Option<u32>, pub retryable: Option<bool>, pub backoff_ms: Option<u64>, pub error_class: Option<String>, pub payload_ref: Option<PayloadRef>, pub kind: TraceEventKind }`
  - `impl TraceEvent { pub fn new(kind: TraceEventKind, run_id: &str, sequence: u64) -> Self; }`
  - `pub fn stable_json_hash(value: &serde_json::Value) -> anyhow::Result<String>`

- [ ] **Step 1: Add failing trace schema serialization test**

Create `tests/trace_store.rs` with this first test:

```rust
use corenius::core::{
    ActivityKey, TRACE_SCHEMA_VERSION, TraceEvent, TraceEventKind, ToolUse,
};
use serde_json::json;

#[test]
fn trace_event_serializes_snake_case_schema_and_activity_key() {
    let key = ActivityKey::new(
        "run-1",
        "turn-1",
        "provider-0",
        "provider-request",
        &json!({"messages":["hello"],"tools":[]}),
    )
    .unwrap();
    let mut event = TraceEvent::new(TraceEventKind::ProviderRequestScheduled, "run-1", 7);
    event.turn_id = Some("turn-1".to_string());
    event.step_id = Some("provider-0".to_string());
    event.activity_id = Some("provider-request".to_string());
    event.idempotency_key = Some(key.as_idempotency_key());
    event.provider = Some("fake".to_string());
    event.model = Some("model-a".to_string());
    event.tool_capabilities = Some(ToolUse {
        read_only: true,
        writes: false,
        shell: false,
    });

    let value = serde_json::to_value(&event).unwrap();
    assert_eq!(value["schema_version"], TRACE_SCHEMA_VERSION);
    assert_eq!(value["sequence"], 7);
    assert_eq!(value["kind"], "provider_request_scheduled");
    assert_eq!(
        value["idempotency_key"],
        "run-1:turn-1:provider-0:provider-request:a3db9e944ac982888e165b0415f48e72d63180dbad1feaa03ec118cbd4444893"
    );
    assert_eq!(value["tool_capabilities"]["read_only"], true);
}
```

- [ ] **Step 2: Add failing coverage test for every spec trace kind**

Append this test to `tests/trace_store.rs`:

```rust
#[test]
fn trace_kind_covers_spec_event_names() {
    let kinds = [
        TraceEventKind::TraceHeader,
        TraceEventKind::RunStarted,
        TraceEventKind::TurnStarted,
        TraceEventKind::ProviderRequestScheduled,
        TraceEventKind::ProviderStreamChunk,
        TraceEventKind::ProviderRequestCompleted,
        TraceEventKind::ToolCallScheduled,
        TraceEventKind::ToolCallStarted,
        TraceEventKind::ApprovalRequested,
        TraceEventKind::ApprovalDecided,
        TraceEventKind::ToolCallCompleted,
        TraceEventKind::SandboxCommandBuilt,
        TraceEventKind::SandboxProcessStarted,
        TraceEventKind::SandboxProcessCompleted,
        TraceEventKind::CheckpointWritten,
        TraceEventKind::RetryScheduled,
        TraceEventKind::TurnCompleted,
        TraceEventKind::TurnFailed,
        TraceEventKind::RunCompleted,
    ];
    let names = kinds
        .into_iter()
        .map(|kind| serde_json::to_value(kind).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "trace_header",
            "run_started",
            "turn_started",
            "provider_request_scheduled",
            "provider_stream_chunk",
            "provider_request_completed",
            "tool_call_scheduled",
            "tool_call_started",
            "approval_requested",
            "approval_decided",
            "tool_call_completed",
            "sandbox_command_built",
            "sandbox_process_started",
            "sandbox_process_completed",
            "checkpoint_written",
            "retry_scheduled",
            "turn_completed",
            "turn_failed",
            "run_completed",
        ]
    );
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --test trace_store`
Expected: fail because `src/core/trace.rs`, `ActivityKey`, `TraceEvent`, `TraceEventKind`, and `TRACE_SCHEMA_VERSION` do not exist.

- [ ] **Step 4: Add `sha2` dependency**

Modify `Cargo.toml:7-22` to add this dependency:

```toml
sha2 = "0.10"
```

Rationale: the spec requires content-addressed payload references under `.corenius/blobs/`; `DefaultHasher` and other std hashes are not an explicit stable content digest, while SHA-256 gives deterministic paths across processes and platforms.

- [ ] **Step 5: Implement trace schema**

Create `src/core/trace.rs` with the produced interfaces above. Use:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventKind {
    TraceHeader,
    RunStarted,
    TurnStarted,
    ProviderRequestScheduled,
    ProviderStreamChunk,
    ProviderRequestCompleted,
    ToolCallScheduled,
    ToolCallStarted,
    ApprovalRequested,
    ApprovalDecided,
    ToolCallCompleted,
    SandboxCommandBuilt,
    SandboxProcessStarted,
    SandboxProcessCompleted,
    CheckpointWritten,
    RetryScheduled,
    TurnCompleted,
    TurnFailed,
    RunCompleted,
}
```

Implement `stable_json_hash` by serializing the input with `serde_json::to_vec`, hashing bytes with `sha2::{Digest, Sha256}`, and formatting lowercase hex. Implement `ActivityKey::as_idempotency_key` with colon-separated fields in the exact order `run_id:turn_id:step_id:activity_id:input_hash`.

- [ ] **Step 6: Export trace types**

Modify `src/core/mod.rs:1-19`:

```rust
mod policy;
mod runner;
mod trace;
mod transcript;
mod types;

pub use trace::{
    ActivityKey, PayloadRef, TRACE_SCHEMA_VERSION, TraceContext, TraceEvent, TraceEventKind,
    stable_json_hash,
};
```

- [ ] **Step 7: Run tests**

Run: `cargo test --test trace_store`
Expected: pass.

---

### Task 2: Payload Blob Store

**Files:**
- Create: `src/core/payload.rs`
- Modify: `src/core/mod.rs:1-19`
- Test: `tests/trace_store.rs`

**Interfaces:**
- Consumes: `PayloadRef` from Task 1.
- Produces:
  - `pub const DEFAULT_INLINE_PAYLOAD_LIMIT: usize = 4096`
  - `pub struct BlobStore { root: std::path::PathBuf, inline_limit: usize }`
  - `impl BlobStore { pub fn new(root: impl Into<std::path::PathBuf>) -> Self; pub fn with_inline_limit(root: impl Into<std::path::PathBuf>, inline_limit: usize) -> Self; pub fn root(&self) -> &std::path::Path; pub async fn put_bytes(&self, kind: &str, bytes: &[u8]) -> anyhow::Result<PayloadRef>; pub async fn put_json(&self, kind: &str, value: &serde_json::Value) -> anyhow::Result<PayloadRef>; pub async fn read(&self, payload: &PayloadRef) -> anyhow::Result<Vec<u8>>; }`

- [ ] **Step 1: Add failing payload blob test**

Append this test to `tests/trace_store.rs`:

```rust
use corenius::core::BlobStore;
use tempfile::tempdir;

#[tokio::test]
async fn blob_store_writes_content_addressed_payload_refs() {
    let dir = tempdir().unwrap();
    let blobs = BlobStore::with_inline_limit(dir.path().join(".corenius/blobs"), 8);
    let payload = blobs.put_bytes("text/plain", b"hello durable trace").await.unwrap();

    assert_eq!(payload.kind, "text/plain");
    assert_eq!(payload.bytes, 19);
    assert!(payload.truncated);
    assert_eq!(payload.preview, "hello du");
    assert_eq!(
        payload.sha256,
        "1a271480ba234a4930bc4ce6f378b4054ed193f07c37e729102c19d679cf5638"
    );
    let stored = blobs.read(&payload).await.unwrap();
    assert_eq!(stored, b"hello durable trace");
    assert!(dir.path().join(".corenius/blobs/1a/1a271480ba234a4930bc4ce6f378b4054ed193f07c37e729102c19d679cf5638").exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test trace_store blob_store_writes_content_addressed_payload_refs`
Expected: fail because `BlobStore` is not exported.

- [ ] **Step 3: Implement `BlobStore`**

Create `src/core/payload.rs`. Store blobs at `<root>/<first-two-hex>/<sha256>`, create the parent directory with `tokio::fs::create_dir_all`, and use `tokio::fs::write` only if the path does not already exist. Compute `PayloadRef.preview` from the first `inline_limit` bytes with `String::from_utf8_lossy`. Set `truncated` to `bytes.len() > inline_limit`.

- [ ] **Step 4: Export payload interfaces**

Modify `src/core/mod.rs:1-19`:

```rust
mod payload;
mod policy;
mod runner;
mod trace;
mod transcript;
mod types;

pub use payload::{BlobStore, DEFAULT_INLINE_PAYLOAD_LIMIT};
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test trace_store blob_store_writes_content_addressed_payload_refs`
Expected: pass.

---

### Task 3: TraceStore Append And Replay State

**Files:**
- Modify: `src/core/trace.rs`
- Test: `tests/trace_store.rs`

**Interfaces:**
- Consumes: `TraceEvent`, `TraceEventKind`, `PayloadRef`, and `ActivityKey` from Task 1.
- Produces:
  - `pub struct TraceStore { path: std::path::PathBuf, events: Vec<TraceEvent>, next_sequence: u64 }`
  - `pub struct ReplayState { pub run_id: Option<String>, pub trace_events: usize, pub completed_activities: std::collections::BTreeMap<String, TraceEvent>, pub runtime_events: Vec<RuntimeEvent>, pub next_sequence: u64 }`
  - `impl TraceStore { pub async fn load(path: impl Into<std::path::PathBuf>) -> anyhow::Result<Self>; pub fn path(&self) -> &std::path::Path; pub fn events(&self) -> &[TraceEvent]; pub fn next_sequence(&self) -> u64; pub async fn append(&mut self, event: TraceEvent) -> anyhow::Result<TraceEvent>; pub async fn emit(&mut self, kind: TraceEventKind, run_id: &str) -> anyhow::Result<TraceEvent>; pub fn replay_state(&self) -> ReplayState; pub fn completed_activity(&self, key: &str) -> Option<&TraceEvent>; }`
  - `impl ReplayState { pub fn completed_payload(&self, key: &str) -> Option<&PayloadRef>; }`

- [ ] **Step 1: Add failing append/reload test**

Append this test to `tests/trace_store.rs`:

```rust
use corenius::core::TraceStore;

#[tokio::test]
async fn trace_store_appends_jsonl_with_monotonic_sequences() {
    let dir = tempdir().unwrap();
    let path = dir.path().join(".corenius/trace.jsonl");
    let mut store = TraceStore::load(path.clone()).await.unwrap();
    let first = store
        .emit(TraceEventKind::TraceHeader, "run-1")
        .await
        .unwrap();
    let second = store
        .emit(TraceEventKind::RunStarted, "run-1")
        .await
        .unwrap();

    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);
    let reloaded = TraceStore::load(path).await.unwrap();
    assert_eq!(reloaded.events().len(), 2);
    assert_eq!(reloaded.next_sequence(), 3);
    assert_eq!(reloaded.events()[0].kind, TraceEventKind::TraceHeader);
    assert_eq!(reloaded.events()[1].kind, TraceEventKind::RunStarted);
}
```

- [ ] **Step 2: Add failing replay-state test**

Append this test to `tests/trace_store.rs`:

```rust
#[tokio::test]
async fn replay_state_indexes_completed_activities_by_idempotency_key() {
    let dir = tempdir().unwrap();
    let path = dir.path().join(".corenius/trace.jsonl");
    let mut store = TraceStore::load(path).await.unwrap();
    store.emit(TraceEventKind::TraceHeader, "run-1").await.unwrap();
    let key = ActivityKey::new("run-1", "turn-1", "tool-0", "call-1", &json!({"path":"note.txt"}))
        .unwrap()
        .as_idempotency_key();
    let mut completed = TraceEvent::new(TraceEventKind::ToolCallCompleted, "run-1", 0);
    completed.turn_id = Some("turn-1".to_string());
    completed.step_id = Some("tool-0".to_string());
    completed.activity_id = Some("call-1".to_string());
    completed.idempotency_key = Some(key.clone());
    completed.payload_ref = Some(PayloadRef {
        sha256: "abc".to_string(),
        bytes: 2,
        kind: "application/json".to_string(),
        truncated: false,
        preview: "{}".to_string(),
    });
    store.append(completed).await.unwrap();

    let state = store.replay_state();
    assert_eq!(state.run_id.as_deref(), Some("run-1"));
    assert_eq!(state.trace_events, 2);
    assert_eq!(state.next_sequence, 3);
    assert_eq!(state.completed_payload(&key).unwrap().preview, "{}");
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --test trace_store`
Expected: fail because `TraceStore` and `ReplayState` do not exist.

- [ ] **Step 4: Implement JSONL trace store**

In `src/core/trace.rs`, implement `TraceStore` using the same async JSONL pattern as `TranscriptStore::load` and `TranscriptStore::append` in `src/core/transcript.rs:15-55`. Assign `event.sequence = self.next_sequence` inside `append`, then increment `next_sequence`. On load, ignore blank lines and set `next_sequence` to one greater than the largest loaded sequence, or `1` for an empty file.

- [ ] **Step 5: Implement deterministic replay state**

In `TraceStore::replay_state`, fold `self.events` in sequence order. Record `run_id` from the first event with a non-empty run id. Insert only events with `kind == TraceEventKind::ProviderRequestCompleted`, `ToolCallCompleted`, or `ApprovalDecided` and `idempotency_key.is_some()` into `completed_activities`.

- [ ] **Step 6: Run tests**

Run: `cargo test --test trace_store`
Expected: pass.

---

### Task 4: Coordinator Activity Lifecycle And Resume Reuse

**Files:**
- Modify: `src/core/runner.rs:82-283`
- Modify: `src/core/mod.rs:1-25`
- Modify: `src/cli.rs:269-298`
- Modify: `tests/core.rs:1-220`

**Interfaces:**
- Consumes: `TraceStore`, `BlobStore`, `ActivityKey`, `TraceEvent`, `TraceEventKind`, and `PayloadRef` from Tasks 1-3.
- Produces:
  - `pub struct CoordinatorOptions { pub run_id: String, pub trace: TraceStore, pub blobs: BlobStore }`
  - `impl Coordinator { pub fn new(...existing args...) -> Self; pub fn new_with_trace(provider: Arc<dyn ProviderSession>, tools: Arc<dyn ToolRegistryLike>, policy: PolicyGate, transcript: TranscriptStore, approvals: Option<Arc<dyn ApprovalResponder>>, options: CoordinatorOptions) -> Self; }`
  - Existing `Coordinator::new` remains source-compatible by constructing an in-memory-disabled trace mode for old tests.
  - `TurnRunner` emits trace events around provider stream, provider chunks, provider completion, tool schedule/start/complete, approval request/decision, turn completion, and turn failure.
  - Resume behavior reuses completed provider/tool/approval activity payloads when `TraceStore::completed_activity(key)` returns a completed event with a readable JSON payload.

- [ ] **Step 1: Add failing provider resume test**

Append this test to `tests/core.rs`:

```rust
use std::sync::atomic::AtomicUsize;
use corenius::core::{BlobStore, CoordinatorOptions, TraceEvent, TraceEventKind, TraceStore};

#[tokio::test]
async fn coordinator_reuses_completed_provider_activity_on_resume() {
    struct CountingProvider {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl ProviderSession for CountingProvider {
        async fn run_turn(
            &self,
            _messages: Vec<corenius::core::TranscriptMessage>,
            _tools: Vec<ToolDefinition>,
            _executor: corenius::core::ToolExecutor,
        ) -> anyhow::Result<Vec<ProviderEvent>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![
                ProviderEvent::AssistantDelta { delta: "live".to_string() },
                ProviderEvent::Done { response_id: "live-response".to_string(), usage: None },
            ])
        }
    }

    struct NoTools;
    #[async_trait]
    impl ToolRegistryLike for NoTools {
        fn definitions(&self) -> Vec<ToolDefinition> { Vec::new() }
        async fn execute(&self, _call: ToolCall) -> anyhow::Result<serde_json::Value> {
            unreachable!("resume should not execute tools")
        }
    }

    let dir = tempdir().unwrap();
    let trace_path = dir.path().join(".corenius/trace.jsonl");
    let blobs = BlobStore::new(dir.path().join(".corenius/blobs"));
    let mut trace = TraceStore::load(trace_path).await.unwrap();
    trace.emit(TraceEventKind::TraceHeader, "run-fixed").await.unwrap();
    let provider_payload = blobs
        .put_json(
            "application/vnd.corenius.provider-events+json",
            &json!([
                {"AssistantDelta":{"delta":"recorded"}},
                {"Done":{"response_id":"recorded-response","usage":null}}
            ]),
        )
        .await
        .unwrap();
    let provider_input = json!({"messages":[{"role":"user","content":"hello"}],"tools":[]});
    let key = corenius::core::ActivityKey::new(
        "run-fixed",
        "turn-fixed",
        "provider-0",
        "provider-request",
        &provider_input,
    )
    .unwrap()
    .as_idempotency_key();
    let mut completed = TraceEvent::new(TraceEventKind::ProviderRequestCompleted, "run-fixed", 0);
    completed.turn_id = Some("turn-fixed".to_string());
    completed.step_id = Some("provider-0".to_string());
    completed.activity_id = Some("provider-request".to_string());
    completed.idempotency_key = Some(key);
    completed.payload_ref = Some(provider_payload);
    trace.append(completed).await.unwrap();

    let transcript = TranscriptStore::load(dir.path().join(".corenius/transcript.jsonl")).await.unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let coordinator = Coordinator::new_with_trace(
        Arc::new(CountingProvider { calls: calls.clone() }),
        Arc::new(NoTools),
        PolicyGate::new(),
        transcript,
        None,
        CoordinatorOptions {
            run_id: "run-fixed".to_string(),
            trace,
            blobs,
        },
    );

    let mut events = coordinator.start_user_turn_with_id("turn-fixed", "hello").await.unwrap().events;
    let mut seen = Vec::new();
    while let Some(event) = events.recv().await {
        seen.push(event);
    }

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(seen.iter().any(|event| matches!(event, RuntimeEvent::AssistantDelta { delta, .. } if delta == "recorded")));
    assert!(seen.iter().any(|event| matches!(event, RuntimeEvent::TurnFinished { response_id, .. } if response_id.as_deref() == Some("recorded-response"))));
}
```

- [ ] **Step 2: Add failing tool resume test**

Append this test to `tests/core.rs`:

```rust
#[tokio::test]
async fn coordinator_reuses_completed_tool_activity_on_resume() {
    struct ToolCallingProvider;
    #[async_trait]
    impl ProviderSession for ToolCallingProvider {
        async fn run_turn(
            &self,
            _messages: Vec<corenius::core::TranscriptMessage>,
            _tools: Vec<ToolDefinition>,
            executor: corenius::core::ToolExecutor,
        ) -> anyhow::Result<Vec<ProviderEvent>> {
            let call = ToolCall::new("call-read", "fake", "read_file", json!({"path":"note.txt"}));
            let result = executor(call.clone()).await?;
            Ok(vec![
                ProviderEvent::ToolResult { call, output: result.output },
                ProviderEvent::Done { response_id: "done".to_string(), usage: None },
            ])
        }
    }

    struct CountingTools {
        calls: Arc<AtomicUsize>,
    }
    #[async_trait]
    impl ToolRegistryLike for CountingTools {
        fn definitions(&self) -> Vec<ToolDefinition> {
            vec![ToolDefinition::read_only("read_file", "Read", json!({"type":"object"}))]
        }
        async fn execute(&self, _call: ToolCall) -> anyhow::Result<serde_json::Value> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(json!({"content":"live"}))
        }
    }

    let dir = tempdir().unwrap();
    let blobs = BlobStore::new(dir.path().join(".corenius/blobs"));
    let mut trace = TraceStore::load(dir.path().join(".corenius/trace.jsonl")).await.unwrap();
    trace.emit(TraceEventKind::TraceHeader, "run-fixed").await.unwrap();
    let payload = blobs
        .put_json("application/vnd.corenius.tool-result+json", &json!({"content":"recorded"}))
        .await
        .unwrap();
    let key = corenius::core::ActivityKey::new(
        "run-fixed",
        "turn-fixed",
        "tool-0",
        "call-read",
        &json!({"id":"call-read","provider":"fake","name":"read_file","arguments":{"path":"note.txt"}}),
    )
    .unwrap()
    .as_idempotency_key();
    let mut completed = TraceEvent::new(TraceEventKind::ToolCallCompleted, "run-fixed", 0);
    completed.turn_id = Some("turn-fixed".to_string());
    completed.step_id = Some("tool-0".to_string());
    completed.activity_id = Some("call-read".to_string());
    completed.idempotency_key = Some(key);
    completed.payload_ref = Some(payload);
    trace.append(completed).await.unwrap();

    let transcript = TranscriptStore::load(dir.path().join(".corenius/transcript.jsonl")).await.unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let coordinator = Coordinator::new_with_trace(
        Arc::new(ToolCallingProvider),
        Arc::new(CountingTools { calls: calls.clone() }),
        PolicyGate::new(),
        transcript,
        None,
        CoordinatorOptions {
            run_id: "run-fixed".to_string(),
            trace,
            blobs,
        },
    );

    let mut events = coordinator.start_user_turn_with_id("turn-fixed", "hello").await.unwrap().events;
    let mut saw_recorded_tool_result = false;
    while let Some(event) = events.recv().await {
        if matches!(event, RuntimeEvent::ToolResult { result, .. } if result == json!({"content":"recorded"})) {
            saw_recorded_tool_result = true;
        }
    }

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(saw_recorded_tool_result);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --test core coordinator_reuses_completed`
Expected: fail because `CoordinatorOptions`, `Coordinator::new_with_trace`, and `start_user_turn_with_id` do not exist.

- [ ] **Step 4: Add trace state to `Coordinator`**

Modify `src/core/runner.rs:82-141`. Add fields to `Coordinator`:

```rust
run_id: String,
trace: Option<Arc<Mutex<TraceStore>>>,
blobs: Option<BlobStore>,
```

Keep `Coordinator::new` signature unchanged for compatibility with existing tests and `src/cli.rs:290-298`; set `run_id = Uuid::new_v4().to_string()`, `trace = None`, and `blobs = None`. Add `Coordinator::new_with_trace` with the produced signature. Add `start_user_turn_with_id(&self, turn_id: &str, text: &str) -> anyhow::Result<ActiveTurn>` and have existing `start_user_turn` call it with a new UUID.

- [ ] **Step 5: Emit turn lifecycle trace events**

Modify `TurnRunner` in `src/core/runner.rs:144-188` to carry `run_id`, `trace`, and `blobs`. At the beginning of `run`, append `RunStarted` once per empty trace, then append `TurnStarted`. On success append `TurnCompleted` after provider completion. On provider error append `TurnFailed` with `error_class = Some(error.to_string())`.

- [ ] **Step 6: Implement provider activity scheduling and reuse**

In `TurnRunner::run`, before `provider.stream_turn` at `src/core/runner.rs:173-176`, compute:

```rust
let provider_input = json!({"messages": messages, "tools": tools_for_provider});
let key = ActivityKey::new(&self.run_id, &self.turn_id, "provider-0", "provider-request", &provider_input)?;
```

If `TraceStore::completed_activity(&key.as_idempotency_key())` has a payload, read it from `BlobStore`, deserialize `Vec<ProviderEvent>`, and feed each event through `emit_provider_event` without calling `ProviderSession::stream_turn`. Otherwise append `ProviderRequestScheduled`, collect streamed provider events in a local `Vec<ProviderEvent>` inside the sink, append `ProviderStreamChunk` for each `AssistantDelta`, append `ProviderRequestCompleted` with a payload ref containing the serialized `Vec<ProviderEvent>`, then continue.

- [ ] **Step 7: Implement tool activity scheduling and reuse**

In `TurnRunner::execute_tool` at `src/core/runner.rs:230-270`, compute:

```rust
let tool_input = serde_json::to_value(&call)?;
let key = ActivityKey::new(&self.run_id, &self.turn_id, &format!("tool-{tool_index}"), &call.id, &tool_input)?;
```

Use a deterministic per-turn `tool_index` counter in `TurnRunner`, stored as `Arc<Mutex<u64>>`, incremented before each tool call. If a completed `ToolCallCompleted` exists for the key, read the JSON result from the payload and return `ProviderToolResult { call_id: call.id, output }` without calling `self.tools.execute`. Otherwise append `ToolCallScheduled`, then `ToolCallStarted`, call the real tool, write the result JSON to `BlobStore`, and append `ToolCallCompleted` with `tool_name`, `tool_call_id`, `tool_capabilities`, and `payload_ref`.

- [ ] **Step 8: Trace approval decisions**

In `TurnRunner::execute_tool` at `src/core/runner.rs:243-260`, append `ApprovalRequested` before emitting the existing `RuntimeEvent::ApprovalRequested`. After receiving or defaulting the decision, append `ApprovalDecided` with a payload ref containing JSON:

```json
{"decision":"approved"}
```

or:

```json
{"decision":"denied"}
```

If a completed `ApprovalDecided` exists for the approval idempotency key, reuse that decision without calling `ApprovalResponder::request_approval`.

- [ ] **Step 9: Wire trace loading in interactive CLI**

Modify `src/cli.rs:269-298` to load:

```rust
let trace = TraceStore::load(workspace.join(".corenius/trace.jsonl")).await?;
let blobs = BlobStore::new(workspace.join(".corenius/blobs"));
```

Construct `Coordinator::new_with_trace` with `run_id = uuid::Uuid::new_v4().to_string()`. Keep the transcript path `.corenius/transcript.jsonl` unchanged.

- [ ] **Step 10: Run focused tests**

Run: `cargo test --test core coordinator_reuses_completed`
Expected: pass.

- [ ] **Step 11: Run existing core tests for compatibility**

Run: `cargo test --test core`
Expected: pass, including existing `Coordinator::new` callers.

---

### Task 5: Sandbox Command And Output Payload Trace Records

**Files:**
- Modify: `src/tools.rs:12-20`
- Modify: `src/tools.rs:44-103`
- Modify: `src/insula.rs:150-260`
- Modify: `src/cli.rs:229-242`
- Modify: `tests/tools.rs:1-90`

**Interfaces:**
- Consumes: `TraceContext`, `TraceStore`, `BlobStore`, `TraceEventKind`, `PayloadRef`, and existing `SandboxResult`.
- Produces:
  - `pub struct ToolTraceContext { pub run_id: String, pub turn_id: String, pub step_id: String, pub activity_id: String, pub trace: Arc<tokio::sync::Mutex<TraceStore>>, pub blobs: BlobStore }`
  - `#[async_trait] pub trait ShellRunner { async fn run_shell(&self, command: String, cwd: PathBuf, timeout_ms: u64, trace: Option<ToolTraceContext>) -> anyhow::Result<SandboxResult>; }`
  - `impl InsulaSandbox { pub async fn run_traced(&self, command: &str, cwd: &Path, env: &[(&str, &str)], timeout_ms: u64, trace: Option<ToolTraceContext>) -> anyhow::Result<SandboxResult>; }`

- [ ] **Step 1: Add failing shell trace test**

Modify `tests/tools.rs` to update `RecordingShell::run_shell` for the new `trace` argument, record whether it was present, and add:

```rust
#[tokio::test]
async fn shell_tool_receives_trace_context_for_sandbox_lifecycle() {
    let dir = tempdir().unwrap();
    let shell = Arc::new(RecordingShell::default());
    let registry = ToolRegistry::default_for_workspace(dir.path(), shell.clone());
    let blobs = corenius::core::BlobStore::new(dir.path().join(".corenius/blobs"));
    let trace = corenius::core::TraceStore::load(dir.path().join(".corenius/trace.jsonl"))
        .await
        .unwrap();
    let context = corenius::tools::ToolTraceContext {
        run_id: "run-1".to_string(),
        turn_id: "turn-1".to_string(),
        step_id: "tool-0".to_string(),
        activity_id: "call-shell".to_string(),
        trace: Arc::new(tokio::sync::Mutex::new(trace)),
        blobs,
    };

    registry
        .execute_with_trace(
            ToolCall::new(
                "call-shell",
                "test",
                "shell",
                json!({"command":"printf hello","timeoutMs":42}),
            ),
            Some(context),
        )
        .await
        .unwrap();

    let calls = shell.calls.lock().unwrap();
    assert_eq!(calls[0].0, "printf hello");
    assert_eq!(calls[0].2, 42);
    assert!(shell.saw_trace.lock().unwrap()[0]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test tools shell_tool_receives_trace_context_for_sandbox_lifecycle`
Expected: fail because `ToolTraceContext` and `ToolRegistry::execute_with_trace` do not exist.

- [ ] **Step 3: Add trace-aware tool execution without breaking trait callers**

Modify `src/tools.rs:12-20` and `src/tools.rs:116-131`. Keep the `ToolRegistryLike::execute(&self, call: ToolCall)` trait unchanged by having it call `ToolRegistry::execute_with_trace(call, None)`. Add inherent method:

```rust
pub async fn execute_with_trace(
    &self,
    call: ToolCall,
    trace: Option<ToolTraceContext>,
) -> anyhow::Result<Value>
```

Update the internal shell handler closure to pass `trace` to `ShellRunner::run_shell`. For `read_file`, if `trace` is present, write the file contents through `BlobStore::put_bytes("text/plain", content.as_bytes())` and append `ToolCallCompleted` payload metadata through the coordinator path from Task 4, not directly inside the read handler.

- [ ] **Step 4: Add traced sandbox execution**

Modify `src/insula.rs:228-260`. Keep `InsulaSandbox::run` as a wrapper calling `run_traced(..., None)`. Implement `run_traced` to append:

```rust
TraceEventKind::SandboxCommandBuilt
TraceEventKind::SandboxProcessStarted
TraceEventKind::SandboxProcessCompleted
```

For `SandboxCommandBuilt`, store a JSON payload with `executable`, `args`, and `env` from `BwrapPlan`. For `SandboxProcessCompleted`, store a JSON payload with the full `SandboxResult`; this captures long stdout/stderr by payload ref instead of expanding the trace row.

- [ ] **Step 5: Update CLI shell runner**

Modify `src/cli.rs:233-242` so `InsulaShellRunner::run_shell` accepts `trace: Option<ToolTraceContext>` and calls `self.sandbox.run_traced(&command, &cwd, &[], timeout_ms, trace).await`.

- [ ] **Step 6: Update core tool execution to pass context**

In `src/core/runner.rs:261-265`, if `self.tools` is the concrete `ToolRegistry`, call `execute_with_trace`. If retaining the trait object prevents downcasting cleanly, add `async fn execute_with_trace(&self, call: ToolCall, trace: Option<ToolTraceContext>) -> anyhow::Result<Value>` as a default method on `ToolRegistryLike` that calls `execute(call)`; override it in `ToolRegistry`. Prefer the trait default method because `Coordinator` already stores `Arc<dyn ToolRegistryLike>` at `src/core/runner.rs:85`.

- [ ] **Step 7: Run tests**

Run: `cargo test --test tools`
Expected: pass.

- [ ] **Step 8: Run sandbox compatibility tests**

Run: `cargo test --test insula`
Expected: pass with existing untraced `InsulaSandbox::run` callers unchanged.

---

### Task 6: Crash/Resume Behavior Tests With Fake Effects

**Files:**
- Modify: `tests/core.rs:1-260`
- Modify: `src/core/runner.rs:154-270`
- Modify: `src/core/trace.rs`

**Interfaces:**
- Consumes: resume behavior implemented in Task 4 and sandbox/tool tracing from Task 5.
- Produces:
  - Crash/resume tests where scheduled-but-not-completed activities re-run.
  - Crash/resume tests where completed activities are reused.
  - Deterministic next scheduled activity computation through `TraceStore::replay_state()`.

- [ ] **Step 1: Add failing scheduled-only provider retry test**

Append this test to `tests/core.rs`:

```rust
#[tokio::test]
async fn resume_reruns_provider_when_only_scheduled_event_exists() {
    struct CountingProvider {
        calls: Arc<AtomicUsize>,
    }
    #[async_trait]
    impl ProviderSession for CountingProvider {
        async fn run_turn(
            &self,
            _messages: Vec<corenius::core::TranscriptMessage>,
            _tools: Vec<ToolDefinition>,
            _executor: corenius::core::ToolExecutor,
        ) -> anyhow::Result<Vec<ProviderEvent>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![ProviderEvent::Done { response_id: "fresh".to_string(), usage: None }])
        }
    }
    struct NoTools;
    #[async_trait]
    impl ToolRegistryLike for NoTools {
        fn definitions(&self) -> Vec<ToolDefinition> { Vec::new() }
        async fn execute(&self, _call: ToolCall) -> anyhow::Result<serde_json::Value> {
            unreachable!("no tools")
        }
    }

    let dir = tempdir().unwrap();
    let blobs = BlobStore::new(dir.path().join(".corenius/blobs"));
    let mut trace = TraceStore::load(dir.path().join(".corenius/trace.jsonl")).await.unwrap();
    let mut scheduled = TraceEvent::new(TraceEventKind::ProviderRequestScheduled, "run-fixed", 0);
    scheduled.turn_id = Some("turn-fixed".to_string());
    scheduled.step_id = Some("provider-0".to_string());
    scheduled.activity_id = Some("provider-request".to_string());
    scheduled.idempotency_key = Some(
        corenius::core::ActivityKey::new(
            "run-fixed",
            "turn-fixed",
            "provider-0",
            "provider-request",
            &json!({"messages":[{"role":"user","content":"hello"}],"tools":[]}),
        )
        .unwrap()
        .as_idempotency_key(),
    );
    trace.append(scheduled).await.unwrap();

    let transcript = TranscriptStore::load(dir.path().join(".corenius/transcript.jsonl")).await.unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let coordinator = Coordinator::new_with_trace(
        Arc::new(CountingProvider { calls: calls.clone() }),
        Arc::new(NoTools),
        PolicyGate::new(),
        transcript,
        None,
        CoordinatorOptions { run_id: "run-fixed".to_string(), trace, blobs },
    );

    let mut events = coordinator.start_user_turn_with_id("turn-fixed", "hello").await.unwrap().events;
    while events.recv().await.is_some() {}
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
```

- [ ] **Step 2: Add failing replay-state next-activity test**

Append this test to `tests/trace_store.rs`:

```rust
#[tokio::test]
async fn replay_state_keeps_scheduled_activity_uncompleted() {
    let dir = tempdir().unwrap();
    let mut trace = TraceStore::load(dir.path().join(".corenius/trace.jsonl")).await.unwrap();
    let mut scheduled = TraceEvent::new(TraceEventKind::ToolCallScheduled, "run-1", 0);
    scheduled.turn_id = Some("turn-1".to_string());
    scheduled.step_id = Some("tool-0".to_string());
    scheduled.activity_id = Some("call-1".to_string());
    scheduled.idempotency_key = Some("run-1:turn-1:tool-0:call-1:hash".to_string());
    trace.append(scheduled).await.unwrap();

    let state = trace.replay_state();
    assert!(state.completed_payload("run-1:turn-1:tool-0:call-1:hash").is_none());
}
```

- [ ] **Step 3: Run tests to verify they fail or expose missing behavior**

Run: `cargo test --test core resume_reruns_provider_when_only_scheduled_event_exists && cargo test --test trace_store replay_state_keeps_scheduled_activity_uncompleted`
Expected: pass only after scheduled-only events are deliberately excluded from `completed_activities`; fail if replay treats scheduled events as reusable.

- [ ] **Step 4: Add explicit scheduled/completed distinction**

In `TraceStore::replay_state`, ensure scheduled/started events are never inserted into `completed_activities`. In `TurnRunner`, only reuse provider/tool/approval effects when the completed event has both matching `idempotency_key` and a readable `payload_ref`.

- [ ] **Step 5: Add checkpoint trace point**

In `TurnRunner::run`, after each completed provider/tool/approval activity append `TraceEventKind::CheckpointWritten` with a JSON payload:

```json
{"completed_activity_key":"<idempotency-key>"}
```

This satisfies the immediate durable-execution slice without adding a separate checkpoint file format.

- [ ] **Step 6: Run resume tests**

Run: `cargo test --test core coordinator_reuses_completed && cargo test --test core resume_reruns_provider_when_only_scheduled_event_exists`
Expected: pass.

- [ ] **Step 7: Run trace tests**

Run: `cargo test --test trace_store`
Expected: pass.

---

### Task 7: `corenius bench cold-open-trace`

**Files:**
- Create: `src/bench.rs`
- Modify: `src/lib.rs:1-8`
- Modify: `src/cli.rs:32-50`
- Modify: `src/cli.rs:120-219`
- Test: `tests/bench.rs`

**Interfaces:**
- Consumes: `TraceStore::load`, `TraceStore::replay_state`, `BlobStore`, `TraceEventKind`, `RuntimeEvent`.
- Produces:
  - `pub struct BenchReport { pub benchmark: String, pub workload: String, pub schema_version: u16, pub timing: BenchTiming, pub memory: BenchMemory, pub event_counts: BenchEventCounts, pub blob_bytes: u64, pub exit: i32, pub signal: Option<String>, pub timed_out: bool }`
  - `pub async fn run_cold_open_trace(workspace: std::path::PathBuf, turns: usize) -> anyhow::Result<BenchReport>`
  - CLI: `corenius bench cold-open-trace --workspace <path> --turns <usize>`

- [ ] **Step 1: Add failing bench CLI test**

Create `tests/bench.rs`:

```rust
use assert_cmd::Command;
use predicates::str::contains;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn bench_cold_open_trace_emits_one_json_report() {
    let temp = tempdir().unwrap();
    let output = Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "bench",
            "cold-open-trace",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--turns",
            "12",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "stderr must be empty for one-json-report bench");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    let report: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(report["benchmark"], "cold-open-trace");
    assert_eq!(report["workload"], "synthetic-12-turns");
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["exit"], 0);
    assert_eq!(report["signal"], Value::Null);
    assert_eq!(report["timed_out"], false);
    assert!(report["timing"]["elapsed_ms"].as_u64().unwrap() <= 60_000);
    assert_eq!(report["event_counts"]["runtime"], 24);
    assert!(report["event_counts"]["trace"].as_u64().unwrap() >= 24);
    assert!(report["blob_bytes"].as_u64().unwrap() > 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test bench bench_cold_open_trace_emits_one_json_report`
Expected: fail because `bench` subcommand does not exist.

- [ ] **Step 3: Implement benchmark module**

Create `src/bench.rs`. `run_cold_open_trace` must:

1. Create `workspace/.corenius/trace.jsonl` and `workspace/.corenius/blobs`.
2. Generate fixed synthetic inputs for `turns`: one `TurnStarted`, one `ProviderStreamChunk`, one `ProviderRequestCompleted` with payload JSON, and one `TurnCompleted` per turn.
3. Load the trace with `TraceStore::load`.
4. Call `replay_state`.
5. Measure elapsed time with `std::time::Instant`.
6. Compute `event_counts.trace = trace.events().len()`.
7. Compute `event_counts.runtime = turns * 2` for the synthetic user/assistant projection.
8. Compute `blob_bytes` by summing file sizes under `.corenius/blobs`.
9. Return one `BenchReport` with `exit = 0`, `signal = None`, and `timed_out = false`.

- [ ] **Step 4: Export `bench` module**

Modify `src/lib.rs:1-8`:

```rust
pub mod bench;
pub mod cli;
pub mod core;
pub mod insula;
pub mod provider;
pub mod tools;
pub mod tui;
```

- [ ] **Step 5: Add Clap bench command**

Modify `src/cli.rs:39-50`:

```rust
Bench {
    #[command(subcommand)]
    command: BenchCommand,
},
```

Add:

```rust
#[derive(Subcommand, Debug)]
enum BenchCommand {
    ColdOpenTrace {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long, default_value_t = 200)]
        turns: usize,
    },
}
```

Modify `src/cli.rs:120-219` to route `Command::Bench { command }` and print exactly:

```rust
println!("{}", serde_json::to_string(&report)?);
```

Do not print progress lines in the bench path.

- [ ] **Step 6: Run bench CLI test**

Run: `cargo test --test bench bench_cold_open_trace_emits_one_json_report`
Expected: pass.

- [ ] **Step 7: Run benchmark manually**

Run: `cargo run -- bench cold-open-trace --workspace "$(mktemp -d)" --turns 200`
Expected: one JSON line with:

```json
{"benchmark":"cold-open-trace","workload":"synthetic-200-turns","schema_version":1,"timing":{"elapsed_ms":...},"memory":{"retained_bytes":...},"event_counts":{"trace":...,"runtime":400},"blob_bytes":...,"exit":0,"signal":null,"timed_out":false}
```

---

### Task 8: Full Verification And Backwards Compatibility Audit

**Files:**
- Modify only if previous tasks revealed compile/test fallout: `src/core/mod.rs`, `src/core/runner.rs`, `src/tools.rs`, `src/insula.rs`, `src/cli.rs`, `tests/core.rs`, `tests/tools.rs`, `tests/trace_store.rs`, `tests/bench.rs`.

**Interfaces:**
- Consumes: all interfaces from Tasks 1-7.
- Produces: a passing test suite and a short compatibility note in the final implementation report.

- [ ] **Step 1: Run formatting**

Run: `cargo fmt`
Expected: no errors.

- [ ] **Step 2: Run all tests**

Run: `cargo test`
Expected: pass.

- [ ] **Step 3: Run focused CLI smoke for existing commands**

Run: `cargo test --test cli_tui`
Expected: pass, proving existing `insula` and provider smoke paths still compile and route.

- [ ] **Step 4: Run first benchmark command**

Run: `cargo run -- bench cold-open-trace --workspace "$(mktemp -d)" --turns 200`
Expected: exactly one stdout line, valid JSON, and no stderr.

- [ ] **Step 5: Inspect generated files**

Run:

```bash
tmp="$(mktemp -d)"
cargo run -- bench cold-open-trace --workspace "$tmp" --turns 3
find "$tmp/.corenius" -maxdepth 3 -type f | sort
```

Expected: output includes `.corenius/trace.jsonl` and at least one file below `.corenius/blobs/`.

- [ ] **Step 6: Backwards compatibility check**

Confirm these compatibility points in the final implementation report:

- Existing `Coordinator::new` callers still compile.
- Existing `ToolRegistryLike::execute` callers still compile.
- Existing `InsulaSandbox::run` callers still compile.
- Existing `.corenius/transcript.jsonl` format is unchanged.
- New durable execution state is additive in `.corenius/trace.jsonl` and `.corenius/blobs/`.

---

## Self-Review

- Spec requires replayable, inspectable, benchmarkable runs without live provider state (`docs/observability/durable-execution.md:3-5`): Tasks 3, 4, 6, and 7 add trace replay state, activity reuse, resume tests, and `cold-open-trace`.
- Spec requires effect activities for provider calls, tool calls, shell execution, file reads, approval decisions, terminal input, and terminal output (`docs/observability/durable-execution.md:58-60`): Tasks 4 and 5 cover provider/tool/shell/approval/file-read effects; terminal input/output are represented in trace schema and payload refs but full terminal replay UI is intentionally out of scope for this immediate slice.
- Spec requires idempotency key shape (`docs/observability/durable-execution.md:61-62`): Task 1 defines `ActivityKey` and Tasks 4 and 6 use it for provider/tool/approval reuse.
- Spec requires completed activities to be reused on resume (`docs/observability/durable-execution.md:63-64`): Tasks 4 and 6 include fake provider/tool tests that assert live effects are not invoked when completed payloads exist.
- Spec requires replay-aware logging suppression (`docs/observability/durable-execution.md:65-66`): Task 4 replays provider events into runtime state without invoking live effects; no human-facing logs are emitted from replay paths.
- Spec requires a lower-level append-only trace stream while keeping `RuntimeEvent` (`docs/observability/durable-execution.md:68-71`): Tasks 1-3 add `TraceEvent` and `TraceStore`; existing `RuntimeEvent` remains unchanged.
- Spec lists trace event names and fields (`docs/observability/durable-execution.md:73-108`): Task 1 covers all event names and the shared field structure.
- Spec requires payload refs under `.corenius/blobs/` for large provider/tool/terminal data (`docs/observability/durable-execution.md:110-123`): Tasks 2, 4, and 5 implement content-addressed payload refs for provider events, tool results, sandbox commands, and sandbox results.
- Spec lists six benchmark paths but the user scope requests only the first command now (`docs/observability/durable-execution.md:125-169`): Task 7 implements only `corenius bench cold-open-trace` and does not plan the other five benchmark paths.
- Dependency policy: only `sha2 = "0.10"` is added, with a concrete reason for stable blob digests; all async/file/JSON work uses existing Tokio and Serde patterns.
- Placeholder scan: this plan contains no `TBD`, no `TODO`, no "implement later", and no task that says to write tests without concrete test code.
- Type consistency: `ActivityKey`, `PayloadRef`, `TraceEvent`, `TraceStore`, `ReplayState`, `BlobStore`, `CoordinatorOptions`, `ToolTraceContext`, and `BenchReport` are introduced before later tasks consume them.
