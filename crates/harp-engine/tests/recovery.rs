#![cfg(unix)]

mod common;

use std::collections::{BTreeMap, VecDeque};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use common::{FakeOutcome, PersistentFakeBackend};
use harp_artifacts::ArtifactStore;
use harp_contracts::{
    Budget, Checkpoint, NodeKind, ResultEnvelope, ResultStatus, RetryPolicy, RunId, TaskGraph,
    TaskId, TaskNode, TaskRole, TurnStatus, WorkspaceMode,
};
use harp_engine::{
    decode_result_envelope, recovery_action, validate_graph, CrashPoint, Engine, EngineConfig,
    EngineError, GraphPolicy, ProjectionPolicy, RecoveryAction, RunExecutionSpec, WallClock,
};
use harp_runtime::{ActivitySpec, CodexRuntime};
use harp_state::{AttemptRecord, AttemptState, RunState, StateStore};
use sha2::Digest;
use tempfile::TempDir;

#[derive(Debug, Eq, PartialEq)]
struct NormalizedRun {
    graph_sha256: String,
    run_state: RunState,
    tasks: Vec<(String, String, Option<u32>, Option<String>)>,
    attempts: Vec<NormalizedAttempt>,
    operations: Vec<NormalizedOperation>,
    events: Vec<(Option<String>, Option<u32>, String)>,
    execution_provenance: serde_json::Value,
}

#[derive(Debug)]
struct TestWallClock {
    now: AtomicI64,
}

#[derive(Debug)]
struct TestLeaseClock {
    now: AtomicI64,
}

#[derive(Debug)]
struct TokioLeaseClock {
    base: i64,
    started: tokio::time::Instant,
}

impl TokioLeaseClock {
    fn new(base: i64) -> Self {
        Self {
            base,
            started: tokio::time::Instant::now(),
        }
    }
}

impl harp_state::LeaseClock for TokioLeaseClock {
    fn unix_seconds(&self) -> harp_state::StateResult<i64> {
        let elapsed = i64::try_from(self.started.elapsed().as_secs()).map_err(|_| {
            harp_state::StateError::Clock {
                context: "test Tokio lease clock overflowed".to_owned(),
                source: None,
            }
        })?;
        self.base
            .checked_add(elapsed)
            .ok_or_else(|| harp_state::StateError::Clock {
                context: "test Tokio lease clock overflowed".to_owned(),
                source: None,
            })
    }
}

impl TestLeaseClock {
    fn new(now: i64) -> Self {
        Self {
            now: AtomicI64::new(now),
        }
    }

    fn set(&self, now: i64) {
        self.now.store(now, Ordering::SeqCst);
    }

    fn advance(&self, seconds: i64) {
        self.now.fetch_add(seconds, Ordering::SeqCst);
    }
}

impl harp_state::LeaseClock for TestLeaseClock {
    fn unix_seconds(&self) -> harp_state::StateResult<i64> {
        Ok(self.now.load(Ordering::SeqCst))
    }
}

impl TestWallClock {
    fn new(now: i64) -> Self {
        Self {
            now: AtomicI64::new(now),
        }
    }

    fn set(&self, now: i64) {
        self.now.store(now, Ordering::SeqCst);
    }
}

impl WallClock for TestWallClock {
    fn unix_seconds(&self) -> Result<i64, EngineError> {
        Ok(self.now.load(Ordering::SeqCst))
    }

    fn monotonic_elapsed(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }
}

