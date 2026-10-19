#![cfg(feature = "test-cli-fixture")]

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use harp_cli_process::{CliProcessRuntime, ProcessRuntimeConfig};
use harp_contracts::{
    OperationId, RuntimeErrorKind, RuntimeEvent, ThreadId, ThreadSpec, TurnId, TurnSpec, TurnStatus,
};
use harp_runtime::{ActivityHandle, ActivityRuntime, ActivitySpec, InterruptPurpose};
use tempfile::TempDir;

fn fake_cli() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_harp_cli_process_fake_cli"))
}

fn thread_spec(cwd: &Path) -> ThreadSpec {
    let cwd = cwd.to_str().expect("test path is UTF-8").to_owned();
    ThreadSpec {
        base_instructions: "base".to_owned(),
        developer_instructions: "developer".to_owned(),
        cwd: cwd.clone(),
        runtime_workspace_roots: vec![cwd],
        workspace_authority: None,
        approval_policy: "never".to_owned(),
        sandbox_mode: "workspace-write".to_owned(),
        model: "fake-model".to_owned(),
        reasoning_effort: Some("low".to_owned()),
        ephemeral: true,
    }
}

fn turn_spec(schema: serde_json::Value) -> TurnSpec {
    TurnSpec {
        instruction: "return a result".to_owned(),
        operation_marker: OperationId::new(),
        output_schema: schema,
        model: None,
        reasoning_effort: None,
    }
}

fn activity_spec(root: &TempDir, external: Option<&str>) -> ActivitySpec {
    let activity_dir = root.path().join("activity");
    fs::create_dir_all(&activity_dir).expect("create activity dir");
    ActivitySpec {
        logical_session_id: ThreadId::from_str("logical-session").unwrap(),
        logical_turn_id: TurnId::from_str("logical-turn").unwrap(),
        thread_spec: thread_spec(&activity_dir),
        turn_spec: turn_spec(serde_json::json!({"type": "object"})),
        activity_dir: activity_dir.to_str().unwrap().to_owned(),
        invocation_sha256: "a".repeat(64),
        external_session_id: external.map(|value| value.parse().unwrap()),
    }
}

async fn runtime_with_env(
    root: &TempDir,
    env: &[(&str, String)],
) -> (CliProcessRuntime, ActivitySpec) {
    let mut config = ProcessRuntimeConfig::builder(fake_cli(), root.path().join("codex-home"))
        .exec_args(["exec", "--json"])
        .build()
        .unwrap();
    for (key, value) in env {
        config = config.with_env(*key, value.clone()).unwrap();
    }
    (
        CliProcessRuntime::spawn(config).await.unwrap(),
        activity_spec(root, None),
    )
}

#[tokio::test]
async fn fake_cli_success_yields_logical_events_and_external_session() {
    let root = TempDir::new().unwrap();
    let (mut runtime, spec) = runtime_with_env(
        &root,
        &[
            ("HARP_FAKE_CLI_SESSION_ID", "codex-session-1".to_owned()),
            (
                "HARP_FAKE_CLI_FINAL_MESSAGE",
                r#"{"answer":"ok"}"#.to_owned(),
            ),
            ("HARP_FAKE_CLI_TOTAL_TOKENS", "9".to_owned()),
        ],
    )
    .await;

    let handle = runtime.start_activity(spec.clone()).await.unwrap();
    assert_eq!(handle.logical_session_id, spec.logical_session_id);
    assert_eq!(handle.logical_turn_id, spec.logical_turn_id);
    assert_eq!(
        handle.external_session_id.as_ref().map(ToString::to_string),
        Some("codex-session-1".to_owned())
    );

    assert!(matches!(
        runtime
            .next_event(&handle, Duration::from_secs(1))
            .await
            .unwrap(),
        RuntimeEvent::ThreadStarted(_)
    ));
    assert!(matches!(
        runtime
            .next_event(&handle, Duration::from_secs(1))
            .await
            .unwrap(),
        RuntimeEvent::TurnStarted(_)
    ));
    let usage = runtime
        .next_event(&handle, Duration::from_secs(1))
        .await
        .unwrap();
    assert!(matches!(
        usage,
        RuntimeEvent::TokenUsage(event)
            if event.thread_id == spec.logical_session_id
                && event.turn_id == spec.logical_turn_id
                && event.usage.total_tokens == 9
    ));
    let terminal = runtime
        .next_event(&handle, Duration::from_secs(1))
        .await
        .unwrap();
    assert!(matches!(
        terminal,
        RuntimeEvent::TurnCompleted(event)
            if event.thread_id == spec.logical_session_id
                && event.turn.turn_id == spec.logical_turn_id
                && event.turn.status == TurnStatus::Completed
                && event.turn.final_agent_message.as_deref() == Some(r#"{"answer":"ok"}"#)
    ));
}

#[tokio::test]
async fn malformed_jsonl_fails_protocol_after_thread_started() {
    let root = TempDir::new().unwrap();
    let (mut runtime, spec) = runtime_with_env(
        &root,
        &[("HARP_FAKE_CLI_MODE", "malformed-jsonl".to_owned())],
    )
    .await;

    let error = runtime.start_activity(spec).await.unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
}

#[tokio::test]
async fn missing_thread_started_fails_start_activity() {
    let root = TempDir::new().unwrap();
    let (mut runtime, spec) = runtime_with_env(
        &root,
        &[("HARP_FAKE_CLI_MODE", "missing-thread-started".to_owned())],
    )
    .await;

    let error = runtime.start_activity(spec).await.unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
}

#[tokio::test]
async fn schema_failure_is_reported_before_completion() {
    let root = TempDir::new().unwrap();
    let (mut runtime, spec) = runtime_with_env(
        &root,
        &[("HARP_FAKE_CLI_MODE", "schema-failure".to_owned())],
    )
    .await;
    let handle = runtime.start_activity(spec).await.unwrap();
    assert!(matches!(
        runtime
            .next_event(&handle, Duration::from_secs(1))
            .await
            .unwrap(),
        RuntimeEvent::ThreadStarted(_)
    ));
    assert!(matches!(
        runtime
            .next_event(&handle, Duration::from_secs(1))
            .await
            .unwrap(),
        RuntimeEvent::TurnStarted(_)
    ));

    let error = runtime
        .next_event(&handle, Duration::from_secs(1))
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::OutputSchema);
}

