use assert_cmd::Command;
use serde_json::Value;

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
        &["--format", "json", "providers", "doctor"],
        "provider.not_implemented",
    );
    assert_error_code(
        &["--format", "json", "releases", "compile"],
        "context_control.not_implemented",
    );
    assert_error_code(
        &["--format", "json", "releases", "list"],
        "context_control.not_implemented",
    );
    assert_error_code(
        &["--format", "json", "releases", "inspect", "sha256-example"],
        "context_control.not_implemented",
    );
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
