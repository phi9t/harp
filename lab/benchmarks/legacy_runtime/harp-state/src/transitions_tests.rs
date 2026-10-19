#![cfg(unix)]

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Barrier, Mutex, OnceLock};
use std::thread;

use harp_contracts::{
    ArtifactRef, AttemptId, Budget, ExternalSessionId, NodeKind, OperationId, ResultEnvelope,
    ResultStatus, RetryPolicy, RunId, TaskGraph, TaskId, TaskNode, TaskRole, ThreadId, TurnId,
    TurnSpec, WorkspaceMode,
};
use harp_state::{
    AcceptResult, ActivityPreparation, AttemptState, CliActivityKind, CliActivityState,
    CliAttemptState, CliSignalStage, CliTerminalFailureClass, InterruptPurpose, LeaseToken,
    OperationKind, OperationState, RunBudget, RunState, StateError, StateStore, TaskState,
    UsageOutcome,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

#[derive(Debug)]
struct TestLeaseClock(AtomicI64);

impl harp_state::LeaseClock for TestLeaseClock {
    fn unix_seconds(&self) -> harp_state::StateResult<i64> {
        Ok(self.0.load(Ordering::SeqCst))
    }
}

fn open_state(path: &Path) -> harp_state::StateResult<StateStore> {
    let clock = test_clocks()
        .lock()
        .unwrap()
        .entry(path.to_path_buf())
        .or_insert_with(|| Arc::new(TestLeaseClock(AtomicI64::new(0))))
        .clone();
    StateStore::open_with_lease_clock(path, clock)
}

fn test_clocks() -> &'static Mutex<HashMap<PathBuf, Arc<TestLeaseClock>>> {
    static CLOCKS: OnceLock<Mutex<HashMap<PathBuf, Arc<TestLeaseClock>>>> = OnceLock::new();
    CLOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn set_test_lease_now(path: &Path, now: i64) {
    test_clocks()
        .lock()
        .unwrap()
        .get(path)
        .expect("test state clock")
        .0
        .store(now, Ordering::SeqCst);
}

fn private_directory() -> TempDir {
    let directory = tempfile::Builder::new()
        .prefix("harp-state-")
        .tempdir_in("/private/tmp")
        .expect("temporary directory");
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .expect("private directory");
    directory
}

fn state_path(directory: &TempDir) -> PathBuf {
    directory.path().join("state.sqlite")
}

fn task(task_id: &str, dependencies: &[&str]) -> TaskNode {
    TaskNode {
        task_id: TaskId::from_str(task_id).expect("task id"),
        kind: NodeKind::Analysis,
        role: TaskRole::Explore,
        instruction: format!("execute {task_id}"),
        dependencies: dependencies
            .iter()
            .map(|dependency| TaskId::from_str(dependency).expect("dependency id"))
            .collect(),
        inputs: Vec::new(),
        workspace_mode: WorkspaceMode::Scratch,
        model_policy: "default".to_owned(),
        permission_profile: "isolated".to_owned(),
        budget: Budget::new(100, 60, 1_024),
        output_schema: "{}".to_owned(),
        retry_policy: RetryPolicy {
            max_transient_attempts: 1,
        },
    }
}

fn graph(nodes: Vec<TaskNode>) -> TaskGraph {
    TaskGraph {
        schema_version: 1,
        nodes,
    }
}

fn task_with_budget(
    task_id: &str,
    dependencies: &[&str],
    max_tokens: u64,
    max_storage_bytes: u64,
) -> TaskNode {
    let mut node = task(task_id, dependencies);
    node.budget = Budget::new(max_tokens, 60, max_storage_bytes);
    node
}

fn run_budget() -> RunBudget {
    RunBudget::new(1_000, 10_000, 3_600).expect("run budget")
}

fn artifact(digest_byte: u8, media_type: &str, size_bytes: u64) -> ArtifactRef {
    ArtifactRef::sha256(
        format!("{digest_byte:02x}").repeat(32),
        media_type,
        size_bytes,
    )
    .unwrap()
}

fn result(task_id: &str) -> (ResultEnvelope, ArtifactRef) {
    let result = ResultEnvelope {
        schema_version: 1,
        task_id: TaskId::from_str(task_id).unwrap(),
        status: ResultStatus::Success,
        answer_ref: Some(artifact(0xaa, "text/plain", 7)),
        evidence: vec![artifact(0xbb, "application/json", 9)],
        trace_ref: artifact(0xcc, "application/jsonl", 11),
        summary: "completed".to_owned(),
        token_usage: 42,
        confidence: Some(0.9),
        failure_class: None,
    };
    let bytes = serde_json::to_vec(&result).unwrap();
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let result_ref = ArtifactRef::sha256(
        digest,
        "application/vnd.harp.result+json",
        bytes.len() as u64,
    )
    .unwrap();
    (result, result_ref)
}

fn create_single_task_run(store: &mut StateStore, task_id: &str) -> (RunId, LeaseToken) {
    let run = store
        .create_run(
            &graph(vec![task(task_id, &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    (run.run_id, claim.lease())
}

fn claim_single_task(store: &mut StateStore, task_id: &str) -> (RunId, LeaseToken) {
    let run = store
        .create_run(
            &graph(vec![task(task_id, &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    (run.run_id, claim.lease())
}

fn claim_single_cli_task(store: &mut StateStore, task_id: &str) -> (RunId, harp_state::TaskClaim) {
    let run = store
        .create_run(
            &graph(vec![task(task_id, &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_cli_task(
            &run.run_id,
            "worker",
            10,
            100,
            ThreadId::from_str("logical-session").unwrap(),
        )
        .unwrap()
        .unwrap();
    (run.run_id, claim)
}

fn activity_preparation(name: &str) -> ActivityPreparation {
    ActivityPreparation::new(format!("/private/tmp/harp-activity-{name}"), "a".repeat(64)).unwrap()
}

fn drive_cli_activity_to_reconciling(
    store: &mut StateStore,
    claim: &harp_state::TaskClaim,
    name: &str,
    base: i64,
) -> OperationId {
    let activity_id = OperationId::new();
    store
        .prepare_activity(
            &claim.lease(),
            activity_id.clone(),
            &claim.attempt_id,
            CliActivityKind::StartActivity,
            TurnId::from_str(&activity_id.to_string()).unwrap(),
            &activity_preparation(name),
            base,
        )
        .unwrap();
    store
        .mark_activity_dispatching(&claim.lease(), &activity_id, base + 1)
        .unwrap();
    store
        .record_process(
            &claim.lease(),
            &activity_id,
            &format!("{:02x}", (base as u8).max(1)).repeat(32),
            base + 2,
        )
        .unwrap();
    store
        .mark_activity_running(&claim.lease(), &activity_id, base + 3)
        .unwrap();
    store
        .mark_activity_reconciling(&claim.lease(), &activity_id, base + 4)
        .unwrap();
    activity_id
}

fn insert_competing_attempt(
    path: &Path,
    run_id: &RunId,
    task_id: &str,
    ordinal: i64,
    observed_tokens: i64,
) -> AttemptId {
    let attempt_id = AttemptId::new();
    let connection = rusqlite::Connection::open(path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    connection
        .execute(
            "INSERT INTO attempts (
                attempt_id, run_id, task_id, ordinal, state,
                lease_owner, lease_expires_at, continuation_count,
                observed_tokens, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, 'prepared', 'other-worker', 100, 0, ?5, 2, 2)",
            rusqlite::params![
                attempt_id.to_string(),
                run_id.to_string(),
                task_id,
                ordinal,
                observed_tokens
            ],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO budget_reservations (
                attempt_id, reserved_tokens, reserved_storage_bytes,
                reserved_wall_seconds, observed_tokens,
                observed_storage_bytes, created_at, updated_at
             ) VALUES (?1, 100, 1024, 60, ?2, 77, 2, 2)",
            rusqlite::params![attempt_id.to_string(), observed_tokens],
        )
        .unwrap();
    attempt_id
}

fn insert_dispatching_competing_attempt(
    path: &Path,
    run_id: &RunId,
    task_id: &str,
    ordinal: i64,
) -> (AttemptId, harp_contracts::OperationId) {
    let attempt_id = AttemptId::new();
    let operation_id = harp_contracts::OperationId::new();
    let connection = rusqlite::Connection::open(path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    connection
        .execute(
            "INSERT INTO attempts (
                attempt_id, run_id, task_id, ordinal, state,
                lease_owner, lease_expires_at, thread_id,
                continuation_count, observed_tokens, created_at, updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4, 'dispatching_thread',
                'dispatch-worker', 100, 'dispatch-thread',
                0, 29, 2, 2
             )",
            rusqlite::params![attempt_id.to_string(), run_id.to_string(), task_id, ordinal,],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO budget_reservations (
                attempt_id, reserved_tokens, reserved_storage_bytes,
                reserved_wall_seconds, observed_tokens,
                observed_storage_bytes, created_at, updated_at
             ) VALUES (?1, 100, 1024, 60, 29, 41, 2, 2)",
            [attempt_id.to_string()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO operations (
                operation_id, attempt_id, kind, ordinal, state,
                operation_marker, created_at, updated_at
             ) VALUES (?1, ?2, 'start_thread', 0, 'dispatching', 'dispatch-marker', 2, 2)",
            rusqlite::params![operation_id.to_string(), attempt_id.to_string()],
        )
        .unwrap();
    (attempt_id, operation_id)
}

#[test]
fn open_creates_private_database_and_applies_required_pragmas() {
    let directory = private_directory();
    let path = state_path(&directory);

    let store = open_state(&path).expect("open state store");

    assert_eq!(store.path(), path);
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o600);
    assert_eq!(store.schema_version().unwrap(), 6);
    assert!(store.foreign_keys_enabled().unwrap());
    assert_eq!(store.journal_mode().unwrap(), "delete");
    assert_eq!(store.synchronous_mode().unwrap(), 2);
    assert_eq!(store.busy_timeout_millis().unwrap(), 5_000);
}

#[test]
fn open_rejects_symlink_paths_wrong_modes_and_nonregular_files() {
    let directory = private_directory();
    let target = directory.path().join("target.sqlite");
    fs::write(&target, []).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
    let link = directory.path().join("link.sqlite");
    symlink(&target, &link).unwrap();
    assert!(matches!(
        open_state(&link),
        Err(StateError::InvalidInput { .. })
    ));

    let parent_target = private_directory();
    let parent_link_root = private_directory();
    let parent_link = parent_link_root.path().join("linked-parent");
    symlink(parent_target.path(), &parent_link).unwrap();
    assert!(matches!(
        open_state(&parent_link.join("state.sqlite")),
        Err(StateError::InvalidInput { .. })
    ));

    let open_parent = private_directory();
    fs::set_permissions(open_parent.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(
        open_state(&state_path(&open_parent)),
        Err(StateError::InvalidInput { .. })
    ));

    let wrong_file = private_directory();
    let wrong_file_path = state_path(&wrong_file);
    fs::write(&wrong_file_path, []).unwrap();
    fs::set_permissions(&wrong_file_path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(
        open_state(&wrong_file_path),
        Err(StateError::InvalidInput { .. })
    ));

    let nonregular = private_directory();
    fs::create_dir(state_path(&nonregular)).unwrap();
    assert!(matches!(
        open_state(&state_path(&nonregular)),
        Err(StateError::InvalidInput { .. })
    ));
}

#[test]
fn open_rejects_newer_schema_versions() {
    let directory = private_directory();
    let path = state_path(&directory);
    {
        let _store = open_state(&path).unwrap();
    }
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 7).unwrap();
    drop(connection);

    assert!(matches!(
        open_state(&path),
        Err(StateError::Unsupported { .. })
    ));
}

#[test]
fn create_run_persists_deterministic_ready_tasks_and_reopens() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let graph = graph(vec![
        task("zeta", &[]),
        task("blocked", &["alpha"]),
        task("alpha", &[]),
    ]);
    let provenance = serde_json::json!({"source": "test"});

    let run = store
        .create_run(&graph, &provenance, &run_budget(), 10)
        .expect("create run");
    let ready = store.ready_tasks(&run.run_id).unwrap();
    assert_eq!(
        ready
            .iter()
            .map(|task| task.task_id.to_string())
            .collect::<Vec<_>>(),
        ["alpha", "zeta"]
    );
    assert!(ready.iter().all(|task| task.state == TaskState::Ready));
    drop(store);

    let mut reopened = open_state(&path).unwrap();
    assert_eq!(reopened.get_run(&run.run_id).unwrap(), Some(run.clone()));
    assert_eq!(reopened.ready_tasks(&run.run_id).unwrap().len(), 2);
}

#[test]
fn create_run_rolls_back_duplicate_and_unknown_dependencies() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let provenance = serde_json::json!({});

    let duplicate = graph(vec![task("same", &[]), task("same", &[])]);
    assert!(matches!(
        store.create_run(&duplicate, &provenance, &run_budget(), 1),
        Err(StateError::InvalidInput { .. })
    ));

    let unknown = graph(vec![task("only", &["missing"])]);
    assert!(matches!(
        store.create_run(&unknown, &provenance, &run_budget(), 2),
        Err(StateError::InvalidInput { .. })
    ));
    assert!(store.incomplete_runs().unwrap().is_empty());
}

#[test]
fn create_run_rejects_unbounded_and_non_object_inputs() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let graph = graph(vec![task("only", &[])]);

    assert!(matches!(
        store.create_run(
            &graph,
            &serde_json::Value::String("not-object".to_owned()),
            &run_budget(),
            1
        ),
        Err(StateError::InvalidInput { .. })
    ));
    assert!(matches!(
        RunBudget::new(i64::MAX as u64 + 1, 1, 1),
        Err(StateError::LimitExceeded { .. })
    ));
}

#[test]
fn opened_database_identity_matches_the_validated_path() {
    let directory = private_directory();
    let path = state_path(&directory);
    let _store = open_state(&path).unwrap();
    let parent = fs::metadata(directory.path()).unwrap();
    let file = fs::metadata(&path).unwrap();

    assert_ne!(parent.ino(), 0);
    assert_ne!(file.ino(), 0);
    assert_eq!(file.uid(), unsafe { libc::geteuid() });
}

#[allow(dead_code)]
fn assert_send<T: Send>() {}

#[test]
fn state_store_can_move_between_threads_but_requires_mutable_mutations() {
    assert_send::<StateStore>();
}

#[test]
fn lease_renewal_and_reclaim_return_new_immutable_authority() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");

    let renewed = store.renew_lease(&lease, 120, 20).unwrap();
    assert_eq!(renewed.attempt_id(), lease.attempt_id());
    assert_eq!(renewed.owner(), lease.owner());
    assert_eq!(renewed.expires_at(), 120);
    assert!(matches!(
        store.renew_lease(&lease, 130, 21),
        Err(StateError::Conflict { .. })
    ));

    let run_id = store
        .get_attempt(lease.attempt_id())
        .unwrap()
        .unwrap()
        .run_id;
    set_test_lease_now(&path, 120);
    assert_eq!(store.expire_leases(&run_id, 120).unwrap(), 1);
    let reclaimed = store
        .reclaim_attempt(lease.attempt_id(), "worker-2", 120, 121, 150)
        .unwrap();
    assert_eq!(reclaimed.attempt_id(), lease.attempt_id());
    assert_eq!(reclaimed.owner(), "worker-2");
    assert_eq!(reclaimed.expires_at(), 150);
}

#[test]
fn lease_capability_is_opaque_and_prior_tokens_cannot_authorize() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");
    let visible = store.get_attempt(lease.attempt_id()).unwrap().unwrap();
    let restored = store.restore_task_claim(&lease, 20).unwrap();

    assert!(!format!("{lease:?}").contains("capability"));
    assert!(!format!("{restored:?}").contains("capability"));
    assert_eq!(visible.lease_owner.as_deref(), Some(lease.owner()));
    assert_eq!(visible.lease_expires_at, Some(lease.expires_at()));
    let forged = store.test_only_forge_lease_from_visible(
        visible.attempt_id,
        lease.owner(),
        lease.expires_at(),
    );
    assert!(matches!(
        store.prepare_operation(&forged, OperationKind::StartThread, 0, 20),
        Err(StateError::Conflict { .. })
    ));

    let renewed = store.renew_lease(&lease, 120, 20).unwrap();
    assert!(matches!(
        store.prepare_operation(&lease, OperationKind::StartThread, 0, 21),
        Err(StateError::Conflict { .. })
    ));
    store
        .prepare_operation(&renewed, OperationKind::StartThread, 0, 21)
        .unwrap();

    let connection = rusqlite::Connection::open(path).unwrap();
    let events_containing_capability: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM events
             WHERE CAST(payload_json AS TEXT) LIKE '%leaseCapability%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(events_containing_capability, 0);
}

#[test]
fn expired_published_result_can_be_reclaimed_for_acceptance_recovery() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    publish_success(&mut store, &lease, "alpha", 11);

    set_test_lease_now(&path, lease.expires_at());
    assert_eq!(store.expire_leases(&run_id, lease.expires_at()).unwrap(), 1);
    let reclaimed = store
        .reclaim_attempt(
            lease.attempt_id(),
            "recovery-worker",
            lease.expires_at(),
            lease.expires_at() + 1,
            lease.expires_at() + 100,
        )
        .expect("published result remains reclaimable for durable acceptance");

    assert_eq!(reclaimed.attempt_id(), lease.attempt_id());
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::ResultPublished
    );
}

#[test]
fn stale_lease_cannot_mutate_after_expiry_or_reclaim() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, stale) = claim_single_task(&mut store, "alpha");
    set_test_lease_now(&path, stale.expires_at());
    assert_eq!(store.expire_leases(&run_id, stale.expires_at()).unwrap(), 1);
    let current = store
        .reclaim_attempt(
            stale.attempt_id(),
            "worker-2",
            stale.expires_at(),
            stale.expires_at() + 1,
            stale.expires_at() + 100,
        )
        .unwrap();

    assert!(matches!(
        store.prepare_operation(
            &stale,
            OperationKind::StartThread,
            0,
            stale.expires_at() + 2
        ),
        Err(StateError::Conflict { .. })
    ));
    let operation = store
        .prepare_operation(
            &current,
            OperationKind::StartThread,
            0,
            stale.expires_at() + 2,
        )
        .unwrap();
    assert!(matches!(
        store.mark_dispatching_thread(&stale, &operation.operation_id, stale.expires_at() + 3),
        Err(StateError::Conflict { .. })
    ));
    store
        .mark_dispatching_thread(&current, &operation.operation_id, stale.expires_at() + 3)
        .unwrap();
    let stale_thread = ThreadId::from_str("stale-thread").unwrap();
    assert!(matches!(
        store.record_thread_started(
            &stale,
            &operation.operation_id,
            &stale_thread,
            stale.expires_at() + 4
        ),
        Err(StateError::Conflict { .. })
    ));
    store
        .record_thread_started(
            &current,
            &operation.operation_id,
            &ThreadId::from_str("current-thread").unwrap(),
            stale.expires_at() + 4,
        )
        .unwrap();
    assert!(matches!(
        store.reconcile_usage(&stale, 0, 1, 0, stale.expires_at() + 3),
        Err(StateError::Conflict { .. })
    ));
    let turn = prepare_test_turn_operation(
        &mut store,
        &current,
        OperationKind::StartTurn,
        0,
        stale.expires_at() + 5,
    );
    store
        .mark_dispatching_turn(&current, &turn.operation_id, stale.expires_at() + 6)
        .unwrap();
    store
        .record_turn_started(
            &current,
            &turn.operation_id,
            &TurnId::from_str("current-turn").unwrap(),
            stale.expires_at() + 7,
        )
        .unwrap();
    let publish = prepare_completed_operation(
        &mut store,
        &current,
        OperationKind::PublishResult,
        0,
        stale.expires_at() + 8,
    );
    let (mut result, _) = result("alpha");
    result.token_usage = 0;
    let result_bytes = serde_json::to_vec(&result).unwrap();
    let result_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&result_bytes)),
        "application/vnd.harp.result+json",
        result_bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &result, &result_ref, stale.expires_at() + 11);
    assert!(matches!(
        store.record_result_published(
            &stale,
            &publish,
            &result,
            &result_ref,
            stale.expires_at() + 12
        ),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn claim_ready_task_is_deterministic_and_reserves_budget() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("zeta", &[]), task("alpha", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();

    let claim = store
        .claim_ready_task(&run.run_id, "worker-1", 10, 20)
        .unwrap()
        .expect("ready task");

    assert_eq!(claim.task_id.to_string(), "alpha");
    assert_eq!(claim.lease_owner, "worker-1");
    assert_eq!(claim.lease_expires_at, 20);
    assert_eq!(claim.budget, Budget::new(100, 60, 1_024));
    let attempt = store.get_attempt(&claim.attempt_id).unwrap().unwrap();
    assert_eq!(attempt.state, AttemptState::Prepared);
    assert_eq!(attempt.ordinal, 0);
    assert_eq!(attempt.lease_owner.as_deref(), Some("worker-1"));
    assert_eq!(
        store
            .ready_tasks(&run.run_id)
            .unwrap()
            .iter()
            .map(|task| task.task_id.to_string())
            .collect::<Vec<_>>(),
        ["zeta"]
    );
}

#[test]
fn cli_claim_and_activity_lifecycle_are_atomic_and_persist_before_effect() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let cli_attempt = store.get_cli_attempt(&claim.attempt_id).unwrap().unwrap();
    assert_eq!(cli_attempt.state, CliAttemptState::Prepared);
    assert_eq!(
        cli_attempt.logical_session_id.to_string(),
        "logical-session"
    );
    assert_eq!(
        store.get_attempt(&claim.attempt_id).unwrap().unwrap().state,
        AttemptState::Prepared
    );

    let legacy_run = store
        .create_run(
            &graph(vec![task("legacy", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            2,
        )
        .unwrap();
    let legacy = store
        .claim_ready_task(&legacy_run.run_id, "legacy-worker", 11, 100)
        .unwrap()
        .unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    let unattached = OperationId::new();
    assert!(connection
        .execute(
            "INSERT INTO cli_activities(
                activity_id, attempt_id, kind, ordinal, state,
                logical_turn_id, activity_dir, invocation_sha256,
                created_at, updated_at
             ) VALUES (?1, ?2, 'start_activity', 0, 'prepared',
                       ?1, '/private/tmp/unattached', ?3, 1, 1)",
            rusqlite::params![
                unattached.to_string(),
                legacy.attempt_id.to_string(),
                "a".repeat(64)
            ],
        )
        .is_err());
    drop(connection);

    let activity_id = OperationId::new();
    let logical_turn_id = TurnId::from_str(&activity_id.to_string()).unwrap();
    let activity = store
        .prepare_activity(
            &claim.lease(),
            activity_id.clone(),
            &claim.attempt_id,
            CliActivityKind::StartActivity,
            logical_turn_id,
            &activity_preparation("start"),
            12,
        )
        .unwrap();
    assert_eq!(activity.activity_id, activity_id);
    assert_eq!(
        activity.logical_turn_id.to_string(),
        activity_id.to_string()
    );
    assert_eq!(activity.state, CliActivityState::Prepared);
    assert!(matches!(
        store.mark_activity_running(&claim.lease(), &activity_id, 13),
        Err(StateError::Conflict { .. })
    ));
    assert!(matches!(
        store.record_process(&claim.lease(), &activity_id, &"b".repeat(64), 13),
        Err(StateError::Conflict { .. })
    ));

    store
        .mark_activity_dispatching(&claim.lease(), &activity_id, 14)
        .unwrap();
    assert_eq!(
        store
            .prepare_activity(
                &claim.lease(),
                activity_id.clone(),
                &claim.attempt_id,
                CliActivityKind::StartActivity,
                TurnId::from_str(&activity_id.to_string()).unwrap(),
                &activity_preparation("start"),
                15,
            )
            .unwrap()
            .activity_id,
        activity_id
    );
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Running
    );
    assert_eq!(
        store.get_attempt(&claim.attempt_id).unwrap().unwrap().state,
        AttemptState::Prepared
    );
    let process_digest = "b".repeat(64);
    store
        .record_process(&claim.lease(), &activity_id, &process_digest, 16)
        .unwrap();
    store
        .record_process(&claim.lease(), &activity_id, &process_digest, 17)
        .unwrap();
    assert!(matches!(
        store.record_process(&claim.lease(), &activity_id, &"c".repeat(64), 18),
        Err(StateError::Conflict { .. })
    ));
    store
        .mark_activity_running(&claim.lease(), &activity_id, 19)
        .unwrap();

    let external = ExternalSessionId::from_str("external-session").unwrap();
    store
        .record_cli_external_session(&claim.lease(), &activity_id, external.clone(), 20)
        .unwrap();
    store
        .record_cli_external_session(&claim.lease(), &activity_id, external, 21)
        .unwrap();
    assert!(matches!(
        store.record_cli_external_session(
            &claim.lease(),
            &activity_id,
            ExternalSessionId::from_str("other-session").unwrap(),
            22,
        ),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(
        store
            .recoverable_activities(&run_id)
            .unwrap()
            .first()
            .unwrap()
            .external_session_id
            .as_ref()
            .unwrap()
            .to_string(),
        "external-session"
    );
}

#[test]
fn cli_claim_rolls_back_if_extension_insert_fails() {
    for trigger in [
        "CREATE TRIGGER reject_cli_claim
         BEFORE INSERT ON cli_attempts
         BEGIN
             SELECT RAISE(ABORT, 'injected cli extension failure');
         END;",
        "CREATE TRIGGER reject_cli_claim
         BEFORE INSERT ON budget_reservations
         BEGIN
             SELECT RAISE(ABORT, 'injected budget reservation failure');
         END;",
        "CREATE TRIGGER reject_cli_claim
         BEFORE INSERT ON events
         WHEN NEW.event_type = 'task_claimed'
         BEGIN
             SELECT RAISE(ABORT, 'injected claim event failure');
         END;",
    ] {
        let directory = private_directory();
        let path = state_path(&directory);
        let mut store = open_state(&path).unwrap();
        let run = store
            .create_run(
                &graph(vec![task("alpha", &[])]),
                &serde_json::json!({}),
                &run_budget(),
                1,
            )
            .unwrap();
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.execute_batch(trigger).unwrap();
        drop(connection);

        assert!(matches!(
            store.claim_ready_cli_task(
                &run.run_id,
                "worker",
                10,
                100,
                ThreadId::from_str("logical-session").unwrap(),
            ),
            Err(StateError::Sqlite { .. })
        ));
        assert!(store.attempts(&run.run_id).unwrap().is_empty());
        assert_eq!(store.ready_tasks(&run.run_id).unwrap().len(), 1);
    }
}

#[test]
fn external_session_ids_are_unique_across_cli_attempts() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[]), task("beta", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let alpha = store
        .claim_ready_cli_task(
            &run.run_id,
            "alpha-worker",
            10,
            100,
            ThreadId::from_str("alpha-logical").unwrap(),
        )
        .unwrap()
        .unwrap();
    let beta = store
        .claim_ready_cli_task(
            &run.run_id,
            "beta-worker",
            11,
            100,
            ThreadId::from_str("beta-logical").unwrap(),
        )
        .unwrap()
        .unwrap();
    let alpha_activity = drive_cli_activity_to_reconciling(&mut store, &alpha, "alpha-session", 20);
    let beta_activity = drive_cli_activity_to_reconciling(&mut store, &beta, "beta-session", 30);
    let shared = ExternalSessionId::from_str("shared-external-session").unwrap();
    store
        .record_cli_external_session(&alpha.lease(), &alpha_activity, shared.clone(), 40)
        .unwrap();
    assert!(matches!(
        store.record_cli_external_session(&beta.lease(), &beta_activity, shared, 41),
        Err(StateError::Conflict { .. })
    ));
    assert!(store
        .get_cli_attempt(&beta.attempt_id)
        .unwrap()
        .unwrap()
        .external_session_id
        .is_none());
}

#[test]
fn cli_completion_bridge_is_atomic_idempotent_and_closes_recovery() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let activity_id = drive_cli_activity_to_reconciling(&mut store, &claim, "bridge", 20);
    assert_eq!(
        store.recoverable_activities(&run_id).unwrap().len(),
        1,
        "reconciling activity remains recoverable before the bridge"
    );

    let publish_operation_id = OperationId::new();
    let published = store
        .bridge_cli_completion(&claim.lease(), &activity_id, &publish_operation_id, 30)
        .unwrap();
    assert_eq!(published.operation_id, publish_operation_id);
    assert_eq!(published.kind, OperationKind::PublishResult);
    assert_eq!(published.ordinal, 0);
    assert_eq!(published.state, OperationState::Prepared);
    assert_eq!(
        store.get_cli_activity(&activity_id).unwrap().unwrap().state,
        CliActivityState::Completed
    );
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Reconciling
    );
    assert_eq!(
        store.get_attempt(&claim.attempt_id).unwrap().unwrap().state,
        AttemptState::Reconciling
    );
    let recoverable = store.recoverable_activities(&run_id).unwrap();
    assert_eq!(recoverable.len(), 1);
    assert_eq!(recoverable[0].activity_id, activity_id);
    assert_eq!(recoverable[0].activity_state, CliActivityState::Completed);
    assert!(store.get_operation(&activity_id).unwrap().is_none());

    assert_eq!(
        store
            .bridge_cli_completion(&claim.lease(), &activity_id, &publish_operation_id, 31,)
            .unwrap(),
        published
    );
    assert!(matches!(
        store.bridge_cli_completion(&claim.lease(), &activity_id, &OperationId::new(), 32,),
        Err(StateError::Conflict { .. } | StateError::Integrity { .. })
    ));
}

#[test]
fn cli_publish_result_is_owned_by_the_completion_bridge() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let activity_id = drive_cli_activity_to_reconciling(&mut store, &claim, "bridge-owned", 20);
    let publish_operation_id = OperationId::new();
    store
        .bridge_cli_completion(&claim.lease(), &activity_id, &publish_operation_id, 30)
        .unwrap();

    assert!(matches!(
        store.prepare_operation(&claim.lease(), OperationKind::PublishResult, 1, 31),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(
        store
            .operations_for_attempt(&claim.attempt_id)
            .unwrap()
            .into_iter()
            .filter(|operation| operation.kind == OperationKind::PublishResult)
            .count(),
        1
    );
}

#[test]
fn cli_attempt_rejects_legacy_thread_turn_and_interrupt_operations() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, claim) = claim_single_cli_task(&mut store, "alpha");

    assert!(matches!(
        store.prepare_operation(&claim.lease(), OperationKind::StartThread, 0, 20),
        Err(StateError::Conflict { .. })
    ));
    let start_turn_operation_id = OperationId::new();
    let start_turn = harp_state::TurnOperationPreparation {
        operation_id: start_turn_operation_id.clone(),
        kind: OperationKind::StartTurn,
        ordinal: 0,
        intent: TurnSpec {
            instruction: "legacy turn should be rejected".to_owned(),
            operation_marker: start_turn_operation_id,
            output_schema: serde_json::json!({"type": "object"}),
            model: Some("default".to_owned()),
            reasoning_effort: None,
        },
        checkpoint_sha256: None,
    };
    assert!(matches!(
        store.prepare_turn_operation(&claim.lease(), &start_turn, 21),
        Err(StateError::Conflict { .. })
    ));
    let interrupt =
        harp_state::InterruptIntent::budget(harp_state::InterruptBudgetDimension::Tokens, "budget")
            .unwrap();
    assert!(matches!(
        store.prepare_interrupt_operation(&claim.lease(), 0, &interrupt, 22),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn cli_completion_bridge_replay_rejects_extra_publish_operation() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (_run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let activity_id = drive_cli_activity_to_reconciling(&mut store, &claim, "bridge-extra", 20);
    let publish_operation_id = OperationId::new();
    store
        .bridge_cli_completion(&claim.lease(), &activity_id, &publish_operation_id, 30)
        .unwrap();

    let extra_operation_id = OperationId::new();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "INSERT INTO operations(
                operation_id, attempt_id, kind, ordinal, state, created_at, updated_at
             ) VALUES (?1, ?2, 'publish_result', 1, 'prepared', 31, 31)",
            rusqlite::params![extra_operation_id.to_string(), claim.attempt_id.to_string()],
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        store.bridge_cli_completion(&claim.lease(), &activity_id, &publish_operation_id, 32),
        Err(StateError::Conflict { .. } | StateError::Integrity { .. })
    ));
}

#[test]
fn cli_completion_bridge_rolls_back_all_writes_when_event_insert_fails() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let activity_id = drive_cli_activity_to_reconciling(&mut store, &claim, "bridge-rollback", 20);
    let publish_operation_id = OperationId::new();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER reject_cli_completion_bridge_event
             BEFORE INSERT ON events
             WHEN NEW.event_type = 'cli_completion_bridged'
             BEGIN
                 SELECT RAISE(ABORT, 'injected bridge event failure');
             END;",
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        store.bridge_cli_completion(&claim.lease(), &activity_id, &publish_operation_id, 30),
        Err(StateError::Sqlite { .. })
    ));
    assert_eq!(
        store.get_cli_activity(&activity_id).unwrap().unwrap().state,
        CliActivityState::Reconciling
    );
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Running
    );
    assert_eq!(
        store.get_attempt(&claim.attempt_id).unwrap().unwrap().state,
        AttemptState::Prepared
    );
    assert!(store
        .get_operation(&publish_operation_id)
        .unwrap()
        .is_none());
    assert!(store
        .events_page(&run_id, None, 1_000)
        .unwrap()
        .events
        .iter()
        .all(|event| event.event_type != "cli_completion_bridged"));
}

