#![cfg(unix)]
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::str::FromStr;
use std::sync::Arc;

use harp_contracts::{
    Budget, NodeKind, RetryPolicy, TaskGraph, TaskId, TaskNode, TaskRole, WorkspaceMode,
};
use harp_engine::{explain_run, InspectionAction};
use harp_state::{RunBudget, StateStore};

#[derive(Debug)]
struct Clock(i64);
impl harp_state::LeaseClock for Clock {
    fn unix_seconds(&self) -> harp_state::StateResult<i64> {
        Ok(self.0)
    }
}

fn task(id: &str, dependencies: &[&str]) -> TaskNode {
    TaskNode {
        task_id: TaskId::from_str(id).unwrap(),
        kind: NodeKind::Analysis,
        role: TaskRole::Explore,
        instruction: "implement".to_owned(),
        dependencies: dependencies
            .iter()
            .map(|id| TaskId::from_str(id).unwrap())
            .collect(),
        inputs: vec![],
        workspace_mode: WorkspaceMode::Scratch,
        model_policy: "default".to_owned(),
        permission_profile: "isolated".to_owned(),
        budget: Budget::new(100, 60, 1024),
        output_schema: "{}".to_owned(),
        retry_policy: RetryPolicy {
            max_transient_attempts: 1,
        },
    }
}

#[test]
fn explains_dependency_failures_with_persisted_evidence_and_no_replay_advice() {
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let mut state =
        StateStore::open_with_lease_clock(&dir.path().join("state.sqlite"), Arc::new(Clock(10)))
            .unwrap();
    let run = state
        .create_run(
            &TaskGraph {
                schema_version: 1,
                nodes: vec![task("edit", &[]), task("verify", &["edit"])],
            },
            &serde_json::json!({}),
            &RunBudget::new(1000, 10000, 3600).unwrap(),
            10,
        )
        .unwrap();
    let initial = explain_run(
        state
            .inspect_run(&run.run_id, None, None, 100)
            .unwrap()
            .unwrap(),
        None,
    );
    assert_eq!(initial.tasks[0].action, InspectionAction::Resume);
    assert_eq!(initial.tasks[1].reason, "task.dependencies_pending");
    let claim = state
        .claim_ready_task(&run.run_id, "worker", 10, 100)
        .unwrap()
        .unwrap();
    let verify = TaskId::from_str("verify").unwrap();
    let filtered = explain_run(
        state
            .inspect_run(&run.run_id, Some(&verify), None, 100)
            .unwrap()
            .unwrap(),
        Some(&verify),
    );
    assert_eq!(filtered.tasks.len(), 1);
    assert_eq!(filtered.tasks[0].reason, "recovery.live_lease");
    assert!(filtered.attempts.is_empty());
    state.set_lease_clock(Arc::new(Clock(200))).unwrap();
    let expired = explain_run(
        state
            .inspect_run(&run.run_id, None, None, 100)
            .unwrap()
            .unwrap(),
        None,
    );
    assert_eq!(expired.tasks[0].reason, "recovery.reconciliation_required");
    assert_eq!(expired.tasks[0].action, InspectionAction::Resume);
    assert!(expired.attempts[0].lease_expires_at.is_some());
    state.set_lease_clock(Arc::new(Clock(10))).unwrap();

    state
        .fail_terminal_semantic(&claim.lease(), "result.invalid_schema", 11)
        .unwrap();
    let report = explain_run(
        state
            .inspect_run(&run.run_id, None, None, 100)
            .unwrap()
            .unwrap(),
        None,
    );
    assert_eq!(
        report.attempts[0].semantic_failure_class.as_deref(),
        Some("result.invalid_schema")
    );
    assert!(report
        .tasks
        .iter()
        .all(|task| task.action == InspectionAction::ReviewFailure));
    assert_eq!(
        report.tasks[1].blocked_by,
        vec![TaskId::from_str("edit").unwrap()]
    );
    assert!(report
        .events
        .iter()
        .any(|event| event.event_type == "terminal_semantic_failure"));
    assert!(report.render_text().contains("result.invalid_schema"));
    assert!(report.render_text().contains("blocked by: edit"));
}
