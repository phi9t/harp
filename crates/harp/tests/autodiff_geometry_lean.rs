use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
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

#[test]
fn missing_lake_is_actionable() {
    let empty_path = TempDir::new().expect("empty PATH directory");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_autodiff_geometry_lean.sh")
        .env("PATH", empty_path.path())
        .assert()
        .failure()
        .stderr("Autodiff Geometry Lean toolchain is unavailable\n");
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
        .arg("scripts/check_autodiff_geometry_lean.sh")
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
    let scoped_elan_home = scoped_elan_home();
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
        .arg("scripts/check_autodiff_geometry_lean.sh")
        .env(
            "HARP_AUTODIFF_GEOMETRY_PROJECT_ROOT",
            temporary_project.path(),
        )
        .env("ELAN_HOME", scoped_elan_home.path())
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
    let scoped_elan_home = scoped_elan_home();
    let fake_lake = fake_bin.path().join("lake");
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake lake");

    fs::write(
        &fake_lake,
        "#!/bin/sh\nprintf '%s\n' \"$ELAN_TOOLCHAIN\" > \"$ELAN_TOOLCHAIN_CAPTURE\"\n",
    )
    .expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_autodiff_geometry_lean.sh")
        .env("ELAN_TOOLCHAIN", "untrusted/ambient:toolchain")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("PATH", &path)
        .env("ELAN_TOOLCHAIN_CAPTURE", &toolchain_capture)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(toolchain_capture).expect("captured toolchain"),
        "leanprover/lean4:v4.32.1\n"
    );
}

#[test]
fn non_scoped_elan_home_is_rejected() {
    let fake_bin = TempDir::new().expect("fake lake directory");
    let fake_lake = fake_bin.path().join("lake");
    let untrusted_elan_home = TempDir::new().expect("untrusted Elan home");

    fs::write(&fake_lake, "#!/bin/sh\nexit 0\n").expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_autodiff_geometry_lean.sh")
        .env("ELAN_HOME", untrusted_elan_home.path())
        .env("PATH", fake_bin.path())
        .assert()
        .failure()
        .stderr("Autodiff Geometry Lean verification requires a task-scoped ELAN_HOME\n");
}

#[test]
fn symlinked_elan_home_must_resolve_under_scoped_root() {
    let fake_bin = TempDir::new().expect("fake lake directory");
    let fake_lake = fake_bin.path().join("lake");
    let untrusted_elan_home = TempDir::new().expect("untrusted Elan home");
    let symlink_parent = TempDir::new().expect("symlink parent");
    let symlinked_elan_home = symlink_parent.path().join("elan-link");

    symlink(untrusted_elan_home.path(), &symlinked_elan_home).expect("create Elan symlink");
    fs::write(&fake_lake, "#!/bin/sh\nexit 0\n").expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_autodiff_geometry_lean.sh")
        .env("ELAN_HOME", &symlinked_elan_home)
        .env("PATH", fake_bin.path())
        .assert()
        .failure()
        .stderr("Autodiff Geometry Lean verification requires a task-scoped ELAN_HOME\n");
}
