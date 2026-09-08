use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
#[cfg(feature = "test-cli-fixture")]
use std::str::FromStr;

use assert_cmd::Command;
#[cfg(feature = "test-cli-fixture")]
use harp_artifacts::ArtifactStore;
#[cfg(feature = "test-cli-fixture")]
use harp_contracts::{ArtifactRef, ResultEnvelope, ResultStatus, TaskId};
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn harp() -> Command {
    Command::cargo_bin("harp").expect("harp binary")
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
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
        &repo_root().join("crates/harp/tests/fixtures/crouzeix_textbook/valid"),
        fixture.path(),
    );
    let coverage_path = fixture
        .path()
        .join("content/crouzeix_textbook/coverage.json");
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
        .join("content/crouzeix_textbook/exercises.json");
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
        "formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
        "formalization/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
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
            "source_path": "formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
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
            "source_path": "formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
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
            "source_path": "formalization/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
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
            "source_path": "formalization/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
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

#[cfg(feature = "test-cli-fixture")]
fn fake_cli_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("harp_rlm_fake_cli")
}

#[cfg(feature = "test-cli-fixture")]
fn workflow_path(repo: &TempDir) -> std::path::PathBuf {
    let workflow = serde_json::json!({
        "schemaVersion": 1,
        "name": "research-workflow",
        "root": {
            "kind": "sequence",
            "steps": [
                {
                    "kind": "agent",
                    "callId": "discover",
                    "prompt": "Run discover.",
                    "outputSchema": "{\"type\":\"object\"}",
                    "modelPolicy": "fake-model",
                    "permissionProfile": "workspace-write",
                    "workspaceMode": "scratch",
                    "budget": {
                        "maxTokens": 100,
                        "timeoutSeconds": 3,
                        "maxStorageBytes": 2048
                    },
                    "retryPolicy": {"maxTransientAttempts": 1}
                },
                {
                    "kind": "parallel",
                    "branches": [
                        {
                            "kind": "agent",
                            "callId": "market",
                            "prompt": "Run market.",
                            "outputSchema": "{\"type\":\"object\"}",
                            "modelPolicy": "fake-model",
                            "permissionProfile": "workspace-write",
                            "workspaceMode": "scratch",
                            "budget": {
                                "maxTokens": 100,
                                "timeoutSeconds": 3,
                                "maxStorageBytes": 2048
                            },
                            "retryPolicy": {"maxTransientAttempts": 1}
                        },
                        {
                            "kind": "agent",
                            "callId": "technical",
                            "prompt": "Run technical.",
                            "outputSchema": "{\"type\":\"object\"}",
                            "modelPolicy": "fake-model",
                            "permissionProfile": "workspace-write",
                            "workspaceMode": "scratch",
                            "budget": {
                                "maxTokens": 100,
                                "timeoutSeconds": 3,
                                "maxStorageBytes": 2048
                            },
                            "retryPolicy": {"maxTransientAttempts": 1}
                        }
                    ]
                }
            ]
        }
    });
    let path = repo.path().join("workflow.json");
    fs::write(&path, serde_json::to_vec_pretty(&workflow).unwrap()).unwrap();
    path
}

#[cfg(feature = "test-cli-fixture")]
fn result_messages(repo: &TempDir) -> String {
    let messages = [
        "discover",
        "market",
        "technical",
        "research-workflow-reduce",
    ]
    .into_iter()
    .map(|task_id| (task_id.to_owned(), result_message(repo, task_id)))
    .collect::<std::collections::BTreeMap<_, _>>();
    serde_json::to_string(&messages).unwrap()
}

