# RFC0002 v0.2 implementation reconciliation

Status: independent Architecture and Spec gates PASS; implementation/qualification outstanding
Source commit read: `942f037800102f3d786ac547e72f11ab928b50ae`
Accepted v1 kernel: `264756edd550225449202b6f20ac4b560808d14e`
Owner direction: [RFC0002](../../rfcs/0002-persistent-engineering-teams.md), including its preserved v0.2 source
Canonical requirements: [SPEC.md](SPEC.md) and the linked [tickets](tickets/)

## Target and evidence boundary

Make Corenius the executable reference for persistent teams with replaceable
Codex/Claude leads and TraeCLI workers. Kata owns the work graph; Corenius and
PostgreSQL own admitted authority and acceptance; Temporal owns durable
coordination; jj is the preferred worker VCS candidate to qualify; Armada/Insula
owns independently supervised computation. Verification identifies the exact
result that counts. This effort changes Corenius alone.

RFC0002 supersedes the earlier optional-Kata and indefinitely deferred-jj target.
Its capture is preserved verbatim with provenance. External product assertions
and illustrative workflow code are inputs to qualification, not proof.
No Markdown policy migration, shared repository conversion or new unbounded
provider spend follows from capture. A capability that fails qualification
stops its dependent milestone and records a decision; it is not silently dropped.

Accepted v1 tickets and evidence keep their original specification bindings.
Do not reopen them to manufacture a v0.2 pass. New contracts are additive until
their migrations and assembled proof are reviewed. The current integration
service prepares a fixture handoff, not a protected-ref mutation. Existing
lead_generation already separates lead-seat replacement from worker task
generation. Extend these boundaries rather than claiming they are absent.

## Admitted work and strategy

`WorkRef` includes ledger kind, deployment identity, tenant/project and immutable
work-item identity. An `IntentRevision` pins the source revision, canonical
snapshot digest, objective, constraints, allowed capabilities and an
`AcceptanceContract`. Deleted or unavailable source revisions are explicit
states, not invitations to substitute latest content. An admitted revision
remains intelligible from its retained snapshot during a Kata outage.

Dependencies pin prerequisite WorkRef, IntentRevision, accepted subject and
required evidence policy. Kata READY/CLOSED or mutable current relationships
do not satisfy admission. A newly broker-admitted prerequisite IntentRevision
(not a Kata text edit or event) suspends new dependent
grants pending explicit revalidation; already granted operations retain their
original receipt and reconcile under that authority. No retroactive rewrite of
history, implicit new capability or cancellation-as-confirmed-termination.
An unadmitted Kata edit creates attention/proposal only; current admitted
dependencies retain their pinned meaning until explicit broker amendment.

`PlanRevision` describes strategy separately from success. Plans have immutable
digests, stable step identities, explicit dependencies/joins, effect allowlists,
bounded loops/repair and resource limits. Plan admission cannot alter intent,
acceptance, epochs or budgets. Model-generated source is never evaluated as a
Temporal workflow. Changes to intent require separate admission; changes to
strategy require PlanRevision admission.

## Scope, jobs and independent epochs

An ExecutionScope normally serves one admitted work-item revision. Scope
ownership persists across lead replacement, attempt failure and workflow
rollover. An Attempt remains a bounded try; its episode, workspace and producing
provenance do not become the lifetime of the job. One scope can own multiple
jobs and attempts; one attempt can create multiple jobs. Role episodes and
legacy jobs may legitimately have no creating Attempt. Retain optional
original role/Episode/Attempt provenance; never synthesize a past Attempt.
At most one current writer controls a WorkspaceGeneration.

Every consequential grant checks deployment recovery epoch, ScopeEpoch,
RoleEpoch when relevant, admitted IntentRevision/PlanRevision, operation
identity and applicable WorkspaceGeneration/RuntimeIncarnation. These axes
are independent. Replacing a lead changes its seat epoch; it does not reset
scope authority, move worker ownership, cancel jobs or refund budget. Scope
revocation blocks future effects while preserving previously granted effect
receipts and unresolved termination.

