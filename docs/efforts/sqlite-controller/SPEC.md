# Local controller and SQLite milestone

Status: owner-selected baseline; implementation and independent qualification pending.
Source read: a21316f. Owner selected controller + SQLite after the two-worker design discussion. This spec supersedes the earlier larger first-release roadmap and the initial SQLite draft that deferred Kata. Kata is included as the work ledger. Git is the required initial workspace/landing backend; jj qualification is explicitly tracked behind the same workspace contract. Historical evidence and the longer-term vision remain preserved.

## Goal

Give the MTS a coherent lead and control-center interface that executes two bounded engineering tasks concurrently, retains decisions and execution state through failures, and advances trunk only to exact verified and accepted candidates.

## Deployment and authority

One local controller service is the sole authoritative writer. Lead, two worker slots, CLI and dashboard issue authenticated commands through its API. The lead is replaceable. Workers cannot update canonical refs or controller storage. No PostgreSQL, Temporal or financial budget system is required to run this milestone. Kata integration is required; jj is a qualified optional workspace backend and cannot replace exact Git-compatible candidate/landing guarantees without proof. Preserve existing implementations and unfinished work without deleting or rewriting shared branches. No automatic historical-state migration is implied; a fresh project starts with explicit intent and a pinned repository base.

## Highest priority and permanent baseline

Owner explicitly instructs: implement this whole baseline, park all unrelated work, then use this retained system to build richer versions. No unrelated feature, larger orchestration migration or speculative infrastructure work precedes baseline acceptance. Park existing work at safe recorded boundaries; preserve branches, dirty work, jobs and evidence. Unknown effects still require reconciliation; parking never means deleting or abandoning live executions. Maintain this as a supported runnable local mode with its own configuration, documentation and regression suite. Future features must not make PostgreSQL, Temporal, extra agents or a new VCS mandatory for this baseline. Fixes may evolve it compatibly; retain supported recovery and explicit migration contracts.

## Kata and participant protocol

All participants use the Corenius API for shared coordination. Kata owns human-facing work items, dependencies and source task revisions; SQLite owns execution, assignments, admitted brief revisions, decisions, acceptance and operations. Use Kata supported APIs, not its internal tables. Bind tasks to deployment/project/item identity. Capture an exact source revision and content digest at admission. Worker tools provide own assignment, relevant linked tasks, progress, questions, follow-up proposals and candidate submission. Workers receive no unrestricted Kata credentials and cannot independently claim, alter acceptance/dependencies or close work as accepted. Controller publishes progress and disposition through idempotent journaled synchronization. Kata changes during execution become visible revision proposals, never silent assignment changes. Offline/unconfirmed synchronization remains visible and retryable; Kata closure is not acceptance. Worker-to-worker messages route through Corenius with explicit sender/recipient and durable delivery/acknowledgement state.

## VCS workspace contract

Expose create_workspace(base), capture_candidate, materialize_candidate, compare, land(candidate, expected_trunk) and retire_workspace behind a narrow controller-owned interface. Git private repositories are the supported first backend. Qualification of jj covers private metadata isolation, pinned executable/configuration, exact retained commit identities, repair/history retention and interoperability with the Git protected landing gate. Mutable jj change IDs are engineering lineage, not verification identity. Preserve existing jj evaluation evidence without treating it as backend acceptance.

## SQLite storage

Store the database on supported persistent local storage outside all worker mounts. Enable WAL, synchronous=FULL and foreign_keys on every connection; validate effective configuration. Use bounded busy handling and short transactions. Never hold a transaction across runtime, verification or Git operations. Serialize controller writes; use database constraints and expected revisions to enforce two occupied worker slots maximum and unique live task ownership. CLI/dashboard never open the database directly.

Records: project, intent/brief revisions, tasks, worker slots, attempts, candidates, verification reports, acceptance decisions, commands with idempotency keys, messages/questions, decisions, operations, events and artifact manifests. Candidate and verification records are append-only. Mutating commands authenticate a scoped principal and check expected revision/current authority. Repeating an idempotency key with different arguments is rejected.

Assignment atomically claims the slot and task, creates an attempt and start operation, and appends its event before dispatch. Durable pending operations provide the local dispatch queue; notifications are hints. Event sequence supports reconnecting dashboard clients. Large logs/transcripts/reports are retained as artifacts, with digests in the database. Persist and verify artifact content before recording a durable reference; orphaned unpublished artifacts are recoverable cleanup.

## Lifecycle and concurrency

Tasks: created, active, submitted, repair-required, integration-required, accepted, landed, aborted. Execution outcome, cancellation request and cleanup status are separate fields/records; an aborted task does not establish stopped processes. Attempts: starting, running, exited, unknown. Operations: prepared, applying, confirmed, failed, unknown.

