use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::{Builder, TempDir};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

fn scoped_elan_home() -> TempDir {
    let root = Path::new("/private/tmp/harp-mathematical-foundations-elan");
    fs::create_dir_all(root).expect("create wrapper-owned Elan root");
    Builder::new()
        .prefix("test-")
        .tempdir_in(root)
        .expect("create scoped Elan home")
}

fn fake_lake_bin(script: &str) -> (TempDir, OsString) {
    let fake_bin = TempDir::new().expect("fake lake directory");
    let fake_lake = fake_bin.path().join("lake");
    fs::write(&fake_lake, script).expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake lake");
    (fake_bin, path)
}

#[test]
fn shared_wrapper_builds_focused_lake_target_and_reports_timing() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\n' \"$*\" > \"$LAKE_ARGS\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("AutodiffGeometry")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=AutodiffGeometry")
                .and(predicate::str::contains("[lean] root=formalization/lean"))
                .and(predicate::str::contains("[lean] scan_seconds="))
                .and(predicate::str::contains("[lean] lake_seconds="))
                .and(predicate::str::contains("[lean] total_seconds="))
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "build AutodiffGeometry\n"
    );
}

#[test]
fn shared_wrapper_builds_crouzeix_focused_target() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\n' \"$*\" > \"$LAKE_ARGS\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("Crouzeix")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=Crouzeix")
                .and(predicate::str::contains("[lean] root=formalization/lean"))
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "build Crouzeix\n"
    );
}

#[test]
fn shared_wrapper_builds_all_targets_from_shared_root_once() {
    let (_fake_bin, path) =
        fake_lake_bin("#!/bin/sh\npwd > \"$LAKE_PWD\"\nprintf '%s\n' \"$*\" > \"$LAKE_ARGS\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");
    let lake_pwd = scoped_elan_home.path().join("lake-pwd");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("all")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("LAKE_PWD", &lake_pwd)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=all")
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "build\n"
    );
    assert_eq!(
        fs::read_to_string(lake_pwd).expect("read lake working directory"),
        format!("{}\n", repo_root().join("formalization/lean").display())
    );
}

#[test]
fn shared_wrapper_rejects_unknown_targets_before_lake_runs() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_marker = scoped_elan_home.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("UnknownLibrary")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_CALLED_FILE", &lake_marker)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown Lean library target"))
        .stdout(
            predicate::str::contains("[lean] outcome=failed")
                .and(predicate::str::contains("[lean] failure_stage=policy")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite an unknown Lean target"
    );
}

#[test]
fn nng4_admit_is_rejected_before_lake_runs() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    fs::write(
        temporary_project.path().join("Admit.lean"),
        "theorem admit_fixture : True := by admit\n",
    )
    .expect("write admit fixture");
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("NNG4Intro")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("PATH", path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicate::str::contains("proof-placeholder text"))
        .stdout(
            predicate::str::contains("[lean] outcome=failed")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(!lake_marker.exists(), "lake ran despite an admit fixture");
}

#[test]
fn all_rejects_crouzeix_admit_before_lake_runs() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let crouzeix_dir = temporary_project.path().join("Crouzeix");
    fs::create_dir_all(&crouzeix_dir).expect("create Crouzeix fixture directory");
    fs::write(
        crouzeix_dir.join("Admit.lean"),
        "theorem crouzeix_admit_fixture : True := by admit\n",
    )
    .expect("write Crouzeix admit fixture");
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("all")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("PATH", path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicate::str::contains("proof-placeholder text"))
        .stdout(
            predicate::str::contains("[lean] outcome=failed")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite a Crouzeix admit fixture"
    );
}
