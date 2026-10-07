# pi-durable: source review for Corenius reuse

Read on 2026-10-06. Upstream repository: `earendil-works/pi`, commit **`ae92585d3b3e5f1e4b123d14a34314d826d8d9f5`** (`fix(durable): report settings failures when checking kept context retention`). All upstream links below pin this commit. Corenius comparison baseline is `d7f674829e9844d011869b711d64fcf55313f7f1`; its domain glossary explicitly describes intended concepts, not implementation readiness. This is research, not an integration design or approval.

**Assessment:** pi-durable is a substantial, working TypeScript agent harness with atomic local persistence, explicit task checkpoints, conversation state, queued submissions, structured task ownership, and committed UI observations. Its most reusable material for Corenius is the boundary between admitted work, externally performed effects, durable receipts, and observers. It is not a distributed execution controller, a process supervisor, or an authority boundary. A direct dependency would introduce a Node/pi-ai/Chord runtime beside Corenius's Rust runtime and Rust SQLite controller; copying selected semantics and conformance scenarios is a smaller initial commitment. That assessment is an inference from the implementation and contract limits documented below.

## Evidence vocabulary and scope

- **Advertised:** README or normative specification states the behavior.
- **Implemented:** a concrete source path implements it at this commit.
- **Tested here:** selected existing upstream tests were executed successfully in an isolated copy; this does not mean a production deployment or fault-injection campaign.
- **Inference:** a consequence or proposed application of those findings; not an upstream guarantee.

The package manifest says `1.0.4`, but this checkout includes an **Unreleased** section with breaking changes: storage scan ordering and `TaskRuntime.context()` options, plus task timestamps, Cloudflare support, and retained context caching. Findings apply to the commit, **not necessarily npm release 1.0.4**. The README explicitly calls the API experimental and subject to change without notice; the changelog dates initial 1.0.0 to October 1, 2026. [Package manifest][package] [Changelog][changelog] [README status][readme]

## Architecture and public API map

The implementation has three separable layers: `Storage` persists detached records atomically; `Session` serializes mutations, tracks documents, and publishes committed changes; `Harness` extends Session with tasks, models, conversations, submissions, and views. Extensions are installed code, while stored documents contain extension/tool/model names and choices. This separation is real, not only a diagram: `createSession()` exposes the kernel independently, and `HarnessImpl extends SessionImpl`. [Session kernel][session] [Harness implementation][harness] [Public exports][exports]

| Surface | APIs and records | What it buys a host |
|---|---|---|
| Kernel | `createSession(storage)`, `commit(tx => ...)`, `Storage`, `StorageWrite`, `Seq` | One serialized atomic mutation boundary across entries, documents, submissions, and task records |
| Conversation | `Harness.open`, `root`, `createConversation`, `conversation`, `fork`, `configure`, `context`, `entries`, `reset` | Immutable transcript ancestry, per-conversation choices, active model context |
| Admission | `submit`, `submission`, `Submission.status/wait/abort`, `requestId`, busy policies | Durable admission and settlement separate from a caller's wait |
| Documents | `defineDoc`, `defineDocFamily`, `tx.doc`, `snapshot`, `snapshotAsOf`, `watchDoc`, `documentState` | Typed JSON with scoped lifetimes, versions, Chord deltas, optional historical reads |
| Tasks | `defineTask`, phase map, `initial`, `abort`, `migrate`, `runtime.commit`, `memo`, `outcomes`, `waitForTask` | Explicit checkpoint state machines and durable ownership/waits |
| Models | `HarnessOptions.models: Models`; built-in `GenerationTask`, `CompactionTask` | pi-ai model resolution, streaming, retry, deferred polling, context compaction |
| Extensions | `createRegistry`, `defineExtension`, `defineTool`, `hook`, `section`, `wrapTool`, `wrapSection` | Named tools, prompt sections, hooks, versioned task implementations |
| Execution capability | `HarnessOptions.env`, `ExecutionEnv = FileSystem & Shell`; `/env/node` | Tools operate through a host-supplied environment |
| Observation | `watch`, `viewState`, `watchEvents`, `taskGraph`, `inspect`, `usage` | Committed state and derived events, live task tree, diagnostics |
| Backends/testing | `/storage/memory`, `/storage/jsonl[/node]`, `/storage/sqlite[/node|/cloudflare]`, `/testing` | Pluggable storage and environment conformance suites |

