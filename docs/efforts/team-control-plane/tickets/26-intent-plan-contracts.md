# 26 intent-plan-contracts: exact admitted work and success contracts

Type: task
Status: open
Owner:
Blocked by: 16
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement immutable WorkRef/IntentRevision and separate AcceptanceContract and
PlanRevision at the broker public seam. Follow [SPEC.md](../SPEC.md) and
[v0.2 reconciliation](../v02-reconciliation.md). Source revisions, constraints,
dependencies, allowed effects and accepted subjects are exact admitted inputs.
Latest ticket text, status or model conversation confers no authority.

Dependencies bind prerequisite WorkRef + IntentRevision + accepted subject +
policy. A newly broker-admitted prerequisite IntentRevision suspends future
dependent grants until explicit
revalidation, preserving previously granted operations. Deleted or unavailable
sources retain admitted snapshots; new admission refuses missing evidence.
An unadmitted Kata edit/event is attention/proposal only and cannot change
these bindings. Plan edits change strategy separately from outcome, authority
and budget.

## Interface contract

Versioned `intent.admit`, `plan.admit` and `dependency.revalidate` commands use
existing authenticated idempotency/revision checks.
Authority follows the [caller/delegation map](../v02-reconciliation.md):
Owner-only absent an explicit current-lead bounded delegation; ordinary worker
credentials can propose but cannot admit intent, strategy or dependencies.
Reads expose canonical snapshot digests, admitted version, immutable acceptance/plan digests and exact
dependency disposition. Git input remains a supported source adapter during
migration; a legacy spec binding is preserved rather than substituted.
Plan records carry schema/interpreter version, stable step IDs, effect allowlist,
bounded loops/joins/repair and explicit resource ceilings. No code evaluation.

## Exclusions

No Kata daemon internals, runtime launch, global tracker migration, automatic
acceptance on source closure or workflow interpreter implementation here.
No change to accepted v1 specification/evidence identity.

## Acceptance

- [ ] Public behavioral RED then GREEN against real PostgreSQL: same exact admission is idempotent; changed payload conflicts; revision mismatch admits nothing.
- [ ] Mutable/deleted/missing source and cross-deployment same ticket ID cannot change or alias admitted intent.
- [ ] Prerequisite amendment blocks fresh dependent grant; an earlier operation receipt remains attributable and reconcilable; explicit new admission/revalidation is required.
- [ ] Plan amendment cannot change outcome, acceptance, epochs, completed step identity or cumulative limits; duplicate/invalid/cyclic/unbounded steps are refused.
- [ ] Valid negative research evidence can satisfy an expressly admitted research contract; fabricated PASS and evaluator-change proposal cannot satisfy the production contract (F31/F32).
- [ ] Legacy admissions retain exact proof bindings; migration closes ambiguous admission and preserves original snapshot/provenance.
- [ ] Independent Standards/Spec review, focused checks, exact source/evidence freeze and independent branch review accepted before integration.

## Comments

- 2026-10-03T17:30:00Z lead: Added from the owner's RFC0002 correction; affected execution awaits independent reconciliation critique. Open is not proof or runtime admission.
- 2026-10-03T17:53:11Z corenius-intent26: claimed until 2026-10-04T17:53:11Z

- 2026-10-03T20:44:51Z lead: Repair1 Standards PASS, Spec REJECT on old production proof bypassing a newer failed verification; four original consumer fixes conform. Exact269 records/18 source snapshots and both reports retained in ../evidence/intent26/repair1/. Bounded repair2 uses actual HTTP/PG RED/GREEN and postfailure/replay controls under ../decisions/20261003-intent26-review2-current-gate-repair.md. Still claimed; no active core source integration or acceptance.

- 2026-10-03T21:39:14Z lead: Repair2 both axes PASS,18 exact sources imported, fresh root focused/full gate70687 and20 controller tests PASS. Wholebranch632 terminal1 rejects accepted-intent amendment reexecution and source-amendment Plan ceiling bypass; exact report ../evidence/intent26/repair2/branch632/. Final bounded repair3 assigned under ../decisions/20261003-intent26-branch632-final-repair.md; source32ec033 unaccepted/unlanded, status remains claimed.

- 2026-10-03T22:21:58Z lead: Final repair3 has genuine public REDs for delegated Plan ceiling bypass, accepted revised-work admission, additional-generation storage and expired historical-verifier retirement. Fresh-work GREEN preserves old receipts/counters and proves new generation/candidate/binding/measured proof/acceptance plus explicit dependent revalidation. Compatibility/freeze and narrow Owner Plan authority assessment remain pending; no acceptance or landing. Retirement boundary is recorded in ../decisions/20261003-intent26-amendment-retirement.md.

- 2026-10-03T22:27:02Z lead: Independent narrow architecture assessment finds the provisional direct Owner Plan expansion exception broader than the canonical cumulative-limit contract. ../decisions/20261003-intent26-plan-ceilings-clarification.md preserves prior immutable ceilings for every caller; correction needs actual RED/GREEN within ongoing repair3. Exact static report/metadata retained; this is not frozen conformance or acceptance.

