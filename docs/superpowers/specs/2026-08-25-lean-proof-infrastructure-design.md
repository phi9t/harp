# Harp Lean proof infrastructure design

## Status

Candidate design, version 0. The product boundary and authority model are
approved: the system is Harp-integrated, Crouzeix-first, and Harp Engine is the
only durable executor. Wire formats remain provisional until the Crouzeix
aggregate evidence and crash/replay fixtures exist.

This design is grounded in:

- [the LeanDojo project refresh](../../workstream/lean-proof-infrastructure/lean-dojo-refresh-2026-08-25.md);
- [the adjacent-efforts refresh](../../workstream/lean-proof-infrastructure/adjacent-efforts-refresh-2026-08-25.md);
- [the repository-grounded research synthesis](../../workstream/lean-proof-infrastructure/research.md);
- [ADR-0002](../../adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md);
- the Crouzeix CPFR-087 through CPFR-090 evidence contracts.

## Problem

Harp has strong pieces but not yet a coherent proof-system runtime. Its Engine
already owns durable scheduling, leases, retry, cancellation, recovery,
artifact publication, and technical result acceptance. Its Crouzeix work has
also developed unusually strict proof-evidence contracts: route identity,
source correspondence, reuse, type and axiom audits, immutable publication,
and independent reviews.

The current trace is nevertheless orchestrated by controllers and publication
scripts. It is not a declarative proof program executed by Harp's task graph.
The present execution types are agent-specific as well: `TaskNode` contains
prompt, model, permission, workspace, and output-schema fields; one
`ActivityRuntime` serves an entire run; runtime operations use thread and turn
handles; and durable attempt states name thread and turn transitions. Lean
cannot be added honestly by pretending a compiler worker is an agent session.

The next generation needs both of these loops:

1. a fast, incremental loop that helps produce proof candidates; and
2. a fresh, sealed loop that can certify a candidate and publish complete
   evidence.

They must share Harp's durable substrate without sharing proof authority.

## Goals

- Express a portable, bounded proof program and lower it into Harp execution.
- Keep one scheduler, state store, artifact store, recovery model, and status
  authority.
- Preserve existing agent-run behavior and persisted-run recovery.
- Make interactive results explicitly non-authoritative.
- Certify in a fresh worker over sealed, content-addressed inputs.
- Bind theorem, environment, checker, dependency, source, correction-lineage,
  review, and promotion identity.
- Derive semantic proof dependencies from Lean and fail closed on missing,
  opaque, extra, or out-of-boundary references.
- Use the completed Crouzeix trace as the first golden acceptance case.
- Keep caches useful for speed but outside the proof authority boundary.

## Non-goals

The first slice is not a LeanDojo replacement, remote proof service, persistent
Lean daemon, tactic-search platform, RL or training system, universal proof
protocol, or second workflow engine. It does not schedule one durable task per
tactic action. It does not hydrate dependencies during proof work, upgrade the
pinned Lean toolchain, claim cross-machine portability, or treat transcripts as
the system of record.

## Architectural shape

```text
ProofProgramCandidateV0
        | compile and validate
        v
backend-neutral ExecutionPlan
        |
        v
Harp Engine + Harp State + Harp ArtifactStore
        |-- AgentDriver              existing agent behavior
        |-- LeanInteractiveDriver    observations and candidates only
        `-- LeanCertifyDriver        fresh sealed certification

CertificationRunReceiptV0
        + CertificationOutcomeV0
        + semantic dependency audit
        + source/route correspondence
        + independent review receipts
        v
complete EvidenceBundleRefV0
        v
human-authored PromotionDecisionRefV0
        v
