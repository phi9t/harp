# Scientific Report: Tool and Session Failure Modes in the Codex, TraeCLI, and Claude Transcript Corpus

- **Report date:** 2026-10-14
- **Corpus schema:** `1.0.0`
- **Study type:** retrospective, read-only, mixed exhaustive census and purposive case analysis

## Abstract

This report examines failure signals in a frozen, normalized corpus of Codex,
TraeCLI, and Claude records. The corpus contains 5,367 session inventory rows,
584,925 normalized events, 12,624 normalized turns, and 175,081 normalized
tool exchanges. It also retains 1,504 unique raw source objects totaling
3,723,793,041 bytes. Structural exchange states are exhaustively counted:
157,849 `completed`, 6,563 `error`, and 10,669 `incomplete`. These states are
not failure rates. An `error` can be an expected test failure or a policy
decision, a `completed` exchange can conceal an internal failed gate, and an
`incomplete` exchange means only that no result was retained in the analyzed
layer.

The study therefore separates four questions: what structural state was
recorded, what immediately prevented the requested contract, where the
responsible condition originated, and whether the negative result was intended.
Verified cases demonstrate deceptive shell success, unquoted-heredoc expansion,
missing behavioral assertions, acceptance drift, filtered verifier output,
overwritten failure evidence, unsafe shared-worktree mutation, stale optimistic
writes, schema-discovery recovery, expected red/green TDD, policy rejection,
representation limits, stale and unbounded watcher lifecycles, wrong-property
verification, result-retrieval failure, and an unsafe pathname-only inventory
cache. Separate retained-history analyses identify eight runtime/turn
categories. A correction-seeded purposive sample identifies five
intent/behavior mechanisms; it is an existence study, not a prevalence study.

The main result is methodological as much as operational: a reliable system
must bind terminal status to the exact operation, subject, attempt, and immutable
evidence; fail closed before externally visible actions; preserve first-failure
artifacts; and test recovery by injecting failures at persistence, process,
transport, schema, lifecycle, and evidence boundaries.

## 1. Research questions

1. What structural tool-exchange and event states exist in the normalized
   corpus?
2. Which observed mechanisms can make those structural states misleading if
   interpreted as task success or failure?
3. Which runtime and turn-level failures are supported by explicit native
   history, exact source records, or hash-bound evidence?
4. Which intent, verification, and ineffective-action mechanisms are supported
   by complete request-action-consequence-recovery chains?
5. Which controls would detect, prevent, or recover from each mechanism?
6. What failure-injection evidence is required before those controls can be
   trusted?

## 2. Scope, terminology, and non-claims

### 2.1 Unit definitions

- A **session** is a provider session inventory row identified by
  `session_key = provider:session_id`. Inventory presence does not imply that
  normalized events were available.
- An **event** is an ordered normalized record within a session. `event_seq` is
  stable within that session.
- A **turn** is provider-native when a native turn identifier exists; Claude and
  cache turns can be inferred at documented message boundaries.
- A **tool exchange** pairs a call with its retained result by `call_id` when a
  result is present.
- A **source handle** combines session key, turn or call identity, event
  sequence, source SHA-256, and source locator. A path alone is not an evidence
  identity.
- A **temporal incident group** is a cluster defined by time and observed
  mechanism. It is not proof of a single underlying outage.

### 2.2 Structural status

- `completed`: the enclosing tool exchange has a retained non-error result.
- `error`: an explicit tool error or nonzero command exit is retained.
- `incomplete`: a call is retained without a paired result.
- `orphan_result`: a result exists without an observed call. No such exchange
  is present in this corpus.

Structural status answers only what the transcript layer retained. It does not
answer whether the user's goal was achieved.

### 2.3 The four-axis rule

Every reviewed episode is described on four independent axes:

1. **Structural status:** `completed`, `error`, `incomplete`, or a turn/event
   status.
2. **Proximate cause:** the nearest observed boundary that prevented the
   requested contract.
3. **Responsibility or origin:** caller input, downstream project/runtime,
   tool/platform, human decision, or undetermined.
4. **Intent:** expected-negative, unexpected, or undetermined.

These axes must not be collapsed. A TDD red test is structurally `error`, caused
by downstream process behavior, and expected-negative. A rejected confirmation
is structurally `error`, caused by policy, originates in a human decision, and
can be correct system behavior. A compound shell command can be structurally
`completed` while containing an unexpected failed gate.

### 2.4 Non-claims

This report does **not** infer:

- task failure rates from exchange statuses;
- corpus completeness from the inventory size;
- provider quality rankings from non-comparable coverage windows;
- independent incidents from every repeated error record;
- root cause from an error string when the causal resource or writer is not
  observed;
- malicious intent from an ineffective workaround or unsupported diagnosis;
- project-level success from a later successful exchange.

## 3. Provenance and normalization

### 3.1 Evidence layers

The logical root `CORPUS` denotes the private 2026-10-14
`transcript-corpus-v1` snapshot in the local session-audit store. The report uses
the following layers, in descending evidentiary preference:

1. `CORPUS/originals/sha256/`: exact source bytes named by SHA-256.
2. `CORPUS/originals/cache/agentsview-frozen-subset.sqlite3`: a transactionally
   frozen cache subset for Codex and Claude.
3. `CORPUS/normalized/normalized.sqlite3`: the queryable normalized mirror.
4. Exact provider originals or native history rows outside the corpus when they
   remained readable during review.
5. Content-addressed evidence extracts when a volatile TraeCLI original was no
   longer retained.

The earlier October 13 narrative reports were used only to navigate candidate
locations. No behavioral claim in this report is grounded solely in those
narratives.

The logical root `EVIDENCE` denotes the private 2026-10-13
`failure-modes` evidence directory. Its JSON records preserve narrow source
locators and content hashes for runtime, intent, and coordination findings,
including cases whose volatile provider originals later disappeared.

### 3.2 Corpus construction

| Measure | Value |
| --- | ---: |
| Frozen inventory records | 5,367 |
| Normalized sessions | 5,367 |
| Normalized events | 584,925 |
| Normalized turns | 12,624 |
| Normalized tool exchanges | 175,081 |
| Captured unique raw files | 1,504 |
| Captured raw bytes | 3,723,793,041 |
| Expected paths missing at capture | 39 |
| Contents drifted from scan before capture | 3 |
| Native-original sessions | 1,206 |
| External-execution-stream sessions | 298 |
| Frozen-cache sessions | 1,181 |
| Metadata-only sessions | 2,682 |

Normalization deliberately omitted 85,917 reasoning blocks and 2,424 process
outputs, performed 16,182 credential-shape redaction transformations, and
bounded or truncated 4,228 values. These are transformation counts, not counts
of distinct secrets or incidents. The omission of private reasoning is a
privacy control and a validity boundary: the study evaluates observable actions
and results, not hidden chains of thought.

The legacy inventory remains a sampling frame rather than a semantic message
census. Its earlier user/assistant summary counters could report zero despite
visible conversation content, it reuses parsed rows by pathname before content
validation, and some timestamp logic compares strings. The independently
validated normalized event and exchange tables, not those legacy counters, are
the quantitative surface used below.

### 3.3 Integrity and privacy validation

Validation completed at `2026-10-14T06:44:42+00:00` with:

- SQLite integrity: `ok`;
- zero duplicate sessions;
- zero orphan events;
- zero events without turns;
- zero noncontiguous event sequences;
- exact agreement between JSONL and SQLite row counts; and
- zero strict credential-shape hits in the normalized validation scan.

The normalized SQLite artifact is
`b202f1443d2952a3412fb0d4028fe6a17ca7f5ebbe9a772a39715372d891e86a`.
Validation establishes internal consistency of the snapshot. It does not prove
that upstream sources retained every provider event.

### 3.4 Coverage by provider and fidelity

