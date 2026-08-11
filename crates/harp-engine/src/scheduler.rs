use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use harp_artifacts::{ArtifactStore, ArtifactStoreIdentity};
use harp_contracts::{
    ArtifactRef, NodeKind, OperationId, ResultEnvelope, RunId, RuntimeEvent, TaskId, ThreadId,
    ThreadSpec, TurnSpec, TurnStatus,
};
use harp_runtime::{
    ActivityHandle, ActivityRuntime, ActivitySpec, InterruptPurpose as RuntimeInterruptPurpose,
    RuntimeProvenance,
};
use harp_state::{
    ActivityPreparation, AttemptState, CliActivityKind, CliSignalStage, InterruptBudgetDimension,
    InterruptIntent, OperationKind, OperationState, RunBudget, RunState, StateStore, UsageOutcome,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;

use crate::projection::project_execution_child_context;
use crate::{validate_graph, GraphPolicy, ProjectionPolicy, ProjectionRequest, ValidatedGraph};

const MAX_EXECUTION_RECEIPT_BYTES: usize = 240 * 1024;
const RESULT_MEDIA_TYPE: &str = "application/vnd.harp.result+json";
const EVALUATION_MEDIA_TYPE: &str = "application/vnd.harp.evaluation-receipt+json";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrashPoint {
    BeforeClaimCommit,
    AfterClaim,
    AfterPrepared,
    AfterDispatchingThread,
    AfterThreadId,
    AfterDispatchingTurn,
    DuringTurn,
    AfterArtifactPublication,
    AfterResultPublished,
    BeforeReducerReady,
    DuringReducer,
    AfterEvaluationPublication,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineConfig {
    pub worker_id: String,
    pub lease_seconds: i64,
    pub lease_renewal_threshold_seconds: i64,
    pub runtime_operation_timeout_seconds: u64,
    pub max_concurrency: usize,
    pub logical_time: i64,
    pub crash_point: Option<CrashPoint>,
}

impl EngineConfig {
    fn validate(&self) -> Result<(), EngineError> {
        if self.worker_id.is_empty()
            || self.worker_id.len() > 256
            || self.worker_id.chars().any(char::is_control)
        {
            return Err(EngineError::InvalidConfig {
                context: "worker_id must contain 1..=256 non-control bytes".to_owned(),
            });
        }
        if self.lease_seconds <= 0 {
            return Err(EngineError::InvalidConfig {
                context: "lease_seconds must be positive".to_owned(),
            });
        }
        if self.lease_renewal_threshold_seconds <= 0
            || self.lease_renewal_threshold_seconds >= self.lease_seconds
        {
            return Err(EngineError::InvalidConfig {
                context: "lease_renewal_threshold_seconds must be positive and below lease_seconds"
                    .to_owned(),
            });
        }
        if self.runtime_operation_timeout_seconds == 0 {
            return Err(EngineError::InvalidConfig {
                context: "runtime_operation_timeout_seconds must be positive".to_owned(),
            });
        }
        if self.max_concurrency == 0 {
            return Err(EngineError::InvalidConfig {
                context: "max_concurrency must be positive".to_owned(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RunExecutionReceipt {
    schema_version: u8,
    graph_sha256: String,
    graph_policy: GraphPolicy,
    projection_policy: ProjectionPolicy,
    artifact_store_identity: ArtifactStoreIdentity,
    runtime_provenance: RuntimeProvenance,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PersistedExecutionSpec {
    receipt: RunExecutionReceipt,
    receipt_sha256: String,
}

#[derive(Debug)]
pub struct RunExecutionSpec {
    validated: ValidatedGraph,
    receipt: RunExecutionReceipt,
    receipt_sha256: String,
}

impl RunExecutionSpec {
    pub fn new(
        validated: ValidatedGraph,
        graph_policy: GraphPolicy,
        projection_policy: ProjectionPolicy,
        artifacts: &ArtifactStore,
        runtime_provenance: &RuntimeProvenance,
    ) -> Result<Self, EngineError> {
        let replayed =
            validate_graph(validated.graph().clone(), &graph_policy, &projection_policy)?;
        if replayed != validated {
            return Err(EngineError::ExecutionReceipt {
                context: "replayed validation does not equal the supplied validated graph"
                    .to_owned(),
            });
        }
        let graph_bytes =
            serde_json::to_vec(validated.graph()).map_err(|source| EngineError::Serialization {
                context: "execution graph",
                source,
            })?;
        verify_approved_artifacts(artifacts, &graph_policy)?;
        let receipt = RunExecutionReceipt {
            schema_version: 1,
            graph_sha256: format!("{:x}", Sha256::digest(graph_bytes)),
            graph_policy,
            projection_policy,
            artifact_store_identity: artifacts.identity()?,
            runtime_provenance: runtime_provenance.clone(),
        };
        let receipt_sha256 = receipt_digest(&receipt)?;
        Ok(Self {
            validated,
            receipt,
            receipt_sha256,
        })
    }

    pub fn validated(&self) -> &ValidatedGraph {
        &self.validated
    }

    pub fn receipt_sha256(&self) -> &str {
        &self.receipt_sha256
    }

    pub fn graph_policy(&self) -> &GraphPolicy {
        &self.receipt.graph_policy
    }

    pub fn projection_policy(&self) -> &ProjectionPolicy {
        &self.receipt.projection_policy
    }

    pub(crate) fn run_budget(&self) -> Result<RunBudget, EngineError> {
        RunBudget::new(
            self.receipt.graph_policy.max_total_tokens,
            self.receipt.graph_policy.max_total_storage_bytes,
            self.receipt.graph_policy.max_total_timeout_seconds,
        )
        .map_err(EngineError::from)
    }

    fn provenance(&self) -> Value {
        json!({
            "executionSpec": {
                "receipt": self.receipt,
                "receiptSha256": self.receipt_sha256,
            }
        })
    }

    pub(crate) fn from_persisted(
        graph: harp_contracts::TaskGraph,
        graph_sha256: &str,
        provenance: &Value,
        artifacts: &ArtifactStore,
        runtime_provenance: &RuntimeProvenance,
    ) -> Result<Self, EngineError> {
        let persisted = decode_persisted_execution_spec(graph_sha256, provenance)?;
        if persisted.receipt.artifact_store_identity != artifacts.identity()? {
            return Err(EngineError::ExecutionReceipt {
                context: "artifact store identity differs from the pinned execution receipt"
                    .to_owned(),
            });
        }
        if &persisted.receipt.runtime_provenance != runtime_provenance {
            return Err(EngineError::ExecutionReceipt {
                context: "runtime provenance differs from the pinned execution receipt".to_owned(),
            });
        }
        verify_approved_artifacts(artifacts, &persisted.receipt.graph_policy)?;
        let validated = validate_graph(
            graph,
            &persisted.receipt.graph_policy,
            &persisted.receipt.projection_policy,
        )?;
        Ok(Self {
            validated,
            receipt: persisted.receipt,
            receipt_sha256: persisted.receipt_sha256,
        })
    }

    fn verify_authorities(
        &self,
        artifacts: &ArtifactStore,
        runtime_provenance: &RuntimeProvenance,
    ) -> Result<(), EngineError> {
        if self.receipt.artifact_store_identity != artifacts.identity()? {
            return Err(EngineError::ExecutionReceipt {
                context: "artifact store identity differs from execution spec".to_owned(),
            });
        }
        if &self.receipt.runtime_provenance != runtime_provenance {
            return Err(EngineError::ExecutionReceipt {
                context: "runtime provenance differs from execution spec".to_owned(),
            });
        }
        verify_approved_artifacts(artifacts, &self.receipt.graph_policy)
    }
}

pub(crate) fn verify_persisted_runtime_control(
    graph_sha256: &str,
    provenance: &Value,
    runtime_provenance: &RuntimeProvenance,
) -> Result<(), EngineError> {
    let persisted = decode_persisted_execution_spec(graph_sha256, provenance)?;
    if &persisted.receipt.runtime_provenance != runtime_provenance {
        return Err(EngineError::ExecutionReceipt {
            context: "runtime provenance differs from execution spec".to_owned(),
        });
    }
    Ok(())
}

fn decode_persisted_execution_spec(
    graph_sha256: &str,
    provenance: &Value,
) -> Result<PersistedExecutionSpec, EngineError> {
    let persisted: PersistedExecutionSpec =
        serde_json::from_value(provenance.get("executionSpec").cloned().ok_or_else(|| {
            EngineError::ExecutionReceipt {
                context: "run provenance is missing executionSpec".to_owned(),
            }
        })?)
        .map_err(|source| EngineError::Serialization {
            context: "persisted execution receipt",
            source,
        })?;
    if persisted.receipt.schema_version != 1
        || persisted.receipt.graph_sha256 != graph_sha256
        || receipt_digest(&persisted.receipt)? != persisted.receipt_sha256
    {
        return Err(EngineError::ExecutionReceipt {
            context: "persisted execution receipt failed digest or graph verification".to_owned(),
        });
    }
    Ok(persisted)
}

fn verify_approved_artifacts(
    store: &ArtifactStore,
    policy: &GraphPolicy,
) -> Result<(), EngineError> {
    for approved in policy.approved_artifacts.values() {
        store.read_verified(&approved.reference)?;
    }
    Ok(())
}

fn receipt_digest(receipt: &RunExecutionReceipt) -> Result<String, EngineError> {
    let bytes = serde_json::to_vec(receipt).map_err(|source| EngineError::Serialization {
        context: "execution receipt",
        source,
    })?;
    if bytes.len() > MAX_EXECUTION_RECEIPT_BYTES {
        return Err(EngineError::ExecutionReceipt {
            context: format!(
                "execution receipt uses {} bytes, exceeding {}",
                bytes.len(),
                MAX_EXECUTION_RECEIPT_BYTES
            ),
        });
    }
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunSummary {
    pub run_id: RunId,
    pub completed_tasks: usize,
    pub indeterminate_attempts: usize,
}

pub trait WallClock: Send + Sync + std::fmt::Debug {
    fn unix_seconds(&self) -> Result<i64, EngineError>;

    fn monotonic_elapsed(&self) -> Duration;
}

#[derive(Debug)]
struct SystemWallClock {
    started: Instant,
}

impl WallClock for SystemWallClock {
    fn unix_seconds(&self) -> Result<i64, EngineError> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| EngineError::WallClock)?
            .as_secs();
        i64::try_from(seconds).map_err(|_| EngineError::WallClock)
    }

    fn monotonic_elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}

#[derive(Debug)]
struct SystemLeaseClock;

impl harp_state::LeaseClock for SystemLeaseClock {
    fn unix_seconds(&self) -> harp_state::StateResult<i64> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|source| harp_state::StateError::Clock {
                context: "system time is before the Unix epoch".to_owned(),
                source: Some(source),
            })?
            .as_secs();
        i64::try_from(seconds).map_err(|_| harp_state::StateError::Clock {
            context: "system time exceeds SQLite integer range".to_owned(),
            source: None,
        })
    }
}

#[derive(Debug)]
pub struct Engine {
    pub(crate) config: EngineConfig,
    pub(crate) now: Arc<AtomicI64>,
    semaphore: Arc<Semaphore>,
    pub(crate) injected: bool,
    pub(crate) active_leases:
        Arc<Mutex<BTreeMap<harp_contracts::AttemptId, harp_state::LeaseToken>>>,
    pub(crate) wall_clock: Arc<dyn WallClock>,
    pub(crate) lease_clock: Arc<dyn harp_state::LeaseClock>,
    #[cfg(debug_assertions)]
    pub(crate) before_thread_start_hook: Option<ThreadStartHook>,
}

#[cfg(debug_assertions)]
pub(crate) struct ThreadStartHook(pub(crate) Box<dyn FnOnce() + Send>);

#[cfg(debug_assertions)]
impl std::fmt::Debug for ThreadStartHook {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ThreadStartHook(..)")
    }
}

impl Engine {
    pub fn new(config: EngineConfig) -> Result<Self, EngineError> {
        Self::with_clocks(
            config,
            Arc::new(SystemWallClock {
                started: Instant::now(),
            }),
            Arc::new(SystemLeaseClock),
        )
    }

    pub fn with_wall_clock(
        config: EngineConfig,
        wall_clock: Arc<dyn WallClock>,
    ) -> Result<Self, EngineError> {
        Self::with_clocks(config, wall_clock, Arc::new(SystemLeaseClock))
    }

    pub fn with_clocks(
        config: EngineConfig,
        wall_clock: Arc<dyn WallClock>,
        lease_clock: Arc<dyn harp_state::LeaseClock>,
    ) -> Result<Self, EngineError> {
        config.validate()?;
        wall_clock.unix_seconds()?;
        lease_clock.unix_seconds()?;
        Ok(Self {
            now: Arc::new(AtomicI64::new(config.logical_time)),
            semaphore: Arc::new(Semaphore::new(config.max_concurrency)),
            config,
            injected: false,
            active_leases: Arc::new(Mutex::new(BTreeMap::new())),
            wall_clock,
            lease_clock,
            #[cfg(debug_assertions)]
            before_thread_start_hook: None,
        })
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn test_only_before_thread_start(&mut self, hook: impl FnOnce() + Send + 'static) {
        self.before_thread_start_hook = Some(ThreadStartHook(Box::new(hook)));
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn test_only_active_lease_count(&self) -> usize {
        self.active_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub async fn execute_run(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        spec: &RunExecutionSpec,
    ) -> Result<RunSummary, EngineError> {
        state.set_lease_clock(Arc::clone(&self.lease_clock))?;
        spec.verify_authorities(artifacts, &runtime.provenance()?)?;
        let run_budget = spec.run_budget()?;
        let run = state.create_run(
            spec.validated.graph(),
            &spec.provenance(),
            &run_budget,
            self.tick()?,
        )?;
        self.schedule_run(state, artifacts, runtime, spec, &run.run_id)
            .await
    }

    pub(crate) async fn schedule_run(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        spec: &RunExecutionSpec,
        run_id: &RunId,
    ) -> Result<RunSummary, EngineError> {
        loop {
            let run = state
                .get_run(run_id)?
                .ok_or_else(|| EngineError::MissingRun {
                    run_id: run_id.clone(),
                })?;
            if run.state != RunState::Active {
                break;
            }
            state.rebuild_ready_tasks(run_id, self.tick()?)?;
            let ready = state.ready_tasks(run_id)?;
            if ready.is_empty() {
                break;
            }
            for ready_task in ready {
                if ready_task.kind == NodeKind::Reducer {
                    self.inject(CrashPoint::BeforeReducerReady)?;
                }
                self.inject(CrashPoint::BeforeClaimCommit)?;
                let claim_now = self.tick()?;
                let lease_expiry = self.lease_expiry()?;
                let logical_session_id = ThreadId::from_str(&OperationId::new().to_string())
                    .map_err(EngineError::Contract)?;
                let claim = state
                    .claim_ready_cli_task(
                        run_id,
                        &self.config.worker_id,
                        claim_now,
                        lease_expiry,
                        logical_session_id,
                    )?
                    .ok_or_else(|| EngineError::ExecutionReceipt {
                        context: "ready task disappeared before claim".to_owned(),
                    })?;
                let _lease_scope =
                    ActiveLeaseScope::register(Arc::clone(&self.active_leases), claim.lease());
                self.inject(CrashPoint::AfterClaim)?;
                let semaphore = Arc::clone(&self.semaphore);
                let _permit = semaphore
                    .acquire_owned()
                    .await
                    .map_err(|_| EngineError::SemaphoreClosed)?;
                let result = self
                    .execute_claim(state, artifacts, runtime, spec, &claim)
                    .await;
                result?;
            }
        }
        summarize(state, run_id)
    }

    pub(crate) async fn execute_claim(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        spec: &RunExecutionSpec,
        claim: &harp_state::TaskClaim,
    ) -> Result<(), EngineError> {
        let node = state
            .task_node(&claim.run_id, &claim.task_id)?
            .ok_or_else(|| EngineError::MissingTask {
                task_id: claim.task_id.clone(),
            })?;
        let mut lease = claim.lease();
        let scratch = resolve_and_bind_scratch(state, artifacts, claim, &lease, self.tick()?)?;
        state.start_wall_tracking(&lease, self.wall_clock.unix_seconds()?, self.tick()?)?;
        let logical_session_id = state
            .get_cli_attempt(&claim.attempt_id)?
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "claimed CLI attempt omitted logical session".to_owned(),
            })?
            .logical_session_id;
        let activity_id = OperationId::new();
        let logical_turn_id = harp_contracts::TurnId::from_str(&activity_id.to_string())
            .map_err(EngineError::Contract)?;
        let turn_spec = turn_spec(spec, claim, &activity_id, &scratch)?;
        let activity_spec = ActivitySpec {
            logical_session_id: logical_session_id.clone(),
            logical_turn_id: logical_turn_id.clone(),
            thread_spec: thread_spec(&node, spec, &scratch)?,
            turn_spec: turn_spec.clone(),
            activity_dir: scratch
                .canonical_path()
                .to_str()
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "resolved scratch path is not UTF-8".to_owned(),
                })?
                .to_owned(),
            invocation_sha256: activity_invocation_digest(&turn_spec)?,
            external_session_id: None,
        };
        activity_spec.validate()?;
        let preparation = ActivityPreparation::new(
            activity_spec.activity_dir.clone(),
            activity_spec.invocation_sha256.clone(),
        )?;
        let activity = state.prepare_activity(
            &lease,
            activity_id.clone(),
            &claim.attempt_id,
            CliActivityKind::StartActivity,
            logical_turn_id,
            &preparation,
            self.tick()?,
        )?;
        self.inject(CrashPoint::AfterPrepared)?;
        state.mark_activity_dispatching(&lease, &activity.activity_id, self.tick()?)?;
        self.renew_lease_if_needed(state, &mut lease)?;
        #[cfg(debug_assertions)]
        if let Some(hook) = self.before_thread_start_hook.take() {
            (hook.0)();
        }
        artifacts.verify_runtime_path(&scratch)?;
        let _logical_thread = self
            .drive_runtime_future(
                state,
                &mut lease,
                Duration::from_secs(claim.budget.timeout_seconds),
                runtime.start_logical_session(activity_spec.thread_spec.clone()),
            )
            .await?;
        self.renew_lease_if_needed(state, &mut lease)?;
        self.inject(CrashPoint::AfterDispatchingThread)?;
        self.renew_lease_if_needed(state, &mut lease)?;
        artifacts.verify_runtime_path(&scratch)?;
        let handle = self
            .drive_runtime_future(
                state,
                &mut lease,
                Duration::from_secs(claim.budget.timeout_seconds),
                runtime.start_activity(activity_spec.clone()),
            )
            .await?;
        if handle.logical_session_id != logical_session_id
            || handle.logical_turn_id != activity.logical_turn_id
        {
            return Err(EngineError::ExecutionReceipt {
                context: "runtime returned a different activity identity".to_owned(),
            });
        }
        state.record_process(
            &lease,
            &activity.activity_id,
            &handle.process_record_sha256,
            self.tick()?,
        )?;
        if let Some(external_session_id) = handle.external_session_id.clone() {
            state.record_cli_external_session(
                &lease,
                &activity.activity_id,
                external_session_id,
                self.tick()?,
            )?;
        }
        state.mark_activity_running(&lease, &activity.activity_id, self.tick()?)?;
        self.renew_lease_if_needed(state, &mut lease)?;
        self.inject(CrashPoint::AfterDispatchingTurn)?;
        let result = self
            .consume_activity(
                state,
                artifacts,
                runtime,
                claim,
                &mut lease,
                &handle,
                &turn_spec.output_schema,
            )
            .await?;
        self.publish_and_accept_cli(
            state,
            artifacts,
            claim,
            &lease,
            &node,
            &activity.activity_id,
            &result,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn consume_activity(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        claim: &harp_state::TaskClaim,
        lease: &mut harp_state::LeaseToken,
        activity: &ActivityHandle,
        output_schema: &serde_json::Value,
    ) -> Result<ResultEnvelope, EngineError> {
        let node_kind = state
            .get_task(&claim.run_id, &claim.task_id)?
            .ok_or_else(|| EngineError::MissingTask {
                task_id: claim.task_id.clone(),
            })?
            .kind;
        let mut observed = state
            .reservation_usage(&claim.attempt_id)?
            .map(|usage| usage.0)
            .unwrap_or(0);
        let mut latest_turn_tokens = state
            .get_attempt(&claim.attempt_id)?
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "activity attempt disappeared before event collection".to_owned(),
            })?
            .latest_turn_observed_tokens;
        let monotonic_started = self.wall_clock.monotonic_elapsed();
        let attempt =
            state
                .get_attempt(&claim.attempt_id)?
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "activity attempt disappeared before wall collection".to_owned(),
                })?;
        let wall_started_at =
            attempt
                .wall_started_at
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "activity attempt has no persisted wall tracking start".to_owned(),
                })?;
        let initial_wall = attempt.observed_wall_seconds.max(elapsed_epoch_seconds(
            wall_started_at,
            self.wall_clock.unix_seconds()?,
        )?);
        let observed_at = self.wall_clock.unix_seconds()?;
        self.reconcile_wall_usage_checked(
            state,
            lease,
            attempt.observed_wall_seconds,
            initial_wall,
            observed_at,
            &claim.task_id,
        )?;
        loop {
            self.renew_lease_if_needed(state, lease)?;
            let observed_wall =
                self.charged_wall_seconds(initial_wall, wall_started_at, monotonic_started)?;
            let remaining_wall = claim.budget.timeout_seconds.saturating_sub(observed_wall);
            if remaining_wall == 0 {
                let previous_wall = state
                    .get_attempt(&claim.attempt_id)?
                    .ok_or_else(|| EngineError::ExecutionReceipt {
                        context: "activity attempt disappeared at wall deadline".to_owned(),
                    })?
                    .observed_wall_seconds;
                let observed_at = self.wall_clock.unix_seconds()?;
                self.reconcile_wall_usage_checked(
                    state,
                    lease,
                    previous_wall,
                    observed_wall,
                    observed_at,
                    &claim.task_id,
                )?;
                let intent = InterruptIntent::budget(InterruptBudgetDimension::Wall, "wall_limit")?;
                self.interrupt_activity_for_limit(state, runtime, lease, activity, &intent)
                    .await?;
                state.fail_terminal_budget_exhausted(lease, &intent.reason_code, self.tick()?)?;
                return Err(EngineError::BudgetExceeded {
                    task_id: claim.task_id.clone(),
                });
            }
            let wait = Duration::from_secs(remaining_wall.min(5));
            let event = match self
                .drive_runtime_future(state, lease, wait, runtime.next_event(activity, wait))
                .await
            {
                Ok(event) => event,
                Err(EngineError::RuntimeOperationTimeout { .. }) => {
                    let previous = state
                        .get_attempt(&claim.attempt_id)?
                        .ok_or_else(|| EngineError::ExecutionReceipt {
                            context: "activity attempt disappeared during wall timeout".to_owned(),
                        })?
                        .observed_wall_seconds;
                    let charged = self
                        .charged_wall_seconds(initial_wall, wall_started_at, monotonic_started)?
                        .max(
                            previous
                                .checked_add(wait.as_secs())
                                .ok_or(EngineError::UsageOverflow)?,
                        );
                    let observed_at = self.wall_clock.unix_seconds()?;
                    let wall = self.reconcile_wall_usage_checked(
                        state,
                        lease,
                        previous,
                        charged,
                        observed_at,
                        &claim.task_id,
                    )?;
                    if charged >= claim.budget.timeout_seconds
                        || wall.attempt_limit_exceeded
                        || wall.run_limit_exceeded
                    {
                        let intent =
                            InterruptIntent::budget(InterruptBudgetDimension::Wall, "wall_limit")?;
                        self.interrupt_activity_for_limit(state, runtime, lease, activity, &intent)
                            .await?;
                        state.fail_terminal_budget_exhausted(
                            lease,
                            &intent.reason_code,
                            self.tick()?,
                        )?;
                        return Err(EngineError::BudgetExceeded {
                            task_id: claim.task_id.clone(),
                        });
                    }
                    continue;
                }
                Err(error) => return Err(error),
            };
            self.renew_lease_if_needed(state, lease)?;
            let charged =
                self.charged_wall_seconds(initial_wall, wall_started_at, monotonic_started)?;
            let previous_wall = state
                .get_attempt(&claim.attempt_id)?
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "activity attempt disappeared during wall reconciliation".to_owned(),
                })?
                .observed_wall_seconds;
            let observed_at = self.wall_clock.unix_seconds()?;
            let wall = self.reconcile_wall_usage_checked(
                state,
                lease,
                previous_wall,
                charged,
                observed_at,
                &claim.task_id,
            )?;
            match event {
                RuntimeEvent::TokenUsage(event) => {
                    if event.usage.total_tokens < latest_turn_tokens {
                        return Err(EngineError::InvalidResult {
                            task_id: claim.task_id.clone(),
                            context: "activity token usage decreased".to_owned(),
                        });
                    }
                    latest_turn_tokens = event.usage.total_tokens;
                    let expected_previous_turn_tokens = state
                        .get_attempt(&claim.attempt_id)?
                        .ok_or_else(|| EngineError::ExecutionReceipt {
                            context: "activity attempt disappeared during usage reconciliation"
                                .to_owned(),
                        })?
                        .latest_turn_observed_tokens;
                    let (new_total, outcome) = state.reconcile_turn_usage(
                        lease,
                        expected_previous_turn_tokens,
                        latest_turn_tokens,
                        0,
                        self.tick()?,
                    )?;
                    observed = new_total;
                    let active_storage = match attempt_file_storage(artifacts, claim) {
                        Ok(storage) => storage,
                        Err(error) if scanner_policy_error(&error) => {
                            let observed_storage =
                                scanner_observed_storage(&error, claim.budget.max_storage_bytes);
                            state.reconcile_usage(
                                lease,
                                observed,
                                observed,
                                observed_storage,
                                self.tick()?,
                            )?;
                            let intent = scanner_interrupt_intent(&error)?;
                            self.interrupt_activity_for_limit(
                                state, runtime, lease, activity, &intent,
                            )
                            .await?;
                            terminalize_interrupt_intent(state, lease, &intent, self.tick()?)?;
                            return Err(error);
                        }
                        Err(error) => return Err(error),
                    };
                    let (_, storage_outcome) = state.reconcile_turn_usage(
                        lease,
                        latest_turn_tokens,
                        latest_turn_tokens,
                        active_storage,
                        self.tick()?,
                    )?;
                    if observed > claim.budget.max_tokens
                        || active_storage > claim.budget.max_storage_bytes
                        || matches!(outcome, UsageOutcome::Exceeded { .. })
                        || matches!(storage_outcome, UsageOutcome::Exceeded { .. })
                        || wall.attempt_limit_exceeded
                        || wall.run_limit_exceeded
                    {
                        let intent = usage_interrupt_intent(
                            observed,
                            active_storage,
                            claim,
                            &outcome,
                            &storage_outcome,
                            &wall,
                        )?;
                        self.interrupt_activity_for_limit(state, runtime, lease, activity, &intent)
                            .await?;
                        state.fail_terminal_budget_exhausted(
                            lease,
                            &intent.reason_code,
                            self.tick()?,
                        )?;
                        return Err(EngineError::BudgetExceeded {
                            task_id: claim.task_id.clone(),
                        });
                    }
                    self.inject(if node_kind == NodeKind::Reducer {
                        CrashPoint::DuringReducer
                    } else {
                        CrashPoint::DuringTurn
                    })?;
                }
                RuntimeEvent::TurnCompleted(completed) => {
                    if completed.turn.status != TurnStatus::Completed {
                        return Err(EngineError::IncompleteTurn {
                            task_id: claim.task_id.clone(),
                        });
                    }
                    let message = match completed.turn.final_agent_message {
                        Some(message) => message,
                        None => {
                            let error = EngineError::InvalidResult {
                                task_id: claim.task_id.clone(),
                                context: "completed activity omitted final agent message"
                                    .to_owned(),
                            };
                            return self.fail_semantic_result(
                                state,
                                artifacts,
                                claim,
                                lease,
                                "missing_output",
                                error,
                            );
                        }
                    };
                    let result = match crate::output::decode_result_envelope(
                        output_schema,
                        message.as_bytes(),
                        &claim.task_id,
                        Some(observed),
                    ) {
                        Ok(result) => result,
                        Err(error) if semantic_output_error(&error) => {
                            return self.fail_semantic_result(
                                state,
                                artifacts,
                                claim,
                                lease,
                                semantic_failure_class(&error),
                                error,
                            );
                        }
                        Err(error) => return Err(error),
                    };
                    let storage_bytes = match result_storage_bytes(artifacts, claim, &result) {
                        Ok(storage) => storage,
                        Err(error) if scanner_policy_error(&error) => {
                            let observed_storage =
                                scanner_observed_storage(&error, claim.budget.max_storage_bytes);
                            state.reconcile_usage(
                                lease,
                                observed,
                                observed,
                                observed_storage,
                                self.tick()?,
                            )?;
                            state.fail_terminal_semantic(
                                lease,
                                "storage_integrity",
                                self.tick()?,
                            )?;
                            return Err(error);
                        }
                        Err(error) => return Err(error),
                    };
                    let outcome = state.reconcile_usage(
                        lease,
                        observed,
                        observed,
                        storage_bytes,
                        self.tick()?,
                    )?;
                    if storage_bytes > claim.budget.max_storage_bytes
                        || matches!(outcome, UsageOutcome::Exceeded { .. })
                        || wall.attempt_limit_exceeded
                        || wall.run_limit_exceeded
                    {
                        state.fail_terminal_budget_exhausted(
                            lease,
                            if wall.attempt_limit_exceeded || wall.run_limit_exceeded {
                                "wall_limit"
                            } else {
                                "storage_limit"
                            },
                            self.tick()?,
                        )?;
                        return Err(EngineError::BudgetExceeded {
                            task_id: claim.task_id.clone(),
                        });
                    }
                    return Ok(result);
                }
                RuntimeEvent::Disconnected(_) | RuntimeEvent::Lagged(_) => {
                    return Err(EngineError::IncompleteTurn {
                        task_id: claim.task_id.clone(),
                    });
                }
                RuntimeEvent::ServerRequest(_) => {
                    return Err(EngineError::InvalidResult {
                        task_id: claim.task_id.clone(),
                        context: "runtime requested unsupported server interaction".to_owned(),
                    });
                }
                RuntimeEvent::TurnStarted(_) | RuntimeEvent::ThreadStarted(_) => {}
            }
        }
    }

    pub(crate) async fn interrupt_activity_for_limit(
        &mut self,
        state: &mut StateStore,
        runtime: &mut dyn ActivityRuntime,
        lease: &mut harp_state::LeaseToken,
        activity: &ActivityHandle,
        intent: &InterruptIntent,
    ) -> Result<(), EngineError> {
        let purpose = match intent.purpose {
            harp_state::InterruptPurpose::Budget => RuntimeInterruptPurpose::Budget,
            harp_state::InterruptPurpose::ScannerIntegrity => {
                RuntimeInterruptPurpose::ScannerIntegrity
            }
            harp_state::InterruptPurpose::Cancellation => RuntimeInterruptPurpose::Cancellation,
        };
        let interrupt_id = OperationId::new();
        let preparation = ActivityPreparation::new(
            format!("/private/tmp/harp-interrupt-{}", interrupt_id),
            activity.process_record_sha256.clone(),
        )?;
        let interrupt = state.prepare_interrupt_activity(
            lease,
            interrupt_id,
            0,
            &preparation,
            &activity.process_record_sha256,
            intent.purpose,
            self.tick()?,
        )?;
        let receipt = self
            .drive_runtime_future(
                state,
                lease,
                self.runtime_control_deadline(),
                runtime.interrupt(activity, purpose),
            )
            .await?;
        if receipt.process_record_sha256 != activity.process_record_sha256 {
            return Err(EngineError::ExecutionReceipt {
                context: "interrupt receipt targets a different process record".to_owned(),
            });
        }
        state.advance_cli_signal_stage(
            lease,
            &interrupt.activity_id,
            CliSignalStage::Prepared,
            CliSignalStage::SigintPrepared,
            self.tick()?,
        )?;
        state.advance_cli_signal_stage(
            lease,
            &interrupt.activity_id,
            CliSignalStage::SigintPrepared,
            CliSignalStage::SigintSent,
            self.tick()?,
        )?;
        state.advance_cli_signal_stage(
            lease,
            &interrupt.activity_id,
            CliSignalStage::SigintSent,
            if receipt.quiescent {
                CliSignalStage::Quiescent
            } else {
                CliSignalStage::Indeterminate
            },
            self.tick()?,
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn publish_and_accept_cli(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        claim: &harp_state::TaskClaim,
        lease: &harp_state::LeaseToken,
        node: &harp_contracts::TaskNode,
        activity_id: &OperationId,
        result: &ResultEnvelope,
    ) -> Result<(), EngineError> {
        state.mark_activity_reconciling(lease, activity_id, self.tick()?)?;
        let publish_operation_id = OperationId::new();
        let operation =
            state.bridge_cli_completion(lease, activity_id, &publish_operation_id, self.tick()?)?;
        if operation.state == OperationState::Prepared {
            state.mark_operation_dispatching(lease, &operation.operation_id, self.tick()?)?;
            state.complete_operation(lease, &operation.operation_id, None, self.tick()?)?;
        } else if operation.state == OperationState::Dispatching {
            state.complete_operation(lease, &operation.operation_id, None, self.tick()?)?;
        } else if operation.state != OperationState::Completed {
            return Err(EngineError::ExecutionReceipt {
                context: format!(
                    "CLI publication operation is unexpectedly {}",
                    operation.state.as_str()
                ),
            });
        }
        verify_and_register_result_artifacts(state, artifacts, result, self.tick()?)?;
        let result_bytes =
            serde_json::to_vec(result).map_err(|source| EngineError::Serialization {
                context: "result envelope",
                source,
            })?;
        let result_ref = artifacts.publish(&result_bytes, RESULT_MEDIA_TYPE)?;
        self.inject(CrashPoint::AfterArtifactPublication)?;
        state.register_artifact(&result_ref, self.tick()?)?;
        state.record_result_published(
            lease,
            &operation.operation_id,
            result,
            &result_ref,
            self.tick()?,
        )?;
        self.inject(CrashPoint::AfterResultPublished)?;
        self.finish_published_result(
            state,
            artifacts,
            claim,
            lease,
            node.kind,
            &result_ref.sha256,
        )?;
        Ok(())
    }

    pub(crate) fn finish_published_result(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        claim: &harp_state::TaskClaim,
        lease: &harp_state::LeaseToken,
        node_kind: NodeKind,
        result_sha256: &str,
    ) -> Result<(), EngineError> {
        if node_kind == NodeKind::Reducer {
            let evaluation =
                state.prepare_operation(lease, OperationKind::Evaluate, 0, self.tick()?)?;
            match evaluation.state {
                OperationState::Prepared => {
                    state.mark_operation_dispatching(
                        lease,
                        &evaluation.operation_id,
                        self.tick()?,
                    )?;
                    let evaluation_ref =
                        publish_evaluation_receipt(artifacts, &claim.task_id, result_sha256)?;
                    state.register_artifact(&evaluation_ref, self.tick()?)?;
                    let (tokens, _) =
                        state.reservation_usage(&claim.attempt_id)?.ok_or_else(|| {
                            EngineError::ExecutionReceipt {
                                context: "evaluation attempt has no budget reservation".to_owned(),
                            }
                        })?;
                    let evaluation_storage =
                        published_result_storage(state, artifacts, claim, result_sha256)?
                            .checked_add(evaluation_ref.size_bytes)
                            .ok_or(EngineError::UsageOverflow)?;
                    let outcome = state.reconcile_usage(
                        lease,
                        tokens,
                        tokens,
                        evaluation_storage,
                        self.tick()?,
                    )?;
                    if evaluation_storage > claim.budget.max_storage_bytes
                        || matches!(outcome, UsageOutcome::Exceeded { .. })
                    {
                        state.fail_terminal_budget_exhausted(
                            lease,
                            "evaluation_storage_limit",
                            self.tick()?,
                        )?;
                        return Err(EngineError::BudgetExceeded {
                            task_id: claim.task_id.clone(),
                        });
                    }
                    self.inject(CrashPoint::AfterEvaluationPublication)?;
                    state.complete_operation(
                        lease,
                        &evaluation.operation_id,
                        Some(&evaluation_ref.sha256),
                        self.tick()?,
                    )?;
                }
                OperationState::Dispatching => {
                    let evaluation_ref =
                        publish_evaluation_receipt(artifacts, &claim.task_id, result_sha256)?;
                    state.register_artifact(&evaluation_ref, self.tick()?)?;
                    let (tokens, _) =
                        state.reservation_usage(&claim.attempt_id)?.ok_or_else(|| {
                            EngineError::ExecutionReceipt {
                                context: "evaluation attempt has no budget reservation".to_owned(),
                            }
                        })?;
                    let evaluation_storage =
                        published_result_storage(state, artifacts, claim, result_sha256)?
                            .checked_add(evaluation_ref.size_bytes)
                            .ok_or(EngineError::UsageOverflow)?;
                    let outcome = state.reconcile_usage(
                        lease,
                        tokens,
                        tokens,
                        evaluation_storage,
                        self.tick()?,
                    )?;
                    if evaluation_storage > claim.budget.max_storage_bytes
                        || matches!(outcome, UsageOutcome::Exceeded { .. })
                    {
                        state.fail_terminal_budget_exhausted(
                            lease,
                            "evaluation_storage_limit",
                            self.tick()?,
                        )?;
                        return Err(EngineError::BudgetExceeded {
                            task_id: claim.task_id.clone(),
                        });
                    }
                    state.complete_operation(
                        lease,
                        &evaluation.operation_id,
                        Some(&evaluation_ref.sha256),
                        self.tick()?,
                    )?;
                }
                OperationState::Completed => {}
                other => {
                    return Err(EngineError::ExecutionReceipt {
                        context: format!("evaluation operation is unexpectedly {}", other.as_str()),
                    });
                }
            }
        }
        state.accept_result(
            &claim.run_id,
            &claim.task_id,
            &claim.attempt_id,
            self.tick()?,
        )?;
        Ok(())
    }

    pub(crate) fn inject(&mut self, point: CrashPoint) -> Result<(), EngineError> {
        if !self.injected && self.config.crash_point == Some(point) {
            self.injected = true;
            #[cfg(all(debug_assertions, unix))]
            if std::env::var_os("HARP_ENGINE_TEST_ABRUPT_EXIT").is_some() {
                unsafe {
                    libc::_exit(86);
                }
            }
            return Err(EngineError::InjectedCrash { point });
        }
        Ok(())
    }

    pub(crate) fn tick(&self) -> Result<i64, EngineError> {
        self.now
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |now| now.checked_add(1))
            .map(|previous| previous + 1)
            .map_err(|_| EngineError::ClockOverflow)
    }

    pub(crate) fn lease_expiry(&self) -> Result<i64, EngineError> {
        self.lease_clock
            .unix_seconds()?
            .checked_add(self.config.lease_seconds)
            .ok_or(EngineError::ClockOverflow)
    }

    pub(crate) fn runtime_control_deadline(&self) -> Duration {
        Duration::from_secs(self.config.runtime_operation_timeout_seconds)
    }

    pub(crate) fn renew_lease_if_needed(
        &mut self,
        state: &mut StateStore,
        lease: &mut harp_state::LeaseToken,
    ) -> Result<(), EngineError> {
        let mut active = self
            .active_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if active.get(lease.attempt_id()) != Some(lease) {
            return Err(EngineError::LeaseAuthorityTransferred {
                attempt_id: lease.attempt_id().clone(),
            });
        }
        let lease_now = self.lease_clock.unix_seconds()?;
        if lease.expires_at().saturating_sub(lease_now)
            > self.config.lease_renewal_threshold_seconds
        {
            return Ok(());
        }
        let new_expires_at = lease_now
            .checked_add(self.config.lease_seconds)
            .ok_or(EngineError::ClockOverflow)?;
        let renewed = state.renew_lease(lease, new_expires_at, self.tick()?)?;
        *lease = renewed;
        active.insert(lease.attempt_id().clone(), lease.clone());
        Ok(())
    }

    pub(crate) async fn drive_runtime_future<T, F>(
        &mut self,
        state: &mut StateStore,
        lease: &mut harp_state::LeaseToken,
        deadline: Duration,
        future: F,
    ) -> Result<T, EngineError>
    where
        F: Future<Output = Result<T, harp_runtime::RuntimeError>>,
    {
        tokio::pin!(future);
        let renewal_period =
            Duration::from_secs(self.config.lease_renewal_threshold_seconds as u64);
        let renewal = tokio::time::sleep(renewal_period);
        let timeout = tokio::time::sleep(deadline);
        tokio::pin!(renewal);
        tokio::pin!(timeout);
        loop {
            tokio::select! {
                result = &mut future => return result.map_err(EngineError::from),
                _ = &mut renewal => {
                    self.renew_lease_if_needed(state, lease)?;
                    renewal.as_mut().reset(tokio::time::Instant::now() + renewal_period);
                }
                _ = &mut timeout => {
                    return Err(EngineError::RuntimeOperationTimeout { deadline });
                }
            }
        }
    }

    fn charged_wall_seconds(
        &self,
        initial: u64,
        wall_started_at: i64,
        monotonic_started: Duration,
    ) -> Result<u64, EngineError> {
        // Charge the larger of process-monotonic elapsed and Unix-epoch delta.
        // This preserves active-process monotonicity and conservatively charges
        // host suspend on targets where `Instant` may pause during sleep.
        let elapsed = self
            .wall_clock
            .monotonic_elapsed()
            .saturating_sub(monotonic_started);
        let monotonic = charged_monotonic_seconds(initial, elapsed)?;
        let epoch = elapsed_epoch_seconds(wall_started_at, self.wall_clock.unix_seconds()?)?;
        Ok(monotonic.max(epoch))
    }

    pub(crate) fn reconcile_recovered_wall(
        &mut self,
        state: &mut StateStore,
        lease: &harp_state::LeaseToken,
        attempt: &harp_state::AttemptRecord,
    ) -> Result<harp_state::WallUsageOutcome, EngineError> {
        let wall_started_at =
            attempt
                .wall_started_at
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "recovered attempt has no persisted wall tracking start".to_owned(),
                })?;
        let charged = attempt.observed_wall_seconds.max(elapsed_epoch_seconds(
            wall_started_at,
            self.wall_clock.unix_seconds()?,
        )?);
        self.reconcile_wall_usage_checked(
            state,
            lease,
            attempt.observed_wall_seconds,
            charged,
            self.wall_clock.unix_seconds()?,
            &attempt.task_id,
        )
    }

    pub(crate) fn fail_semantic_result<T>(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        claim: &harp_state::TaskClaim,
        lease: &harp_state::LeaseToken,
        failure_class: &str,
        error: EngineError,
    ) -> Result<T, EngineError> {
        let storage = match attempt_file_storage(artifacts, claim) {
            Ok(storage) => storage,
            Err(error) if scanner_policy_error(&error) => {
                let (tokens, previous_storage) = state
                    .reservation_usage(&claim.attempt_id)?
                    .ok_or_else(|| EngineError::ExecutionReceipt {
                        context: "semantic failure attempt has no budget reservation".to_owned(),
                    })?;
                let observed_storage =
                    scanner_observed_storage(&error, claim.budget.max_storage_bytes)
                        .max(previous_storage);
                state.reconcile_usage(lease, tokens, tokens, observed_storage, self.tick()?)?;
                state.fail_terminal_semantic(lease, "storage_integrity", self.tick()?)?;
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        let (tokens, previous_storage) =
            state.reservation_usage(&claim.attempt_id)?.ok_or_else(|| {
                EngineError::ExecutionReceipt {
                    context: "semantic failure attempt has no budget reservation".to_owned(),
                }
            })?;
        state.reconcile_usage(
            lease,
            tokens,
            tokens,
            storage.max(previous_storage),
            self.tick()?,
        )?;
        state.fail_terminal_semantic(lease, failure_class, self.tick()?)?;
        Err(error)
    }

    fn reconcile_wall_usage_checked(
        &mut self,
        state: &mut StateStore,
        lease: &harp_state::LeaseToken,
        previous: u64,
        charged: u64,
        observed_at: i64,
        task_id: &TaskId,
    ) -> Result<harp_state::WallUsageOutcome, EngineError> {
        match state.reconcile_wall_usage(lease, previous, charged, observed_at, self.tick()?) {
            Ok(outcome) => Ok(outcome),
            Err(harp_state::StateError::Clock { context, .. }) => {
                state.fail_terminal_semantic(lease, "wall_clock_integrity", self.tick()?)?;
                Err(EngineError::WallClockIntegrity {
                    task_id: task_id.clone(),
                    context,
                })
            }
            Err(error) => Err(error.into()),
        }
    }
}

pub(crate) struct ActiveLeaseScope {
    leases: Arc<Mutex<BTreeMap<harp_contracts::AttemptId, harp_state::LeaseToken>>>,
    attempt_id: harp_contracts::AttemptId,
}

impl ActiveLeaseScope {
    pub(crate) fn register(
        leases: Arc<Mutex<BTreeMap<harp_contracts::AttemptId, harp_state::LeaseToken>>>,
        lease: harp_state::LeaseToken,
    ) -> Self {
        let attempt_id = lease.attempt_id().clone();
        leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(attempt_id.clone(), lease);
        Self { leases, attempt_id }
    }
}

impl Drop for ActiveLeaseScope {
    fn drop(&mut self) {
        self.leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&self.attempt_id);
    }
}

