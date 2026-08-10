mod support;

use std::collections::BTreeMap;

use assert_cmd::Command;
use serde_json::Value;

use support::context_control::ProviderFixtures;

fn harp() -> Command {
    Command::cargo_bin("harp").expect("harp binary")
}

#[test]
fn context_control_commands_are_exposed_without_running_a_provider() {
    harp().args(["providers", "--help"]).assert().success();
    harp().args(["releases", "--help"]).assert().success();
    harp().args(["run", "--help"]).assert().success();
}

#[test]
fn context_control_skeletons_return_typed_not_implemented_errors() {
    assert_error_code(
        &[
            "--format",
            "json",
            "run",
            "--provider",
            "trae",
            "--",
            "fix",
            "the",
            "test",
        ],
        "run.not_implemented",
    );
}

#[test]
fn run_accepts_every_documented_context_control_option_value() {
    let cases: &[(&str, &[&str])] = &[
        ("provider trae", &["--provider", "trae"]),
        ("provider codex", &["--provider", "codex"]),
        ("workflow auto", &["--workflow", "auto"]),
        ("workflow ci_repair", &["--workflow", "ci_repair"]),
        ("workflow code_review", &["--workflow", "code_review"]),
        (
            "workflow dependency_update",
            &["--workflow", "dependency_update"],
        ),
        ("workflow general_coding", &["--workflow", "general_coding"]),
        ("sandbox read-only", &["--sandbox", "read-only"]),
        ("sandbox workspace-write", &["--sandbox", "workspace-write"]),
        (
            "sandbox danger-full-access",
            &["--sandbox", "danger-full-access"],
        ),
        ("approval untrusted", &["--approval", "untrusted"]),
        ("approval on-request", &["--approval", "on-request"]),
        ("approval never", &["--approval", "never"]),
    ];

    for (name, options) in cases {
        let mut args = vec!["--format", "json", "run"];
        if options.first().copied() != Some("--provider") {
            args.extend(["--provider", "trae"]);
        }
        args.extend_from_slice(options);
        args.extend(["--", "test"]);

        assert_error_code_for_case(&args, "run.not_implemented", name);
    }
}

#[test]
fn releases_compile_list_and_inspect_return_the_same_stable_id() {
    let temp = tempfile::tempdir().expect("isolated HARP_HOME parent");
    let home = std::fs::canonicalize(temp.path())
        .expect("canonical HARP_HOME parent")
        .join("harp-state");

    let compiled = run_release_json(&home, &["releases", "compile"]);
    let release_id = compiled["data"]["release_id"]
        .as_str()
        .expect("compile release ID")
        .to_owned();
    assert!(release_id.starts_with("sha256-"));
    assert_eq!(compiled["command"], "releases.compile");

    let compiled_again = run_release_json(&home, &["releases", "compile"]);
    assert_eq!(compiled_again["data"]["release_id"], release_id);

    let listed = run_release_json(&home, &["releases", "list"]);
    assert_eq!(listed["command"], "releases.list");
    assert_eq!(listed["data"]["releases"], serde_json::json!([release_id]));

    let inspected = run_release_json(&home, &["releases", "inspect", &release_id]);
    assert_eq!(inspected["command"], "releases.inspect");
    assert_eq!(inspected["data"]["release_id"], release_id);
    assert_eq!(
        inspected["data"]["identity"]["schema_version"],
        "harp-context-release-identity/v1"
    );
}

fn run_release_json(home: &std::path::Path, args: &[&str]) -> Value {
    let output = harp()
        .env("HARP_HOME", home)
        .env_remove("HOME")
        .args(["--format", "json"])
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("JSON success envelope")
}

