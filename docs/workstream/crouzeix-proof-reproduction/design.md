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
SHA-256-derived random stream. The archive retains every functioning node,
including lower-scoring children. Only functioning admitted children increment
the parent's child count. Both draws in a generation use one immutable
pre-generation archive snapshot; child execution and admission begin only
after both selections are recorded.

This work does not claim an exact replay of the private historical run or
mathematical certification of either public proof.

## Authority

The workstream has one authority per artifact class:

| Artifact | Authority |
|---|---|
| Product intent and experiment design | approved design and PRD linked above |
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
change.

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

### Expert node

An immutable result from one pristine expert session. It binds:

- node, parent, generation, role, and direction identity;
- exact context and parent-artifact digests;
- mechanism and proved statements;
- obligations, circularity risks, and proposed directions;
- optional complete candidate proof;
- two evaluator records and their reconciliation;
- proof-progress score `alpha_i`; and
- provider receipts and usage.

### Functioning node

A schema-valid, leakage-compliant result with either:

- a concrete mechanism and at least one proved statement; or
- a precise falsifiable blocker and a materially new direction.

Functioning is an archive-admission predicate. It does not mean improved or
correct.

### Proof-progress score

Two fresh evaluators independently answer ten route-neutral probes. A probe
passes only when both answer `pass`. `alpha_i` is the unanimous-pass count
divided by ten. The outer harness computes it; experts and evaluators never
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
alpha vector
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

`alpha_i = 1` makes a node eligible for separate mathematical review only when
it also contains a content-addressed candidate proof. It is not proof
certification. The node becomes `score_complete`, is excluded from later parent
selection by the paper's `alpha_i < 1` rule, and remains in the archive. Search
continues over all other eligible nodes until the fixed generation/call/node
budget is exhausted or the eligible set is empty.

`complete` is reserved for a later independent correctness-review outcome. It
is never an expert-node lifecycle state and is never inferred from `alpha_i`.

The corrected expert-node lifecycle is:

```text
attempted -> functioning | blocked
functioning -> archived
archived -> selected | score_complete | superseded
selected -> archived
```

`score_complete` requires `alpha_i = 1` and a content-addressed candidate
proof. It does not imply `proof_outcome=complete`.

Every score-complete candidate is content-addressed and independently reviewed.
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
| `frontier.py` | pure archive, score reconciliation, DGM selection, direction ranking |
| `expert_contracts.py` | expert/evaluator contexts and output validation |
| `frontier_store.py` | append-only ledgers, snapshots, node materialization, candidate freeze |
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
reconcile_evaluations(node_sha256: str, first: Mapping, second: Mapping) -> dict
selection_terms(archive: FrontierArchive) -> list[dict]
select_parents(archive: FrontierArchive, *, generation: int, seed: int,
               count: int = 2) -> dict
assign_directions(parent_ids: Sequence[str], nodes: Mapping) -> list[dict]

# expert_contracts.py
build_expert_context(...) -> dict
build_evaluator_context(...) -> dict
validate_expert_result(...) -> dict
validate_evaluator_result(...) -> dict

# frontier_store.py
open_frontier_store(run_dir: Path) -> FrontierStore
FrontierStore.append_attempt(...)
FrontierStore.materialize_node(...)
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