pub(crate) fn semantic_output_error(error: &EngineError) -> bool {
    matches!(
        error,
        EngineError::InvalidResult { .. }
            | EngineError::OutputJson { .. }
            | EngineError::OutputSchema { .. }
            | EngineError::Contract(_)
            | EngineError::IncompleteTurn { .. }
    )
}

pub(crate) fn semantic_failure_class(error: &EngineError) -> &'static str {
    match error {
        EngineError::OutputJson { .. } => "output_json",
        EngineError::OutputSchema { .. } => "output_schema",
        EngineError::InvalidResult { .. } => "invalid_result",
        EngineError::IncompleteTurn { .. } => "incomplete_turn",
        EngineError::Contract(_) => "output_contract",
        _ => "semantic_output",
    }
}

pub(crate) fn scanner_policy_error(error: &EngineError) -> bool {
    matches!(
        error,
        EngineError::Artifact(harp_artifacts::ArtifactError::StoragePolicyViolation { .. })
    )
}

fn scanner_interrupt_intent(error: &EngineError) -> Result<InterruptIntent, EngineError> {
    match error {
        EngineError::Artifact(harp_artifacts::ArtifactError::StoragePolicyViolation {
            observed_bytes: Some(_),
            ..
        }) => Ok(InterruptIntent::budget(
            InterruptBudgetDimension::Storage,
            "storage_limit",
        )?),
        EngineError::Artifact(harp_artifacts::ArtifactError::StoragePolicyViolation {
            observed_bytes: None,
            ..
        }) => Ok(InterruptIntent::scanner_integrity("storage_integrity")?),
        _ => Err(EngineError::ExecutionReceipt {
            context: "non-scanner error cannot define interrupt intent".to_owned(),
        }),
    }
}

