# Effort: durable team control plane

Status: approved baseline; admission implementation underway
Base: main at `963f58b31e518a8162dfe719e386ef9d9843321f`
Lead: Codex design session; replaceable in the proposed system

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