#[derive(Debug, Eq, PartialEq)]
struct NormalizedAttempt {
    task_id: String,
    ordinal: u32,
    state: String,
    continuation_count: u64,
    observed_tokens: u64,
    observed_storage_bytes: u64,
    result_sha256: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
struct NormalizedOperation {
    task_id: String,
    attempt_ordinal: u32,
    ordinal: u32,
    kind: String,
    state: String,
    has_marker: bool,
    marker_matches_operation: bool,
    external_matches_thread_or_turn: bool,
    target_matches_latest_turn: bool,
    has_intent: bool,
    checkpoint_sha256: Option<String>,
}

fn private_directory(prefix: &str) -> TempDir {
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in("/private/tmp")
        .expect("temporary directory");
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .expect("private directory");
    directory
}

fn task(task_id: &str, kind: NodeKind, dependencies: &[&str]) -> TaskNode {
    TaskNode {
        task_id: TaskId::from_str(task_id).unwrap(),
        kind,
        role: if kind == NodeKind::Reducer {
            TaskRole::Reduce
        } else {
            TaskRole::Explore
        },
        instruction: format!("execute {task_id}"),
        dependencies: dependencies
            .iter()
            .map(|dependency| TaskId::from_str(dependency).unwrap())
            .collect(),
        inputs: Vec::new(),
        workspace_mode: WorkspaceMode::Scratch,
        model_policy: "test-model".to_owned(),
        permission_profile: "never".to_owned(),
        budget: Budget::new(100, 60, 1_024),
        output_schema: r#"{"type":"object"}"#.to_owned(),
        retry_policy: RetryPolicy {
            max_transient_attempts: 1,
        },
    }
}

fn graph() -> TaskGraph {
    TaskGraph {
        schema_version: 1,
        nodes: vec![
            task("alpha", NodeKind::Analysis, &[]),
            task("beta", NodeKind::Analysis, &[]),
            task("reduce", NodeKind::Reducer, &["alpha", "beta"]),
        ],
    }
}

fn policies(graph: &TaskGraph) -> (GraphPolicy, ProjectionPolicy) {
    let graph_policy = GraphPolicy {
        max_nodes: 64,
        max_concurrency: 2,
        max_total_tokens: 1_000,
        max_total_storage_bytes: 10_000,
        max_total_timeout_seconds: 3_600,
        max_node_tokens: 100,
        max_node_storage_bytes: 1_024,
        max_node_timeout_seconds: 60,
        max_projected_prompt_bytes: 1024 * 1024,
        max_recursion_depth: 1,
        allowed_roles: ["explore", "reduce"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        allowed_output_schemas: [r#"{"type":"object"}"#]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        allowed_model_policies: ["test-model"].into_iter().map(str::to_owned).collect(),
        allowed_permission_profiles: ["never"].into_iter().map(str::to_owned).collect(),
        allowed_workspace_modes: ["scratch"].into_iter().map(str::to_owned).collect(),
        approved_artifacts: BTreeMap::new(),
    };
    let projection_policy = ProjectionPolicy {
        base_instructions: "Follow the child protocol.".to_owned(),
        role_instructions: [
            ("explore".to_owned(), "Explore evidence.".to_owned()),
            (
                "reduce".to_owned(),
                "Reduce accepted child results.".to_owned(),
            ),
        ]
        .into_iter()
        .collect(),
        checkpoint_instructions: "Write semantic checkpoints.".to_owned(),
        max_bytes: 1024 * 1024,
        scratch_paths: graph
            .nodes
            .iter()
            .map(|node| {
                (
                    node.task_id.clone(),
                    format!("/private/tmp/harp/{}", node.task_id),
                )
            })
            .collect(),
    };
    (graph_policy, projection_policy)
}

fn execution_spec(artifacts: &ArtifactStore, backend: &PersistentFakeBackend) -> RunExecutionSpec {
    let graph = graph();
    let (graph_policy, projection_policy) = policies(&graph);
    let validated = validate_graph(graph, &graph_policy, &projection_policy).unwrap();
    let runtime = backend.runtime();
    RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap()
}

fn result_message(store: &ArtifactStore, task_id: &str, token_usage: u64) -> String {
    let answer_ref = store
        .publish(format!("answer for {task_id}").as_bytes(), "text/plain")
        .unwrap();
    let trace_ref = store
        .publish(
            format!("trace for {task_id}\n").as_bytes(),
            "application/jsonl",
        )
        .unwrap();
    serde_json::to_string(&ResultEnvelope {
        schema_version: 1,
        task_id: TaskId::from_str(task_id).unwrap(),
        status: ResultStatus::Success,
        answer_ref: Some(answer_ref),
        evidence: Vec::new(),
        trace_ref,
        summary: format!("{task_id} completed"),
        token_usage,
        confidence: Some(1.0),
        failure_class: None,
    })
    .unwrap()
}

fn test_activity_invocation_digest(turn: &harp_contracts::TurnSpec) -> String {
    let bytes = serde_json::to_vec(turn).unwrap();
    format!("{:x}", sha2::Sha256::digest(bytes))
}

fn test_continuation_turn_spec(
    task_id: &TaskId,
    checkpoint: &harp_artifacts::CheckpointVersion,
    continuation_id: &harp_contracts::OperationId,
) -> harp_contracts::TurnSpec {
    harp_contracts::TurnSpec {
        instruction: serde_json::json!({
            "schemaVersion": 1,
            "taskId": task_id,
            "continuation": true,
            "checkpoint": {
                "phase": checkpoint.checkpoint.phase,
                "completedUnits": checkpoint.checkpoint.completed_units,
                "pendingUnits": checkpoint.checkpoint.pending_units,
                "evidenceCount": checkpoint.checkpoint.evidence_count,
            },
            "instructions": [
                "Inspect existing thread history and durable scratch state.",
                "Continue from the validated checkpoint without repeating completed analysis.",
                "Retain existing evidence and satisfy the original output schema.",
                "Publish a final result envelope with cumulative token usage."
            ],
        })
        .to_string(),
        operation_marker: continuation_id.clone(),
        output_schema: serde_json::json!({"type": "object"}),
        model: Some("test-model".to_owned()),
        reasoning_effort: None,
    }
}

fn outcomes(store: &ArtifactStore) -> BTreeMap<String, VecDeque<FakeOutcome>> {
    ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(store, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect()
}

fn open_artifacts(directory: &TempDir) -> ArtifactStore {
    let root = directory.path().join("artifacts");
    if !root.exists() {
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    }
    ArtifactStore::open(root).unwrap()
}

fn cli_interrupt_count(
    state_path: &std::path::Path,
    attempt_id: &harp_contracts::AttemptId,
    purpose: &str,
    state: &str,
    signal_stage: &str,
) -> i64 {
    rusqlite::Connection::open(state_path)
        .unwrap()
        .query_row(
            "SELECT COUNT(*)
             FROM cli_activities
             WHERE attempt_id = ?1
               AND kind = 'interrupt_activity'
               AND interrupt_purpose = ?2
               AND state = ?3
               AND signal_stage = ?4",
            rusqlite::params![attempt_id.to_string(), purpose, state, signal_stage,],
            |row| row.get(0),
        )
        .unwrap()
}

fn latest_semantic_activity_id(
    state_path: &std::path::Path,
    attempt_id: &harp_contracts::AttemptId,
) -> harp_contracts::OperationId {
    let activity_id: String = rusqlite::Connection::open(state_path)
        .unwrap()
        .query_row(
            "SELECT activity_id
             FROM cli_activities
             WHERE attempt_id = ?1
               AND kind IN ('start_activity', 'continue_activity')
             ORDER BY updated_at DESC, activity_id DESC
             LIMIT 1",
            [attempt_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    harp_contracts::OperationId::from_str(&activity_id).unwrap()
}

fn config(crash_point: Option<CrashPoint>, logical_time: i64) -> EngineConfig {
    EngineConfig {
        worker_id: "recovery-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 2,
        logical_time,
        crash_point,
    }
}

fn fixed_clock_engine(config: EngineConfig) -> Engine {
    let lease_now = config.logical_time;
    Engine::with_clocks(
        config,
        Arc::new(TestWallClock::new(1_000)),
        Arc::new(TestLeaseClock::new(lease_now)),
    )
    .unwrap()
}

fn fixed_clock_engine_with_lease(config: EngineConfig, lease_clock: Arc<TestLeaseClock>) -> Engine {
    Engine::with_clocks(config, Arc::new(TestWallClock::new(1_000)), lease_clock).unwrap()
}

fn normalize(state: &mut StateStore, run_id: &harp_contracts::RunId) -> NormalizedRun {
    let execution = state.run_execution(run_id).unwrap().unwrap();
    let run = execution.run.clone();
    let attempts_by_id = state
        .attempts(run_id)
        .unwrap()
        .into_iter()
        .map(|attempt| (attempt.attempt_id.clone(), attempt))
        .collect::<BTreeMap<_, _>>();
    let tasks = state
        .tasks(run_id)
        .unwrap()
        .into_iter()
        .map(|task| {
            let accepted = task
                .accepted_attempt_id
                .as_ref()
                .and_then(|attempt_id| attempts_by_id.get(attempt_id));
            (
                task.task_id.to_string(),
                task.state.as_str().to_owned(),
                accepted.map(|attempt| attempt.ordinal),
                accepted.and_then(|attempt| attempt.result_sha256.clone()),
            )
        })
        .collect();
    let attempts = state
        .attempts(run_id)
        .unwrap()
        .into_iter()
        .map(|attempt| {
            let usage = state
                .reservation_usage(&attempt.attempt_id)
                .unwrap()
                .unwrap();
            NormalizedAttempt {
                task_id: attempt.task_id.to_string(),
                ordinal: attempt.ordinal,
                state: attempt.state.as_str().to_owned(),
                continuation_count: attempt.continuation_count,
                observed_tokens: usage.0,
                observed_storage_bytes: usage.1,
                result_sha256: attempt.result_sha256,
            }
        })
        .collect::<Vec<_>>();
    let mut operations = Vec::new();
    for attempt in attempts_by_id.values() {
        operations.extend(
            state
                .operations_for_attempt(&attempt.attempt_id)
                .unwrap()
                .into_iter()
                .map(|operation| NormalizedOperation {
                    task_id: attempt.task_id.to_string(),
                    attempt_ordinal: attempt.ordinal,
                    ordinal: operation.ordinal,
                    kind: operation.kind.as_str().to_owned(),
                    state: operation.state.as_str().to_owned(),
                    has_marker: operation.operation_marker.is_some(),
                    marker_matches_operation: operation.operation_marker.as_deref()
                        == Some(format!("harp-operation:{}", operation.operation_id).as_str()),
                    external_matches_thread_or_turn: operation.external_id.as_deref()
                        == attempt
                            .thread_id
                            .as_ref()
                            .map(ToString::to_string)
                            .as_deref()
                        || operation.external_id.as_deref()
                            == attempt
                                .latest_turn_id
                                .as_ref()
                                .map(ToString::to_string)
                                .as_deref(),
                    target_matches_latest_turn: operation.target_external_id.as_deref()
                        == attempt
                            .latest_turn_id
                            .as_ref()
                            .map(ToString::to_string)
                            .as_deref(),
                    has_intent: operation.intent_sha256.is_some(),
                    checkpoint_sha256: operation.checkpoint_sha256,
                }),
        );
    }
    let mut events = Vec::new();
    let mut after = None;
    loop {
        let page = state.events_page(run_id, after, 1_000).unwrap();
        for event in page.events {
            if matches!(
                event.event_type.as_str(),
                "ready_tasks_rebuilt"
                    | "lease_expired"
                    | "attempt_reclaimed"
                    | "usage_reconciled"
                    | "turn_usage_reconciled"
            ) {
                continue;
            }
            events.push((
                event.task_id.map(|task_id| task_id.to_string()),
                event
                    .attempt_id
                    .as_ref()
                    .and_then(|attempt_id| attempts_by_id.get(attempt_id))
                    .map(|attempt| attempt.ordinal),
                event.event_type,
            ));
        }
        let Some(next) = page.next_after_sequence else {
            break;
        };
        after = Some(next);
    }
    NormalizedRun {
        graph_sha256: run.graph_sha256,
        run_state: run.state,
        tasks,
        attempts,
        operations,
        events,
        execution_provenance: normalize_provenance(execution.provenance),
    }
}

fn normalize_provenance(mut provenance: serde_json::Value) -> serde_json::Value {
    if let Some(receipt) = provenance
        .get_mut("executionSpec")
        .and_then(|value| value.get_mut("receipt"))
    {
        if let Some(identity) = receipt.get_mut("artifactStoreIdentity") {
            identity["canonicalRoot"] = serde_json::json!("<artifact-root>");
            identity["device"] = serde_json::json!(0);
            identity["inode"] = serde_json::json!(0);
        }
        if let Some(runtime) = receipt.get_mut("runtimeProvenance") {
            runtime["persistentStateIdentity"] = serde_json::json!("<runtime-state>");
        }
    }
    if let Some(spec) = provenance.get_mut("executionSpec") {
        spec["receiptSha256"] = serde_json::json!("<receipt-digest>");
    }
    provenance
}

fn crash_point_name(point: CrashPoint) -> &'static str {
    match point {
        CrashPoint::BeforeClaimCommit => "before_claim",
        CrashPoint::AfterClaim => "after_claim",
        CrashPoint::AfterPrepared => "after_prepared",
        CrashPoint::AfterDispatchingThread => "after_dispatching_thread",
        CrashPoint::AfterThreadId => "after_thread_id",
        CrashPoint::AfterDispatchingTurn => "after_dispatching_turn",
        CrashPoint::DuringTurn => "during_turn",
        CrashPoint::AfterArtifactPublication => "after_artifact_publication",
        CrashPoint::AfterResultPublished => "after_result_published",
        CrashPoint::BeforeReducerReady => "before_reducer_ready",
        CrashPoint::DuringReducer => "during_reducer",
        CrashPoint::AfterEvaluationPublication => "after_evaluation_publication",
    }
}

fn parse_crash_point(value: &str) -> CrashPoint {
    [
        CrashPoint::BeforeClaimCommit,
        CrashPoint::AfterClaim,
        CrashPoint::AfterPrepared,
        CrashPoint::AfterDispatchingThread,
        CrashPoint::AfterDispatchingTurn,
        CrashPoint::DuringTurn,
        CrashPoint::AfterArtifactPublication,
        CrashPoint::AfterResultPublished,
        CrashPoint::BeforeReducerReady,
        CrashPoint::DuringReducer,
        CrashPoint::AfterEvaluationPublication,
    ]
    .into_iter()
    .find(|point| crash_point_name(*point) == value)
    .expect("known crash point")
}

#[test]
fn abrupt_crash_worker_entrypoint() {
    let Some(root) = env::var_os("HARP_ENGINE_CRASH_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let crash_point = parse_crash_point(&env::var("HARP_ENGINE_CRASH_POINT").unwrap());
    let artifact_root = root.join("artifacts");
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let sidecar = root.join("fake-runtime.json");
    let backend = PersistentFakeBackend::open_file(&sidecar, outcomes(&artifacts)).unwrap();
    let mut runtime = backend.runtime();
    let spec = execution_spec(&artifacts, &backend);
    let mut state = StateStore::open(&root.join("state.sqlite")).unwrap();
    let mut engine = fixed_clock_engine(config(Some(crash_point), 10));
    let tokio = tokio::runtime::Runtime::new().unwrap();
    tokio.block_on(async {
        let _ = engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await;
    });
    panic!("crash point was not reached");
}

async fn reference_state() -> NormalizedRun {
    let directory = private_directory("harp-reference-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let mut runtime = backend.runtime();
    let mut state = StateStore::open(&directory.path().join("state.sqlite")).unwrap();
    let mut engine = fixed_clock_engine(config(None, 10));
    let summary = engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .unwrap();
    normalize(&mut state, &summary.run_id)
}

#[test]
fn recovery_mapping_matches_normative_attempt_states() {
    fn attempt(state: AttemptState) -> AttemptRecord {
        AttemptRecord {
            attempt_id: harp_contracts::AttemptId::new(),
            run_id: harp_contracts::RunId::new(),
            task_id: TaskId::from_str("task").unwrap(),
            ordinal: 0,
            state,
            lease_owner: None,
            lease_expires_at: None,
            last_lease_expires_at: None,
            thread_id: None,
            latest_turn_id: None,
            latest_operation_marker: None,
            scratch_path: None,
            scratch_device: None,
            scratch_inode: None,
            wall_started_at: None,
            wall_last_observed_at: None,
            observed_wall_seconds: 0,
            semantic_failure_class: None,
            continuation_count: 0,
            observed_tokens: 0,
            latest_turn_observed_tokens: 0,
            result_sha256: None,
            result_status: None,
            result_token_usage: None,
        }
    }

    assert_eq!(
        recovery_action(&attempt(AttemptState::Prepared)),
        RecoveryAction::DispatchThread
    );
    assert_eq!(
        recovery_action(&attempt(AttemptState::DispatchingThread)),
        RecoveryAction::MarkIndeterminateAndRetry
    );
    let mut thread_started = attempt(AttemptState::ThreadStarted);
    thread_started.thread_id = Some(harp_contracts::ThreadId::from_str("thread-known").unwrap());
    assert_eq!(recovery_action(&thread_started), RecoveryAction::StartTurn);
    assert_eq!(
        recovery_action(&attempt(AttemptState::DispatchingTurn)),
        RecoveryAction::ReadThreadForMarker
    );
    assert_eq!(
        recovery_action(&attempt(AttemptState::TurnStarted)),
        RecoveryAction::ReadThreadForTurn
    );
    assert_eq!(
        recovery_action(&attempt(AttemptState::ResultPublished)),
        RecoveryAction::AcceptPublishedResult
    );
    for terminal in [
        AttemptState::Succeeded,
        AttemptState::Failed,
        AttemptState::Indeterminate,
        AttemptState::Cancelled,
    ] {
        assert_eq!(recovery_action(&attempt(terminal)), RecoveryAction::None);
    }
}

#[tokio::test]
async fn crash_restart_matrix_converges_to_reference_results() {
    let reference = reference_state().await;
    for crash_point in [
        CrashPoint::BeforeClaimCommit,
        CrashPoint::AfterClaim,
        CrashPoint::AfterPrepared,
        CrashPoint::AfterDispatchingThread,
        CrashPoint::AfterDispatchingTurn,
        CrashPoint::DuringTurn,
        CrashPoint::AfterArtifactPublication,
        CrashPoint::AfterResultPublished,
        CrashPoint::BeforeReducerReady,
        CrashPoint::DuringReducer,
        CrashPoint::AfterEvaluationPublication,
    ] {
        let directory = private_directory("harp-recovery-");
        let artifacts = open_artifacts(&directory);
        let backend = PersistentFakeBackend::new(outcomes(&artifacts));
        let state_path = directory.path().join("state.sqlite");
        let run_id = {
            let mut state = StateStore::open(&state_path).unwrap();
            let mut runtime = backend.runtime();
            let mut engine = fixed_clock_engine(config(Some(crash_point), 10));
            let error = engine
                .execute_run(
                    &mut state,
                    &artifacts,
                    &mut runtime,
                    &execution_spec(&artifacts, &backend),
                )
                .await
                .expect_err("injected crash");
            assert!(
                matches!(error, harp_engine::EngineError::InjectedCrash { point } if point == crash_point),
                "{crash_point:?}: {error}"
            );
            state.incomplete_runs().unwrap()[0].run_id.clone()
        };

        let mut reopened = StateStore::open(&state_path).unwrap();
        let reopened_artifacts = open_artifacts(&directory);
        let mut resumed_runtime = backend.runtime();
        let mut resumed_engine = fixed_clock_engine(config(None, 1_000));
        resumed_engine
            .resume_run(
                &mut reopened,
                &reopened_artifacts,
                &mut resumed_runtime,
                &run_id,
            )
            .await
            .unwrap_or_else(|error| panic!("{crash_point:?}: {error}"));
        let normalized = normalize(&mut reopened, &run_id);
        assert_eq!(normalized.run_state, RunState::Completed, "{crash_point:?}");

        if crash_point == CrashPoint::AfterDispatchingThread {
            assert_eq!(
                normalized
                    .tasks
                    .iter()
                    .map(|(task, state, _, digest)| (task, state, digest))
                    .collect::<Vec<_>>(),
                reference
                    .tasks
                    .iter()
                    .map(|(task, state, _, digest)| (task, state, digest))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                normalized
                    .attempts
                    .iter()
                    .filter(|attempt| attempt.state == "indeterminate")
                    .count(),
                1
            );
            assert_eq!(backend.start_thread_calls(), 0);
            assert_eq!(backend.start_turn_calls(), 0);
            assert_eq!(backend.start_activity_calls(), 3);
        } else {
            assert_eq!(normalized.tasks, reference.tasks, "{crash_point:?}");
            assert_eq!(normalized.attempts, reference.attempts, "{crash_point:?}");
            assert_eq!(
                normalized.operations, reference.operations,
                "{crash_point:?}"
            );
            assert_eq!(backend.start_thread_calls(), 0, "{crash_point:?}");
            assert_eq!(backend.start_turn_calls(), 0, "{crash_point:?}");
            assert_eq!(backend.start_activity_calls(), 3, "{crash_point:?}");
        }
    }
}

#[test]
fn abrupt_subprocess_crash_matrix_reopens_all_durable_authorities() {
    let reference = {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(reference_state())
    };
    for crash_point in [
        CrashPoint::BeforeClaimCommit,
        CrashPoint::AfterClaim,
        CrashPoint::AfterPrepared,
        CrashPoint::AfterDispatchingThread,
        CrashPoint::AfterDispatchingTurn,
        CrashPoint::DuringTurn,
        CrashPoint::AfterArtifactPublication,
        CrashPoint::AfterResultPublished,
        CrashPoint::BeforeReducerReady,
        CrashPoint::DuringReducer,
        CrashPoint::AfterEvaluationPublication,
    ] {
        let directory = private_directory("harp-subprocess-recovery-");
        let artifact_root = directory.path().join("artifacts");
        fs::create_dir(&artifact_root).unwrap();
        fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
        let artifacts = ArtifactStore::open(&artifact_root).unwrap();
        for task_id in ["alpha", "beta", "reduce"] {
            let _ = result_message(&artifacts, task_id, 8);
        }
        drop(artifacts);

        let status = Command::new(env::current_exe().unwrap())
            .arg("--exact")
            .arg("abrupt_crash_worker_entrypoint")
            .arg("--nocapture")
            .env("HARP_ENGINE_CRASH_ROOT", directory.path())
            .env("HARP_ENGINE_CRASH_POINT", crash_point_name(crash_point))
            .env("HARP_ENGINE_TEST_ABRUPT_EXIT", "1")
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(86), "{crash_point:?}");

        let state_path = directory.path().join("state.sqlite");
        let mut state = StateStore::open(&state_path).unwrap();
        let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
        let artifacts = ArtifactStore::open(&artifact_root).unwrap();
        let sidecar = directory.path().join("fake-runtime.json");
        let backend = PersistentFakeBackend::open_file(&sidecar, outcomes(&artifacts)).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(None, 10_000));
        let tokio = tokio::runtime::Runtime::new().unwrap();
        tokio
            .block_on(engine.resume_run(&mut state, &artifacts, &mut runtime, &run_id))
            .unwrap_or_else(|error| panic!("{crash_point:?}: {error}"));
        let normalized = normalize(&mut state, &run_id);
        assert_eq!(normalized.graph_sha256, reference.graph_sha256);
        assert_eq!(normalized.run_state, RunState::Completed);
        assert_eq!(
            normalized.execution_provenance,
            reference.execution_provenance
        );
        if crash_point == CrashPoint::AfterDispatchingThread {
            assert_eq!(
                normalized
                    .tasks
                    .iter()
                    .map(|(task, state, _, digest)| (task, state, digest))
                    .collect::<Vec<_>>(),
                reference
                    .tasks
                    .iter()
                    .map(|(task, state, _, digest)| (task, state, digest))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                normalized
                    .attempts
                    .iter()
                    .filter(|attempt| attempt.state == "indeterminate")
                    .count(),
                1
            );
            assert!(normalized
                .events
                .iter()
                .any(|(_, _, event)| event == "attempt_indeterminate"));
        } else {
            assert_eq!(normalized.tasks, reference.tasks, "{crash_point:?}");
            assert_eq!(normalized.attempts, reference.attempts, "{crash_point:?}");
            assert_eq!(
                normalized.operations, reference.operations,
                "{crash_point:?}"
            );
            assert_eq!(normalized.events, reference.events, "{crash_point:?}");
        }
        for accepted in state.accepted_results(&run_id).unwrap() {
            let digest = accepted.attempt.result_sha256.unwrap();
            let artifact = state.artifact(&digest).unwrap().unwrap();
            let result_bytes = artifacts.read_verified(&artifact).unwrap();
            let result: ResultEnvelope = serde_json::from_slice(&result_bytes).unwrap();
            for nested in result
                .answer_ref
                .iter()
                .chain(result.evidence.iter())
                .chain(std::iter::once(&result.trace_ref))
            {
                artifacts.read_verified(nested).unwrap();
            }
        }
    }
}

#[tokio::test]
async fn terminal_incomplete_turn_continues_once_from_matching_checkpoint() {
    let directory = private_directory("harp-continuation-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.insert(
        "alpha".to_owned(),
        VecDeque::from([
            FakeOutcome {
                status: TurnStatus::Interrupted,
                final_message: None,
                total_tokens: 8,
                token_snapshots: Vec::new(),
            },
            FakeOutcome {
                status: TurnStatus::Completed,
                final_message: Some(result_message(&artifacts, "alpha", 13)),
                total_tokens: 5,
                token_snapshots: Vec::new(),
            },
        ]),
    );
    let backend = PersistentFakeBackend::new(scripted);
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(None, 10));
        let error = engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("interrupted turn is not silently completed");
        assert!(matches!(
            error,
            harp_engine::EngineError::IncompleteTurn { .. }
        ));
        let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
        let alpha = state
            .attempts(&run_id)
            .unwrap()
            .into_iter()
            .find(|attempt| attempt.task_id.to_string() == "alpha")
            .unwrap();
        let mut checkpoint = Checkpoint::new(
            run_id.clone(),
            alpha.task_id.clone(),
            alpha.attempt_id.clone(),
            "analysis_interrupted",
            100,
        )
        .unwrap();
        checkpoint.completed_units = vec!["first-pass".to_owned()];
        checkpoint.pending_units = vec!["finalize".to_owned()];
        checkpoint.evidence_count = 1;
        artifacts.write_checkpoint(&checkpoint).unwrap();
        run_id
    };

    let mut reopened = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));
    let summary = engine
        .resume_run(&mut reopened, &artifacts, &mut runtime, &run_id)
        .await
        .expect("bounded continuation succeeds");

    assert_eq!(summary.completed_tasks, 3);
    let alpha = reopened
        .attempts(&run_id)
        .unwrap()
        .into_iter()
        .find(|attempt| attempt.task_id.to_string() == "alpha")
        .unwrap();
    assert_eq!(
        reopened
            .get_cli_attempt(&alpha.attempt_id)
            .unwrap()
            .unwrap()
            .continuation_count,
        1
    );
    assert_eq!(alpha.observed_tokens, 13);
    assert_eq!(
        reopened
            .reservation_usage(&alpha.attempt_id)
            .unwrap()
            .unwrap()
            .0,
        13
    );
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 4);
}

