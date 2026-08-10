#![cfg(unix)]

mod common;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use common::{FakeOutcome, PersistentFakeBackend};
use harp_artifacts::{ArtifactStore, AttemptKey};
use harp_contracts::{
    ArtifactRef, Budget, NodeKind, ResultEnvelope, ResultStatus, RetryPolicy, TaskGraph, TaskId,
    TaskNode, TaskRole, TurnStatus, WorkspaceMode,
};
use harp_engine::{
    validate_graph, Engine, EngineConfig, GraphPolicy, ProjectionPolicy, RunExecutionSpec,
};
use harp_runtime::ActivityRuntime;
use harp_state::StateStore;
use tempfile::TempDir;

#[derive(Debug)]
struct TestExecutionLeaseClock(i64);

impl TestExecutionLeaseClock {
    fn new(now: i64) -> Self {
        Self(now)
    }
}

impl harp_state::LeaseClock for TestExecutionLeaseClock {
    fn unix_seconds(&self) -> harp_state::StateResult<i64> {
        Ok(self.0)
    }
}

#[derive(Debug)]
struct TestExecutionWallClock;

impl harp_engine::WallClock for TestExecutionWallClock {
    fn unix_seconds(&self) -> Result<i64, harp_engine::EngineError> {
        Ok(1_000)
    }

