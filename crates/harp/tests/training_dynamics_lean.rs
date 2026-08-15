use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
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
        .arg("scripts/check_training_dynamics_lean.sh")
        .env("PATH", empty_path.path())
        .assert()
        .failure()
        .stderr("Training Dynamics Lean toolchain is unavailable\n");
}

#[test]
fn proof_holes_are_rejected_before_lake_runs() {
    let project_root = repo_root().join("formalization/training_dynamics");
    let mut scoped_source = tempfile::Builder::new()
        .prefix("proof_hole_fixture_")
        .suffix(".lean")
        .tempfile_in(project_root.join("TrainingDynamics"))
        .expect("create scoped proof-hole fixture");
    let fake_bin = TempDir::new().expect("fake lake directory");
    let lake_marker = fake_bin.path().join("lake-was-called");
    let fake_lake = fake_bin.path().join("lake");
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake lake");

    scoped_source
        .write_all(b"theorem proof_hole_fixture : True := by sorry\n")
        .expect("write scoped proof-hole fixture");
    fs::write(&fake_lake, "#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\nexit 0\n")
        .expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    let assertion = Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_training_dynamics_lean.sh")
        .env("PATH", path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("proof hole"));

    assert!(
        !lake_marker.exists(),
        "lake ran despite a scoped Lean proof hole: {assertion:?}"
    );
}
