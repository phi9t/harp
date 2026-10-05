use corenius_sqlite_controller::Controller;
use serde_json::json;
#[test]
#[cfg(feature = "fault-injection")]
fn actual_observer_git_authority_has_no_effect_and_preserves_authorized_control() {
    let output = std::process::Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/git_protocol.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_GIT_MODES", "observer-authority")
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn workspace_inventory_is_an_authenticated_controller_projection() {
    let r = tempfile::tempdir().unwrap();
    let c = Controller::open(r.path().join("state.db"), "operator-git-secret").unwrap();
    let response=c.handle(json!({"auth":{"principal":"operator","token":"operator-git-secret"},"project":"p","query":{"kind":"workspace_inventory","args":{}}}));
    assert!(
        response.is_ok(),
        "workspace inventory must be a supported authenticated projection: {response:?}"
    );
}
#[test]
#[cfg(feature = "fault-injection")]
fn actual_private_git_candidates_verification_landing_recovery_and_hygiene() {
    let output = std::process::Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/git_protocol.py"
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

#[test]
#[cfg(feature = "fault-injection")]
fn repair2_bounded_physical_inventory_preserves_history_and_inputs() {
    let output = std::process::Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/git_protocol.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env(
            "CORENIUS_TEST_GIT_MODES",
            "bounded-inventory,inventory-slow",
        )
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[cfg(feature = "fault-injection")]
fn repair3_landing_applying_refuses_competing_repair() {
    let status = std::process::Command::new("python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/git_protocol.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_GIT_MODES", "repair3-applying")
        .status()
        .unwrap();
    assert!(status.success(), "repair must preserve admitted landing");
}

#[test]
#[cfg(feature = "fault-injection")]
fn repair3_landing_unknown_refuses_competing_repair() {
    let status = std::process::Command::new("python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/git_protocol.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_GIT_MODES", "repair3-unknown")
        .status()
        .unwrap();
    assert!(status.success(), "repair must preserve admitted landing");
}

#[test]
#[cfg(feature = "fault-injection")]
fn current_attempt_progress_excludes_predecessor_and_task_reports() {
    let output = std::process::Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/git_protocol.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_GIT_MODES", "attempt-progress")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
