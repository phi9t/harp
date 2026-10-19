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
