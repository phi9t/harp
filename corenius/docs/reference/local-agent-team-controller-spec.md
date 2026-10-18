# Local agent-team controller: portable baseline specification

Version: 0.1
Status: implementation requirements, not a readiness claim.
Provenance: consolidated from Corenius owner-approved baseline through commit4dfe7fb and its controller-first controlled-trial contract. Source-read main:a21316f. This is a shareable adaptation, not a second authoritative Corenius effort spec. Repository-specific implementation status is intentionally excluded. Adopting projects should record their own approved configuration and evidence.

## 1. Goal and scope

Build a durable local engineering team: one replaceable Codex lead, two concurrent Codex worker slots, a central controller, SQLite, Kata task integration, isolated Git workspaces and a dashboard. Prove controller behavior first, then integrations, then a bounded live trial. Retain this runnable baseline and its regression suite when adding richer versions.

No PostgreSQL, Temporal, multi-host scheduling, generic workflow interpreter or financial budget feature is required. Git is the initial supported backend; jj is optional until qualified against the same workspace contract. Use explicit execution deadlines and bounded automatic repair policies. Preserve existing work when changing priorities.

## 2. Architecture and authority

Human, lead, workers, CLI and dashboard communicate with the controller for shared coordination. Workers edit files/run tools within isolated environments. The lead chooses engineering work; controller validates, persists and executes admitted decisions. SQLite owns assignments, attempts, decisions, messages, acceptance and operation outcomes. Kata owns human-facing work items, dependencies and source task revisions, and receives controller-derived status summaries. Protected Git refs own integrated code identity. Runtime servers execute agent turns. A supervisor owns independently addressable process lifecycles.

Only one local controller mutates authoritative execution state. Lead/worker credentials are scoped. Workers cannot claim arbitrary tasks, alter acceptance/dependencies, accept their own results or land to trunk. Human interventions use the same recorded command path. The two worker slots limit implementation executions; controller/supervisor/dashboard and authorized evaluation activity are not additional implementation slots.

## 3. Durable objects

Project/mission: objective, constraints and protected trunk reference. WorkRef: ledger deployment/project/item identity. Intent/brief revision: admitted immutable requirements and acceptance criteria. Task: outcome to achieve. Attempt: one bounded try with its own execution/workspace. Candidate: exact retained proposed result. Verification: evidence for a particular candidate and suite/environment. Acceptance: authorized decision referencing candidate/evidence. Operation: journaled start/cancel/land/sync/cleanup. Runtime binding: role/attempt mapping to server/thread/turn. Message/question/decision: durable team continuity. Artifact manifest: retained content identity and location.

Mutable branches, terminals, threads and agent names are not identities of completed work.

## 4. SQLite and command protocol

Use persistent supported local storage outside worker mounts. Enable and validate WAL, synchronous=FULL and foreign keys; apply explicit schema migrations. Acquire exclusive controller ownership before dispatch. Use short transactions, bounded busy handling, revision checks and database constraints. Never keep a transaction open during model calls, process launch, tests or Git mutation.

Each mutating request includes authenticated principal, project, command kind/arguments, expected revision and idempotency key. Validate live authority before replay. Same key and same request returns the retained receipt; changed arguments are rejected. Failed transactions leave no partial state/event/receipt. A receipt identifies resulting revision and event sequence. Queries authenticate and return scoped data without credentials.

Assignment atomically claims the task and one of two slots, creates an attempt, pins inputs, prepares a start operation and appends an event/receipt. Dispatch occurs afterward. Pending operations form a durable dispatch queue; notifications are hints. Dashboard reconnects from an event cursor.

Large artifacts stay outside SQLite. Publish/durably retain content before committing a manifest reference. Keep candidate Git objects independently of disposable workspaces. Unpublished orphan artifacts can be cleaned recoverably.

## 5. Specs, tasks and Kata

Filesystem source references include repository/store identity, relative path, exact commit or immutable snapshot, digest and source revision. Capture referenced supporting files. Dirty files require an explicit snapshot. An evolving pathname alone is not admitted intent.

Link this snapshot to Kata WorkRef and item revision. Admission pins the brief; edits become visible amendment proposals. Authorized revision/reassignment creates new immutable input and reconciles existing attempts. Never silently change an active worker's objective.

Use supported Kata APIs, not internal tables. Journal synchronization and retain source/controller/sync revisions. Define concrete status mapping for the deployed Kata version. Show pending updates and drift; lost acknowledgements require deduplication or readback before repetition. Kata closing a task does not prove acceptance, landing or process termination. Worker access goes through controller tools; workers have no unrestricted Kata credentials.