#[test]
fn cli_result_acceptance_succeeds_winner_and_quarantines_competitor() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let competing_attempt = insert_competing_attempt(&path, &run_id, "alpha", 1, 0);
    let competing_activity = OperationId::new();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    connection
        .execute(
            "INSERT INTO cli_attempts(
                attempt_id, state, logical_session_id,
                continuation_count, created_at, updated_at
             ) VALUES (?1, 'running', 'competing-session', 0, 11, 11)",
            [competing_attempt.to_string()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO cli_activities(
                activity_id, attempt_id, kind, ordinal, state,
                logical_turn_id, activity_dir, invocation_sha256,
                created_at, updated_at
             ) VALUES (
                ?1, ?2, 'start_activity', 0, 'prepared',
                ?1, '/private/tmp/competing-activity', ?3, 11, 11
             )",
            rusqlite::params![
                competing_activity.to_string(),
                competing_attempt.to_string(),
                "d".repeat(64)
            ],
        )
        .unwrap();
    drop(connection);

    let activity_id = drive_cli_activity_to_reconciling(&mut store, &claim, "winner", 20);
    store.reconcile_usage(&claim.lease(), 0, 42, 0, 25).unwrap();
    let publish_operation_id = OperationId::new();
    store
        .bridge_cli_completion(&claim.lease(), &activity_id, &publish_operation_id, 26)
        .unwrap();
    store
        .mark_operation_dispatching(&claim.lease(), &publish_operation_id, 27)
        .unwrap();
    store
        .complete_operation(&claim.lease(), &publish_operation_id, None, 28)
        .unwrap();
    let (result, result_ref) = result("alpha");
    register_result_artifacts(&mut store, &result, &result_ref, 29);
    store
        .record_result_published(
            &claim.lease(),
            &publish_operation_id,
            &result,
            &result_ref,
            30,
        )
        .unwrap();
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Reconciling
    );

    assert_eq!(
        store
            .accept_result(
                &run_id,
                &TaskId::from_str("alpha").unwrap(),
                &claim.attempt_id,
                31,
            )
            .unwrap(),
        AcceptResult::Accepted
    );
    assert_eq!(
        store.get_attempt(&claim.attempt_id).unwrap().unwrap().state,
        AttemptState::Succeeded
    );
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Succeeded
    );
    assert_eq!(
        store
            .get_attempt(&competing_attempt)
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Indeterminate
    );
    assert_eq!(
        store
            .get_cli_attempt(&competing_attempt)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Indeterminate
    );
    assert_eq!(
        store
            .get_cli_activity(&competing_activity)
            .unwrap()
            .unwrap()
            .state,
        CliActivityState::Indeterminate
    );
    assert!(store.recoverable_activities(&run_id).unwrap().is_empty());
    assert_eq!(
        store
            .accept_result(
                &run_id,
                &TaskId::from_str("alpha").unwrap(),
                &competing_attempt,
                32,
            )
            .unwrap(),
        AcceptResult::AlreadyAccepted {
            attempt_id: claim.attempt_id
        }
    );
}

