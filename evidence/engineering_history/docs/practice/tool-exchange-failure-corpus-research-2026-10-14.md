# Tool-exchange failure corpus research

Research date: 2026-10-14 UTC. Repository read:
`e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`.

## Question and answer

This report asks what failure modes are actually present in the normalized
October 14 Codex, TraeCLI and Claude transcript corpus, how the normalized
statuses should be interpreted, and whether representative recovery and
deceptive-success chains survive comparison with the exact provider originals.

The structural census is exhaustive for the normalized corpus: 175,081 tool
exchanges divide into 157,849 `completed`, 6,563 `error`, and 10,669
`incomplete` exchanges. The 6,563 explicit errors can be partitioned exactly by
provider and tool surface. They cannot be partitioned automatically into
"unexpected agent failures," because the same nonzero command result can be an
expected red TDD test, an intentional search miss, a policy rejection, or a
genuine runtime defect. Causal and intent labels therefore have to remain
separate.

Nine promoted chains were checked against hash-addressed originals. Together
they cover every proximate-cause class in the taxonomy and show:

1. a Claude schema failure followed by tool discovery and a corrected call;
2. a TraeCLI stale patch followed by inspection and a successful targeted
   patch;
3. a Codex expected TDD failure followed by an implementation change and a
   passing rerun; and
4. a Claude exchange normalized as completed even though a gate embedded in a
   compound shell command failed before the command pushed a branch and opened
   an MR;
5. an optimistic-write conflict followed by reread and successful retry;
6. an explicit user rejection that correctly blocked one tool call without
   implying a runtime defect;
7. an oversized read followed by bounded reads;
8. a task-stop race caused by targeting an already completed job; and
9. host thread exhaustion reported through an otherwise ordinary test failure.

The fourth case is the central semantic warning: normalized green exchange
status is not proof that every operation embedded in a shell command succeeded.
Conversely, `incomplete` means that no result was retained; it is not proof
that the operation failed at runtime.

## Evidence frame

The authoritative research inputs are:

- the normalized SQLite database
  [`normalized.sqlite3`](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/normalized/normalized.sqlite3);
- the corpus fidelity and status definitions in the
  [README](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/README.md:26);
- the capture inventory in
  [`source-manifest.jsonl`](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl);
- the normalizer implementation, especially its
  [error predicate](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/control/build_dataset-final.py:243)
  and [exchange completion logic](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/control/build_dataset-final.py:749); and
- the nine hash-addressed provider originals cited in the promoted chains
  below.

The October 13 investigation was used only as a discovery index. Behavioral
claims in this report come from the October 14 normalized records and their
underlying originals, not from the earlier audit's descriptions.

The database exposes three relevant levels:

- `sessions` records provider, source fidelity, source hash and aggregate
  counts;
- `events` records normalized event kind, turn identity, event sequence and
  exact source locator; and
- `tool_exchanges` joins one call to its retained result and records
  `completed`, `error`, `incomplete`, or `orphan_result` status.

The stable evidence handles are `session_key`, `call_id`, `turn_id`,
`call_event_seq`, `result_event_seq`, `call_source_locator`,
`result_source_locator`, and `source_sha256`. Session names or pathnames alone
are not sufficient provenance.

## Corpus shape and source fidelity

The normalized database contains:

| Unit | Count |
| --- | ---: |
| Sessions | 5,367 |
| Events | 584,925 |
| Turns | 12,624 |
| Tool exchanges | 175,081 |

Session and normalized-content coverage by provider and source fidelity is:

| Provider | Source fidelity | Sessions | Events | Turns | Exchanges |
| --- | --- | ---: | ---: | ---: | ---: |
| Claude | `agentsview_cache` | 23 | 2,500 | 112 | 831 |
| Claude | `external_execution_stream` | 1 | 1 | 1 | 0 |
| Claude | `metadata_only` | 12 | 0 | 0 | 0 |
| Claude | `native_original` | 446 | 87,716 | 3,737 | 27,128 |
| Codex | `agentsview_cache` | 1,158 | 157,191 | 2,928 | 51,677 |
| Codex | `metadata_only` | 9 | 0 | 0 | 0 |
| Codex | `native_original` | 760 | 226,576 | 5,462 | 61,587 |
| TraeCLI | `external_execution_stream` | 297 | 110,941 | 384 | 33,858 |
| TraeCLI | `metadata_only` | 2,656 | 0 | 0 | 0 |
| Unresolved CLI | `metadata_only` | 5 | 0 | 0 | 0 |

