use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::{
    ApprovalDecision, ApprovalRequest, PolicyDecision, PolicyGate, RuntimeEvent, ToolCall,
    ToolDefinition, TranscriptMessage, TranscriptStore, UsageRecord, timestamp,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ProviderEvent {
    AssistantDelta {
        delta: String,
    },
    ToolResult {
        call: ToolCall,
        output: Value,
    },
    Done {
        response_id: String,
        usage: Option<UsageRecord>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderToolResult {
    pub call_id: String,
    pub output: Value,
}

pub type ToolExecutor = Arc<
    dyn Fn(ToolCall) -> Pin<Box<dyn Future<Output = anyhow::Result<ProviderToolResult>> + Send>>
        + Send
        + Sync,
>;

pub type ProviderEventSink = Arc<
    dyn Fn(ProviderEvent) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> + Send + Sync,
>;

#[async_trait]
pub trait ProviderSession: Send + Sync {
    async fn run_turn(
        &self,
        messages: Vec<TranscriptMessage>,
        tools: Vec<ToolDefinition>,
        executor: ToolExecutor,
    ) -> anyhow::Result<Vec<ProviderEvent>>;

    async fn stream_turn(
        &self,
        messages: Vec<TranscriptMessage>,
        tools: Vec<ToolDefinition>,
        executor: ToolExecutor,
        sink: ProviderEventSink,
    ) -> anyhow::Result<()> {
        for event in self.run_turn(messages, tools, executor).await? {
            sink(event).await?;
        }
        Ok(())
    }
}

#[async_trait]
pub trait ToolRegistryLike: Send + Sync {
    fn definitions(&self) -> Vec<ToolDefinition>;
    async fn execute(&self, call: ToolCall) -> anyhow::Result<Value>;
}

#[async_trait]
pub trait ApprovalResponder: Send + Sync {
    async fn request_approval(&self, request: ApprovalRequest) -> anyhow::Result<ApprovalDecision>;
}

#[derive(Clone)]
pub struct Coordinator {
    provider: Arc<dyn ProviderSession>,
    tools: Arc<dyn ToolRegistryLike>,
    policy: PolicyGate,
    transcript: Arc<Mutex<TranscriptStore>>,
    approvals: Option<Arc<dyn ApprovalResponder>>,
}

pub struct ActiveTurn {
    pub events: mpsc::Receiver<RuntimeEvent>,
    task: JoinHandle<()>,
}

impl ActiveTurn {
    pub fn abort(&self) {
        self.task.abort();
    }
}

impl Coordinator {
    pub fn new(
        provider: Arc<dyn ProviderSession>,
        tools: Arc<dyn ToolRegistryLike>,
        policy: PolicyGate,
        transcript: TranscriptStore,
        approvals: Option<Arc<dyn ApprovalResponder>>,
    ) -> Self {
        Self {
            provider,
            tools,
            policy,
            transcript: Arc::new(Mutex::new(transcript)),
            approvals,
        }
    }

    pub async fn run_user_turn(&self, text: &str) -> anyhow::Result<mpsc::Receiver<RuntimeEvent>> {
        Ok(self.start_user_turn(text).await?.events)
    }

    pub async fn start_user_turn(&self, text: &str) -> anyhow::Result<ActiveTurn> {
        let (tx, rx) = mpsc::channel(128);
        let runner = TurnRunner {
            turn_id: Uuid::new_v4().to_string(),
            provider: self.provider.clone(),
            tools: self.tools.clone(),
            policy: self.policy.clone(),
            transcript: self.transcript.clone(),
            approvals: self.approvals.clone(),
            events: tx,
        };
        let user_text = text.to_string();
        let task = tokio::spawn(async move {
            if let Err(error) = runner.run(user_text).await {
                eprintln!("turn failed: {error:#}");
            }
        });
        Ok(ActiveTurn { events: rx, task })
    }
}

struct TurnRunner {
    turn_id: String,
    provider: Arc<dyn ProviderSession>,
    tools: Arc<dyn ToolRegistryLike>,
    policy: PolicyGate,
    transcript: Arc<Mutex<TranscriptStore>>,
    approvals: Option<Arc<dyn ApprovalResponder>>,
    events: mpsc::Sender<RuntimeEvent>,
}

impl TurnRunner {
    async fn run(self, user_text: String) -> anyhow::Result<()> {
        self.emit(RuntimeEvent::turn_started(&self.turn_id, &user_text))
            .await?;
        let messages = {
            let transcript = self.transcript.lock().await;
            transcript.current_transcript()
        };
        let tools_for_provider = self.tools.definitions();
        let executor_runner = self.clone_for_executor();
        let executor: ToolExecutor = Arc::new(move |call| {
            let runner = executor_runner.clone_for_executor();
            Box::pin(async move { runner.execute_tool(call).await })
        });
        let event_runner = self.clone_for_executor();
        let sink: ProviderEventSink = Arc::new(move |event| {
            let runner = event_runner.clone_for_executor();
            Box::pin(async move { runner.emit_provider_event(event).await })
        });
        match self
            .provider
            .stream_turn(messages, tools_for_provider, executor, sink)
            .await
        {
            Ok(()) => {}
            Err(error) => {
                self.emit(RuntimeEvent::TurnFailed {
                    turn_id: self.turn_id.clone(),
                    error: error.to_string(),
                    timestamp: timestamp(),
                })
                .await?;
            }
        }
        Ok(())
    }

    async fn emit_provider_event(&self, event: ProviderEvent) -> anyhow::Result<()> {
        match event {
            ProviderEvent::AssistantDelta { delta } => {
                self.emit(RuntimeEvent::assistant_delta(&self.turn_id, &delta))
                    .await?;
            }
            ProviderEvent::ToolResult { call, output } => {
                self.emit(RuntimeEvent::ToolResult {
                    turn_id: self.turn_id.clone(),
                    tool_call: call,
                    result: output,
                    timestamp: timestamp(),
                })
                .await?;
            }
            ProviderEvent::Done { response_id, usage } => {
                self.emit(RuntimeEvent::turn_finished(
                    &self.turn_id,
                    Some(response_id),
                    usage,
                ))
                .await?;
            }
        }
        Ok(())
    }

    fn clone_for_executor(&self) -> Self {
        Self {
            turn_id: self.turn_id.clone(),
            provider: self.provider.clone(),
            tools: self.tools.clone(),
            policy: self.policy.clone(),
            transcript: self.transcript.clone(),
            approvals: self.approvals.clone(),
            events: self.events.clone(),
        }
    }

    async fn execute_tool(&self, call: ToolCall) -> anyhow::Result<ProviderToolResult> {
        let definition = self
            .tools
            .definitions()
            .into_iter()
            .find(|tool| tool.name == call.name)
            .with_context(|| format!("unknown tool {}", call.name))?;
        self.emit(RuntimeEvent::ToolRequested {
            turn_id: self.turn_id.clone(),
            tool_call: call.clone(),
            timestamp: timestamp(),
        })
        .await?;
        if let PolicyDecision::Ask(request) = self.policy.evaluate(&call, &definition) {
            self.emit(RuntimeEvent::ApprovalRequested {
                turn_id: self.turn_id.clone(),
                request: request.clone(),
                timestamp: timestamp(),
            })
            .await?;
            let decision = match &self.approvals {
                Some(approvals) => approvals.request_approval(request).await?,
                None => ApprovalDecision::Denied,
            };
            if decision != ApprovalDecision::Approved {
                return Ok(ProviderToolResult {
                    call_id: call.id,
                    output: json!({"isError": true, "error": "Tool denied by approval policy."}),
                });
            }
        }
        let output = self
            .tools
            .execute(call.clone())
            .await
            .unwrap_or_else(|error| json!({"isError": true, "error": error.to_string()}));
        Ok(ProviderToolResult {
            call_id: call.id,
            output,
        })
    }

    async fn emit(&self, event: RuntimeEvent) -> anyhow::Result<()> {
        {
            let mut transcript = self.transcript.lock().await;
            transcript.append(&event).await?;
        }
        self.events
            .send(event)
            .await
            .context("send runtime event")?;
        Ok(())
    }
}
