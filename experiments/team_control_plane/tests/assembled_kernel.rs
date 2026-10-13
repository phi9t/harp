use team_control_plane::*;
use uuid::Uuid;
async fn cli(
    dsn: &str,
    epoch: Uuid,
    token: &str,
    args: &[&str],
    workspace: Option<&std::path::Path>,
) -> std::process::Output {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_team-control-plane"));
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(args)
        .env("TEAM_CONTROL_PLANE_DSN", dsn)
        .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", epoch.to_string())
        .env("TEAM_CONTROL_PLANE_TOKEN", token);
    if let Some(root) = workspace {
        command.env("TEAM_CONTROL_PLANE_WORKSPACE_ROOT", root);
    }
    tokio::task::spawn_blocking(move || command.output().expect("owned CLI subprocess"))
        .await
        .unwrap()
}

#[tokio::test]
async fn authenticated_operator_status_is_a_public_cli_view_without_execution_ownership() {
    let base = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").expect("real PostgreSQL required");
    let (admin, connection) = tokio_postgres::connect(&base, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let schema = format!("assembled_cli_{}", Uuid::new_v4().simple());
    admin
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{base} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let output = cli(&dsn, epoch, &owner, &["status"], None).await;
    assert!(
        output.status.success(),
        "authenticated status must work without starting/owning an execution: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let status: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(status["recovery_epoch"], serde_json::json!(epoch));
    assert!(status["missions"].as_array().unwrap().is_empty());
    let denied = cli(&dsn, epoch, "wrong-token", &["status"], None).await;
    assert!(!denied.status.success());
    assert!(denied.stdout.is_empty());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("unauthorized"));
    admin
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}

fn private_git(root: &std::path::Path, arguments: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["-c", "core.hooksPath=/dev/null"])
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "private fixture Git: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
#[tokio::test]
async fn lead_episode_allocates_a_real_workspace_without_synthetic_task_or_worker_authority() {
    let base = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").expect("real PostgreSQL required");
    let (admin, connection) = tokio_postgres::connect(&base, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let schema = format!("assembled_episode_{}", Uuid::new_v4().simple());
    admin
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{base} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let root = std::env::temp_dir().join(format!("kernel-episode-{}", Uuid::new_v4()));
    let store = root.join("store");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::write(store.join("SPEC.md"), b"immutable planning input").unwrap();
    private_git(&store, &["init", "-q"]);
    private_git(&store, &["add", "."]);
    private_git(
        &store,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "base",
        ],
    );
    let base_commit = private_git(&store, &["rev-parse", "HEAD"]);
    let mission = Uuid::new_v4();
    let principal = Uuid::new_v4();
    let episode = Uuid::new_v4();
    let packet = Uuid::new_v4();
    let envelope = |key: &str, command| CommandEnvelope {
        schema_version: 1,
        key: key.into(),
        expected_revision: 0,
        expected_generation: None,
        recovery_epoch: epoch,
        command,
    };
    broker
        .execute(
            &owner,
            envelope(
                "mission",
                Command::CreateMission {
                    id: mission,
                    objective: "real planning episode".into(),
                    input: GitInput {
                        repository: store.to_str().unwrap().into(),
                        commit: base_commit.clone(),
                        path: "SPEC.md".into(),
                        digest: artifacts::sha256(b"immutable planning input"),
                    },
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
                "lead",
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: principal,
                    role: Role::Lead,
                    mission_id: Some(mission),
                    task_id: None,
                    generation: None,
                    token_hash: token_hash("planning-fixture"),
                    expires_in_seconds: 120,
                },
            ),
        )
        .await
        .unwrap();
    temporal::Bridge::new(broker.clone())
        .register(&owner, temporal::AggregateKind::Mission, mission, 0)
        .await
        .unwrap();
    let mailbox = mailbox::MailboxService::new(broker.clone());
    let mail = |key: &str, command| {
        serde_json::from_value(serde_json::json!({"schema_version":1,"key":key,"recovery_epoch":epoch,"command":command})).unwrap()
    };
    mailbox.execute("planning-fixture",mail("capture",serde_json::json!({"kind":"capture_input","mission":mission,"id":packet,"attempt":null,"purpose":"planning","limit":4}))).await.unwrap();
    let captured = mailbox
        .record("planning-fixture", mailbox::RecordKind::Input, packet)
        .await
        .unwrap();
    mailbox.execute("planning-fixture",mail("admit",serde_json::json!({"kind":"admit_episode","mission":mission,"id":episode,"attempt":null,"purpose":"planning","input_packet":packet,"input_digest":captured["digest"]}))).await.unwrap();
    let denied_root = root.join("denied-workspaces");
    let denied = cli(
        &dsn,
        epoch,
        "wrong-token",
        &["allocate-episode", &episode.to_string()],
        Some(&denied_root),
    )
    .await;
    assert!(!denied.status.success());
    assert!(
        !denied_root.exists(),
        "unauthenticated allocation must not create any filesystem path"
    );
    let output = cli(
        &dsn,
        epoch,
        "planning-fixture",
        &["allocate-episode", &episode.to_string()],
        Some(&root.join("workspaces")),
    )
    .await;
    assert!(
        output.status.success(),
        "admitted Lead episode must allocate without a fictitious Task/Attempt or elevated tool grant: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let handle: supervisor::ExecutionHandle = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        private_git(&handle.path, &["rev-parse", "HEAD"]),
        base_commit
    );
    assert_eq!(
        std::fs::read(handle.path.join("SPEC.md")).unwrap(),
        b"immutable planning input"
    );
    let (db, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    assert_eq!(
        db.query_one("SELECT count(*) FROM tasks", &[])
            .await
            .unwrap()
            .get::<_, i64>(0),
        0
    );
    assert_eq!(
        db.query_one("SELECT count(*) FROM attempts", &[])
            .await
            .unwrap()
            .get::<_, i64>(0),
        0
    );
    assert_eq!(
        db.query_one(
            "SELECT used_capacity FROM missions WHERE id=$1",
            &[&mission]
        )
        .await
        .unwrap()
        .get::<_, i64>(0),
        1,
        "physical allocation must not charge admitted episode twice"
    );
    assert_eq!(
        db.query_one(
            "SELECT role FROM credentials WHERE principal_id=$1",
            &[&principal]
        )
        .await
        .unwrap()
        .get::<_, String>(0),
        "lead"
    );
    let service = supervisor::Supervisor::open(
        broker.clone(),
        &root.join("workspaces"),
        env!("CARGO_BIN_EXE_team-control-plane"),
    )
    .await
    .unwrap();
    let pause = cli(
        &dsn,
        epoch,
        &owner,
        &[
            "pause",
            &mission.to_string(),
            "pause-fixture",
            "0",
            "true",
            "Pause owned fixture before workload",
        ],
        None,
    )
    .await;
    assert!(
        pause.status.success(),
        "public authenticated pause: {}",
        String::from_utf8_lossy(&pause.stderr)
    );
    assert!(
        service
            .start("planning-fixture", handle.id, vec!["/bin/true".into()], 30)
            .await
            .is_err(),
        "durable mission pause must fence physical start after allocation"
    );
    let resume = cli(
        &dsn,
        epoch,
        &owner,
        &[
            "pause",
            &mission.to_string(),
            "resume-fixture",
            "1",
            "false",
            "Resume owned fixture after inspection",
        ],
        None,
    )
    .await;
    assert!(
        resume.status.success(),
        "public authenticated resume: {}",
        String::from_utf8_lossy(&resume.stderr)
    );
    service.start("planning-fixture",handle.id,vec!["/bin/sh".into(),"-c".into(),"cat SPEC.md; if echo forbidden > SPEC.md; then exit 9; fi; echo capsule_scope_held".into()],30).await.unwrap();
    let timeout = std::time::Instant::now() + std::time::Duration::from_secs(8);
    loop {
        let observed = service
            .inspect("planning-fixture", handle.id)
            .await
            .unwrap();
        if observed.state == supervisor::ExecutionState::Starting {
            assert!(
                std::time::Instant::now() < timeout,
                "trusted monitor must finish durable startup"
            );
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            continue;
        }
        let view = service
            .reconcile("planning-fixture", handle.id)
            .await
            .unwrap();
        if view.state == supervisor::ExecutionState::Stopped {
            break;
        }
        assert_ne!(
            view.state,
            supervisor::ExecutionState::Quarantined,
            "precise quarantine: {:?}",
            view.quarantine_reason
        );
        assert!(
            std::time::Instant::now() < timeout,
            "real planning capsule must stop"
        );
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let logs = service.logs("planning-fixture", handle.id).await.unwrap();
    assert!(String::from_utf8_lossy(&logs).contains("capsule_scope_held"));
    assert_eq!(
        std::fs::read(handle.path.join("SPEC.md")).unwrap(),
        b"immutable planning input"
    );
    mailbox.execute("planning-fixture",mail("yield-owned",serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"Yield planning while physical allocation remains reserved","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    assert_eq!(
        db.query_one(
            "SELECT used_capacity FROM missions WHERE id=$1",
            &[&mission]
        )
        .await
        .unwrap()
        .get::<_, i64>(0),
        1,
        "a physical ownership binding prevents unknown/allocated capacity refund on conversational yield"
    );
    let service = supervisor::Supervisor::open(
        broker,
        &root.join("workspaces"),
        env!("CARGO_BIN_EXE_team-control-plane"),
    )
    .await
    .unwrap();
    service.cancel("planning-fixture", handle.id).await.unwrap();
    service
        .release("planning-fixture", handle.id)
        .await
        .unwrap();
    assert_eq!(
        db.query_one(
            "SELECT used_capacity FROM missions WHERE id=$1",
            &[&mission]
        )
        .await
        .unwrap()
        .get::<_, i64>(0),
        0
    );
    assert_eq!(
        db.query_one(
            "SELECT admitted_episodes FROM missions WHERE id=$1",
            &[&mission]
        )
        .await
        .unwrap()
        .get::<_, i64>(0),
        1
    );
    mailbox.execute("planning-fixture",mail("new-input",serde_json::json!({"kind":"send","mission":mission,"id":Uuid::new_v4(),"recipients":[principal],"message_kind":"notice","text":"Independent startup-boundary observation","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let next_packet = Uuid::new_v4();
    let next_episode = Uuid::new_v4();
    mailbox.execute("planning-fixture",mail("capture-next",serde_json::json!({"kind":"capture_input","mission":mission,"id":next_packet,"attempt":null,"purpose":"planning","limit":4}))).await.unwrap();
    let next_input = mailbox
        .record("planning-fixture", mailbox::RecordKind::Input, next_packet)
        .await
        .unwrap();
    mailbox.execute("planning-fixture",mail("admit-next",serde_json::json!({"kind":"admit_episode","mission":mission,"id":next_episode,"attempt":null,"purpose":"planning","input_packet":next_packet,"input_digest":next_input["digest"]}))).await.unwrap();
    let next = service
        .create_episode(
            "planning-fixture",
            next_episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    service
        .start_with_fault(
            "planning-fixture",
            next.id,
            vec!["/bin/true".into()],
            30,
            supervisor::StartFault::BeforeMonitor,
        )
        .await
        .unwrap();
    for _ in 0..3 {
        assert_eq!(
            service
                .inspect("planning-fixture", next.id)
                .await
                .unwrap()
                .state,
            supervisor::ExecutionState::Starting,
            "view does not invent identity or stop"
        );
    }
    let unknown = service
        .reconcile("planning-fixture", next.id)
        .await
        .unwrap();
    assert_eq!(unknown.state, supervisor::ExecutionState::Quarantined);
    assert_eq!(
        unknown.quarantine_reason.as_deref(),
        Some("launch identity and full membership are unknown")
    );
    assert!(service.release("planning-fixture", next.id).await.is_err());
    assert_eq!(
        db.query_one(
            "SELECT used_capacity FROM missions WHERE id=$1",
            &[&mission]
        )
        .await
        .unwrap()
        .get::<_, i64>(0),
        1,
        "unknown startup holds slot"
    );
    // This explicitly injected pre-monitor boundary launched no process; fixture-owner cleanup
    // removes only its private files after asserting public release remains forbidden.

    std::fs::remove_dir_all(root).unwrap();
    admin
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}

#[path = "support/assembled.rs"]
#[allow(dead_code)]
mod assembled_support;
#[tokio::test]
async fn operator_cli_inspects_cancels_reconciles_and_releases_independently_owned_execution() {
    let f = assembled_support::Fixture::new().await;
    let episode = f.worker_episode("operator-owned-view").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let id = execution.id.to_string();
    let root = f.root.join("workspaces");
    let inspect = cli(
        &f.dsn,
        f.epoch,
        &f.owner,
        &["inspect-execution", &id],
        Some(&root),
    )
    .await;
    // Preserve cleanup even when the missing public command is the red outcome.
    if !inspect.status.success() {
        supervisor.cancel(&f.owner, execution.id).await.unwrap();
        supervisor.reconcile(&f.owner, execution.id).await.unwrap();
        supervisor.release(&f.owner, execution.id).await.unwrap();
        f.finish().await;
        panic!(
            "independent authenticated operator inspect is required: {}",
            String::from_utf8_lossy(&inspect.stderr)
        );
    }
    let view: serde_json::Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(view["state"], "ready");
    let status = cli(&f.dsn, f.epoch, &f.owner, &["status"], None).await;
    assert!(status.status.success());
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    let observed = status["executions"].as_array().is_some_and(|rows| {
        rows.iter().any(|r| {
            r["id"] == id && r["observed_state"] == "ready" && r["episode"] == episode.to_string()
        })
    });
    for command in [
        "cancel-execution",
        "reconcile-execution",
        "release-execution",
    ] {
        let output = cli(&f.dsn, f.epoch, &f.owner, &[command, &id], Some(&root)).await;
        assert!(
            output.status.success(),
            "same authenticated public broker {command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data["used_capacity"],
        0
    );
    let denied = cli(
        &f.dsn,
        f.epoch,
        "invalid-token",
        &["inspect-execution", &id],
        Some(&root),
    )
    .await;
    assert!(!denied.status.success());
    assert!(denied.stdout.is_empty());
    f.finish().await;
    assert!(
        observed,
        "ownership-independent status must expose bounded observed physical state and canonical episode"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn operator_cli_cancels_running_capsule_with_nested_background_children() {
    let f = assembled_support::Fixture::new().await;
    let episode = f.worker_episode("operator-running-stop").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let id = execution.id.to_string();
    let root = f.root.join("workspaces");
    let argv=serde_json::json!(["/bin/sh","-c","setsid /bin/sh -c 'while :; do printf x >> src/background; sleep .02; done' & while :; do printf y >> src/foreground; sleep .02; done"]).to_string();
    let started = cli(
        &f.dsn,
        f.epoch,
        "assembled-worker",
        &["start-execution", &id, "30", &argv],
        Some(&root),
    )
    .await;
    assert!(
        started.status.success(),
        "{}",
        String::from_utf8_lossy(&started.stderr)
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
    loop {
        let view = supervisor.inspect(&f.owner, execution.id).await.unwrap();
        assert_ne!(
            view.state,
            supervisor::ExecutionState::Quarantined,
            "{:?}",
            view.quarantine_reason
        );
        if view.state == supervisor::ExecutionState::Running
            && execution.path.join("src/background").exists()
            && execution.path.join("src/foreground").exists()
        {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let cancelled = cli(
        &f.dsn,
        f.epoch,
        &f.owner,
        &["cancel-execution", &id],
        Some(&root),
    )
    .await;
    assert!(
        cancelled.status.success(),
        "{}",
        String::from_utf8_lossy(&cancelled.stderr)
    );
    loop {
        let result = cli(
            &f.dsn,
            f.epoch,
            &f.owner,
            &["reconcile-execution", &id],
            Some(&root),
        )
        .await;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let view: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_ne!(view["state"], "quarantined", "{view}");
        if view["state"] == "stopped" {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let lengths = (
        std::fs::metadata(execution.path.join("src/background"))
            .unwrap()
            .len(),
        std::fs::metadata(execution.path.join("src/foreground"))
            .unwrap()
            .len(),
    );
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;
    assert_eq!(
        lengths,
        (
            std::fs::metadata(execution.path.join("src/background"))
                .unwrap()
                .len(),
            std::fs::metadata(execution.path.join("src/foreground"))
                .unwrap()
                .len()
        ),
        "proven namespace shutdown stops both actual writers"
    );
    let released = cli(
        &f.dsn,
        f.epoch,
        &f.owner,
        &["release-execution", &id],
        Some(&root),
    )
    .await;
    assert!(
        released.status.success(),
        "{}",
        String::from_utf8_lossy(&released.stderr)
    );
    assert_eq!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data["used_capacity"],
        0
    );
    f.finish().await;
}