The README defines the fidelity values as `native_original`,
`external_execution_stream`, `agentsview_cache`, and `metadata_only`
([definition](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/README.md:26)).
These are evidence strata, not quality rankings that can be silently pooled.
Metadata-only sessions contribute to the session frame but not to event or
exchange behavior. AgentsView rows are a transactionally frozen cache subset,
not provider-native transcript bytes. TraeCLI external execution streams are
captured underlying run records, but their schema differs from native Codex and
Claude transcripts.

The frozen AgentsView subset is recorded with `integrity_check: "ok"` and
SHA-256
`e9413d8061683d970ff80d011d5de658561851c31687e1f12c289515fed34047`
([manifest](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:1505)).
That establishes the exact cache snapshot analyzed. It does not establish that
the cache retained every result ever emitted by the providers.

## Exchange status census

### Status by provider

| Provider | Completed | Error | Incomplete | Total |
| --- | ---: | ---: | ---: | ---: |
| Claude | 27,163 | 588 | 208 | 27,959 |
| Codex | 99,671 | 3,132 | 10,461 | 113,264 |
| TraeCLI | 31,015 | 2,843 | 0 | 33,858 |
| **Total** | **157,849** | **6,563** | **10,669** | **175,081** |

The normalizer begins a call as `incomplete`
([implementation](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/control/build_dataset-final.py:677))
and changes it to `error` or `completed` only when it observes a result
([implementation](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/control/build_dataset-final.py:749)).
An error is derived only from an explicit provider error/failure status or a
structured nonzero exit code
([predicate](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/control/build_dataset-final.py:243)).
The README states the same boundary directly
([status semantics](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/README.md:37)).

No `orphan_result` exchange occurs in this corpus. The builder supports that
state when a result has no observed call
([implementation](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/control/build_dataset-final.py:709)),
but the observed count is zero.

### Exhaustive structural partition of explicit errors

The following table accounts for all 6,563 normalized error exchanges without
interpreting intent:

| Provider | Tool surface | Errors |
| --- | --- | ---: |
| Claude | `Bash` | 552 |
| Claude | `bash` | 2 |
| Claude | `Skill` | 6 |
| Claude | `Write` | 6 |
| Claude | `Monitor` | 5 |
| Claude | `Read` | 5 |
| Claude | `Edit` | 4 |
| Claude | `AskUserQuestion` | 3 |
| Claude | `Artifact` | 1 |
| Claude | `ExitPlanMode` | 1 |
| Claude | `TaskStop` | 1 |
| Claude | `WebFetch` | 1 |
| Claude | `bash_placeholder` | 1 |
| Codex | `exec` | 3,131 |
| Codex | `wait` | 1 |
| TraeCLI | `command_execution` | 2,687 |
| TraeCLI | `file_change` | 156 |
| **Total** | | **6,563** |

At a coarser mutually exclusive level, those are:

| Structural class | Derivation | Count |
| --- | --- | ---: |
| Command execution | Codex `exec`; TraeCLI `command_execution`; Claude `Bash` and `bash` | 6,372 |
| Collaboration wait | Codex `wait` | 1 |
| File mutation | TraeCLI `file_change`; Claude `Edit` and `Write` | 166 |
| Specialized non-shell tools | Remaining Claude tools | 24 |
| **Total** | | **6,563** |

This partition is exhaustive because it uses normalized tool identity. It says
where an error was surfaced, not why it happened or whether the failure was
expected.

### Exhaustive TraeCLI file-change modes

All 156 TraeCLI `file_change` errors can be divided by their returned error
text:

| File-change mode | Count |
| --- | ---: |
| Expected lines/context not found | 150 |
| Write failure | 2 |
| Malformed or empty hunk | 1 |
| Missing target | 1 |
| Target is a directory | 1 |
| Target is not valid UTF-8 | 1 |
| **Total** | **156** |

The dominant case is therefore a stale or inaccurate patch precondition, not a
filesystem write failure. The classification is structural: the transcript can
show that expected context was absent, but it cannot always distinguish whether
another writer changed the file, the caller copied old context, or the original
context was wrong.

### Claude's broader error surfaces

Claude's 588 errors include 554 shell-tool errors and 34 errors from non-shell
tools. Inspection of those non-shell cases shows the following modes:

