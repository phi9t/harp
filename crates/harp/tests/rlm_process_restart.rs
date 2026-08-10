#![cfg(feature = "test-cli-fixture")]

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Stdio};
use std::str::FromStr;
use std::time::{Duration, Instant};

use assert_cmd::Command;
use harp_artifacts::ArtifactStore;
use harp_contracts::{
    Budget, NodeKind, ResultEnvelope, ResultStatus, RetryPolicy, TaskGraph, TaskId, TaskNode,
    TaskRole, WorkspaceMode,
};
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn harp_bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("harp")
}

fn fake_cli_bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("harp_rlm_fake_cli")
}

fn graph_path(repo: &TempDir) -> PathBuf {
    let graph = TaskGraph {
        schema_version: 1,
        nodes: vec![
            task("alpha", NodeKind::Analysis, TaskRole::Explore, &[]),
            task("beta", NodeKind::Analysis, TaskRole::Explore, &[]),
            task(
                "reduce",
                NodeKind::Reducer,
                TaskRole::Reduce,
                &["alpha", "beta"],
            ),
        ],
    };
    let path = repo.path().join("graph.json");
    fs::write(&path, serde_json::to_vec_pretty(&graph).unwrap()).unwrap();
    path
}

fn task(task_id: &str, kind: NodeKind, role: TaskRole, dependencies: &[&str]) -> TaskNode {
    TaskNode {
        task_id: TaskId::from_str(task_id).unwrap(),
        kind,
        role,
        instruction: format!("produce {task_id}"),
        dependencies: dependencies
            .iter()
            .map(|dependency| TaskId::from_str(dependency).unwrap())
            .collect(),
        inputs: Vec::new(),
        workspace_mode: WorkspaceMode::Scratch,
        model_policy: "fake-model".to_owned(),
        permission_profile: "workspace-write".to_owned(),
        budget: Budget::new(100, 3, 2_048),
        output_schema: result_schema(),
        retry_policy: RetryPolicy {
            max_transient_attempts: 1,
        },
    }
}

fn result_schema() -> String {
    serde_json::json!({"type": "object"}).to_string()
}

fn result_messages(repo: &TempDir) -> String {
    let messages = ["alpha", "beta", "reduce"]
        .into_iter()
        .map(|task_id| (task_id.to_owned(), result_message(repo, task_id)))
        .collect::<BTreeMap<_, _>>();
    serde_json::to_string(&messages).unwrap()
}

fn result_message(repo: &TempDir, task_id: &str) -> String {
    let artifact_root = repo.path().join(".harp/rlm/artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    fs::set_permissions(&artifact_root, fs::Permissions::from_mode(0o700)).unwrap();
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
        evidence: Vec::new(),
        trace_ref,
        summary: format!("{task_id} completed"),
        token_usage: 12,
        confidence: Some(1.0),
        failure_class: None,
    })
    .unwrap()
}

#[test]
fn rlm_process_restart_accepts_one_result_digest() {
    let repo = TempDir::new().unwrap();
    let graph = graph_path(&repo);
    let barrier = repo.path().join("barrier");
    let ledger = repo.path().join("ledger.jsonl");
    fs::write(&barrier, b"hold").unwrap();

    let mut child = StdCommand::new(harp_bin())
        .current_dir(repo.path())
        .env("HARP_FAKE_CLI_MODE", "durable-barrier")
        .env("HARP_FAKE_CLI_BARRIER", &barrier)
        .env("HARP_FAKE_CLI_LEDGER", &ledger)
        .env("HARP_FAKE_CLI_FINAL_MESSAGES", result_messages(&repo))
        .env("HARP_FAKE_CLI_TOTAL_TOKENS", "12")
        .env("HARP_RLM_TEST_LEASE_SECONDS", "2")
        .args([
            "rlm",
            "run",
            "--benchmark",
            graph.to_str().unwrap(),
            "--runtime",
            "codex",
            "--runtime-executable",
            fake_cli_bin().to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    if !wait_for_ledger(&ledger) {
        let output = child.wait_with_output().unwrap();
        panic!(
            "fake CLI ledger was not written; child status {:?}; stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    child.kill().unwrap();
    let _ = child.wait();
    fs::remove_file(&barrier).unwrap();
    std::thread::sleep(Duration::from_secs(3));

    let resume_output = resume_all(&repo, &barrier, &ledger);
    let resume_envelope: Value = serde_json::from_slice(&resume_output).unwrap();
    let run_id = resume_envelope["data"]["runs"][0]["run_id"]
        .as_str()
        .unwrap()
        .to_owned();
    for _ in 0..3 {
        if status_state(repo.path(), &run_id) == "completed" {
            break;
        }
        let _ = resume_all(&repo, &barrier, &ledger);
    }

    let accepted = accepted_result_count(repo.path(), &run_id);
    assert_eq!(accepted, 3);
    let ledger_lines = fs::read_to_string(&ledger).unwrap().lines().count();
    assert_eq!(
        ledger_lines, 4,
        "the interrupted activity may use one continuation, but final acceptance must stay one result per task"
    );
}

fn resume_all(repo: &TempDir, barrier: &Path, ledger: &Path) -> Vec<u8> {
    Command::new(harp_bin())
        .current_dir(repo.path())
        .env("HARP_FAKE_CLI_MODE", "durable-barrier")
        .env("HARP_FAKE_CLI_BARRIER", barrier)
        .env("HARP_FAKE_CLI_LEDGER", ledger)
        .env("HARP_FAKE_CLI_FINAL_MESSAGES", result_messages(repo))
        .env("HARP_FAKE_CLI_TOTAL_TOKENS", "12")
        .env("HARP_RLM_TEST_LEASE_SECONDS", "2")
        .args([
            "--format",
            "json",
            "rlm",
            "resume",
            "--all-incomplete",
            "--runtime",
            "codex",
            "--runtime-executable",
            fake_cli_bin().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"rlm.resume\""))
        .get_output()
        .stdout
        .clone()
}

fn wait_for_ledger(path: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if path.exists() && fs::metadata(path).unwrap().len() > 0 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

fn accepted_result_count(repo: &Path, run_id: &str) -> usize {
    let output = Command::new(harp_bin())
        .current_dir(repo)
        .args(["--format", "json", "rlm", "status", run_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let envelope: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["data"]["state"], "completed");
    envelope["data"]["accepted_results"].as_u64().unwrap() as usize
}

fn status_state(repo: &Path, run_id: &str) -> String {
    let output = Command::new(harp_bin())
        .current_dir(repo)
        .args(["--format", "json", "rlm", "status", run_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let envelope: Value = serde_json::from_slice(&output).unwrap();
    envelope["data"]["state"].as_str().unwrap().to_owned()
}