    fn monotonic_elapsed(&self) -> Duration {
        Duration::ZERO
    }
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
        budget: Budget::new(100, 60, 2_048),
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
        max_node_storage_bytes: 2_048,
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
    let projection = ProjectionPolicy {
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
    (graph_policy, projection)
}

fn publish_support_artifacts(store: &ArtifactStore, task_id: &str) -> (ArtifactRef, ArtifactRef) {
    let answer = store
        .publish(format!("answer for {task_id}").as_bytes(), "text/plain")
        .unwrap();
    let trace = store
        .publish(
            format!("trace for {task_id}\n").as_bytes(),
            "application/jsonl",
        )
        .unwrap();
    (answer, trace)
}

fn result_message(store: &ArtifactStore, task_id: &str, token_usage: u64) -> String {
    let (answer_ref, trace_ref) = publish_support_artifacts(store, task_id);
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

#[tokio::test]
async fn executes_two_analysis_tasks_and_one_reducer() {
    let directory = private_directory("harp-engine-execution-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(&artifacts, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let backend = PersistentFakeBackend::new(outcomes);
    backend.enable_scratch_writes();
    let mut runtime = backend.runtime();
    let graph = graph();
    let (graph_policy, projection_policy) = policies(&graph);
    let validated =
        validate_graph(graph, &graph_policy, &projection_policy).expect("valid execution graph");
    let execution_spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .expect("execution spec");
    let state_path = directory.path().join("state.sqlite");
    let mut state = StateStore::open(&state_path).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "reference-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 2,
        logical_time: 10,
        crash_point: None,
    })
    .unwrap();

    let summary = engine
        .execute_run(&mut state, &artifacts, &mut runtime, &execution_spec)
        .await
        .expect("uninterrupted execution");

    assert_eq!(summary.completed_tasks, 3);
    assert_eq!(summary.indeterminate_attempts, 0);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 3);
    assert_eq!(backend.interrupt_calls(), 0);
    assert_eq!(
        state
            .accepted_results(&summary.run_id)
            .expect("accepted results")
            .len(),
        3
    );
    assert_eq!(
        state
            .tasks(&summary.run_id)
            .unwrap()
            .into_iter()
            .map(|task| task.task_id.to_string())
            .collect::<BTreeSet<_>>(),
        ["alpha", "beta", "reduce"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
    for attempt in state.attempts(&summary.run_id).unwrap() {
        let operations = state.operations_for_attempt(&attempt.attempt_id).unwrap();
        assert_eq!(
            operations
                .iter()
                .filter(|operation| operation.kind == harp_state::OperationKind::PublishResult)
                .count(),
            1
        );
        assert!(
            operations.iter().all(|operation| {
                !matches!(
                    operation.kind,
                    harp_state::OperationKind::StartThread
                        | harp_state::OperationKind::StartTurn
                        | harp_state::OperationKind::ContinueTurn
                        | harp_state::OperationKind::InterruptTurn
                )
            }),
            "new CLI attempts must not create legacy thread/turn operations"
        );
        let activities = state.recoverable_activities(&summary.run_id).unwrap();
        assert!(
            activities
                .iter()
                .all(|activity| activity.attempt_id != attempt.attempt_id),
            "completed CLI activity must not remain recoverable"
        );
        let key = AttemptKey {
            run_id: attempt.run_id.clone(),
            task_id: attempt.task_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
        };
        let resolved = artifacts.resolve_attempt_scratch(&key).unwrap();
        assert_eq!(
            attempt.scratch_path.as_deref(),
            resolved.canonical_path().to_str()
        );
        assert_eq!(attempt.scratch_device, Some(resolved.device()));
        assert_eq!(attempt.scratch_inode, Some(resolved.inode()));
        let scratch_usage = artifacts.attempt_storage_usage(&key, 2_049).unwrap();
        assert!(scratch_usage.checkpoint_bytes > 0);
        assert!(scratch_usage.evidence_bytes > 0);
        assert_eq!(scratch_usage.notes_bytes, 13);
        assert_eq!(scratch_usage.result_scratch_bytes, 22);
        assert!(
            u128::from(
                state
                    .reservation_usage(&attempt.attempt_id)
                    .unwrap()
                    .unwrap()
                    .1,
            ) >= scratch_usage.total_bytes().unwrap()
        );
    }
}

#[tokio::test]
async fn repeated_token_snapshots_are_accounted_cumulatively_not_added_twice() {
    let directory = private_directory("harp-engine-token-snapshots-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(&artifacts, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: vec![3, 8],
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let backend = PersistentFakeBackend::new(outcomes);
    let mut runtime = backend.runtime();
    let graph = graph();
    let (graph_policy, projection_policy) = policies(&graph);
    let validated =
        validate_graph(graph, &graph_policy, &projection_policy).expect("valid execution graph");
    let execution_spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .expect("execution spec");
    let state_path = directory.path().join("state.sqlite");
    let mut state = StateStore::open(&state_path).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "snapshot-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 2,
        logical_time: 10,
        crash_point: None,
    })
    .unwrap();

    let summary = engine
        .execute_run(&mut state, &artifacts, &mut runtime, &execution_spec)
        .await
        .expect("cumulative snapshots execute");

    for attempt in state.attempts(&summary.run_id).unwrap() {
        assert_eq!(attempt.observed_tokens, 8);
        assert_eq!(attempt.latest_turn_observed_tokens, 8);
    }
}

#[tokio::test]
async fn live_completion_rejects_result_that_violates_pinned_output_schema() {
    let directory = private_directory("harp-engine-output-schema-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(&artifacts, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let backend = PersistentFakeBackend::new(outcomes);
    let mut runtime = backend.runtime();
    let mut graph = graph();
    graph.nodes[0].output_schema = serde_json::json!({
        "type": "object",
        "required": ["mustNotExist"],
        "properties": {"mustNotExist": {"const": true}}
    })
    .to_string();
    let (mut graph_policy, projection_policy) = policies(&graph);
    graph_policy
        .allowed_output_schemas
        .insert(graph.nodes[0].output_schema.clone());
    let validated = validate_graph(graph, &graph_policy, &projection_policy).unwrap();
    let spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap();
    let state_path = directory.path().join("state.sqlite");
    let mut state = StateStore::open(&state_path).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "schema-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 2,
        logical_time: 10,
        crash_point: None,
    })
    .unwrap();

    let error = engine
        .execute_run(&mut state, &artifacts, &mut runtime, &spec)
        .await
        .expect_err("live result violates pinned schema");

    assert!(matches!(
        error,
        harp_engine::EngineError::OutputSchema { .. }
    ));
    assert_eq!(engine.test_only_active_lease_count(), 0);
    let connection = rusqlite::Connection::open(&state_path).unwrap();
    let run_id_text: String = connection
        .query_row("SELECT run_id FROM runs LIMIT 1", [], |row| row.get(0))
        .unwrap();
    let run_id = harp_contracts::RunId::from_str(&run_id_text).unwrap();
    assert!(state.accepted_results(&run_id).unwrap().is_empty());
    assert_eq!(
        state.attempts(&run_id).unwrap()[0].state,
        harp_state::AttemptState::Failed
    );
    assert_eq!(
        state.get_run(&run_id).unwrap().unwrap().state,
        harp_state::RunState::Failed
    );
    drop(state);

    let mut reopened = StateStore::open(&state_path).unwrap();
    let mut resumed_runtime = backend.runtime();
    let mut resumed = Engine::new(EngineConfig {
        worker_id: "schema-restart".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 1,
        logical_time: 1_000,
        crash_point: None,
    })
    .unwrap();
    resumed
        .resume_run(&mut reopened, &artifacts, &mut resumed_runtime, &run_id)
        .await
        .expect("failed semantic result restart is inert");
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 1);
}