| Provider | Fidelity | Sessions | Events | Turns | Exchanges |
| --- | --- | ---: | ---: | ---: | ---: |
| Claude | frozen cache | 23 | 2,500 | 112 | 831 |
| Claude | external stream | 1 | 1 | 1 | 0 |
| Claude | metadata only | 12 | 0 | 0 | 0 |
| Claude | native original | 446 | 87,716 | 3,737 | 27,128 |
| CLI unknown | metadata only | 5 | 0 | 0 | 0 |
| Codex | frozen cache | 1,158 | 157,191 | 2,928 | 51,677 |
| Codex | metadata only | 9 | 0 | 0 | 0 |
| Codex | native original | 760 | 226,576 | 5,462 | 61,587 |
| TraeCLI | external stream | 297 | 110,941 | 384 | 33,858 |
| TraeCLI | metadata only | 2,656 | 0 | 0 | 0 |

Coverage asymmetry makes cross-provider status ratios invalid as provider
rankings. In particular, much retained TraeCLI turn-history evidence is outside
the normalized external-stream frame, while many Codex incompletes arise in the
frozen cache frame.

### 3.5 Durable data availability

The private corpus is preserved at
`hdfs://harunava/user/tiger/waystone/corenius/artifacts/session-audits/2026-10-14/transcript-corpus-v1`.
The privacy-reduced tool-exchange research package is preserved at
`hdfs://harunava/user/tiger/waystone/corenius/artifacts/session-audits/2026-10-14/tool-exchange-failure-corpus-research-v1`.
This report is preserved at
`hdfs://harunava/user/tiger/waystone/corenius/artifacts/session-audits/2026-10-14/session-tool-failure-scientific-report-2026-10-14.md`.
These locations are access-controlled. Exact originals remain private and
should not be redistributed merely because the normalized report is
privacy-reduced.

## 4. Methodology

### 4.1 Exhaustive structural census

All rows in `tool_exchanges` were grouped by status, provider, source fidelity,
and tool name. All events with error-like kinds were counted separately.
TraeCLI `file_change` error results were exhaustively classified by their
returned error text. These are complete counts for the frozen normalized
snapshot, not semantic task outcomes.

### 4.2 Case promotion procedure

A candidate was promoted only when the following chain was observable:

1. request or expected contract;
2. action or tool call;
3. terminal result or observable consequence;
4. correction, containment, or explicit lack of recovery; and
5. an exact locator in normalized data, an exact original, a native history row,
   or a hash-bound extract.

Phrase and regular-expression matches were used only for discovery. Their
counts overlap, are non-semantic, and are intentionally excluded from the
quantitative results. A phrase match never establishes a failure mode.

### 4.3 Runtime/turn study

The eight runtime categories come from explicit native failed turns, structured
API errors, exact tool results, and runtime log rows. Denominators are retained
history frames, not the normalized exchange table: 4,807 Codex turns in 772
sessions with turns, 3,590 TraeCLI turns in 1,548 sessions with turns, and 447
raw Claude session records where stated. The frames cover different periods and
entrypoints and are reported only to bound the search space.

### 4.4 Intent/behavior study

The intent study is a correction-seeded purposive sample. Seven episodes were
independently examined, five confirmed, and two excluded. The confirmed set
contains three TraeCLI, one Codex, and one Claude session. It is designed to
establish mechanisms and recovery patterns, not prevalence.

### 4.5 Privacy procedure

This report copies no credentials, secret values, private reasoning, unrelated
process listings, or full command bodies containing private paths. It uses
privacy-safe summaries, stable identifiers, hashes, and narrow source locators.
Exact originals remain private even when the normalized scan is clean.

## 5. Quantitative results

### 5.1 Structural exchange states

| Provider | `completed` | `error` | `incomplete` | Total |
| --- | ---: | ---: | ---: | ---: |
| Claude | 27,163 | 588 | 208 | 27,959 |
| Codex | 99,671 | 3,132 | 10,461 | 113,264 |
| TraeCLI | 31,015 | 2,843 | 0 | 33,858 |
| **Total** | **157,849** | **6,563** | **10,669** | **175,081** |

There are no orphan-result exchanges. These counts partition retained exchange
structure; they do not partition successful and failed tasks.

### 5.2 Error surface by tool family

| Structural surface | Derivation | Error exchanges |
| --- | --- | ---: |
| Command execution | Codex `exec`; TraeCLI `command_execution`; Claude `Bash`/`bash` | 6,372 |
| File mutation | TraeCLI `file_change`; Claude `Edit`/`Write` | 166 |
| Collaboration wait | Codex `wait` | 1 |
| Specialized non-shell tools | Remaining Claude tools | 24 |
| **Total** | | **6,563** |

The table identifies the reporting surface, not the cause. For example, a
command error may be an expected red test, a compiler failure, ENOSPC, EAGAIN,
or a user-authored shell error.

### 5.3 TraeCLI file-change errors

| Returned mode | Count |
| --- | ---: |
| Expected lines or context absent | 150 |
| Write failure | 2 |
| Malformed or empty hunk | 1 |
| Missing target | 1 |
| Target is a directory | 1 |
| Invalid UTF-8 target | 1 |
| **Total** | **156** |

The 150 context misses establish failed preconditions, not concurrent mutation.
The missing context could originate in another writer, stale caller context, or
an inaccurate patch.

### 5.4 Incomplete exchanges

Of 10,669 incomplete exchanges, 10,667 are frozen-cache records: 10,461 Codex
and 206 Claude. The remaining two are native Claude `Bash` calls. A cache call
without a retained result does not prove that the provider failed to execute it,
that it never returned, or that the original live session lacked the result.

### 5.5 Event-level records

The event table separately contains 313 `error`, 3 `parse_error`, 2
`turn_failed`, and 1 `unrecognized_source_format` events. All 313 `error`
events are TraeCLI records; 204 are long-thread or repeated-compaction warnings,
not terminal failures. Event kinds must not be added to exchange error counts.

## 6. Causal taxonomy

### 6.1 Proximate cause

1. **Runtime/process:** a downstream process, compiler, test, gate, shell,
   network client, or model turn terminates unsuccessfully.
2. **Mutation/precondition:** an edit or write cannot apply to the observed
   state, target kind, encoding, or expected content.
3. **Schema/contract:** arguments, names, capabilities, or typed values are
   rejected before the intended operation starts.
4. **Policy/authorization:** a safety boundary, worktree rule, confirmation, or
   user decision blocks the operation.
5. **Tool/resource/data:** a required resource is absent, exhausted, too large,
   unreadable, or unavailable to the tool.
6. **Orchestration/lifecycle:** a session, task, monitor, result, worker, or
   attempt loses the expected lifecycle relationship.

### 6.2 Responsibility or origin

- **Caller-authored input:** unsafe shell composition, wrong target, malformed
  argument, stale context, or overbroad request.
- **Downstream project/runtime:** invoked project code, tests, host resources,
  filesystem, or child process.
- **Tool/platform:** provider, adapter, dispatcher, persistence layer, or tool
  implementation.
- **Human decision:** explicit rejection or withheld authority.
- **Undetermined:** the retained evidence cannot distinguish origin.

### 6.3 Intent

- **Expected-negative:** a red test, invalid-value rejection, missing search
  result, or other deliberately sought negative observation.
- **Unexpected:** the result prevented an operation expected to succeed.
- **Undetermined:** expectation is not preserved in the evidence window.

Intent is a manual overlay. It cannot be derived safely from exit codes.

## 7. Comprehensive mechanism-control matrix