The table maps exported source APIs, not a claim that each export was exercised locally. [Exports][exports] [Task interfaces][task-api] [Harness option types][harness-types] [Storage contract][storage-contract]

## What “durable” means here

### One mutation line; persist before publish

**Implemented:** Session queues commit callbacks on one promise chain. The callback produces a staged batch; Storage receives it without the caller's cancellation signal; after success, Session adopts document state and publishes a commit sequence. A callback failure rolls back staged changes. `StorageRejected` specifically means no batch effect committed; other storage failures poison the Session because the outcome may be uncertain. Failed in-memory adoption after storage success also poisons it. This is a useful explicit treatment of ambiguous persistence outcomes. [Session settlement][session-settlement]

**Advertised and implemented:** observers see committed progress only. Generation and tools throttle partial/output commits (default 100 ms), so not every received token becomes durable. The surviving committed partial is authoritative; an uncommitted tail can disappear. `watchEvents` is a derived projection of committed views, not a separately persisted event journal, and slow consumers can receive a fresh snapshot instead of every intervening event batch. A Corenius consumer should treat it as a state synchronization interface, not an append-only audit feed. [Generation progress][generation-progress] [Progress defaults][agent-defaults] [Events implementation][events] [Event contract][event-readme]

**Tested here:** lifecycle tests cover cancellation during storage settlement, preventing close-time observer frames, rejecting late operations, and joining handlers before closing storage. Document tests cover rollback when checkpoint predicates throw and migration-only versus delta writes. [Lifecycle tests][lifecycle-tests] [Checkpoint/migration tests][document-tests]

A semantic gotcha: table reads after a table write within a transaction throw `ReadAfterWrite`. New conversation hooks run after creation, so they cannot freely scan tables. Document mutation tracking is a different mechanism. Plan table reads before writes; avoid placing slow external I/O in a Session callback, which holds the global mutation line. [Transaction guard][transaction-guard] [Creation hook contract][harness-types]

### Storage backends and failure bounds

| Backend | Implemented mechanism | Limits and local evidence |
|---|---|---|
| Memory | Reference semantics with copied records/read results | Useful for deterministic tests; no restart persistence |
| SQLite | One database transaction per Session commit; tables for conversations, entries, submissions, tasks, documents and deltas/bases | Node `DatabaseSync` behind async queued facade. WAL, `synchronous=NORMAL`, default busy timeout 5 s and checkpoint threshold 1,000 pages; newest commits can be lost on host/power failure. SQLite storage suite passed here |
| JSONL | Main commit markers plus per-document/per-live-task sidecars; reconstructs its memory representation on open | Sidecar append before marker; torn/unconfirmed tails truncated; malformed confirmed data rejects open. No global compaction. JSONL storage suite passed here |
| Cloudflare SQLite | Async facade over a Durable Object's `storage.sql` and `storage.transaction` | Adapter exists at this commit, but local tests use a Node SQLite stand-in, not deployed Cloudflare |

[Backend contract][storage-readme] [SQLite commit][sqlite-commit] [Node SQLite adapter][sqlite-node] [JSONL implementation][jsonl-commit] [JSONL recovery][jsonl-recovery] [Cloudflare test stand-in][cloudflare-tests] [Non-goals][non-goals]

**Important JSONL detail:** `fsync: true` flushes affected sidecars before appending the main marker. Ordinary publication does **not** fsync `main.jsonl`; a main-only commit has nothing to flush. The main file is flushed when needed to authorize destructive sidecar reclamation. Therefore acknowledged tail commits can still disappear after power loss. This is explicitly documented in the normative spec and asserted by the exact ordering test; it is not a discovered undocumented bug. Do not translate “fsync enabled” into stronger receipt durability. [JSONL specification][jsonl-spec] [Exact flush ordering test][jsonl-flush-test]