#[tokio::test]
async fn prepared_continuation_intent_resumes_without_incrementing_twice() {
    let directory = private_directory("harp-continuation-prepared-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.insert(
        "alpha".to_owned(),
        VecDeque::from([
            FakeOutcome {
                status: TurnStatus::Interrupted,
                final_message: None,
                total_tokens: 8,
                token_snapshots: Vec::new(),
            },
            FakeOutcome {
                status: TurnStatus::Completed,
                final_message: Some(result_message(&artifacts, "alpha", 13)),
                total_tokens: 5,
                token_snapshots: Vec::new(),
            },
        ]),
    );
    let backend = PersistentFakeBackend::new(scripted);
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(None, 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("initial turn is interrupted");
        let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
        let alpha = state.attempts(&run_id).unwrap()[0].clone();
        artifacts
            .write_checkpoint(
                &Checkpoint::new(
                    run_id.clone(),
                    alpha.task_id.clone(),
                    alpha.attempt_id.clone(),
                    "resume",
                    100,
                )
                .unwrap(),
            )
            .unwrap();
        state
            .set_lease_clock(Arc::new(TestLeaseClock::new(1_000)))
            .unwrap();
        state.expire_leases(&run_id, 1_000).unwrap();
        let activity_id = latest_semantic_activity_id(&state_path, &alpha.attempt_id);
        let activity = state.get_cli_activity(&activity_id).unwrap().unwrap();
        let lease = state
            .reclaim_attempt(
                &alpha.attempt_id,
                "preparing-worker",
                alpha.lease_expires_at.unwrap(),
                1_000,
                1_100,
            )
            .unwrap();
        state
            .mark_activity_reconciling(&lease, &activity.activity_id, 1_001)
            .unwrap();
        let checkpoint_version = artifacts
            .read_checkpoint_version(&harp_artifacts::AttemptKey {
                run_id: run_id.clone(),
                task_id: alpha.task_id.clone(),
                attempt_id: alpha.attempt_id.clone(),
            })
            .unwrap()
            .unwrap();
        let continuation_id = harp_contracts::OperationId::new();
        let intent =
            test_continuation_turn_spec(&alpha.task_id, &checkpoint_version, &continuation_id);
        state
            .prepare_continuation_after_failure(
                &lease,
                &activity.activity_id,
                harp_state::CliTerminalFailureClass::ResumableInterrupted,
                continuation_id,
                &harp_state::ActivityPreparation::new(
                    activity.activity_dir,
                    test_activity_invocation_digest(&intent),
                )
                .unwrap(),
                1_002,
            )
            .unwrap();
        run_id
    };

    let mut state =
        StateStore::open_with_lease_clock(&state_path, Arc::new(TestLeaseClock::new(1_000)))
            .unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 2_000));
    let summary = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect("prepared continuation intent resumes");

    assert_eq!(summary.completed_tasks, 3);
    let alpha = state
        .attempts(&run_id)
        .unwrap()
        .into_iter()
        .find(|attempt| attempt.task_id.to_string() == "alpha")
        .unwrap();
    assert_eq!(
        state
            .get_cli_attempt(&alpha.attempt_id)
            .unwrap()
            .unwrap()
            .continuation_count,
        1
    );
    assert_eq!(alpha.observed_tokens, 13);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 4);
}