#[test]
fn cli_result_acceptance_rejects_leftover_nonterminal_activities() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");

    let activity_id = drive_cli_activity_to_reconciling(&mut store, &claim, "leftover", 20);
    store.reconcile_usage(&claim.lease(), 0, 42, 0, 25).unwrap();
    let publish_operation_id = OperationId::new();
    store
        .bridge_cli_completion(&claim.lease(), &activity_id, &publish_operation_id, 26)
        .unwrap();
    store
        .mark_operation_dispatching(&claim.lease(), &publish_operation_id, 27)
        .unwrap();
    store
        .complete_operation(&claim.lease(), &publish_operation_id, None, 28)
        .unwrap();
    let (result, result_ref) = result("alpha");
    register_result_artifacts(&mut store, &result, &result_ref, 29);
    store
        .record_result_published(
            &claim.lease(),
            &publish_operation_id,
            &result,
            &result_ref,
            30,
        )
        .unwrap();

    let interrupt_id = OperationId::new();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "INSERT INTO cli_activities(
                activity_id, attempt_id, kind, ordinal, state, logical_turn_id,
                activity_dir, invocation_sha256, interrupt_purpose,
                target_process_record_sha256, signal_stage, created_at, updated_at
             ) VALUES (
                ?1, ?2, 'interrupt_activity', 0, 'prepared', ?1,
                '/private/tmp/leftover-interrupt', ?3, 'budget', ?4, 'prepared', 31, 31
             )",
            rusqlite::params![
                interrupt_id.to_string(),
                claim.attempt_id.to_string(),
                "b".repeat(64),
                "c".repeat(64)
            ],
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        store.accept_result(
            &run_id,
            &TaskId::from_str("alpha").unwrap(),
            &claim.attempt_id,
            32,
        ),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Reconciling
    );
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::ResultPublished
    );
}

#[test]
fn failed_and_cancelled_cli_publication_terminalize_both_state_domains() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (failed_run_id, failed_claim) = claim_single_cli_task(&mut store, "failed");
    let failed_activity =
        drive_cli_activity_to_reconciling(&mut store, &failed_claim, "failed-publication", 20);
    let failed_publish = OperationId::new();
    store
        .bridge_cli_completion(&failed_claim.lease(), &failed_activity, &failed_publish, 25)
        .unwrap();
    store
        .mark_operation_dispatching(&failed_claim.lease(), &failed_publish, 26)
        .unwrap();
    store
        .complete_operation(&failed_claim.lease(), &failed_publish, None, 27)
        .unwrap();
    let (mut failed, _) = result("failed");
    failed.status = ResultStatus::Failed;
    failed.answer_ref = None;
    failed.failure_class = Some("non_resumable_protocol".to_owned());
    failed.token_usage = 0;
    let failed_bytes = serde_json::to_vec(&failed).unwrap();
    let failed_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&failed_bytes)),
        "application/vnd.harp.result+json",
        failed_bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &failed, &failed_ref, 28);
    store
        .record_result_published(
            &failed_claim.lease(),
            &failed_publish,
            &failed,
            &failed_ref,
            29,
        )
        .unwrap();
    assert_eq!(
        store
            .get_attempt(&failed_claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Failed
    );
    let failed_cli = store
        .get_cli_attempt(&failed_claim.attempt_id)
        .unwrap()
        .unwrap();
    assert_eq!(failed_cli.state, CliAttemptState::Failed);
    assert_eq!(
        failed_cli.terminal_failure_class,
        Some(CliTerminalFailureClass::NonResumableProtocolFailure)
    );
    assert!(store
        .recoverable_activities(&failed_run_id)
        .unwrap()
        .is_empty());

    let (cancelled_run_id, cancelled_claim) = claim_single_cli_task(&mut store, "cancelled");
    let cancelled_activity = drive_cli_activity_to_reconciling(
        &mut store,
        &cancelled_claim,
        "cancelled-publication",
        40,
    );
    let cancelled_publish = OperationId::new();
    store
        .bridge_cli_completion(
            &cancelled_claim.lease(),
            &cancelled_activity,
            &cancelled_publish,
            45,
        )
        .unwrap();
    store
        .mark_operation_dispatching(&cancelled_claim.lease(), &cancelled_publish, 46)
        .unwrap();
    store
        .complete_operation(&cancelled_claim.lease(), &cancelled_publish, None, 47)
        .unwrap();
    store
        .request_run_cancellation(&cancelled_run_id, 48)
        .unwrap();
    let (mut cancelled, _) = result("cancelled");
    cancelled.status = ResultStatus::Cancelled;
    cancelled.answer_ref = None;
    cancelled.failure_class = None;
    cancelled.token_usage = 0;
    let cancelled_bytes = serde_json::to_vec(&cancelled).unwrap();
    let cancelled_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&cancelled_bytes)),
        "application/vnd.harp.result+json",
        cancelled_bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &cancelled, &cancelled_ref, 49);
    store
        .record_result_published(
            &cancelled_claim.lease(),
            &cancelled_publish,
            &cancelled,
            &cancelled_ref,
            50,
        )
        .unwrap();
    assert_eq!(
        store
            .get_attempt(&cancelled_claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Cancelled
    );
    let cancelled_cli = store
        .get_cli_attempt(&cancelled_claim.attempt_id)
        .unwrap()
        .unwrap();
    assert_eq!(cancelled_cli.state, CliAttemptState::Cancelled);
    assert_eq!(
        cancelled_cli.terminal_failure_class,
        Some(CliTerminalFailureClass::Cancelled)
    );
    assert!(store
        .recoverable_activities(&cancelled_run_id)
        .unwrap()
        .is_empty());
}

#[test]
fn cli_continuation_is_prepared_atomically_and_bounded_to_one() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let failed_activity = drive_cli_activity_to_reconciling(&mut store, &claim, "initial", 20);
    store
        .record_cli_external_session(
            &claim.lease(),
            &failed_activity,
            ExternalSessionId::from_str("external-session").unwrap(),
            25,
        )
        .unwrap();
    let continuation_id = OperationId::new();
    let continuation = store
        .prepare_continuation_after_failure(
            &claim.lease(),
            &failed_activity,
            CliTerminalFailureClass::ResumableCliFailure,
            continuation_id.clone(),
            &activity_preparation("continuation"),
            26,
        )
        .unwrap();
    assert_eq!(continuation.activity_id, continuation_id);
    assert_eq!(continuation.kind, CliActivityKind::ContinueActivity);
    assert_eq!(continuation.state, CliActivityState::Prepared);
    assert_eq!(
        store
            .get_cli_activity(&failed_activity)
            .unwrap()
            .unwrap()
            .terminal_failure_class,
        Some(CliTerminalFailureClass::ResumableCliFailure)
    );
    let cli_attempt = store.get_cli_attempt(&claim.attempt_id).unwrap().unwrap();
    assert_eq!(cli_attempt.state, CliAttemptState::Reconciling);
    assert_eq!(cli_attempt.continuation_count, 1);
    assert_eq!(
        store.get_attempt(&claim.attempt_id).unwrap().unwrap().state,
        AttemptState::Prepared
    );
    let recoverable = store.recoverable_activities(&run_id).unwrap();
    assert_eq!(recoverable.len(), 1);
    assert_eq!(recoverable[0].activity_id, continuation.activity_id);
    assert_eq!(recoverable[0].activity_state, CliActivityState::Prepared);
    assert_eq!(recoverable[0].continuation_count, 1);
    assert_eq!(
        store
            .prepare_continuation_after_failure(
                &claim.lease(),
                &failed_activity,
                CliTerminalFailureClass::ResumableCliFailure,
                continuation_id,
                &activity_preparation("continuation"),
                27,
            )
            .unwrap(),
        continuation
    );
    assert!(matches!(
        store.prepare_continuation_after_failure(
            &claim.lease(),
            &failed_activity,
            CliTerminalFailureClass::ResumableCliFailure,
            OperationId::new(),
            &activity_preparation("second-continuation"),
            28,
        ),
        Err(StateError::Conflict { .. })
    ));
    store
        .mark_activity_dispatching(&claim.lease(), &continuation.activity_id, 29)
        .unwrap();
    assert_eq!(
        store
            .prepare_continuation_after_failure(
                &claim.lease(),
                &failed_activity,
                CliTerminalFailureClass::ResumableCliFailure,
                continuation.activity_id.clone(),
                &activity_preparation("continuation"),
                30,
            )
            .unwrap()
            .activity_id,
        continuation.activity_id
    );
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Running
    );
}

#[test]
fn cli_interrupt_signal_stages_are_ordered_and_replay_restricted() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let target_activity = OperationId::new();
    store
        .prepare_activity(
            &claim.lease(),
            target_activity.clone(),
            &claim.attempt_id,
            CliActivityKind::StartActivity,
            TurnId::from_str(&target_activity.to_string()).unwrap(),
            &activity_preparation("signal-target"),
            20,
        )
        .unwrap();
    store
        .mark_activity_dispatching(&claim.lease(), &target_activity, 21)
        .unwrap();
    let target_digest = "e".repeat(64);
    store
        .record_process(&claim.lease(), &target_activity, &target_digest, 22)
        .unwrap();
    store
        .mark_activity_running(&claim.lease(), &target_activity, 23)
        .unwrap();

    let interrupt_id = OperationId::new();
    store
        .prepare_interrupt_activity(
            &claim.lease(),
            interrupt_id.clone(),
            0,
            &activity_preparation("interrupt"),
            &target_digest,
            InterruptPurpose::Budget,
            24,
        )
        .unwrap();
    assert!(matches!(
        store.advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::Prepared,
            CliSignalStage::SigintSent,
            25,
        ),
        Err(StateError::InvalidInput { .. })
    ));
    store
        .advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::Prepared,
            CliSignalStage::SigintPrepared,
            26,
        )
        .unwrap();
    assert!(matches!(
        store.advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::Prepared,
            CliSignalStage::SigintPrepared,
            27,
        ),
        Err(StateError::Conflict { .. })
    ));
    store
        .advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::SigintPrepared,
            CliSignalStage::SigintSent,
            28,
        )
        .unwrap();
    store
        .advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::SigintSent,
            CliSignalStage::Quiescent,
            29,
        )
        .unwrap();
    let interrupt = store.get_cli_activity(&interrupt_id).unwrap().unwrap();
    assert_eq!(interrupt.state, CliActivityState::Completed);
    assert_eq!(interrupt.signal_stage, Some(CliSignalStage::Quiescent));
    assert!(matches!(
        store.advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::Quiescent,
            CliSignalStage::Indeterminate,
            30,
        ),
        Err(StateError::InvalidInput { .. })
    ));
}

#[test]
fn cli_cancellation_interrupt_requires_durable_cancellation_request() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let target_activity = OperationId::new();
    store
        .prepare_activity(
            &claim.lease(),
            target_activity.clone(),
            &claim.attempt_id,
            CliActivityKind::StartActivity,
            TurnId::from_str(&target_activity.to_string()).unwrap(),
            &activity_preparation("cancellation-authority-target"),
            20,
        )
        .unwrap();
    store
        .mark_activity_dispatching(&claim.lease(), &target_activity, 21)
        .unwrap();
    let target_digest = "9".repeat(64);
    store
        .record_process(&claim.lease(), &target_activity, &target_digest, 22)
        .unwrap();
    store
        .mark_activity_running(&claim.lease(), &target_activity, 23)
        .unwrap();

    let interrupt_id = OperationId::new();
    assert!(matches!(
        store.prepare_interrupt_activity(
            &claim.lease(),
            interrupt_id.clone(),
            0,
            &activity_preparation("cancellation-authority-interrupt"),
            &target_digest,
            InterruptPurpose::Cancellation,
            24,
        ),
        Err(StateError::Conflict { .. })
    ));

    assert!(store.request_run_cancellation(&run_id, 25).unwrap());
    store
        .prepare_interrupt_activity(
            &claim.lease(),
            interrupt_id.clone(),
            0,
            &activity_preparation("cancellation-authority-interrupt"),
            &target_digest,
            InterruptPurpose::Cancellation,
            26,
        )
        .unwrap();
    assert!(matches!(
        store.advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::Prepared,
            CliSignalStage::SigintPrepared,
            27,
        ),
        Ok(())
    ));
}

#[test]
fn cli_signal_prepared_stages_can_become_indeterminate() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let target_activity = OperationId::new();
    store
        .prepare_activity(
            &claim.lease(),
            target_activity.clone(),
            &claim.attempt_id,
            CliActivityKind::StartActivity,
            TurnId::from_str(&target_activity.to_string()).unwrap(),
            &activity_preparation("ambiguity-target"),
            20,
        )
        .unwrap();
    store
        .mark_activity_dispatching(&claim.lease(), &target_activity, 21)
        .unwrap();
    let target_digest = "f".repeat(64);
    store
        .record_process(&claim.lease(), &target_activity, &target_digest, 22)
        .unwrap();
    store
        .mark_activity_running(&claim.lease(), &target_activity, 23)
        .unwrap();

    let prepared_paths = [
        vec![],
        vec![CliSignalStage::SigintPrepared],
        vec![
            CliSignalStage::SigintPrepared,
            CliSignalStage::SigintSent,
            CliSignalStage::SigtermPrepared,
        ],
        vec![
            CliSignalStage::SigintPrepared,
            CliSignalStage::SigintSent,
            CliSignalStage::SigtermPrepared,
            CliSignalStage::SigtermSent,
            CliSignalStage::SigkillPrepared,
        ],
    ];
    for (ordinal, path) in prepared_paths.iter().enumerate() {
        let interrupt_id = OperationId::new();
        store
            .prepare_interrupt_activity(
                &claim.lease(),
                interrupt_id.clone(),
                ordinal as u32,
                &activity_preparation(&format!("ambiguity-{ordinal}")),
                &target_digest,
                InterruptPurpose::Budget,
                30 + ordinal as i64 * 10,
            )
            .unwrap();
        let mut previous = CliSignalStage::Prepared;
        for next in path {
            store
                .advance_cli_signal_stage(
                    &claim.lease(),
                    &interrupt_id,
                    previous,
                    *next,
                    31 + ordinal as i64 * 10,
                )
                .unwrap();
            previous = *next;
        }
        let invalid_skip = match previous {
            CliSignalStage::Prepared => CliSignalStage::SigtermPrepared,
            CliSignalStage::SigintPrepared => CliSignalStage::SigtermSent,
            CliSignalStage::SigtermPrepared => CliSignalStage::SigkillSent,
            CliSignalStage::SigkillPrepared => CliSignalStage::Quiescent,
            _ => unreachable!("test paths end at prepared signal stages"),
        };
        assert!(matches!(
            store.advance_cli_signal_stage(
                &claim.lease(),
                &interrupt_id,
                previous,
                invalid_skip,
                38 + ordinal as i64 * 10,
            ),
            Err(StateError::InvalidInput { .. })
        ));
        store
            .advance_cli_signal_stage(
                &claim.lease(),
                &interrupt_id,
                previous,
                CliSignalStage::Indeterminate,
                39 + ordinal as i64 * 10,
            )
            .unwrap();
        let interrupt = store.get_cli_activity(&interrupt_id).unwrap().unwrap();
        assert_eq!(interrupt.state, CliActivityState::Indeterminate);
        assert_eq!(
            interrupt.terminal_failure_class,
            Some(CliTerminalFailureClass::Indeterminate)
        );
        assert_eq!(interrupt.signal_stage, Some(CliSignalStage::Indeterminate));
    }
}

