use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use async_trait::async_trait;
use corenius::core::{
    ApprovalDecision, ApprovalResponder, Coordinator, PolicyGate, ProviderEvent, ProviderSession,
    RuntimeEvent, SandboxResult, ToolCall, ToolDefinition, ToolRegistryLike, TranscriptStore,
};
use serde_json::json;
use tempfile::tempdir;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep, timeout};

#[test]
fn crate_exposes_version() {
    assert!(!corenius::VERSION.is_empty());
}

#[test]
fn policy_allows_read_and_asks_for_shell() {
    let gate = PolicyGate::new();
    let read = ToolDefinition::read_only("read_file", "Read", json!({"type":"object"}));
    let shell = ToolDefinition::shell("shell", "Shell", json!({"type":"object"}));
    let call = ToolCall::new("1", "fake", "read_file", json!({}));
    assert!(gate.evaluate(&call, &read).is_allow());
    let ask = gate.evaluate(
        &ToolCall::new("2", "fake", "shell", json!({"command":"pwd"})),
        &shell,
    );
    assert!(ask.is_ask());
}

#[tokio::test]
async fn transcript_replays_jsonl_into_current_projection() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("transcript.jsonl");
    let mut store = TranscriptStore::load(path.clone()).await.unwrap();
    store
        .append(&RuntimeEvent::turn_started("t1", "hello"))
        .await
        .unwrap();
    store
        .append(&RuntimeEvent::assistant_delta("t1", "hi"))
        .await
        .unwrap();
    store
        .append(&RuntimeEvent::assistant_delta("t1", " there"))
        .await
        .unwrap();
    store
        .append(&RuntimeEvent::turn_finished("t1", None, None))
        .await
        .unwrap();

    let replayed = TranscriptStore::load(path).await.unwrap();
    let messages = replayed.current_transcript();
    assert_eq!(messages[0].content, "hello");
    assert_eq!(messages[1].content, "hi there");
}

#[tokio::test]
async fn approval_event_is_emitted_before_runner_waits_for_answer() {
    struct FakeProvider;

    #[async_trait]
    impl ProviderSession for FakeProvider {
        async fn run_turn(
            &self,
            _messages: Vec<corenius::core::TranscriptMessage>,
            _tools: Vec<ToolDefinition>,
            executor: corenius::core::ToolExecutor,
        ) -> anyhow::Result<Vec<ProviderEvent>> {
            let result = executor(ToolCall::new(
                "call-shell",
                "fake",
                "shell",
                json!({"command":"touch denied"}),
            ))
            .await?;
            Ok(vec![
                ProviderEvent::ToolResult {
                    call: ToolCall::new("call-shell", "fake", "shell", json!({})),
                    output: result.output,
                },
                ProviderEvent::Done {
                    response_id: "fake".to_string(),
                    usage: None,
                },
            ])
        }
    }

    struct FakeTools {
        executed: Arc<AtomicBool>,
    }

    #[async_trait]
    impl ToolRegistryLike for FakeTools {
        fn definitions(&self) -> Vec<ToolDefinition> {
            vec![ToolDefinition::shell(
                "shell",
                "Shell",
                json!({"type":"object"}),
            )]
        }

        async fn execute(&self, _call: ToolCall) -> anyhow::Result<serde_json::Value> {
            self.executed.store(true, Ordering::SeqCst);
            Ok(json!(SandboxResult::success("")))
        }
    }

    struct ChannelApproval {
        seen: mpsc::Sender<()>,
    }

    #[async_trait]
    impl ApprovalResponder for ChannelApproval {
        async fn request_approval(
            &self,
            _request: corenius::core::ApprovalRequest,
        ) -> anyhow::Result<ApprovalDecision> {
            self.seen.send(()).await.unwrap();
            Ok(ApprovalDecision::Denied)
        }
    }

    let dir = tempdir().unwrap();
    let store = TranscriptStore::load(dir.path().join("transcript.jsonl"))
        .await
        .unwrap();
    let (seen_tx, mut seen_rx) = mpsc::channel(1);
    let executed = Arc::new(AtomicBool::new(false));
    let coordinator = Coordinator::new(
        Arc::new(FakeProvider),
        Arc::new(FakeTools {
            executed: executed.clone(),
        }),
        PolicyGate::new(),
        store,
        Some(Arc::new(ChannelApproval { seen: seen_tx })),
    );

    let mut events = coordinator.run_user_turn("run shell").await.unwrap();
    let mut saw_approval_event = false;
    while let Some(event) = events.recv().await {
        if matches!(event, RuntimeEvent::ApprovalRequested { .. }) {
            saw_approval_event = true;
            break;
        }
    }

    assert!(saw_approval_event);
    assert!(seen_rx.try_recv().is_ok());
    assert!(!executed.load(Ordering::SeqCst));
}

#[tokio::test]
async fn turn_handle_aborts_active_provider_work() {
    struct StuckProvider;

    #[async_trait]
    impl ProviderSession for StuckProvider {
        async fn run_turn(
            &self,
            _messages: Vec<corenius::core::TranscriptMessage>,
            _tools: Vec<ToolDefinition>,
            _executor: corenius::core::ToolExecutor,
        ) -> anyhow::Result<Vec<ProviderEvent>> {
            sleep(Duration::from_secs(60)).await;
            Ok(vec![ProviderEvent::Done {
                response_id: "late".to_string(),
                usage: None,
            }])
        }
    }

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

    let dir = tempdir().unwrap();
    let store = TranscriptStore::load(dir.path().join("transcript.jsonl"))
        .await
        .unwrap();
    let coordinator = Coordinator::new(
        Arc::new(StuckProvider),
        Arc::new(NoTools),
        PolicyGate::new(),
        store,
        None,
    );

    let mut handle = coordinator.start_user_turn("wait forever").await.unwrap();
    assert!(matches!(
        handle.events.recv().await.unwrap(),
        RuntimeEvent::TurnStarted { .. }
    ));

    handle.abort();

    let next = timeout(Duration::from_secs(1), handle.events.recv())
        .await
        .unwrap();
    assert!(next.is_none());
}