- schema and argument validation, including deferred tool discovery;
- stale `Edit` and `Write` preconditions;
- missing or oversized reads;
- disabled or unavailable skills;
- user or confirmation rejection;
- monitor and task lifecycle races;
- session interruption; and
- policy blocks involving sleep, safety, or worktree boundaries.

Eight Claude failures explicitly record a user or confirmation rejection. That
is a human-decision outcome, not evidence that the tool implementation or
downstream runtime malfunctioned.

## Causal and intent taxonomy

Tool identity, proximate cause, responsibility, and intent answer different
questions. Combining them into one label creates contradictions: an expected
TDD failure is both a runtime error and intended; a stale patch is both a file
mutation error and caller-input failure; a user rejection is a valid policy
outcome even though the provider marks the exchange as an error.

### Axis 1: proximate cause

Use one of these mutually exclusive proximate-cause classes when manually
reviewing an exchange:

1. **Runtime/process failure.** A downstream program, compiler, test, gate,
   search, network client, or shell pipeline returns nonzero or is terminated.
2. **Mutation/precondition failure.** A patch or write cannot be applied because
   its expected state, target, encoding, or target kind does not match reality.
3. **Schema/contract failure.** Tool arguments do not satisfy the client schema,
   a deferred schema was not loaded, or a typed contract rejects the call before
   the intended operation starts.
4. **Policy/authorization outcome.** A permission rule, safety rule, worktree
   boundary, confirmation decision, or explicit user rejection blocks the
   operation.
5. **Tool/resource/data failure.** The tool cannot obtain or represent the
   requested resource: missing or oversized data, unavailable skill/service,
   invalid encoding, or a genuine read/write service failure.
6. **Orchestration/lifecycle failure.** A wait, monitor, task, background job, or
   session loses the expected lifecycle relationship, races, is interrupted, or
   cannot retrieve the expected result.

These classes assign the nearest observed failure boundary. They do not assign
blame. When two boundaries are visible, use the earliest boundary that prevented
the requested operation from satisfying its contract and record downstream
effects separately.

### Axis 2: responsibility or origin

Responsibility should be recorded independently as one of:

- **caller-authored input:** malformed parameters, stale patch context, unsafe
  shell composition, or a wrong target supplied by the agent or user;
- **downstream project/runtime:** the invoked build, test, application, or host
  environment failed;
- **tool/platform:** the tool implementation or provider service failed;
- **human decision:** a user rejected or withheld authority; or
- **undetermined:** the retained evidence does not distinguish the origin.

This is where caller-caused failures belong. They should not be merged with the
schema or runtime axis merely to produce a single count.

### Axis 3: intent

Every reviewed failure should additionally be labeled:

- **expected-negative:** the caller intentionally sought a red test, missing
  search result, rejected invalid value, or other negative observation;
- **unexpected:** the result prevented an operation the caller expected to
  succeed; or
- **undetermined:** the retained call/result does not establish the caller's
  expectation.

The normalized schema does not encode this intent. Expected-negative failures
are therefore an overlay, not a fourth exchange status and not a number that can
be derived safely from exit codes alone.

## Incomplete exchanges and event-level errors

### Incomplete exchanges

The 10,669 incomplete exchanges divide as follows:

| Provider | Fidelity | Tool | Count |
| --- | --- | --- | ---: |
| Claude | `agentsview_cache` | `Read` | 191 |
| Claude | `agentsview_cache` | `ToolSearch` | 13 |
| Claude | `agentsview_cache` | `Glob` | 2 |
| Claude | `native_original` | `Bash` | 2 |
| Codex | `agentsview_cache` | `wait` | 3,804 |
| Codex | `agentsview_cache` | `wait_agent` | 3,772 |
| Codex | `agentsview_cache` | `send_message` | 1,878 |
| Codex | `agentsview_cache` | `spawn_agent` | 519 |
| Codex | `agentsview_cache` | `followup_task` | 488 |
| **Total** | | | **10,669** |

Thus 10,667 of 10,669 incompletes come from AgentsView cache records. Only two
are calls from a native original, both Claude Bash calls. Cache incompleteness
shows that the captured cache lacks a matching retained result; it does not show
that the provider runtime failed, that the operation never ran, or that the
result was unavailable to the original session at the time.

### Event-level error records

Event `kind` is independent of exchange status. The corpus contains:

- 313 `error` events, all in TraeCLI sessions;
- 3 `parse_error` events, one Claude and two Codex;
- 2 `turn_failed` events, both TraeCLI; and
- 1 `unrecognized_source_format` event.

