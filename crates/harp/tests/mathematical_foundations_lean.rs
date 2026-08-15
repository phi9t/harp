use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::TempDir;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

#[test]
fn missing_lake_is_actionable() {
    let empty_path = TempDir::new().expect("empty PATH directory");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env("PATH", empty_path.path())
        .assert()
        .failure()
        .stderr("Mathematical Foundations Lean toolchain is unavailable\n");
}

#[test]
fn proof_holes_are_rejected_before_lake_runs() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    fs::write(
        temporary_project.path().join("ProofHole.lean"),
        "theorem proof_hole_fixture : True := by sorry\n",
    )
    .expect("write scoped proof-hole fixture");
    let fake_bin = TempDir::new().expect("fake lake directory");
    let lake_marker = fake_bin.path().join("lake-was-called");
    let fake_lake = fake_bin.path().join("lake");
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake lake");

    fs::write(&fake_lake, "#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\nexit 0\n")
        .expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    let assertion = Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("proof-placeholder text"));

    assert!(
        !lake_marker.exists(),
        "lake ran despite a scoped Lean proof hole: {assertion:?}"
    );
}

#[test]
fn ambient_project_override_is_ignored() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let scoped_elan_home = temporary_project.path().join("elan");
    fs::create_dir(&scoped_elan_home).expect("create scoped Elan home");
    fs::write(
        temporary_project.path().join("ProofHole.lean"),
        "theorem proof_hole_fixture : True := by sorry\n",
    )
    .expect("write scoped proof-hole fixture");
    let fake_bin = TempDir::new().expect("fake lake directory");
    let lake_marker = fake_bin.path().join("lake-was-called");
    let fake_lake = fake_bin.path().join("lake");
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake lake");

    fs::write(&fake_lake, "#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\nexit 0\n")
        .expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env(
            "HARP_MATHEMATICAL_FOUNDATIONS_PROJECT_ROOT",
            temporary_project.path(),
        )
        .env("ELAN_HOME", &scoped_elan_home)
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .success();

    assert!(
        lake_marker.exists(),
        "lake did not run against the canonical project root"
    );
}

#[test]
fn ambient_elan_toolchain_is_overridden_for_normal_builds() {
    let fake_bin = TempDir::new().expect("fake lake directory");
    let toolchain_capture = fake_bin.path().join("elan-toolchain");
    let scoped_elan_home = fake_bin.path().join("scoped-elan-home");
    fs::create_dir(&scoped_elan_home).expect("create scoped Elan home");
    let fake_lake = fake_bin.path().join("lake");
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake lake");

    fs::write(
        &fake_lake,
        "#!/bin/sh\nprintf '%s\\n' \"$ELAN_TOOLCHAIN\" > \"$ELAN_TOOLCHAIN_CAPTURE\"\n",
    )
    .expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env("ELAN_TOOLCHAIN", "untrusted/ambient:toolchain")
        .env("ELAN_HOME", &scoped_elan_home)
        .env("ELAN_TOOLCHAIN_CAPTURE", &toolchain_capture)
        .env("PATH", path)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(toolchain_capture).expect("read captured toolchain"),
        "leanprover/lean4:v4.32.1\n"
    );
}

#[test]
fn normal_build_rejects_missing_or_home_scoped_elan_home_before_lake_runs() {
    let fake_bin = TempDir::new().expect("fake lake directory");
    let lake_marker = fake_bin.path().join("lake-was-called");
    let fake_lake = fake_bin.path().join("lake");
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake lake");

    fs::write(&fake_lake, "#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n").expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    let assertion = Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env_remove("ELAN_HOME")
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("task-scoped ELAN_HOME"));

    assert!(
        !lake_marker.exists(),
        "lake ran without a task-scoped ELAN_HOME: {assertion:?}"
    );

    let home_dir = fake_bin.path().join("home");
    let home_elan = home_dir.join(".elan");
    fs::create_dir(&home_dir).expect("create fake home");
    fs::create_dir(&home_elan).expect("create home-scoped Elan directory");
    let assertion = Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env("HOME", &home_dir)
        .env("ELAN_HOME", &home_elan)
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("task-scoped ELAN_HOME"));

    assert!(
        !lake_marker.exists(),
        "lake ran with a home-scoped ELAN_HOME: {assertion:?}"
    );

    let scoped_elan_home = fake_bin.path().join("scoped-elan-home");
    fs::create_dir(&scoped_elan_home).expect("create scoped Elan home");
    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env_remove("HOME")
        .env("ELAN_HOME", &scoped_elan_home)
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("task-scoped ELAN_HOME"));

    let home_child = home_dir.join("child");
    fs::create_dir(&home_child).expect("create home child directory");
    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env("HOME", &home_dir)
        .env("ELAN_HOME", home_child.join("../.elan"))
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("task-scoped ELAN_HOME"));

    let home_link = fake_bin.path().join("home-elan-link");
    symlink(&home_elan, &home_link).expect("create home Elan symlink");
    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env("HOME", &home_dir)
        .env("ELAN_HOME", &home_link)
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("task-scoped ELAN_HOME"));

    assert!(
        !lake_marker.exists(),
        "lake ran with an indirect home-scoped ELAN_HOME"
    );
}

#[test]
fn source_scan_errors_stop_before_lake_runs() {
    let fake_bin = TempDir::new().expect("fake command directory");
    let lake_marker = fake_bin.path().join("lake-was-called");
    let fake_lake = fake_bin.path().join("lake");
    let fake_grep = fake_bin.path().join("grep");
    let scoped_elan_home = fake_bin.path().join("scoped-elan-home");
    fs::create_dir(&scoped_elan_home).expect("create scoped Elan home");
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake commands");

    fs::write(&fake_lake, "#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n").expect("write fake lake");
    fs::write(&fake_grep, "#!/bin/sh\nexit 2\n").expect("write failing fake grep");
    for command in [&fake_lake, &fake_grep] {
        let mut permissions = fs::metadata(command)
            .expect("read fake command permissions")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(command, permissions).expect("make fake command executable");
    }

    let assertion = Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_mathematical_foundations_lean.sh")
        .env("ELAN_HOME", &scoped_elan_home)
        .env("PATH", path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("source scan failed"));

    assert!(
        !lake_marker.exists(),
        "lake ran despite a source scan failure: {assertion:?}"
    );
}
