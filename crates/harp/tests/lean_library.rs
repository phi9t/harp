use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
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

fn canonical_mathlib_artifact_root() -> PathBuf {
    repo_root()
        .join("formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean")
        .canonicalize()
        .expect("canonical Mathlib artifact root must exist under formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean")
}

struct LeanEnvFixture {
    root: TempDir,
    lean_cache_root: PathBuf,
    allowed_elan_home: PathBuf,
    scoped_elan_home: TempDir,
}

impl LeanEnvFixture {
    fn new() -> Self {
        let root = TempDir::new().expect("create Lean env fixture root");
        let lean_cache_root = root.path().join("harp/lean/lean-4.32.1");
        fs::create_dir_all(&lean_cache_root).expect("create wrapper-owned Lean cache root");
        let allowed_elan_home = root.path().join("harp/lean/elan");
        fs::create_dir_all(&allowed_elan_home).expect("create wrapper-owned Elan root");
        let scoped_elan_home = Builder::new()
            .prefix("test-")
            .tempdir_in(&allowed_elan_home)
            .expect("create scoped Elan home");
        Self {
            root,
            lean_cache_root,
            allowed_elan_home,
            scoped_elan_home,
        }
    }

    fn root(&self) -> &Path {
        self.root.path()
    }

    fn lean_cache_root(&self) -> &Path {
        &self.lean_cache_root
    }

    fn default_mathlib_artifact_root(&self) -> PathBuf {
        self.lean_cache_root
            .join("packages/mathlib/.lake/build/lib/lean")
    }

    fn scoped_elan_home(&self) -> &Path {
        self.scoped_elan_home.path()
    }

    fn apply<'a>(&self, command: &'a mut Command) -> &'a mut Command {
        command
            .env("HARP_LEAN_CACHE_ROOT", self.lean_cache_root())
            .env("HARP_ELAN_HOME", &self.allowed_elan_home)
            .env("ELAN_HOME", self.scoped_elan_home())
    }
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

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write executable fixture");
    let mut permissions = fs::metadata(path)
        .expect("read executable permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("make executable fixture");
}