Account separately for each budget dimension. For a cumulative dimension,
node allocation = own unspent + own reserved/unknown + own consumed + sum of
child allocations. Each child's allocation already includes its subtree's
consumption/reservations; never add those again at the parent. Child transfer
atomically moves own unspent into a child envelope. Reservation moves unspent
to committed, then to consumed or known unused; consumed is monotonic.

Reusable concurrency capacity has a separate ledger: each node envelope equals
own free + own reserved/active/unknown occupancy + child envelopes. Confirmed
termination returns occupancy to free; chat silence, attempt replacement or
cancel request does not. Capacity reuse never refunds cumulative spend.
Different units/currencies are not summed without an admitted conversion policy.
Concurrent child creation, unknown usage and provider/goal replacement cannot
increase available capacity. Explicit owner adjustments retain rationale and
attribution outside ordinary release.

Migration runs with affected admission closed. Backfill legacy attempt-bound
workspaces/jobs only from unambiguous mission/task/specification provenance,
including explicitly represented role/mission scopes for legitimate NULL
Attempt records. Preserve their absent creating Attempt as absent.
Keep old manifest identities, grants, usage and accepted subjects intact;
unknown ownership is quarantined, not guessed. Remove legacy one-job-per-attempt
constraints only after explicit scope ownership is represented and tested.

## Caller authority and bounded delegation

Authentication never implies permission to mutate these records. Default intent
and authority changes are Owner-only. An Owner may admit a bounded delegation
pinning mission/scope, role identity+epoch, allowed command families/effects,
intent/plan limits, resource ceilings and expiry. Recheck it in the same broker
transaction as the mutation. Credentials are not transferable by editing sender
fields or Kata ownership. Workers can propose; they cannot self-admit work.

| Mutation | Authorized caller and boundary |
| --- | --- |
| intent.admit / dependency.revalidate | Owner, or current lead with an explicit admitted delegation for these exact mission/work scopes; expanding objective, acceptance, permissions or root budget remains Owner-only |
| plan.admit | Owner or current delegated lead within admitted intent/effect/resource limits; worker proposal is inert unless a separate bounded subplan delegation was admitted |
| scope.admit / child.allocate | Owner or delegated lead; current parent scope/intent and atomic conserved envelope are mandatory; no implicit root-budget increase |
| scope.revoke / execution.stop | Owner or explicitly scoped controller/cancellation capability; stops future grants, then records physical outcome separately |
| scope.reconcile / execution.adopt | Recovery operator or delegated broker reconciler under current recovery/scope epoch; records proven effects against old receipts; unresolved evidence cannot mint fresh authority or free capacity |
| controller.transfer | Recovery operator or expressly delegated control service, after old transport/writer ownership is fenced and effects reconciled; ordinary lead/worker tokens cannot transfer controllers |
| Kata decomposition/amendment | Broker planning service executing an admitted Owner/current-lead proposal under scoped ledger credentials; journal each operation; no runtime grant is derived from the ledger result |
| Kata projection / event ingestion | Dedicated adapter credential limited to qualified project/read/projection capabilities; origin and revision checks prevent loops; cannot accept intent or candidates |
| execution.ensure / physical integration.execute | Dedicated execution/integration service consumes exact broker grant/approval under current applicable epochs; model tools request domain operations, never raw privileged RPC/ref mutation |

Retain distinct authenticated verifier and integrator roles. An integration
approval binds normalized arguments and immutable proof/subject; the service
does not inherit general authority from a lead conversation. Changes requiring
more authority produce proposals/approval requests rather than implicit grants.

## Kata service boundary

Use qualified service APIs/events, never Kata's internal tables. Its deployment
and schema are independently owned. Pin version, API schema and capability
profile with executable tests. Before a mutation, journal stable operation ID,
idempotency key, payload digest and revision precondition. An uncertain response
requires qualified lookup/reconciliation before another mutation.