## 6. Runtime registry and agent communication

Codex lead/workers may share an app-server or use distinct servers. Record server identity, endpoint/transport, protected authentication reference, incarnation, persistent thread-store identity, pinned version and capabilities. Bind roles/attempts to server+thread, known active turn, authority epoch, execution handle and workspace. Validate discovery/registration; never identify a thread solely from terminal title. Reconcile server replacement before dispatch.

Controller adapter initializes/connects, subscribes, inspects/resumes threads, starts turns and uses qualified steering/cancellation. TUI observes the same server/thread. Shared servers must not broaden grants or defeat filesystem/process isolation; otherwise use separate servers.

Lead tools read context/tasks, propose/create assignments, request starts, answer questions, record decisions and request authorized acceptance/landing. Worker tools read own assignment/relevant context, report progress, ask questions, exchange scoped messages and submit candidates. Tool wrappers handle transport/authentication/revisions/retry keys.

Questions progress through persisted, submitted-to-runtime, explicitly acknowledged and resolved. Ordinary updates batch into an idle lead turn. Active steering targets the expected turn; a turn race returns delivery to pending. Runtime acknowledgement does not mean the lead answered. Ambiguous dispatch is reconciled before retry. Worker-to-worker messages use the controller without requiring lead relay. Replies and transcripts are evidence; terminal phrases are not lifecycle or acceptance signals.

## 7. State machines

Task states: created, active, submitted, repair-required, integration-required, accepted, landed, aborted. Verification can be a separate tracked operation; task state alone does not describe process liveness.

Attempt states: starting, running, exited, unknown. Exit is terminal. Cancellation requested/observed is separately recorded.

Operation states: prepared, applying, confirmed, failed, unknown. Allowed transitions: prepared to applying/failed/unknown; applying to confirmed/failed/unknown; unknown to applying/confirmed/failed after reconciliation. Confirmed/failed are terminal.

Unknown executions retain conflicting capacity. Aborted task, failed operation, lease expiry and absent terminal do not establish stopped processes. Blocked questions/dependencies are explicit records, with an episode yield/resume path, rather than wasted polling turns.

## 8. Workspace isolation and hygiene

Create a private repository, home, scratch and Git metadata per attempt, outside the human checkout. One writer per mutable workspace. Exclude canonical repository, other attempts, controller database/state/control sockets, host credentials, SSH agents and push credentials. Expose selected toolchains/dependencies read-only. Agent-facing communication is a narrowly scoped bridge, not unrestricted controller administrative access.

Use an enforced OS/container boundary and independently supervised descendants. Restrict cross-attempt inspection/signalling. Provider access uses qualified narrow credential/relay handling; it cannot expose controller secrets or bypass authorization. Separate Git repositories alone are not a security boundary.

Inventory records owner, repository/worktree kind, path, base, branch/bookmarks, candidate, dirty/untracked state, runtime/process identity and retention/cleanup state. Preserve dirty work. Never rebase, force-push or delete shared branches. Retire only explicitly owned disposable resources after process termination and evidence/object retention. Cleanup failure is separately retryable.

## 9. Git/jj interface, candidates and landing

Workspace interface: create_workspace(base), capture_candidate, materialize_candidate, compare(base,candidate), land(candidate,expected_trunk), retire_workspace. Git private repositories implement the baseline. jj qualification requires pinned tools/config, isolated metadata, immutable candidate identity, retained repair/history and Git landing interoperability. A jj change ID may describe lineage but cannot replace exact verification identity.

Submission pins commit/tree/base, admitted brief revision and report/artifact references. Stop mutation of the submitted result; repair produces a new candidate. Prefer one commit per task without making it a domain restriction. Verification uses a separately materialized exact candidate and records suite/environment identity, commands, results and logs. Lead assesses without repairing in place. Acceptance is distinct from testing and references exact evidence.

Only the protected landing service updates trunk. It checks authority, candidate acceptance/evidence, expected trunk and fast-forward ancestry. Persist prepared landing before conditional ref mutation, confirm afterward. No post-verification squash, cherry-pick, rebase or tiny fix. Canonical ref authority is separate from checked-out views.

Two workers may start at B and submit C and D. If C lands, D needs reintegration: reapply into a fresh attempt at C, produce D2, verify and accept D2 before landing. Old evidence remains historical. Landing is serialized; both workers still implement concurrently.

## 10. Recovery and backup