fn fake_python_and_lake_bin(python_script: &str, lake_script: &str) -> (TempDir, OsString) {
    let fake_bin = TempDir::new().expect("fake tool directory");
    write_executable(&fake_bin.path().join("python3"), python_script);
    write_executable(&fake_bin.path().join("lake"), lake_script);
    let path = std::env::join_paths([fake_bin.path(), Path::new("/usr/bin"), Path::new("/bin")])
        .expect("construct PATH with fake python and lake");
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

fn write_fake_all_mathlib_artifacts(root: &Path) {
    write_fake_mathlib_artifacts(
        root,
        &[
            "Mathlib",
            "Mathlib.Algebra.BigOperators.Fin",
            "Mathlib.Algebra.BigOperators.Ring.Finset",
            "Mathlib.Algebra.Module.Submodule.Ker",
            "Mathlib.Algebra.Polynomial.AlgebraMap",
            "Mathlib.Algebra.Polynomial.Eval.Defs",
            "Mathlib.Analysis.Calculus.Deriv.Mul",
            "Mathlib.Analysis.Calculus.TangentCone.Real",
            "Mathlib.Analysis.Complex.Basic",
            "Mathlib.Analysis.Convex.Caratheodory",
            "Mathlib.Analysis.Convex.Integral",
            "Mathlib.Analysis.Convex.Topology",
            "Mathlib.Analysis.CStarAlgebra.ContinuousFunctionalCalculus.Basic",
            "Mathlib.Analysis.CStarAlgebra.ContinuousLinearMap",
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
            "Mathlib.LinearAlgebra.Basis.Defs",
            "Mathlib.LinearAlgebra.Basis.VectorSpace",
            "Mathlib.LinearAlgebra.Complex.FiniteDimensional",
            "Mathlib.LinearAlgebra.Dimension.Finite",
            "Mathlib.LinearAlgebra.Dual.Lemmas",
            "Mathlib.LinearAlgebra.FiniteDimensional.Lemmas",
            "Mathlib.LinearAlgebra.Isomorphisms",
            "Mathlib.LinearAlgebra.Matrix.Charpoly.Basic",
            "Mathlib.LinearAlgebra.Matrix.Charpoly.Coeff",
            "Mathlib.LinearAlgebra.Matrix.DotProduct",
            "Mathlib.LinearAlgebra.Matrix.PosDef",
            "Mathlib.LinearAlgebra.Matrix.ToLin",
            "Mathlib.LinearAlgebra.Matrix.Trace",
            "Mathlib.LinearAlgebra.Quotient.Basic",
            "Mathlib.Lean.CoreM",
            "Mathlib.MeasureTheory.Function.Holder",
            "Mathlib.MeasureTheory.Function.L2Space",
            "Mathlib.MeasureTheory.Function.LpSpace.ContinuousFunctions",
            "Mathlib.MeasureTheory.Function.LpSeminorm.Count",
            "Mathlib.MeasureTheory.Integral.Average",
            "Mathlib.MeasureTheory.Integral.Bochner.SumMeasure",
            "Mathlib.MeasureTheory.Integral.CircleIntegral",
            "Mathlib.MeasureTheory.SpecificCodomains.Pi",
            "Mathlib.MeasureTheory.Measure.Count",
            "Mathlib.NumberTheory.Real.Irrational",
            "Mathlib.Tactic",
            "Mathlib.Tactic.FieldSimp",
            "Mathlib.Tactic.Linarith",
            "Mathlib.Tactic.Ring",
            "Mathlib.Topology.Separation.Connected",
            "Mathlib.Util.AssertNoSorry",
        ],
    );
}

fn write_fake_crouzeix_mathlib_artifacts(root: &Path) {
    write_fake_mathlib_artifacts(
        root,
        &[
            "Mathlib.Analysis.Convex.Caratheodory",
            "Mathlib.Analysis.Convex.Integral",
            "Mathlib.Analysis.Convex.Topology",
            "Mathlib.Analysis.InnerProductSpace.Adjoint",
            "Mathlib.Analysis.InnerProductSpace.Rayleigh",
            "Mathlib.Analysis.Normed.Module.FiniteDimension",
            "Mathlib.Analysis.SpecialFunctions.ContinuousFunctionalCalculus.Rpow.Basic",
            "Mathlib.Analysis.SpecialFunctions.ContinuousFunctionalCalculus.Rpow.Isometric",
            "Mathlib.Analysis.SpecificLimits.Basic",
            "Mathlib.LinearAlgebra.AffineSpace.FiniteDimensional",
            "Mathlib.LinearAlgebra.Complex.FiniteDimensional",
            "Mathlib.MeasureTheory.Function.Holder",
            "Mathlib.MeasureTheory.Function.L2Space",
            "Mathlib.MeasureTheory.Function.LpSpace.ContinuousFunctions",
            "Mathlib.MeasureTheory.Function.LpSeminorm.Count",
            "Mathlib.MeasureTheory.Integral.Average",
            "Mathlib.MeasureTheory.Integral.Bochner.SumMeasure",
            "Mathlib.MeasureTheory.Measure.Count",
            "Mathlib.MeasureTheory.SpecificCodomains.Pi",
            "Mathlib.Tactic.FieldSimp",
            "Mathlib.Tactic.Linarith",
            "Mathlib.Tactic.Ring",
        ],
    );
}

fn write_lean_module(root: &Path, module: &str, source: &str) {
    let path = root.join(module.replace('.', "/")).with_extension("lean");
    fs::create_dir_all(path.parent().expect("Lean module parent"))
        .expect("create Lean module parent");
    fs::write(path, source).expect("write Lean module fixture");
}

#[test]
fn shared_wrapper_builds_focused_lake_target_and_reports_timing() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$LAKE_ARGS\"\n");
    let fixture = LeanEnvFixture::new();
    let lake_args = fixture.root().join("lake-args");
    let artifact_root = fixture.root().join("fake-mathlib-artifacts");
    write_fake_mathlib_artifacts(
        &artifact_root,
        &[
            "Mathlib.Algebra.BigOperators.Fin",
            "Mathlib.Data.Matrix.Basic",
        ],
    );

    let mut command = Command::new("/bin/sh");
    fixture
        .apply(
            command
                .current_dir(repo_root())
                .arg("scripts/check_lean_library.sh")
                .arg("AutodiffGeometry"),
        )
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
    let fixture = LeanEnvFixture::new();
    let lake_args = fixture.root().join("lake-args");
    write_fake_crouzeix_mathlib_artifacts(&fixture.default_mathlib_artifact_root());

    let mut command = Command::new("/bin/sh");
    fixture
        .apply(
            command
                .current_dir(repo_root())
                .arg("scripts/check_lean_library.sh")
                .arg("Crouzeix"),
        )
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
fn shared_wrapper_builds_crouzeix_textbook_target() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$LAKE_ARGS\"\n");
    let fixture = LeanEnvFixture::new();
    let lake_args = fixture.root().join("lake-args");
    write_fake_all_mathlib_artifacts(&fixture.default_mathlib_artifact_root());

    fixture
        .apply(&mut Command::new("/bin/sh"))
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixTextbook")
        .env("LAKE_ARGS", &lake_args)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixTextbook")
                .and(predicate::str::contains("[lean] root=formalization/lean"))
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "--try-cache build CrouzeixTextbook\n"
    );
}