- 2026-10-03T22:50:09Z lead: Final repair3 freezes865 records/20 source postimages; author compatibility87/87 passes. Standards PASS; Spec REJECT P1 on accepted research amendments retaining old Worker/Verifier authority. Exact reports and root disposition retained in ../evidence/intent26/repair3/reviews/ and root-conformance-disposition.json. No repair3 active source import, root gate or landing; existing three-round allocation exhausted and read-only outcome architecture assessment underway. Status remains claimed;27 blocked.

- 2026-10-03T22:56:02Z lead: Architecture escalation completed; exact report/metadata retained in ../evidence/intent26/repair3/outcome-architecture/. Proposed derived current-accepted lifecycle and command-specific terminal/replay rules are in ../decisions/20261003-intent26-outcome-lifecycle-proposal.md; independent clean-context critique underway. Existing rounds1-3 remain exhausted; no new writer/allocation or code import yet.

- 2026-10-03T23:03:31Z lead: Independent outcome-lifecycle design critique PASS; exact report/metadata retained. Proposal adopted under ../decisions/20261003-intent26-outcome-lifecycle-allocation1.md. Explicit post-architecture allocation1 bounds one new isolated writer,10 test launches/3600s and one fresh conformance pass; prior rounds1-3 stay exhausted/rejected. Former author tree unchanged. No source acceptance/landing;27 blocked.

- 2026-10-03T23:06:12Z lead: Fresh clean-context writer /root/intent26_outcome_lifecycle1_author assigned sole tree corenius-codex-intent26-outcome-lifecycle1, branch codex-intent26/outcome-lifecycle1 at9d6b482. Exclusive local Cargo/PostgreSQL slot; root gates wait for explicit resource release. Exact assignment retained in ../evidence/intent26/outcome-lifecycle1-root-assignment.json. Former tree frozen; claim owner remains logical corenius-intent26.

- 2026-10-03T23:17:45Z lead: Post-architecture launch1/10 terminal101 at121.1s has two genuine public research-lifecycle/receipt REDs; root checked raw-log SHA and all59 archived input bytes. Owner-target missing-ID setup is not proof and is being strengthened. Observation ../evidence/intent26/outcome-lifecycle1-first-red-observation.json; no code import, root Cargo/PostgreSQL gate, conformance or acceptance.

- 2026-10-04T00:11:56Z lead: Post-architecture outcome-lifecycle1 frozen:22 exact sources,320 sealed records plus seal, final96/96 compatibility and strict lint pass. Root verified all selected hashes/preimages/final inputs, observation links, measured research bytes and14 retained database batches. Slot released with PostgreSQL stopped;17 roots/29 unknown runtime rows retained without reconciliation claims. Exact handoff ../evidence/intent26/outcome-lifecycle1/handoff.md and root-retention.json. One independent Standards/Spec pass now pending; active repair2 source unchanged, no root gate/acceptance/landing;27 blocked.

- 2026-10-04T00:28:08Z lead: Allocation1 both axes REJECT: Standards Important S1 (late valid Owner-target RED; current behavior not rejected), Spec P1 F1 (candidate-import research workspace loses exact intent pins; static, not runtime reproduced). Exact reports/metadata and root-conformance-disposition.json retained under ../evidence/intent26/outcome-lifecycle1/. Allocation1 stopped/exhausted, source unchanged/unaccepted/unlanded. Explicit follow-up proposal ../decisions/20261004-intent26-candidate-import-repair-proposal.md awaits independent critique and allocates no implementation yet. Original rejection/chronology and unknown effects remain unchanged;27 blocked.

- 2026-10-04T00:36:31Z lead: Independent candidate-import design/proof critique PASS; exact report/metadata retained at ../evidence/intent26/candidate-import2-design/. Adopted ../decisions/20261004-intent26-candidate-import-allocation2.md explicitly bounds one new isolated writer,6 tests/1800s/one conformance pass. F1 must get genuine public baseline RED before source change. S1 remains rejected allocation1 history; only unchanged inherited Owner-target guard receives explicit retrospective-proof exception. No new writer/source integration/acceptance yet;27 blocked.

- 2026-10-04T00:38:54Z lead: Assigned fresh sole writer /root/intent26_candidate_import2_author to codex-intent26/candidate-import2 at29bd30a in its new isolated tree. Exact assignment ../evidence/intent26/candidate-import2-root-assignment.json; exclusive local Cargo/PG slot,6 launch maximum/900s command/1800s cumulative. Both former author trees frozen. Root gates wait for explicit resource disposition and fresh conformance. No test outcome/source integration/acceptance yet.

- 2026-10-04T03:55:29Z codex-design: Prior implementation claim retired at safe terminal boundary by owner SQLite/controller-first priority. Dirty candidate-import2 tree and all evidence retained; no conformance/acceptance/landing inferred. Parked, unfinished; no new PostgreSQL work admitted. See ../decisions/20261004-owner-sqlite-controller-first-supersession.md.
