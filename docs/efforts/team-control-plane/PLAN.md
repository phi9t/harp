# Durable team control plane implementation plan

Status: approved baseline; kernel accepted/landed; optional evaluation and live prerequisites underway
Source commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`
Spec: [SPEC.md](SPEC.md)
Preserved intent: [RFC](../../rfcs/0001-durable-team-control-plane.md)

## Goal

Make Corenius the executable reference for a persistent engineering team with
replaceable Codex/Claude leads and TraeCLI workers: assignments, decisions,
questions, budgets and evidence survive failure, uncertain execution is
reconciled before retry, and acceptance binds the exact verified candidate.

## Current execution checkpoint

Kernel16 is accepted and landed on main at `264756edd550225449202b6f20ac4b560808d14e`, with independent conformance, integrated checks, whole-branch617 review and fresh detached exact-tip gate. Optional Kata adapter18 is resolved as deferred under evaluated17; Git planning remains authoritative. jj19 remains under executable evaluation and has not been adopted.

Initial synthetic Trae preflight exhausted four incarnations and observed nine attempted requests, exceeding the six-request/no-retry envelope. Routing and stopped namespaces are evidence; live execution readiness is not established. The original failure remains retained. The [bounded repair](decisions/20261003-trae-synthetic-budget-repair.md) and [counting clarification](decisions/20261003-trae-budget-admission-clarification.md) authorize offline guard preparation with independent critique; no new runtime admission has occurred. Denied retries count as attempts, discovery gets no free allowance, and stopping a local stream does not prove remote billing stopped.

Independent lead demonstrations and upgrade/backup/rollback-restore remain required. Current ticket statuses, [STATE.md](STATE.md) and [completion audit](completion-audit.md) distinguish acceptance from preparation and evidence from inference; the milestone contracts below remain unchanged.

## Architecture and implementation baseline

Build an external Rust broker and supervised gateway. PostgreSQL owns domain
transitions, execution ownership, budgets, journals and evidence bindings.
Temporal owns durable waits and lifecycle control; external actions run through
Activities and idempotent broker operations. Execution supervisors own actual
workloads and attempt workspaces. Agent tools and operator views call the same
authenticated command boundary.

Use Git workspaces and approved Git planning input first. Evaluate Kata for
ledger/UI reuse and jj for workspace/change management independently. Neither
becomes an execution authority or a prerequisite for the durable kernel.
Preferred workflow stack is Rust, contingent on a pinned SDK feasibility gate;
Python is the fallback without changing the broker contract. These baseline defaults were explicitly approved by the owner; see the
[decision](decisions/20261001-approved-baseline.md). Live-probe resource limits
and actual provider guarantees remain separately gated.

The repository workflow owns decomposition: this plan sequences canonical
[tickets](tickets/); exact file/function/code micro-plans belong in session-local
scratch when a ticket is executed. The Matt specification/decomposition route
is applied through the repo crosswalk with writing-plans support, since those
owner skills are not directly invocable in this harness. No duplicate
`docs/superpowers` plan is created.

## Global constraints

- One writer per working tree; each implementation ticket gets an isolated checkout.
- Lead proposes; broker admits/enforces; runtime executes; supervisor owns effects.
- Persist consequential intent and reserve budget before external I/O.
- No automatic retry of ambiguous non-idempotent starts, even after lease expiry.
- Fence credentials and grants by ownership generation and deployment recovery epoch.
- Enforce unique control of physical runtime/store/thread identity across aliases.
- Separate persisted, presented, acknowledged and resolved message states.
- Keep approved specification/candidate versions immutable and verification exact.
- Cancellation/deadline completion requires stopped effects or explicit quarantine.
- Quarantine holds conflicting capacity and prevents unsafe workspace reuse.
- Existing Markdown planning and human integration policy stays effective pending an explicit policy decision.
- Every mission has a finite admitted budget; replacement does not reset it.
- No deployment, commits, pushes, PRs or human merge are performed by this planning task.

## Milestone 0: settle the implementation boundary

Resolve existing decision tickets [03](tickets/03-implementation-language.md)
through [09](tickets/09-first-slice-acceptance.md), with bounded experiments
where evidence is required. Select the worker SDK from compile/replay evidence,
approve the command/read-model/supervisor public test seam, bind planning versus
execution authority and set fixture/live-run budgets. Decide the recovery
contract and first acceptance boundary from the reviewed spec.

Deliverable: recorded decisions and a pinned technology/runtime profile.
Exit: no material decision blocks ticket 10. Do not declare live runtime
guarantees from help/schema observations. Separate model-free setup from any
budgeted live probe. There is no elapsed-time estimate masquerading as evidence.

## Milestone 1: durable kernel with a fault runtime

| Ticket | Independently reviewable deliverable | Blockers |
| --- | --- | --- |
| [10: admission](tickets/10-kernel-admission.md) | Real PG command API; atomic intent, ownership, budgets and outbox | 03, 05, 09 |
| [11: artifacts/acceptance](tickets/11-artifact-acceptance.md) | Immutable candidate, exact verification and acceptance | 10 |
| [12: gateway](tickets/12-gateway-journal.md) | Serialized external-operation journal and fault runtime | 10 |
| [13: workflows](tickets/13-temporal-bridge.md) | Real Temporal replay, DB command bridge and durable lifecycle | 10, 12 |
| [14: team tools](tickets/14-mailbox-lead.md) | Scoped mailbox, yield/checkpoint and reconstructible lead view | 10, 11, 13 |
| [15: environments](tickets/15-supervised-environments.md) | Git workspaces, complete process inventory and Execution handles | 10, 12 |
| [16: assembled proof](tickets/16-kernel-proof.md) | End-to-end failure-tested loop and basic operator CLI | 11, 13, 14, 15 |

The first useful loop is: approve/import mission → admit bounded task → allocate
attempt/environment → record start intent → execute fault runtime → yield/answer
→ submit immutable candidate → verify → accept → prepare integration handoff.
Every arrow passes through recorded authority and the public observable seam.

Exit: all kernel rows of SPEC's failure matrix pass against real PostgreSQL and
Temporal. Duplicate delivery does not duplicate admission or external effects.
Lost acknowledgments become proven adoption or visible quarantine. Lead/view
loss preserves plan/questions. Old credentials, controllers and alias bindings
cannot gain current authority. Budgets and candidate changes cannot bypass
acceptance. Actual supervisor fixtures prove nested-process shutdown independently
of fake runtime state. This is kernel proof, not live Trae proof.

## Milestone 2: reuse evaluations and optional adapters

| Ticket | Decision or deliverable | Blockers |
| --- | --- | --- |
| [17: Kata evaluation](tickets/17-kata-evaluation.md) | Evidence-backed adopt/defer decision and authority boundary | 05, 10 |
| [18: Kata adapter](tickets/18-kata-adapter.md) | Conditional approved-snapshot import and duplicate-safe projections | 16, 17 |
| [19: jj evaluation](tickets/19-jj-evaluation.md) | Fixture-backed adopt/defer decision for a workspace backend | 11, 15, 16 |
| [25: jj backend](tickets/25-jj-backend.md) | Conditional implementation and accepted backend proof | 11, 15, 16, 19 |

Kata evaluation can proceed alongside kernel work after authority is fixed.
Custom issue UI waits for that decision. The Kata adapter cannot block recovery
when Kata is unavailable; claims/closures remain proposals/completion claims
until broker admission/acceptance applies. No tracker migration is implicit.

jj evaluation keeps Git as the reference baseline. Any selected backend must
retain exact conflict-free candidate commits/trees, respect shared-store
mutation boundaries and pass existing evidence/gate/GC/landing contracts.
Operation logs aid repository recovery; they do not undo external effects.

Exit: explicit decision for each integration; selected adapters pass their
contract fixtures. Deferred adapters stay out of the critical path. Selecting
jj activates ticket 25 before live use; deferred selection resolves that ticket
with an explicit Git-only scope. Evaluation never silently expands to rollout.

## Milestone 3: actual Trae execution and recovery

| Ticket | Deliverable | Blockers |
| --- | --- | --- |
| [20: live probes](tickets/20-trae-probes.md) | Version-pinned capability and recovery evidence | 04, 16 |
| [21: live adapter](tickets/21-trae-adapter.md) | Measured provider behavior behind kernel contracts | 12, 15, 16, 20 |

Use dedicated fixtures and explicitly admitted model/resource budgets. Probe
unknown thread creation separately from lost turn-start replies, then detached
views, active steering, server restart, cancellation and surviving background
jobs. Record unsupported observation or deduplication rather than inventing
safe retry behavior. Native runtime goal continuation remains disabled.

Exit: each supported recovery path has provider and independent environment
evidence. The adapter either recovers safely or stops in a precise unresolved
state. The kernel matrix is rerun against actual supported provider operations;
fake evidence never substitutes for live guarantees.

## Milestone 4: the engineering reference

| Ticket | Deliverable | Blockers |
| --- | --- | --- |
| [22: Codex lead](tickets/22-codex-demonstration.md) | Bounded real engineering mission and lead-loss proof | 11, 14, 21 |
| [23: Claude lead](tickets/23-claude-demonstration.md) | Same-baseline independent mission and lead-loss proof | 11, 14, 21 |

Use the same repository baseline, acceptance criteria and finite budgets in
separate workspaces. Each demonstration includes implementers, independent
verification, a yielded technical question, loss of lead/view, a measured
runtime fault and exact integration preparation. Retain candidate/test/environment
identity and real unresolved risks. Compare contract adherence; do not infer a
general multi-agent speed advantage from two runs.

Exit: two independent evidence bundles show interchangeable lead contracts and
recoverable work. Distill measured practice into canonical guidance only after
proof, with policy version/manifest updates and sync tests if policy changes.
Any optional backend used here must already have passed its acceptance gates.

## Milestone 5: persistent-use readiness

[24: restore and rollout handoff](tickets/24-restore-rollout.md) depends on 16,
18, 21, 22, 23 and 25. Conditional tickets 18 and 25 resolve either with accepted
adapter evidence or an explicit defer/Git-only disposition; selected adapters
cannot bypass rollout prerequisites. It covers readiness, credential separation, schema migration,
workflow replay compatibility, artifact retention, backups and restore.

Restore drills must retain a post-backup job and an old credential while
rolling back the domain database. A newly allocated recovery epoch rejects the
old credential; inventory finds jobs absent from the backup; dispatch remains
disabled until effects are adopted/stopped/quarantined. A restored database
alone is never proof that external work stopped.

Exit: fresh checks, restore/upgrade evidence, independent whole-branch review,
known limitations and an operator handoff. Production placement, RPO/RTO and
multi-host failover remain separately approved scope. This milestone prepares
a reviewable deployment; it does not perform one.

## Execution and review method

For every ticket: check resolved blockers → claim → isolated clean-context
worker → public failing fixture → minimal implementation → passing fixture →
focused/repository checks → independent Standards and Spec review → evidence
handoff. Use at most three repair/review iterations; unresolved uncertainty
returns to a decision instead of extending authority or budget.

Parallelize only after prerequisite contracts are stable: 11 and 12 can split;
13 and 15 can split after 12; 14 follows 13; 17 is separate learning work;
22 and 23 use independent workspaces and budget capacity. The lead owns
sequencing and integration; implementing workers do not manage other runtimes.
Do not use open ticket status as permission to bypass owner decisions.

Evidence for each ticket records fixed base, candidate, specification revision,
environment, commands/results, red/green behavior, independent review and
limitations. Raw private output stays outside tracked public evidence. Do not
mark a ticket resolved from an implementation claim alone. Commit/push authority
and human integration remain separate from a successful ticket review.

## Coverage and review focus

| High-risk condition | Owning tickets | Required check |
| --- | --- | --- |
| DB rollback revives old grants and misses newer executions | 10, 24 | Recovery epoch rejection and orphan inventory before dispatch |
| Endpoint/binding aliases address the same physical thread | 12, 21 | One authoritative owner and no second active episode |
| Temporal retry repeats an uncertain provider start | 12, 13, 21 | Journal lookup/adoption/quarantine, no blind resend |
| Workspace/store operations change verified content | 11, 15, 19 | Immutable candidate preservation and new-proof requirement |
| Planning close/claim or projection loop becomes execution authority | 17, 18 | Explicit revision admission and broker acceptance only |

Other SPEC coverage: domain/API/auth/budget → 10/14; deterministic workflows,
cursors/rollover → 13; message/yield/lead loss → 14/22/23; shared effects,
cancellation/GC → 15/21; artifact verification/integration → 11; operator takeover
and kernel matrix → 16; capability pinning → 20; restore/retention/readiness → 24.

## Definition of done

Corenius has a reviewed executable control plane, measured Trae adapter,
separate Codex/Claude reference runs and a restore/upgrade handoff. Every required
fault recovers or reports a precise safe stop; task acceptance is tied to actual
verified content. Optional products have explicit evidence-backed decisions.
The final human handoff names remaining limitations and has fresh test/review
results. A written plan, fake runtime success or healthy terminal does not meet
this definition.
