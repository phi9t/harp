#![cfg(unix)]
use harp_contracts::WorkflowV2;
use harp_state::{AdmissionRequest, JobPreparation, JobTransition, ResourceVector, StateStore};
use std::{fs, os::unix::fs::PermissionsExt};

#[test]
fn ambiguous_submission_retains_exposure_and_competing_writer_is_fenced() {
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    let mut a = StateStore::open(&path).unwrap();
    let run = a
        .admit_workflow(
            &AdmissionRequest {
                namespace: "test".into(),
                submission_key: "jobs".into(),
                workflow: WorkflowV2::from_json(include_bytes!("fixtures/workflow-v2.json"))
                    .unwrap(),
                execution_authority: harp_contracts::ArtifactRef::sha256(
                    "f".repeat(64),
                    "application/json",
                    2,
                )
                .unwrap(),
                schemas_sha256: "d".repeat(64),
                trial_limit: 1,
                trial_bindings: std::collections::BTreeMap::from([(
                    "train".parse().unwrap(),
                    "trial-1".into(),
                )]),
            },
            1,
        )
        .unwrap();
    let guard = a.acquire_controller(&run.run_id, run.revision, 2).unwrap();
    let resources = ResourceVector {
        cpu_seconds: 60,
        wall_seconds: 60,
        storage_bytes: 1048576,
        trials: 1,
        ..Default::default()
    };
    let job = JobPreparation {
        job_id: "job-1".into(),
        task_id: "train".parse().unwrap(),
        trial_id: Some("trial-1".into()),
        resources,
    };
    let guard = a.prepare_job(&run.run_id, guard, &job, 3).unwrap();
    let guard = a
        .transition_job(&run.run_id, guard, "job-1", &JobTransition::Submit, 4)
        .unwrap();
    drop(a);
    let mut a = StateStore::open(&path).unwrap();
    let mut b = StateStore::open(&path).unwrap();
    let next = b
        .acquire_controller(&run.run_id, guard.revision, 5)
        .unwrap();
    assert!(a
        .transition_job(&run.run_id, guard, "job-1", &JobTransition::Submit, 6)
        .is_err());
    assert_eq!(
        a.workflow_accounting(&run.run_id).unwrap().reserved,
        resources
    );
    let second = JobPreparation {
        job_id: "job-2".into(),
        task_id: "train".parse().unwrap(),
        trial_id: Some("trial-2".into()),
        resources,
    };
    assert!(b.prepare_job(&run.run_id, next, &second, 7).is_err());
    assert_eq!(
        a.workflow_accounting(&run.run_id).unwrap().reserved,
        resources
    );
}

fn receipt() -> harp_contracts::ArtifactRef {
    harp_contracts::ArtifactRef::sha256("e".repeat(64), "application/json", 2).unwrap()
}
fn open_run(
    path: &std::path::Path,
) -> (
    StateStore,
    harp_state::AdmissionRecord,
    harp_state::RunGuard,
) {
    let mut s = StateStore::open(path).unwrap();
    let run = s
        .admit_workflow(
            &AdmissionRequest {
                namespace: "jobs".into(),
                submission_key: "case".into(),
                workflow: WorkflowV2::from_json(include_bytes!("fixtures/workflow-v2.json"))
                    .unwrap(),
                execution_authority: harp_contracts::ArtifactRef::sha256(
                    "f".repeat(64),
                    "application/json",
                    2,
                )
                .unwrap(),
                schemas_sha256: "d".repeat(64),
                trial_limit: 1,
                trial_bindings: std::collections::BTreeMap::from([(
                    "train".parse().unwrap(),
                    "trial-1".into(),
                )]),
            },
            1,
        )
        .unwrap();
    let guard = s.acquire_controller(&run.run_id, run.revision, 2).unwrap();
    (s, run, guard)
}
fn prepared(
    s: &mut StateStore,
    run: &harp_contracts::RunId,
    guard: harp_state::RunGuard,
) -> harp_state::RunGuard {
    s.prepare_job(
        run,
        guard,
        &JobPreparation {
            job_id: "first".into(),
            task_id: "train".parse().unwrap(),
            trial_id: Some("trial-1".into()),
            resources: ResourceVector {
                cpu_seconds: 60,
                wall_seconds: 60,
                storage_bytes: 1048576,
                trials: 1,
                ..Default::default()
            },
        },
        3,
    )
    .unwrap()
}