The TraeCLI `error` event class includes long-thread warning records and must not
be added to the 6,563 error exchanges. Parse and source-format errors describe
normalization coverage, not necessarily a failed tool operation.

The 313 TraeCLI `error` events themselves partition exactly as follows:

| Event message class | Count | Interpretation |
| --- | ---: | --- |
| Long-thread and repeated-compaction warning | 204 | Advisory context-quality warning, not a failed turn |
| Ignored `features.plugin_hooks` setting | 48 | Configuration/capability mismatch |
| Ignored `cwd` setting | 16 | Configuration/capability mismatch |
| Repository inaccessible | 28 | Authorization, address, or transport remained unresolved by the event alone |
| Code-mode host unavailable | 4 | Missing executable/capability; code mode failed closed |
| Marketplace DNS failure | 4 | Network/name-resolution failure |
| Marketplace state write rejected by read-only filesystem | 4 | Filesystem policy boundary |
| Marketplace source referenced an invalid worktree | 4 | Stale workspace identity |
| Endpoint fallback notice | 1 | Successful recovery notice recorded with error-kind events |
| **Total** | **313** | |

This table illustrates why event type is not a causal taxonomy. The same
`error` kind contains advisories, configuration drift, runtime failures, and a
successful endpoint fallback.

## Verified primary-record chains

The nine examples below were promoted only after their normalized handles were
matched to exact source locators and their hash-addressed object files were
checked against the source manifest.

### 1. Claude schema discovery and recovery

**Classification:** schema/contract failure; caller-authored input; unexpected
but recovered.

- Session: `claude:ac4636b4-64ee-4f06-974e-45c964e0f8f0`
- Turn: `271de8a8-f810-4313-b360-849865f041ff`
- Source SHA-256:
  `255170066454c82894e1f2ad8b71615f268454d5d64c78daa636b75de39a264f`
- Capture record: [manifest line 434](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:434)

The initial `Monitor` call supplied a discovery-style query rather than the
required `description` and `timeout_ms`. The tool result says `description` is
missing, explains that the schema was not in the discovered-tool set, and
instructs the caller to load it with `ToolSearch`
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/25/255170066454c82894e1f2ad8b71615f268454d5d64c78daa636b75de39a264f.jsonl:120),
[error](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/25/255170066454c82894e1f2ad8b71615f268454d5d64c78daa636b75de39a264f.jsonl:121)).
The next exchange loads `Monitor`
([search call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/25/255170066454c82894e1f2ad8b71615f268454d5d64c78daa636b75de39a264f.jsonl:123),
[search result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/25/255170066454c82894e1f2ad8b71615f268454d5d64c78daa636b75de39a264f.jsonl:124)),
and a corrected `Monitor` starts successfully
([corrected call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/25/255170066454c82894e1f2ad8b71615f268454d5d64c78daa636b75de39a264f.jsonl:132),
[success](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/25/255170066454c82894e1f2ad8b71615f268454d5d64c78daa636b75de39a264f.jsonl:133)).

| Call ID | Event seq | Status | Source lines |
| --- | --- | --- | --- |
| `toolu_01UG4xpZBPRX2KD9NNTzqLnj` | 49 -> 50 | `error` | 120 -> 121 |
| `toolu_01C4SSwc2hoFB88JL31wqzfj` | 51 -> 52 | `completed` | 123 -> 124 |
| `toolu_01F7gVcqjGAmRobSB4SCrfH8` | 54 -> 55 | `completed` | 132 -> 133 |

This is a tool-contract recovery, not a monitor runtime failure: the intended
monitor command did not begin until after the schema was loaded and the call was
corrected.

### 2. TraeCLI stale patch and recovery

**Classification:** mutation/precondition failure; caller-authored input;
unexpected but recovered.

- Session: `traecli:01a0efe3-16a1-7e91-b41a-6c7b02ed8d24`
- Turn: `stream-turn-000001`
- Source SHA-256:
  `0da029e07ffde705832ce4e03b2fc648725e302595c34e5e74197aef2cf2093d`
- Capture record: [manifest line 1248](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:1248)