#[tokio::test]
async fn terminal_discovered_storage_overage_fails_without_interrupt() {
    let directory = private_directory("harp-terminal-storage-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(&artifacts, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let backend = PersistentFakeBackend::new(outcomes);
    let mut runtime = backend.runtime();
    let mut graph = graph();
    graph.nodes[0].budget.max_storage_bytes = 64;
    let (mut graph_policy, projection_policy) = policies(&graph);
    graph_policy.max_node_storage_bytes = 2_048;
    let validated = validate_graph(graph, &graph_policy, &projection_policy).unwrap();
    let spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap();
    let mut state = StateStore::open(&directory.path().join("state.sqlite")).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "terminal-storage-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 2,
        logical_time: 10,
        crash_point: None,
    })
    .unwrap();

    let error = engine
        .execute_run(&mut state, &artifacts, &mut runtime, &spec)
        .await
        .expect_err("terminal result exceeds storage");

    assert!(
        matches!(error, harp_engine::EngineError::BudgetExceeded { .. }),
        "{error:?}"
    );
    assert_eq!(backend.interrupt_calls(), 0);
    let run_id = state.incomplete_runs().unwrap_or_default();
    assert!(run_id.is_empty(), "budget exhaustion fails the run");
}

#[tokio::test(start_paused = true)]
async fn active_wall_timeout_persists_interrupt_before_runtime_interrupt_and_restart_is_quiet() {
    let directory = private_directory("harp-active-wall-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(&artifacts, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let backend = PersistentFakeBackend::new(outcomes);
    backend.set_next_event_delay(Duration::from_secs(2));
    let mut runtime = backend.runtime();
    let mut graph = graph();
    graph.nodes[0].budget.timeout_seconds = 1;
    let (graph_policy, projection_policy) = policies(&graph);
    let validated = validate_graph(graph, &graph_policy, &projection_policy).unwrap();
    let spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap();
    let state_path = directory.path().join("state.sqlite");
    let mut state = StateStore::open(&state_path).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "wall-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 1,
        logical_time: 10,
        crash_point: None,
    })
    .unwrap();

    let error = engine
        .execute_run(&mut state, &artifacts, &mut runtime, &spec)
        .await
        .expect_err("active turn exceeds wall budget");
    assert!(
        matches!(error, harp_engine::EngineError::BudgetExceeded { .. }),
        "{error:?}"
    );
    assert_eq!(backend.interrupt_calls(), 1);
    let connection = rusqlite::Connection::open(&state_path).unwrap();
    let run_id_text: String = connection
        .query_row("SELECT run_id FROM runs LIMIT 1", [], |row| row.get(0))
        .unwrap();
    let run_id = harp_contracts::RunId::from_str(&run_id_text).unwrap();
    let events = state.events_page(&run_id, None, 1000).unwrap().events;
    let event_types = events
        .into_iter()
        .map(|event| event.event_type)
        .collect::<Vec<_>>();
    let interrupt_prepared = event_types
        .iter()
        .position(|event| event == "cli_interrupt_prepared")
        .expect("CLI interrupt intent persisted");
    let quiescent = event_types
        .iter()
        .rposition(|event| event == "cli_signal_stage_advanced")
        .expect("CLI interrupt signal ledger persisted");
    assert!(interrupt_prepared < quiescent);

    drop(state);
    let mut reopened = StateStore::open(&state_path).unwrap();
    let mut resumed_runtime = backend.runtime();
    let mut resumed_engine = Engine::new(EngineConfig {
        worker_id: "restart-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 1,
        logical_time: 100,
        crash_point: None,
    })
    .unwrap();
    let summary = resumed_engine
        .resume_run(&mut reopened, &artifacts, &mut resumed_runtime, &run_id)
        .await
        .expect("failed run resumes without semantic work");
    assert_eq!(summary.completed_tasks, 0);
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
    assert_eq!(backend.start_activity_calls(), 1);
}