fn prepare_dispatching_continuation(
    state: &mut StateStore,
    artifacts: &ArtifactStore,
    state_path: &std::path::Path,
    run_id: &harp_contracts::RunId,
    now: i64,
) -> (
    harp_state::LeaseToken,
    harp_state::CliActivityRecord,
    harp_contracts::TurnSpec,
) {
    let attempt = state.attempts(run_id).unwrap()[0].clone();
    let checkpoint_version = artifacts
        .read_checkpoint_version(&harp_artifacts::AttemptKey {
            run_id: run_id.clone(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
        })
        .unwrap()
        .unwrap();
    state
        .set_lease_clock(Arc::new(TestLeaseClock::new(now)))
        .unwrap();
    state.expire_leases(run_id, now).unwrap();
    let lease = state
        .reclaim_attempt(
            &attempt.attempt_id,
            "recovery-worker",
            attempt.lease_expires_at.unwrap(),
            now,
            now + 100,
        )
        .unwrap();
    let failed_activity_id = latest_semantic_activity_id(state_path, &attempt.attempt_id);
    let failed_activity = state
        .get_cli_activity(&failed_activity_id)
        .unwrap()
        .unwrap();
    state
        .mark_activity_reconciling(&lease, &failed_activity.activity_id, now + 1)
        .unwrap();
    let continuation_id = harp_contracts::OperationId::new();
    let intent =
        test_continuation_turn_spec(&attempt.task_id, &checkpoint_version, &continuation_id);
    let activity = state
        .prepare_continuation_after_failure(
            &lease,
            &failed_activity.activity_id,
            harp_state::CliTerminalFailureClass::ResumableInterrupted,
            continuation_id,
            &harp_state::ActivityPreparation::new(
                failed_activity.activity_dir,
                test_activity_invocation_digest(&intent),
            )
            .unwrap(),
            now + 2,
        )
        .unwrap();
    state
        .mark_activity_dispatching(&lease, &activity.activity_id, now + 3)
        .unwrap();
    (lease, activity, intent)
}

#[tokio::test]
async fn dispatching_continuation_without_external_turn_starts_pinned_intent_once() {
    let directory = private_directory("harp-continuation-dispatch-before-effect-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.insert(
        "alpha".to_owned(),
        VecDeque::from([
            FakeOutcome {
                status: TurnStatus::Interrupted,
                final_message: None,
                total_tokens: 8,
                token_snapshots: Vec::new(),
            },
            FakeOutcome {
                status: TurnStatus::Completed,
                final_message: Some(result_message(&artifacts, "alpha", 5)),
                total_tokens: 5,
                token_snapshots: Vec::new(),
            },
        ]),
    );
    let backend = PersistentFakeBackend::new(scripted);
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(None, 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("initial turn interrupted");
        let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
        let attempt = state.attempts(&run_id).unwrap()[0].clone();
        artifacts
            .write_checkpoint(
                &Checkpoint::new(
                    run_id.clone(),
                    attempt.task_id,
                    attempt.attempt_id,
                    "resume",
                    100,
                )
                .unwrap(),
            )
            .unwrap();
        prepare_dispatching_continuation(&mut state, &artifacts, &state_path, &run_id, 1_000);
        run_id
    };
    let activities_before = backend.start_activity_calls();
    let mut state =
        StateStore::open_with_lease_clock(&state_path, Arc::new(TestLeaseClock::new(1_000)))
            .unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 2_000));

    engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect("dispatching continuation starts exact pinned intent");

    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), activities_before + 3);
    let alpha_attempts = state
        .attempts(&run_id)
        .unwrap()
        .into_iter()
        .filter(|attempt| attempt.task_id.to_string() == "alpha")
        .collect::<Vec<_>>();
    assert!(alpha_attempts
        .iter()
        .any(|attempt| attempt.state == AttemptState::Indeterminate));
    assert!(alpha_attempts
        .iter()
        .any(|attempt| attempt.state == AttemptState::Succeeded && attempt.observed_tokens == 5));
}

#[tokio::test]
async fn lost_continuation_response_reconciles_marker_without_duplicate_turn() {
    let directory = private_directory("harp-continuation-dispatch-after-effect-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.insert(
        "alpha".to_owned(),
        VecDeque::from([
            FakeOutcome {
                status: TurnStatus::Interrupted,
                final_message: None,
                total_tokens: 8,
                token_snapshots: Vec::new(),
            },
            FakeOutcome {
                status: TurnStatus::Completed,
                final_message: Some(result_message(&artifacts, "alpha", 13)),
                total_tokens: 5,
                token_snapshots: Vec::new(),
            },
        ]),
    );
    let backend = PersistentFakeBackend::new(scripted);
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(None, 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("initial turn interrupted");
        let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
        let attempt = state.attempts(&run_id).unwrap()[0].clone();
        artifacts
            .write_checkpoint(
                &Checkpoint::new(
                    run_id.clone(),
                    attempt.task_id,
                    attempt.attempt_id,
                    "resume",
                    100,
                )
                .unwrap(),
            )
            .unwrap();
        let (lease, activity, intent) =
            prepare_dispatching_continuation(&mut state, &artifacts, &state_path, &run_id, 1_000);
        let cli_attempt = state
            .get_cli_attempt(&activity.attempt_id)
            .unwrap()
            .unwrap();
        let mut effect_runtime = backend.runtime();
        let handle = harp_runtime::ActivityRuntime::start_activity(
            &mut effect_runtime,
            ActivitySpec {
                logical_session_id: cli_attempt.logical_session_id.clone(),
                logical_turn_id: activity.logical_turn_id.clone(),
                thread_spec: harp_contracts::ThreadSpec {
                    base_instructions: String::new(),
                    developer_instructions: String::new(),
                    cwd: activity.activity_dir.clone(),
                    runtime_workspace_roots: vec![activity.activity_dir.clone()],
                    workspace_authority: None,
                    approval_policy: "never".to_owned(),
                    sandbox_mode: "workspace-write".to_owned(),
                    model: "test-model".to_owned(),
                    reasoning_effort: None,
                    ephemeral: true,
                },
                turn_spec: intent,
                activity_dir: activity.activity_dir,
                invocation_sha256: activity.invocation_sha256,
                external_session_id: cli_attempt.external_session_id,
            },
        )
        .await
        .unwrap();
        state
            .record_process(
                &lease,
                &activity.activity_id,
                &handle.process_record_sha256,
                1_004,
            )
            .unwrap();
        run_id
    };
    let activities_before = backend.start_activity_calls();
    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 2_000));

    engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect("marker reconciliation finds continuation external turn");

    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), activities_before + 2);
    let alpha = state
        .attempts(&run_id)
        .unwrap()
        .into_iter()
        .find(|attempt| attempt.task_id.to_string() == "alpha")
        .unwrap();
    assert_eq!(
        state
            .get_cli_attempt(&alpha.attempt_id)
            .unwrap()
            .unwrap()
            .continuation_count,
        1
    );
    assert_eq!(alpha.observed_tokens, 13);
}

#[tokio::test]
async fn second_continuation_is_rejected_without_starting_another_turn() {
    let directory = private_directory("harp-continuation-bound-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.insert(
        "alpha".to_owned(),
        VecDeque::from([
            FakeOutcome {
                status: TurnStatus::Interrupted,
                final_message: None,
                total_tokens: 8,
                token_snapshots: Vec::new(),
            },
            FakeOutcome {
                status: TurnStatus::Interrupted,
                final_message: None,
                total_tokens: 5,
                token_snapshots: Vec::new(),
            },
        ]),
    );
    let backend = PersistentFakeBackend::new(scripted);
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(None, 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("initial turn is interrupted");
        let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
        let alpha = state.attempts(&run_id).unwrap()[0].clone();
        artifacts
            .write_checkpoint(
                &Checkpoint::new(
                    run_id.clone(),
                    alpha.task_id,
                    alpha.attempt_id,
                    "resume",
                    100,
                )
                .unwrap(),
            )
            .unwrap();
        run_id
    };

    {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(None, 1_000));
        engine
            .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
            .await
            .expect_err("the only continuation is also interrupted");
    }
    let activities_before_second_restart = backend.start_activity_calls();

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 2_000));
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("second continuation is forbidden");
    assert!(matches!(
        error,
        harp_engine::EngineError::State(harp_state::StateError::Conflict { ref entity, .. })
            if entity.contains("continuation count")
    ));
    let alpha = state.attempts(&run_id).unwrap()[0].clone();
    assert_eq!(
        state
            .get_cli_attempt(&alpha.attempt_id)
            .unwrap()
            .unwrap()
            .continuation_count,
        1
    );
    assert_eq!(alpha.observed_tokens, 13);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(
        backend.start_activity_calls(),
        activities_before_second_restart
    );
}

#[tokio::test]
async fn recovered_completed_turn_revalidates_pinned_output_schema() {
    let directory = private_directory("harp-recovered-output-schema-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let mut graph = graph();
    graph.nodes[0].output_schema = serde_json::json!({
        "type": "object",
        "required": ["mustNotExist"],
        "properties": {"mustNotExist": {"const": true}}
    })
    .to_string();
    let mut graph_policy = execution_spec(&artifacts, &backend).graph_policy().clone();
    graph_policy
        .allowed_output_schemas
        .insert(graph.nodes[0].output_schema.clone());
    let projection_policy = execution_spec(&artifacts, &backend)
        .projection_policy()
        .clone();
    let validated = validate_graph(graph, &graph_policy, &projection_policy).unwrap();
    let runtime_for_spec = backend.runtime();
    let spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime_for_spec.provenance().unwrap(),
    )
    .unwrap();
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::DuringTurn), 10));
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
            .expect_err("crash before terminal result is decoded");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    backend.complete_all_turns();
    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));

    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("recovered completion violates pinned schema");

    assert!(matches!(
        error,
        harp_engine::EngineError::OutputSchema { .. }
    ));
    assert!(state.accepted_results(&run_id).unwrap().is_empty());
    assert_eq!(
        state.attempts(&run_id).unwrap()[0].state,
        AttemptState::Failed
    );
    assert_eq!(
        state.get_run(&run_id).unwrap().unwrap().state,
        RunState::Failed
    );
    let calls = (backend.start_thread_calls(), backend.start_turn_calls());
    drop(state);

    let mut reopened = StateStore::open(&state_path).unwrap();
    let mut resumed_runtime = backend.runtime();
    let mut resumed = fixed_clock_engine(config(None, 2_000));
    resumed
        .resume_run(&mut reopened, &artifacts, &mut resumed_runtime, &run_id)
        .await
        .expect("semantic failure restart is inert");
    assert_eq!(
        (backend.start_thread_calls(), backend.start_turn_calls()),
        calls
    );
}

#[test]
fn published_object_decoder_revalidates_exact_pinned_output_schema() {
    let directory = private_directory("harp-published-output-schema-");
    let artifacts = open_artifacts(&directory);
    let raw = result_message(&artifacts, "alpha", 8);
    let published = artifacts
        .publish(raw.as_bytes(), "application/vnd.harp.result+json")
        .unwrap();
    let bytes = artifacts.read_verified(&published).unwrap();
    let restrictive_schema = serde_json::json!({
        "type": "object",
        "required": ["mustNotExist"],
        "properties": {"mustNotExist": {"const": true}}
    });

    let error = decode_result_envelope(
        &restrictive_schema,
        &bytes,
        &TaskId::from_str("alpha").unwrap(),
        Some(8),
    )
    .expect_err("published object violates pinned schema");

    assert!(matches!(
        error,
        harp_engine::EngineError::OutputSchema { .. }
    ));
}

