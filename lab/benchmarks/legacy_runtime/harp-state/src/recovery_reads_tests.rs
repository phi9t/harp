#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use harp_contracts::{
    ArtifactRef, Budget, NodeKind, RetryPolicy, TaskGraph, TaskId, TaskNode, TaskRole,
    WorkspaceMode,
};
use harp_state::{
    CliActivityKind, CliActivityState, CliAttemptState, RunBudget, StateError, StateStore,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const BASE_V1_SCHEMA: &str = include_str!("../migrations/0001_init.sql");
const BASE_V2_SCHEMA: &str = include_str!("../migrations/0002_execution_authority.sql");
const BASE_V3_SCHEMA: &str = include_str!("../migrations/0003_scratch_wall_lease.sql");
const BASE_V4_SCHEMA: &str = include_str!("../migrations/0004_failure_clock.sql");

#[derive(Debug)]
struct TestLeaseClock(AtomicI64);

impl harp_state::LeaseClock for TestLeaseClock {
    fn unix_seconds(&self) -> harp_state::StateResult<i64> {
        Ok(self.0.load(Ordering::SeqCst))
    }
}

fn open_state(path: &std::path::Path) -> harp_state::StateResult<StateStore> {
    StateStore::open_with_lease_clock(path, Arc::new(TestLeaseClock(AtomicI64::new(0))))
}

fn private_directory() -> TempDir {
    let directory = tempfile::Builder::new()
        .prefix("harp-state-recovery-")
        .tempdir_in("/private/tmp")
        .expect("temporary directory");
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .expect("private directory");
    directory
}

fn state_path(directory: &TempDir) -> PathBuf {
    directory.path().join("state.sqlite")
}

fn task(task_id: &str) -> TaskNode {
    TaskNode {
        task_id: TaskId::from_str(task_id).expect("task id"),
        kind: NodeKind::Analysis,
        role: TaskRole::Explore,
        instruction: format!("execute {task_id}"),
        dependencies: Vec::new(),
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

#[test]
fn reloads_typed_execution_inputs_and_task_nodes() {
    let directory = private_directory();
    let path = state_path(&directory);
    let graph = TaskGraph {
        schema_version: 1,
        nodes: vec![task("alpha")],
    };
    let provenance = serde_json::json!({
        "executionSpec": {
            "schemaVersion": 1,
            "receiptSha256": "a".repeat(64),
        }
    });
    let budget = RunBudget::new(1_000, 10_000, 3_600).expect("run budget");
    let run_id = {
        let mut store = open_state(&path).expect("open state");
        store
            .create_run(&graph, &provenance, &budget, 1)
            .expect("create run")
            .run_id
    };

    let mut reopened = open_state(&path).expect("reopen state");
    let execution = reopened
        .run_execution(&run_id)
        .expect("load execution")
        .expect("run exists");
    assert_eq!(execution.graph, graph);
    assert_eq!(execution.provenance, provenance);
    assert_eq!(execution.budget, budget);
    assert_eq!(execution.run.graph_sha256, execution.verified_graph_sha256);
    assert_eq!(
        reopened
            .task_node(&run_id, &TaskId::from_str("alpha").unwrap())
            .expect("load task")
            .expect("task exists"),
        task("alpha")
    );
}

#[test]
fn rejects_persisted_graph_bytes_that_do_not_match_the_digest() {
    let directory = private_directory();
    let path = state_path(&directory);
    let run_id = {
        let mut store = open_state(&path).expect("open state");
        store
            .create_run(
                &TaskGraph {
                    schema_version: 1,
                    nodes: vec![task("alpha")],
                },
                &serde_json::json!({}),
                &RunBudget::new(1_000, 10_000, 3_600).unwrap(),
                1,
            )
            .expect("create run")
            .run_id
    };
    drop(open_state(&path).expect("closeable state"));

    let connection = rusqlite::Connection::open(&path).expect("open raw database");
    let tampered = serde_json::to_vec(&TaskGraph {
        schema_version: 1,
        nodes: vec![task("tampered")],
    })
    .unwrap();
    connection
        .execute(
            "UPDATE runs SET graph_json = ?2 WHERE run_id = ?1",
            rusqlite::params![run_id.to_string(), tampered],
        )
        .expect("tamper graph");
    drop(connection);

    let mut reopened = open_state(&path).expect("reopen state");
    assert!(matches!(
        reopened.run_execution(&run_id),
        Err(StateError::Integrity { .. })
    ));
}

#[test]
fn enumerates_tasks_attempts_operations_and_accepted_results() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).expect("open state");
    let run = store
        .create_run(
            &TaskGraph {
                schema_version: 1,
                nodes: vec![task("alpha")],
            },
            &serde_json::json!({}),
            &RunBudget::new(1_000, 10_000, 3_600).unwrap(),
            1,
        )
        .expect("create run");
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 2, 100)
        .expect("claim")
        .expect("ready task");
    let operation = store
        .prepare_operation(&claim.lease(), harp_state::OperationKind::StartThread, 0, 3)
        .expect("prepare operation");

    assert_eq!(store.tasks(&run.run_id).expect("tasks").len(), 1);
    assert_eq!(store.attempts(&run.run_id).expect("attempts").len(), 1);
    assert_eq!(
        store
            .operations_for_attempt(&claim.attempt_id)
            .expect("operations"),
        vec![operation]
    );
    assert!(store
        .accepted_results(&run.run_id)
        .expect("accepted results")
        .is_empty());

    let artifact =
        ArtifactRef::sha256("a".repeat(64), "application/vnd.harp.result+json", 12).unwrap();
    store.register_artifact(&artifact, 4).unwrap();
    assert_eq!(store.artifact(&artifact.sha256).unwrap(), Some(artifact));
}