#[tokio::test]
async fn tampered_persisted_scratch_identity_fails_before_runtime_work() {
    let directory = private_directory("harp-scratch-tamper-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(&artifacts, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let backend = PersistentFakeBackend::new(outcomes);
    let mut runtime = backend.runtime();
    let graph = graph();
    let (graph_policy, projection_policy) = policies(&graph);
    let validated = validate_graph(graph, &graph_policy, &projection_policy).unwrap();
    let spec = RunExecutionSpec::new(
        validated,
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap();
    let state_path = directory.path().join("state.sqlite");
    let mut state = StateStore::open(&state_path).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "scratch-worker".to_owned(),
        lease_seconds: 10,
        lease_renewal_threshold_seconds: 3,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 1,
        logical_time: 10,
        crash_point: Some(harp_engine::CrashPoint::AfterPrepared),
    })
    .unwrap();
    assert!(matches!(
        engine
            .execute_run(&mut state, &artifacts, &mut runtime, &spec)
            .await,
        Err(harp_engine::EngineError::InjectedCrash { .. })
    ));
    assert_eq!(backend.start_thread_calls(), 0);
    drop(state);

    let connection = rusqlite::Connection::open(&state_path).unwrap();
    let run_id_text: String = connection
        .query_row("SELECT run_id FROM runs LIMIT 1", [], |row| row.get(0))
        .unwrap();
    let lease_expires_at: i64 = connection
        .query_row("SELECT lease_expires_at FROM attempts LIMIT 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    connection
        .execute(
            "UPDATE attempts SET scratch_path = '/private/tmp/not-the-artifact-attempt'",
            [],
        )
        .unwrap();
    drop(connection);

    let run_id = harp_contracts::RunId::from_str(&run_id_text).unwrap();
    let lease_clock = Arc::new(TestExecutionLeaseClock::new(lease_expires_at + 1));
    let mut reopened = StateStore::open_with_lease_clock(&state_path, lease_clock.clone()).unwrap();
    let mut resumed_runtime = backend.runtime();
    let mut resumed = Engine::with_clocks(
        EngineConfig {
            worker_id: "scratch-recovery".to_owned(),
            lease_seconds: 10,
            lease_renewal_threshold_seconds: 3,
            runtime_operation_timeout_seconds: 30,
            max_concurrency: 1,
            logical_time: 100,
            crash_point: None,
        },
        Arc::new(TestExecutionWallClock),
        lease_clock,
    )
    .unwrap();
    let error = resumed
        .resume_run(&mut reopened, &artifacts, &mut resumed_runtime, &run_id)
        .await
        .expect_err("persisted scratch mismatch is rejected");
    assert!(matches!(
        error,
        harp_engine::EngineError::State(harp_state::StateError::Conflict { .. })
    ));
    assert_eq!(backend.start_thread_calls(), 0);
    assert_eq!(backend.start_turn_calls(), 0);
}

#[tokio::test]
async fn visible_artifact_root_substitution_fails_before_start_thread() {
    let directory = private_directory("harp-visible-root-substitution-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(&artifacts, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let backend = PersistentFakeBackend::new(outcomes);
    let mut runtime = backend.runtime();
    let graph = graph();
    let (graph_policy, projection_policy) = policies(&graph);
    let spec = RunExecutionSpec::new(
        validate_graph(graph, &graph_policy, &projection_policy).unwrap(),
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap();
    let mut state = StateStore::open(&directory.path().join("state.sqlite")).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "substitution-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 1,
        logical_time: 10,
        crash_point: None,
    })
    .unwrap();
    let moved = directory.path().join("artifacts-held");
    let replacement = artifact_root.clone();
    engine.test_only_before_thread_start(move || {
        fs::rename(&replacement, &moved).unwrap();
        fs::create_dir(&replacement).unwrap();
        fs::set_permissions(&replacement, fs::Permissions::from_mode(0o700)).unwrap();
    });

    let error = engine
        .execute_run(&mut state, &artifacts, &mut runtime, &spec)
        .await
        .expect_err("visible root substitution is rejected");

    assert!(matches!(error, harp_engine::EngineError::Artifact(_)));
    assert_eq!(backend.start_thread_calls(), 0);
}

#[tokio::test]
async fn runtime_rejects_substitution_after_engine_verification_before_external_effect() {
    let directory = private_directory("harp-runtime-root-substitution-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some(result_message(&artifacts, task_id, 8)),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let moved = directory.path().join("artifacts-held");
    let replacement = artifact_root.clone();
    let backend = PersistentFakeBackend::new(outcomes).with_before_workspace_verify_hook(Arc::new(
        move || {
            fs::rename(&replacement, &moved).unwrap();
            fs::create_dir(&replacement).unwrap();
            fs::set_permissions(&replacement, fs::Permissions::from_mode(0o700)).unwrap();
        },
    ));
    let mut runtime = backend.runtime();
    let graph = graph();
    let (graph_policy, projection_policy) = policies(&graph);
    let spec = RunExecutionSpec::new(
        validate_graph(graph, &graph_policy, &projection_policy).unwrap(),
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap();
    let mut state = StateStore::open(&directory.path().join("state.sqlite")).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "runtime-substitution-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 1,
        logical_time: 10,
        crash_point: None,
    })
    .unwrap();

    let error = engine
        .execute_run(&mut state, &artifacts, &mut runtime, &spec)
        .await
        .expect_err("runtime path substitution is rejected inside adapter call");

    assert!(matches!(error, harp_engine::EngineError::Runtime(_)));
    assert_eq!(backend.start_thread_calls(), 0);
}

#[tokio::test]
async fn invalid_output_with_symlink_terminalizes_storage_integrity_and_resume_is_inert() {
    use std::os::unix::fs::symlink;

    let directory = private_directory("harp-invalid-output-symlink-");
    let artifact_root = directory.path().join("artifacts");
    fs::create_dir(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let outcomes = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| {
            (
                task_id.to_owned(),
                VecDeque::from([FakeOutcome {
                    status: TurnStatus::Completed,
                    final_message: Some("{\"invalid\":true}".to_owned()),
                    total_tokens: 8,
                    token_snapshots: Vec::new(),
                }]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let hook_calls = Arc::new(AtomicUsize::new(0));
    let backend = PersistentFakeBackend::new(outcomes).with_next_event_hook(Arc::new({
        let artifact_root = artifact_root.clone();
        let hook_calls = hook_calls.clone();
        move || {
            if hook_calls.fetch_add(1, Ordering::SeqCst) < 2 {
                return;
            }
            let attempts = walkdir::WalkDir::new(&artifact_root)
                .into_iter()
                .filter_map(Result::ok)
                .find(|entry| {
                    entry.file_type().is_dir()
                        && entry
                            .path()
                            .parent()
                            .and_then(|parent| parent.file_name())
                            .is_some_and(|name| name == "attempts")
                });
            if let Some(attempt) = attempts {
                let link = attempt.path().join("bad-link");
                if !link.exists() {
                    symlink("/private/tmp", link).unwrap();
                }
            }
        }
    }));
    let mut runtime = backend.runtime();
    let graph = graph();
    let (graph_policy, projection_policy) = policies(&graph);
    let spec = RunExecutionSpec::new(
        validate_graph(graph, &graph_policy, &projection_policy).unwrap(),
        graph_policy,
        projection_policy,
        &artifacts,
        &runtime.provenance().unwrap(),
    )
    .unwrap();
    let state_path = directory.path().join("state.sqlite");
    let mut state = StateStore::open(&state_path).unwrap();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "invalid-storage-worker".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 1,
        logical_time: 10,
        crash_point: None,
    })
    .unwrap();

    let error = engine
        .execute_run(&mut state, &artifacts, &mut runtime, &spec)
        .await
        .expect_err("scanner policy violation overrides invalid output return");

    assert!(matches!(
        error,
        harp_engine::EngineError::Artifact(
            harp_artifacts::ArtifactError::StoragePolicyViolation { .. }
        )
    ));
    let connection = rusqlite::Connection::open(&state_path).unwrap();
    let run_id_text: String = connection
        .query_row("SELECT run_id FROM runs LIMIT 1", [], |row| row.get(0))
        .unwrap();
    let run_id = harp_contracts::RunId::from_str(&run_id_text).unwrap();
    assert_eq!(
        state.attempts(&run_id).unwrap()[0].state,
        harp_state::AttemptState::Failed
    );
    let calls = (backend.start_thread_calls(), backend.start_turn_calls());
    drop(state);

    let mut reopened = StateStore::open(&state_path).unwrap();
    let mut runtime = backend.runtime();
    let mut engine = Engine::new(EngineConfig {
        worker_id: "invalid-storage-restart".to_owned(),
        lease_seconds: 60,
        lease_renewal_threshold_seconds: 20,
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 1,
        logical_time: 1_000,
        crash_point: None,
    })
    .unwrap();
    engine
        .resume_run(&mut reopened, &artifacts, &mut runtime, &run_id)
        .await
        .expect("terminal storage integrity restart is inert");
    assert_eq!(
        (backend.start_thread_calls(), backend.start_turn_calls()),
        calls
    );
}
