# Harp Architecture Deepening Program

**Status:** implementation authority for the six directions identified by the
2026-09-06 architecture review

## Goal

Increase Harp's architectural depth by concentrating behavior behind small
interfaces at six proven seams: Trace dialect interpretation, Session index
publication, Crouzeix proof graphs, durable execution, Dynamic Workflow
compilation, and knowledge registration. The program must improve locality and
test leverage without changing proof authority, execution semantics, corpus
authority, or the raw-rollout trust model.

## Scope and ownership split

The program spans two histories and therefore two isolated worktrees.

| Direction | Owning branch | Owning worktree | Current authority |
|---|---|---|---|
| Trace dialect module | `feat/harp-session-index` | `.worktrees/harp-session-index` | branch-local ADR 0003 and dialect `SPEC.md` |
| Session index publication | `feat/harp-session-index` | `.worktrees/harp-session-index` | branch-local ADR 0003 and dialect `SPEC.md` |
| Crouzeix graph kernel | `feat/architecture-deepening-wave-1` | `.worktrees/architecture-deepening-wave-1` | route manifests and proof receipts |
| Durable execution | `feat/architecture-deepening-wave-1` | `.worktrees/architecture-deepening-wave-1` | Engine and `ActivityRuntime` contracts |
| Dynamic Workflow compilation | `feat/architecture-deepening-wave-1` | `.worktrees/architecture-deepening-wave-1` | ADR 0002 and authored workflow contract |
| Knowledge registration | `feat/architecture-deepening-wave-1` | `.worktrees/architecture-deepening-wave-1` | managed Markdown and registered packet inputs |

No Session analyzer source is copied into the master-based branch. No ref owned
by another worktree is replaced. Integration across these histories is a later
owner-controlled landing decision.

## Program invariants

1. A module earns its seam under the deletion test: deleting it must spread
   meaningful behavior back across callers.
2. The interface is the test surface. New tests do not call private helpers.
3. Internal seams stay internal; a test seam is not automatically public.
4. Existing authorities remain unchanged:
   - Crouzeix route manifests and proof receipts remain proof authority.
   - `Engine` remains the only durable executor.
   - `ActivityRuntime` remains the runtime adapter seam.
   - managed Markdown remains prose authority; generated corpus and Atlas HTML
     remain derived.
   - Rollout artifacts remain Session analyzer authority.
5. First-wave work is behavior-preserving except for closing explicitly
   documented correctness gaps: dialect materiality, Session identity safety,
   and compiled workflow policy loss.
6. Every behavior change follows red-green-refactor. Structural moves retain
   characterization tests before old paths are deleted.
7. Generated artifacts are regenerated only after canonical inputs settle.
8. `docs/import-receipt.md` is refreshed last on the master-based branch.

## Direction 1: Trace dialect codec

### Problem

`detect.rs` selects one enum variant, while `index.rs` owns TraeCLI-specific
record interpretation. The seam is shallow: adding a second Trace dialect
would add branches inside indexing rather than a new adapter. Detection also
implements `history_mutation > 10` plus a fixture-only exception instead of the
specified materiality rule `> 10 OR >= 1% of nonblank records`.

### Target module

`dialect` is a deep module whose interface is:

```rust
pub struct DetectionCounts {
    pub nonblank: u64,
    pub event_msg: u64,
    pub history_mutation: u64,
    pub response_item: u64,
}

pub enum DetectedDialect {
    Supported(DialectId),
    Unsupported { dialect: String, detail: String },
}

pub trait TraceDialect {
    fn id(&self) -> DialectId;
    fn ingest(&self, record: &serde_json::Value, state: &mut IndexState);
}
```

Only two adapters justify the long-term seam: the supported
`TraeCliDualStreamV1` adapter and a typed unsupported-dialect adapter that
reports recognized response-item traces without interpreting them. Wave 1 may
keep the trait private while exposing `detect_dialect` and one codec entry
point.

### Required behavior

- Count nonblank JSONL records with checked arithmetic.
- Accept dual-stream only when `event_msg >= 1` and
  `history_mutation > 10 || history_mutation * 100 >= nonblank`.
- Refuse response-item-dominant input with `UnsupportedDialect`.
- Keep malformed JSONL errors line-addressed.
- Move TraeCLI record interpretation out of the storage/publication path.

### Tests

- A 100-record fixture with one `history_mutation` and one `event_msg` is
  accepted by the 1% rule.
- A 101-record fixture with one `history_mutation` is rejected.
- Existing response-item refusal remains typed.
- Codec tests assert `SessionSummary` observations through the public indexing
  interface, not codec internals.

