# Durable team control plane: wayfinding map

Label: wayfinder:map

## Destination

The complete persistent-team objective in [PLAN.md](PLAN.md): a real durable
PostgreSQL/Temporal kernel, optional ledger/workspace evaluation, live Trae
recovery, independent Codex/Claude lead demonstrations, and upgrade/restore
readiness. [STATE.md](STATE.md) records the current accepted execution boundary.
The original RFC remains preserved; planning alone does not achieve this goal.

## Discovery notes (historical)

Read EFFORT.md, the supplied RFC and the canonical engineering/skill crosswalk
at session start. Use research for external/runtime facts, grilling and domain
modeling for owner decisions, and prototype only for an approved runnable
question. The owner supplied PostgreSQL and Temporal as the preferred
architecture; implementation language, phase boundaries and production rollout
remain proposals until reviewed.

The supplied RFC will live in `../../rfcs/0001-durable-team-control-plane.md`.
Research tickets use the existing tracker format (`Type: task`) plus a
`wayfinder:research` label, since tracker.py accepts feature/bug/task, not
research/grilling types. Claim before research and record source provenance.

## Discovery decisions (historical)

- [Establish the actual Trae adapter surface](tickets/01-runtime-capabilities.md) — observed schemas expose thread/turn controls but leave start deduplication and live recovery unproven.
- [Ground the proposed durability contracts](tickets/02-durability-sources.md) — primary sources support database/workflow separation and require explicit external-effect reconciliation.

During discovery, these observations did not approve an implementation. The
owner subsequently approved the baseline below; live scope04 remains open.

## Frontier query

Use the ticket headers to select open, unclaimed tickets whose blockers are
resolved. Current frontier: Choose the broker/workflow stack, Define the live
Trae probe contract and Reconcile broker/Git authority. Later protocol decisions
are already ticketed with their prerequisite edges; they are not fog.

## Not yet specified

- Production service deployment and upgrade/rollback arrangements after the
  service language and first acceptance boundary are settled.
- Detailed bootstrap import of existing thread stores and planning efforts,
  after authority and binding identity are settled.
- Operator UI and takeover behavior after gateway ownership rules are settled.
- Backup/restore consistency, artifact lifecycle and long-lived workload
  placement beyond the first local recovery proof.

## Discovery exclusions (historical)

Deploying or implementing during discovery; model runtime forks, large messaging
infrastructure, multi-machine placement and a new graphical interface before
the recovery contract has been proven. These are outside this discovery's
destination rather than rejected long-term capabilities.

## Discovery specification (historical)

[SPEC.md](SPEC.md) now proposes concrete contracts and failure acceptance;
[implementation-breakdown.md](implementation-breakdown.md) supplies the build
graph. [Kata assessment](research/kata-assessment.md) adds a ledger reuse
decision before custom issue UI work. Prior frontier decisions remain open
until owner review; proposed defaults are not recorded approvals.

## Approved implementation frontier

Owner selected the recommended baseline; kernel decisions03/05/06/07/08/09
are resolved at the design-contract level. Tickets10–15 are accepted after independent review and fresh joint checks.
Ticket16 is claimed for complete kernel assembly. Live04 stays open. The approved first slice uses real PG and
Temporal with a fault runtime before provider/model expenditure.

## Kata evaluation disposition

Ticket17 accepted with real remote and broker-boundary evidence; Kata deferred
for first kernel. [Decision](decisions/20261001-kata-deferred-first-kernel.md).
Ticket18 retains an explicit conditional deferred path. Tickets10–15 are accepted and16 is active
in its isolated worktree; full objective and live/restore gates remain intact.

Ticket14 [joint acceptance manifest](evidence/ticket14-accepted-manifest.json)
fixes187 source/evidence files, both conformance reviews, separate shared-glue
review and fresh252-test gate. Ticket16 must verify this boundary before assembly;
its claim does not authorize live-provider or production execution.
