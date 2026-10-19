use std::str::FromStr;
use std::time::Duration;

use harp_contracts::{
    OperationId, RuntimeEvent, ThreadHandle, ThreadId, ThreadSpec, TokenUsage, TokenUsageEvent,
    TurnCompletedEvent, TurnId, TurnSnapshot, TurnSpec, TurnStatus,
};
use harp_runtime::fake::{FakeCodexRuntime, FakeStep};
use harp_runtime::{
    collect_until_terminal, ActivityRuntime, ActivitySpec, CollectionLimits, InterruptPurpose,
    InterruptReceipt,
};

fn thread_spec() -> ThreadSpec {
    ThreadSpec {
        base_instructions: "base".to_owned(),
        developer_instructions: "developer".to_owned(),
        cwd: "/private/tmp".to_owned(),
        runtime_workspace_roots: vec!["/private/tmp".to_owned()],
        workspace_authority: None,
        approval_policy: "never".to_owned(),
        sandbox_mode: "workspace-write".to_owned(),
        model: "fake-model".to_owned(),
        reasoning_effort: Some("low".to_owned()),
        ephemeral: true,
    }
}

fn turn_spec(marker: OperationId) -> TurnSpec {
    TurnSpec {
        instruction: "produce a result".to_owned(),
        operation_marker: marker,
        output_schema: serde_json::json!({"type": "object"}),
        model: None,
        reasoning_effort: None,
    }
}

#[tokio::test]
async fn activity_runtime_tracks_logical_ids_process_digest_and_interrupt_receipts() {
    let logical_thread = ThreadHandle {
        thread_id: ThreadId::from_str("logical-session").unwrap(),
    };
    let logical_turn = TurnId::from_str("logical-turn").unwrap();
    let marker = OperationId::new();
    let spec = ActivitySpec {
        logical_session_id: logical_thread.thread_id.clone(),
        logical_turn_id: logical_turn.clone(),
        thread_spec: thread_spec(),
        turn_spec: turn_spec(marker),
        activity_dir: "/private/tmp/harp-runtime-activity".to_owned(),
        invocation_sha256: "a".repeat(64),
        external_session_id: None,
    };
    let handle_process_record = "b".repeat(64);

    let mut runtime = FakeCodexRuntime::new(vec![
        FakeStep::StartLogicalSession {
            expected: spec.thread_spec.clone(),
            result: Ok(logical_thread.clone()),
        },
        FakeStep::StartActivity {
            expected: spec.clone(),
            process_record_sha256: handle_process_record.clone(),
            external_session_id: Some("codex-session-1".parse().unwrap()),
            result: Ok(()),
        },
        FakeStep::ActivityEvent {
            expected_logical_session_id: logical_thread.thread_id.clone(),
            expected_logical_turn_id: logical_turn.clone(),
            result: Ok(RuntimeEvent::TokenUsage(TokenUsageEvent {
                thread_id: logical_thread.thread_id.clone(),
                turn_id: logical_turn.clone(),
                usage: TokenUsage {
                    total_tokens: 5,
                    input_tokens: 3,
                    cached_input_tokens: 0,
                    output_tokens: 2,
                    reasoning_output_tokens: 0,
                },
            })),
        },
        FakeStep::ActivityEvent {
            expected_logical_session_id: logical_thread.thread_id.clone(),
            expected_logical_turn_id: logical_turn.clone(),
            result: Ok(RuntimeEvent::TurnCompleted(TurnCompletedEvent {
                thread_id: logical_thread.thread_id.clone(),
                turn: TurnSnapshot {
                    turn_id: logical_turn.clone(),
                    operation_marker: Some(spec.turn_spec.operation_marker.clone()),
                    status: TurnStatus::Completed,
                    final_agent_message: Some(r#"{"ok":true}"#.to_owned()),
                },
            })),
        },
        FakeStep::ActivityInterrupt {
            expected_logical_session_id: logical_thread.thread_id.clone(),
            expected_logical_turn_id: logical_turn.clone(),
            expected_purpose: InterruptPurpose::Cancellation,
            result: Ok(InterruptReceipt {
                process_record_sha256: handle_process_record.clone(),
                quiescent: true,
            }),
        },
    ])
    .unwrap();

    let started = runtime
        .start_logical_session(spec.thread_spec.clone())
        .await
        .unwrap();
    assert_eq!(started, logical_thread);
    let handle = runtime.start_activity(spec.clone()).await.unwrap();
    assert_eq!(handle.logical_session_id, logical_thread.thread_id);
    assert_eq!(handle.logical_turn_id, logical_turn);
    assert_eq!(handle.process_record_sha256, handle_process_record);
    assert_eq!(
        handle.external_session_id.as_ref().map(ToString::to_string),
        Some("codex-session-1".to_owned())
    );

    let events = collect_until_terminal(
        &mut runtime,
        &handle,
        &CollectionLimits {
            max_events: 4,
            max_serialized_bytes: 1024 * 1024,
            max_total_wait: Duration::from_secs(1),
            per_event_wait: Duration::from_millis(100),
        },
    )
    .await
    .unwrap();
    assert!(matches!(events[0], RuntimeEvent::TokenUsage(_)));
    assert!(matches!(events[1], RuntimeEvent::TurnCompleted(_)));

    let receipt = runtime
        .interrupt(&handle, InterruptPurpose::Cancellation)
        .await
        .unwrap();
    assert_eq!(receipt.process_record_sha256, handle.process_record_sha256);
    assert!(receipt.quiescent);
}
