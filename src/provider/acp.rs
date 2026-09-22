use std::path::PathBuf;

use anyhow::{Context, bail};
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{Duration, timeout};

use crate::core::{
    ProviderEvent, ProviderEventSink, ProviderSession, ToolDefinition, ToolExecutor,
    TranscriptMessage, UsageRecord,
};
use crate::supervisor::{ProcessSnapshot, ProcessSupervisor};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpOptions {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub model: Option<String>,
    pub prompt: String,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpOutput {
    pub text: String,
    pub session_id: String,
    pub stop_reason: Option<String>,
    pub usage: Option<UsageRecord>,
    pub supervision: Vec<ProcessSnapshot>,
}

pub type SupervisorSink =
    std::sync::Arc<dyn Fn(ProcessSnapshot) -> anyhow::Result<()> + Send + Sync>;

#[derive(Clone, Debug)]
pub struct TraeCliAcpProvider {
    pub command: String,
    pub cwd: PathBuf,
    pub model: Option<String>,
    pub timeout_ms: u64,
    pub serve_args: Option<Vec<String>>,
}

impl TraeCliAcpProvider {
    pub fn new(cwd: PathBuf, model: Option<String>) -> Self {
        Self {
            command: std::env::var("TRAECLI_PATH").unwrap_or_else(|_| "traecli".to_string()),
            cwd,
            model,
            timeout_ms: 120_000,
            serve_args: None,
        }
    }

    pub fn args(&self) -> Vec<String> {
        if let Some(args) = &self.serve_args {
            return args.clone();
        }
        let mut args = vec!["acp".to_string(), "serve".to_string()];
        if let Some(model) = &self.model {
            args.push("-c".to_string());
            args.push(format!("model={}", toml_string(model)));
        }
        args
    }
}

#[async_trait]
impl ProviderSession for TraeCliAcpProvider {
    async fn run_turn(
        &self,
        messages: Vec<TranscriptMessage>,
        _tools: Vec<ToolDefinition>,
        _executor: ToolExecutor,
    ) -> anyhow::Result<Vec<ProviderEvent>> {
        let prompt = messages
            .last()
            .map(|message| message.content.clone())
            .unwrap_or_default();
        let output = run_acp_prompt(AcpOptions {
            command: self.command.clone(),
            args: self.args(),
            cwd: self.cwd.clone(),
            model: self.model.clone(),
            prompt,
            timeout_ms: self.timeout_ms,
        })
        .await?;
        Ok(vec![
            ProviderEvent::AssistantDelta { delta: output.text },
            ProviderEvent::Done {
                response_id: output.session_id,
                usage: output.usage,
            },
        ])
    }

    async fn stream_turn(
        &self,
        messages: Vec<TranscriptMessage>,
        _tools: Vec<ToolDefinition>,
        _executor: ToolExecutor,
        sink: ProviderEventSink,
    ) -> anyhow::Result<()> {
        let prompt = messages
            .last()
            .map(|message| message.content.clone())
            .unwrap_or_default();
        let output = run_acp_prompt_streaming(
            AcpOptions {
                command: self.command.clone(),
                args: self.args(),
                cwd: self.cwd.clone(),
                model: self.model.clone(),
                prompt,
                timeout_ms: self.timeout_ms,
            },
            Some(sink.clone()),
        )
        .await?;
        sink(ProviderEvent::Done {
            response_id: output.session_id,
            usage: output.usage,
        })
        .await
    }
}

pub async fn run_acp_prompt(options: AcpOptions) -> anyhow::Result<AcpOutput> {
    run_acp_prompt_observed(options, None, None).await
}

pub async fn run_acp_prompt_streaming(
    options: AcpOptions,
    sink: Option<ProviderEventSink>,
) -> anyhow::Result<AcpOutput> {
    run_acp_prompt_observed(options, sink, None).await
}

pub async fn run_acp_prompt_observed(
    options: AcpOptions,
    sink: Option<ProviderEventSink>,
    supervisor_sink: Option<SupervisorSink>,
) -> anyhow::Result<AcpOutput> {
    timeout(
        Duration::from_millis(options.timeout_ms),
        run_acp_prompt_inner(options, sink, supervisor_sink),
    )
    .await
    .context("ACP prompt timed out")?
}

async fn run_acp_prompt_inner(
    options: AcpOptions,
    sink: Option<ProviderEventSink>,
    supervisor_sink: Option<SupervisorSink>,
) -> anyhow::Result<AcpOutput> {
    let mut child = Command::new(&options.command)
        .args(&options.args)
        .current_dir(&options.cwd)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn ACP server {}", options.command))?;
    let root_pid = child.id().context("ACP child pid unavailable")?;
    let mut supervision = Vec::new();
    capture_supervision(root_pid, &supervisor_sink, &mut supervision)?;
    let mut stdin = child.stdin.take().context("ACP child stdin unavailable")?;
    let stdout = child
        .stdout
        .take()
        .context("ACP child stdout unavailable")?;
    let stderr = child
        .stderr
        .take()
        .context("ACP child stderr unavailable")?;
    let mut lines = BufReader::new(stdout).lines();
    let stderr_task = tokio::spawn(async move {
        let mut stderr = BufReader::new(stderr).lines();
        let mut text = String::new();
        while let Ok(Some(line)) = stderr.next_line().await {
            text.push_str(&line);
            text.push('\n');
        }
        text
    });

    send(
        &mut stdin,
        1,
        "initialize",
        json!({
            "protocolVersion": 1,
            "clientCapabilities": {},
            "clientInfo": {"name": "corenius", "version": crate::VERSION}
        }),
    )
    .await?;
    expect_response(&mut lines, 1).await?;
    capture_supervision(root_pid, &supervisor_sink, &mut supervision)?;

    send(
        &mut stdin,
        2,
        "session/new",
        json!({
            "cwd": options.cwd,
            "mcpServers": []
        }),
    )
    .await?;
    let session_response = expect_response(&mut lines, 2).await?;
    capture_supervision(root_pid, &supervisor_sink, &mut supervision)?;
    let session_id = session_response
        .get("sessionId")
        .and_then(Value::as_str)
        .context("ACP session/new response did not include sessionId")?
        .to_string();

    send(
        &mut stdin,
        3,
        "session/prompt",
        json!({
            "sessionId": session_id,
            "prompt": [{"type": "text", "text": options.prompt}]
        }),
    )
    .await?;
    capture_supervision(root_pid, &supervisor_sink, &mut supervision)?;

    let mut text = String::new();
    let (stop_reason, usage) = loop {
        let line = lines
            .next_line()
            .await?
            .context("ACP server exited before prompt completed")?;
        let message: Value = serde_json::from_str(&line).context("parse ACP JSON-RPC line")?;
        if let Some(error) = message.get("error") {
            bail!("ACP error: {}", error_summary(error));
        }
        if message.get("id").and_then(Value::as_u64) == Some(3) {
            let result = &message["result"];
            let stop_reason = result
                .get("stopReason")
                .and_then(Value::as_str)
                .map(str::to_string);
            let usage = usage_of(result.get("usage").unwrap_or(&Value::Null));
            break (stop_reason, usage);
        }
        if message.get("method").and_then(Value::as_str) == Some("session/update")
            && let Some(chunk) = update_text(&message)
        {
            text.push_str(&chunk);
            if let Some(sink) = &sink {
                sink(ProviderEvent::AssistantDelta { delta: chunk }).await?;
            }
            capture_supervision(root_pid, &supervisor_sink, &mut supervision)?;
        }
    };

    capture_supervision(root_pid, &supervisor_sink, &mut supervision)?;
    drop(stdin);
    let _ = child.kill().await;
    let _ = child.wait().await;
    let stderr = stderr_task.await.unwrap_or_default();
    if text.is_empty() && !stderr.trim().is_empty() {
        bail!("ACP prompt produced no text; stderr: {}", stderr.trim());
    }
    Ok(AcpOutput {
        text,
        session_id,
        stop_reason,
        usage,
        supervision,
    })
}

fn capture_supervision(
    root_pid: u32,
    sink: &Option<SupervisorSink>,
    snapshots: &mut Vec<ProcessSnapshot>,
) -> anyhow::Result<()> {
    if sink.is_none() {
        return Ok(());
    }
    let snapshot = ProcessSupervisor::system().snapshot(root_pid)?;
    if let Some(sink) = sink {
        sink(snapshot.clone())?;
    }
    snapshots.push(snapshot);
    Ok(())
}

async fn send(
    stdin: &mut tokio::process::ChildStdin,
    id: u64,
    method: &str,
    params: Value,
) -> anyhow::Result<()> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    });
    stdin
        .write_all(serde_json::to_string(&payload)?.as_bytes())
        .await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;
    Ok(())
}