#[test]
fn recovery_reads_parse_typed_cli_state_and_canonical_session() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let run = store
        .create_run(
            &TaskGraph {
                schema_version: 1,
                nodes: vec![task("alpha")],
            },
            &serde_json::json!({}),
            &RunBudget::new(1_000, 10_000, 3_600).unwrap(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_task(&run.run_id, "worker", 2, 100)
        .unwrap()
        .unwrap();
    drop(store);

    let activity_id = "01900000-0000-7000-8000-000000000301";
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    connection
        .execute(
            "INSERT INTO cli_attempts(
                attempt_id, state, logical_session_id, external_session_id,
                continuation_count, created_at, updated_at
             ) VALUES (?1, 'running', 'logical-session', 'external-session', 0, 3, 3)",
            [claim.attempt_id.to_string()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO cli_activities(
                activity_id, attempt_id, kind, ordinal, state,
                logical_turn_id, activity_dir, invocation_sha256,
                process_record_sha256, created_at, updated_at
             ) VALUES (
                ?1, ?2, 'start_activity', 0, 'running',
                ?1, '/private/tmp/activity',
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                3, 3
             )",
            rusqlite::params![activity_id, claim.attempt_id.to_string()],
        )
        .unwrap();
    drop(connection);

    let mut reopened = open_state(&path).unwrap();
    let cli_attempt = reopened
        .get_cli_attempt(&claim.attempt_id)
        .unwrap()
        .unwrap();
    assert_eq!(cli_attempt.state, CliAttemptState::Running);
    assert_eq!(
        cli_attempt.external_session_id.unwrap().to_string(),
        "external-session"
    );
    let activity = reopened
        .get_cli_activity(&activity_id.parse().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(activity.kind, CliActivityKind::StartActivity);
    assert_eq!(activity.state, CliActivityState::Running);
    assert_eq!(activity.logical_turn_id.to_string(), activity_id);
    let recoverable = reopened.recoverable_activities(&run.run_id).unwrap();
    assert_eq!(recoverable.len(), 1);
    assert_eq!(recoverable[0].attempt_state, CliAttemptState::Running);
    assert_eq!(
        recoverable[0]
            .external_session_id
            .as_ref()
            .unwrap()
            .to_string(),
        "external-session"
    );
    assert_eq!(
        recoverable[0].process_record_sha256.as_deref(),
        Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );
    assert_eq!(recoverable[0].logical_turn_id.to_string(), activity_id);
    assert_eq!(recoverable[0].continuation_count, 0);
}

#[test]
fn recovery_reads_reject_invalid_persisted_cli_enums() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let run = store
        .create_run(
            &TaskGraph {
                schema_version: 1,
                nodes: vec![task("alpha")],
            },
            &serde_json::json!({}),
            &RunBudget::new(1_000, 10_000, 3_600).unwrap(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_cli_task(
            &run.run_id,
            "worker",
            2,
            100,
            "logical-session".parse().unwrap(),
        )
        .unwrap()
        .unwrap();

    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "ignore_check_constraints", true)
        .unwrap();
    connection
        .execute(
            "UPDATE cli_attempts SET state = 'invented_state' WHERE attempt_id = ?1",
            [claim.attempt_id.to_string()],
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        store.get_cli_attempt(&claim.attempt_id),
        Err(StateError::Integrity { .. })
    ));
}

#[test]
fn rejects_failed_spawned_activity_without_process_record() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let run = store
        .create_run(
            &TaskGraph {
                schema_version: 1,
                nodes: vec![task("alpha")],
            },
            &serde_json::json!({}),
            &RunBudget::new(1_000, 10_000, 3_600).unwrap(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_cli_task(
            &run.run_id,
            "worker",
            2,
            100,
            "logical-session".parse().unwrap(),
        )
        .unwrap()
        .unwrap();
    let activity_id = "01900000-0000-7000-8000-000000000311";
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    let insert = || {
        connection.execute(
            "INSERT INTO cli_activities(
                activity_id, attempt_id, kind, ordinal, state,
                logical_turn_id, activity_dir, invocation_sha256,
                terminal_failure_class, created_at, updated_at
             ) VALUES (
                ?1, ?2, 'start_activity', 0, 'failed',
                ?1, '/private/tmp/failed-spawned',
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                'resumable_cli_failure', 3, 3
             )",
            rusqlite::params![activity_id, claim.attempt_id.to_string()],
        )
    };
    assert!(insert().is_err());
    connection
        .pragma_update(None, "ignore_check_constraints", true)
        .unwrap();
    insert().unwrap();
    drop(connection);

    assert!(matches!(
        store.get_cli_activity(&activity_id.parse().unwrap()),
        Err(StateError::Integrity { .. })
    ));
}

#[test]
fn rejects_impossible_interrupt_activity_state_and_signal_stage_pairs() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut store = open_state(&path).unwrap();
    let run = store
        .create_run(
            &TaskGraph {
                schema_version: 1,
                nodes: vec![task("alpha")],
            },
            &serde_json::json!({}),
            &RunBudget::new(1_000, 10_000, 3_600).unwrap(),
            1,
        )
        .unwrap();
    let claim = store
        .claim_ready_cli_task(
            &run.run_id,
            "worker",
            2,
            100,
            "logical-session".parse().unwrap(),
        )
        .unwrap()
        .unwrap();
    let completed_at_prepared = "01900000-0000-7000-8000-000000000312";
    let cancelled_at_sigint_prepared = "01900000-0000-7000-8000-000000000313";
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    let insert_completed = || {
        connection.execute(
            "INSERT INTO cli_activities(
                activity_id, attempt_id, kind, ordinal, state,
                logical_turn_id, activity_dir, invocation_sha256,
                interrupt_purpose, target_process_record_sha256, signal_stage,
                created_at, updated_at
             ) VALUES (
                ?1, ?2, 'interrupt_activity', 0, 'completed',
                ?1, '/private/tmp/completed-at-prepared',
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                'cancellation',
                'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                'prepared', 3, 3
             )",
            rusqlite::params![completed_at_prepared, claim.attempt_id.to_string()],
        )
    };
    let insert_cancelled = || {
        connection.execute(
            "INSERT INTO cli_activities(
                activity_id, attempt_id, kind, ordinal, state,
                logical_turn_id, activity_dir, invocation_sha256,
                terminal_failure_class, interrupt_purpose,
                target_process_record_sha256, signal_stage,
                created_at, updated_at
             ) VALUES (
                ?1, ?2, 'interrupt_activity', 1, 'cancelled',
                ?1, '/private/tmp/cancelled-at-prepared',
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                'cancelled', 'cancellation',
                'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                'sigint_prepared', 3, 3
             )",
            rusqlite::params![cancelled_at_sigint_prepared, claim.attempt_id.to_string()],
        )
    };
    assert!(insert_completed().is_err());
    assert!(insert_cancelled().is_err());
    connection
        .pragma_update(None, "ignore_check_constraints", true)
        .unwrap();
    insert_completed().unwrap();
    insert_cancelled().unwrap();
    drop(connection);

    for activity_id in [completed_at_prepared, cancelled_at_sigint_prepared] {
        assert!(matches!(
            store.get_cli_activity(&activity_id.parse().unwrap()),
            Err(StateError::Integrity { .. })
        ));
    }
}

