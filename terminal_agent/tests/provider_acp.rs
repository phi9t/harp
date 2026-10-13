use std::fs;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use corenius::core::{
    Coordinator, PolicyGate, RuntimeEvent, ToolCall, ToolDefinition, ToolRegistryLike,
    TranscriptStore,
};
use corenius::provider::acp::TraeCliAcpProvider;
use corenius::provider::acp::{
    AcpOptions, AcpSessionProfile, run_acp_prompt, run_acp_prompt_observed,
};
use tempfile::tempdir;
use tokio::time::{Duration, sleep, timeout};

#[tokio::test]
async fn acp_prompt_streams_agent_chunks_and_usage_from_fake_server() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-acp.py");
    fs::write(
        &server,
        r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    msg = json.loads(line)
    method = msg.get("method")
    if method == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"protocolVersion":1,"agentInfo":{"name":"fake","version":"1"}}}), flush=True)
    elif method == "session/new":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"sessionId":"s1"}}), flush=True)
    elif method == "session/prompt":
        print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"s1","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"o"}}}}), flush=True)
        print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"s1","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"k"}}}}), flush=True)
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"stopReason":"end_turn","usage":{"inputTokens":3,"outputTokens":2,"cachedReadTokens":1,"cachedWriteTokens":0}}}), flush=True)
"#,
    )
    .unwrap();

    let output = run_acp_prompt(AcpOptions {
        command: "python3".to_string(),
        args: vec![server.to_string_lossy().into_owned()],
        cwd: temp.path().to_path_buf(),
        model: Some("fake-model".to_string()),
        prompt: "reply ok".to_string(),
        timeout_ms: 5_000,
        profile: AcpSessionProfile::default(),
    })
    .await
    .unwrap();

    assert_eq!(output.text, "ok");
    assert_eq!(output.session_id, "s1");
    assert_eq!(output.stop_reason, Some("end_turn".to_string()));
    assert_eq!(output.usage.unwrap().input_tokens, 3);
}

#[tokio::test]
async fn acp_prompt_supervision_reports_child_lifecycle() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("supervised-acp.py");
    fs::write(
        &server,
        r#"#!/usr/bin/env python3
import json, subprocess, sys, time
helper = subprocess.Popen(["python3", "-c", "import time; time.sleep(5)"])
for line in sys.stdin:
    msg = json.loads(line)
    if msg.get("method") == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"protocolVersion":1}}), flush=True)
    elif msg.get("method") == "session/new":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"sessionId":"supervised"}}), flush=True)
    elif msg.get("method") == "session/prompt":
        print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"supervised","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"ok"}}}}), flush=True)
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"stopReason":"end_turn","usage":{"inputTokens":1,"outputTokens":1}}}), flush=True)
        helper.terminate()
"#,
    )
    .unwrap();

    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let observed = snapshots.clone();
    let output = run_acp_prompt_observed(
        AcpOptions {
            command: "python3".to_string(),
            args: vec![server.to_string_lossy().into_owned()],
            cwd: temp.path().to_path_buf(),
            model: None,
            prompt: "hello".to_string(),
            timeout_ms: 5_000,
            profile: AcpSessionProfile::default(),
        },
        None,
        Some(Arc::new(move |snapshot| {
            observed.lock().unwrap().push(snapshot);
            Ok(())
        })),
    )
    .await
    .unwrap();

    let snapshots = snapshots.lock().unwrap();
    assert_eq!(output.text, "ok");
    assert!(!output.supervision.is_empty());
    assert!(!snapshots.is_empty());
    assert!(output.supervision.iter().any(|snapshot| {
        snapshot
            .processes
            .iter()
            .any(|process| process.command.contains("supervised-acp.py"))
    }));
}

#[tokio::test]
async fn acp_prompt_reports_json_rpc_errors() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("error-acp.py");
    fs::write(
        &server,
        r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    msg = json.loads(line)
    print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"error":{"code":-32000,"message":"boom"}}), flush=True)
