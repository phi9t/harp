use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{validate_bounded_string, Budget, ContractResult, RetryPolicy, WorkspaceMode};

const MAX_NAME_BYTES: usize = 128;
const MAX_AGENT_CALLS: usize = 256;
const MAX_SEQUENCE_STEPS: usize = 256;
const MAX_PARALLEL_BRANCHES: usize = 64;
const MAX_PIPELINE_ITEMS: usize = 256;
const MAX_PIPELINE_STAGES: usize = 16;
const MAX_CALL_ID_BYTES: usize = 128;
const MAX_ITEM_ID_BYTES: usize = 128;
const MAX_PROMPT_BYTES: usize = 16_384;
const MAX_OUTPUT_SCHEMA_BYTES: usize = 4_096;
const MAX_POLICY_BYTES: usize = 256;
const MAX_PHASE_BYTES: usize = 256;
const MAX_LOG_BYTES: usize = 4_096;
const MAX_PIPELINE_ITEM_BYTES: usize = 16_384;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DynamicWorkflow {
    #[schemars(extend("const" = 1))]
    pub schema_version: u8,
    #[schemars(length(min = 1, max = 128))]
    pub name: String,
    pub root: DynamicWorkflowStep,
}

impl DynamicWorkflow {
    pub fn validate_shape(&self) -> ContractResult<()> {
        if self.schema_version != 1 {
            return Err(crate::invalid("schemaVersion", "must equal 1"));
        }
        validate_bounded_string("name", &self.name, MAX_NAME_BYTES, false)?;
        let mut seen_calls = HashSet::new();
        self.root.validate(&mut seen_calls)?;
        if seen_calls.len() > MAX_AGENT_CALLS {
            return Err(crate::invalid(
                "agentCalls",
                "must contain at most 256 unique call IDs",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "kind")]
pub enum DynamicWorkflowStep {
    Agent(DynamicAgentCall),
    Sequence {
        #[schemars(length(min = 1, max = 256))]
        steps: Vec<DynamicWorkflowStep>,
    },
    Parallel {
        #[schemars(length(min = 1, max = 64))]
        branches: Vec<DynamicWorkflowStep>,
    },
    Pipeline {
        #[schemars(length(min = 1, max = 256))]
        items: Vec<DynamicPipelineItem>,
        #[schemars(length(min = 1, max = 16))]
        stages: Vec<DynamicAgentCall>,
    },
    Phase {
        #[schemars(length(min = 1, max = 256))]
        title: String,
        step: Box<DynamicWorkflowStep>,
    },
    Log {
        #[schemars(length(min = 1, max = 4096))]
        message: String,
    },
}

impl DynamicWorkflowStep {
    fn validate(&self, seen_calls: &mut HashSet<String>) -> ContractResult<()> {
        match self {
            Self::Agent(agent) => agent.validate(seen_calls),
            Self::Sequence { steps } => {
                validate_nonempty_len("sequence.steps", steps.len(), MAX_SEQUENCE_STEPS)?;
                for step in steps {
                    step.validate(seen_calls)?;
                }
                Ok(())
            }
            Self::Parallel { branches } => {
                validate_nonempty_len("parallel.branches", branches.len(), MAX_PARALLEL_BRANCHES)?;
                for branch in branches {
                    branch.validate(seen_calls)?;
                }
                Ok(())
            }
            Self::Pipeline { items, stages } => {
                validate_nonempty_len("pipeline.items", items.len(), MAX_PIPELINE_ITEMS)?;
                validate_nonempty_len("pipeline.stages", stages.len(), MAX_PIPELINE_STAGES)?;
                let mut seen_items = HashSet::with_capacity(items.len());
                for item in items {
                    item.validate(&mut seen_items)?;
                }
                for stage in stages {
                    stage.validate(seen_calls)?;
                }
                Ok(())
            }
            Self::Phase { title, step } => {
                validate_bounded_string("phase.title", title, MAX_PHASE_BYTES, false)?;
                step.validate(seen_calls)
            }
            Self::Log { message } => {
                validate_bounded_string("log.message", message, MAX_LOG_BYTES, false)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DynamicAgentCall {
    #[schemars(length(min = 1, max = 128))]
    pub call_id: String,
    #[schemars(length(min = 1, max = 16_384))]
    pub prompt: String,
    #[schemars(length(min = 1, max = 4_096))]
    pub output_schema: String,
    #[schemars(length(min = 1, max = 256))]
    pub model_policy: String,
    #[schemars(length(min = 1, max = 256))]
    pub permission_profile: String,
    pub workspace_mode: WorkspaceMode,
    pub budget: Budget,
    pub retry_policy: RetryPolicy,
}

impl DynamicAgentCall {
    fn validate(&self, seen_calls: &mut HashSet<String>) -> ContractResult<()> {
        validate_bounded_string("callId", &self.call_id, MAX_CALL_ID_BYTES, false)?;
        validate_bounded_string("prompt", &self.prompt, MAX_PROMPT_BYTES, false)?;
        validate_bounded_string(
            "outputSchema",
            &self.output_schema,
            MAX_OUTPUT_SCHEMA_BYTES,
            false,
        )?;
        validate_bounded_string("modelPolicy", &self.model_policy, MAX_POLICY_BYTES, false)?;
        validate_bounded_string(
            "permissionProfile",
            &self.permission_profile,
            MAX_POLICY_BYTES,
            false,
        )?;
        if self.budget.max_tokens == 0 {
            return Err(crate::invalid("budget.maxTokens", "must be nonzero"));
        }
        if self.budget.timeout_seconds == 0 {
            return Err(crate::invalid("budget.timeoutSeconds", "must be nonzero"));
        }
        if self.budget.max_storage_bytes == 0 {
            return Err(crate::invalid("budget.maxStorageBytes", "must be nonzero"));
        }
        if self.retry_policy.max_transient_attempts > 3 {
            return Err(crate::invalid(
                "retryPolicy.maxTransientAttempts",
                "must be at most 3",
            ));
        }
        if !seen_calls.insert(self.call_id.clone()) {
            return Err(crate::invalid(
                "callId",
                "must be unique across workflow agent calls",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DynamicPipelineItem {
    #[schemars(length(min = 1, max = 128))]
    pub item_id: String,
    pub input: Value,
}

impl DynamicPipelineItem {
    fn validate(&self, seen_items: &mut HashSet<String>) -> ContractResult<()> {
        validate_bounded_string("itemId", &self.item_id, MAX_ITEM_ID_BYTES, false)?;
        let bytes = serde_json::to_vec(&self.input)
            .map_err(|_| crate::invalid("pipeline.input", "could not serialize as JSON"))?;
        if bytes.len() > MAX_PIPELINE_ITEM_BYTES {
            return Err(crate::invalid(
                "pipeline.input",
                "serialized JSON exceeds the maximum byte length",
            ));
        }
        if !seen_items.insert(self.item_id.clone()) {
            return Err(crate::invalid("itemId", "must be unique within a pipeline"));
        }
        Ok(())
    }
}

fn validate_nonempty_len(field: &'static str, len: usize, max: usize) -> ContractResult<()> {
    if len == 0 {
        return Err(crate::invalid(field, "must not be empty"));
    }
    if len > max {
        return Err(crate::invalid(field, "exceeds the maximum length"));
    }
    Ok(())
}