#[test]
fn cli_cancellation_requires_terminal_interrupt_for_attributable_process() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let activity_id = OperationId::new();
    store
        .prepare_activity(
            &claim.lease(),
            activity_id.clone(),
            &claim.attempt_id,
            CliActivityKind::StartActivity,
            TurnId::from_str(&activity_id.to_string()).unwrap(),
            &activity_preparation("cancel-target"),
            20,
        )
        .unwrap();
    store
        .mark_activity_dispatching(&claim.lease(), &activity_id, 21)
        .unwrap();
    let process_digest = "d".repeat(64);
    store
        .record_process(&claim.lease(), &activity_id, &process_digest, 22)
        .unwrap();
    store
        .mark_activity_running(&claim.lease(), &activity_id, 23)
        .unwrap();
    assert!(store.request_run_cancellation(&run_id, 24).unwrap());
    assert!(matches!(
        store.finalize_task_cancelled(&claim.lease(), 25),
        Err(StateError::Conflict { .. })
    ));

    let interrupt_id = OperationId::new();
    store
        .prepare_interrupt_activity(
            &claim.lease(),
            interrupt_id.clone(),
            0,
            &activity_preparation("cancel-interrupt"),
            &process_digest,
            InterruptPurpose::Cancellation,
            26,
        )
        .unwrap();
    store
        .advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::Prepared,
            CliSignalStage::SigintPrepared,
            27,
        )
        .unwrap();
    store
        .advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::SigintPrepared,
            CliSignalStage::SigintSent,
            28,
        )
        .unwrap();
    store
        .advance_cli_signal_stage(
            &claim.lease(),
            &interrupt_id,
            CliSignalStage::SigintSent,
            CliSignalStage::Quiescent,
            29,
        )
        .unwrap();
    store.finalize_task_cancelled(&claim.lease(), 30).unwrap();
    assert_eq!(
        store
            .get_cli_activity(&interrupt_id)
            .unwrap()
            .unwrap()
            .state,
        CliActivityState::Completed
    );
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Cancelled
    );
}

#[test]
fn cli_cancellation_preserves_dispatching_without_process_record_as_indeterminate() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, claim) = claim_single_cli_task(&mut store, "alpha");
    let activity_id = OperationId::new();
    store
        .prepare_activity(
            &claim.lease(),
            activity_id.clone(),
            &claim.attempt_id,
            CliActivityKind::StartActivity,
            TurnId::from_str(&activity_id.to_string()).unwrap(),
            &activity_preparation("cancel-no-process"),
            20,
        )
        .unwrap();
    store
        .mark_activity_dispatching(&claim.lease(), &activity_id, 21)
        .unwrap();
    assert!(store.request_run_cancellation(&run_id, 22).unwrap());
    assert!(matches!(
        store.finalize_task_cancelled(&claim.lease(), 23),
        Err(StateError::Conflict { .. })
    ));

    store
        .mark_indeterminate(&claim.lease(), "lost_spawn", 24)
        .unwrap();

    assert_eq!(
        store.get_attempt(&claim.attempt_id).unwrap().unwrap().state,
        AttemptState::Indeterminate
    );
    assert_eq!(
        store
            .get_cli_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        CliAttemptState::Indeterminate
    );
    let activity = store.get_cli_activity(&activity_id).unwrap().unwrap();
    assert_eq!(activity.state, CliActivityState::Indeterminate);
    assert_eq!(
        activity.terminal_failure_class,
        Some(CliTerminalFailureClass::Indeterminate)
    );
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
}

#[test]
fn claim_rejects_invalid_workers_leases_and_aggregate_budget_overcommit() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![
                task_with_budget("alpha", &[], 60, 600),
                task_with_budget("beta", &[], 60, 600),
            ]),
            &serde_json::json!({}),
            &RunBudget::new(100, 1_000, 3_600).unwrap(),
            1,
        )
        .unwrap();

    assert!(matches!(
        store.claim_ready_task(&run.run_id, "", 10, 20),
        Err(StateError::InvalidInput { .. })
    ));
    set_test_lease_now(&state_path(&directory), 10);
    assert!(matches!(
        store.claim_ready_task(&run.run_id, "worker", 10, 10),
        Err(StateError::InvalidInput { .. })
    ));
    store
        .claim_ready_task(&run.run_id, "worker", 10, 20)
        .unwrap()
        .unwrap();
    let reduced = store
        .claim_ready_task(&run.run_id, "worker", 11, 21)
        .unwrap()
        .expect("second task receives remaining run budget");
    assert_eq!(reduced.budget.max_tokens, 40);
    assert_eq!(reduced.budget.max_storage_bytes, 400);
    assert_eq!(
        store
            .ready_tasks(&run.run_id)
            .unwrap()
            .iter()
            .map(|task| task.task_id.to_string())
            .collect::<Vec<_>>(),
        Vec::<String>::new()
    );
}

#[test]
fn operation_sequence_is_persisted_before_effect_and_idempotent_by_key() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    let lease = claim.lease();

    let start_thread = store
        .prepare_operation(&lease, OperationKind::StartThread, 0, 11)
        .unwrap();
    assert_eq!(start_thread.state, OperationState::Prepared);
    assert_eq!(
        store
            .prepare_operation(&lease, OperationKind::StartThread, 0, 12)
            .unwrap(),
        start_thread
    );
    store
        .mark_dispatching_thread(&lease, &start_thread.operation_id, 13)
        .unwrap();
    assert_eq!(
        store.get_attempt(&claim.attempt_id).unwrap().unwrap().state,
        AttemptState::DispatchingThread
    );
    let thread_id = ThreadId::from_str("thread-alpha").unwrap();
    store
        .record_thread_started(&lease, &start_thread.operation_id, &thread_id, 14)
        .unwrap();

    let start_turn =
        prepare_test_turn_operation(&mut store, &lease, OperationKind::StartTurn, 0, 15);
    let marker = store
        .mark_dispatching_turn(&lease, &start_turn.operation_id, 16)
        .unwrap();
    let turn_id = TurnId::from_str("turn-alpha").unwrap();
    store
        .record_turn_started(&lease, &start_turn.operation_id, &turn_id, 17)
        .unwrap();

    let attempt = store.get_attempt(&claim.attempt_id).unwrap().unwrap();
    assert_eq!(attempt.state, AttemptState::TurnStarted);
    assert_eq!(attempt.thread_id, Some(thread_id));
    assert_eq!(attempt.latest_turn_id, Some(turn_id));
    assert_eq!(
        attempt.latest_operation_marker.as_deref(),
        Some(marker.as_str())
    );
    assert_eq!(
        store
            .get_operation(&start_thread.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Completed
    );
    assert_eq!(
        store
            .get_operation(&start_turn.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Completed
    );
}

#[test]
fn operation_sequence_rejects_out_of_order_calls_and_external_id_overwrite() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    let lease = claim.lease();
    assert!(matches!(
        store.prepare_operation(&lease, OperationKind::StartTurn, 0, 11),
        Err(StateError::Conflict { .. })
    ));
    let operation = store
        .prepare_operation(&lease, OperationKind::StartThread, 0, 11)
        .unwrap();
    let thread_id = ThreadId::from_str("thread-alpha").unwrap();
    assert!(matches!(
        store.record_thread_started(&lease, &operation.operation_id, &thread_id, 12),
        Err(StateError::Conflict { .. })
    ));
    store
        .mark_dispatching_thread(&lease, &operation.operation_id, 13)
        .unwrap();
    assert!(matches!(
        store.mark_dispatching_thread(&lease, &operation.operation_id, 14),
        Err(StateError::Conflict { .. })
    ));
    store
        .record_thread_started(&lease, &operation.operation_id, &thread_id, 15)
        .unwrap();
    let different = ThreadId::from_str("thread-different").unwrap();
    assert!(matches!(
        store.record_thread_started(&lease, &operation.operation_id, &different, 16),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(
        store
            .get_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .thread_id,
        Some(thread_id)
    );
    assert!(matches!(
        store.prepare_operation(&lease, OperationKind::StartThread, 1, 17),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn lease_renewal_uses_owner_and_expiry_cas_and_expiry_preserves_identity() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 20)
        .unwrap()
        .unwrap();
    let lease = claim.lease();
    let renewed = store.renew_lease(&lease, 30, 11).unwrap();
    let wrong_owner = store.test_only_forge_lease_from_visible(
        renewed.attempt_id().clone(),
        "stale-worker",
        renewed.expires_at(),
    );
    assert!(matches!(
        store.renew_lease(&wrong_owner, 40, 12),
        Err(StateError::Conflict { .. })
    ));
    assert!(matches!(
        store.renew_lease(&lease, 40, 12),
        Err(StateError::Conflict { .. })
    ));

    let operation = store
        .prepare_operation(&renewed, OperationKind::StartThread, 0, 13)
        .unwrap();
    store
        .mark_dispatching_thread(&renewed, &operation.operation_id, 14)
        .unwrap();
    let thread_id = ThreadId::from_str("thread-alpha").unwrap();
    store
        .record_thread_started(&renewed, &operation.operation_id, &thread_id, 15)
        .unwrap();
    set_test_lease_now(&state_path(&directory), 29);
    assert_eq!(store.expire_leases(&run.run_id, 29).unwrap(), 0);
    set_test_lease_now(&state_path(&directory), 30);
    assert_eq!(store.expire_leases(&run.run_id, 30).unwrap(), 1);

    let attempt = store.get_attempt(&claim.attempt_id).unwrap().unwrap();
    assert_eq!(attempt.state, AttemptState::ThreadStarted);
    assert_eq!(attempt.thread_id, Some(thread_id));
    assert_eq!(attempt.lease_owner, None);
    assert_eq!(attempt.lease_expires_at, None);
    assert!(matches!(
        store.renew_lease(&renewed, 40, 31),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn continuation_count_uses_compare_and_swap() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    let lease = claim.lease();

    assert_eq!(store.increment_continuation(&lease, 0, 2, 11).unwrap(), 1);
    assert!(matches!(
        store.increment_continuation(&lease, 0, 2, 12),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(store.increment_continuation(&lease, 1, 2, 13).unwrap(), 2);
    assert!(matches!(
        store.increment_continuation(&lease, 2, 2, 14),
        Err(StateError::LimitExceeded { .. })
    ));
}

#[test]
fn usage_is_monotonic_and_exceeded_actual_cost_is_committed() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task_with_budget("alpha", &[], 100, 1_000)]),
            &serde_json::json!({}),
            &RunBudget::new(100, 1_000, 3_600).unwrap(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    let lease = claim.lease();

    assert_eq!(
        store.reconcile_usage(&lease, 0, 80, 800, 11).unwrap(),
        UsageOutcome::WithinBudget
    );
    let exceeded = store.reconcile_usage(&lease, 80, 120, 1_200, 12).unwrap();
    assert_eq!(
        exceeded,
        UsageOutcome::Exceeded {
            max_tokens: 100,
            observed_tokens: 120,
            max_storage_bytes: 1_000,
            observed_storage_bytes: 1_200,
        }
    );
    assert_eq!(
        store
            .get_attempt(&claim.attempt_id)
            .unwrap()
            .unwrap()
            .observed_tokens,
        120
    );
    assert!(matches!(
        store.reconcile_usage(&lease, 80, 130, 1_300, 13),
        Err(StateError::Conflict { .. })
    ));
    assert!(matches!(
        store.reconcile_usage(&lease, 120, 119, 1_300, 14),
        Err(StateError::InvalidInput { .. })
    ));
}

#[test]
fn aggregate_usage_over_u64_records_actual_cost_before_reporting_limit() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, lease) = create_single_task_run(&mut store, "alpha");
    insert_competing_attempt(&path, &run_id, "alpha", 1, i64::MAX);
    insert_competing_attempt(&path, &run_id, "alpha", 2, i64::MAX);

    let outcome = store
        .reconcile_usage(&lease, 0, 1, 0, 11)
        .expect("real usage must commit even when aggregate exceeds u64");

    assert_eq!(
        outcome,
        UsageOutcome::Exceeded {
            max_tokens: 1_000,
            observed_tokens: u128::from(u64::MAX),
            max_storage_bytes: 10_000,
            observed_storage_bytes: 154,
        }
    );
    assert_eq!(
        store
            .get_attempt(lease.attempt_id())
            .unwrap()
            .unwrap()
            .observed_tokens,
        1
    );
}

#[test]
fn open_hook_rejects_database_path_replacement_between_checks() {
    let directory = private_directory();
    let path = state_path(&directory);
    let replaced = directory.path().join("replaced.sqlite");

    let error = StateStore::open_with_test_hook(&path, || {
        fs::rename(&path, &replaced).unwrap();
        fs::write(&path, []).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    })
    .expect_err("replacement must be rejected");

    assert!(matches!(
        error,
        StateError::InvalidInput { .. } | StateError::Sqlite { .. }
    ));
}

#[test]
fn schema_enforces_foreign_keys_and_check_constraints() {
    let directory = private_directory();
    let path = state_path(&directory);
    let _store = open_state(&path).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();

    let invalid_state = connection.execute(
        "INSERT INTO runs (
            run_id, graph_sha256, graph_json, provenance_json,
            max_tokens, max_storage_bytes, max_wall_seconds,
            state, cancellation_requested, created_at, updated_at
         ) VALUES (?1, ?2, X'7b7d', X'7b7d', 1, 1, 1, 'unknown', 0, 1, 1)",
        rusqlite::params![RunId::new().to_string(), "a".repeat(64)],
    );
    assert!(invalid_state.is_err());

    let missing_run_task = connection.execute(
        "INSERT INTO tasks (
            run_id, task_id, kind, state, task_json, created_at, updated_at
         ) VALUES (?1, 'task', 'analysis', 'ready', X'7b7d', 1, 1)",
        [RunId::new().to_string()],
    );
    assert!(missing_run_task.is_err());
}

#[test]
fn reconciliation_and_failure_transitions_reject_stale_states() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, lease) = create_single_task_run(&mut store, "alpha");
    assert!(matches!(
        store.mark_reconciling(&lease, 11),
        Err(StateError::Conflict { .. })
    ));

    let operation = store
        .prepare_operation(&lease, OperationKind::StartThread, 0, 12)
        .unwrap();
    assert!(matches!(
        store.mark_operation_failed(&lease, &operation.operation_id, true, "network", 13),
        Err(StateError::Conflict { .. })
    ));
    store
        .mark_dispatching_thread(&lease, &operation.operation_id, 14)
        .unwrap();
    store
        .mark_operation_failed(&lease, &operation.operation_id, true, "network", 15)
        .unwrap();
    assert_eq!(
        store
            .get_attempt(lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Failed
    );
    assert_eq!(
        store
            .get_operation(&operation.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Failed
    );
    assert!(matches!(
        store.mark_indeterminate(&lease, "late-observation", 16),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn artifact_registry_is_idempotent_and_rejects_conflicting_metadata() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let artifact = artifact(0xaa, "application/json", 7);
    store.register_artifact(&artifact, 1).unwrap();
    store.register_artifact(&artifact, 2).unwrap();
    let mut conflicting_ref = artifact.clone();
    conflicting_ref.media_type = "application/json".to_owned();
    conflicting_ref.size_bytes = 8;
    assert!(matches!(
        store.register_artifact(&conflicting_ref, 3),
        Err(StateError::Integrity { .. })
    ));
}

#[test]
fn acceptance_is_exactly_once_across_connections() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, lease) = create_single_task_run(&mut store, "alpha");
    publish_success(&mut store, &lease, "alpha", 11);
    let attempt_id = lease.attempt_id().clone();
    drop(store);

    let barrier = Arc::new(Barrier::new(2));
    let handles = (0..2)
        .map(|offset| {
            let path = path.clone();
            let run_id = run_id.clone();
            let attempt_id = attempt_id.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut store = open_state(&path).unwrap();
                barrier.wait();
                store
                    .accept_result(
                        &run_id,
                        &TaskId::from_str("alpha").unwrap(),
                        &attempt_id,
                        20 + offset,
                    )
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| **outcome == AcceptResult::Accepted)
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                **outcome
                    == AcceptResult::AlreadyAccepted {
                        attempt_id: attempt_id.clone(),
                    }
            })
            .count(),
        1
    );
    let mut reopened = open_state(&path).unwrap();
    let task = reopened
        .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(task.state, TaskState::Completed);
    assert_eq!(task.accepted_attempt_id, Some(attempt_id.clone()));
    assert_eq!(
        reopened.get_attempt(&attempt_id).unwrap().unwrap().state,
        AttemptState::Succeeded
    );
    assert_eq!(
        reopened.get_run(&run_id).unwrap().unwrap().state,
        RunState::Completed
    );
}