#[tokio::test]
async fn published_result_resume_consumes_bridged_publish_once_and_is_inert() {
    let directory = private_directory("harp-published-output-convergence-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::AfterResultPublished), 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash after result publication");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    let connection = rusqlite::Connection::open(&state_path).unwrap();
    let publish_count: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM operations o
             JOIN attempts a ON a.attempt_id = o.attempt_id
             WHERE a.run_id = ?1
               AND a.task_id = 'alpha'
               AND o.kind = 'publish_result'
               AND o.ordinal = 0
               AND o.state = 'completed'",
            [run_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(publish_count, 1);
    drop(connection);

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));
    let summary = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect("published bridged result is consumed");

    assert_eq!(summary.completed_tasks, 3);
    assert_eq!(
        state.attempts(&run_id).unwrap()[0].state,
        AttemptState::Succeeded
    );
    assert_eq!(
        state.get_run(&run_id).unwrap().unwrap().state,
        RunState::Completed
    );
    let calls = (
        backend.start_thread_calls(),
        backend.start_turn_calls(),
        backend.start_activity_calls(),
    );
    drop(state);

    let mut reopened = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 2_000));
    engine
        .resume_run(&mut reopened, &artifacts, &mut runtime, &run_id)
        .await
        .expect("failed published result restart is inert");
    assert_eq!(
        (
            backend.start_thread_calls(),
            backend.start_turn_calls(),
            backend.start_activity_calls()
        ),
        calls
    );
}

#[tokio::test]
async fn cancellation_persists_before_interrupt_and_survives_restart() {
    let directory = private_directory("harp-cancellation-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::AfterDispatchingTurn), 10));
        let error = engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash after external turn creation");
        assert!(matches!(
            error,
            harp_engine::EngineError::InjectedCrash {
                point: CrashPoint::AfterDispatchingTurn
            }
        ));
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };

    let cancellation_clock = Arc::new(TestLeaseClock::new(20));
    let mut state =
        StateStore::open_with_lease_clock(&state_path, cancellation_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let mut non_owner = fixed_clock_engine_with_lease(
        EngineConfig {
            worker_id: "cancellation-controller".to_owned(),
            ..config(None, 20)
        },
        cancellation_clock,
    );
    let summary = non_owner
        .cancel_run(&mut state, &mut runtime, &run_id)
        .await
        .expect("non-owner persists cancellation without stealing");

    let run = state.get_run(&run_id).unwrap().unwrap();
    assert_eq!(run.state, RunState::Active);
    assert!(run.cancellation_requested);
    assert_eq!(summary.completed_tasks, 0);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 1);
    assert_eq!(backend.interrupt_calls(), 0);
    drop(state);

    let mut state =
        StateStore::open_with_lease_clock(&state_path, Arc::new(TestLeaseClock::new(1_000)))
            .unwrap();
    let mut owner_runtime = backend.runtime();
    let mut owner = fixed_clock_engine(config(None, 1_000));
    owner
        .resume_run(&mut state, &artifacts, &mut owner_runtime, &run_id)
        .await
        .expect("active owner observes cancellation and interrupts");
    assert_eq!(
        state.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert_eq!(backend.interrupt_calls(), 1);
    let events = state.events_page(&run_id, None, 1_000).unwrap().events;
    let requested = events
        .iter()
        .position(|event| event.event_type == "run_cancellation_requested")
        .unwrap();
    let interrupt_dispatch = events
        .iter()
        .position(|event| event.event_type == "cli_interrupt_prepared")
        .unwrap();
    assert!(requested < interrupt_dispatch);
    drop(state);

    let mut reopened = StateStore::open(&state_path).unwrap();
    let mut resumed_runtime = backend.runtime();
    let mut resumed_engine = fixed_clock_engine(config(None, 1_000));
    resumed_engine
        .resume_run(&mut reopened, &artifacts, &mut resumed_runtime, &run_id)
        .await
        .expect("cancelled restart is inert");
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 1);
    assert_eq!(backend.interrupt_calls(), 1);
}

