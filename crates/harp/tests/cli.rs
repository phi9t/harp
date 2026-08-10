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
        .and_then(Path::parent)
        .expect("workspace root")
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
            "canonical_documents": 69,
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
    harp()
        .current_dir(repo_root())
        .args(["--format", "json", "sources", "verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"sources.verify\""))
        .stdout(predicate::str::contains("\"snapshot_files\":105"))
        .stdout(predicate::str::contains("\"binary_objects\":56"))
        .stdout(predicate::str::contains("\"implementation_sources\":11"));
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