Lead decomposition is a durable bundle of ticket and relationship mutations.
Each item has stable identity and an outcome; a partially created bundle is
visible and blocks admission until repaired/reconciled. There is no presumed
atomic transaction spanning Kata and Corenius. Missing idempotency/lookup for
an operation limits automation instead of permitting blind retry.

Event ingestion records namespace, order/cursor and applied dispositions.
Duplicate or reordered wakeups cause authoritative rereads. Cursor gaps and
retention expiry trigger consistent snapshot/high-watermark resync; unsafe
ingestion/admission remains closed until coverage is established. Bounded
queues/backpressure and projection-origin suppression prevent loops or floods.
Outage preserves already admitted execution; new work requiring unavailable
source evidence stops. Projections of acceptance/attention are duplicate-safe
and cannot grant authority or reinterpret ticket closure as verified success.

## Stable per-scope Temporal interpreter

Each active ExecutionScope gets an independently addressed workflow; Kata's
parent/child graph does not determine Temporal child-workflow lifetimes.
Short Activities ensure, inspect or stop durable operations via broker grants;
they do not hold a model stream or long external job in an Activity heartbeat.
All state-changing Activities check current authority in broker transactions.

Use monotonic wake sequence/cursor handling so a wake during refresh/drain
cannot be cleared as processed. Signals carry hints; bounded durable timers
and service anti-entropy find missed wakes. The current v1 500 ms timer bounds
boolean-hint delay; preserve that protection until the replacement is proven.
Test refresh/wait races, reconnect duplicates, terminal-wake rules and lost
start acknowledgment. Unknown operations are inspected, never blindly ensured
as a fresh effect.

Continue-As-New carries scope/workflow generation, intent/plan/interpreter
versions, step/operation identities, event cursor and durable wait references.
Reread PostgreSQL after rollover; replayed state confers no authority.
Plan amendments take effect at a recorded safe point. In-flight effects retain
their original operation/plan binding and must complete, cancel/reconcile or
remain quarantined; completed steps are never reinterpreted by new code.
Keep compatible interpreter versions and replay-test retained histories before
promotion. An incompatible deployment blocks dispatch, not evidence reads.

## Exact candidate and physical protected integration

Workers use private jj stores after qualification; change/operation IDs record
lineage. Candidate verification pins immutable commit/tree, intent and acceptance
contract, policy/test-suite, verifier identity and environment. A rewrite or base
change requires a new candidate and proof; retained original bundles stay usable.
Git remains the protected-ref substrate initially. Existing shared branches and
the user's checkout are preserved under fast-forward-only repository policy.

One integration gate journals intent before physical CAS. Grant pins candidate,
tree, evidence/approval digest, target identity and expected predecessor;
validate fast-forward ancestry and exact subject. Serialize conflicting effects
and check current authority immediately before the protected mutation. Lost
acknowledgment or successful ref mutation before PostgreSQL receipt requires
operation-attributable reconciliation. Merely observing the target at candidate
does not identify which operation performed it. Hold conflicts while unknown.
Out-of-band writers are excluded by the qualified gate ownership contract or
cause explicit unresolved-state handling. Tests use private targets, including
untracked-file collisions; no implementation fixture mutates main.

## Independent execution provider contract

Expose ensure/inspect/stop against stable Execution handles, target and Insula
identity, recipe/requirements and pinned provider capability profile. Allocate
identity and journal before launch. Record scope ownership separately from
creating-attempt provenance and current controller. Job inventory, logs,
checkpoints and termination observations survive runtime/lead/workflow loss.

