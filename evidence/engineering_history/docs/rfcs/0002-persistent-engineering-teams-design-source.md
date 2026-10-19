# Corenius: Durable Teams, Replaceable Agents

**Design RFC v0.1 — proposed, not implementation-verified**  
**Date:** October 2, 2026, America/Los_Angeles  
**Scope:** An expert-operated control plane for long-horizon software engineering and ML/systems research.

## Executive decision

Build Corenius as a durable team authority and reconciliation layer over existing agent runtimes and execution substrates. Its product is a persistent, inspectable engineering effort—not a persistent conversation and not a new agent loop.

The lead exercises delegated engineering judgment. The broker admits and records decisions, enforces permissions and budgets, and determines whether results satisfy an authorized contract. Temporal makes coordination recoverable. Native runtimes perform reasoning and tool use. Armada/Insula supervises independently addressable execution. Protected verifiers establish evidence; a separate integration gate controls consequential promotion.

**Design target:** Replacing a lead, losing a terminal, restarting a broker, or reconnecting to a GPU environment must not erase the objective, reset the budget, silently duplicate work, resurrect revoked permissions, or change which candidate was verified. This target is limited to the declared storage, runtime, and trust guarantees; it does not assume an honest or reachable arbitrary host.

The proposed differentiator is the combination of durable work ownership, bounded adaptive plans, native runtime interoperability, exact-result verification, and an interface organized around the operator's decisions rather than around agent chatter.

## 1. Consolidation and decision precedence

This RFC consolidates the current Corenius brief, the earlier Harp Durable Execution proposal [P1], the Insula execution-contract specification [P2], the later Armada product-surface specification [P3], and the runtime and durable-workflow requirements discussed alongside them. The older documents are design inputs, not evidence of a currently implemented Corenius system. Accessible repository searches did not establish a Corenius implementation to audit.

| Topic | Decision for this RFC |
|---|---|
| Product boundary | A durable team control plane, not a replacement for Codex, Claude, TraeCLI, or Armada. |
| Persistence | PostgreSQL is the approved authority store. HDE's earlier local-SQLite suggestion is superseded here. |
| Coordination | Temporal is an approved baseline dependency, not an optional future backend. |
| Agent roles | Lead, implementing worker, and verifier are roles. Default providers can differ; roles are not defined by provider names. |
| Execution | Reuse supervised execution and the immutable Execution Contract seam; do not create another SSH/remote-launch path. |
| Acceptance | Immutable candidate, applicable evidence, current authorization, and conditional integration. |
| Isolation | Kata Containers is optional and must qualify on the concrete target. |
| Version control | Git is sufficient for the first implementation. jj is optional behind a workspace adapter. |
| Historical naming | HDE references to Kata as a task tracker are distinct from Kata Containers. Neither should create a second execution authority. |
| Status | New recommendations below require design review. No performance, safety proof, or runtime conformance is claimed achieved. |

The current Corenius brief takes precedence over older proposals. The September 17 Armada specification supersedes older CLI examples where they differ. Integration should target a versioned execution interface, not copy historical command spellings into the broker.

## 2. What an excellent product should feel like

A user should be able to give a mission a goal, repository/resource scope, acceptance criteria, autonomy envelope, and budget. Corenius should produce a reviewable task decomposition, obtain any required approvals, perform independently verifiable work, and surface only decisions that actually need the operator.

The operator should not need to remember which terminal owns a task, whether an agent is still alive, which commit a green test belongs to, or whether reconnecting will launch a second training run. Those facts should be available without invoking a model.

The six user-visible promises are:

1. **Continuity of work:** The mission and its jobs survive loss of particular conversations.
2. **Legible autonomy:** It is clear what may happen automatically and what requires another grant.
3. **Preserved investment:** Existing useful jobs, checkpoints, findings, and negative results are adopted rather than casually repeated.
4. **Trustworthy completion:** “Accepted” identifies a contract, candidate, evidence set, and any required promotion receipt.
5. **Low incidental attention:** Waiting and bookkeeping do not repeatedly consume model calls or human attention.
6. **Inspectability:** Every important state has an explanation, source, freshness, and available next action.

These are design criteria, not achieved service-level promises.

## 3. Architecture: one logical authority, several replaceable mechanisms

```text
Human / CLI / TUI / editor client
                  |
          Authenticated command API
                  |
       Corenius broker / domain kernel
       intent · grants · decisions · budgets
       operations · acceptance · reconciliation
            |                  |
       PostgreSQL          Artifact store
            |
       Transactional outbox
            |
       Temporal coordination workers
            |
       Authenticated broker operations
            |
   +--------+-------------------+------------------+
   |                            |                  |
Runtime gateway          Execution gateway     Verification / integration
Codex / Claude / Trae     Armada / Insula       Trusted check wrappers
threads · turns · goals  workspaces · jobs     Protected promotion target
   |                            |                  |
   +--------- attributed observations / receipts -+
```

These are logical boundaries, not a demand for separate services. Start with a modular broker, a Temporal worker process, PostgreSQL, an artifact store, and the runtime/environment gateways that must run near execution. Do not require Redis, Kafka, a graph database, a new cluster scheduler, or a microservice per entity.

For a workload expected to progress while the laptop sleeps, host the required broker/coordination services on an appropriately persistent machine. The workstation is then an attached control surface, not the hidden owner of the mission.

### 3.1 What each component owns

