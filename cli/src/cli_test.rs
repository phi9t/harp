use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn harp() -> Command {
    Command::cargo_bin("harp").expect("harp binary")
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create copied fixture directory");
    let mut entries = fs::read_dir(source)
        .expect("read fixture directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("read fixture entries");
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("fixture entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy fixture file");
        }
    }
}

fn crouzeix_textbook_cli_fixture() -> (TempDir, std::path::PathBuf) {
    let fixture = TempDir::new().expect("Crouzeix textbook CLI fixture");
    copy_tree(
        &repo_root().join("cli/src/test_fixtures/crouzeix_textbook/valid"),
        fixture.path(),
    );
    let coverage_path = fixture
        .path()
        .join("lab/math/textbook/coverage.json");
    let mut coverage: Value =
        serde_json::from_slice(&fs::read(&coverage_path).expect("read CLI coverage"))
            .expect("parse CLI coverage");
    coverage["items"][0]["lean_declaration"]["type_sha256"] =
        serde_json::json!("8e917e8c0326f9985c85b913d540c30ce433ac36fa15725f05cdd3825c2fc07f");
    coverage["items"][1]["lean_declaration"]["type_sha256"] =
        serde_json::json!("52415f3ef8b73cccc791e79afde418919b624b9796b613de131c714d6f7a02cb");
    let mut coverage_bytes = serde_json::to_vec_pretty(&coverage).expect("serialize CLI coverage");
    coverage_bytes.push(b'\n');
    fs::write(&coverage_path, coverage_bytes).expect("write CLI coverage");
    let exercises_path = fixture
        .path()
        .join("lab/math/textbook/exercises.json");
    let mut exercises: Value =
        serde_json::from_slice(&fs::read(&exercises_path).expect("read CLI exercises"))
            .expect("parse CLI exercises");
    exercises["exercises"][0]["lean_solution"]["type_sha256"] =
        serde_json::json!("13712c12307882f3f478550a66e7eeda393744d7bb4738034daf59194d9fd922");
    exercises["exercises"][1]["lean_solution"]["type_sha256"] =
        serde_json::json!("d80e6b0b5921f33484bff16e37d0dce7cd069eded835cef1c2e4c0f030311fc1");
    let mut exercise_bytes =
        serde_json::to_vec_pretty(&exercises).expect("serialize CLI exercises");
    exercise_bytes.push(b'\n');
    fs::write(&exercises_path, exercise_bytes).expect("write CLI exercises");
    let compile = fixture.path().join("fixture/Compile.lean");
    for relative in [
        "lab/math/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
        "lab/math/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
    ] {
        let target = fixture.path().join(relative);
        fs::create_dir_all(target.parent().expect("Lean source parent"))
            .expect("create Lean source directory");
        fs::copy(&compile, target).expect("copy maintained Lean source");
    }
    let mut declarations = vec![
        serde_json::json!({
            "name": "CrouzeixTextbook.Part01.matrix_column_is_basis_image",
            "kind": "theorem",
            "source_path": "lab/math/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
            "line": 10,
            "column": 1,
            "normalized_type": "theorem fixture type",
            "type_sha256": "8e917e8c0326f9985c85b913d540c30ce433ac36fa15725f05cdd3825c2fc07f",
            "direct_dependencies": [],
            "axioms": []
        }),
        serde_json::json!({
            "name": "CrouzeixTextbook.Part01.LinearTransformation",
            "kind": "definition",
            "source_path": "lab/math/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
            "line": 4,
            "column": 1,
            "normalized_type": "definition fixture type",
            "type_sha256": "52415f3ef8b73cccc791e79afde418919b624b9796b613de131c714d6f7a02cb",
            "direct_dependencies": [],
            "axioms": []
        }),
        serde_json::json!({
            "name": "CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution",
            "kind": "theorem",
            "source_path": "lab/math/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
            "line": 12,
            "column": 1,
            "normalized_type": "exercise one fixture type",
            "type_sha256": "13712c12307882f3f478550a66e7eeda393744d7bb4738034daf59194d9fd922",
            "direct_dependencies": [],
            "axioms": ["propext"]
        }),
        serde_json::json!({
            "name": "CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_02_solution",
            "kind": "theorem",
            "source_path": "lab/math/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
            "line": 20,
            "column": 1,
            "normalized_type": "exercise two fixture type",
            "type_sha256": "d80e6b0b5921f33484bff16e37d0dce7cd069eded835cef1c2e4c0f030311fc1",
            "direct_dependencies": [],
            "axioms": ["Classical.choice", "Quot.sound", "propext"]
        }),
    ];
    declarations.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    let receipt = fixture.path().join("lean-receipt.json");
    fs::write(
        &receipt,
        serde_json::to_vec(&serde_json::json!({
            "schema_version": "crouzeix-textbook-lean-receipt/v1",
            "toolchain": "leanprover/lean4:v4.32.1",
            "target": "CrouzeixTextbook",
            "declarations": declarations
        }))
        .expect("serialize CLI receipt"),
    )
    .expect("write CLI receipt");
    (fixture, receipt)
}