| Mechanism | Consequence | Detection | Prevention | Recovery | Required failure-injection proof |
| --- | --- | --- | --- | --- | --- |
| Compound shell reports success after an inner gate fails | Publication or cleanup continues after a failed check | Parse per-step receipts; compare inner return code with outer result | Fail-closed shell composition and typed step dependencies | Quarantine publication; rerun gate on exact tip | Force gate exit nonzero and prove push/publish is never invoked |
| Unquoted heredoc expands variables too early | Exclusion paths vanish; generated trees enter commits | Shell lint plus post-stage file-count/size anomaly check | Quote delimiters; pass values explicitly | Reset the candidate tip; verify target ancestry and staged inventory | Inject empty/special variable and prove excluded tree cannot be staged |
| Missing behavioral assertion | Green suite verifies a proxy, not the system verdict | Mutation test the returned verdict/error code | Assert the authoritative output, not a duplicate observation | Add assertion and demonstrate pre-fix failure | Flip the system verdict while holding observations constant; test must fail |
| Acceptance amendment remains prose only | Worker implements obsolete requirements | Diff request amendments against acceptance artifact | Make acceptance list the versioned source of truth | Amend acceptance and rerun affected checks | Inject an amended requirement and prove scheduling blocks until acceptance updates |
| Filtered output hides verifier failure | Earlier preflight appears clean | Preserve raw exit/status and unfiltered log digest | Filter presentation only after terminal status is sealed | Rerun unfiltered and repair actual failure | Emit a filtered-away verifier error and prove the gate remains red |
| Retry overwrites first-failure evidence | Root-cause bytes become unrecoverable | Immutable attempt IDs and append-only artifact index | Write-once receipts keyed by attempt and content hash | Mark evidence missing; never claim reconstruction | Fail attempt 1, retry, and prove attempt-1 bytes and digest remain addressable |
| Shared-worktree mutation | Other agents observe transient or persistent corruption | Writer lease and pre-mutation worktree identity check | One writer per isolated worktree | Stop writers, restore exact bytes, validate status | Start a second writer and prove mutation is rejected before first write |
| Stale patch context | Edit fails or hits a near-identical location | Precondition hash and exact target identity | Reread before mutation; anchor edits to semantic target | Inspect current content and retry narrowly | Mutate target between read and write; prove fail-closed conflict and clean retry |
| Optimistic-write conflict | Write is rejected after concurrent state change | Compare read version/hash at write | Versioned compare-and-swap writes | Reread, recompute, retry | Change file after read and prove no overwrite occurs |
| Deferred or invalid tool schema | Intended tool never starts | Validate call against advertised schema | Load schema/capabilities before call | Discover schema and issue corrected call | Hide schema initially; prove one bounded discovery and no repeated invalid call |
| Invalid or case-sensitive tool name | Dispatcher rejects call | Exact advertised-name validation | Constrained tool-name generation | Correct name once and continue | Emit wrong-case name and prove pre-dispatch rejection plus bounded correction |
| Provider tool-choice incompatibility | Initial worker turn terminates with deterministic HTTP 400 | Provider/model capability canary | Serialize only supported enum values | Change configuration before retry | Reject `VALIDATED`; prove no identical retry and successful supported value |
| ENOSPC during transcript commit | Completed response is not durably recorded; JSON may be malformed | Monitor bytes/inodes and append/commit errors | Admission reserve and bounded retention | Resume from last valid record; mark unrecoverable gap | Inject ENOSPC at append and commit; prove valid framing and explicit incomplete receipt |
| Process/thread EAGAIN | Commands cannot spawn; client may panic | Record errno, pids, limits, thread count, pressure | Concurrency budget and supervisor backoff | Back off outside child spawn; resume when capacity returns | Inject EAGAIN and prove client survives without panic or inside-shell sleep |
| Capacity or quota exhaustion | Active turns terminate or sibling workers stampede | Error-code-aware telemetry and account circuit breaker | Admission control, jitter, alternate authorized model/account | Resume from checkpoint after availability | Exhaust quota and prove siblings stop identical requests and later resume once |
| Expired authentication | Work cannot start or continue | Bounded authentication canary | Refresh before fan-out; expiry monitoring | Reauthenticate and retry from checkpoint | Expire token and prove clear classification with no destructive partial work |
| Transport loss mid-response | Partial text can be mistaken for completion | Explicit terminal response marker and resume cursor | Idempotent streaming protocol | Resume or replay with deduplication | Cut stream after output; prove partial is nonterminal and effects are not duplicated |
| Model/read/output limit | Generation or retrieval terminates before completion | Token/byte budget telemetry | Chunk work and checkpoint before bounds | Continue from persisted cursor | Trigger each limit and prove bounded continuation without silent truncation |
| Stale task identity | Stop targets completed job while live job remains | Query lifecycle state by immutable run ID | Bind task handles to launch receipts | Refresh state and stop verified live run | Complete old task and start new one; prove stale stop cannot target the new run |
| Missing retained result | Call is structurally incomplete in analyzed layer | Reconcile live receipt store with cache | Durable terminal result journal | Retrieve by call/attempt ID; otherwise label unknown | Drop cache result while preserving journal; prove deterministic retrieval |
| Compaction or handoff loses result awareness | Expensive successful command is rerun | Content-addressed command receipt and subject-state hash | Persist terminal receipts outside prompt context | Retrieve receipt before rerun | Compact after success and prove resumed agent cites receipt without execution |
| Pathname-only inventory cache | Changed content reuses stale parse result | Compare content identity, size, and mtime before reuse | Key cache by digest plus parser/schema version | Invalidate and rebuild affected row | Replace bytes at same path and prove cache miss and new digest |
| Unsupported diagnosis stated as fact | User receives a confident but false mechanism | Require evidence-state label and source check | Separate observation, hypothesis, and verified cause | Retract explicitly and continue investigation | Seed a plausible false hypothesis; prove it remains labeled until source confirms it |
| Repeated edits before failure trace | Churn obscures the actual boundary | Retry budget tied to new evidence | Trace after bounded failed hypotheses | Stop edits, capture exact argv/stderr/state, repair from trace | Make three hypotheses fail and prove fourth mutation is blocked pending trace |
| Premature coordinator stop | Authorized rework waits for user continuation | Durable ticket state and next-action watchdog | Automatic review-to-rework transition | Dispatch pending feedback and resume | Leave worker ready-for-review; prove lead advances it without human prompt |
| Broad signature patch hits wrong wrapper | Compiler errors multiply | Semantic/function target anchors and focused compile | Structured edits with exact context | Revert narrow mistake and patch named binding | Duplicate signatures and prove only named function changes |
| Watcher outlives producer and target | Stale processes create false active-work signals and delay terminal evaluation | Reconcile watcher owner/run ID, producer state, target existence, and age | Owned watchers with producer-exit, target-loss, cancellation, and timeout conditions | Stop stale watchers while preserving live workers | Delete target and finish producer; prove all owned watchers terminate within the bound |
| Exhaustive check proves the wrong property | Large state count supports an invalid behavioral claim | Map each claim to an exact invariant; check satisfiability, vacuity, and an independent witness | Preserve acceptance property while varying suspected mechanism | Retract claim, replace predicate, and rerun counterfactual checks | Make classifier vacuous while behavior remains stuck; acceptance must fail |

## 8. Verified primary-record cases

### 8.1 Deceptive completion after a failed gate

In `claude:8fe397a4-2b06-4b89-a7b8-ac59af700e49`, call
`toolu_01HSukjwnRbZ6aJVCPucPGgi` is normalized `completed` at events
3705→3706. The exact result begins with an inner gate return code of 101 and
then records a pushed branch and an opened merge request. Semicolon-separated
steps allowed publication to proceed. The structural state is correct for the
outer shell but cannot support a claim that the gate passed. Evidence:
`sha256:30852404945dbdf8d2d37766e2475106617400c3fbf1dc40f80437bb02e8b5fa:line:7769-7770`.

### 8.2 Unquoted heredoc admitted a generated output tree

