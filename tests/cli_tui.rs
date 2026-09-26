mod support;

use support::scripted_acp::{assert_script_consumed, scripted_acp, ScriptedAcpStep};

use std::fs;
use std::io::{BufRead, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::thread;

use assert_cmd::Command;
use corenius::core::{RuntimeEvent, ToolCall};
use corenius::tui::{TuiState, render_rows};
use predicates::str::contains;
use serde_json::json;
use tempfile::tempdir;

fn marker(root: &std::path::Path) {
    fs::create_dir_all(root.join("usr/bin")).unwrap();
    fs::write(root.join("usr/bin/node"), "").unwrap();
    fs::write(
        root.join(".corenius-insula-rootfs.json"),
        r#"{"schema_version":1,"owner":"corenius-insula","digest":"abc","node_path":"/usr/bin/node"}"#,
    )
    .unwrap();
}

fn fake_traecli(path: &Path, session_id: &str, response: &str) {
    fs::write(
        path,
        format!(
            r#"#!/usr/bin/env python3
import json, sys
response = {response:?}
session_id = {session_id:?}
if sys.argv[1:3] != ["acp", "serve"]:
    raise SystemExit(2)
for line in sys.stdin:
    msg = json.loads(line)
    if msg.get("method") == "initialize":
        print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":{{"protocolVersion":1}}}}), flush=True)
    elif msg.get("method") == "session/new":
        print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":{{"sessionId":session_id}}}}), flush=True)
    elif msg.get("method") == "session/prompt":
        print(json.dumps({{"jsonrpc":"2.0","method":"session/update","params":{{"sessionId":session_id,"update":{{"sessionUpdate":"agent_message_chunk","content":{{"type":"text","text":response}}}}}}}}), flush=True)
        print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":{{"stopReason":"end_turn","usage":{{"inputTokens":1,"outputTokens":1}}}}}}), flush=True)
"#,
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[test]
fn insula_select_and_doctor_dry_run_work_from_cli() {
    let temp = tempdir().unwrap();
    let store = temp.path().join("store");
    let rootfs = store.join("rootfs");
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&rootfs).unwrap();
    marker(&rootfs);

    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "insula",
            "select",
            "--workspace",
            workspace.to_str().unwrap(),
            "--rootfs",
            rootfs.to_str().unwrap(),
            "--store-root",
            store.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("selected"));

    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "insula",
            "doctor",
            "--workspace",
            workspace.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(contains("bwrap plan:"));
}

#[test]
fn tui_state_renders_streaming_approval_and_tool_result() {
    let mut state = TuiState::default();
    state.apply(RuntimeEvent::turn_started("t1", "hello"));
    state.apply(RuntimeEvent::assistant_delta("t1", "hi"));
    state.apply(RuntimeEvent::ApprovalRequested {
        turn_id: "t1".to_string(),
        request: corenius::core::ApprovalRequest {
            id: "a1".to_string(),
            tool_call: ToolCall::new("c1", "fake", "shell", json!({})),
            risk_summary: "Requires approval for shell command execution.".to_string(),
        },
        timestamp: "now".to_string(),
    });
    state.apply(RuntimeEvent::ToolResult {
        turn_id: "t1".to_string(),
        tool_call: ToolCall::new("c1", "fake", "shell", json!({})),
        result: json!({"exit_code": 2, "stderr": "failed"}),
        timestamp: "now".to_string(),
    });
    let rows = render_rows(&state);
    assert!(rows.contains(&"you: hello".to_string()));
    assert!(rows.contains(&"assistant: hi".to_string()));
    assert!(rows.iter().any(|row| row.contains("tool completed: shell")));
    assert!(rows.iter().any(|row| row.contains("exit 2 - failed")));
}

#[test]
fn provider_smoke_traecli_uses_acp_child() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli");
    fake_traecli(&server, "cli-smoke", "ok");

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "provider",
            "smoke",
            "traecli",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--prompt",
            "hello",
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("provider: traecli-acp"))
        .stdout(contains("session_id: cli-smoke"))
        .stdout(contains("text: ok"));
}