reader-facing promoted claim projection
```

`ExecutionPlan` is the migration seam. Existing `TaskGraph` version 1 lowers
losslessly into it, and a proof program lowers into it directly. The Engine
executes only the normalized plan. This preserves ADR-0002's single-executor
decision while removing the assumption that every activity is an agent turn.

The CLI remains thin. It validates an authored contract, resolves immutable
references, asks the Engine to execute, and renders projections. It does not
schedule work or interpret proof state.

## Three graphs, three authorities

The implementation must name and keep these graphs separate:

1. **Execution graph.** The macro schedule produced by lowering an authored
   proof program or existing task graph into `ExecutionPlan`.
2. **Semantic dependency graph.** The constants used by elaborated Lean proof
   bodies, with helper contraction and deterministic witness paths.
3. **Source/route graph.** Human-authored correspondence, provenance, reuse,
   and claim structure.

No graph substitutes for another. A route manifest is not proof-body evidence;
a proof-body edge does not establish source correspondence; and neither graph
decides operational scheduling.

## Authority model

The following outputs have deliberately different authority:

| Output | Producer | Authority |
| --- | --- | --- |
| `ProofObservationV0` | interactive Lean worker | Diagnostic only |
| `ProofCandidateRefV0` | interactive Lean worker through Engine publication | Immutable candidate, not certified |
| `CertificationOutcomeV0` | fresh Lean worker | Deterministic proof/check result |
| `CertificationRunReceiptV0` | Harp Engine | Operational execution evidence |
| `DependencyAuditRefV0` | Lean audit, independently revalidated | Semantic dependency evidence |
| `ReviewReceiptRefV0` | named review run | Review evidence within its declared independence tier |
| `EvidenceBundleRefV0` | Harp evidence validator | Complete evidence closure, not promotion |
| `PromotionDecisionRefV0` | human authority recorded by Harp | Promotion or rejection of one exact bundle digest |

Engine result acceptance means only that a technical task completed under its
contract. It must never imply mathematical promotion. Harp may record a human
decision with create-only, content-addressed semantics, but it may not infer or
auto-generate that decision.

Transcripts, dashboards, and reader pages are projections from these records.
They are not authority stores.

## Candidate contract sketch

The public candidate contracts live in `harp-contracts::proof`. They use strict
unknown-field rejection, bounded collections and text, canonical serialization,
and content references at every boundary.

```rust
pub struct ProofProgramCandidateV0 {
    pub schema_version: u8,
    pub program_id: ProofProgramId,
    pub environment: EnvironmentRefV0,
    pub source_graph: ArtifactRef,
    pub correction_lineage: Vec<ArtifactRef>,
    pub tasks: Vec<ProofTaskV0>,
}

pub struct ProofTaskV0 {
    pub task_id: ProofTaskId,
    pub theorem_contract: ArtifactRef,
    pub declaration: LeanDeclarationId,
    pub declaration_type_sha256: Sha256,
    pub declared_dependency_boundary: ArtifactRef,
    pub certification_policy: CertificationPolicyRefV0,
    pub artifact_channel: ProofArtifactChannelV0,
}

pub struct EnvironmentSpecV0 {
    pub source_tree: ArtifactRef,
    pub lean_toolchain: ArtifactRef,
    pub lake_manifest: ArtifactRef,
    pub dependency_tree: ArtifactRef,
    pub generated_sources: Vec<ArtifactRef>,
    pub lean_options: CanonicalJsonRef,
    pub platform_policy: PlatformPolicyV0,
    pub trust_policy: TrustPolicyRefV0,
    pub checker_policy: CheckerPolicyRefV0,
    pub cache_policy: CachePolicyRefV0,
}

pub struct EnvironmentBlockV0 {
    pub spec: EnvironmentRefV0,
    pub resolved_executables: Vec<ArtifactRef>,
    pub observed_inputs: Vec<ArtifactRef>,
    pub cache_snapshot_before: ArtifactRef,
    pub network_policy: NetworkPolicyV0,
    pub status: EnvironmentStatusV0,
}
```

`EnvironmentSpecV0` states what must be true. `EnvironmentBlockV0` records the
resolved, observed environment used by one run. A successful process exit such
as `elan which` is not environment identity.

The minimum result-side set is:

```text
ProofObservationV0
ProofCandidateRefV0
DependencyAuditRefV0
CertificationPolicyRefV0
CertificationRequestV0
CertificationOutcomeV0
CertificationRunReceiptV0
ReviewReceiptRefV0
EvidenceBundleRefV0
PromotionDecisionRefV0
```

`CertificationOutcomeV0` contains deterministic facts suitable for replay
comparison. `CertificationRunReceiptV0` contains operational facts such as
attempt identity, worker provenance, timestamps, and normalized logs. Replay
compares outcome digests, not timestamps, local paths, or worker metadata.

A proof-task identity binds the exact theorem contract, declaration and type,
artifact channel, source/route graph, declared dependency boundary, correction
lineage, environment, checker policy, and certification policy. The observed
dependency audit is receipt evidence and must equal the declared boundary.

## Engine and state migration

### Normalized execution plan

`harp-engine::execution_plan` owns a versioned internal representation:

```rust
pub struct ExecutionPlan {
    pub source_contract: ArtifactRef,
    pub tasks: Vec<ExecutionTask>,
}

