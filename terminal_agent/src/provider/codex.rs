use std::fs;
use std::future::Future;
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;

use anyhow::{Context, bail};
use serde_json::{Value, json};

use async_trait::async_trait;

use crate::core::{
    ProviderEvent, ProviderSession, ProviderToolResult, ToolCall, ToolDefinition, ToolExecutor,
    TranscriptMessage, UsageRecord,
};

#[derive(Clone, Debug, PartialEq)]
pub struct CodexDirectOptions {
    pub model: String,
    pub instructions: String,
    pub input: Vec<Value>,
    pub tools: Vec<ToolDefinition>,
    pub max_steps: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CodexResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CodexEvent {
    TextDelta(String),
    Completed(Value),
    OutputItem(Value),
    Failed(Value),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CodexOutput {
    pub text: String,
    pub response_id: String,
    pub usage: UsageRecord,
}

#[derive(Clone, Debug)]
pub struct CodexDirectProvider {
    pub endpoint: String,
    pub instructions: String,
    pub max_steps: usize,
}

#[derive(Clone, Debug)]
pub struct CodexCredentials {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub account_id: String,
    pub path: PathBuf,
}

impl CodexDirectProvider {
    pub fn new(instructions: impl Into<String>) -> Self {
        Self {
            endpoint: "https://chatgpt.com/backend-api/codex/responses".to_string(),
            instructions: instructions.into(),
            max_steps: 8,
        }
    }
}

#[async_trait]
impl ProviderSession for CodexDirectProvider {
    async fn run_turn(
        &self,
        messages: Vec<TranscriptMessage>,
        tools: Vec<ToolDefinition>,
        executor: ToolExecutor,
    ) -> anyhow::Result<Vec<ProviderEvent>> {
        let credentials = load_codex_credentials()?;
        let client = reqwest::Client::new();
        let endpoint = self.endpoint.clone();
        let model = configured_model();
        let output = run_codex_direct(
            CodexDirectOptions {
                model,
                instructions: self.instructions.clone(),
                input: messages
                    .into_iter()
                    .map(|message| json!({"role": message.role, "content": message.content}))
                    .collect(),
                tools,
                max_steps: self.max_steps,
            },
            move |request| {
                let client = client.clone();
                let endpoint = endpoint.clone();
                let credentials = credentials.clone();
                async move {
                    let response = client
                        .post(&endpoint)
                        .bearer_auth(&credentials.access_token)
                        .header("ChatGPT-Account-Id", &credentials.account_id)
                        .header("accept", "text/event-stream")
                        .header("content-type", "application/json")
                        .header("user-agent", "corenius/1")
                        .json(&request)
                        .send()
                        .await?;
                    let status = response.status().as_u16();
                    let body = response.text().await?;
                    Ok(CodexResponse { status, body })
                }
            },
            move |call| {
                let executor = executor.clone();
                async move {
                    let result: ProviderToolResult = executor(call).await?;
                    Ok(result.output)
                }
            },
        )
        .await?;
        Ok(vec![
            ProviderEvent::AssistantDelta { delta: output.text },
            ProviderEvent::Done {
                response_id: output.response_id,
                usage: Some(output.usage),
            },
        ])
    }
}

pub fn parse_sse_events(input: &str) -> anyhow::Result<Vec<CodexEvent>> {
    if !input.ends_with("\n\n") && !input.trim_end().ends_with("data: [DONE]") {
        bail!("Codex direct response ended with an incomplete SSE event");
    }
    let mut events = Vec::new();
    for block in input.replace('\r', "").split("\n\n") {
        if block.trim().is_empty() {
            continue;
        }
        let data = block
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim_start)
            .collect::<Vec<_>>()
            .join("\n");
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let parsed: Value = serde_json::from_str(&data)
            .context("Codex direct response contained malformed SSE JSON")?;
        let kind = parsed.get("type").and_then(Value::as_str).unwrap_or("");
        match kind {
            "response.output_text.delta" => events.push(CodexEvent::TextDelta(
                parsed
                    .get("delta")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            )),
            "response.output_item.done" => {
                events.push(CodexEvent::OutputItem(parsed["item"].clone()));
            }
            "response.completed" => events.push(CodexEvent::Completed(parsed["response"].clone())),
            "response.failed" | "error" => events.push(CodexEvent::Failed(parsed)),
            _ => {}
        }
    }
    Ok(events)
}

pub async fn run_codex_direct<T, TFut, E, EFut>(
    options: CodexDirectOptions,
    mut transport: T,
    mut execute_tool: E,
) -> anyhow::Result<CodexOutput>
where
    T: FnMut(Value) -> TFut,
    TFut: Future<Output = anyhow::Result<CodexResponse>>,
    E: FnMut(ToolCall) -> EFut,
    EFut: Future<Output = anyhow::Result<Value>>,
{
    let mut input = options.input.clone();
    let mut text = String::new();
    let mut usage = UsageRecord {
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
    };

    for _step in 0..options.max_steps {
        let request = json!({
            "model": options.model,
            "instructions": options.instructions,
            "input": input,
            "tools": options.tools.iter().map(|tool| json!({
                "type": "function",
                "name": tool.name,
                "description": tool.description,
                "parameters": tool.parameters,
                "strict": false
            })).collect::<Vec<_>>(),
            "tool_choice": "auto",
            "parallel_tool_calls": true,
            "include": ["reasoning.encrypted_content"],
            "stream": true,
            "store": false
        });
        let response = transport(request).await?;
        if response.status >= 400 {
            bail!("Codex direct request failed ({})", response.status);
        }
        let mut completed = None;
        let mut observed_output = Vec::new();
        for event in parse_sse_events(&response.body)? {
            match event {
                CodexEvent::TextDelta(delta) => text.push_str(&delta),
                CodexEvent::OutputItem(item) => observed_output.push(item),
                CodexEvent::Completed(value) => completed = Some(value),
                CodexEvent::Failed(value) => bail!("Codex direct response failed: {}", value),
            }
        }
        let completed =
            completed.context("Codex direct response ended without response.completed")?;
        let response_id = completed
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        usage = add_usage(usage, usage_of(&completed));
        let output = completed
            .get("output")
            .and_then(Value::as_array)
            .filter(|items| !items.is_empty())
            .cloned()
            .unwrap_or(observed_output);
        let calls = tool_calls(&output);
        if calls.is_empty() {
            return Ok(CodexOutput {
                text,
                response_id,
                usage,
            });
        }
        let mut results = Vec::new();
        for call in calls {
            let name = call
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let call_id = call
                .get("call_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let args = call
                .get("arguments")
                .and_then(Value::as_str)
                .filter(|args| !args.is_empty())
                .map(serde_json::from_str)
                .transpose()
                .context("Tool arguments were not valid JSON")?
                .unwrap_or(Value::Object(Default::default()));
            let result = execute_tool(ToolCall::new(&call_id, "codex-direct", &name, args)).await?;
            results.push(json!({
                "type": "function_call_output",
                "call_id": call_id,
                "output": serde_json::to_string(&result)?
            }));
        }
        input = output.into_iter().chain(results).collect();
    }
    bail!("Codex exceeded Corenius tool limit")
}

fn tool_calls(output: &[Value]) -> Vec<Value> {
    output
        .iter()
        .filter(|item| {
            item.get("type").and_then(Value::as_str) == Some("function_call")
                && item.get("name").and_then(Value::as_str).is_some()
                && item.get("call_id").and_then(Value::as_str).is_some()
        })
        .cloned()
        .collect()
}

fn usage_of(value: &Value) -> UsageRecord {
    let usage = &value["usage"];
    UsageRecord {
        input_tokens: usage
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        output_tokens: usage
            .get("output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        cache_read_tokens: usage
            .get("input_tokens_details")
            .and_then(|details| details.get("cached_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        cache_write_tokens: 0,
    }
}

fn add_usage(left: UsageRecord, right: UsageRecord) -> UsageRecord {
    UsageRecord {
        input_tokens: left.input_tokens + right.input_tokens,
        output_tokens: left.output_tokens + right.output_tokens,
        cache_read_tokens: left.cache_read_tokens + right.cache_read_tokens,
        cache_write_tokens: left.cache_write_tokens + right.cache_write_tokens,
    }
}

pub fn codex_home() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".codex")
        })
}

pub fn configured_model() -> String {
    if let Ok(model) = std::env::var("SAND_CODEX_MODEL")
        && !model.trim().is_empty()
    {
        return model.trim().to_string();
    }
    let config = codex_home().join("config.toml");
    if let Ok(text) = fs::read_to_string(config) {
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("model") {
                let rest = rest.trim_start();
                if let Some(value) = rest.strip_prefix('=') {
                    return value.trim().trim_matches(['"', '\'']).to_string();
                }
            }
        }
    }
    "gpt-5.4".to_string()
}

pub fn load_codex_credentials() -> anyhow::Result<CodexCredentials> {
    let path = codex_home().join("auth.json");
    let meta = fs::symlink_metadata(&path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.file_type().is_socket() {
        bail!("Codex login credentials must be a private regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            bail!("Codex login credentials must be private");
        }
    }
    let document: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let tokens = &document["tokens"];
    let access_token = required_token(tokens, "access_token")?;
    let refresh_token = required_token(tokens, "refresh_token")?;
    let id_token = required_token(tokens, "id_token")?;
    let account_id = required_token(tokens, "account_id")?;
    if document.get("auth_mode").and_then(Value::as_str) != Some("chatgpt") {
        bail!("Codex is not signed in with ChatGPT");
    }
    Ok(CodexCredentials {
        access_token,
        refresh_token,
        id_token,
        account_id,
        path,
    })
}

fn required_token(tokens: &Value, key: &str) -> anyhow::Result<String> {
    tokens
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .with_context(|| format!("Codex auth token {key} is missing"))
}