### Waves

- Wave 1: correct materiality; introduce the codec entry point; move the
  existing `ingest_record` behavior behind it.
- Wave 2: add a second supported dialect only when fixtures meet the Index
  trust bar; then make the adapter seam public inside the crate.

## Direction 2: Session index streaming and publication

### Problem

`index_rollout` hashes by reading the whole Rollout artifact, reads it again,
buffers oversize lines, joins an unvalidated Execution session id into a path,
and writes final files directly. `verify_session` reuses the same indexing
implementation, so equality proves repeatability but not independent semantic
correctness.

### Target modules

Three deep modules divide the responsibilities:

1. `index`: read-only transformation from Rollout records to an in-memory
   `SessionSummaryDraft` plus spill intents.
2. `store`: validate Session identity, create a private staging tree, stream
   spills, and atomically publish one complete Session index.
3. `verify`: independently recompute documented invariants from the Rollout
   artifact and compare the stored projection.

The Wave 1 store interface is intentionally small:

```rust
pub fn validate_session_id(value: &str) -> SessionIndexResult<&str>;
pub fn session_dir(index_root: &Path, session_id: &str)
    -> SessionIndexResult<PathBuf>;
```

The later publication interface is:

```rust
pub fn publish_session_index(
    root: &Path,
    session_id: &str,
    write: impl FnOnce(&Path) -> SessionIndexResult<()>,
) -> SessionIndexResult<PathBuf>;
```

### Required behavior

- Session ids are nonempty ASCII UUID text and never contain separators, dot
  components, NULs, or control characters.
- No derived path escapes the Session index store.
- Publication is create-or-replace by complete atomic tree, never a partially
  visible directory.
- Oversize handling does not retain the whole line after it has been written
  to its content-addressed spill.
- Rollout SHA-256 is computed during the same streaming pass.
- Verification separately recounts schema-level invariants; deterministic
  reparse remains an additional check.

### Tests

- Reject `../escape`, `/absolute`, `a/b`, `.`, `..`, empty, control-containing,
  and non-UUID Session identities.
- Generated valid UUID identities resolve beneath the root.
- Simulated failure before rename leaves no visible partial index.
- A generated line larger than the threshold yields one stub and byte-identical
  spill while peak retained line data remains bounded by the streaming buffer.
- Corrupt one stored count and prove independent verification rejects it.

### Waves

- Wave 1: validate Session identity at the store seam and migrate all callers.
- Wave 2: single-pass hashing and streaming spill into staging.
- Wave 3: atomic tree publication and independent invariant verifier.

## Direction 3: Crouzeix theorem and obligation graph kernel

### Problem

`theorem_graph.py` is a valuable deep outer module, but generic graph
integrity, repository evidence binding, Crouzeix policy, serialization, and
Prove2Me projection share one implementation. Tests call `_load_readbacks`,
`_apply_readbacks`, `_validate_obligation_graph`, and
`_validate_receipt_binding`, making private helpers part of the effective
interface. Four depth-first traversals separately implement cycle detection,
topological order, and dependency closure.

### Target modules

- `graph_kernel.py`: pure deterministic algorithms over node ids and
  dependency maps. It knows nothing about repositories, Lean, Crouzeix, or
  Prove2Me.
- `theorem_graph.py`: the public Crouzeix interface. It binds route evidence,
  applies proof policy, and produces graph/export values using the kernel.
- A later `theorem_graph_repository.py`: bounded repository reads and receipt
  binding, introduced only after two concrete consumers justify the seam.

Wave 1 kernel interface:

```python
def topological_order(
    dependencies: Mapping[str, Iterable[str]],
    *,
    external_ids: Iterable[str] = (),
    cycle_label: str = "graph",
) -> tuple[str, ...]: ...

def dependency_closure(
    roots: Iterable[str],
    dependencies: Mapping[str, Iterable[str]],
    *,
    excluded: Iterable[str] = (),
    external_ids: Iterable[str] = (),
) -> tuple[str, ...]: ...
```

Both functions sort ids and dependency lists, reject dangling internal
dependencies, and return deterministic tuples. Duplicate domain ids remain an
outer-interface error because a dependency mapping cannot represent them.
Domain errors are translated to `TheoremGraphError` at the outer interface.

### Required behavior

- Route manifests and proof receipts remain authority.
- The obligation ledger and readbacks remain planning/inspection metadata.
- Graph schemas and serialized ordering do not change in Wave 1.
- Prove2Me remains deterministic, stdout-only, dry-run, and offline.
- Tests exercise public build/validate/project calls. Direct kernel tests cover
  generic algorithms; no new test calls Crouzeix private helpers.