#[test]
fn textbook_receipt_exporter_writes_the_exact_path_without_stdout_redirection() {
    let (_fake_bin, path) = fake_lake_bin(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$LAKE_ARGS\"\nif [ \"$1\" = env ]; then\n  printf '%s' \"$RECEIPT_PAYLOAD\" > \"$5\"\n  chmod 600 \"$5\"\n  printf '%s\\n' exporter-stdout-sentinel\nfi\n",
    );
    let fixture = LeanEnvFixture::new();
    let lake_args = fixture.root().join("lake-args");
    write_fake_all_mathlib_artifacts(&fixture.default_mathlib_artifact_root());
    let receipt_dir = tempfile::tempdir().expect("receipt output directory");
    fs::set_permissions(receipt_dir.path(), fs::Permissions::from_mode(0o700))
        .expect("make receipt output directory private");
    let receipt_path = receipt_dir.path().join("receipt.json");
    let ambient_temp = tempfile::tempdir().expect("ambient temporary directory");
    let receipt_payload = "{\"schema_version\":\"test\"}\n";

    fixture
        .apply(&mut Command::new("/bin/sh"))
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .args([
            "CrouzeixTextbook",
            "--receipt-output",
            receipt_path.to_str().expect("UTF-8 receipt path"),
        ])
        .env("LAKE_ARGS", &lake_args)
        .env("RECEIPT_PAYLOAD", receipt_payload)
        .env("TMPDIR", ambient_temp.path())
        .env("PATH", path)
        .assert()
        .success()
        .stdout(predicate::str::contains("exporter-stdout-sentinel"));

    assert_eq!(
        fs::read_to_string(&receipt_path).expect("read exported receipt"),
        receipt_payload
    );
    let metadata = fs::metadata(&receipt_path).expect("receipt metadata");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    assert_eq!(metadata.nlink(), 1);
    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        format!(
            "--try-cache build CrouzeixTextbook\nenv lean --run CrouzeixTextbook.lean {}\n",
            receipt_dir
                .path()
                .canonicalize()
                .expect("canonical receipt directory")
                .join("receipt.json")
                .display()
        )
    );
}

#[test]
fn textbook_receipt_rejects_public_parent_without_consulting_ambient_tmpdir() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$LAKE_ARGS\"\n");
    let fixture = LeanEnvFixture::new();
    let lake_args = fixture.root().join("lake-args");
    write_fake_all_mathlib_artifacts(&fixture.default_mathlib_artifact_root());
    let public_parent = tempfile::tempdir().expect("public receipt directory");
    fs::set_permissions(public_parent.path(), fs::Permissions::from_mode(0o755))
        .expect("make receipt directory public");
    let output = public_parent.path().join("receipt.json");

    fixture
        .apply(&mut Command::new("/bin/sh"))
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .args([
            "CrouzeixTextbook",
            "--receipt-output",
            output.to_str().expect("UTF-8 output"),
        ])
        .env("LAKE_ARGS", &lake_args)
        .env("TMPDIR", repo_root())
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("caller-owned and private"));
    assert!(!output.exists());
    assert!(!lake_args.exists());
}