**Single-owner constraint:** the package requires one process per storage and provides no cross-process ownership lock. SQLite's physical locks and busy timeout do not make its in-memory Session/scheduler caches safe for multiple Harness writers. A custom SQL facade must queue operations behind transactions and invalidate the transaction handle after return; using the outer database inside its own transaction callback deadlocks by contract. [Storage ownership and facade contract][storage-readme]

There is a separate **experimental coding-agent frontend** that wraps session directories with `proper-lockfile` before opening the database. Its crash-stale lock handling demonstrates host-level exclusion, not leases/fencing built into Storage, and it is not evidence that the normal CLI is already replaced by pi-durable. [Experimental frontend][experimental] [Session lock implementation][session-lock]

## Tasks, recovery, external effects, and cancellation

### Explicit checkpoint machines, not replay of arbitrary program history

`defineTask` registers a name/version, initial checkpoint, exhaustive phase map, abort handler, and optional migration. The durable states are `pending`, `running`, `waiting`, `completing`, and `terminal`; outcomes distinguish `completed`, domain `failed`, `aborted`, `orphaned`, and runtime `faulted`. A phase must commit durable progress or a terminal outcome. Returning without progress faults the task. Task records replace current state; there is no automatic recording/replay of every external function call. [Task API][task-api] [Task states and receipts][task-states]

On open, surviving `running` tasks become `pending` at their last checkpoint. Opening does not dispatch their phase code; `resume()` or a progress API such as submit/wait enables scheduling. Registry definitions are not persisted. Missing/newer/incompatible definitions block work rather than silently substituting code; migration is applied at reservation. Local tests exercise missing definitions, failing migrations, replacement at phase boundaries, and rejection of late old-invocation commits. [Scheduler recovery][scheduler-open] [Recovery and migration tests][task-recovery-tests]

The intended effect recipe is **commit intent → perform idempotent external effect → commit receipt/checkpoint**. The specification's payment example explicitly relies on an external idempotency key. Durable memos are first-writer-wins JSON retained while a task is live, useful for fixing a key or child identity, but they do not transact with an external service. A crash after the effect and before its receipt still requires reconciliation or safe replay. [Task effect example][effect-spec] [Memo/runtime API][task-api]

**Corenius inference:** this is a useful template for durable controller transitions, but pi task ownership is not an `ExecutionScope`/`ScopeEpoch` authority system. Do not infer revocable execution grants, immutable admitted intent, remote effect receipts, capacity accounting, or quarantine from task checkpoints alone.

### Tool effects have an explicit ambiguity policy

The built-in `pi.tool` first resolves the tool, repairs/validates arguments, runs `beforeTool`, validates again, and persists `{phase: "execute", arguments, replay}` before calling `execute`. Default replay is `unsafe`. On recovery:

- If **both stored and current** policy are `safe`, it clears old progress and calls the currently selected implementation with the persisted final arguments.
- Otherwise it appends an error result saying the call was interrupted and may have partially run; it does not automatically rerun that tool task.
- Safe recovery builds the environment again using current conversation configuration.

These branches are implemented and tested against SQLite close/reopen, including changes to replay policy, environment/cwd, deselection, committed partial output, and a real bash process. [Tool implementation][tool] [Tool recovery tests][tool-recovery-tests]

Three important consequences follow:

1. **`safe` is an author assertion, not a proved idempotency property.** A safe tool may execute more than once; a default unsafe tool may have performed an unknown amount of external work. The harness does not roll back those effects.
2. **Recovery is not equivalent to a fresh permission decision.** The safe `execute` phase bypasses `beforeTool`, argument repair, and schema revalidation; it uses current implementation and stored arguments. Enforce current `ScopeEpoch`/workspace grants in the authority-owning adapter at each effect, or persist and reconcile an immutable operation grant. A selected extension hook alone is insufficient for Corenius's execution authority.
3. **A recovery error is not a quarantine workflow.** An interrupted unsafe call can be followed by a new model request and a new tool call, so “not automatically replayed” does not mean the overall agent will never repeat its external action.