#[test]
fn acceptance_quarantines_competing_attempt_without_discarding_cost() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, accepted_lease) = create_single_task_run(&mut store, "alpha");
    let competing = insert_competing_attempt(&path, &run_id, "alpha", 1, 33);
    publish_success(&mut store, &accepted_lease, "alpha", 11);
    let accepted_attempt = accepted_lease.attempt_id().clone();

    assert_eq!(
        store
            .accept_result(
                &run_id,
                &TaskId::from_str("alpha").unwrap(),
                &accepted_attempt,
                12
            )
            .unwrap(),
        AcceptResult::Accepted
    );
    let competing_record = store.get_attempt(&competing).unwrap().unwrap();
    assert_eq!(competing_record.state, AttemptState::Indeterminate);
    assert_eq!(competing_record.observed_tokens, 33);
    assert_eq!(competing_record.lease_owner, None);
    assert_eq!(store.reservation_usage(&competing).unwrap(), Some((33, 77)));
}

#[test]
fn acceptance_activates_dependencies_once_and_cancellation_blocks_activation() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[]), task("beta", &["alpha"])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    publish_success(&mut store, &claim.lease(), "alpha", 11);
    store
        .accept_result(
            &run.run_id,
            &TaskId::from_str("alpha").unwrap(),
            &claim.attempt_id,
            23,
        )
        .unwrap();
    assert_eq!(
        store
            .ready_tasks(&run.run_id)
            .unwrap()
            .iter()
            .map(|task| task.task_id.to_string())
            .collect::<Vec<_>>(),
        ["beta"]
    );
    assert_eq!(store.rebuild_ready_tasks(&run.run_id, 13).unwrap(), 0);

    let cancelled = store
        .create_run(
            &graph(vec![task("root", &[]), task("dependent", &["root"])]),
            &serde_json::json!({}),
            &run_budget(),
            20,
        )
        .unwrap();
    let cancelled_claim = store
        .claim_ready_task(&cancelled.run_id, "worker", 21, 100)
        .unwrap()
        .unwrap();
    let cancelled_competitor =
        insert_competing_attempt(store.path(), &cancelled.run_id, "root", 1, 17);
    assert!(store
        .request_run_cancellation(&cancelled.run_id, 22)
        .unwrap());
    assert!(!store
        .request_run_cancellation(&cancelled.run_id, 23)
        .unwrap());
    assert_eq!(store.rebuild_ready_tasks(&cancelled.run_id, 24).unwrap(), 0);
    assert!(store
        .claim_ready_task(&cancelled.run_id, "worker", 24, 100)
        .unwrap()
        .is_none());
    store
        .finalize_task_cancelled(&cancelled_claim.lease(), 25)
        .unwrap();
    let cancelled_competitor = store.get_attempt(&cancelled_competitor).unwrap().unwrap();
    assert_eq!(cancelled_competitor.state, AttemptState::Cancelled);
    assert_eq!(cancelled_competitor.lease_owner, None);
    assert_eq!(
        store.get_run(&cancelled.run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert!(store.ready_tasks(&cancelled.run_id).unwrap().is_empty());
}

#[test]
fn reopen_recovery_queries_preserve_nonterminal_state_events_and_readiness() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[]), task("beta", &["alpha"])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    let lease = claim.lease();
    let operation = store
        .prepare_operation(&lease, OperationKind::StartThread, 0, 11)
        .unwrap();
    store
        .mark_dispatching_thread(&lease, &operation.operation_id, 12)
        .unwrap();
    let thread_id = ThreadId::from_str("thread-alpha").unwrap();
    store
        .record_thread_started(&lease, &operation.operation_id, &thread_id, 13)
        .unwrap();
    drop(store);

    let mut reopened = open_state(&path).unwrap();
    assert_eq!(reopened.incomplete_runs().unwrap().len(), 1);
    let attempts = reopened.nonterminal_attempts(&run.run_id).unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].state, AttemptState::ThreadStarted);
    assert_eq!(attempts[0].thread_id, Some(thread_id));
    assert!(collect_events(&mut reopened, &run.run_id, 2).len() >= 4);
    assert_eq!(reopened.rebuild_ready_tasks(&run.run_id, 14).unwrap(), 0);
}

#[test]
fn malformed_persisted_identifiers_and_json_fail_closed() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, lease) = create_single_task_run(&mut store, "alpha");
    drop(store);

    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE attempts SET thread_id = '' WHERE attempt_id = ?1",
            [lease.attempt_id().to_string()],
        )
        .unwrap();
    drop(connection);
    let mut reopened = open_state(&path).unwrap();
    assert!(matches!(
        reopened.get_attempt(lease.attempt_id()),
        Err(StateError::Integrity { .. })
    ));
    drop(reopened);

    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE attempts SET thread_id = NULL WHERE attempt_id = ?1",
            [lease.attempt_id().to_string()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE tasks SET task_json = X'7b' WHERE run_id = ?1 AND task_id = 'alpha'",
            [run_id.to_string()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE tasks SET state = 'ready' WHERE run_id = ?1 AND task_id = 'alpha'",
            [run_id.to_string()],
        )
        .unwrap();
    drop(connection);
    let mut reopened = open_state(&path).unwrap();
    assert!(matches!(
        reopened.claim_ready_task(&run_id, "worker", 20, 30),
        Err(StateError::Integrity { .. } | StateError::Serialization { .. })
    ));
}

fn register_result_artifacts(
    store: &mut StateStore,
    result: &ResultEnvelope,
    result_ref: &ArtifactRef,
    now: i64,
) {
    store.register_artifact(result_ref, now).unwrap();
    if let Some(answer) = &result.answer_ref {
        store.register_artifact(answer, now).unwrap();
    }
    for evidence in &result.evidence {
        store.register_artifact(evidence, now).unwrap();
    }
    store.register_artifact(&result.trace_ref, now).unwrap();
}

fn prepare_test_turn_operation(
    store: &mut StateStore,
    lease: &LeaseToken,
    kind: OperationKind,
    ordinal: u32,
    now: i64,
) -> harp_state::OperationRecord {
    let operation_id = harp_contracts::OperationId::new();
    let intent = TurnSpec {
        instruction: format!("test {} intent", kind.as_str()),
        operation_marker: operation_id.clone(),
        output_schema: serde_json::json!({"type": "object"}),
        model: Some("default".to_owned()),
        reasoning_effort: None,
    };
    let checkpoint = (kind == OperationKind::ContinueTurn).then(|| "dd".repeat(32));
    store
        .prepare_turn_operation(
            lease,
            &harp_state::TurnOperationPreparation {
                operation_id,
                kind,
                ordinal,
                intent,
                checkpoint_sha256: checkpoint,
            },
            now,
        )
        .unwrap()
}

fn prepare_budget_interrupt(
    store: &mut StateStore,
    lease: &LeaseToken,
    ordinal: u32,
    now: i64,
) -> harp_state::OperationRecord {
    let intent = harp_state::InterruptIntent::budget(
        harp_state::InterruptBudgetDimension::Wall,
        "test_budget_limit",
    )
    .unwrap();
    store
        .prepare_interrupt_operation(lease, ordinal, &intent, now)
        .unwrap()
}

fn prepare_cancellation_interrupt(
    store: &mut StateStore,
    lease: &LeaseToken,
    ordinal: u32,
    now: i64,
) -> harp_state::OperationRecord {
    let intent = harp_state::InterruptIntent::cancellation("run_cancellation").unwrap();
    store
        .prepare_interrupt_operation(lease, ordinal, &intent, now)
        .unwrap()
}

fn drive_to_turn_started(
    store: &mut StateStore,
    lease: &LeaseToken,
    base: i64,
) -> (harp_contracts::OperationId, harp_contracts::OperationId) {
    let thread = store
        .prepare_operation(lease, OperationKind::StartThread, 0, base)
        .unwrap();
    store
        .mark_dispatching_thread(lease, &thread.operation_id, base + 1)
        .unwrap();
    store
        .record_thread_started(
            lease,
            &thread.operation_id,
            &ThreadId::from_str("thread-lineage").unwrap(),
            base + 2,
        )
        .unwrap();
    let turn = prepare_test_turn_operation(store, lease, OperationKind::StartTurn, 0, base + 3);
    store
        .mark_dispatching_turn(lease, &turn.operation_id, base + 4)
        .unwrap();
    store
        .record_turn_started(
            lease,
            &turn.operation_id,
            &TurnId::from_str("turn-0").unwrap(),
            base + 5,
        )
        .unwrap();
    (thread.operation_id, turn.operation_id)
}

fn prepare_completed_operation(
    store: &mut StateStore,
    lease: &LeaseToken,
    kind: OperationKind,
    ordinal: u32,
    now: i64,
) -> harp_contracts::OperationId {
    let operation = store.prepare_operation(lease, kind, ordinal, now).unwrap();
    store
        .mark_operation_dispatching(lease, &operation.operation_id, now + 1)
        .unwrap();
    store
        .complete_operation(lease, &operation.operation_id, None, now + 2)
        .unwrap();
    operation.operation_id
}

fn publish_success(
    store: &mut StateStore,
    lease: &LeaseToken,
    task_id: &str,
    base: i64,
) -> ArtifactRef {
    drive_to_turn_started(store, lease, base);
    let (mut result, _) = result(task_id);
    result.token_usage = 42;
    store
        .reconcile_usage(lease, 0, result.token_usage, 0, base + 6)
        .unwrap();
    let operation =
        prepare_completed_operation(store, lease, OperationKind::PublishResult, 0, base + 7);
    let bytes = serde_json::to_vec(&result).unwrap();
    let result_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&bytes)),
        "application/vnd.harp.result+json",
        bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(store, &result, &result_ref, base + 10);
    store
        .record_result_published(lease, &operation, &result, &result_ref, base + 11)
        .unwrap();
    result_ref
}

#[test]
fn turn_operations_preserve_distinct_markers_and_external_ids() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");
    let (_thread, first_turn) = drive_to_turn_started(&mut store, &lease, 11);

    assert_eq!(store.increment_continuation(&lease, 0, 2, 20).unwrap(), 1);
    let second =
        prepare_test_turn_operation(&mut store, &lease, OperationKind::ContinueTurn, 0, 21);
    store
        .mark_dispatching_turn(&lease, &second.operation_id, 22)
        .unwrap();
    store
        .record_turn_started(
            &lease,
            &second.operation_id,
            &TurnId::from_str("turn-1").unwrap(),
            23,
        )
        .unwrap();

    let first = store.get_operation(&first_turn).unwrap().unwrap();
    assert_eq!(
        first.operation_marker.as_deref(),
        Some(format!("harp-operation:{first_turn}").as_str())
    );
    assert_eq!(first.external_id.as_deref(), Some("turn-0"));
    let second = store.get_operation(&second.operation_id).unwrap().unwrap();
    assert_eq!(
        second.operation_marker.as_deref(),
        Some(format!("harp-operation:{}", second.operation_id).as_str())
    );
    assert_eq!(second.external_id.as_deref(), Some("turn-1"));
    let attempt = store.get_attempt(lease.attempt_id()).unwrap().unwrap();
    assert_eq!(
        attempt.latest_turn_id,
        Some(TurnId::from_str("turn-1").unwrap())
    );
    assert_eq!(
        attempt.latest_operation_marker.as_deref(),
        Some(format!("harp-operation:{}", second.operation_id).as_str())
    );
    assert!(matches!(
        store.record_turn_started(
            &lease,
            &first.operation_id,
            &TurnId::from_str("turn-overwrite").unwrap(),
            24
        ),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn turn_operation_persists_exact_intent_before_dispatch() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");
    let thread = store
        .prepare_operation(&lease, OperationKind::StartThread, 0, 11)
        .unwrap();
    store
        .mark_dispatching_thread(&lease, &thread.operation_id, 12)
        .unwrap();
    store
        .record_thread_started(
            &lease,
            &thread.operation_id,
            &ThreadId::from_str("thread-intent").unwrap(),
            13,
        )
        .unwrap();
    let operation_id = harp_contracts::OperationId::new();
    let intent = TurnSpec {
        instruction: "Pinned exact instruction".to_owned(),
        operation_marker: operation_id.clone(),
        output_schema: serde_json::json!({
            "type": "object",
            "required": ["schemaVersion", "taskId"]
        }),
        model: Some("default".to_owned()),
        reasoning_effort: None,
    };

    let operation = store
        .prepare_turn_operation(
            &lease,
            &harp_state::TurnOperationPreparation {
                operation_id,
                kind: OperationKind::StartTurn,
                ordinal: 0,
                intent: intent.clone(),
                checkpoint_sha256: None,
            },
            14,
        )
        .unwrap();
    let loaded = store
        .get_operation(&operation.operation_id)
        .unwrap()
        .unwrap();

    assert_eq!(loaded.turn_intent.as_ref(), Some(&intent));
    assert_eq!(
        loaded.intent_sha256.as_deref(),
        Some(format!("{:x}", Sha256::digest(serde_json::to_vec(&intent).unwrap())).as_str())
    );
    assert_eq!(loaded.checkpoint_sha256, None);
    store
        .mark_dispatching_turn(&lease, &operation.operation_id, 15)
        .unwrap();
}

#[test]
fn interrupt_operation_persists_typed_reason_and_dimension_before_dispatch() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 11);
    let intent = harp_state::InterruptIntent::budget(
        harp_state::InterruptBudgetDimension::Storage,
        "aggregate_storage_limit",
    )
    .unwrap();

    let operation = store
        .prepare_interrupt_operation(&lease, 0, &intent, 20)
        .unwrap();
    let loaded = store
        .get_operation(&operation.operation_id)
        .unwrap()
        .unwrap();

    assert_eq!(loaded.interrupt_intent.as_ref(), Some(&intent));
    assert_eq!(loaded.turn_intent, None);
    assert_eq!(
        loaded.intent_sha256.as_deref(),
        Some(format!("{:x}", Sha256::digest(serde_json::to_vec(&intent).unwrap())).as_str())
    );
    assert_eq!(loaded.target_external_id.as_deref(), Some("turn-0"));
}

#[test]
fn generic_interrupt_publish_and_evaluate_operations_are_ordered() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 11);

    let interrupt = prepare_budget_interrupt(&mut store, &lease, 0, 20);
    assert!(matches!(
        store.complete_operation(&lease, &interrupt.operation_id, None, 21),
        Err(StateError::Conflict { .. })
    ));
    store
        .mark_operation_dispatching(&lease, &interrupt.operation_id, 21)
        .unwrap();
    store
        .complete_operation(&lease, &interrupt.operation_id, Some("interrupt-0"), 22)
        .unwrap();
    assert_eq!(
        store
            .get_attempt(lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Reconciling
    );

    let publish =
        prepare_completed_operation(&mut store, &lease, OperationKind::PublishResult, 0, 23);
    let mut result = result("alpha").0;
    result.token_usage = 0;
    let bytes = serde_json::to_vec(&result).unwrap();
    let result_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&bytes)),
        "application/vnd.harp.result+json",
        bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &result, &result_ref, 26);
    store
        .record_result_published(&lease, &publish, &result, &result_ref, 27)
        .unwrap();

    let evaluate = store
        .prepare_operation(&lease, OperationKind::Evaluate, 0, 28)
        .unwrap();
    store
        .mark_operation_dispatching(&lease, &evaluate.operation_id, 29)
        .unwrap();
    store
        .complete_operation(&lease, &evaluate.operation_id, Some("evaluation-0"), 30)
        .unwrap();
}

#[test]
fn result_publication_requires_registered_nested_artifacts_and_matching_usage() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 11);
    store.reconcile_usage(&lease, 0, 42, 0, 20).unwrap();
    let publish =
        prepare_completed_operation(&mut store, &lease, OperationKind::PublishResult, 0, 21);
    let (result, result_ref) = result("alpha");
    store.register_artifact(&result_ref, 24).unwrap();
    assert!(matches!(
        store.record_result_published(&lease, &publish, &result, &result_ref, 25),
        Err(StateError::Integrity { .. })
    ));
    register_result_artifacts(&mut store, &result, &result_ref, 26);
    store
        .record_result_published(&lease, &publish, &result, &result_ref, 27)
        .unwrap();
    let attempt = store.get_attempt(lease.attempt_id()).unwrap().unwrap();
    assert_eq!(attempt.result_status, Some(ResultStatus::Success));
    assert_eq!(attempt.result_token_usage, Some(42));
}