Each attempt starts from an exact trunk commit and immutable brief revision in a private Git repository. Submission retains an exact commit/tree/base/report identity before acknowledgement. Prefer one commit per task without baking that restriction into domain identity. Repair produces a new candidate; historical evidence remains bound to the old candidate.

Two workers may implement and submit concurrently. Landing is serialized through one controller-owned gate. Verification uses a separate materialization of the exact candidate, with recorded suite and environment identities. Acceptance is a durable authorized decision distinct from successful tests. Landing requires accepted exact candidate and expected trunk; perform a conditional ref update after ancestry checks. Canonical authority is a protected repository ref; checked-out views are materializations, not another independently writable authority.

If C and D both start at B and C lands, D requires a new integration attempt based on C. Reapply the retained change into a fresh private repository, submit D2, verify and accept D2. Never rebase shared branches, silently merge or substitute code after verification.

## Recovery

Acquire exclusive controller ownership before dispatch. SQLite recovers storage; controller then reconciles unfinished operations and surviving supervised executions. Controller crashes must not terminate ownership records or cause blind re-launch. Supervisor survives controller restart and supplies identity-bound execution handles; a PID alone is insufficient. Adopt an observed execution only under validated current authority. Ambiguous starts remain unknown and occupy conflicting capacity until resolved. Lease expiry does not prove a process stopped.

Persist start/cancel/land/cleanup intention before effects and observed outcome afterward. Landing recovery inspects protected ref and retained operation lineage: candidate at expected ref confirms completion; unchanged base permits checked retry; unexplained divergence blocks. If later recorded landings have advanced the ref, reconcile journal ancestry rather than mistaking all noncandidate heads for failure.

Backup uses SQLite-supported online backup or quiesced procedures, plus retained Git objects and artifacts. Never copy only a live database file while omitting its WAL. Restoring an older backup requires explicit recovery mode, rotation of authority against surviving executors, reconciliation of external effects, and operator-visible release of dispatch. First implementation may fail closed until an operator completes that restore procedure; it must not claim automatic rollback recovery.

## Worker boundary

Private Git metadata, repository, home and scratch per attempt. No writable shared objects, alternate-object dependency or canonical repo mount. Read-only pinned toolchain/dependencies; host home, SSH agents, controller sockets, database, other attempts and push credentials absent. Insula process/filesystem isolation and supervised descendants prevent cross-attempt inspection/signalling and support cancellation. Provider connectivity is narrowly exposed through qualified gateway/credential handling; controller-only services are inaccessible. Attempt grants permit own brief, messages, progress and submission, not acceptance or landing. Superseded attempts cannot submit authoritative results. Cleanup waits for observed termination and retained evidence; failure is a separately retryable state.

## Lead and control center

Both interfaces use the same commands, events and durable projections. Views show objective/plan, both assignments and health, questions and uncertainty, candidate evidence and landing queue. Distinguish requested versus observed cancellation, model activity versus evidence of progress, and confirmed versus unknown effects. Support pause dispatch, cancel attempt, answer question, revise brief, request repair, accept candidate and replace lead. Every action returns a durable receipt and visible disposition. Replacement context includes admitted intent, decisions, active work, pending candidates/questions, uncertain operations and next required decisions, independent of old conversation continuity.

## Qualification

Behavioral proof through public controller APIs and real on-disk SQLite/Git/process effects: duplicate assignment/commands, two-worker concurrency, simultaneous submissions/landing, stale-base reintegration, superseded grants, exact-candidate evidence, cancellation races, crash boundaries before/after dispatch and ref update, dashboard event replay, lead replacement and backup restore refusal/reconciliation. Isolation probes attempt forbidden filesystem, process, credential and API access. Process-kill tests prove process recovery; power-loss durability needs separate storage fault qualification. No live-runtime guarantees inferred from model-free fixtures.

## Execution sequence

01 storage and public command seam -> 02 supervised attempt recovery. Ticket03 Kata protocol and ticket04 Git workspace/landing proceed after01 (execution-dependent checks also require02); ticket05 lead/control center consumes their public seams. Ticket06 assembled proof requires01–05. Ticket07 jj qualification is tracked separately and does not delay a fully Git-backed baseline. Preserve and reuse suitable domain/runtime code after inspection; do not mechanically port the complete PostgreSQL/Temporal kernel or maintain multiple authoritative stores.

Route: gated architectural work. Existing approved design conversation supplies intent; independent critique, ticket conformance and final branch review remain required. Matt owner skills are unavailable in this harness; apply the repository crosswalk directly with available supporting skills.