The first two are direct consequences of the execution branch; the third is also demonstrated by the unsafe recovery test, which completes the submission after presenting the error to the model. These are integration caveats, not claims that pi-durable promises exactly-once external effects. [Tool recovery branch][tool] [Unsafe test][tool-recovery-tests]

Source inspection of the built-in read/write/edit/bash definitions found no `replay: "safe"`; the conservative default applies even to reads at this commit. [Read definition][read-tool] [Write definition][write-tool] [Edit definition][edit-tool] [Bash definition][bash-tool] Tool selection and hooks are extension-scoped: omitting a guard extension omits that guard. There is no registry-wide mandatory permission hook in this contract. [Tool definition default][tool-types] [Built-in tools][coding-tools] [Guard omission warning][gotchas]

### Structured ownership is stronger than a list of children

Tasks belong to their conversation or a same-conversation owner task. Conversations can themselves belong to tasks, so the scheduler traverses a combined ownership tree. Task outcome settlement is delayed in `completing` until foreground owned work settles. Failure/abort cascades down that tree; abort handlers run bottom-up. `waiting` parks task execution without a live handler; `failFast` is restricted to owned children, while `allSettled` can join other existing tasks. Waiting on oneself or an owner is rejected. [Ownership contracts][task-api] [Scheduler ownership invariant][scheduler-invariant] [Wait validation][wait-validation]

Background is a **traversal boundary**, not an OS process: only conversation-owned tasks can be background; ordinary idle/abort traversal skips their subtrees, while explicit broader abort can include them. The background-subagent example uses an immediately completed background anchor task to retain child-conversation provenance, then separate reporter tasks to deliver/report messages with request IDs. The foreground example scans for an existing owned child and reuses the submission key, making restart idempotence application-visible. [Background example][background-example] [Foreground example][foreground-example]

**Tested here:** ownership tests cover foreground completion holds, background exclusions, descendant abort after recovery, work admitted after a cascade, invocation-bound handles, replay-safe subagents reusing the same child/submission, and background supervisor resubmission. Those are substantial local semantic tests, not a distributed execution qualification. [Ownership tests][ownership-tests]

Concurrency is in-process async concurrency: the scheduler reserves every eligible task in one commit and starts each reservation. Tool rounds default to parallel, while a sequential tool can make its round sequential. There is no concurrency-budget field in `HarnessSettings`; the inspected scheduling loop is not a capacity scheduler. A host needing bounded subprocess slots or account-wide model limits must supply those controls separately. [Reservation loop][reservation] [Tool scheduling options][tool-types] [Settings][harness-types]

### Cancellation remains cooperative

A caller cancelling `wait()` cancels only that wait. An explicit task abort durably marks work before signalling it; late runtime commits are rejected, and abort cleanup gets a fresh invocation. Closing seals admission, signals invocations, joins them, and closes Storage without converting all live tasks to terminal outcomes; reopen can resume them. [Runtime contract][task-api] [Scheduler open/join][scheduler-open] [Harness close][harness-close]

An extension ignoring its cancellation signal keeps `close()` and potentially abort completion pending. A local test proves storage stays open until a deliberately stubborn handler returns. The spec explicitly excludes forced termination of non-cooperative in-process extension code. Independently supervised Corenius **Executions** therefore remain valuable: a JS promise is not evidence a process tree stopped or released capacity. A host should own termination/reconciliation outside this harness. [Non-cooperative close test][stubborn-test] [Non-goals][non-goals]

## Conversation, documents, and model semantics

### Conversations and submissions