| Component | Owns | Does not own |
|---|---|---|
| Lead | Engineering proposals and decisions within its delegated envelope | Root policy, budget creation, or self-approval of results |
| Broker + PostgreSQL | Authorized intent, grants, current domain state, operation identities, acceptance records | Omniscient knowledge of a disconnected host |
| Temporal | Recorded coordination progress, waits, retry schedules, workflow history | Independent permission to execute or a second editable task truth |
| Runtime gateway | Native lifecycle translation and attributed observations | Inferring authority from a thread's existence |
| Armada/Insula | Admitted physical execution, resource observations, supervision, environment evidence | Mission decomposition or cross-run research policy |
| Artifact store | Immutable bytes, integrity metadata, manifests | Proof that stored claims are true |
| Verifier | Evidence under a declared recipe, subject, environment, and trust boundary | Permission to weaken acceptance or publish |
| Integration gate | Conditional promotion of a specific authorized candidate | Creating new engineering objectives |
| UI | Rebuildable views and authenticated commands | An independent writable copy of execution state |

An approved contract may be stored as immutable artifact bytes. Its authorization and active applicability are broker records. “Artifacts are not authority” must not be misread as “approved contract bytes are irrelevant”; the distinction is between storing content and granting that content operational force.

## 4. Domain model: separate lifetimes before adding features

The most important refinement to an agent-centric model is that a long-running job must not be owned by a disposable conversation.

```text
Project
  Mission: goal, accepted constraints, overall allocation
    Task: durable outcome identity
      TaskRevision: immutable outcome and acceptance contract
      PlanRevision: approved strategy within that contract
      Scope: durable authority and execution ownership
        Attempt A1 -> runtime binding R1 -> native thread/turn
        Attempt A2 -> runtime binding R2 -> replacement participant
        Execution E1 -> externally supervised job
        Workspace W, generation 4
        Candidates -> evidence -> acceptance / promotion
```

`Execution.created_by_attempt` records provenance. `Execution.owned_by_scope` determines its lifetime. Replacing A1 does not automatically kill E1. Cancelling the scope can revoke further work and request E1's termination according to the admitted policy.

### 4.1 Essential records

| Record | Purpose |
|---|---|
| Project / Mission | Authority domain, goal, policy, resources, and aggregate allocation |
| Task / TaskRevision | Stable objective and immutable admitted acceptance definition |
| PlanRevision | Strategy, dependencies, bounded branches, joins, and decision points |
| Scope | Durable ownership, allowed operation classes, mode, generation, and budget |
| RoleAssignment | Who currently occupies a lead/worker/verifier role and its role epoch |
| Attempt | One admitted participation episode, context manifest, outcome, and provenance |
| RuntimeBinding | Provider-native thread/session/turn identity and runtime incarnation |
| Execution | Independently supervised command, service, experiment, or verification job |
| WorkspaceGeneration | One mutable writer-owned incarnation and its recovery source |
| Operation | One logical admitted effect, immutable payload, deduplication key, and uncertainty |
| Candidate | Immutable deliverable manifest and input/provenance references |
| Evidence | Subject-bound result, verifier identity, recipe, environment, coverage, limitations |
| Promotion | Candidate, target, expected predecessor, evidence, grant, and effect receipt |
| Decision / Finding / Claim | Explicit reasoning outputs, supersession, assumptions, and supporting evidence |
| Reservation / Usage | Allocations, outstanding commitments, settlement, and accounting uncertainty |

These need not be separate deployment units. Versioned schemas and foreign keys are preferable to a large universal object with opaque JSON and ambiguous lifecycle semantics.

### 4.2 Keep independent versions independent

- `task_revision` identifies the authorized outcome and acceptance criteria.
- `plan_revision` identifies the chosen strategy.
- `role_epoch` fences obsolete occupants of a role.
- `scope_epoch` fences obsolete execution authority.
- `workspace_generation` separates mutable writer incarnations.
- `runtime_incarnation` distinguishes physical runtime instances.
- `authority_domain` changes during disaster recovery when an old authority store may have been rolled back.

A new lead normally changes the lead's role epoch, not every task scope epoch. Grants for valid child work are rooted in durable scopes rather than in the continued existence of the lead process.

### 4.3 Orthogonal state, not an enormous task status enum

Represent task disposition, execution authorization, attempt phase, observed runtime condition, verification, promotion, and cleanup separately. For example, all of the following can be true simultaneously:

```text
Task: awaiting acceptance
Scope: paused for new ordinary work
Attempt: disconnected
Execution: healthy, retained by pause policy
Candidate: sealed
Verification: running
Cleanup: not yet due
```

`UNKNOWN` is a real observation state. It is not an alias for absent, dead, failed, free, cancelled, or safe to repeat. A successful workload followed by failed cleanup preserves both facts, following the Insula conformance model [P2].

## 5. A small authority kernel

The broker should expose a small family of domain operations: admit, inspect, observe, reconcile, checkpoint, seal, verify, and promote. Its protected decisions should be deterministic relative to persisted state and pinned policy. An LLM may provide a review or an engineering choice; it cannot be the sole implementation of permission checks.

A proposed command envelope contains:

```yaml
schema: corenius.command/v1
principal: authenticated-runtime-or-human
scope_id: scope-17
expected_scope_epoch: 4
expected_entity_revision: 29
idempotency_key: caller-stable-operation-key
payload_digest: canonical-payload-digest
command: execution.submit
payload: structured-and-schema-validated
```