fn terminalize_interrupt_intent(
    state: &mut StateStore,
    lease: &harp_state::LeaseToken,
    intent: &InterruptIntent,
    now: i64,
) -> Result<(), EngineError> {
    match intent.purpose {
        harp_state::InterruptPurpose::Budget => {
            state.fail_terminal_budget_exhausted(lease, &intent.reason_code, now)?;
        }
        harp_state::InterruptPurpose::ScannerIntegrity => {
            state.fail_terminal_semantic(lease, &intent.reason_code, now)?;
        }
        harp_state::InterruptPurpose::Cancellation => {
            return Err(EngineError::ExecutionReceipt {
                context: "scheduler limit path cannot terminalize cancellation intent".to_owned(),
            });
        }
    }
    Ok(())
}

fn scanner_observed_storage(error: &EngineError, configured_max: u64) -> u64 {
    match error {
        EngineError::Artifact(harp_artifacts::ArtifactError::StoragePolicyViolation {
            observed_bytes,
            ..
        }) => observed_bytes
            .unwrap_or_else(|| u128::from(configured_max) + 1)
            .min(u128::from(i64::MAX as u64)) as u64,
        _ => u128::from(configured_max)
            .saturating_add(1)
            .min(u128::from(i64::MAX as u64)) as u64,
    }
}