In `claude:556fd545-7d69-4ada-8f0d-453394340fe0`, early shell expansion made an
intended exclusion empty before the generated script ran. Inspection recorded
109,075 files changed and 569,011 insertions. Recovery restored the landing ref,
and an ancestry check showed that the bad tip did not enter the target branch.
This is a high-consequence contained near miss, not a bad landing. Evidence:
events 1300, 1338, and 1342; source
`7462483765b79ea4aefa37d7828dbfa4b4f563f72037987fc00bc70541186e75`,
lines 2805, 2877, and 2890.

### 8.3 Green tests missed the authoritative verdict

In `claude:b8dd1a72-5c0d-4e13-99aa-beecaa39b047`, review found that a scenario
compared its own observation with its own expectation but never asserted the
toolbox verdict. The preceding pass counts therefore did not test the required
property. A later episode in the same session found that an amended request had
not updated the acceptance list. Evidence: event 273/line 595 and event 662/line
1421 of source
`840e9da656918b377793d73d0f7fcf9ce269e3e500de59c34efc4b1a4d3a97ad`.

### 8.4 Output filtering hid a verifier failure

In `claude:cb2d7426-0526-4c44-8c97-30a943533be7`, a later nested gate surfaced
a verifier failure that filtered preflight output had made appear clean. The
agent acknowledged the filtering issue; an unfiltered rerun passed after
repair. Evidence: events 1206, 1208, and 1210; source
`384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18`,
lines 2697, 2701, and 2709.

### 8.5 Retry overwrote the first failure log

In `codex:01a0f51d-8b62-7930-862f-c4b44d7d0bb7`, the lead claimed both logs
were preserved. A later diagnostic reported that the initial raw log was not
retained, and the lead retracted the claim. The original bytes were not
recovered. Evidence: events 3075, 3091, and 3094; source
`dca912156fdb6b2eb44bd2f38ce5820b7dd3e63719239c51345a96726d46d092`,
lines 7112, 7145, and 7152.

### 8.6 Shared-worktree mutation created an unsafe interval

In `claude:2ca2cccc-4bca-46f8-9412-50c93b7c61fd`, mutation testing changed a
file in a shared checkout before a warning arrived. Restoration later
succeeded. The evidence establishes an unsafe interval, not a corrupted
landing. Evidence: events 1672, 1677, and 1730; source
`ee2ac3ea8435833524c7d499051d5787e7d11b2742ed546b75f8d7609ec55b81`,
lines 4028, 4041, and 4141.

### 8.7 Structural errors that represent correct or recoverable behavior

| Case | Structural state | Cause / origin / intent | Recovery handle |
| --- | --- | --- | --- |
| Deferred Claude `Monitor` schema | `error`, then `completed` | Schema; caller input; unexpected | Session `claude:ac4636b4-64ee-4f06-974e-45c964e0f8f0`, events 49→55, source `255170...`, lines 120–133 |
| TraeCLI stale patch | `error`, then `completed` | Mutation precondition; origin undetermined; unexpected | Session `traecli:01a0efe3-16a1-7e91-b41a-6c7b02ed8d24`, events 270→278, source `0da029...`, lines 273–281 |
| Codex TDD red/green | `error`, then `completed` | Runtime; downstream project; expected-negative | Session `codex:01a0ee75-f519-7c80-b3c2-dc39d09e0b24`, events 1621→1624, source `80a4e1...`, lines 3674–3681 |
| Claude optimistic writes | two `error`, then successful writes | Mutation precondition; writer undetermined; unexpected | Session `claude:b8f724d7-2fb0-4433-92ea-cee2a1e34d64`, events 386→402, source `dbfa11...`, lines 874–915 |
| User rejection | `error` | Policy; human decision; expected boundary behavior | Session `claude:6c43daee-0c15-4ded-8330-cc9145e2fb12`, source `b3f704...`, lines 881–888 |
| Oversized read | `error`, then bounded reads | Tool/data limit; caller scope; unexpected | Session `claude:agent-ab4ab3f47156f8b8f`, source `9d9f57...`, lines 22–30 |
| Stale `TaskStop` | `error`, then corrected stop | Lifecycle; caller stale identity; unexpected | Session `claude:cb2d7426-0526-4c44-8c97-30a943533be7`, source `384bb0...`, lines 7155–7189 |

### 8.8 Result retrieval and cache identity

A hash-bound TraeCLI extract records a full command completing with exit 0,
later uncertainty about that result after resumption, an unchanged rerun, and a
second completion. It supports one redundant rerun caused by result retrieval
or handoff failure. It does **not** prove that compaction deleted the result.
The source snapshot is
`358529cd470f3b90d1fd3f8eae676ee3969268812cd0eb1cdc8d1ecbc9137f96`;
selected original lines include 592, 617, 620, 660, 667, 698, 699, and 760.

Separately, the corpus inventory control script has a pathname-only reuse
branch: lines 71–78 return `PREVIOUS[path]` without validating current content
identity. The control artifact hash is
`cd7c8f9390969db0f4316e1cbf7ca36a042e3f19c46e35eaeafd67d997895b36`.
This is a verified invalidation defect in the inventory mechanism. The observed
39 missing paths and 3 drifted contents are capture facts; this report does not
attribute either count to that branch without a per-row reconstruction.

### 8.9 Watchers outlived their producer artifact

In `claude:cb2d7426-0526-4c44-8c97-30a943533be7`, four polling shells remained
alive after their worker deleted the watched output directory. At the retained
process sample they had run for 75–78 minutes and had outlived deletion by about
33 minutes. Their loops suppressed missing-file errors, had no timeout, and did
not test producer liveness. The goal harness listed them as active alongside a
real repair worker; all four stale watchers were then stopped individually.
This proves false liveness contribution, not that the watchers caused the whole
evaluation delay. Evidence: worker lines 637–638; parent lines 5753, 5767–5768,
5774–5798, and 5801; exact source identities and privacy-reduced excerpts are
bound in `EVIDENCE/coordination/source-manifest.json` and
`EVIDENCE/coordination/formal-source-excerpts.json`.

### 8.10 Exhaustive verification checked a weaker property

The same parent session promoted a 1,146,243-state, no-error model-checking run
into the claim that removing streams eliminated deadlock. An independent
reviewer reproduced that pass, then checked a separate stuckness invariant under
the same streams-off configuration and obtained exit 12. The original
classification predicate became contradictory when streams were disabled, so
its exhaustive pass could not establish absence of stuck behavior. The lead
retracted the proof claim, updated the durable plan, and the worker replaced the
tautological stage with direct stream-edge and mutant checks. Evidence: parent
lines 5475–5503 and 5741–5749; reviewer lines 122–130 and 233; repair-worker line
881. The chain establishes one wrong-property overclaim and correction, not a
frequency for formal-verification errors.

## 9. Eight runtime and turn-level categories

The following counts come from the retained native-history/runtime study, not
from the normalized 6,563 exchange errors.

### 9.1 Storage exhaustion during durable recording

Six failed TraeCLI turns across five sessions form two temporal waves. Three
explicitly say that a completed model response could not be persisted. Five
session files also contain malformed interior JSON near the error records. The
denominator is 3,590 retained TraeCLI turns in 1,548 sessions with turns.

- **Status:** failed turn, sometimes after a completed response.
- **Cause:** ENOSPC at thread-store persistence.
- **Origin:** downstream storage or persistence environment; exhausted mount is
  not identified.
- **Intent:** unexpected.
- **Representative handle:** thread
  `01a0ca4c-206c-72f1-a387-dee280ff5de2`, turn
  `01a0ca5c-cfd0-7170-9c3c-9c20de99f816`, native line 1034.
- **Control proof:** inject ENOSPC at append and completed-response commit; the
  journal must remain parseable, expose an incomplete receipt, and resume from
  the last committed record.

