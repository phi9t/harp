use std::fs;
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
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let source_dir = temporary_project.path().join("TrainingDynamics");
    fs::create_dir(&source_dir).expect("create scoped Lean source directory");
    fs::write(
        source_dir.join("ProofHole.lean"),
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
        .arg("scripts/check_training_dynamics_lean.sh")
        .env(
            "HARP_TRAINING_DYNAMICS_PROJECT_ROOT",
            temporary_project.path(),
        )
        .env("PATH", path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("proof-placeholder text"));

    assert!(
        !lake_marker.exists(),
        "lake ran despite a scoped Lean proof hole: {assertion:?}"
    );
}