fn usage_interrupt_intent(
    observed_tokens: u64,
    observed_storage: u64,
    claim: &harp_state::TaskClaim,
    token_outcome: &UsageOutcome,
    storage_outcome: &UsageOutcome,
    wall: &harp_state::WallUsageOutcome,
) -> Result<InterruptIntent, EngineError> {
    if wall.attempt_limit_exceeded || wall.run_limit_exceeded {
        return Ok(InterruptIntent::budget(
            InterruptBudgetDimension::Wall,
            "wall_limit",
        )?);
    }
    let aggregate_tokens_exceeded = matches!(
        token_outcome,
        UsageOutcome::Exceeded {
            max_tokens,
            observed_tokens,
            ..
        } if observed_tokens > &u128::from(*max_tokens)
    );
    if observed_tokens > claim.budget.max_tokens || aggregate_tokens_exceeded {
        return Ok(InterruptIntent::budget(
            InterruptBudgetDimension::Tokens,
            "token_limit",
        )?);
    }
    let aggregate_storage_exceeded = matches!(
        storage_outcome,
        UsageOutcome::Exceeded {
            max_storage_bytes,
            observed_storage_bytes,
            ..
        } if observed_storage_bytes > &u128::from(*max_storage_bytes)
    );
    if observed_storage > claim.budget.max_storage_bytes || aggregate_storage_exceeded {
        return Ok(InterruptIntent::budget(
            InterruptBudgetDimension::Storage,
            "storage_limit",
        )?);
    }
    Err(EngineError::ExecutionReceipt {
        context: "usage limit branch had no exceeded dimension".to_owned(),
    })
}

