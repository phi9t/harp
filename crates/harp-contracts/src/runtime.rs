use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{invalid, ContractResult, OperationId, ThreadId, TurnId};

const KIB: usize = 1024;

#[derive(Clone, Copy)]
enum ControlPolicy {
    None,
    Newline,
    Text,
}

fn validate_string_allow_empty(
    field: &'static str,
    value: &str,
    max_bytes: usize,
    control_policy: ControlPolicy,
) -> ContractResult<()> {
    if value.len() > max_bytes {
        return Err(invalid(field, "exceeds the maximum byte length"));
    }
    if value.chars().any(|character| {
        character.is_control()
            && !match control_policy {
                ControlPolicy::None => false,
                ControlPolicy::Newline => character == '\n',
                ControlPolicy::Text => matches!(character, '\n' | '\t'),
            }
    }) {
        return Err(invalid(field, "contains a forbidden control character"));
    }
    Ok(())
}

fn validate_required_string(
    field: &'static str,
    value: &str,
    max_bytes: usize,
    control_policy: ControlPolicy,
) -> ContractResult<()> {
    if value.is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    validate_string_allow_empty(field, value, max_bytes, control_policy)
}

fn validate_json_object(
    field: &'static str,
    value: &Value,
    max_bytes: usize,
) -> ContractResult<()> {
    if !value.is_object() {
        return Err(invalid(field, "must be a JSON object"));
    }
    let bytes =
        serde_json::to_vec(value).map_err(|_| invalid(field, "could not be serialized as JSON"))?;
    if bytes.len() > max_bytes {
        return Err(invalid(
            field,
            "serialized JSON exceeds the maximum byte length",
        ));
    }
    Ok(())
}

fn validate_thread_id(thread_id: &ThreadId) -> ContractResult<()> {
    validate_required_string("threadId", &thread_id.to_string(), 256, ControlPolicy::None)
}

