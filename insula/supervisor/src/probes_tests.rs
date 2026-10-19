//! Exercises real kernel process identity, timeout cleanup, cancellation and
//! private tmux endpoints. Run the tmux case with INSULA_TEST_TMUX if tmux is not
//! installed at /usr/bin/tmux. A custom build can also use INSULA_TEST_TMUX_LIB.

use insula_supervisor::{
    model::*,
    probes::{read_process_identity, verify_process_identity, LocalProbes},
};
use std::{
    collections::BTreeMap,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tempfile::TempDir;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, UnixListener},
    process::{Child, Command},
    time::{sleep, timeout, Instant},
};
use uuid::Uuid;

fn me() -> ProcessIdentity {
    read_process_identity(std::process::id(), Uuid::new_v4(), Uuid::new_v4()).unwrap()
}
fn request(identity: &ProcessIdentity, probe: ProbeKind, timeout_ms: u64) -> ProbeRequest {
    ProbeRequest {
        key: ServiceKey {
            insula_id: "root".into(),
            service_id: "probe-test".into(),
        },
        generation: identity.generation,
        spec: ProbeSpec {
            probe,
            timeout_ms,
            ..ProbeSpec::default()
        },
    }
}
fn command(executable: &str, args: &[&str]) -> ProbeKind {
    ProbeKind::Command {
        command: CommandSpec {
            executable: executable.into(),
            args: args.iter().map(|x| (*x).into()).collect(),
            cwd: "/tmp".into(),
            env_refs: BTreeMap::new(),
            identity: RuntimeIdentity {
                uid: rustix::process::geteuid().as_raw(),
                gid: rustix::process::getegid().as_raw(),
            },
        },
    }
}
fn python() -> &'static str {
    ["/usr/bin/python3", "/usr/local/bin/python3"]
        .into_iter()
        .find(|x| Path::new(x).exists())
        .expect("python3 needed for direct-child probe tests")
}
async fn read_pid(path: &Path) -> u32 {
    timeout(Duration::from_secs(3), async {
        loop {
            if let Ok(text) = tokio::fs::read_to_string(path).await {
                if let Ok(pid) = text.parse() {
                    return pid;
                }
            }
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("probe must create PID marker")
}
async fn wait_reaped(pid: u32) {
    timeout(Duration::from_secs(3), async {
        while Path::new(&format!("/proc/{pid}")).exists() {
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("timed-out or cancelled probe child must be reaped")
}

#[tokio::test]
async fn process_identity_and_generation_are_checked_without_signalling() {
    let probes = LocalProbes::new(2).unwrap();
    let identity = me();
    let req = request(&identity, ProbeKind::Process, 200);
    assert_eq!(
        probes.probe(&identity, &req, BTreeMap::new()).await.class,
        ProbeClass::Success
    );
    let mut wrong = identity.clone();
    wrong.start_time_ticks += 1;
    assert!(!verify_process_identity(&wrong).unwrap());
    assert_eq!(
        probes.probe(&wrong, &req, BTreeMap::new()).await.class,
        ProbeClass::TargetUnavailable
    );
    wrong = identity.clone();
    wrong.boot_id = Uuid::new_v4().to_string();
    assert!(!verify_process_identity(&wrong).unwrap());
    let mut stale = req.clone();
    stale.generation = Uuid::new_v4();
    assert_eq!(
        probes.probe(&identity, &stale, BTreeMap::new()).await.code,
        "stale_generation"
    );
    assert!(verify_process_identity(&identity).unwrap());
}

#[tokio::test]
async fn command_arguments_are_literal_and_output_is_not_exposed() {
    let probes = LocalProbes::new(2).unwrap();
    let identity = me();
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("not-executed");
    let literal = format!("SECRET_SENTINEL_735 $(touch {})", marker.display());
    let req = request(
        &identity,
        command("/usr/bin/printf", &["%s", &literal]),
        2000,
    );
    let observation = probes.probe(&identity, &req, BTreeMap::new()).await;
    assert_eq!(observation.class, ProbeClass::Success);
    assert!(!marker.exists());
    assert!(!serde_json::to_string(&observation)
        .unwrap()
        .contains("SECRET_SENTINEL"));
    let req = request(&identity, command("/usr/bin/false", &[]), 2000);
    assert_eq!(
        probes.probe(&identity, &req, BTreeMap::new()).await.class,
        ProbeClass::ServiceFailure
    );
    let req = request(&identity, command("/does/not/exist", &[]), 2000);
    assert_eq!(
        probes.probe(&identity, &req, BTreeMap::new()).await.class,
        ProbeClass::MechanismFailure
    );
}

#[tokio::test]
async fn command_environment_is_resolved_explicitly_and_never_returned() {
    let probes = LocalProbes::new(2).unwrap();
    let identity = me();
    let mut kind = command(
        python(),
        &[
            "-c",
            "import os; print(os.environ['TOKEN']); assert 'HOME' not in os.environ",
        ],
    );
    if let ProbeKind::Command { command } = &mut kind {
        command
            .env_refs
            .insert("TOKEN".into(), "vault:secret".into());
    }
    let req = request(&identity, kind, 2000);
    assert_eq!(
        probes.probe(&identity, &req, BTreeMap::new()).await.code,
        "environment_reference_unresolved"
    );
    let observation = probes
        .probe(
            &identity,
            &req,
            BTreeMap::from([("TOKEN".into(), "PRIVATE_TOKEN_4981".into())]),
        )
        .await;
    assert_eq!(observation.class, ProbeClass::Success);
    assert!(!serde_json::to_string(&observation)
        .unwrap()
        .contains("PRIVATE_TOKEN"));
}

#[tokio::test]
async fn hung_command_times_out_is_reaped_and_releases_capacity() {
    let probes = LocalProbes::new(1).unwrap();
    let identity = me();
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("pid");
    let program = "import os,pathlib,sys,time; pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); time.sleep(60)";
    let req = request(
        &identity,
        command(python(), &["-c", program, marker.to_str().unwrap()]),
        250,
    );
    let start = Instant::now();
    let observation = probes.probe(&identity, &req, BTreeMap::new()).await;
    assert_eq!(observation.code, "probe_timeout");
    assert!(start.elapsed() < Duration::from_secs(2));
    wait_reaped(read_pid(&marker).await).await;
    let req = request(&identity, ProbeKind::Process, 200);
    assert_eq!(
        probes.probe(&identity, &req, BTreeMap::new()).await.class,
        ProbeClass::Success
    );
}

#[tokio::test]
async fn cancellation_kills_and_reaps_child_without_dropping_its_waiter() {
    let probes = LocalProbes::new(1).unwrap();
    let identity = me();
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("pid");
    let req = request(&identity, command(python(), &["-c", "import os,pathlib,sys,time; pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); time.sleep(60)", marker.to_str().unwrap()]), 60_000);
    let worker = probes.clone();
    let child_identity = identity.clone();
    let caller =
        tokio::spawn(async move { worker.probe(&child_identity, &req, BTreeMap::new()).await });
    let pid = read_pid(&marker).await;
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    wait_reaped(pid).await;
    assert_eq!(
        probes
            .probe(
                &identity,
                &request(&identity, ProbeKind::Process, 200),
                BTreeMap::new()
            )
            .await
            .class,
        ProbeClass::Success
    );
}

#[tokio::test]
async fn capacity_deadline_prevents_a_second_probe_spawn() {
    let probes = LocalProbes::new(1).unwrap();
    let identity = me();
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("first");
    let second = dir.path().join("second");
    let program = "import os,pathlib,sys,time; pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); time.sleep(60)";
    let req = request(
        &identity,
        command(python(), &["-c", program, marker.to_str().unwrap()]),
        60_000,
    );
    let worker = probes.clone();
    let first_identity = identity.clone();
    let caller =
        tokio::spawn(async move { worker.probe(&first_identity, &req, BTreeMap::new()).await });
    let pid = read_pid(&marker).await;
    let req = request(
        &identity,
        command(python(), &["-c", program, second.to_str().unwrap()]),
        30,
    );
    assert_eq!(
        probes.probe(&identity, &req, BTreeMap::new()).await.code,
        "probe_capacity_timeout"
    );
    assert!(!second.exists());
    caller.abort();
    let _ = caller.await;
    wait_reaped(pid).await;
}

#[tokio::test]
async fn combined_stdout_stderr_limit_stops_flood_and_redacts_output() {
    let probes = LocalProbes::new(1).unwrap();
    let identity = me();
    let req = request(
        &identity,
        command(
            python(),
            &[
                "-c",
                "import os; os.write(1,b'A'*5000); os.write(2,b'SECRET_SENTINEL'*5000)",
            ],
        ),
        2000,
    );
    let observation = probes.probe(&identity, &req, BTreeMap::new()).await;
    assert_eq!(observation.code, "probe_output_limit");
    assert!(!serde_json::to_string(&observation)
        .unwrap()
        .contains("SECRET_SENTINEL"));
}

#[tokio::test]
async fn http_status_is_bounded_and_an_unresponsive_endpoint_times_out() {
    let identity = me();
    let probes = LocalProbes::new(2).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let responder = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut bytes = [0; 1024];
        let count = stream.read(&mut bytes).await.unwrap();
        assert!(std::str::from_utf8(&bytes[..count])
            .unwrap()
            .starts_with("GET /ready HTTP/1.1\r\n"));
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\nX-Secret: DO_NOT_RETURN\r\n\r\n")
            .await
            .unwrap();
    });
    let kind = ProbeKind::Http {
        address: "127.0.0.1".into(),
        port,
        path: "/ready".into(),
        expected_status: 204,
    };
    let observation = probes
        .probe(&identity, &request(&identity, kind, 1000), BTreeMap::new())
        .await;
    assert_eq!(observation.class, ProbeClass::Success);
    assert!(!serde_json::to_string(&observation)
        .unwrap()
        .contains("DO_NOT_RETURN"));
    responder.await.unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let hung = tokio::spawn(async move {
        let (_stream, _) = listener.accept().await.unwrap();
        sleep(Duration::from_secs(60)).await;
    });
    let kind = ProbeKind::Http {
        address: "127.0.0.1".into(),
        port,
        path: "/".into(),
        expected_status: 200,
    };
    assert_eq!(
        probes
            .probe(&identity, &request(&identity, kind, 50), BTreeMap::new())
            .await
            .code,
        "probe_timeout"
    );
    hung.abort();
}

#[tokio::test]
async fn unix_socket_probe_checks_peer_process_identity() {
    let identity = me();
    let probes = LocalProbes::new(2).unwrap();
    let dir = TempDir::new().unwrap();
    let socket = dir.path().join("api.sock");
    let _listener = UnixListener::bind(&socket).unwrap();
    let req = request(
        &identity,
        ProbeKind::UnixSocket {
            path: socket.clone(),
        },
        1000,
    );
    assert_eq!(
        probes.probe(&identity, &req, BTreeMap::new()).await.class,
        ProbeClass::Success
    );
    let mut other = Command::new("/usr/bin/sleep")
        .arg("60")
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let other_identity =
        read_process_identity(other.id().unwrap(), identity.owner_instance, Uuid::new_v4())
            .unwrap();
    let req = request(
        &other_identity,
        ProbeKind::UnixSocket { path: socket },
        1000,
    );
    assert_eq!(
        probes
            .probe(&other_identity, &req, BTreeMap::new())
            .await
            .code,
        "socket_peer_identity_mismatch"
    );
    other.kill().await.unwrap();
}

struct Tmux {
    executable: PathBuf,
    env: BTreeMap<String, String>,
    socket: PathBuf,
    server: Child,
    _directory: TempDir,
}
impl Tmux {
    async fn start() -> Self {
        let executable = std::env::var_os("INSULA_TEST_TMUX")
            .map(PathBuf::from)
            .unwrap_or_else(|| "/usr/bin/tmux".into());
        assert!(
            executable.exists(),
            "real tmux acceptance requires tmux: install /usr/bin/tmux or set INSULA_TEST_TMUX (and INSULA_TEST_TMUX_LIB for a private build)"
        );
        let mut env = BTreeMap::from([
            ("TERM".into(), "xterm-256color".into()),
            ("PATH".into(), "/usr/bin:/bin".into()),
        ]);
        if let Ok(value) = std::env::var("INSULA_TEST_TMUX_LIB") {
            env.insert("LD_LIBRARY_PATH".into(), value);
        }
        let directory = tempfile::Builder::new()
            .prefix("tm-probe-")
            .tempdir_in("/tmp")
            .unwrap();
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket = directory.path().join("server.sock");
        let server = Command::new(&executable)
            .args(["-D", "-f", "/dev/null", "-S"])
            .arg(&socket)
            .env_clear()
            .envs(&env)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        timeout(Duration::from_secs(3), async {
            while !socket.exists() {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("foreground tmux server socket");
        Self {
            executable,
            env,
            socket,
            server,
            _directory: directory,
        }
    }
    fn client(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .args(["-N", "-f", "/dev/null", "-S"])
            .arg(&self.socket)
            .env_clear()
            .envs(&self.env)
            .kill_on_drop(true);
        command
    }
    async fn command(&self, args: &[&str]) -> String {
        let output = timeout(Duration::from_secs(3), self.client().args(args).output())
            .await
            .unwrap()
            .unwrap();
        assert!(output.status.success(), "tmux test client command failed");
        String::from_utf8(output.stdout).unwrap()
    }
    fn probe(
        &self,
        identity: &ProcessIdentity,
        expected_sessions: Vec<String>,
        expected_panes: Vec<String>,
    ) -> ProbeRequest {
        request(
            identity,
            ProbeKind::Tmux {
                executable: self.executable.clone(),
                socket: self.socket.clone(),
                expected_sessions,
                expected_panes,
            },
            3000,
        )
    }
    async fn attach_then_detach(&self) {
        let mut client = self
            .client()
            .args(["-C", "attach-session", "-t", "work"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        timeout(Duration::from_secs(3), async {
            loop {
                if !self
                    .command(&["list-clients", "-F", "#{client_control_mode}"])
                    .await
                    .trim()
                    .is_empty()
                {
                    break;
                }
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        client
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"detach-client\n")
            .await
            .unwrap();
        timeout(Duration::from_secs(3), client.wait())
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn real_tmux_identity_detach_reconnect_and_failed_pane_stays_server_healthy() {
    let mut tmux = Tmux::start().await;
    let identity =
        read_process_identity(tmux.server.id().unwrap(), Uuid::new_v4(), Uuid::new_v4()).unwrap();
    let probes = LocalProbes::new(2).unwrap();
    let empty = probes
        .probe(
            &identity,
            &tmux.probe(&identity, vec![], vec![]),
            tmux.env.clone(),
        )
        .await;
    assert_eq!(empty.class, ProbeClass::Success, "{empty:?}");
    assert_eq!(empty.expectations_met, Some(true));
    assert!(empty.panes.is_empty());
    let absent_socket = tmux.socket.with_extension("absent");
    let mut absent_request = tmux.probe(&identity, vec![], vec![]);
    if let ProbeKind::Tmux { socket, .. } = &mut absent_request.spec.probe {
        *socket = absent_socket.clone();
    }
    let absent = probes
        .probe(&identity, &absent_request, tmux.env.clone())
        .await;
    assert_eq!(absent.code, "endpoint_missing");
    assert!(!absent_socket.exists());
    assert!(verify_process_identity(&identity).unwrap());
    std::fs::set_permissions(&tmux.socket, std::fs::Permissions::from_mode(0o666)).unwrap();
    let exposed = probes
        .probe(
            &identity,
            &tmux.probe(&identity, vec![], vec![]),
            tmux.env.clone(),
        )
        .await;
    assert_eq!(exposed.class, ProbeClass::MechanismFailure);
    assert_eq!(exposed.code, "tmux_socket_not_private");
    std::fs::set_permissions(&tmux.socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    let missing = probes
        .probe(
            &identity,
            &tmux.probe(&identity, vec!["work".into()], vec![]),
            tmux.env.clone(),
        )
        .await;
    assert_eq!(missing.class, ProbeClass::Success);
    assert_eq!(missing.expectations_met, Some(false));
    // Work is launched by the client, independently from empty service bootstrap.
    tmux.command(&["new-session", "-d", "-s", "work", "/usr/bin/sleep", "60"])
        .await;
    let before = tmux
        .command(&["list-panes", "-a", "-F", "#{pane_id}:#{pane_pid}"])
        .await;
    tmux.attach_then_detach().await;
    tmux.attach_then_detach().await;
    assert_eq!(
        tmux.command(&["list-panes", "-a", "-F", "#{pane_id}:#{pane_pid}"])
            .await,
        before
    );
    tmux.command(&["set-option", "-g", "remain-on-exit", "on"])
        .await;
    tmux.command(&[
        "new-session",
        "-d",
        "-s",
        "failed",
        python(),
        "-c",
        "raise SystemExit(7)",
    ])
    .await;
    let failed_pane = timeout(Duration::from_secs(3), async {
        loop {
            let observed = probes
                .probe(
                    &identity,
                    &tmux.probe(&identity, vec!["work".into()], vec![]),
                    tmux.env.clone(),
                )
                .await;
            assert_eq!(observed.class, ProbeClass::Success, "{observed:?}");
            if let Some(pane) = observed
                .panes
                .into_iter()
                .find(|pane| pane.dead && pane.exit_code.is_some())
            {
                return pane;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(failed_pane.exit_code, Some(7));
    let observed = probes
        .probe(
            &identity,
            &tmux.probe(&identity, vec!["work".into()], vec![failed_pane.pane_id]),
            tmux.env.clone(),
        )
        .await;
    assert_eq!(observed.class, ProbeClass::Success);
    assert_eq!(observed.expectations_met, Some(false));
    assert!(verify_process_identity(&identity).unwrap());
    tmux.command(&["kill-server"]).await;
    timeout(Duration::from_secs(3), tmux.server.wait())
        .await
        .unwrap()
        .unwrap();
    // -N plus identity checks must not create a replacement empty server.
    let observed = probes
        .probe(
            &identity,
            &tmux.probe(&identity, vec![], vec![]),
            tmux.env.clone(),
        )
        .await;
    assert_eq!(observed.class, ProbeClass::ServiceFailure);
    assert!(!verify_process_identity(&identity).unwrap());
}
