# NDTL reference (experimental distributed-training observation repo)

- Repo: `/data02/home/philip.yang/workspace/ndtl` (remote `git@code.byted.org:philip.yang/ndtl.git`)
- Commit read: `8bc8ba71198b3900081a744dd887cbdde78de4a5` ("chore: ignore local worktrees"), branch `main`
- Also read, for design intent only (NOT the primary commit): local worktree branch
  `codex/ndtl-reconciliation-design` at `5880bfd` (2026-09-29), checked out at
  `ndtl/.worktrees/ndtl-reconciliation-design`. It is not on `origin`.
- Date of this document: 2026-09-29
- Read-only pass. No tests, builds, cargo, or GPU jobs were run. Every "tests pass" statement below is a
  claim from repo docs or ledgers, not something verified here.
- Working tree at read time: tracked `__pycache__/*.pyc` files modified, untracked `.superpowers/` and extra
  `.pyc` files (`git status`). No source edits pending.

Evidence notation. Paths are relative to the ndtl repo unless prefixed. `file:line` is from the commit above.
Each non-obvious claim carries one of these labels:

- **[READ]** stated directly by code or docs at the cited location.
- **[INFERRED]** my reasoning from code that I read; not documented and not executed.
- **[DOC]** a claim made by repo docs that I did not or could not verify.
- **[BRANCH]** true only on `codex/ndtl-reconciliation-design`, not on `main`.

Line-citation caveat: `src/ndtl/torchtitan_harness.py`, `tools/synd/src/*.rs` citations were computed from
per-file offsets; spot-check with the verification list at the end.

---

## 0. One-paragraph summary

NDTL is a small Python package plus a Rust sidecar (`synd`) for observing one distributed-training run
without modifying the trainer. Ranks append typed JSON lines (manifest, heartbeat, step lifecycle,
operation lifecycle, failure, collector health, backend-timing quality) to per-rank files. Post-hoc
analyzers and the live `synd` TUI/reporter read those files and answer: is every rank alive, which rank is
slow, which operation is stuck, did the collector itself lose data, and how trustworthy is each timing.
It is experimental (v0.1.0, `pyproject.toml:7`), owned by one author, and currently integrates with exactly
one trainer (TorchTitan) through a logging-handler seam. A large unpublished redesign (v3 fact ledger,
semantic derivation, Arnold/Chrome projections) exists on a local branch and is the stated future direction.

---

## 1. Problem, audience, and the pain without it

### 1.1 The problem

Large NCCL/ND-parallel training jobs fail in three characteristic ways: a rank dies, a rank is slow
(straggler), or a collective hangs. The existing production instrument, `bytedance.ndtimeline`
(repo `distributed_timeline`), emits aggregated timer rows that end up in ClickHouse and are read by
Arnold's Megavision trace generator. NDTL exists to try a different model: raw, append-only, per-rank
records where liveness, progress, operation lifecycle, collector health, and measurement quality are
first-class facts rather than derived columns.

Evidence for the stated purpose:
- README opening: "small Python package for writing distributed-training timing records to JSONL"
  (`README.md:3-4`). [READ]
- The reconciliation design states the purpose as making NDTL "the owner of a unified distributed-training
  observation model" that stays compatible with NDTL records, `bytedance.ndtimeline` data, and Arnold's
  consumers (`docs/superpowers/specs/2026-09-28-ndtl-source-reconciliation-design.md`, "Purpose", on the
  branch commit `982e1c7`). [READ, BRANCH]
- The sibling observatory repo calls it "the experimental NDTL authority: append-only JSONL records, v2
  liveness and operation schemas, collector health, evidence quality, and the `synd` sidecar harness"
  (`robust-training-observatory/CONTEXT.md:71`). [READ]

### 1.2 Who it is for

- The primary user is a modeling or ML-systems researcher running TorchTitan on the local B200/H-class
  node: "The target user is an MTS working on modeling and ML-systems research"
  (`docs/synd-live-harness-spec.md:11`). The three questions the dashboard must answer are how the model
  is training, which rank maps to which DP/TP shard and GPU, and what low-level events explain the current
  state (`docs/synd-live-harness-spec.md:12-16`). [READ]
- Secondary audience: agents and engineers who need a machine-readable run artifact (`summary.json`,
  `steps.txt`, raw JSONL) they can gate on. [INFERRED from the report contract, `docs/synd.md:259-283`]

### 1.3 What goes wrong without it (concrete pain)

Each item is what the repo's checks or docs are built to catch:

1. **Silent hang.** A rank blocks inside a collective; the launcher process is alive, stdout is quiet.
   NDTL records a `started` `OperationEvent` with no terminal event and a heartbeat pointing at that
   operation (`docs/schema.md:124-126`, `src/ndtl/analysis.py:44-82`). Without it there is only "the job
   looks stuck". [READ]
2. **Straggler.** One rank's median completed-step duration exceeds the fastest rank's median by a
   ratio (default 2.0) (`src/ndtl/analysis.py:7-28`). [READ]
3. **Failed rank without the original exception being swallowed.** `mark_failure` records exception type,
   a 16-hex SHA-256 prefix, and a message truncated to 240 chars and re-raises
   (`src/ndtl/observatory.py:441-476`, `:301-302`). [READ]
4. **The observer lying about itself.** Bounded queue drops and write failures are counted and reported as
   `CollectorHealth` (`src/ndtl/observatory.py:59-145`, `:647-666`), and the analyzer fails the run on
   `write_failures > 0` (`src/ndtl/analyze.py:36`). [READ]
5. **Timing that pretends to be device timing.** Host submit time is recorded separately and quality is a
   three-value vocabulary (`complete`, `correlated`, `unsupported`) so a reader cannot mistake host latency
   for NCCL stream duration (`docs/schema.md:93-100`). [READ] See 4.6 for where the implementation
   weakens this.
6. **Trainer metrics only in stdout.** TorchTitan prints loss/TPS/MFU; NDTL promotes them into
   `StepProgress` fields so they are durable and joinable (`src/ndtl/schema.py:98-109`,
   `src/ndtl/torchtitan_bridge.py:338-389`). [READ]
7. **No live view of a multi-GPU smoke.** `synd` tails spools, samples `nvidia-smi`, `/proc/net/dev`,
   RDMA sysfs counters, and optionally runs `py-spy dump`, and renders it (`tools/synd/src/main.rs`,
   `gpu.rs`, `network.rs`, `profile.rs`). [READ]

---

## 2. Non-goals, boundaries, authority, dependents

### 2.1 Stated non-goals

For `synd` (`docs/synd.md:23-32`): no per-step rootfs relaunch; no Rust/Ratatui/non-stdlib dependency in the
Python package; no TorchTitan source mutation; no TTY required for correctness. [READ]

For the v3 redesign (`...reconciliation-design.md`, "Non-Goals"): do not vendor
`distributed_timeline`'s PyTorch/FSDP patching runtime; do not replace Arnold's production ClickHouse
reader in the first delivery; do not claim exact NCCL transport attribution from Torch-level facts; do
not treat wall-clock proximity as collective identity; no byte-for-byte legacy round trip; observer
failure does not fail training by default. [READ, BRANCH]

Non-goals implied by the code [INFERRED]:
- NDTL does not patch PyTorch or NCCL. It wraps user-supplied context managers
  (`src/ndtl/observatory.py:365-418`) or listens to TorchTitan's structured logger.
- It does not record tensor payloads. Only counts and named metadata are recorded
  (`observatory.py:715-730`).
