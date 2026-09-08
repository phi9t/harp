# Execution contracts for supervised research

Status: proposed normative detail for the next implementation. Existing behavior
is described by `design.md`; none of these additions is claimed implemented.
Use [plan.md](plan.md) for dependencies and qualification gates. This document owns
cross-scenario execution semantics; workload science stays in
[frontier-research-scenarios.md](frontier-research-scenarios.md).

## C1. Admission, identity and versioning

An admitted run is immutable intent plus an append-only execution history.
The caller supplies a bounded idempotency key, a versioned plan, input manifests,
backend/environment requirements and budgets. Resolve mutable refs before admission.
A unique `(ledger identity, caller namespace, submission key)` maps to one run.
Identical canonical request content returns that run; different content conflicts.
Commit the run, request digest and admission event atomically before publishing
the receipt or submitting work. An unavailable provider is a run incident after
admission, not a reason to lose the caller's durable run reference.

A run reference identifies the ledger, run ID and admission digest. It contains
no credentials. The initial qualification assumes the ledger remains available
on its original storage. A fresh agent can attach there; loss of that disk is not
covered. Do not place SQLite on an SSH-mounted filesystem or create independent
writable copies of a run ledger on several hosts.

Introduce workflow/execution receipt v2 for typed agent and command actions,
explicit bindings and authored terminal outputs. Preserve v1 parsing, graph rules,
receipt digest semantics and inspection output. V1 currently requires at least
two analysis nodes and exactly one reducer; v2 permits one or more tasks and
explicit report outputs. Do not insert fake agent tasks to satisfy v1 rules.
The initial v2 graph still has at most 64 admitted nodes and no runtime expansion.
Reject expanded static plans over the limit before effects. A nine-trial study
must count setup, training, evaluation and report nodes as well as trial nodes.

Separate graph dependencies, semantic role and executor. A verification role can
execute a command; a training command is not a model conversation. The existing
ActivityRuntime remains responsible for agent sessions. A command-job backend
provides submit/observe/cancel/collect to the same Engine scheduler. Its provenance
is pinned per task in v2, replacing the assumption that one agent provider
provenance identifies every activity in a mixed run. Legacy provenance is unchanged.

For canonical digests, use one Rust-owned versioned encoding: recursively sort
object keys by UTF-8 bytes, reject duplicate keys, preserve array order, encode
UTF-8 without insignificant whitespace, and reject nonfinite numbers. Digest
fields use lowercase SHA-256. Identity-bearing numeric configuration uses integers
or normalized decimal strings, never unconstrained floating-point serialization.
Cross-language golden vectors must establish identical bytes. Raw artifact hashes
remain hashes of raw bytes, not parsed/reformatted content. Paths are names under
an admitted root, not globally trustworthy filesystem authority.

## C2. Input and result bindings

A binding is either an immutable artifact or a named output of a declared producer.
Its fields are consumer input name, producer task/output name or artifact digest,
expected schema/version, and materialization mode. Producers must be ancestors;
cycles, duplicate input names, unknown outputs and implicit filesystem sharing
fail validation. Successful producer acceptance pins output digests once.

Before consumer launch, validate schema and bytes, materialize read-only inputs,
and commit a resolved-input manifest linked to the consumer attempt. A changed
producer attempt cannot silently replace an accepted output. A change to code,
data or plan requires a new run/trial as applicable. JSON schema validity alone
does not validate source locators, numerical values, tensor shapes or provenance;
scenario validators enforce those additional contracts.

Keep ResultEnvelope for agent transport. A scenario output is a referenced,
validated artifact, not an arbitrary schema the CLI silently overwrites. Command
results use native completion receipts; they do not manufacture an LLM transcript.
Both routes create typed accepted outputs through the same state acceptance boundary.

