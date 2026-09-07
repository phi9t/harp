# Harp Architecture Deepening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use phi9t_stack parallel-agents
> (recommended) or phi9t_stack plan to implement this plan task-by-task. Steps
> use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deepen all six reviewed architecture directions through staged,
test-first migrations, completing the first dependency wave now.

**Architecture:** Work remains split between the master-based architecture
worktree and the existing Session analyzer worktree. Each direction first
establishes one behavior-bearing interface, migrates callers in a later slice,
and deletes superseded paths only after interface-level tests cover them.

**Tech Stack:** Rust 2021, Python 3 `unittest`, TypeScript/React with pnpm,
Lean 4 through the existing warm cache, Git worktrees, mise.

---

## File map

| File | Responsibility after the program |
|---|---|
| `labs/crouzeix_proof_reproduction/graph_kernel.py` | Pure graph topology and closure algorithms |
| `labs/crouzeix_proof_reproduction/theorem_graph.py` | Crouzeix evidence binding, policy, graph and export interface |
| `crates/harp-engine/src/execution_plan.rs` | Engine-ready graph and policy validation |
| `crates/harp-engine/src/dynamic_workflow.rs` | Authored workflow to exact execution semantics |
| `crates/harp/src/corpus/registration.rs` | Typed knowledge registration authority |
| `crates/harp-session-index/src/dialect.rs` | Trace dialect record interpretation |
| `crates/harp-session-index/src/store.rs` | Session identity and publication |
| `crates/harp-session-index/src/index.rs` | Read-only indexing orchestration |
| `crates/harp-session-index/src/verify.rs` | Independent Session index invariants |

## Wave 1 — execute now

### Task 1: Pure Crouzeix graph kernel

**Files:**

- Create: `labs/crouzeix_proof_reproduction/graph_kernel.py`
- Create: `labs/crouzeix_proof_reproduction/tests/test_graph_kernel.py`
- Modify: `labs/crouzeix_proof_reproduction/theorem_graph.py`

- [ ] **Step 1: Write failing kernel tests**

```python
class GraphKernelTest(unittest.TestCase):
    def test_topological_order_is_deterministic(self):
        dependencies = {"c": ("b",), "a": (), "b": ("a",)}
        self.assertEqual(
            graph_kernel.topological_order(dependencies),
            ("a", "b", "c"),
        )

    def test_external_dependencies_are_not_emitted(self):
        dependencies = {"child": ("route-parent",)}
        self.assertEqual(
            graph_kernel.topological_order(
                dependencies, external_ids={"route-parent"}
            ),
            ("child",),
        )

    def test_cycle_is_typed(self):
        with self.assertRaisesRegex(graph_kernel.GraphKernelError, "cycle"):
            graph_kernel.topological_order({"a": ("b",), "b": ("a",)})

    def test_closure_is_dependency_first_and_unique(self):
        dependencies = {"root": ("b", "a"), "a": (), "b": ("a",)}
        self.assertEqual(
            graph_kernel.dependency_closure(("root",), dependencies, excluded={"root"}),
            ("a", "b"),
        )
```

- [ ] **Step 2: Verify red**

Run: `python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_graph_kernel`

Expected: import failure because `graph_kernel.py` does not exist.

- [ ] **Step 3: Implement the kernel**

```python
class GraphKernelError(ValueError):
    pass

def topological_order(
    dependencies: Mapping[str, Iterable[str]],
    *,
    external_ids: Iterable[str] = (),
    cycle_label: str = "graph",
) -> tuple[str, ...]:
    # DFS over sorted node ids and sorted dependencies. Reject unknown ids that
    # are not declared external. Emit each internal node after dependencies.

def dependency_closure(
    roots: Iterable[str],
    dependencies: Mapping[str, Iterable[str]],
    *,
    excluded: Iterable[str] = (),
    external_ids: Iterable[str] = (),
) -> tuple[str, ...]:
    # Validate through topological_order, then return the dependency-first
    # reachable subset without excluded roots.
```

- [ ] **Step 4: Verify green**

Run: `python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_graph_kernel`

Expected: all kernel tests pass.

- [ ] **Step 5: Characterize serialized output**

