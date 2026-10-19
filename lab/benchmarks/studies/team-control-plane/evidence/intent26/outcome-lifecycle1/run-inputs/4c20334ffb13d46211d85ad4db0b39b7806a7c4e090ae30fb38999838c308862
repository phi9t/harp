use team_control_plane::artifacts::*;
use team_control_plane::migrate;
use uuid::Uuid;
#[test]
fn publication_is_verified_and_faults_leave_no_eligible_manifest() {
    let root = std::env::temp_dir().join(format!("artifact-fixture-{}", uuid::Uuid::new_v4()));
    let store = ArtifactStore::open(&root).unwrap();
    let bytes = b"candidate evidence";
    let digest = "2d2816069bc9e3c2d9d59a54d79d4c58b8d9f7fce7dde46591e760598de1f866";
    assert!(
        store.publish(bytes, digest, PublishFault::None).is_err(),
        "incorrect digest must fail"
    );
    let digest = sha256(bytes);
    assert!(
        store
            .publish(bytes, &digest, PublishFault::BeforeFinalize)
            .is_err()
    );
    assert!(
        store.read(&digest).is_err(),
        "unfinalized bytes must not be eligible"
    );
    let manifest = store.publish(bytes, &digest, PublishFault::None).unwrap();
    assert_eq!(manifest.bytes, 18);
    assert_eq!(store.read(&digest).unwrap(), bytes);
    std::fs::write(root.join("objects").join(&digest), b"corrupt").unwrap();
    assert!(store.read(&digest).is_err());
    assert!(
        store.publish(bytes, &digest, PublishFault::None).is_err(),
        "immutable existing object cannot be overwritten"
    );
    std::fs::remove_dir_all(root).unwrap();
}