#[test]
fn transition_boundaries_reopen_and_connectivity_never_terminalizes_jobs() {
    use harp_state::{JobConnectivity, JobObservation, JobState, JobTermination};
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    let (mut s, run, guard) = open_run(&path);
    let mut guard = prepared(&mut s, &run.run_id, guard);
    assert!(s
        .get_job(&run.run_id, "first")
        .unwrap()
        .unwrap()
        .submission_key
        .ends_with("/train/1"));
    guard = s
        .transition_job(&run.run_id, guard, "first", &JobTransition::Submit, 4)
        .unwrap();
    guard = s
        .observe_job(
            &run.run_id,
            guard,
            "first",
            &JobObservation {
                connectivity: JobConnectivity::Unreachable,
                observed_at: 5,
                evidence: None,
            },
        )
        .unwrap();
    assert_eq!(
        s.get_job(&run.run_id, "first").unwrap().unwrap().state,
        JobState::SubmissionUnknown
    );
    assert!(s
        .transition_job(
            &run.run_id,
            guard,
            "first",
            &JobTransition::Settle { measured: None },
            6
        )
        .is_err());
    let key = s
        .get_job(&run.run_id, "first")
        .unwrap()
        .unwrap()
        .submission_key;
    let steps = [
        (
            JobTransition::ConfirmAbsent { receipt: receipt() },
            JobState::Prepared,
        ),
        (JobTransition::Submit, JobState::SubmissionUnknown),
        (
            JobTransition::Bind {
                backend_job_id: "backend-owned-1".into(),
                binding_receipt: receipt(),
            },
            JobState::Running,
        ),
        (
            JobTransition::Terminate {
                termination: JobTermination::Exit {
                    code: 1,
                    receipt: receipt(),
                },
                quiescence: harp_state::JobQuiescence::ProcessTree { receipt: receipt() },
            },
            JobState::Terminal,
        ),
        (JobTransition::BeginCollection, JobState::Collecting),
        (
            JobTransition::FinishCollection {
                manifest: receipt(),
            },
            JobState::Collected,
        ),
        (
            JobTransition::Settle { measured: None },
            JobState::Collected,
        ),
    ];
    for (step, state) in steps {
        guard = s
            .transition_job(&run.run_id, guard, "first", &step, 7)
            .unwrap();
        drop(s);
        s = StateStore::open(&path).unwrap();
        let job = s.get_job(&run.run_id, "first").unwrap().unwrap();
        assert_eq!(job.state, state);
        assert_eq!(job.submission_key, key);
    }
    let account = s.workflow_accounting(&run.run_id).unwrap();
    assert_eq!(account.reserved, ResourceVector::default());
    assert_eq!(account.settled.cpu_seconds, 60);
    assert_eq!(account.settled.trials, 1);
    assert!(
        s.get_job(&run.run_id, "first")
            .unwrap()
            .unwrap()
            .reservation
            .estimated
    );
}

