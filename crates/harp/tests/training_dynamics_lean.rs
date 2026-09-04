use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::TempDir;

struct ScopedLeanFixture {
    _cache_root: TempDir,
    elan_home: PathBuf,
    lean_cache_root: PathBuf,
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

fn scoped_lean_fixture() -> ScopedLeanFixture {
    let cache_root = tempfile::tempdir().expect("XDG cache fixture");
    let lean_root = cache_root.path().join("harp/lean");
    let elan_home = lean_root.join("elan");
    let lean_cache_root = lean_root.join("lean-4.32.1");
    fs::create_dir_all(&elan_home).expect("create wrapper-owned Elan root");
    fs::create_dir_all(&lean_cache_root).expect("create wrapper-owned Lean cache root");
    ScopedLeanFixture {
        _cache_root: cache_root,
        elan_home,
        lean_cache_root,
    }
}

fn write_fake_mathlib_artifact(root: &Path, module: &str) {
    let artifact = root.join(module.replace('.', "/")).with_extension("olean");
    fs::create_dir_all(artifact.parent().expect("artifact parent"))
        .expect("create artifact parent");
    fs::write(artifact, b"olean-test-fixture").expect("write fake mathlib artifact");
}

fn write_fake_mathlib_artifacts(root: &Path, modules: &[&str]) {
    for module in modules {
        write_fake_mathlib_artifact(root, module);
    }
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
        .arg("--project-for-test")
        .arg(temporary_project.path())
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

#[test]
fn ambient_project_override_is_ignored() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let fixture = scoped_lean_fixture();
    let artifact_root = fixture
        .lean_cache_root
        .join("packages/mathlib/.lake/build/lib/lean");
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
    write_fake_mathlib_artifacts(
        &artifact_root,
        &[
            "Mathlib",
            "Mathlib.Algebra.BigOperators.Fin",
            "Mathlib.Data.Matrix.Basic",
        ],
    );

    fs::write(&fake_lake, "#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\nexit 0\n")
        .expect("write fake lake");
    let mut permissions = fs::metadata(&fake_lake)
        .expect("read fake lake permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_lake, permissions).expect("make fake lake executable");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_training_dynamics_lean.sh")
        .env(
            "HARP_TRAINING_DYNAMICS_PROJECT_ROOT",
            temporary_project.path(),
        )
        .env("ELAN_HOME", &fixture.elan_home)
        .env("HARP_LEAN_CACHE_ROOT", &fixture.lean_cache_root)
        .env("PATH", path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .success();

    assert!(
        lake_marker.exists(),
        "lake did not run against the canonical project root"
    );
}
