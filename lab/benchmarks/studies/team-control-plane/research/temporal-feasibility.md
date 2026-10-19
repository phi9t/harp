# Rust Temporal feasibility spike

Recorded 2026-10-01. Corenius commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`; isolated detached worktree `/data02/home/philip.yang/workspace/corenius-temporal-spike`. Inputs read: reference-design `SPEC.md` and ticket `03-implementation-language.md`. This is model-free evidence for the owner, **not a language-stack decision**. No product code, commit, push, paid model or shared service use.

## Question and pre-run scope

Can the public Rust SDK compile and execute a deterministic timer workflow across an abruptly terminated worker, Continue-As-New, and offline replay? Scope and versions were recorded before the build in `.scratch/team-control-plane/scope.md`. This deliberately small test does not exercise the PostgreSQL bridge, durable mailbox, runtime adapter, deployment routing, schema evolution or production operations.

## Pins and official sources

- Official SDK release `v1.0.0`, git commit `399cb7adb40f45eefbef90149e212743e474142c`. [Release](https://github.com/temporalio/sdk-rust/releases/tag/v1.0.0), [crate manifest](https://github.com/temporalio/sdk-rust/blob/399cb7adb40f45eefbef90149e212743e474142c/crates/sdk/Cargo.toml). SDK MSRV is Rust1.92.0; source toolchain pins1.94. Actual test compiler: rustc1.95.0 `59807616e` and cargo1.95.0 `f2d3ce0bd`.
- Exact crates.io SDK/client/macros/workflow1.0.0; resolved Core0.9.0. Reproduction must preserve generated Cargo.lock (hash below). All source pins resolve successfully; source checkout is untouched under scratch.
- [Official SDK guide](https://github.com/temporalio/sdk-rust/blob/399cb7adb40f45eefbef90149e212743e474142c/crates/sdk/README.md) describes deterministic workflow constraints, Continue-As-New, patching, and replay. [WorkflowReplayer implementation](https://github.com/temporalio/sdk-rust/blob/399cb7adb40f45eefbef90149e212743e474142c/crates/sdk/src/workflow_replayer.rs) returns failure on incompatible history. [Client history serializer](https://github.com/temporalio/sdk-rust/blob/399cb7adb40f45eefbef90149e212743e474142c/crates/client/src/workflow_handle.rs) defines `from_json`, `to_json`, and fetching. [Worker options](https://github.com/temporalio/sdk-rust/blob/399cb7adb40f45eefbef90149e212743e474142c/crates/sdk/src/lib.rs) exposes deployment options. These are source-supported features, not all exercised here.
- Local `temporal version1.7.0 (Server1.31.0, UI2.49.1)`; standalone loopback gRPC127.0.0.1:18233, headless dev server with scratch SQLite. No Docker needed. Server restart was **not** exercised.

## Exercised results

| Check | Fresh evidence |
| --- | --- |
| Standalone compile | Passed, including offline replayer. First attempt failed because workflow macros require a direct `temporalio-workflow` dependency; adding exact1.0.0 resolved it. |
| Worker restart | First worker PID2950504 killed with SIGKILL after exported history contained events1–5 ending in TimerStarted; a new process handled later TimerFired and completed control flow. No mocked workflow/server. |
| Continue-As-New | Initial run `01a0f637-58fd-71fb-95cc-ceb7af7b886e` ended at event10 ContinuedAsNew; successor `3b7c6c3e-6c87-4146-a402-5bfa3cd7c705` ended event10 Completed with result1. |
| Offline replay | SDK-exported histories for both runs passed actual `WorkflowReplayer::replay_workflow`, exit0. Worker restart alone is not the replay claim. |
| Replay negative control | Removed timer from workflow and rebuilt; same initial history failed exit1, `[TMPRL1100] Nondeterminism error`, referencing TimerStarted event5. Restored original source afterward. |
| CLI export compatibility | Direct CLI `workflow show --output json` failed SDK `from_json`: missing `event_id` (CLI camelCase/shorthand payload format). SDK client `fetch_history(...).to_json()` worked. Archive histories using SDK serializer unless an explicit conversion is verified. |

Inference: Rust-only implementation is technically plausible on this pinned toy case. This evidence refutes the claim that no public Rust SDK exists. It does not prove general replay correctness, production support suitability, bridge idempotency, safe upgrades, command cursor continuity, rollover races or old-runtime compatibility.

## Reproduction

All generated files and local logs are in `.scratch/team-control-plane/` in this worktree. Start with the Cargo manifest and source below; `cargo build --manifest-path .scratch/team-control-plane/probe/Cargo.toml`. Then:

```bash
temporal server start-dev --headless --ip 127.0.0.1 --port 18233 --db-filename .scratch/team-control-plane/temporal.sqlite
TEMPORAL_ADDRESS=127.0.0.1:18233 .scratch/team-control-plane/probe/target/debug/temporal-feasibility-probe
temporal --address 127.0.0.1:18233 workflow start --workflow-id feasibility-1 --type Probe --task-queue temporal-feasibility --input 0 --output json
temporal --address 127.0.0.1:18233 workflow show --workflow-id feasibility-1 --output json
# After TimerStarted and before TimerFired, SIGKILL the probe worker PID; restart same worker command.
temporal --address 127.0.0.1:18233 workflow result --workflow-id feasibility-1 --output json
# Export each observed run ID through the SDK, not CLI shorthand JSON:
TEMPORAL_ADDRESS=127.0.0.1:18233 .scratch/team-control-plane/probe/target/debug/temporal-feasibility-probe export RUN_ID history.json
.scratch/team-control-plane/probe/target/debug/temporal-feasibility-probe replay history.json
# Negative control: remove ctx.timer(...).await, rebuild, replay same initial history; expect exit1.
```

Use a new workflow ID or fresh scratch database on another execution. Adapt fixed ID in the export mode accordingly. For crash recovery the worker must be killed during its durable timer wait, not after completion.

```toml
[package]
name = "temporal-feasibility-probe"
version = "0.0.0"
edition = "2024"
[dependencies]
temporalio-sdk = { version = "=1.0.0", default-features = false, features = ["envconfig"] }
temporalio-client = { version = "=1.0.0", default-features = false, features = ["envconfig"] }
temporalio-macros = "=1.0.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