Role-sensitive commands also validate the current role assignment. The response includes a durable operation ID, resulting revision, audit reference, and actual status. An asynchronous admission returns ADMITTED, not a fictional “job started.”

The same idempotency key with the same canonical payload returns the existing logical operation. The same key with different payload bytes is a conflict. An operation ID alone is not exactly-once execution: the receiver must deduplicate, support a conditional write, or expose a discoverable receipt [R1].

### 5.1 Permission and trust boundaries

Grants constrain actions, target predicates, data classes, write/conflict domains, delegation, time, and resource limits. Delegation narrows permissions and transfers an allocation; it does not create new authority or money.

Repository text, messages, tool output, model summaries, and modified skill files remain task data or proposals. They cannot enlarge a grant. A worker modifying its own test policy does not change the accepted verification contract.

Private file edits can occur directly inside an already admitted workspace. Do not turn every write system call into a PostgreSQL transaction. Instead, enforce the environment boundary and mediate protected effects: privileged resource creation, cross-scope mutation, provider usage where required, external communication, integration, publication, and access to protected credentials.

The strict profile withholds broker credentials, protected repository metadata, unrelated home directories, host control sockets, unrestricted cloud credentials, and verifier signing material. Where those paths cannot be controlled, the run is explicitly enrolled at a weaker assurance level.

## 6. PostgreSQL and Temporal: one authority, durable coordination

### 6.1 Admission and dispatch

The authoritative state-changing transaction:

1. Authenticates the principal and checks the current role/scope revisions.
2. Validates the grant, dependencies, resource ownership, and immutable payload.
3. Reserves the required budget/capacity.
4. Creates or deduplicates the operation.
5. Updates domain state and appends an audit event.
6. Inserts an outbox entry.
7. Commits before any external effect is dispatched.

Use database uniqueness and appropriate row locking for local invariants; where serializable transactions are selected, retry the entire transaction after a serialization failure rather than retrying an already executed external effect [R3]. No database transaction remains open while awaiting an agent, SSH connection, GPU job, or Git mutation.

An outbox relay starts or wakes the appropriate Temporal workflow. Delivery can repeat. Stable workflow/operation identities, inbox deduplication, and reconciliation absorb repetition.

### 6.2 Workflow discipline

Temporal workflow code coordinates deterministic control flow. Database/network/model calls and external effects occur through Activities; an Activity may return immutable state or invoke an authenticated broker operation. A previously recorded workflow result never becomes permission to bypass a fresh admission check [R2].

Use workflows around task scopes and substantial coordination operations, not around each keystroke. Submit long jobs through short Activities, persist their external execution IDs, then wait for observations or timers. A training run does not need a model session or an hours-long polling Activity merely to keep its identity alive.

A scope workflow can maintain an observed scope revision and a wake revision. Every committed scope change writes its revision into an outbox event. On a wake or timer, a read Activity refreshes the relevant state. If a newer wake arrived during the read, the workflow refreshes again before parking. Periodic reconciliation is a safety net for delayed notifications. A wakeup is a hint to inspect state, not an authorization message.

Pin workflow implementations and support replay-compatible deployment. Bound histories with an explicit history-management strategy, including Continue-As-New where appropriate [R4]. Old workflow generations must still encounter current broker authorization when requesting an effect.

### 6.3 Persistence boundaries

PostgreSQL stores current domain tables, audit events, inbox/outbox records, and artifact references. Large logs, token streams, datasets, and checkpoint payloads remain in the artifact/data store, not in workflow history.

Complete and verify immutable object writes before acknowledging durable metadata references. A failed metadata commit may leave a collectible orphan blob; a successful reference must not intentionally refer to unpersisted bytes. Accepted candidates, evidence, active checkpoints, and unresolved operations pin their dependencies.

There is no assumed atomic transaction across PostgreSQL, Temporal, Git, a model provider, and remote execution. Each boundary has a journaled operation and an explicit uncertainty policy.

## 7. Dynamic plans without a self-modifying control plane

### 7.1 Recommended approach

Use a trusted, versioned plan interpreter over typed operations, with bounded decision points for agent judgment. This combines adaptability with inspectable execution.

Three options have different tradeoffs:

| Approach | Strength | Main limitation |
|---|---|---|
| Free-form agent coordination | Flexible, minimal initial scaffolding | Reconstructing sequencing, authority, joins, and retries from chat is unreliable |
| Arbitrary generated workflow code in trusted workers | Expressive programmable orchestration | Makes generated code part of the control-plane trust and deployment surface |
| Typed plans plus bounded judgment | Explicit effects and waits; adaptable strategies | Requires a carefully limited schema and reviewed extension path |

Recommend the third as the ordinary path. Avoid building a giant universal DSL: begin with the small repertoire required by actual engineering and experiment tasks.

A plan step can invoke an admitted attempt, execute a pinned recipe, await a durable condition, verify a candidate, request a decision, join child outcomes, or propose promotion. Branches and repetition have explicit budgets and exit conditions.

### 7.2 Separate objective, strategy, and decision

A TaskRevision freezes what counts as success. A PlanRevision states how to pursue it. A DecisionRecord names the question, admissible alternatives, evidence considered, selected action, deciding principal, and authority envelope.

A lead can choose a different implementation strategy inside its envelope. Weakening acceptance, increasing spend, changing external effects, or accessing new data requires a newly authorized revision. “The tests are difficult, so remove them” is not an ordinary plan repair.