#[cfg(unix)]
#[test]
fn textbook_receipt_rejects_existing_targets_and_symlinked_parents_before_lake() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nexit 99\n");
    let fixture = LeanEnvFixture::new();
    let private_parent = tempfile::tempdir().expect("private receipt directory");
    fs::set_permissions(private_parent.path(), fs::Permissions::from_mode(0o700))
        .expect("private receipt directory mode");
    let existing = private_parent.path().join("receipt.json");
    fs::write(&existing, "preserve").expect("existing receipt target");
    fixture
        .apply(&mut Command::new("/bin/sh"))
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .args([
            "CrouzeixTextbook",
            "--receipt-output",
            existing.to_str().expect("UTF-8 existing target"),
        ])
        .env("PATH", &path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must not already exist"));
    assert_eq!(fs::read_to_string(&existing).unwrap(), "preserve");

    let link_root = tempfile::tempdir().expect("parent link root");
    let linked_parent = link_root.path().join("linked");
    std::os::unix::fs::symlink(private_parent.path(), &linked_parent)
        .expect("symlink receipt parent");
    let linked_output = linked_parent.join("new.json");
    fixture
        .apply(&mut Command::new("/bin/sh"))
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .args([
            "CrouzeixTextbook",
            "--receipt-output",
            linked_output.to_str().expect("UTF-8 linked output"),
        ])
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must not be a symlink"));
    assert!(!private_parent.path().join("new.json").exists());
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
fn shared_wrapper_builds_jin_only_cached_target_without_scanning_other_providers() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    write_lean_module(
        temporary_project.path(),
        "CrouzeixJin",
        "import Crouzeix.Jin.Terminal\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Jin.Terminal",
        "import CrouzeixConjecture.NeutralDependency\n",
    );
    write_lean_module(
        temporary_project.path(),
        "CrouzeixConjecture.NeutralDependency",
        "/-\nimport Crouzeix.LoristSchwenninger.Consequences\nimport Crouzeix.Harp.Consequences\n-/\n\
theorem neutral_dependency_fixture : True := by trivial\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.LoristSchwenninger.Consequences",
        "theorem ls_fixture_must_not_be_scanned : True := by sorry\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Harp.Consequences",
        "theorem harp_fixture_must_not_be_scanned : True := by admit\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Jin.Unrelated",
        "theorem unrelated_jin_fixture_must_not_be_scanned : True := by sorry\n",
    );

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
        .arg("CrouzeixJin")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("LAKE_ARGS", &lake_args)
        .env("GREP_ARGS", &grep_args)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixJin")
                .and(predicate::str::contains("[lean] root=formalization/lean"))
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "--try-cache build CrouzeixJin\n"
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
            .join("CrouzeixJin.lean")
            .to_string_lossy()
            .into_owned(),
        canonical_project
            .join("Crouzeix/Jin/Terminal.lean")
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
        "Jin-only scan did not match the exact import closure"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/LoristSchwenninger/"),
        "LS source entered the Jin-only scan: {grep_invocations}"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/Harp/"),
        "Harp source entered the Jin-only scan: {grep_invocations}"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/Jin/Unrelated.lean"),
        "non-closure Jin source entered the Jin-only scan: {grep_invocations}"
    );
}

#[test]
fn jin_only_wrapper_rejects_a_direct_ls_provider_import() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    write_lean_module(
        temporary_project.path(),
        "CrouzeixJin",
        "import Crouzeix.Jin.Terminal\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Jin.Terminal",
        "import Crouzeix.LoristSchwenninger.Consequences\n",
    );
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixJin")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("LAKE_CALLED_FILE", &lake_marker)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rejected provider import Crouzeix.LoristSchwenninger.Consequences",
        ))
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixJin")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite a direct LS provider import"
    );
}

