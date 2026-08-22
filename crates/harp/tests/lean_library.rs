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
fn shared_wrapper_builds_focused_lake_target_and_reports_timing() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$LAKE_ARGS\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");
    let artifact_root = scoped_elan_home.path().join("fake-mathlib-artifacts");
    write_fake_mathlib_artifacts(
        &artifact_root,
        &[
            "Mathlib.Algebra.BigOperators.Fin",
            "Mathlib.Data.Matrix.Basic",
        ],
    );

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("AutodiffGeometry")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("HARP_LEAN_MATHLIB_ARTIFACT_ROOT", &artifact_root)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=AutodiffGeometry")
                .and(predicate::str::contains("[lean] root=formalization/lean"))
                .and(predicate::str::contains("[lean] scan_seconds="))
                .and(predicate::str::contains("[lean] cache_seconds="))
                .and(predicate::str::contains("[lean] lake_seconds="))
                .and(predicate::str::contains("[lean] total_seconds="))
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "--try-cache build AutodiffGeometry\n"
    );
}

#[test]
fn shared_wrapper_builds_crouzeix_focused_target() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$LAKE_ARGS\"\n");
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
        "--try-cache build Crouzeix\n"
    );
}

#[test]
fn shared_wrapper_builds_ls_only_cached_target_without_scanning_other_providers() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let ls_dir = temporary_project.path().join("Crouzeix/LoristSchwenninger");
    let neutral_dir = temporary_project.path().join("CrouzeixConjecture");
    let jin_dir = temporary_project.path().join("Crouzeix/Jin");
    let harp_dir = temporary_project.path().join("Crouzeix/Harp");
    for directory in [&ls_dir, &neutral_dir, &jin_dir, &harp_dir] {
        fs::create_dir_all(directory).expect("create Lean fixture directory");
    }
    fs::write(
        temporary_project
            .path()
            .join("CrouzeixLoristSchwenninger.lean"),
        "import Crouzeix.LoristSchwenninger.Consequences\n",
    )
    .expect("write LS aggregate fixture");
    fs::write(
        ls_dir.join("Consequences.lean"),
        "import CrouzeixConjecture.NeutralDependency\n",
    )
    .expect("write LS consequence fixture");
    fs::write(
        neutral_dir.join("NeutralDependency.lean"),
        "/-\nimport Crouzeix.Jin.Terminal\nimport Crouzeix.Harp.Consequences\n-/\ntheorem neutral_dependency_fixture : True := by trivial\n",
    )
    .expect("write provider-neutral fixture");
    fs::write(
        ls_dir.join("Unrelated.lean"),
        "theorem unrelated_ls_fixture_must_not_be_scanned : True := by sorry\n",
    )
    .expect("write unrelated LS fixture");
    fs::write(
        jin_dir.join("Terminal.lean"),
        "theorem jin_fixture_must_not_be_scanned : True := by sorry\n",
    )
    .expect("write Jin fixture");
    fs::write(
        harp_dir.join("Consequences.lean"),
        "theorem harp_fixture_must_not_be_scanned : True := by admit\n",
    )
    .expect("write Harp fixture");

    let (fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$LAKE_ARGS\"\n");
    let fake_grep = fake_bin.path().join("grep");
    fs::write(
        &fake_grep,
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$GREP_ARGS\"\nexec /usr/bin/grep \"$@\"\n",
    )
    .expect("write recording grep");
    let mut permissions = fs::metadata(&fake_grep)
        .expect("read fake grep permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_grep, permissions).expect("make fake grep executable");

    let lake_args = temporary_project.path().join("lake-args");
    let grep_args = temporary_project.path().join("grep-args");
    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixLoristSchwenninger")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("LAKE_ARGS", &lake_args)
        .env("GREP_ARGS", &grep_args)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixLoristSchwenninger")
                .and(predicate::str::contains("[lean] root=formalization/lean"))
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "--try-cache build CrouzeixLoristSchwenninger\n"
    );
    let grep_invocations = fs::read_to_string(grep_args).expect("read grep arguments");
    let canonical_project =
        fs::canonicalize(temporary_project.path()).expect("canonicalize temporary Lean project");
    let mut scanned_sources = grep_invocations
        .lines()
        .filter_map(|line| line.strip_prefix("-n -H sorry "))
        .collect::<Vec<_>>();
    scanned_sources.sort_unstable();
    let mut expected_sources = [
        canonical_project
            .join("CrouzeixLoristSchwenninger.lean")
            .to_string_lossy()
            .into_owned(),
        canonical_project
            .join("Crouzeix/LoristSchwenninger/Consequences.lean")
            .to_string_lossy()
            .into_owned(),
        canonical_project
            .join("CrouzeixConjecture/NeutralDependency.lean")
            .to_string_lossy()
            .into_owned(),
    ];
    expected_sources.sort_unstable();
    assert_eq!(
        scanned_sources,
        expected_sources
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        "LS-only scan did not match the exact import closure"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/Jin/"),
        "Jin source entered the LS-only scan: {grep_invocations}"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/Harp/"),
        "Harp source entered the LS-only scan: {grep_invocations}"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/LoristSchwenninger/Unrelated.lean"),
        "non-closure LS source entered the LS-only scan: {grep_invocations}"
    );
}

