#![cfg(unix)]
use harp_contracts::WorkflowV2;
use harp_state::{AdmissionRequest, StateStore};
use std::{fs, os::unix::fs::PermissionsExt};

fn request() -> AdmissionRequest {
    AdmissionRequest {
        namespace: "test".into(),
        submission_key: "campaign-1".into(),
        workflow: WorkflowV2::from_json(include_bytes!("fixtures/workflow-v2.json")).unwrap(),
        execution_authority: harp_contracts::ArtifactRef::sha256(
            "f".repeat(64),
            "application/json",
            2,
        )
        .unwrap(),
        schemas_sha256: "d".repeat(64),
        trial_limit: 2,
        trial_bindings: std::collections::BTreeMap::from([(
            "train".parse().unwrap(),
            "trial-1".into(),
        )]),
    }
}

#[test]
fn admission_survives_reopen_and_duplicate_content_conflicts() {
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    let mut state = StateStore::open(&path).unwrap();
    let first = state.admit_workflow(&request(), 1).unwrap();
    drop(state);
    let mut state = StateStore::open(&path).unwrap();
    assert_eq!(first, state.admit_workflow(&request(), 2).unwrap());
    assert_eq!(
        state.workflow_events(&first.run_id, 0, 100).unwrap().len(),
        1
    );
    let mut changed = request();
    changed.trial_limit += 1;
    assert!(state.admit_workflow(&changed, 3).is_err());
    assert_eq!(state.get_admission(&first.run_id).unwrap().unwrap(), first);
}

#[test]
fn simultaneous_duplicate_admission_has_one_identity_and_event() {
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    drop(StateStore::open(&path).unwrap());
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let path = path.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut state = StateStore::open(&path).unwrap();
                barrier.wait();
                state.admit_workflow(&request(), 1).unwrap()
            })
        })
        .collect();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results[0], results[1]);
    let state = StateStore::open(&path).unwrap();
    assert_eq!(
        state
            .workflow_events(&results[0].run_id, 0, 100)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn migrates_real_v5_schema_but_read_only_and_unknown_versions_do_not_write() {
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    drop(StateStore::open(&path).unwrap());
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute_batch("DROP TABLE workflow_events; DROP TABLE workflow_decisions; DROP TABLE workflow_environments; DROP TABLE workflow_jobs; DROP TABLE workflow_admissions; PRAGMA user_version=5;").unwrap();
    let old_fingerprint: String = c
        .query_row(
            "SELECT value FROM state_metadata WHERE key='schema_fingerprint'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    drop(c);
    let before = fs::read(&path).unwrap();
    assert!(StateStore::open_read_only(&path).is_err());
    assert_eq!(before, fs::read(&path).unwrap());
    let mut state = StateStore::open(&path).unwrap();
    state.admit_workflow(&request(), 1).unwrap();
    drop(state);
    let c = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        c.pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        6
    );
    assert_eq!(
        c.query_row::<String, _, _>(
            "SELECT value FROM state_metadata WHERE key='schema_fingerprint'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        old_fingerprint
    );
    c.pragma_update(None, "user_version", 7).unwrap();
    drop(c);
    let before = fs::read(&path).unwrap();
    assert!(StateStore::open(&path).is_err());
    assert!(StateStore::open_read_only(&path).is_err());
    assert_eq!(before, fs::read(&path).unwrap());
}

#[test]
fn workflow_schema_tampering_is_rejected() {
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    drop(StateStore::open(&path).unwrap());
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute_batch("DROP INDEX workflow_events_run_idx")
        .unwrap();
    drop(c);
    assert!(StateStore::open(&path).is_err());
    assert!(StateStore::open_read_only(&path).is_err());
}
