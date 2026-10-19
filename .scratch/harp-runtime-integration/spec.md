# Harp durable coding-worker integration

Status: implemented; qualification recorded in tickets.md
Owner: project owner
Branch: codex/harp-runtime-integration-spec
Baseline: 1418455d
Authority: specification only; implementation, live provider runs and service setup require their own authorization.

## Problem Statement

Harp contains several overlapping execution and controller implementations after
absorbing Corenius. No single supported path yet combines durable coding work,
both Codex and Trae, isolated workspaces, exact verification, explicit human
approval and a correctly synchronized local landing. Preserving source history
has not qualified that combined behavior.

The owner wants one local system whose work survives interruption without losing
accepted results or silently repeating ambiguous effects. It must progress from
one worker to two and then four, while keeping intended work in Git and avoiding
Temporal, PostgreSQL and competing controller authorities.

## Solution

Adopt the Corenius adaptation of the lightweight Python + SQLite durable runtime
as Harp's sole execution authority. Retain its replay, activity, guardian, receipt,
Insula and cgroup mechanisms. Integrate the useful task-admission, provider,
candidate, verification and landing contracts from the other implementations.

Provide a CLI-first interface on Linux at devx_b200_dbg. The operator explicitly
starts and stops a project-scoped runtime; SQLite and files persist between
invocations. Codex and Trae are both supported from the first milestone. Provider
selection is explicit and remains bound to an admitted attempt.

Git-managed Markdown owns task intent and acceptance criteria. The runtime pins
admitted input revisions and owns operational history. Files retain logs,
artifacts and receipts. Insula owns execution admission and isolation. A human
approves the exact verified candidate before local landing.

### Delivery milestones

| Milestone | Required useful result | Exit evidence |
| --- | --- | --- |
| 1: one worker | Complete a real coding task through admission, isolated execution, candidate capture, verification, human approval and local landing. Exercise Codex and Trae in separate single-worker runs. Include cancellation and safe restart recovery. | Public CLI lifecycle and fault tests pass; real tasks complete with each provider; the configured capacity of one is enforced. |
| 2: two workers | Run a mixed Codex/Trae pair in isolated workspaces without duplicated admission or competing landing authority. | Two workers actually overlap; cancellation and recovery preserve the other worker's identity; stale-base submission/landing requires a fresh checked candidate and approval. |
| 3: up to four workers | Qualify the same lifecycle at a configurable capacity up to four, including a mixed four-worker run. Finish retirement of competing controller paths. | Four real workers actually overlap, including both providers; queued work respects capacity; failure, cleanup, verification and serialized landing remain correct. No supported execution path delegates lifecycle authority to an old controller. |

Each milestone depends on the previous milestone's acceptance. A scheduler setting
of four or four queued tasks is not evidence of four simultaneous coding workers.

## User Stories

1. As an owner, I want one documented Harp execution interface, so that I do not choose between competing controllers.
2. As an owner, I want Git-managed task intent and acceptance criteria, so that execution state does not become another editable backlog.
3. As an operator, I want to start a project-scoped runtime explicitly, so that work does not require an installed always-on service.
4. As an operator, I want durable state retained after stopping the runtime, so that later invocations can inspect and recover work.
5. As an operator, I want missing execution prerequisites reported before dispatch, so that failed setup does not produce a misleading worker attempt.
6. As an owner, I want to select Codex or Trae for a task, so that both providers use the same lifecycle guarantees.
7. As an owner, I want the admitted task revision, base and execution contract pinned, so that later edits cannot silently change active work.
8. As a worker, I want an isolated workspace and bounded task inputs, so that I can produce a candidate without changing the primary checkout or another worker's work.
9. As an operator, I want duplicate submission to recover the same admitted identity, so that a lost response does not create duplicate work.
10. As an operator, I want current progress, blocked reasons and durable logs, so that I can distinguish useful progress from a merely live process.
11. As an operator, I want completed workflow steps replayed from accepted results, so that restart does not repeat completed work.
12. As an operator, I want interrupted ambiguous effects shown as uncertain, so that recovery does not silently repeat coding or external mutations.
13. As an operator, I want cleanup acknowledgment separate from effect resolution, so that stopping a process is not mistaken for undoing its effects.
14. As an operator, I want cancellation to stop the owned workload and descendants, so that cancelled work cannot keep mutating a workspace.
15. As an operator, I want capacity released only after cleanup is established, so that a paused or lost worker cannot cause overlapping execution.
16. As an operator, I want stale attempts and receipts rejected, so that a late result cannot overwrite the current attempt's result.
17. As an owner, I want an immutable candidate identified by its base, commit and tree, so that verification and approval refer to the same source.
18. As a verifier, I want checks run in an independent materialization of that candidate, so that a worker cannot substitute different bytes during verification.
19. As an owner, I want to inspect the candidate, check results and evidence before approval, so that successful execution is not automatically accepted work.
20. As an owner, I want approval bound to the exact verified candidate, so that neither a worker nor a lead can grant itself landing authority.
21. As an owner, I want landing to preserve dirty or advanced primary state, so that another contribution is never overwritten to make a landing succeed.
22. As an owner, I want a successful local landing to synchronize the destination branch and checkout, so that a moved ref is not misreported as applied work.
23. As an operator, I want two and then four workers to share the same admission and recovery rules, so that increasing concurrency does not create new authorities.
24. As an owner, I want candidate landings serialized against an expected base, so that concurrent proposals cannot silently overwrite each other.
25. As a maintainer, I want old callers migrated or explicitly retired, so that obsolete controllers do not remain hidden fallback execution paths.
26. As a maintainer, I want normal builds and gates independent of Temporal and PostgreSQL, so that the selected local product has no dependency on the abandoned stack.
27. As a reader and proof author, I want math, proofs, SICP, education and PDF publication preserved, so that runtime consolidation does not retire those useful parts of Harp.

