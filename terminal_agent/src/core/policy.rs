use super::{ApprovalRequest, ToolCall, ToolDefinition};

#[derive(Clone, Debug, PartialEq)]
pub enum PolicyDecision {
    Allow,
    Ask(ApprovalRequest),
}

impl PolicyDecision {
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    pub fn is_ask(&self) -> bool {
        matches!(self, Self::Ask(_))
    }
}

#[derive(Clone, Debug, Default)]
pub struct PolicyGate;

impl PolicyGate {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, call: &ToolCall, tool: &ToolDefinition) -> PolicyDecision {
        if tool.capabilities.read_only && !tool.capabilities.writes && !tool.capabilities.shell {
            return PolicyDecision::Allow;
        }
        let mut risks = Vec::new();
        if tool.capabilities.shell {
            risks.push("shell command execution");
        }
        if tool.capabilities.writes {
            risks.push("workspace writes");
        }
        if risks.is_empty() {
            risks.push("non-read-only tool use");
        }
        PolicyDecision::Ask(ApprovalRequest {
            id: format!("approval-{}", call.id),
            tool_call: call.clone(),
            risk_summary: format!("Requires approval for {}.", risks.join(" and ")),
        })
    }
}
