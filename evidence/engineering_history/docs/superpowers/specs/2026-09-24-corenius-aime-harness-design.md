# Corenius AIME Harness Improvement Design

## Goal

Improve Corenius' AIME evaluator using the confirmed DeepSeek Harness and DS4
eval patterns as references, while keeping Corenius focused on AIME rather than
turning it into a general benchmark platform.

The first implementation slice should make evaluator changes safer to audit:
old model outputs can be regraded without rerunning providers, provider failure
behavior can be tested through scripted fake ACP runs, and run artifacts explain
execution behavior well enough to compare sanity and full AIME runs.

## Source Evidence

DeepSeek Harness benchmark guidance establishes the strongest external model
for harness structure:

- benchmarks are organized by measured user path, not package tree;
- fixed synthetic inputs are preferred over user material, ambient repositories,
  or network services;
- process-level samples run in fresh child processes with private temporary
  roots, explicit timeouts, and cleanup;
- reports include enough raw and aggregate data to explain each verdict; and
- benchmarks exercise production entry points instead of copying product
  algorithms into benchmark-only paths.

DeepSeek's LLM mock server adds a concrete testing pattern: FIFO scripted
provider behavior, request validation before advancing the script cursor,
scripted failures such as resets, stalls, empty completions, malformed chunks,
429/500/503 errors, and seeded random behavior for reproducible stress runs.

DeepSeek's LLM replay adapter adds a stricter replay pattern: fixtures are
validated and migrated rather than repaired, live sessions bind to recorded
scripts, cursor consumption is asserted, and unreconstructable failures use an
explicit override sidecar.

DS4's `ds4-eval` establishes a useful local real-model eval pattern:

- it is explicitly a regression harness, not a leaderboard runner;
- it uses an audited embedded subset chosen for local inspectability;
- it has a `--regrade-trace` path that reruns current extraction and scoring
  against previous model output without loading the model; and
- it defines a small deterministic smoke gate before broader evaluation.

No standalone Pi harness checkout was found in the local workspace. The
available Pi evidence is tooling-level: Pi core does not provide a standard
subagent or task-list tool, and DS4 documents Pi as a provider-host target via
`~/.pi/agent/models.json`. This design therefore treats Pi as a provider
compatibility target to revisit when the actual Pi harness source is available.

## Current Corenius Shape

The current AIME hardening branch already has the right central module to build
on:

- `src/cli.rs` exposes `eval aime`, `eval aime-sanity`, and `eval aime-replay`.
- `src/eval.rs` owns AIME case loading, prompt construction, provider
  execution, answer extraction, failure classification, manifest writing,
  summary writing, triage rendering, and replay selection.
- `AimeSample` is the durable per-case record, with prompt, response, status,
  elapsed time, usage, score, and `failure_kind`.
- `manifest.json`, `samples.jsonl`, `summary.json`, and `triage.md` are the
  durable artifact set for each run.

The next changes should deepen `src/eval.rs` around a small set of evaluator
interfaces rather than adding a new generic benchmark layer.

## Design

### Regrade Runs

Add `corenius eval aime-regrade --from-run <dir>`.

Regrade reads `<from-run>/samples.jsonl`, reruns the current extraction,
normalization, scoring, and failure classification against the stored responses,
and writes a new independent artifact directory:

```text
.scratch/aime-benchmark/regrade-<UTC timestamp>/
  manifest.json
  samples.jsonl
  summary.json
  triage.md
  scorer-delta.json
```

The original run is never mutated. Regrade samples retain the original prompt,
response, expected answer, elapsed time, return status, provider error, and
usage fields. They update only the derived evaluator fields:

- `extracted_answer`
- `passed`
- `score`
- `failure_kind`

`scorer-delta.json` records one row per changed sample:

- `id`
- `old_extracted_answer`
- `new_extracted_answer`
- `old_passed`
- `new_passed`
- `old_failure_kind`
- `new_failure_kind`

An empty delta is valid and should be explicit.

### Scripted ACP Fault Harness

Keep real AIME execution on the existing TraeCLI ACP provider path, but add a
test-only scripted ACP executable used by CLI tests. It should emulate the ACP
JSON-RPC flow enough to exercise Corenius' production provider adapter, not a
shortcut in `src/eval.rs`.

The script format should be FIFO and validation-first:

- a scripted response is consumed only after the request shape is accepted;
- exhaustion is a loud test failure;
- every started script must be fully consumed unless the test explicitly opts
  out for an early-failure scenario.

Required scripted behaviors for the first slice:

- success with `FINAL: <integer>`;
- ordinary wrong final;
- contaminated wrong response;
- no final response;
- provider error;
- timeout or stall;
- malformed provider response if the ACP test fixture can express it cleanly.

This should replace ad hoc fake-provider cases in tests over time, but only
where it improves coverage of the real provider path.

### Run Metrics

Add `run_metrics.json` beside the existing artifacts.