use team_control_plane::*;
fn envelope(epoch: Uuid, key: &str, rev: i64, command: Command) -> CommandEnvelope {
    CommandEnvelope {
        schema_version: 1,
        key: key.into(),
        expected_revision: rev,
        expected_generation: None,
        recovery_epoch: epoch,
        command,
    }
}
async fn fixture() -> (
    ArtifactService,
    Broker,
    String,
    String,
    String,
    Uuid,
    Uuid,
    Candidate,
    std::path::PathBuf,
) {
    let dsn = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").unwrap();
    // Every test gets a disposable schema on the real fixture server.
    let schema = format!("artifacts_{}", Uuid::new_v4().simple());
    let (client, conn) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        conn.await.unwrap();
    });
    client
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{dsn} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let mission = Uuid::new_v4();
    let task = Uuid::new_v4();
    let worker = Uuid::new_v4();
    let verifier = Uuid::new_v4();
    let input = GitInput {
        repository: "fixture".into(),
        commit: "a".repeat(40),
        path: "SPEC.md".into(),
        digest: "b".repeat(64),
    };
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "mission",
                0,
                Command::CreateMission {
                    id: mission,
                    objective: "fixture".into(),
                    input: input.clone(),
                    policy: Policy {
                        capacity: 2,
                        max_attempts: 8,
                        max_episodes: 8,
                        max_repairs: 3,
                        episode_deadline_seconds: 30,
                    },
                },
            ),
        )
        .await
        .unwrap();
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "task",
                0,
                Command::CreateTask {
                    id: task,
                    mission_id: mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "fixture".into(),
                        input,
                        acceptance: vec!["check".into()],
                        deliverables: vec!["result".into()],
                        write_scope: vec!["src".into()],
                    },
                },
            ),
        )
        .await
        .unwrap();
    let digest = broker
        .snapshot(&owner, SnapshotQuery::Task(task))
        .await
        .unwrap()
        .data["spec_digest"]
        .as_str()
        .unwrap()
        .to_string();
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "admit",
                1,
                Command::AdmitTask {
                    task_id: task,
                    spec_digest: digest.clone(),
                },
            ),
        )
        .await
        .unwrap();
    let root = std::env::temp_dir().join(format!("artifact-fixture-{}", Uuid::new_v4()));
    let service = ArtifactService::open(broker.clone(), &root).await.unwrap();
    for (key, principal, role, token, generation) in [
        ("worker", worker, Role::Worker, "worker", Some(1)),
        ("verifier", verifier, Role::Verifier, "verifier", Some(1)),
    ] {
        broker
            .execute(
                &owner,
                envelope(
                    epoch,
                    key,
                    0,
                    Command::GrantCredential {
                        credential_id: Uuid::new_v4(),
                        principal_id: principal,
                        role,
                        mission_id: Some(mission),
                        task_id: Some(task),
                        generation,
                        token_hash: token_hash(token),
                        expires_in_seconds: 600,
                    },
                ),
            )
            .await
            .unwrap();
    }
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "assign",
                2,
                Command::AssignAttempt {
                    task_id: task,
                    worker_id: worker,
                },
            ),
        )
        .await
        .unwrap();
    let bytes = b"frozen Git bundle";
    let artifact = sha256(bytes);
    service
        .store()
        .publish(bytes, &artifact, PublishFault::None)
        .unwrap();
    let candidate = Candidate {
        commit: "c".repeat(40),
        tree: "d".repeat(40),
        base: "a".repeat(40),
        spec_digest: digest,
        spec_revision: 2,
        generation: 1,
        artifact,
        deliverables: vec!["result".into()],
        risks: vec![],
    };
    (
        service,
        broker,
        owner,
        "worker".into(),
        "verifier".into(),
        task,
        verifier,
        candidate,
        root,
    )
}
fn cmd(
    epoch: Uuid,
    task: Uuid,
    key: &str,
    revision: i64,
    command: ArtifactCommand,
) -> ArtifactEnvelope {
    ArtifactEnvelope {
        key: key.into(),
        task,
        expected_revision: revision,
        generation: 1,
        recovery_epoch: epoch,
        command,
    }
}
#[tokio::test]
async fn exact_independent_verification_and_explicit_acceptance() {
    let (service, broker, owner, worker, verifier, task, verifier_id, candidate, root) =
        fixture().await;
    let epoch = broker
        .snapshot(&owner, SnapshotQuery::Task(task))
        .await
        .unwrap()
        .data["attempts"][0]["recovery_epoch"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let configure = ArtifactCommand::Configure {
        verifier: verifier_id,
        lead_acceptance: false,
        base: candidate.base.clone(),
        suite: "e".repeat(64),
        environment: "f".repeat(64),
        mandatory_commands: vec![vec!["check".into()]],
    };
    let configured = service
        .execute(
            &owner,
            cmd(epoch, task, "policy", 3, configure),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert_eq!(
        configured.revision, 4,
        "configuration must be a durable task revision"
    );

    let submit = cmd(
        epoch,
        task,
        "submit",
        4,
        ArtifactCommand::Submit {
            candidate: candidate.clone(),
        },
    );
    assert!(
        service
            .execute(&verifier, submit.clone(), ReferenceFault::None)
            .await
            .is_err()
    );
    assert!(
        service
            .execute(&worker, submit.clone(), ReferenceFault::BeforeReference)
            .await
            .is_err()
    );
    let submitted = service
        .execute(&worker, submit.clone(), ReferenceFault::None)
        .await
        .unwrap();
    assert_eq!(
        submitted,
        service
            .execute(&worker, submit, ReferenceFault::None)
            .await
            .unwrap()
    );
    let candidate_id = submitted.candidate_id.unwrap();
    println!(
        "ticket11 task={task} epoch={epoch} candidate={candidate_id} artifact={}",
        candidate.artifact
    );
    let logs = sha256(b"check passed");
    service
        .store()
        .publish(b"check passed", &logs, PublishFault::None)
        .unwrap();
    let report = Verification {
        candidate: candidate.clone(),
        suite: "e".repeat(64),
        environment: "f".repeat(64),
        checks: vec![Check {
            command: vec!["check".into()],
            exit_code: 0,
            logs: logs.clone(),
        }],
        resolved_decisions: true,
    };
    let verify = cmd(
        epoch,
        task,
        "verify",
        5,
        ArtifactCommand::Verify {
            verification: report.clone(),
        },
    );
    assert!(
        service
            .execute(&worker, verify.clone(), ReferenceFault::None)
            .await
            .is_err()
    );
    let mut stale = report.clone();
    stale.candidate.tree = "0".repeat(40);
    assert!(
        service
            .execute(
                &verifier,
                cmd(
                    epoch,
                    task,
                    "stale",
                    5,
                    ArtifactCommand::Verify {
                        verification: stale
                    }
                ),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    let verified = service
        .execute(&verifier, verify, ReferenceFault::None)
        .await
        .unwrap();
    println!(
        "ticket11 verification={}",
        verified.report_id.as_ref().unwrap()
    );
    let accept = cmd(
        epoch,
        task,
        "accept",
        6,
        ArtifactCommand::Accept {
            candidate_id,
            report_id: verified.report_id.unwrap(),
        },
    );
    assert!(
        service
            .execute(&worker, accept.clone(), ReferenceFault::None)
            .await
            .is_err()
    );
    assert!(
        service
            .execute(&verifier, accept.clone(), ReferenceFault::None)
            .await
            .is_err()
    );
    assert!(
        service
            .collect(&owner, std::time::Duration::ZERO)
            .await
            .unwrap()
            .is_empty(),
        "referenced candidates/logs cannot be collected"
    );
    std::fs::remove_file(root.join("objects").join(&logs)).unwrap();
    assert!(
        service
            .execute(&owner, accept.clone(), ReferenceFault::None)
            .await
            .is_err(),
        "missing referenced evidence blocks acceptance"
    );
    service
        .store()
        .publish(b"check passed", &logs, PublishFault::None)
        .unwrap();
    // Failed acceptance is retryable only with a new idempotency key if semantic rejection was recorded.
    let mut accept = accept;
    accept.key = "accept-restored".into();
    assert!(
        service
            .execute(&owner, accept, ReferenceFault::None)
            .await
            .unwrap()
            .accepted
    );
    assert_eq!(
        service.snapshot(&owner, task).await.unwrap().status,
        "accepted"
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn git(repo: &std::path::Path, args: &[&str]) -> String {
    let result = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().to_string()
}
#[test]
fn changed_base_prepares_and_checks_actual_git_integration_candidate() {
    let repo = std::env::temp_dir().join(format!("integration-fixture-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-b", "fixture"]);
    git(&repo, &["config", "user.name", "Fixture"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    std::fs::write(repo.join("base"), "base").unwrap();
    git(&repo, &["add", "base"]);
    git(&repo, &["commit", "-m", "base"]);
    let base = git(&repo, &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("feature"), "feature").unwrap();
    git(&repo, &["add", "feature"]);
    git(&repo, &["commit", "-m", "feature"]);
    let source = git(&repo, &["rev-parse", "HEAD"]);
    let commands = vec![vec![
        "sh".into(),
        "-c".into(),
        "test -f feature && test -f base".into(),
    ]];
    let first = prepare_integration(&repo, &source, &base, &commands)
        .expect("construct and check actual integration candidate");
    assert_eq!(first.base, base);
    assert_eq!(first.checks[0].exit_code, 0);
    assert_eq!(
        first.tree,
        git(&repo, &["rev-parse", &format!("{}^{{tree}}", first.commit)])
    );
    git(&repo, &["checkout", "--detach", &base]);
    std::fs::write(repo.join("new-base"), "new base").unwrap();
    git(&repo, &["add", "new-base"]);
    git(&repo, &["commit", "-m", "new base"]);
    let new_base = git(&repo, &["rev-parse", "HEAD"]);
    let checks = vec![vec![
        "sh".into(),
        "-c".into(),
        "test -f feature && test -f new-base".into(),
    ]];
    let second = prepare_integration(&repo, &source, &new_base, &checks).unwrap();
    println!(
        "ticket11 integration first={} tree={} base={} changed={} tree={} base={}",
        first.commit, first.tree, first.base, second.commit, second.tree, second.base
    );
    assert_ne!(first.commit, second.commit);
    assert_ne!(first.tree, second.tree);
    assert_eq!(second.base, new_base);
    assert_eq!(second.checks[0].exit_code, 0);
    assert_eq!(
        git(&repo, &["rev-parse", "HEAD"]),
        new_base,
        "preparation cannot move mainline"
    );
    assert_eq!(
        git(&repo, &["worktree", "list", "--porcelain"])
            .matches("worktree ")
            .count(),
        1,
        "temporary verifier worktree released"
    );
    let failing = prepare_integration(
        &repo,
        &source,
        &new_base,
        &[vec!["sh".into(), "-c".into(), "exit 7".into()]],
    )
    .unwrap();
    assert_eq!(
        failing.checks[0].exit_code, 7,
        "gate result is retained, never replaced with source proof"
    );
    std::fs::remove_dir_all(repo).unwrap();
}

async fn epoch(broker: &Broker, owner: &str, task: Uuid) -> Uuid {
    broker
        .snapshot(owner, SnapshotQuery::Task(task))
        .await
        .unwrap()
        .data["attempts"][0]["recovery_epoch"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}
fn policy_command(verifier: Uuid, base: String) -> ArtifactCommand {
    ArtifactCommand::Configure {
        verifier,
        lead_acceptance: false,
        base,
        suite: "e".repeat(64),
        environment: "f".repeat(64),
        mandatory_commands: vec![vec!["check".into()]],
    }
}
fn verification(candidate: Candidate, logs: String) -> Verification {
    Verification {
        candidate,
        suite: "e".repeat(64),
        environment: "f".repeat(64),
        checks: vec![Check {
            command: vec!["check".into()],
            exit_code: 0,
            logs,
        }],
        resolved_decisions: true,
    }
}
#[tokio::test]
async fn evidence_reads_are_scoped_and_revalidate_parent_authority() {
    let (service, broker, owner, worker, verifier, task, verifier_id, candidate, root) =
        fixture().await;
    let epoch = epoch(&broker, &owner, task).await;
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "configure",
                3,
                policy_command(verifier_id, candidate.base.clone()),
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "submit",
                4,
                ArtifactCommand::Submit {
                    candidate: candidate.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert_eq!(
        service
            .read_evidence(&verifier, task, &candidate.artifact)
            .await
            .unwrap(),
        b"frozen Git bundle",
        "verifier reads the retained bytes rather than implementer summary"
    );
    let orphan = sha256(b"unrelated secret");
    service
        .store()
        .publish(b"unrelated secret", &orphan, PublishFault::None)
        .unwrap();
    assert!(
        service
            .read_evidence(&verifier, task, &orphan)
            .await
            .is_err(),
        "publication alone does not grant task access"
    );
    assert!(
        service
            .read_evidence("unknown", task, &candidate.artifact)
            .await
            .is_err()
    );
    assert!(service.snapshot(&verifier, Uuid::new_v4()).await.is_err());
    broker.rotate_epoch(&owner, Uuid::new_v4()).await.unwrap();
    assert!(
        service
            .read_evidence(&verifier, task, &candidate.artifact)
            .await
            .is_err()
    );
    assert!(service.snapshot(&owner, task).await.is_err());
    assert!(
        service
            .execute(
                &worker,
                cmd(
                    epoch,
                    task,
                    "stale",
                    5,
                    ArtifactCommand::Submit { candidate }
                ),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn old_base_spec_candidate_and_report_never_accept_current_candidate() {
    let (service, broker, owner, worker, verifier, task, verifier_id, candidate, root) =
        fixture().await;
    let epoch = epoch(&broker, &owner, task).await;
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "configure",
                3,
                policy_command(verifier_id, candidate.base.clone()),
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    for (key, mut bad) in [
        ("generation", candidate.clone()),
        ("spec", candidate.clone()),
        ("spec-revision", candidate.clone()),
    ] {
        match key {
            "generation" => bad.generation = 2,
            "spec" => bad.spec_digest = "0".repeat(64),
            _ => bad.spec_revision = 1,
        };
        assert!(
            service
                .execute(
                    &worker,
                    cmd(
                        epoch,
                        task,
                        key,
                        4,
                        ArtifactCommand::Submit { candidate: bad }
                    ),
                    ReferenceFault::None
                )
                .await
                .is_err()
        );
    }
    let first = service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "submit",
                4,
                ArtifactCommand::Submit {
                    candidate: candidate.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let logs = sha256(b"pass");
    service
        .store()
        .publish(b"pass", &logs, PublishFault::None)
        .unwrap();
    let report = verification(candidate.clone(), logs.clone());
    for (key, mut bad) in [
        ("suite", report.clone()),
        ("environment", report.clone()),
        ("base", report.clone()),
        ("commit", report.clone()),
    ] {
        match key {
            "suite" => bad.suite = "0".repeat(64),
            "environment" => bad.environment = "0".repeat(64),
            "base" => bad.candidate.base = "0".repeat(40),
            _ => bad.candidate.commit = "0".repeat(40),
        };
        assert!(
            service
                .execute(
                    &verifier,
                    cmd(
                        epoch,
                        task,
                        key,
                        5,
                        ArtifactCommand::Verify { verification: bad }
                    ),
                    ReferenceFault::None
                )
                .await
                .is_err()
        );
    }
    let verified = service
        .execute(
            &verifier,
            cmd(
                epoch,
                task,
                "verify",
                5,
                ArtifactCommand::Verify {
                    verification: report,
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "base-changed",
                6,
                policy_command(verifier_id, "0".repeat(40)),
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let old = ArtifactCommand::Accept {
        candidate_id: first.candidate_id.clone().unwrap(),
        report_id: verified.report_id.clone().unwrap(),
    };
    assert!(
        service
            .execute(
                &owner,
                cmd(epoch, task, "old-base", 7, old.clone()),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    let mut second = candidate;
    second.base = "0".repeat(40);
    second.commit = "1".repeat(40);
    second.tree = "2".repeat(40);
    let current = service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "new-candidate",
                7,
                ArtifactCommand::Submit {
                    candidate: second.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let new_report = service
        .execute(
            &verifier,
            cmd(
                epoch,
                task,
                "new-verify",
                8,
                ArtifactCommand::Verify {
                    verification: verification(second, logs),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert!(
        service
            .execute(
                &owner,
                cmd(epoch, task, "old-candidate", 9, old),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    assert!(
        service
            .execute(
                &owner,
                cmd(
                    epoch,
                    task,
                    "old-report",
                    9,
                    ArtifactCommand::Accept {
                        candidate_id: current.candidate_id.clone().unwrap(),
                        report_id: verified.report_id.unwrap()
                    }
                ),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    let accept = cmd(
        epoch,
        task,
        "current",
        9,
        ArtifactCommand::Accept {
            candidate_id: current.candidate_id.unwrap(),
            report_id: new_report.report_id.unwrap(),
        },
    );
    assert!(
        service
            .execute(&owner, accept.clone(), ReferenceFault::None)
            .await
            .unwrap()
            .accepted
    );
    let mut changed = accept;
    changed.task = Uuid::new_v4();
    assert!(
        service
            .execute(&owner, changed, ReferenceFault::None)
            .await
            .is_err(),
        "cached results cannot cross task scope"
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn grace_collection_removes_unreferenced_fault_orphans_only() {
    let (service, broker, owner, worker, _, task, verifier, candidate, root) = fixture().await;
    let epoch = epoch(&broker, &owner, task).await;
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "configure",
                3,
                policy_command(verifier, candidate.base.clone()),
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let unfinalized = sha256(b"unfinalized");
    assert!(
        service
            .store()
            .publish(b"unfinalized", &unfinalized, PublishFault::BeforeFinalize)
            .is_err()
    );
    assert!(
        service
            .execute(
                &worker,
                cmd(
                    epoch,
                    task,
                    "orphan",
                    4,
                    ArtifactCommand::Submit {
                        candidate: candidate.clone()
                    }
                ),
                ReferenceFault::BeforeReference
            )
            .await
            .is_err()
    );
    assert!(
        service
            .snapshot(&owner, task)
            .await
            .unwrap()
            .candidate
            .is_none()
    );
    assert!(
        service
            .collect(&owner, std::time::Duration::from_secs(3600))
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        service
            .collect(&worker, std::time::Duration::ZERO)
            .await
            .is_err()
    );
    let removed = service
        .collect(&owner, std::time::Duration::ZERO)
        .await
        .unwrap();
    assert!(removed.contains(&format!("objects/{unfinalized}")));
    assert!(removed.contains(&format!("objects/{}", candidate.artifact)));
    assert!(removed.contains(&format!("manifests/{}", candidate.artifact)));
    assert!(
        service
            .execute(
                &worker,
                cmd(
                    epoch,
                    task,
                    "missing",
                    4,
                    ArtifactCommand::Submit {
                        candidate: candidate.clone()
                    }
                ),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    service
        .store()
        .publish(
            b"frozen Git bundle",
            &candidate.artifact,
            PublishFault::None,
        )
        .unwrap();
    service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "referenced",
                4,
                ArtifactCommand::Submit {
                    candidate: candidate.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert!(
        service
            .collect(&owner, std::time::Duration::ZERO)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        service.store().read(&candidate.artifact).unwrap(),
        b"frozen Git bundle"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn verifier_credential_has_no_lead_execution_authority() {
    let (service, broker, owner, worker, verifier, task, verifier_id, candidate, root) =
        fixture().await;
    let epoch = epoch(&broker, &owner, task).await;
    let forbidden = broker
        .execute(
            &verifier,
            envelope(
                epoch,
                "verifier-cannot-assign",
                3,
                Command::AssignAttempt {
                    task_id: task,
                    worker_id: Uuid::new_v4(),
                },
            ),
        )
        .await
        .unwrap_err();
    assert_eq!(
        forbidden.code,
        ErrorCode::Unauthorized,
        "a verifier credential must never enter execution admission as a lead"
    );
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "configure",
                3,
                policy_command(verifier_id, candidate.base.clone()),
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "submit",
                4,
                ArtifactCommand::Submit {
                    candidate: candidate.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert_eq!(
        service
            .read_evidence(&verifier, task, &candidate.artifact)
            .await
            .unwrap(),
        b"frozen Git bundle"
    );
    let logs = sha256(b"verifier check");
    service
        .store()
        .publish(b"verifier check", &logs, PublishFault::None)
        .unwrap();
    let report = service
        .execute(
            &verifier,
            cmd(
                epoch,
                task,
                "verify",
                5,
                ArtifactCommand::Verify {
                    verification: verification(candidate.clone(), logs),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert!(report.report_id.is_some());
    assert!(
        service
            .execute(
                &verifier,
                cmd(
                    epoch,
                    task,
                    "cannot-configure",
                    6,
                    policy_command(verifier_id, candidate.base)
                ),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    assert!(
        service
            .execute(
                &verifier,
                cmd(
                    epoch,
                    task,
                    "cannot-accept",
                    6,
                    ArtifactCommand::Accept {
                        candidate_id: report.candidate_id.unwrap(),
                        report_id: report.report_id.unwrap()
                    }
                ),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn git_bundle(repo: &std::path::Path, commit: &str, base: &str) -> Vec<u8> {
    let bundle = repo.join(format!("candidate-{}.bundle", Uuid::new_v4()));
    // git bundle requires a named tip. This reference exists only in the
    // disposable fixture repository and does not move its source/mainline.
    let reference = format!("refs/fixture-candidates/{}", Uuid::new_v4());
    git(repo, &["update-ref", &reference, commit]);
    assert_eq!(git(repo, &["cat-file", "-t", base]), "commit");
    git(
        repo,
        &["bundle", "create", bundle.to_str().unwrap(), &reference],
    );
    git(repo, &["update-ref", "-d", &reference]);
    let bytes = std::fs::read(&bundle).unwrap();
    std::fs::remove_file(bundle).unwrap();
    bytes
}
fn independent_git_check(
    bytes: &[u8],
    commit: &str,
    base: &str,
    commands: &[Vec<String>],
) -> (String, String, String, Vec<IntegrationCheck>) {
    let repo =
        std::env::temp_dir().join(format!("independent-artifact-verifier-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-b", "verifier"]);
    let bundle = repo.join("candidate.bundle");
    std::fs::write(&bundle, bytes).unwrap();
    git(&repo, &["fetch", bundle.to_str().unwrap(), commit]);
    git(&repo, &["checkout", "--detach", "FETCH_HEAD"]);
    let measured_commit = git(&repo, &["rev-parse", "HEAD"]);
    let measured_tree = git(&repo, &["rev-parse", "HEAD^{tree}"]);
    let measured_base = git(&repo, &["rev-parse", "HEAD^1"]);
    assert_eq!(
        measured_base, base,
        "base is derived from retained integration commit ancestry"
    );
    let checks = commands
        .iter()
        .map(|command| {
            let result = std::process::Command::new(&command[0])
                .args(&command[1..])
                .current_dir(&repo)
                .output()
                .unwrap();
            let mut logs = result.stdout;
            logs.extend_from_slice(b"\n--- stderr ---\n");
            logs.extend_from_slice(&result.stderr);
            IntegrationCheck {
                command: command.clone(),
                exit_code: result.status.code().unwrap_or(-1),
                logs,
            }
        })
        .collect();
    std::fs::remove_dir_all(repo).unwrap();
    (measured_commit, measured_tree, measured_base, checks)
}
#[tokio::test]
async fn actual_git_bytes_and_independent_executed_logs_bind_postgres_acceptance() {
    let (service, broker, owner, worker, verifier, task, verifier_id, template, root) =
        fixture().await;
    let epoch = epoch(&broker, &owner, task).await;
    assert_eq!(
        broker
            .execute(
                &verifier,
                envelope(
                    epoch,
                    "verifier-no-execution",
                    3,
                    Command::AssignAttempt {
                        task_id: task,
                        worker_id: Uuid::new_v4()
                    }
                )
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    let repo = root.join("git-fixture");
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-b", "source"]);
    git(&repo, &["config", "user.name", "Fixture"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    std::fs::write(repo.join("base"), "initial-base\n").unwrap();
    git(&repo, &["add", "base"]);
    git(&repo, &["commit", "-m", "base"]);
    let base = git(&repo, &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("feature"), "candidate\n").unwrap();
    git(&repo, &["add", "feature"]);
    git(&repo, &["commit", "-m", "feature"]);
    let source = git(&repo, &["rev-parse", "HEAD"]);
    let commands=vec![vec!["sh".into(),"-c".into(),"test -f feature && cat feature && cat base && if test -f base-change; then cat base-change; else printf old-base; fi".into()]];
    let first = prepare_integration(&repo, &source, &base, &commands).unwrap();
    assert_eq!(first.checks[0].exit_code, 0);
    let configure = |base: String| ArtifactCommand::Configure {
        verifier: verifier_id,
        lead_acceptance: false,
        base,
        suite: "e".repeat(64),
        environment: "f".repeat(64),
        mandatory_commands: commands.clone(),
    };
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "configure-real-base",
                3,
                configure(base.clone()),
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let first_bytes = git_bundle(&repo, &first.commit, &first.base);
    let first_digest = sha256(&first_bytes);
    service
        .store()
        .publish(&first_bytes, &first_digest, PublishFault::None)
        .unwrap();
    let first_candidate = Candidate {
        commit: first.commit.clone(),
        tree: first.tree.clone(),
        base: first.base.clone(),
        artifact: first_digest.clone(),
        ..template.clone()
    };
    let first_submitted = service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "submit-real-first",
                4,
                ArtifactCommand::Submit {
                    candidate: first_candidate.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let retained = service
        .read_evidence(&verifier, task, &first_digest)
        .await
        .unwrap();
    assert_eq!(retained, first_bytes);
    let (measured_commit, measured_tree, measured_base, executed) =
        independent_git_check(&retained, &first.commit, &first.base, &commands);
    assert_eq!(measured_commit, first_candidate.commit);
    assert_eq!(measured_tree, first_candidate.tree);
    assert_eq!(measured_base, first_candidate.base);
    let first_logs = sha256(&executed[0].logs);
    service
        .store()
        .publish(&executed[0].logs, &first_logs, PublishFault::None)
        .unwrap();
    let first_report = Verification {
        candidate: first_candidate,
        suite: "e".repeat(64),
        environment: "f".repeat(64),
        checks: vec![Check {
            command: executed[0].command.clone(),
            exit_code: executed[0].exit_code,
            logs: first_logs.clone(),
        }],
        resolved_decisions: true,
    };
    let first_verified = service
        .execute(
            &verifier,
            cmd(
                epoch,
                task,
                "verify-measured-first",
                5,
                ArtifactCommand::Verify {
                    verification: first_report.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    git(&repo, &["checkout", "--detach", &base]);
    std::fs::write(repo.join("base-change"), "changed-real-base\n").unwrap();
    git(&repo, &["add", "base-change"]);
    git(&repo, &["commit", "-m", "changed base"]);
    let changed_base = git(&repo, &["rev-parse", "HEAD"]);
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "configure-changed-real-base",
                6,
                configure(changed_base.clone()),
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let old_accept = ArtifactCommand::Accept {
        candidate_id: first_submitted.candidate_id.unwrap(),
        report_id: first_verified.report_id.unwrap(),
    };
    assert!(
        service
            .execute(
                &owner,
                cmd(epoch, task, "old-real-proof", 7, old_accept.clone()),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    let second = prepare_integration(&repo, &source, &changed_base, &commands).unwrap();
    assert_ne!(first.commit, second.commit);
    assert_ne!(first.tree, second.tree);
    let second_bytes = git_bundle(&repo, &second.commit, &second.base);
    let second_digest = sha256(&second_bytes);
    assert_ne!(first_digest, second_digest);
    service
        .store()
        .publish(&second_bytes, &second_digest, PublishFault::None)
        .unwrap();
    let second_candidate = Candidate {
        commit: second.commit.clone(),
        tree: second.tree.clone(),
        base: second.base.clone(),
        artifact: second_digest.clone(),
        ..template
    };
    let second_submitted = service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "submit-real-second",
                7,
                ArtifactCommand::Submit {
                    candidate: second_candidate.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert!(
        service
            .execute(
                &verifier,
                cmd(
                    epoch,
                    task,
                    "old-report-new-real-candidate",
                    8,
                    ArtifactCommand::Verify {
                        verification: first_report
                    }
                ),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    let retained = service
        .read_evidence(&verifier, task, &second_digest)
        .await
        .unwrap();
    assert_eq!(retained, second_bytes);
    let (measured_commit, measured_tree, measured_base, executed) =
        independent_git_check(&retained, &second.commit, &second.base, &commands);
    assert_eq!(measured_commit, second_candidate.commit);
    assert_eq!(measured_tree, second_candidate.tree);
    assert_eq!(measured_base, second_candidate.base);
    assert_eq!(executed[0].exit_code, 0);
    assert!(String::from_utf8_lossy(&executed[0].logs).contains("changed-real-base"));
    let second_logs = sha256(&executed[0].logs);
    assert_ne!(first_logs, second_logs);
    service
        .store()
        .publish(&executed[0].logs, &second_logs, PublishFault::None)
        .unwrap();
    let second_report = Verification {
        candidate: second_candidate.clone(),
        suite: "e".repeat(64),
        environment: "f".repeat(64),
        checks: vec![Check {
            command: executed[0].command.clone(),
            exit_code: executed[0].exit_code,
            logs: second_logs.clone(),
        }],
        resolved_decisions: true,
    };
    let second_verified = service
        .execute(
            &verifier,
            cmd(
                epoch,
                task,
                "verify-measured-second",
                8,
                ArtifactCommand::Verify {
                    verification: second_report,
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert!(
        service
            .execute(
                &owner,
                cmd(
                    epoch,
                    task,
                    "old-proof-after-new-real-verification",
                    9,
                    old_accept
                ),
                ReferenceFault::None
            )
            .await
            .is_err()
    );
    let accepted = service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "accept-measured-second",
                9,
                ArtifactCommand::Accept {
                    candidate_id: second_submitted.candidate_id.unwrap(),
                    report_id: second_verified.report_id.unwrap(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    assert!(accepted.accepted);
    let snapshot = service.snapshot(&owner, task).await.unwrap();
    assert_eq!(snapshot.candidate.unwrap(), second_candidate);
    assert_eq!(
        git(&repo, &["rev-parse", "HEAD"]),
        changed_base,
        "protected branch is unchanged"
    );
    println!(
        "ticket11 real_git_pg task={task} first_commit={} first_tree={} first_base={} first_artifact={first_digest} first_logs={first_logs} new_commit={} new_tree={} new_base={} new_artifact={second_digest} new_logs={second_logs} accepted_candidate={} accepted_report={}",
        first.commit,
        first.tree,
        first.base,
        second.commit,
        second.tree,
        second.base,
        accepted.candidate_id.unwrap(),
        accepted.report_id.unwrap()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn verifier_grants_are_narrow_and_rechecked_for_cached_report_reads_and_mutations() {
    let (service, broker, owner, worker, verifier, task, verifier_id, candidate, root) =
        fixture().await;
    let epoch = epoch(&broker, &owner, task).await;
    let mission: Uuid = broker
        .snapshot(&owner, SnapshotQuery::Task(task))
        .await
        .unwrap()
        .data["mission_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let grant = |key: &str,
                 token: &str,
                 task_scope: Option<Uuid>,
                 generation: Option<i64>,
                 expires: i64| {
        envelope(
            epoch,
            key,
            0,
            Command::GrantCredential {
                credential_id: Uuid::new_v4(),
                principal_id: verifier_id,
                role: Role::Verifier,
                mission_id: Some(mission),
                task_id: task_scope,
                generation,
                token_hash: token_hash(token),
                expires_in_seconds: expires,
            },
        )
    };
    for (key, task_scope, generation) in [
        ("no-task", None, Some(1)),
        ("no-generation", Some(task), None),
        ("zero-generation", Some(task), Some(0)),
    ] {
        assert_eq!(
            broker
                .execute(&owner, grant(key, key, task_scope, generation, 600))
                .await
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "configure",
                3,
                policy_command(verifier_id, candidate.base.clone()),
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "submit",
                4,
                ArtifactCommand::Submit {
                    candidate: candidate.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let logs = sha256(b"narrow verifier proof");
    service
        .store()
        .publish(b"narrow verifier proof", &logs, PublishFault::None)
        .unwrap();
    let verify = cmd(
        epoch,
        task,
        "verify-cached",
        5,
        ArtifactCommand::Verify {
            verification: verification(candidate.clone(), logs),
        },
    );
    let result = service
        .execute(&verifier, verify.clone(), ReferenceFault::None)
        .await
        .unwrap();
    assert_eq!(
        service
            .execute(&verifier, verify.clone(), ReferenceFault::None)
            .await
            .unwrap(),
        result
    );
    broker
        .execute(
            &owner,
            grant(
                "future-generation",
                "future-verifier",
                Some(task),
                Some(2),
                600,
            ),
        )
        .await
        .unwrap();
    assert_eq!(
        service
            .read_evidence("future-verifier", task, &candidate.artifact)
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    assert_eq!(
        service
            .snapshot("future-verifier", task)
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    assert_eq!(
        service
            .execute("future-verifier", verify.clone(), ReferenceFault::None)
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    assert_eq!(
        broker
            .snapshot("future-verifier", SnapshotQuery::Task(task))
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    broker
        .execute(
            &owner,
            grant("short-verifier", "short-verifier", Some(task), Some(1), 1),
        )
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    assert_eq!(
        service
            .read_evidence("short-verifier", task, &candidate.artifact)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    assert_eq!(
        service
            .snapshot("short-verifier", task)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    assert_eq!(
        service
            .execute("short-verifier", verify, ReferenceFault::None)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    assert_eq!(
        broker
            .snapshot("short-verifier", SnapshotQuery::Task(task))
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    let forbidden_commands = vec![
        Command::SetDependencies {
            task_id: task,
            dependencies: vec![],
        },
        Command::AdmitTask {
            task_id: task,
            spec_digest: candidate.spec_digest.clone(),
        },
        grant("nested-grant", "nested-grant", Some(task), Some(1), 600).command,
    ];
    for (index, command) in forbidden_commands.into_iter().enumerate() {
        assert_eq!(
            broker
                .execute(
                    &verifier,
                    envelope(epoch, &format!("forbidden-{index}"), 6, command)
                )
                .await
                .unwrap_err()
                .code,
            ErrorCode::Unauthorized
        );
    }
    assert_eq!(
        broker
            .snapshot(&verifier, SnapshotQuery::Mission(mission))
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized,
        "task verifier grant cannot read mission-wide state"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn core_migrate_installs_verifier_grants_without_artifact_constructor() {
    let dsn = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").unwrap();
    let schema = format!("core_verifier_{}", Uuid::new_v4().simple());
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    client
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{dsn} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    // No ArtifactService/ArtifactStore constructor is involved in this test.
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let mission = Uuid::new_v4();
    let task = Uuid::new_v4();
    let input = GitInput {
        repository: "fixture".into(),
        commit: "a".repeat(40),
        path: "SPEC.md".into(),
        digest: "b".repeat(64),
    };
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "mission",
                0,
                Command::CreateMission {
                    id: mission,
                    objective: "fixture".into(),
                    input: input.clone(),
                    policy: Policy {
                        capacity: 2,
                        max_attempts: 8,
                        max_episodes: 8,
                        max_repairs: 3,
                        episode_deadline_seconds: 30,
                    },
                },
            ),
        )
        .await
        .unwrap();
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "task",
                0,
                Command::CreateTask {
                    id: task,
                    mission_id: mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "fixture".into(),
                        input,
                        acceptance: vec!["verified".into()],
                        deliverables: vec!["result".into()],
                        write_scope: vec!["src".into()],
                    },
                },
            ),
        )
        .await
        .unwrap();
    let grant = broker
        .execute(
            &owner,
            envelope(
                epoch,
                "verifier",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: Uuid::new_v4(),
                    role: Role::Verifier,
                    mission_id: Some(mission),
                    task_id: Some(task),
                    generation: Some(1),
                    token_hash: token_hash("verifier"),
                    expires_in_seconds: 60,
                },
            ),
        )
        .await;
    assert!(
        grant.is_ok(),
        "public core migrate must install every schema constraint required by public core verifier grants: {grant:?}"
    );
    // Migration is repeatable through the same authoritative public entrypoint.
    migrate(&dsn).await.unwrap();
}

#[tokio::test]
async fn repeated_failed_report_hash_requires_each_exact_charged_verification_cycle() {
    let (service, broker, owner, worker, verifier, task, verifier_id, candidate, root) =
        fixture().await;
    let epoch = broker
        .snapshot(&owner, SnapshotQuery::Task(task))
        .await
        .unwrap()
        .data["attempts"][0]["recovery_epoch"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let argv = vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        "printf deterministic-failed-gate; exit 17".to_string(),
    ];
    service
        .execute(
            &owner,
            cmd(
                epoch,
                task,
                "cycle-policy",
                3,
                ArtifactCommand::Configure {
                    verifier: verifier_id,
                    lead_acceptance: false,
                    base: candidate.base.clone(),
                    suite: "e".repeat(64),
                    environment: "f".repeat(64),
                    mandatory_commands: vec![argv.clone()],
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap();
    let mut revision = service
        .execute(
            &worker,
            cmd(
                epoch,
                task,
                "cycle-initial",
                4,
                ArtifactCommand::Submit {
                    candidate: candidate.clone(),
                },
            ),
            ReferenceFault::None,
        )
        .await
        .unwrap()
        .revision;
    let mut report_id = None;
    for cycle in 0..4 {
        // This ordinary artifact-service contract fixture executes the declared
        // command each time. Registered supervised SDK proof is a separate case.
        let output = std::process::Command::new(&argv[0])
            .args(&argv[1..])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(17));
        let logs = sha256(&output.stdout);
        service
            .publish_evidence(&verifier, task, &logs, &output.stdout)
            .await
            .unwrap();
        let verification = Verification {
            candidate: candidate.clone(),
            suite: "e".repeat(64),
            environment: "f".repeat(64),
            checks: vec![Check {
                command: argv.clone(),
                exit_code: 17,
                logs,
            }],
            resolved_decisions: true,
        };
        let verify = cmd(
            epoch,
            task,
            &format!("cycle-verify-{cycle}"),
            revision,
            ArtifactCommand::Verify { verification },
        );
        let failed = service
            .execute(&verifier, verify.clone(), ReferenceFault::None)
            .await
            .unwrap();
        assert_eq!(
            service
                .execute(&verifier, verify, ReferenceFault::None)
                .await
                .unwrap(),
            failed,
            "exact replay creates no new verification event"
        );
        assert_eq!(
            service.snapshot(&owner, task).await.unwrap().status,
            "repair_required"
        );
        if let Some(prior) = &report_id {
            assert_eq!(
                failed.report_id.as_ref(),
                Some(prior),
                "identical actual gate bytes keep content hash"
            );
        }
        report_id = failed.report_id.clone();
        revision = failed.revision;
        assert_eq!(
            service
                .execute(
                    &worker,
                    cmd(
                        epoch,
                        task,
                        &format!("cycle-uncharged-{cycle}"),
                        revision,
                        ArtifactCommand::Submit {
                            candidate: candidate.clone()
                        }
                    ),
                    ReferenceFault::None
                )
                .await
                .unwrap_err()
                .code,
            ErrorCode::DependencyUnmet,
            "prior charged repair cannot authorize a later failure with same hash"
        );
        if cycle == 0 {
            let policy = |suite: String| ArtifactCommand::Configure {
                verifier: verifier_id,
                lead_acceptance: false,
                base: candidate.base.clone(),
                suite,
                environment: "f".repeat(64),
                mandatory_commands: vec![argv.clone()],
            };
            revision = service
                .execute(
                    &owner,
                    cmd(
                        epoch,
                        task,
                        "invalidate-failed-policy",
                        revision,
                        policy("9".repeat(64)),
                    ),
                    ReferenceFault::None,
                )
                .await
                .unwrap()
                .revision;
            assert_eq!(
                service
                    .execute(
                        &worker,
                        cmd(
                            epoch,
                            task,
                            "invalid-policy-cannot-reset-repair",
                            revision,
                            ArtifactCommand::Submit {
                                candidate: candidate.clone()
                            }
                        ),
                        ReferenceFault::None
                    )
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet
            );
            assert!(
                service
                    .execute(
                        &owner,
                        cmd(
                            epoch,
                            task,
                            "invalid-policy-cannot-charge-stale-proof",
                            revision,
                            ArtifactCommand::Repair {
                                candidate_id: failed.candidate_id.clone().unwrap(),
                                report_id: failed.report_id.clone().unwrap(),
                                reason: "stale policy proof must not authorize work".into()
                            }
                        ),
                        ReferenceFault::None
                    )
                    .await
                    .is_err()
            );
            assert_eq!(
                kernel::Kernel::new(broker.clone())
                    .status(&owner)
                    .await
                    .unwrap()["missions"][0]["consumed_repairs"],
                0
            );
            revision = service
                .execute(
                    &owner,
                    cmd(
                        epoch,
                        task,
                        "restore-failed-policy",
                        revision,
                        policy("e".repeat(64)),
                    ),
                    ReferenceFault::None,
                )
                .await
                .unwrap()
                .revision;
        }
        let repair = cmd(
            epoch,
            task,
            &format!("cycle-repair-{cycle}"),
            revision,
            ArtifactCommand::Repair {
                candidate_id: failed.candidate_id.unwrap(),
                report_id: failed.report_id.unwrap(),
                reason: "authorize exactly this executed failure event".into(),
            },
        );
        let result = service
            .execute(&owner, repair.clone(), ReferenceFault::None)
            .await;
        if cycle == 3 {
            assert_eq!(result.unwrap_err().code, ErrorCode::BudgetExhausted);
            break;
        }
        let result = result.unwrap();
        assert_eq!(
            service
                .execute(&owner, repair, ReferenceFault::None)
                .await
                .unwrap(),
            result,
            "one charge per exact command replay"
        );
        revision = service
            .execute(
                &worker,
                cmd(
                    epoch,
                    task,
                    &format!("cycle-authorized-{cycle}"),
                    result.revision,
                    ArtifactCommand::Submit {
                        candidate: candidate.clone(),
                    },
                ),
                ReferenceFault::None,
            )
            .await
            .unwrap()
            .revision;
    }
    let status = kernel::Kernel::new(broker.clone())
        .status(&owner)
        .await
        .unwrap();
    assert_eq!(status["missions"][0]["consumed_repairs"], 3);
    if let Ok(evidence) = std::env::var("TEMPORAL_FIXTURE_EVIDENCE") {
        std::fs::write(
            std::path::Path::new(&evidence).join("identical-report-cycles.json"),
            serde_json::to_vec(&serde_json::json!({"report_id":report_id,"status":status}))
                .unwrap(),
        )
        .unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
}