### Tests

- Kernel returns stable order from deliberately shuffled input.
- Kernel detects self and multi-node cycles.
- Kernel accepts declared external dependencies and rejects undeclared ones.
- Kernel closure includes transitive prerequisites exactly once.
- Existing 35 graph tests remain green and serialized fixtures are unchanged.

### Waves

- Wave 1: add kernel; replace all four traversals; add public test helpers only
  where behavior cannot be reached through existing graph validation.
- Wave 2: introduce a repository evidence reader after the theorem graph and a
  second proof-graph consumer demonstrate shared behavior.
- Wave 3: replace tests of private Crouzeix helpers with fixture-driven public
  interface tests, then delete the exposed private paths.

## Direction 4: Durable execution application module

### Problem

`run_rlm` and `run_workflow` each assemble state, artifact storage, policies,
runtime adapter provenance, validated graphs, `RunExecutionSpec`, execution,
resume, and status output. `ActivityRuntime` is a real seam with fake and
process adapters; the CLI orchestration is duplicated implementation rather
than a module.

### Target module

Create a `durable_execution` application module above `Engine`. Its interface
accepts an Engine-ready plan and the existing environment/runtime adapters:

```rust
pub struct ExecutionPlan {
    pub graph: TaskGraph,
    pub graph_policy: GraphPolicy,
    pub projection_policy: ProjectionPolicy,
}

impl ExecutionPlan {
    pub fn validate(self) -> Result<ValidatedExecutionPlan, ValidationError>;
}
```

`ValidatedExecutionPlan` owns the validated graph and policies needed to form
`RunExecutionSpec`. It does not own storage or create runtime adapters. The CLI
remains responsible for command parsing and output formatting only.

### Required behavior

- Raw TaskGraph and Dynamic Workflow inputs both become `ExecutionPlan`.
- One path validates and starts a run; one path resumes a run.
- Runtime adapter construction remains outside the module.
- Engine, state, and artifact error types remain typed and are translated once
  at the CLI edge.
- RLM JSON output is byte-compatible in Wave 1.

### Tests

- An `ExecutionPlan` rejects a graph/policy mismatch.
- Fake runtime executes the same plan through RLM and Dynamic Workflow inputs.
- Resume does not duplicate accepted results.
- CLI tests assert only command parsing and envelope rendering after migration.

### Waves

- Wave 1: introduce `ExecutionPlan`; route Dynamic Workflow through it.
- Wave 2: route raw RLM run/resume through the same module.
- Wave 3: delete duplicated CLI orchestration and replace its tests with
  application-interface tests plus thin CLI checks.

## Direction 5: Dynamic Workflow compiled semantics

### Problem

`compile_dynamic_workflow` returns a graph, projection policy, and copied
options. Production uses the graph but reconstructs graph and projection policy
through `default_policies`, so the compiler's policy output is not the policy
that executes. The output interface is shallow and contradicts ADR 0002's
assignment of workflow translation to the compiler.

### Target module

`CompiledDynamicWorkflow` becomes an Engine-ready execution plan:

```rust
pub struct CompiledDynamicWorkflow {
    pub graph: TaskGraph,
    pub graph_policy: GraphPolicy,
    pub projection_policy: ProjectionPolicy,
}
```

The copied `options` field is removed. Policy ceilings and allowlists are
derived from the exact compiled nodes. The CLI contributes only environment
facts through `DynamicWorkflowCompileOptions`, including the scratch root.

### Required behavior

- The authored workflow is normalized before compilation exactly as today.
- The compiled graph policy admits exactly the roles, schemas, model policies,
  permission profiles, workspace modes, and budgets present in the graph.
- The compiled projection policy is the policy passed to `validate_graph`.
- Existing graph shape, reducer dependency order, and output remain unchanged.
- Compiler errors remain typed.

### Tests

- Give compiler-specific base instructions and prove the validated projected
  prompt contains them.
- Give an authored model policy and prove the reducer and graph allowlist use
  it.
- Give a schema absent from another node and prove both schemas are admitted.
- The production CLI workflow test exercises a policy value that would fail if
  `default_policies` were still substituted.

### Waves

- Wave 1: derive and return graph policy; remove copied options; consume both
  compiled policies in production.
- Wave 2: compose with the shared durable-execution application module and
  remove workflow-specific validation orchestration from `main.rs`.

## Direction 6: Knowledge registration and Atlas projection

