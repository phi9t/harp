//! End-to-end tests: real CLI/daemon processes, an independently surviving owner,
//! actual tmux, SIGKILL, reconnect, and generation-fenced stop/restart requests.
#[path = "../fixture_owner.rs"]
mod fixture_owner;
use fixture_owner::{foreground_spec, FixtureOwner, FixtureRpcServer};
use insula_supervisor::{
    transport::{self, ControlAction, ControlReply},
    *,
};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tempfile::TempDir;
use tokio::process::{Child, Command};
use uuid::Uuid;

const BIN: &str = env!("CARGO_BIN_EXE_insula-supervisor");
fn daemon(server: &FixtureRpcServer, state: &Path) -> Child {
    Command::new(BIN)
        .args(["serve", "--root-id", "root", "--owner-socket"])
        .arg(server.socket())
        .arg("--state-dir")
        .arg(state)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap()
}
async fn call(socket: &Path, action: ControlAction) -> Result<ControlReply> {
    transport::request(
        socket,
        rustix::process::geteuid().as_raw(),
        action,
        Duration::from_secs(3),
    )
    .await
}
async fn wait_ready(child: &mut Child, socket: &Path) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(4);
    loop {
        assert!(
            child.try_wait().unwrap().is_none(),
            "daemon exited before control socket was ready"
        );
        if call(socket, ControlAction::Capabilities).await.is_ok() {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "daemon startup deadline"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
async fn status(socket: &Path, key: &ServiceKey) -> ServiceStatus {
    match call(socket, ControlAction::Inspect { key: key.clone() })
        .await
        .unwrap()
    {
        ControlReply::Status(s) => *s,
        other => panic!("unexpected reply {other:?}"),
    }
}
async fn mutate(socket: &Path, request: OperationRequest) -> Result<OperationRecord> {
    match call(socket, ControlAction::Mutate { request }).await? {
        ControlReply::Operation(o) => Ok(o),
        _ => Err(Error::Unavailable),
    }
}
fn start(spec: &ServiceSpec) -> OperationRequest {
    OperationRequest {
        operation_id: Uuid::new_v4(),
        mutation: Mutation::Start {
            spec: Box::new(spec.clone()),
        },
    }
}
fn stop(key: &ServiceKey, generation: Generation) -> OperationRequest {
    OperationRequest {
        operation_id: Uuid::new_v4(),
        mutation: Mutation::Stop {
            key: key.clone(),
            expected_generation: generation,
        },
    }
}
fn restart(key: &ServiceKey, generation: Generation) -> OperationRequest {
    OperationRequest {
        operation_id: Uuid::new_v4(),
        mutation: Mutation::Restart {
            key: key.clone(),
            expected_generation: generation,
        },
    }
}
fn tmux_path() -> PathBuf {
    let p = std::env::var_os("INSULA_TEST_TMUX")
        .map(PathBuf::from)
        .unwrap_or_else(|| "/usr/bin/tmux".into());
    assert!(
        p.is_file(),
        "install tmux or set INSULA_TEST_TMUX to run all acceptance tests"
    );
    p
}
fn tmux_client(executable: &Path, socket: &Path) -> Command {
    let mut c = Command::new(executable);
    c.args(["-N", "-S"]).arg(socket).kill_on_drop(true);
    if let Some(lib) = std::env::var_os("INSULA_TEST_TMUX_LIB") {
        c.env("LD_LIBRARY_PATH", lib);
    }
    c
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn real_daemon_excludes_competing_controller_adopts_tmux_and_never_replays_pane_work() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let server = FixtureRpcServer::start(owner.clone(), &temp.path().join("owner.sock")).unwrap();
    let state = temp.path().join("state");
    let socket = state.join("control.sock");
    let mut first = daemon(&server, &state);
    wait_ready(&mut first, &socket).await;
    let tmux = tmux_path();
    let tmux_socket = temp.path().join("tmux-private/server.sock");
    let mut spec = foreground_spec(tmux.to_str().unwrap(), &[]);
    spec.key.service_id = "terminal".into();
    spec.adapter = ServiceAdapter::Tmux {
        socket: tmux_socket.clone(),
    };
    if let Ok(lib) = std::env::var("INSULA_TEST_TMUX_LIB") {
        owner
            .set_environment_reference("test.tmux-libraries", &lib)
            .await;
        spec.command
            .env_refs
            .insert("LD_LIBRARY_PATH".into(), "test.tmux-libraries".into());
    }
    spec.readiness.probe = ProbeKind::Tmux {
        executable: tmux.clone(),
        socket: tmux_socket.clone(),
        expected_sessions: vec![],
        expected_panes: vec![],
    };
    spec.health.probe = spec.readiness.probe.clone();
    spec.readiness.interval_ms = 20;
    spec.health.interval_ms = 20;
    spec.stop.graceful = GracefulStop::Tmux {
        executable: tmux.clone(),
        socket: tmux_socket.clone(),
    };
    let specification = temp.path().join("tmux.json");
    std::fs::write(&specification, serde_json::to_vec(&spec).unwrap()).unwrap();
    let original = start(&spec);
    // Exercise the actual thin CLI, not only the in-process client library.
    let output = Command::new(BIN)
        .args(["start", "--socket"])
        .arg(&socket)
        .arg("--spec")
        .arg(&specification)
        .arg("--operation-id")
        .arg(original.operation_id.to_string())
        .output()
        .await
        .unwrap();
    assert!(
        output.status.success(),
        "CLI start failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: ControlReply = serde_json::from_slice(&output.stdout).unwrap();
    assert!(matches!(
        parsed,
        ControlReply::Operation(OperationRecord {
            outcome: OperationOutcome::Succeeded { .. },
            ..
        })
    ));
    let g1 = status(&socket, &spec.key).await.generation.unwrap();
    let mut duplicates = tokio::task::JoinSet::new();
    for _ in 0..8 {
        let socket = socket.clone();
        let request = original.clone();
        duplicates.spawn(async move { mutate(&socket, request).await.unwrap() });
    }
    while let Some(result) = duplicates.join_next().await {
        assert!(
            matches!(result.unwrap().outcome,OperationOutcome::Succeeded{generation:Some(g)} if g==g1)
        );
    }
    assert_eq!(owner.total_launches(), 1);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while !status(&socket, &spec.key).await.ready {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // Pane creation is an explicit user action after service launch, absent from
    // the declarative server bootstrap. The completed effect must stay at one.
    assert!(tmux_client(&tmux, &tmux_socket)
        .args(["set-option", "-g", "remain-on-exit", "on"])
        .status()
        .await
        .unwrap()
        .success());
    let counter = temp.path().join("completed-effects");
    let effect = format!("printf once >> '{}'; exit 7", counter.display());
    assert!(tmux_client(&tmux, &tmux_socket)
        .args(["new-session", "-d", "-s", "completed", "/bin/sh", "-c"])
        .arg(effect)
        .status()
        .await
        .unwrap()
        .success());
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        let observed = status(&socket, &spec.key).await;
        if observed
            .last_probe
            .as_ref()
            .is_some_and(|p| p.panes.iter().any(|p| p.dead && p.exit_code == Some(7)))
        {
            assert_eq!(observed.health, Health::Healthy);
            break;
        }
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let before = status(&socket, &spec.key).await;
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "once");
    // A second OS process with DIFFERENT state/socket paths cannot obtain control.
    let mut competitor = daemon(&server, &temp.path().join("different-state"));
    let exit = tokio::time::timeout(Duration::from_secs(3), competitor.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(!exit.success());
    assert_eq!(owner.total_launches(), 1);
    first.kill().await.unwrap();
    first.wait().await.unwrap();
    // EOF fencing is performed by the surviving owner, not by a test override.
    tokio::time::sleep(Duration::from_millis(80)).await;
    let mut recovered = daemon(&server, &state);
    wait_ready(&mut recovered, &socket).await;
    let after = status(&socket, &spec.key).await;
    assert_eq!(after.generation, Some(g1));
    assert_eq!(after.identity, before.identity);
    assert_eq!(owner.total_launches(), 1);
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "once");
    let panes = tmux_client(&tmux, &tmux_socket)
        .args(["list-panes", "-a", "-F", "#{pane_dead_status}"])
        .output()
        .await
        .unwrap();
    assert!(panes.status.success());
    assert_eq!(String::from_utf8_lossy(&panes.stdout).trim(), "7");
    let explicit_restart = mutate(&socket, restart(&spec.key, g1)).await.unwrap();
    assert!(
        matches!(explicit_restart.outcome, OperationOutcome::Succeeded { .. }),
        "{explicit_restart:?}"
    );
    let g2 = status(&socket, &spec.key).await.generation.unwrap();
    assert_ne!(g1, g2);
    assert_eq!(owner.total_launches(), 2);
    assert_eq!(
        mutate(&socket, stop(&spec.key, g1)).await.unwrap_err(),
        Error::StaleGeneration
    );
    assert_eq!(status(&socket, &spec.key).await.generation, Some(g2));
    assert_eq!(std::fs::read_to_string(&counter).unwrap(), "once");
    // The replacement is an empty fresh server; it did not recreate pane work.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        let sessions = tmux_client(&tmux, &tmux_socket)
            .args(["list-sessions"])
            .output()
            .await
            .unwrap();
        if sessions.status.success() {
            assert!(sessions.stdout.is_empty());
            break;
        }
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let historical = mutate(&socket, original).await.unwrap();
    assert!(matches!(historical.outcome,OperationOutcome::Succeeded{generation:Some(g)} if g==g1));
    assert_eq!(owner.total_launches(), 2);
    mutate(&socket, stop(&spec.key, g2)).await.unwrap();
    owner.shutdown_all().await.unwrap();
    recovered.kill().await.unwrap();
    recovered.wait().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sigkill_during_launch_fences_delayed_old_request_and_requires_new_explicit_operation() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let server = FixtureRpcServer::start(owner.clone(), &temp.path().join("owner.sock")).unwrap();
    let state = temp.path().join("state");
    let socket = state.join("control.sock");
    let mut first = daemon(&server, &state);
    wait_ready(&mut first, &socket).await;
    let spec = foreground_spec("/bin/sleep", &["30"]);
    let request = start(&spec);
    owner.delay_next_launch(Duration::from_millis(500));
    let request_copy = request.clone();
    let socket_copy = socket.clone();
    let waiting = tokio::spawn(async move { mutate(&socket_copy, request_copy).await });
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        if let Ok(ControlReply::Status(s)) = call(
            &socket,
            ControlAction::Inspect {
                key: spec.key.clone(),
            },
        )
        .await
        {
            if s.lifecycle == Lifecycle::Starting {
                break;
            }
        }
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    first.kill().await.unwrap();
    first.wait().await.unwrap();
    let _ = waiting.await;
    tokio::time::sleep(Duration::from_millis(80)).await;
    let mut recovered = daemon(&server, &state);
    wait_ready(&mut recovered, &socket).await;
    tokio::time::sleep(Duration::from_millis(550)).await;
    assert_eq!(
        owner.total_launches(),
        0,
        "old owner request materialized after epoch revocation"
    );
    let old = mutate(&socket, request).await.unwrap();
    assert!(matches!(old.outcome, OperationOutcome::Unknown));
    let next = mutate(&socket, start(&spec)).await.unwrap();
    assert!(matches!(next.outcome, OperationOutcome::Succeeded { .. }));
    assert_eq!(owner.total_launches(), 1);
    let generation = status(&socket, &spec.key).await.generation.unwrap();
    mutate(&socket, stop(&spec.key, generation)).await.unwrap();
    owner.shutdown_all().await.unwrap();
    recovered.kill().await.unwrap();
    recovered.wait().await.unwrap();
}

#[tokio::test]
async fn graceful_deadline_forces_only_the_requested_real_process_and_reaps_it() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let supervisor = Supervisor::open(owner.clone(), "root".into(), temp.path().join("state"))
        .await
        .unwrap();
    let mut stubborn = foreground_spec("/bin/sh", &["-c", "trap '' TERM; exec /bin/sleep 30"]);
    stubborn.stop.deadline_ms = 80;
    let mut sentinel = foreground_spec("/bin/sleep", &["30"]);
    sentinel.key.service_id = "unrelated".into();
    supervisor.execute(start(&stubborn)).await.unwrap();
    supervisor.execute(start(&sentinel)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(60)).await;
    let target = supervisor.inspect(&stubborn.key).unwrap();
    let untouched = supervisor.inspect(&sentinel.key).unwrap();
    let result = supervisor
        .execute(stop(&stubborn.key, target.generation.unwrap()))
        .await
        .unwrap();
    assert!(matches!(result.outcome, OperationOutcome::Succeeded { .. }));
    let stopped = supervisor.inspect(&stubborn.key).unwrap();
    assert_eq!(stopped.lifecycle, Lifecycle::Exited);
    assert_eq!(stopped.last_exit.unwrap().signal, Some(9));
    assert!(!std::path::Path::new(&format!("/proc/{}", target.identity.unwrap().pid)).exists());
    supervisor.tick().await.unwrap();
    let alive = supervisor.inspect(&sentinel.key).unwrap();
    assert_eq!(alive.identity, untouched.identity);
    assert_eq!(alive.lifecycle, Lifecycle::Running);
    supervisor
        .execute(stop(&sentinel.key, alive.generation.unwrap()))
        .await
        .unwrap();
    supervisor.shutdown().await.unwrap();
    owner.shutdown_all().await.unwrap();
}