Run before migration and save outside the repository:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py theorem-graph > "${TMPDIR:-/tmp}/harp-theorem-graph-before.json"
python3 labs/crouzeix_proof_reproduction/proof_evidence.py proof-obligation-graph > "${TMPDIR:-/tmp}/harp-obligation-graph-before.json"
python3 labs/crouzeix_proof_reproduction/proof_evidence.py prove2me-export --dry-run > "${TMPDIR:-/tmp}/harp-prove2me-before.json"
```

- [ ] **Step 6: Replace four traversals**

Import `graph_kernel`; translate `GraphKernelError` to `TheoremGraphError`.
Replace `_dependency_closure`, `_topological_order_obligations`,
`_assert_acyclic`, and `_topological_order` implementations with the kernel.
Do not change serialized shapes or error classes.

- [ ] **Step 7: Verify behavior and byte equality**

Run:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_graph_kernel
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_theorem_graph
python3 labs/crouzeix_proof_reproduction/proof_evidence.py theorem-graph > "${TMPDIR:-/tmp}/harp-theorem-graph-after.json"
python3 labs/crouzeix_proof_reproduction/proof_evidence.py proof-obligation-graph > "${TMPDIR:-/tmp}/harp-obligation-graph-after.json"
python3 labs/crouzeix_proof_reproduction/proof_evidence.py prove2me-export --dry-run > "${TMPDIR:-/tmp}/harp-prove2me-after.json"
cmp "${TMPDIR:-/tmp}/harp-theorem-graph-before.json" "${TMPDIR:-/tmp}/harp-theorem-graph-after.json"
cmp "${TMPDIR:-/tmp}/harp-obligation-graph-before.json" "${TMPDIR:-/tmp}/harp-obligation-graph-after.json"
cmp "${TMPDIR:-/tmp}/harp-prove2me-before.json" "${TMPDIR:-/tmp}/harp-prove2me-after.json"
```

Expected: tests pass and all `cmp` commands exit 0.

### Task 2: Engine-ready Dynamic Workflow compilation

**Files:**

- Create: `crates/harp-engine/src/execution_plan.rs`
- Modify: `crates/harp-engine/src/dynamic_workflow.rs`
- Modify: `crates/harp-engine/src/lib.rs`
- Modify: `crates/harp-engine/tests/dynamic_workflow.rs`
- Modify: `crates/harp/src/main.rs`
- Modify: `crates/harp/tests/cli.rs` only if the existing production-path tests
  do not expose the policy substitution

- [ ] **Step 1: Write failing execution-plan tests**

Extend `crates/harp-engine/tests/dynamic_workflow.rs`:

```rust
#[test]
fn compiled_workflow_owns_the_exact_validation_policies() {
    let compiled = compile(DynamicWorkflowStep::Agent(agent("discover")));
    assert!(compiled
        .graph_policy
        .allowed_model_policies
        .contains("test-model"));
    assert_eq!(compiled.projection_policy.base_instructions, "Follow the workflow.");
    let validated = ExecutionPlan::from(compiled)
        .validate()
        .expect("compiled plan validates");
    assert_eq!(validated.graph().topological_order().len(), 2);
}
```

Update the invalid-workflow test to construct options directly instead of
reading a copied `compiled.options` field.

- [ ] **Step 2: Verify red**

Run: `cargo test -p harp-engine --test dynamic_workflow`

Expected: compile failure because `graph_policy` and `validate` do not exist.

- [ ] **Step 3: Implement `ExecutionPlan`**

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPlan {
    graph: TaskGraph,
    graph_policy: GraphPolicy,
    projection_policy: ProjectionPolicy,
}

impl ExecutionPlan {
    pub fn new(
        graph: TaskGraph,
        graph_policy: GraphPolicy,
        projection_policy: ProjectionPolicy,
    ) -> Self;