fn charged_monotonic_seconds(initial: u64, elapsed: Duration) -> Result<u64, EngineError> {
    let elapsed_seconds = elapsed.as_secs();
    let rounded_seconds = if elapsed.subsec_nanos() == 0 {
        elapsed_seconds
    } else {
        elapsed_seconds
            .checked_add(1)
            .ok_or(EngineError::UsageOverflow)?
    };
    initial
        .checked_add(rounded_seconds)
        .ok_or(EngineError::UsageOverflow)
}

fn elapsed_epoch_seconds(started_at: i64, now: i64) -> Result<u64, EngineError> {
    u64::try_from(now.saturating_sub(started_at).max(0)).map_err(|_| EngineError::WallClock)
}

fn publish_evaluation_receipt(
    artifacts: &ArtifactStore,
    task_id: &TaskId,
    result_sha256: &str,
) -> Result<ArtifactRef, EngineError> {
    let evaluation_bytes = serde_json::to_vec(&json!({
        "schemaVersion": 1,
        "taskId": task_id,
        "resultSha256": result_sha256,
        "outcome": "deferred",
    }))
    .map_err(|source| EngineError::Serialization {
        context: "evaluation receipt",
        source,
    })?;
    artifacts
        .publish(&evaluation_bytes, EVALUATION_MEDIA_TYPE)
        .map_err(EngineError::from)
}

