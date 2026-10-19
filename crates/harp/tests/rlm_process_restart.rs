//! Historical graph inputs and databases are preserved, never resumed by Harp.
use std::fs;
use std::os::unix::fs::PermissionsExt;

use assert_cmd::Command;
use rusqlite::Connection;
use serde_json::Value;
use tempfile::TempDir;

fn retired(args: &[&str], root: &TempDir, family: &str) {
    let output = Command::cargo_bin("harp")
        .unwrap()
        .current_dir(root.path())
        .args(["--format", "json"])
        .args(args)
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["data"]["code"], format!("{family}.retired"));
    let message = error["data"]["message"].as_str().unwrap();
    assert!(message.contains("harp durable"));
    assert!(message.contains("not imported"));
}

#[test]
fn every_old_graph_command_refuses_before_initializing_state_or_reading_graphs() {
    let cases: &[(&str, &[&str])] = &[
        (
            "rlm",
            &[
                "rlm",
                "run",
                "--benchmark",
                "missing.json",
                "--runtime",
                "fake",
            ],
        ),
        ("rlm", &["rlm", "resume", "historical-run"]),
        ("rlm", &["rlm", "resume", "--all-incomplete"]),
        ("rlm", &["rlm", "status", "historical-run"]),
        ("rlm", &["rlm", "checkpoint"]),
        ("workflow", &["workflow", "run", "--file", "missing.json"]),
        ("workflow", &["workflow", "resume", "historical-run"]),
        ("workflow", &["workflow", "resume", "--all-incomplete"]),
        ("workflow", &["workflow", "status", "historical-run"]),
        ("workflow", &["workflow", "inspect", "historical-run"]),
        (
            "workflow",
            &[
                "workflow",
                "explain",
                "historical-run",
                "--task",
                "old-task",
            ],
        ),
    ];
    for (family, args) in cases {
        let root = TempDir::new().unwrap();
        retired(args, &root, family);
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0, "{args:?}");
    }
}

#[test]
fn old_state_and_runtime_executable_are_untouched_by_recovery_commands() {
    for family in ["rlm", "workflow"] {
        let root = TempDir::new().unwrap();
        let state = root.path().join("old-state");
        fs::create_dir(&state).unwrap();
        let database = state.join("state.sqlite");
        let connection = Connection::open(&database).unwrap();
        connection.execute_batch("CREATE TABLE retained(value TEXT); INSERT INTO retained VALUES('old work'); PRAGMA user_version=41;").unwrap();
        connection.close().unwrap();
        let original = fs::read(&database).unwrap();
        let marker = root.path().join("provider-called");
        let executable = root.path().join("old-runtime");
        fs::write(
            &executable,
            format!("#!/bin/sh\nprintf called > '{}'\n", marker.display()),
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        retired(
            &[
                family,
                "resume",
                "historical-run",
                "--state-dir",
                "old-state",
                "--runtime-executable",
                executable.to_str().unwrap(),
            ],
            &root,
            family,
        );
        for command in if family == "rlm" {
            vec!["status", "checkpoint"]
        } else {
            vec!["status", "inspect", "explain"]
        } {
            let mut args = vec![family, command];
            if command != "checkpoint" {
                args.push("historical-run");
            }
            if command == "explain" {
                args.extend(["--task", "old-task"]);
            }
            args.extend(["--state-dir", "old-state"]);
            retired(&args, &root, family);
        }
        assert_eq!(original, fs::read(&database).unwrap());
        assert_eq!(fs::read_dir(&state).unwrap().count(), 1);
        assert!(!marker.exists());
        assert!(!root.path().join(".harp").exists());
    }
}