#[test]
fn result_token_mismatch_blocks_publication_and_acceptance() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 11);
    let publish =
        prepare_completed_operation(&mut store, &lease, OperationKind::PublishResult, 0, 20);
    let (result, result_ref) = result("alpha");
    register_result_artifacts(&mut store, &result, &result_ref, 23);
    assert!(matches!(
        store.record_result_published(&lease, &publish, &result, &result_ref, 24),
        Err(StateError::Conflict { .. })
    ));

    store.reconcile_usage(&lease, 0, 42, 0, 25).unwrap();
    store
        .record_result_published(&lease, &publish, &result, &result_ref, 26)
        .unwrap();
    drop(store);

    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE attempts
             SET result_token_usage = 41
             WHERE attempt_id = ?1",
            [lease.attempt_id().to_string()],
        )
        .unwrap();
    drop(connection);
    let mut reopened = open_state(&path).unwrap();
    assert!(matches!(
        reopened.accept_result(
            &run_id,
            &TaskId::from_str("alpha").unwrap(),
            lease.attempt_id(),
            27
        ),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn partial_result_is_not_accepted_and_later_success_can_publish() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 11);
    let partial_op =
        prepare_completed_operation(&mut store, &lease, OperationKind::PublishResult, 0, 20);
    let (mut partial, _) = result("alpha");
    partial.status = ResultStatus::Partial;
    partial.token_usage = 0;
    let partial_bytes = serde_json::to_vec(&partial).unwrap();
    let partial_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&partial_bytes)),
        "application/vnd.harp.result+json",
        partial_bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &partial, &partial_ref, 23);
    store
        .record_result_published(&lease, &partial_op, &partial, &partial_ref, 24)
        .unwrap();
    assert_eq!(
        store
            .get_attempt(lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Reconciling
    );
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Running
    );
    assert!(matches!(
        store.accept_result(
            &run_id,
            &TaskId::from_str("alpha").unwrap(),
            lease.attempt_id(),
            25
        ),
        Err(StateError::Conflict { .. })
    ));

    let success_op =
        prepare_completed_operation(&mut store, &lease, OperationKind::PublishResult, 1, 26);
    let (mut success, _) = result("alpha");
    success.token_usage = 0;
    let success_bytes = serde_json::to_vec(&success).unwrap();
    let success_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&success_bytes)),
        "application/vnd.harp.result+json",
        success_bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &success, &success_ref, 29);
    store
        .record_result_published(&lease, &success_op, &success, &success_ref, 30)
        .unwrap();
    assert_eq!(
        store
            .accept_result(
                &run_id,
                &TaskId::from_str("alpha").unwrap(),
                lease.attempt_id(),
                31
            )
            .unwrap(),
        AcceptResult::Accepted
    );
}

#[test]
fn failed_and_cancelled_results_never_enter_acceptance_states() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();

    let (failed_run, failed_lease) = claim_single_task(&mut store, "failed-task");
    drive_to_turn_started(&mut store, &failed_lease, 11);
    let failed_operation = prepare_completed_operation(
        &mut store,
        &failed_lease,
        OperationKind::PublishResult,
        0,
        20,
    );
    let (mut failed, _) = result("failed-task");
    failed.status = ResultStatus::Failed;
    failed.answer_ref = None;
    failed.failure_class = Some("semantic".to_owned());
    failed.token_usage = 0;
    let failed_bytes = serde_json::to_vec(&failed).unwrap();
    let failed_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&failed_bytes)),
        "application/vnd.harp.result+json",
        failed_bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &failed, &failed_ref, 23);
    store
        .record_result_published(&failed_lease, &failed_operation, &failed, &failed_ref, 24)
        .unwrap();
    assert_eq!(
        store
            .get_attempt(failed_lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Failed
    );
    assert_eq!(
        store.get_run(&failed_run).unwrap().unwrap().state,
        RunState::Failed
    );
    assert_eq!(
        store
            .get_task(&failed_run, &TaskId::from_str("failed-task").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Failed
    );
    assert!(matches!(
        store.accept_result(
            &failed_run,
            &TaskId::from_str("failed-task").unwrap(),
            failed_lease.attempt_id(),
            25
        ),
        Err(StateError::Conflict { .. })
    ));

    let (cancelled_run, cancelled_lease) = claim_single_task(&mut store, "cancelled-task");
    drive_to_turn_started(&mut store, &cancelled_lease, 30);
    let cancelled_operation = prepare_completed_operation(
        &mut store,
        &cancelled_lease,
        OperationKind::PublishResult,
        0,
        39,
    );
    let (mut cancelled, _) = result("cancelled-task");
    cancelled.status = ResultStatus::Cancelled;
    cancelled.answer_ref = None;
    cancelled.failure_class = None;
    cancelled.token_usage = 0;
    let cancelled_bytes = serde_json::to_vec(&cancelled).unwrap();
    let cancelled_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&cancelled_bytes)),
        "application/vnd.harp.result+json",
        cancelled_bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &cancelled, &cancelled_ref, 42);
    assert!(store.request_run_cancellation(&cancelled_run, 42).unwrap());
    store
        .record_result_published(
            &cancelled_lease,
            &cancelled_operation,
            &cancelled,
            &cancelled_ref,
            43,
        )
        .unwrap();
    assert_eq!(
        store
            .get_attempt(cancelled_lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Cancelled
    );
    assert_eq!(
        store.get_run(&cancelled_run).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert_eq!(
        store
            .get_task(&cancelled_run, &TaskId::from_str("cancelled-task").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert!(matches!(
        store.accept_result(
            &cancelled_run,
            &TaskId::from_str("cancelled-task").unwrap(),
            cancelled_lease.attempt_id(),
            44
        ),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn transient_failure_retries_then_exhausts_and_indeterminate_requeues() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let first = store
        .claim_ready_task(&run.run_id, "worker-1", 10, 100)
        .unwrap()
        .unwrap();
    let first_lease = first.lease();
    let operation = store
        .prepare_operation(&first_lease, OperationKind::StartThread, 0, 11)
        .unwrap();
    store
        .mark_dispatching_thread(&first_lease, &operation.operation_id, 12)
        .unwrap();
    store
        .mark_operation_failed(&first_lease, &operation.operation_id, true, "network", 13)
        .unwrap();
    let second = store
        .claim_ready_task(&run.run_id, "worker-2", 14, 100)
        .unwrap()
        .unwrap();
    assert_eq!(
        store
            .get_attempt(&second.attempt_id)
            .unwrap()
            .unwrap()
            .ordinal,
        1
    );
    let second_lease = second.lease();
    let operation = store
        .prepare_operation(&second_lease, OperationKind::StartThread, 0, 15)
        .unwrap();
    store
        .mark_dispatching_thread(&second_lease, &operation.operation_id, 16)
        .unwrap();
    store
        .mark_operation_failed(&second_lease, &operation.operation_id, true, "network", 17)
        .unwrap();
    assert_eq!(
        store.get_run(&run.run_id).unwrap().unwrap().state,
        RunState::Failed
    );

    let other = store
        .create_run(
            &graph(vec![task("beta", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            20,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&other.run_id, "worker", 21, 100)
        .unwrap()
        .unwrap();
    store.reconcile_usage(&claim.lease(), 0, 9, 13, 22).unwrap();
    store
        .mark_indeterminate(&claim.lease(), "lost-response", 23)
        .unwrap();
    let retry = store
        .claim_ready_task(&other.run_id, "worker-2", 24, 100)
        .unwrap()
        .unwrap();
    assert_eq!(
        store
            .get_attempt(&retry.attempt_id)
            .unwrap()
            .unwrap()
            .ordinal,
        1
    );
    assert_eq!(
        store.reservation_usage(&claim.attempt_id).unwrap(),
        Some((9, 13))
    );

    let nontransient = store
        .create_run(
            &graph(vec![task("gamma", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            30,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&nontransient.run_id, "worker", 31, 100)
        .unwrap()
        .unwrap();
    let lease = claim.lease();
    let operation = store
        .prepare_operation(&lease, OperationKind::StartThread, 0, 32)
        .unwrap();
    store
        .mark_dispatching_thread(&lease, &operation.operation_id, 33)
        .unwrap();
    store
        .mark_operation_failed(&lease, &operation.operation_id, false, "semantic", 34)
        .unwrap();
    assert_eq!(
        store.get_run(&nontransient.run_id).unwrap().unwrap().state,
        RunState::Failed
    );
    assert!(store
        .claim_ready_task(&nontransient.run_id, "worker-2", 35, 100)
        .unwrap()
        .is_none());
}

#[test]
fn fresh_cancellation_terminalizes_unstarted_work_immediately() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[]), task("beta", &["alpha"])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    assert!(store.request_run_cancellation(&run.run_id, 2).unwrap());
    assert_eq!(
        store.get_run(&run.run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("beta").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert!(!store.request_run_cancellation(&run.run_id, 3).unwrap());
}

#[test]
fn active_turn_cancellation_requires_completed_interrupt() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 11);
    assert!(store.request_run_cancellation(&run_id, 20).unwrap());
    assert!(matches!(
        store.finalize_task_cancelled(&lease, 21),
        Err(StateError::Conflict { .. })
    ));

    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 11);
    assert!(store.request_run_cancellation(&run_id, 20).unwrap());
    let interrupt = prepare_cancellation_interrupt(&mut store, &lease, 0, 21);
    store
        .mark_operation_dispatching(&lease, &interrupt.operation_id, 22)
        .unwrap();
    store
        .complete_operation(&lease, &interrupt.operation_id, None, 23)
        .unwrap();
    assert_eq!(
        store
            .get_operation(&interrupt.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Completed
    );
    store.finalize_task_cancelled(&lease, 24).unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
}

#[test]
fn cancellation_interrupt_must_complete_for_the_latest_turn() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 10);
    let turn_zero = TurnId::from_str("turn-0").unwrap();
    let interrupt_zero = prepare_budget_interrupt(&mut store, &lease, 0, 20);
    assert_eq!(
        interrupt_zero.target_external_id.as_deref(),
        Some(turn_zero.to_string().as_str())
    );
    store
        .mark_operation_dispatching(&lease, &interrupt_zero.operation_id, 21)
        .unwrap();
    store
        .complete_operation(
            &lease,
            &interrupt_zero.operation_id,
            Some("interrupt-response-0"),
            22,
        )
        .unwrap();

    store.increment_continuation(&lease, 0, 2, 23).unwrap();
    let continuation =
        prepare_test_turn_operation(&mut store, &lease, OperationKind::ContinueTurn, 0, 24);
    store
        .mark_dispatching_turn(&lease, &continuation.operation_id, 25)
        .unwrap();
    store
        .record_turn_started(
            &lease,
            &continuation.operation_id,
            &TurnId::from_str("turn-1").unwrap(),
            26,
        )
        .unwrap();
    assert!(store.request_run_cancellation(&run_id, 27).unwrap());
    assert!(matches!(
        store.prepare_interrupt_operation(
            &lease,
            0,
            &harp_state::InterruptIntent::cancellation("run_cancellation").unwrap(),
            28,
        ),
        Err(StateError::Conflict { .. })
    ));
    assert!(matches!(
        store.finalize_task_cancelled(&lease, 28),
        Err(StateError::Conflict { .. })
    ));

    let failed_latest = prepare_cancellation_interrupt(&mut store, &lease, 1, 29);
    assert_eq!(failed_latest.target_external_id.as_deref(), Some("turn-1"));
    store
        .mark_operation_dispatching(&lease, &failed_latest.operation_id, 30)
        .unwrap();
    store
        .mark_operation_failed(&lease, &failed_latest.operation_id, true, "delivery", 31)
        .unwrap();
    assert!(matches!(
        store.finalize_task_cancelled(&lease, 32),
        Err(StateError::Conflict { .. })
    ));

    let completed_latest = prepare_cancellation_interrupt(&mut store, &lease, 2, 33);
    assert_eq!(
        completed_latest.target_external_id.as_deref(),
        Some("turn-1")
    );
    store
        .mark_operation_dispatching(&lease, &completed_latest.operation_id, 34)
        .unwrap();
    store
        .complete_operation(&lease, &completed_latest.operation_id, None, 35)
        .unwrap();
    store.finalize_task_cancelled(&lease, 36).unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
}

#[test]
fn unresolved_continuation_turn_dispatch_fences_cancellation() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    drive_to_turn_started(&mut store, &lease, 10);
    let historical_interrupt = prepare_budget_interrupt(&mut store, &lease, 0, 20);
    store
        .mark_operation_dispatching(&lease, &historical_interrupt.operation_id, 21)
        .unwrap();
    store
        .complete_operation(&lease, &historical_interrupt.operation_id, None, 22)
        .unwrap();
    store.increment_continuation(&lease, 0, 2, 23).unwrap();
    let continuation =
        prepare_test_turn_operation(&mut store, &lease, OperationKind::ContinueTurn, 0, 24);
    store
        .mark_dispatching_turn(&lease, &continuation.operation_id, 25)
        .unwrap();
    assert!(store.request_run_cancellation(&run_id, 26).unwrap());

    assert!(matches!(
        store.prepare_interrupt_operation(
            &lease,
            1,
            &harp_state::InterruptIntent::cancellation("run_cancellation").unwrap(),
            27,
        ),
        Err(StateError::Conflict { .. })
    ));
    assert!(matches!(
        store.finalize_task_cancelled(&lease, 27),
        Err(StateError::Conflict { .. })
    ));

    store
        .record_turn_started(
            &lease,
            &continuation.operation_id,
            &TurnId::from_str("turn-1").unwrap(),
            28,
        )
        .unwrap();
    let latest_interrupt = prepare_cancellation_interrupt(&mut store, &lease, 1, 29);
    assert_eq!(
        latest_interrupt.target_external_id.as_deref(),
        Some("turn-1")
    );
    store
        .mark_operation_dispatching(&lease, &latest_interrupt.operation_id, 30)
        .unwrap();
    store
        .complete_operation(&lease, &latest_interrupt.operation_id, None, 31)
        .unwrap();
    store.finalize_task_cancelled(&lease, 32).unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
}

#[test]
fn unresolved_initial_turn_can_converge_indeterminate_under_cancellation() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    let thread = store
        .prepare_operation(&lease, OperationKind::StartThread, 0, 10)
        .unwrap();
    store
        .mark_dispatching_thread(&lease, &thread.operation_id, 11)
        .unwrap();
    store
        .record_thread_started(
            &lease,
            &thread.operation_id,
            &ThreadId::from_str("thread-initial").unwrap(),
            12,
        )
        .unwrap();
    let turn = prepare_test_turn_operation(&mut store, &lease, OperationKind::StartTurn, 0, 13);
    store
        .mark_dispatching_turn(&lease, &turn.operation_id, 14)
        .unwrap();
    assert!(store.request_run_cancellation(&run_id, 15).unwrap());

    assert!(matches!(
        store.prepare_interrupt_operation(
            &lease,
            0,
            &harp_state::InterruptIntent::cancellation("run_cancellation").unwrap(),
            16,
        ),
        Err(StateError::Conflict { .. })
    ));
    assert!(matches!(
        store.finalize_task_cancelled(&lease, 16),
        Err(StateError::Conflict { .. })
    ));
    store
        .mark_indeterminate(&lease, "turn-start-ambiguous", 17)
        .unwrap();
    assert_eq!(
        store
            .get_attempt(lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Indeterminate
    );
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert!(store.ready_tasks(&run_id).unwrap().is_empty());
    assert_eq!(
        store
            .get_operation(&turn.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Cancelled
    );
}

#[test]
fn reservation_aggregation_near_i64_max_fails_without_sqlite_overflow() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[]), task("beta", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    store
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    insert_competing_attempt(&path, &run.run_id, "alpha", 1, i64::MAX);
    insert_competing_attempt(&path, &run.run_id, "alpha", 2, i64::MAX);

    assert!(matches!(
        store.claim_ready_task(&run.run_id, "worker-2", 11, 100),
        Err(StateError::BudgetExceeded { .. } | StateError::LimitExceeded { .. })
    ));
}

#[test]
fn indeterminate_attempts_share_the_transient_retry_budget() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("alpha", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();

    let first = store
        .claim_ready_task(&run.run_id, "worker-1", 10, 100)
        .unwrap()
        .unwrap();
    store
        .mark_indeterminate(&first.lease(), "lost-first", 11)
        .unwrap();
    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Ready
    );

    let second = store
        .claim_ready_task(&run.run_id, "worker-2", 12, 100)
        .unwrap()
        .unwrap();
    assert_eq!(
        store
            .get_attempt(&second.attempt_id)
            .unwrap()
            .unwrap()
            .ordinal,
        1
    );
    store
        .mark_indeterminate(&second.lease(), "lost-second", 13)
        .unwrap();

    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Failed
    );
    assert_eq!(
        store.get_run(&run.run_id).unwrap().unwrap().state,
        RunState::Failed
    );
    assert!(store
        .claim_ready_task(&run.run_id, "worker-3", 14, 100)
        .unwrap()
        .is_none());
}

#[test]
fn zero_observed_indeterminate_attempt_releases_unused_reservation_for_retry() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task_with_budget("alpha", &[], 100, 1_024)]),
            &serde_json::json!({}),
            &RunBudget::new(100, 1_024, 60).unwrap(),
            1,
        )
        .unwrap();
    let first = store
        .claim_ready_task(&run.run_id, "worker-1", 2, 100)
        .unwrap()
        .unwrap();
    store
        .mark_indeterminate(&first.lease(), "lost_thread", 3)
        .unwrap();

    let retry = store
        .claim_ready_task(&run.run_id, "worker-2", 4, 100)
        .unwrap()
        .expect("unused terminal reservation is released");

    assert_eq!(retry.budget.max_tokens, 100);
    assert_eq!(retry.budget.max_storage_bytes, 1_024);
}

