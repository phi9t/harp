# 26 intent-plan-contracts: exact admitted work and success contracts

Type: task
Status: claimed
Owner: corenius-intent26
Blocked by: 16
Claimed: 2026-10-03T17:53:11Z expires 2026-10-04T17:53:11Z
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
