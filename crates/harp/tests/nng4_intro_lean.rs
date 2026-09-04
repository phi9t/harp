use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
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
        .arg("scripts/check_nng4_intro_lean.sh")
        .env("PATH", empty_path.path())
        .assert()
        .failure()
        .stderr("NNG4 introductory Lean toolchain is unavailable\n");
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
        .arg("scripts/check_nng4_intro_lean.sh")
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
    let fixture = scoped_lean_fixture();
    let artifact_root = fixture
        .lean_cache_root
        .join("packages/mathlib/.lake/build/lib/lean");
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
        .arg("scripts/check_nng4_intro_lean.sh")
        .env("HARP_NNG4_INTRO_PROJECT_ROOT", temporary_project.path())
        .env("ELAN_HOME", &fixture.elan_home)
        .env("HARP_LEAN_CACHE_ROOT", &fixture.lean_cache_root)
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
    let fixture = scoped_lean_fixture();
    let artifact_root = fixture
        .lean_cache_root
        .join("packages/mathlib/.lake/build/lib/lean");
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
        .arg("scripts/check_nng4_intro_lean.sh")
        .env("ELAN_TOOLCHAIN", "untrusted/ambient:toolchain")
        .env("ELAN_HOME", &fixture.elan_home)
        .env("HARP_LEAN_CACHE_ROOT", &fixture.lean_cache_root)
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
    let missing_elan_cache_root = fake_bin.path().join("missing-elan/harp/lean/lean-4.32.1");
    fs::create_dir_all(fake_bin.path().join("missing-elan/harp/lean/elan"))
        .expect("create sibling Elan root");
    fs::create_dir_all(&missing_elan_cache_root).expect("create fake cache root");

    let assertion = Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_nng4_intro_lean.sh")
        .env_remove("ELAN_HOME")
        .env("HARP_LEAN_CACHE_ROOT", &missing_elan_cache_root)
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
    fs::create_dir_all(home_dir.join(".cache/harp/lean/elan")).expect("create sibling Elan root");
    fs::create_dir_all(home_dir.join(".cache/harp/lean/lean-4.32.1"))
        .expect("create fake cache root");
    let assertion = Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_nng4_intro_lean.sh")
        .env("HOME", &home_dir)
        .env("ELAN_HOME", &home_elan)
        .env(
            "HARP_LEAN_CACHE_ROOT",
            home_dir.join(".cache/harp/lean/lean-4.32.1"),
        )
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("task-scoped ELAN_HOME"));

    assert!(
        !lake_marker.exists(),
        "lake ran with a home-scoped ELAN_HOME: {assertion:?}"
    );

    let home_child = home_dir.join("child");
    fs::create_dir(&home_child).expect("create home child directory");
    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_nng4_intro_lean.sh")
        .env("HOME", &home_dir)
        .env("ELAN_HOME", home_child.join("../.elan"))
        .env(
            "HARP_LEAN_CACHE_ROOT",
            home_dir.join(".cache/harp/lean/lean-4.32.1"),
        )
        .env("PATH", &path)
        .env("LAKE_CALLED_FILE", &lake_marker)
        .assert()
        .failure()
        .stderr(predicates::str::contains("task-scoped ELAN_HOME"));

    let home_link = fake_bin.path().join("home-elan-link");
    symlink(&home_elan, &home_link).expect("create home Elan symlink");
    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_nng4_intro_lean.sh")
        .env("HOME", &home_dir)
        .env("ELAN_HOME", &home_link)
        .env(
            "HARP_LEAN_CACHE_ROOT",
            home_dir.join(".cache/harp/lean/lean-4.32.1"),
        )
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