Dynamic expansion is a new admitted graph revision. Completed history is not rewritten. Joins consume specific child outcomes/artifact identities, not a model's belief that “everyone seems done.” Interface and schema dependencies are explicit conflict domains even where source paths do not overlap.

### 7.3 Agent-authored Temporal workflows

Retain this as a valuable advanced path: an agent authors a workflow bundle, tests it, and submits it as a candidate. Promote its exact code/dependencies after review, replay checks, capability checks, and appropriate security evaluation. It then becomes a registered workflow recipe.

Do not evaluate arbitrary generated Python inside the privileged worker. Temporal's Python workflow sandbox is a determinism helper, not a security boundary [R2]. A research workflow may be flexible and code-authored without being allowed to modify the broker's active policy.

### 7.4 Relation to runtime durability

Pi Durable's advertised persistence operates within an agent/task harness, including model/tool/compaction boundaries. Its documented interrupted-operation behavior still requires attention to retries and tool safety [R5]. Treat such a runtime as a potentially useful adapter beneath Corenius. It does not replace the need for durable team authority, resource accounting, independent jobs, or protected promotion. Do not introduce two owners racing to advance the same participant.

## 8. Native runtime integration, not a new agent loop

Keep the native runtime responsible for inference, native tool execution, conversation management, and provider-specific features. The adapter maps Corenius semantics to tested native operations and preserves native IDs for inspection.

A runtime capability report should describe:

- session/thread inspection and historical reads;
- start, steer, interrupt, and native goal behavior;
- lookup after an uncertain start;
- active-job enumeration and adoption;
- event sequencing, gaps, and reconnect behavior;
- checkpoint/cold-start/warm-resume support;
- usage accounting and actual enforceable limits;
- native descendant visibility and registration;
- isolation and protected-effect mediation.

Capabilities are qualified against a pinned runtime, endpoint, and environment. Codex support does not establish TraeCLI or Claude parity. No native identifier is assumed idempotent unless tested as such.

Codex's current app-server exposes thread/turn/item concepts, distinguishes reading stored threads from resuming them, and provides turn steering. Its documentation also states that changing a native goal objective resets that goal's usage accounting [R6]. Consequently, Corenius should keep native IDs as bindings and maintain scope-level budgets independently.

Every attempt chooses one **continuation owner**: broker-managed advancement or a native goal loop operating inside a broker-admitted envelope. The broker must not issue a second turn merely because a native goal is already autonomously advancing. Goal replacement is not a new budget.

Use integration levels that say what is actually controlled: observation-only, supervised execution, and managed protected effects. A wrapper that launches a CLI is not automatically exact provider-call accounting. Native subagents either register and consume the same conserved allocation, route through the broker, or lower the declared accounting/control guarantee.

The runtime UI is an attached view. Human steering that changes task execution should become an authenticated broker command. Direct legacy terminal mutation may be supported as explicitly weaker/manual control, but terminal scraping and send-keys are not the core protocol.

## 9. Execution: compose over Armada and Insula

Keep target identity, environment identity, workspace identity, and job identity distinct. Target kind describes a resource family; a fresh capability observation determines what a concrete target can actually support.

Reuse the Insula pattern: a typed workload profile and recipe compile into an immutable Execution Contract. Generic execution code then owns placement, construction, launch, supervision, observation, stop, cleanup, and evidence capture [P2]. Corenius supplies the admitted scope, operation identity, resource envelope, and higher-level coordination. It must not bypass the existing substrate with another SSH launcher.

Corenius can queue, fan out, retry over time, and coordinate an experiment matrix. An Armada profile remains a single-execution adapter rather than a second campaign scheduler [P3]. Placement can reject stale resource assumptions; a previously selected target is not permission to bypass fresh allocation checks.

Long-running training, serving, builds, and verification jobs have application-level progress contracts: startup deadline, maximum admitted runtime, telemetry expectations, checkpoint consistency, stop mechanism, partition policy, and recovery capability. Heartbeats indicate liveness; they do not prove useful progress or a correct result.

For GPU work, allow the workload and approved diagnostics to share the required visibility/PID namespace while remaining separately attributed and budgeted. Introspection privileges such as ptrace or eBPF are explicit target-dependent capabilities, not assumed available inside an arbitrary existing container.

Kata Containers remains an optional environment backend, qualified for the target's actual isolation, accelerator, filesystem, networking, debugging, and cleanup requirements [R10]. If required isolation is unavailable, strict execution is blocked; no automatic unsandboxed fallback is permitted.

## 10. Candidates, verification, and exact landing

### 10.1 Candidate and evidence identities

A candidate manifest names immutable deliverables and relevant inputs: exact code revision/tree, artifacts, parent/base identities, TaskRevision, producer, environment or toolchain dependencies where relevant, and content digests. A branch name or mutable workspace path is never the evidence subject.

Evidence binds candidate digest, acceptance-contract digest, verifier recipe, actual execution environment, collected checks, outcome, coverage, skips, limitations, and retained raw artifacts. The trusted wrapper records what actually ran. Candidate code cannot mint the trusted receipt or access its signing material.

A model reviewer is useful but not equivalent to a trusted test runner and not statistically independent merely because it is a different conversation. Independence first means separate permissions, immutable subject, protected acceptance policy, and an appropriate checking environment [P1].

### 10.2 The integration protocol