#[test]
fn ls_only_wrapper_rejects_a_missing_provider_neutral_import_before_lake_runs() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let ls_dir = temporary_project.path().join("Crouzeix/LoristSchwenninger");
    fs::create_dir_all(&ls_dir).expect("create LS fixture directory");
    fs::write(
        temporary_project
            .path()
            .join("CrouzeixLoristSchwenninger.lean"),
        "import Crouzeix.LoristSchwenninger.Consequences\n",
    )
    .expect("write LS aggregate fixture");
    fs::write(
        ls_dir.join("Consequences.lean"),
        "import CrouzeixConjecture.MissingDependency\n",
    )
    .expect("write LS consequence fixture");
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixLoristSchwenninger")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("LAKE_CALLED_FILE", &lake_marker)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Crouzeix Lorist--Schwenninger Lean source scan failed").and(
                predicate::str::contains(
                    "missing local import CrouzeixConjecture.MissingDependency",
                ),
            ),
        )
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixLoristSchwenninger")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite a missing provider-neutral dependency"
    );
}

#[test]
fn ls_only_wrapper_rejects_an_import_of_another_terminal_provider() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let ls_dir = temporary_project.path().join("Crouzeix/LoristSchwenninger");
    fs::create_dir_all(&ls_dir).expect("create LS fixture directory");
    fs::write(
        temporary_project
            .path()
            .join("CrouzeixLoristSchwenninger.lean"),
        "import Crouzeix.LoristSchwenninger.Consequences\n",
    )
    .expect("write LS aggregate fixture");
    fs::write(
        ls_dir.join("Consequences.lean"),
        "import Crouzeix.Jin.Terminal\n",
    )
    .expect("write provider-crossing fixture");
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixLoristSchwenninger")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("LAKE_CALLED_FILE", &lake_marker)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rejected provider import Crouzeix.Jin.Terminal",
        ))
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixLoristSchwenninger")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite an import of another terminal provider"
    );
}

#[test]
fn ls_only_wrapper_rejects_a_harp_terminal_provider_import() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let ls_dir = temporary_project.path().join("Crouzeix/LoristSchwenninger");
    fs::create_dir_all(&ls_dir).expect("create LS fixture directory");
    fs::write(
        temporary_project
            .path()
            .join("CrouzeixLoristSchwenninger.lean"),
        "import Crouzeix.LoristSchwenninger.Consequences\n",
    )
    .expect("write LS aggregate fixture");
    fs::write(
        ls_dir.join("Consequences.lean"),
        "import Crouzeix.Harp.Consequences\n",
    )
    .expect("write provider-crossing fixture");
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixLoristSchwenninger")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("LAKE_CALLED_FILE", &lake_marker)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rejected provider import Crouzeix.Harp.Consequences",
        ))
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixLoristSchwenninger")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite a Harp terminal provider import"
    );
}

