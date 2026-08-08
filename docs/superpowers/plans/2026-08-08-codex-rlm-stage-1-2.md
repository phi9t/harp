# Codex-RLM stage 1+2 implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a low-disk Rust vertical slice that asks a root Codex thread for a validated depth-one `TaskGraph`, executes fresh child threads through process and in-process App Server adapters, persists artifacts and task state, resumes after failure, and evaluates the result with deterministic checks plus a protected semantic rubric.

**Architecture:** Keep the existing standalone `harp` binary as the CLI and split new execution code into focused workspace crates. `harp-engine` owns the deterministic graph state machine and SQLite transactions; runtime adapters only translate between App Server and normalized runtime events. Routine tests use fake runtimes and a fake stdio App Server. Real Codex builds and the in-process adapter remain explicit acceptance tasks because the machine has very little free disk space.

**Tech Stack:** Rust 2021, Tokio, serde/serde_json, schemars, rusqlite with bundled SQLite, sha2, tempfile, clap, async-trait, futures, App Server v2 JSON-RPC, `mise`, macOS `dwarfdump`, and the sibling Codex checkout at `~/workspace/codex`.

---

## Repository prerequisite

The approved spec is commit `a0b98d3` on branch `harp-standalone-import`. This
plan is committed on top of that spec. The current standalone corpus/Atlas tree
is present in `~/workspace/harp` but is still untracked. A
separate RLM evidence worktree exists at
`~/.agents/worktrees/harp/rlm-deep-dive` and has its own
uncommitted tracker change. Do not blanket-add either tree.

Task 1 commits the standalone source and content as one focused import commit on
top of the approved spec and plan. It then creates the implementation worktree
from that combined tracked baseline. Leave the separate `feat/rlm-deep-dive`
worktree untouched.

Every commit in this plan must end with:

```text
Co-authored-by: TRAE CLI <noreply@bytedance.com>
```

## Planned file map

### Existing files to modify

- `Cargo.toml`: add workspace members, shared dependencies, and no-DWARF profiles.
- `Cargo.lock`: update only through Cargo commands using the compact target directory.
- `crates/harp/Cargo.toml`: depend on the new execution crates.
- `crates/harp/src/lib.rs`: export RLM entry points without mixing them into corpus compilation.
- `crates/harp/src/main.rs`: add `rlm run`, `rlm resume`, `rlm checkpoint`, and `rlm compare`.
- `crates/harp/tests/cli.rs`: add CLI contract and resume tests.
- `.gitignore`: ignore `.harp/`, compact target directories, packaged binaries, and in-process build state.

### New main-workspace crates

- `crates/harp-contracts/`: pure serializable IDs, graph, budget, runtime, result, checkpoint, and evaluation types.
- `crates/harp-artifacts/`: content-addressed object publication and verified reads.
- `crates/harp-state/`: SQLite schema, durable transitions, leases, budgets, events, and compare-and-swap result acceptance.
- `crates/harp-runtime/`: `CodexRuntime` trait, normalized errors/events, and a deterministic fake runtime.
- `crates/harp-app-server-process/`: stdio JSON-RPC client and process-backed `CodexRuntime`.
- `crates/harp-engine/`: graph validation, scheduler, recovery, continuation, and cancellation.
- `crates/harp-eval/`: deterministic checks, semantic rubric contract, and comparison metrics.

### New benchmark and experiment files

- `benchmarks/codex-architecture/benchmark.toml`: workload, budgets, schemas, and thresholds.
- `benchmarks/codex-architecture/prompts/root.md`: root planner prompt.
- `benchmarks/codex-architecture/prompts/child.md`: projected child prompt.
- `benchmarks/codex-architecture/prompts/continuation.md`: interrupted-task continuation prompt.
- `benchmarks/codex-architecture/prompts/reducer.md`: evidence-linked report reducer prompt.
- `benchmarks/codex-architecture/schemas/task-graph.json`: checked-in graph schema receipt.
- `benchmarks/codex-architecture/schemas/result-envelope.json`: checked-in result schema receipt.
- `benchmarks/codex-architecture/rubric.json`: protected semantic rubric.
- `experiments/codex-inprocess/Cargo.toml`: independent Cargo workspace with sibling Codex path dependencies.
- `experiments/codex-inprocess/src/main.rs`: in-process runtime runner using `codex-app-server-client`.
- `experiments/codex-inprocess/tests/conformance.rs`: shared adapter contract tests.
- `scripts/check_disk.sh`: fail before heavyweight builds when free space is below threshold.
- `scripts/check_no_dwarf.sh`: reject debug sections and `.dSYM` output.
- `scripts/package_inprocess.sh`: build, copy the stripped runner and provenance manifest, then clean intermediates.
- `mise.toml`: compact test/build, process acceptance, in-process build, no-DWARF, and cleanup tasks.

## Commands and disk policy

Use these variables throughout implementation:

```bash
export HARP_TARGET_DIR="$PWD/.build/harp-target"
export HARP_INPROCESS_TARGET_DIR="$PWD/.build/codex-inprocess-target"
export CARGO_INCREMENTAL=0
```

Routine Rust commands:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small --workspace
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo clippy --profile test-small --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo fmt --all --check
```

Do not run:

```bash
cargo build
cargo test
cargo test --all-features
```

without the compact profile and explicit target directory. Do not run a real Codex-linked build until `scripts/check_disk.sh` reports enough free space.

---

### Task 1: Establish the tracked baseline and compact Rust build gate

**Files:**
- Create: `.gitignore`
- Create: `.cargo/config.toml`
- Create: `mise.toml`
- Create: `scripts/check_disk.sh`
- Create: `scripts/check_no_dwarf.sh`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Test: `crates/harp/tests/build_policy.rs`

- [ ] **Step 1: Add baseline ignore rules and track the standalone import**

Stage only the standalone source and content tree. The spec and plan are already
tracked. Exclude `target/`, `__pycache__/`, `.harp/`, and `.build/`.

Create `.gitignore`:

```gitignore
/target/
/.build/
/.harp/
/.sources/
/.worktrees/
**/__pycache__/
**/*.pyc
**/*.dSYM/
```

```bash
git add .gitattributes .gitignore Cargo.toml Cargo.lock atlas content crates evidence labs
git status --short
```

Expected: only source, content, and lockfiles are staged. `target/` is not
staged. `.sources/` is not staged because it contains independent nested source
checkouts. The staged diff does not modify the approved spec or plan.

- [ ] **Step 2: Commit the standalone baseline**

```bash
git commit -m "chore: import standalone Harp baseline" \
  -m "Track the standalone Rust corpus compiler, Atlas, content, and evidence without generated Rust build artifacts." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Expected: one import commit whose tree contains `Cargo.toml` and
`crates/harp/src/main.rs`, with the approved spec and plan already in its
history.

- [ ] **Step 3: Create the implementation worktree**

```bash
git worktree add .worktrees/codex-rlm-stage-1-2 -b feat/codex-rlm-stage-1-2 HEAD
```

Expected: the feature branch contains the standalone baseline, the approved
spec, and this implementation plan.

Run Steps 4-10 with the working directory set to:

```text
~/workspace/harp/.worktrees/codex-rlm-stage-1-2
```