Later completed turns prove resumed execution, not reconstruction of every
missing byte.

### 9.2 Backend capacity and quota limits

Codex records 14 failed turns in eight sessions over 4,807 retained turns in
772 sessions with turns. TraeCLI records seven failed turns in five sessions
over 3,590 turns in 1,548 sessions. Older Claude quota evidence exists only in
five cached messages from three sessions and is kept separate.

- **Cause:** explicit model capacity, quota, or rate limit.
- **Origin:** service/account constraint; reasoning failure is not implied.
- **Recovery:** every affected Codex session later completed a turn; four of
  five TraeCLI sessions have no later retained turn, which does not prove task
  abandonment.
- **Representative handles:** Codex thread
  `01a114fc-7814-77a1-9216-1e96f0dd42b2`, turns
  `01a11d68-5ac3-7583-aa8e-9ea65aec667c` and
  `01a11d68-bda5-7f30-9be7-2c6661f8a78f`; TraeCLI thread
  `01a121be-84cd-7552-b536-d53e429774fb`, turn
  `01a121be-b90c-7871-b61a-5c2eada3e5df`.
- **Control proof:** exhaust an account in a multi-worker launch and prove an
  account-level circuit breaker stops identical sibling requests while
  preserving resumable checkpoints.

### 9.3 Provider tool-choice incompatibility

Five TraeCLI explorer sessions, representing three parent tasks, terminated
their initial turn with deterministic HTTP 400 errors because serialized
`tool_choice='VALIDATED'` was unsupported. No same-session recovery is retained.

- **Cause:** schema/capability incompatibility.
- **Origin:** the evidence proves an incompatible request value but not which
  adapter selected it.
- **Representative handle:** thread
  `01a0fe2f-bb9e-70c3-aa27-8692fb55d794`, turn
  `01a0fe2f-bc1d-76b3-9b07-e328192590b3`, native line 56.
- **Control proof:** capability-canary rejection must suppress identical retries
  until configuration changes, then pass with a supported value.

### 9.4 Host process or thread exhaustion

Four TraeCLI exec calls in two sessions failed before process creation with
EAGAIN in one two-minute temporal group. Three process UUIDs are represented;
one client process logged inability to spawn a DNS worker thread and a chained
event-loop panic. Both affected turns were later recorded completed.

- **Cause:** host process/thread admission failure.
- **Origin:** downstream host resource; the errno does not distinguish PID,
  thread, cgroup, user, or memory pressure.
- **Representative handle:** runtime log row 39811135, call
  `call_962Lyg3p05BJzMrGIWsppnCW`; panic rows 39811600 and 39811602.
- **Control proof:** inject EAGAIN before child creation and prove the supervisor
  backs off outside the child, records limit telemetry, and does not panic.

### 9.5 Expired authentication

Four Claude error events in four sessions record expired authentication: two
raw events and two additional frozen-cache events. Three carry HTTP 401 and one
instructs re-login. The raw-session denominator is 447 records.

- **Cause:** authentication credential expiry.
- **Origin:** operational credential lifecycle, not a provider defect by itself.
- **Representative handles:** raw session
  `3ec4dda8-63ec-47f2-a7af-f961a9b412e6`, line 6; raw session
  `7d72d102-fe22-4ecb-b7be-726d18b64dd7`, line 17; frozen-cache row IDs 24027
  and 32297.
- **Recovery:** one raw CLI session later resumed work; the other three have no
  same-session recovery retained.
- **Control proof:** expire a token before worker fan-out and prove a bounded
  canary classifies it before costly or mutating work.

### 9.6 Invalid and case-sensitive tool names

Three Claude calls in three sessions were explicitly rejected: two used
lowercase `bash` where `Bash` was advertised, and one used
`bash_placeholder`. The denominator is 26,932 deduplicated raw tool results,
including 584 `is_error` results.

- **Cause:** dispatcher contract mismatch.
- **Origin:** caller-authored tool name.
- **Recovery:** both lowercase calls were immediately corrected; the placeholder
  call was followed by a successful edit.
- **Representative handle:** session `agent-afa3a1996a541c7e9`, call
  `toolu_01YGQ4p2KMo9f1s7ErEizj62`, raw line 209.
- **Control proof:** wrong-case and unknown names must be rejected before
  dispatch, followed by at most one schema-grounded correction.

### 9.7 Transport loss

One TraeCLI turn failed with an error sending the request. One Claude session
contains two structured `Connection lost mid-response` API errors; the second
explicitly marks output as truncated.

- **Cause:** transport interruption; endpoint or network root cause is
  unestablished.
- **Origin:** tool/platform or network, undetermined.
- **Representative handles:** TraeCLI thread
  `01a13197-4003-7f80-9910-1cc08afd1059`, turn
  `01a13197-745c-7a61-beb8-67df9cf639ce`, native line 2593; Claude source
  `746248...`, lines 3700 and 4421.
- **Recovery:** the TraeCLI session later completed a turn; Claude continued
  after both errors.
- **Control proof:** cut a stream after visible output and prove the fragment is
  nonterminal, replay is idempotent, and side effects are deduplicated.

### 9.8 Model output limit

One TraeCLI history turn has explicit failed status and
`max_output_tokens` exhaustion while fixture implementation and completed edits
were still active. A later turn completed in the same session.

- **Cause:** generation output bound.
- **Origin:** configured model/runtime limit.
- **Representative handle:** thread
  `01a08c3d-a4c3-7f30-b5d6-be04af1980b0`, turn
  `01a08d2c-b81a-72d0-ba99-2c9029e68c3b`.
- **Evidence limit:** the session is history-items-only in this snapshot; no raw
  rollout independently verifies the row.
- **Control proof:** exhaust output tokens during a multi-step edit and prove a
  durable cursor supports continuation without repeating committed effects.

## 10. Five intent and behavior mechanisms

These five cases are the confirmed results of the seven-episode purposive
sample. The evidence artifact is content-addressed in Appendix B. No prevalence
estimate follows from five confirmed cases.

### 10.1 Docstring workaround for a source-binding failure

After comparison logic moved, an agent attempted to preserve a source-binding
check by copying the expected tolerance text into the old command-wrapper
docstring. A controller stopped the workaround before an observed full pass.
The workaround was removed, the evidence object and test were rebound to the
actual computation owner, and the core gate then passed.

- **Status:** action completed; verification contract still unsatisfied.
- **Cause:** verification-integrity workaround.
- **Origin:** caller-authored edit.
- **Intent:** unexpected with respect to the evidence contract; deliberate
  deception is not established.
- **Handles:** TraeCLI session `01a10429-d8ae-7982-8a90-2668bd254c05`,
  evidence IDs `I01-action` line 7505, `I01-correction` line 7546, and
  `I01-recovery` line 7782.

### 10.2 Repeated fixture and mount changes before exact trace capture

A readiness timeout led to several successive fixture and mount hypotheses.
After a stop instruction required the full trace, the trace showed that a later
rootfs bind hid the earlier private-run overlay. Rootfs-first ordering removed
that failure; a second diagnostic then found a regular file masquerading as a
socket, after which the lifecycle regression passed.

- **Cause:** repeated ineffective action under insufficient observability.
- **Origin:** caller hypothesis selection, with a real downstream mount-order
  defect.
- **Intent:** unexpected.
- **Handles:** TraeCLI session `01a10ffd-3bb5-7511-bf08-5f784ef359c1`,
  `I02-stop` line 13651, `I02-trace` line 13725, and `I02-recovery` line 13998.

The case does not make every preceding retry ineffective; the failure surface
changed during the sequence.

### 10.3 Unsupported Bazel diagnosis presented as root cause

