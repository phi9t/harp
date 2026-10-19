# Kata evaluation source binding and authority mapping

Corenius commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`, with the
accepted ticket10 content manifest. Kata source read/built:
`b70a4306464e319701df30a3b92860289613b15a`.
This is the executable ticket17 evaluation mapping, not a deployed adapter or
policy migration. Canonical requirements remain SPEC and ticket17.

## Identity and immutable approval

`ledger_origin` is Kata's persisted `origin_instance_uid`, not the HTTP endpoint
or an issue short ID. A source binding key is:

`(ledger_origin, project_uid, issue_uid, approved_digest)`.

`approved_digest` is SHA256 of UTF8 canonical JSON (sorted object keys, compact
separators, no ASCII escaping). The version1 fixture approval contains origin,
project UID, issue UID, approved title/body, acceptance criteria, deliverables
and write scope. It deliberately excludes mutable claim/close state, local row
IDs, short IDs, source revision and projections. Approval is explicit fixture
owner input; issue creation or a ledger claim alone does not approve execution.
The captured bytes are fsynced, made read-only and committed in a disposable Git
repository. Broker input binds the actual immutable Git commit/path and byte
digest; subsequent ledger edits cannot rewrite that commit or admitted spec.

| Source/approval identity | Broker binding in the public fixture |
| --- | --- |
| `(ledger_origin, project_uid)` | Mission ID = UUIDv5(URL namespace, canonical tuple JSON) |
| Full source binding key | Task ID = UUIDv5(URL namespace, canonical tuple JSON) |
| Captured approval bytes | `GitInput.digest = approved_digest`, actual immutable Git commit and `approved.json` path |
| Approved task fields + Git input | Broker-derived TaskSpec digest; this differs intentionally from approved source-byte digest |
| Owner `AdmitTask` receipt | Immutable admitted broker task revision2 and approved TaskSpec digest |
| Actual polled Kata claim event UID | Assignment proposal only, zero admitted attempts until a separate owner decision |
| Owner decision ID + source event UID + source binding | Stable `kata.assign:` command key, scoped by broker principal/family |
| Explicit assignment receipt | Attempt ID, generation1, reservation ID and prepared operation ID |

The assignment decision is separately persisted/fsynced/read-only. It binds the
source event UID, approved digest, exact broker task and worker, expected task
revision2 and generation0. The operation key hashes ledger origin, event UID,
approved digest and owner decision ID. A fresh reader reloads both files; no
in-memory seen-event set provides execution authority. Duplicate source delivery
uses the same public broker command and returns its durable original receipt.

The fixture exports the mapping, approval fields, decision, actual source event,
immutable broker events and durable snapshots in
[ticket17-kata-broker-results.json](../evidence/ticket17-kata-broker-results.json).
The fixture databases/Git repository are disposable and removed; those IDs are
retained evidence rather than pointers to a live deployment.

## Authoritative versus projected fields

| Field | Authority |
| --- | --- |
| Mutable Kata title/body/source revision | Planning proposal; an edit is a proposed amendment requiring another explicit approval/admission |
| Kata claim/owner/assignment expiry | Planning assignment request; never a broker execution lease |
| Kata close/reason/test prose | Attributable completion claim; never broker verification or acceptance |
| Source canonical approval digest and its Git commit | Immutable approved planning input; admitted execution fields remain broker-owned |
| Broker admitted spec/revision, attempt/generation, budgets/reservations, operation/outbox | PostgreSQL broker authority |
| Namespaced `corenius_execution` metadata | Reconstructible projection of broker task/event/revision/state/attempt/generation; never an import/admission trigger |
| Ledger credentials | Kata API authority only; their tokens fail broker HTTP authentication |

An actual broker `attempt.assign` event is projected into guarded Kata metadata.
Its stable broker event ID/revision is the marker; repeated identical remote
metadata commands return `changed=false` and no new event. The single resulting
Kata metadata event is replayed into the evaluation receiver and classified as
a projection observation. Actual broker snapshots and mission admissions/capacity
remain unchanged. Production needs a durable outbox drain/reconciliation worker;
this fixture does not install one or claim failure-complete availability.

## Proven seam and remaining production cost

The Python fixture exercises Kata's actual remote HTTP service and the accepted
Rust broker HTTP implementation against two databases on one owned PostgreSQL
server. It does not use Go embedding hooks or a mock broker. The remote event,
claim, edit, close and guarded-metadata capabilities are measured independently
of the Go embedding transaction hooks described in pinned Kata documentation.
No Rust Kata client, provider/runtime adapter or production projection service
is claimed.

Actual concurrent duplicate claim-event delivery returns one exact receipt and
one attempt/operation/reservation/outbox entry. Source edit and close events
replayed twice do not change the admitted digest or broker revision/status.
Ledger credentials cannot execute a broker command. The approval remains exact
before and after source mutation and binding reload. A real stopped-ledger
`pg_dump`/drop/recreate/`pg_restore` preserves source origin/event UID and
projection. With the broker still running, replay returns the original receipt
and all four counts remain one. Owned container label and bound loopback port
are rechecked immediately before deleting only the ledger fixture database.

Inference: adoption still needs production credential provisioning, compatible
API/schema upgrades, source amendment review, durable projection delivery/reset
handling, and outage/restore reconciliation across the two stores. The measured
fixture size/run cost is evidence of a bounded evaluation, not a production
engineering estimate. Keeping the existing Git-planning/PG-execution baseline
for the first kernel avoids those additional services/seams while no custom
issue UI is being built. Defer is the proposed disposition pending independent
review; this document does not accept ticket17 or authorize adoption.