#[test]
fn check_reports_the_standalone_corpus_contract_as_json() {
    let output = harp()
        .current_dir(repo_root())
        .args(["--format", "json", "check"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("JSON envelope");
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "check");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["warnings"], serde_json::json!([]));
    let data = envelope["data"].as_object().expect("corpus summary");
    assert_eq!(data.len(), 3);
    assert!(data["canonical_documents"].as_u64().unwrap() > 0);
    assert!(data["reader_routes"].as_u64().unwrap() > 0);
    assert!(data["document_aliases"].as_u64().unwrap() > 0);
}

#[test]
fn build_check_accepts_the_checked_in_payload_without_writing() {
    let generated = repo_root().join("knowledge/generated/corpus.json");
    let before = fs::read(&generated).expect("checked-in payload");

    harp()
        .current_dir(repo_root())
        .args(["build", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("matches canonical inputs"));

    assert_eq!(fs::read(generated).expect("payload after check"), before);
}

#[test]
fn build_rejects_outputs_outside_the_generated_directory() {
    harp()
        .current_dir(repo_root())
        .args(["build", "--output", "../escape.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "output must stay beneath knowledge/generated",
        ));
}

#[test]
fn crouzeix_textbook_publish_then_check_uses_the_required_receipt() {
    let (fixture, receipt) = crouzeix_textbook_cli_fixture();

    harp()
        .current_dir(fixture.path())
        .args([
            "crouzeix-textbook",
            "check",
            "--receipt",
            receipt.to_str().expect("UTF-8 receipt path"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not match canonical inputs"));

    let output = harp()
        .current_dir(fixture.path())
        .args([
            "--format",
            "json",
            "crouzeix-textbook",
            "publish",
            "--receipt",
            receipt.to_str().expect("UTF-8 receipt path"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let envelope: Value = serde_json::from_slice(&output).expect("publication JSON envelope");
    assert_eq!(envelope["command"], "crouzeix-textbook.publish");
    assert_eq!(envelope["data"]["theorem_count"], 9);
    assert_eq!(envelope["data"]["exercise_count"], 2);
    assert_eq!(envelope["data"]["exact_theorem_correspondence_count"], 2);
    assert_eq!(envelope["data"]["checked_exercise_solution_count"], 2);
    assert!(envelope["data"].get("exact_correspondence_count").is_none());
    assert_eq!(envelope["data"]["outputs"].as_array().unwrap().len(), 6);
    assert_eq!(envelope["data"]["matched"], false);

    harp()
        .current_dir(fixture.path())
        .args([
            "crouzeix-textbook",
            "check",
            "--receipt",
            receipt.to_str().expect("UTF-8 receipt path"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("matches canonical inputs"));
}

#[test]
fn crouzeix_textbook_cli_requires_the_receipt_argument() {
    harp()
        .args(["crouzeix-textbook", "check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--receipt <RECEIPT>"));
    harp()
        .args(["crouzeix-textbook", "publish"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--receipt <RECEIPT>"));
}

#[test]
fn search_status_rejects_a_missing_index() {
    let repo = TempDir::new().expect("temp repository");
    fs::create_dir_all(repo.path().join("knowledge/sicp")).expect("SICP knowledge");
    fs::write(
        repo.path().join("knowledge/sicp/intro.md"),
        "# SICP\n\nA bounded improvement loop.\n",
    )
    .expect("SICP document");

    harp()
        .current_dir(repo.path())
        .args(["search", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("search index is missing"));
}

#[test]
fn search_refresh_status_and_query_share_a_digest_receipt() {
    let repo = TempDir::new().expect("temp repository");
    fs::create_dir_all(repo.path().join("knowledge/sicp")).expect("SICP knowledge");
    fs::create_dir_all(repo.path().join("knowledge/deepseek_harness"))
        .expect("implementation packet");
    fs::create_dir_all(repo.path().join("knowledge/agentic_engineering")).expect("knowledge");
    fs::create_dir_all(repo.path().join("knowledge/harness_benchmarks")).expect("benchmark packet");
    fs::create_dir_all(repo.path().join("knowledge/agentic_engineering")).expect("survey packet");
    fs::create_dir_all(repo.path().join("knowledge/harness_benchmarks"))
        .expect("verified coevolution packet");
    fs::create_dir_all(repo.path().join("knowledge/crouzeix_conjecture")).expect("Crouzeix packet");
    fs::create_dir_all(repo.path().join("knowledge/evaluator_integrity"))
        .expect("evaluator packet");
    fs::create_dir_all(repo.path().join("knowledge/mathematical_foundations"))
        .expect("mathematical foundations packet");
    fs::create_dir_all(repo.path().join("knowledge/autodiff_geometry"))
        .expect("autodiff geometry packet");
    fs::create_dir_all(repo.path().join("knowledge/private")).expect("loose knowledge");
    fs::create_dir_all(repo.path().join("evidence/sicp")).expect("evidence");
    fs::write(
        repo.path().join("knowledge/sicp/intro.md"),
        "# SICP\n\nA bounded SICP loop.\n",
    )
    .expect("SICP document");
    fs::write(
        repo.path().join("knowledge/private/draft.md"),
        "# Private draft\n\nThis must not enter the search index.\n",
    )
    .expect("loose document");
    fs::write(
        repo.path()
            .join("knowledge/agentic_engineering/codex_harness_deep_dive.md"),
        "# Codex harness\n\nPackage identity is an executable candidate contract.\n",
    )
    .expect("knowledge file");
    fs::write(
        repo.path()
            .join("knowledge/agentic_engineering/source_registry.md"),
        "# Engineering sources\n\nProvider contracts separate model parameters from scaffold state.\n",
    )
    .expect("survey file");
    fs::write(
        repo.path()
            .join("knowledge/harness_benchmarks/dynamic_workflow_need_and_eval.md"),
        "# Dynamic workflow\n\nDurable replay must preserve effect receipts.\n",
    )
    .expect("workflow file");
    fs::write(
        repo.path()
            .join("knowledge/crouzeix_conjecture/04_jin_positive_real_completion.md"),
        "# Crouzeix\n\nThe origin sample cancels the diagonal correction.\n",
    )
    .expect("Crouzeix file");
    fs::write(
        repo.path().join("knowledge/harp_knowledge_home.md"),
        "# Harp knowledge home\n\nReader entrypoint.\n",
    )
    .expect("knowledge home");
    fs::write(
        repo.path()
            .join("knowledge/evaluator_integrity/evaluator_integrity_benchmark_suite.md"),
        "# Evaluator integrity\n\nProtected scoring preserves evaluation independence.\n",
    )
    .expect("evaluator guide");
    fs::write(
        repo.path()
            .join("knowledge/mathematical_foundations/03_probability_and_gaussian_models.md"),
        "# Probability\n\nA covariance matrix records paired variation.\n",
    )
    .expect("mathematical foundations file");
    fs::write(
        repo.path()
            .join("knowledge/autodiff_geometry/formalization_roadmap.md"),
        "# Autodiff geometry roadmap\n\nJVP VJP duality connects pushed tangent and pulled cotangent coordinates.\n",
    )
    .expect("autodiff geometry file");
    fs::write(
        repo.path().join("evidence/sicp/source.txt"),
        "Harness evidence for SICP.",
    )
    .expect("source text");

    harp()
        .current_dir(repo.path())
        .args(["search", "refresh"])
        .assert()
        .success();
    harp()
        .current_dir(repo.path())
        .args(["--format", "json", "search", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"schema_version\":\"harp-search/v1\"",
        ));
    harp()
        .current_dir(repo.path())
        .args(["--format", "json", "search", "query", "SICP"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/sicp/intro.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args(["--format", "json", "search", "query", "\"private draft\""])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"data\":[]"))
        .stdout(predicate::str::contains("knowledge/private/draft.md").not());
    harp()
        .current_dir(repo.path())
        .args([
            "--format",
            "json",
            "search",
            "query",
            "\"package identity\"",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/agentic_engineering/codex_harness_deep_dive.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args(["--format", "json", "search", "query", "\"scaffold state\""])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/agentic_engineering/source_registry.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args(["--format", "json", "search", "query", "\"durable replay\""])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/harness_benchmarks/dynamic_workflow_need_and_eval.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args(["--format", "json", "search", "query", "\"origin sample\""])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/crouzeix_conjecture/04_jin_positive_real_completion.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args([
            "--format",
            "json",
            "search",
            "query",
            "\"protected scoring\"",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/evaluator_integrity/evaluator_integrity_benchmark_suite.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args([
            "--format",
            "json",
            "search",
            "query",
            "\"paired variation\"",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/mathematical_foundations/03_probability_and_gaussian_models.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args(["--format", "json", "search", "query", "\"pushed tangent\""])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/autodiff_geometry/formalization_roadmap.md\"",
        ));

    fs::write(
        repo.path().join("knowledge/sicp/intro.md"),
        "# SICP\n\nThe corpus changed.\n",
    )
    .expect("mutated SICP document");
    harp()
        .current_dir(repo.path())
        .args(["search", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("search index is stale"));
}

#[test]
fn sources_verify_accepts_the_tracked_offline_evidence() {
    let output = harp()
        .current_dir(repo_root())
        .args(["--format", "json", "sources", "verify"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let envelope: Value = serde_json::from_slice(&output).expect("JSON envelope");

    assert_eq!(envelope["command"], "sources.verify");
    assert_eq!(envelope["status"], "ok");
    assert!(
        envelope["data"]["evidence_artifacts"]
            .as_u64()
            .expect("artifact count")
            > 0
    );
    let lean_inventory = fs::read_to_string(
        repo_root().join("evidence/lean_proof_engineering/artifact_inventory.tsv"),
    )
    .expect("Lean proof-engineering artifact inventory");
    assert!(lean_inventory.starts_with("path\tbytes\tsha256\n"));
    assert!(lean_inventory.contains("lean4agent-2606.06523v2.pdf"));
    let manifest = fs::read_to_string(repo_root().join("evidence/implementations/manifest.tsv"))
        .expect("implementation manifest");
    let snapshot_rows = manifest.lines().skip(1).collect::<Vec<_>>();
    let implementation_sources = snapshot_rows
        .iter()
        .filter_map(|row| row.split('\t').next())
        .collect::<BTreeSet<_>>();
    assert_eq!(envelope["data"]["snapshot_files"], snapshot_rows.len());
    assert!(
        envelope["data"]["binary_objects"]
            .as_u64()
            .expect("binary count")
            > 0
    );
    assert_eq!(
        envelope["data"]["implementation_sources"],
        implementation_sources.len()
    );
    assert_eq!(envelope["data"]["crouzeix_source_receipts"], 29);
    assert_eq!(envelope["data"]["crouzeix_verification_receipts"], 4);
}

#[test]
fn obsidian_skills_vendor_is_pinned_and_manifested() {
    let root = repo_root().join("evidence/implementations/obsidian_skills");
    assert_eq!(
        fs::read_to_string(root.join("REMOTE")).expect("vendor remote"),
        "https://github.com/kepano/obsidian-skills.git\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("REVISION")).expect("vendor revision"),
        "a1dc48e68138490d522c04cbf5822214c6eb1202\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("LICENSE_STATUS")).expect("vendor license status"),
        "MIT\n"
    );

    let manifest = fs::read_to_string(repo_root().join("evidence/implementations/manifest.tsv"))
        .expect("implementation manifest");
    let snapshot_rows = manifest
        .lines()
        .skip(1)
        .filter(|row| row.starts_with("OBSIDIAN-SKILLS\t"))
        .collect::<Vec<_>>();
    assert!(!snapshot_rows.is_empty());
    assert!(snapshot_rows.iter().all(|row| {
        row.contains("\thttps://github.com/kepano/obsidian-skills.git\t")
            && row.contains("\ta1dc48e68138490d522c04cbf5822214c6eb1202\t")
            && row.contains("\tevidence/implementations/obsidian_skills/snapshot/")
    }));
}

#[test]
fn sources_materialize_requires_one_selection_mode() {
    harp()
        .current_dir(repo_root())
        .args(["sources", "materialize"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "choose either --source ID or --all",
        ));

    harp()
        .current_dir(repo_root())
        .args(["sources", "materialize", "--source", "PI-MONO", "--all"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "choose either --source ID or --all",
        ));
}

#[test]
fn repository_verify_checks_import_and_generated_contracts() {
    harp()
        .current_dir(repo_root())
        .args(["--format", "json", "repository", "verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"command\":\"repository.verify\"",
        ))
        .stdout(predicate::str::contains("\"import_rows\":521"))
        .stdout(predicate::str::contains("\"forbidden_references\":0"));
}

#[test]
fn sanitize_text_scrubs_stdin_and_preserves_clean_utf8_and_line_endings() {
    let output = harp()
        .args(["--format", "json", "sanitize", "text"])
        .write_stdin("Email person@private.dev\r\n数学\r\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let envelope: Value = serde_json::from_slice(&output).expect("sanitizer envelope");
    assert_eq!(
        envelope["data"]["scrubbed"],
        "Email [REDACTED_EMAIL]\r\n数学\r\n"
    );
}

#[test]
fn sanitize_check_is_read_only_and_scrub_is_idempotent() {
    let temporary = TempDir::new().expect("sanitizer fixture");
    let path = temporary
        .path()
        .canonicalize()
        .unwrap()
        .join("private note.txt");
    let input = "Contact person@private.dev\n";
    fs::write(&path, input).unwrap();
    harp()
        .args(["sanitize", "check"])
        .arg(&path)
        .assert()
        .failure();
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    harp()
        .args(["sanitize", "scrub"])
        .arg(&path)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "Contact [REDACTED_EMAIL]\n"
    );
    harp()
        .args(["sanitize", "check"])
        .arg(&path)
        .assert()
        .success();
    let output = harp()
        .args(["--format", "json", "sanitize", "scrub"])
        .arg(&path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let envelope: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["data"]["modified_files"], 0);
}