#[tokio::test(start_paused = true)]
async fn concurrent_owner_control_cancels_pending_execution_without_lease_expiry() {
    let directory = private_directory("harp-concurrent-control-cancel-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    backend.set_next_event_delay(Duration::from_secs(120));
    let state_path = directory.path().join("state.sqlite");
    let lease_clock = Arc::new(TokioLeaseClock::new(1_000));
    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let runtime_control = runtime.control_handle().expect("fake runtime control");
    let engine_config = EngineConfig {
        lease_seconds: 30,
        lease_renewal_threshold_seconds: 10,
        runtime_operation_timeout_seconds: 20,
        ..config(None, 10)
    };
    let mut engine = Engine::with_clocks(
        engine_config.clone(),
        Arc::new(TestWallClock::new(1_000)),
        lease_clock.clone(),
    )
    .unwrap();
    let owner_control = engine.control_handle();
    let spec = execution_spec(&artifacts, &backend);
    let execution = tokio::spawn(async move {
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
    });

    let run_id = loop {
        tokio::task::yield_now().await;
        let mut observer =
            StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
        let Some(run) = observer.incomplete_runs().unwrap().into_iter().next() else {
            continue;
        };
        let activities = observer.recoverable_activities(&run.run_id).unwrap();
        if activities.iter().any(|activity| {
            matches!(
                activity.activity_state,
                harp_state::CliActivityState::Running | harp_state::CliActivityState::Reconciling
            )
        }) {
            break run.run_id;
        }
    };

    let non_owner_engine = Engine::with_clocks(
        EngineConfig {
            worker_id: "non-owner-control".to_owned(),
            ..engine_config
        },
        Arc::new(TestWallClock::new(1_000)),
        lease_clock.clone(),
    )
    .unwrap();
    let non_owner = non_owner_engine.control_handle();
    let mut non_owner_state =
        StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let summary = non_owner
        .cancel_run(&mut non_owner_state, runtime_control.as_ref(), &run_id)
        .await
        .expect("non-owner handle persists request only");
    assert_eq!(summary.completed_tasks, 0);
    assert_eq!(backend.interrupt_calls(), 0);
    assert_eq!(
        non_owner_state.get_run(&run_id).unwrap().unwrap().state,
        RunState::Active
    );
    drop(non_owner_state);

    let mut control_state =
        StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    owner_control
        .cancel_run(&mut control_state, runtime_control.as_ref(), &run_id)
        .await
        .expect("owner handle interrupts under live lease");

    assert_eq!(backend.interrupt_calls(), 1);
    assert_eq!(
        control_state.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert!(control_state
        .tasks(&run_id)
        .unwrap()
        .iter()
        .all(|task| task.state == harp_state::TaskState::Cancelled));
    let attempt_id = control_state.attempts(&run_id).unwrap()[0]
        .attempt_id
        .clone();
    let interrupt_count: i64 = rusqlite::Connection::open(state_path.clone())
        .unwrap()
        .query_row(
            "SELECT COUNT(*)
                 FROM cli_activities
                 WHERE attempt_id = ?1
                   AND kind = 'interrupt_activity'
                   AND interrupt_purpose = 'cancellation'
                   AND signal_stage = 'quiescent'
                   AND state = 'completed'",
            [attempt_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(interrupt_count, 1);

    tokio::time::advance(Duration::from_secs(120)).await;
    tokio::task::yield_now().await;
    assert!(execution.await.unwrap().is_err());
    assert_eq!(backend.interrupt_calls(), 1);
}

#[tokio::test(start_paused = true)]
async fn timed_out_cancellation_interrupt_replays_with_cancellation_purpose() {
    let directory = private_directory("harp-cancellation-interrupt-timeout-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    backend.set_next_event_delay(Duration::from_secs(120));
    backend.set_interrupt_delay(Duration::from_secs(12));
    let state_path = directory.path().join("state.sqlite");
    let lease_clock = Arc::new(TokioLeaseClock::new(1_000));
    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let runtime_control = runtime.control_handle().unwrap();
    let engine_config = EngineConfig {
        lease_seconds: 30,
        lease_renewal_threshold_seconds: 10,
        runtime_operation_timeout_seconds: 5,
        ..config(None, 10)
    };
    let mut engine = Engine::with_clocks(
        engine_config.clone(),
        Arc::new(TestWallClock::new(1_000)),
        lease_clock.clone(),
    )
    .unwrap();
    let control = engine.control_handle();
    let spec = execution_spec(&artifacts, &backend);
    let execution = tokio::spawn(async move {
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
    });
    let run_id = loop {
        tokio::task::yield_now().await;
        let mut observer =
            StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
        let Some(run) = observer.incomplete_runs().unwrap().into_iter().next() else {
            continue;
        };
        let activities = observer.recoverable_activities(&run.run_id).unwrap();
        if activities.iter().any(|activity| {
            matches!(
                activity.activity_state,
                harp_state::CliActivityState::Running | harp_state::CliActivityState::Reconciling
            )
        }) {
            break run.run_id;
        }
    };
    let mut control_state =
        StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let error = control
        .cancel_run(&mut control_state, runtime_control.as_ref(), &run_id)
        .await
        .expect_err("cancellation interrupt exceeds control deadline");

    assert!(matches!(error, EngineError::RuntimeOperationTimeout { .. }));
    assert_eq!(backend.interrupt_calls(), 0);
    let attempt = control_state.attempts(&run_id).unwrap()[0].clone();
    let (interrupt_id, signal_stage): (String, String) =
        rusqlite::Connection::open(state_path.clone())
            .unwrap()
            .query_row(
                "SELECT activity_id, signal_stage
                 FROM cli_activities
                 WHERE attempt_id = ?1
                   AND kind = 'interrupt_activity'
                   AND interrupt_purpose = 'cancellation'
                   AND state = 'prepared'",
                [attempt.attempt_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
    assert_eq!(signal_stage, "prepared");
    drop(control_state);

    backend.set_interrupt_delay(Duration::ZERO);
    tokio::time::advance(Duration::from_secs(31)).await;
    let mut reopened = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut resumed_runtime = backend.runtime();
    let mut resumed = Engine::with_clocks(
        engine_config,
        Arc::new(TestWallClock::new(1_000)),
        lease_clock,
    )
    .unwrap();
    resumed
        .resume_run(
            &mut reopened,
            &open_artifacts(&directory),
            &mut resumed_runtime,
            &run_id,
        )
        .await
        .expect("restart replays cancellation interrupt");

    assert_eq!(backend.interrupt_calls(), 1);
    assert_eq!(
        reopened.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert_eq!(
        reopened
            .get_cli_activity(&harp_contracts::OperationId::from_str(&interrupt_id).unwrap())
            .unwrap()
            .unwrap()
            .state,
        harp_state::CliActivityState::Completed
    );
    tokio::time::advance(Duration::from_secs(120)).await;
    tokio::task::yield_now().await;
    assert!(execution.await.unwrap().is_err());
}

#[tokio::test]
async fn restart_finishes_a_persisted_cancellation_request_without_semantic_work() {
    let directory = private_directory("harp-cancellation-restart-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::AfterDispatchingTurn), 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash after external turn creation");
        let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
        state.request_run_cancellation(&run_id, 30).unwrap();
        run_id
    };

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "recovery-worker".to_owned(),
        ..config(None, 1_000)
    })
    .unwrap();
    let summary = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect("restart completes durable cancellation");

    assert_eq!(summary.completed_tasks, 0);
    assert_eq!(
        state.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 1);
    assert_eq!(backend.interrupt_calls(), 1);
}

#[tokio::test]
async fn cancellation_preserves_unknown_thread_start_as_indeterminate() {
    let directory = private_directory("harp-cancellation-thread-ambiguity-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::AfterDispatchingThread), 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("thread was created but response was not persisted");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "recovery-worker".to_owned(),
        ..config(None, 1_000)
    })
    .unwrap();
    let summary = engine
        .cancel_run(&mut state, &mut runtime, &run_id)
        .await
        .expect("cancellation preserves ambiguity");

    assert_eq!(summary.indeterminate_attempts, 1);
    assert_eq!(
        state.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.interrupt_calls(), 0);
}

#[tokio::test]
async fn resume_does_not_steal_an_unexpired_lease() {
    let directory = private_directory("harp-live-lease-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::AfterClaim), 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash after claim");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    let before = {
        let mut state = StateStore::open(&state_path).unwrap();
        state.attempts(&run_id).unwrap()[0].clone()
    };

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 20));
    let summary = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect("live lease is observed");
    let after = state.attempts(&run_id).unwrap()[0].clone();

    assert_eq!(summary.completed_tasks, 0);
    assert_eq!(before.attempt_id, after.attempt_id);
    assert_eq!(before.lease_owner, after.lease_owner);
    assert_eq!(before.lease_expires_at, after.lease_expires_at);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
}

#[tokio::test]
async fn future_event_clock_cannot_expire_healthy_shared_clock_lease() {
    let directory = private_directory("harp-shared-lease-clock-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let lease_clock = Arc::new(TestLeaseClock::new(1_000));
    let run_id = {
        let mut state =
            StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine_with_lease(
            config(Some(CrashPoint::AfterClaim), 10),
            lease_clock.clone(),
        );
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash after claim");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };

    let mut competing =
        StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut competing_runtime = backend.runtime();
    let mut competing_engine =
        fixed_clock_engine_with_lease(config(None, 1_000_000), lease_clock.clone());
    let summary = competing_engine
        .resume_run(&mut competing, &artifacts, &mut competing_runtime, &run_id)
        .await
        .expect("future event clock observes healthy lease");

    assert_eq!(summary.completed_tasks, 0);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(
        competing.attempts(&run_id).unwrap()[0]
            .lease_owner
            .as_deref(),
        Some("recovery-worker")
    );
}

#[tokio::test]
async fn advanced_shared_lease_clock_expires_and_reclaims_attempt() {
    let directory = private_directory("harp-shared-lease-expiry-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let lease_clock = Arc::new(TestLeaseClock::new(1_000));
    let run_id = {
        let mut state =
            StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine_with_lease(
            config(Some(CrashPoint::AfterClaim), 10),
            lease_clock.clone(),
        );
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash after claim");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    lease_clock.set(1_061);

    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine_with_lease(config(None, 20), lease_clock.clone());
    let summary = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect("expired lease is reclaimed from shared clock");

    assert_eq!(summary.completed_tasks, 3);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 3);
}

#[tokio::test]
async fn long_event_stream_renews_and_rotates_lease_capability() {
    let directory = private_directory("harp-lease-renewal-");
    let artifacts = open_artifacts(&directory);
    let lease_clock = Arc::new(TestLeaseClock::new(1_000));
    let hook_clock = lease_clock.clone();
    let mut scripted = outcomes(&artifacts);
    scripted.get_mut("alpha").unwrap()[0].token_snapshots = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let backend = PersistentFakeBackend::new(scripted).with_next_event_hook(Arc::new(move || {
        hook_clock.advance(3);
    }));
    let mut state = StateStore::open_with_lease_clock(
        &directory.path().join("state.sqlite"),
        lease_clock.clone(),
    )
    .unwrap();
    let mut runtime = backend.runtime();
    let mut config = config(None, 10);
    config.lease_seconds = 8;
    config.lease_renewal_threshold_seconds = 4;
    let mut engine = fixed_clock_engine_with_lease(config, lease_clock.clone());

    let summary = engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .expect("long stream renews healthy lease");

    assert_eq!(summary.completed_tasks, 3);
    let events = state
        .events_page(&summary.run_id, None, 1_000)
        .unwrap()
        .events;
    assert!(
        events
            .iter()
            .filter(|event| event.event_type == "lease_renewed")
            .count()
            >= 2
    );
}

#[tokio::test(start_paused = true)]
async fn delayed_start_turn_renews_lease_without_duplicate_marker_effect() {
    let directory = private_directory("harp-delayed-start-turn-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    backend.set_start_turn_delay(Duration::from_secs(12));
    let lease_clock = Arc::new(TokioLeaseClock::new(1_000));
    let mut state = StateStore::open_with_lease_clock(
        &directory.path().join("state.sqlite"),
        lease_clock.clone(),
    )
    .unwrap();
    let mut runtime = backend.runtime();
    let mut config = config(None, 10);
    config.lease_seconds = 8;
    config.lease_renewal_threshold_seconds = 2;
    let mut engine =
        Engine::with_clocks(config, Arc::new(TestWallClock::new(1_000)), lease_clock).unwrap();

    let summary = engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .expect("delayed start turn renews until completion");

    assert_eq!(summary.completed_tasks, 3);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 3);
    assert!(state
        .events_page(&summary.run_id, None, 1_000)
        .unwrap()
        .events
        .iter()
        .any(|event| event.event_type == "lease_renewed"));
}

#[tokio::test(start_paused = true)]
async fn delayed_interrupt_renews_lease_and_completes_persisted_intent() {
    let directory = private_directory("harp-delayed-interrupt-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.get_mut("alpha").unwrap()[0].total_tokens = 101;
    scripted.get_mut("alpha").unwrap()[0].token_snapshots = vec![101];
    let backend = PersistentFakeBackend::new(scripted);
    backend.set_interrupt_delay(Duration::from_secs(12));
    let lease_clock = Arc::new(TokioLeaseClock::new(1_000));
    let mut state = StateStore::open_with_lease_clock(
        &directory.path().join("state.sqlite"),
        lease_clock.clone(),
    )
    .unwrap();
    let mut runtime = backend.runtime();
    let mut config = config(None, 10);
    config.lease_seconds = 8;
    config.lease_renewal_threshold_seconds = 2;
    let mut engine =
        Engine::with_clocks(config, Arc::new(TestWallClock::new(1_000)), lease_clock).unwrap();

    let error = engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .expect_err("token overage interrupts after delayed control call");

    assert!(matches!(error, EngineError::BudgetExceeded { .. }));
    assert_eq!(backend.interrupt_calls(), 1);
    let attempt_id_text: String = rusqlite::Connection::open(directory.path().join("state.sqlite"))
        .unwrap()
        .query_row("SELECT attempt_id FROM attempts LIMIT 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    let attempt_id = harp_contracts::AttemptId::from_str(&attempt_id_text).unwrap();
    let completed_interrupts = cli_interrupt_count(
        &directory.path().join("state.sqlite"),
        &attempt_id,
        "budget",
        "completed",
        "quiescent",
    );
    assert_eq!(completed_interrupts, 1);
}

#[tokio::test(start_paused = true)]
async fn timed_out_budget_interrupt_replays_after_restart_with_reason_preserved() {
    let directory = private_directory("harp-budget-interrupt-timeout-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.get_mut("alpha").unwrap()[0].total_tokens = 101;
    scripted.get_mut("alpha").unwrap()[0].token_snapshots = vec![101];
    let backend = PersistentFakeBackend::new(scripted);
    backend.set_interrupt_delay(Duration::from_secs(12));
    let state_path = directory.path().join("state.sqlite");
    let lease_clock = Arc::new(TestLeaseClock::new(1_000));
    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let mut engine_config = config(None, 10);
    engine_config.runtime_operation_timeout_seconds = 5;
    let mut engine = fixed_clock_engine_with_lease(engine_config.clone(), lease_clock.clone());

    let error = engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .expect_err("interrupt response exceeds control deadline");

    assert!(matches!(error, EngineError::RuntimeOperationTimeout { .. }));
    assert_eq!(backend.interrupt_calls(), 0);
    let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
    let attempt = state.attempts(&run_id).unwrap()[0].clone();
    assert_eq!(
        cli_interrupt_count(
            &state_path,
            &attempt.attempt_id,
            "budget",
            "prepared",
            "prepared",
        ),
        1
    );
    drop(state);

    backend.set_interrupt_delay(Duration::ZERO);
    lease_clock.set(2_000);
    let mut reopened = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut resumed_runtime = backend.runtime();
    let mut resumed = fixed_clock_engine_with_lease(engine_config, lease_clock);
    let error = resumed
        .resume_run(&mut reopened, &artifacts, &mut resumed_runtime, &run_id)
        .await
        .expect_err("restart replays durable budget interrupt");

    assert!(matches!(error, EngineError::BudgetExceeded { .. }));
    assert_eq!(backend.interrupt_calls(), 1);
    assert_eq!(
        reopened.attempts(&run_id).unwrap()[0].state,
        AttemptState::Failed
    );
    assert_eq!(
        cli_interrupt_count(
            &state_path,
            &attempt.attempt_id,
            "budget",
            "completed",
            "quiescent",
        ),
        1
    );
    assert!(reopened
        .events_page(&run_id, None, 1_000)
        .unwrap()
        .events
        .iter()
        .any(|event| {
            event.event_type == "terminal_budget_exhausted"
                && event
                    .payload
                    .get("reasonCode")
                    .and_then(serde_json::Value::as_str)
                    == Some("token_limit")
        }));
}

#[tokio::test(start_paused = true)]
async fn lost_interrupt_response_is_reconciled_without_duplicate_effect() {
    let directory = private_directory("harp-interrupt-lost-response-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.get_mut("alpha").unwrap()[0].total_tokens = 101;
    scripted.get_mut("alpha").unwrap()[0].token_snapshots = vec![101];
    let backend = PersistentFakeBackend::new(scripted);
    backend.set_interrupt_response_delay(Duration::from_secs(12));
    let state_path = directory.path().join("state.sqlite");
    let lease_clock = Arc::new(TestLeaseClock::new(1_000));
    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let mut engine_config = config(None, 10);
    engine_config.runtime_operation_timeout_seconds = 5;
    let mut engine = fixed_clock_engine_with_lease(engine_config.clone(), lease_clock.clone());

    engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .expect_err("interrupt effect succeeds but response is lost");
    assert_eq!(backend.interrupt_calls(), 1);
    let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
    drop(state);

    backend.set_interrupt_response_delay(Duration::ZERO);
    lease_clock.set(2_000);
    let mut reopened = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut resumed_runtime = backend.runtime();
    let mut resumed = fixed_clock_engine_with_lease(engine_config, lease_clock);
    let error = resumed
        .resume_run(&mut reopened, &artifacts, &mut resumed_runtime, &run_id)
        .await
        .expect_err("restart observes terminal turn and finalizes budget failure");

    assert!(matches!(error, EngineError::BudgetExceeded { .. }));
    assert_eq!(backend.interrupt_calls(), 1);
    assert_eq!(
        reopened.attempts(&run_id).unwrap()[0].state,
        AttemptState::Failed
    );
}

#[tokio::test(start_paused = true)]
async fn start_turn_deadline_preserves_dispatching_marker_ambiguity() {
    let directory = private_directory("harp-start-turn-deadline-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    backend.set_start_turn_delay(Duration::from_secs(61));
    let lease_clock = Arc::new(TokioLeaseClock::new(1_000));
    let mut state = StateStore::open_with_lease_clock(
        &directory.path().join("state.sqlite"),
        lease_clock.clone(),
    )
    .unwrap();
    let mut runtime = backend.runtime();
    let mut config = config(None, 10);
    config.lease_seconds = 8;
    config.lease_renewal_threshold_seconds = 2;
    let mut engine =
        Engine::with_clocks(config, Arc::new(TestWallClock::new(1_000)), lease_clock).unwrap();

    let error = engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .expect_err("start turn exceeds operation deadline");

    assert!(matches!(error, EngineError::RuntimeOperationTimeout { .. }));
    let connection = rusqlite::Connection::open(directory.path().join("state.sqlite")).unwrap();
    let state_text: String = connection
        .query_row(
            "SELECT state FROM attempts ORDER BY attempt_id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(state_text, "prepared");
    assert_eq!(backend.start_turn_calls(), 0);
}

#[tokio::test(start_paused = true)]
async fn pending_runtime_future_renews_while_competing_engine_cannot_reclaim() {
    let directory = private_directory("harp-pending-future-competition-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    backend.set_start_turn_delay(Duration::from_secs(12));
    let lease_clock = Arc::new(TokioLeaseClock::new(1_000));
    let state_path = directory.path().join("state.sqlite");
    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let mut config = config(None, 10);
    config.lease_seconds = 8;
    config.lease_renewal_threshold_seconds = 2;
    let mut engine = Engine::with_clocks(
        config.clone(),
        Arc::new(TestWallClock::new(1_000)),
        lease_clock.clone(),
    )
    .unwrap();
    let spec = execution_spec(&artifacts, &backend);
    let execution = tokio::spawn(async move {
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
    });

    tokio::time::advance(Duration::from_secs(6)).await;
    tokio::task::yield_now().await;

    let mut competing_state =
        StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let run_id = competing_state.incomplete_runs().unwrap()[0].run_id.clone();
    let mut competing_runtime = backend.runtime();
    let mut competing = Engine::with_clocks(
        EngineConfig {
            worker_id: "competing-worker".to_owned(),
            ..config
        },
        Arc::new(TestWallClock::new(1_000)),
        lease_clock,
    )
    .unwrap();
    let summary = competing
        .resume_run(
            &mut competing_state,
            &open_artifacts(&directory),
            &mut competing_runtime,
            &run_id,
        )
        .await
        .expect("competing engine observes renewed lease");

    assert_eq!(summary.completed_tasks, 0);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert!(backend.start_activity_calls() <= 1);

    for _ in 0..40 {
        if execution.is_finished() {
            break;
        }
        tokio::time::advance(Duration::from_secs(2)).await;
        tokio::task::yield_now().await;
    }
    let summary = execution.await.unwrap().unwrap();
    assert_eq!(summary.completed_tasks, 3);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 3);
}

#[tokio::test]
async fn renewal_failure_clears_cached_lease_authority() {
    let directory = private_directory("harp-lease-renewal-failure-");
    let artifacts = open_artifacts(&directory);
    let lease_clock = Arc::new(TestLeaseClock::new(1_000));
    let hook_clock = lease_clock.clone();
    let backend = PersistentFakeBackend::new(outcomes(&artifacts)).with_next_event_hook(Arc::new(
        move || {
            hook_clock.advance(9);
        },
    ));
    let mut state = StateStore::open_with_lease_clock(
        &directory.path().join("state.sqlite"),
        lease_clock.clone(),
    )
    .unwrap();
    let mut runtime = backend.runtime();
    let mut config = config(None, 10);
    config.lease_seconds = 8;
    config.lease_renewal_threshold_seconds = 4;
    let mut engine = fixed_clock_engine_with_lease(config, lease_clock);

    let error = engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .expect_err("expired lease renewal fails closed");

    assert!(matches!(
        error,
        EngineError::State(harp_state::StateError::Conflict { .. })
    ));
    assert_eq!(engine.test_only_active_lease_count(), 0);
}

#[tokio::test]
async fn completed_run_resume_performs_no_new_semantic_work() {
    let directory = private_directory("harp-completed-resume-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(None, 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .unwrap()
            .run_id
    };
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 3);

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));
    let summary = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect("completed run is inert");

    assert_eq!(summary.completed_tasks, 3);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 3);
}

#[tokio::test]
async fn resume_rejects_tampered_execution_receipt_before_runtime_work() {
    let directory = private_directory("harp-receipt-tamper-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::BeforeClaimCommit), 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash before first claim");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    let connection = rusqlite::Connection::open(&state_path).unwrap();
    let mut provenance: serde_json::Value = connection
        .query_row(
            "SELECT provenance_json FROM runs WHERE run_id = ?1",
            [run_id.to_string()],
            |row| {
                let bytes: Vec<u8> = row.get(0)?;
                Ok(serde_json::from_slice(&bytes).unwrap())
            },
        )
        .unwrap();
    provenance["executionSpec"]["receipt"]["graphPolicy"]["maxTotalTokens"] =
        serde_json::json!(999_999);
    connection
        .execute(
            "UPDATE runs SET provenance_json = ?2 WHERE run_id = ?1",
            rusqlite::params![run_id.to_string(), serde_json::to_vec(&provenance).unwrap()],
        )
        .unwrap();
    drop(connection);

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("receipt digest catches widened policy");
    assert!(matches!(
        error,
        harp_engine::EngineError::ExecutionReceipt { .. }
    ));
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
}

#[tokio::test]
async fn resume_rejects_wrong_artifact_store_before_runtime_work() {
    let directory = private_directory("harp-artifact-authority-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::BeforeClaimCommit), 10));
        let spec = execution_spec(&artifacts, &backend);
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
            .expect_err("crash before runtime work");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    let wrong_directory = private_directory("harp-wrong-artifact-authority-");
    let wrong_artifacts = open_artifacts(&wrong_directory);
    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));

    let error = engine
        .resume_run(&mut state, &wrong_artifacts, &mut runtime, &run_id)
        .await
        .expect_err("artifact authority mismatch");

    assert!(matches!(
        error,
        harp_engine::EngineError::ExecutionReceipt { .. }
    ));
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
}

#[tokio::test]
async fn resume_rejects_wrong_runtime_identity_before_runtime_work() {
    let directory = private_directory("harp-runtime-authority-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::BeforeClaimCommit), 10));
        let spec = execution_spec(&artifacts, &backend);
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
            .expect_err("crash before runtime work");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    let wrong_backend =
        PersistentFakeBackend::with_runtime_identity(outcomes(&artifacts), "different-runtime");
    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = wrong_backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));

    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("runtime authority mismatch");

    assert!(matches!(
        error,
        harp_engine::EngineError::ExecutionReceipt { .. }
    ));
    assert_eq!(wrong_backend.start_thread_calls(), 0);
    assert_eq!(wrong_backend.start_turn_calls(), 0);
}

#[tokio::test]
async fn resume_rejects_corrupted_approved_input_before_runtime_work() {
    let directory = private_directory("harp-input-authority-");
    let artifacts = open_artifacts(&directory);
    let approved = artifacts
        .publish(b"approved input", "application/json")
        .unwrap();
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let mut graph = graph();
    graph.nodes[0].inputs.push(approved.clone());
    let (mut graph_policy, mut projection_policy) = {
        let spec = execution_spec(&artifacts, &backend);
        let graph_policy = spec.graph_policy().clone();
        let projection_policy = spec.projection_policy().clone();
        (graph_policy, projection_policy)
    };
    graph_policy.approved_artifacts.insert(
        approved.uri.clone(),
        harp_engine::ApprovedArtifact {
            reference: approved.clone(),
            materialized_path: "/approved/input.json".to_owned(),
            read_only: true,
        },
    );
    projection_policy.scratch_paths = graph
        .nodes
        .iter()
        .map(|node| {
            (
                node.task_id.clone(),
                format!("/private/tmp/harp/{}", node.task_id),
            )
        })
        .collect();
    let validated = validate_graph(graph, &graph_policy, &projection_policy).unwrap();
    let runtime = backend.runtime();
    let spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap();
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::BeforeClaimCommit), 10));
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
            .expect_err("crash before runtime work");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    let object_path = directory
        .path()
        .join("artifacts/objects/sha256")
        .join(&approved.sha256[..2])
        .join(&approved.sha256);
    fs::set_permissions(&object_path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&object_path, b"tampered bytes").unwrap();
    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));

    engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("approved input corruption fails closed");
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
}