#[test]
fn decisions_are_atomic_idempotent_and_changed_or_stale_content_conflicts() {
    use harp_state::JobDecision;
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    let (mut s, run, guard) = open_run(&path);
    let guard = prepared(&mut s, &run.run_id, guard);
    let decision = JobDecision {
        decision_id: "submit-first".into(),
        expected: guard,
        job_id: "first".into(),
        transition: JobTransition::Submit,
        reason: "admitted".into(),
        evidence: vec![],
    };
    let first = s.apply_job_decision(&run.run_id, &decision, 4).unwrap();
    drop(s);
    let mut s = StateStore::open(&path).unwrap();
    let count = s.workflow_events(&run.run_id, 0, 100).unwrap().len();
    assert_eq!(
        s.apply_job_decision(&run.run_id, &decision, 5).unwrap(),
        first
    );
    assert_eq!(s.workflow_events(&run.run_id, 0, 100).unwrap().len(), count);
    let mut changed = decision.clone();
    changed.reason = "changed".into();
    assert!(s.apply_job_decision(&run.run_id, &changed, 6).is_err());
    changed.decision_id = "new-decision".into();
    assert!(s.apply_job_decision(&run.run_id, &changed, 6).is_err());
    let guard = s
        .cancel_workflow(&run.run_id, first.resulting_guard, 7)
        .unwrap();
    let guard = s
        .transition_job(
            &run.run_id,
            guard,
            "first",
            &JobTransition::ConfirmAbsent { receipt: receipt() },
            8,
        )
        .unwrap();
    assert!(s
        .transition_job(&run.run_id, guard, "first", &JobTransition::Submit, 9)
        .is_err());
}

#[test]
fn concurrent_reservations_cannot_overdraw_trial_budget() {
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    let mut doc: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/workflow-v2.json")).unwrap();
    let mut other = doc["tasks"][0].clone();
    other["taskId"] = serde_json::json!("evaluate");
    doc["tasks"].as_array_mut().unwrap().push(other);
    for key in ["cpuSeconds", "wallSeconds", "storageBytes"] {
        doc["limits"][key] = serde_json::json!(doc["limits"][key].as_u64().unwrap() * 2);
    }
    let mut s = StateStore::open(&path).unwrap();
    let run = s
        .admit_workflow(
            &AdmissionRequest {
                namespace: "jobs".into(),
                submission_key: "compete".into(),
                workflow: WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap(),
                execution_authority: harp_contracts::ArtifactRef::sha256(
                    "f".repeat(64),
                    "application/json",
                    2,
                )
                .unwrap(),
                schemas_sha256: "d".repeat(64),
                trial_limit: 1,
                trial_bindings: std::collections::BTreeMap::from([
                    ("train".parse().unwrap(), "train".into()),
                    ("evaluate".parse().unwrap(), "evaluate".into()),
                ]),
            },
            1,
        )
        .unwrap();
    let guard = s.acquire_controller(&run.run_id, 0, 2).unwrap();
    drop(s);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = ["train", "evaluate"]
        .into_iter()
        .map(|task| {
            let path = path.clone();
            let run = run.run_id.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut s = StateStore::open(&path).unwrap();
                barrier.wait();
                s.prepare_job(
                    &run,
                    guard,
                    &JobPreparation {
                        job_id: task.into(),
                        task_id: task.parse().unwrap(),
                        trial_id: Some(task.into()),
                        resources: ResourceVector {
                            cpu_seconds: 60,
                            wall_seconds: 60,
                            storage_bytes: 1048576,
                            trials: 1,
                            ..Default::default()
                        },
                    },
                    3,
                )
                .is_ok()
            })
        })
        .collect();
    let outcomes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(outcomes.iter().filter(|success| **success).count(), 1);
    let mut s = StateStore::open(&path).unwrap();
    let jobs = s.workflow_jobs(&run.run_id).unwrap();
    let missing = if jobs[0].task_id.to_string() == "train" {
        "evaluate"
    } else {
        "train"
    };
    let guard = s.get_admission(&run.run_id).unwrap().unwrap().guard();
    assert!(s
        .prepare_job(
            &run.run_id,
            guard,
            &JobPreparation {
                job_id: missing.into(),
                task_id: missing.parse().unwrap(),
                trial_id: Some(missing.into()),
                resources: ResourceVector {
                    cpu_seconds: 60,
                    wall_seconds: 60,
                    storage_bytes: 1048576,
                    trials: 1,
                    ..Default::default()
                }
            },
            4
        )
        .is_err());
    assert_eq!(
        s.workflow_accounting(&run.run_id).unwrap().reserved.trials,
        1
    );
}

