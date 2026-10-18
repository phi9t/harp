use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, bail};
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::fs;

use crate::core::{SandboxResult, ToolCall, ToolDefinition, ToolRegistryLike};

#[async_trait]
pub trait ShellRunner: Send + Sync {
    async fn run_shell(
        &self,
        command: String,
        cwd: PathBuf,
        timeout_ms: u64,
    ) -> anyhow::Result<SandboxResult>;
}

type ToolHandler = Arc<
    dyn Fn(ToolCall) -> futures_util::future::BoxFuture<'static, anyhow::Result<Value>>
        + Send
        + Sync,
>;

#[derive(Clone)]
struct RegisteredTool {
    definition: ToolDefinition,
    handler: ToolHandler,
}

#[derive(Clone, Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, RegisteredTool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_for_workspace(workspace: &Path, shell: Arc<dyn ShellRunner>) -> Self {
        let workspace = workspace.to_path_buf();
        let mut registry = Self::new();
        let read_workspace = workspace.clone();
        registry.register(
            ToolDefinition::read_only(
                "read_file",
                "Read a UTF-8 file from the current workspace.",
                json!({
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "path": { "type": "string", "description": "Workspace-relative file path." }
                    },
                    "required": ["path"]
                }),
            ),
            Arc::new(move |call| {
                let workspace = read_workspace.clone();
                Box::pin(async move {
                    let path = required_string(&call.arguments, "path")?;
                    let full = resolve_under_workspace(&workspace, &path)?;
                    let content = fs::read_to_string(full).await?;
                    Ok(json!({ "content": content }))
                })
            }),
        );
        let shell_workspace = workspace;
        registry.register(
            ToolDefinition::shell(
                "shell",
                "Run a shell command inside the selected insula sandbox.",
                json!({
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "command": { "type": "string", "description": "Command to run through /bin/sh -lc." },
                        "cwd": { "type": "string", "description": "Optional workspace-relative working directory." },
                        "timeoutMs": { "type": "number", "description": "Optional timeout in milliseconds." }
                    },
                    "required": ["command"]
                }),
            ),
            Arc::new(move |call| {
                let workspace = shell_workspace.clone();
                let shell = shell.clone();
                Box::pin(async move {
                    let command = required_string(&call.arguments, "command")?;
                    let cwd = match optional_string(&call.arguments, "cwd") {
                        Some(cwd) => resolve_under_workspace(&workspace, &cwd)?,
                        None => workspace.clone(),
                    };
                    let timeout_ms = optional_u64(&call.arguments, "timeoutMs").unwrap_or(120_000);
                    let result = shell.run_shell(command, cwd, timeout_ms).await?;
                    Ok(serde_json::to_value(result)?)
                })
            }),
        );
        registry
    }

    pub fn register(&mut self, definition: ToolDefinition, handler: ToolHandler) {
        self.tools.insert(
            definition.name.clone(),
            RegisteredTool {
                definition,
                handler,
            },
        );
    }
}

#[async_trait]
impl ToolRegistryLike for ToolRegistry {
    fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| tool.definition.clone())
            .collect()
    }

    async fn execute(&self, call: ToolCall) -> anyhow::Result<Value> {
        let tool = self
            .tools
            .get(&call.name)
            .with_context(|| format!("unknown tool {}", call.name))?;
        (tool.handler)(call).await
    }
}

fn required_string(arguments: &Value, key: &str) -> anyhow::Result<String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .with_context(|| format!("tool argument {key} must be a non-empty string"))
}

fn optional_string(arguments: &Value, key: &str) -> Option<String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn optional_u64(arguments: &Value, key: &str) -> Option<u64> {
    arguments.get(key).and_then(Value::as_u64)
}

fn resolve_under_workspace(workspace: &Path, requested: &str) -> anyhow::Result<PathBuf> {
    let root = workspace.canonicalize()?;
    let full = root.join(requested);
    let parent = full
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.clone());
    let canonical_parent = parent.canonicalize()?;
    if !canonical_parent.starts_with(&root) {
        bail!("path must stay under workspace");
    }
    Ok(full)
}
