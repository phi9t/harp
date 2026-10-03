# RFC 0002: Persistent engineering teams with replaceable agents

Status: captured owner design; owner requests adoption into active implementation.
Captured: 2026-10-03 (America/Los_Angeles).
Repository commit read: `264756edd550225449202b6f20ac4b560808d14e`.
Effort: [Durable team control plane](../efforts/team-control-plane/EFFORT.md).
Predecessor: [RFC 0001](0001-durable-team-control-plane.md).

## Preserved sources

- [Complete design RFC v0.1](0002-persistent-engineering-teams-design-source.md).
- [Complete accompanying design discussion](0002-persistent-engineering-teams-discussion-source.md).
- [Source hashes and provenance](0002-persistent-engineering-teams-sources.json).

Both source bodies are byte-for-byte copies. Historical status statements,
source references, and sandbox links are preserved as supplied; they do not
establish current repository state or newly verified external facts. Referenced
Harp, Armada, and Insula inputs were not independently retrieved in this capture.

## Owner adoption directive

The owner requested: “capture these design text in an RFC, instruct the active
working agent session to adopt”. Adopt this RFC as architectural input to the
ongoing six-phase effort. Preserve PostgreSQL + Temporal as the approved baseline
and the existing verification and landing gates. Reconcile refinements into the
canonical SPEC and blocker-linked tickets before implementing affected behavior.

Prioritize these distinctions:

1. Executions belong to durable scopes; attempts record creation provenance.
2. Role takeover, scope revocation, task revision, plan revision, workspace
   generation, and runtime incarnation have separate meanings and lifetimes.
3. Acceptance contracts define success; admitted plans define strategy.
4. PostgreSQL owns authority; Temporal coordinates through Activities;
   gateways provide attributed physical observations.
5. Each attempt has exactly one continuation owner. Runtime guarantees must
   be qualified against the concrete provider and version.
6. Verification and conditional integration bind exact immutable subjects,
   policy, environment, authority, and expected target predecessor.
7. Unknown execution and usage remain explicit and reserved; restore requires
   fresh authority and reconciliation before admission.
8. Armada/Insula supplies the execution seam. Kata Containers and jj remain
   optional qualification decisions, distinct from kenn-io/kata.

The supplied delivery slices and F01–F32 fault cases supplement the existing
implementation plan. They do not erase completed evidence, automatically resolve
tickets, or authorize evaluating generated workflow code in privileged workers.

## Required adoption handoff

The implementation lead should acknowledge receipt, record each refinement and
fault case as covered by exact evidence, an implementation gap, a decision gap,
or deferred scope, and update SPEC/tickets/STATE with resulting dependencies.
Surface conflicts with current contracts before changing schema or authority
semantics. In particular, inspect whether current lead takeover revokes worker
scope authority; this capture makes no claim about the implemented behavior.

Adoption is complete when the reconciliation is durable and affected worker
assignments reference the reconciled contracts. Publication of this RFC alone
is not implementation or qualification completion.
