use std::{path::PathBuf, process::Command as OsCommand};
use team_control_plane::{supervisor::*, *};
use uuid::Uuid;
fn git(root: &std::path::Path, args: &[&str]) -> String {
    let out = OsCommand::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().into()
}
struct Fixture {
    service: Supervisor,
    owner: String,
    attempt: Uuid,
    root: PathBuf,
    dsn: String,
    epoch: Uuid,
}
async fn fixture() -> Fixture {
    fixture_with_scope(&["src"], None).await
}
async fn fixture_with_scope(scopes: &[&str], link: Option<&std::path::Path>) -> Fixture {
    let dsn =
        std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").expect("real PostgreSQL runner required");
    let schema = format!("supervisor_{}", Uuid::new_v4().simple());
    let (c, conn) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { conn.await.unwrap() });
    c.batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{dsn} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let root = std::env::temp_dir().join(format!("supervisor-fixture-{}", Uuid::new_v4()));
    std::fs::create_dir_all(root.join("store/src")).unwrap();
    std::fs::write(root.join("store/src/input"), "base").unwrap();
    std::fs::write(root.join("store/src/existing"), "existing").unwrap();
    if let Some(target) = link {
        std::fs::remove_dir_all(root.join("store/src")).unwrap();
        std::os::unix::fs::symlink(target, root.join("store/src")).unwrap();
    }
    git(&root.join("store"), &["init", "-q"]);
    git(&root.join("store"), &["add", "."]);
    git(
        &root.join("store"),
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
    let base = git(&root.join("store"), &["rev-parse", "HEAD"]);
    let mission = Uuid::new_v4();
    let task = Uuid::new_v4();
    let input = GitInput {
        repository: root.join("store").to_str().unwrap().into(),
        commit: base,
        path: "src/input".into(),
        digest: team_control_plane::artifacts::sha256(b"base"),
    };
    let env = |key: &str, rev, command| CommandEnvelope {
        schema_version: 1,
        key: key.into(),
        expected_revision: rev,
        expected_generation: None,
        recovery_epoch: epoch,
        command,
    };
    broker
        .execute(
            &owner,
            env(
                "mission",
                0,
                Command::CreateMission {
                    id: mission,
                    objective: "supervise".into(),
                    input: input.clone(),
                    policy: Policy {
                        capacity: 1,
                        max_attempts: 5,
                        max_episodes: 5,
                        max_repairs: 1,
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
            env(
                "task",
                0,
                Command::CreateTask {
                    id: task,
                    mission_id: mission,
                    spec: TaskSpec {
                        objective: "supervise".into(),
                        input,
                        acceptance: vec!["stop descendants".into()],
                        deliverables: vec!["output".into()],
                        write_scope: scopes.iter().map(|s| s.to_string()).collect(),
                        dependencies: vec![],
                    },
                },
            ),
        )
        .await
        .unwrap();
    let snap = broker
        .snapshot(&owner, SnapshotQuery::Task(task))
        .await
        .unwrap();
    let digest = snap.data["spec_digest"].as_str().unwrap().into();
    broker
        .execute(
            &owner,
            env(
                "admit",
                1,
                Command::AdmitTask {
                    task_id: task,
                    spec_digest: digest,
                },
            ),
        )
        .await
        .unwrap();
    let worker = Uuid::new_v4();
    broker
        .execute(
            &owner,
            env(
                "credential",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(mission),
                    task_id: Some(task),
                    generation: Some(1),
                    token_hash: token_hash("fixture-worker"),
                    expires_in_seconds: 600,
                },
            ),
        )
        .await
        .unwrap();
    broker
        .execute(
            &owner,
            env(
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
    let (c, conn) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { conn.await.unwrap() });
    let attempt = c
        .query_one("SELECT id FROM attempts WHERE task_id=$1", &[&task])
        .await
        .unwrap()
        .get(0);
    let service = Supervisor::open(
        broker,
        &root.join("workspaces"),
        env!("CARGO_BIN_EXE_team-control-plane"),
    )
    .await
    .unwrap();
    Fixture {
        service,
        owner,
        attempt,
        root,
        dsn,
        epoch,
    }
}
#[tokio::test]
async fn registered_before_create_and_unknown_gap_is_quarantined() {
    let f = fixture().await;
    let handle = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::AfterRegister)
        .await
        .unwrap();
    assert!(!handle.path.exists());
    let view = f.service.inspect(&f.owner, handle.id).await.unwrap();
    assert_eq!(view.state, ExecutionState::Registered);
    f.service.reconcile(&f.owner, handle.id).await.unwrap();
    assert_eq!(
        f.service.inspect(&f.owner, handle.id).await.unwrap().state,
        ExecutionState::Stopped
    );
    f.service.release(&f.owner, handle.id).await.unwrap();
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn cancel_stops_nested_setsid_children_and_fences_shared_git() {
    let f = fixture().await;
    let handle = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::None)
        .await
        .unwrap();
    f.service.start(&f.owner,handle.id,vec!["/bin/sh".into(),"-c".into(),"touch /workspace/.git/FORBIDDEN 2>/dev/null && exit 99; echo fixture-execution >&2; setsid sh -c 'sleep 120 & wait' & echo ready > /workspace/src/ready; wait".into()],30).await.unwrap();
    for _ in 0..100 {
        if handle.path.join("src/ready").exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        handle.path.join("src/ready").exists(),
        "real capsule workload did not start"
    );
    assert!(!f.root.join("store/.git/FORBIDDEN").exists());
    for _ in 0..100 {
        if f.service
            .inspect(&f.owner, handle.id)
            .await
            .unwrap()
            .members
            .len()
            >= 3
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let running = f.service.inspect(&f.owner, handle.id).await.unwrap();
    assert!(
        running.members.len() >= 3,
        "nested membership must be inventoried: {running:?}"
    );
    f.service.cancel(&f.owner, handle.id).await.unwrap();
    for _ in 0..100 {
        if f.service.inspect(&f.owner, handle.id).await.unwrap().state == ExecutionState::Stopped {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let stopped = f.service.inspect(&f.owner, handle.id).await.unwrap();
    assert_eq!(stopped.state, ExecutionState::Stopped);
    assert!(stopped.members.is_empty());
    assert!(
        f.service
            .start(&f.owner, handle.id, vec!["/bin/true".into()], 5)
            .await
            .is_err()
    );
    assert!(
        String::from_utf8(f.service.logs(&f.owner, handle.id).await.unwrap())
            .unwrap()
            .contains("fixture-execution")
    );
    let history = f.service.history(&f.owner, handle.id).await.unwrap();
    assert!(history.iter().any(|v| v.members.len() >= 3));
    f.service.release(&f.owner, handle.id).await.unwrap();
    assert!(
        String::from_utf8(f.service.logs(&f.owner, handle.id).await.unwrap())
            .unwrap()
            .contains("fixture-execution")
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
async fn wait_stopped(f: &Fixture, id: Uuid) -> ExecutionView {
    for _ in 0..150 {
        let observed = f.service.inspect(&f.owner, id).await.unwrap();
        if observed.state == ExecutionState::Starting {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            continue;
        }
        let v = f.service.reconcile(&f.owner, id).await.unwrap();
        if v.state == ExecutionState::Stopped {
            return v;
        }
        assert_ne!(
            v.state,
            ExecutionState::Quarantined,
            "expected complete namespace proof: {v:?}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("workload did not stop")
}
async fn wait_members(f: &Fixture, id: Uuid) -> ExecutionView {
    for _ in 0..100 {
        let v = f.service.inspect(&f.owner, id).await.unwrap();
        if v.members.len() >= 3 {
            return v;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("nested workload was not inventoried")
}
fn nested() -> Vec<String> {
    vec![
        "/bin/sh".into(),
        "-c".into(),
        "echo fixture-execution >&2; setsid sh -c 'sleep 120 & wait' & echo ready > /workspace/src/ready; wait".into(),
    ]
}
#[tokio::test]
async fn deadline_and_controller_sigkill_use_complete_namespace_stop() {
    for kill in [false, true] {
        let f = fixture().await;
        let h = f
            .service
            .create_attempt(&f.owner, f.attempt, CreateFault::None)
            .await
            .unwrap();
        f.service
            .start(&f.owner, h.id, nested(), if kill { 30 } else { 1 })
            .await
            .unwrap();
        let running = wait_members(&f, h.id).await;
        if kill {
            let p = running.controller.unwrap();
            assert!(
                OsCommand::new("kill")
                    .args(["-KILL", &p.pid.to_string()])
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let stopped = wait_stopped(&f, h.id).await;
        assert!(stopped.members.is_empty());
        for member in running.members {
            assert!(
                !std::path::Path::new(&format!("/proc/{}/ns/pid", member.pid)).exists(),
                "member survived PID namespace shutdown"
            );
        }
        f.service.release(&f.owner, h.id).await.unwrap();
        std::fs::remove_dir_all(f.root).unwrap();
    }
}
#[tokio::test]
async fn registered_create_crash_and_gc_launch_race_never_reuse_workspace() {
    let f = fixture().await;
    let h = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::AfterCreate)
        .await
        .unwrap();
    assert!(h.path.exists());
    assert_eq!(
        f.service.inspect(&f.owner, h.id).await.unwrap().state,
        ExecutionState::Registered
    );
    assert!(f.service.start(&f.owner, h.id, nested(), 5).await.is_err());
    f.service.reconcile(&f.owner, h.id).await.unwrap();
    let (release, start) = tokio::join!(
        f.service.release(&f.owner, h.id),
        f.service.start(&f.owner, h.id, nested(), 5)
    );
    assert!(release.is_ok());
    assert!(start.is_err());
    assert!(!h.path.exists());
    assert_eq!(
        f.service.inspect(&f.owner, h.id).await.unwrap().state,
        ExecutionState::Released
    );
    assert!(
        f.service
            .create_attempt(&f.owner, f.attempt, CreateFault::None)
            .await
            .is_err()
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn crash_create_helper() {
    let Ok(mode) = std::env::var("SUPERVISOR_CRASH_MODE") else {
        return;
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let dsn = std::env::var("SUPERVISOR_FIXTURE_DSN").unwrap();
        let epoch = Uuid::parse_str(&std::env::var("SUPERVISOR_FIXTURE_EPOCH").unwrap()).unwrap();
        let attempt =
            Uuid::parse_str(&std::env::var("SUPERVISOR_FIXTURE_ATTEMPT").unwrap()).unwrap();
        let root = PathBuf::from(std::env::var("SUPERVISOR_FIXTURE_ROOT").unwrap());
        let owner = std::env::var("SUPERVISOR_FIXTURE_OWNER").unwrap();
        let service = Supervisor::open(
            Broker::connect(&dsn, epoch).await.unwrap(),
            &root.join("workspaces"),
            env!("CARGO_BIN_EXE_team-control-plane"),
        )
        .await
        .unwrap();
        if mode == "git" {
            let _ = service
                .create_attempt(&owner, attempt, CreateFault::DuringGit)
                .await;
            panic!("blocked Git unexpectedly returned");
        }
        let h = service
            .create_attempt(
                &owner,
                attempt,
                if mode == "register" {
                    CreateFault::AfterRegister
                } else if mode == "launch" {
                    CreateFault::None
                } else {
                    CreateFault::AfterCreate
                },
            )
            .await
            .unwrap();
        if mode == "launch" {
            service.start(&owner, h.id, nested(), 30).await.unwrap();
            for _ in 0..100 {
                if service.inspect(&owner, h.id).await.unwrap().members.len() >= 3 {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            assert!(service.inspect(&owner, h.id).await.unwrap().members.len() >= 3);
        }
        let file = std::fs::File::create(root.join("crash-handle.json")).unwrap();
        serde_json::to_writer(&file, &h).unwrap();
        file.sync_all().unwrap();
        // Real process death at the boundary, not an in-memory error return.
        unsafe {
            libc::kill(libc::getpid(), libc::SIGKILL);
        }
    });
    panic!("SIGKILL did not terminate helper")
}
#[tokio::test]
async fn actual_create_sigkill_boundaries_leave_durable_reconcilable_registration() {
    use std::os::unix::process::ExitStatusExt;
    for mode in ["register", "create"] {
        let f = fixture().await;
        let status = OsCommand::new(std::env::current_exe().unwrap())
            .args(["--exact", "crash_create_helper", "--nocapture"])
            .env("SUPERVISOR_CRASH_MODE", mode)
            .env("SUPERVISOR_FIXTURE_DSN", &f.dsn)
            .env("SUPERVISOR_FIXTURE_EPOCH", f.epoch.to_string())
            .env("SUPERVISOR_FIXTURE_ATTEMPT", f.attempt.to_string())
            .env("SUPERVISOR_FIXTURE_ROOT", &f.root)
            .env("SUPERVISOR_FIXTURE_OWNER", &f.owner)
            .status()
            .unwrap();
        assert_eq!(status.signal(), Some(libc::SIGKILL));
        let h: ExecutionHandle =
            serde_json::from_slice(&std::fs::read(f.root.join("crash-handle.json")).unwrap())
                .unwrap();
        assert_eq!(h.path.exists(), mode == "create");
        assert_eq!(
            f.service.inspect(&f.owner, h.id).await.unwrap().state,
            ExecutionState::Registered
        );
        assert!(f.service.start(&f.owner, h.id, nested(), 5).await.is_err());
        assert_eq!(
            f.service.reconcile(&f.owner, h.id).await.unwrap().state,
            ExecutionState::Stopped
        );
        f.service.release(&f.owner, h.id).await.unwrap();
        std::fs::remove_dir_all(f.root).unwrap();
    }
}
#[tokio::test]
async fn uncertain_namespace_quarantines_capacity_and_current_credentials_are_required() {
    let f = fixture().await;
    let h = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::None)
        .await
        .unwrap();
    assert_eq!(
        f.service
            .inspect("not-a-credential", h.id)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    let (c, conn) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { conn.await.unwrap() });
    // Simulate an interrupted starting reservation before identity was durably captured.
    c.execute(
        "UPDATE supervised_workspaces SET state='starting' WHERE id=$1",
        &[&h.id],
    )
    .await
    .unwrap();
    assert_eq!(
        f.service.reconcile(&f.owner, h.id).await.unwrap().state,
        ExecutionState::Quarantined
    );
    assert!(f.service.release(&f.owner, h.id).await.is_err());
    assert!(f.service.start(&f.owner, h.id, nested(), 1).await.is_err());
    assert_eq!(
        c.query_one("SELECT used_capacity FROM missions", &[])
            .await
            .unwrap()
            .get::<_, i64>(0),
        1
    );
    c.execute("UPDATE credentials SET expires_at=clock_timestamp()-interval '1 second' WHERE token_hash=$1",&[&token_hash(&f.owner)]).await.unwrap();
    assert_eq!(
        f.service.inspect(&f.owner, h.id).await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    // Quarantine intentionally retains its workspace. No workload was launched in this negative fixture.
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn stale_generation_fences_dispatch_and_cancels_existing_namespace() {
    let f = fixture().await;
    let h = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::None)
        .await
        .unwrap();
    let manifest = f.service.inspect(&f.owner, h.id).await.unwrap().manifest;
    assert_eq!(git(&h.path, &["rev-parse", "HEAD"]), manifest.base);
    assert_eq!(manifest.store, f.root.join("store"));
    assert_eq!(manifest.attempt, Some(f.attempt));
    assert_eq!(manifest.write_scope, vec!["src"]);
    f.service
        .start("fixture-worker", h.id, nested(), 30)
        .await
        .unwrap();
    let running = wait_members(&f, h.id).await;
    let (c, conn) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { conn.await.unwrap() });
    c.execute(
        "UPDATE tasks SET generation=generation+1 WHERE id=$1",
        &[&manifest.task],
    )
    .await
    .unwrap();
    assert_eq!(
        f.service
            .inspect("fixture-worker", h.id)
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    assert_eq!(
        f.service
            .start(&f.owner, h.id, nested(), 5)
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    // Current Owner performs recovery through public APIs; stale Worker remains fenced.
    f.service.inspect(&f.owner, h.id).await.unwrap();
    f.service.cancel(&f.owner, h.id).await.unwrap();
    let stopped = wait_stopped(&f, h.id).await;
    assert!(stopped.members.is_empty());
    for p in running.members {
        assert!(!std::path::Path::new(&format!("/proc/{}/ns/pid", p.pid)).exists());
    }
    f.service.release(&f.owner, h.id).await.unwrap();
    assert!(!f.root.join("store/src/ready").exists());
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn initiating_agent_sigkill_leaves_independent_inspect_and_cancel_handle() {
    use std::os::unix::process::ExitStatusExt;
    let f = fixture().await;
    let status = OsCommand::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_create_helper", "--nocapture"])
        .env("SUPERVISOR_CRASH_MODE", "launch")
        .env("SUPERVISOR_FIXTURE_DSN", &f.dsn)
        .env("SUPERVISOR_FIXTURE_EPOCH", f.epoch.to_string())
        .env("SUPERVISOR_FIXTURE_ATTEMPT", f.attempt.to_string())
        .env("SUPERVISOR_FIXTURE_ROOT", &f.root)
        .env("SUPERVISOR_FIXTURE_OWNER", &f.owner)
        .status()
        .unwrap();
    assert_eq!(status.signal(), Some(libc::SIGKILL));
    let h: ExecutionHandle =
        serde_json::from_slice(&std::fs::read(f.root.join("crash-handle.json")).unwrap()).unwrap();
    assert_eq!(
        f.service.inspect(&f.owner, h.id).await.unwrap().state,
        ExecutionState::Running
    );
    assert!(wait_members(&f, h.id).await.members.len() >= 3);
    f.service.cancel(&f.owner, h.id).await.unwrap();
    assert!(wait_stopped(&f, h.id).await.members.is_empty());
    f.service.release(&f.owner, h.id).await.unwrap();
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn actual_bwrap_launcher_sigkill_stops_all_namespace_members() {
    let f = fixture().await;
    let h = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::None)
        .await
        .unwrap();
    f.service.start(&f.owner, h.id, nested(), 30).await.unwrap();
    let running = wait_members(&f, h.id).await;
    let launcher = running.launcher.unwrap();
    assert!(
        OsCommand::new("kill")
            .args(["-KILL", &launcher.pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert!(wait_stopped(&f, h.id).await.members.is_empty());
    for p in running.members {
        assert!(!std::path::Path::new(&format!("/proc/{}/ns/pid", p.pid)).exists());
    }
    f.service.release(&f.owner, h.id).await.unwrap();
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn repair_worker_principal_must_own_attempt_for_create_and_control() {
    let f = fixture().await;
    let broker = Broker::connect(&f.dsn, f.epoch).await.unwrap();
    let (c, conn) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { conn.await.unwrap() });
    let r = c
        .query_one(
            "SELECT t.id,t.mission_id FROM attempts a JOIN tasks t ON t.id=a.task_id WHERE a.id=$1",
            &[&f.attempt],
        )
        .await
        .unwrap();
    broker
        .execute(
            &f.owner,
            CommandEnvelope {
                schema_version: 1,
                key: "wrong-worker".into(),
                expected_revision: 0,
                expected_generation: None,
                recovery_epoch: f.epoch,
                command: Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: Uuid::new_v4(),
                    role: Role::Worker,
                    mission_id: Some(r.get(1)),
                    task_id: Some(r.get(0)),
                    generation: Some(1),
                    token_hash: token_hash("wrong-worker"),
                    expires_in_seconds: 600,
                },
            },
        )
        .await
        .unwrap();
    assert_eq!(
        f.service
            .create_attempt("wrong-worker", f.attempt, CreateFault::None)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    let h = f
        .service
        .create_attempt("fixture-worker", f.attempt, CreateFault::None)
        .await
        .unwrap();
    for result in [
        f.service.start("wrong-worker", h.id, nested(), 1).await,
        f.service.cancel("wrong-worker", h.id).await,
        f.service.release("wrong-worker", h.id).await,
    ] {
        assert_eq!(result.unwrap_err().code, ErrorCode::Unauthorized);
    }
    assert_eq!(
        f.service
            .inspect("wrong-worker", h.id)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    f.service.cancel(&f.owner, h.id).await.unwrap();
    f.service.release(&f.owner, h.id).await.unwrap();
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn repair_symlink_ancestor_rejected_before_any_external_write() {
    let outside = std::env::temp_dir().join(format!("supervisor-sentinel-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("sentinel"), "unchanged").unwrap();
    let f = fixture_with_scope(&["src/newdir"], Some(&outside)).await;
    assert!(
        f.service
            .create_attempt_with_scopes(
                &f.owner,
                f.attempt,
                vec![WriteScope {
                    path: "src/newdir".into(),
                    kind: ScopeKind::Directory
                }],
                CreateFault::None
            )
            .await
            .is_err()
    );
    assert!(
        !outside.join("newdir").exists(),
        "tracked ancestor symlink caused an external filesystem mutation"
    );
    assert_eq!(
        std::fs::read(outside.join("sentinel")).unwrap(),
        b"unchanged"
    );
    std::fs::remove_dir_all(f.root).unwrap();
    std::fs::remove_dir_all(outside).unwrap();
}
#[tokio::test]
async fn repair_existing_and_explicit_missing_files_are_exact_capsule_write_scope() {
    file_scope_probe().await
}
async fn file_scope_probe() {
    let f = fixture_with_scope(
        &[
            "src/existing",
            "src/missing",
            "new-parent/new-file",
            "empty.file",
        ],
        None,
    )
    .await;
    let scopes = vec![
        WriteScope {
            path: "src/existing".into(),
            kind: ScopeKind::File,
        },
        WriteScope {
            path: "src/missing".into(),
            kind: ScopeKind::File,
        },
        WriteScope {
            path: "new-parent/new-file".into(),
            kind: ScopeKind::File,
        },
        WriteScope {
            path: "empty.file".into(),
            kind: ScopeKind::Directory,
        },
    ];
    assert_eq!(
        f.service
            .create_attempt(&f.owner, f.attempt, CreateFault::None)
            .await
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        f.service
            .create_attempt_with_scopes(
                &f.owner,
                f.attempt,
                vec![WriteScope {
                    path: "src".into(),
                    kind: ScopeKind::Directory
                }],
                CreateFault::None
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        f.service
            .create_attempt_with_scopes(
                &f.owner,
                f.attempt,
                vec![WriteScope {
                    path: "src/existing".into(),
                    kind: ScopeKind::Directory
                }],
                CreateFault::None
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let h = f
        .service
        .create_attempt_with_scopes(&f.owner, f.attempt, scopes, CreateFault::None)
        .await
        .unwrap();
    assert!(h.path.join("src/missing").is_file());
    assert!(h.path.join("new-parent/new-file").is_file());
    f.service.start(&f.owner,h.id,vec!["/bin/sh".into(),"-c".into(),"echo allowed > src/existing; echo allowed > src/missing; echo allowed > new-parent/new-file; echo allowed > empty.file/output; if touch src/input new-parent/neighbor 2>/dev/null; then exit 99; fi; echo file-proof".into()],5).await.unwrap();
    assert!(wait_stopped(&f, h.id).await.members.is_empty());
    for path in ["src/existing", "src/missing", "new-parent/new-file"] {
        assert_eq!(
            std::fs::read_to_string(h.path.join(path)).unwrap(),
            "allowed\n"
        );
    }
    assert_eq!(
        std::fs::read_to_string(h.path.join("src/input")).unwrap(),
        "base"
    );
    assert!(!h.path.join("new-parent/neighbor").exists());
    assert_eq!(
        std::fs::read_to_string(h.path.join("empty.file/output")).unwrap(),
        "allowed\n"
    );
    assert!(
        String::from_utf8(f.service.logs(&f.owner, h.id).await.unwrap())
            .unwrap()
            .contains("file-proof")
    );
    f.service.release(&f.owner, h.id).await.unwrap();
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn repair_schema_guard_precedes_open_and_monitor_effects() {
    for missing in [false, true] {
        let f = fixture().await;
        let h = f
            .service
            .create_attempt(&f.owner, f.attempt, CreateFault::None)
            .await
            .unwrap();
        let (c, conn) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move { conn.await.unwrap() });
        if missing {
            c.batch_execute("DROP TABLE supervisor_schema_version")
                .await
                .unwrap();
        } else {
            c.batch_execute("ALTER TABLE supervisor_schema_version DROP CONSTRAINT supervisor_schema_version_version_check; UPDATE supervisor_schema_version SET version=2").await.unwrap();
        }
        let newroot = f.root.join("must-not-create");
        let broker = Broker::connect(&f.dsn, f.epoch).await.unwrap();
        assert_eq!(
            Supervisor::open(
                broker.clone(),
                &newroot,
                env!("CARGO_BIN_EXE_team-control-plane")
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::IncompatibleSchema
        );
        assert!(!newroot.exists());
        c.execute("UPDATE supervised_workspaces SET state='starting',argv=$2,deadline=clock_timestamp()+interval '10 seconds' WHERE id=$1",&[&h.id,&serde_json::json!(["/bin/sh","-c","touch /workspace/src/ready"])]).await.unwrap();
        assert_eq!(
            run_monitor(broker, h.id).await.unwrap_err().code,
            ErrorCode::IncompatibleSchema
        );
        assert!(!h.path.join("src/ready").exists());
        assert_eq!(
            std::fs::read_dir(f.root.join("workspaces/.execution-logs"))
                .unwrap()
                .count(),
            0
        );
        std::fs::remove_dir_all(f.root).unwrap();
    }
}
#[tokio::test]
async fn repair_launch_rechecks_epoch_generation_deadline_and_cancel_after_claim() {
    for change in ["generation", "epoch", "deadline", "cancel"] {
        let f = fixture().await;
        let h = f
            .service
            .create_attempt(&f.owner, f.attempt, CreateFault::None)
            .await
            .unwrap();
        f.service
            .start_with_fault(
                &f.owner,
                h.id,
                vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "touch /workspace/src/ready".into(),
                ],
                if change == "deadline" { 1 } else { 20 },
                StartFault::BeforeMonitor,
            )
            .await
            .unwrap();
        let claimed = std::sync::Arc::new(tokio::sync::Notify::new());
        let resume = std::sync::Arc::new(tokio::sync::Notify::new());
        let broker = Broker::connect(&f.dsn, f.epoch).await.unwrap();
        let gate = MonitorGate {
            claimed: claimed.clone(),
            resume: resume.clone(),
        };
        let monitor = tokio::spawn(run_monitor_with_gate(broker.clone(), h.id, gate));
        tokio::time::timeout(std::time::Duration::from_secs(3), claimed.notified())
            .await
            .unwrap();
        match change {
            "generation" => {
                let (c, conn) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
                    .await
                    .unwrap();
                tokio::spawn(async move { conn.await.unwrap() });
                c.execute("UPDATE tasks SET generation=generation+1", &[])
                    .await
                    .unwrap();
            }
            "epoch" => {
                broker.rotate_epoch(&f.owner, Uuid::new_v4()).await.unwrap();
            }
            "deadline" => {
                tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
            }
            "cancel" => {
                f.service.cancel(&f.owner, h.id).await.unwrap();
            }
            _ => unreachable!(),
        }
        resume.notify_one();
        assert!(
            monitor.await.unwrap().is_err(),
            "{change} after durable controller claim must prevent launch, not cancel a newly started workload"
        );
        assert!(
            !h.path.join("src/ready").exists(),
            "{change} produced external workload effect"
        );
        assert_eq!(
            std::fs::read_dir(f.root.join("workspaces/.execution-logs"))
                .unwrap()
                .count(),
            0,
            "{change} produced launch filesystem effects"
        );
        std::fs::remove_dir_all(f.root).unwrap();
    }
}
#[tokio::test]
async fn repair_current_owner_recovers_superseded_execution_and_refunds_only_proven_stop() {
    let f = fixture().await;
    let h = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::None)
        .await
        .unwrap();
    f.service
        .start("fixture-worker", h.id, nested(), 30)
        .await
        .unwrap();
    wait_members(&f, h.id).await;
    let (c, conn) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { conn.await.unwrap() });
    c.execute("UPDATE tasks SET generation=generation+1", &[])
        .await
        .unwrap();
    assert_eq!(
        f.service
            .inspect("fixture-worker", h.id)
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    f.service.inspect(&f.owner, h.id).await.unwrap();
    f.service.cancel(&f.owner, h.id).await.unwrap();
    assert!(wait_stopped(&f, h.id).await.members.is_empty());
    assert!(f.service.start(&f.owner, h.id, nested(), 5).await.is_err());
    f.service.release(&f.owner, h.id).await.unwrap();
    assert!(!h.path.exists());
    assert_eq!(
        f.service.inspect(&f.owner, h.id).await.unwrap().state,
        ExecutionState::Released
    );
    assert_eq!(
        c.query_one("SELECT used_capacity FROM missions", &[])
            .await
            .unwrap()
            .get::<_, i64>(0),
        0
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn repair_actual_in_progress_git_survivor_never_proves_stop_or_refunds() {
    use std::os::unix::process::ExitStatusExt;
    let f = fixture().await;
    let mut helper = OsCommand::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_create_helper", "--nocapture"])
        .env("SUPERVISOR_CRASH_MODE", "git")
        .env("SUPERVISOR_FIXTURE_DSN", &f.dsn)
        .env("SUPERVISOR_FIXTURE_EPOCH", f.epoch.to_string())
        .env("SUPERVISOR_FIXTURE_ATTEMPT", f.attempt.to_string())
        .env("SUPERVISOR_FIXTURE_ROOT", &f.root)
        .env("SUPERVISOR_FIXTURE_OWNER", &f.owner)
        .spawn()
        .unwrap();
    let mut record = None;
    for _ in 0..100 {
        for entry in std::fs::read_dir(f.root.join("workspaces/.execution-logs")).unwrap() {
            let p = entry.unwrap().path();
            if p.extension().is_some_and(|x| x == "prepare-identity")
                && let Ok(bytes) = std::fs::read(p)
                && let Ok(pair) =
                    serde_json::from_slice::<(ExecutionHandle, ProcessIdentity)>(&bytes)
            {
                record = Some(pair);
            }
        }
        if record.is_some() {
            break;
        }
        assert!(
            helper.try_wait().unwrap().is_none(),
            "unexpected preparation helper exit"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let (h, git_process) = record.expect("actual Git spawn marker required");
    assert_eq!(
        std::fs::read_link(format!("/proc/{}/exe", git_process.pid)).unwrap(),
        std::path::Path::new("/usr/bin/git")
    );
    helper.kill().unwrap();
    assert_eq!(helper.wait().unwrap().signal(), Some(libc::SIGKILL));
    assert!(
        std::path::Path::new(&format!("/proc/{}/exe", git_process.pid)).exists(),
        "Git did not actually survive initiating agent death"
    );
    let recovery = f.service.reconcile(&f.owner, h.id).await.unwrap();
    // Cleanup the deliberately owned blocked real Git process before any assertion can fail.
    assert!(
        OsCommand::new("kill")
            .args(["-KILL", &git_process.pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    for _ in 0..100 {
        if !std::path::Path::new(&format!("/proc/{}/exe", git_process.pid)).exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(!std::path::Path::new(&format!("/proc/{}/exe", git_process.pid)).exists());
    assert!(
        recovery
            .quarantine_reason
            .as_ref()
            .unwrap()
            .contains("Git preparation")
    );
    assert_eq!(
        recovery.state,
        ExecutionState::Quarantined,
        "un-inventoried live Git preparation must never count as stopped"
    );
    assert!(f.service.release(&f.owner, h.id).await.is_err());
    assert!(f.service.start(&f.owner, h.id, nested(), 5).await.is_err());
    let (c, conn) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { conn.await.unwrap() });
    assert_eq!(
        c.query_one("SELECT used_capacity FROM missions", &[])
            .await
            .unwrap()
            .get::<_, i64>(0),
        1
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
#[tokio::test]
async fn repair_current_owner_recovers_execution_after_recovery_epoch_rotation() {
    let f = fixture().await;
    let h = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::None)
        .await
        .unwrap();
    f.service
        .start("fixture-worker", h.id, nested(), 30)
        .await
        .unwrap();
    wait_members(&f, h.id).await;
    let broker = Broker::connect(&f.dsn, f.epoch).await.unwrap();
    let newepoch = Uuid::new_v4();
    let owner = broker.rotate_epoch(&f.owner, newepoch).await.unwrap();
    let service = Supervisor::open(
        Broker::connect(&f.dsn, newepoch).await.unwrap(),
        &f.root.join("workspaces"),
        env!("CARGO_BIN_EXE_team-control-plane"),
    )
    .await
    .unwrap();
    assert_eq!(
        service
            .inspect("fixture-worker", h.id)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    service.inspect(&owner, h.id).await.unwrap();
    service.cancel(&owner, h.id).await.unwrap();
    let mut stopped = false;
    for _ in 0..150 {
        let v = service.reconcile(&owner, h.id).await.unwrap();
        if v.state == ExecutionState::Stopped {
            stopped = true;
            break;
        }
        assert_ne!(v.state, ExecutionState::Quarantined);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(stopped);
    assert!(service.start(&owner, h.id, nested(), 5).await.is_err());
    service.release(&owner, h.id).await.unwrap();
    assert!(!h.path.exists());
    std::fs::remove_dir_all(f.root).unwrap();
}

#[tokio::test]
async fn concurrent_short_commands_treat_disappeared_proc_tasks_as_exited() {
    for _ in 0..4 {
        let mut probes = vec![];
        for _ in 0..8 {
            probes.push(tokio::spawn(file_scope_probe()));
        }
        for probe in probes {
            probe.await.unwrap();
        }
    }
}

#[tokio::test]
async fn certified_pid1_whole_group_exit_recovers_without_namespace_inode_signals() {
    use std::os::fd::FromRawFd;
    let f = fixture().await;
    let h = f
        .service
        .create_attempt(&f.owner, f.attempt, CreateFault::None)
        .await
        .unwrap();
    f.service.start(&f.owner, h.id, vec!["/bin/sh".into(), "-c".into(),
        "setsid sh -c 'while :; do echo x >> /workspace/src/heartbeat; sleep 0.02; done' & sleep 120 & wait".into()], 30).await.unwrap();
    let running = wait_members(&f, h.id).await;
    let init = running
        .members
        .iter()
        .find(|p| p.namespace_pid == Some(1))
        .unwrap();
    let status = std::fs::read_to_string(format!("/proc/{}/status", init.pid)).unwrap();
    assert_eq!(
        status
            .lines()
            .find_map(|l| l.strip_prefix("NSpid:"))
            .unwrap()
            .split_whitespace()
            .last(),
        Some("1")
    );
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", init.pid)).unwrap();
    assert_eq!(
        stat.rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(19)
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        init.start_ticks
    );
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, init.pid, 0) } as i32;
    assert!(raw >= 0);
    let _pidfd = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) };
    let controller = running.controller.unwrap();
    assert!(
        OsCommand::new("kill")
            .args(["-KILL", &controller.pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    let mut poll = libc::pollfd {
        fd: raw,
        events: libc::POLLIN,
        revents: 0,
    };
    for _ in 0..100 {
        assert!(unsafe { libc::poll(&mut poll, 1, 0) } >= 0);
        if poll.revents & libc::POLLIN != 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_ne!(
        poll.revents & libc::POLLIN,
        0,
        "whole-process PID1 did not finalize"
    );
    let stopped = wait_stopped(&f, h.id).await;
    assert!(stopped.members.is_empty());
    let heartbeat = std::fs::read(h.path.join("src/heartbeat")).unwrap();
    assert!(!heartbeat.is_empty());
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;
    assert_eq!(
        std::fs::read(h.path.join("src/heartbeat")).unwrap(),
        heartbeat
    );
    for _ in 0..100 {
        if !std::path::Path::new(&format!("/proc/{}/stat", init.pid)).exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(
        !std::path::Path::new(&format!("/proc/{}/stat", init.pid)).exists(),
        "original PID1 was not reaped"
    );
    f.service.release(&f.owner, h.id).await.unwrap();
    std::fs::remove_dir_all(f.root).unwrap();
}