#[test]
fn jin_only_wrapper_rejects_a_transitive_harp_provider_import() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    write_lean_module(
        temporary_project.path(),
        "CrouzeixJin",
        "import Crouzeix.Jin.Terminal\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Jin.Terminal",
        "import CrouzeixConjecture.NeutralDependency\n",
    );
    write_lean_module(
        temporary_project.path(),
        "CrouzeixConjecture.NeutralDependency",
        "import Crouzeix.Harp.Consequences\n",
    );
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixJin")
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
            predicate::str::contains("[lean] target=CrouzeixJin")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite a transitive Harp provider import"
    );
}

#[test]
fn shared_wrapper_builds_harp_only_cached_target_without_scanning_terminal_ls_provider() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    write_lean_module(
        temporary_project.path(),
        "CrouzeixHarp",
        "import Crouzeix.Harp.Consequences\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Harp.Consequences",
        "import Crouzeix.Harp.MainTheorem\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Harp.MainTheorem",
        "import Crouzeix.LoristSchwenninger.Dilation\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.LoristSchwenninger.Dilation",
        "import CrouzeixConjecture.NeutralDependency\n",
    );
    write_lean_module(
        temporary_project.path(),
        "CrouzeixConjecture.NeutralDependency",
        "theorem neutral_dependency_fixture : True := by trivial\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.LoristSchwenninger.Consequences",
        "theorem ls_consequences_fixture_must_not_be_scanned : True := by sorry\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.LoristSchwenninger.MainTheorem",
        "theorem ls_terminal_fixture_must_not_be_scanned : True := by admit\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Jin.Terminal",
        "theorem jin_fixture_must_not_be_scanned : True := by sorry\n",
    );

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
        .arg("CrouzeixHarp")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("LAKE_ARGS", &lake_args)
        .env("GREP_ARGS", &grep_args)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixHarp")
                .and(predicate::str::contains("[lean] root=formalization/lean"))
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "--try-cache build CrouzeixHarp\n"
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
            .join("CrouzeixHarp.lean")
            .to_string_lossy()
            .into_owned(),
        canonical_project
            .join("Crouzeix/Harp/Consequences.lean")
            .to_string_lossy()
            .into_owned(),
        canonical_project
            .join("Crouzeix/Harp/MainTheorem.lean")
            .to_string_lossy()
            .into_owned(),
        canonical_project
            .join("Crouzeix/LoristSchwenninger/Dilation.lean")
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
        "Harp-only scan did not match the exact import closure"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/LoristSchwenninger/Consequences.lean"),
        "LS consequences source entered the Harp-only scan: {grep_invocations}"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/LoristSchwenninger/MainTheorem.lean"),
        "LS terminal source entered the Harp-only scan: {grep_invocations}"
    );
    assert!(
        !grep_invocations.contains("Crouzeix/Jin/"),
        "Jin source entered the Harp-only scan: {grep_invocations}"
    );
}

#[test]
fn harp_only_wrapper_rejects_a_direct_ls_terminal_provider_import() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    write_lean_module(
        temporary_project.path(),
        "CrouzeixHarp",
        "import Crouzeix.Harp.Consequences\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Harp.Consequences",
        "import Crouzeix.LoristSchwenninger.Consequences\n",
    );
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixHarp")
        .arg("--project-for-test")
        .arg(temporary_project.path())
        .env("LAKE_CALLED_FILE", &lake_marker)
        .env("PATH", path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rejected provider import Crouzeix.LoristSchwenninger.Consequences",
        ))
        .stdout(
            predicate::str::contains("[lean] target=CrouzeixHarp")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite a direct LS terminal provider import"
    );
}