pub(crate) fn thread_spec(
    node: &harp_contracts::TaskNode,
    spec: &RunExecutionSpec,
    resolved_scratch: &harp_artifacts::ResolvedAttemptScratch,
) -> Result<ThreadSpec, EngineError> {
    let scratch_path = resolved_scratch
        .canonical_path()
        .to_str()
        .ok_or_else(|| EngineError::ExecutionReceipt {
            context: "resolved scratch path is not UTF-8".to_owned(),
        })?
        .to_owned();
    let thread = ThreadSpec {
        base_instructions: spec.receipt.projection_policy.base_instructions.clone(),
        developer_instructions: spec
            .receipt
            .projection_policy
            .role_instructions
            .get(crate::role_name(node.role))
            .cloned()
            .unwrap_or_default(),
        cwd: scratch_path.clone(),
        runtime_workspace_roots: vec![scratch_path],
        workspace_authority: Some(resolved_scratch.runtime_authority()?),
        approval_policy: node.permission_profile.clone(),
        sandbox_mode: crate::workspace_mode_name(node.workspace_mode).to_owned(),
        model: node.model_policy.clone(),
        reasoning_effort: None,
        ephemeral: false,
    };
    thread.validate().map_err(EngineError::Contract)?;
    Ok(thread)
}

pub(crate) fn turn_spec(
    spec: &RunExecutionSpec,
    claim: &harp_state::TaskClaim,
    operation_id: &OperationId,
    resolved_scratch: &harp_artifacts::ResolvedAttemptScratch,
) -> Result<TurnSpec, EngineError> {
    let attempt_key = harp_artifacts::AttemptKey {
        run_id: claim.run_id.clone(),
        task_id: claim.task_id.clone(),
        attempt_id: claim.attempt_id.clone(),
    };
    let projected = project_execution_child_context(
        &spec.validated,
        ProjectionRequest {
            task_id: &claim.task_id,
            operation_id,
            remaining_budget: claim.budget.clone(),
        },
        &attempt_key,
        resolved_scratch,
        &spec.receipt.artifact_store_identity,
    )?;
    let instruction =
        String::from_utf8(projected.to_bounded_json()?.as_bytes().to_vec()).map_err(|_| {
            EngineError::ExecutionReceipt {
                context: "projected child context is not UTF-8".to_owned(),
            }
        })?;
    let node = spec
        .validated
        .graph()
        .nodes
        .iter()
        .find(|node| node.task_id == claim.task_id)
        .ok_or_else(|| EngineError::MissingTask {
            task_id: claim.task_id.clone(),
        })?;
    let output_schema =
        serde_json::from_str(&node.output_schema).map_err(|source| EngineError::Serialization {
            context: "task output schema",
            source,
        })?;
    let turn = TurnSpec {
        instruction,
        operation_marker: operation_id.clone(),
        output_schema,
        model: Some(node.model_policy.clone()),
        reasoning_effort: None,
    };
    turn.validate().map_err(EngineError::Contract)?;
    Ok(turn)
}

