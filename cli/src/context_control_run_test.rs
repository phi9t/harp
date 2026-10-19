use std::fs;
use std::os::unix::fs::PermissionsExt;

use assert_cmd::Command;
use harp::context_control::provider::ProviderRunOptions;
use harp::context_control::run::{run, RunReporter, RunRequest};
use harp::context_control::{ProviderId, WorkflowChoice};
use serde_json::Value;
use tempfile::TempDir;

fn fixture() -> TempDir {
    let root = TempDir::new().unwrap();
    assert!(std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(root.path())
        .status()
        .unwrap()
        .success());
    let bin = root.path().join("bin");
    fs::create_dir(&bin).unwrap();
    for name in ["codex", "traecli"] {
        let executable = bin.join(name);
        fs::write(
            &executable,
            "#!/bin/sh\nprintf probe >> \"$HARP_RETIRE_MARKER\"\nexit 1\n",
        )
        .unwrap();
        fs::set_permissions(executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    root
}

#[test]
fn retired_provider_commands_refuse_before_probes_or_private_state() {
    for provider in ["codex", "trae"] {
        let root = fixture();
        let marker = root.path().join("provider-called");
        let paths = std::env::join_paths([root.path().join("bin")].into_iter().chain(
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
        ))
        .unwrap();
        let output = Command::cargo_bin("harp")
            .unwrap()
            .current_dir(root.path())
            .env("PATH", paths)
            .env("HARP_RETIRE_MARKER", &marker)
            .env("HARP_HOME", root.path().join("private-state"))
            .env("HOME", root.path().join("home"))
            .args([
                "--format",
                "json",
                "run",
                "--provider",
                provider,
                "--workflow",
                "ci_repair",
                "--model",
                "retained-model",
                "--profile",
                "old-profile",
                "--sandbox",
                "danger-full-access",
                "--approval",
                "never",
                "--",
                "fix the task",
            ])
            .assert()
            .failure()
            .get_output()
            .clone();
        assert!(output.stdout.is_empty());
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["data"]["code"], "run.retired");
        let message = error["data"]["message"].as_str().unwrap();
        assert!(message.contains("harp durable"));
        assert!(message.contains("docs/durable-runtime.md"));
        assert!(
            !marker.exists(),
            "retirement must precede every provider probe"
        );
        for directory in ["private-state", "home", ".harp"] {
            assert!(!root.path().join(directory).exists());
        }
    }
}

struct NoLifecycle;
impl RunReporter for NoLifecycle {
    fn lifecycle(&mut self, _: &str) {
        panic!("a retired execution must not begin a lifecycle")
    }
    fn warning(&mut self, _: &str) {
        panic!("a retired execution must not run preflight")
    }
}

#[test]
fn retired_library_run_refuses_without_starting_lifecycle() {
    for provider in [ProviderId::Codex, ProviderId::Trae] {
        let error = run(
            RunRequest {
                provider,
                workflow: WorkflowChoice::Auto,
                task: "historical direct task".into(),
                options: ProviderRunOptions::default(),
            },
            &mut NoLifecycle,
        )
        .unwrap_err();
        assert_eq!(error.code(), "run.retired");
        assert!(error.message.contains("harp durable"));
    }
}

#[test]
fn retirement_help_keeps_legacy_spellings_and_points_to_durable() {
    for command in ["run", "rlm", "workflow"] {
        let output = Command::cargo_bin("harp")
            .unwrap()
            .args([command, "--help"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let help = String::from_utf8(output).unwrap();
        assert!(help.contains("Retired"), "{command}: {help}");
        assert!(help.contains("durable"), "{command}: {help}");
    }
}