#[test]
fn eval_aime_sanity_runs_tracked_dataset_and_writes_artifacts() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-eval");
    let run_dir = temp.path().join("run");
    fake_traecli(&server, "cli-eval", "The answer is small.\nFINAL: 70\n");

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2025-I:I-1",
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 1"))
        .stdout(contains("\"correct\": 1"));

    let summary: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("summary.json")).unwrap()).unwrap();
    assert_eq!(summary["total"], 1);
    assert_eq!(summary["correct"], 1);
    assert_eq!(summary["dataset"], "evals/aime_sanity.jsonl");

    let samples = fs::read_to_string(run_dir.join("samples.jsonl")).unwrap();
    let sample: serde_json::Value = serde_json::from_str(samples.lines().next().unwrap()).unwrap();
    assert_eq!(sample["id"], "2025-I:I-1");
    assert_eq!(sample["expected_answer"], "70");
    assert_eq!(sample["extracted_answer"], "70");
    assert_eq!(sample["passed"], true);
    assert_eq!(sample.get("failure_kind"), None);
    assert!(
        sample["prompt"]
            .as_str()
            .unwrap()
            .contains("FINAL: <integer>")
    );
    assert!(run_dir.join("manifest.json").exists());
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["dataset"], "evals/aime_sanity.jsonl");
    assert_eq!(manifest["timeout_ms"], 5000);
    assert_eq!(manifest["stream"], false);
    assert_eq!(manifest["dataset_cache_files"].as_array().unwrap().len(), 1);
    assert_eq!(
        manifest["dataset_cache_files"][0]["path"],
        "evals/aime_sanity.jsonl"
    );
    assert!(
        manifest["dataset_cache_files"][0]["sha256"]
            .as_str()
            .unwrap()
            .len()
            == 64
    );

    let metrics: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("run_metrics.json")).unwrap())
            .unwrap();
    assert_eq!(metrics["schema_version"], 1);
    assert_eq!(metrics["sample_count"], 1);
    assert_eq!(metrics["execution_mode"], "shared_provider");
    assert_eq!(metrics["timeout_ms"], 5000);
    assert_eq!(metrics["samples"][0]["id"], "2025-I:I-1");
    assert_eq!(metrics["samples"][0]["passed"], true);
    assert_eq!(metrics["aggregate"]["provider_error_count"], 0);
}

#[test]
fn eval_aime_sanity_records_fresh_provider_per_sample_mode() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-fresh");
    let run_dir = temp.path().join("run");
    fake_traecli(&server, "cli-fresh", "FINAL: 70\n");

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2025-I:I-1",
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--timeout-ms",
            "5000",
            "--fresh-provider-per-sample",
        ])
        .assert()
        .success();

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["execution_mode"], "fresh_provider_per_sample");
    let metrics: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("run_metrics.json")).unwrap())
            .unwrap();
    assert_eq!(metrics["execution_mode"], "fresh_provider_per_sample");
}

#[test]
fn eval_aime_sanity_skips_timeout_probes_by_default() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-eval-skip-probe");
    let run_dir = temp.path().join("run");
    fake_traecli(&server, "cli-eval", "FINAL: 0\n");

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 5"));

    let samples = fs::read_to_string(run_dir.join("samples.jsonl")).unwrap();
    assert!(samples.contains("2025-II:II-2"));
    assert!(!samples.contains("2025-I:I-15"));
}