#[tokio::test]
async fn hard_task_token_limit_persists_interrupt_before_returning_failure() {
    let directory = private_directory("harp-budget-interrupt-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.get_mut("alpha").unwrap()[0].total_tokens = 101;
    let backend = PersistentFakeBackend::new(scripted);
    let state_path = directory.path().join("state.sqlite");
    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 10));
    let error = engine
        .execute_run(
            &mut state,
            &artifacts,
            &mut runtime,
            &execution_spec(&artifacts, &backend),
        )
        .await
        .expect_err("hard task limit interrupts execution");
    assert!(matches!(
        error,
        harp_engine::EngineError::BudgetExceeded { .. }
    ));

    let connection = rusqlite::Connection::open(&state_path).unwrap();
    let run_id_text: String = connection
        .query_row("SELECT run_id FROM runs LIMIT 1", [], |row| row.get(0))
        .unwrap();
    let run_id = RunId::from_str(&run_id_text).unwrap();
    let alpha = state
        .attempts(&run_id)
        .unwrap()
        .into_iter()
        .find(|attempt| attempt.task_id.to_string() == "alpha")
        .unwrap();
    assert_eq!(alpha.observed_tokens, 101);
    assert_eq!(backend.interrupt_calls(), 1);
    let events = state.events_page(&run_id, None, 1_000).unwrap().events;
    let exceeded = events
        .iter()
        .position(|event| event.event_type == "turn_usage_reconciled")
        .unwrap();
    let interrupt_dispatch = events
        .iter()
        .position(|event| event.event_type == "cli_interrupt_prepared")
        .unwrap();
    assert!(exceeded < interrupt_dispatch);
}

#[tokio::test]
async fn restart_charges_wall_downtime_and_interrupts_without_new_semantic_work() {
    let directory = private_directory("harp-wall-restart-");
    let artifacts = open_artifacts(&directory);
    let backend_path = directory.path().join("fake-runtime.json");
    let backend = PersistentFakeBackend::open_file(&backend_path, outcomes(&artifacts)).unwrap();
    let state_path = directory.path().join("state.sqlite");
    let clock = Arc::new(TestWallClock::new(1_000));
    let lease_clock = Arc::new(TestLeaseClock::new(10));
    let run_id = {
        let mut graph = graph();
        graph.nodes[0].budget.timeout_seconds = 1;
        let (graph_policy, projection_policy) = policies(&graph);
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let spec = RunExecutionSpec::new(
            validate_graph(graph, &graph_policy, &projection_policy).unwrap(),
            graph_policy,
            projection_policy,
            &artifacts,
            &runtime.provenance().unwrap(),
        )
        .unwrap();
        let mut engine = Engine::with_clocks(
            config(Some(CrashPoint::AfterDispatchingTurn), 10),
            clock.clone(),
            lease_clock.clone(),
        )
        .unwrap();
        assert!(matches!(
            engine
                .execute_run(&mut state, &artifacts, &mut runtime, &spec)
                .await,
            Err(EngineError::InjectedCrash {
                point: CrashPoint::AfterDispatchingTurn
            })
        ));
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 1);

    clock.set(1_002);
    lease_clock.set(71);
    let reopened_backend =
        PersistentFakeBackend::open_file(&backend_path, BTreeMap::new()).unwrap();
    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = reopened_backend.runtime();
    let mut engine = Engine::with_clocks(config(None, 1_000), clock.clone(), lease_clock).unwrap();
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("downtime exhausts the persisted task wall budget");

    assert!(
        matches!(error, EngineError::BudgetExceeded { .. }),
        "{error:?}"
    );
    assert_eq!(reopened_backend.start_thread_calls(), 0);
    assert_eq!(reopened_backend.start_turn_calls(), 0);
    assert_eq!(reopened_backend.interrupt_calls(), 0);
    let attempt = state.attempts(&run_id).unwrap().remove(0);
    assert!(attempt.observed_wall_seconds >= 2);
    assert_eq!(attempt.state, AttemptState::Failed);
}

