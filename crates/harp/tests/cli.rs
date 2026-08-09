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
            "canonical_documents": 64,
            "systems": 16,
            "weng_sections": 9,
            "diagnostic_fields": 28,
            "diagnostic_rules": 29,
            "diagnostic_cases": 12,
            "lessons": 6,
            "source_registry_rows": 68,
            "evidence_edges": 79
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
    fs::create_dir_all(repo.path().join("content")).expect("content");
    fs::write(
        repo.path().join("content/intro.md"),
        "# Recursive improvement\n\nA bounded improvement loop.\n",
    )
    .expect("content file");

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
    fs::create_dir_all(repo.path().join("content")).expect("content");
    fs::create_dir_all(repo.path().join("knowledge/meta_harness")).expect("knowledge");
    fs::create_dir_all(repo.path().join("evidence/weng/text")).expect("evidence");
    fs::write(
        repo.path().join("content/intro.md"),
        "# Recursive improvement\n\nA bounded recursive improvement loop.\n",
    )
    .expect("content file");
    fs::write(
        repo.path()
            .join("knowledge/meta_harness/meta_harness_deep_dive.md"),
        "# Meta Harness\n\nPackage identity is an executable candidate contract.\n",
    )
    .expect("knowledge file");
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
        .stdout(predicate::str::contains("\"path\":\"content/intro.md\""));
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

    fs::write(
        repo.path().join("content/intro.md"),
        "# Recursive improvement\n\nThe corpus changed.\n",
    )
    .expect("mutated content");
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
        .stdout(predicate::str::contains("\"snapshot_files\":95"))
        .stdout(predicate::str::contains("\"binary_objects\":54"))
        .stdout(predicate::str::contains("\"implementation_sources\":9"));
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
