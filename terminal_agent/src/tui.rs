use serde_json::Value;

use crate::core::{ApprovalRequest, RuntimeEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolRow {
    pub id: String,
    pub name: String,
    pub status: String,
    pub summary: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TuiState {
    pub transcript: Vec<(String, String)>,
    pub status: String,
    pub tools: Vec<ToolRow>,
    pub approval: Option<ApprovalRequest>,
    pub sandbox_status: Option<String>,
}

impl TuiState {
    pub fn apply(&mut self, event: RuntimeEvent) {
        match event {
            RuntimeEvent::TurnStarted { user_text, .. } => {
                self.status = "running".to_string();
                self.transcript.push(("you".to_string(), user_text));
                self.transcript
                    .push(("assistant".to_string(), String::new()));
            }
            RuntimeEvent::AssistantDelta { delta, .. } => {
                if let Some((role, content)) = self.transcript.last_mut()
                    && role == "assistant"
                {
                    content.push_str(&delta);
                    return;
                }
                self.transcript.push(("assistant".to_string(), delta));
            }
            RuntimeEvent::ToolRequested { tool_call, .. } => {
                self.status = format!("tool: {}", tool_call.name);
                self.update_tool(&tool_call.id, &tool_call.name, "running", None);
            }
            RuntimeEvent::ApprovalRequested { request, .. } => {
                self.status = "approval requested".to_string();
                self.update_tool(
                    &request.tool_call.id,
                    &request.tool_call.name,
                    "pending",
                    Some(request.risk_summary.clone()),
                );
                self.approval = Some(request);
            }
            RuntimeEvent::ToolResult {
                tool_call, result, ..
            } => {
                let denied = result
                    .get("error")
                    .and_then(Value::as_str)
                    .is_some_and(|error| error.contains("denied"));
                self.update_tool(
                    &tool_call.id,
                    &tool_call.name,
                    if denied { "denied" } else { "completed" },
                    result_summary(&result),
                );
                if self
                    .approval
                    .as_ref()
                    .is_some_and(|request| request.tool_call.id == tool_call.id)
                {
                    self.approval = None;
                }
            }
            RuntimeEvent::TurnFinished { .. } => {
                self.status = "idle".to_string();
                self.approval = None;
            }
            RuntimeEvent::TurnFailed { error, .. } => {
                self.status = format!("failed: {error}");
                self.approval = None;
            }
            RuntimeEvent::SandboxStatus { status, detail, .. } => {
                self.sandbox_status = Some(match detail {
                    Some(detail) => format!("{status}: {detail}"),
                    None => status,
                });
            }
        }
    }

    fn update_tool(&mut self, id: &str, name: &str, status: &str, summary: Option<String>) {
        if let Some(row) = self.tools.iter_mut().find(|row| row.id == id) {
            row.name = name.to_string();
            row.status = status.to_string();
            row.summary = summary;
            return;
        }
        self.tools.push(ToolRow {
            id: id.to_string(),
            name: name.to_string(),
            status: status.to_string(),
            summary,
        });
    }
}

pub fn render_rows(state: &TuiState) -> Vec<String> {
    let mut rows = Vec::new();
    for (role, content) in state.transcript.iter().rev().take(12).rev() {
        rows.push(format!("{role}: {content}"));
    }
    rows.push(format!(
        "status: {}",
        if state.status.is_empty() {
            "idle"
        } else {
            &state.status
        }
    ));
    if let Some(sandbox) = &state.sandbox_status {
        rows.push(format!("sandbox: {sandbox}"));
    }
    for tool in state.tools.iter().rev().take(6).rev() {
        rows.push(format!(
            "tool {}: {} ({}){}",
            tool.status,
            tool.name,
            tool.id,
            tool.summary
                .as_ref()
                .map(|summary| format!(" - {summary}"))
                .unwrap_or_default()
        ));
    }
    if let Some(approval) = &state.approval {
        rows.push(format!("approval: {}", approval.risk_summary));
    }
    rows
}

fn result_summary(result: &Value) -> Option<String> {
    if let Some(error) = result.get("error").and_then(Value::as_str) {
        return Some(error.to_string());
    }
    let mut parts = Vec::new();
    if let Some(exit_code) = result.get("exit_code").and_then(Value::as_i64) {
        parts.push(format!("exit {exit_code}"));
    }
    if result
        .get("timed_out")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        parts.push("timed out".to_string());
    }
    if let Some(stderr) = result.get("stderr").and_then(Value::as_str)
        && !stderr.trim().is_empty()
    {
        parts.push(stderr.trim().chars().take(120).collect());
    }
    if let Some(stdout) = result.get("stdout").and_then(Value::as_str)
        && !stdout.trim().is_empty()
    {
        parts.push(stdout.trim().chars().take(120).collect());
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" - "))
    }
}
