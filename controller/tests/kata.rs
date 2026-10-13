#![cfg(feature = "fault-injection")]

use std::process::Command;

#[test]
fn pinned_kata_public_protocol() {
    run_fixture(None);
}

#[test]
fn snapshot_path_replacement_stays_within_registered_source() {
    run_fixture(Some("snapshot"));
}

#[test]
fn unchanged_source_recovery_retains_history_and_projection_drift() {
    run_fixture(Some("availability"));
}

#[test]
fn unresolved_task_preserves_own_order_and_peer_publication() {
    run_fixture(Some("isolation"));
}

#[test]
fn maximum_progress_is_retained_and_publishes_bounded_summary() {
    run_fixture(Some("progress"));
}

#[test]
fn genuine_upstream_delay_does_not_starve_control_receipts() {
    run_fixture(Some("responsiveness"));
}

#[test]
fn recovered_git_before_ref_effect_publishes_once_to_supported_kata() {
    run_fixture(Some("git-publication-before_ref_update"));
}
#[test]
fn recovered_git_after_ref_effect_publishes_once_to_supported_kata() {
    run_fixture(Some("git-publication-after_ref_update"));
}

#[test]
fn repair3_captured_kata_brief_amendment_advances_exact_admitted_source() {
    run_fixture(Some("brief-amendment"));
}

fn run_fixture(case: Option<&str>) {
    let mut command = Command::new("python3");
    command
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/kata_protocol.py"
        ))
        .env(
            "CONTROLLER_BINARY",
            env!("CARGO_BIN_EXE_corenius-sqlite-controller"),
        )
        .env_remove("CORENIUS_KATA_REPAIR_CASE");
    if let Some(case) = case {
        command.env("CORENIUS_KATA_REPAIR_CASE", case);
    }
    let status = command.status().expect("run retained real Kata fixture");
    assert!(
        status.success(),
        "real Kata/public controller qualification failed"
    );
}