#[test]
fn harp_only_wrapper_rejects_a_transitive_jin_provider_import() {
    let temporary_project = TempDir::new().expect("temporary Lean project");
    write_lean_module(
        temporary_project.path(),
        "CrouzeixHarp",
        "import Crouzeix.Harp.Consequences\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Harp.Consequences",
        "import Crouzeix.Harp.MainTheorem\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.Harp.MainTheorem",
        "import Crouzeix.LoristSchwenninger.Dilation\n",
    );
    write_lean_module(
        temporary_project.path(),
        "Crouzeix.LoristSchwenninger.Dilation",
        "import Crouzeix.Jin.Terminal\n",
    );
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\n: > \"$LAKE_CALLED_FILE\"\n");
    let lake_marker = temporary_project.path().join("lake-was-called");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("CrouzeixHarp")
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
            predicate::str::contains("[lean] target=CrouzeixHarp")
                .and(predicate::str::contains("[lean] failure_stage=scan")),
        );

    assert!(
        !lake_marker.exists(),
        "lake ran despite a transitive Jin provider import"
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
    let fixture = LeanEnvFixture::new();
    let lake_marker = fixture.root().join("lake-was-called");
    let empty_artifact_root = fixture.root().join("empty-mathlib-artifacts");
    fs::create_dir_all(&empty_artifact_root).expect("create empty artifact root");

    let mut command = Command::new("/bin/sh");
    fixture
        .apply(
            command
                .current_dir(repo_root())
                .arg("scripts/check_lean_library.sh")
                .arg("CrouzeixLoristSchwenninger"),
        )
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
    assert!(
        !default_targets.contains("CrouzeixJin"),
        "route-isolated Jin target entered defaultTargets: {default_targets}"
    );
    assert!(
        !default_targets.contains("CrouzeixHarp"),
        "route-isolated Harp target entered defaultTargets: {default_targets}"
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
    assert!(
        !all_target_loop.contains("CrouzeixJin"),
        "route-isolated Jin target entered all: {all_target_loop}"
    );
    assert!(
        !all_target_loop.contains("CrouzeixHarp"),
        "route-isolated Harp target entered all: {all_target_loop}"
    );
}

#[test]
fn shared_wrapper_builds_all_targets_from_shared_root_once() {
    let (_fake_bin, path) =
        fake_lake_bin("#!/bin/sh\npwd > \"$LAKE_PWD\"\nprintf '%s\n' \"$*\" >> \"$LAKE_ARGS\"\n");
    let fixture = LeanEnvFixture::new();
    let lake_args = fixture.root().join("lake-args");
    let lake_pwd = fixture.root().join("lake-pwd");
    let artifact_root = fixture.root().join("fake-mathlib-artifacts");
    write_fake_all_mathlib_artifacts(&artifact_root);

    let mut command = Command::new("/bin/sh");
    fixture
        .apply(
            command
                .current_dir(repo_root())
                .arg("scripts/check_lean_library.sh")
                .arg("all"),
        )
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
    let fixture = LeanEnvFixture::new();
    let lake_args = fixture.root().join("lake-args");
    let lake_build_marker = fixture.root().join("lake-build-was-called");
    let artifact_root = fixture.root().join("fake-mathlib-artifacts");

    let mut command = Command::new("/bin/sh");
    fixture
        .apply(
            command
                .current_dir(repo_root())
                .arg("scripts/check_lean_library.sh")
                .arg("AutodiffGeometry"),
        )
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
    let fixture = LeanEnvFixture::new();
    let lake_args = fixture.root().join("lake-args");
    let artifact_root = fixture.root().join("fake-mathlib-artifacts");
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

    let mut command = Command::new("/bin/sh");
    fixture
        .apply(
            command
                .current_dir(repo_root())
                .arg("scripts/check_lean_library.sh")
                .arg("AutodiffGeometry"),
        )
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
    let fixture = LeanEnvFixture::new();
    let lake_marker = fixture.root().join("lake-was-called");

    let mut command = Command::new("/bin/sh");
    fixture
        .apply(
            command
                .current_dir(repo_root())
                .arg("scripts/check_lean_library.sh")
                .arg("UnknownLibrary"),
        )
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

#[test]
fn route_wrappers_fail_closed_on_python_preflight_block_before_lake() {
    let cases = [
        ("CrouzeixJin", "jin"),
        ("CrouzeixLoristSchwenninger", "lorist-schwenninger"),
        ("CrouzeixHarp", "harp"),
    ];

    for (target, route) in cases {
        let fixture = LeanEnvFixture::new();
        let python_args = fixture.root().join(format!("{route}-python-args"));
        let lake_marker = fixture.root().join(format!("{route}-lake-marker"));
        let (fake_bin, path) = fake_python_and_lake_bin(
            "#!/bin/sh\nprintf '%s\n' \"$*\" > \"$PYTHON_ARGS\"\nprintf '%s\n' '{\"status\":\"blocked\",\"reason\":\"missing-mathlib-artifacts\",\"missing_artifacts\":[\"transitive-only.olean\"]}'\nexit 1\n",
            "#!/bin/sh\n: > \"$LAKE_MARKER\"\nexit 0\n",
        );

        let mut command = Command::new("/bin/sh");
        let assert = fixture
            .apply(
                command
                    .current_dir(repo_root())
                    .arg("scripts/check_lean_library.sh")
                    .arg(target),
            )
            .env("PYTHON_ARGS", &python_args)
            .env("LAKE_MARKER", &lake_marker)
            .env("PATH", &path)
            .assert();

        assert
            .failure()
            .stdout(
                predicate::str::contains(format!("[lean] target={target}"))
                    .and(predicate::str::contains("[lean] outcome=failed"))
                    .and(predicate::str::contains("[lean] failure_stage=preflight")),
            )
            .stderr(predicate::str::contains("missing-mathlib-artifacts"));

        assert_eq!(
            fs::read_to_string(&python_args).expect("read fake python args"),
            format!(
                "labs/crouzeix_proof_reproduction/proof_evidence.py preflight --route {route}\n"
            ),
            "wrapper did not call python preflight with the exact route mapping",
        );
        assert!(
            !lake_marker.exists(),
            "lake ran despite blocked python preflight for {target}"
        );
        drop(fake_bin);
    }
}

#[test]
fn route_wrappers_run_lake_once_after_successful_python_preflight() {
    let cases = [
        ("CrouzeixJin", "jin"),
        ("CrouzeixLoristSchwenninger", "lorist-schwenninger"),
        ("CrouzeixHarp", "harp"),
    ];
    let canonical_mathlib_artifacts = canonical_mathlib_artifact_root();

    for (target, route) in cases {
        let fixture = LeanEnvFixture::new();
        let python_args = fixture.root().join(format!("{route}-python-args"));
        let lake_args = fixture.root().join(format!("{route}-lake-args"));
        let (_fake_bin, path) = fake_python_and_lake_bin(
            "#!/bin/sh\nprintf '%s\n' \"$*\" > \"$PYTHON_ARGS\"\nexit 0\n",
            "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$LAKE_ARGS\"\nexit 0\n",
        );

        let mut command = Command::new("/bin/sh");
        fixture
            .apply(
                command
                    .current_dir(repo_root())
                    .arg("scripts/check_lean_library.sh")
                    .arg(target),
            )
            .env("PYTHON_ARGS", &python_args)
            .env("LAKE_ARGS", &lake_args)
            .env(
                "HARP_LEAN_MATHLIB_ARTIFACT_ROOT",
                &canonical_mathlib_artifacts,
            )
            .env("PATH", path)
            .assert()
            .success()
            .stdout(
                predicate::str::contains(format!("[lean] target={target}"))
                    .and(predicate::str::contains("[lean] outcome=passed")),
            );

        assert_eq!(
            fs::read_to_string(&python_args).expect("read fake python args"),
            format!(
                "labs/crouzeix_proof_reproduction/proof_evidence.py preflight --route {route}\n"
            ),
            "wrapper did not call python preflight with the exact route mapping",
        );
        assert_eq!(
            fs::read_to_string(&lake_args).expect("read fake lake args"),
            format!("--try-cache build {target}\n"),
            "lake should run exactly once after a successful python preflight",
        );
    }
}