#[test]
fn providers_doctor_reports_both_provider_snapshots_in_stable_order() {
    let fixtures = ProviderFixtures::supported();
    let output = harp()
        .env("PATH", fixtures.path())
        .args(["--format", "json", "providers", "doctor"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("provider doctor JSON");
    assert_eq!(envelope["command"], "providers.doctor");
    assert_eq!(envelope["status"], "ok");
    let providers = envelope["data"].as_array().expect("provider snapshots");
    assert_eq!(providers.len(), 2);
    assert_eq!(providers[0]["provider"], "trae");
    assert!(providers[0]["executable"]
        .as_str()
        .expect("Trae executable")
        .ends_with("/traecli"));
    assert_eq!(providers[0]["version"], "0.200.19");
    assert_eq!(providers[1]["provider"], "codex");
    assert!(providers[1]["executable"]
        .as_str()
        .expect("Codex executable")
        .ends_with("/codex"));
    assert_eq!(providers[1]["version"], "0.144.5");

    for provider in providers {
        assert_eq!(provider["schema_version"], "harp-provider-capabilities/v1");
        for required in [
            "exec_json",
            "output_last_message",
            "working_directory",
            "approval_config",
        ] {
            assert_eq!(provider[required], true, "{required}");
        }
        for optional in [
            "model",
            "profile",
            "sandbox",
            "resume_json",
            "app_server_schema",
        ] {
            assert_eq!(provider[optional], true, "{optional}");
        }
        let digest = provider["capability_sha256"]
            .as_str()
            .expect("capability digest");
        assert!(digest.starts_with("sha256:"));
        assert_eq!(digest.len(), 71);
    }
}

#[test]
fn providers_doctor_text_reports_provider_identity_and_capability_status() {
    let fixtures = ProviderFixtures::supported();
    let output = harp()
        .env("PATH", fixtures.path())
        .args(["providers", "doctor"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).expect("provider doctor text");
    let lines = output.lines().collect::<Vec<_>>();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("trae "));
    assert!(lines[0].contains("executable="));
    assert!(lines[0].contains("/traecli"));
    assert!(lines[0].contains("version=0.200.19"));
    assert!(lines[0].contains("capabilities=ready"));
    assert!(lines[1].starts_with("codex "));
    assert!(lines[1].contains("executable="));
    assert!(lines[1].contains("/codex"));
    assert!(lines[1].contains("version=0.144.5"));
    assert!(lines[1].contains("capabilities=ready"));
}

#[test]
fn providers_doctor_can_probe_one_provider() {
    let fixtures = ProviderFixtures::supported();
    let output = harp()
        .env("PATH", fixtures.path())
        .args([
            "--format",
            "json",
            "providers",
            "doctor",
            "--provider",
            "codex",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("provider doctor JSON");
    let providers = envelope["data"].as_array().expect("provider snapshots");
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0]["provider"], "codex");
}

#[test]
fn providers_doctor_digests_do_not_depend_on_fixture_roots() {
    let first = doctor_digests(&ProviderFixtures::supported());
    let second = doctor_digests(&ProviderFixtures::supported());
    assert_eq!(first, second);
}

fn doctor_digests(fixtures: &ProviderFixtures) -> BTreeMap<String, String> {
    let output = harp()
        .env("PATH", fixtures.path())
        .args(["--format", "json", "providers", "doctor"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let envelope: Value = serde_json::from_slice(&output).expect("provider doctor JSON");
    envelope["data"]
        .as_array()
        .expect("provider snapshots")
        .iter()
        .map(|provider| {
            (
                provider["provider"]
                    .as_str()
                    .expect("provider name")
                    .to_owned(),
                provider["capability_sha256"]
                    .as_str()
                    .expect("capability digest")
                    .to_owned(),
            )
        })
        .collect()
}

fn assert_error_code(args: &[&str], expected_code: &str) {
    assert_error_code_for_case(args, expected_code, "CLI invocation");
}

fn assert_error_code_for_case(args: &[&str], expected_code: &str, case: &str) {
    let output = harp()
        .args(args)
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let envelope: Value = serde_json::from_slice(&output)
        .unwrap_or_else(|error| panic!("{case}: expected JSON error envelope: {error}"));
    assert_eq!(envelope["status"], "error", "{case}");
    assert_eq!(envelope["data"]["code"], expected_code, "{case}");
}