Conversation handles are lightweight ID handles; transcript entries are immutable and forks retain ancestry at a visible entry. A reset is an entry advancing the active context head rather than deletion of old history. Fork document policy is explicit: `initial`, `current`, or `asOf` when history permits. Snapshot/fork semantics use the entry's commit sequence, not a wall-clock approximation. [Conversation implementation][harness] [Fork copy implementation][forks]

`submit()` atomically admits a user input or a transcript write and returns a durable submission handle. Busy input can reject, steer at a boundary, or follow up after the current run. `requestId` deduplicates **per conversation**, but only submission **type** is checked on reuse: supplying different same-type content returns the old ID without detecting a payload conflict. The existing test deliberately submits `"hi"` then `"different"` with the same key. External adapters must bind operation identity to payload/intent themselves if that distinction matters. [Admission implementation][submission-source] [Submission tests][submission-tests]

`Submission.abort()` withdraws queued work; already placed work reports `already_placed`. Stopping the caller's wait is not withdrawal and stopping a conversation is a different operation. This separation is useful for operator reattachment and RPC timeouts. [Submission abort/wait][submission-source]

### Documents and observational state

Document roots are JSON objects. Singleton and keyed-family definitions carry kind/version and initial/migrate functions. Session/task scope is latest state; conversation scope chooses latest versus rewindable and fork policy. Chord supplies tracked mutation deltas and replicated observations. Document checkpointing stores full bases according to `checkpointWhen`, otherwise deltas; no automatic heuristic is promised. Terminal task settlement retires its task-scoped documents, while terminal task receipts remain. [Document definitions][documents] [Document semantics][document-types] [Terminal retirement and checkpoints][transaction-documents]

Migrations on reads can materialize a new shape without writing it; the first successful transaction accessing it persists a migration base. Incompatible newer stored versions reject. Fork copies can preserve unknown persisted document kinds without requiring their definitions to be installed. These behaviors are useful for extension reload, but consumers must separate document shape versions from immutable admitted work identity. The migration/fork suites passed here. [Migration implementation][documents] [Migration tests][document-tests] [Fork tests][fork-tests]

### pi-ai integration and restart behavior

The host supplies a pi-ai `Models` catalog; `GenerationTask` resolves provider/model IDs and uses `streamSimple`. Its persisted phases are prepare/request/retry/poll/tools. Preparation fixes model, thinking, stream options, and transcript cutoff. A crash during a request resends from that cutoff; a committed partial becomes an aborted assistant entry. That is a new provider request, not continuation of a partially received byte stream, and can consume additional provider work. Before-request hooks run again and can alter the outgoing messages. [Generation phases][generation] [Streaming implementation][generation-progress] [Generation recovery tests][generation-tests]

Deferred model handles and poll deadlines are persisted, and recovery can resume `fetchDeferred`; abort attempts provider cancellation. Retry deadlines are checkpointed. A durable per-conversation UUIDv7 in `pi.provider` supplies provider session/cache affinity; children/forks receive new IDs. Affinity is not a provider guarantee of exactly-once generation. Local tests use faux model responses; provider-specific caching and live billing behavior were not tested here. [Deferred poll/cancel][generation-poll] [Provider identity][provider]

Compaction is itself a task, with select/summarize/retry checkpoints. It can run manually, at a background threshold, or as a blocking owned task; result placement is a transcript write with a new head and can become stale. This preserves historical transcript records while changing active model context. It is not storage vacuuming or deletion of old work evidence. [Compaction implementation][compaction] [Compaction behavior][compaction-readme]

## Environment seam and security implications

`HarnessOptions.env` receives conversation ID, current cwd, and committed document reads; it runs off the Session line and may construct/lookup an environment per use. `ExecutionEnv` combines an extensive async `FileSystem` with `Shell.exec(string | argv)` and cleanup. File namespace `id` distinguishes local/remote/container filesystems. This is the interface implemented/adapted by the separate pi-env package; durable itself already provides an unrestricted local Node implementation. [Environment factory][harness-types] [Environment interface][environment]

