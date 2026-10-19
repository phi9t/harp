//! Process-boundary tests for the Python bridge, using a disposable backend
//! that reports its invocation. These tests do not simulate worker success.
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

fn fixture() -> TempDir {
    let directory = TempDir::new().expect("fixture root");
    fs::write(directory.path().join("Cargo.toml"), "[workspace]\n").expect("repository marker");
    let package = directory.path().join("durable");
    fs::create_dir(&package).expect("backend package");
    fs::create_dir_all(directory.path().join("nested/working-directory"))
        .expect("nested working directory");
    fs::write(package.join("__init__.py"), "").expect("package marker");
    fs::write(
        package.join("__main__.py"),
        r#"import json, os, signal, sys
if 'backend-error' in sys.argv:
    print(json.dumps({'code': 'fixture.refusal'}), file=sys.stderr)
    sys.exit(23)
if 'backend-signal' in sys.argv:
    os.kill(os.getpid(), signal.SIGTERM)
print(json.dumps({
    'argv': sys.argv[1:], 'cwd': os.getcwd(), 'pid': os.getpid(),
    'input': sys.stdin.read(),
    'env': {key: os.environ.get(key) for key in (
        'PYTHONPATH', 'PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE',
        'PYTHONHOME', 'PYTHONSTARTUP', 'PYTHONINSPECT')},
}))
"#,
    )
    .expect("backend fixture");
    directory
}

fn harp(root: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("harp"));
    command.current_dir(root);
    command
}

#[test]
fn forwards_exact_arguments_streams_and_controlled_python_environment() {
    let fixture = fixture();
    let root = fs::canonicalize(fixture.path()).expect("canonical root");
    let mut command = harp(&root.join("nested/working-directory"));
    command
        .args([
            "--format",
            "json",
            "durable",
            "--state",
            "/state with spaces",
            "submit",
            "quotes ' \" and $()",
            "--args",
            "{\"task\":\"a\nb\"}",
        ])
        .env("PYTHONPATH", "/unrelated/imports")
        .env("PYTHONHOME", "/missing/python-home")
        .env("PYTHONSTARTUP", "/unrelated/startup.py")
        .env("PYTHONINSPECT", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn bridge");
    let process_id = child.id();
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"streamed input\n")
        .expect("write input");
    let output = child.wait_with_output().expect("wait for backend");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_slice(&output.stdout).expect("backend JSON");
    assert_eq!(
        result["argv"],
        serde_json::json!([
            "--format",
            "json",
            "--state",
            "/state with spaces",
            "submit",
            "quotes ' \" and $()",
            "--args",
            "{\"task\":\"a\nb\"}",
        ])
    );
    assert_eq!(result["cwd"], root.to_str().expect("root text"));
    assert_eq!(result["pid"], process_id);
    assert_eq!(result["input"], "streamed input\n");
    assert_eq!(result["env"]["PYTHONPATH"], root.display().to_string());
    assert_eq!(result["env"]["PYTHONNOUSERSITE"], "1");
    assert_eq!(result["env"]["PYTHONDONTWRITEBYTECODE"], "1");
    for key in ["PYTHONHOME", "PYTHONSTARTUP", "PYTHONINSPECT"] {
        assert!(result["env"][key].is_null());
    }
}

#[test]
fn backend_owns_help_and_nonzero_exit_output() {
    let fixture = fixture();
    let output = harp(fixture.path())
        .args(["durable", "--help"])
        .stdin(Stdio::null())
        .output()
        .expect("help invocation");
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).expect("forwarded help");
    assert_eq!(
        result["argv"],
        serde_json::json!(["--format", "text", "--help"])
    );
    let output = harp(fixture.path())
        .args(["durable", "backend-error"])
        .output()
        .expect("backend refusal");
    assert_eq!(output.status.code(), Some(23));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).expect("structured refusal");
    assert_eq!(error["code"], "fixture.refusal");
}

#[test]
fn backend_global_flags_and_separator_are_forwarded_verbatim() {
    let fixture = fixture();
    let output = harp(fixture.path())
        .args(["durable", "--format", "json", "--", "status"])
        .stdin(Stdio::null())
        .output()
        .expect("backend flags");
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).expect("backend invocation");
    assert_eq!(
        result["argv"],
        serde_json::json!(["--format", "text", "--format", "json", "--", "status"])
    );
}

#[test]
fn backend_signal_remains_the_public_process_signal() {
    use std::os::unix::process::ExitStatusExt;
    let fixture = fixture();
    let output = harp(fixture.path())
        .args(["durable", "backend-signal"])
        .output()
        .expect("signalled backend");
    assert_eq!(output.status.signal(), Some(libc::SIGTERM));
}

#[test]
fn rejects_missing_or_symlinked_backend_instead_of_using_pythonpath() {
    use std::os::unix::fs::symlink;
    let empty = TempDir::new().expect("empty checkout");
    let output = harp(empty.path())
        .args(["--format", "json", "durable", "status"])
        .output()
        .expect("missing backend");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("durable.root"));
    let fixture = fixture();
    fs::write(empty.path().join("Cargo.toml"), "[workspace]\n").expect("marker");
    symlink(fixture.path().join("durable"), empty.path().join("durable"))
        .expect("foreign package link");
    let output = harp(empty.path())
        .args(["--format", "json", "durable", "status"])
        .env("PYTHONPATH", fixture.path())
        .output()
        .expect("symlinked backend");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("durable.root"));
}