pub(crate) fn activity_invocation_digest(turn: &TurnSpec) -> Result<String, EngineError> {
    let bytes = serde_json::to_vec(turn).map_err(|source| EngineError::Serialization {
        context: "activity invocation",
        source,
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub(crate) fn resolve_and_bind_scratch(
    state: &mut StateStore,
    artifacts: &ArtifactStore,
    claim: &harp_state::TaskClaim,
    lease: &harp_state::LeaseToken,
    now: i64,
) -> Result<harp_artifacts::ResolvedAttemptScratch, EngineError> {
    let key = harp_artifacts::AttemptKey {
        run_id: claim.run_id.clone(),
        task_id: claim.task_id.clone(),
        attempt_id: claim.attempt_id.clone(),
    };
    let scratch = artifacts.resolve_attempt_scratch(&key)?;
    state.bind_attempt_scratch(
        lease,
        scratch.canonical_path(),
        scratch.device(),
        scratch.inode(),
        now,
    )?;
    Ok(scratch)
}

pub(crate) fn verify_and_register_result_artifacts(
    state: &mut StateStore,
    store: &ArtifactStore,
    result: &ResultEnvelope,
    now: i64,
) -> Result<(), EngineError> {
    let mut artifacts = Vec::new();
    if let Some(answer) = &result.answer_ref {
        artifacts.push(answer);
    }
    artifacts.extend(result.evidence.iter());
    artifacts.push(&result.trace_ref);
    for artifact in artifacts {
        store.read_verified(artifact)?;
        state.register_artifact(artifact, now)?;
    }
    Ok(())
}

fn result_storage_bytes(
    artifacts: &ArtifactStore,
    claim: &harp_state::TaskClaim,
    result: &ResultEnvelope,
) -> Result<u64, EngineError> {
    let encoded = serde_json::to_vec(result).map_err(|source| EngineError::Serialization {
        context: "result storage accounting",
        source,
    })?;
    let mut total = u64::try_from(encoded.len()).map_err(|_| EngineError::UsageOverflow)?;
    total = total
        .checked_add(logical_nested_artifact_bytes(artifacts, result)?)
        .ok_or(EngineError::UsageOverflow)?;
    total
        .checked_add(attempt_file_storage(artifacts, claim)?)
        .ok_or(EngineError::UsageOverflow)
}

fn logical_nested_artifact_bytes(
    artifacts: &ArtifactStore,
    result: &ResultEnvelope,
) -> Result<u64, EngineError> {
    // Writable storage is logical-unique within one attempt: repeated
    // references to the same digest are charged once here. Each attempt calls
    // this independently, so retries and indeterminate duplicate work charge
    // the same digest again as intentional observed cost.
    let mut unique = BTreeSet::new();
    let mut total = 0u64;
    for artifact in result
        .answer_ref
        .iter()
        .chain(result.evidence.iter())
        .chain(std::iter::once(&result.trace_ref))
    {
        if unique.insert(artifact.sha256.as_str()) {
            artifacts.read_verified(artifact)?;
            total = total
                .checked_add(artifact.size_bytes)
                .ok_or(EngineError::UsageOverflow)?;
        }
    }
    Ok(total)
}

fn published_result_storage(
    state: &mut StateStore,
    artifacts: &ArtifactStore,
    claim: &harp_state::TaskClaim,
    result_sha256: &str,
) -> Result<u64, EngineError> {
    let result_ref =
        state
            .artifact(result_sha256)?
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "published result artifact metadata is missing".to_owned(),
            })?;
    let result_bytes = artifacts.read_verified(&result_ref)?;
    let result: ResultEnvelope =
        serde_json::from_slice(&result_bytes).map_err(|source| EngineError::Serialization {
            context: "published result storage envelope",
            source,
        })?;
    result_storage_bytes(artifacts, claim, &result)
}

fn attempt_file_storage(
    artifacts: &ArtifactStore,
    claim: &harp_state::TaskClaim,
) -> Result<u64, EngineError> {
    let scan_limit = claim
        .budget
        .max_storage_bytes
        .saturating_add(1)
        .min(harp_artifacts::MAX_ATTEMPT_SCAN_BYTES);
    let logical_bytes = artifacts
        .attempt_storage_usage(
            &harp_artifacts::AttemptKey {
                run_id: claim.run_id.clone(),
                task_id: claim.task_id.clone(),
                attempt_id: claim.attempt_id.clone(),
            },
            scan_limit,
        )?
        .total_bytes()?;
    u64::try_from(logical_bytes).map_err(|_| EngineError::UsageOverflow)
}

pub(crate) fn pinned_output_schema(
    state: &mut StateStore,
    attempt_id: &harp_contracts::AttemptId,
) -> Result<serde_json::Value, EngineError> {
    let attempt = state
        .get_attempt(attempt_id)?
        .ok_or_else(|| EngineError::ExecutionReceipt {
            context: "attempt is missing while loading pinned output schema".to_owned(),
        })?;
    if state.get_cli_attempt(attempt_id)?.is_some() {
        let node = state
            .task_node(&attempt.run_id, &attempt.task_id)?
            .ok_or_else(|| EngineError::MissingTask {
                task_id: attempt.task_id,
            })?;
        return serde_json::from_str(&node.output_schema).map_err(|source| {
            EngineError::Serialization {
                context: "CLI output schema",
                source,
            }
        });
    }
    let marker = attempt.latest_operation_marker.as_deref().ok_or_else(|| {
        EngineError::ExecutionReceipt {
            context: "attempt has no latest operation marker for output schema".to_owned(),
        }
    })?;
    let operation =
        state
            .get_operation_by_marker(marker)?
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "latest operation marker has no operation row".to_owned(),
            })?;
    operation
        .turn_intent
        .map(|intent| intent.output_schema)
        .ok_or_else(|| EngineError::ExecutionReceipt {
            context: "latest turn operation has no pinned output schema".to_owned(),
        })
}