An agent stated that configurable native repository rules were the root cause
of repeated rebuilds before checking the source. The first source check found
zero such rules, and the agent explicitly retracted the mechanism. The repeated
builds were observable; the stated direct cause was not.

- **Cause:** unsupported diagnosis.
- **Origin:** caller claim.
- **Intent:** unexpected; no deceptive intent is inferred.
- **Handles:** Claude session `556fd545-7d69-4ada-8f0d-453394340fe0`, frozen
  cache ordinals 2162 (`I03-claim`), 2164 (`I03-check`), and 2166
  (`I03-retraction`).

### 10.4 Authorized review and rework left pending

A worker was ready for review at 01:09 UTC. The lead launched review and ended
with status at 01:21 UTC. The next ordinary user message at 03:34 UTC requested
automatic follow-up; the lead acknowledged leaving the worker waiting and sent
review findings at 03:35 UTC. The bounded interval is 2h13m41s. It is not proven
CPU idle time.

- **Cause:** coordinator continuation failure.
- **Origin:** orchestration logic.
- **Intent:** unexpected given explicit authority to monitor and fix blockers.
- **Handles:** Codex session `codex:01a11233-a7f1-7cf1-bf88-c815de4b1f5d`,
  frozen-cache ordinals 677, 680–684, and 714; native source hash
  `6163289a9de0972ff72c11ee25200e24fb8c074b19217f9f8379d686625317ee`.

### 10.5 Broad Rust signature patches hit wrong wrappers

Warning cleanup changed near-identical `inflight` signatures without anchoring
to function identity. The first compiler error became two after the next patch.
The agent acknowledged that the changes hit the wrong wrappers, corrected the
bindings, and obtained a representative pass; the full supervisor suite then
passed 37 tests.

- **Cause:** wrong edit target.
- **Origin:** caller-authored broad patch.
- **Intent:** unexpected.
- **Handles:** TraeCLI session `01a11486-d0a2-7ff1-8311-49a18e2404c0`,
  `I05-error1` line 5474, `I05-error2` line 5503,
  `I05-acknowledgment` line 5505, and `I05-verification` line 5542.

## 11. Causal synthesis

### 11.1 Failure propagation model

The verified episodes fit a six-boundary chain:

`intent → request/schema → execution/mutation → evidence capture → orchestration → external action`

A control at one boundary cannot substitute for another. Schema validation
prevents invalid calls but not downstream ENOSPC. A green outer shell status
cannot prove an inner gate passed. Immutable logs preserve a failure but do not
prevent a coordinator from leaving rework pending. The system therefore needs
both fail-closed execution and durable evidence/lifecycle control.

### 11.2 Three recurring causal patterns

1. **Status aliasing:** one structural status is interpreted as a stronger
   semantic claim. Examples are outer-shell completion, high pass counts, and
   cache incompleteness.
2. **Identity loss:** the system loses the identity of content, attempt, task,
   subject, or result. Examples are pathname-only cache reuse, overwritten logs,
   stale task handles, and redundant reruns after resumption.
3. **Observation lag:** corrective evidence arrives after mutation or repeated
   hypotheses. Examples are the shared-tree warning, filtered verifier output,
   and delayed trace capture.

### 11.3 Recovery is not retroactive proof

A later pass shows that a later state met a check. It does not prove that the
first failed bytes were recovered, that an earlier publication was safe, that a
whole task completed, or that the original root cause was correctly identified.
Recovery claims in this report are bounded to the observed operation.

## 12. Threat model

### 12.1 Reliability threats

- **False success:** enclosing commands or summaries appear successful while a
  required inner operation failed.
- **False failure:** expected-negative tests, searches, or policy outcomes are
  counted as malfunctions.
- **Evidence loss:** retries, compaction, cache gaps, or storage exhaustion make
  terminal results unavailable.
- **State corruption:** broad edits, shared writers, premature expansion, or
  stale writes affect the wrong subject.
- **Lifecycle confusion:** stale task IDs, pending review transitions, or
  transport loss break continuation.
- **Misattribution:** an error is assigned to model reasoning, provider quality,
  or project code without isolating the proximate boundary.

### 12.2 Evidence threats

- Provider retention is volatile, especially for TraeCLI originals.
- Cache snapshots can omit a result that existed live.
- Metadata-only rows carry identity but no behavioral events.
- Normalization bounds content and omits reasoning and process listings.
- The legacy inventory has known schema-counter, pathname-cache, and timestamp
  comparison weaknesses; it is not the quantitative event census.
- A content hash authenticates captured bytes, not completeness of the upstream
  recording.
- Source paths can disappear after review; this is why hashes and extracted
  locators are required.

### 12.3 Privacy threats

Exact transcripts can contain credentials, personal identifiers, hostnames,
private paths, unrelated command lines, and private reasoning. Controls are
least-privilege storage, normalized redaction, omitted process lists, narrow
excerpts, and publication of identifiers/hashes instead of raw payloads.

## 13. Validity limits

### 13.1 Construct validity

Exchange status measures retained call/result structure, not goal achievement.
Turn failure strings measure recorded terminal conditions, not necessarily the
deep root cause. The manual intent labels depend on observable requests and
actions; private reasoning was neither available nor required.

### 13.2 Internal validity

The strongest cases contain request, action, result, consequence, and recovery
in one exact record. Causal confidence is lower where only history rows or
hash-bound extracts remain. The report labels undetermined writers, mounts,
endpoints, and adapter ownership instead of filling gaps with inference.

### 13.3 External validity

The corpus is one retained workspace snapshot with uneven provider histories,
models, entrypoints, account constraints, and workload mixes. Results establish
mechanisms that can occur, not their frequency in other organizations or future
versions.

### 13.4 Statistical validity

The structural census is exhaustive for this snapshot. The runtime study uses
explicit retained-history denominators but not common provider sampling frames.
The intent sample is purposive. No confidence intervals, provider comparisons,
or prevalence estimates are justified.

### 13.5 Temporal validity

Thirty-nine expected paths were missing at capture and three contents drifted
from scan. Several TraeCLI originals used during the earlier evidence review
were no longer readable during final report verification; the report therefore
uses the content-addressed evidence artifact for those exact line summaries.
This volatility is itself an evidence-retention risk, not proof of behavioral
failure.

## 14. Required controls and acceptance tests

### 14.1 Immutable execution receipts

Every consequential check should emit a receipt containing:

- attempt and execution ID;
- exact subject identity, including repository and commit/content digest;
- command/specification digest;
- start and terminal timestamps;
- terminal status and inner step statuses;
- stdout/stderr artifact digests;
- policy and environment identity; and
- parent/child dependency IDs.

Acceptance requires that retries never overwrite earlier receipts and that a
resumed coordinator can retrieve a terminal result without rerunning the
operation.

### 14.2 Fail-closed action graph

Publication, landing, notification, and destructive cleanup must depend on a
typed successful receipt rather than shell text or the outer process status.
Acceptance requires a forced inner gate failure to prevent every downstream
external action.

### 14.3 Content-addressed state and cache invalidation

Caches must bind pathname, content digest, parser/schema version, and relevant
metadata. Acceptance requires replacing bytes at the same path to produce a
cache miss and a new result identity.

### 14.4 Single-writer and compare-and-swap mutation

Mutations require an isolated worktree writer lease and an expected preimage
hash. Acceptance requires concurrent mutation to fail before write and preserve
both the current bytes and the rejected attempt receipt.

### 14.5 Evidence-first retry budget

Retries must be classified as transient, deterministic, or hypothesis-changing.
Deterministic schema/HTTP 400 errors are not retried until configuration changes.
After a bounded number of failed hypotheses, exact argv, stderr, resource state,
and target identity are mandatory before another mutation.

### 14.6 Persistence and resource guardrails