The `item_178` patch failed because its expected lines were not present
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/0d/0da029e07ffde705832ce4e03b2fc648725e302595c34e5e74197aef2cf2093d.jsonl:273),
[error](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/0d/0da029e07ffde705832ce4e03b2fc648725e302595c34e5e74197aef2cf2093d.jsonl:274)).
The caller then located and read the current code with two command executions
([first inspection](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/0d/0da029e07ffde705832ce4e03b2fc648725e302595c34e5e74197aef2cf2093d.jsonl:275),
[second inspection](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/0d/0da029e07ffde705832ce4e03b2fc648725e302595c34e5e74197aef2cf2093d.jsonl:277)).
A narrower patch then succeeded
([corrected patch](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/0d/0da029e07ffde705832ce4e03b2fc648725e302595c34e5e74197aef2cf2093d.jsonl:280),
[result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/0d/0da029e07ffde705832ce4e03b2fc648725e302595c34e5e74197aef2cf2093d.jsonl:281)).

| Call ID | Event seq | Status | Source lines |
| --- | --- | --- | --- |
| `item_178` | 270 -> 271 | `error` | 273 -> 274 |
| `item_179` | 272 -> 273 | `completed` | 275 -> 276 |
| `item_180` | 274 -> 275 | `completed` | 277 -> 278 |
| `item_182` | 277 -> 278 | `completed` | 280 -> 281 |

The chain supports stale-context recovery. It does not establish why the
caller's first context was stale, because the retained exchange does not prove
whether another writer intervened or the caller began from an inaccurate
snippet.

### 3. Codex expected TDD failure and recovery

**Classification:** runtime/process failure; downstream project behavior;
expected-negative and recovered.

- Session: `codex:01a0ee75-f519-7c80-b3c2-dc39d09e0b24`
- Turn: `01a0f15b-3050-7da2-ad9f-ea8f39198a2d`
- Source SHA-256:
  `80a4e11c63f97ce708624fbfb9bf887b976a860cd8f4c0be18dee1116a17b7e7`
- Capture record: [manifest line 448](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:448)

The first command adds `1.0` to an invalid-enum test and runs the focused suite
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/80/80a4e11c63f97ce708624fbfb9bf887b976a860cd8f4c0be18dee1116a17b7e7.jsonl:3674)).
The test fails because `ValueError` was not raised, and the structured result
records exit code 1
([result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/80/80a4e11c63f97ce708624fbfb9bf887b976a860cd8f4c0be18dee1116a17b7e7.jsonl:3677)).
The next command requires the value to be an actual integer in the accepted
enum, reruns the same focused suite, and gets `OK`
([repair call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/80/80a4e11c63f97ce708624fbfb9bf887b976a860cd8f4c0be18dee1116a17b7e7.jsonl:3679),
[passing result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/80/80a4e11c63f97ce708624fbfb9bf887b976a860cd8f4c0be18dee1116a17b7e7.jsonl:3681)).

| Call ID | Event seq | Status | Source lines |
| --- | --- | --- | --- |
| `call_7bbvkLl2eL0rKbTt7i9cWHRI` | 1621 -> 1622 | `error` | 3674 -> 3677 |
| `call_0AWQONFnFaZcfvYWCUCBa201` | 1623 -> 1624 | `completed` | 3679 -> 3681 |

Counting the first exchange as an unexpected agent failure would invert its
role in the TDD chain. Its normalized `error` status is accurate for process
exit, while its intent label is `expected-negative`.

### 4. Claude deceptive success after a failed gate

**Classification:** caller-authored unsafe shell composition with a downstream
runtime failure; unexpected; normalized as completed.

- Session: `claude:8fe397a4-2b06-4b89-a7b8-ac59af700e49`
- Turn: `0d535c2f-8834-4190-93f8-ac3567a0d5fc`
- Call: `toolu_01HSukjwnRbZ6aJVCPucPGgi`
- Event sequence: 3705 -> 3706
- Source SHA-256:
  `30852404945dbdf8d2d37766e2475106617400c3fbf1dc40f80437bb02e8b5fa`
- Capture record: [manifest line 55](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:55)

The tool call runs a repository gate in a subshell, prints its return code, and
then separates cleanup, push, and MR creation with `;`, so those later actions
do not depend on the gate's success
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/30/30852404945dbdf8d2d37766e2475106617400c3fbf1dc40f80437bb02e8b5fa.jsonl:7769)).
The result begins with `rc=101`, then reports a new remote branch and an open MR
number 16. The provider records `is_error: false`
([result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/30/30852404945dbdf8d2d37766e2475106617400c3fbf1dc40f80437bb02e8b5fa.jsonl:7770)).
The normalized exchange is consequently `completed`.

This is not a normalizer defect under the declared schema. The outer shell/tool
result was reported successful, and the embedded gate failure existed only as
text printed before later successful operations. It is nevertheless a deceptive
semantic success: the exchange's green status cannot support the claim that the
gate passed or that push/MR creation was gated on its success.

