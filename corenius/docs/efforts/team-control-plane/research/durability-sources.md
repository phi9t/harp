# External team broker: durability and protocol sources

Retrieved: 2026-10-01 UTC. Repository guidance read at Corenius commit `963f58b31e518a8162dfe719e386ef9d9843321f`. Research scope: official specifications, product documentation, and maintainer repositories. Live documentation is mutable; pin provider binaries and generated schemas during implementation. No Trae model, service, or provider session was launched. Facts below describe the cited system; recommendations are explicitly inference, not evidence that Trae implements the same behavior.

## Findings supported by primary sources

### PostgreSQL and an outbox

A PostgreSQL transaction groups updates into one atomic operation; committed changes become visible together and rollback cancels the group. This supplies an atomic boundary for broker state and an outbox record, rather than an atomic transaction with an external agent provider. [PostgreSQL transactions](https://www.postgresql.org/docs/current/tutorial-transactions.html)

`FOR UPDATE SKIP LOCKED` skips rows held by competing transactions. PostgreSQL explicitly identifies queue consumers as a use case and warns that the resulting view is inconsistent for general queries. It is a claim primitive, not by itself a durable lease, ordering guarantee, or protection against an already-running stale worker. [PostgreSQL SELECT locking clause](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE)

AWS's first-party outbox guidance describes recording application changes and events in the same database transaction, then forwarding committed events separately. It warns about duplicate deliveries, idempotent consumers, ordering, and rollback. Thus a successful database commit followed by a process crash must leave a replayable outbound intention. A crash after external acceptance but before acknowledgement can cause redelivery; outbox alone cannot promise exactly-once provider side effects. [Transactional outbox pattern](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html)

### Temporal execution semantics

Workflow execution recovers by replaying recorded history and checking commands against that history. Its local cached state is an optimization; the execution history enables recovery after loss of that cache or worker. Namespace, Workflow ID and Run ID identify an execution. [Workflow execution](https://docs.temporal.io/workflow-execution)

Workflow code must generate the same sequence of Workflow API calls for the same inputs/history. Temporal explicitly directs API calls, database queries, LLM invocations and external interactions into Activities. Changing command order for an existing history can break replay; workflow versioning must therefore be part of deployment design. [Workflow definition and deterministic constraints](https://docs.temporal.io/workflow-definition#deterministic-constraints)

Activities retry by default; whole Workflow Executions do not have a retry policy by default. Configure bounded retry/time budgets and permanent error classification for the broker instead of accepting unlimited default retries for costly provider calls. Workflow replay and retrying an Activity are distinct operations. [Retry policies](https://docs.temporal.io/encyclopedia/retry-policies)

The Activity lifecycle uses timeouts to detect lost tasks, including a crash after the function has been called, and may retry the task. Activities must heartbeat to receive service cancellation. Cancellation is cooperative: an Activity can accept or ignore it, and the Workflow can choose whether to wait for acceptance. Temporal does not itself kill an external provider subprocess. [Activity execution](https://docs.temporal.io/activity-execution)

Continue-As-New passes explicitly selected state into a fresh execution with the same Workflow ID, a new Run ID, and a new history. It addresses history growth and can help transition between workflow code versions. The SDK's suggested checkpoint signal is an input to deciding when to roll over. [Continue-As-New](https://docs.temporal.io/workflow-execution/continue-as-new)

### SDK availability and stability today

| Language | Verified availability/stability | Implication (inference) |
| --- | --- | --- |
| Rust | Temporal announced the fully featured Rust SDK as generally available on **2026-09-04**. Its current official developer guide includes workflows, activities, workers, clients and Continue-As-New. [GA announcement](https://temporal.io/changelog/rust-sdk-generally-available), [Rust guide](https://docs.temporal.io/develop/rust) | Rust is now a legitimate option. Do not repeat older claims that the only Rust offering is an experimental internal Core SDK. Newly GA still merits a pinned-version replay/restart spike. |
| Python | The maintainer's 1.0.0 release declares Python GA and promises workflow-history compatibility across newer versions. The current SDK is distributed as `temporalio` and documents worker/client APIs. [Stable release](https://github.com/temporalio/sdk-python/releases/tag/1.0.0), [SDK README](https://github.com/temporalio/sdk-python) | A reasonable first worker language where the existing toolbox is Python. Experimental subfeatures must be evaluated separately from SDK stability. |
| TypeScript | The maintained official SDK documents installable client, workflow, worker and activity packages. It officially supports Node 20, 22 and 24; all `@temporalio/*` packages must use the same version. It discourages workers on other JavaScript runtimes. [SDK README](https://github.com/temporalio/sdk-typescript), [1.0.0 release](https://github.com/temporalio/sdk-typescript/releases/tag/v1.0.0) | An established released SDK alternative, with a Node deployment requirement. This report does not infer Bun/Deno worker support from client compatibility. |

The Rust GA announcement was fetched directly with Python `urllib.request` after the web reader returned an internal error. Search results containing older Public Preview README snapshots are stale relative to that dated announcement and the current fetched guide. No exact latest package version is recommended here.

### Symphony's baseline is recovery by reconstruction

The current OpenAI Symphony spec identifies itself as Draft v1. It defines a scheduler/runner and tracker reader with bounded concurrency, retries, workspaces, reconciliation, and repository-owned workflow policy. Restart recovery comes from tracker/filesystem state; exact in-memory scheduler state is explicitly not restored, and a persistent database is not required. Its non-goals include a general workflow engine, distributed scheduler and multi-tenant control plane. This is a useful operational baseline, but does not establish durable team messages or exact claim restoration. [Symphony SPEC, sections 1–3 and 4.1.8](https://github.com/openai/symphony/blob/main/SPEC.md)

### MCP scope and an evolving task extension

The fetched latest architecture resolves to MCP **2026-07-28**, describing JSON-RPC context exchange between a host, clients and focused servers. The host owns authorization and orchestration; servers expose resources, tools and prompts. Current architecture is stateless with capabilities carried per request. Do not assume an older session/initialization contract without pinning the supported protocol version. [MCP architecture](https://modelcontextprotocol.io/specification/2026-07-28/architecture)

MCP's current Tasks extension describes durable task handles, reconnect polling, input-required states and cooperative cancellation. This is stronger than saying MCP has no durable asynchronous primitive. However, it is an extension requiring advertised support; the server must implement its persistence and lifecycle. It does not demonstrate Trae support or supply the broker's team membership, role assignment, fencing, message ordering or business authorization rules. [Tasks extension overview](https://modelcontextprotocol.io/extensions/tasks/overview)

### Codex app-server identifiers and control

Official Codex documentation specifies `initialize` then `initialized` once per transport connection; `thread/start` or `thread/resume`; and `turn/start` targeting `threadId`, returning an initial turn with its ID. `turn/steer` appends input to the active turn and requires matching `expectedTurnId`; it fails without an active turn and does not create a new turn. `turn/interrupt` takes `threadId` and `turnId`, requests cancellation, and ends with interrupted status. Stream `turn/completed` to observe completion. These are Codex facts only. JSON-RPC request IDs, thread IDs and turn IDs serve different roles and must remain separate from broker command, attempt, team and Temporal identifiers. [Official Codex app-server documentation](https://developers.openai.com/codex/app-server/)

## Proposed broker contract (inference)

Make the external broker authoritative for teams, roles, task claims, attempts, command acceptance, messages, cancellation intent and audit history. Treat Trae as an execution adapter whose observed capabilities determine start/steer/cancel/recover behavior. The database stores desired state and validated outcomes; provider observations are evidence, not permission to rewrite claims.

Keep PostgreSQL business authority and Temporal execution authority distinct. A PostgreSQL transaction commits a domain transition plus an outbox command. A relay delivers the stable command ID to a deterministic Workflow identity. Activities validate the current database version/fence, perform bounded provider I/O, and record outcomes transactionally. Temporal history owns orchestration progress, timers and retry state; PostgreSQL owns accepted business transitions. Do not mutate Temporal's internal persistence tables to bridge them.

Recommended identifiers: `team_id`, `task_id`, `attempt_id`, `command_id`, `message_id`, monotonically increasing claim fence and domain version, Temporal Workflow ID/Run ID, provider session/thread/turn IDs where actually supported. Retrying must reuse a logical command ID, while a genuinely new attempt gets a new attempt ID. Deduplicate inbox application with a database uniqueness constraint and preserve per-aggregate sequence where ordering matters. A fencing token prevents a stale attempt's later database write from becoming authoritative; it does not retroactively stop already-issued external effects.

A lost provider launch acknowledgement is a distinct uncertainty state. Reconcile using a proven provider lookup/resume seam before replaying a non-idempotent start. If none exists, surface attention-needed rather than promising exactly-once launches. Likewise store cancellation request, adapter acceptance and confirmed stop separately; a transport acknowledgement is not proof that work stopped.

## Small initial slice and durability acceptance (recommendation)

1. First build one broker path with PostgreSQL authority: create one team, assign one bounded task, persist one claim/attempt, accept one command, and persist a transactional outbox entry. Use a fake adapter to prove duplicate-command handling, competing claims, stale fences and crash recovery. This is a preparatory vertical slice, not durability acceptance.
2. **Temporal integration is mandatory before durability acceptance.** Run that same slice through a deterministic Workflow and ordinary Activities. Add explicit timeout/retry policy, heartbeat/cancellation propagation and Continue-As-New state transfer. Use a pinned SDK; choose Python for a small toolbox-aligned implementation, Rust if retaining one language is material, or TypeScript if a Node service is already preferred. SDK availability alone cannot decide the repository boundary.
3. Require failure injection at commit-before-relay, delivery-before-ack, worker death during Activity, provider outcome-before-recording, cancellation during retry, stale-worker return and Continue-As-New. Prove no accepted message loss, no stale authoritative update, preserved IDs/order, replay compatibility, and visible uncertainty for unresolved external effects. A real Trae adapter must pass its separately verified start/steer/cancel/reconnect contract before claims of Trae-backed durability.

These are proposed acceptance conditions for the team-control-plane effort, not guarantees supplied by PostgreSQL, Temporal, MCP, Symphony or Trae individually. Architectural approval remains with the human; this research makes no code, deployment or merge decision.
