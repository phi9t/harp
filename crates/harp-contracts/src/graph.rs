use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{validate_bounded_string, ArtifactRef, ContractResult, TaskId};

const MAX_NODES: usize = 64;
const MAX_NODE_REFERENCES: usize = 64;
const MAX_AGGREGATE_REFERENCES: usize = MAX_NODES * MAX_NODE_REFERENCES;
const MAX_INSTRUCTION_BYTES: usize = 16_384;
const MAX_POLICY_BYTES: usize = 256;
const MAX_OUTPUT_SCHEMA_BYTES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskGraph {
    #[schemars(extend("const" = 1))]
    pub schema_version: u8,
    #[schemars(length(min = 1, max = 64))]
    pub nodes: Vec<TaskNode>,
}

impl TaskGraph {
    pub fn validate_shape(&self) -> ContractResult<()> {
        if self.schema_version != 1 {
            return Err(crate::invalid("schemaVersion", "must equal 1"));
        }
        if !(1..=MAX_NODES).contains(&self.nodes.len()) {
            return Err(crate::invalid(
                "nodes",
                "must contain between 1 and 64 nodes",
            ));
        }

        let mut dependency_count = 0usize;
        let mut input_count = 0usize;
        for node in &self.nodes {
            if node.dependencies.len() > MAX_NODE_REFERENCES {
                return Err(crate::invalid(
                    "dependencies",
                    "must contain at most 64 entries per node",
                ));
            }
            if node.inputs.len() > MAX_NODE_REFERENCES {
                return Err(crate::invalid(
                    "inputs",
                    "must contain at most 64 entries per node",
                ));
            }
            dependency_count = dependency_count
                .checked_add(node.dependencies.len())
                .ok_or_else(|| crate::invalid("dependencies", "aggregate count overflowed"))?;
            input_count = input_count
                .checked_add(node.inputs.len())
                .ok_or_else(|| crate::invalid("inputs", "aggregate count overflowed"))?;
            if dependency_count > MAX_AGGREGATE_REFERENCES {
                return Err(crate::invalid(
                    "dependencies",
                    "must contain at most 4096 aggregate entries",
                ));
            }
            if input_count > MAX_AGGREGATE_REFERENCES {
                return Err(crate::invalid(
                    "inputs",
                    "must contain at most 4096 aggregate entries",
                ));
            }
            validate_bounded_string(
                "instruction",
                &node.instruction,
                MAX_INSTRUCTION_BYTES,
                false,
            )?;
            validate_bounded_string("modelPolicy", &node.model_policy, MAX_POLICY_BYTES, false)?;
            validate_bounded_string(
                "permissionProfile",
                &node.permission_profile,
                MAX_POLICY_BYTES,
                false,
            )?;
            validate_bounded_string(
                "outputSchema",
                &node.output_schema,
                MAX_OUTPUT_SCHEMA_BYTES,
                false,
            )?;
            if node.budget.max_tokens == 0 {
                return Err(crate::invalid("budget.maxTokens", "must be nonzero"));
            }
            if node.budget.timeout_seconds == 0 {
                return Err(crate::invalid("budget.timeoutSeconds", "must be nonzero"));
            }
            if node.budget.max_storage_bytes == 0 {
                return Err(crate::invalid("budget.maxStorageBytes", "must be nonzero"));
            }
            if node.retry_policy.max_transient_attempts > 3 {
                return Err(crate::invalid(
                    "retryPolicy.maxTransientAttempts",
                    "must be at most 3",
                ));
            }
            for input in &node.inputs {
                input.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskNode {
    pub task_id: TaskId,
    pub kind: NodeKind,
    pub role: TaskRole,
    #[schemars(length(min = 1, max = 16_384))]
    pub instruction: String,
    #[schemars(length(max = 64))]
    pub dependencies: Vec<TaskId>,
    #[schemars(length(max = 64))]
    pub inputs: Vec<ArtifactRef>,
    pub workspace_mode: WorkspaceMode,
    #[schemars(length(min = 1, max = 256))]
    pub model_policy: String,
    #[schemars(length(min = 1, max = 256))]
    pub permission_profile: String,
    pub budget: Budget,
    #[schemars(length(min = 1, max = 4_096))]
    pub output_schema: String,
    pub retry_policy: RetryPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NodeKind {
    Analysis,
    Reducer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TaskRole {
    Explore,
    Classify,
    Implement,
    Verify,
    Critic,
    Reduce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceMode {
    ReadOnly,
    Scratch,
    GitWorktree,
    CopyOnWrite,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Budget {
    #[schemars(range(min = 1))]
    pub max_tokens: u64,
    #[schemars(range(min = 1))]
    pub timeout_seconds: u64,
    #[schemars(range(min = 1))]
    pub max_storage_bytes: u64,
}

impl Budget {
    pub const fn new(max_tokens: u64, timeout_seconds: u64, max_storage_bytes: u64) -> Self {
        Self {
            max_tokens,
            timeout_seconds,
            max_storage_bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RetryPolicy {
    #[schemars(range(max = 3))]
    pub max_transient_attempts: u8,
}
