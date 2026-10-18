use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    pub profile: AcpSessionProfile,
}

/// Controls how much of the user's global TraeCLI configuration an ACP session
/// inherits, and how the harness detects a stalled prompt.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AcpSessionProfile {
    /// Start the session without plugins, skills, memory, MCP servers, or tools.
    pub isolated: bool,
    /// Pin `model_reasoning_effort` for the session.
    pub reasoning_effort: Option<String>,
    /// Fail the prompt when no `session/update` arrives for this long.
    pub idle_timeout_ms: Option<u64>,
}

/// TraeCLI feature flags that pull user-global context or tools into a session.
pub const ISOLATION_FEATURE_FLAGS: &[&str] = &[
    "memories",
    "goals",
    "multi_agent",
    "multi_agent_v2",
    "code_mode_host",
    "plugins",
    "plugin_hooks",
    "hooks",
];

/// Tool names disallowed in an isolated session. TraeCLI exposes different
/// names per feature set, so the list covers every known variant.
pub const ISOLATION_DISALLOWED_TOOLS: &[&str] = &[
    "exec",
    "exec_command",
    "write_stdin",
    "shell",
    "shell_command",
    "run_shell_command",
    "web_search",
    "apply_patch",
    "view_image",
    "update_plan",
    "LSP",
    "wait",
    "get_goal",
    "create_goal",
    "update_goal",
    "spawn_agent",
    "send_input",
    "wait_agent",
    "close_agent",
    "resume_agent",
];

impl AcpSessionProfile {
    pub fn serve_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(effort) = &self.reasoning_effort {
            args.push("-c".to_string());
            args.push(format!("model_reasoning_effort={}", toml_string(effort)));
        }
        if self.isolated {
            for flag in ISOLATION_FEATURE_FLAGS {
                args.push("-c".to_string());
                args.push(format!("features.{flag}=false"));
            }
            args.push("-c".to_string());
            args.push("project_doc_max_bytes=0".to_string());
            for tool in ISOLATION_DISALLOWED_TOOLS {
                args.push("--disallowed-tool".to_string());
                args.push((*tool).to_string());
            }
        }
        args
    }

    fn session_new_params(&self, cwd: &PathBuf) -> Value {
        let mut params = json!({"cwd": cwd, "mcpServers": []});
        if self.isolated {
            params["_meta"] = json!({"trae": {"skills": [], "mcpServers": []}});
        }
        params
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcpFailureKind {
    /// The overall prompt deadline elapsed.
    Timeout,
    /// No session update arrived within the idle timeout.
    Stall,
    Other,
}

/// A failed prompt together with whatever text streamed before the failure.
#[derive(Debug)]
pub struct AcpPromptFailure {
    pub kind: AcpFailureKind,
    pub message: String,
    pub partial_text: String,
}

impl std::fmt::Display for AcpPromptFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AcpPromptFailure {}

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
    pub profile: AcpSessionProfile,
}

impl TraeCliAcpProvider {
    pub fn new(cwd: PathBuf, model: Option<String>) -> Self {
        Self {
            command: std::env::var("TRAECLI_PATH").unwrap_or_else(|_| "traecli".to_string()),
            cwd,
            model,
            timeout_ms: 120_000,
            serve_args: None,
            profile: AcpSessionProfile::default(),
        }
    }

    pub fn with_profile(mut self, profile: AcpSessionProfile) -> Self {
        self.profile = profile;
        self
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
        args.extend(self.profile.serve_args());
        args
    }