#[test]
fn terminal_evidence_requires_binding_and_matching_quiescence() {
    use harp_state::{JobQuiescence, JobTermination};
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    let (mut s, run, guard) = open_run(&path);
    let guard = prepared(&mut s, &run.run_id, guard);
    let guard = s
        .transition_job(&run.run_id, guard, "first", &JobTransition::Submit, 4)
        .unwrap();
    let exit = JobTransition::Terminate {
        termination: JobTermination::Exit {
            code: 0,
            receipt: receipt(),
        },
        quiescence: JobQuiescence::ProcessTree { receipt: receipt() },
    };
    assert!(s
        .transition_job(&run.run_id, guard, "first", &exit, 5)
        .is_err());
    let guard = s
        .transition_job(
            &run.run_id,
            guard,
            "first",
            &JobTransition::Bind {
                backend_job_id: "owned".into(),
                binding_receipt: receipt(),
            },
            6,
        )
        .unwrap();
    assert!(s
        .transition_job(
            &run.run_id,
            guard,
            "first",
            &JobTransition::Bind {
                backend_job_id: "replacement".into(),
                binding_receipt: receipt()
            },
            7
        )
        .is_err());
    assert!(s
        .transition_job(
            &run.run_id,
            guard,
            "first",
            &JobTransition::Terminate {
                termination: JobTermination::Absent { receipt: receipt() },
                quiescence: JobQuiescence::AuthoritativeAbsence { receipt: receipt() }
            },
            7
        )
        .is_err());
    assert!(s
        .transition_job(
            &run.run_id,
            guard,
            "first",
            &JobTransition::Terminate {
                termination: JobTermination::Exit {
                    code: 0,
                    receipt: receipt()
                },
                quiescence: JobQuiescence::AuthoritativeAbsence { receipt: receipt() }
            },
            7
        )
        .is_err());
    s.transition_job(&run.run_id, guard, "first", &exit, 8)
        .unwrap();
    drop(s);
    let s = StateStore::open(&path).unwrap();
    let job = s.get_job(&run.run_id, "first").unwrap().unwrap();
    assert_eq!(job.binding_receipt, Some(receipt()));
    assert!(matches!(
        job.quiescence,
        Some(JobQuiescence::ProcessTree { .. })
    ));
}

#[test]
fn environment_verification_pins_evidence_without_conflating_config_and_receipt() {
    use harp_state::EnvironmentState;
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    let (mut s, run, guard) = open_run(&path);
    let identity = run.request.workflow.tasks[0].environment_sha256.clone();
    assert!(s
        .transition_environment(
            &run.run_id,
            guard,
            &identity,
            EnvironmentState::Verified,
            Some(&receipt()),
            3
        )
        .is_err());
    let guard = s
        .transition_environment(
            &run.run_id,
            guard,
            &identity,
            EnvironmentState::Preparing,
            None,
            3,
        )
        .unwrap();
    assert!(s
        .transition_environment(
            &run.run_id,
            guard,
            &identity,
            EnvironmentState::Verified,
            None,
            4
        )
        .is_err());
    let guard = s
        .transition_environment(
            &run.run_id,
            guard,
            &identity,
            EnvironmentState::Unknown,
            None,
            4,
        )
        .unwrap();
    s.transition_environment(
        &run.run_id,
        guard,
        &identity,
        EnvironmentState::Verified,
        Some(&receipt()),
        5,
    )
    .unwrap();
    drop(s);
    let s = StateStore::open(&path).unwrap();
    let environments = s.workflow_environments(&run.run_id).unwrap();
    assert_eq!(environments.len(), 1);
    assert_eq!(environments[0].state, EnvironmentState::Verified);
    assert_eq!(environments[0].environment_sha256, identity);
    assert_eq!(environments[0].receipt, Some(receipt()));
}