Strict dependencies require a successful workload outcome. Explicit settled
bindings consume a terminal outcome record, including a failed trial with evidence.
They never pretend a failed trial produced weights. This enables partial campaign
reports. Default remains strict. V2 permits continued independent branches after
failure only when admitted in policy; failed required tasks make the overall run
unsuccessful even if a partial report is published. Infrastructure exhaustion
produces a terminal failure record for settled consumers. Reports preserve missing
coverage and do not turn infrastructure failure into a scientific result.

Finalization and cancellation reconciliation are Engine lifecycle operations.
They must run even if an authored report task cannot execute; cleanup cannot depend
on a reducer whose prerequisites have failed. An Engine incident report requires
no live model provider.

## C3. Jobs, ownership and reconciliation

Job states are distinct from connectivity observations. Persist these job states:
`prepared`, `submission_unknown`, `running`, `terminal`, `collecting`, `collected`.
Persist `cancel_requested` independently. Terminal carries an exit/signal or an
authoritative loss receipt; last-contact failure does not establish termination.
A disconnected running job has stale observation, not a fabricated failed state.

| From | Evidence and guard | Transition/effect |
| --- | --- | --- |
| prepared | admitted inputs, resource reservation and backend qualification | record submission intent; call submit with stable key |
| submission_unknown | lookup confirms exact job | bind original job; observe |
| submission_unknown | authoritative absence plus exclusion of delayed creation | submit same key; never invent a new key |
| submission_unknown | evidence unavailable | wait or request decision; retain reservation |
| running | exit receipt plus required process-tree quiescence | terminal |
| running | cancellation intent | signal owned tree; remain nonterminal until confirmed |
| terminal | manifest and permitted collection | collecting |
| collecting | all required bytes and metadata validated | collected and accept result/terminal outcome |
| terminal failure | compatible checkpoint, confirmed old-owner death, retry budget | new execution attempt for same trial, preserving trial identity |

A backend must make submission idempotent at the execution owner. Local intent
alone cannot make a remote spawn exactly once. At owner startup, durable intent,
exclusive ownership and process containment must allow reconciliation of a crash
between spawning and recording the child. If live children cannot be identified
or fenced, return unknown; do not claim autonomous recovery for that backend.
Test this boundary explicitly, including delayed submissions after a timeout.

Controller epochs fence state transitions through compare-and-swap. Backend
operations must also honor ownership or equivalent containment guarantees; a
local lease does not fence an old remote writer. Do not transfer authority based
only on lease expiry. Process identity includes backend run identity and host boot
identity where available; PID or tmux session name alone is insufficient.

For local execution use a per-run owned process with durable spool/status, not a
new permanent daemon. Reuse qualified lifecycle code where possible. For remote
execution select one Armada-owned job boundary and place Vaso execution inside
it. Do not wrap independent competing launch/retry loops around each other.
Qualification determines actual descendant containment and termination behavior.

Retry ceilings are separate: agent transient retries and continuation keep their
existing ceilings; command retries are declared per task; training restart uses
its checkpoint policy. Initial training profile permits one restart. Unknown
submission consumes no new trial but retains the reserved exposure. Every actual
attempt, including lost work, consumes cumulative resource budget.

## C4. Observation, decisions and cancellation

Proposed CLI operations are distinct: `submit` admits and returns; `drive` advances
an admitted run under exclusive ownership; `wait` reads bounded events; `inspect`,
`explain` and `review` read persisted evidence; `cancel` records cancellation;
`resume` acquires/reconciles execution before further dispatch. These names are
proposed, not current commands. Existing `run` remains a convenience wrapper with
its v1 contract preserved. A v2 submit returns before provider execution; a driver
may then operate independently of its observing shell.

`wait` accepts a run reference, opaque ledger-scoped cursor, maximum events and
at most 60 seconds. Return reason is progress, attention, terminal or timeout,
with the next cursor and snapshot revision. Events are at-least-once to observers;
sequence IDs support deduplication. No hidden state mutation occurs in wait.
Persisted event retention is append-only initially. A cursor for another run or
beyond the log fails explicitly; never silently reset to zero.