#[test]
fn eval_aime_runs_cached_full_dataset_and_writes_triage() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-aime");
    let run_dir = temp.path().join("run");
    let data_dir = temp.path().join(".scratch/aime-benchmark/data");
    fs::create_dir_all(&data_dir).unwrap();
    write_aime_fixture(&data_dir.join("2024.jsonl"), "2024", "60");
    write_aime_fixture(&data_dir.join("2025-I.jsonl"), "2025-I", "I-1");
    write_aime_fixture(&data_dir.join("2025-II.jsonl"), "2025-II", "II-1");
    fake_traecli(&server, "cli-aime", "reasoning\nFINAL: 42\n");

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 60"))
        .stdout(contains("\"correct\": 60"));

    let summary: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("summary.json")).unwrap()).unwrap();
    assert_eq!(summary["total"], 60);
    assert_eq!(summary["years"], json!(["2024", "2025-I", "2025-II"]));
    assert_eq!(summary["dataset"], "gair");
    assert_eq!(summary["by_year"]["2025-II"]["total"], 15);

    let samples = fs::read_to_string(run_dir.join("samples.jsonl")).unwrap();
    assert_eq!(samples.lines().count(), 60);
    let sample: serde_json::Value = serde_json::from_str(samples.lines().next().unwrap()).unwrap();
    assert_eq!(sample["source"], "gair");
    assert_eq!(sample["id"], "2024:60");
    assert_eq!(sample["return_status"], "end_turn");
    assert_eq!(sample["usage"]["input_tokens"], 1);

    let triage = fs::read_to_string(run_dir.join("triage.md")).unwrap();
    assert!(triage.contains("| 2024 | 30 | 30 | 1.000 |"));
    assert!(triage.contains("| failure_kind | count |"));
    assert!(triage.contains("| 2024:60 | 2024 | 42 | 42 | pass |  |"));
}

#[test]
fn eval_aime_records_fresh_provider_per_sample_mode() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-aime-fresh");
    let run_dir = temp.path().join("run");
    let data_dir = temp.path().join(".scratch/aime-benchmark/data");
    fs::create_dir_all(&data_dir).unwrap();
    write_aime_fixture(&data_dir.join("2025-I.jsonl"), "2025-I", "I-1");
    fake_traecli(&server, "cli-aime-fresh", "reasoning\nFINAL: 42\n");

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--years",
            "2025-I",
            "--limit",
            "1",
            "--timeout-ms",
            "5000",
            "--fresh-provider-per-sample",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 1"))
        .stdout(contains("\"correct\": 1"));

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["execution_mode"], "fresh_provider_per_sample");
    let metrics: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("run_metrics.json")).unwrap())
            .unwrap();
    assert_eq!(metrics["execution_mode"], "fresh_provider_per_sample");
}

#[test]
fn eval_aime_records_wrong_and_contaminated_failure_kinds() {
    // Covered by eval_aime_scripted_acp_covers_success_wrong_no_final_and_provider_error and
    // eval_aime_scripted_acp_covers_contamination_and_timeout.
}

#[test]
fn eval_aime_scripted_acp_covers_success_wrong_no_final_and_provider_error() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("scripted-traecli");
    let run_dir = temp.path().join("run");
    scripted_acp(
        &server,
        &[
            ScriptedAcpStep::Success { text: "FINAL: 25\n" },
            ScriptedAcpStep::Success { text: "FINAL: 999\n" },
            ScriptedAcpStep::Success {
                text: "No final marker here.",
            },
            ScriptedAcpStep::ProviderError { message: "scripted 503" },
        ],
    );

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2024:67",
            "--filter-id",
            "2025-I:I-1",
            "--filter-id",
            "2025-II:II-2",
            "--filter-id",
            "2025-II:II-15",
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 4"))
        .stdout(contains("\"correct\": 1"));

    let samples = fs::read_to_string(run_dir.join("samples.jsonl")).unwrap();
    assert!(samples.contains("\"failure_kind\":\"wrong_final\""));
    assert!(samples.contains("\"failure_kind\":\"no_final\""));
    assert!(samples.contains("\"failure_kind\":\"provider_error\""));
    assert_script_consumed(&server);
}

#[test]
fn eval_aime_scripted_acp_covers_contamination_and_timeout() {
    let temp = tempdir().unwrap();
    let contaminated_server = temp.path().join("scripted-contaminated");
    let contaminated_run = temp.path().join("contaminated-run");
    scripted_acp(
        &contaminated_server,
        &[ScriptedAcpStep::Success {
            text: "I will invoke skills and open files.\nFINAL: 999\n",
        }],
    );

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &contaminated_server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2024:67",
            "--run-dir",
            contaminated_run.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success();
    let samples = fs::read_to_string(contaminated_run.join("samples.jsonl")).unwrap();
    assert!(samples.contains("\"failure_kind\":\"prompt_contamination\""));
    assert_script_consumed(&contaminated_server);

    let timeout_server = temp.path().join("scripted-timeout");
    let timeout_run = temp.path().join("timeout-run");
    scripted_acp(&timeout_server, &[ScriptedAcpStep::Stall]);
    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &timeout_server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2025-I:I-15",
            "--include-timeout-probe",
            "--run-dir",
            timeout_run.to_str().unwrap(),
            "--timeout-ms",
            "200",
        ])
        .assert()
        .success();
    let timeout_samples = fs::read_to_string(timeout_run.join("samples.jsonl")).unwrap();
    assert!(timeout_samples.contains("\"failure_kind\":\"timeout_no_response\""));
}