#[test]
fn observed_indeterminate_cost_reduces_retry_remaining_budget() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task_with_budget("alpha", &[], 100, 1_024)]),
            &serde_json::json!({}),
            &RunBudget::new(100, 1_024, 60).unwrap(),
            1,
        )
        .unwrap();
    let first = store
        .claim_ready_task(&run.run_id, "worker-1", 2, 100)
        .unwrap()
        .unwrap();
    store
        .reconcile_usage(&first.lease(), 0, 30, 200, 3)
        .unwrap();
    store
        .mark_indeterminate(&first.lease(), "lost_thread", 4)
        .unwrap();

    let retry = store
        .claim_ready_task(&run.run_id, "worker-2", 5, 100)
        .unwrap()
        .expect("remaining budget supports bounded retry");

    assert_eq!(retry.budget.max_tokens, 70);
    assert_eq!(retry.budget.max_storage_bytes, 824);
}

#[test]
fn observed_indeterminate_wall_cost_reduces_retry_timeout_without_clamping() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task_with_budget("alpha", &[], 100, 1_024)]),
            &serde_json::json!({}),
            &RunBudget::new(100, 1_024, 60).unwrap(),
            1,
        )
        .unwrap();
    let first = store
        .claim_ready_task(&run.run_id, "worker-1", 2, 100)
        .unwrap()
        .unwrap();

    let outcome = store
        .reconcile_wall_usage(&first.lease(), 0, 17, 17, 3)
        .unwrap();
    assert_eq!(outcome.observed_attempt_wall_seconds, 17);
    assert_eq!(outcome.observed_run_wall_seconds, 17);
    store
        .mark_indeterminate(&first.lease(), "lost_thread", 4)
        .unwrap();

    let retry = store
        .claim_ready_task(&run.run_id, "worker-2", 5, 100)
        .unwrap()
        .expect("remaining wall budget supports bounded retry");
    assert_eq!(retry.budget.timeout_seconds, 43);

    let retry_outcome = store
        .reconcile_wall_usage(&retry.lease(), 0, 44, 44, 6)
        .unwrap();
    assert_eq!(retry_outcome.observed_attempt_wall_seconds, 44);
    assert_eq!(retry_outcome.observed_run_wall_seconds, 61);
    assert!(retry_outcome.run_limit_exceeded);
}

#[test]
fn wall_tracking_start_is_durable_idempotent_and_cannot_move_forward() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");

    let first = store.start_wall_tracking(&lease, 1_000, 20).unwrap();
    assert_eq!(first.wall_started_at, Some(1_000));
    let repeated = store.start_wall_tracking(&lease, 1_000, 21).unwrap();
    assert_eq!(repeated.wall_started_at, Some(1_000));
    assert!(matches!(
        store.start_wall_tracking(&lease, 1_001, 22),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn aggregate_wall_usage_across_attempts_exhausts_run_budget() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![
                task_with_budget("alpha", &[], 100, 1_024),
                task_with_budget("beta", &[], 100, 1_024),
            ]),
            &serde_json::json!({}),
            &RunBudget::new(200, 2_048, 60).unwrap(),
            1,
        )
        .unwrap();
    let alpha = store
        .claim_ready_task(&run.run_id, "worker-alpha", 2, 100)
        .unwrap()
        .unwrap();
    store
        .reconcile_wall_usage(&alpha.lease(), 0, 40, 40, 3)
        .unwrap();
    store
        .mark_indeterminate(&alpha.lease(), "duplicate_cost", 4)
        .unwrap();
    let beta = store
        .claim_ready_task(&run.run_id, "worker-beta", 5, 100)
        .unwrap()
        .unwrap();
    assert_eq!(beta.budget.timeout_seconds, 20);
    let outcome = store
        .reconcile_wall_usage(&beta.lease(), 0, 21, 21, 6)
        .unwrap();
    assert!(outcome.attempt_limit_exceeded);
    assert!(outcome.run_limit_exceeded);
    assert_eq!(outcome.observed_run_wall_seconds, 61);
}

