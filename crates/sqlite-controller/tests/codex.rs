use corenius_sqlite_controller::Controller;
use serde_json::json;
use std::process::Command;

#[test]
#[cfg(feature = "fault-injection")]
fn actual_native_peer_tool_interleaving_routes_current_owner_in_both_topologies() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_CONTROL_CENTER", "1")
        .env("CORENIUS_TEST_TOOL_INTERLEAVING", "1")
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
fn actual_native_ui_drain_preserves_current_binding_under_interleaving() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_CONTROL_CENTER", "1")
        .env("CORENIUS_TEST_UI_DRAIN_RACE", "1")
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
fn actual_native_control_races_and_authenticated_correction() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_CONTROL_CENTER", "1")
        .env("CORENIUS_TEST_MONITOR_NATIVE", "1")
        .env("CORENIUS_TEST_CONTROL_RACES", "1")
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
fn actual_native_product_lead_question_transport_and_explicit_tools() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_CONTROL_CENTER", "1")
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
fn actual_native_scoped_tui_attachment_and_literal_input() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_CONTROL_CENTER", "1")
        .env("CORENIUS_TEST_UI_FIXTURE", "1")
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
fn actual_native_visible_worker_ingestion_is_advisory_and_retained() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_CONTROL_CENTER", "1")
        .env("CORENIUS_TEST_MONITOR_NATIVE", "1")
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
fn native_registry_is_a_durable_authenticated_controller_projection() {
    let native = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_init.py"
        ))
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&native.stdout));
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    let root = tempfile::tempdir().unwrap();
    let controller =
        Controller::open(root.path().join("state.db"), "operator-registry-secret").unwrap();
    let registry = controller.handle(json!({"auth":{"principal":"operator","token":"operator-registry-secret"},"project":"registry","query":{"kind":"codex_registry","args":{}}}));
    assert!(
        registry.is_ok(),
        "qualified native registry projection must exist: {registry:?}"
    );
    let refused = controller.handle(json!({"auth":{"principal":"operator","token":"wrong"},"project":"registry","query":{"kind":"codex_registry","args":{}}}));
    assert!(refused.is_err());
}