#[test]
#[should_panic(expected = "scripted ACP fixture was not fully consumed")]
fn scripted_acp_assert_consumed_fails_on_unconsumed_script() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("scripted-unconsumed");
    scripted_acp(
        &server,
        &[ScriptedAcpStep::Success { text: "FINAL: 70\n" }],
    );

    assert_script_consumed(&server);
}

#[test]
fn eval_aime_replay_replays_failed_samples_by_default() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-replay");
    let from_run = temp.path().join("from-run");
    let replay_run = temp.path().join("replay-run");
    fs::create_dir_all(&from_run).unwrap();
    fake_traecli(&server, "cli-replay", "replayed\nFINAL: 70\n");
    fs::write(
        from_run.join("samples.jsonl"),
        [
            prior_sample_json("2025-I:I-1", "2025-I", "70", true),
            prior_sample_json("2025-II:II-2", "2025-II", "49", false),
        ]
        .join("\n")
            + "\n",
    )
    .unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime-replay",
            "--from-run",
            from_run.to_str().unwrap(),
            "--workspace",
            temp.path().to_str().unwrap(),
            "--run-dir",
            replay_run.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 1"));

    let replay_samples = fs::read_to_string(replay_run.join("samples.jsonl")).unwrap();
    assert_eq!(replay_samples.lines().count(), 1);
    assert!(replay_samples.contains("2025-II:II-2"));
    assert!(!replay_samples.contains("2025-I:I-1"));
    assert!(from_run.join("samples.jsonl").exists());

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_run.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["source_run_dir"], from_run.display().to_string());
    assert_eq!(manifest["replayed_from_sample_count"], 1);

    let triage = fs::read_to_string(replay_run.join("triage.md")).unwrap();
    assert!(triage.contains("source_run_dir"));
    assert!(triage.contains("replayed_from_sample_count"));
}

#[test]
fn eval_aime_replay_filter_id_selects_failed_sample() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-replay-filter");
    let from_run = temp.path().join("from-run");
    let replay_run = temp.path().join("replay-run");
    fs::create_dir_all(&from_run).unwrap();
    fake_traecli(&server, "cli-replay-filter", "replayed\nFINAL: 49\n");
    fs::write(
        from_run.join("samples.jsonl"),
        [
            prior_sample_json("2025-I:I-1", "2025-I", "70", false),
            prior_sample_json("2025-II:II-2", "2025-II", "49", false),
        ]
        .join("\n")
            + "\n",
    )
    .unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime-replay",
            "--from-run",
            from_run.to_str().unwrap(),
            "--workspace",
            temp.path().to_str().unwrap(),
            "--run-dir",
            replay_run.to_str().unwrap(),
            "--filter-id",
            "2025-II:II-2",
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 1"))
        .stdout(contains("\"correct\": 1"));

    let replay_samples = fs::read_to_string(replay_run.join("samples.jsonl")).unwrap();
    assert!(replay_samples.contains("2025-II:II-2"));
    assert!(!replay_samples.contains("2025-I:I-1"));
}

#[test]
fn eval_aime_replay_fails_for_missing_or_empty_selection() {
    let temp = tempdir().unwrap();
    let from_run = temp.path().join("from-run");
    fs::create_dir_all(&from_run).unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "eval",
            "aime-replay",
            "--from-run",
            from_run.to_str().unwrap(),
            "--workspace",
            temp.path().to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .failure()
        .stderr(contains("samples.jsonl"));

    fs::write(
        from_run.join("samples.jsonl"),
        prior_sample_json("2025-I:I-1", "2025-I", "70", true) + "\n",
    )
    .unwrap();
    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "eval",
            "aime-replay",
            "--from-run",
            from_run.to_str().unwrap(),
            "--workspace",
            temp.path().to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .failure()
        .stderr(contains("AIME replay selection is empty"));
}