    pub fn prompt_options(&self, prompt: String) -> AcpOptions {
        AcpOptions {
            command: self.command.clone(),
            args: self.args(),
            cwd: self.cwd.clone(),
            model: self.model.clone(),
            prompt,
            timeout_ms: self.timeout_ms,
            profile: self.profile.clone(),
        }
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
        let output = run_acp_prompt(self.prompt_options(prompt)).await?;
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
        let output =
            run_acp_prompt_streaming(self.prompt_options(prompt), Some(sink.clone())).await?;
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
    run_acp_prompt_detailed(options, sink, supervisor_sink)
        .await
        .map_err(anyhow::Error::from)
}

/// Like [`run_acp_prompt_observed`], but a failure carries its kind and the
/// text streamed before it, so callers can keep partial output and retry stalls.
pub async fn run_acp_prompt_detailed(
    options: AcpOptions,
    sink: Option<ProviderEventSink>,
    supervisor_sink: Option<SupervisorSink>,
) -> Result<AcpOutput, AcpPromptFailure> {
    let timeout_ms = options.timeout_ms;
    let partial = Arc::new(Mutex::new(String::new()));
    let result = timeout(
        Duration::from_millis(timeout_ms),
        run_acp_prompt_inner(options, sink, supervisor_sink, partial.clone()),
    )
    .await;
    let partial_text = partial.lock().map(|text| text.clone()).unwrap_or_default();
    match result {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(error)) => {
            let kind = if error.downcast_ref::<IdleStall>().is_some() {
                AcpFailureKind::Stall
            } else {
                AcpFailureKind::Other
            };
            Err(AcpPromptFailure {
                kind,
                message: format!("{error:#}"),
                partial_text,
            })
        }
        Err(_) => Err(AcpPromptFailure {
            kind: AcpFailureKind::Timeout,
            message: format!("ACP prompt timed out after {timeout_ms}ms"),
            partial_text,
        }),
    }
}

#[derive(Debug)]
struct IdleStall {
    idle_timeout_ms: u64,
}

impl std::fmt::Display for IdleStall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ACP prompt stalled: no session update for {}ms",
            self.idle_timeout_ms
        )
    }
}

impl std::error::Error for IdleStall {}

async fn next_line_or_stall(
    lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    idle_timeout_ms: Option<u64>,
) -> anyhow::Result<Option<String>> {
    match idle_timeout_ms {
        Some(idle_timeout_ms) => {
            match timeout(Duration::from_millis(idle_timeout_ms), lines.next_line()).await {
                Ok(line) => Ok(line?),
                Err(_) => Err(IdleStall { idle_timeout_ms }.into()),
            }
        }
        None => Ok(lines.next_line().await?),
    }
}

async fn run_acp_prompt_inner(
    options: AcpOptions,
    sink: Option<ProviderEventSink>,
    supervisor_sink: Option<SupervisorSink>,
    partial: Arc<Mutex<String>>,
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
        options.profile.session_new_params(&options.cwd),
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
        let line = next_line_or_stall(&mut lines, options.profile.idle_timeout_ms)
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
            if let Ok(mut partial) = partial.lock() {
                partial.push_str(&chunk);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_adds_no_serve_args_or_session_meta() {
        let profile = AcpSessionProfile::default();
        assert!(profile.serve_args().is_empty());
        let params = profile.session_new_params(&PathBuf::from("/run/corenius-tmp/ws"));
        assert_eq!(
            params,
            json!({"cwd": "/run/corenius-tmp/ws", "mcpServers": []})
        );
    }

    #[test]
    fn isolated_profile_disables_features_tools_and_skills() {
        let profile = AcpSessionProfile {
            isolated: true,
            reasoning_effort: Some("high".to_string()),
            idle_timeout_ms: Some(1_000),
        };
        let args = profile.serve_args();
        assert_eq!(args[0], "-c");
        assert_eq!(args[1], "model_reasoning_effort=\"high\"");
        for flag in ISOLATION_FEATURE_FLAGS {
            assert!(args.contains(&format!("features.{flag}=false")));
        }
        assert!(args.contains(&"project_doc_max_bytes=0".to_string()));
        let disallowed = args
            .windows(2)
            .filter(|pair| pair[0] == "--disallowed-tool")
            .count();
        assert_eq!(disallowed, ISOLATION_DISALLOWED_TOOLS.len());
        let params = profile.session_new_params(&PathBuf::from("/run/corenius-tmp/ws"));
        assert_eq!(params["_meta"]["trae"]["skills"], json!([]));
        assert_eq!(params["_meta"]["trae"]["mcpServers"], json!([]));
    }

    #[test]
    fn provider_args_append_profile_after_model() {
        let provider = TraeCliAcpProvider {
            command: "traecli".to_string(),
            cwd: PathBuf::from("/run/corenius-tmp/ws"),
            model: Some("gpt-5.2".to_string()),
            timeout_ms: 1_000,
            serve_args: None,
            profile: AcpSessionProfile {
                isolated: false,
                reasoning_effort: Some("medium".to_string()),
                idle_timeout_ms: None,
            },
        };
        assert_eq!(
            provider.args(),
            vec![
                "acp",
                "serve",
                "-c",
                "model=\"gpt-5.2\"",
                "-c",
                "model_reasoning_effort=\"medium\""
            ]
        );
    }
}