Built-in tools access files/processes through this interface, but arbitrary extension code still runs in the host process. Selecting fewer tools, changing cwd, or installing a permission hook is not an OS sandbox. Corenius's **Insula** boundary should remain the authority-owning realization of filesystem/process access. An adapter could expose operations against an existing live Insula and declared contract; it must not treat cwd as confinement or confuse a fresh JS environment object with a fresh Insula identity. This is a proposed adaptation, not an implemented integration.

`write`/`edit` use a process-local queue keyed by environment namespace ID and canonical path, including canonical parents for newly created files. The source explicitly says it is **not a lock against bash or other processes**. It cannot replace Corenius's one-writer-per-worktree rule or a workspace generation/fencing protocol. Correct namespace identity matters when wrapping multiple environments sharing one writable projection. [File mutation queue][file-queue]

## Concrete reuse options

These are proposals for evaluation, not decisions to replace current Corenius components.

| Option | Reuse | Boundary that should stay Corenius-owned | Evaluation criterion |
|---|---|---|---|
| Port selected semantics to Rust | Atomic intent/receipt commits, invocation fencing, poisoned-store handling, durable wait versus work cancellation, held completion | Rust SQLite controller, ExecutionScope authority, independent supervisor | Match chosen failure-cut scenarios with controller tests |
| Isolated Node worker pilot | pi-durable conversations, pi-ai generation, Chord views, tooling | Controller admits work; Insula adapter enforces effect grants; supervisor owns Node process | Kill/restart worker around intent/effect/receipt without duplicate admitted external operation |
| Read-only UI/state protocol study | Snapshot plus committed deltas, task graph projection, explicit resnapshot on overflow | Controller remains canonical ledger | Late attach and slow consumer converge without interpreting UI events as audit history |
| Environment-only adapter study | Durable FileSystem/Shell shape and conformance scenarios | Insula contract, namespace identity, command supervision, grant revocation | Path, cancellation, output spill, symlink, and namespace behavior match expected contract |
| Subagent admission patterns | Existing-child lookup plus stable per-conversation request IDs; separately checkpointed delivery/reporting | Immutable IntentRevision and AcceptanceContract binding | Payload/key mismatch and stale authority are rejected before effects |

The highest-value borrowing is the **failure matrix and tests**, even if no TypeScript ships: interrupted pre-intent vs post-intent effects, safe/unsafe replay, unknown persistence outcomes, cancellation racing a commit, child outcome held during descendants, and unavailable definitions after restart. A direct runtime adoption needs a protocol for immutable operation identity, authority revocation, workspace identity, external receipt reconciliation, and crash-safe host ownership beyond what the package supplies.

Corenius context: the current retained controller is Rust in `crates/sqlite-controller`; Python tools are launch/operator adapters. Its live two-worker qualification remains explicitly incomplete in `docs/efforts/sqlite-controller/IMPLEMENTATION-STATUS.md`. Neither that roadmap nor Corenius's glossary should be treated as already implemented guarantees when comparing against pi. This report does not audit those implementations; the parent spike owns that comparison.

## Maturity, dependencies, and licensing

The library is MIT-licensed and declares Node >=22.19.0. Direct dependencies are Chord, pi-ai, diff 8.0.4, and TypeBox 1.3.27; provider SDKs are dependencies of pi-ai, not direct durable dependencies. pi-ai's root source intentionally separates provider factories/API implementations from its core exports, so a faux-provider test does not require every live provider SDK to execute. Chord and pi-ai remain meaningful runtime/type/protocol dependencies for direct reuse. [Manifest][package] [pi-ai root][ai-root] [pi-ai manifest][ai-package] [MIT license][license]

The package is young by its own changelog and explicitly unstable despite a 1.x version. Its implementation is much more than a sketch: numerous targeted storage, session, task, generation, tool, ownership, view, and environment tests and 32 numbered examples exist. The local 301-test result is strong evidence for the selected contract paths, not evidence for multi-process safety, hostile-code containment, cloud deployment, production workload scale, or upstream release compatibility. [Changelog][changelog] [Examples index][examples]

## Verification performed and reproducibility