#[test]
fn eval_aime_regrade_writes_independent_artifacts() {
    let temp = tempdir().unwrap();
    let from_run = temp.path().join("from-run");
    let regrade_run = temp.path().join("regrade-run");
    fs::create_dir_all(&from_run).unwrap();
    fs::write(
        from_run.join("samples.jsonl"),
        prior_sample_json("2025-I:I-1", "2025-I", "70", false) + "\n",
    )
    .unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "eval",
            "aime-regrade",
            "--from-run",
            from_run.to_str().unwrap(),
            "--workspace",
            temp.path().to_str().unwrap(),
            "--run-dir",
            regrade_run.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 1"));

    assert!(from_run.join("samples.jsonl").exists());
    assert!(regrade_run.join("manifest.json").exists());
    assert!(regrade_run.join("samples.jsonl").exists());
    assert!(regrade_run.join("summary.json").exists());
    assert!(regrade_run.join("triage.md").exists());
    assert!(regrade_run.join("scorer-delta.json").exists());

    let summary: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(regrade_run.join("summary.json")).unwrap())
            .unwrap();
    assert_eq!(
        summary["dataset"],
        format!("regrade:{}", from_run.display())
    );

    let triage = fs::read_to_string(regrade_run.join("triage.md")).unwrap();
    assert!(triage.contains(&format!(
        "- dataset: regrade:{}",
        from_run.display()
    )));
}

#[test]
fn eval_aime_regrade_fails_for_missing_samples() {
    let temp = tempdir().unwrap();
    let from_run = temp.path().join("from-run");
    fs::create_dir_all(&from_run).unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "eval",
            "aime-regrade",
            "--from-run",
            from_run.to_str().unwrap(),
            "--workspace",
            temp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("samples.jsonl"));
}

#[test]
fn eval_aime_rejects_truncated_default_cache() {
    let temp = tempdir().unwrap();
    let data_dir = temp.path().join(".scratch/aime-benchmark/data");
    fs::create_dir_all(&data_dir).unwrap();
    write_aime_fixture(&data_dir.join("2024.jsonl"), "2024", "60");
    write_aime_fixture(&data_dir.join("2025-I.jsonl"), "2025-I", "I-1");
    fs::write(
        data_dir.join("2025-II.jsonl"),
        r#"{"id":"II-1","problem":"Problem","answer":"42"}"#,
    )
    .unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "eval",
            "aime",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .failure()
        .stderr(contains("expected 15 AIME cases for 2025-II, found 1"));
}

#[test]
fn eval_aime_supports_year_limit_offset_filter_and_refresh_fetch() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-aime-fetch");
    let run_dir = temp.path().join("run");
    let http = serve_fixture_once("/test2025-I.jsonl", make_aime_fixture("2025-I", "I-1"));
    fake_traecli(&server, "cli-aime-fetch", "FINAL: 42\n");

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .env("CORENIUS_AIME_BASE_URL", http)
        .args([
            "eval",
            "aime",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--years",
            "2025-I",
            "--offset",
            "2",
            "--limit",
            "1",
            "--filter-id",
            "2025-I:I-3",
            "--refresh-data",
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 1"));

    let samples = fs::read_to_string(run_dir.join("samples.jsonl")).unwrap();
    assert!(samples.contains("2025-I:I-3"));
    assert!(!samples.contains("2025-I:I-2"));
    assert!(
        temp.path()
            .join(".scratch/aime-benchmark/data/2025-I.jsonl")
            .exists()
    );
}

#[test]
fn provider_smoke_traecli_can_print_streamed_chunks() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-stream");
    fs::write(
        &server,
        r#"#!/usr/bin/env python3
import json, sys
if sys.argv[1:3] != ["acp", "serve"]:
    raise SystemExit(2)
for line in sys.stdin:
    msg = json.loads(line)
    if msg.get("method") == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"protocolVersion":1}}), flush=True)
    elif msg.get("method") == "session/new":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"sessionId":"cli-stream"}}), flush=True)
    elif msg.get("method") == "session/prompt":
        print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"cli-stream","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"o"}}}}), flush=True)
        print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"cli-stream","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"k"}}}}), flush=True)
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"stopReason":"end_turn","usage":{"inputTokens":1,"outputTokens":2}}}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&server).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&server, permissions).unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "provider",
            "smoke",
            "traecli",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--prompt",
            "hello",
            "--timeout-ms",
            "5000",
            "--stream",
        ])
        .assert()
        .success()
        .stdout(contains("delta: o"))
        .stdout(contains("delta: k"))
        .stdout(contains("text: ok"));
}