1. Resolve the protected target at predecessor B.
2. Construct and seal the exact integration candidate C based on B.
3. Run the required checks against C, not an earlier feature-branch artifact.
4. Compute eligibility from current authority, applicable evidence, acceptance criteria, and target preconditions.
5. Record a prepared promotion identifying operation O, B, C, evidence, and the bounded grant.
6. Serialize commit admission against cancellation and other conflicting promotions.
7. Have the exclusive integration gateway conditionally advance B to C, after verifying the fast-forward relationship.
8. Persist the effect receipt and reconcile any ambiguous acknowledgment before another conflicting promotion.

Git's expected-old-value update provides a useful compare-and-set primitive for the ref [R7]. It does not make Git and PostgreSQL one transaction or automatically synchronize a checked-out worktree.

If the target moved, do not merge, rebase, or cherry-pick after verification and reuse the old green status. Construct a new integration candidate and obtain applicable evidence. Start with exact-subject matching; dependency-aware reuse comes later, after its invalidation rules have been tested.

### 10.3 Protected repository and crash boundary

Prefer a gate-owned bare integration repository. Ordinary workers use repositories/workspaces without write access to its common Git metadata. Git worktrees share repository state, so a separate worktree alone is not a security boundary [R8]. Synchronization into the user's editable checkout is a separate operation that refuses dirty-state or untracked-path collisions rather than resetting files.

The gate serializes unresolved mutations for each protected target and retains durable operation intent and attributed receipts. After a crash, inspect the actual target and gate history. A match to C establishes successful application only when the sole-writer/operation history is sufficient to attribute that transition; otherwise remain RECONCILING. Never claim a general atomic multi-resource commit.

Local acceptance, local landing, remote push, publication, deployment, and deletion are distinct effect classes. A completed local landing does not imply permission to push or publish.

### 10.4 jj remains optional

A jj adapter may improve editing and change navigation. Its stable change ID can identify evolving work while the corresponding commit changes [R9]. Therefore acceptance still binds an immutable revision/manifest, not an unqualified logical change ID. Candidate semantics must remain independent of workspace tooling.

## 11. Recovery and cancellation are normal product paths

Reconciliation compares three separately labeled things: currently authorized intent, recorded operation history, and fresh physical observations. An observation includes its source, runtime incarnation, and observation time.

| Situation | Required behavior |
|---|---|
| Lead disappears; workers/jobs healthy | Replace the lead role and reconstruct its briefing; do not restart valid child work. |
| Start response lost | Discover by durable operation identity or remain UNKNOWN; do not issue a blind new launch. |
| Worker is partitioned | Preserve uncertainty and outstanding reservations; apply its admitted partition policy. |
| Old lead returns | Retain historical output; reject role-sensitive commands under the old role epoch. |
| Old writer might still be alive | Quarantine its writable generation; a replacement gets an isolated generation. |
| Cancellation races with an effect | Serialize the authority decision and enumerate effects already commit-granted. |
| Candidate changes after verification | Keep old evidence attached to the old subject; require new applicability checks. |
| Git mutation may have succeeded | Reconcile the exact operation and block conflicting landing until resolved. |
| Cancelled work produces useful output | Bounded receive-only quarantine; new authority and independent verification before use. |
| Database restored from backup | Admission lockout, fresh external authority epoch/credentials, reconcile surviving execution. |

“Never blind-spawn under uncertainty” is not “block all progress forever.” An operator or previously admitted policy may authorize isolated replacement/speculative work with a new workspace, separate reservation, and non-overlapping protected effects. The old possible execution remains outstanding and accounted for. Isolation and explicit risk treatment are the difference.

### 11.1 Human control vocabulary

**Pause:** stop admitting new ordinary work; retain or park existing jobs according to the displayed policy.

**Quiesce:** drain to a declared consistency boundary for checkpoint or transfer.

**Cancel:** revoke future protected admission and request termination. Return both the revocation receipt and per-job stop status.

**Take over:** replace a controlling role/attempt while retaining durable work that remains authorized.

Resource-side fencing is necessary for strong stale-writer exclusion; lease expiration alone does not stop a paused process from resuming writes [R11]. For a mounted mutable filesystem without effective fencing, isolation into a new generation is the safe handoff default.

An effect granted before cancellation may physically complete afterward. Corenius must identify this rather than promise that cancellation undoes an irreversible effect. Likewise, a remote timeout cannot prove a process stopped.

### 11.2 Disaster recovery

Storage durability and authority freshness are separate. A restored database can contain obsolete grants and missing completed effects. Recovery therefore closes admission, disables old effect paths, establishes an independently controlled authority domain or credential epoch, inventories surviving jobs and promotions, and explicitly re-admits retained work. Recovery stays blocked when old privileged paths cannot be fenced.

Declare whether acknowledged state is local-durable, independently replicated, or exported with a tested restore path. Do not describe surviving a process crash as surviving disk loss.

## 12. Context compiler and durable team knowledge

A replacement should receive a machine-derived recovery manifest and a concise rendered briefing—not an unstructured transcript dump.

The manifest includes a consistent state cutoff and source references for the goal, non-goals, active contract, current scope, role, task graph, approved decisions, open questions, workspace/checkpoint identity, candidates/evidence, running jobs, observation freshness, pending uncertain operations, and settled/reserved budget.

Separate three classes:

- **Authoritative context:** admitted intent, approved decisions, exact acceptance criteria, current machine-derived state, and a non-reusable summary of permissions.
- **Working context:** relevant code, diffs, findings, experiments, hypotheses, and proposed next steps.
- **Historical context:** raw transcripts, old approaches, and related records retrieved when needed.