Record intention before external action and observed outcome afterward. SQLite restores storage consistency; controller reconciles external reality. Supervisor survives controller restart and supplies identity-bound execution handles, not PID-only identifiers. Validate process identity/lineage before termination; prevent PID-reuse targeting. Adopt surviving execution only with current validated authority. Lost start acknowledgements never trigger blind re-launch.

Landing recovery confirms a candidate already at trunk, permits checked retry at unchanged base, or inspects later recorded landing lineage. Unexplained divergence blocks. Disconnecting TUIs/dashboard does not terminate execution. Replacement lead reconstructs intent, decisions, running attempts, pending questions/candidates, uncertain effects and next decisions from durable records.

Use supported SQLite backup or quiesced procedures plus Git objects/artifacts. Do not copy a live database alone while ignoring WAL. Older-backup restore enters explicit recovery mode, invalidates old grants against surviving executors, reconciles effects and releases dispatch only through a recorded operator decision. Safe refusal pending reconciliation is acceptable; unsupported automatic restoration must not be advertised.

## 11. Dashboard/control center

Show mission/objective, accepted plan and outstanding work; two worker assignments; pinned brief/runtime/workspace; reported progress; observed execution/model/tool activity; latest material evidence; questions; desired/observed cancellation; unknown effects; candidates/reviews/acceptance/landing; current/admitted filesystem and Kata revisions; synchronization lag and workspace hygiene.

Provide transcript/log/diff/evidence drilldown. Distinguish reported from observed facts and flag stale observations. Controls: pause dispatch, cancel, answer, revise intent, request repair, authorized acceptance/landing and replace lead. Return durable receipts and visible disposition. Chat and dashboard must show the same decisions/state.

## 12. Controlled tmux trial

Build order: controller/state-machine+SQLite tests; deterministic actual Git/Kata/runtime integration; then live Codex trial.

Create a unique tmux session with windows0 harness/status,1 controller,2 dashboard,3 lead,4 workerA,5 workerB. An independent outer harness runs outside that session. Windows observe/attach supervised services/executions; closing views is not lifecycle control. Never reuse or clean unrelated user sessions.

A unique run root contains database, canonical Git, artifacts and private attempt repositories; use an isolated Kata project. Pin source/build/schema/runtime/API/toolchain/suite versions and record pane/process/binding identities. Outer harness sends saved literal input to the identified lead pane and verifies correlated controller admission separately. It may inject declared faults into owned processes and inspect evidence. It cannot secretly assign work, fabricate results, bypass acceptance, mutate storage or move trunk. Scripted operator actions use normal authenticated commands and remain attributable.

Judge structured controller events, immutable evidence and independently observed Git/process outcomes, not terminal success text. Record event ranges, hashes, runtime/process observations and log digests. Enforce trial deadlines and retain failed/unknown runs for reconciliation.

## 13. Acceptance matrix

Require meaningful public-interface tests against real SQLite/Git/process effects: atomic admission; two overlapping workers/third refusal; duplicate and conflicting requests; stale revisions; message access/acknowledgement; question delivery to intended lead; shared/separate server bindings; wrong-thread refusal; source amendment; Kata disconnect/sync; exact candidate verification; competing land requests; stale-base D2 reintegration; cancellation races; controller restart with workers alive; lost start reply; crash after ref update; lead replacement; dashboard reconnect; backup refusal/reconciliation; dirty-work retention and safe cleanup.

Isolation probes attempt canonical ref writes, other-workspace access, host secret reads, controller signalling and revoked-identity submission. Every prohibited action must fail without harming existing work.

At least one live Codex lead+two Codex workers must complete the combined loop through actual Kata/Git/controller integrations, with independent evidence review. Model-free proof does not establish live runtime/provider guarantees. Process-kill tests do not establish power-loss durability. Document unsupported properties explicitly.

## 14. Delivery and permanent support

Tracer bullets:01 SQLite command/state core;02 supervision/recovery/isolation;03 Kata/participant protocol;04 workspace/candidate/verification/landing;05 lead tools/dashboard;06 assembled deterministic and live trial;07 optional jj qualification.01 precedes02/03;04 depends on01/02;05 consumes01–04;06 requires01–05. Use independent Standards/Spec and assembled defect review, fresh tests, scoped ownership and gated fast-forward integration.

Deliver reproducible launch/run commands, configuration, migrations, operating/recovery docs, regression suite and retained qualification evidence. Keep this mode supported as future orchestration grows. Do not require later PostgreSQL/Temporal/multi-agent features merely to run this baseline.