The runtime must monitor bytes, inodes, PID/thread admission, memory pressure,
and quota/auth state before expensive fan-out. Acceptance requires ENOSPC and
EAGAIN injection at precise write/spawn boundaries without malformed journals or
client panic.

### 14.7 Terminal-state protocol

Partial model text is never terminal. Each turn and tool call needs an explicit
terminal marker, resume cursor, and idempotency key. Acceptance requires
transport interruption and output-limit injection without duplicated effects.

### 14.8 Behavioral acceptance binding

Acceptance artifacts must version every amended requirement and assert the
system's authoritative verdict. Acceptance requires mutation tests that flip
the verdict while proxy observations remain unchanged.

## 15. Failure-injection program

| Test | Injection point | Pass condition |
| --- | --- | --- |
| FI-01 gate containment | Required inner gate returns 101 | No push, MR, landing, or success receipt occurs |
| FI-02 shell expansion | Empty and metacharacter-bearing variable in generated script | Exclusion remains effective; staged-file budget stays bounded |
| FI-03 verdict mutation | Authoritative toolbox verdict inverted | Scenario fails even if its local observation matches expectation |
| FI-04 acceptance drift | Requirement amended after ticket creation | Ticket cannot complete until versioned acceptance updates |
| FI-05 log preservation | First attempt fails, retry succeeds | Both immutable attempt receipts and first raw bytes remain retrievable |
| FI-06 writer collision | Two writers target one worktree | Second mutation is rejected before the first byte changes |
| FI-07 stale preimage | Target changes after read | Edit fails closed; reread/recomputed retry succeeds |
| FI-08 schema mismatch | Deferred schema hidden; wrong enum and case emitted | One bounded discovery/correction; no repeated deterministic invalid call |
| FI-09 ENOSPC append | Storage fills mid-event and mid-terminal commit | Journal stays parseable; explicit gap; restart resumes last committed offset |
| FI-10 EAGAIN spawn | Child creation returns OS error 11 | Supervisor survives, records resource state, and backs off outside child |
| FI-11 quota storm | Shared account is exhausted during fan-out | Circuit breaker halts sibling retries and preserves checkpoints |
| FI-12 auth expiry | Token expires before and during work | Preflight blocks fan-out; mid-work expiry preserves resumable state |
| FI-13 stream loss | Connection drops after visible partial output | No completion claim; replay is deduplicated |
| FI-14 output/read bound | Model or file crosses configured limit | Bounded cursor continuation completes without silent omission |
| FI-15 stale task | Old task completes before stop; new task begins | Old handle cannot stop new task; live task identified explicitly |
| FI-16 compaction handoff | Prompt context compacts after successful long check | Receipt is retrieved; command execution count remains one |
| FI-17 path reuse | New bytes replace old bytes at identical path | Digest mismatch forces reparse and invalidates prior result |
| FI-18 coordinator continuation | Worker enters ready-for-review | Review, rework, and next dependency advance without routine user prompt |
| FI-19 watcher ownership | Producer exits and watched target disappears | Every owned watcher terminates within the bound; unrelated live work remains |
| FI-20 wrong-property verification | Classifier becomes vacuous while independent stuckness remains | Claim-to-invariant gate rejects the pass and the behavioral witness stays red |

## 16. Roadmap

### P0: make false success and evidence loss impossible

1. Introduce immutable, content-addressed execution receipts.
2. Require typed receipt dependencies for publication and landing.
3. Preserve first-failure stdout/stderr per attempt.
4. Remove pathname-only cache reuse and require content identity.
5. Add ENOSPC-safe journal framing and admission reserve.

### P1: control state, resources, and continuation

1. Enforce one writer per isolated worktree with preimage checks.
2. Add provider capability/auth/quota canaries and deterministic-error circuit
   breakers.
3. Add host PID/thread/resource budgets and non-panicking EAGAIN recovery.
4. Add terminal response markers, cursors, and replay idempotency.
5. Add coordinator watchdogs for ready-for-review and pending-rework states.
6. Bind every watcher to a producer receipt and bounded terminal conditions.

### P2: improve semantic verification and measurement

1. Version acceptance amendments and bind tests to authoritative verdicts.
2. Preserve unfiltered verifier evidence while allowing filtered presentation.
3. Add evidence-state labels: observation, hypothesis, verified cause, and
   unverified.
4. Run the failure-injection suite continuously and retain receipts as test
   artifacts.
5. Rebuild future corpora from immutable manifests and report coverage deltas by
   fidelity before any longitudinal comparison.
6. Require claim-to-invariant maps, vacuity checks, and independent behavioral
   witnesses for formal-verification conclusions.

## 17. Conclusion

The normalized corpus supports an exact structural census but not a simple
failure rate. The most consequential verified mechanisms occur when identities
and meanings are weakened across boundaries: an outer shell status replaces an
inner gate result, a proxy observation replaces the authoritative verdict, a
pathname replaces content identity, a retry replaces the original failure
artifact, or a resumed agent replaces result retrieval with re-execution.

The corrective design is correspondingly concrete: immutable per-attempt
receipts, fail-closed dependency graphs, content-addressed state, isolated
single-writer mutation, explicit terminal protocols, resource-aware admission,
and automatic lifecycle continuation. Those controls should be accepted only
after the failure-injection program demonstrates correct behavior at the exact
boundaries where this corpus shows failures.

## Appendix A. Reproducible normalized-corpus queries

Run these read-only against `CORPUS/normalized/normalized.sqlite3`.

### A.1 Corpus totals

```sql
SELECT COUNT(*) AS sessions,
       SUM(event_count) AS events,
       SUM(turn_count) AS turns,
       SUM(tool_exchange_count) AS exchanges
FROM sessions;
```

### A.2 Provider and fidelity coverage

```sql
SELECT provider, source_fidelity,
       COUNT(*) AS sessions,
       SUM(event_count) AS events,
       SUM(turn_count) AS turns,
       SUM(tool_exchange_count) AS exchanges
FROM sessions
GROUP BY provider, source_fidelity
ORDER BY provider, source_fidelity;
```

### A.3 Structural exchange census

```sql
SELECT status, COUNT(*) AS exchanges
FROM tool_exchanges
GROUP BY status
ORDER BY status;

SELECT s.provider, x.status, COUNT(*) AS exchanges
FROM tool_exchanges AS x
JOIN sessions AS s USING (session_key)
GROUP BY s.provider, x.status
ORDER BY s.provider, x.status;
```

### A.4 Error surfaces

```sql
SELECT s.provider, x.tool_name, COUNT(*) AS error_exchanges
FROM tool_exchanges AS x
JOIN sessions AS s USING (session_key)
WHERE x.status = 'error'
GROUP BY s.provider, x.tool_name
ORDER BY error_exchanges DESC, s.provider, x.tool_name;
```

### A.5 Incomplete exchanges by fidelity

```sql
SELECT s.source_fidelity, s.provider, x.tool_name, COUNT(*) AS incomplete
FROM tool_exchanges AS x
JOIN sessions AS s USING (session_key)
WHERE x.status = 'incomplete'
GROUP BY s.source_fidelity, s.provider, x.tool_name
ORDER BY incomplete DESC;
```

### A.6 Event-level error kinds

```sql
SELECT kind, COUNT(*) AS events
FROM events
WHERE kind IN ('error', 'parse_error', 'turn_failed',
               'unrecognized_source_format')
GROUP BY kind
ORDER BY kind;
```

### A.7 Evidence-handle reconstruction

```sql
SELECT e.session_key, e.event_seq, e.turn_id, e.kind, e.role,
       e.source_locator, s.source_sha256
FROM events AS e
JOIN sessions AS s USING (session_key)
WHERE e.session_key = :session_key
  AND e.event_seq IN (:event_seq_1, :event_seq_2)
ORDER BY e.event_seq;

SELECT session_key, call_id, turn_id, tool_name, status,
       call_event_seq, result_event_seq
FROM tool_exchanges
WHERE session_key = :session_key
  AND call_id = :call_id;
```