Each item has provenance, applicability, classification, and supersession. An operational statement such as “verification passed” links to the exact receipt. A proposed explanation remains a hypothesis. Actions revalidate current state after a briefing was compiled.

Warm continuation is allowed when it is useful and supported. Cold continuation is the portability contract. Do not make hidden reasoning, proprietary KV snapshots, or a mandatory reset interval part of correctness. The quality benchmark is safe reconstruction and continuation, not reproducing the same wording or stochastic output [P1].

Reusable lessons are reviewed artifacts, not automatically trusted summaries. Store evidence, applicable environments, failed conditions, supersedes/refutes relationships, and a reason a future attempt might differ. Negative results deserve durable memory without becoming unconditional prohibitions.

## 13. The human interface: attention first, native detail on demand

The main view should answer: What changed? What is blocked? What needs my decision? What is still running? What is the current accepted state? What uncertainty remains?

An illustrative—not live—task card:

```text
Task T17: reduce attention backward memory
Contract R3 · plan P6 · scope active
Candidate C31: sealed; numerical checks passed; performance check running
Execution E42: healthy; observation 8 seconds old; checkpoint CP9
Lead: disconnected; takeover available without restarting E42
Budget: settled 46%; reserved 18%; one usage record unresolved
Integration: not landed; target precondition B12
Human decision: none
```

Provide an attention queue, task/dependency view, execution inspector, candidate/evidence comparison, and causal timeline. Native agent TUIs remain available through attach links, but are not the only way to determine status.

An `explain` view should trace an outcome from task revision through decision, grant, operation, runtime/job, candidate, evidence, and promotion. A “blocked” explanation names the violated predicate, last observation, responsible role, and safe next actions.

Use the same stable IDs and command semantics in CLI, TUI, and editor clients. A trace explorer can support hierarchical events, Dired-like marking, evidence previews, and sending selected events into a new diagnostic attempt. These are projections and context selections—not alternate authority stores.

Bulk commands preview affected scopes, retained jobs, terminating jobs, shared targets, and irreversible in-flight effects. Live streams are resumable with source coverage and gap markers. Token streaming remains provisional until a complete tool proposal is admitted.

## 14. Scheduling, accounting, and useful autonomy

Start with an explainable scheduler: dependency readiness, scope authorization, resource availability, critical-path priority with aging, and interface/write-conflict domains. Do not infer semantic independence from disjoint files.

Reserve capacity for verification, reconciliation, and control actions so a burst of generation cannot starve checks or cancellation. Waiting should normally be an event/timer operation, not a sequence of model calls asking whether work finished. Batch nonurgent observations before waking the lead.

For one resource unit:

`available = allocation - settled_usage - outstanding_reservations`

Child allocations are transfers, not new allocations counted twice. Outstanding reservations include in-flight and uncertain operations. A disappearing conversation neither erases cost nor frees the uncertain reservation. Report provider accounting as observed, estimated, cumulative, per-call, or reconciled as appropriate.

Hard ceilings are promised only where an underlying gateway/provider/runtime can enforce a credible bound. Otherwise the UI and admission policy explicitly describe a soft budget and its uncertainty. Native goal resets, new credentials, resumed threads, and model replacement must not reset the scope ledger.

Repair loops have bounded attempts, spending, and escalation. A stall detector cites observations and can request bounded diagnostics; it does not certify failure from silence. Use workload-specific progress contracts for builds, training, proof checking, and interactive work.

Measure accepted quality, end-to-end latency, cost, human attention, verification delay, rework, duplicate-work exposure, orphaned-runtime exposure, and cold-resume fidelity separately. More agents, messages, or patches are not product success metrics.

## 15. Domain extensions and controlled improvement

Keep the authority/operation core generic; add domain-specific contracts and evidence adapters.

For software engineering, bind behavior, compatibility, protected interfaces, actual check collection, skips, and relevant static/dynamic analysis. An empty test collection is not an ordinary passing suite. Expected-red tests identify their intended failure before the result is known.

For ML and systems research, bind research question, dataset/model/code identities, baseline, metric, hardware/toolchain, seeds, trial selection, failed/censored runs, and checkpoint lineage. A successful investigation can establish a negative result or a tradeoff without supporting the original hypothesis. Trial completion and claim verification remain separate.

Long training workflows preserve application checkpoints and job identity through agent replacement. A recipe can coordinate a compile/benchmark/analysis sequence without making the agent the owner of the GPU allocation. Expensive data and model artifacts stay outside Git and are referenced by identity and access policy.

For formal or numerical work, specify the exact proposition, assumptions, checker/toolchain, certificate format, and trust profile. A passed check establishes its declared predicate, not every intended scientific interpretation.

Keep harness improvement in a separate loop. Context policies, tools, model routing, decomposition templates, and verification recipes are versioned candidates tested on frozen and held-out tasks. The evaluated attempt cannot change the active evaluator or acceptance policy. Canary and promote improvements separately; ongoing production attempts keep their pinned contract until explicit migration [P1].

## 16. Delivery strategy and qualification gates

This section is an architectural sequence, not an implementation-authorized task plan.

### Slice A: durable single-scope core

Implement one logical authority, scope and role epochs, PostgreSQL operations/outbox/reservations, Temporal wake/wait coordination, a deterministic fake runtime, and an execution registry. Build fault injection before adding many providers.

**Gate:** Duplicate commands, stale roles, cancellation races, unknown launches, and reservation conservation preserve the specified state invariants.