#[cfg(feature = "test-cli-fixture")]
fn result_message(repo: &TempDir, task_id: &str) -> String {
    let artifact_root = repo.path().join(".harp/workflow/artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    let artifacts = ArtifactStore::open(&artifact_root).unwrap();
    let answer_ref = artifacts
        .publish(format!("{task_id} answer").as_bytes(), "text/plain")
        .unwrap();
    let trace_ref = artifacts
        .publish(
            format!(r#"{{"event":"{task_id}"}}"#).as_bytes(),
            "application/jsonl",
        )
        .unwrap();
    serde_json::to_string(&ResultEnvelope {
        schema_version: 1,
        task_id: TaskId::from_str(task_id).unwrap(),
        status: ResultStatus::Success,
        answer_ref: Some(answer_ref),
        evidence: Vec::<ArtifactRef>::new(),
        trace_ref,
        summary: format!("{task_id} completed"),
        token_usage: 12,
        confidence: Some(1.0),
        failure_class: None,
    })
    .unwrap()
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
    assert_eq!(
        envelope["data"],
        serde_json::json!({
            "retained_concepts": 76,
            "coverage_entries": 76,
            "canonical_documents": 186,
            "systems": 17,
            "weng_sections": 9,
            "diagnostic_fields": 28,
            "diagnostic_rules": 29,
            "diagnostic_cases": 12,
            "lessons": 6,
            "source_registry_rows": 89,
            "evidence_edges": 107
        })
    );
}

#[test]
fn build_check_accepts_the_checked_in_payload_without_writing() {
    let generated = repo_root().join("atlas/src/content/generated/corpus.json");
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
            "output must stay beneath atlas/src/content/generated",
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
    fs::create_dir_all(repo.path().join("knowledge/rsi")).expect("RSI knowledge");
    fs::write(
        repo.path().join("knowledge/rsi/intro.md"),
        "# Recursive improvement\n\nA bounded improvement loop.\n",
    )
    .expect("RSI document");

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
    fs::create_dir_all(repo.path().join("knowledge/rsi")).expect("RSI knowledge");
    fs::create_dir_all(repo.path().join("knowledge/darwin_godel_machine")).expect("DGM packet");
    fs::create_dir_all(repo.path().join("knowledge/meta_harness")).expect("knowledge");
    fs::create_dir_all(repo.path().join("knowledge/harness_benchmarks")).expect("benchmark packet");
    fs::create_dir_all(repo.path().join("knowledge/self_improving_agents_survey"))
        .expect("survey packet");
    fs::create_dir_all(repo.path().join("knowledge/verified_coevolution_agenda"))
        .expect("verified coevolution packet");
    fs::create_dir_all(repo.path().join("knowledge/crouzeix_conjecture")).expect("Crouzeix packet");
    fs::create_dir_all(repo.path().join("knowledge/darwinx")).expect("DarwinX packet");
    fs::create_dir_all(repo.path().join("knowledge/mathematical_foundations"))
        .expect("mathematical foundations packet");
    fs::create_dir_all(repo.path().join("knowledge/autodiff_geometry"))
        .expect("autodiff geometry packet");
    fs::create_dir_all(repo.path().join("knowledge/private")).expect("loose knowledge");
    fs::create_dir_all(repo.path().join("evidence/weng/text")).expect("evidence");
    fs::write(
        repo.path().join("knowledge/rsi/intro.md"),
        "# Recursive improvement\n\nA bounded recursive improvement loop.\n",
    )
    .expect("RSI document");
    fs::write(
        repo.path().join("knowledge/private/draft.md"),
        "# Private draft\n\nThis must not enter the search index.\n",
    )
    .expect("loose document");
    fs::write(
        repo.path()
            .join("knowledge/meta_harness/meta_harness_deep_dive.md"),
        "# Meta Harness\n\nPackage identity is an executable candidate contract.\n",
    )
    .expect("knowledge file");
    fs::write(
        repo.path()
            .join("knowledge/self_improving_agents_survey/synthesis.md"),
        "# Survey\n\nModern self-improving agents pair model parameters with scaffold state.\n",
    )
    .expect("survey file");
    fs::write(
        repo.path()
            .join("knowledge/verified_coevolution_agenda/verified_coevolution_agenda.md"),
        "# Verified coevolution\n\nRecursive closure makes model-harness coevolution falsifiable.\n",
    )
    .expect("verified coevolution file");
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
        repo.path().join("knowledge/darwinx/darwinx_index.md"),
        "# DarwinX\n\nPopulation selection preserves diverse harness candidates.\n",
    )
    .expect("DarwinX index");
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
        repo.path().join("evidence/weng/text/source.txt"),
        "Harness evidence for recursive improvement.",
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
        .args([
            "--format",
            "json",
            "search",
            "query",
            "recursive improvement",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/rsi/intro.md\"",
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
            "\"path\":\"knowledge/meta_harness/meta_harness_deep_dive.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args(["--format", "json", "search", "query", "\"scaffold state\""])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/self_improving_agents_survey/synthesis.md\"",
        ));
    harp()
        .current_dir(repo.path())
        .args([
            "--format",
            "json",
            "search",
            "query",
            "\"recursive closure\"",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/verified_coevolution_agenda/verified_coevolution_agenda.md\"",
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
            "\"population selection\"",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/darwinx/darwinx_index.md\"",
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
        repo.path().join("knowledge/rsi/intro.md"),
        "# Recursive improvement\n\nThe corpus changed.\n",
    )
    .expect("mutated RSI document");
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
    assert_eq!(envelope["data"]["evidence_artifacts"], 481);
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
    assert_eq!(envelope["data"]["binary_objects"], 67);
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
fn rlm_checkpoint_reports_default_state_paths() {
    let repo = TempDir::new().expect("temp repository");
    let output = harp()
        .current_dir(repo.path())
        .args(["--format", "json", "rlm", "checkpoint"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("JSON envelope");
    assert_eq!(envelope["command"], "rlm.checkpoint");
    assert_eq!(envelope["status"], "ok");
    assert!(envelope["data"]["state_path"]
        .as_str()
        .unwrap()
        .ends_with(".harp/rlm/state.sqlite"));
}

#[test]
fn rlm_resume_requires_exactly_one_selection_mode() {
    harp()
        .current_dir(repo_root())
        .args(["rlm", "resume"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "provide exactly one run id or --all-incomplete",
        ));

    harp()
        .current_dir(repo_root())
        .args([
            "rlm",
            "resume",
            "018f22e2-7c3b-7def-8123-456789abcdef",
            "--all-incomplete",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "provide exactly one run id or --all-incomplete",
        ));
}

#[cfg(feature = "test-cli-fixture")]
#[test]
fn workflow_run_executes_dynamic_workflow_through_the_durable_engine() {
    let repo = TempDir::new().expect("temp repository");
    let workflow = workflow_path(&repo);
    let output = harp()
        .current_dir(repo.path())
        .env("HARP_FAKE_CLI_FINAL_MESSAGES", result_messages(&repo))
        .env("HARP_FAKE_CLI_TOTAL_TOKENS", "12")
        .args([
            "--format",
            "json",
            "workflow",
            "run",
            "--file",
            workflow.to_str().unwrap(),
            "--runtime",
            "codex",
            "--runtime-executable",
            fake_cli_bin().to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("JSON envelope");
    assert_eq!(envelope["command"], "workflow.run");
    assert_eq!(envelope["status"], "ok");
    let run_id = envelope["data"]["run_id"].as_str().expect("run id");
    assert_eq!(envelope["data"]["completed_tasks"], 4);
    assert!(
        envelope["data"]["compiled_graph_sha256"]
            .as_str()
            .unwrap()
            .len()
            == 64
    );

    let status = harp()
        .current_dir(repo.path())
        .args(["--format", "json", "workflow", "status", run_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status_envelope: Value = serde_json::from_slice(&status).expect("status JSON");
    assert_eq!(status_envelope["command"], "workflow.status");
    assert_eq!(status_envelope["data"]["state"], "completed");
    assert_eq!(status_envelope["data"]["accepted_results"], 4);
    let state_path = repo.path().join(".harp/workflow/state.sqlite");
    let before = fs::read(&state_path).unwrap();
    for command in ["inspect", "explain"] {
        let mut inspection = harp();
        inspection
            .current_dir(repo.path())
            .args(["--format", "json", "workflow", command, run_id]);
        if command == "explain" {
            inspection.args(["--task", "research-workflow-reduce"]);
        }
        let output = inspection.assert().success().get_output().stdout.clone();
        let report: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(report["data"]["schema_version"], 1);
        assert_eq!(report["data"]["state"], "completed");
        let tasks = report["data"]["tasks"].as_array().unwrap();
        assert_eq!(tasks.len(), if command == "inspect" { 4 } else { 1 });
        assert!(tasks.iter().all(|task| task["action"] == "none"));
    }
    harp()
        .current_dir(repo.path())
        .args(["workflow", "explain", run_id, "--task", "absent"])
        .assert()
        .failure();
    harp()
        .current_dir(repo.path())
        .args(["workflow", "inspect", run_id, "--limit", "0"])
        .assert()
        .failure();
    assert_eq!(before, fs::read(&state_path).unwrap());
}

#[cfg(feature = "test-cli-fixture")]
#[test]
fn workflow_run_uses_traecli_runtime_instead_of_rejecting_it() {
    let repo = TempDir::new().expect("temp repository");
    let workflow = workflow_path(&repo);
    let output = harp()
        .current_dir(repo.path())
        .env("HARP_FAKE_CLI_FINAL_MESSAGES", result_messages(&repo))
        .env("HARP_FAKE_CLI_TOTAL_TOKENS", "12")
        .args([
            "--format",
            "json",
            "workflow",
            "run",
            "--file",
            workflow.to_str().unwrap(),
            "--runtime",
            "traecli",
            "--runtime-executable",
            fake_cli_bin().to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: Value = serde_json::from_slice(&output).expect("JSON envelope");
    assert_eq!(envelope["command"], "workflow.run");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["completed_tasks"], 4);
}

#[cfg(feature = "test-cli-fixture")]
#[test]
fn workflow_run_uses_authored_model_for_generated_reducer() {
    let repo = TempDir::new().expect("temp repository");
    let workflow = workflow_path(&repo);
    let workflow_json = fs::read_to_string(&workflow).expect("workflow JSON");
    fs::write(&workflow, workflow_json.replace("fake-model", "gpt-5.5"))
        .expect("workflow with real model");
    let messages = [
        "discover",
        "market",
        "technical",
        "research-workflow-reduce",
    ]
    .into_iter()
    .map(|task_id| (task_id.to_owned(), result_message(&repo, task_id)))
    .collect::<std::collections::BTreeMap<_, _>>();

    harp()
        .current_dir(repo.path())
        .env(
            "HARP_FAKE_CLI_FINAL_MESSAGES",
            serde_json::to_string(&messages).unwrap(),
        )
        .env("HARP_FAKE_CLI_TOTAL_TOKENS", "12")
        .args([
            "--format",
            "json",
            "workflow",
            "run",
            "--file",
            workflow.to_str().unwrap(),
            "--runtime",
            "traecli",
            "--runtime-executable",
            fake_cli_bin().to_str().unwrap(),
        ])
        .assert()
        .success();

    let records = repo
        .path()
        .join(".harp/workflow/runtime-home/harp-cli-process/records");
    let reducer_records = fs::read_dir(records)
        .expect("process records")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            if !entry.file_type().ok()?.is_file() {
                return None;
            }
            let value: Value = serde_json::from_slice(&fs::read(entry.path()).ok()?).ok()?;
            let activity_dir = value["activityDir"].as_str()?;
            if activity_dir.contains("research-workflow-reduce") {
                Some(value["argv"].clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    assert_eq!(reducer_records.len(), 1);
    let argv = reducer_records[0].as_array().expect("argv array");
    let model_index = argv
        .iter()
        .position(|arg| arg.as_str() == Some("--model"))
        .expect("model flag");
    assert_eq!(argv[model_index + 1], "gpt-5.5");

    let state_path = repo.path().join(".harp/workflow/state.sqlite");
    let max_tokens: i64 = rusqlite::Connection::open(state_path)
        .unwrap()
        .query_row(
            "SELECT b.reserved_tokens
             FROM budget_reservations b
             JOIN attempts a ON a.attempt_id = b.attempt_id
             WHERE a.task_id = 'research-workflow-reduce'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(max_tokens >= 160_000);
}

#[test]
fn workflow_inspection_does_not_create_missing_state() {
    let repo = TempDir::new().unwrap();
    let run_id = "00000000-0000-7000-8000-000000000001";
    harp()
        .current_dir(repo.path())
        .args(["workflow", "inspect", run_id])
        .assert()
        .failure();
    assert!(!repo.path().join(".harp").exists());
}
