mod artifact;
mod checkpoint;
mod dynamic_workflow;
mod evaluation;
mod graph;
mod id;
mod job;
mod result;
mod runtime;
mod strict_json;
mod workflow_v2;

pub use artifact::ArtifactRef;
pub use checkpoint::Checkpoint;
pub use dynamic_workflow::{
    DynamicAgentCall, DynamicPipelineItem, DynamicWorkflow, DynamicWorkflowStep,
};
pub use evaluation::{
    AdapterKind, CheckResult, CheckStatus, DeterministicEvaluation, RunMetrics, SemanticEvaluation,
};
pub use graph::{Budget, NodeKind, RetryPolicy, TaskGraph, TaskNode, TaskRole, WorkspaceMode};
pub use id::{AttemptId, ExternalSessionId, OperationId, RunId, TaskId, ThreadId, TurnId};
pub use result::{ResultEnvelope, ResultStatus};
pub use runtime::{
    DisconnectedEvent, LaggedEvent, RuntimeErrorKind, RuntimeEvent, RuntimeFailure,
    RuntimeWorkspaceAuthority, ServerRequestEvent, ThreadHandle, ThreadSnapshot, ThreadSpec,
    ThreadStartedEvent, ThreadStatus, TokenUsage, TokenUsageEvent, TurnCompletedEvent, TurnHandle,
    TurnSnapshot, TurnSpec, TurnStartedEvent, TurnStatus,
};

pub use workflow_v2::{
    BackendIdentity, CommandArgument, DependencyCondition, InputSource, TaskAction, TaskDependency,
    TaskOutputRef, TaskResources, WorkflowFailurePolicy, WorkflowInput, WorkflowLimits,
    WorkflowOutput, WorkflowTask, WorkflowV2,
};

pub use strict_json::decode_strict_json;

pub type ContractResult<T> = Result<T, ContractError>;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid {field}: {message}")]
pub struct ContractError {
    field: &'static str,
    message: &'static str,
}

impl ContractError {
    pub const fn new(field: &'static str, message: &'static str) -> Self {
        Self { field, message }
    }
}

fn invalid(field: &'static str, message: &'static str) -> ContractError {
    ContractError::new(field, message)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_bounded_string(
    field: &'static str,
    value: &str,
    max_bytes: usize,
    allow_empty: bool,
) -> ContractResult<()> {
    if !allow_empty && value.is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    if value.len() > max_bytes {
        return Err(invalid(field, "exceeds the maximum byte length"));
    }
    if value.contains('\0') {
        return Err(invalid(field, "must not contain NUL"));
    }
    Ok(())
}

pub use job::{TaskOutcomeRecord, WorkloadOutcome};