#[test]
fn admitted_trial_identity_survives_retry_and_setup_does_not_consume_trial() {
    use harp_state::{JobQuiescence, JobTermination};
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("state.sqlite");
    let mut doc: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/workflow-v2.json")).unwrap();
    doc["tasks"][0]["maxRetries"] = serde_json::json!(1);
    let mut setup = doc["tasks"][0].clone();
    setup["taskId"] = serde_json::json!("setup");
    setup["maxRetries"] = serde_json::json!(0);
    doc["tasks"].as_array_mut().unwrap().push(setup);
    for key in ["cpuSeconds", "wallSeconds", "storageBytes"] {
        doc["limits"][key] = serde_json::json!(doc["limits"][key].as_u64().unwrap() * 3);
    }
    let mut s = StateStore::open(&path).unwrap();
    let run = s
        .admit_workflow(
            &AdmissionRequest {
                namespace: "jobs".into(),
                submission_key: "trial-binding".into(),
                workflow: WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap(),
                schemas_sha256: "d".repeat(64),
                execution_authority: receipt(),
                trial_limit: 1,
                trial_bindings: std::collections::BTreeMap::from([(
                    "train".parse().unwrap(),
                    "trial-1".into(),
                )]),
            },
            1,
        )
        .unwrap();
    let guard = s.acquire_controller(&run.run_id, 0, 2).unwrap();
    let mut resources = ResourceVector {
        cpu_seconds: 60,
        wall_seconds: 60,
        storage_bytes: 1048576,
        ..Default::default()
    };
    let bad = JobPreparation {
        job_id: "setup".into(),
        task_id: "setup".parse().unwrap(),
        trial_id: Some("invented".into()),
        resources,
    };
    assert!(s.prepare_job(&run.run_id, guard, &bad, 3).is_err());
    let guard = s
        .prepare_job(
            &run.run_id,
            guard,
            &JobPreparation {
                trial_id: None,
                ..bad
            },
            3,
        )
        .unwrap();
    assert_eq!(
        s.workflow_accounting(&run.run_id).unwrap().reserved.trials,
        0
    );
    let mut guard = prepared(&mut s, &run.run_id, guard);
    for transition in [
        JobTransition::Submit,
        JobTransition::Bind {
            backend_job_id: "owned".into(),
            binding_receipt: receipt(),
        },
        JobTransition::Terminate {
            termination: JobTermination::Exit {
                code: 1,
                receipt: receipt(),
            },
            quiescence: JobQuiescence::ProcessTree { receipt: receipt() },
        },
        JobTransition::BeginCollection,
        JobTransition::FinishCollection {
            manifest: receipt(),
        },
        JobTransition::Settle { measured: None },
    ] {
        guard = s
            .transition_job(&run.run_id, guard, "first", &transition, 4)
            .unwrap();
    }
    let retry = JobPreparation {
        job_id: "retry".into(),
        task_id: "train".parse().unwrap(),
        trial_id: Some("changed".into()),
        resources,
    };
    assert!(s.prepare_job(&run.run_id, guard, &retry, 5).is_err());
    resources.trials = 0;
    s.prepare_job(
        &run.run_id,
        guard,
        &JobPreparation {
            trial_id: Some("trial-1".into()),
            resources,
            ..retry
        },
        5,
    )
    .unwrap();
    drop(s);
    let mut s = StateStore::open(&path).unwrap();
    let retry = s.get_job(&run.run_id, "retry").unwrap().unwrap();
    assert_eq!(retry.attempt, 2);
    assert!(retry.submission_key.ends_with("/train/2"));
    let account = s.workflow_accounting(&run.run_id).unwrap();
    assert_eq!(account.settled.trials, 1);
    assert_eq!(account.reserved.trials, 0);
    assert_eq!(
        account.settled.cpu_seconds + account.reserved.cpu_seconds,
        180
    );
}