The case supports two practical rules:

1. shell commands that guard irreversible or externally visible actions must
   fail closed, for example with `&&`, `set -e`, or explicit return-code checks;
2. check evidence must bind the check result to the exact subject and must not
   infer gate success from the enclosing tool status.

### 5. Claude optimistic-write conflict and recovery

**Classification:** mutation/precondition failure; concurrent state change of
undetermined origin; unexpected but recovered.

- Session: `claude:b8f724d7-2fb0-4433-92ea-cee2a1e34d64`
- Turn: `608046a5-7668-4210-ba43-87dda086bbec`
- Source SHA-256:
  `dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2`
- Capture record: [manifest line 15](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:15)

The first `Write` was rejected because the target had changed since it was
read; the tool explicitly named the user or a linter as possible sources
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/db/dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2.jsonl:874),
[result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/db/dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2.jsonl:875)).
After a shell inspection and a fresh `Read`, the retry succeeded
([inspection](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/db/dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2.jsonl:884),
[reread](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/db/dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2.jsonl:888),
[successful write](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/db/dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2.jsonl:891)).
A second write later met the same guard, then succeeded only after another
reread
([second conflict](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/db/dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2.jsonl:904),
[second reread](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/db/dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2.jsonl:906),
[second success](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/db/dbfa11f5c6e06b174eed21f8999ad4c7578faefe431c69a01b534e9129481ac2.jsonl:915)).

| Call ID | Event seq | Status | Meaning |
| --- | --- | --- | --- |
| `toolu_01U4tgQKATXUtY5xdkBGPbj4` | 386 -> 387 | `error` | First optimistic-write conflict |
| `toolu_01HuaJg3G6UxptUPzVhUpMB5` | 391 -> 392 | `completed` | Fresh read |
| `toolu_01SJ8R6kJBe5fzSEuKB1JUhe` | 393 -> 394 | `completed` | First corrected write |
| `toolu_01EXqFWuJq7xzT7aQDtjgwyi` | 397 -> 398 | `error` | State changed again |
| `toolu_012tdfpFWGKd7rsvw1TKfwU6` | 399 -> 400 | `completed` | Second fresh read |
| `toolu_01UNZkPt3AcAWnPxVb8tdsoa` | 401 -> 402 | `completed` | Second corrected write |

This is a successful fail-closed concurrency guard. The transcript proves
that state changed between read and write; it does not identify which writer
caused either change.

### 6. Claude user rejection is a policy outcome, not a runtime defect

**Classification:** policy/authorization outcome; human decision;
expected behavior for the tool boundary.

- Session: `claude:6c43daee-0c15-4ded-8330-cc9145e2fb12`
- Initial turn: `43d3ee82-db0a-481a-9118-ca0a95037fff`
- Source SHA-256:
  `b3f7049234d2d774729fcca8aace0a7c36cb96f23c55530a05b7b122dad1528f`
- Capture record: [manifest line 267](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:267)

An `AskUserQuestion` call was explicitly rejected; its result instructed the
agent to stop and wait
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/b3/b3f7049234d2d774729fcca8aace0a7c36cb96f23c55530a05b7b122dad1528f.jsonl:881),
[rejection](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/b3/b3f7049234d2d774729fcca8aace0a7c36cb96f23c55530a05b7b122dad1528f.jsonl:882)).
A later user turn supplied new direction, and a reformulated question completed
normally
([new call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/b3/b3f7049234d2d774729fcca8aace0a7c36cb96f23c55530a05b7b122dad1528f.jsonl:887),
[answer](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/b3/b3f7049234d2d774729fcca8aace0a7c36cb96f23c55530a05b7b122dad1528f.jsonl:888)).
The normalized `error` status accurately preserves the rejected call, but a
failure-rate analysis must not count it as a tool malfunction.

### 7. Claude oversized read and bounded recovery

**Classification:** tool/resource/data failure; caller requested too much
retained content; unexpected but recovered.

- Session: `claude:agent-ab4ab3f47156f8b8f`
- Turn: `98d792ad-3f32-44e1-9fad-b03ecf7b86dd`
- Source SHA-256:
  `9d9f5719f136c2a22488489c43376f9edb093f72f970ff43e67f8684d33dbd12`
- Capture record: [manifest line 397](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:397)