"#,
    )
    .unwrap();

    let err = run_acp_prompt(AcpOptions {
        command: "python3".to_string(),
        args: vec![server.to_string_lossy().into_owned()],
        cwd: temp.path().to_path_buf(),
        model: None,
        prompt: "hello".to_string(),
        timeout_ms: 5_000,
        profile: AcpSessionProfile::default(),
    })
    .await
    .unwrap_err();

    assert!(err.to_string().contains("boom"));
}

#[tokio::test]
async fn acp_prompt_timeout_kills_child_process() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("hanging-acp.py");
    let alive_marker = temp.path().join("still-alive");
    fs::write(
        &server,
        format!(
            r#"#!/usr/bin/env python3
import json, pathlib, sys, time
alive_marker = pathlib.Path({alive_marker:?})
for line in sys.stdin:
    msg = json.loads(line)
    if msg.get("method") == "initialize":
        print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":{{"protocolVersion":1}}}}), flush=True)
    elif msg.get("method") == "session/new":
        print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":{{"sessionId":"hanging"}}}}), flush=True)
    elif msg.get("method") == "session/prompt":
        time.sleep(0.5)
        alive_marker.write_text("child survived timeout")
        time.sleep(10)
"#,
            alive_marker = alive_marker.to_string_lossy()
        ),
    )
    .unwrap();

    let err = run_acp_prompt(AcpOptions {
        command: "python3".to_string(),
        args: vec![server.to_string_lossy().into_owned()],
        cwd: temp.path().to_path_buf(),
        model: None,
        prompt: "hello".to_string(),
        timeout_ms: 100,
        profile: AcpSessionProfile::default(),
    })
    .await
    .unwrap_err();

    assert!(err.to_string().contains("timed out"));
    sleep(Duration::from_secs(1)).await;
    assert!(!alive_marker.exists());
}

#[tokio::test]
async fn acp_provider_streams_chunks_before_prompt_completes() {
    struct NoTools;

    #[async_trait]
    impl ToolRegistryLike for NoTools {
        fn definitions(&self) -> Vec<ToolDefinition> {
            Vec::new()
        }

        async fn execute(&self, _call: ToolCall) -> anyhow::Result<serde_json::Value> {
            unreachable!("no tools are exposed")
        }
    }

    let temp = tempdir().unwrap();
    let server = temp.path().join("streaming-acp.py");
    fs::write(
        &server,
        r#"#!/usr/bin/env python3
import json, sys, time
for line in sys.stdin:
    msg = json.loads(line)
    if msg.get("method") == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"protocolVersion":1}}), flush=True)
    elif msg.get("method") == "session/new":
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"sessionId":"streaming"}}), flush=True)
    elif msg.get("method") == "session/prompt":
        print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"streaming","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"early"}}}}), flush=True)
        time.sleep(2)
        print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":{"stopReason":"end_turn","usage":{"inputTokens":1,"outputTokens":1}}}), flush=True)
"#,
    )
    .unwrap();

    let store = TranscriptStore::load(temp.path().join("transcript.jsonl"))
        .await
        .unwrap();
    let provider = TraeCliAcpProvider {
        command: "python3".to_string(),
        cwd: temp.path().to_path_buf(),
        model: None,
        timeout_ms: 5_000,
        serve_args: Some(vec![server.to_string_lossy().into_owned()]),
        profile: AcpSessionProfile::default(),
    };
    let coordinator = Coordinator::new(
        Arc::new(provider),
        Arc::new(NoTools),
        PolicyGate::new(),
        store,
        None,
    );

    let mut turn = coordinator.start_user_turn("hello").await.unwrap();
    assert!(matches!(
        turn.events.recv().await.unwrap(),
        RuntimeEvent::TurnStarted { .. }
    ));
    let streamed = timeout(Duration::from_secs(1), turn.events.recv())
        .await
        .unwrap()
        .unwrap();

    assert!(matches!(
        streamed,
        RuntimeEvent::AssistantDelta { ref delta, .. } if delta == "early"
    ));
}
