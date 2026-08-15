# Crouzeix Proof Reproduction Workstream

**Status:** planning gate

**Branch:** `feat/crouzeix-proof-reproduction`

**Worktree:** `.worktrees/crouzeix-proof-reproduction`

**Execution tracker:** [tracker.org](tracker.org)

**Approved experiment design:**
[Crouzeix proof reproduction design](../../superpowers/specs/2026-08-14-crouzeix-proof-reproduction-design.md)

**Product requirements:**
[Crouzeix proof reproduction PRD](../../superpowers/specs/2026-08-14-crouzeix-proof-reproduction-prd.md)

## Objective

Reproduce the public Crouzeix proof-search task prospectively, then run a
source-isolated archive search over dedicated mathematical experts. The
improved treatment uses the parent-selection algorithm documented in Harp's
[DGM algorithm derivation](../../../knowledge/darwin_godel_machine/03_algorithm_derivation.md#combined-selection-probability):

```text
eligible_i = alpha_i < 1
s_i = 1 / (1 + exp(-10 * (alpha_i - 0.5)))
h_i = 1 / (1 + functioning_children_i)
w_i = s_i * h_i
p_i = w_i / sum(w)
```

Two parents are sampled with replacement per generation from a precommitted
SHA-256-derived random stream. The archive retains every admitted mathematical
node, including lower-scoring children. DGM's functioning-child count is the
number of admitted child mathematical nodes; failed, malformed, and rejected
attempts do not increment it. Both draws in a generation use one immutable
pre-generation archive snapshot; child execution and admission begin only
after both selections are recorded.

This work does not claim an exact replay of the private historical run or
mathematical certification of either public proof.

## Authority

The workstream has one authority per artifact class:

| Artifact | Authority |
|---|---|
| Product intent, treatments, and claim ceilings | approved design and PRD linked above |
| Corrected domain model, target architecture, and execution contracts | this `design.md` |
| Executable work and status | `tracker.org` |
| Provider-neutral experiment machinery | `labs/crouzeix_proof_reproduction/` |
| Mutable attempts | ignored `labs/crouzeix_proof_reproduction/.runs/` |
| Sealed run evidence | create-only phase roots under `evidence/crouzeix_conjecture_reproduction/` |
| Mathematical synthesis | `knowledge/crouzeix_conjecture/` |
| Material reader claims | Crouzeix claim-evidence ledger |
| Derived reader/search artifacts | generated Atlas and search outputs |

The earlier
[`2026-08-14-crouzeix-proof-reproduction-issues.md`](../../superpowers/plans/2026-08-14-crouzeix-proof-reproduction-issues.md)
records the first decomposition. It is not an active issue tracker after this
workstream lands.

This active workstream corrects one clause in the approved design: reaching
`alpha_i = 1` freezes and excludes that node, but does not stop search over
other eligible nodes. The early-stop wording in the approved design is
incompatible with the paper-level eligible-set algorithm the user selected.
The correction is isolated to search control; budgets and claim ceilings do not
change. It also disambiguates the approved design's overloaded use of "expert
node": attempts, expert results, immutable mathematical nodes, archive entries,
and candidate projections are separate records with separate identities.

## Ticket Discipline

Every action that can change code, content, evidence, run state, review state,
or landing state has one stable `CPFR-*` ticket in `tracker.org`.

Execution rules:

1. Do not perform untracked implementation work.
2. Set exactly one ticket to `IMPLEMENTING` before editing its owned files.
3. Start a fresh agent session for each ticket and provide only this design,
   the PRD, and that ticket.
4. Do not execute tickets with nonterminal `DEPENDS_ON` properties. `DONE` and
   conditionally justified `CANCELED` are terminal; all other states block.
5. Do not run tickets concurrently when their owned files overlap.
6. Record red and green test commands under `Verification Evidence`.
7. Record live provider calls, operator interventions, and resource blocks in
   both the ticket and run artifacts.
8. A `TODO` ticket uses `unassigned` for assignment properties. Claiming it
   replaces those sentinels with a concrete owner, agent run, and model.
9. Every repository ticket through the final tracked closeout produces its own
   focused commit because its tracker state is tracked, even when the product
   action changes only ignored runtime state. Stage only its owned files, its
   own tracker subtree, and the shared receipt. CPFR-055 and CPFR-056 are the
   predeclared post-freeze operator tickets described below; they produce
   ignored terminal receipts and no tracked commit.
10. `docs/import-receipt.md` is a serially shared tail file: refresh it after
    the ticket's other tracked files settle, then include it in that ticket's
    commit.
11. Every commit ends with
    `Co-authored-by: TRAE CLI <noreply@bytedance.com>`.
12. `DONE` means acceptance criteria and verification have passed. A failed or
    blocked experiment may still complete its execution ticket when its
    failure receipt is complete and correctly classified.
13. A review finding that requires unplanned tracked edits gets a new
    `CPFR-R###` repository ticket before the first edit. The review ticket may
    not absorb an untracked repair.
14. If a file contains pre-existing or later-ticket hunks, whole-file staging
    is forbidden. CPFR-022 assigns every hunk a stable patch ID built from the
    base blob ID, normalized hunk header, and context SHA-256. The ticket owner
    stages only assigned patch IDs with interactive patch staging, verifies the
    cached diff against the assignment map, and proves all unowned hunks remain
    unstaged and byte-identical. For an initially untracked file, the base blob
    is Git's empty blob
    `e69de29bb2d1d6434b8b29ae775ad8c2e48c5391`; run
    `git add -N <path>` before `git add -p <path>`.

Unless a ticket specifies a product-facing subject, its exact closeout subject
is `chore(crouzeix): complete <ticket-id>`. CPFR-025 uses its explicit planning
subject. CPFR-055 and CPFR-056 do not create commits.

CPFR-009, CPFR-024, and CPFR-025 form the one exceptional planning commit group
`planning-gate-001`: authoring and audit are independently ticketed actions, but
their two new files do not exist in the parent commit and therefore land
atomically through CPFR-025. Their `DONE` state means accepted into that named
group. CPFR-025's staged `DONE` transition and commit creation are one atomic
closeout operation: the transition is effective only if the commit succeeds.
No other ticket may join the group.

The normal repository-ticket lifecycle is:

```text
TODO -> IMPLEMENTING -> REVIEW -> DONE
          |           |
          +-> BLOCKED-+
BACKLOG -> TODO | CANCELED
```

`BLOCKED` records a reversible implementation blocker and its evidence;
`CANCELED` is reserved for owner-directed scope removal. Runtime outcomes such
as `failed`, `blocked_resource`, and `not_promoted` do not by themselves make
the governing repository ticket `BLOCKED`.

## Runtime Ticket Contract

Repository tickets govern implementation, repository review, and operations.
Every live provider call, mathematical evaluation/review, formal attempt,
selector draw, and operator intervention also creates one immutable run-local
ticket before the action:

```text
tickets/<ticket_id>/ticket.json
tickets/<ticket_id>/ticket_events.jsonl
```

Tasks that require run-local tickets include:

- every root expert;
- every descendant expert;
- every proof-progress evaluator;
- every parent-selection draw;
- candidate freeze;
- resource/operator intervention;
- each correctness reviewer;
- finding reconciliation;
- mechanism classification; and
- each formal-verification attempt.

The ticket is published before the governed action. In particular, one parent
selection generation creates two draw tickets from the immutable pre-draw
inputs `(run_id, generation, draw_index, seed, archive_digest)`. Each draw
ticket exists before its SHA-256 preimage is hashed. The selected parent is an
output event, never an input used to create its own ticket.

`ticket.json` is create-only and contains:

- ticket, run, task-kind, node, parent, generation, direction, and role IDs;
- objective and expected deliverable;
- dependency ticket IDs;
- exact context, schema, prompt, and parent-artifact digests;
- allowed tools and forbidden sources;
- timeout and resource limits;
- owner type (`expert`, `evaluator`, `reviewer`, `orchestrator`, or `operator`);
- creation timestamp; and
- initial state.

`ticket_events.jsonl` is append-only. Its closed state transitions are:

```text
created -> admitted | canceled
admitted -> running | canceled
running -> completed | failed | timed_out | blocked_resource | canceled
completed -> accepted | rejected | superseded
```

A call request must bind `ticket_id` and the SHA-256 of `ticket.json`. A call
receipt must bind the same values. An expert result, evaluator result,
selection event, intervention, or review without a valid ticket is invalid
run evidence.

Dynamic task creation is deterministic:

- root expert tickets are created during run preparation;
- evaluator tickets are created after a functioning expert result is sealed;
- selection-draw tickets are created from pre-draw generation inputs;
- child expert tickets are created from the selected parent and direction; and
- review tickets are created only after candidate freeze.

Runtime ticket IDs are derived from task kind and stable coordinates rather
than timestamps. Examples are
`expert-g0-function-theory`, `evaluate-node-g0-function-theory-e1`,
`select-g1-d0`, and `expert-g1-d0`. A collision is a hard error. Unplanned
operator interventions use the next append-only event sequence within a
pre-created operator ticket; they do not mint a replacement history.

The run receipt reconciles ticket counts against attempts, provider calls,
selection draws, evaluator records, reviews, and terminal ticket states.

Published evidence is phase-separated so later review cannot mutate a sealed
generation tree:

```text
evidence/crouzeix_conjecture_reproduction/
  runs/<run_id>/
  reviews/<review_id>/
  formal/<attempt_id>/
  manifests/<publication_id>.json
```

Every leaf is create-only. Aggregate manifests are new content-addressed
publications rather than in-place edits to an earlier sealed leaf.

The H and O0 attempts predate this runtime-ticket contract. Their original
timestamps and artifacts are immutable historical evidence; no later ticket is
represented as having existed before those actions. A dedicated migration
ticket records a typed `legacy_pre_ticket_contract` exception, links the
original intervention receipts, and brings sealing/accounting under the new
validator without fabricating provenance.

## Current State

Completed and committed:

- `bf9e992`: research reconstruction, experiment design, and packet
  registration;
- `672a5cd`: strict historical protocol and phase schemas;
- `68e4b9f`: create-only historical and flat-orchestration runner;
- `48cc344`: live-run boundary hardening;
- `aec729d`: approved DGM-selected dedicated-expert design.

Preserved baseline attempts:

- `historical-001`: resource-interrupted historical arm, no candidate;
- `orchestrated-001`: superseded flat arm; three workers and controller
  completed, redirect interrupted, no candidate.

Uncommitted implementation exists for the DGM frontier. It is work in progress,
not design authority. A dedicated reconciliation ticket inventories every dirty
path, records its provenance and observed tests, and maps each hunk to exactly
one later owner ticket before any implementation resumes. No partial hunk is
accepted merely because an earlier local test passed.

`master` is not currently an ancestor of this branch. Integration tickets must
reconcile the latest `master` before final verification and landing.

## Domain Model

| Record | Exists when | Identity | Mutable? | DGM input? |
|---|---|---|---|---|
| Expert attempt | a runtime ticket is admitted | `attempt_id` plus ticket/event chain | append-only execution history | no |
| Expert result | provider output passes strict schema validation | `expert_result_sha256` | no | no |
| Admission decision | a strict expert result exists | `admission_decision_sha256` | no | no |
| Mathematical node | admission is accepted | `node_id` and `node_artifact_sha256` | no | by reference |
| Node evaluation | evaluator output or conservative fallback is sealed | `node_evaluation_sha256` | no | through reconciliation |
| Reconciliation | both node evaluations exist | `reconciliation_sha256` | no | score and findings |
| Archive entry | node and reconciliation join successfully | `archive_entry_sha256` | no | yes |
| Selection event | a pre-created draw ticket executes | event digest under one snapshot | no | records output |
| Candidate projection | an archive entry is score-complete | `candidate_projection_sha256` | no | no |

Only attempts have lifecycle transitions. Every other row is an immutable
value or append-only fact. A later record may reference an earlier digest but
cannot add fields to, replace, or change the meaning of the earlier record.

### Expert attempt

An expert attempt is one ticketed execution of a root or descendant
mathematical task. The attempt owns execution facts: runtime ticket, context,
provider call and receipt, raw output, usage, terminal status, and any resource
or operator intervention.

An attempt may fail, time out, be resource-blocked, return malformed output, or
produce a strict expert result. None of those outcomes is itself a
mathematical node. Attempt identity and state are governed by the runtime
ticket contract.

### Expert result

An expert result is a strict, schema-valid provider response whose identity and
selected direction agree with its context. It contains proposed mathematical
content but has not yet passed admission. Raw malformed output remains attempt
evidence and is never called an expert result.

The harness assigns `proposed_node_id` before the attempt so tickets, contexts,
and descendants have deterministic coordinates. Before admission it is only a
proposed identifier. An accepted admission promotes that exact value to the
mathematical node's `node_id`; a rejected result never creates a node with that
ID.

The expert-result record contains:

```text
schema_version
run_id
attempt_id
ticket_id
proposed_node_id
parent_node_id | null
parent_node_artifact_sha256 | null
generation
expert_role
selected_direction_id
mathematical_payload
expert_result_sha256
```

The result digest covers every preceding field. Provider call, context, and
leakage-audit digests remain in the admission and node provenance records
rather than in model-generated mathematical output.

The result has one tagged endpoint variant:

```text
endpoint = {
  kind: "candidate_proof",
  text: nonempty string
}
```

or:

```text
endpoint = {
  kind: "blocker",
  text: nonempty string
}
```

Nullable sibling fields are forbidden. The structural variant does not decide
whether the result is functioning.

### Admission decision

Admission is a pure, immutable decision over one expert result plus its
leakage and provenance checks. It is `accepted` only when the result is:

- bound to one valid terminal expert attempt;
- schema-valid and compliant with the declared leakage boundary;
- internally consistent with its context, parent, generation, role, and
  selected direction; and
- mathematically functioning, meaning it contains either a concrete mechanism
  plus at least one proved statement, or a precise falsifiable blocker plus a
  materially new direction.

Every strict expert result receives exactly one admission decision. Its closed
outcome is:

```text
accepted
rejected_leakage
rejected_provenance
rejected_nonfunctioning
```

The decision binds the run, attempt, ticket, context, call receipt, expert
result, leakage-audit, and prospective node IDs. It never contains
`node_artifact_sha256`: the accepted decision is hashed first and its digest is
then included in the mathematical node. This one-way edge prevents a digest
cycle.

The admission-decision record contains:

```text
schema_version
run_id
attempt_id
ticket_id
proposed_node_id
expert_result_sha256
context_sha256
call_receipt_sha256
leakage_audit_sha256
outcome
reason_code
diagnostic_detail | null
admission_decision_sha256
```

The decision digest covers every preceding field. `accepted` uses the closed
reason code `admission_criteria_satisfied`; each rejected outcome has a
corresponding closed reason code. Any bounded diagnostic detail is included in
the record and its digest.

An accepted decision creates exactly one mathematical node. A rejected
decision records one outcome and creates none. An attempt that fails, times
out, is resource-blocked, or returns malformed output produces no strict expert
result and therefore no admission decision. Every such attempt remains durable
attempt evidence but never enters the mathematical archive.

### Mathematical node

A mathematical node is the immutable admitted mathematical artifact used as a
parent in proof search. Its canonical record contains:

```text
schema_version
run_id
node_id
theorem_sha256
leakage_tier
parent_node_id | null
parent_node_artifact_sha256 | null
generation
expert_role
selected_direction
selected_direction_sha256
source_attempt_id
source_ticket_id
source_expert_result_sha256
source_context_sha256
source_call_receipt_sha256
admission_decision_sha256
mathematical_payload
mathematical_payload_sha256
node_artifact_sha256
```

`mathematical_payload` contains only:

```text
proof_family
mechanism
proved_statements[] = {
  statement_id,
  statement,
  justification,
  depends_on_statement_ids[]
}
unproved_obligations
circularity_risks[] = {
  risk_id,
  statement,
  locator
}
proposed_directions
endpoint = {
  kind: "candidate_proof" | "blocker",
  text
}
confidence_basis
```

`selected_direction` is a lineage record outside the mathematical payload:

```text
direction_id
kind = root_task | obligation | evaluator_finding | proposed_direction
statement
strength = root | theorem_strength | critical | major | local | minor | proposed
recommended_role
source_node_artifact_sha256 | null
source_reconciliation_sha256 | null
```

For a root, both source digests are null and `kind = root_task`. For a child,
the node source digest is required; the reconciliation source digest is also
required when the direction came from an evaluator finding. The direction
digest is SHA-256 over canonical JSON of this complete record.

The payload digest is SHA-256 over canonical JSON of
`mathematical_payload`. The artifact digest is SHA-256 over canonical JSON of
the complete node record except the `node_artifact_sha256` field itself.
Canonical JSON uses sorted keys, UTF-8, no insignificant whitespace, and
rejects non-finite numbers. Timestamps, evaluator output, scores, child counts,
selection events, review outcomes, and mutable filesystem paths are excluded
from both identities.

`node_id` is the stable run-local coordinate used in lineage and tickets;
`node_artifact_sha256` is the immutable content-and-provenance identity.
Distinct nodes may have equal payload digests, but duplicate artifact digests
within one run are rejected. Parent linkage is part of artifact identity, so a
mathematically identical payload reached from another parent remains a
different node.

### Node evaluation and reconciliation

Each admitted mathematical node receives two independent node evaluations.
They reference `node_artifact_sha256`, but their model-visible contexts contain
only the theorem and mathematical payload; lineage, role, treatment, attempt
metadata, scores, and other evaluator output are absent.

The model returns only an evaluation payload containing ten probe decisions and
findings. The harness wraps it in an immutable node-evaluation envelope that
binds evaluator index, runtime ticket, context, call receipt,
`node_artifact_sha256`, `mathematical_payload_sha256`, evaluation-payload
digest, and node-evaluation artifact digest. The model is never asked to echo a
hidden node identity.

The node-evaluation envelope contains:

```text
schema_version
evaluation_id
evaluator_index
ticket_id
node_artifact_sha256
mathematical_payload_sha256
context_sha256
source_kind = provider_output | conservative_fallback
call_receipt_sha256 | null
terminal_ticket_event_sha256
evaluation_payload
evaluation_payload_sha256
node_evaluation_sha256
```

The artifact digest covers every preceding field. Evaluator indices are exactly
one and two, and their ticket, context, call, payload, and artifact digests must
all be distinct even when their judgments happen to be equal. Provider output
requires an accepted completed call receipt and terminal ticket event.
Conservative fallback requires a terminal failed, timed-out, or
resource-blocked ticket event; it binds a failed call receipt when one exists
and uses null only when no provider call began. Its ten probe statuses are
`insufficient_evidence`. The fallback is harness evidence, never attributed to
the evaluator model.

Reconciliation is a separate immutable record that binds the node artifact and
both node-evaluation artifact digests. It also binds its own digest and the
ordered probe outcomes and namespaced findings:

```text
schema_version
reconciliation_id
node_artifact_sha256
mathematical_payload_sha256
node_evaluation_sha256s[2]
probes[10]
unanimous_pass_count
disagreement_count
evaluator_findings
reconciliation_sha256
```

The two evaluation digests are ordered by evaluator index. The reconciliation
digest covers every preceding field. Its authoritative score is the integer
`unanimous_pass_count` in `[0, 10]`. The DGM value
`alpha_i = unanimous_pass_count / 10` is derived exactly; a JSON float is never
score authority. Evaluator findings belong to reconciliation, not to the
mathematical node.

### Archive entry

An archive entry is the immutable pairing of one mathematical node with one
evaluation reconciliation:

```text
node_id
node_artifact_sha256
mathematical_payload_sha256
reconciliation_sha256
unanimous_pass_count
candidate_proof_sha256 | null
archive_entry_sha256
```

The archive-entry digest covers every preceding field. The candidate digest is
present only for a `candidate_proof` endpoint and is computed from the exact
UTF-8 endpoint text; a blocker endpoint requires null.

The archive uses `keep_all`: every admitted node receives an archive entry,
including children that score below their parents. DGM selection operates on
archive entries. Functioning-child count, eligibility, weights, probabilities,
and selection history are projections from immutable archive entries and
events; they are not fields of the mathematical node or archive entry.

### Record graph and durable layout

The authoritative relation is:

```text
ExpertAttempt -> ExpertResult -> AdmissionDecision -> MathematicalNode
MathematicalNode -> NodeEvaluation[1]
MathematicalNode -> NodeEvaluation[2]
NodeEvaluation[1,2] -> Reconciliation
(MathematicalNode, Reconciliation) -> ArchiveEntry
ArchiveEntry -> SelectionEvent -> child ExpertAttempt
ArchiveEntry(score_complete) -> CandidateProjection -> CorrectnessReview
```

Every arrow is an explicit identifier-and-digest reference. Containment on
disk does not imply identity or authority. The target run layout is:

```text
attempts/<attempt_id>/
  attempt.json
  context.json
  expert_result.json | terminal_failure.json
  provider_call/
  receipt.json
admissions/<attempt_id>.json
mathematical_nodes/<node_id>/
  node.json
  mathematical_payload.json
  inventory.json
node_evaluations/<node_id>/
  evaluator-1/
  evaluator-2/
  reconciliation.json
archive_entries/<node_id>.json
candidate_projections/
  index.json
  <candidate_sha256>.tex
frontier_events.jsonl
selection_events.jsonl
frontier_snapshot.json
run_receipt.json
```

The attempt ledger references every attempt regardless of outcome. The
mathematical-node, evaluation, and archive-entry paths exist only after their
respective immutable records are accepted. `frontier_snapshot.json` is a
deterministic read model over archive entries, parent links, and append-only
events; it is never an identity source.

### Proof-progress score

Two fresh evaluators independently answer ten route-neutral probes. A probe
passes only when both answer `pass`. `alpha_i` is derived from the reconciled
integer pass count. The outer harness computes it; experts and evaluators never
receive or emit the scalar.

### Parent-selection random stream

The selector uses the paper-level DGM weights and a study-specific deterministic
replacement for the paper's runtime RNG. Its exact preimage is:

```text
b"crouzeix-frontier/v1\0"
|| ascii_decimal(seed) || b"\0"
|| ascii_decimal(generation) || b"\0"
|| ascii_decimal(draw_index)
```

All integers are nonnegative, base-10 ASCII without signs or leading zeros.
`generation` starts at one and `draw_index` is zero or one. The SHA-256 digest
is interpreted as an unsigned big-endian integer and divided by `2**256`.
Eligible node IDs are closed ASCII identifiers sorted by byte value.

The selector authority uses Python standard-library `decimal` arithmetic, not
platform `libm`:

```text
Context(
  prec=80,
  rounding=ROUND_HALF_EVEN,
  Emin=-999999,
  Emax=999999,
  capitals=1,
  clamp=0,
)
```

`alpha` enters as the exact decimal ratio `unanimous_pass_count / 10`. Constants
are constructed from decimal strings or integers. The context computes `exp`,
division, normalization, and cumulative sums in sorted node-ID order. The
uniform is `Decimal(unsigned_digest_integer) / Decimal(2**256)` in the same
context. Authoritative terms and interval bounds are canonical normalized
decimal strings; JSON floats, when emitted for convenience, are non-authority
display fields. Intervals are left-closed and right-open, with the final
eligible node as the rounding fallback. Golden vectors for every configured
generation include the complete preimage, hash, unsigned integer, decimal
uniform, terms, cumulative bounds, and selected node.

One generation first freezes:

```text
archive_snapshot_digest
eligible_node_ids
eligible_archive_entry_sha256s
unanimous-pass-count vector
functioning-child-count vector
weight/probability vector
```

Both draw tickets bind those same values. No attempt, child admission, snapshot
rewrite, or child-count change is allowed between draw zero and draw one.

### Frontier direction

An unresolved obligation, evaluator finding, or expert-proposed direction.
Priority order is:

1. theorem-strength obligation;
2. critical evaluator finding;
3. major obligation;
4. major evaluator finding;
5. local obligation;
6. proposed direction.

Stable direction ID breaks ties.

### Promotion boundary

An archive entry is `score_complete` when its reconciliation has
`unanimous_pass_count = 10` and its mathematical node contains a
content-addressed candidate proof. `score_complete` is a derived qualification,
not a mathematical-node state and not proof certification. The entry becomes
parent-ineligible under the paper's `alpha_i < 1` rule and remains in the
archive. Search continues over all other eligible entries until the fixed
generation/call/node budget is exhausted or the eligible set is empty.

Attempts alone have lifecycle states. Mathematical nodes and archive entries
are immutable values. Frontier records are append-only facts:

```text
attempt_terminal
admission_accepted | admission_rejected
archive_entry_created
selection_recorded
candidate_projected
```

Repeated selection creates repeated selection events; it never changes a node
to a `selected` state. `superseded` describes a treatment or attempt
disposition, not a mathematical node. `complete` is reserved for a later
independent correctness-review outcome and is never inferred from the DGM
score.

A candidate projection is a create-only review input containing the exact
candidate bytes and their digest, source `node_artifact_sha256`, and source
`reconciliation_sha256`:

```text
schema_version
projection_id
source_node_id
source_node_artifact_sha256
source_reconciliation_sha256
candidate_sha256
candidate_byte_count
candidate_projection_sha256
```

The projection digest covers the metadata fields through
`candidate_byte_count`; the candidate bytes live in a separate
content-addressed file. Every score-complete
candidate projection is independently reviewed, and review records bind both
the projection and candidate digests.
The harness does not choose one winner using `alpha`, because all such nodes
have the same search score. If one repair is allowed, the target is selected
model-independently from candidates whose frozen outcome is `incomplete`.
Candidates are ordered by:

```text
(critical finding count,
 theorem-strength obligation count,
 major finding count,
 minor finding count,
 candidate SHA-256)
```

and the lexicographic minimum is selected. `complete`, `invalid`,
`indeterminate`, and no-candidate outcomes are not repair targets. The repaired
bytes receive a new digest and two fresh correctness reviews.

## Module Boundaries

The target file structure is:

| Module | Responsibility |
|---|---|
| `protocol.py` | shared strict JSON, IDs, paths, and run-spec boundaries |
| `runner.py` | create-only provider call, event capture, timeout, usage receipt |
| `frontier.py` | pure evaluation reconciliation, archive-entry construction, DGM selection, and direction ranking |
| `expert_contracts.py` | expert/evaluator contexts, output validation, admission, and mathematical-node construction |
| `frontier_store.py` | append-only attempts/events, mathematical nodes, evaluations, archive entries, snapshots, and candidate projections |
| `expert_runner.py` | bounded orchestration over the modules above |
| `prepare_frontier.py` | create-only expert-frontier run preparation |
| `run_frontier.py` | thin operator CLI |
| `review_contracts.py` | anonymized correctness, repair, mechanism, and formal-status contracts |
| `review_runner.py` | ticketed create-only correctness, repair, and classification calls |
| `guided_runner.py` | create-only Arm G preparation and one-call reconstruction |
| `formal_receipt.py` | strict candidate-specific formal-attempt receipt validation |
| `seal_run.py` | deterministic evidence publication and secret scan |

The current uncommitted `expert_runner.py` is too broad. The implementation
plan splits contracts and persistence out before adding more behavior.

The public module and CLI boundaries to preserve are:

```python
# tickets.py
parse_tracker(path: Path) -> Tracker
validate_tracker(path: Path, *, claim_id: str | None = None) -> None
create_runtime_ticket(root: Path, ticket: Mapping[str, object]) -> TicketRef
append_ticket_event(root: Path, ticket_id: str, event: Mapping[str, object]) -> None
validate_runtime_tickets(root: Path) -> TicketSummary

# frontier.py: pure, with no filesystem or provider imports
reconcile_evaluations(node_artifact_sha256: str,
                      first: Mapping, second: Mapping) -> dict
make_archive_entry(node: Mapping, reconciliation: Mapping) -> dict
selection_terms(archive: FrontierArchive) -> list[dict]
select_parents(archive: FrontierArchive, *, generation: int, seed: int,
               count: int = 2) -> dict
assign_directions(parent_ids: Sequence[str], nodes: Mapping,
                  reconciliations: Mapping) -> list[dict]

# expert_contracts.py
build_expert_context(...) -> dict
build_evaluator_context(...) -> dict
validate_expert_result(...) -> dict
validate_evaluator_result(...) -> dict
decide_admission(attempt: Mapping, result: Mapping,
                 provenance: Mapping) -> dict
build_mathematical_node(admission: Mapping, result: Mapping,
                        provenance: Mapping) -> dict

# frontier_store.py
open_frontier_store(run_dir: Path) -> FrontierStore
FrontierStore.append_attempt(...)
FrontierStore.record_admission(...)
FrontierStore.materialize_mathematical_node(...)
FrontierStore.materialize_node_evaluation(...)
FrontierStore.materialize_archive_entry(...)
FrontierStore.project_snapshot(...)
FrontierStore.freeze_candidate(...)
FrontierStore.reconcile_run(...)
```

Operator commands are fixed before implementation:

```sh
python3 labs/crouzeix_proof_reproduction/tickets.py \
  validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
python3 labs/crouzeix_proof_reproduction/prepare_frontier.py \
  --run-dir labs/crouzeix_proof_reproduction/.runs/expert-frontier-001 \
  --historical-prompt PROMPT_PATH --cli TRAECLI_PATH \
  --model gpt-5.6-sol --timeout-seconds 3600
python3 labs/crouzeix_proof_reproduction/run_frontier.py \
  roots --run-dir labs/crouzeix_proof_reproduction/.runs/expert-frontier-001
python3 labs/crouzeix_proof_reproduction/run_frontier.py \
  evaluate-roots --run-dir labs/crouzeix_proof_reproduction/.runs/expert-frontier-001
python3 labs/crouzeix_proof_reproduction/run_frontier.py \
  generations --run-dir labs/crouzeix_proof_reproduction/.runs/expert-frontier-001
python3 labs/crouzeix_proof_reproduction/run_frontier.py \
  finalize --run-dir labs/crouzeix_proof_reproduction/.runs/expert-frontier-001
python3 labs/crouzeix_proof_reproduction/run_frontier.py \
  check --run-dir labs/crouzeix_proof_reproduction/.runs/expert-frontier-001
python3 labs/crouzeix_proof_reproduction/review_runner.py \
  correctness --review-dir REVIEW_DIR --candidate-index CANDIDATE_INDEX
python3 labs/crouzeix_proof_reproduction/review_runner.py \
  reconcile --review-dir REVIEW_DIR
python3 labs/crouzeix_proof_reproduction/review_runner.py \
  repair --review-dir REVIEW_DIR --decision REPAIR_DECISION
python3 labs/crouzeix_proof_reproduction/review_runner.py \
  classify --review-dir REVIEW_DIR --reference-card REFERENCE_CARD
python3 labs/crouzeix_proof_reproduction/review_runner.py \
  check --review-dir REVIEW_DIR
python3 labs/crouzeix_proof_reproduction/guided_runner.py prepare \
  --run-dir labs/crouzeix_proof_reproduction/.runs/guided-001 \
  --mechanism-card CARD_PATH --leakage L3
python3 labs/crouzeix_proof_reproduction/guided_runner.py run \
  --run-dir labs/crouzeix_proof_reproduction/.runs/guided-001
python3 labs/crouzeix_proof_reproduction/formal_receipt.py \
  validate --attempt-dir FORMAL_ATTEMPT_DIR
python3 labs/crouzeix_proof_reproduction/seal_run.py \
  publish --run-dir RUN_DIR --destination EVIDENCE_DIR \
  --manifest MANIFEST_PATH
python3 labs/crouzeix_proof_reproduction/seal_run.py \
  check-run --run-dir RUN_DIR
python3 labs/crouzeix_proof_reproduction/seal_run.py \
  check-evidence --evidence-dir EVIDENCE_DIR --manifest MANIFEST_PATH
```

Every mutating CLI has a provider-neutral `check` or `validate` subcommand.
`mise run verify` invokes checks and fake providers only.

The frontier CLI is resumable only at those ticket boundaries. Each subcommand
requires the prior phase's terminal receipt, refuses an existing terminal
receipt for its own phase, and rejects undeclared files or partial later-phase
state. It never resumes a provider call or reuses a node/call/ticket ID.

## Milestones

### M0: Historical contract and baselines

Reconstruction, historical execution, and flat baseline are complete and
preserved, including interventions.

### M1: Provider-neutral DGM frontier

Complete strict contracts, pure selector, append-only state, provider adapter,
orchestration, failure accounting, and fake-provider verification.

### M2: Live expert-frontier attempt

Prepare, inspect, execute, monitor, and validate one immutable `E` run under
the fixed model, seed, access policy, and resource budget.

### M3: Evidence and mathematical review

Seal all three attempts, verify them offline, perform blind correctness review,
classify mechanism similarity only after correctness findings freeze, and
resolve the conditional mechanism-guided diagnostic under a distinct leakage
label.

### M4: Reader integration and landing

Register protocol/results documents and claims, regenerate derived outputs,
reconcile latest `master`, run the release gate, and fast-forward local
`master` without pushing.

## Global Verification

Focused tests while iterating:

```sh
python3 -m unittest discover \
  -s labs/crouzeix_proof_reproduction/tests -v
cargo test -p harp sources::crouzeix_reproduction::tests --lib \
  -- --test-threads=1
cargo test -p harp --test crouzeix_conjecture_knowledge_packet \
  -- --test-threads=1
cargo test -p harp corpus::tests --lib -- --test-threads=1
cd atlas && corepack pnpm run test
```

Release gate:

```sh
mise run verify
```

Live provider commands are explicit ticket actions and must never run from the
release gate.

The exact final commit has one unavoidable evidence boundary: tracked bytes
cannot contain the result of a gate run over the commit that contains those
same bytes. Each closeout ticket therefore records its pre-commit checks,
commits, then runs a no-edit post-commit gate over the resulting exact tip. The
terminal post-commit result is reported in the operator closeout and must leave
the tree clean; it is never backfilled by amending the tested commit.

The last two repository tickets are predeclared operator records. Their
terminal events are persisted under the ignored, create-only
`.runs/landing-001/` tree after the final tracked closeout commit:

```text
gate_ticket.json
gate_events.jsonl
gate_receipt.json
landing_ticket.json
landing_events.jsonl
landing_receipt.json
```

This is not an exception to "every task gets a ticket"; it is the only way to
avoid a false self-attestation. The committed tracker contains their complete
scope, dependencies, commands, and expected artifacts. The final user report
provides the terminal receipt digests.

## Conditional Guided Diagnostic

Arm `G` is not silently omitted and is not mixed into blind generation. After
the blind correctness outcome freezes, one repository ticket records whether
the trigger is met:

```text
run G iff H and E produced no independently reviewed complete proof
         and the fixed resource preflight passes
```

If false, the ticket records `not_applicable`. If true, a second ticket runs
one mechanism-guided reconstruction under a new run ID and a non-blind leakage
tier, with separate generation, review, sealing, and runtime tickets. Either
path is reported in the reader packet. A guided output can demonstrate
reconstruction under supplied mechanism context; it cannot establish
independent rediscovery.
