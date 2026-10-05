use corenius_sqlite_controller::client::call;
use serde_json::{Value, json};
use std::io::Write;
use std::process::Command;

struct Service(std::process::Child);

#[test]
fn repair2_accumulated_questions_keep_dashboard_mcp_and_pages_usable() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/client_projection.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
fn actual_dashboard_mcp_and_saved_control_replay_share_controller_state() {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("controller.sock");
    let server = Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .args([
            root.path().join("state.db").to_str().unwrap(),
            socket.to_str().unwrap(),
        ])
        .env_clear()
        .env("CORENIUS_OPERATOR_TOKEN", "client-test-secret")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut owner = Service(server);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !socket.exists() {
        assert!(owner.0.try_wait().unwrap().is_none());
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let auth = json!({"principal":"operator","token":"client-test-secret"});
    let path = socket.to_str().unwrap();
    let mut revision = 0;
    for (kind, args) in [
        (
            "create_project",
            json!({"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}),
        ),
        (
            "set_mission",
            json!({"objective":"persistent client fixture","plan":"two tasks","constraints":"receipt evidence","intent":"saved-spec"}),
        ),
    ] {
        let receipt=call(path,&json!({"auth":auth,"project":"p","command":{"kind":kind,"expected_revision":revision,"idempotency_key":kind,"args":args}})).unwrap();
        revision = receipt["revision"].as_u64().unwrap();
    }
    let saved = json!({"kind":"pause_dispatch","expected_revision":revision,"idempotency_key":"durable-client-retry","args":{"paused":true}});
    let file = root.path().join("saved.json");
    std::fs::write(&file, saved.to_string()).unwrap();
    let control = || {
        Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
            .args(["--control", path, "p", file.to_str().unwrap()])
            .env_clear()
            .env("CORENIUS_CLIENT_TOKEN", "client-test-secret")
            .output()
            .unwrap()
    };
    let first = control();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let second = control();
    assert_eq!(
        first.stdout, second.stdout,
        "reconnect must replay exact saved receipt"
    );
    let events = call(
        path,
        &json!({"auth":auth,"project":"p","query":{"kind":"events","args":{"after":0}}}),
    )
    .unwrap();
    let cursor = events.as_array().unwrap().last().unwrap()["seq"].clone();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "pause_dispatch")
            .count(),
        1
    );
    assert_eq!(
        call(
            path,
            &json!({"auth":auth,"project":"p","query":{"kind":"events","args":{"after":cursor}}})
        )
        .unwrap(),
        json!([])
    );
    let dashboard = Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .args(["--dashboard", path, "p", "--once"])
        .env_clear()
        .env("CORENIUS_CLIENT_TOKEN", "client-test-secret")
        .output()
        .unwrap();
    assert!(dashboard.status.success());
    let display = String::from_utf8(dashboard.stdout).unwrap();
    assert!(
        display.contains("persistent client fixture")
            && display.contains("replay cursor")
            && display.contains("Unknown effects")
    );
    assert!(!display.contains("client-test-secret"));
    let mut mcp = Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .args(["--mcp", path, "p"])
        .env_clear()
        .env("CORENIUS_CLIENT_TOKEN", "client-test-secret")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(mcp.stdin.take().unwrap(),"{}",json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"corenius_query","arguments":{"kind":"control_center"}}})).unwrap();
    let output = mcp.wait_with_output().unwrap();
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    let center: Value =
        serde_json::from_str(result["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(center["mission"]["objective"], "persistent client fixture");
    assert_eq!(center["project"]["paused"], true);
}
#[test]
fn control_center_cli_reports_transport_failure_without_opening_storage() {
    let output = Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .args([
            "--dashboard",
            "/tmp/corenius-absent-fixture.sock",
            "p",
            "--once",
        ])
        .env("CORENIUS_OPERATOR_TOKEN", "operator-client-secret")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("controller transport"),
        "dashboard must use controller socket, observed: {error}"
    );
    assert!(!error.contains("operator-client-secret"));
}

#[test]
fn repair3_authenticated_websocket_control_frames() {
    let status = Command::new("python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/websocket_controls.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .status()
        .unwrap();
    assert!(status.success(), "authenticated WebSocket controls failed");
}

#[test]
fn repair3_monitor_corrections_have_bounded_complete_projection() {
    let status = Command::new("python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/monitor_projection.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .status()
        .unwrap();
    assert!(status.success(), "bounded monitoring projection failed");
}