## Implementation Decisions

### Ownership and interfaces

- The Corenius-adapted Python durable engine is the foundation. Use its existing
  safety improvements rather than replacing it with the older reference archive
  or immediately porting it to Rust.
- The Harp CLI prepares immutable task/context inputs, submits commands, displays
  runtime projections and records explicit operator decisions. It must not launch,
  cancel or recover the same worker through a second scheduler.
- The durable runtime owns admission, capacity, workflow progression, attempt
  lifecycle, cancellation, recovery and authoritative result acceptance. Candidate,
  verification, approval and landing become operations under that same authority.
- Insula admission and activity guardians remain execution mechanisms, not
  competing task schedulers. Reuse the existing guarded launch and receipt protocol.
- Adapt provider transports behind one worker-activity contract. A provider's
  thread or turn is an execution binding, not the task or workflow identity.
- Both provider adapters must identify the configured executable and supported
  capabilities, preserve bounded protocol/output evidence and refuse incompatible
  setup. Do not silently switch providers or grant broader permissions on recovery.
- The runtime must be standalone within Harp; another checkout, private source
  archive or historical session is not a runtime/build/test prerequisite.

### Durable state and recovery

- Persist immutable task input identities, workflow version, execution contract,
  operation identity, provider binding, attempts, capacity ownership and outcomes.
  Admission does not silently follow a moving task file or branch.
- Preserve operation-order/specification validation, stable idempotency keys,
  persisted retry budgets, timers and signals. Workflow replay reconstructs control
  flow from accepted operation results; external effects belong in activities.
- Preserve durable claim/release intent before execution, guardian ownership,
  cleanup evidence and current-attempt acceptance fencing. An output file alone
  is never authoritative completion.
- Recover an already durable valid receipt without repeating the activity. A
  started activity with an ambiguous outcome becomes uncertain and requires
  explicit reconciliation. Automatic retry requires an operation-specific safety
  contract or destination-enforced deduplication.
- Runtime death may terminate the active worker through its guardian. The
  milestone does not promise automatic continuation of an interrupted provider
  conversation. Completed steps remain reusable; interrupted coding effects must
  be inspected before a new attempt is authorized.
- Cleanup acknowledgment and effect resolution record distinct decisions and
  launch no work by themselves. Unproved cleanup blocks further dispatch. Proven
  cleanup does not prove that filesystem or external effects never happened.
- Use supported local SQLite/filesystem semantics and the existing version gate.
  Keep authoritative state, credentials and receipts outside worker-writable
  mounts. Large logs/artifacts remain files referenced by bounded metadata.

### Isolation and capacity

- Use the existing admitted Insula boundary for coding workers. Qualify the
  selected ownership backend on the target host; fail admission when its required
  capability is missing rather than silently weakening cleanup guarantees.
- Process-group ownership remains a trusted, nonescaping-activity mode. It cannot
  establish containment or descendant cleanup for arbitrary coding processes.
  Use the existing delegated cgroup backend for qualification that makes that
  stronger claim, with membership established before release and kernel-observed
  cleanup before capacity reuse.
- Capacity belongs to one project-scoped runtime/state directory. Qualify limits
  of one, two and four. Queued work, cancelled work and unresolved cleanup must
  not cause actual live activity count to exceed the admitted limit.
- CPU/GPU slot accounting is logical admission. This spec makes no new claim of
  enforced CPU quotas, GPU memory isolation or cluster-wide scheduling.

### Candidate verification, approval and landing

- Capture the task/input identity, expected base, candidate commit/tree and relevant
  execution contract before verification. Protect accepted candidate objects from
  later workspace mutation.
- Run operator-configured checks against an independently materialized exact
  candidate. Bind outcome, command, environment identity and logs to that subject.
  A changed candidate or required input invalidates the old verification.
- Human approval is a separately recorded decision for the exact verified subject.
  Worker/lead roles and passing checks cannot manufacture it. Keep approval and
  landing capabilities outside model-controlled processes and writable state.
- Landing requires current approval, matching evidence and the expected destination
  base. Serialize landing mutations and reconcile a lost acknowledgment before
  retrying a potentially completed mutation.