- [ ] **Step 4: Write the failing build-policy test**

Create `crates/harp/tests/build_policy.rs`:

```rust
use std::fs;
use std::path::Path;

#[test]
fn every_cargo_profile_disables_debug_and_incremental_output() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("root Cargo.toml");

    for profile in ["dev", "test", "test-small", "release", "size"] {
        let header = format!("[profile.{profile}]");
        let section = manifest
            .split(&header)
            .nth(1)
            .unwrap_or_else(|| panic!("missing {header}"))
            .split("\n[")
            .next()
            .expect("profile section");
        assert!(section.contains("debug = 0"), "{header} enables debug info");
        assert!(
            section.contains("incremental = false"),
            "{header} enables incremental artifacts"
        );
    }

    assert!(manifest.contains("split-debuginfo = \"off\""));
    assert!(manifest.contains("strip = \"symbols\""));
}
```

- [ ] **Step 5: Run the build-policy test and verify it fails**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp --test build_policy
```

Expected: FAIL because `test-small`, `size`, `split-debuginfo`, or `strip` is missing.

- [ ] **Step 6: Add compact profiles and ignored paths**

Add to `Cargo.toml`:

```toml
[profile.dev]
debug = 0
split-debuginfo = "off"
incremental = false
strip = "symbols"

[profile.test]
debug = 0
split-debuginfo = "off"
incremental = false
strip = "symbols"

[profile.test-small]
inherits = "test"
opt-level = 1
debug = 0
split-debuginfo = "off"
incremental = false
strip = "symbols"

[profile.release]
debug = 0
split-debuginfo = "off"
incremental = false
strip = "symbols"

[profile.size]
inherits = "release"
opt-level = "z"
lto = "thin"
codegen-units = 1
panic = "abort"
debug = 0
split-debuginfo = "off"
incremental = false
strip = "symbols"
```

Create `.cargo/config.toml`:

```toml
[build]
incremental = false
```

- [ ] **Step 7: Add disk and no-DWARF checks**

Create `scripts/check_disk.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

root="${1:-.}"
minimum_gib="${HARP_MIN_FREE_GIB:-8}"
available_kib=$(df -Pk "$root" | awk 'NR == 2 {print $4}')
required_kib=$((minimum_gib * 1024 * 1024))

if (( available_kib < required_kib )); then
  printf 'need at least %s GiB free; only %.2f GiB available\n' \
    "$minimum_gib" "$(awk "BEGIN {print $available_kib / 1024 / 1024}")" >&2
  exit 1
fi
```

Create `scripts/check_no_dwarf.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

binary="$1"
test -f "$binary"

if find "$(dirname "$binary")" -maxdepth 1 -name '*.dSYM' -print -quit | grep -q .; then
  echo "unexpected dSYM beside $binary" >&2
  exit 1
fi

if dwarfdump --uuid "$binary" 2>/dev/null | grep -q 'UUID:'; then
  if otool -l "$binary" | grep -q '__DWARF'; then
    echo "unexpected __DWARF segment in $binary" >&2
    exit 1
  fi
fi
```

Make both executable.

- [ ] **Step 8: Add `mise` tasks**

Create `mise.toml`:

```toml
[env]
HARP_TARGET_DIR = "{{config_root}}/.build/harp-target"
HARP_INPROCESS_TARGET_DIR = "{{config_root}}/.build/codex-inprocess-target"
CARGO_INCREMENTAL = "0"

[tasks.test]
run = 'CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small --workspace'

[tasks.fmt]
run = 'CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo fmt --all --check'

[tasks.clippy]
run = 'CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo clippy --profile test-small --workspace --all-targets -- -D warnings'

[tasks.build]
run = 'CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo build --profile size -p harp'

[tasks.no-dwarf]
depends = ["build"]
run = 'scripts/check_no_dwarf.sh "$HARP_TARGET_DIR/size/harp"'

[tasks.clean-build]
run = 'rm -rf "$HARP_TARGET_DIR" "$HARP_INPROCESS_TARGET_DIR"'
```

- [ ] **Step 9: Run the compact build gate**

Run:

```bash
mise run test
mise run build
mise run no-dwarf
du -sh "$HARP_TARGET_DIR"
```

Expected: tests pass, the size-profile binary contains no `__DWARF` segment, and one target directory is reported.

- [ ] **Step 10: Commit**

```bash
git add .cargo/config.toml mise.toml scripts/check_disk.sh scripts/check_no_dwarf.sh Cargo.toml Cargo.lock crates/harp/tests/build_policy.rs
git commit -m "build: enforce compact no-DWARF Rust artifacts" \
  -m "Use explicit compact profiles, one target directory, disk preflight, and executable debug-section checks." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 2: Define stable contracts and checked-in JSON schemas

**Files:**
- Create: `crates/harp-contracts/Cargo.toml`
- Create: `crates/harp-contracts/src/lib.rs`
- Create: `crates/harp-contracts/src/id.rs`
- Create: `crates/harp-contracts/src/artifact.rs`
- Create: `crates/harp-contracts/src/graph.rs`
- Create: `crates/harp-contracts/src/runtime.rs`
- Create: `crates/harp-contracts/src/result.rs`
- Create: `crates/harp-contracts/src/checkpoint.rs`
- Create: `crates/harp-contracts/src/evaluation.rs`
- Create: `crates/harp-contracts/tests/schema.rs`
- Create: `benchmarks/codex-architecture/schemas/task-graph.json`
- Create: `benchmarks/codex-architecture/schemas/result-envelope.json`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Add the contracts crate and dependencies**

Add workspace dependencies:

```toml
async-trait = "0.1"
futures = "0.3"
schemars = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["macros", "process", "rt-multi-thread", "sync", "time"] }
toml = "0.9"
uuid = { version = "1", features = ["serde", "v7"] }
```

Add `crates/harp-contracts` to `workspace.members`.

- [ ] **Step 2: Write the failing schema and round-trip tests**

Create `crates/harp-contracts/tests/schema.rs`:

```rust
use harp_contracts::{
    ArtifactRef, Budget, NodeKind, ResultEnvelope, ResultStatus, TaskGraph, TaskNode,
};
use schemars::schema_for;

#[test]
fn task_graph_schema_rejects_unknown_fields_and_round_trips() {
    let graph = TaskGraph {
        schema_version: 1,
        nodes: vec![TaskNode {
            task_id: "analyze-app-server".parse().expect("task id"),
            kind: NodeKind::Analysis,
            instruction: "Inspect App Server persistence.".to_string(),
            dependencies: Vec::new(),
            inputs: Vec::new(),
            budget: Budget::new(20_000, 300, 16 * 1024 * 1024),
            output_schema: "result-envelope/v1".to_string(),
        }],
    };

    let encoded = serde_json::to_value(&graph).expect("serialize graph");
    assert_eq!(serde_json::from_value::<TaskGraph>(encoded).unwrap(), graph);

    let schema = serde_json::to_value(schema_for!(TaskGraph)).expect("schema");
    assert_eq!(
        schema.pointer("/additionalProperties"),
        Some(&serde_json::json!(false))
    );
}

#[test]
fn result_envelope_keeps_large_answers_out_of_the_inline_summary() {
    let envelope = ResultEnvelope {
        schema_version: 1,
        task_id: "analyze-app-server".parse().unwrap(),
        status: ResultStatus::Success,
        answer_ref: Some(ArtifactRef::sha256(
            "a".repeat(64),
            "application/json",
            128,
        )),
        evidence: Vec::new(),
        trace_ref: ArtifactRef::sha256("b".repeat(64), "application/jsonl", 64),
        summary: "Bounded result.".to_string(),
        token_usage: 42,
        failure_class: None,
    };

    assert!(envelope.validate().is_ok());
}
```