Qualify idempotency or discovery, independent inventory/orphan coverage,
resource limits, cancellation versus physical termination, evidence integrity
and recovery. Local PID-namespace proof does not establish remote/GPU semantics.
Implement the Corenius seam and test adapters here; no sibling repository edits
or production placement are implicit. An unavailable provider or incomplete
inventory leaves precise quarantine. The adapter is independently testable
without the new workflow interpreter, avoiding a dependency cycle.

## Delivery order and review gates

| Ticket | Deliverable | Blockers |
| --- | --- | --- |
| [26](tickets/26-intent-plan-contracts.md) | Exact admitted work, dependency and acceptance/strategy contracts | 16 |
| [27](tickets/27-durable-scopes.md) | Scope/job ownership, independent epochs and conserved child allocations | 15, 26 |
| [28](tickets/28-kata-work-ledger.md) | Qualified API/events adapter and decomposition recovery | 17, 26, 27 |
| [31](tickets/31-independent-execution.md) | Independently testable external ExecutionContract and inventory | 15, 27 |
| [29](tickets/29-scope-plan-interpreter.md) | Per-scope workflows, stable Plan IR and compatible rollover | 13, 26, 27, 31 |
| [30](tickets/30-protected-integration.md) | Physical protected-ref CAS and uncertain-effect reconciliation | 11, 27, 31 |
| [25](tickets/25-jj-backend.md) | Qualified preferred worker backend with immutable proofs | 19, 27, 30, 31 |
| [32](tickets/32-v02-assembled-proof.md) | Real PostgreSQL/Temporal v0.2 assembled fault proof | 25, 28, 29, 30, 31 |

Existing bounded runtime investigation20 can finish independently; live
adapter21 waits for20 and32. Independent lead demonstrations22/23 follow21;
restore24 qualifies all selected adapters and migrations. No row's mere
existence grants runtime spend or constitutes its acceptance. Each unit needs
meaningful red/green, exact freeze, Standards/Spec conformance and independent
branch review before gated integration. Source-level micro-plans remain scratch.

## Fault coverage without upgrading historical claims

The following is a conservative source-inspection map at accepted v1 commit
`264756edd550225449202b6f20ac4b560808d14e`. Every v1 pointer is partial for the
RFC fault: it identifies related source assertions, not a newly executed or
fully covered F-row. Gap means no passing proof is claimed. The accepted
[source manifest](evidence/kernel16/review611-portable-proofs/source-manifest.json),
[conformance](evidence/kernel16-review611-conformance.json),
[handoff](evidence/kernel16-handoff.md) and
[quarantine disposition](evidence/kernel16-review611-acceptance-disposition.json)
retain the exact original acceptance boundary. Ticket32 must cite exact accepted manifests and
observable assertions for the assembled v0.2 matrix, not just test names.