temporalio-workflow = "=1.0.0"
```

```rust
use std::time::Duration;
use temporalio_macros::{workflow, workflow_methods};
use temporalio_sdk::{ContinueAsNewOptions, WorkflowContext, WorkflowResult, Runtime, Worker, WorkerOptions};
#[workflow]
#[derive(Default)]
struct Probe;
#[workflow_methods]
impl Probe {
    #[run]
    async fn run(ctx: &mut WorkflowContext<Self>, generation: u32) -> WorkflowResult<u32> {
        ctx.timer(Duration::from_secs(15)).await;
        if generation == 0 { ctx.continue_as_new(1u32, ContinueAsNewOptions::default())?; }
        Ok(generation)
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("replay") {
        use temporalio_sdk::workflow_replayer::{WorkflowReplayer, WorkflowReplayerOptions};
        let replayer = WorkflowReplayer::new(WorkflowReplayerOptions::new().register_workflow::<Probe>()?.build())?;
        for file in &args[2..] {
            let history = temporalio_client::WorkflowHistory::from_json(&std::fs::read(file)?)?;
            replayer.replay_workflow(history).await?;
            println!("REPLAY_OK {file}");
        }
        return Ok(());
    }
    use temporalio_client::{Client, ClientOptions, Connection, envconfig::LoadClientConfigProfileOptions};
    let runtime = Runtime::from_current_tokio(Default::default())?;
    let (conn, opts) = ClientOptions::load_from_config(LoadClientConfigProfileOptions::default())?;
    let client = Client::new(Connection::connect(conn).await?, opts)?;
    if args.get(1).map(String::as_str) == Some("export") {
        use temporalio_client::{WorkflowExecutionInfo, WorkflowFetchHistoryOptions};
        let info = WorkflowExecutionInfo::builder().namespace("default").workflow_id("feasibility-1").run_id(args[2].clone()).build();
        let history = info.bind_untyped(client).fetch_history(WorkflowFetchHistoryOptions::default());
        std::fs::write(&args[3], history.to_json().await?)?;
        return Ok(());
    }
    let opts = WorkerOptions::new("temporal-feasibility").register_workflow::<Probe>()?.build();
    let mut worker = Worker::new(&runtime, client, opts)?;
    println!("WORKER_READY");
    worker.run().await?;
    Ok(())
}
```

## Evidence and cleanup

Tool execution sessions: first build47682 (exit101), corrected build96188 (exit0), dev server92799, original worker42433, restarted worker51468, result/export/replay61710, export build8437, negative build/replay79582, restored build40994. Process handles and shutdown signals are in `killed-worker.txt` and `cleanup.log`. This report records these handles for audit; they are not reproducibility inputs.

Logs: `build-first.log`, `build.log`, `build-export.log`, `build-negative.log`, `build-restored.log`, `server.log`, `worker-first.log`, `worker-restart.log`, `start.json`, `history-before.json`, `history-first.json`, `history-final.json`, `result.json`, `replay.log`, `replay-sdk.log`, `replay-negative.log`, `replay-negative.exit`. Generated scratch is intentionally untracked.

Cleanup sent SIGTERM to probe worker and the dedicated dev-server PID, then verified neither process remained and TCP18233 was closed. No shared services changed. Exact content hashes:

- `probe/Cargo.lock`: SHA256 `d7b8cf2b3ec919ef8e416b00d12c711b766d48a268136be54842fcd95418a514`
- `probe-good.rs`: SHA256 `01cddc6cd2967209be861cbf19d9193de3f7da68feab795e210abd688f68ccb9`
- `sdk-history-first.json`: SHA256 `90f3fd5ab5bd58fb1d58ce1f39645297b66bd1e0d1e69db6da504f7d01ce87ff`
- `sdk-history-final.json`: SHA256 `08627624843ac4ca4c9efc53ee108c74e1578a0ca304cde45da76c0f6e0507cb`
- `replay-sdk.log`: SHA256 `ad35134f9137d9eceb1ca7511fafe43a01c5b8496fea3ddc2892d7cca9545c76`
- `replay-negative.log`: SHA256 `c7bccabec770e9773f8885e3cddb9702b0aece1f724934e468e6d4fb56e64b8f`

## Owner decision and remaining verification criteria

Owner must still select Rust worker or the permitted Python fallback. Deployment consequences: pin SDK/Core and toolchain together, preserve replay fixtures, use deterministic SDK primitives, and treat workflow code evolution as a compatibility change. Before accepting upgrades, replay representative old histories with new code and verify patch/version routing; this spike only tested an incompatible change being rejected. Before kernel acceptance, exercise real bridge delivery/recovery, rollover input/cursor transfer, pending handler handling and terminal wakeups. Both worker and server restart against persistent production-like storage remain broader acceptance work; only worker restart was tested here.