#[test]
fn provider_supervise_traecli_reports_current_process_tree() {
    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "provider",
            "supervise",
            "traecli",
            "--pid",
            &std::process::id().to_string(),
        ])
        .assert()
        .success()
        .stdout(contains("\"root_pid\""))
        .stdout(contains("\"healthy\""))
        .stdout(contains("\"processes\""));
}

#[test]
fn provider_smoke_traecli_can_emit_supervisor_snapshots() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-supervised");
    fs::write(
        &server,
        r#"#!/usr/bin/env python3
import json, sys
if sys.argv[1:3] != ["acp", "serve"]:
    raise SystemExit(2)
for line in sys.stdin:
    msg = json.loads(line)
    if msg.get("method") == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"protocolVersion":1}}), flush=True)
    elif msg.get("method") == "session/new":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"sessionId":"cli-supervised"}}), flush=True)
    elif msg.get("method") == "session/prompt":
        print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"cli-supervised","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"ok"}}}}), flush=True)
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"stopReason":"end_turn","usage":{"inputTokens":1,"outputTokens":1}}}), flush=True)
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&server).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&server, permissions).unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "provider",
            "smoke",
            "traecli",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--prompt",
            "hello",
            "--timeout-ms",
            "5000",
            "--supervise",
        ])
        .assert()
        .success()
        .stdout(contains("supervisor: {\"root_pid\""))
        .stdout(contains("\"healthy\":true"))
        .stdout(contains("provider: traecli-acp"))
        .stdout(contains("text: ok"));
}

fn write_aime_fixture(path: &Path, year: &str, first_id: &str) {
    fs::write(path, make_aime_fixture(year, first_id)).unwrap();
}

fn make_aime_fixture(year: &str, first_id: &str) -> String {
    let mut rows = Vec::new();
    for index in 0..expected_fixture_count(year) {
        let id = if index == 0 {
            first_id.to_string()
        } else if year == "2024" {
            (60 + index).to_string()
        } else if year == "2025-I" {
            format!("I-{}", index + 1)
        } else {
            format!("II-{}", index + 1)
        };
        rows.push(format!(
            r#"{{"id":{id:?},"problem":"Problem {year} {id}","answer":"42"}}"#
        ));
    }
    rows.join("\n") + "\n"
}

fn expected_fixture_count(year: &str) -> usize {
    match year {
        "2024" => 30,
        "2025-I" | "2025-II" => 15,
        _ => panic!("unexpected fixture year {year}"),
    }
}

fn prior_sample_json(id: &str, year: &str, answer: &str, passed: bool) -> String {
    json!({
        "id": id,
        "year": year,
        "tags": [year],
        "source": "prior-run",
        "expected_answer": answer,
        "extracted_answer": if passed { answer } else { "999" },
        "passed": passed,
        "score": if passed { 1 } else { 0 },
        "elapsed_seconds": 0.5,
        "return_status": "end_turn",
        "prompt": format!("Prompt for {id}\nFINAL: <integer>"),
        "response": if passed { format!("FINAL: {answer}") } else { "FINAL: 999".to_string() },
        "failure_kind": if passed { serde_json::Value::Null } else { json!("wrong_final") },
    })
    .to_string()
}

fn serve_fixture_once(expected_path: &'static str, body: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request_line = String::new();
        std::io::BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut request_line)
            .unwrap();
        assert!(request_line.contains(expected_path), "{request_line}");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    format!("http://{addr}")
}
