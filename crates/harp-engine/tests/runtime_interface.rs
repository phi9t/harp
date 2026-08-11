use harp_contracts::RunId;
use harp_engine::{Engine, EngineError, RunSummary};
use harp_runtime::ActivityRuntime;
use harp_state::StateStore;

async fn cancel_run_uses_the_activity_runtime_seam(
    engine: &mut Engine,
    state: &mut StateStore,
    runtime: &mut dyn ActivityRuntime,
    run_id: &RunId,
) -> Result<RunSummary, EngineError> {
    engine.cancel_run(state, runtime, run_id).await
}

#[test]
fn cancel_run_accepts_an_activity_runtime() {
    let _ = cancel_run_uses_the_activity_runtime_seam;
}