| RFC0002 fault | Source-inspected v1 pointer (partial) or gap | Required extension / owner |
| --- | --- | --- |
| F01–F02 duplicate admission/key conflict | [kernel.rs:31](../../../crates/team-control-plane/tests/kernel.rs) admission idempotency; original [10 proof](evidence/ticket10-kernel-admission.md) | Intent/scope binding and payload conflicts; 26/27/32 |
| F03 commit before dispatch crash | partial13 outbox recovery | Exact operation attribution across scope bridge; 29/32 |
| F04 repeated/reordered wake | [temporal.rs:907](../../../crates/team-control-plane/tests/temporal.rs) SDK/CAN/replay | Sequence race, per-scope cursor and anti-entropy; 29/32 |
| F05 lost runtime start reply | [gateway.rs:27](../../../crates/team-control-plane/tests/gateway.rs) runtime fault; original [12 proof](tickets/12-gateway-journal.md) | Independent job discovery and live provider; 31/21 |
| F06 competing goal continuation | broker-only baseline | Native/broker exclusive-owner fault; 21 |
| F07 lead death during job | [supervisor.rs:600](../../../crates/team-control-plane/tests/supervisor.rs) initiator death; original [15 manifest](evidence/ticket15-accepted-manifest.json) | Scope-owned multiple/remote jobs; 27/31/22/23 |
| F08 obsolete lead returns | [mailbox.rs:185](../../../crates/team-control-plane/tests/mailbox.rs) old-lead rejection; original [14 manifest](evidence/ticket14-accepted-manifest.json) | Role/scope separation through assembly; 27/32 |
| F09 writer survives lease | [supervisor.rs:544](../../../crates/team-control-plane/tests/supervisor.rs) stale-generation fence | Workspace epoch and protected gate; 27/30/32 |
| F10–F11 cancellation/unknown stop | partial assembled cancellation/15 | Granted-effect receipt, independent unreachable stop; 27/31/32 |
| F12 useful late cancelled result | [assembled_vertical.rs:2326](../../../crates/team-control-plane/tests/assembled_vertical.rs) cancelled submission rejection | Retained quarantine and new-authority reuse; 27/32 |
| F13 concurrent nested allocations | gap | Atomic parent/child conservation; 27/32 |
| F14–F15 budget/provider changes and usage inclusion | [kernel.rs:603](../../../crates/team-control-plane/tests/kernel.rs) and [assembled_vertical.rs:2294](../../../crates/team-control-plane/tests/assembled_vertical.rs) usage checks | Cross-attempt/provider accounting and deduplication; 27/31/32 |
| F16 candidate edited after pass | [artifacts.rs:643](../../../crates/team-control-plane/tests/artifacts.rs) stale candidate/spec/base rejected | Actual jj retained/fresh proof; 19/25/32 |
| F17–F18 changed evaluator/fabricated pass | [assembled_vertical.rs:523,6609,8345](../../../crates/team-control-plane/tests/assembled_vertical.rs) verification policy/tamper cases; original16 links above | New contract/Plan/backend exact bindings; 26/25/32 |
| F19–F20 target move / success before DB receipt | gap:11 prepares only | Physical CAS, lost acknowledgment attribution; 30/32 |
| F21 user checkout untracked collision | gap | Private-target collision preservation/refusal; 30/32 |
| F22 missing sandbox capability | partial15 namespace/isolation | Capability-negative admission attribution; 31/32 |
| F23 poisoned build cache | gap | Trusted verifier cache/provenance policy; 25/32 |
| F24 durable ENOSPC | [artifacts.rs:5](../../../crates/team-control-plane/tests/artifacts.rs) interrupted publication only | ENOSPC/no acknowledgment/checkpoint preserved; 32 |
| F25 GC during uncertain effect | [artifacts.rs:877](../../../crates/team-control-plane/tests/artifacts.rs) referenced-artifact GC | Outstanding-operation retention race; 27/30/32 |
| F26 DB rollback with survivors | gap | External recovery epoch and orphan inventory; 24 |
| F27 incompatible workflow upgrade | v1 retained replay | Versioned interpreter migration/replay rejection; 29/24 |
| F28–F29 telemetry flood / silent healthy job | bounded mailbox sources only | Control-lane fairness and independent progress; 29/31/32 |
| F30 hidden lead transcript | kernel reconstructed view | Actual Codex/Claude replacements; 22/23 |
| F31–F32 negative research / evaluator candidate edit | gap | AcceptanceContract research and isolated evaluator proposal; 26/32 |

## Compatibility and completion

The v1 assembled loop remains a valid baseline, not a v0.2 assembled proof.
Historical tickets17/18 retain their evaluated/deferred disposition; new28
implements the owner's later first-class-ledger direction. jj19 records measured
limitations; its evaluator's older defer recommendation cannot resolve25 by
itself under this target. An unqualified jj capability needs an explicit recorded
disposition, with no unsafe fallback or default-selection claim.

Only a reviewed accepted32, live21, independent22/23 and restore24 justify the
full goal. A precise safe stop passes a recovery invariant when the specified
policy allows it; it does not prove a missing capability or successful job
transfer. Keep active runs, negative evidence and dirty worktrees preserved.