The initial `Read` requested a file whose 28,549-token content exceeded the
25,000-token limit. The result advised offset/limit reads or search instead
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/9d/9d9f5719f136c2a22488489c43376f9edb093f72f970ff43e67f8684d33dbd12.jsonl:22),
[limit result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/9d/9d9f5719f136c2a22488489c43376f9edb093f72f970ff43e67f8684d33dbd12.jsonl:23)).
The next reads completed, including a later bounded `limit: 700` request
([first successful read](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/9d/9d9f5719f136c2a22488489c43376f9edb093f72f970ff43e67f8684d33dbd12.jsonl:24),
[bounded call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/9d/9d9f5719f136c2a22488489c43376f9edb093f72f970ff43e67f8684d33dbd12.jsonl:29),
[bounded result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/9d/9d9f5719f136c2a22488489c43376f9edb093f72f970ff43e67f8684d33dbd12.jsonl:30)).
This is a recoverable representation limit, not evidence that the source file
was missing or corrupt.

### 8. Claude stale task stop and lifecycle correction

**Classification:** orchestration/lifecycle failure; caller targeted stale job
state; unexpected but bounded.

- Session: `claude:cb2d7426-0526-4c44-8c97-30a943533be7`
- Initial turn: `759545eb-1d14-412a-84d1-b53fd95ea73d`
- Source SHA-256:
  `384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18`
- Capture record: [manifest line 5](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:5)

`TaskStop` targeted task `a69c3d77437aead0a`, but the task had already
completed, so there was nothing left to stop
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/38/384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18.jsonl:7155),
[result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/38/384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18.jsonl:7156)).
A following status command found another run still active
([status call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/38/384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18.jsonl:7160),
[status result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/38/384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18.jsonl:7161));
the next turn stopped that live task successfully
([corrected stop](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/38/384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18.jsonl:7188),
[success](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/38/384bb0227f7964d51a13bd6ec85cac7fe1cce01a96deb8618325595d1164ac18.jsonl:7189)).
The failure is about stale lifecycle identity, not an inability to stop running
tasks generally.

### 9. Codex test failure caused by host thread exhaustion

**Classification:** runtime/process failure; downstream host resource limit;
unexpected and not established as recovered in the immediate exchange chain.

- Session: `codex:01a10aa8-edc1-72c2-a76f-1bd586caf136`
- Turn: `01a11780-0394-78f0-873e-5a685c04d1dd`
- Call: `call_fnYINjh34cXMMxu7vOKloCTO`
- Event sequence: 2174 -> 2175
- Source SHA-256:
  `880e267f7f08c9b5a4c0e2567456dfa5312fa30cbd1af982b8f167456374d622`
- Capture record: [manifest line 802](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/source-manifest.jsonl:802)

The command's outer test result was exit 101, but the retained failure body
identifies the proximate host error: spawning a thread returned OS code 11,
`WouldBlock`, `Resource temporarily unavailable`
([call](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/88/880e267f7f08c9b5a4c0e2567456dfa5312fa30cbd1af982b8f167456374d622.jsonl:5084),
[result](/data02/home/philip.yang/.local/share/session-audit/2026-10-14/transcript-corpus-v1/originals/sha256/88/880e267f7f08c9b5a4c0e2567456dfa5312fa30cbd1af982b8f167456374d622.jsonl:5087)).
The transcript therefore supports a resource-exhaustion classification rather
than a product assertion failure. It does not establish that the host limit was
later repaired; subsequent exchanges continued other active work. This case
shows why the terminal exit code alone is insufficient even when `error` is
the correct structural status.

## Reproducible queries

Run these against
`normalized/normalized.sqlite3` in the October 14 corpus.

### Overall status counts

```sql
SELECT status, COUNT(*) AS exchanges
FROM tool_exchanges
GROUP BY status
ORDER BY status;
```

### Provider status matrix

```sql
SELECT s.provider, x.status, COUNT(*) AS exchanges
FROM tool_exchanges AS x
JOIN sessions AS s USING (session_key)
GROUP BY s.provider, x.status
ORDER BY s.provider, x.status;
```

### Exhaustive error surfaces

```sql
SELECT s.provider, x.tool_name, COUNT(*) AS errors
FROM tool_exchanges AS x
JOIN sessions AS s USING (session_key)
WHERE x.status = 'error'
GROUP BY s.provider, x.tool_name
ORDER BY s.provider, errors DESC, x.tool_name;
```

### Fidelity coverage