**Result:** 11 test files passed, **301 tests passed**, no selected tests failed; Vitest reported 3.76 seconds. Linux x86_64 (`5.15.152.bsk.9-amd64`), Node **v24.13.0**, npm **11.6.2**, Vitest **4.1.11**. Source commit remained read-only. Source and existing tests were copied into this research worktree's ignored `.scratch/pi-durable-review/`.

The first setup attempts failed during module resolution, not test assertions: missing local package links, then pi-ai's dist-only export entries. Final setup added `source` export conditions to **copied pi-ai/telemetry package manifests only**; runtime source, test source, and durable's Vitest config were unmodified. This avoids falsely calling the result an unmodified clean-checkout npm build. No API credentials or live models were used.

A reproducible equivalent setup, from a fresh working directory with the pinned checkout available at `$UPSTREAM`, is:

```bash
mkdir -p scratch-durable-review
cp -a "$UPSTREAM/packages" scratch-durable-review/
cd scratch-durable-review
cat > package.json <<'JSON'
{"private":true,"type":"module","dependencies":{"vitest":"4.1.11","typebox":"1.3.27","diff":"8.0.4","partial-json":"0.1.7","esbuild":"0.28.2"}}
JSON
npm install --ignore-scripts --no-audit --no-fund
mkdir -p node_modules/@earendil-works
ln -s ../../packages/chord node_modules/@earendil-works/chord
ln -s ../../packages/ai node_modules/@earendil-works/pi-ai
ln -s ../../packages/telemetry node_modules/@earendil-works/pi-telemetry
ln -s ../../packages/durable node_modules/@earendil-works/pi-durable
python3 - <<'PY'
import json, pathlib
for name in ['ai', 'telemetry']:
    p = pathlib.Path('packages') / name / 'package.json'
    d = json.loads(p.read_text())
    for key, value in list(d['exports'].items()):
        if not isinstance(value, dict):
            continue
        target = value.get('import')
        if not isinstance(target, str) or not target.startswith('./dist/'):
            continue
        source = target.replace('./dist/', './src/').removesuffix('.js') + '.ts'
        if '*' in key or (p.parent / source).exists():
            d['exports'][key] = {'source': source, **value}
    p.write_text(json.dumps(d, indent=2) + '\n')
PY
cd packages/durable
../../node_modules/.bin/vitest run \
  test/harness-tasks-recovery.test.ts \
  test/harness-tools-recovery.test.ts \
  test/harness-generation-recovery.test.ts \
  test/harness-ownership.test.ts \
  test/harness-lifecycle.test.ts \
  test/session-checkpoints-migrations.test.ts \
  test/jsonl-storage.test.ts \
  test/sqlite-storage.test.ts \
  test/sqlite-cloudflare.test.ts \
  test/session-forks.test.ts \
  test/harness-submissions.test.ts \
  --maxWorkers=2
```

Sanitized final output:

```text
Test Files  11 passed (11)
     Tests  301 passed (301)
  Start at  19:17:20
  Duration  3.76s (transform 1.17s, setup 0ms, import 3.02s, tests 3.11s, environment 1ms)
```

Limitations: this is a minimal dependency source-test environment, not the monorepo's pinned full build; npm may resolve different transitive versions on a later date. Node emitted its experimental SQLite warning. Generation/provider tests use faux models. Cloudflare tests use a Node SQLite stand-in. Task “crash” tests include controlled storage cuts with blocked handlers, while other tests use orderly close/reopen and a real interrupted bash command. There was no host power-loss test, process-kill campaign, live provider/cache test, cross-process writer test, full durable suite, performance benchmark, or Corenius integration test. [Crash simulation mechanism][crash-tests] [Cloudflare stand-in][cloudflare-tests] [Tool tests][tool-recovery-tests]