### Slice B: one real end-to-end engineering task

Connect a lead and one implementing runtime using the capability contract. Add an isolated workspace, sealed candidate, trusted verification wrapper, protected local integration, and the recovery briefing. No remote publication is implicit.

**Gate:** Goal to verified local landing works; killing the lead and broker at specified boundaries does not lose authoritative work or bypass verification.

### Slice C: independently durable remote jobs

Integrate the existing Armada execution seam. Retain training/build/verification jobs across agent replacement, use actual target capability observations, and qualify stop/reconnect/unknown paths.

**Gate:** A fresh lead can identify and adopt an already-running job without duplicate launch; unsupported strict environments are blocked without silent downgrade.

### Slice D: controlled parallelism and a strong operator experience

Add typed dynamic plan revisions, child allocation, join semantics, conflict domains, semantic trace selection, and the attention queue. Increase concurrency only when the single-scope protocol remains reliable.

**Gate:** Parallel work improves accepted throughput under a fixed quality/attention budget without weakening scope, accounting, or integration guarantees.

### Slice E: research contracts and measured harness improvement

Add experiment/claim schemas, richer memory promotion, runtime comparisons, and separately evaluated harness candidates. Qualify jj or Kata Containers only when the observed workflow benefit warrants their integration cost.

**Gate:** Comparisons report failures, uncertainty, cost, and operator interventions; evaluator changes cannot improve an attempt's result retroactively.

## Appendix A. Non-negotiable safety properties

These are proposed test/model-checking properties, not proved claims about an implementation.

1. An obsolete role cannot issue a new role-authorized command.
2. A revoked scope cannot receive a new protected commit grant.
3. Effects committed before revocation remain explicitly tracked until resolved.
4. A runtime replacement cannot mint another allocation or reset settled usage.
5. At most one active writer is admitted for a workspace generation; stale physical writers are isolated or effectively fenced.
6. Duplicate admission of one idempotency key cannot produce conflicting logical payloads.
7. Missing observations cannot become success, death, absence, or zero cost by default.
8. A candidate cannot inherit evidence for a different subject without an explicit, validated applicability rule.
9. A producer cannot modify its active acceptance policy or trusted receipt producer.
10. Integration requires current authority and the expected target predecessor.
11. Recovery of context or storage does not implicitly restore historical grants.
12. Retention cannot remove dependencies of accepted evidence, active checkpoints, or unresolved operations.
13. Publication and deployment remain separately authorized from internal acceptance.
14. Observation-only integration is never advertised as fully enforced execution.

Liveness remains conditional on available services/resources, valid authority, budget, and fair scheduling. The design does not promise termination or discovery for arbitrary programs or research tasks.

## Appendix B. Fault-injection and conformance suite

| ID | Fault | Required result |
|---|---|---|
| F01 | Admission response lost | Retry returns the same logical operation and reservation. |
| F02 | Idempotency key reused with another payload | Conflict, with no second effect admission. |
| F03 | Broker dies after commit before outbox dispatch | Existing operation is dispatched after recovery. |
| F04 | Temporal wake delivered repeatedly/out of order | Latest committed revision is inspected; no duplicate logical effect. |
| F05 | Runtime starts but response is lost | Discover and adopt, or UNKNOWN without blind duplicate launch. |
| F06 | Native goal and broker both could continue | Exactly one configured continuation owner advances the attempt. |
| F07 | Lead dies during a GPU/verification job | Durable job remains attributed and inspectable; replacement does not reset it. |
| F08 | Old lead returns after takeover | Old role-sensitive commands fail; useful historical output remains readable. |
| F09 | Old writer resumes after lease expiry | No access to replacement's writable generation or protected integration. |
| F10 | Cancellation races with commit admission | Serialized authority outcome; pre-granted effects appear in cancellation receipt. |
| F11 | Remote process unreachable during stop | Revocation acknowledged separately from unconfirmed termination. |
| F12 | Cancelled job returns a useful result | Bounded quarantine only; new authority needed for use. |
| F13 | Child scopes reserve concurrently | Ledger remains conserved with no duplicated parent/child allocation. |
| F14 | Goal or provider changes | Settled/reserved scope usage persists. |
| F15 | Provider emits cumulative and per-call usage | No duplicate aggregation; unknown inclusion semantics remain unknown. |
| F16 | Candidate is edited after PASS | Old receipt cannot accept new candidate. |
| F17 | Tests deleted or verifier changed by worker | Active external contract remains pinned; bypass fails. |
| F18 | Worker fabricates a PASS receipt | Not trusted as a verifier-originated result. |
| F19 | Protected target moves before landing | Conditional integration fails; no forced merge/reset. |
| F20 | Git effect succeeds before PG receipt | Gate reconciles operation attribution or blocks further conflicting effects. |
| F21 | Primary user checkout contains untracked files | Synchronization preserves them or refuses collision. |
| F22 | Required sandbox capability is missing | Strict admission blocked; no silent host execution. |
| F23 | Producer poisons build cache | Trusted verifier policy rejects untrusted provenance/cache reuse. |
| F24 | Disk full during checkpoint/artifact commit | No false durable acknowledgment; prior valid checkpoint preserved. |
| F25 | GC races unresolved operation | Referenced dependencies remain pinned. |
| F26 | Database rollback while old workers survive | Recovery lockout; fresh authority root and reconciliation before admission. |
| F27 | Workflow code upgraded incompatibly | Replay/deployment qualification fails before trusted rollout. |
| F28 | Noisy telemetry floods the team | Control and verification lanes remain available; lead wakeups are coalesced. |
| F29 | Long healthy job emits no chat messages | Progress policy does not infer failure solely from chat silence. |
| F30 | All prior lead transcript hidden | New lead reconstructs constraints, jobs, evidence, uncertainty, and safe next actions. |
| F31 | Negative research result | Investigation can complete without marking the original hypothesis established. |
| F32 | Evaluated agent modifies evaluator | Change is a separate unpromoted candidate; current result cannot improve through it. |

