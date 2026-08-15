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
            "retained_concepts": 75,
            "coverage_entries": 75,
            "canonical_documents": 88,
            "systems": 16,
            "weng_sections": 9,
            "diagnostic_fields": 28,
            "diagnostic_rules": 29,
            "diagnostic_cases": 12,
            "lessons": 6,
            "source_registry_rows": 76,
            "evidence_edges": 98
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
    fs::create_dir_all(repo.path().join("knowledge/crouzeix_conjecture")).expect("Crouzeix packet");
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
            .join("knowledge/crouzeix_conjecture/04_jin_positive_real_completion.md"),
        "# Crouzeix\n\nThe origin sample cancels the diagonal correction.\n",
    )
    .expect("Crouzeix file");
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
        .args(["--format", "json", "search", "query", "\"origin sample\""])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"path\":\"knowledge/crouzeix_conjecture/04_jin_positive_real_completion.md\"",
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
    assert_eq!(envelope["data"]["evidence_artifacts"], 396);
    assert_eq!(envelope["data"]["snapshot_files"], 105);
    assert_eq!(envelope["data"]["binary_objects"], 58);
    assert_eq!(envelope["data"]["implementation_sources"], 11);
    assert_eq!(envelope["data"]["crouzeix_source_receipts"], 29);
    assert_eq!(envelope["data"]["crouzeix_verification_receipts"], 4);
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