#[tokio::test]
async fn nonzero_exit_fails_start_activity() {
    let root = TempDir::new().unwrap();
    let (mut runtime, spec) =
        runtime_with_env(&root, &[("HARP_FAKE_CLI_MODE", "nonzero-exit".to_owned())]).await;

    let error = runtime.start_activity(spec).await.unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Transport);
}

#[tokio::test]
async fn delayed_output_still_completes_within_wait_budget() {
    let root = TempDir::new().unwrap();
    let (mut runtime, spec) = runtime_with_env(
        &root,
        &[
            ("HARP_FAKE_CLI_MODE", "delayed-output".to_owned()),
            ("HARP_FAKE_CLI_DELAY_MS", "25".to_owned()),
        ],
    )
    .await;

    let handle = runtime.start_activity(spec).await.unwrap();
    let event = runtime
        .next_event(&handle, Duration::from_secs(1))
        .await
        .unwrap();
    assert!(matches!(event, RuntimeEvent::ThreadStarted(_)));
}

#[tokio::test]
async fn resume_invocation_passes_external_session_to_cli() {
    let root = TempDir::new().unwrap();
    let ledger = root.path().join("ledger.jsonl");
    let mut config = ProcessRuntimeConfig::builder(fake_cli(), root.path().join("codex-home"))
        .exec_args(["exec", "--json"])
        .build()
        .unwrap();
    config = config
        .with_env("HARP_FAKE_CLI_LEDGER", ledger.to_str().unwrap())
        .unwrap();
    let mut runtime = CliProcessRuntime::spawn(config).await.unwrap();
    let spec = activity_spec(&root, Some("codex-session-existing"));

    let _handle = runtime.start_activity(spec).await.unwrap();
    let ledger = fs::read_to_string(ledger).unwrap();
    let record: serde_json::Value = serde_json::from_str(ledger.trim()).unwrap();
    assert_eq!(record["resume"], serde_json::json!(true));
}

#[tokio::test]
async fn interrupt_signals_activity_process_group() {
    let root = TempDir::new().unwrap();
    let mut config = ProcessRuntimeConfig::builder(fake_cli(), root.path().join("codex-home"))
        .exec_args(["exec", "--json"])
        .build()
        .unwrap();
    config = config
        .with_env("HARP_FAKE_CLI_MODE", "descendant-held-descriptors")
        .unwrap();
    let mut runtime = CliProcessRuntime::spawn(config).await.unwrap();
    let spec = activity_spec(&root, None);
    let handle = runtime.start_activity(spec).await.unwrap();
    let receipt = runtime
        .interrupt(&handle, InterruptPurpose::Cancellation)
        .await
        .unwrap();
    assert_eq!(receipt.process_record_sha256, handle.process_record_sha256);
}

#[allow(dead_code)]
fn find_recorded_activity(root: &TempDir) -> Option<ActivityHandle> {
    let records = root
        .path()
        .join("codex-home")
        .join("harp-cli-process")
        .join("records");
    if let Some(entry) = fs::read_dir(records).ok()?.next() {
        let entry = entry.ok()?;
        let digest = entry
            .file_name()
            .to_string_lossy()
            .trim_end_matches(".json")
            .to_owned();
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(entry.path()).ok()?).ok()?;
        return Some(ActivityHandle {
            logical_session_id: value["logicalSessionId"].as_str()?.parse().ok()?,
            logical_turn_id: value["logicalTurnId"].as_str()?.parse().ok()?,
            process_record_sha256: digest,
            external_session_id: None,
        });
    }
    None
}