Model-check a bounded core involving two role epochs, two workspace generations, duplicate messages, one cancellation, and one promotion. Then test real adapters independently. A model of the control protocol does not prove OS, network, database, or provider behavior.

## Appendix C. Suggested API surface

Names below are illustrative proposed contracts, not existing commands.

| Family | Operations |
|---|---|
| Mission | admit, inspect, revise, pause, cancel |
| Task | propose, admit-revision, inspect, explain |
| Plan | validate, admit-revision, inspect |
| Role | assign, take-over, inspect |
| Scope | inspect, pause, quiesce, revoke, transfer |
| Attempt | admit, inspect, steer, checkpoint, finish |
| Execution | submit, inspect, observe, stop, reconcile |
| Artifact | stage, finalize, inspect, export |
| Candidate | seal, inspect, compare |
| Verification | request, inspect, submit-receipt |
| Promotion | prepare, commit, inspect, reconcile |
| Evidence / Claim | inspect, link, challenge, supersede |
| Message | send-typed, acknowledge, subscribe |
| Budget | reserve, settle, inspect, reconcile |
| Events | subscribe-with-cursor, explain, select-context |

The model-facing subset omits root-authority and policy-administration operations. Every mutating request is authenticated and participates in the relevant revision/idempotency protocol. APIs return structured blockers, not generic success/failure strings.

## Appendix D. Decisions that need explicit review before implementation

The architecture does not require resolving every product preference before building a vertical slice, but the following choices must be explicit at the appropriate gate:

- The first real runtime adapter and the exact pinned runtime/transport to qualify.
- The controller host, PostgreSQL durability/backup profile, and artifact persistence tier.
- Whether initial model usage is fully mediated or a transparently weaker native-provider accounting profile.
- The protected integration repository and operator-approved local-checkout synchronization policy.
- The first representative task and its frozen acceptance/evaluation contract.
- The initial autonomy/budget envelope, retained-job partition policy, and escalation thresholds.

Language and deployment layout should follow the actual implementation context rather than forcing a rewrite of working Armada components. Logical modularity matters more than prematurely splitting services.

## Source notes

### Project design inputs

**[P1]** `harp-durable-execution-design.md`, September 5, 2026, proposed v0.1. Especially sections 4–12 (identities, authority, effects, context, evidence, promotion), 14–19 (budgets, adapters, observability, improvement, validation). Read from the connected file library. This source explicitly describes a proposed, not implemented or verified, system.

**[P2]** `2026-09-16-insula-execution-profiles.md`, approved product/architecture specification. Execution Contract seam, generic lifecycle ownership, conformance facets, indeterminate mutations, placement, and evidence qualification. Read from the connected file library.

**[P3]** `2026-09-17-armada-product-surface-design.md`, approved design. Layered scope, single-execution profiles, persistent sessions, public execution seam, and CLI convergence. Read from the connected file library. Historical CLI details are not claims about a currently audited binary.

### External primary references checked during this synthesis

These support specific underlying mechanisms, not the correctness or performance of Corenius. Pin implementation versions rather than assuming rolling documentation is a compatibility contract.

**[R1]** Temporal, idempotency and durable execution. `https://temporal.io/blog/idempotency-and-durable-execution`

**[R2]** Temporal Python SDK README: deterministic workflows, Activities, and “Sandbox is not Secure.” `https://github.com/temporalio/sdk-python`

**[R3]** PostgreSQL transaction isolation documentation. `https://www.postgresql.org/docs/current/transaction-iso.html`

**[R4]** Temporal Python Continue-As-New documentation. `https://docs.temporal.io/develop/python/continue-as-new`

**[R5]** Earendil, Pi Durable. Experimental agent/task-harness durability and interrupted-operation behavior. `https://earendil.com/posts/pi-durable/`

**[R6]** OpenAI Codex app-server documentation. Thread/turn/item lifecycle, native goal accounting, and steering. `https://developers.openai.com/codex/app-server`

**[R7]** Git `update-ref` reference: expected-old-value checks and ref transaction semantics. `https://git-scm.com/docs/git-update-ref`

**[R8]** Git `worktree` reference: shared repository metadata and references. `https://git-scm.com/docs/git-worktree`

**[R9]** Jujutsu tutorial: stable change identity versus evolving commit identity. `https://github.com/jj-vcs/jj/blob/main/docs/tutorial.md`

**[R10]** Kata Containers project documentation and software overview. `https://katacontainers.io/` and `https://katacontainers.io/software/`

**[R11]** Martin Kleppmann, How to do distributed locking: resource-side fencing. `https://martin.kleppmann.com/2016/02/08/how-to-do-distributed-locking.html`

## Closing thesis

Corenius should make agency replaceable without making work disposable. Its dependable core preserves intent, authority, resources, and evidence; its flexible edges let better models, workflows, environments, and verifiers improve the engineering effort without redefining what is authorized or what counts as success.