    pub fn validate(self) -> Result<ValidatedExecutionPlan, ValidationError>;
}
```

Implement `From<CompiledDynamicWorkflow> for ExecutionPlan`.
`ValidatedExecutionPlan` exposes read-only accessors and an `into_parts` method
for the existing `RunExecutionSpec` constructor. `ExecutionPlan::validate` calls
`validate_graph`; `RunExecutionSpec::new` deliberately replays validation at the
durable-receipt boundary and must continue to do so.

- [ ] **Step 4: Derive exact graph policy in the compiler**

Add a private `graph_policy(&TaskGraph) -> GraphPolicy` function in
`dynamic_workflow.rs`. Compute ceilings with checked addition and derive every
allowlist from the compiled nodes. Set `max_concurrency` to
`nodes.len().clamp(1, 4)`, matching current production behavior. Return:

```rust
pub struct CompiledDynamicWorkflow {
    pub graph: TaskGraph,
    pub graph_policy: GraphPolicy,
    pub projection_policy: ProjectionPolicy,
}
```

Delete `options: DynamicWorkflowCompileOptions`.

- [ ] **Step 5: Verify compiler green**

Run: `cargo test -p harp-engine --test dynamic_workflow`

Expected: all compiler tests pass.

- [ ] **Step 6: Consume compiled policies in production**

In `run_workflow`, create `ExecutionPlan::new(compiled.graph,
compiled.graph_policy, compiled.projection_policy)`, validate it, and pass the
validated graph plus the exact policies into `RunExecutionSpec`. Keep
`default_policies` for raw RLM only.

- [ ] **Step 7: Verify the production path**

Run:

```sh
cargo test -p harp-engine --test dynamic_workflow
cargo test -p harp --features test-cli-fixture --test cli workflow_run -- --test-threads=1
```

Expected: all selected tests pass and the workflow run output is unchanged.

### Task 3: Typed knowledge registration

**Files:**

- Create: `crates/harp/src/corpus/registration.rs`
- Modify: `crates/harp/src/corpus/mod.rs`
- Modify: `crates/harp/src/corpus/contracts.rs`
- Modify: `crates/harp/src/corpus/render.rs`
- Modify: `crates/harp/src/corpus/tests.rs`

- [ ] **Step 1: Capture generated output before migration**

Run: `cp atlas/src/content/generated/corpus.json "${TMPDIR:-/tmp}/harp-corpus-before.json"`

- [ ] **Step 2: Write failing typed-registration tests**

```rust
#[test]
fn registration_derives_reader_and_auxiliary_views() {
    let registration = registration::static_registration().unwrap();
    assert_eq!(registration.reader_routes().count(), 17);
    assert_eq!(registration.auxiliary_documents().count(), 82);
}

#[test]
fn registration_rejects_duplicate_id_and_path() {
    let duplicate_id = [
        RegisteredDocument::auxiliary("same", "knowledge/a.md"),
        RegisteredDocument::auxiliary("same", "knowledge/b.md"),
    ];
    assert!(KnowledgeRegistration::new(&duplicate_id).is_err());
}
```

Add separate duplicate-path and duplicate-reader-label cases. The Wave 1
registration accepts owned values, so Wave 2 can merge optional and generated
documents without changing its interface. Existing knowledge-home and textbook
collision tests remain green behind their current validators until the complete
consumer migration.

- [ ] **Step 3: Verify red**

Run: `cargo test -p harp corpus::tests::registration --lib -- --test-threads=1`

Expected: compile failure because the registration module does not exist.

- [ ] **Step 4: Implement typed registration**

Move the static values behind `RegisteredDocument` constructors with an optional
`ReaderRegistration { route_id, label }` role. Validate unique document ids,
paths, reader route ids, and labels. Provide derived iterators used by contracts,
rendering, and tests. Do not change ordering.

- [ ] **Step 5: Migrate consumers and verify**

Run:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo run -p harp -- build --output atlas/src/content/generated/corpus.json
cmp "${TMPDIR:-/tmp}/harp-corpus-before.json" atlas/src/content/generated/corpus.json
(cd atlas && corepack pnpm run test)
```

Expected: tests pass and generated JSON is byte-identical.

### Task 4: Trace dialect materiality and codec locality

**Worktree:** `.worktrees/harp-session-index`

**Files:**

- Create: `crates/harp-session-index/src/dialect.rs`
- Modify: `crates/harp-session-index/src/detect.rs`
- Modify: `crates/harp-session-index/src/index.rs`
- Modify: `crates/harp-session-index/src/lib.rs`
- Modify: `crates/harp-session-index/tests/detect_fixtures.rs`

- [ ] **Step 1: Write failing percentage tests**

Generate temporary JSONL with one `session_meta`, one `event_msg`, one
`history_mutation`, one `response_item`, and enough neutral records for exactly
100 nonblank lines. Assert detection succeeds. Generate 101 nonblank lines with
the same material records and assert `DetectFailed`. Including one
`response_item` ensures the old fixture-only `response_item == 0` exception
cannot make the 100-line RED test pass accidentally.

- [ ] **Step 2: Verify red**

Run: `cargo test -p harp-session-index --test detect_fixtures`

Expected: the 100-line test fails because the fixture-only exception is not the
specified percentage rule, or the 101-line test exposes the exception as too
permissive.

- [ ] **Step 3: Implement checked materiality**

```rust
fn is_material(history_mutation: u64, nonblank: u64) -> bool {
    history_mutation > 10
        || history_mutation
            .checked_mul(100)
            .is_some_and(|scaled| scaled >= nonblank)
}
```

Remove the `response_item == 0` fixture exception.

- [ ] **Step 4: Move record interpretation**

Move `ingest_record` and its dialect-specific accumulation state into
`dialect.rs`. Expose one crate-internal operation that accepts parsed JSON and
updates a draft. Keep `SessionSummary` as the public result.

- [ ] **Step 5: Verify green**