#[test]
fn ls_only_wrapper_preflights_public_mathlib_imports_before_lake() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_marker = scoped_elan_home.path().join("lake-was-called");
    let empty_artifact_root = scoped_elan_home.path().join("empty-mathlib-artifacts");
    fs::create_dir_all(&empty_artifact_root).expect("create empty artifact root");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixLoristSchwenninger")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_CALLED_FILE", &lake_marker)
        .env("HARP_LEAN_MATHLIB_ARTIFACT_ROOT", &empty_artifact_root)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Lean dependency cache is missing or invalid")
                .and(predicate::str::contains(
                    "Mathlib/Analysis/LocallyConvex/Separation.olean",
                ))
                .and(predicate::str::contains(
                    "Lean verification refuses to rebuild common dependencies",
                )),
        )
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixLoristSchwenninger")
                .and(predicate::str::contains("[lean] outcome=failed"))
                .and(predicate::str::contains("[lean] failure_stage=cache")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite a missing public Mathlib dependency cache"
    );
}

#[test]
fn ls_only_target_is_not_part_of_default_or_all_builds() {
    let lakefile = fs::read_to_string(repo_root().join("formalization/lean/lakefile.toml"))
        .expect("read Lake configuration");
    let default_targets = lakefile
        .split_once("defaultTargets = [")
        .and_then(|(_, tail)| tail.split_once(']'))
        .map(|(targets, _)| targets)
        .expect("parse defaultTargets");
    assert!(
        !default_targets.contains("CrouzeixLoristSchwenninger"),
        "receipt-only target entered defaultTargets: {default_targets}"
    );

    let wrapper = fs::read_to_string(repo_root().join("scripts/check_lean_library.sh"))
        .expect("read Lean wrapper");
    let all_target_loop = wrapper
        .split_once("for library in ")
        .and_then(|(_, tail)| tail.split_once("; do"))
        .map(|(targets, _)| targets)
        .expect("parse all-target library list");
    assert!(
        !all_target_loop.contains("CrouzeixLoristSchwenninger"),
        "receipt-only target entered all: {all_target_loop}"
    );
}

#[test]
fn shared_wrapper_builds_all_targets_from_shared_root_once() {
    let (_fake_bin, path) =
        fake_lake_bin("#!/bin/sh\npwd > \"$LAKE_PWD\"\nprintf '%s\n' \"$*\" >> \"$LAKE_ARGS\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");
    let lake_pwd = scoped_elan_home.path().join("lake-pwd");
    let artifact_root = scoped_elan_home.path().join("fake-mathlib-artifacts");
    write_fake_mathlib_artifacts(
        &artifact_root,
        &[
            "Mathlib",
            "Mathlib.Algebra.BigOperators.Fin",
            "Mathlib.Algebra.BigOperators.Ring.Finset",
            "Mathlib.Algebra.Module.Submodule.Ker",
            "Mathlib.Algebra.Polynomial.AlgebraMap",
            "Mathlib.Algebra.Polynomial.Eval.Defs",
            "Mathlib.Analysis.Complex.Basic",
            "Mathlib.Analysis.Convex.Caratheodory",
            "Mathlib.Analysis.Convex.Integral",
            "Mathlib.Analysis.Convex.Topology",
            "Mathlib.Analysis.CStarAlgebra.Matrix",
            "Mathlib.Analysis.InnerProductSpace.Adjoint",
            "Mathlib.Analysis.InnerProductSpace.PiL2",
            "Mathlib.Analysis.InnerProductSpace.Rayleigh",
            "Mathlib.Analysis.Normed.Module.FiniteDimension",
            "Mathlib.Analysis.SpecialFunctions.ContinuousFunctionalCalculus.Rpow.Basic",
            "Mathlib.Analysis.SpecialFunctions.ContinuousFunctionalCalculus.Rpow.Isometric",
            "Mathlib.Analysis.SpecificLimits.Basic",
            "Mathlib.Analysis.SpecificLimits.Normed",
            "Mathlib.Data.Matrix.Basic",
            "Mathlib.Data.Matrix.Mul",
            "Mathlib.Data.Real.Basic",
            "Mathlib.LinearAlgebra.AffineSpace.FiniteDimensional",
            "Mathlib.LinearAlgebra.Complex.FiniteDimensional",
            "Mathlib.LinearAlgebra.Matrix.DotProduct",
            "Mathlib.LinearAlgebra.Matrix.PosDef",
            "Mathlib.LinearAlgebra.Matrix.ToLin",
            "Mathlib.MeasureTheory.Function.Holder",
            "Mathlib.MeasureTheory.Function.L2Space",
            "Mathlib.MeasureTheory.Function.LpSpace.ContinuousFunctions",
            "Mathlib.MeasureTheory.Function.LpSeminorm.Count",
            "Mathlib.MeasureTheory.Integral.Average",
            "Mathlib.MeasureTheory.Integral.Bochner.SumMeasure",
            "Mathlib.MeasureTheory.SpecificCodomains.Pi",
            "Mathlib.MeasureTheory.Measure.Count",
            "Mathlib.Tactic.FieldSimp",
            "Mathlib.Tactic.Linarith",
            "Mathlib.Tactic.Ring",
        ],
    );

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("all")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("LAKE_PWD", &lake_pwd)
        .env("HARP_LEAN_MATHLIB_ARTIFACT_ROOT", &artifact_root)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=all")
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "--try-cache build\n"
    );
    assert_eq!(
        fs::read_to_string(lake_pwd).expect("read lake working directory"),
        format!("{}\n", repo_root().join("formalization/lean").display())
    );
}

