use std::str::FromStr;

use harp_contracts::{
    ExternalSessionId, OperationId, ThreadHandle, ThreadId, ThreadSpec, TurnHandle, TurnId,
    TurnSpec, TurnStatus,
};
use harp_runtime::conformance::RuntimeFixture;
use harp_runtime::fake::scenarios::FakeFixture;
use serde_json::json;

const MARKER: &str = "018f22e2-7c3b-7def-8123-456789abcdef";
const CONTINUATION_MARKER: &str = "018f22e2-7c3c-7abc-9234-56789abcdef0";

pub fn runtime_fixture() -> RuntimeFixture {
    RuntimeFixture {
        thread_spec: ThreadSpec {
            base_instructions: "Analyze the assigned task.".to_string(),
            developer_instructions: "Return only the required schema.".to_string(),
            cwd: "/private/work".to_string(),
            runtime_workspace_roots: vec!["/private/work".to_string()],
            workspace_authority: None,
            approval_policy: "never".to_string(),
            sandbox_mode: "workspace-write".to_string(),
            model: "codex".to_string(),
            reasoning_effort: Some("high".to_string()),
            ephemeral: false,
        },
        turn_spec: TurnSpec {
            instruction: "Inspect the fixture.".to_string(),
            operation_marker: OperationId::from_str(MARKER).unwrap(),
            output_schema: json!({"type": "object"}),
            model: Some("codex".to_string()),
            reasoning_effort: Some("high".to_string()),
        },
        logical_turn_id: TurnId::from_str("generated-turn").unwrap(),
        activity_dir: "/private/work/.harp-activity".to_owned(),
        invocation_sha256: "a".repeat(64),
        expected_process_record_sha256: "b".repeat(64),
        expected_external_session_id: Some(
            ExternalSessionId::from_str("generated-thread").unwrap(),
        ),
        expected_status: TurnStatus::Completed,
        expected_final_message: Some(r#"{"answer":"ok"}"#.to_string()),
        minimum_total_tokens: 8,
    }
}

pub fn fake_fixture() -> FakeFixture {
    let runtime = runtime_fixture();
    let thread = ThreadHandle {
        thread_id: ThreadId::from_str("generated-thread").unwrap(),
    };
    let turn = TurnHandle {
        thread_id: thread.thread_id.clone(),
        turn_id: TurnId::from_str("generated-turn").unwrap(),
    };
    let continuation = TurnHandle {
        thread_id: thread.thread_id.clone(),
        turn_id: TurnId::from_str("generated-continuation").unwrap(),
    };
    FakeFixture {
        runtime,
        continuation_turn_spec: TurnSpec {
            instruction: "Continue from checkpoint.".to_string(),
            operation_marker: OperationId::from_str(CONTINUATION_MARKER).unwrap(),
            output_schema: json!({"type": "object"}),
            model: Some("codex".to_string()),
            reasoning_effort: Some("high".to_string()),
        },
        expected_thread: thread.clone(),
        expected_turn: turn,
        expected_continuation_turn: continuation,
    }
}