Run: `cargo test -p harp-session-index`

Expected: all Session index tests pass.

### Task 5: Session identity at the store seam

**Worktree:** `.worktrees/harp-session-index`

**Files:**

- Modify: `crates/harp-session-index/src/store.rs`
- Modify: `crates/harp-session-index/src/index.rs`
- Modify: `crates/harp-session-index/src/error.rs`
- Modify: `crates/harp-session-index/src/verify.rs`
- Modify: `crates/harp-session-index/src/bin/harp-session-index.rs`
- Modify: `crates/harp-session-index/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/harp-session-index/tests/index_fixtures.rs`
- Modify: `crates/harp-session-index/fixtures/*.jsonl` session identities only

- [ ] **Step 1: Write failing path-escape tests**

For each value `""`, `"."`, `".."`, `"../escape"`, `"a/b"`,
`"/tmp/escape"`, `"not-a-uuid"`, and a control-containing string, create
a minimal otherwise-valid dual-stream Rollout artifact and assert
`index_rollout` returns `InvalidSessionId` and creates nothing outside the
temporary index root.

- [ ] **Step 2: Verify red**

Run: `cargo test -p harp-session-index --test index_fixtures`

Expected: path escape or wrong error because `session_dir` blindly joins.

- [ ] **Step 3: Implement the fallible store interface**

```rust
pub fn session_dir(index_root: &Path, session_id: &str)
    -> SessionIndexResult<PathBuf> {
    let parsed = uuid::Uuid::parse_str(session_id)
        .map_err(|_| SessionIndexError::InvalidSessionId {
            value: session_id.to_owned(),
        })?;
    if parsed.to_string() != session_id {
        return Err(SessionIndexError::InvalidSessionId { value: session_id.to_owned() });
    }
    Ok(index_root.join(session_id))
}
```

Add the `uuid` dependency and a typed error variant. Require lowercase canonical
hyphenated UUID text so one Execution session has one store identity. Replace
the fixture-only non-UUID identifiers with stable canonical UUIDs and update
their exact test assertions; do not alter other captured record content.
Add direct store-interface tests for one canonical UUID plus uppercase, simple,
braced, and non-hyphenated UUID spellings. Migrate indexing, verification, and
CLI show callers to propagate the typed result.

- [ ] **Step 4: Migrate callers and verify green**

Run: `cargo test -p harp-session-index`

Expected: all Session index tests pass and no unsafe path is created.

### Task 6: First-wave repository verification and review

**Files:**

- Modify: `docs/workstream/architecture-deepening/tracker.org`
- Regenerate: `atlas/src/content/generated/corpus.json` only if canonical input
  produces a change
- Regenerate: `atlas/dist/harp-atlas.html` whenever corpus JSON changes
- Modify last: `docs/import-receipt.md`

- [ ] **Step 1: Run formatting and focused tests in both worktrees**

```sh
cargo fmt --all -- --check
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_graph_kernel
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_theorem_graph
cargo test -p harp-engine --test dynamic_workflow
cargo test -p harp --features test-cli-fixture --test cli workflow_run -- --test-threads=1
cargo test -p harp corpus::tests --lib -- --test-threads=1

# In .worktrees/harp-session-index
cargo fmt --all -- --check
cargo test -p harp-session-index
```

- [ ] **Step 2: Run independent diff reviews**

Review standards and specification separately. Repair every finding, then rerun
the focused tests.

- [ ] **Step 3: Run the master-based full gate**

Run: `mise run verify`

If and only if repository verification reports a stale payload digest, copy the
reported digest into `docs/import-receipt.md` as the final tracked edit and run
`mise run verify` again. Never hydrate Lean dependencies.

- [ ] **Step 4: Record evidence**

Move implemented tracker items to `DONE` only after exact commands, pass counts,
and review disposition are recorded. Leave later waves in `BACKLOG`.

- [ ] **Step 5: Repeat the architecture review**

Apply the same glossary, deletion test, and six-candidate scoring rubric used by
the original review to both worktrees. Write
`${TMPDIR:-/tmp}/architecture-review-<timestamp>.html`, open it with the local
browser, and record the absolute path plus a before/after table under
`ARCH-011` Verification Evidence. The table must classify every direction as
deep, transitional, or unchanged and link each score to current source/tests.

## Wave 2 — migrate callers and publication mechanics

### Task 7: Single-pass Session streaming and atomic publication

**Files:** `src/index.rs`, `src/spill.rs`, `src/store.rs`,
`tests/index_fixtures.rs`, and new `tests/publication.rs` in
`crates/harp-session-index/`.