#[test]
fn configured_native_topologies_are_isolated_and_controller_owned() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
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
fn pinned_native_services_use_supervised_private_network_profiles() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_profile.py"
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
fn actual_native_turn_ack_loss_reconciles_without_replay() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_ACK_FIXTURE", "1")
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
fn durable_native_configuration_refuses_retargeted_topology() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_RETARGET_FIXTURE", "1")
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
fn disconnected_native_executor_refuses_fallback_and_rebinds_owned_realm() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_DISCONNECT_FIXTURE", "1")
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
fn durable_native_confirmation_survives_crash_before_binding_adoption() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_CONFIRM_FIXTURE", "1")
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
fn configured_native_daemon_refuses_missing_runtime_configuration() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_RETARGET_FIXTURE", "1")
        .env("CORENIUS_TEST_RETARGET_KIND", "missing")
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
fn native_registry_projects_capabilities_and_scoped_typed_turns() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_PROJECTION_FIXTURE", "1")
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(feature = "fault-injection")]
#[test]
fn unknown_native_start_refuses_new_turn_until_reconciled() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_ACK_FIXTURE", "1")
        .env("CORENIUS_TEST_ACK_FENCE_FIXTURE", "1")
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(feature = "fault-injection")]
fn repair_fixture(extra: &[(&str, &str)]) {
    let mut failures = Vec::new();
    for topology in ["shared", "separate"] {
        let mut command = Command::new("/usr/bin/python3");
        command
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/native_registry.py"
            ))
            .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
            .env("CORENIUS_TEST_TOPOLOGY", topology);
        for (key, value) in extra {
            command.env(key, value);
        }
        let output = command.output().unwrap();
        println!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            failures.push(format!(
                "{topology}: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[cfg(feature = "fault-injection")]
#[test]
fn repair_later_turn_confirmation_survives_binding_crash() {
    repair_fixture(&[("CORENIUS_TEST_ADOPT_CRASH", "turn_confirmation")]);
}
#[cfg(feature = "fault-injection")]
#[test]
fn repair_history_confirmation_survives_binding_crash() {
    repair_fixture(&[("CORENIUS_TEST_ADOPT_CRASH", "history_confirmation")]);
}
#[cfg(feature = "fault-injection")]
#[test]
fn repair_empty_bound_thread_survives_controller_crash() {
    repair_fixture(&[("CORENIUS_TEST_EMPTY_CRASH", "1")]);
}
#[cfg(feature = "fault-injection")]
#[test]
fn repair_native_supervisors_reopen_cancelled_records() {
    repair_fixture(&[("CORENIUS_TEST_STORE_RESTART_FIXTURE", "cancelled")]);
}
#[cfg(feature = "fault-injection")]
#[test]
fn repair_native_supervisors_reopen_unknown_records() {
    repair_fixture(&[("CORENIUS_TEST_STORE_RESTART_FIXTURE", "live")]);
}
#[cfg(feature = "fault-injection")]
#[test]
fn repair_interleaved_native_events_reach_their_exact_binding() {
    repair_fixture(&[("CORENIUS_TEST_EVENT_FIXTURE", "1")]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair_empty_bound_thread_changed_incarnation_stays_unknown() {
    repair_fixture(&[
        ("CORENIUS_TEST_EMPTY_CRASH", "1"),
        ("CORENIUS_TEST_EMPTY_RESTART", "1"),
    ]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair2_native_ui_lost_ack_and_retained_steer() {
    repair_fixture(&[
        ("CORENIUS_TEST_CONTROL_CENTER", "1"),
        ("CORENIUS_TEST_REPAIR2_UI", "1"),
    ]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair_native_executor_readiness_precedes_registration() {
    repair_fixture(&[("CORENIUS_TEST_EXEC_READY_FIXTURE", "1")]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair2_initial_turn_receipt_survives_app_restart_before_adoption() {
    repair_fixture(&[("CORENIUS_TEST_INITIAL_CONFIRM", "1")]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair2_empty_thread_after_first_loaded_page_recovers() {
    repair_fixture(&[
        ("CORENIUS_TEST_EMPTY_CRASH", "1"),
        ("CORENIUS_TEST_PAGE_FIXTURE", "1"),
        ("CORENIUS_TEST_PAGE_PEER_FIRST", "1"),
    ]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair2_retired_shared_notifications_preserve_active_peer() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_RETIRED_EVENT_FIXTURE", "1")
        .env("CORENIUS_TEST_TOPOLOGY", "shared")
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair3_adopted_thread_delivers_received_first_turn_notifications() {
    repair_fixture(&[
        ("CORENIUS_TEST_CONFIRM_FIXTURE", "1"),
        ("CORENIUS_TEST_ADOPT_EVENT_FIXTURE", "1"),
    ]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair3_cleanup_retry_preserves_terminal_stop_and_pending_unsubscribe() {
    repair_fixture(&[("CORENIUS_TEST_CLEANUP_FIXTURE", "1")]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair3_pending_cleanup_rejects_changed_controller_connection() {
    repair_fixture(&[
        ("CORENIUS_TEST_CLEANUP_FIXTURE", "1"),
        ("CORENIUS_TEST_CLEANUP_RESTART", "connection"),
    ]);
}
#[cfg(feature = "fault-injection")]
#[test]
fn repair3_pending_cleanup_rejects_changed_native_incarnation() {
    repair_fixture(&[
        ("CORENIUS_TEST_CLEANUP_FIXTURE", "1"),
        ("CORENIUS_TEST_CLEANUP_RESTART", "incarnation"),
    ]);
}

#[test]
#[cfg(feature = "fault-injection")]
fn ticket04_shared_first_cleanup_qualifies_fresh_restart() {
    native_ticket04_restart("shared", false);
}
#[test]
#[cfg(feature = "fault-injection")]
fn ticket04_separate_first_cleanup_qualifies_fresh_restart() {
    native_ticket04_restart("separate", false);
}
#[cfg(feature = "fault-injection")]
fn native_ticket04_restart(topology: &str, lost_ack: bool) {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_PRE_UNSUBSCRIBE_FIXTURE", "1")
        .env("CORENIUS_TEST_GIT_NATIVE_FIXTURE", "1")
        .env("CORENIUS_TEST_TOPOLOGY", topology)
        .env(
            "CORENIUS_TEST_RESTART_LOST_ACK_FIXTURE",
            if lost_ack { "1" } else { "0" },
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
fn ticket04_shared_restart_unsubscribe_lost_ack_preserves_qualification() {
    native_ticket04_restart("shared", true);
}
#[test]
#[cfg(feature = "fault-injection")]
fn ticket04_separate_restart_unsubscribe_lost_ack_preserves_qualification() {
    native_ticket04_restart("separate", true);
}

#[test]
#[cfg(feature = "fault-injection")]
fn repair1_before_source_actual_native_ui_start_replay() {
    repair_fixture(&[
        ("CORENIUS_TEST_CONTROL_CENTER", "1"),
        ("CORENIUS_TEST_UI_REPAIR", "replay"),
    ]);
}
#[test]
#[cfg(feature = "fault-injection")]
fn repair1_actual_native_ui_exact_interrupt() {
    repair_fixture(&[
        ("CORENIUS_TEST_CONTROL_CENTER", "1"),
        ("CORENIUS_TEST_UI_REPAIR", "interrupt"),
    ]);
}

#[test]
#[cfg(feature = "fault-injection")]
fn repair1_before_source_original_initial_confirmation_ordering() {
    repair_fixture(&[
        ("CORENIUS_TEST_INITIAL_CONFIRM", "1"),
        ("CORENIUS_TEST_INITIAL_CONFIRM_ORDERING", "1"),
    ]);
}

#[cfg(feature = "fault-injection")]
#[test]
fn repair3_native_ui_unadmitted_revision_retry() {
    repair_fixture(&[
        ("CORENIUS_TEST_CONTROL_CENTER", "1"),
        ("CORENIUS_TEST_REPAIR3_UI", "1"),
    ]);
}

#[test]
#[cfg(feature = "fault-injection")]
fn stale_correction_authority_preserves_history_and_allows_current_delivery() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/native_registry.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_CONTROL_CENTER", "1")
        .env("CORENIUS_TEST_CORRECTION_AUTHORITY", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn native_registry_selection_skips_unbound_peer() {
    let output = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/test_native_registry_selection.py"
        ))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