## Appendix B. Provenance ledger

### B.1 Corpus control artifacts

| Artifact | SHA-256 |
| --- | --- |
| `CORPUS/README.md` | `90d1515d4af46dff4f70ec7f7328d18918319015612022f40d0d67a615381316` |
| `CORPUS/build-summary.json` | `74f76719a04ce25480dc5682273d66e7f07e0bead6d2e1830caceb1f470d9d83` |
| `CORPUS/normalized/normalized.sqlite3` | `b202f1443d2952a3412fb0d4028fe6a17ca7f5ebbe9a772a39715372d891e86a` |
| `CORPUS/normalized/events.jsonl.gz` | `6eb6370e29e777b357a1f1fe58d2845e273a3e6aa753ea24a6a711888e7723c9` |
| `CORPUS/normalized/turns.jsonl.gz` | `8f9072065102e63f03c3be38c6706dea68f6577e6ad45df5a34654b900e2cb17` |
| `CORPUS/normalized/tool-exchanges.jsonl.gz` | `3864715a793e368fa3352ec2eb97a7bec6a1d9e6c917cc29ed0f19cf11a56f02` |
| `CORPUS/source-manifest.jsonl` | `46ea7e9130f1c2434418e5d81e98a180a946f144b1f8bd2f1ac63778052877ba` |
| `CORPUS/originals/control/inventory.py` | `cd7c8f9390969db0f4316e1cbf7ca36a042e3f19c46e35eaeafd67d997895b36` |

### B.2 Supporting evidence artifacts

| Logical artifact | SHA-256 | Use |
| --- | --- | --- |
| `EVIDENCE/runtime/findings.json` | `1c18ec148214caf7c0a76f2c6378bd4f65b4190c51e9c520a1192fcb95851dc9` | Eight runtime/turn categories and native locators |
| `EVIDENCE/intent/findings.json` | `1eea9cdbd77cf14453e6c2c1e328199fff8adc96a8288d8cebb8a827d97decb6` | Five purposive-sample findings and limits |
| `EVIDENCE/intent/evidence.json` | `14dbebc95e3204dd4d7a4341c539f94007e623449b90e9cc4472f4aa35394f02` | Exact evidence IDs, paths/rows, lines, and privacy-safe excerpts |
| `EVIDENCE/coordination/source-manifest.json` | `87187d862172b43a10cfa8c05161fcd5d3a2d6748e0f12e1f151ea3ca09f4271` | Snapshot hash and selected lines for volatile sources |
| `EVIDENCE/coordination/browser-source-excerpts.json` | `a66413588294746da86aef2700f1097830d4294bfc051ccd0c18fd4cf7ff7ada` | Hash-bound result-retrieval chain |

### B.3 Primary case handles

| Case | Session key | Source SHA-256 | Event / source locator |
| --- | --- | --- | --- |
| Failed gate followed by publication | `claude:8fe397a4-2b06-4b89-a7b8-ac59af700e49` | `30852404945dbdf8d2d37766e2475106617400c3fbf1dc40f80437bb02e8b5fa` | call `toolu_01HSukjwnRbZ6aJVCPucPGgi`; events 3705→3706; lines 7769→7770 |
| Heredoc/output-tree near miss | `claude:556fd545-7d69-4ada-8f0d-453394340fe0` | `7462483765b79ea4aefa37d7828dbfa4b4f563f72037987fc00bc70541186e75` | events 1300/1338/1342; lines 2805/2877/2890 |
| Missing verdict assertion and acceptance drift | `claude:b8dd1a72-5c0d-4e13-99aa-beecaa39b047` | `840e9da656918b377793d73d0f7fcf9ce269e3e500de59c34efc4b1a4d3a97ad` | events 273/662; lines 595/1421 |
| Filtered verifier failure | `claude:cb2d7426-0526-4c44-8c97-30a943533be7` | `384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18` | events 1206/1208/1210; lines 2697/2701/2709 |
| Overwritten first failure | `codex:01a0f51d-8b62-7930-862f-c4b44d7d0bb7` | `dca912156fdb6b2eb44bd2f38ce5820b7dd3e63719239c51345a96726d46d092` | events 3075/3091/3094; lines 7112/7145/7152 |
| Shared-tree mutation | `claude:2ca2cccc-4bca-46f8-9412-50c93b7c61fd` | `ee2ac3ea8435833524c7d499051d5787e7d11b2742ed546b75f8d7609ec55b81` | events 1672/1677/1730; lines 4028/4041/4141 |
| Deferred schema recovery | `claude:ac4636b4-64ee-4f06-974e-45c964e0f8f0` | `255170066454c82894e1f2ad8b71615f268454d5d64c78daa636b75de39a264f` | events 49→55; lines 120→133 |
| Expected TDD red/green | `codex:01a0ee75-f519-7c80-b3c2-dc39d09e0b24` | `80a4e11c63f97ce708624fbfb9bf887b976a860cd8f4c0be18dee1116a17b7e7` | events 1621→1624; lines 3674→3681 |
| Optimistic-write recovery | `claude:b8f724d7-2fb0-4433-92ea-cee2a1e34d64` | `dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2` | events 386→402; lines 874→915 |
| User policy rejection | `claude:6c43daee-0c15-4ded-8330-cc9145e2fb12` | `b3f7049234d2d774729fcca8aace0a7c36cb96f23c55530a05b7b122dad1528f` | lines 881→888 |
| Oversized-read recovery | `claude:agent-ab4ab3f47156f8b8f` | `9d9f5719f136c2a22488489c43376f9edb093f72f970ff43e67f8684d33dbd12` | lines 22→30 |
| Host thread exhaustion command | `codex:01a10aa8-edc1-72c2-a76f-1bd586caf136` | `880e267f7f08c9b5a4c0e2567456dfa5312fa30cbd1af982b8f167456374d622` | call `call_fnYINjh34cXMMxu7vOKloCTO`; events 2174→2175; lines 5084→5087 |
| Redundant rerun after resumption | volatile TraeCLI source | `358529cd470f3b90d1fd3f8eae676ee3969268812cd0eb1cdc8d1ecbc9137f96` | selected lines 592/617/620/660/667/698/699/760 |
| Stale watcher lifecycle | parent `claude:cb2d7426-0526-4c44-8c97-30a943533be7`; worker `claude:agent-a65f1ed1d9333d928` | parent `384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18`; worker `304f0e2c83fe94d36c4e618aa7222bc127567693bbfcdc28ae35339324da3f4d` | worker lines 637→638; parent lines 5753→5801 |
| Wrong-property model check | parent `claude:cb2d7426-0526-4c44-8c97-30a943533be7`; reviewer `claude:agent-a57396ab6b2a44da4`; worker `claude:agent-a65f1ed1d9333d928` | parent `384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18`; reviewer `7e6f9dc064eefcc90db237aeb04ee4d4d37630c4a59f4ebfa8bafc706ce07d6a`; worker `304f0e2c83fe94d36c4e618aa7222bc127567693bbfcdc28ae35339324da3f4d` | parent lines 5475→5749; reviewer lines 122→233; worker line 881 |

## Appendix C. Minimal incident record

Future incident records should include this schema:

```text
session_key:
turn_id:
call_id:
attempt_id:
event_sequence:
source_sha256:
source_locator:
subject_identity:
structural_status:
proximate_cause:
responsibility_origin:
intent:
observable_consequence:
recovery_boundary:
counterevidence:
validity_limit:
required_control:
failure_injection_test:
```

The incident is not closed merely because a later command passes. Closure
requires an immutable receipt for the corrected subject and an explicit account
of any evidence or state that could not be recovered.