#[test]
fn opens_and_upgrades_an_actual_v1_database_without_losing_rows() {
    let directory = private_directory();
    let path = state_path(&directory);
    let graph = TaskGraph {
        schema_version: 1,
        nodes: vec![task("alpha")],
    };
    let graph_json = serde_json::to_vec(&graph).unwrap();
    let graph_sha256 = format!("{:x}", Sha256::digest(&graph_json));
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch(BASE_V1_SCHEMA).unwrap();
    let fingerprint = schema_fingerprint(&connection);
    connection
        .execute(
            "INSERT INTO state_metadata(key, value)
             VALUES ('schema_fingerprint', ?1)",
            [&fingerprint],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO runs(
                run_id, graph_sha256, graph_json, provenance_json,
                max_tokens, max_storage_bytes, max_wall_seconds,
                state, cancellation_requested, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000001', ?1, ?2, X'7b7d',
                1000, 10000, 3600, 'active', 0, 1, 1
             )",
            rusqlite::params![graph_sha256, graph_json],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO tasks(
                run_id, task_id, kind, state, max_transient_attempts,
                task_json, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000001',
                'alpha', 'analysis', 'ready', 1, ?1, 1, 1
             )",
            [serde_json::to_vec(&task("alpha")).unwrap()],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    drop(connection);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

    let mut upgraded = open_state(&path).expect("upgrade v1 database");

    assert_eq!(upgraded.schema_version().unwrap(), 6);
    assert_eq!(
        upgraded
            .tasks(
                &harp_contracts::RunId::from_str("01900000-0000-7000-8000-000000000001").unwrap()
            )
            .unwrap()
            .len(),
        1
    );
    let connection = rusqlite::Connection::open(&path).unwrap();
    let added_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('operations')
             WHERE name IN ('intent_json', 'intent_sha256', 'checkpoint_sha256')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(added_columns, 3);
}

#[test]
fn opens_and_upgrades_an_actual_v2_database_without_losing_rows() {
    let directory = private_directory();
    let path = state_path(&directory);
    let graph = TaskGraph {
        schema_version: 1,
        nodes: vec![task("alpha")],
    };
    let graph_json = serde_json::to_vec(&graph).unwrap();
    let graph_sha256 = format!("{:x}", Sha256::digest(&graph_json));
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch(BASE_V1_SCHEMA).unwrap();
    connection.execute_batch(BASE_V2_SCHEMA).unwrap();
    let fingerprint = schema_fingerprint(&connection);
    connection
        .execute(
            "INSERT INTO state_metadata(key, value)
             VALUES ('schema_fingerprint', ?1)",
            [&fingerprint],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO runs(
                run_id, graph_sha256, graph_json, provenance_json,
                max_tokens, max_storage_bytes, max_wall_seconds,
                state, cancellation_requested, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000002', ?1, ?2, X'7b7d',
                1000, 10000, 3600, 'active', 0, 1, 1
             )",
            rusqlite::params![graph_sha256, graph_json],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO tasks(
                run_id, task_id, kind, state, max_transient_attempts,
                task_json, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000002',
                'alpha', 'analysis', 'ready', 1, ?1, 1, 1
             )",
            [serde_json::to_vec(&task("alpha")).unwrap()],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

    let mut upgraded = open_state(&path).expect("upgrade v2 database");

    assert_eq!(upgraded.schema_version().unwrap(), 6);
    assert_eq!(
        upgraded
            .tasks(
                &harp_contracts::RunId::from_str("01900000-0000-7000-8000-000000000002",).unwrap(),
            )
            .unwrap()
            .len(),
        1
    );
    let connection = rusqlite::Connection::open(&path).unwrap();
    let added_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('attempts')
             WHERE name IN (
                'lease_capability_sha256',
                'scratch_path',
                'scratch_device',
                'scratch_inode',
                'wall_started_at',
                'observed_wall_seconds'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(added_columns, 6);
}

#[test]
fn opens_and_upgrades_an_actual_v3_database_without_losing_rows() {
    let directory = private_directory();
    let path = state_path(&directory);
    let graph = TaskGraph {
        schema_version: 1,
        nodes: vec![task("alpha")],
    };
    let graph_json = serde_json::to_vec(&graph).unwrap();
    let graph_sha256 = format!("{:x}", Sha256::digest(&graph_json));
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch(BASE_V1_SCHEMA).unwrap();
    connection.execute_batch(BASE_V2_SCHEMA).unwrap();
    connection.execute_batch(BASE_V3_SCHEMA).unwrap();
    let fingerprint = schema_fingerprint(&connection);
    connection
        .execute(
            "INSERT INTO state_metadata(key, value)
             VALUES ('schema_fingerprint', ?1)",
            [&fingerprint],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO runs(
                run_id, graph_sha256, graph_json, provenance_json,
                max_tokens, max_storage_bytes, max_wall_seconds,
                state, cancellation_requested, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000003', ?1, ?2, X'7b7d',
                1000, 10000, 3600, 'active', 0, 1, 1
             )",
            rusqlite::params![graph_sha256, graph_json],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO tasks(
                run_id, task_id, kind, state, max_transient_attempts,
                task_json, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000003',
                'alpha', 'analysis', 'ready', 1, ?1, 1, 1
             )",
            [serde_json::to_vec(&task("alpha")).unwrap()],
        )
        .unwrap();
    let capability = "migration-capability";
    connection
        .execute(
            "INSERT INTO attempts(
                attempt_id, run_id, task_id, ordinal, state,
                lease_owner, lease_expires_at, last_lease_expires_at,
                lease_capability_sha256, wall_started_at, observed_wall_seconds,
                continuation_count, observed_tokens, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000103',
                '01900000-0000-7000-8000-000000000003',
                'alpha', 0, 'prepared', 'worker', 200, 200,
                ?1, 100, 7, 0, 0, 1, 1
             )",
            [format!("{:x}", Sha256::digest(capability.as_bytes()))],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO budget_reservations(
                attempt_id, reserved_tokens, reserved_storage_bytes,
                reserved_wall_seconds, observed_tokens, observed_storage_bytes,
                observed_wall_seconds, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000103',
                100, 1024, 60, 0, 0, 7, 1, 1
             )",
            [],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO operations(
                operation_id, attempt_id, kind, ordinal, state, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000203',
                '01900000-0000-7000-8000-000000000103',
                'start_thread', 0, 'prepared', 1, 1
             )",
            [],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 3).unwrap();
    drop(connection);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

    let mut upgraded = open_state(&path).expect("upgrade v3 database");

    assert_eq!(upgraded.schema_version().unwrap(), 6);
    assert_eq!(
        upgraded
            .tasks(
                &harp_contracts::RunId::from_str("01900000-0000-7000-8000-000000000003").unwrap()
            )
            .unwrap()
            .len(),
        1
    );
    let connection = rusqlite::Connection::open(&path).unwrap();
    let added_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('attempts')
             WHERE name IN ('semantic_failure_class', 'wall_last_observed_at')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(added_columns, 2);
    let backfilled: Option<i64> = connection
        .query_row(
            "SELECT wall_last_observed_at FROM attempts
             WHERE attempt_id = '01900000-0000-7000-8000-000000000103'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(backfilled, Some(100));
    drop(connection);
    let lease = upgraded.test_only_lease_with_capability(
        harp_contracts::AttemptId::from_str("01900000-0000-7000-8000-000000000103").unwrap(),
        "worker",
        200,
        capability,
    );
    assert!(matches!(
        upgraded.reconcile_wall_usage(&lease, 7, 7, 99, 2),
        Err(StateError::Clock { .. })
    ));
}

#[test]
fn populated_v4_upgrade_preserves_every_base_object_and_row() {
    let directory = private_directory();
    let path = state_path(&directory);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch(BASE_V1_SCHEMA).unwrap();
    connection.execute_batch(BASE_V2_SCHEMA).unwrap();
    connection.execute_batch(BASE_V3_SCHEMA).unwrap();
    connection.execute_batch(BASE_V4_SCHEMA).unwrap();
    let base_fingerprint = schema_fingerprint(&connection);
    connection
        .execute_batch(
            "BEGIN IMMEDIATE;
             INSERT INTO state_metadata(key, value)
             VALUES ('schema_fingerprint', 'placeholder');
             INSERT INTO runs(
                run_id, graph_sha256, graph_json, provenance_json,
                max_tokens, max_storage_bytes, max_wall_seconds,
                state, cancellation_requested, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000004',
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                X'7b22736368656d6156657273696f6e223a312c226e6f646573223a5b5d7d',
                X'7b7d', 1000, 10000, 3600, 'active', 0, 1, 1
             );
             INSERT INTO tasks(
                run_id, task_id, kind, state, max_transient_attempts,
                task_json, created_at, updated_at
             ) VALUES
             (
                '01900000-0000-7000-8000-000000000004',
                'alpha', 'analysis', 'running', 1, X'7b7d', 1, 1
             ),
             (
                '01900000-0000-7000-8000-000000000004',
                'beta', 'analysis', 'pending', 1, X'7b7d', 1, 1
             );
             INSERT INTO task_dependencies(run_id, task_id, depends_on_task_id)
             VALUES (
                '01900000-0000-7000-8000-000000000004', 'beta', 'alpha'
             );
             INSERT INTO attempts(
                attempt_id, run_id, task_id, ordinal, state,
                lease_owner, lease_expires_at, last_lease_expires_at,
                lease_capability_sha256, continuation_count,
                observed_tokens, latest_turn_observed_tokens,
                scratch_path, scratch_device, scratch_inode,
                wall_started_at, wall_last_observed_at, observed_wall_seconds,
                semantic_failure_class, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000104',
                '01900000-0000-7000-8000-000000000004',
                'alpha', 0, 'prepared', 'worker', 200, 200,
                'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                0, 7, 3, '/private/tmp/harp-attempt', 1, 2,
                10, 17, 7, NULL, 1, 17
             );
             INSERT INTO operations(
                operation_id, attempt_id, kind, ordinal, state,
                intent_json, intent_sha256, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000204',
                '01900000-0000-7000-8000-000000000104',
                'start_thread', 0, 'prepared', NULL, NULL, 1, 1
             );
             INSERT INTO budget_reservations(
                attempt_id, reserved_tokens, reserved_storage_bytes,
                reserved_wall_seconds, observed_tokens, observed_storage_bytes,
                observed_wall_seconds, created_at, updated_at
             ) VALUES (
                '01900000-0000-7000-8000-000000000104',
                100, 1024, 60, 7, 11, 7, 1, 17
             );
             INSERT INTO artifacts(sha256, media_type, size_bytes, metadata_json, created_at)
             VALUES (
                'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
                'application/json', 2, X'7b7d', 1
             );
             INSERT INTO events(
                run_id, task_id, attempt_id, event_type, payload_json, timestamp
             ) VALUES (
                '01900000-0000-7000-8000-000000000004',
                'alpha',
                '01900000-0000-7000-8000-000000000104',
                'fixture_created', X'7b7d', 1
             );
             COMMIT;",
        )
        .unwrap();
    connection
        .execute(
            "UPDATE state_metadata SET value = ?1 WHERE key = 'schema_fingerprint'",
            [&base_fingerprint],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 4).unwrap();

    let base_schema_before = schema_objects(&connection);
    let base_rows_before = table_row_digests(&connection, base_schema_before.keys());
    let stored_before: String = connection
        .query_row(
            "SELECT value FROM state_metadata WHERE key = 'schema_fingerprint'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    drop(connection);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

    let upgraded = open_state(&path).expect("upgrade populated v4 database");
    assert_eq!(upgraded.schema_version().unwrap(), 6);
    drop(upgraded);

    let connection = rusqlite::Connection::open(&path).unwrap();
    let schema_after = schema_objects(&connection);
    for (name, sql) in &base_schema_before {
        assert_eq!(schema_after.get(name), Some(sql), "base object {name}");
    }
    assert_eq!(
        table_row_digests(&connection, base_schema_before.keys()),
        base_rows_before
    );
    let stored_after: String = connection
        .query_row(
            "SELECT value FROM state_metadata WHERE key = 'schema_fingerprint'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored_after, stored_before);
    assert_eq!(stored_after, base_fingerprint);
    assert!(schema_after.contains_key("table:cli_attempts"));
    assert!(schema_after.contains_key("table:cli_activities"));
}

fn schema_objects(connection: &rusqlite::Connection) -> BTreeMap<String, String> {
    let mut statement = connection
        .prepare(
            "SELECT type, name, COALESCE(sql, '')
             FROM sqlite_master
             WHERE type IN ('table', 'index', 'trigger')
               AND name NOT LIKE 'sqlite_%'
             ORDER BY type, name",
        )
        .unwrap();
    statement
        .query_map([], |row| {
            Ok((
                format!("{}:{}", row.get::<_, String>(0)?, row.get::<_, String>(1)?),
                row.get::<_, String>(2)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn table_row_digests<'a>(
    connection: &rusqlite::Connection,
    objects: impl Iterator<Item = &'a String>,
) -> BTreeMap<String, String> {
    objects
        .filter_map(|object| object.strip_prefix("table:"))
        .map(|table| {
            let escaped = table.replace('"', "\"\"");
            let mut statement = connection
                .prepare(&format!("SELECT * FROM \"{escaped}\" ORDER BY rowid"))
                .unwrap();
            let column_count = statement.column_count();
            let mut rows = statement.query([]).unwrap();
            let mut digest = Sha256::new();
            let mut row_count = 0u64;
            while let Some(row) = rows.next().unwrap() {
                digest.update(row_count.to_le_bytes());
                for column in 0..column_count {
                    use rusqlite::types::ValueRef;
                    match row.get_ref(column).unwrap() {
                        ValueRef::Null => digest.update([0]),
                        ValueRef::Integer(value) => {
                            digest.update([1]);
                            digest.update(value.to_le_bytes());
                        }
                        ValueRef::Real(value) => {
                            digest.update([2]);
                            digest.update(value.to_bits().to_le_bytes());
                        }
                        ValueRef::Text(value) => {
                            digest.update([3]);
                            digest.update((value.len() as u64).to_le_bytes());
                            digest.update(value);
                        }
                        ValueRef::Blob(value) => {
                            digest.update([4]);
                            digest.update((value.len() as u64).to_le_bytes());
                            digest.update(value);
                        }
                    }
                }
                row_count += 1;
            }
            (table.to_owned(), format!("{:x}", digest.finalize()))
        })
        .collect()
}

fn schema_fingerprint(connection: &rusqlite::Connection) -> String {
    let mut statement = connection
        .prepare(
            "SELECT type, name, COALESCE(sql, '')
             FROM sqlite_master
             WHERE type IN ('table', 'index', 'trigger')
               AND name NOT LIKE 'sqlite_%'
             ORDER BY type, name",
        )
        .unwrap();
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .unwrap();
    let mut digest = Sha256::new();
    for row in rows {
        let (object_type, name, sql) = row.unwrap();
        for value in [object_type.as_bytes(), name.as_bytes(), sql.as_bytes()] {
            digest.update((value.len() as u64).to_le_bytes());
            digest.update(value);
        }
    }
    format!("{:x}", digest.finalize())
}

#[test]
fn inspection_is_read_only_bounded_and_releases_snapshot_after_errors() {
    let directory = private_directory();
    let path = state_path(&directory);
    let mut writer = open_state(&path).unwrap();
    let run = writer
        .create_run(
            &TaskGraph {
                schema_version: 1,
                nodes: vec![task("alpha")],
            },
            &serde_json::json!({}),
            &RunBudget::new(1_000, 10_000, 3_600).unwrap(),
            1,
        )
        .unwrap();
    let claim = writer
        .claim_ready_task(&run.run_id, "worker", 2, 100)
        .unwrap()
        .unwrap();
    writer
        .fail_terminal_semantic(&claim.lease(), "result.invalid_schema", 3)
        .unwrap();
    drop(writer);
    let before = fs::read(&path).unwrap();
    let mut reader = StateStore::open_read_only(&path).unwrap();
    let snapshot = reader
        .inspect_run(&run.run_id, None, None, 1)
        .unwrap()
        .unwrap();
    assert_eq!(
        snapshot.attempts[0]
            .attempt
            .semantic_failure_class
            .as_deref(),
        Some("result.invalid_schema")
    );
    assert_eq!(snapshot.events.events.len(), 1);
    let cursor = snapshot.events.next_after_sequence.unwrap();
    let next = reader
        .inspect_run(&run.run_id, None, Some(cursor), 1000)
        .unwrap()
        .unwrap();
    assert!(next
        .events
        .events
        .iter()
        .all(|event| event.sequence > cursor as u64));
    assert!(next
        .events
        .events
        .iter()
        .any(|event| event.event_type == "terminal_semantic_failure"));
    assert!(reader.inspect_run(&run.run_id, None, None, 0).is_err());
    let alpha = TaskId::from_str("alpha").unwrap();
    let filtered = reader
        .inspect_run(&run.run_id, Some(&alpha), None, 1000)
        .unwrap()
        .unwrap();
    assert!(filtered
        .events
        .events
        .iter()
        .all(|event| event.task_id.as_ref() == Some(&alpha)));
    assert!(reader.rebuild_ready_tasks(&run.run_id, 4).is_err());
    drop(reader);
    assert_eq!(before, fs::read(&path).unwrap());
}

#[test]
fn read_only_open_rejects_missing_symlinked_and_old_state_without_mutation() {
    let directory = private_directory();
    let path = state_path(&directory);
    assert!(StateStore::open_read_only(&path).is_err());
    assert!(!path.exists());
    drop(open_state(&path).unwrap());
    let link = directory.path().join("link.sqlite");
    std::os::unix::fs::symlink(&path, &link).unwrap();
    assert!(StateStore::open_read_only(&link).is_err());
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 4).unwrap();
    drop(connection);
    let before = fs::read(&path).unwrap();
    assert!(StateStore::open_read_only(&path).is_err());
    assert_eq!(before, fs::read(&path).unwrap());
}