1. Add a generated threshold-plus-one record test that asserts exact spill
   bytes/digest and a fault-injection test whose writer fails before rename.
2. Run `cargo test -p harp-session-index --test index_fixtures --test publication`
   and observe retained-line/publication failures.
3. Replace whole-file hashing and `Vec<Vec<u8>>` with one bounded reader that
   hashes bytes while streaming oversized bytes to a private staging tree.
4. Implement `publish_session_index(root, session_id, write)` with same-filesystem
   staging, file/directory sync, and one rename; clean staging on failure.
5. Rerun the two tests, then `cargo test -p harp-session-index`; require exact
   spill equality and no partially visible final directory.

### Task 8: Independent Session verification

**Files:** `src/verify.rs`, new `src/recount.rs`, and
`tests/verify_fixtures.rs`.

1. Add table-driven mutations for line count, top-level type counts, Session id,
   and spill digest/length; run `cargo test -p harp-session-index --test verify_fixtures`
   and observe false passes.
2. Add a verifier-local bounded recount that does not call `index_rollout` and
   compare those fields before deterministic scratch re-indexing.
3. Run the focused test and `cargo test -p harp-session-index`; every mutation
   must fail with the matching field check while an untouched index passes.

### Task 9: Raw RLM durable-execution migration

**Files:** new `crates/harp/src/durable_execution.rs`,
`crates/harp/src/main.rs`, `crates/harp-engine/src/execution_plan.rs`, new
`crates/harp/tests/durable_execution.rs`, and `crates/harp/tests/cli.rs`.

1. Add fake-runtime tests proving equivalent RLM/workflow plans execute and
   resume without duplicate accepted results; run the focused test RED.
2. Move run/resume application orchestration behind one module accepting a
   validated plan plus injected state, artifact, and `ActivityRuntime` adapters.
3. Route both CLI forms through it, delete duplicated orchestration, and retain
   only parsing/envelope assertions in CLI tests.
4. Run `cargo test -p harp --test durable_execution`, the Engine recovery suite,
   and the feature-enabled focused CLI run tests.

### Task 10: Complete knowledge-registration migration

**Files:** `crates/harp/src/corpus/registration.rs`, `mod.rs`, `contracts.rs`,
`render.rs`, `tests.rs`, `atlas/src/content/contracts.ts`, and generated corpus
and Atlas HTML only when bytes change.

1. Add registration-interface tests for optional knowledge home, textbook
   aliases, missing reader documents, id/path/label collisions, and stable order.
2. Run the focused corpus tests RED, then make `KnowledgeRegistration::load`
   merge every source and migrate contracts/render/tests one consumer at a time.
3. Delete tuple registries only after `rg` finds no callers; classify each
   TypeScript restriction as boundary validation or duplicated authority.
4. Run corpus and Atlas suites, rebuild with `cargo run -p harp -- build --output
   atlas/src/content/generated/corpus.json`, and compare corpus/HTML baselines.

### Task 11: Remove Crouzeix private test surfaces

**Files:** `theorem_graph.py`, optional new `theorem_graph_repository.py`, and
`tests/test_theorem_graph.py`. Add fixture-root public-interface tests for altered
route status, receipt membership, and readback binding; observe them fail, then
move bounded repository reads behind one internal adapter. Migrate every direct
underscore-helper test, delete helpers only after `rg` finds no test callers, and
run both graph suites plus byte comparisons for all three exports.

## Wave 3 — prove real seams and delete transitional paths

### Task 12: Add a second supported Trace dialect

**Files:** `src/dialect.rs`, `src/detect.rs`, `tests/detect_fixtures.rs`,
`tests/index_fixtures.rs`, and a pinned licensed fixture family. First add public
index tests for the second dialect and retain typed refusal for an unknown third
shape. Implement the adapter without changing the shared indexing loop, run
`cargo test -p harp-session-index`, and record fixture provenance/digests.

### Task 13: Audit TypeScript corpus knowledge

**Files:** `atlas/src/content/types.ts`, `contracts.ts`, their tests, and the Rust
corpus schema emitter. Add a test that introduces a valid registered route absent
from the old hard-coded TypeScript roster and observe rejection. Retain structural
JSON validation, replace duplicated registration knowledge with generated or
data-driven checks, and run `(cd atlas && corepack pnpm run test)` plus the corpus
suite and generated-output comparison.

## Final architecture review

Repeat `/improve-codebase-architecture` after Wave 1. Read the updated glossary
and ADRs, inspect both worktrees, apply the deletion test, write a fresh
`architecture-review-<timestamp>.html` under the OS temp directory, open it, and
compare candidate strength with the original report. The report must state
which seams are now deep, which are transitional, and which later waves remain.
