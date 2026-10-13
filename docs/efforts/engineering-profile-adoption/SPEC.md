# Engineering profile adaptation specification

## Intended outcome

One Corenius collaboration owner executes admitted skill-derived work through
bounded private Attempts. Matt skills guide investigation, decisions, specs,
tracer-bullet decomposition, implementation and two-axis review. They do not
create a nested competing dispatcher. Planning proposes strategy; runtime controls
admit authority. The originals are archived separately and remain unedited.

## Compatibility before implementation

The retained product is I2 (`controller`), with Kata as its work
ledger and retained Git seams. Git is required; optional jj support remains
pending its own qualification. I3 (`experiments/team_control_plane`) is explicitly
parked by the owner decision of 2026-10-04. The selected design instead proposes
GitMarkdownWorkStore, PostgreSQL authority and Temporal continuation. This plan
reopens that choice for ticket 01; it neither supersedes that owner decision nor
imports unfinished I3 wholesale. Use an append-only decision identifying the
chosen path and contracts it supersedes or preserves. If retaining I2, translate
approved mechanisms through focused Mechanism Promotion tickets. If selecting
PostgreSQL/Temporal, specify storage/ledger migration, ownership and rollback
before admitting implementation. Preserve all other portfolio work and evidence.

## Public seams and invariants

| Seam | Planned contract | Proof required |
| --- | --- | --- |
| Skill/profile and WorkStore | Pin effective skills, policy and recursive context; compile Markdown into immutable IntentRevision and AcceptanceContract; make PlanRevision provisional | Changed source/context/policy invalidates admission; malformed graph, missing context and duplicate identity fail closed |
| Collaboration broker | Exactly one coordinator admits claims, Attempts, reservations, grants and budgets in the selected authoritative store | Concurrent admission has one winner; revision race, cancellation, stale RoleEpoch/ScopeEpoch and rollback cannot leak capacity |
| Native gateway | Persist start intent and stable operation identity before dispatch; bind thread, turn, RuntimeIncarnation and workspace generation | Lost acknowledgment enters Quarantine; reconcile native identity before retry; unknown evidence cannot imply completion |
| TaskWorkspace / Insula | Attempt-private repository and writable generation; protected repository, peers and gate credentials absent | A worker cannot write protected refs/metadata; stale writer is fenced; dirty/live work is retained during cleanup |
| Verification and promotion | Freeze final integrated Candidate on current trunk; trusted acceptance recipe and separate reviewer; serial lane and conditional ref update plus unique receipt | Trunk movement requires rebuild/reverification; lost acknowledgment reconciles receipt; cancellation and commit admission have a defined order |
| Temporal continuation | Observe canonical store, preserve waits/cursors, schedule bounded Activities and reconcile before dispatch | Worker death, lost hints/replies, replay and Continue-As-New do not duplicate external effects or recreate authority |
| Evidence and operator control | Content identities, allowed projections, exact outcomes and unresolved operations; local landing, push, publication and deployment are separate grants | Failed/missing gates block acceptance; restart briefing retains jobs and unknown effects; no hidden spend or secret capture |

Temporal history is not the authorization database. A lease expiry is not proof
that an old process or external effect stopped. PostgreSQL and Git do not share
an atomic transaction: the durable operation journal and create-only Git receipt
must be reconciled under the sole-writer invariant. Database rollback requires
external-effect reconciliation and authority-domain fencing. Do not refund
capacity solely because an acknowledgment was lost.

## First deliverable and growth path

After ticket 01, qualify one deterministic scripted worker against a Trial
Repository: admitted source revision -> private Attempt -> sealed Candidate ->
protected verification -> conditional local landing -> reconciled receipt -> safe
cleanup. Test doubles are confined to qualification fixtures and explicitly
labeled; production runtime uses actual provider services. A failed check must
prevent promotion. Only then qualify a supervised native Codex path, recovery,
a second independent worker and research/prototype loops.

The Git receipt transaction proposed in the original design is a target, not a
claim that current land.py implements it. Preserve existing exact-tip gates while
the new gate is qualified. Tests execute without gate/signing credentials.
Candidate edits to policy, tests or skills cannot redefine their own trusted
acceptance recipe. Independent grading stays outside controller orchestration,
consistent with ADR0001; automatic qualification landing uses disposable owned
repositories. Operator checkout synchronization is separate and refuses unrelated
dirty work and untracked collisions.

## Qualification and limits

The labs prove 16 PostgreSQL kernel tests and seven Temporal tests, including
owned worker replacement and explicit SDK replay. Native evidence proves a
bounded Codex CLI completion and app-server observations. It does not qualify
active-turn restart, lost native acknowledgments, enforced provider spending,
production database restore, gate failover or unattended engineering readiness.
Historical app-server evidence lacks item categories required by the current
completion guard. TraeCLI live outcomes failed with an unknown cause.

Future proof must bind the exact Candidate, base, admitted contract, recipe,
environment and inputs. Keep failed histories, record skipped checks and require
all applicable gates. No live retry, production migration or hardware campaign is
implied by accepting this planning record. Operational deployment decisions belong
in ticket 09 after the substrate and native recovery proofs.
