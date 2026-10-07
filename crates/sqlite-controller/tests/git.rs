use corenius_sqlite_controller::{
    Controller,
    git_workspace::{GitBackend, digest},
};
use serde_json::json;
use std::{fs, path::Path, process::Command};

fn run_git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn pin(path: &Path) -> serde_json::Value {
    json!({"path":path,"sha256":digest(&fs::read(path).unwrap())})
}

fn write_runtime_policy(
    root: &Path,
    runtime_root: &Path,
    manifest: serde_json::Value,
) -> (String, tempfile::TempDir) {
    let r = tempfile::tempdir().unwrap();
    let authority = r.path().join("authority");
    let attempts = r.path().join("attempts");
    let canonical = r.path().join("canonical");
    let seed = r.path().join("seed");
    fs::create_dir_all(&authority).unwrap();
    fs::create_dir_all(&attempts).unwrap();
    fs::create_dir_all(&canonical).unwrap();
    fs::create_dir_all(&seed).unwrap();
    run_git(&canonical, &["init", "--bare", "--template="]);
    run_git(&seed, &["init", "--template="]);
    fs::write(seed.join("README"), "runtime manifest fixture\n").unwrap();
    run_git(&seed, &["add", "README"]);
    run_git(
        &seed,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@invalid",
            "commit",
            "-m",
            "B",
        ],
    );
    let base = run_git(&seed, &["rev-parse", "HEAD"]);
    run_git(
        &seed,
        &[
            "push",
            canonical.to_str().unwrap(),
            &format!("{base}:refs/heads/protected"),
        ],
    );
    let suite = r.path().join("suite.py");
    fs::write(&suite, "print('ok')\n").unwrap();
    let manifest_path = root.join("runtime-manifest.json");
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let policy_path = r.path().join("git.json");
    fs::write(
        &policy_path,
        serde_json::to_string(&json!({
            "project":"p",
            "root":authority,
            "attempts_root":attempts,
            "canonical":canonical,
            "protected_ref":"refs/heads/protected",
            "git":pin(Path::new("/usr/bin/git")),
            "python":pin(Path::new("/usr/bin/python3")),
            "bubblewrap":pin(Path::new("/usr/bin/bwrap")),
            "verifier_runtime":{
                "root":runtime_root,
                "manifest":{"path":manifest_path,"sha256":digest(&fs::read(&manifest_path).unwrap())}
            },
            "suites":[{"id":"s","task":"t","base":base,"script":pin(&suite),"args":[]}]
        }))
        .unwrap(),
    )
    .unwrap();
    (policy_path.to_str().unwrap().to_owned(), r)
}

#[test]
fn verifier_runtime_manifest_rejects_escaping_missing_and_tampered_entries() {
    fn expect_error(name: &str, build: impl FnOnce(&Path, &Path) -> serde_json::Value) {
        let runtime_holder = tempfile::tempdir().unwrap();
        let runtime_root = runtime_holder.path().join("runtime-snapshot/usr");
        fs::create_dir_all(&runtime_root).unwrap();
        let manifest = build(runtime_holder.path(), &runtime_root);
        let (policy_path, _policy_holder) =
            write_runtime_policy(runtime_holder.path(), &runtime_root, manifest);
        let error = match GitBackend::open(Path::new(&policy_path)) {
            Ok(_) => panic!("{name} unexpectedly accepted"),
            Err(error) => error,
        };
        assert!(
            error.contains("verifier runtime"),
            "{name} produced unexpected error: {error}"
        );
    }

    expect_error("chained escaping symlink", |_, runtime| {
        std::os::unix::fs::symlink(".", runtime.join("a")).unwrap();
        std::os::unix::fs::symlink("a/../outside", runtime.join("b")).unwrap();
        json!({
            "schema_version":"corenius.verifier-runtime.v1",
            "entries":[
                {"path":"a","kind":"symlink","target":"."},
                {"path":"b","kind":"symlink","target":"a/../outside"}
            ]
        })
    });
    expect_error("direct escaping symlink", |_, runtime| {
        std::os::unix::fs::symlink("../outside", runtime.join("bad")).unwrap();
        json!({
            "schema_version":"corenius.verifier-runtime.v1",
            "entries":[{"path":"bad","kind":"symlink","target":"../outside"}]
        })
    });
    expect_error("missing regular entry", |_, _| {
        json!({
            "schema_version":"corenius.verifier-runtime.v1",
            "entries":[{"path":"missing.txt","kind":"regular","bytes":7,"sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}]
        })
    });
    expect_error("tampered regular entry", |_, runtime| {
        let file = runtime.join("value.txt");
        fs::write(&file, "before").unwrap();
        let entry = json!({
            "path":"value.txt",
            "kind":"regular",
            "bytes":6,
            "sha256":digest(&fs::read(&file).unwrap())
        });
        fs::write(&file, "after!").unwrap();
        json!({"schema_version":"corenius.verifier-runtime.v1","entries":[entry]})
    });
}

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
fn verifier_runtime_snapshot_is_bound_to_public_proof() {
    let output = std::process::Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/git_protocol.py"
        ))
        .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .env("CORENIUS_TEST_GIT_MODES", "runtime-snapshot")
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