pub struct ExecutionTask {
    pub task_id: TaskId,
    pub dependencies: Vec<TaskId>,
    pub activity: ActivityPlan,
    pub budget: Budget,
    pub retry_policy: RetryPolicy,
}

pub enum ActivityPlan {
    AgentV1(AgentActivityPlanV1),
    LeanInteractiveV0(LeanInteractivePlanV0),
    LeanCertifyV0(LeanCertifyPlanV0),
}
```

The current `TaskGraph` schema and validation behavior remain unchanged. Its
lowering must preserve graph identity and all current agent semantics. A later
wire version may expose the backend-neutral form only after the candidate
fixtures stabilize.

### Driver registry

`harp-engine::activity` replaces the one-runtime-per-run assumption with an
Engine-owned driver registry. Drivers translate a typed activity plan into
bounded effects and typed events. They do not lease tasks, retry attempts,
publish named evidence, or decide terminal state.

The existing agent runtime is wrapped as `AgentDriver`; it is not rewritten as
part of the Lean work. `LeanInteractiveDriver` may keep process-local warm
state for one Engine-owned attempt, but that state is disposable and cannot be
evidence. `LeanCertifyDriver` always starts fresh from sealed references.

Operations such as prepare, advance, audit, and certify remain internal driver
operations within an Engine-owned attempt. They are not durable service APIs.

### Generic durable attempts

`harp-state` must migrate from thread/turn-named phases to backend-neutral
activity phases:

```text
prepared
dispatching
running
reconciling
result_published
succeeded
failed
indeterminate
cancelled
```

An activity-effect journal records intent before each external effect and its
observed outcome afterward. Each record carries a stable effect identity and
idempotency key. Family-specific extension data records agent session/turn
handles or Lean worker/process details without changing the generic state
machine.

This is one database and one state authority. The migration must resume
persisted pre-migration agent fixtures and map their thread/turn phases without
losing ambiguity information. A second proof-state database is forbidden.

The existing content-addressed `harp-artifacts` store remains the only artifact
substrate unless crash testing proves a missing primitive.

## Lean adapter behavior

`crates/harp-lean` is a process adapter with no durable control plane.

### Interactive lane

- Resolve and validate a frozen environment before starting.
- Use an adapter-local mechanism such as a REPL, LSP, or Pantograph only behind
  the typed driver contract.
- Check the smallest affected module or declaration closure.
- Emit bounded observations and content-addressed candidate checkpoints.
- Persist macro checkpoints, not individual tactic actions or raw tactic state.
- Mark every result non-authoritative.

### Certification lane

- Start a fresh process with no inherited interactive state.
- Resolve only sealed source, toolchain, dependency, option, policy, and
  candidate references.
- Disable network access and dependency hydration.
- Treat the cache as an optional accelerator and compare before/after cache
  snapshots.
- Build the exact target, check declaration types and axioms, derive semantic
  dependencies, and emit deterministic outcomes plus operational receipts.
- Fail closed on missing bodies, opaque frontiers, out-of-closure project
  references, unexpected axioms, environment drift, cache mutation, or
  incomplete evidence.

## Semantic dependency contract

CPFR-088 is the first authority for this contract. Dependency extraction uses
Lean's elaborated environment and value bodies. Type references are retained
as diagnostics but do not create proof-provider edges.

For each registered declaration, the auditor performs a sorted FIFO traversal
through non-route project helpers. It stops at another registered route node,
emits the direct contracted edge, and records the lexicographically first
shortest witness path. Private and generated helpers are not filtered away.
Missing traversable bodies create an `opaque_frontier`; project-local constants
outside the declared module closure are blocking errors.

Python and Rust independently rebuild the graph and require exact set equality
between extracted and declared dependencies. Published route receipts remain
immutable; a mismatch creates additive correction lineage and recertification.

## Failure and retry policy

Failures are typed at the boundary: contract, environment, checker-policy,
dependency, proof, evidence, review, promotion, process, storage, and
indeterminate external effect. Deterministic contract, proof, dependency, and
policy failures are not retried. Transient process failures may use the
Engine's bounded retry policy only when the effect journal proves retry or
reconciliation is safe.

Unknown fields, oversized data, path escape, symlinks or hardlinks where
forbidden, stale digests, missing artifacts, ambiguous starts, and unsupported
schema versions fail closed.

## Compatibility contract

Before a Lean activity is admitted, the migration seam must prove:

- current `TaskGraph` version 1 JSON and validation are unchanged;
- Dynamic Workflow produces the same graph digest;
- existing agent result and artifact semantics are unchanged;
- a golden agent run has the same normalized state and artifact digests before
  and after lowering;
- the current crash/restart matrix still converges;
- persisted pre-migration SQLite fixtures resume successfully;
- runtime provenance, leases, cancellation, token and storage accounting, and
  workspace protections remain intact.

Compatibility means behavior and durable evidence, not merely source-level API
compilation.

## Crouzeix golden trace

The first end-to-end projection is intentionally narrow:

- CPFR-087 supplies the Harp route graph, exact LS reuse, and immutable route
  receipt/review identities.
- CPFR-088 supplies the six-member aggregate bundle, semantic dependency
  audit, provider and axiom evidence, and Python/Rust canonical fixtures.
- CPFR-089 supplies accurate reader-facing claims without changing evidence.
- CPFR-090 supplies the frozen verification record, review ordering, and the
  real human promotion boundary.

The new infrastructure projects these existing artifacts into candidate proof
contracts. It does not rewrite or republish them.

## Verification and acceptance

### Contract and parity

- Rust and Python accept identical canonical fixtures and reject the same
  mutations and unknown fields.
- Canonical bytes, digests, roster ordering, and bounded-size behavior are
  explicit.
- The six Crouzeix formalization rows and all route bindings reproduce exactly.

### Authority

- Interactive output cannot satisfy a certification or promotion input.
- Certification cannot reuse a mutable interactive worker.
- Engine technical acceptance cannot produce a promotion decision.
- Promotion names one exact complete evidence bundle and explicit review set.

### Recovery

- Crash tests cover intent-before-effect, ambiguous worker start, candidate
  publication, certification receipt publication, cancellation, and restart.
- Reconciliation produces zero duplicate accepted effects and zero lost
  terminal receipts.
- Re-running completed validation is idempotent.

### Proof evidence

- Direct and helper-mediated semantic dependencies are detected.
- Cycles terminate and equal shortest witnesses are deterministic.
- Type-only references do not create proof edges.
- Missing, extra, forged, opaque, or out-of-closure dependencies block.
- Sealed replay produces the same normalized certification outcome digest.

### Efficiency calibration

The initial measurement targets are provisional, not release promises:

- sealed preflight at or below 2 seconds;
- warm interaction p50 at or below 2 seconds and p95 at or below 10 seconds;
- affected-route certification p50 at or below 30 seconds and p95 at or below
  60 seconds;
- at least 99 percent normalized replay agreement before investigating every
  disagreement.

CPFR-088 measurements freeze or revise these targets. Cache hits may improve
latency but cannot change authoritative output.

## Frozen decisions

- Harp-integrated, Crouzeix-first, one durable Engine and state authority.
- Existing agent execution remains supported.
- Macro activities are durable; tactic actions are not.
- Interactive observations and candidates are non-authoritative.
- Certification uses a fresh sealed worker.
- Kernel checking, semantic dependency evidence, correspondence review,
  mathematical review, and promotion are separate authorities.
- Cache policy and identity are recorded; cache bytes are not proof evidence.
- Semantic dependency extraction is mandatory and fail-closed.
- No training, RL, standalone Lean service, or second control plane in v1.

## Evidence-gated decisions

- Stable proof wire version 1 waits for CPFR-088 fixtures and recovery acceptance.
- Promotion schema waits for the actual CPFR-089/090 review and owner-decision
  boundary.
- Checker-policy lifecycle waits for an explicit assessment of Lean 4.33.1
  soundness advisories. Current 4.32.1 results are provisional and
  replay-required, not silently upgraded.
- Cross-machine portability waits for an offline sealed replay on another
  machine; `complete-local` remains machine-local.
- REPL, LSP, Pantograph, or another interactive mechanism remains adapter-local.
- Reviewer independence records context, run, model, provider, organization,
  method, and shared-exposure fields, with unknowns explicit.

## Synthesis decision

The architectural base is the `ProofProgram -> Harp Engine` candidate. The
CPFR-088 candidate is grafted in as a mandatory evidence gate, because
handwritten route dependencies have already missed a real edge. The research
candidate supplies environment, task identity, correction-lineage, checker,
cache, and artifact-channel identity.

The decisive correction is the explicit backend-neutral `ExecutionPlan` and
generic activity-state migration. The earlier claim that `harp-state` could
remain unchanged was rejected after reading the current thread/turn-specific
schema and recovery path.

## Tradeoffs accepted

- We accept a real Engine/state seam migration in exchange for avoiding
  permanent Lean exceptions inside agent-shaped abstractions.
- We accept candidate wire formats in exchange for learning from CPFR-088 and
  crash/replay evidence before freezing a public protocol.
- We accept fresh-certification startup cost in exchange for a clear trust
  boundary.
- We accept non-authoritative interactive checkpoints in exchange for fast,
  disposable workers.
- We accept a Crouzeix-only first trace in exchange for a falsifiable,
  end-to-end acceptance target.

## Alternatives considered

- **Standalone LeanDojo successor.** Rejected because it duplicates Harp's
  scheduler, state, recovery, and authority model.
- **CPFR-088 scripts as the runtime.** Rejected because they are excellent
  evidence oracles but do not become a durable executor.
- **Lean encoded as agent turns.** Rejected because it hides incompatible
  process, result, recovery, and authority semantics behind misleading types.
- **Reuse the interactive worker for certification.** Rejected because mutable
  process state would enter the trust boundary.
- **One durable node per tactic action.** Rejected because the state and replay
  surface is disproportionate; content-addressed macro checkpoints suffice.
- **Route manifests as dependency truth.** Rejected because semantic proof-body
  use must be derived and independently checked.
- **A separate proof database.** Rejected as a second control plane.
- **Immediate universal stable protocol.** Rejected until the golden trace
  identifies the minimal real contract.

## Delivery sequence

The Crouzeix campaign keeps its existing ticket meanings and lands first:

1. finish CPFR-087 publication, verification, and local landing;
2. publish CPFR-088's semantic audit and exact six-row aggregate bundle;
3. reconcile CPFR-089 canonical prose and Atlas without evidence mutation;
4. execute CPFR-090 frozen verification and human promotion decision;
5. complete CPFR-091 worktree audit and the separate retrospective.

The infrastructure then proceeds as independently verifiable issues:

1. freeze candidate proof contracts and cross-language fixtures;
2. add lossless `TaskGraphV1 -> ExecutionPlan` lowering;
3. migrate generic activity persistence and add the driver registry;
4. certify a synthetic theorem with sealed preflight and a fresh worker;
5. project CPFR-087/088 into the new contracts without rewriting evidence;
6. add interactive observations and candidate checkpoints;
7. pass bounded crash/replay and atomic bundle-publication acceptance;
8. run the human stabilization gate for wire v1, promotion semantics, and
   portable-replay scope.

Issues 1 through 7 are agent-executable after their evidence dependencies are
met. Issue 8 is human-owned.

## Open questions and risks

- Does the generic activity journal need one new reconciliation primitive after
  the persisted-agent compatibility fixtures are exercised?
- Which Lean 4.33.1 soundness advisories apply to the pinned 4.32.1 Crouzeix
  results, and which receipts require replay?
- What exact signature or local identity should bind a human promotion record
  without pretending to provide public-key attestation?
- Which adapter gives the best interactive latency while remaining disposable
  and replaceable behind the contract?
- Which normalized fields are sufficiently platform-independent for the first
  honest cross-machine replay claim?

## Next implementation step

Finish CPFR-087 through CPFR-091, then write the implementation plan for the
candidate proof contracts and the lossless `ExecutionPlan` migration using the
frozen CPFR-088 fixtures.