fn validate_turn_id(turn_id: &TurnId) -> ContractResult<()> {
    validate_required_string("turnId", &turn_id.to_string(), 256, ControlPolicy::None)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RuntimeWorkspaceAuthority {
    #[schemars(length(min = 1, max = 4_096))]
    pub canonical_path: String,
    #[schemars(length(min = 1, max = 4_096))]
    pub canonical_root: String,
    pub root_device: u64,
    pub root_inode: u64,
    pub attempt_device: u64,
    pub attempt_inode: u64,
    #[schemars(length(min = 64, max = 64))]
    pub attempt_key_sha256: String,
}

impl RuntimeWorkspaceAuthority {
    pub fn validate(&self) -> ContractResult<()> {
        validate_required_string(
            "workspaceAuthority.canonicalPath",
            &self.canonical_path,
            4096,
            ControlPolicy::None,
        )?;
        validate_required_string(
            "workspaceAuthority.canonicalRoot",
            &self.canonical_root,
            4096,
            ControlPolicy::None,
        )?;
        if self.attempt_key_sha256.len() != 64
            || !self
                .attempt_key_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(invalid(
                "workspaceAuthority.attemptKeySha256",
                "must be a lowercase SHA-256 digest",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ThreadSpec {
    #[schemars(length(max = 65_536))]
    pub base_instructions: String,
    #[schemars(length(max = 65_536))]
    pub developer_instructions: String,
    #[schemars(length(min = 1, max = 4_096))]
    pub cwd: String,
    #[schemars(length(max = 16), inner(length(min = 1, max = 4_096)))]
    pub runtime_workspace_roots: Vec<String>,
    pub workspace_authority: Option<RuntimeWorkspaceAuthority>,
    #[schemars(length(min = 1, max = 128))]
    pub approval_policy: String,
    #[schemars(length(min = 1, max = 128))]
    pub sandbox_mode: String,
    #[schemars(length(min = 1, max = 256))]
    pub model: String,
    #[schemars(length(min = 1, max = 128))]
    pub reasoning_effort: Option<String>,
    pub ephemeral: bool,
}

impl ThreadSpec {
    pub fn validate(&self) -> ContractResult<()> {
        validate_string_allow_empty(
            "baseInstructions",
            &self.base_instructions,
            64 * KIB,
            ControlPolicy::Text,
        )?;
        validate_string_allow_empty(
            "developerInstructions",
            &self.developer_instructions,
            64 * KIB,
            ControlPolicy::Text,
        )?;
        validate_required_string("cwd", &self.cwd, 4096, ControlPolicy::None)?;
        if self.runtime_workspace_roots.len() > 16 {
            return Err(invalid(
                "runtimeWorkspaceRoots",
                "must contain at most 16 entries",
            ));
        }
        for root in &self.runtime_workspace_roots {
            validate_required_string("runtimeWorkspaceRoots", root, 4096, ControlPolicy::None)?;
        }
        if let Some(authority) = &self.workspace_authority {
            authority.validate()?;
            if authority.canonical_path != self.cwd {
                return Err(invalid(
                    "workspaceAuthority.canonicalPath",
                    "must equal cwd",
                ));
            }
            if self.runtime_workspace_roots != [authority.canonical_path.clone()] {
                return Err(invalid(
                    "runtimeWorkspaceRoots",
                    "must contain exactly the authoritative cwd",
                ));
            }
        }
        validate_required_string(
            "approvalPolicy",
            &self.approval_policy,
            128,
            ControlPolicy::None,
        )?;
        validate_required_string("sandboxMode", &self.sandbox_mode, 128, ControlPolicy::None)?;
        validate_required_string("model", &self.model, 256, ControlPolicy::None)?;
        if let Some(reasoning_effort) = &self.reasoning_effort {
            validate_required_string(
                "reasoningEffort",
                reasoning_effort,
                128,
                ControlPolicy::None,
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TurnSpec {
    #[schemars(length(min = 1, max = 262_144))]
    pub instruction: String,
    pub operation_marker: OperationId,
    pub output_schema: Value,
    #[schemars(length(min = 1, max = 256))]
    pub model: Option<String>,
    #[schemars(length(min = 1, max = 128))]
    pub reasoning_effort: Option<String>,
}

impl TurnSpec {
    pub fn validate(&self) -> ContractResult<()> {
        validate_required_string(
            "instruction",
            &self.instruction,
            256 * KIB,
            ControlPolicy::Text,
        )?;
        validate_required_string(
            "operationMarker",
            &self.operation_marker.to_string(),
            256,
            ControlPolicy::None,
        )?;
        validate_json_object("outputSchema", &self.output_schema, 256 * KIB)?;
        if let Some(model) = &self.model {
            validate_required_string("model", model, 256, ControlPolicy::None)?;
        }
        if let Some(reasoning_effort) = &self.reasoning_effort {
            validate_required_string(
                "reasoningEffort",
                reasoning_effort,
                128,
                ControlPolicy::None,
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ThreadHandle {
    pub thread_id: ThreadId,
}

impl ThreadHandle {
    pub fn validate(&self) -> ContractResult<()> {
        validate_thread_id(&self.thread_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TurnHandle {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
}

impl TurnHandle {
    pub fn validate(&self) -> ContractResult<()> {
        validate_thread_id(&self.thread_id)?;
        validate_turn_id(&self.turn_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ThreadSnapshot {
    pub thread_id: ThreadId,
    pub status: ThreadStatus,
    #[schemars(length(max = 1024))]
    pub turns: Vec<TurnSnapshot>,
}

impl ThreadSnapshot {
    pub fn validate(&self) -> ContractResult<()> {
        validate_thread_id(&self.thread_id)?;
        if self.turns.len() > 1024 {
            return Err(invalid("turns", "must contain at most 1024 entries"));
        }
        let mut operation_markers = std::collections::HashSet::new();
        for turn in &self.turns {
            turn.validate()?;
            if let Some(marker) = &turn.operation_marker {
                if !operation_markers.insert(marker) {
                    return Err(invalid(
                        "operationMarker",
                        "must be unique across thread turns",
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn turn_by_operation_marker(
        &self,
        marker: &OperationId,
    ) -> ContractResult<Option<&TurnSnapshot>> {
        let mut matching = self
            .turns
            .iter()
            .filter(|turn| turn.operation_marker.as_ref() == Some(marker));
        let result = matching.next();
        if matching.next().is_some() {
            return Err(invalid(
                "operationMarker",
                "must be unique across thread turns",
            ));
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TurnSnapshot {
    pub turn_id: TurnId,
    pub operation_marker: Option<OperationId>,
    pub status: TurnStatus,
    #[schemars(length(max = 1_048_576))]
    pub final_agent_message: Option<String>,
}

impl TurnSnapshot {
    pub fn validate(&self) -> ContractResult<()> {
        validate_turn_id(&self.turn_id)?;
        if let Some(marker) = &self.operation_marker {
            validate_required_string(
                "operationMarker",
                &marker.to_string(),
                256,
                ControlPolicy::None,
            )?;
        }
        if let Some(message) = &self.final_agent_message {
            validate_string_allow_empty(
                "finalAgentMessage",
                message,
                1024 * KIB,
                ControlPolicy::Text,
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ThreadStatus {
    Idle,
    Active,
    NotLoaded,
    SystemError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TurnStatus {
    Completed,
    Interrupted,
    Failed,
    InProgress,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TokenUsage {
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
}

impl TokenUsage {
    pub fn validate(&self) -> ContractResult<()> {
        if self.cached_input_tokens > self.input_tokens {
            return Err(invalid("cachedInputTokens", "must not exceed inputTokens"));
        }
        if self.reasoning_output_tokens > self.output_tokens {
            return Err(invalid(
                "reasoningOutputTokens",
                "must not exceed outputTokens",
            ));
        }
        let expected_total = self
            .input_tokens
            .checked_add(self.output_tokens)
            .ok_or_else(|| invalid("totalTokens", "input and output token sum overflowed"))?;
        if self.total_tokens != expected_total {
            return Err(invalid(
                "totalTokens",
                "must equal inputTokens plus outputTokens",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RuntimeEvent {
    ThreadStarted(ThreadStartedEvent),
    TurnStarted(TurnStartedEvent),
    TokenUsage(TokenUsageEvent),
    TurnCompleted(TurnCompletedEvent),
    ServerRequest(ServerRequestEvent),
    Lagged(LaggedEvent),
    Disconnected(DisconnectedEvent),
}

impl RuntimeEvent {
    pub fn validate(&self) -> ContractResult<()> {
        match self {
            Self::ThreadStarted(event) => event.validate(),
            Self::TurnStarted(event) => event.validate(),
            Self::TokenUsage(event) => event.validate(),
            Self::TurnCompleted(event) => event.validate(),
            Self::ServerRequest(event) => event.validate(),
            Self::Lagged(event) => event.validate(),
            Self::Disconnected(event) => event.validate(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ThreadStartedEvent {
    pub thread_id: ThreadId,
}

impl ThreadStartedEvent {
    pub fn validate(&self) -> ContractResult<()> {
        validate_thread_id(&self.thread_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TurnStartedEvent {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
}

impl TurnStartedEvent {
    pub fn validate(&self) -> ContractResult<()> {
        validate_thread_id(&self.thread_id)?;
        validate_turn_id(&self.turn_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TokenUsageEvent {
    pub thread_id: ThreadId,
    pub turn_id: TurnId,
    pub usage: TokenUsage,
}

impl TokenUsageEvent {
    pub fn validate(&self) -> ContractResult<()> {
        validate_thread_id(&self.thread_id)?;
        validate_turn_id(&self.turn_id)?;
        self.usage.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TurnCompletedEvent {
    pub thread_id: ThreadId,
    pub turn: TurnSnapshot,
}

impl TurnCompletedEvent {
    pub fn validate(&self) -> ContractResult<()> {
        validate_thread_id(&self.thread_id)?;
        self.turn.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ServerRequestEvent {
    pub thread_id: ThreadId,
    pub turn_id: Option<TurnId>,
    pub request: Value,
}

impl ServerRequestEvent {
    pub fn validate(&self) -> ContractResult<()> {
        validate_thread_id(&self.thread_id)?;
        if let Some(turn_id) = &self.turn_id {
            validate_turn_id(turn_id)?;
        }
        validate_json_object("request", &self.request, 64 * KIB)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LaggedEvent {
    #[schemars(range(min = 1))]
    pub dropped_events: u64,
}

impl LaggedEvent {
    pub fn validate(&self) -> ContractResult<()> {
        if self.dropped_events == 0 {
            return Err(invalid("droppedEvents", "must be nonzero"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DisconnectedEvent {
    #[schemars(length(min = 1, max = 4096))]
    pub reason: String,
}

impl DisconnectedEvent {
    pub fn validate(&self) -> ContractResult<()> {
        validate_required_string("reason", &self.reason, 4096, ControlPolicy::Newline)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeErrorKind {
    Startup,
    Transport,
    Protocol,
    ApprovalRequired,
    OutputSchema,
    Disconnected,
    NotFound,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RuntimeFailure {
    pub kind: RuntimeErrorKind,
    #[schemars(length(min = 1, max = 8192))]
    pub message: String,
    pub transient: bool,
}

impl RuntimeFailure {
    pub fn validate(&self) -> ContractResult<()> {
        validate_required_string("message", &self.message, 8192, ControlPolicy::Newline)
    }
}