Decision records bind ID, run/task, expected revision, owner epoch, reason,
evidence and allowed transition. Applying a duplicate decision returns its
previous result; applying stale or changed content conflicts. Inspect offers
advisory explanations; execution rechecks prerequisites before persisting a
decision. Record requested human decisions separately from routine Engine recovery.

Ctrl-C on an observer detaches. Explicit cancellation durably stops new dispatch
before requesting worker termination. A disconnected host leaves cancellation
pending. After process termination, capture permitted partial artifacts and
reconcile owned resource release. Report execution cancellation and unresolved
finalization separately. No state called fully finalized may retain unknown jobs
or unacknowledged owned-resource release. Cancellation never starts a repair.

Local monotonic time enforces local deadlines; backend-relative time enforces
remote job deadlines. Record UTC for human review but do not order distributed
events or calculate elapsed budget by subtracting unsynchronized host clocks.
Queue wait, last contact, last progress and deadline are separate fields. A quiet
checkpoint write is not automatically a hung trainer; phase-specific expected
progress intervals are part of the workload profile.

## C5. Environment admission and budgets

Environment preparation has states `planned`, `preparing`, `verified`, `failed`
and `unknown`. An interrupted setup may resume only in its task-owned staging
root. Publication is marker-last after recipe and capability verification.
The receipt pins source snapshot, dependency lock/artifacts, executable/toolchain,
rootfs where applicable, sandbox policy, writable roots, devices and relevant
host capability facts. A mutable rootfs path is not a content identity.

Downloads are a separate declared preparation operation. Each executable input
requires an immutable official release/object, verified digest, confined install
root and compatibility probe. Never execute a mutable download pipe. Dependency
resolution may happen when authoring the profile; execution consumes the frozen
lock and artifact manifest. Hashes certify bytes, not trust; the source authority
must be recorded. Keep credentials outside receipts and use secret references.

A capability qualification record binds tested backend binary/config/policy and
host capability signature. It lists exact passed and unsupported guarantees.
Changing the relevant backend/environment invalidates it. Qualification does not
grant execution authority, allocate devices or substitute for an environment
receipt. Optional capabilities can be absent; required capabilities fail before
workload launch, with no unsandboxed fallback.

Reserve task resources atomically before submission. The accounting vector is
agent tokens, CPU allocation-seconds, GPU device-seconds, wall deadline, storage
bytes and trial count. A single-host claim specifies device IDs and ownership
source. No-device-reservation evidence means no exclusive-use claim. Initial
GPU runs require an explicitly admitted device allocation; SSH reachability alone
cannot satisfy this condition.

For each dimension enforce `settled_usage + outstanding_reservations <= limit`.
On ambiguous outcome keep the reservation. Settle using authoritative measured
usage when available; otherwise charge the reserved maximum and mark estimated.
Reserve full retry exposure or re-reserve before each permitted restart. A parent
budget covers all children, with no additional allowance on supervisor restart.
Release a reservation only after authoritative absence/termination and settlement.
Use checked integer arithmetic and explicit units; GPU count multiplies duration.

The backend enforces its admitted maximum runtime while Harp is absent. Record
termination grace as reserved exposure. If a backend cannot enforce a hard stop,
report that limitation and do not qualify it for unattended bounded GPU work.
Disk/output limits include logs, temporary checkpoint publication and retained
previous checkpoints, not just accepted output bytes. Exhaustion preserves an
incident without pretending an incomplete checkpoint is usable.

## C6. Artifacts and training state

Harp currently caps a single artifact object at 64 MiB. W0 must fit its tensor
and metric artifacts within that cap. Larger profiles require a versioned chunked
manifest: chunks no larger than 64 MiB with ordered offsets, lengths and hashes,
checked total length and tensor metadata. Do not globally raise the cap or read
an entire large checkpoint into memory. Source-candidate limits in the coding
spec are separate from model-state limits.

