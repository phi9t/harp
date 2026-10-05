use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Daemon {
    child: Child,
    socket: PathBuf,
}
impl Daemon {
    fn spawn(db: &Path, socket: &Path, barrier: Option<&Path>) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"));
        command
            .args([db, socket])
            .env("CORENIUS_OPERATOR_TOKEN", "operator-secret")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(path) = barrier {
            command.env("CORENIUS_TEST_ADMISSION_BARRIER", path);
        }
        let child = command.spawn().unwrap();
        let mut daemon = Self {
            child,
            socket: socket.into(),
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if UnixStream::connect(socket).is_ok() {
                break;
            }
            assert!(
                daemon.child.try_wait().unwrap().is_none(),
                "daemon exited before serving public seam"
            );
            assert!(Instant::now() < deadline, "daemon startup deadline");
            std::thread::sleep(Duration::from_millis(10));
        }
        println!(
            "daemon_ready pid={} socket={}",
            daemon.child.id(),
            socket.display()
        );
        daemon
    }
    fn kill(&mut self) {
        let pid = self.child.id();
        self.child.kill().unwrap();
        let status = self.child.wait().unwrap();
        println!("owned_daemon_killed pid={pid} status={status}");
    }
    fn request(&self, request: Value) -> Value {
        let mut stream = UnixStream::connect(&self.socket).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        writeln!(stream, "{request}").unwrap();
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert!(response.get("error").is_none(), "{response}");
        response["ok"].clone()
    }
    fn command(&self, kind: &str, revision: u64, key: &str, args: Value) -> Value {
        self.request(command(kind, revision, key, args))
    }
    fn snapshot(&self) -> Value {
        self.request(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":"snapshot","args":{}}}))
    }
}
impl Drop for Daemon {
    fn drop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
fn command(kind: &str, revision: u64, key: &str, args: Value) -> Value {
    json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":kind,"expected_revision":revision,"idempotency_key":key,"args":args}})
}
fn initialize(d: &Daemon) {
    d.command(
        "create_project",
        0,
        "project",
        json!({"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}),
    );
    d.command("create_task",1,"task",json!({"task":"a","work_ref":{"deployment":"local","project":"sandbox","item":"a"},"source_revision":"r1","brief":"brief"}));
}

#[test]
fn real_process_restart_retains_operations_and_ownership() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let socket = dir.path().join("api.sock");
    let mut daemon = Daemon::spawn(&db, &socket, None);
    initialize(&daemon);
    let a = daemon.command("admit", 2, "admit", json!({"task":"a"}));
    daemon.command(
        "ask_question",
        3,
        "question",
        json!({"task":"a","text":"recover me"}),
    );
    daemon.command(
        "record_decision",
        4,
        "decision",
        json!({"text":"retain evidence"}),
    );
    daemon.command("observe_attempt",5,"unknown",json!({"attempt":a["result"]["attempt"],"state":"unknown","observation":"unconfirmed start"}));
    let mut competitor_child = Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .args([&db, &dir.path().join("competitor.sock")])
        .env("CORENIUS_OPERATOR_TOKEN", "operator-secret")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let competitor_pid = competitor_child.id();
    let deadline = Instant::now() + Duration::from_secs(5);
    while competitor_child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            competitor_child.kill().unwrap();
            competitor_child.wait().unwrap();
            panic!("competing controller did not refuse ownership");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let competitor = competitor_child.wait_with_output().unwrap();
    println!(
        "competing_controller pid={competitor_pid} status={} reason={}",
        competitor.status,
        String::from_utf8_lossy(&competitor.stderr).trim()
    );
    assert!(!competitor.status.success());
    assert!(String::from_utf8_lossy(&competitor.stderr).contains("ownership"));
    daemon.kill();
    let daemon = Daemon::spawn(&db, &socket, None);
    let state = daemon.snapshot();
    assert_eq!(state["revision"], 6);
    assert_eq!(state["attempts"][0]["state"], "unknown");
    assert_eq!(state["occupied_slots"], 1);
    assert_eq!(state["operations"][0]["state"], "prepared");
    assert_eq!(state["questions"][0]["text"], "recover me");
    assert_eq!(state["decisions"][0]["text"], "retain evidence");
    println!(
        "restart_observed revision={} occupied_slots={} attempt_state={} pending_operation_state={}",
        state["revision"],
        state["occupied_slots"],
        state["attempts"][0]["state"],
        state["operations"][0]["state"]
    );
    assert_eq!(a, daemon.command("admit", 2, "admit", json!({"task":"a"})));
}

#[cfg(feature = "fault-injection")]
#[test]
fn kill_during_atomic_admission_exposes_none_then_retry_exposes_all() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let socket = dir.path().join("api.sock");
    let marker = dir.path().join("before-commit");
    let mut daemon = Daemon::spawn(&db, &socket, Some(&marker));
    initialize(&daemon);
    let mut stream = UnixStream::connect(&socket).unwrap();
    writeln!(
        stream,
        "{}",
        command("admit", 2, "admit", json!({"task":"a"}))
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !marker.exists() {
        assert!(Instant::now() < deadline, "admission barrier never reached");
        std::thread::sleep(Duration::from_millis(10));
    }
    println!(
        "admission_barrier_observed pid={} marker={}",
        daemon.child.id(),
        marker.display()
    );
    daemon.kill();
    drop(stream);
    let daemon = Daemon::spawn(&db, &socket, None);
    let before = daemon.snapshot();
    assert_eq!(before["revision"], 2);
    assert_eq!(before["occupied_slots"], 0);
    assert_eq!(before["tasks"][0]["state"], "created");
    assert!(before["attempts"].as_array().unwrap().is_empty());
    assert!(before["operations"].as_array().unwrap().is_empty());
    let events=daemon.request(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":"events","args":{"after":2}}}));
    assert!(events.as_array().unwrap().is_empty());
    println!(
        "rollback_observed revision={} occupied_slots={} attempts={} operations={}",
        before["revision"],
        before["occupied_slots"],
        before["attempts"].as_array().unwrap().len(),
        before["operations"].as_array().unwrap().len()
    );
    let receipt = daemon.command("admit", 2, "admit", json!({"task":"a"}));
    let after = daemon.snapshot();
    assert_eq!(after["revision"], 3);
    assert_eq!(after["occupied_slots"], 1);
    assert_eq!(after["tasks"][0]["state"], "active");
    assert_eq!(after["attempts"].as_array().unwrap().len(), 1);
    assert_eq!(after["operations"][0]["id"], receipt["result"]["operation"]);
}

#[test]
fn slow_drip_client_cannot_extend_the_whole_request_deadline() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let socket = dir.path().join("api.sock");
    let daemon = Daemon::spawn(&db, &socket, None);
    initialize(&daemon);
    let mut slow = UnixStream::connect(&socket).unwrap();
    slow.write_all(b" ").unwrap();
    let drip = std::thread::spawn(move || {
        let until = Instant::now() + Duration::from_millis(2300);
        while Instant::now() < until {
            std::thread::sleep(Duration::from_millis(150));
            if slow.write_all(b" ").is_err() {
                break;
            }
        }
    });
    let begin = Instant::now();
    let snapshot = daemon.snapshot();
    let elapsed = begin.elapsed();
    drip.join().unwrap();
    println!(
        "slow_client_queued_request_elapsed_ms={}",
        elapsed.as_millis()
    );
    assert_eq!(snapshot["revision"], 2);
    assert!(
        elapsed < Duration::from_millis(1800),
        "drip feed extended whole-request deadline: {elapsed:?}"
    );
}

#[test]
fn existing_database_aliases_share_process_ownership() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.db");
    let alias = dir.path().join("alias.db");
    let socket = dir.path().join("api.sock");
    let mut daemon = Daemon::spawn(&target, &socket, None);
    initialize(&daemon);
    symlink(&target, &alias).unwrap();
    for path in [&target, &alias] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
            .args([path, &dir.path().join("competitor.sock")])
            .env("CORENIUS_OPERATOR_TOKEN", "operator-secret")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = child.id();
        let deadline = Instant::now() + Duration::from_secs(5);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("alias competitor did not refuse ownership");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let result = child.wait_with_output().unwrap();
        println!(
            "alias_competitor pid={pid} path={} status={} reason={}",
            path.display(),
            result.status,
            String::from_utf8_lossy(&result.stderr).trim()
        );
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("ownership"));
    }
    daemon.kill();
    let daemon = Daemon::spawn(&alias, &socket, None);
    assert_eq!(daemon.snapshot()["revision"], 2);
}