pub(crate) fn summarize(state: &mut StateStore, run_id: &RunId) -> Result<RunSummary, EngineError> {
    let tasks = state.tasks(run_id)?;
    let attempts = state.attempts(run_id)?;
    Ok(RunSummary {
        run_id: run_id.clone(),
        completed_tasks: tasks
            .iter()
            .filter(|task| task.state == harp_state::TaskState::Completed)
            .count(),
        indeterminate_attempts: attempts
            .iter()
            .filter(|attempt| attempt.state == AttemptState::Indeterminate)
            .count(),
    })
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("invalid engine configuration: {context}")]
    InvalidConfig { context: String },
    #[error("execution receipt failure: {context}")]
    ExecutionReceipt { context: String },
    #[error("run {run_id} does not exist")]
    MissingRun { run_id: RunId },
    #[error("task {task_id} does not exist")]
    MissingTask { task_id: TaskId },
    #[error("task {task_id} produced an invalid result: {context}")]
    InvalidResult { task_id: TaskId, context: String },
    #[error("task {task_id} output JSON is invalid")]
    OutputJson {
        task_id: TaskId,
        #[source]
        source: serde_json::Error,
    },
    #[error("task {task_id} output does not match its pinned schema: {context}")]
    OutputSchema { task_id: TaskId, context: String },
    #[error("task {task_id} ended without a complete result")]
    IncompleteTurn { task_id: TaskId },
    #[error("task {task_id} cannot continue without a valid matching checkpoint")]
    MissingCheckpoint { task_id: TaskId },
    #[error("task {task_id} exceeded a hard budget")]
    BudgetExceeded { task_id: TaskId },
    #[error("injected abrupt crash at {point:?}")]
    InjectedCrash { point: CrashPoint },
    #[error("logical clock overflowed")]
    ClockOverflow,
    #[error("wall clock could not produce a bounded Unix timestamp")]
    WallClock,
    #[error("runtime operation exceeded deadline {deadline:?}")]
    RuntimeOperationTimeout { deadline: Duration },
    #[error("lease authority for attempt {attempt_id} was transferred to the control plane")]
    LeaseAuthorityTransferred {
        attempt_id: harp_contracts::AttemptId,
    },
    #[error("task {task_id} wall clock integrity failure: {context}")]
    WallClockIntegrity { task_id: TaskId, context: String },
    #[error("task {task_id} recovered durable integrity interrupt {reason_code}")]
    RecoveredIntegrityInterrupt {
        task_id: TaskId,
        reason_code: String,
    },
    #[error("token usage overflowed")]
    UsageOverflow,
    #[error("scheduler semaphore closed")]
    SemaphoreClosed,
    #[error("could not serialize {context}")]
    Serialization {
        context: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Validation(#[from] crate::ValidationError),
    #[error(transparent)]
    Projection(#[from] crate::ProjectionError),
    #[error(transparent)]
    State(#[from] harp_state::StateError),
    #[error(transparent)]
    Runtime(#[from] harp_runtime::RuntimeError),
    #[error(transparent)]
    Artifact(#[from] harp_artifacts::ArtifactError),
    #[error(transparent)]
    Contract(#[from] harp_contracts::ContractError),
}

#[cfg(all(test, unix))]
mod accounting_tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::str::FromStr;

    use harp_contracts::{ResultStatus, TaskId};

    use super::*;

    #[test]
    fn nested_digest_is_unique_within_attempt_and_charged_again_across_attempts() {
        let directory = tempfile::Builder::new()
            .prefix("harp-logical-storage-")
            .tempdir_in("/private/tmp")
            .unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let artifacts = ArtifactStore::open(directory.path()).unwrap();
        let shared = artifacts
            .publish(b"shared", "application/octet-stream")
            .unwrap();
        let result = ResultEnvelope {
            schema_version: 1,
            task_id: TaskId::from_str("alpha").unwrap(),
            status: ResultStatus::Success,
            answer_ref: Some(shared.clone()),
            evidence: vec![shared.clone(), shared.clone()],
            trace_ref: shared.clone(),
            summary: "shared refs".to_owned(),
            token_usage: 1,
            confidence: None,
            failure_class: None,
        };

        let one_attempt = logical_nested_artifact_bytes(&artifacts, &result).unwrap();
        assert_eq!(one_attempt, shared.size_bytes);
        assert_eq!(
            one_attempt.checked_add(one_attempt).unwrap(),
            shared.size_bytes * 2
        );
    }
}