#[test]
fn shared_wrapper_fails_before_build_when_mathlib_cache_is_missing() {
    let (_fake_bin, path) = fake_lake_bin(
        r#"#!/bin/sh
printf '%s\n' "$*" >> "$LAKE_ARGS"
if [ "$1" = "--try-cache" ] && [ "$2" = "build" ]; then
  : > "$LAKE_BUILD_MARKER"
fi
"#,
    );
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");
    let lake_build_marker = scoped_elan_home.path().join("lake-build-was-called");
    let artifact_root = scoped_elan_home.path().join("fake-mathlib-artifacts");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("AutodiffGeometry")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("LAKE_BUILD_MARKER", &lake_build_marker)
        .env("HARP_LEAN_MATHLIB_ARTIFACT_ROOT", &artifact_root)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Lean verification refuses to rebuild common dependencies",
        ))
        .stdout(
            predicate::str::contains("[lean] outcome=failed")
                .and(predicate::str::contains("[lean] failure_stage=cache")),
        );

    assert!(
        !lake_args.exists(),
        "lake ran despite a missing common dependency cache"
    );
    assert!(
        !lake_build_marker.exists(),
        "lake build ran despite a missing common dependency cache"
    );
}

#[test]
fn shared_wrapper_rejects_invalid_mathlib_cache_artifacts() {
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");
    let artifact_root = scoped_elan_home.path().join("fake-mathlib-artifacts");
    let invalid_artifact = artifact_root
        .join("Mathlib/Algebra/BigOperators/Fin")
        .with_extension("olean");
    fs::create_dir_all(invalid_artifact.parent().expect("invalid artifact parent"))
        .expect("create invalid artifact parent");
    fs::write(&invalid_artifact, b"").expect("write invalid fake mathlib artifact");
    write_fake_mathlib_artifact(&artifact_root, "Mathlib.Data.Matrix.Basic");
    let (_fake_bin, path) = fake_lake_bin(
        r#"#!/bin/sh
printf '%s\n' "$*" >> "$LAKE_ARGS"
"#,
    );

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("AutodiffGeometry")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("HARP_LEAN_MATHLIB_ARTIFACT_ROOT", &artifact_root)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Lean dependency cache is missing or invalid",
        ))
        .stdout(predicate::str::contains("[lean] failure_stage=cache"));

    assert!(
        !lake_args.exists(),
        "lake ran despite an invalid common dependency cache"
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

#[test]
fn all_rejects_crouzeix_conjecture_admit_before_lake_runs() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    let crouzeix_conjecture_dir = temporary_project.path().join("CrouzeixConjecture");
    fs::create_dir_all(&crouzeix_conjecture_dir)
        .expect("create CrouzeixConjecture fixture directory");
    fs::write(
        crouzeix_conjecture_dir.join("Admit.lean"),
        "theorem crouzeix_conjecture_admit_fixture : True := by admit\n",
    )
    .expect("write CrouzeixConjecture admit fixture");
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
        "lake ran despite a CrouzeixConjecture admit fixture"
    );
}
