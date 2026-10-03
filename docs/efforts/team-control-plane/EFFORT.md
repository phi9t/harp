# Effort: durable team control plane

Status: accepted v1 kernel; v0.2 implementation and runtime qualification underway
Base: main at `264756edd550225449202b6f20ac4b560808d14e`
Lead: replaceable Codex implementation coordinator

## Current execution contract

Corenius alone is the active scope. The owner-authorized objective is the
complete persistent engineering-team implementation, including verification
and gated fast-forward landing; see
[full-goal authority](decisions/20261003-owner-authorizes-full-goal-execution.md).
The accepted kernel is v1 evidence, not completion of live runtime, independent
lead or restore requirements. RFC0002 v0.2 is the current architectural target:
Kata as first-class work ledger, qualified jj worker workspaces, durable
ExecutionScopes, independently supervised jobs and a stable per-scope Plan
interpreter. [The reconciliation](v02-reconciliation.md), [PLAN.md](PLAN.md)
and tickets26–32 define the additional work and preserve earlier evidence.

Ticket26 is being implemented in its own worktree. jj19's final history-export
repair is frozen for independent review. One fixed offline namespace cleanup
case has independently verified terminal0 cleanup measurements; full conformance
is partial because original prelaunch controller ordering remains unverified.
It does not admit a Trae or paid provider launch. Historical quarantines and
spent budgets remain recorded. [STATE.md](STATE.md) and the
[completion audit](completion-audit.md) carry current progress.

## Preserved discovery and execution history

The following sections retain the earlier discovery scope and approvals.
They do not limit or replace the current execution contract above.

## Intent

Preserve the owner's supplied architecture as an RFC and determine how to
implement an external durable team broker above Trae. The lead proposes work;
the broker records and enforces authority; runtimes execute turns; supervised
environments own effects; user interfaces observe and intervene.

## Route and scope

`ask-matt` routes this larger, uncertain architecture through wayfinding,
supported by research, domain modeling and brainstorming. Apply installed Matt
instructions through the repository invocation-availability adapter. Durable
decision intent is stored here; scratch execution remains untracked.

Authorized now: RFC capture, local read-only investigation, primary-source
research, independent critique, an implementation feasibility map and the
owner-requested detailed implementation specification. The active goal now carries the full implementation objective. Architecture
decision prerequisites still apply before admitting product tickets; model-free
local feasibility probes are underway. This does not grant production
deployment, live paid model budgets, commit, push or mainline integration.

The earlier reference-quality design remains at
`../agentic-engineering/reference-design.md`. Its safety and evidence work is
input to this effort; its pending implementation graph is not silently adopted
as the broker design. In particular, a database-owned broker would require an
explicit decision about the current canonical Markdown tracker policy.

## Milestones and exit criteria

- Preserve the complete supplied proposal, including diagrams and unresolved
  source markers, with an explicit proposal/evidence distinction.
- Investigate actual local runtime capabilities and first-party durability
  contracts. Distinguish help/schema observations from exercised guarantees.
- Map authoritative domain state, runtime uncertainty, ownership/fencing,
  recovery, artifact retention and staged implementation decisions.
- Hand off a first buildable slice only after its decisions are resolved and
  a reviewed tracker specification and blocker-linked tickets are approved.

## Non-goals for this discovery

Building the entire broker, provisioning PostgreSQL/Temporal, replacing Trae,
deploying a new UI, changing sibling repositories or assuming the pasted
architecture's citations establish observed behavior.

## Detailed implementation handoff

The owner requested [SPEC.md](SPEC.md) and a [proposed build graph](implementation-breakdown.md).
Recommended defaults are explicit and remain reviewable. Kata is now included
as a planning-ledger reuse candidate, not an assumed replacement for the broker.

## Detailed plan handoff

Owner requested the detailed plan and goal. [PLAN.md](PLAN.md) sequences
16 canonical implementation/evaluation tickets (10–25), each with requirements,
exclusions, blockers, interfaces and behavioral evidence. Existing decisions
03–09 remain prerequisites, not assumed resolved. No product implementation or
Git/deployment actions performed.

## Execution start

Owner approved recommended baseline. Decision record:
[20261001-approved-baseline.md](decisions/20261001-approved-baseline.md).
Ticket10 is claimed and assigned to an isolated implementer. The primary owns
intent and integrates only reviewed artifacts. Architecture approval is not
commit, push, paid-probe budget or production deployment authority.