- It is not a transport. There is no MQ, ClickHouse, or network sink. JSONL on a shared filesystem is the
  only egress.
- It is not a job scheduler or controller. `synd` launches one TorchTitan job and reports; it does not
  restart, reschedule, or remediate.

### 2.2 What NDTL treats as authority

- **JSONL records are the durable source of truth** (`docs/synd.md:20`). The live aggregator is a derived
  view. [READ]
- **Trainer-native step metrics are authoritative over stdout parsing** (`docs/synd-live-harness-spec.md:159-167`).
  See 4.7 for an enforcement gap. [READ]
- **TorchTitan synchronous semantics are authoritative for progress.** Progress is the all-rank minimum
  completed step (`tools/synd/src/aggregate.rs:504-511`; `docs/synd-live-harness-spec.md:113-133`). [READ]
- **Torchetta's bwrap rootfs is the authority for CUDA certification.** `ndtl-verify cuda-collective
  --require-torchetta-rootfs` refuses to pass outside it (`src/ndtl/verify.py:385-404`). [READ]
- On the branch, **the fact ledger is the authority; the semantic model and all projections are rebuildable
  and lossy** (`docs/legacy-crosswalk.md`, "Canonical facts remain the source of truth"). [READ, BRANCH]

### 2.3 What depends on it

- `synd` (in-repo Rust crate) depends on the Python record shapes, mirrored by hand in
  `tools/synd/src/records.rs:14-42` (`StepProgressRecord`). There is no shared schema file; drift is
  caught only by a Rust test named `rust_step_progress_record_mirrors_trainer_native_schema`
  (`tools/synd/tests/synd_contract.rs`, grep for the name). [READ]
- Outside the repo: `git grep -w ndtl` in tracked files of `torchtitan`, `torchetta`, `milano`, `vaso`,
  `arnold`, `distributed_timeline`, and `corenius` returned nothing. In `ultron` it matches only
  `CONTEXT.md:17` and `README.md:4` ("followed systems"). `robust-training-observatory` cites NDTL
  extensively. So, today, **nothing in the workspace imports or executes NDTL except NDTL's own scripts
  and the observatory's documentation.** [READ, from my grep]

---

## 3. Architecture

### 3.1 Component map (main, commit 8bc8ba7)

```text
                       trainer process (per rank)                       observer (separate process)
  ┌──────────────────────────────────────────────────────────┐      ┌────────────────────────────────┐
  │ user code:  with obs.step()/work()/collective()/p2p()    │      │ synd (Rust, tools/synd)        │
  │      or                                                  │      │  launcher thread  ─ child proc │
  │ TorchTitan structured logger ──▶ TorchTitanNdtlHandler   │      │  spool tailer     ─┐           │
  │                                       │                  │      │  stdout tailer    ─┤ events    │
  │                                Observatory               │      │  gpu monitor      ─┤ (4096)    │
  │  heartbeat thread (5s) ─┐             │                  │      │  network monitor  ─┤           │
  │                         ▼             ▼                  │      │  aggregator ◀──────┘           │
  │            bounded queue (65536, drop-on-full)           │      │     │ snapshots (32)           │
  │                         │  writer thread                 │      │     ├──▶ Ratatui UI / progress │
  │                         ▼                                │      │     └──▶ summary.json, steps.txt│
  │  <root>/.ndtl/runs/<run_id>/<run_id>.ndtl.jsonl          │─────▶│ tails .ndtl/runs/*/*.ndtl.jsonl │
  │  <root>/titan_metrics/rank-N.jsonl  (metric bridge)      │─────▶│ and titan_metrics/*.jsonl       │
  └──────────────────────────────────────────────────────────┘      └────────────────────────────────┘
        offline: ndtl-analyze events|steps   ndtl-verify {schema,fake,synthetic-faults,mlu,cuda}
```

Module inventory (LOC from `wc -l`):

| File | LOC | Role |
| --- | --- | --- |
| `src/ndtl/schema.py` | 287 | Dataclass records; `Record.to_record()` adds `record_type` and `schema_version` (`schema.py:10-28`) |
| `src/ndtl/observatory.py` | 747 | Public API, writer thread, heartbeat, operation stack |
| `src/ndtl/adapters.py` | 280 | CUDA/MLU/fake queue-and-event capture |
| `src/ndtl/spool.py` | 36 | Append-only file handle, one JSON per line |
| `src/ndtl/analysis.py` | 82 | Straggler, failed-rank, stale-rank detectors |
| `src/ndtl/analyze.py` | 425 | `ndtl-analyze events|steps` |
| `src/ndtl/verify.py` | 501 | `ndtl-verify` smoke/certification commands |
| `src/ndtl/torchtitan_bridge.py` | 488 | Logging handler and metrics-logger monkeypatch |
| `src/ndtl/torchtitan_harness.py` | 192 | Python-side smoke launcher and summary |
| `tools/synd/src/*.rs` | 4,302 | aggregate 1196, ui 1070, main 805, gpu 182, ... |
| `tools/synd/tests/synd_contract.rs` | 1,446 | 34 `#[test]` functions (+3 in `main.rs`) |
| `tests/*.py` | ~2,000 | 65 `def test_` in 11 files |

### 3.2 Key abstractions and invariants

**Record.** A `@dataclass` whose `to_record()` emits a flat dict with `record_type` (class name) and
`schema_version` first (`schema.py:10-28`). `V2Record` sets `schema_version = 2` (`schema.py:44-45`).
Invariants (documented): every line starts with those two keys; readers must preserve unknown
`MeasurementQuality.status` (`docs/schema.md:61-62`); the live model must count and ignore unknown record
types (`docs/synd.md:244-245`, implemented at `aggregate.rs:493`). [READ]

**Identity.** v2 join keys are `run_id, attempt_id, rank, step_id, operation_id, parent_operation_id`
(`docs/schema.md:102-117`). `operation_id` is `"<prefix>-<uuid4>"` (`observatory.py:732-733`) and
`attempt_id` defaults to a fresh uuid4 per `Observatory` (`observatory.py:185`). [READ]
Hazard: because `attempt_id` is per-process by default, two ranks that do not share an explicit
`attempt_id` never agree on it. The TorchTitan bridge reads it from
`TORCHTITAN_ATTEMPT_ID`/`NDTL_ATTEMPT_ID` (`torchtitan_bridge.py:28`) and otherwise each rank generates
its own. **synd does not set either variable** (`launcher.rs:68-73` payload). [READ; consequence INFERRED]
The v3 design calls this out (all ranks must share Run and Attempt).

**Per-rank run id workaround.** The bridge sets `run_id=f"{run_id}-rank-{rank}"`
(`torchtitan_bridge.py:37`), so one job yields N distinct `run_id`s. The v3 design explicitly names this
"the current TorchTitan `run-rank-N` workaround" and removes it. [READ]

**Observatory.** Context managers `update`, `phase`, `microbatch`, `step`, `work`, `collective`, `p2p`
(`observatory.py:269-418`), plus `declare_mesh`, `register_process_group`, `mark_failure`, `flush`, `close`.
- Context is a plain dict swapped in `_push` (`observatory.py:593-600`) and is **not thread-local**; the
  heartbeat thread reads it unlocked (`:689-690`). [READ; race INFERRED, low severity]
- Operation nesting is tracked by a lock-protected stack (`:698-713`), so `parent_operation_id` is
  "whatever was open on the stack", which is only correct for single-threaded use per Observatory. [INFERRED]
- `Observatory` writes **both** a v1 `RunManifest` and a `RunManifestV2` at init (`:211-243`), and for every
  `work`/`collective` writes **both** an `OperationEvent` (v2) and a `WorkEvent`/`CollectiveOperation` (v1)
  (`:339-362`, `:525-548`). Dual-write is permanent on `main`. [READ]

**Writer.** `_AsyncJsonlWriter` (`observatory.py:59-145`): `queue.Queue(maxsize=max_queue_records)`,
`put_nowait`, on `Full` increments `dropped_records` and returns (`:72-81`). One daemon thread writes and
flushes at `flush_interval_s` (`:112-135`). `flush()` is `queue.join()` plus a userspace `flush()` (`:83-91`).
`JsonlSpool.flush` only calls `file.flush()` (`spool.py:26`). **No `fsync` anywhere in the core writer.**
The docs say as much: `flush_async` "should not be treated as a durable-write barrier"
(`docs/schema.md:25-28`). [READ]

**Heartbeat.** One daemon thread per process, default 5 s (`observatory.py:246-248`, `:668-696`).
Carries `step_id`, `phase`, `last_operation_id`, and writer counters (`dropped_records`, `write_failures`,
`flush_count`). [READ]

**CollectorHealth.** Emitted **only** from `flush()` and `close()` (`observatory.py:427`, `:437`,
definition `:647-666`) and `adapter_degraded` is hard-coded `False` (`:661`). So a process that dies
without `close()` writes no health record; the heartbeat counters are then the only live drop signal.
synd's `apply_heartbeat` does not read those counters (`aggregate.rs:631-645`). [READ; consequence INFERRED]

**Quality vocabulary.** `MeasurementQuality.status` in {`complete`, `correlated`, `unsupported`}, plus
booleans `event_recorded_on_actual_backend_queue` (`schema.py:211-216`). `BackendQueueIdentity` carries the
stream/queue id (`schema.py:219-226`). Adapter probing yields `CapabilityReport` (`schema.py:238-243`).
Note that `correlated` is defined in docs but **no adapter on `main` emits it** (`grep correlated
src/ndtl` finds only docs). [READ]

**Adapters** (`adapters.py`):
- CUDA: resolve the backend via `process_group._get_backend(device)` and pick
  `get_coll_cuda_stream_id` else `get_coll_stream_id` (P2P: `get_p2p_cuda_stream_id`/`get_p2p_stream_id`
  with `(device, peer, is_batched)`) (`adapters.py:146-183`). Record `torch.cuda.Event(enable_timing=True)`
  on `torch.cuda.ExternalStream(stream_id)` (`:201-214`). These stream-id getters are **not stock PyTorch**;
  they exist in Torchetta's libtorch fork (see 6.2). Without them the quality is `unsupported`. [READ]
- MLU: probes `torch_mlu.cncl.current_queue_id` and `torch_mlu.notifier.record` (`:227-240`).
- Fake faithful/unsupported adapters for tests (`:73-110`).

**Analyzer detectors** (`analysis.py`): straggler = per-rank median completed `duration_ns` compared to the
smallest median (`:7-28`); failed = `RankFailure.terminal is True` (`:32-40`); stale = latest heartbeat
older than `stale_after_ns` AND its `last_operation_id` is still open (`:44-82`). [READ]

### 3.3 Data flow, TorchTitan path (the only integrated trainer)

1. `synd` (or the Python harness) stages the NDTL checkout into `<report>/staged_ndtl` and writes
   `sitecustomize/sitecustomize.py` containing `import ndtl.torchtitan_bridge` and
   `install_torchtitan_metric_bridge()` (`launcher.rs:105-120`; `torchtitan_harness.py:118-135`). [READ]
2. It runs `<torchtitan>/scripts/rootfs/enter_rootfs.sh -- bash -lc "cd /workspace/torchtitan &&
   PYTHONPATH=…staged_ndtl/sitecustomize:…staged_ndtl/src NDTL_SPOOL_ROOT=<report>
   NDTL_TORCHTITAN_METRICS_PATH=<report>/titan_metrics
   TITAN_STRUCT_LOGGER_HANDLERS=ndtl.torchtitan_bridge.register_ndtl_handler CC=/usr/bin/gcc
   CXX=/usr/bin/g++ MODULE=… CONFIG=… NGPU=… ./run_train.sh --training.steps N"`
   (`launcher.rs:49-88`). [READ]
3. TorchTitan's structured logger calls the factory with `structured_logger, rank, source, output_dir`
   (`torchtitan_bridge.py:157-166`; contract in `torchtitan` repo, `torchtitan/observability/structured_logger/README.md:145`).
   The handler maps `step_start` to `StepProgress(started)`, `metric_value` to a per-step scratch dict,
   `step_end` to `StepProgress(completed)` with promoted metrics, `*_error` to a non-terminal
   `RankFailure`, and every other `*_start/_end` pair to `OperationEvent` (`torchtitan_bridge.py:75-154`). [READ]
4. `install_torchtitan_metric_bridge()` monkey-patches `torchtitan.components.metrics.BaseLogger.log` and
   `LoggerContainer.log` so the trainer's metrics dict is copied to
   `<metrics_dir>/rank-<N>.jsonl` (with `fsync`) and pushed through `log_trace_scalar`
   (`torchtitan_bridge.py:177-287`). [READ]
5. `synd` tails `.ndtl/runs/*/*.ndtl.jsonl` and `titan_metrics/*.jsonl` every 200 ms with offset and
   partial-line handling (`tail.rs:48-65`, `:93-140`), tails `stdout.log` for legacy metric lines
   (`main.rs:303-350`), polls GPU/net/RDMA (`gpu.rs:115`, `network.rs:165-180`), and feeds one aggregator over
   a bounded 4096-slot channel (`main.rs:78`). The UI gets immutable snapshots over a 32-slot channel (`:79`).
6. At exit it writes `summary.json`, `steps.txt`, `gpu.jsonl`, `network.jsonl`, `gpu_processes.jsonl`,
   `network_rates.jsonl`, `rdma.jsonl`, `py_spy_artifacts.jsonl`, and `failure_summary.txt` only on
   failure or interruption (`report.rs:7-104`). Process exit code is 0 iff status is `Succeeded`
   (`main.rs:180-184`). [READ]

### 3.4 Mesh/placement model in synd

`build_mesh_grid` places ranks by `local_rank` row-major with TP as the fastest axis:
`column = local_rank % tp`, `row = (local_rank / tp) % rows`, `rows = dp_replicate * dp_shard * cp`
(`aggregate.rs:790-832`). Row axis is labelled `hsdp` if `dp_replicate > 1` else `fsdp`. Topology sizes come
from `MeshAxis` records or are parsed from TorchTitan stdout topology lines (`aggregate.rs:581`). [READ]
It uses `local_rank`, not global `rank`, so multi-node placement would be wrong. [INFERRED; synd is
single-node by construction, `expected_ranks = args.gpus` at `main.rs:110`]

---

## 4. Design decisions, trade-offs, lessons and known failures

### 4.1 Append-only JSONL, one file per rank

Decision: each rank owns one file; no cross-rank writes; readers tolerate partial tails.
Rationale documented: keep the observer isolated so "observer failures [do not] interrupt training"
(`docs/synd.md:21`). Trade-off: no atomicity across records, no framing, no checksums, so torn writes and
loss are only detectable by the collector's own counters. The v3 design's motivation for integrity framing
(`SegmentOpened/SegmentClosed`, checksum, `producer_sequence`) is exactly these gaps
(`docs/schema-v3.md:37-50`). [READ, BRANCH]

Asymmetry worth knowing: the Rust tailer tolerates partial lines and malformed JSON (`tail.rs:93-140`,
`:185`), but the Python analyzer does `json.loads` on every line with no guard
(`analyze.py:124-130`), as does `verify._read_records` (`verify.py:18-21`) and the Python harness
(`torchtitan_harness.py:46-52`). One torn final line makes `ndtl-analyze` raise. [READ; consequence INFERRED]

### 4.2 Python package stays stdlib-only; Rust sidecar for live view

Decision recorded as a non-goal (`docs/synd.md:27-28`). `pyproject.toml` has no runtime dependencies
(`pyproject.toml:1-13`) and `uv.lock` contains only `ndtl` itself. Torch is imported lazily inside adapters
(`adapters.py:119-122`), so the package imports on hosts without torch (`tests/test_public_api.py:1-13`). [READ]
Trade-off: the record schema exists twice (Python dataclasses, Rust serde structs), with no generated
contract. [INFERRED]

### 4.3 Synchronous threads, not async, in synd

`docs/synd-live-harness-spec.md:97` "synchronous Rust threads and bounded channels, not an async runtime".
Bounded channels mean back-pressure: producers use `let _ = event_tx.send(..)` so if the aggregator falls
behind the tail threads block, not drop. [READ; blocking behavior INFERRED from `sync_channel` semantics]

### 4.4 Dual observer path: logging handler plus metric-logger monkeypatch

Decision: do not modify TorchTitan; use its documented handler-factory env var for lifecycle, and patch
`BaseLogger.log`/`LoggerContainer.log` for trainer metrics because those metrics are not emitted through
the structured logger in the inspected TorchTitan (`torchtitan_bridge.py:177-188`). Commit `bf9f67a`
"Capture TorchTitan step metrics durably" introduced this after stdout parsing proved lossy. [READ]
Trade-off: the monkeypatch couples NDTL to TorchTitan internals (`components/metrics.py:107,233`; and
`observability.structured_logger.step_state.set_step` at `step_state.py:101`). Every import is wrapped in
`try/except: return`, so on a TorchTitan refactor the bridge **silently disables itself**
(`torchtitan_bridge.py:187-191`, `:221-222`). [READ; silent-degrade consequence INFERRED]

### 4.5 Sentinel handling for "successful run with open top-level operation"

Ledger decision (`.superpowers/sdd/synd-rust-live-harness/progress.md`, Task 1 "Ruling", untracked file):
a successful TorchTitan run emits top-level `training` `OperationEvent` starts without completes; after
launcher exit 0 and requested steps done, these are treated as lifecycle sentinels. Cost recorded: a real
hang inside a top-level span "may be downgraded unless accompanied by nonzero exit, missing steps, rank
failure, or stale heartbeat." Implemented at `aggregate.rs:1194-1196` (`op.step_id.is_none() && op.name ==
"training"` and exit 0). [READ]

### 4.6 Timing honesty: documented rule versus implementation

Documented rule: host time is never presented as backend duration (`docs/schema.md:99-100`) and NDTL "does
not insert stream waits" (`docs/schema.md:85-86`). Implementation findings [READ, consequences INFERRED]:

1. **CUDA path blocks the host at scope exit.** `CudaCollectiveCapture.elapsed_us` calls
   `raw_end.synchronize()` before `elapsed_time` (`adapters.py:191-199`), invoked from the `finally` of
   every `collective`/`p2p` scope (`observatory.py:519-520`). For asynchronous NCCL work this turns each
   instrumented collective into a host wait. It is not a stream wait, so the doc sentence is literally
   true, but the perturbation is real. No overhead measurement exists in the repo. [INFERRED]
2. **MLU `complete` is host-clock derived.** `MluCollectiveCapture.record_start/_end` build `BackendEvent`s
   stamped with `time.time_ns()` (`adapters.py:262-276`), and `elapsed_us` is not overridden, so the base
   class computes a wall-clock difference (`adapters.py:65-70`), while quality is `complete` with
   `event_recorded_on_actual_backend_queue=True` (`:250-258`). This contradicts the "faithful backend
   queue timing" meaning of `complete`. [READ]
3. **Mixed clock domains in the same field names.** `StepProgress`, `OperationEvent`, heartbeat use
   `time.monotonic_ns()`, but the v1 `WorkEvent` uses `time.time_ns()` in `started_at_ns`/`ended_at_ns`
   (`observatory.py:317-318`, `:357-358`). `ndtl-analyze` derives "now" as the max over keys including
   `started_at_ns`/`ended_at_ns` (`analyze.py:137-144`), and the Python harness computes
   `records_per_second` over the same keys (`torchtitan_harness.py:148-157`, `_record_elapsed_s`). Any
   spool that contains a `WorkEvent` yields a "now" in wall-clock nanoseconds, so
   `now - heartbeat.monotonic_ns` is enormous and any rank whose last operation is open is reported stale.
   [READ; effect INFERRED, not reproduced]
4. **Cross-rank comparison of monotonic values.** Stale detection compares one global "now" against each
   rank's monotonic heartbeat (`analysis.py:44-82`). Monotonic clocks have per-process origins, and
   nothing records a clock anchor on `main`. Fine on one host with one boot, meaningless across hosts. The
   v3 branch adds `ClockAnchor` for this reason. [INFERRED]
5. **Hang detector needs the heartbeat to go stale.** The heartbeat is emitted by a daemon thread in the same
   process (`observatory.py:668-670`). A rank blocked inside a C++ collective usually releases the GIL, so
   its heartbeat thread keeps running and the heartbeat never goes stale, while `last_operation_id` stays
   the same. `detect_stale_ranks` would then not fire for the canonical NCCL hang. The detector is only
   sound for dead or GIL-starved processes. No test simulates a live-heartbeat hang; `synthetic-faults`
   hand-writes a stale heartbeat (`verify.py:163`). [INFERRED; important]
6. **synd only evaluates staleness after the launcher exits.** `aggregate.rs:351-355` is inside
   `self.launcher_exit_code.is_some()`. During a live run there is no "stale rank" alarm in the snapshot
   warnings. [READ]

### 4.7 "Trainer-native wins over stdout" is enforced only partly

Spec: stdout parsing "must not override trainer-native fields" (`docs/synd-live-harness-spec.md:165-167`).
- Stdout lines are applied with `.or_else` fill-only semantics (`aggregate.rs:521-580`). [READ]
- But synd's stdout tailer also **writes** parsed stdout metrics as `TorchTitanStepMetrics` records into
  `titan_metrics/rank-N.jsonl` (`main.rs:318-335`), the same path the Python metric bridge appends to
  (`torchtitan_bridge.py:279`), and the file tailer re-ingests them. `apply_torchtitan_step_metrics` calls
  `apply_rank_float`, which **overwrites** (`aggregate.rs:1060-1070`, `:685-708`) and does not consult the
  record's `source` (`torchtitan.stdout` vs `torchtitan.metrics_processor`). So a stdout-derived value can
  overwrite a native one depending on arrival order, and two writers share one file. The test named
  `trainer_native_step_progress_wins_over_operation_derived_timing` covers the operation-timing path only.
  [READ; overwrite scenario INFERRED, not reproduced]

### 4.8 Reconciliation design (v3): what was rejected and why

From the design doc [READ, BRANCH]:
- Rejected: making projections primary. "Downstream formats are rebuildable projections and must not
  redefine or silently repair the collected facts."
- Rejected: permanent v2+v3 dual-write. "does not permanently dual-write v2 and v3", and v3 becomes the
  TorchTitan/synd default "only after conformance".
- Rejected: `-1`/`""` sentinels for unknown; use JSON null; units in field names.
- Rejected: wall-clock proximity for collective identity; emit `CollectiveCandidate` with ambiguity instead.
- Rejected: importing `distributed_timeline` at runtime; use its output as a versioned fixture only.
- Rejected: overloading Run identity with Arnold ids; typed `ExternalRunReference` instead
  (`CONTEXT.md`, "External Run Reference").
- Accepted asymmetric strictness: Arnold `strict` profile emits only rows satisfying the legacy contract
  and writes a rejection report; `diagnostic` profile allows host timing with explicit quality tags
  (`docs/legacy-crosswalk.md`).

### 4.9 Lessons and known failures recorded in-repo

- The SDD ledger for trainer metrics is a stub (`.superpowers/sdd/synd-trainer-metrics/progress.md` has
  only a pre-flight line). [READ]
- Commit history shows churn in the Rust UI (`2c1faac` "Simplify synd live TUI", 233+/223-) and repeated
  stdout-versus-native metric reworking. [READ, `git log --stat`]
- Docs lag code (see 8.2). [READ]

---

## 5. Build, test, release, verification (trust and evidence model)

### 5.1 Build

- Python: hatchling, `requires-python >=3.10`, three console scripts (`ndtl-analyze`, `ndtl-verify`,
  `ndtl-torchtitan-harness`) (`pyproject.toml:1-16`). Branch adds `ndtl-derive` and `ndtl-project`
  (venv bin listing in the worktree). [READ]
- Rust: crate `synd` 0.1.0, edition 2021; deps anyhow, clap 4, ctrlc, crossterm 0.28, glob, ratatui 0.29,
  serde, serde_json; dev dep tempfile (`tools/synd/Cargo.toml`). `tools/synd/target` is git-ignored but
  is 820 MB on disk at read time (`du`). [READ]
- There is no CI config, no Makefile, no hooks (`.git/hooks` has only samples), no release process, no
  version tags. [READ, absence]

### 5.2 Test tiers

1. **Host-safe Python unit tests**: `scripts/check_repo.sh` runs `pytest -q` and `compileall src tests`
   via `uv run --with pytest` (`scripts/check_repo.sh:1-10`). 65 test functions across 11 files at
   read time. Coverage themes: schema round trip, bounded-queue drop reports health, write failures reach
   later health records, disabled observatory emits nothing (`tests/test_observatory_v2.py:142-178`), CUDA
   verifier degradations (`tests/test_verify_cli.py:75-196`). [READ; results NOT re-run]
2. **Rust contract tests**: 34 in `tools/synd/tests/synd_contract.rs` plus 3 in `main.rs`, covering CLI
   defaults, rootfs path rules, staging exclusions, tailer partial-line/offset, aggregator semantics,
   parsers for `nvidia-smi`, `/proc/net/dev`, RDMA sysfs, report artifacts, TUI linger. Commands are in
   docs only (`docs/synd-live-harness-spec.md:340-343`): `cargo fmt --check`, `cargo test`,
   `cargo clippy --all-targets -- -D warnings`. **`check_repo.sh` does not run them.** [READ]
3. **Fault-injection verifier**: `ndtl-verify synthetic-faults` writes hand-built records and passes only
   if the straggler, failed-rank, and stale-rank detectors all fire (`verify.py:142-196`, pass condition
   `:194`). This is a positive-control pattern. [READ]
4. **Live GPU certification**: `scripts/verify_cuda_torchetta_rootfs.sh` stages `src/`, enters Torchetta's
   bwrap rootfs, builds a venv with the Torchetta libtorch wheel, and runs
   `python -m ndtl.verify cuda-collective --gpus 2 --require-torchetta-rootfs --spool-root … --output …`
   (`scripts/verify_cuda_torchetta_rootfs.sh:40-58`). The worker runs a 2-rank NCCL all-reduce, batched and
   unbatched P2P, and passes only if every `MeasurementQuality.status == "complete"` when the capability
   probe says CUDA is supported (`verify.py:224-229`, `:232-346`). The report embeds an
   `execution_environment.torchetta_in_rootfs` marker derived from the env var `TORCHETTA_IN_ROOTFS`
   (`verify.py:64-70`). [READ]
5. **TorchTitan smoke**: `ndtl-torchtitan-harness` or `synd torchtitan …` runs the 2-GPU 3-step
   `llama3_debugmodel` and gates on ranks, record types, completed step, and zero write failures
   (`torchtitan_harness.py:55-95`; `docs/synd.md:322-350`). [READ]

Only test-suite scaffolding for tier 1 and 2 was verified by reading; no results are asserted here.
The repo's own ledger asserts 10 Rust tests and 57 Python tests at the time of Task 1
(`.superpowers/sdd/synd-rust-live-harness/progress.md`); those counts are stale (now 37 and 65 by
grep). [DOC, stale]

### 5.3 Trust and evidence model

What is trustworthy by construction:
- The certified CUDA claim is bound to an environment marker plus a pass condition on per-operation
  quality records. A host-only run cannot produce a passing certified report when
  `--require-torchetta-rootfs` is given (`verify.py:385-404`). [READ]
- `synd`'s failure classification is a fixed list of string patterns over warning text
  (`aggregate.rs:1182-1192`). Failure means: nonzero exit, missing ranks, below requested steps, terminal
  failure, stale heartbeat, open non-sentinel operations, collector write failures, spool read errors,
  ranks with no completed step. [READ]
- `ndtl-analyze events` `passed` is false on terminal failure, stale open operation, or collector write
  failure (`analyze.py:36`). Exit code is 1 on failure (`analyze.py:408`). [READ]

What is not (yet) trustworthy [READ unless marked]:
- **Not failures in synd**: dropped records, malformed JSON lines, GPU warnings, unresolved rank-to-GPU
  mapping. They are warnings only (`aggregate.rs:1182-1192` omits them; `docs/synd.md:283`). A run
  that silently dropped records can be reported `succeeded`. [INFERRED from the list]
- **The certification report is unsigned and not content-addressed**: it is a plain JSON file at a cache
  path (`verify_cuda_torchetta_rootfs.sh:9`), rewritten in place, with no digest of the wheel, source, or
  spool. Compare with Milano's digest-bound evidence model (section 7). [INFERRED]
- **The smoke gate is hard-coded to step 3**: `summarize_records` requires `final_completed >= 3` per rank
  regardless of `--steps` (`torchtitan_harness.py:79`), and the command builder has no module/config
  parameters, always `MODULE=llama3 CONFIG=llama3_debugmodel` (`:34`). Running `--steps 1` fails the gate;
  running `--steps 100` passes at step 3. [READ]
- **The staged copy does not exclude large or nested trees.** synd excludes only `.git .venv .ndtl
  .pytest_cache __pycache__ *.pyc` (`launcher.rs:159-166`), and the Python harness the same set
  (`torchtitan_harness.py:121-128`). At read time the NDTL root contains `tools/synd/target` (820 MB) and
  `.worktrees/` (1.7 MB with a venv), both of which would be copied into the report directory on each run.
  [READ; consequence INFERRED]
- **`--report-dir` containment is a textual `starts_with`** on unresolved paths in `main.rs:47`, while the
  rootfs-visible mapping canonicalizes (`launcher.rs:27-49`). Symlinked or `..` inputs can pass the first
  check and fail or diverge in the second. [INFERRED]

### 5.4 Release

None. Version is `0.1.0` in `pyproject.toml:7`, no tag, no changelog, no publishing script. Local `main` is
two commits ahead of `origin/main` (`bf9f67a`): `f7dfaa0` (also on `origin/synd-live-observability`) and
`8bc8ba7`. The v3 branch is local only. [READ, `git branch -vv`, `git log origin/main..main`]

---

## 6. Interfaces with other workspace repos

Legend: who calls whom. "None found" means `git grep -w ndtl` (tracked files) or the specific symbol
search returned nothing; it does not prove there is no unwritten relationship.

### 6.1 torchtitan (`/workspace/torchtitan`), the only live integration

- **Direction:** synd/harness call TorchTitan; TorchTitan calls NDTL code via plug-in points. TorchTitan has
  no reference to NDTL in tracked files. [READ, grep]
- **Contract 1, handler factory.** `TITAN_STRUCT_LOGGER_HANDLERS` is a comma-separated list of
  fully-qualified factory paths; when set, only those run
  (`torchtitan` repo, `torchtitan/observability/structured_logger/README.md:145`; env var documented in
  `structured_logging.py:11`). NDTL's factory signature `register_ndtl_handler(*, structured_logger, rank,
  source, output_dir, **kwargs)` (`torchtitan_bridge.py:157-166`) and it must attach exactly one handler.
  Note: setting the env var **replaces** the default JSONL handler, so TorchTitan's own structured JSONL
  trace is not written in synd runs. [READ; replacement is documented, implication INFERRED]
- **Contract 2, log-record vocabulary.** Handler reads `record.log_type_name`, `step`, `value`,
  `event_name`, `task_name`, `seq_id` (`torchtitan_bridge.py:75-154`). Event names in use:
  `step_start`, `step_end`, `metric_value`, `<x>_start`, `<x>_end`, `<x>_error`. TorchTitan documents
  `log_type_name` (`structured_logging.py:91,134`). [READ]
- **Contract 3, private internals.** `torchtitan.components.metrics.BaseLogger/LoggerContainer` (`metrics.py:107,233`),
  `structured_logger.log_trace_scalar` (`structured_logging.py:257`), `step_state.set_step`
  (`step_state.py:101`). These names exist at torchtitan `843948e9f`. [READ]
- **Contract 4, launch surface.** `scripts/rootfs/enter_rootfs.sh`, `run_train.sh`, env `MODULE CONFIG NGPU`,
  flags `--training.steps`, `--parallelism.data_parallel_{replicate,shard}_degree`,
  `--parallelism.tensor_parallel_degree` (`launcher.rs:49-88`). Path rule: report dir must sit under the
  TorchTitan checkout so it appears at `/workspace/torchtitan/<rel>` in the rootfs (`launcher.rs:27-49`). [READ]

### 6.2 torchetta (`/workspace/torchetta`), CUDA certification substrate

- **Direction:** NDTL's certification script calls Torchetta's rootfs entrypoint and reads its cache layout.
  Torchetta does not call NDTL. [READ]
- **Contract:** `TORCHETTA_ROOT/scripts/rootfs/enter_rootfs.sh`, cache root
  `~/.cache/torchetta`, target `libtorch-v2.10`, a wheel matching
  `spack/opt/<target>/**/artifacts/wheels/torch-*.whl` larger than 10 MB, the in-rootfs marker
  `TORCHETTA_IN_ROOTFS=1`, and the layout `/cache/torchetta/...` inside bwrap
  (`scripts/verify_cuda_torchetta_rootfs.sh:5-38`). Hard-coded absolute defaults include
  `/data02/home/philip.yang/...` (`:5-6`). [READ]
- **Shared vocabulary:** the fork's `get_coll_cuda_stream_id` / `get_p2p_cuda_stream_id` family. Torchetta's
  own verifier `scripts/smoke_ndtimeline_streams.py` checks the same API for `ndtimeline`
  (`torchetta/AGENTS.md:148`, `docs/plans/2026-09-09-native-spack-libtorch-build.md:114`). NDTL's CUDA
  adapter calls the same getters (`adapters.py:155-168`). Whether the two verifiers stay in step is
  unmanaged. [READ; drift risk INFERRED]

### 6.3 distributed_timeline (`bytedance.ndtimeline`)

- **Direction:** none at runtime. NDTL on `main` does not import it. The branch adds an importer for
  `DeviceTimerStreamRecord`, local Chrome Trace, and Arnold ClickHouse rows
  (`src/ndtl/importers/ndtimeline.py` [BRANCH]).
- **Lineage claim (design doc, BRANCH):** `DeviceTimerStreamRecord.to_dict()` produces the legacy field
  family Arnold's `megavision/trace_generator` reads. I confirmed both symbols exist:
  `distributed_timeline/bytedance/ndtimeline/handlers/local_timeline_handler.py` and
  `arnold/megavision/trace_generator/pkg/model/ndtimeline/ndtimeline.go` (`git grep -l`). The field-level
  lineage was not re-verified. [DOC/partial READ]
- **Design doc pins** `distributed_timeline@b43b3e1` and `arnold@756a165c4` and states these are "not proof
  that the same commits are deployed in production." [READ, BRANCH]

### 6.4 arnold

- **Direction:** none on `main`. On the branch, a projector emits Arnold legacy rows
  (`src/ndtl/projectors/arnold_legacy.py`) and a fixture
  `tests/fixtures/arnold_ndcompleteevent_contract.json` pins the row shape (trial, run_id int, role int,
  rank, step, metric, `start_ts[]`, `duration[]`, stage, dp_rank, tp_rank, ...). Credentials, MQ
  destinations, and ClickHouse configuration are explicitly out of scope (`docs/legacy-crosswalk.md`). [READ, BRANCH]
- Arnold is the "product evidence authority" for the observatory (`robust-training-observatory/CONTEXT.md:5`).

### 6.5 milano

- **None found.** No NDTL reference in milano tracked files. The observatory's own research says "Do not
  claim Milano has NdTimeline or NDTL instrumentation today"
  (`robust-training-observatory/docs/research/2026-09-28-training-loop-integration-source-map.md:135`). [READ]
- Conceptual overlap only: Milano's `torch_accel` device is a candidate future collective-capture backend
  ([INFERRED]; NDTL's adapter model would need a `PrivateUse1`/`accel` adapter and NCCL stream getters that
  Milano would have to expose). Milano's evidence model is a useful contrast (section 7).

### 6.6 ultron, vaso

- **ultron:** names "ndtimeline and ndtl" as followed systems in `CONTEXT.md:17` and `README.md:4`. No code
  contract. [READ]
- **vaso:** no NDTL reference in tracked files; its `AGENTS.md` head points at Ultron's agentic-workflow
  block (`vaso/AGENTS.md:1-8`). No relationship to NDTL found. [READ]

### 6.7 robust-training-observatory (documentation consumer)

Cites NDTL at `f7dfaa0` and lists NDTL records as "successor surface". Its research cites line numbers such
as `schema.py:48`, `torchtitan_bridge.py:20`, and `launcher.rs:68`. I re-checked: `schema.py:48`
(`RunManifestV2`) and `torchtitan_bridge.py:20` (`TorchTitanNdtlHandler`) match at `8bc8ba7`; the
`launcher.rs` line numbers are off by one from my count. The observatory recommends validating
`Torchetta torch build -> TorchTitan structured logger -> NDTL handler -> synd report` end to end
(`...training-loop-integration-source-map.md:364`); I found no evidence that has been done beyond NDTL's own
Torchetta script. [READ / DOC]

### 6.8 Shared vocabulary summary

| Term | Meaning in NDTL | Where it must agree |
| --- | --- | --- |
| `run_id`, `attempt_id`, `rank` | Join keys (`docs/schema.md:106-110`) | TorchTitan run-evidence ids (unlinked today); v3 typed external refs |
| `step_id` / `update_id` | Optimizer step / v1 update | TorchTitan `step`; Arnold `step` |
| `axis` in {dp, ddp, fsdp, tp, pp, cp, ep} | Mesh axis names (`docs/schema.md:66`) | Arnold `dp_rank/tp_rank/stage`; synd mesh grid (dp_replicate, dp_shard, tp, cp) |
| `quality` in {complete, correlated, unsupported} | Timing trust (`docs/schema.md:93-97`) | v3 renames `complete` to `faithful` (`docs/schema-v3.md:98`) |
| `operation_id` | Per-rank operation key | TorchTitan ops use `torchtitan:<name>:<step>:<seq>` (`torchtitan_bridge.py:462-465`); analyzer parses that shape (`analyze.py:305-311`) |

---

## 7. Relevance to agentic engineering

### 7.1 How agents work in this repo today

- **No agent guidance files on `main`**: no `AGENTS.md`, `CLAUDE.md`, or `CONTEXT.md` (`ls`). `CONTEXT.md`
  (a domain glossary) exists only on the branch. [READ]
- **Agent workflow artifacts present**: `.superpowers/sdd/<plan>/progress.md` ledgers (untracked) recording
  task rulings and verification results, evidently from a subagent-driven-development workflow, and on the
  branch three `docs/superpowers/plans/*.md` plus a spec under `docs/superpowers/specs/`. Commit messages
  on the branch are one-line `feat:`/`docs:` with no verification trailer. [READ]
- **Gates**: `scripts/check_repo.sh` (pytest + compileall) is the only script gate. Rust gates exist only
  as documented commands. [READ]
- Branch is named `codex/…`, suggesting Codex-family tooling produced it. [INFERRED from prefix only]

### 7.2 Mechanisms worth reusing in corenius

1. **Positive-control verifier.** `synthetic-faults` proves detectors fire on injected faults before
   trusting their silence (`verify.py:142-196`). Reuse: every corenius gate that reports "nothing wrong"
   should ship a paired fault-injection check.
2. **Environment marker as gate input.** A certification report records whether it ran in the certified
   sandbox, and the gate fails closed when it did not (`verify.py:64-70`, `:385-404`). Reuse for any
   "must run in the Insula/rootfs" rule.
3. **Explicit degradation vocabulary.** `complete | correlated | unsupported` with a reason string and a
   boolean provenance flag (`schema.py:211-216`). Reuse: agents should report evidence quality, not just
   pass/fail.
4. **Observer isolation rules.** Never let instrumentation raise into the workload (`bridge emit`
   swallows and records `bridge_degraded`, `torchtitan_bridge.py:53-62`); bounded queues with counted
   drops (`observatory.py:72-81`); "reports written before TUI lingers" (`docs/synd.md:90-93`).
5. **Two-step progress semantics.** `synchronized_completed_step` (min across ranks) versus
   `max_observed_completed_step` (observer skew) as separate fields (`aggregate.rs:504-519`). Reuse:
   distinguish "what the aggregate can assert" from "what one observer saw".
6. **The v3 vocabulary discipline** (branch `CONTEXT.md`): Fact Record vs Semantic Span vs Projection, with
   "_Avoid_" lists, and rules "unknown is null, never -1", "ordering only within a producer", "readers
   never repair source facts". This is directly transferable to corenius's own evidence and receipt models.
7. **Lossy-projection contract with rejection report** (strict vs diagnostic profile, `docs/legacy-crosswalk.md`).
8. **Sidecar over instrumentation.** Observing an unmodified trainer via a documented plug-in seam
   (`TITAN_STRUCT_LOGGER_HANDLERS`) rather than forking it.

### 7.3 Hazards for agents working in or with NDTL

1. **Doc-as-truth.** `docs/synd-live-harness-spec.md:356-365` lists "open implementation work" that is
   already implemented (trainer-native `StepProgress` fields exist at `schema.py:98-109`). `docs/synd.md`
   describes a 12-column `nvidia-smi` query (`docs/synd.md:141`) but the code queries 17 and also compute
   apps (`gpu.rs:49`, `:120-130`); network, RDMA, and `py-spy` features and flags
   (`cli.rs:67-70`) appear nowhere in docs (grep of `docs/` and `README.md` for `py-spy|rdma|network`
   is empty). An agent that trusts docs will mis-scope work. [READ]
2. **Tracked bytecode.** 18 `__pycache__/*.pyc` files are tracked (`git ls-files | grep -c pyc`), and
   `.gitignore` contains only `.worktrees/`. Any Python run dirties the tree; `git add -A` will commit
   bytecode. Multiple interpreters (3.11, 3.12, 3.14) have left files. [READ]
3. **`git status` noise and untracked ledgers.** `.superpowers/` is untracked and not ignored. [READ]
4. **Hard-coded personal absolute paths** in scripts and docs (`/data02/home/philip.yang/...`) make
   commands non-portable and give agents no environment-variable indirection for the workspace root
   (`verify_cuda_torchetta_rootfs.sh:5-6`, docs commands). [READ]
5. **No shared-worktree protocol.** Unlike Milano (`AGENTS.md`, per-agent worktrees, integrate lock),
   here one checkout plus one nested `.worktrees/` exists, and the nested worktree's venv/pyc are inside
   the main checkout tree, which synd would copy into staged runs (5.3). [READ / INFERRED]
6. **GPU use without leases.** synd and the Torchetta script take GPUs directly (`CUDA_VISIBLE_DEVICES`
   default `0,1` in `verify_cuda_torchetta_rootfs.sh:44`, `gpus` arg in synd). There is no lock. Running
   these concurrently with Milano gates (which use flock leases per Milano `AGENTS.md`) can collide.
   [INFERRED; Milano's lease rule is from its AGENTS.md]
7. **`atexit` side effects.** Importing `ndtl.torchtitan_bridge` registers an atexit hook
   (`torchtitan_bridge.py:488`), and via `sitecustomize` it runs in every Python process launched with the
   staged `PYTHONPATH`, including tooling subprocesses, not just training ranks. [INFERRED]
8. **Silent self-disable.** Bridge import failures return without a message (4.4). A green run with zero NDTL
   records is possible only if the gate checks record counts; the Python summary does
   (`torchtitan_harness.py:72-79`), the analyzer's `passed` does not (an empty spool has
   `passed: true`, `analyze.py:36` with empty inputs). [INFERRED; empty-spool behavior from reading only]

---

## 8. Open problems, debt, uncertainties

### 8.1 Design and correctness (all from reading; none reproduced)

1. Live-hang detection is unsound for the common case (4.6.5) and absent during a run in synd (4.6.6).
2. Mixed clock domains in analyzer inputs (4.6.3, 4.6.4).
3. `correlated` quality has no producer on `main` (3.2).
4. MLU `complete` is host-timed (4.6.2). CUDA capture host-syncs (4.6.1). Overhead unmeasured; the v3 design
   lists "Collection overhead is measured against the existing harness baseline" as a to-do (GPU and
   Rootfs verification, BRANCH).
5. `attempt_id` not shared across ranks in bridge runs (3.2), and `run_id` split per rank (3.2).
6. Analyzer and verifier crash on a torn JSONL line (4.1).
7. `CollectorHealth` only at flush/close; synd ignores heartbeat counters (3.2).
8. Native-vs-stdout metric precedence not enforced through files (4.7).
9. `heartbeat_stale_after_s` default 30 (`cli.rs:59-60`), `stale_after_s` default 10 in the analyzer
   (`analyze.py:393`); different defaults for the same concept. [READ]
10. Mesh grid keyed on `local_rank`; single node only (3.4).
11. `ProcessGroupRegistration.ranks` is read from `getattr(process_group, "ranks", [])`
    (`observatory.py:258`); `torch.distributed.ProcessGroup` objects generally do not expose `.ranks`, so
    the field is likely empty in real use. [INFERRED; not checked against the pinned torch]
12. `p2p` in the CUDA verifier labels the axis `pp` even though the test is a 2-rank world with no PP mesh
    (`verify.py:290-298`); harmless for verification, misleading as an example. [READ]
13. `CudaCollectiveCapture._record_event` uses `id(raw_event)` as `event_id` (`adapters.py:209`), which is
    not unique over time. [READ; effect INFERRED]

### 8.2 Debt

- Docs drift (7.3.1). README says the "v1 surface is intentionally narrow" while the package is v2 with a
  sidecar (`README.md:3-4` vs `docs/schema.md:7-9`). [READ]
- Two hand-synchronized schema definitions (Python and Rust).
- `check_repo.sh` omits Rust gates; no CI; tracked pyc files; 820 MB build dir inside the tree that
  staging would copy.
- `tests/test_scripts.py` only asserts substrings in the shell script (`tests/test_scripts.py:8-12`), not
  behavior. [READ]
- Local `main` unpushed (2 commits); v3 branch unpushed (30+ commits, +9,381/-46 lines, 64 files).
- Fixtures on the branch are synthetic. No production Arnold or ClickHouse data has been validated.
- The observatory's docs point at NDTL commit `f7dfaa0`; `main` is now `8bc8ba7` (docs-only delta), so the
  cited line numbers still hold, but the pin will drift as the branch lands.

### 8.3 The v3 branch (status as read)

[BRANCH, read via `git diff --stat main..codex/ndtl-reconciliation-design`]
- 33 files changed in `src/` (+4,731/-46): new packages `fact/`, `ledger/`, `semantic/`, `importers/`,
  `projectors/`, plus `recorder.py`, and `Observatory(recording_format="v2"|"v3")` at `observatory.py:196-213`
  (v3 requires an explicit `attempt_id`).
- Explicitly **not changed**: `torchtitan_bridge.py`, `torchtitan_harness.py`, `schema.py`, and all of
  `tools/synd` (empty `git diff --stat`). Delivery slices 8 and 9 (bridge/synd to v3, Arnold transport) are
  therefore not started. [READ]
- CLI additions: `ndtl-derive`, `ndtl-project`. Integrity: `SegmentOpened`/`SegmentClosed` with SHA-256
  over preceding lines, `producer_sequence`, `flush()` FIFO barrier vs `sync()` with `fsync`
  (`docs/schema-v3.md:37-121`).
- I did not read the branch's implementation or run its tests. The claims above are from its docs and
  diffstat only.

### 8.4 Uncertainties

- Whether anyone other than the repo owner runs NDTL or synd. No evidence either way.
- Whether Torchetta's fork getters return the same stream semantics on the version the observatory pins.
- Whether the MLU adapter has ever run on real hardware; the only in-repo MLU check accepts `unsupported`
  as a pass (`verify.py:199-221`).
- Whether TorchTitan will keep the private logger internals the bridge patches (upstream drift risk).
- Real overhead of the host synchronize in the CUDA path.
- Whether the v3 direction will supersede v2 or be abandoned; the design status line reads "Approved
  conversational design, pending written-spec review".

---

## 9. How to verify this document

Record: read at ndtl `8bc8ba71198b3900081a744dd887cbdde78de4a5` on `main`; branch design intent from
`codex/ndtl-reconciliation-design` `5880bfd`; torchtitan `843948e9f`; robust-training-observatory `584d9b8`.

1. Commit and state: `git -C /data02/home/philip.yang/workspace/ndtl rev-parse HEAD`, `git status --short`,
   `git branch -vv`, `git log origin/main..main --oneline`.
2. Line-count and test-count claims: `wc -l src/ndtl/*.py tools/synd/src/*.rs`;
   `grep -c "^def test" tests/*.py` (sum 65); `grep -c "#\[test\]" tools/synd/tests/synd_contract.rs
   tools/synd/src/main.rs` (34 and 3).
3. Nothing else in the workspace uses NDTL: `for r in torchtitan torchetta milano ultron vaso arnold
   distributed_timeline corenius; do git -C ~/workspace/$r grep -I -l -i -w ndtl; done` (only `ultron`
   CONTEXT.md and README.md match).
4. Hang-detector claim (4.6.5): read `src/ndtl/analysis.py:44-82` and `src/ndtl/observatory.py:668-696`;
   confirm the heartbeat thread is independent of operation state. To test empirically, run a fake
   observatory with an open `collective` scope and a live heartbeat, then feed the JSONL to
   `ndtl-analyze events` and check `stale_ranks` (not done here).
5. Clock-mixing claim (4.6.3): compare `observatory.py:317-318,357-358` (wall) with `:318,344` (monotonic)
   and `analyze.py:137-144`.
6. MLU timing claim: `adapters.py:65-70`, `:250-258`, `:262-276`.
7. CUDA synchronize claim: `adapters.py:191-199` and `observatory.py:519-520`.
8. Stdout/native overwrite claim (4.7): `tools/synd/src/main.rs:318-335`,
   `tools/synd/src/aggregate.rs:685-708`, `:1060-1070`, and `src/ndtl/torchtitan_bridge.py:279`.
9. Harness hard-coding: `src/ndtl/torchtitan_harness.py` lines with `MODULE=llama3` and `>= 3`
   (`grep -n "llama3\|>= 3" src/ndtl/torchtitan_harness.py`).
10. Staging exclusions and sizes: `tools/synd/src/launcher.rs` (`should_exclude`), `du -sh tools/synd/target
    .worktrees`.
11. Docs drift: `grep -n -i "py-spy\|rdma\|network" docs/*.md README.md` (empty) versus
    `tools/synd/src/cli.rs:67-70`, `network.rs`, `profile.rs`.
12. Tracked bytecode: `git ls-files | grep -c pyc` (18); `cat .gitignore`.
13. TorchTitan seam: `git -C ~/workspace/torchtitan grep -n "TITAN_STRUCT_LOGGER_HANDLERS"
    torchtitan/observability/structured_logger/README.md`; and the symbols `BaseLogger`, `LoggerContainer`,
    `log_trace_scalar`, `set_step`.
14. Branch claims: `git -C ndtl diff --stat main codex/ndtl-reconciliation-design -- src tools`, and
    `git -C ndtl show 982e1c7 --stat`; design text is at
    `docs/superpowers/specs/2026-09-28-ndtl-source-reconciliation-design.md` in that branch.
15. Run the repo's own gates yourself (not run by me): `scripts/check_repo.sh`, then
    `cargo fmt --check && cargo test && cargo clippy --all-targets -- -D warnings` under `tools/synd`.
