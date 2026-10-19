# Corenius Observability And Durable Execution

This note records the next design direction for Corenius after the Rust tracer
bullet: make every agent run replayable, inspectable, and benchmarkable without
depending on live provider state.

## Sources Reviewed

- DeepSeek Harness at commit `00102833dfaee1da9f48a3a8eae9d34005a75218`.
- Temporal at commit `543367eec690da3c23e536cf5dfe76928af11b2c`.

## Design Lessons

DeepSeek Harness keeps benchmark ownership aligned with user-visible paths
rather than package layout. Its benchmark guidance requires one directory per
measured path, fixed synthetic inputs, no recorded sessions or network services,
fresh child processes with private temp roots, bounded children, and raw plus
aggregate measurements that explain the verdict
(`benchmarks/AGENTS.md:5-16`).

DeepSeek's benchmark worker launcher is a useful shape for Corenius benchmark
commands: run the compiled artifact in a fresh child, strip ambient runtime
flags, bound the child by timeout, capture `exitCode`, `signal`, `timedOut`,
and `stderr`, and accept exactly one JSON report from stdout
(`benchmarks/support/built-worker.ts:28-97`).

DeepSeek also separates replay performance from raw stream volume. The
conversation-fold benchmark keeps event and compact-record counts fixed while
varying streamed delta count, then gates the compact fold against
delta-proportional replay (`benchmarks/conversation-fold/conversation-fold.bench.client.ts:26-40`).
The active-stream reconnect benchmark reconstructs a 100,000-delta live prefix,
measures replacement, verifies the next frame is still accepted, and reports
record counts, entry counts, latency, and retained memory
(`benchmarks/active-stream-reconnect/reconnect.worker.client.ts:10-49`).

DeepSeek's session-open benchmark gives Corenius a migration pattern: open an
old fixture, materialize an upgraded successor, reopen the successor, and assert
the original fixture bytes are unchanged
(`benchmarks/session-open/session-open.bench.ts:326-364`).

Temporal's server exposes workflow inspection as a first-class operational path:
its README points users to `temporal workflow list` and the Web UI for viewing
workflow execution (`README.md:49-60`). Its code models workflow and activity
lifecycle events explicitly, including workflow start/finish, workflow task
scheduled/started/completed/failed, activity scheduled/started/completed/failed,
timers, and retry events (`common/log/tag/values.go:5-80`).

Temporal's logging tags make correlation keys explicit: workflow id, run id,
event id, scheduled event id, started event id, history size, history byte size,
and event count are all named dimensions (`common/log/tag/tags.go:118-227`,
`common/log/tag/tags.go:349-366`). Temporal also has replay-aware logging that
can suppress duplicate logs while replaying history (`common/log/replay_logger.go:20-79`).

## Corenius Runtime Shape

Corenius should treat a run as an event-sourced workflow:

- The replayable layer decides what to do next from persisted events only.
- Effects are activities: provider calls, tool calls, shell execution, file
  reads, approval decisions, terminal input, and terminal output.
- Every activity has a stable idempotency key:
  `run_id + turn_id + step_id + activity_id + input_hash`.
- During resume, if a completed activity event exists for the idempotency key,
  Corenius reuses the recorded result instead of re-running the effect.
- During replay, Corenius suppresses duplicate human-facing logs while still
  allowing trace validation logs, following the Temporal replay-logger pattern.

## Trace Schema Direction

Keep `RuntimeEvent` for the user-facing TUI projection, but add a lower-level
append-only trace stream for execution forensics:

```text
trace_header
run_started
turn_started
provider_request_scheduled
provider_stream_chunk
provider_request_completed
tool_call_scheduled
tool_call_started
approval_requested
approval_decided
tool_call_completed
sandbox_command_built
sandbox_process_started
sandbox_process_completed
checkpoint_written
retry_scheduled
turn_completed
turn_failed
run_completed
```

Each trace event should include:

- `schema_version`
- `sequence`
- `timestamp`
- `run_id`
- `turn_id` when present
- `step_id` when present
- `activity_id` when present
- `idempotency_key` for effects
- `provider`, `model`, and provider response id when present
- `tool_name`, `tool_call_id`, and normalized capability flags when present
- `attempt`, `retryable`, `backoff_ms`, and `error_class` for retries
- `payload_ref` for large model/tool/terminal data

## Payload Storage

Trace JSONL should stay compact. Store large payloads by content-addressed
reference under `.corenius/blobs/`:

- streamed provider deltas after compaction thresholds,
- long stdout/stderr,
- terminal frame batches,
- screenshots or future binary artifacts,
- full provider raw JSON when debug capture is enabled.

The trace row should keep hashes, byte counts, MIME-like kind, truncation state,
and a short preview. This keeps replay and search proportional to compact
records rather than raw token or terminal-frame volume.

## Benchmark Plan

Add a `corenius bench` command with one subdirectory per measured user path:

- `cold-open-trace`: load a trace and hydrate current projections.
- `resume-agent`: restart from a checkpoint and compute the next scheduled
  activity without re-running completed effects.
- `active-stream-reconnect`: rebuild an in-flight assistant stream and accept
  the next chunk.
- `replay-long-terminal`: fold large terminal output into the TUI projection.
- `tool-retry-after-crash`: crash after scheduling a tool, resume, and verify
  idempotent retry behavior.
- `trace-schema-migration`: read an old fixture, write an upgraded successor,
  replay it, and byte-compare the original fixture unchanged.

Each sample should run as a fresh compiled `corenius` child process with a
private temp root and fixed synthetic inputs. The child should emit one final
JSON object:

```json
{
  "benchmark": "resume-agent",
  "workload": "synthetic-200-turns",
  "schema_version": 1,
  "timing": {"elapsed_ms": 0},
  "memory": {"retained_bytes": 0},
  "event_counts": {"trace": 0, "runtime": 0},
  "blob_bytes": 0,
  "exit": 0,
  "signal": null,
  "timed_out": false
}
```

## Immediate Implementation Slices

1. Add a `TraceStore` beside `TranscriptStore` that appends typed trace events
   and can replay them into a deterministic execution state.
2. Introduce explicit activity records around `ProviderSession::stream_turn`
   and `ToolRegistryLike::execute`.
3. Add `payload_ref` support for large provider deltas and sandbox outputs.
4. Add crash/resume tests with a fake provider and fake tools to prove completed
   activities are reused.
5. Add the first `corenius bench cold-open-trace` command using synthetic traces
   and one-JSON-report child-process output.