#[tokio::test]
async fn backward_wall_clock_terminalizes_attempt_without_model_work() {
    let directory = private_directory("harp-wall-rollback-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let wall_clock = Arc::new(TestWallClock::new(2_000));
    let lease_clock = Arc::new(TestLeaseClock::new(1_000));
    let run_id = {
        let mut state =
            StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = Engine::with_clocks(
            config(Some(CrashPoint::DuringTurn), 10),
            wall_clock.clone(),
            lease_clock.clone(),
        )
        .unwrap();
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash after wall observation");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    lease_clock.set(1_061);
    wall_clock.set(1_999);
    let calls = (backend.start_thread_calls(), backend.start_turn_calls());

    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = Engine::with_clocks(config(None, 20), wall_clock, lease_clock).unwrap();
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("backward wall clock fails closed");

    assert!(matches!(error, EngineError::WallClockIntegrity { .. }));
    assert_eq!(
        state.attempts(&run_id).unwrap()[0].state,
        AttemptState::Failed
    );
    assert_eq!(
        (backend.start_thread_calls(), backend.start_turn_calls()),
        calls
    );
}

#[tokio::test]
async fn large_forward_wall_jump_exhausts_without_overflow() {
    let directory = private_directory("harp-wall-forward-jump-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let wall_clock = Arc::new(TestWallClock::new(2_000));
    let lease_clock = Arc::new(TestLeaseClock::new(1_000));
    let run_id = {
        let mut state =
            StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = Engine::with_clocks(
            config(Some(CrashPoint::DuringTurn), 10),
            wall_clock.clone(),
            lease_clock.clone(),
        )
        .unwrap();
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash after wall observation");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    lease_clock.set(1_061);
    wall_clock.set(i64::MAX);

    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = Engine::with_clocks(config(None, 20), wall_clock, lease_clock).unwrap();
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("large forward jump exhausts wall budget");

    assert!(matches!(error, EngineError::BudgetExceeded { .. }));
    assert_eq!(
        state.attempts(&run_id).unwrap()[0].state,
        AttemptState::Failed
    );
}

#[tokio::test]
async fn recovered_terminal_wall_overage_fails_without_interrupting_terminal_turn() {
    let directory = private_directory("harp-wall-terminal-restart-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let clock = Arc::new(TestWallClock::new(2_000));
    let lease_clock = Arc::new(TestLeaseClock::new(10));
    let run_id = {
        let mut graph = graph();
        graph.nodes[0].budget.timeout_seconds = 1;
        let (graph_policy, projection_policy) = policies(&graph);
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let spec = RunExecutionSpec::new(
            validate_graph(graph, &graph_policy, &projection_policy).unwrap(),
            graph_policy,
            projection_policy,
            &artifacts,
            &runtime.provenance().unwrap(),
        )
        .unwrap();
        let mut engine = Engine::with_clocks(
            config(Some(CrashPoint::AfterDispatchingTurn), 10),
            clock.clone(),
            lease_clock.clone(),
        )
        .unwrap();
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
            .expect_err("crash after turn creation");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    backend.complete_all_turns();
    clock.set(2_002);
    lease_clock.set(71);

    let mut state = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = Engine::with_clocks(config(None, 1_000), clock.clone(), lease_clock).unwrap();
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("terminal turn exceeds persisted wall budget");

    assert!(
        matches!(error, EngineError::BudgetExceeded { .. }),
        "{error:?}"
    );
    assert_eq!(backend.interrupt_calls(), 0);
    assert!(state.accepted_results(&run_id).unwrap().is_empty());
    assert_eq!(
        state.attempts(&run_id).unwrap()[0].state,
        AttemptState::Failed
    );
}

#[tokio::test]
async fn restart_replays_dispatched_wall_limit_interrupt_before_finalizing_failure() {
    let directory = private_directory("harp-wall-interrupt-restart-");
    let artifacts = open_artifacts(&directory);
    let backend = PersistentFakeBackend::new(outcomes(&artifacts));
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::DuringTurn), 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash during active turn");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    let mut state = StateStore::open(&state_path).unwrap();
    let attempt = state.attempts(&run_id).unwrap().remove(0);
    let expired_at = attempt.lease_expires_at.unwrap();
    state
        .set_lease_clock(Arc::new(TestLeaseClock::new(expired_at)))
        .unwrap();
    state.expire_leases(&run_id, expired_at).unwrap();
    let lease = state
        .reclaim_attempt(
            &attempt.attempt_id,
            "limit-worker",
            expired_at,
            expired_at + 1,
            expired_at + 100,
        )
        .unwrap();
    state
        .reconcile_wall_usage(
            &lease,
            attempt.observed_wall_seconds,
            61,
            1_000,
            expired_at + 2,
        )
        .unwrap();
    let intent = harp_state::InterruptIntent::budget(
        harp_state::InterruptBudgetDimension::Wall,
        "wall_limit",
    )
    .unwrap();
    let target = state
        .recoverable_activities(&run_id)
        .unwrap()
        .into_iter()
        .find(|activity| {
            matches!(
                activity.kind,
                harp_state::CliActivityKind::StartActivity
                    | harp_state::CliActivityKind::ContinueActivity
            )
        })
        .unwrap();
    let target_process_record_sha256 = target.process_record_sha256.clone().unwrap();
    let interrupt = state
        .prepare_interrupt_activity(
            &lease,
            harp_contracts::OperationId::new(),
            0,
            &harp_state::ActivityPreparation::new(
                format!("/private/tmp/harp-interrupt-{}", target.activity_id),
                target_process_record_sha256.clone(),
            )
            .unwrap(),
            &target_process_record_sha256,
            intent.purpose,
            expired_at + 3,
        )
        .unwrap();
    drop(state);

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, expired_at + 200));
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("dispatched wall-limit interrupt is reconciled");

    assert!(
        matches!(error, EngineError::BudgetExceeded { .. }),
        "{error:?}"
    );
    assert_eq!(backend.interrupt_calls(), 1);
    assert_eq!(
        state.attempts(&run_id).unwrap()[0].state,
        AttemptState::Failed
    );
    assert_eq!(
        state
            .get_cli_activity(&interrupt.activity_id)
            .unwrap()
            .unwrap()
            .state,
        harp_state::CliActivityState::Completed
    );
}

#[tokio::test]
async fn active_checkpoint_and_evidence_growth_interrupts_before_completion() {
    let directory = private_directory("harp-active-storage-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.insert(
        "alpha".to_owned(),
        VecDeque::from([FakeOutcome {
            status: TurnStatus::Completed,
            final_message: Some(result_message(&artifacts, "alpha", 8)),
            total_tokens: 8,
            token_snapshots: vec![3, 8],
        }]),
    );
    let backend = PersistentFakeBackend::new(scripted);
    let mut graph = graph();
    graph.nodes[0].budget.max_storage_bytes = 256;
    let (graph_policy, projection_policy) = {
        let mut graph_policy = execution_spec(&artifacts, &backend).graph_policy().clone();
        graph_policy.max_node_storage_bytes = 1_024;
        let projection_policy = execution_spec(&artifacts, &backend)
            .projection_policy()
            .clone();
        (graph_policy, projection_policy)
    };
    let validated = validate_graph(graph, &graph_policy, &projection_policy).unwrap();
    let runtime_for_spec = backend.runtime();
    let spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime_for_spec.provenance().unwrap(),
    )
    .unwrap();
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::DuringTurn), 10));
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await
            .expect_err("crash after first token snapshot");
        let run_id = state.incomplete_runs().unwrap()[0].run_id.clone();
        let attempt = state.attempts(&run_id).unwrap()[0].clone();
        let mut checkpoint = Checkpoint::new(
            run_id.clone(),
            attempt.task_id.clone(),
            attempt.attempt_id.clone(),
            "large-progress",
            100,
        )
        .unwrap();
        checkpoint.completed_units = vec!["x".repeat(220)];
        artifacts.write_checkpoint(&checkpoint).unwrap();
        artifacts
            .append_evidence(
                &harp_artifacts::AttemptKey {
                    run_id: run_id.clone(),
                    task_id: attempt.task_id,
                    attempt_id: attempt.attempt_id,
                },
                &serde_json::json!({"claim": "y".repeat(220)}),
            )
            .unwrap();
        run_id
    };

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("active storage growth exceeds task limit");

    assert!(matches!(
        error,
        harp_engine::EngineError::Artifact(
            harp_artifacts::ArtifactError::StoragePolicyViolation { .. }
        )
    ));
    assert_eq!(backend.interrupt_calls(), 1);
    let attempt = state.attempts(&run_id).unwrap()[0].clone();
    let interrupt_count: i64 = rusqlite::Connection::open(state_path.clone())
        .unwrap()
        .query_row(
            "SELECT COUNT(*)
                 FROM cli_activities
                 WHERE attempt_id = ?1
                   AND kind = 'interrupt_activity'
                   AND interrupt_purpose = 'budget'
                   AND signal_stage = 'quiescent'",
            [attempt.attempt_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(interrupt_count, 1);
    assert!(
        state
            .reservation_usage(&attempt.attempt_id)
            .unwrap()
            .unwrap()
            .1
            > 256
    );
}

#[tokio::test]
async fn active_hardlink_storage_violation_interrupts_and_terminalizes() {
    let directory = private_directory("harp-active-hardlink-");
    let artifacts = open_artifacts(&directory);
    let mut scripted = outcomes(&artifacts);
    scripted.get_mut("alpha").unwrap()[0].token_snapshots = vec![1, 8];
    let backend = PersistentFakeBackend::new(scripted);
    let state_path = directory.path().join("state.sqlite");
    let run_id = {
        let mut state = StateStore::open(&state_path).unwrap();
        let mut runtime = backend.runtime();
        let mut engine = fixed_clock_engine(config(Some(CrashPoint::DuringTurn), 10));
        engine
            .execute_run(
                &mut state,
                &artifacts,
                &mut runtime,
                &execution_spec(&artifacts, &backend),
            )
            .await
            .expect_err("crash during active turn");
        state.incomplete_runs().unwrap()[0].run_id.clone()
    };
    let attempt = StateStore::open(&state_path)
        .unwrap()
        .attempts(&run_id)
        .unwrap()[0]
        .clone();
    let attempt_dir = artifacts
        .attempt_dir(&harp_artifacts::AttemptKey {
            run_id: run_id.clone(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
        })
        .unwrap();
    fs::write(attempt_dir.join("source.bin"), b"hardlink").unwrap();
    fs::hard_link(
        attempt_dir.join("source.bin"),
        attempt_dir.join("linked.bin"),
    )
    .unwrap();

    let mut state = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = fixed_clock_engine(config(None, 1_000));
    let error = engine
        .resume_run(&mut state, &artifacts, &mut runtime, &run_id)
        .await
        .expect_err("active hardlink violation fails closed");

    assert!(matches!(
        error,
        EngineError::Artifact(harp_artifacts::ArtifactError::StoragePolicyViolation { .. })
    ));
    assert_eq!(backend.interrupt_calls(), 1);
    let attempt = state.attempts(&run_id).unwrap()[0].clone();
    assert_eq!(attempt.state, AttemptState::Failed);
    let interrupt_count: i64 = rusqlite::Connection::open(state_path.clone())
        .unwrap()
        .query_row(
            "SELECT COUNT(*)
                 FROM cli_activities
                 WHERE attempt_id = ?1
                   AND kind = 'interrupt_activity'
                   AND interrupt_purpose = 'scanner_integrity'
                   AND signal_stage = 'quiescent'",
            [attempt.attempt_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(interrupt_count, 1);
}