#[test]
fn terminal_wall_overage_cannot_be_accepted() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    publish_success(&mut store, &lease, "alpha", 10);
    let wall = store.reconcile_wall_usage(&lease, 0, 61, 61, 30).unwrap();
    assert!(wall.attempt_limit_exceeded);

    assert!(matches!(
        store.accept_result(
            &run_id,
            &TaskId::from_str("alpha").unwrap(),
            lease.attempt_id(),
            31
        ),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn operation_failures_follow_operation_kind_authority() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();

    let (cancel_run, cancel_lease) = claim_single_task(&mut store, "cancel-task");
    drive_to_turn_started(&mut store, &cancel_lease, 10);
    assert!(store.request_run_cancellation(&cancel_run, 20).unwrap());
    let interrupt = prepare_cancellation_interrupt(&mut store, &cancel_lease, 0, 21);
    store
        .mark_operation_dispatching(&cancel_lease, &interrupt.operation_id, 22)
        .unwrap();
    store
        .mark_operation_failed(&cancel_lease, &interrupt.operation_id, true, "delivery", 23)
        .unwrap();
    assert_eq!(
        store
            .get_operation(&interrupt.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Failed
    );
    assert_eq!(
        store
            .get_attempt(cancel_lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Reconciling
    );
    assert!(matches!(
        store.finalize_task_cancelled(&cancel_lease, 24),
        Err(StateError::Conflict { .. })
    ));

    let (evaluate_run, evaluate_lease) = claim_single_task(&mut store, "evaluate-task");
    publish_success(&mut store, &evaluate_lease, "evaluate-task", 30);
    let evaluate = store
        .prepare_operation(&evaluate_lease, OperationKind::Evaluate, 0, 42)
        .unwrap();
    store
        .mark_operation_dispatching(&evaluate_lease, &evaluate.operation_id, 43)
        .unwrap();
    store
        .mark_operation_failed(
            &evaluate_lease,
            &evaluate.operation_id,
            true,
            "evaluator",
            44,
        )
        .unwrap();
    assert_eq!(
        store
            .get_attempt(evaluate_lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::ResultPublished
    );
    assert_eq!(
        store
            .get_task(&evaluate_run, &TaskId::from_str("evaluate-task").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::ResultPublished
    );
    let retry = store
        .prepare_operation(&evaluate_lease, OperationKind::Evaluate, 1, 45)
        .unwrap();
    assert_eq!(retry.state, OperationState::Prepared);
}

#[test]
fn continuation_can_start_from_reconciling_with_new_lineage() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");
    let (_thread, first_turn) = drive_to_turn_started(&mut store, &lease, 10);
    store.mark_reconciling(&lease, 20).unwrap();
    store.increment_continuation(&lease, 0, 2, 21).unwrap();

    let continuation =
        prepare_test_turn_operation(&mut store, &lease, OperationKind::ContinueTurn, 0, 22);
    store
        .mark_dispatching_turn(&lease, &continuation.operation_id, 23)
        .unwrap();
    store
        .record_turn_started(
            &lease,
            &continuation.operation_id,
            &TurnId::from_str("turn-next").unwrap(),
            24,
        )
        .unwrap();

    let first = store.get_operation(&first_turn).unwrap().unwrap();
    assert_eq!(first.external_id.as_deref(), Some("turn-0"));
    let continuation = store
        .get_operation(&continuation.operation_id)
        .unwrap()
        .unwrap();
    assert_eq!(continuation.external_id.as_deref(), Some("turn-next"));
    assert_eq!(
        continuation.operation_marker.as_deref(),
        Some(format!("harp-operation:{}", continuation.operation_id).as_str())
    );
}

#[test]
fn cancellation_terminalizes_published_results_and_blocks_acceptance() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    publish_success(&mut store, &lease, "alpha", 10);

    assert!(store.request_run_cancellation(&run_id, 30).unwrap());
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    let attempt = store.get_attempt(lease.attempt_id()).unwrap().unwrap();
    assert_eq!(attempt.state, AttemptState::Cancelled);
    assert_eq!(attempt.lease_owner, None);
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert!(matches!(
        store.accept_result(
            &run_id,
            &TaskId::from_str("alpha").unwrap(),
            lease.attempt_id(),
            31
        ),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn controller_accepts_durable_success_after_worker_lease_expiry_only() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    publish_success(&mut store, &lease, "alpha", 10);
    set_test_lease_now(&path, lease.expires_at());
    assert_eq!(store.expire_leases(&run_id, lease.expires_at()).unwrap(), 1);
    assert!(matches!(
        store.prepare_operation(&lease, OperationKind::Evaluate, 0, lease.expires_at() + 1),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(
        store
            .accept_result(
                &run_id,
                &TaskId::from_str("alpha").unwrap(),
                lease.attempt_id(),
                lease.expires_at() + 1
            )
            .unwrap(),
        AcceptResult::Accepted
    );

    set_test_lease_now(&path, 0);
    let (cancelled_run, cancelled_lease) = claim_single_task(&mut store, "beta");
    publish_success(&mut store, &cancelled_lease, "beta", 40);
    assert!(store.request_run_cancellation(&cancelled_run, 60).unwrap());
    assert!(matches!(
        store.accept_result(
            &cancelled_run,
            &TaskId::from_str("beta").unwrap(),
            cancelled_lease.attempt_id(),
            61
        ),
        Err(StateError::Conflict { .. })
    ));
}

#[test]
fn mixed_cancellation_converges_published_pending_and_running_work() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![
                task("active", &[]),
                task("published", &[]),
                task("pending", &["active"]),
            ]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let active = store
        .claim_ready_task(&run.run_id, "active-worker", 10, 100)
        .unwrap()
        .unwrap();
    let published = store
        .claim_ready_task(&run.run_id, "published-worker", 11, 100)
        .unwrap()
        .unwrap();
    publish_success(&mut store, &published.lease(), "published", 20);

    assert!(store.request_run_cancellation(&run.run_id, 40).unwrap());
    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("published").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert_eq!(
        store
            .get_attempt(&published.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Cancelled
    );
    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("pending").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("active").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Running
    );
    store.finalize_task_cancelled(&active.lease(), 41).unwrap();
    assert_eq!(
        store.get_run(&run.run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
}

#[test]
fn fail_run_terminalizes_all_sibling_work_and_preserves_cost() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![
                task("a-failing", &[]),
                task("b-running", &[]),
                task("c-pending", &["a-failing"]),
            ]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let failing = store
        .claim_ready_task(&run.run_id, "failing-worker", 10, 100)
        .unwrap()
        .unwrap();
    let sibling = store
        .claim_ready_task(&run.run_id, "sibling-worker", 11, 100)
        .unwrap()
        .unwrap();
    store
        .reconcile_usage(&sibling.lease(), 0, 17, 23, 12)
        .unwrap();
    let sibling_operation = store
        .prepare_operation(&sibling.lease(), OperationKind::StartThread, 0, 13)
        .unwrap();
    store
        .mark_dispatching_thread(&sibling.lease(), &sibling_operation.operation_id, 14)
        .unwrap();
    let failing_operation = store
        .prepare_operation(&failing.lease(), OperationKind::StartThread, 0, 15)
        .unwrap();
    store
        .mark_dispatching_thread(&failing.lease(), &failing_operation.operation_id, 16)
        .unwrap();
    store
        .mark_operation_failed(
            &failing.lease(),
            &failing_operation.operation_id,
            false,
            "semantic",
            17,
        )
        .unwrap();

    assert_eq!(
        store.get_run(&run.run_id).unwrap().unwrap().state,
        RunState::Failed
    );
    for task_id in ["a-failing", "b-running", "c-pending"] {
        assert_eq!(
            store
                .get_task(&run.run_id, &TaskId::from_str(task_id).unwrap())
                .unwrap()
                .unwrap()
                .state,
            TaskState::Failed
        );
    }
    let sibling_attempt = store.get_attempt(&sibling.attempt_id).unwrap().unwrap();
    assert_eq!(sibling_attempt.state, AttemptState::Failed);
    assert_eq!(sibling_attempt.lease_owner, None);
    assert_eq!(sibling_attempt.observed_tokens, 17);
    assert_eq!(
        store.reservation_usage(&sibling.attempt_id).unwrap(),
        Some((17, 23))
    );
    assert_eq!(
        store
            .get_operation(&sibling_operation.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Failed
    );
    assert!(store.nonterminal_attempts(&run.run_id).unwrap().is_empty());
    assert!(store.ready_tasks(&run.run_id).unwrap().is_empty());
    assert!(store
        .claim_ready_task(&run.run_id, "late-worker", 18, 100)
        .unwrap()
        .is_none());
    assert!(collect_events(&mut store, &run.run_id, 2)
        .iter()
        .any(|event| event.event_type == "run_failed_terminalized"));
}

#[test]
fn cancellation_interrupts_reconciling_publish_and_cancels_outstanding_operation() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    let (thread_operation, turn_operation) = drive_to_turn_started(&mut store, &lease, 10);
    store.reconcile_usage(&lease, 0, 19, 31, 20).unwrap();
    let publish = store
        .prepare_operation(&lease, OperationKind::PublishResult, 0, 21)
        .unwrap();
    store
        .mark_operation_dispatching(&lease, &publish.operation_id, 22)
        .unwrap();
    assert_eq!(
        store
            .get_attempt(lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Reconciling
    );

    assert!(store.request_run_cancellation(&run_id, 23).unwrap());
    let interrupt = prepare_cancellation_interrupt(&mut store, &lease, 0, 24);
    store
        .mark_operation_dispatching(&lease, &interrupt.operation_id, 25)
        .unwrap();
    store
        .complete_operation(&lease, &interrupt.operation_id, None, 26)
        .unwrap();
    store.finalize_task_cancelled(&lease, 27).unwrap();

    let attempt = store.get_attempt(lease.attempt_id()).unwrap().unwrap();
    assert_eq!(attempt.state, AttemptState::Cancelled);
    assert_eq!(
        attempt
            .thread_id
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("thread-lineage")
    );
    assert_eq!(
        attempt
            .latest_turn_id
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("turn-0")
    );
    assert_eq!(attempt.observed_tokens, 19);
    assert_eq!(
        store.reservation_usage(lease.attempt_id()).unwrap(),
        Some((19, 31))
    );
    assert_eq!(
        store
            .get_operation(&publish.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Cancelled
    );
    assert_eq!(
        store
            .get_operation(&interrupt.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Completed
    );
    assert_eq!(
        store
            .get_operation(&thread_operation)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Completed
    );
    assert_eq!(
        store.get_operation(&turn_operation).unwrap().unwrap().state,
        OperationState::Completed
    );
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
}

#[test]
fn cancellation_absorbs_published_attempt_and_all_nonterminal_operations() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    publish_success(&mut store, &lease, "alpha", 10);

    let prepared_evaluate = store
        .prepare_operation(&lease, OperationKind::Evaluate, 0, 30)
        .unwrap();
    let dispatched_evaluate = store
        .prepare_operation(&lease, OperationKind::Evaluate, 1, 31)
        .unwrap();
    store
        .mark_operation_dispatching(&lease, &dispatched_evaluate.operation_id, 32)
        .unwrap();

    assert!(store.request_run_cancellation(&run_id, 33).unwrap());

    assert_eq!(
        store
            .get_attempt(lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Cancelled
    );
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    for operation_id in [
        prepared_evaluate.operation_id,
        dispatched_evaluate.operation_id,
    ] {
        assert_eq!(
            store.get_operation(&operation_id).unwrap().unwrap().state,
            OperationState::Cancelled
        );
    }
}

#[test]
fn cancellation_absorbs_every_nonterminal_published_task_attempt() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, published_lease) = claim_single_task(&mut store, "alpha");
    let result_ref = publish_success(&mut store, &published_lease, "alpha", 10);
    let published_before = store
        .get_attempt(published_lease.attempt_id())
        .unwrap()
        .unwrap();
    let prepared = insert_competing_attempt(&path, &run_id, "alpha", 1, 17);
    let (dispatching, dispatch_operation) =
        insert_dispatching_competing_attempt(&path, &run_id, "alpha", 2);

    assert!(store.request_run_cancellation(&run_id, 40).unwrap());

    for attempt_id in [
        published_lease.attempt_id().clone(),
        prepared.clone(),
        dispatching.clone(),
    ] {
        let attempt = store.get_attempt(&attempt_id).unwrap().unwrap();
        assert_eq!(attempt.state, AttemptState::Cancelled);
        assert_eq!(attempt.lease_owner, None);
        assert_eq!(attempt.lease_expires_at, None);
    }
    let published_after = store
        .get_attempt(published_lease.attempt_id())
        .unwrap()
        .unwrap();
    assert_eq!(
        published_after.result_sha256,
        published_before.result_sha256
    );
    assert_eq!(
        published_after.result_status,
        published_before.result_status
    );
    assert_eq!(
        published_after.result_token_usage,
        published_before.result_token_usage
    );
    assert_eq!(published_after.thread_id, published_before.thread_id);
    assert_eq!(
        published_after.latest_turn_id,
        published_before.latest_turn_id
    );
    assert_eq!(store.reservation_usage(&prepared).unwrap(), Some((17, 77)));
    assert_eq!(
        store.reservation_usage(&dispatching).unwrap(),
        Some((29, 41))
    );
    let dispatching_after = store.get_attempt(&dispatching).unwrap().unwrap();
    assert_eq!(
        dispatching_after
            .thread_id
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("dispatch-thread")
    );
    let operation = store.get_operation(&dispatch_operation).unwrap().unwrap();
    assert_eq!(operation.state, OperationState::Cancelled);
    assert_eq!(
        operation.operation_marker.as_deref(),
        Some("dispatch-marker")
    );
    assert_eq!(
        store
            .get_task(&run_id, &TaskId::from_str("alpha").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
    assert!(store.nonterminal_attempts(&run_id).unwrap().is_empty());
    assert_eq!(result_ref.sha256, published_after.result_sha256.unwrap());
}

fn collect_events(
    store: &mut StateStore,
    run_id: &RunId,
    limit: usize,
) -> Vec<harp_state::EventRecord> {
    let mut events = Vec::new();
    let mut after = None;
    loop {
        let page = store.events_page(run_id, after, limit).unwrap();
        events.extend(page.events);
        let Some(next) = page.next_after_sequence else {
            break;
        };
        after = Some(next);
    }
    events
}

#[test]
fn cancellation_reclaims_expired_running_attempts_without_reactivating_work() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "with-turn");
    drive_to_turn_started(&mut store, &lease, 10);
    assert!(store.request_run_cancellation(&run_id, 20).unwrap());
    set_test_lease_now(&path, lease.expires_at());
    assert_eq!(store.expire_leases(&run_id, lease.expires_at()).unwrap(), 1);
    let reclaimed = store
        .reclaim_attempt(
            lease.attempt_id(),
            "cancel-owner",
            lease.expires_at(),
            lease.expires_at() + 1,
            lease.expires_at() + 50,
        )
        .unwrap();
    for kind in [
        OperationKind::StartThread,
        OperationKind::StartTurn,
        OperationKind::PublishResult,
        OperationKind::Evaluate,
    ] {
        assert!(matches!(
            store.prepare_operation(&reclaimed, kind, 99, lease.expires_at() + 2),
            Err(StateError::Conflict { .. })
        ));
    }
    let interrupt =
        prepare_cancellation_interrupt(&mut store, &reclaimed, 0, lease.expires_at() + 2);
    store
        .mark_operation_dispatching(&reclaimed, &interrupt.operation_id, lease.expires_at() + 3)
        .unwrap();
    store
        .complete_operation(
            &reclaimed,
            &interrupt.operation_id,
            None,
            lease.expires_at() + 4,
        )
        .unwrap();
    store
        .finalize_task_cancelled(&reclaimed, lease.expires_at() + 5)
        .unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );

    set_test_lease_now(&path, 0);
    let (run_id, lease) = claim_single_task(&mut store, "without-turn");
    assert!(store.request_run_cancellation(&run_id, 200).unwrap());
    set_test_lease_now(&path, lease.expires_at());
    assert_eq!(store.expire_leases(&run_id, lease.expires_at()).unwrap(), 1);
    let reclaimed = store
        .reclaim_attempt(
            lease.attempt_id(),
            "cancel-owner-2",
            lease.expires_at(),
            lease.expires_at() + 1,
            lease.expires_at() + 50,
        )
        .unwrap();
    store
        .finalize_task_cancelled(&reclaimed, lease.expires_at() + 2)
        .unwrap();
    assert_eq!(
        store.get_run(&run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
}

#[test]
fn cancelled_result_requires_requested_cancellation_and_converges_multi_task_run() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let run = store
        .create_run(
            &graph(vec![task("cancelled", &[]), task("sibling", &[])]),
            &serde_json::json!({}),
            &run_budget(),
            1,
        )
        .unwrap();
    let cancelled = store
        .claim_ready_task(&run.run_id, "cancel-worker", 10, 100)
        .unwrap()
        .unwrap();
    let sibling = store
        .claim_ready_task(&run.run_id, "sibling-worker", 11, 100)
        .unwrap()
        .unwrap();
    drive_to_turn_started(&mut store, &cancelled.lease(), 20);
    let publish = prepare_completed_operation(
        &mut store,
        &cancelled.lease(),
        OperationKind::PublishResult,
        0,
        29,
    );
    let (mut envelope, _) = result("cancelled");
    envelope.status = ResultStatus::Cancelled;
    envelope.answer_ref = None;
    envelope.failure_class = None;
    envelope.token_usage = 0;
    let bytes = serde_json::to_vec(&envelope).unwrap();
    let result_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&bytes)),
        "application/vnd.harp.result+json",
        bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &envelope, &result_ref, 32);
    assert!(matches!(
        store.record_result_published(&cancelled.lease(), &publish, &envelope, &result_ref, 33),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(
        store
            .get_attempt(&cancelled.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Reconciling
    );
    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("cancelled").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Running
    );

    assert!(store.request_run_cancellation(&run.run_id, 34).unwrap());
    store
        .record_result_published(&cancelled.lease(), &publish, &envelope, &result_ref, 35)
        .unwrap();
    assert_eq!(
        store
            .get_attempt(&cancelled.attempt_id)
            .unwrap()
            .unwrap()
            .state,
        AttemptState::Cancelled
    );
    assert_eq!(
        store
            .get_task(&run.run_id, &TaskId::from_str("cancelled").unwrap())
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    store.finalize_task_cancelled(&sibling.lease(), 36).unwrap();
    assert_eq!(
        store.get_run(&run.run_id).unwrap().unwrap().state,
        RunState::Cancelled
    );
}

#[test]
fn turn_markers_are_derived_unique_and_queryable() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (_run_id, lease) = claim_single_task(&mut store, "alpha");
    let thread = store
        .prepare_operation(&lease, OperationKind::StartThread, 0, 10)
        .unwrap();
    store
        .mark_dispatching_thread(&lease, &thread.operation_id, 11)
        .unwrap();
    store
        .record_thread_started(
            &lease,
            &thread.operation_id,
            &ThreadId::from_str("thread-marker").unwrap(),
            12,
        )
        .unwrap();
    let first = prepare_test_turn_operation(&mut store, &lease, OperationKind::StartTurn, 0, 13);
    let first_marker = store
        .mark_dispatching_turn(&lease, &first.operation_id, 14)
        .unwrap();
    assert_eq!(
        first_marker,
        format!("harp-operation:{}", first.operation_id)
    );
    assert_eq!(
        store
            .mark_dispatching_turn(&lease, &first.operation_id, 15)
            .unwrap(),
        first_marker
    );
    assert_eq!(
        store
            .get_operation(&first.operation_id)
            .unwrap()
            .unwrap()
            .operation_marker
            .as_deref(),
        Some(first_marker.as_str())
    );
    assert_eq!(
        store
            .get_operation_by_marker(&first_marker)
            .unwrap()
            .unwrap()
            .operation_id,
        first.operation_id
    );
    store
        .record_turn_started(
            &lease,
            &first.operation_id,
            &TurnId::from_str("turn-marker-0").unwrap(),
            16,
        )
        .unwrap();
    store.increment_continuation(&lease, 0, 2, 17).unwrap();
    let second =
        prepare_test_turn_operation(&mut store, &lease, OperationKind::ContinueTurn, 0, 18);
    let second_marker = store
        .mark_dispatching_turn(&lease, &second.operation_id, 19)
        .unwrap();
    assert_ne!(first_marker, second_marker);

    let connection = rusqlite::Connection::open(&path).unwrap();
    assert!(connection
        .execute(
            "UPDATE operations SET operation_marker = ?2 WHERE operation_id = ?1",
            rusqlite::params![second.operation_id.to_string(), first_marker],
        )
        .is_err());
}

#[test]
fn acceptance_waits_for_terminal_operations_and_latest_completed_evaluation() {
    let directory = private_directory();
    let mut store = open_state(&state_path(&directory)).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    publish_success(&mut store, &lease, "alpha", 10);
    let evaluate = store
        .prepare_operation(&lease, OperationKind::Evaluate, 0, 30)
        .unwrap();
    store
        .mark_operation_dispatching(&lease, &evaluate.operation_id, 31)
        .unwrap();
    assert!(matches!(
        store.accept_result(
            &run_id,
            &TaskId::from_str("alpha").unwrap(),
            lease.attempt_id(),
            32
        ),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(
        store
            .get_attempt(lease.attempt_id())
            .unwrap()
            .unwrap()
            .state,
        AttemptState::ResultPublished
    );
    store
        .complete_operation(&lease, &evaluate.operation_id, None, 33)
        .unwrap();
    assert_eq!(
        store
            .accept_result(
                &run_id,
                &TaskId::from_str("alpha").unwrap(),
                lease.attempt_id(),
                34
            )
            .unwrap(),
        AcceptResult::Accepted
    );

    let (run_id, lease) = claim_single_task(&mut store, "beta");
    publish_success(&mut store, &lease, "beta", 40);
    let failed = store
        .prepare_operation(&lease, OperationKind::Evaluate, 0, 60)
        .unwrap();
    store
        .mark_operation_dispatching(&lease, &failed.operation_id, 61)
        .unwrap();
    store
        .mark_operation_failed(&lease, &failed.operation_id, true, "eval", 62)
        .unwrap();
    assert!(matches!(
        store.accept_result(
            &run_id,
            &TaskId::from_str("beta").unwrap(),
            lease.attempt_id(),
            63
        ),
        Err(StateError::Conflict { .. })
    ));
    let completed = store
        .prepare_operation(&lease, OperationKind::Evaluate, 1, 64)
        .unwrap();
    store
        .mark_operation_dispatching(&lease, &completed.operation_id, 65)
        .unwrap();
    store
        .complete_operation(&lease, &completed.operation_id, None, 66)
        .unwrap();
    assert_eq!(
        store
            .accept_result(
                &run_id,
                &TaskId::from_str("beta").unwrap(),
                lease.attempt_id(),
                67
            )
            .unwrap(),
        AcceptResult::Accepted
    );

    let (run_id, lease) = claim_single_task(&mut store, "gamma");
    drive_to_turn_started(&mut store, &lease, 70);
    let blocking = prepare_budget_interrupt(&mut store, &lease, 0, 80);
    let published =
        prepare_completed_operation(&mut store, &lease, OperationKind::PublishResult, 0, 81);
    let (mut result, _) = result("gamma");
    result.token_usage = 0;
    let bytes = serde_json::to_vec(&result).unwrap();
    let result_ref = ArtifactRef::sha256(
        format!("{:x}", Sha256::digest(&bytes)),
        "application/vnd.harp.result+json",
        bytes.len() as u64,
    )
    .unwrap();
    register_result_artifacts(&mut store, &result, &result_ref, 84);
    store
        .record_result_published(&lease, &published, &result, &result_ref, 85)
        .unwrap();
    assert!(matches!(
        store.accept_result(
            &run_id,
            &TaskId::from_str("gamma").unwrap(),
            lease.attempt_id(),
            86
        ),
        Err(StateError::Conflict { .. })
    ));
    assert_eq!(
        store
            .get_operation(&blocking.operation_id)
            .unwrap()
            .unwrap()
            .state,
        OperationState::Prepared
    );
}

#[test]
fn schema_fingerprint_and_database_integrity_reject_structural_drift() {
    fn prepared_database() -> (TempDir, PathBuf) {
        let directory = private_directory();
        let path = state_path(&directory);
        let mut store = open_state(&path).unwrap();
        create_single_task_run(&mut store, "alpha");
        drop(store);
        (directory, path)
    }

    let (_directory, path) = prepared_database();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("DROP TABLE operations").unwrap();
    drop(connection);
    assert!(matches!(
        open_state(&path),
        Err(StateError::Integrity { .. })
    ));

    let (_directory, path) = prepared_database();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE rogue(id INTEGER PRIMARY KEY) STRICT;
             CREATE INDEX rogue_idx ON rogue(id);",
        )
        .unwrap();
    drop(connection);
    assert!(matches!(
        open_state(&path),
        Err(StateError::Integrity { .. })
    ));

    let (_directory, path) = prepared_database();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("DROP INDEX cli_activities_recovery_idx")
        .unwrap();
    drop(connection);
    assert!(matches!(
        open_state(&path),
        Err(StateError::Integrity { .. })
    ));

    let (_directory, path) = prepared_database();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "DROP INDEX cli_attempts_nonterminal_idx;
             CREATE INDEX cli_attempts_nonterminal_idx
             ON cli_attempts(attempt_id, state)
             WHERE state IN ('prepared', 'running', 'reconciling');",
        )
        .unwrap();
    drop(connection);
    assert!(matches!(
        open_state(&path),
        Err(StateError::Integrity { .. })
    ));

    let (_directory, path) = prepared_database();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("CREATE INDEX cli_activities_rogue_idx ON cli_activities(updated_at)")
        .unwrap();
    drop(connection);
    assert!(matches!(
        open_state(&path),
        Err(StateError::Integrity { .. })
    ));

    let (_directory, path) = prepared_database();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("ALTER TABLE artifacts ADD COLUMN rogue TEXT")
        .unwrap();
    drop(connection);
    assert!(matches!(
        open_state(&path),
        Err(StateError::Integrity { .. })
    ));

    let (_directory, path) = prepared_database();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", false)
        .unwrap();
    let run_id: String = connection
        .query_row("SELECT run_id FROM runs LIMIT 1", [], |row| row.get(0))
        .unwrap();
    connection
        .execute(
            "INSERT INTO task_dependencies(run_id, task_id, depends_on_task_id)
             VALUES (?1, 'alpha', 'missing')",
            [run_id],
        )
        .unwrap();
    drop(connection);
    assert!(matches!(
        open_state(&path),
        Err(StateError::Integrity { .. })
    ));

    let (_directory, path) = prepared_database();
    open_state(&path).unwrap();
}

#[test]
fn schema_integrity_rejects_unknown_views() {
    let directory = private_directory();
    let path = state_path(&directory);
    let store = open_state(&path).unwrap();
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("CREATE VIEW rogue_cli_view AS SELECT attempt_id FROM cli_attempts")
        .unwrap();
    drop(connection);

    assert!(matches!(
        open_state(&path),
        Err(StateError::Integrity { .. })
    ));
}

#[test]
fn event_pages_are_bounded_ordered_and_foreign_key_checked() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let (run_id, lease) = claim_single_task(&mut store, "alpha");
    store.renew_lease(&lease, 120, 20).unwrap();

    assert!(matches!(
        store.events_page(&run_id, None, 0),
        Err(StateError::InvalidInput { .. })
    ));
    assert!(matches!(
        store.events_page(&run_id, None, 1001),
        Err(StateError::InvalidInput { .. })
    ));
    let events = collect_events(&mut store, &run_id, 2);
    assert!(events.len() >= 3);
    assert!(events
        .windows(2)
        .all(|pair| pair[0].sequence < pair[1].sequence));

    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    assert!(connection
        .execute(
            "INSERT INTO events(run_id, task_id, event_type, payload_json, timestamp)
             VALUES (?1, 'missing', 'bad_fk', X'7b7d', 1)",
            [run_id.to_string()],
        )
        .is_err());
    assert!(connection
        .execute(
            "INSERT INTO events(run_id, attempt_id, event_type, payload_json, timestamp)
             VALUES (?1, ?2, 'bad_shape', X'7b7d', 1)",
            rusqlite::params![run_id.to_string(), lease.attempt_id().to_string()],
        )
        .is_err());
    let beta = TaskId::from_str("beta").unwrap();
    connection
        .execute(
            "INSERT INTO tasks(
                run_id, task_id, kind, state, max_transient_attempts,
                task_json, created_at, updated_at
             )
             SELECT run_id, ?2, kind, 'pending', max_transient_attempts,
                    task_json, created_at, updated_at
             FROM tasks
             WHERE run_id = ?1 AND task_id = 'alpha'",
            rusqlite::params![run_id.to_string(), beta.to_string()],
        )
        .unwrap();
    assert!(connection
        .execute(
            "INSERT INTO events(
                run_id, task_id, attempt_id, event_type, payload_json, timestamp
             ) VALUES (?1, ?2, ?3, 'bad_lineage', X'7b7d', 1)",
            rusqlite::params![
                run_id.to_string(),
                beta.to_string(),
                lease.attempt_id().to_string()
            ],
        )
        .is_err());
    connection
        .execute(
            "INSERT INTO events(
                run_id, task_id, attempt_id, event_type, payload_json, timestamp
             ) VALUES (?1, 'alpha', ?2, 'good_lineage', X'7b7d', 1)",
            rusqlite::params![run_id.to_string(), lease.attempt_id().to_string()],
        )
        .unwrap();
    let foreign_key_violations: i64 = connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(foreign_key_violations, 0);
    drop(connection);

    let connection = rusqlite::Connection::open(&path).unwrap();
    let payload = serde_json::to_vec(&"x".repeat(262_000)).unwrap();
    for _ in 0..33 {
        connection
            .execute(
                "INSERT INTO events(run_id, event_type, payload_json, timestamp)
                 VALUES (?1, 'large', ?2, 1)",
                rusqlite::params![run_id.to_string(), payload],
            )
            .unwrap();
    }
    drop(connection);
    assert!(matches!(
        store.events_page(&run_id, None, 1000),
        Err(StateError::LimitExceeded { .. })
    ));
}

#[test]
fn secure_directory_open_preserves_nonpolicy_io_sources() {
    let directory = private_directory();
    let path = directory.path().join("missing").join("state.sqlite");
    let error = open_state(&path).unwrap_err();
    match error {
        StateError::Io { source, .. } => {
            assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        }
        other => panic!("expected sourced I/O error, got {other:?}"),
    }
}

fn _path_is_used(path: &Path) -> bool {
    path.exists()
}