```sql
SELECT
    provider,
    source_fidelity,
    COUNT(*) AS sessions,
    SUM(event_count) AS events,
    SUM(turn_count) AS turns,
    SUM(tool_exchange_count) AS exchanges
FROM sessions
GROUP BY provider, source_fidelity
ORDER BY provider, source_fidelity;
```

### Incomplete-exchange provenance

```sql
SELECT
    s.provider,
    s.source_fidelity,
    x.tool_name,
    COUNT(*) AS exchanges
FROM tool_exchanges AS x
JOIN sessions AS s USING (session_key)
WHERE x.status = 'incomplete'
GROUP BY s.provider, s.source_fidelity, x.tool_name
ORDER BY s.provider, s.source_fidelity, exchanges DESC;
```

### Exact evidence handles for a promoted call

```sql
SELECT
    x.session_key,
    x.call_id,
    x.turn_id,
    x.call_event_seq,
    x.result_event_seq,
    x.tool_name,
    x.status,
    s.source_fidelity,
    s.source_sha256,
    json_extract(x.row_json, '$.call_source_locator') AS call_locator,
    json_extract(x.row_json, '$.result_source_locator') AS result_locator
FROM tool_exchanges AS x
JOIN sessions AS s USING (session_key)
WHERE x.session_key = 'claude:8fe397a4-2b06-4b89-a7b8-ac59af700e49'
  AND x.call_id = 'toolu_01HSukjwnRbZ6aJVCPucPGgi';
```

### Event-level error-like records

```sql
SELECT s.provider, e.kind, COUNT(*) AS events
FROM events AS e
JOIN sessions AS s USING (session_key)
WHERE e.kind IN (
    'error',
    'parse_error',
    'turn_failed',
    'unrecognized_source_format'
)
GROUP BY s.provider, e.kind
ORDER BY s.provider, e.kind;
```

## Explicit limits

1. **This is a structural census, not 175,081 semantic reviews.** Provider,
   tool, status, fidelity and retained-result counts are exhaustive. Causal and
   intent examples are manually verified, but no claim is made that every
   command result was semantically classified.
2. **`error` is not synonymous with unexpected failure.** It includes expected
   red tests, negative validation and search misses whenever the provider emits
   explicit error state or structured nonzero exit.
3. **`completed` is not synonymous with end-to-end success.** Compound commands
   can contain an earlier failed operation and still finish with a successful
   provider result, as the gate/push/MR chain proves.
4. **`incomplete` is not synonymous with runtime failure.** It means a call has
   no retained matching result. Almost every incomplete exchange is from the
   frozen AgentsView cache stratum.
5. **Fidelity strata are not interchangeable.** Native originals,
   external execution streams, cache rows and metadata-only records preserve
   different evidence. Cache absence or incompleteness cannot be promoted to a
   claim about the original runtime.
6. **Sessions are not independent tasks.** Parent/child agents, replays,
   retries and long-running threads can share context or objective. The counts
   are exchange and session counts, not behavioral prevalence across
   independent engineering tasks.
7. **Provider comparisons would be confounded.** Providers have different tool
   surfaces, source fidelity, workloads, capture mechanisms and missingness.
   This report does not rank providers or estimate comparative failure rates.
8. **Event errors and exchange errors are separate.** Event warnings, parse
   errors and failed-turn markers must not be summed with tool-exchange errors.
9. **The promoted chains prove only their stated boundaries.** The Claude
   schema and TraeCLI patch chains prove recovery, not the upstream reason for
   the initial bad input. The Codex chain proves an expected TDD red/green
   sequence, not that all later work was correct. The deceptive-success chain
   proves a failed gate followed by external actions, not the root cause of gate
   return code 101. The write-conflict, policy, read-limit and task-lifecycle
   chains prove their local failure and recovery boundaries, not broader task
   correctness. The resource-exhaustion chain proves the observed OS failure,
   not subsequent host recovery.
10. **Sensitive and irrelevant data was excluded.** The research retained call
    identities, statuses, source hashes and minimal result facts. It does not
    reproduce private reasoning, credentials, or unrelated host process
    listings.

## Bottom line

The corpus supports a precise failure vocabulary only when three layers stay
separate:

1. the exhaustive structural status and tool-surface census;
2. a manual proximate-cause classification; and
3. independent responsibility and intent labels.

That separation accounts for every normalized exchange without turning cache
gaps into runtime failures, expected red tests into agent defects, or green
compound shell calls into proof that every embedded gate passed. The verified
examples also show the recovery behavior that a status-only audit would miss.