### Problem

Reader routes and auxiliary documents are tuple arrays whose meaning is spread
across corpus loading, rendering, namespace validation, tests, and TypeScript
parsing. Adding a packet requires coordinated edits and array-length changes.
The TypeScript parser correctly remains strict, but registration knowledge has
weak locality.

### Target module

Create a typed Rust `KnowledgeRegistration` authority:

```rust
pub struct RegisteredDocument {
    pub id: String,
    pub path: String,
    pub reader: Option<ReaderRegistration>,
}

pub struct ReaderRegistration {
    pub route_id: String,
    pub label: String,
}

pub struct KnowledgeRegistration {
    documents: Vec<RegisteredDocument>,
}

impl KnowledgeRegistration {
    pub fn new(documents: Vec<RegisteredDocument>) -> Result<Self, AppError>;
    pub fn load(repository: &HeldDirectory) -> Result<Self, AppError>;
    pub fn documents(&self) -> impl Iterator<Item = &RegisteredDocument>;
    pub fn reader_routes(&self) -> impl Iterator<Item = &RegisteredDocument>;
    pub fn id_by_path(&self) -> BTreeMap<String, String>;
}
```

The authority merges static packet registration, optional knowledge home, and
generated Crouzeix textbook registration once. Rust consumers query it. Atlas
continues to parse generated JSON strictly; it does not duplicate a route-id
allowlist unless that restriction is an intentional UI contract.

### Required behavior

- Every registered path has exactly one canonical id.
- A reader route is a role on a registered document, not a second registry; its
  route id may differ from the document id for compatibility.
- Generated textbook aliases are explicit and collision-checked.
- Existing corpus JSON and reader ordering remain unchanged in Wave 1.
- TypeScript exhaustive variant handling remains strict.

### Tests

- Duplicate id, duplicate path, missing reader document, and alias collision
  fail through the registration interface.
- Existing 17 reader routes and 82 auxiliary entries preserve order and ids.
- Generated corpus JSON is byte-identical before and after Wave 1.
- Atlas tests parse the regenerated corpus through the existing strict seam.

### Waves

- Wave 1: add owned registration types, validate the existing static
  reader/auxiliary roster through `KnowledgeRegistration::new`, and derive its
  views while retaining the existing generated JSON schema.
- Wave 2: make `KnowledgeRegistration::load` merge optional knowledge-home and
  generated-textbook registrations, migrate contracts/render/tests, and delete
  tuple-array consumers. Existing packet-specific validators remain
  authoritative until that migration is complete.
- Wave 3: assess whether TypeScript route-id restrictions are derived schema
  validation or true UI policy, then remove only duplicated knowledge.

## Dependency order

```text
Session identity validation ──> atomic publication ──> independent verification
Dialect materiality ──────────> dialect codec ──────> second supported adapter

Graph kernel ─────────────────> repository evidence seam ─> private-test removal

Compiled workflow semantics ──> ExecutionPlan ──────> shared RLM/workflow runner

Typed registration ───────────> consumer migration ─> TypeScript duplication audit
```

The first wave chooses the leftmost node of every chain, plus the workflow
compiler's policy correction because the current production path discards
compiled semantics.

## First-wave acceptance

1. This design and `tracker.org` cover all six directions. Wave 1 is executable
   implementation authority; later waves are executable backlog specifications
   whose branch/worktree is created only when their wave begins.
2. The Crouzeix graph uses one tested deterministic kernel for traversal and
   closure while preserving all serialized output.
3. Dynamic Workflow production validation consumes compiler-derived graph and
   projection policies; `CompiledDynamicWorkflow.options` no longer exists.
4. The durable-execution plan type validates graph and policies through one
   interface and Dynamic Workflow uses it.
5. Corpus registrations are typed and can derive reader and auxiliary views
   without output changes.
6. Session dialect detection implements the exact materiality rule.
7. Session store rejects unsafe or malformed Session identities before joining
   paths.
8. Focused suites pass in both worktrees. The master-based worktree passes
   `mise run verify` after derived artifacts and the import receipt are updated.
9. A fresh architecture review reports what disappeared, what deepened, and
   which later-wave candidates remain.

## Non-goals for Wave 1

- No second supported Trace dialect.
- No full streaming oversize rewrite or independent Session semantic verifier.
- No Prove2Me network access, setup, login, launch, or submission.
- No proof statement, Lean theorem, proof receipt, or graph schema change.
- No migration of raw RLM execution to the application module yet.
- No generated corpus schema change.
- No merge, push, PR update, or modification of local `master`.
