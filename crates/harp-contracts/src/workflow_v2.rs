//! Native workflow authoring contract. This does not itself authorize execution.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use crate::{ArtifactRef, ContractError, ContractResult, TaskId, TaskRole, WorkspaceMode};

const MAX_PLAN_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkflowV2 {
    pub schema_version: u8,
    pub name: String,
    pub limits: WorkflowLimits,
    pub failure_policy: WorkflowFailurePolicy,
    pub tasks: Vec<WorkflowTask>,
    pub outputs: Vec<TaskOutputRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkflowLimits {
    pub agent_tokens: u64,
    pub cpu_seconds: u64,
    pub gpu_seconds: u64,
    pub wall_seconds: u64,
    pub storage_bytes: u64,
    pub max_concurrency: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskResources {
    pub agent_tokens: u64,
    pub cpu_seconds: u64,
    pub gpu_seconds: u64,
    pub wall_seconds: u64,
    pub storage_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkflowFailurePolicy {
    FailFast,
    ContinueIndependent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkflowTask {
    pub task_id: TaskId,
    pub role: TaskRole,
    pub dependencies: Vec<TaskDependency>,
    pub inputs: Vec<WorkflowInput>,
    pub backend: BackendIdentity,
    pub environment_sha256: String,
    pub action: TaskAction,
    pub outputs: Vec<WorkflowOutput>,
    pub limits: TaskResources,
    pub max_retries: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BackendIdentity {
    pub kind: String,
    pub version: String,
    pub config_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum TaskAction {
    Agent {
        prompt: String,
        model_policy: String,
        permission_profile: String,
        workspace_mode: WorkspaceMode,
    },
    Command {
        executable: String,
        executable_sha256: String,
        arguments: Vec<CommandArgument>,
        cwd: String,
        env: BTreeMap<String, String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum CommandArgument {
    Literal { value: String },
    Input { name: String },
    Output { name: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskDependency {
    pub task_id: TaskId,
    pub condition: DependencyCondition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyCondition {
    Success,
    Settled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskOutputRef {
    pub task_id: TaskId,
    pub output: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkflowInput {
    pub name: String,
    pub schema: String,
    pub source: InputSource,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum InputSource {
    Artifact { artifact: ArtifactRef },
    Output { task_id: TaskId, output: String },
    Outcome { task_id: TaskId },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkflowOutput {
    pub name: String,
    pub path: String,
    pub schema: String,
    pub max_bytes: u64,
}

impl WorkflowV2 {
    /// Strict admission parsing rejects duplicate keys before typed decoding.
    pub fn from_json(raw: &[u8]) -> ContractResult<Self> {
        let value = crate::strict_json::decode_strict_json(raw, MAX_PLAN_BYTES)?;
        let plan: Self = serde_json::from_value(value)
            .map_err(|_| invalid("workflow", "does not match the versioned workflow contract"))?;
        plan.validate_shape()?;
        Ok(plan)
    }

    pub fn canonical_bytes(&self) -> ContractResult<Vec<u8>> {
        self.validate_shape()?;
        let value = serde_json::to_value(self)
            .map_err(|_| invalid("workflow", "could not serialize workflow"))?;
        let bytes = crate::strict_json::canonical_bytes(&value)?;
        if bytes.len() > MAX_PLAN_BYTES {
            return Err(invalid("workflow", "serialized workflow exceeds 1 MiB"));
        }
        Ok(bytes)
    }

    pub fn validate_shape(&self) -> ContractResult<()> {
        if self.schema_version != 2 {
            return Err(invalid("schemaVersion", "must equal 2"));
        }
        text("name", &self.name, 128)?;
        if self.tasks.is_empty() || self.tasks.len() > 64 {
            return Err(invalid("tasks", "must contain between 1 and 64 tasks"));
        }
        if self.limits.max_concurrency == 0 || self.limits.max_concurrency > 64 {
            return Err(invalid("maxConcurrency", "must be between 1 and 64"));
        }
        if self.limits.wall_seconds == 0 || self.limits.storage_bytes == 0 {
            return Err(invalid(
                "limits",
                "wall and storage limits must be positive",
            ));
        }
        let mut ids = BTreeSet::new();
        for task in &self.tasks {
            if !ids.insert(&task.task_id) {
                return Err(invalid("taskId", "duplicate task identity"));
            }
            task.validate_shape()?;
        }
        if self.outputs.is_empty() || self.outputs.len() > 64 {
            return Err(invalid(
                "outputs",
                "must contain between 1 and 64 report outputs",
            ));
        }
        let mut outputs = BTreeSet::new();
        for output in &self.outputs {
            name(&output.output)?;
            if !outputs.insert((&output.task_id, &output.output)) {
                return Err(invalid("outputs", "duplicate report output"));
            }
        }
        Ok(())
    }
}

impl WorkflowTask {
    fn validate_shape(&self) -> ContractResult<()> {
        text("backend.kind", &self.backend.kind, 128)?;
        text("backend.version", &self.backend.version, 128)?;
        digest(&self.backend.config_sha256)?;
        digest(&self.environment_sha256)?;
        if self.max_retries > 3 {
            return Err(invalid("maxRetries", "must be at most 3"));
        }
        if self.limits.wall_seconds == 0 || self.limits.storage_bytes == 0 {
            return Err(invalid(
                "limits",
                "wall and storage limits must be positive",
            ));
        }
        self.limits.retry_exposure(self.max_retries)?;
        if self.dependencies.len() > 64 || self.inputs.len() > 64 || self.outputs.len() > 64 {
            return Err(invalid("task", "reference count exceeds 64"));
        }
        let mut dependencies = BTreeSet::new();
        for dependency in &self.dependencies {
            if !dependencies.insert(&dependency.task_id) || dependency.task_id == self.task_id {
                return Err(invalid("dependencies", "duplicate or self dependency"));
            }
        }
        let mut inputs = BTreeSet::new();
        for input in &self.inputs {
            name(&input.name)?;
            text("schema", &input.schema, 256)?;
            if !inputs.insert(&input.name) {
                return Err(invalid("inputs", "duplicate input name"));
            }
            match &input.source {
                InputSource::Artifact { artifact } => artifact.validate()?,
                InputSource::Output { output, .. } => name(output)?,
                InputSource::Outcome { .. } => {}
            }
        }
        let mut outputs = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for output in &self.outputs {
            name(&output.name)?;
            relative_path(&output.path, false)?;
            text("schema", &output.schema, 256)?;
            if !outputs.insert(&output.name) || !paths.insert(&output.path) {
                return Err(invalid("outputs", "duplicate output name or path"));
            }
            if output.max_bytes == 0 || output.max_bytes > 64 * 1024 * 1024 {
                return Err(invalid("maxBytes", "must be positive and at most 64 MiB"));
            }
        }
        match &self.action {
            TaskAction::Agent {
                prompt,
                model_policy,
                permission_profile,
                ..
            } => {
                crate::validate_bounded_string("prompt", prompt, 16_384, false)?;
                text("modelPolicy", model_policy, 256)?;
                text("permissionProfile", permission_profile, 256)?;
                if self.limits.agent_tokens == 0 {
                    return Err(invalid(
                        "agentTokens",
                        "agent tasks require a positive token limit",
                    ));
                }
            }
            TaskAction::Command {
                executable,
                executable_sha256,
                arguments,
                cwd,
                env,
            } => {
                text("executable", executable, 4096)?;
                if !Path::new(executable).is_absolute()
                    || Path::new(executable)
                        .components()
                        .any(|c| matches!(c, Component::ParentDir))
                {
                    return Err(invalid(
                        "executable",
                        "must be an absolute non-traversing path",
                    ));
                }
                digest(executable_sha256)?;
                relative_path(cwd, true)?;
                if arguments.len() > 256 || env.len() > 128 {
                    return Err(invalid(
                        "command",
                        "argument or environment count exceeds bound",
                    ));
                }
                for argument in arguments {
                    match argument {
                        CommandArgument::Literal { value } => {
                            crate::validate_bounded_string("argument", value, 16_384, true)?;
                        }
                        CommandArgument::Input { name: value } => {
                            if !inputs.contains(value) {
                                return Err(invalid(
                                    "argument",
                                    "input argument names an undeclared input",
                                ));
                            }
                        }
                        CommandArgument::Output { name: value } => {
                            if !outputs.contains(value) {
                                return Err(invalid(
                                    "argument",
                                    "output argument names an undeclared output",
                                ));
                            }
                        }
                    }
                }
                for (key, value) in env {
                    if key.is_empty()
                        || key.len() > 128
                        || !key.bytes().enumerate().all(|(i, c)| {
                            c == b'_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit())
                        })
                    {
                        return Err(invalid("env", "invalid environment variable name"));
                    }
                    crate::validate_bounded_string("env.value", value, 4096, true)?;
                }
            }
        }
        Ok(())
    }
}

impl TaskResources {
    pub fn retry_exposure(&self, retries: u8) -> ContractResult<Self> {
        let attempts = u64::from(retries) + 1;
        let multiply = |n: u64| {
            n.checked_mul(attempts)
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or_else(|| invalid("limits", "retry exposure exceeds durable integer range"))
        };
        Ok(Self {
            agent_tokens: multiply(self.agent_tokens)?,
            cpu_seconds: multiply(self.cpu_seconds)?,
            gpu_seconds: multiply(self.gpu_seconds)?,
            wall_seconds: multiply(self.wall_seconds)?,
            storage_bytes: multiply(self.storage_bytes)?,
        })
    }
}

fn invalid(field: &'static str, message: &'static str) -> ContractError {
    ContractError::new(field, message)
}
fn text(field: &'static str, value: &str, max: usize) -> ContractResult<()> {
    crate::validate_bounded_string(field, value, max, false)?;
    if value.chars().any(char::is_control) {
        return Err(invalid(field, "must not contain control characters"));
    }
    Ok(())
}
fn name(value: &str) -> ContractResult<()> {
    value.parse::<TaskId>().map(|_| ())
}
fn digest(value: &str) -> ContractResult<()> {
    if !crate::valid_sha256(value) {
        return Err(invalid("sha256", "must be lowercase SHA-256"));
    }
    Ok(())
}
fn relative_path(value: &str, allow_dot: bool) -> ContractResult<()> {
    text("path", value, 4096)?;
    if allow_dot && value == "." {
        return Ok(());
    }
    if value.contains('\\')
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || Path::new(value)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(invalid(
            "path",
            "must be a canonical relative path without traversal",
        ));
    }
    Ok(())
}