## Source references
[package]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/package.json#L1-L118
[changelog]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/CHANGELOG.md#L1-L89
[readme]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/README.md#L1-L9
[session]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/session/session.ts#L48-L115
[harness]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/harness.ts#L83-L177
[exports]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/index.ts#L1-L185
[storage-contract]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/types.ts#L1007-L1107
[task-api]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/types.ts#L145-L283
[harness-types]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/types.ts#L413-L469
[session-settlement]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/session/session.ts#L398-L466
[generation-progress]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/generation.ts#L350-L420
[agent-defaults]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/agent.ts#L19-L72
[events]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/events.ts#L137-L185
[event-readme]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/README.md#L368-L382
[lifecycle-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/harness-lifecycle.test.ts#L126-L335
[document-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/session-checkpoints-migrations.test.ts#L19-L646
[transaction-guard]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/session/transaction.ts#L918-L930
[storage-readme]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/README.md#L536-L556
[sqlite-commit]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/storage/sqlite/storage.ts#L174-L202
[sqlite-node]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/storage/sqlite/node.ts#L179-L209
[jsonl-commit]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/storage/jsonl/storage.ts#L252-L305
[jsonl-recovery]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/storage/jsonl/storage.ts#L534-L802
[cloudflare-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/sqlite-cloudflare.test.ts#L1-L57
[non-goals]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/docs/spec.md#L4733-L4747
[jsonl-spec]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/docs/spec.md#L4544-L4577
[jsonl-flush-test]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/jsonl-storage.test.ts#L533-L600
[experimental]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/coding-agent/src/experimental/durable/README.md#L1-L20
[session-lock]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/coding-agent/src/experimental/durable/sessions.ts#L45-L55
[task-states]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/types.ts#L442-L559
[scheduler-open]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/scheduler.ts#L238-L280
[task-recovery-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/harness-tasks-recovery.test.ts#L428-L854
[effect-spec]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/docs/spec.md#L1750-L1818
[tool]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/tool.ts#L33-L117
[tool-recovery-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/harness-tools-recovery.test.ts#L108-L400
[tool-types]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/types.ts#L212-L235
[coding-tools]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/tools/index.ts#L1-L25
[gotchas]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/docs/spec.md#L4690-L4723
[scheduler-invariant]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/scheduler.ts#L159-L172
[wait-validation]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/scheduler.ts#L1071-L1097
[background-example]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/examples/23-subagent-background.ts#L36-L117
[foreground-example]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/examples/22-subagent-foreground.ts#L39-L75
[ownership-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/harness-ownership.test.ts#L172-L963
[harness-close]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/harness.ts#L323-L330
[stubborn-test]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/harness-lifecycle.test.ts#L126-L148
[forks]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/session/forks.ts#L25-L96
[submission-source]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/submissions.ts#L75-L201
[submission-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/harness-submissions.test.ts#L82-L107
[documents]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/documents.ts#L38-L207
[document-types]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/types.ts#L41-L81
[transaction-documents]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/session/transaction.ts#L766-L848
[fork-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/session-forks.test.ts#L1-L80
[generation]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/generation.ts#L51-L209
[generation-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/harness-generation-recovery.test.ts#L40-L269
[generation-poll]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/generation.ts#L224-L262
[provider]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/provider.ts#L6-L38
[compaction]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/compaction.ts#L98-L169
[compaction-readme]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/README.md#L340-L366
[environment]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/env/index.ts#L170-L324
[file-queue]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/tools/file-mutation-queue.ts#L4-L54
[ai-root]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/ai/src/index.ts#L1-L48
[ai-package]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/ai/package.json
[license]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/LICENSE#L1-L21
[examples]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/README.md#L575-L615
[crash-tests]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/test/harness-tasks-recovery.test.ts#L245-L289
[read-tool]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/tools/read.ts#L73-L165
[write-tool]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/tools/write.ts#L16-L39
[edit-tool]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/tools/edit.ts#L90-L144
[bash-tool]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/tools/bash.ts#L125-L144
[reservation]: https://github.com/earendil-works/pi/blob/ae92585d3b3e5f1e4b123d14a34314d826d8d9f5/packages/durable/src/harness/scheduler.ts#L747-L811