- A successful checked-out destination landing must leave its branch, index and
  tracked worktree at the accepted revision. Ref-only mutation is insufficient.
  Dirty, substituted or advanced destination state is preserved and reported.
- A stale candidate needs a new integration candidate, verification and approval;
  neither the old approval nor worker success authorizes silent rebasing/landing.

### Supersession and migration

- Inventory supported callers and public entry points before changing ownership.
  For each, record migration to the selected runtime or explicit retirement with
  an actionable replacement. Preserve useful contracts without retaining an old
  scheduler underneath a compatibility wrapper.
- Milestone one must have a complete sole-owner worker path. Later milestones
  finish caller migration and retire competing controller paths. By the four-worker
  acceptance point, the selected runtime is the only supported execution authority.
- Remove the PostgreSQL/Temporal implementation and its service/dependency
  requirements from active product workflows and normal builds/gates. Preserve
  recoverable source history and license/evidence records, with explicit import
  dispositions rather than rewriting captured artifacts.
- Keep Git-managed Markdown as work intent. Historical Kata integrations and
  alternative ledgers do not become dependencies of the new path.
- Preserve mathematics, proof sources, educational content and publication
  mechanisms. Physical study relocation and Cargo/toolchain unification are
  separate work, not prerequisites for this runtime integration.

## Testing Decisions

The primary acceptance seam is the public Harp CLI, driving real disposable Git
repositories, local durable state and owned Linux processes. Test observable
identity, source bytes, process cleanup, receipts and destination state, rather
than private helper layout or exact internal call order. Reuse existing durable,
guardian/cgroup, provider, Git-candidate and CLI test facilities where sufficient.

- **Admission:** duplicate/conflicting inputs, pinned task/base/version/provider,
  capacity limits and missing prerequisites. Assert that refused admission creates
  no live worker or misleading successful attempt.
- **Provider parity:** both Codex and Trae must pass the same task/result/cancel
  boundary contracts. Deterministic provider fixtures exercise failure windows;
  real provider runs establish live integration separately.
- **Crash windows:** before release, after release, after a durable receipt and
  before authoritative commit. Establish that accepted work is not repeated,
  old work cannot overlap replacement ownership, and ambiguous effects remain
  visible pending reconciliation.
- **Cancellation/cleanup:** include descendants and a missing/stopped guardian.
  Inspect actual process/cgroup state; a cancellation acknowledgment or expired
  lease alone is insufficient. Missing delegated capability is a reported blocked
  precondition for required cgroup acceptance, not a passing skip or fallback.
- **Fencing:** stale results, changed operation specifications and incompatible
  recovery inputs cannot become authoritative.
- **Git:** independently verify exact candidate bytes; reject stale approval,
  changed source, dirty/advanced primary state and conflicting landing. Exercise
  uncertain landing recovery and inspect branch, index and worktree after success.
- **One-worker acceptance:** complete a real coding task with Codex and another
  with Trae under the single-worker limit, each with retained verification,
  explicit human approval and local landing evidence.
- **Two/four-worker acceptance:** demonstrate actual mixed-provider overlap at
  each limit, queued admission, one-worker failure/cancellation without identity
  confusion, and serialized exact-candidate landing. Use a four-worker mixed run
  including both providers; do not infer this result from small scheduler fixtures.
- **Migration:** selected supported entry points reach the sole owner; retired
  entry points explain their replacement. Normal builds/checks do not require
  PostgreSQL or Temporal. Relevant existing educational/proof/publication contracts
  remain protected when migration touches their consumers.

Record the exact source, provider versions/configuration, runtime/SQLite version,
backend/capability evidence, commands, outcomes and limitations at each milestone.
Historical tests and reference-archive results are prior art, not fresh acceptance.
Live provider runs and setup are explicit implementation-stage operations, not
authorized by publishing this specification.

## Out of Scope

- Temporal services, Temporal SDK compatibility, PostgreSQL and DBOS adoption.
- A new Rust durable-engine implementation or a second workflow scheduler.
- Terminal UI integration, macOS worker execution and a remote-control UI.
- Installed always-on services, fleet scheduling, distributed databases or more
  than four workers in the qualified project runtime.
- Automatic live provider-session resumption or arbitrary exactly-once external
  effects, host-reboot/power-loss guarantees and automatic backup/state migration.
- Full feature parity with every historical experimental controller or automatic
  import of its live databases. Supported caller disposition must still be explicit.
- GPU/training checkpoint recovery, new resource-isolation claims, study directory
  moves, repository-wide licensing changes, pushes or external publication.

## Further Notes

The owner confirmed the foundation, dual-provider requirement, one/two/four
sequence, CLI-first Linux target, safe recovery with explicit reconciliation,
public CLI acceptance seam and explicit project-scoped lifetime. This spec is
ready for implementation planning; it does not report implementation completion.

The supplied reference archive matches Corenius's recorded source hash. The
Corenius adaptation adds important uncertainty, input-validation, Insula and
cgroup improvements. The separate Collab Lite Rust archive mentioned in the
historical provenance was not supplied or independently verified; this spec
does not depend on its claimed tests or delivery.
