use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageRecord {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolUse {
    pub read_only: bool,
    pub writes: bool,
    pub shell: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub capabilities: ToolUse,
}

impl ToolDefinition {
    pub fn read_only(name: &str, description: &str, parameters: Value) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
            capabilities: ToolUse {
                read_only: true,
                writes: false,
                shell: false,
            },
        }
    }

    pub fn shell(name: &str, description: &str, parameters: Value) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
            capabilities: ToolUse {
                read_only: false,
                writes: true,
                shell: true,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub arguments: Value,
}

impl ToolCall {
    pub fn new(id: &str, provider: &str, name: &str, arguments: Value) -> Self {
        Self {
            id: id.to_string(),
            provider: provider.to_string(),
            name: name.to_string(),
            arguments,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub tool_call: ToolCall,
    pub risk_summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approved,
    Denied,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RuntimeEvent {
    TurnStarted {
        turn_id: String,
        user_text: String,
        timestamp: String,
    },
    AssistantDelta {
        turn_id: String,
        delta: String,
        timestamp: String,
    },
    ToolRequested {
        turn_id: String,
        tool_call: ToolCall,
        timestamp: String,
    },
    ApprovalRequested {
        turn_id: String,
        request: ApprovalRequest,
        timestamp: String,
    },
    ToolResult {
        turn_id: String,
        tool_call: ToolCall,
        result: Value,
        timestamp: String,
    },
    TurnFinished {
        turn_id: String,
        response_id: Option<String>,
        usage: Option<UsageRecord>,
        timestamp: String,
    },
    TurnFailed {
        turn_id: String,
        error: String,
        timestamp: String,
    },
    SandboxStatus {
        status: String,
        detail: Option<String>,
        timestamp: String,
    },
}

impl RuntimeEvent {
    pub fn turn_started(turn_id: &str, user_text: &str) -> Self {
        Self::TurnStarted {
            turn_id: turn_id.to_string(),
            user_text: user_text.to_string(),
            timestamp: timestamp(),
        }
    }

    pub fn assistant_delta(turn_id: &str, delta: &str) -> Self {
        Self::AssistantDelta {
            turn_id: turn_id.to_string(),
            delta: delta.to_string(),
            timestamp: timestamp(),
        }
    }

    pub fn turn_finished(
        turn_id: &str,
        response_id: Option<String>,
        usage: Option<UsageRecord>,
    ) -> Self {
        Self::TurnFinished {
            turn_id: turn_id.to_string(),
            response_id,
            usage,
            timestamp: timestamp(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxCommand {
    pub command: String,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxResult {
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub elapsed_ms: u128,
    pub timed_out: bool,
}

impl SandboxResult {
    pub fn success(stdout: &str) -> Self {
        Self {
            exit_code: Some(0),
            signal: None,
            stdout: stdout.to_string(),
            stderr: String::new(),
            elapsed_ms: 0,
            timed_out: false,
        }
    }
}

pub fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}