Training checkpoint publication is complete-optimizer-step only. Include weights,
optimizer slots, scheduler, RNG states, data sampler/order/cursor, global step,
processed and supervised token counts, precision state and all trial identities.
For accumulated gradients, checkpoint only after the full accumulation window.
Changing batch size, sequence length, data, math/code or optimizer configuration
creates a new trial even if initialized from an earlier checkpoint. Resume cannot
rename such a change into a retry.

Store checkpoint tensors and metadata in non-executable formats. Validate names,
shapes, dtypes, counts and digests before loading; do not deserialize arbitrary
agent-supplied executable objects. Publish durable bytes, then manifest, then DB
reference/event. A crash before the DB commit leaves reusable unreferenced bytes;
a missing referenced blob is integrity failure.

Progress metrics use `(trial, attempt, logical_step, metric_name)` identities.
Retain discarded/replayed attempt history. Construct accepted training curves
from the checkpoint lineage, avoiding double-counted replay. Distinguish consumed
logical tokens from physical compute spent replaying them. A final process exit
without required checkpoint/evaluation artifacts is not a complete trial.

Remote collection validates chunks before local publication, resumes transfers
by digest, and does not trust remote paths or symlinks. A terminal job can be
`collection_pending`. Keep its output resources until required artifacts are
verified at the configured durable destination. Initial E2 guarantee survives
SSH loss and worker restart on an intact host. Host/disk loss is recoverable only
up to a verified replicated checkpoint; no remote replication is presumed.

## C7. Research validity and dynamic scope

Keep three verdicts: execution outcome, workload correctness, research conclusion.
The same manifest carries all three without collapsing them into a boolean.
A scientific hypothesis may fail while execution and correctness pass.

Pre-register architecture family, budget, metric, seed list, selection rule and
final-test checkpoint population. For W0, evaluate all three final seed checkpoints
of each selected or tied configuration; do not select a best seed from test data.
Validation informs selection; final test evaluates the frozen selection. No fixed
number of seeds establishes significance by itself. Report individual results,
missing trials, uncertainty and parameter/compute differences. Aggregates link to
exact accepted producer identities, not mutable filenames or notebook cells.

First campaign uses fixed candidates and three seeds. Agent-designed candidates
are a later workload milestone after managed coding is qualified; training the
fixed supplied candidates must not depend on arbitrary coding being implemented.
The full workflow still includes real analysis and independent review agents.

Bounded dynamic expansion is a later v2 feature. A planner emits declarative items
under an admitted schema; it cannot supply executable commands or authority.
Validate children against the parent's template and remaining node/resource caps.
Atomically append one expansion generation and deterministic child identities
before dispatch. Resume reuses that exact generation. Nested expansion and
unbounded repair loops remain excluded. A changed planner decision is a new run,
not an in-place rewrite of running children.

## C8. Compatibility, testing and release evidence

Write-capable opening performs validated migrations; read-only inspection never
does. Unknown executable contracts fail closed. Existing v1 observations and
resume fixtures remain covered; do not infer v2 authority from old labels.
Report v2 explains unsupported features explicitly. Compatibility tests include
v1 runs created before migration and old read-only inspection failure on databases
that need migration.

Test genuine boundaries: real SQLite reopen, disposable files, real process death,
connection loss at submit, delayed replies, broken transfer and stale ownership.
Use production defaults that cannot be redirected through ambient test variables.
Fault injection is an explicit test-only constructor/configuration. No runtime
claim can be established by matching source text or using a fake trainer only.

Publish a qualification manifest with workload/environment revisions, admitted
limits, backend guarantees, scenario cases and evidence references. Each case
has `passed`, `failed`, `not_run` or `unsupported`; no omitted case defaults to
pass. This manifest powers a matrix view but is not a new scheduler or authority
store. Human decisions about implementation adoption, compute beyond admitted
limits, merge and release remain outside automated qualification.