- [ ] **Step 3: Run the tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-contracts
```

Expected: FAIL because the crate and types do not exist.

- [ ] **Step 4: Implement exact contract types**

Use newtypes for `RunId`, `TaskId`, `AttemptId`, `OperationId`, `ThreadId`, and `TurnId`. Reject empty IDs and path separators in task IDs.

Define:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskGraph {
    pub schema_version: u8,
    pub nodes: Vec<TaskNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskNode {
    pub task_id: TaskId,
    pub kind: NodeKind,
    pub instruction: String,
    pub dependencies: Vec<TaskId>,
    pub inputs: Vec<ArtifactRef>,
    pub budget: Budget,
    pub output_schema: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NodeKind {
    Analysis,
    Reducer,
}
```

Define `RuntimeEvent` variants for `ThreadStarted`, `TurnStarted`, `TokenUsage`, `TurnCompleted`, `ServerRequest`, `Lagged`, and `Disconnected`.

Define `Checkpoint` with `completed_units`, `pending_units`, `evidence_count`, and `updated_at_unix_seconds`.

- [ ] **Step 5: Generate and check in schema receipts**

Add a `schema` test helper that serializes `schema_for!(TaskGraph)` and `schema_for!(ResultEnvelope)` with a trailing newline. Compare the bytes to:

```text
benchmarks/codex-architecture/schemas/task-graph.json
benchmarks/codex-architecture/schemas/result-envelope.json
```

On the first run, write the exact generated files with:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-contracts schema -- --nocapture
```

Do not hand-edit generated schemas.

- [ ] **Step 6: Run tests**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-contracts
```

Expected: all contract and schema tests pass.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/harp-contracts benchmarks/codex-architecture/schemas
git commit -m "feat: define Codex-RLM execution contracts" \
  -m "Add typed graph, artifact, runtime, result, checkpoint, budget, and evaluation contracts with checked-in schemas." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 3: Implement content-addressed artifacts and atomic checkpoints

**Files:**
- Create: `crates/harp-artifacts/Cargo.toml`
- Create: `crates/harp-artifacts/src/lib.rs`
- Create: `crates/harp-artifacts/src/store.rs`
- Create: `crates/harp-artifacts/src/checkpoint.rs`
- Create: `crates/harp-artifacts/tests/store.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Write failing publication and checkpoint tests**

Create `crates/harp-artifacts/tests/store.rs`:

```rust
use harp_artifacts::ArtifactStore;
use harp_contracts::{AttemptId, Checkpoint, RunId, TaskId};
use tempfile::TempDir;

#[test]
fn publishing_the_same_bytes_is_idempotent_and_verified() {
    let root = TempDir::new().unwrap();
    let store = ArtifactStore::open(root.path()).unwrap();

    let first = store.publish(b"{\"answer\":42}", "application/json").unwrap();
    let second = store.publish(b"{\"answer\":42}", "application/json").unwrap();

    assert_eq!(first, second);
    assert_eq!(store.read_verified(&first).unwrap(), b"{\"answer\":42}");
}

