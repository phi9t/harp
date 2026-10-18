# Ticket26 accepted-work lifecycle after architecture escalation

Status: proposed; independent design critique required before implementation
Commit read: `b2051cfd737ac5fd9fa359985b683bb55f96295f`

Final repair3 is rejected for accepted research amendments retaining old Worker
and Verifier authority. The original three-round allocation is exhausted, with
all865 frozen records and exact conformance reviews preserved. The independent
[assessment](../evidence/intent26/repair3/outcome-architecture/report.md) identifies
an inconsistent domain boundary across consumers, beyond the amendment predicate.
This proposal selects a coherent internal boundary under existing full-goal
Owner authority. It allocates no implementation yet.

## Representation and alternatives

Derive current accepted work in the existing broker transaction from the exact
Task + contract_current IntentRevision + immutable contract_outcomes subject.
Production and investigation_complete both close that admitted work. A small
internal lifecycle interface provides this classification to existing consumers;
no second authoritative completion table or generic production PASS is added.
Expose exact current intent/outcome/subject separately from historical accepted
subjects and production candidate/report fields in the existing contract read.
The raw admission-open flag remains a migration fact; effective eligibility also
requires current work to be unaccepted and dependencies valid.

An amendment-only research predicate would fix the reported token path while
leaving inconsistent mutations. A persisted completion boolean duplicates typed
accepted authority and requires another migration. Neither is selected.

A legacy accepted Task with no exact current v2 outcome stays a separate inert
historical case. Initial mapping changes no generation, candidate, receipt or
specification. Later work from mapping-only acceptance requires Owner authority.
A current accepted v2 outcome permits only the already authorized bounded
Owner/current-lead admission rules; research confers no additional delegation.

## Operation policy

| Operation | Current work accepted | After authorized new-work amendment |
| --- | --- | --- |
| Fresh Attempt, Episode, runtime/gateway/execution dispatch | Refuse new work; previously granted effects remain inspectable | Require fresh bounded current authority, exact revision and existing exclusivity/capacity checks |
| New outcome proposal/binding/verification/acceptance, artifact mutation, task-scoped mailbox mutation/delivery, verification-policy activation | Refuse consequential mutation, including cached Worker/Verifier work | Old generation credentials refuse immediately; exact new-revision evidence required |
| Plan admission or dependency revalidation for the completed intent | Refuse changing the completed work's strategy/authority | Existing admission, dependency and immutable prior-Plan ceiling rules apply |
| Exact accepted receipt replay and immutable history reads | Current authenticated, scoped actors may inspect existing records; no second acceptance or dispatch | Preserve old subject/receipt identity; old proof cannot accept the new intent |
| Physical inspect/Stop/log collection/Release and usage reconciliation | Preserve existing operation-specific current-actor authority and original effect attribution | Never revive old credentials, relabel effects or refund unknown/consumed usage |
| Legacy CancelAttempt/ReconcileAttempt mutation | Keep the existing production terminal refusal for both outcome types; cleanup uses the existing effect-level path | Existing cancellation/reconciliation fences remain; no cancelled work is reopened |
| Task-ID wait attention | Observe current typed or historical legacy acceptance as a wakeup hint | Prior recorded satisfaction remains history; no v2 dependency grant is released without exact revalidation |

Authentication, scope, recovery epoch, generation and command-payload checks
remain mandatory before receipt access. Exact terminal acceptance replay is
read-only and does not authorize cached proposal, verification, binding or
launch. Production evidence freshness and latest-failed-gate checks remain
production-specific; research does not create a candidate/report or production
verification binding solely to imitate that path.

An explicit later-work admission over accepted current production or research
atomically increments task generation before assignment, appends the exact
immutable amendment witness and suspends dependent fresh grants. Subsequent
assignment advances generation again. Preserve0009's task+generation uniqueness,
all original Attempts and accepted subjects, reservations, cumulative consumption,
unknown usage, writer isolation and the full historical Episode retirement proof.

An in-progress source-only revision remains an unaccepted revision: it does not
justify recycling Attempt/writer authority. Exact current-intent proof checks,
retained receipts, dependent suspension and prior immutable Plan ceilings remain.
This repair introduces no new material-amendment capability; existing Owner
admission limits continue to apply. ScopeEpoch/RoleEpoch redesign and other
material in-progress authority transitions remain ticket27, with no guessed
backfill or synthetic original Attempt for task-free lead jobs.

## Proof and execution boundary

Use the existing real HTTP/PostgreSQL and supervised fixture seams. The
[assessment matrix](../evidence/intent26/repair3/outcome-architecture/metadata.json)
requires actual research acceptance, terminal refusals, immediate amendment
fencing, stale-token refusals before assignment, fresh authority/measured proof/
acceptance, exact dependent revalidation, production parity, inert initial v1
mapping, original receipt/reservation retention, retirement refusals, idempotency
and concurrency ordering, and preserved nullable lead-job provenance. Research
negative results must be genuinely measured and satisfy their admitted contract.
Every changed behavior gets visible RED before implementation and GREEN after;
setup failures remain separate consumed attempts and retained evidence.

After critique, any implementation requires an explicit new bounded assignment
with its own resource/test limits and no reset of rounds1–3. Fresh independent
Standards/Spec, root focused/full gates and independent whole-branch review remain
required. No Trae/provider/remote run, Temporal run, protected-ref mutation or
scope27/interpreter implementation belongs to this follow-up.