This file is not a scoring source of truth. It explains runtime behavior:

- `schema_version: 1`
- `sample_count`
- `execution_mode`: `shared_provider` or `fresh_provider_per_sample`
- `timeout_ms`
- `samples`: ordered rows with `id`, `elapsed_seconds`, `passed`,
  `failure_kind`, `return_status`, `had_error`, and optional `usage`
- `aggregate`: total elapsed seconds, min/p50/p95/max elapsed seconds,
  timeout count, provider error count, no-final count, wrong-final count, and
  prompt-contamination count

Keep the ordering identical to `samples.jsonl` so operators can compare files
without re-sorting.

### Optional Fresh Provider Per Sample

Add `--fresh-provider-per-sample` to `eval aime` and `eval aime-sanity`.

The default remains the current provider behavior for cost and compatibility.
When enabled, each sample uses an isolated provider child. This borrows DSH's
fresh-child isolation without making every full AIME run pay the startup cost.

The manifest and `run_metrics.json` must record the execution mode. If a later
Pi harness integration needs per-sample process isolation to avoid provider
state leakage, it should use this mode rather than a separate evaluator path.

### AIME Sanity And Full Gates

Use two validation levels:

- `eval aime-sanity` is the cheap deterministic gate for scorer, artifact, and
  provider-path changes.
- `eval aime --years 2024 2025-I 2025-II` remains the explicit full gate for
  expensive model behavior checks.

Regrade should be runnable against either sanity or full artifacts. A scorer
change is not considered ready until regrade shows the expected delta against a
known artifact, even if fresh provider execution passes.

## Module Boundaries

Keep the public CLI small and keep the evaluator AIME-specific.

The internal shape should introduce narrow helper modules or private types only
when they deepen the current evaluator:

- `AimeRunCase`: immutable prompt/expected-answer input to one sample attempt.
- `AimeSample`: durable raw plus derived result for one sample.
- `AimeScorer`: private functions for extraction, normalization, pass/fail, and
  failure classification.
- `AimeArtifactWriter`: private helpers for manifest, sample JSONL, summary,
  triage, metrics, and scorer deltas.
- `AimeReplayLoader`: private helpers for loading existing `samples.jsonl`,
  selecting failed samples, and regrade/replay input validation.

The interface should stay small: callers choose a command, source, selection,
run directory, model, timeout, stream flag, and execution mode. The evaluator
owns everything below that interface.

## Error Handling

Provider failures remain failed samples unless the evaluator cannot construct or
persist artifacts. Dataset, manifest, or artifact write failures abort the run
because they make the output untrustworthy.

Regrade fails clearly when:

- `<from-run>/samples.jsonl` is missing;
- a line cannot deserialize as `AimeSample`;
- the regrade selection is empty; or
- the destination run directory cannot be created or written.

Scripted ACP tests fail clearly when:

- a request does not match the expected ACP shape;
- the script is exhausted;
- a script entry remains unconsumed; or
- a timeout/stall test does not exercise the intended timeout path.

## Validation Plan

Unit tests in `src/eval.rs`:

- regrade recomputes extracted answer, pass/fail, score, and failure kind;
- regrade delta records changed and unchanged samples correctly;
- metrics aggregation preserves sample order and computes min/p50/p95/max;
- fresh-provider execution mode is serialized into manifest and metrics;
- failure counts stay consistent between triage and metrics.

CLI tests in `tests/cli_tui.rs`:

- `eval aime-regrade --from-run <dir>` writes independent artifacts and does
  not mutate the original run;
- regrade fails when `samples.jsonl` is missing;
- scripted ACP success, wrong final, contamination, no-final, provider error,
  and timeout all flow through the production eval command;
- `eval aime-sanity --fresh-provider-per-sample` records the isolation mode;
- `eval aime --years 2025-I --limit 1` still writes the original artifact set
  plus `run_metrics.json`.

Verification commands:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Manual smoke:

```sh
cargo run -- eval aime-sanity --workspace . --filter-id 2025-I:I-1 --run-dir .scratch/aime-sanity/design-smoke
cargo run -- eval aime-regrade --from-run .scratch/aime-sanity/design-smoke
```

If a real provider is available, run one bounded full-path sample:

```sh
TRAECLI_PATH=/data02/home/philip.yang/.local/bin/traecli \
cargo run -- eval aime --workspace /data02/home/philip.yang/workspace/corenius \
  --model gpt-5.2 --years 2025-I --limit 1 --timeout-ms 120000 --stream
```

## Deferred Work

- A generic benchmark framework is out of scope.
- Best-of replay summaries are out of scope; replay and regrade artifacts stay
  independent.
- Parallel AIME execution is out of scope until provider process isolation and
  rate-limit behavior are better characterized.
- Pi-specific harness integration is deferred until the actual Pi harness source
  is available.
- Response-quality warnings for contaminated but correct answers are deferred;
  this design keeps `passed` and `score` as the scoring source of truth.