#[test]
fn checkpoint_replacement_is_atomic_and_schema_checked() {
    let root = TempDir::new().unwrap();
    let store = ArtifactStore::open(root.path()).unwrap();
    let checkpoint = Checkpoint::new(
        RunId::new(),
        "task-a".parse::<TaskId>().unwrap(),
        AttemptId::new(),
        "inspect",
    );

    store.write_checkpoint(&checkpoint).unwrap();
    assert_eq!(store.read_checkpoint(checkpoint.attempt_id()).unwrap(), checkpoint);
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-artifacts
```

Expected: FAIL because the crate does not exist.

- [ ] **Step 3: Implement immutable object publication**

`ArtifactStore::publish` must:

1. hash bytes with SHA-256;
2. create `objects/sha256/ab/abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789`, where `ab` is the digest prefix;
3. write to `NamedTempFile` in the destination directory;
4. `write_all` and `sync_all`;
5. atomically persist;
6. verify existing objects before reuse; and
7. return `ArtifactRef`.

Reject symlink ancestors using the existing `HeldDirectory` pattern. Do not expose a raw arbitrary-path reader.

- [ ] **Step 4: Implement task scratch and checkpoint paths**

Use:

```text
.harp/runs/$RUN_ID/tasks/$TASK_ID/attempts/$ATTEMPT_ID/
```

`write_checkpoint` must serialize a `Checkpoint` to a temporary file and atomically replace `checkpoint.json`. Append evidence records to `evidence.jsonl` with `sync_data`.

- [ ] **Step 5: Add corruption tests**

Add tests that mutate an object after publication and expect:

```text
ArtifactError::DigestMismatch
```

Add a symlink-ancestor test that expects publication rejection.

- [ ] **Step 6: Run tests**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-artifacts
```

Expected: all artifact tests pass.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/harp-artifacts
git commit -m "feat: add content-addressed run artifacts" \
  -m "Publish immutable objects and semantic checkpoints with verified digests and atomic replacement." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 4: Implement the authoritative SQLite state machine

**Files:**
- Create: `crates/harp-state/Cargo.toml`
- Create: `crates/harp-state/migrations/0001_init.sql`
- Create: `crates/harp-state/src/lib.rs`
- Create: `crates/harp-state/src/store.rs`
- Create: `crates/harp-state/src/model.rs`
- Create: `crates/harp-state/src/transitions.rs`
- Create: `crates/harp-state/tests/transitions.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Write failing persist-before-effect tests**

Create tests for this exact sequence:

```rust
let run = store.create_run(&graph, &provenance, &budget)?;
let claim = store.claim_ready_task(run.id(), "worker-a", now, lease_until)?;
let operation = store.prepare_operation(claim.attempt_id(), OperationKind::StartThread)?;
store.mark_dispatching_thread(operation.id())?;
store.record_thread_started(operation.id(), ThreadId::from("thr-1"))?;
store.mark_dispatching_turn(operation.id(), "op:run/task/attempt/start-turn")?;
store.record_turn_started(operation.id(), TurnId::from("turn-1"))?;
```

Assert each invalid out-of-order transition fails with `StateError::Conflict`.

- [ ] **Step 2: Write failing exactly-once acceptance test**

```rust
let first = store.accept_result(task_id, attempt_a, artifact_a, usage_a)?;
let second = store.accept_result(task_id, attempt_b, artifact_b, usage_b)?;

assert_eq!(first, AcceptResult::Accepted);
assert_eq!(
    second,
    AcceptResult::AlreadyAccepted {
        attempt_id: attempt_a
    }
);
```

- [ ] **Step 3: Run tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-state
```

Expected: FAIL because the state crate does not exist.

- [ ] **Step 4: Add the SQLite schema**

Create tables:

```sql
CREATE TABLE runs (
    run_id TEXT PRIMARY KEY,
    graph_sha256 TEXT NOT NULL,
    graph_json BLOB NOT NULL,
    provenance_json BLOB NOT NULL,
    state TEXT NOT NULL,
    cancellation_requested INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE tasks (
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    accepted_attempt_id TEXT,
    PRIMARY KEY (run_id, task_id)
);

CREATE TABLE attempts (
    attempt_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    state TEXT NOT NULL,
    lease_owner TEXT,
    lease_expires_at INTEGER,
    thread_id TEXT,
    turn_id TEXT,
    operation_marker TEXT,
    continuation_count INTEGER NOT NULL DEFAULT 0,
    observed_tokens INTEGER NOT NULL DEFAULT 0,
    result_sha256 TEXT
);

CREATE TABLE operations (
    operation_id TEXT PRIMARY KEY,
    attempt_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    external_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (attempt_id, kind)
);

CREATE TABLE budget_reservations (
    attempt_id TEXT PRIMARY KEY,
    reserved_tokens INTEGER NOT NULL,
    reserved_storage_bytes INTEGER NOT NULL,
    observed_tokens INTEGER NOT NULL DEFAULT 0,
    observed_storage_bytes INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE artifacts (
    sha256 TEXT PRIMARY KEY,
    media_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL
);

CREATE TABLE events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    task_id TEXT,
    attempt_id TEXT,
    event_type TEXT NOT NULL,
    payload_json BLOB NOT NULL,
    recorded_at INTEGER NOT NULL
);
```

Add dependency edges in a separate `task_dependencies` table.

- [ ] **Step 5: Implement transactions and compare-and-swap**

Use `rusqlite::TransactionBehavior::Immediate` for:

- task claim;
- lease renewal;
- external ID recording;
- usage reconciliation;
- result acceptance;
- cancellation; and
- newly ready dependent activation.

Every update must include the expected previous state in its `WHERE` clause and require exactly one changed row.

- [ ] **Step 6: Implement recovery queries**

Add:

```rust
pub fn incomplete_runs(&self) -> Result<Vec<RunRecord>, StateError>;
pub fn nonterminal_attempts(&self, run_id: RunId) -> Result<Vec<AttemptRecord>, StateError>;
pub fn expire_leases(&self, run_id: RunId, now: i64) -> Result<usize, StateError>;
pub fn rebuild_ready_tasks(&self, run_id: RunId) -> Result<Vec<TaskId>, StateError>;
```

Do not infer that an expired lease means an external Codex turn is dead.

- [ ] **Step 7: Run state tests**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-state
```

Expected: transition ordering, exactly-once acceptance, lease, budget, and reducer-readiness tests pass.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock crates/harp-state
git commit -m "feat: add durable TaskGraph state transitions" \
  -m "Persist runs, tasks, attempts, operations, leases, budgets, events, and exactly-once accepted results in SQLite." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 5: Define the runtime contract and deterministic fake runtime

**Files:**
- Create: `crates/harp-runtime/Cargo.toml`
- Create: `crates/harp-runtime/src/lib.rs`
- Create: `crates/harp-runtime/src/contract.rs`
- Create: `crates/harp-runtime/src/fake.rs`
- Create: `crates/harp-runtime/tests/conformance.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Write the shared conformance suite**

Create a reusable function:

```rust
pub async fn assert_runtime_conformance(
    runtime: &mut impl CodexRuntime,
) -> Result<(), Box<dyn std::error::Error>> {
    let thread = runtime.start_thread(ThreadSpec::fixture()).await?;
    let turn = runtime
        .start_turn(&thread, TurnSpec::fixture_with_schema())
        .await?;
    let events = runtime.collect_until_terminal(&turn).await?;

    assert!(events.iter().any(RuntimeEvent::is_turn_started));
    assert!(events.iter().any(RuntimeEvent::is_token_usage));
    assert!(events.iter().any(RuntimeEvent::is_completed));
    assert_eq!(
        runtime.read_thread(&thread).await?.thread_id,
        thread.thread_id
    );
    runtime.shutdown_thread(thread).await?;
    Ok(())
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-runtime
```

Expected: FAIL because `CodexRuntime` and the fake do not exist.

- [ ] **Step 3: Define the object-safe runtime trait**

Use `async_trait` for the first slice:

```rust
#[async_trait::async_trait]
pub trait CodexRuntime: Send {
    async fn start_thread(&mut self, spec: ThreadSpec) -> Result<ThreadHandle, RuntimeError>;
    async fn start_turn(
        &mut self,
        thread: &ThreadHandle,
        spec: TurnSpec,
    ) -> Result<TurnHandle, RuntimeError>;
    async fn read_thread(
        &mut self,
        thread: &ThreadHandle,
    ) -> Result<ThreadSnapshot, RuntimeError>;
    async fn resume_thread(
        &mut self,
        thread: &ThreadHandle,
    ) -> Result<ThreadSnapshot, RuntimeError>;
    async fn next_event(&mut self) -> Result<RuntimeEvent, RuntimeError>;
    async fn interrupt(&mut self, turn: &TurnHandle) -> Result<(), RuntimeError>;
    async fn shutdown_thread(&mut self, thread: ThreadHandle) -> Result<(), RuntimeError>;
}
```

Normalize errors into:

```rust
pub enum RuntimeErrorKind {
    Startup,
    Transport,
    Protocol,
    ApprovalRequired,
    OutputSchema,
    Disconnected,
    NotFound,
}
```

- [ ] **Step 4: Implement scripted fake behavior**

`FakeCodexRuntime` must support:

- immediate success;
- disconnect after thread creation;
- disconnect after turn creation;
- completed turn visible only through `read_thread`;
- interrupted turn requiring one continuation;
- token budget crossing;
- approval request; and
- output-schema violation.

The fake records every call so tests can assert no duplicate semantic turn was created.

- [ ] **Step 5: Run conformance tests**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-runtime
```

Expected: fake runtime passes the shared conformance suite.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock crates/harp-runtime
git commit -m "feat: define the normalized Codex runtime" \
  -m "Add an object-safe adapter contract, normalized events and errors, and a scripted fake for deterministic recovery tests." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 6: Validate depth-one TaskGraphs and budgets

**Files:**
- Create: `crates/harp-engine/Cargo.toml`
- Create: `crates/harp-engine/src/lib.rs`
- Create: `crates/harp-engine/src/validate.rs`
- Create: `crates/harp-engine/src/projection.rs`
- Create: `crates/harp-engine/tests/validation.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Write table-driven validation failures**

Cover:

```text
duplicate task IDs
unknown dependency
cycle
more than one reducer
reducer missing a required analysis dependency
unapproved artifact
unknown output schema
aggregate token budget overflow
node count overflow
prompt projection overflow
requested recursion beyond depth one
```

Use stable diagnostic codes such as `graph.cycle` and `budget.aggregate_tokens`.

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-engine validation
```

Expected: FAIL because validation is not implemented.

- [ ] **Step 3: Implement deterministic validation**

Validation returns:

```rust
pub struct ValidatedGraph {
    graph: TaskGraph,
    topological_order: Vec<TaskId>,
    reducer: TaskId,
    aggregate_budget: Budget,
}
```

Do not mutate or repair model output in validation. Return bounded diagnostics for the one root repair attempt.

- [ ] **Step 4: Implement child projection**

`project_child_context` must include only:

- base and role instructions;
- task instruction;
- materialized input paths;
- artifact refs;
- private scratch path;
- operation marker;
- output schema;
- checkpoint contract; and
- remaining task budget.

Add a test that root and sibling sentinel strings do not appear in the child projection.

- [ ] **Step 5: Run validation tests**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-engine validation
```

Expected: all validation and projection tests pass.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock crates/harp-engine
git commit -m "feat: validate projected depth-one TaskGraphs" \
  -m "Reject invalid graphs and budgets before execution and construct bounded child contexts without ancestor transcript leakage." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 7: Execute and resume graphs with fake-runtime failure injection

**Files:**
- Create: `crates/harp-engine/src/scheduler.rs`
- Create: `crates/harp-engine/src/recovery.rs`
- Create: `crates/harp-engine/src/cancel.rs`
- Create: `crates/harp-engine/tests/execution.rs`
- Create: `crates/harp-engine/tests/recovery.rs`
- Modify: `crates/harp-engine/src/lib.rs`

- [ ] **Step 1: Write the uninterrupted reference execution test**

Create a graph with two analysis nodes and one reducer. Script the fake runtime to return two valid child envelopes and one valid reducer envelope.

Assert:

```rust
assert_eq!(summary.completed_tasks, 3);
assert_eq!(summary.indeterminate_attempts, 0);
assert_eq!(fake.start_turn_calls(), 3);
assert_eq!(state.accepted_results(run_id)?.len(), 3);
```

- [ ] **Step 2: Write the failure-injection restart matrix**

Use one parameterized test for:

```rust
enum CrashPoint {
    BeforeClaimCommit,
    AfterClaim,
    AfterPrepared,
    AfterDispatchingThread,
    AfterThreadId,
    AfterDispatchingTurn,
    DuringTurn,
    AfterArtifactPublication,
    AfterResultPublished,
    BeforeReducerReady,
    DuringReducer,
    AfterEvaluationPublication,
}
```

For each crash point:

1. run until injected failure;
2. drop engine and runtime;
3. reopen SQLite and artifacts;
4. construct a new runtime;
5. call `resume_run`;
6. compare normalized final state to the uninterrupted reference.

- [ ] **Step 3: Run recovery tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-engine recovery -- --nocapture
```

Expected: FAIL because scheduler and recovery are missing.

- [ ] **Step 4: Implement the scheduler**

The scheduler must:

- claim ready tasks transactionally;
- reserve budget before runtime calls;
- persist `Prepared` and `Dispatching*` states;
- persist `thread_id` and `turn_id` immediately;
- consume token events;
- interrupt on hard limits;
- publish result artifacts;
- compare-and-swap accepted results; and
- recompute reducer readiness from SQLite.

Keep maximum concurrency host-controlled with a Tokio semaphore.

- [ ] **Step 5: Implement recovery decision rules**

Map durable states exactly:

```rust
match attempt.state {
    AttemptState::Prepared => RecoveryAction::DispatchThread,
    AttemptState::DispatchingThread if attempt.thread_id.is_none() => {
        RecoveryAction::MarkIndeterminateAndRetry
    }
    AttemptState::ThreadStarted if attempt.turn_id.is_none() => {
        RecoveryAction::StartTurn
    }
    AttemptState::DispatchingTurn => RecoveryAction::ReadThreadForMarker,
    AttemptState::TurnStarted => RecoveryAction::ReadThreadForTurn,
    AttemptState::ResultPublished => RecoveryAction::AcceptPublishedResult,
    AttemptState::Succeeded
    | AttemptState::Failed
    | AttemptState::Indeterminate
    | AttemptState::Cancelled => RecoveryAction::None,
}
```

Never infer external death from lease expiry alone.

- [ ] **Step 6: Implement one bounded continuation**

If a known thread has a terminal incomplete turn and a valid checkpoint:

- increment `continuation_count`;
- reject if it exceeds `1`;
- render the continuation prompt;
- start a new turn on the same thread; and
- account for its tokens separately.

- [ ] **Step 7: Implement durable cancellation**

Cancellation must persist before `interrupt`. A restarted engine observes the cancellation flag and performs no new semantic work.

- [ ] **Step 8: Run the full recovery suite**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-engine
```

Expected: uninterrupted and all crash-point cases converge to the same accepted
results, except the explicit thread-start ambiguity at `DispatchingThread`,
which records one indeterminate attempt and bounded duplicate cost.

- [ ] **Step 9: Commit**

```bash
git add crates/harp-engine
git commit -m "feat: resume durable TaskGraph execution" \
  -m "Schedule, reconcile, continue, cancel, and recover graph work from SQLite across the specified crash boundaries." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 8: Implement the stdio App Server process adapter

**Files:**
- Create: `crates/harp-app-server-process/Cargo.toml`
- Create: `crates/harp-app-server-process/src/lib.rs`
- Create: `crates/harp-app-server-process/src/process.rs`
- Create: `crates/harp-app-server-process/src/jsonrpc.rs`
- Create: `crates/harp-app-server-process/src/protocol.rs`
- Create: `crates/harp-app-server-process/src/events.rs`
- Create: `crates/harp-app-server-process/tests/fake_server.rs`
- Create: `crates/harp-app-server-process/tests/fixtures/fake_app_server.py`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Write the fake stdio server**

The Python fixture reads one JSON object per line and implements:

```text
initialize
initialized
thread/start
turn/start
thread/read
thread/resume
turn/interrupt
```

It emits:

```text
turn/started
thread/tokenUsage/updated
item/completed with type=agentMessage
turn/completed
```

It can exit after `thread/start` or `turn/start` based on an environment variable.

- [ ] **Step 2: Write failing transport tests**

Test:

- initialize precedes all other methods;
- `initialized` follows initialize response;
- request IDs correlate responses;
- notifications do not consume pending responses;
- server-initiated requests are rejected fail-closed;
- bounded stderr is captured;
- malformed frames return `Protocol`;
- EOF returns `Disconnected`; and
- the shared runtime conformance suite passes.

- [ ] **Step 3: Run tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-app-server-process
```

Expected: FAIL because the adapter is missing.

- [ ] **Step 4: Implement the JSON-RPC pump**

Use one task to read stdout and one bounded channel for parsed messages. The wire omits `"jsonrpc":"2.0"`.

Initialize with:

```json
{
  "method": "initialize",
  "id": 1,
  "params": {
    "clientInfo": {
      "name": "harp_codex_rlm",
      "title": "Harp Codex-RLM",
      "version": "0.1.0"
    },
    "capabilities": {
      "experimentalApi": true,
      "optOutNotificationMethods": [
        "item/agentMessage/delta",
        "item/commandExecution/outputDelta"
      ]
    }
  }
}
```

Then send:

```json
{"method":"initialized","params":{}}
```

- [ ] **Step 5: Map normalized thread and turn requests**

`thread/start` must set:

```json
{
  "cwd": "/absolute/private/task-scratch",
  "runtimeWorkspaceRoots": ["/absolute/private/task-scratch"],
  "approvalPolicy": "never",
  "sandbox": "workspace-write",
  "ephemeral": false,
  "baseInstructions": "Analyze only the assigned task and return the required schema.",
  "developerInstructions": "Do not spawn subagents. Write checkpoints only beneath the assigned scratch directory.",
  "config": {
    "features.multi_agent": false,
    "features.multi_agent_v2.enabled": false
  }
}
```

The Codex corpus path remains outside `runtimeWorkspaceRoots`. The child may
read that absolute path but can write only beneath its private scratch root.
The root planner, which receives metadata and handles rather than repository
contents, uses a read-only thread.

If dotted request overrides are not accepted by the current Codex commit, write a run-local `CODEX_HOME/config.toml` with both multi-agent features disabled and verify the `spawn_agent` tool is absent from the child-visible tool list. Do not proceed with child execution until that conformance check passes.

`turn/start` includes a unique operation marker in input text and the exact JSON output schema.

`thread/read` uses `includeTurns: true` and extracts the final `agentMessage` item from the target completed turn.

- [ ] **Step 6: Add provenance and persistent Codex home**

For each run:

```text
.harp/runs/$RUN_ID/codex-home/
```

Persist the absolute path in run provenance. Set `CODEX_HOME` on the App Server process so a restarted adapter can `thread/read` and `thread/resume`.

Record Codex binary SHA-256, `codex --version`, sibling checkout SHA, and dirty diff digest.

- [ ] **Step 7: Run fake-server conformance**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-app-server-process
```

Expected: all transport and runtime conformance tests pass without building Codex.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock crates/harp-app-server-process
git commit -m "feat: add the App Server process adapter" \
  -m "Drive fresh, schema-constrained Codex threads over stdio JSON-RPC with persistent thread recovery and fail-closed approvals." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 9: Expose run, resume, checkpoint, and inspection commands

**Files:**
- Modify: `crates/harp/Cargo.toml`
- Modify: `crates/harp/src/lib.rs`
- Modify: `crates/harp/src/main.rs`
- Create: `crates/harp/src/rlm.rs`
- Create: `crates/harp/src/rlm_cli.rs`
- Modify: `crates/harp/tests/cli.rs`
- Modify: `Cargo.lock`

- [ ] **Step 1: Write failing CLI tests**

Add tests for:

```text
harp --format json rlm run --benchmark benchmarks/codex-architecture/benchmark.toml --runtime fake
harp --format json rlm resume "$RUN_ID" --runtime fake
harp --format json rlm status "$RUN_ID"
harp rlm checkpoint \
  --run-id "$RUN_ID" \
  --task-id "$TASK_ID" \
  --attempt-id "$ATTEMPT_ID" \
  --file checkpoint.json
```

JSON envelopes must include:

```json
{
  "schema_version": 1,
  "command": "rlm.run",
  "status": "ok",
  "data": {
    "run_id": "019fe297-4968-7842-9609-4f13d68fa672",
    "run_state": "completed"
  }
}
```

- [ ] **Step 2: Run CLI tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp --test cli rlm
```

Expected: FAIL because `rlm` commands do not exist.

- [ ] **Step 3: Add command structure**

Add:

```rust
enum Command {
    Check,
    Build {
        #[arg(long)]
        check: bool,
        #[arg(long, default_value = "atlas/src/content/generated/corpus.json")]
        output: PathBuf,
    },
    Search {
        #[command(subcommand)]
        command: SearchCommand,
    },
    Sources {
        #[command(subcommand)]
        command: SourcesCommand,
    },
    Rlm {
        #[command(subcommand)]
        command: RlmCommand,
    },
}

enum RlmCommand {
    Run(RunArgs),
    Resume(ResumeArgs),
    Status { run_id: String },
    Checkpoint(CheckpointArgs),
    Cancel { run_id: String },
    Compare(CompareArgs),
}
```

- [ ] **Step 4: Implement CLI boundary validation**

Validate:

- run ID syntax;
- benchmark path is beneath repository `benchmarks/`;
- `--runtime` is `fake`, `process`, or `inprocess`;
- process runtime requires a Codex binary path;
- checkpoint identity matches checkpoint contents; and
- `resume` refuses completed runs unless `--inspect-only`.

- [ ] **Step 5: Run CLI tests**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp --test cli
```

Expected: existing corpus CLI tests and new RLM CLI tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/harp Cargo.lock
git commit -m "feat: expose durable Codex-RLM commands" \
  -m "Add run, resume, status, checkpoint, cancel, and compare commands over the shared engine." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 10: Add root planning, child prompts, and the Codex architecture benchmark

**Files:**
- Create: `benchmarks/codex-architecture/benchmark.toml`
- Create: `benchmarks/codex-architecture/prompts/root.md`
- Create: `benchmarks/codex-architecture/prompts/child.md`
- Create: `benchmarks/codex-architecture/prompts/continuation.md`
- Create: `benchmarks/codex-architecture/prompts/reducer.md`
- Create: `benchmarks/codex-architecture/rubric.json`
- Create: `crates/harp-engine/src/planner.rs`
- Create: `crates/harp-engine/src/benchmark.rs`
- Create: `crates/harp-engine/tests/planner.rs`
- Modify: `crates/harp-engine/src/lib.rs`

- [ ] **Step 1: Define the benchmark**

`benchmark.toml`:

```toml
schema_version = 1
name = "codex-architecture"
corpus = "~/workspace/codex"
max_nodes = 8
max_concurrency = 4
max_root_tokens = 30000
max_child_tokens = 25000
max_total_tokens = 160000
max_wall_seconds = 1800
max_storage_bytes = 536870912
max_continuations_per_task = 1
semantic_threshold = 0.75
```

- [ ] **Step 2: Write root-planning tests**

Script the fake runtime to:

1. return an invalid graph;
2. receive bounded diagnostics;
3. return one valid repaired graph; and
4. fail the run if a third planner turn would be required.

Assert the persisted graph bytes are immutable after acceptance.

- [ ] **Step 3: Run tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-engine planner
```

Expected: FAIL because planner orchestration is missing.

- [ ] **Step 4: Write the root prompt**

The root prompt must state:

```text
You are planning, not executing.
Return only a TaskGraph matching the supplied JSON Schema.
Use only approved artifact handles.
Create 2-7 analysis tasks and exactly one reducer.
Do not ask children to spawn agents.
Do not exceed the supplied aggregate budget.
Every material report claim must name a repository path and evidence digest.
```

Provide corpus metadata and handles, not repository contents.

- [ ] **Step 5: Write child and reducer prompts**

The child prompt requires:

- root-cause versus symptom distinction;
- evidence records with file path and digest;
- checkpoint after every planned unit;
- bounded result summary; and
- no native subagent spawning.

The reducer prompt receives only result handles and bounded summaries.

- [ ] **Step 6: Implement plan, repair, persist, and execute**

`Planner::plan`:

1. starts a fresh root thread;
2. submits the TaskGraph schema;
3. validates the returned graph;
4. allows one repair turn with structured diagnostics;
5. persists exact accepted bytes and digest; and
6. invokes the scheduler.

- [ ] **Step 7: Run planner and benchmark tests**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-engine planner
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp --test cli rlm
```

Expected: fake-runtime benchmark reaches a completed run.

- [ ] **Step 8: Commit**

```bash
git add benchmarks/codex-architecture crates/harp-engine
git commit -m "feat: add model-generated Codex architecture graphs" \
  -m "Generate, repair once, validate, persist, and execute the evidence-linked Codex repository benchmark." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 11: Implement hybrid evaluation and adapter comparison

**Files:**
- Create: `crates/harp-eval/Cargo.toml`
- Create: `crates/harp-eval/src/lib.rs`
- Create: `crates/harp-eval/src/deterministic.rs`
- Create: `crates/harp-eval/src/semantic.rs`
- Create: `crates/harp-eval/src/compare.rs`
- Create: `crates/harp-eval/tests/evaluation.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Write failing deterministic evaluator tests**

Test rejection for:

```text
missing terminal task
schema-invalid envelope
evidence outside pinned corpus
evidence digest mismatch
budget overrun
shared writable scratch path
root transcript containing child result body
missing recovery receipt
DWARF receipt failure
```

- [ ] **Step 2: Write failing semantic rubric tests**

The semantic evaluator output schema is:

```json
{
  "type": "object",
  "properties": {
    "architectural_correctness": {"type": "number", "minimum": 0, "maximum": 1},
    "evidence_coverage": {"type": "number", "minimum": 0, "maximum": 1},
    "boundary_accuracy": {"type": "number", "minimum": 0, "maximum": 1},
    "integration_gap_quality": {"type": "number", "minimum": 0, "maximum": 1},
    "implementation_usefulness": {"type": "number", "minimum": 0, "maximum": 1},
    "unsupported_claim_rate": {"type": "number", "minimum": 0, "maximum": 1},
    "notes": {"type": "string"}
  },
  "required": [
    "architectural_correctness",
    "evidence_coverage",
    "boundary_accuracy",
    "integration_gap_quality",
    "implementation_usefulness",
    "unsupported_claim_rate",
    "notes"
  ],
  "additionalProperties": false
}
```

Assert the evaluator input contains final report and evidence manifest, but no child reasoning trace.

- [ ] **Step 3: Run tests and verify they fail**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-eval
```

Expected: FAIL because evaluator crate does not exist.

- [ ] **Step 4: Implement deterministic evaluation**

Return a receipt:

```rust
pub struct DeterministicEvaluation {
    pub passed: bool,
    pub checks: Vec<CheckResult>,
    pub normalized_result_sha256: String,
}
```

Checks are non-tradeable. Semantic score cannot override a failed deterministic gate.

- [ ] **Step 5: Implement protected semantic evaluation**

Run the semantic evaluator as a separate fresh thread with:

- read-only report and evidence artifacts;
- no access to scheduler DB;
- no access to benchmark threshold;
- fixed rubric;
- fixed budget; and
- schema-constrained output.

- [ ] **Step 6: Implement comparison**

Normalize away:

- adapter name;
- raw thread and turn IDs;
- wall-clock timestamps; and
- transport-specific diagnostics.

Preserve:

- accepted artifact digests;
- task reward;
- token usage;
- latency;
- event lag;
- failures;
- indeterminate attempts;
- duplicate compute;
- disk use;
- build size; and
- operator steps.

- [ ] **Step 7: Run evaluator tests**

Run:

```bash
CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-eval
```

Expected: deterministic gates, protected semantic input, and normalization tests pass.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock crates/harp-eval benchmarks/codex-architecture/rubric.json
git commit -m "feat: evaluate and compare Codex-RLM runs" \
  -m "Add non-tradeable deterministic checks, protected semantic scoring, and normalized adapter comparison metrics." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 12: Run real process-adapter acceptance against the sibling Codex checkout

**Files:**
- Create: `scripts/build_codex_app_server.sh`
- Create: `scripts/package_codex_app_server.sh`
- Modify: `mise.toml`
- Test: `crates/harp-app-server-process/tests/live_process.rs`

- [ ] **Step 1: Add an ignored live test**

The test reads:

```text
HARP_CODEX_BIN
HARP_CODEX_SOURCE="$HOME/workspace/codex"
```

It:

1. starts the real App Server;
2. initializes;
3. starts a read-only fresh thread;
4. verifies `spawn_agent` is unavailable;
5. starts a schema-constrained turn;
6. receives token and completion events;
7. kills and restarts the App Server;
8. reads the completed thread from the same run-local `CODEX_HOME`; and
9. verifies no second semantic turn was created.

Mark it `#[ignore = "requires a built and authenticated sibling Codex"]`.

- [ ] **Step 2: Add the Codex build script**

`scripts/build_codex_app_server.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

codex_root="${HARP_CODEX_SOURCE:-$HOME/workspace/codex}"
target_dir="${HARP_CODEX_TARGET_DIR:-$PWD/.build/codex-process-target}"

scripts/check_disk.sh "$codex_root"

(
  cd "$codex_root/codex-rs"
  CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_RELEASE_DEBUG=0 \
  CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=off \
  CARGO_PROFILE_RELEASE_STRIP=symbols \
  CARGO_TARGET_DIR="$target_dir" \
    cargo build --release -p codex-cli
)
```

Do not run this script while free space is below the configured threshold.

- [ ] **Step 3: Package and clean**

Copy the stripped Codex binary plus:

```json
{
  "source_path": "$HARP_CODEX_SOURCE",
  "source_commit": "$SOURCE_COMMIT",
  "dirty_diff_sha256": "$DIRTY_DIFF_SHA256",
  "binary_sha256": "$BINARY_SHA256",
  "built_at": "$BUILT_AT"
}
```

to:

```text
.build/packages/codex/$SOURCE_COMMIT/
```

The packaging script obtains these values with `git rev-parse HEAD`,
`git diff --binary | shasum -a 256`, `shasum -a 256 "$binary"`, and a UTC
timestamp. It serializes them with a real JSON encoder rather than shell string
concatenation.

Run `scripts/check_no_dwarf.sh` on the packaged binary, then remove the build target directory.

- [ ] **Step 4: Add `mise` tasks**

```toml
[tasks.build-codex-process]
run = "scripts/build_codex_app_server.sh && scripts/package_codex_app_server.sh"

[tasks.test-codex-process]
run = 'CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --profile test-small -p harp-app-server-process --test live_process -- --ignored --nocapture'
```

- [ ] **Step 5: Run only when disk preflight passes**

Run:

```bash
mise run build-codex-process
codex_package=$(find "$PWD/.build/packages/codex" -mindepth 2 -maxdepth 2 -type f -name codex -print -quit)
HARP_CODEX_BIN="$codex_package" mise run test-codex-process
```

Expected: live process conformance and restart recovery pass. If disk preflight fails, report the blocked acceptance test; do not weaken the threshold or emit a debug build.

- [ ] **Step 6: Commit**

```bash
git add scripts/build_codex_app_server.sh scripts/package_codex_app_server.sh mise.toml crates/harp-app-server-process/tests/live_process.rs
git commit -m "test: verify the sibling Codex process adapter" \
  -m "Build and package a stripped App Server from the live Codex checkout and exercise real thread recovery." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 13: Add the isolated in-process adapter experiment

**Files:**
- Create: `experiments/codex-inprocess/Cargo.toml`
- Create: `experiments/codex-inprocess/src/main.rs`
- Create: `experiments/codex-inprocess/src/runtime.rs`
- Create: `experiments/codex-inprocess/src/bootstrap.rs`
- Create: `experiments/codex-inprocess/tests/conformance.rs`
- Create: `scripts/package_inprocess.sh`
- Modify: `mise.toml`

- [ ] **Step 1: Create an independent Cargo workspace**

`experiments/codex-inprocess/Cargo.toml` begins:

```toml
[workspace]

[package]
name = "harp-codex-inprocess"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
harp-contracts = { path = "../../crates/harp-contracts" }
harp-runtime = { path = "../../crates/harp-runtime" }
codex-app-server-client = { path = "../../../codex/codex-rs/app-server-client" }
codex-app-server-protocol = { path = "../../../codex/codex-rs/app-server-protocol" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[profile.dev]
debug = 0
split-debuginfo = "off"
incremental = false
strip = "symbols"

[profile.release]
debug = 0
split-debuginfo = "off"
incremental = false
strip = "symbols"
lto = "thin"
codegen-units = 1
```

- [ ] **Step 2: Write the adapter conformance test**

Reuse the exact shared conformance suite from `harp-runtime`. Add a restart test that drops the in-process client, constructs a new client against the same Codex home, and reads the persisted thread.

- [ ] **Step 3: Run dependency metadata only**

Run:

```bash
cd experiments/codex-inprocess
cargo metadata --no-deps --format-version 1
```

Expected: path dependencies resolve to `~/workspace/codex`; no build artifacts are produced.

- [ ] **Step 4: Implement bootstrap from current Codex APIs**

Construct `InProcessClientStartArgs` using the same bootstrap pattern as current Codex `exec/src/lib.rs` and `tui/src/lib.rs`. Keep the bridge isolated in `bootstrap.rs` because these inputs are internal and may change across Codex commits.

Use:

```rust
let mut client = InProcessAppServerClient::start(args).await?;
let response: ThreadStartResponse = client
    .request_typed(ClientRequest::ThreadStart {
        request_id,
        params,
    })
    .await?;
```

Map typed `ServerNotification` variants to the same normalized `RuntimeEvent` values used by the process adapter.

- [ ] **Step 5: Build only after disk preflight**

Add:

```toml
[tasks.build-inprocess]
run = "scripts/check_disk.sh . && scripts/package_inprocess.sh"
```

`package_inprocess.sh` must:

1. build with `CARGO_TARGET_DIR="$HARP_INPROCESS_TARGET_DIR"` and `--release`;
2. run `scripts/check_no_dwarf.sh`;
3. copy the binary and Codex provenance manifest;
4. report target size; and
5. remove `$HARP_INPROCESS_TARGET_DIR` after successful packaging.

- [ ] **Step 6: Run in-process conformance when space allows**

Run:

```bash
mise run build-inprocess
```

Expected: conformance passes, the packaged runner has no DWARF, and the heavyweight target directory is removed. If preflight fails, record this adapter as build-blocked and continue using process-adapter evidence.

- [ ] **Step 7: Commit**

```bash
git add experiments/codex-inprocess scripts/package_inprocess.sh mise.toml
git commit -m "feat: add the isolated in-process Codex adapter" \
  -m "Bridge the sibling Codex App Server client behind the shared runtime contract without adding Codex to normal Harp builds." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 14: Execute the matched adapter benchmark and publish the comparison bundle

**Files:**
- Create: `scripts/run_codex_adapter_comparison.sh`
- Create: `docs/codex-rlm-stage-1-2-operations.md`
- Modify: `mise.toml`
- Generated, ignored: `.harp/runs/$RUN_ID/`
- Generated, ignored: `.harp/comparisons/$COMPARISON_ID/`

- [ ] **Step 1: Add a comparison script**

The script:

1. checks free space;
2. snapshots Codex source provenance;
3. runs root planning once through the process adapter;
4. persists the accepted graph;
5. executes that exact graph through the process adapter;
6. executes the same graph through the in-process adapter;
7. runs deterministic checks;
8. runs semantic evaluation;
9. normalizes results; and
10. writes `comparison.json` and `comparison.md`.

If the in-process binary is unavailable, exit with a distinct code and preserve the completed process run.

- [ ] **Step 2: Add a fake end-to-end comparison test**

Use two fake runtimes with transport-specific IDs but identical semantic outputs. Assert normalization reports equivalent accepted results while preserving different latency and startup metrics.

- [ ] **Step 3: Run the complete routine gate**

Run:

```bash
mise run fmt
mise run clippy
mise run test
mise run build
mise run no-dwarf
du -sh "$HARP_TARGET_DIR"
```

Expected: formatting, Clippy, all routine tests, compact build, and no-DWARF checks pass.

- [ ] **Step 4: Run real adapters if preflight permits**

Run:

```bash
mise run build-codex-process
mise run build-inprocess
mise run compare-codex-adapters
```

Expected: both adapters execute the same immutable graph. The comparison reports task reward, token usage, latency, event lag, restart outcomes, duplicate compute, peak disk, packaged size, and operator steps.

- [ ] **Step 5: Write operations documentation**

Document:

- exact compact build commands;
- free-space threshold;
- cleanup commands;
- how to run and resume;
- where SQLite and artifacts live;
- how `DispatchingThread` ambiguity appears;
- how to inspect checkpoints;
- how to cancel;
- how to rerun one adapter without replanning; and
- how to interpret comparison metrics.

- [ ] **Step 6: Commit**

```bash
git add scripts/run_codex_adapter_comparison.sh docs/codex-rlm-stage-1-2-operations.md mise.toml
git commit -m "docs: operationalize the Codex-RLM comparison" \
  -m "Document compact builds, durable recovery, matched adapter execution, cleanup, and comparison interpretation." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

## Final verification checklist

Run fresh:

```bash
mise run fmt
mise run clippy
mise run test
mise run build
mise run no-dwarf
git diff --check
git status --short
```

Verify manually:

- no `target/`, `.build/`, `.harp/`, `.dSYM`, or Codex source is tracked;
- the main workspace has no dependency on any Codex crate;
- the in-process package is outside the main workspace;
- every task result is content-addressed;
- every external operation has a stable operation ID;
- recovery does not repeat completed semantic work;
- `DispatchingThread` without `thread_id` becomes `Indeterminate`;
- only one attempt can be accepted;
- reducer readiness reconstructs after restart;
- cancellation survives restart;
- child contexts exclude root and sibling transcript sentinels;
- native `spawn_agent` is absent from child threads;
- deterministic evaluation cannot be overridden by semantic score;
- adapter comparison uses the same graph; and
- every commit contains the required trailer exactly once.

If there is enough free space, also run:

```bash
mise run build-codex-process
mise run test-codex-process
mise run build-inprocess
mise run compare-codex-adapters
```

If disk preflight blocks either heavyweight build, do not lower the threshold or switch to a debug build. Record the blocked acceptance step and retain all passing fake-runtime, process-protocol, persistence, recovery, and no-DWARF evidence.
