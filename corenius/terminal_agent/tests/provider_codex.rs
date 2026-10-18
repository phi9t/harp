use std::sync::{Arc, Mutex};

use corenius::core::{ToolCall, ToolDefinition};
use corenius::provider::codex::{
    CodexDirectOptions, CodexResponse, parse_sse_events, run_codex_direct,
};
use serde_json::json;

fn event(value: serde_json::Value) -> String {
    format!("data: {}\n\n", serde_json::to_string(&value).unwrap())
}

#[test]
fn parser_rejects_malformed_and_truncated_sse() {
    assert!(parse_sse_events("data: {bad}\n\n").is_err());
    assert!(parse_sse_events("data: {").is_err());
}

#[tokio::test]
async fn codex_stream_handles_text_tool_continuation_and_usage() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let responses = Arc::new(Mutex::new(vec![
        CodexResponse {
            status: 200,
            body: event(json!({"type":"response.output_text.delta","delta":"Need "}))
                + &event(
                    json!({"type":"response.completed","response":{"id":"r1","usage":{"input_tokens":1,"output_tokens":2,"input_tokens_details":{"cached_tokens":0}},"output":[{"type":"function_call","name":"read_file","call_id":"c1","arguments":"{\"path\":\"README.md\"}"}]}}),
                ),
        },
        CodexResponse {
            status: 200,
            body: event(json!({"type":"response.output_text.delta","delta":"done"}))
                + &event(
                    json!({"type":"response.completed","response":{"id":"r2","usage":{"input_tokens":3,"output_tokens":4},"output":[]}}),
                ),
        },
    ]));

    let output = run_codex_direct(
        CodexDirectOptions {
            model: "test".to_string(),
            instructions: "test".to_string(),
            input: vec![json!({"role":"user","content":"hello"})],
            tools: vec![ToolDefinition::read_only(
                "read_file",
                "Read",
                json!({"type":"object"}),
            )],
            max_steps: 3,
        },
        |request| {
            let requests = requests.clone();
            let responses = responses.clone();
            async move {
                requests.lock().unwrap().push(request);
                Ok(responses.lock().unwrap().remove(0))
            }
        },
        |call: ToolCall| async move {
            assert_eq!(call.name, "read_file");
            Ok(json!({"ok": true}))
        },
    )
    .await
    .unwrap();

    assert_eq!(output.text, "Need done");
    assert_eq!(output.response_id, "r2");
    assert_eq!(output.usage.input_tokens, 4);
    assert_eq!(requests.lock().unwrap().len(), 2);
    assert!(
        requests.lock().unwrap()[1]
            .to_string()
            .contains("function_call_output")
    );
}

#[tokio::test]
async fn codex_stream_reports_failure_and_max_step_exhaustion() {
    let failed = run_codex_direct(
        CodexDirectOptions {
            model: "test".to_string(),
            instructions: String::new(),
            input: vec![],
            tools: vec![],
            max_steps: 1,
        },
        |_request| async {
            Ok(CodexResponse {
                status: 200,
                body: event(
                    json!({"type":"response.failed","response":{"error":{"message":"bad"}}}),
                ),
            })
        },
        |_call| async { Ok(json!({})) },
    )
    .await;
    assert!(failed.unwrap_err().to_string().contains("response failed"));

    let exhausted = run_codex_direct(
        CodexDirectOptions {
            model: "test".to_string(),
            instructions: String::new(),
            input: vec![],
            tools: vec![ToolDefinition::read_only("read_file", "Read", json!({"type":"object"}))],
            max_steps: 1,
        },
        |_request| async {
            Ok(CodexResponse {
                status: 200,
                body: event(json!({"type":"response.completed","response":{"id":"r","usage":{},"output":[{"type":"function_call","name":"read_file","call_id":"c","arguments":"{}"}]}})),
            })
        },
        |_call| async { Ok(json!({"ok": true})) },
    )
    .await;
    assert!(exhausted.unwrap_err().to_string().contains("tool limit"));
}