async fn expect_response(
    lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    id: u64,
) -> anyhow::Result<Value> {
    loop {
        let line = lines
            .next_line()
            .await?
            .context("ACP server exited before response")?;
        let message: Value = serde_json::from_str(&line).context("parse ACP JSON-RPC line")?;
        if message.get("id").and_then(Value::as_u64) != Some(id) {
            continue;
        }
        if let Some(error) = message.get("error") {
            bail!("ACP error: {}", error_summary(error));
        }
        return Ok(message["result"].clone());
    }
}

fn update_text(message: &Value) -> Option<String> {
    let update = &message["params"]["update"];
    if update.get("sessionUpdate").and_then(Value::as_str) != Some("agent_message_chunk") {
        return None;
    }
    update
        .get("content")
        .and_then(|content| content.get("text"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn usage_of(value: &Value) -> Option<UsageRecord> {
    Some(UsageRecord {
        input_tokens: value
            .get("inputTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        output_tokens: value
            .get("outputTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        cache_read_tokens: value
            .get("cachedReadTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        cache_write_tokens: value
            .get("cachedWriteTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
    })
}

fn error_summary(error: &Value) -> String {
    error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("unknown ACP error")
        .to_string()
}

fn toml_string(value: &str) -> String {
    format!("{:?}", value)
}
