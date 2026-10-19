use std::collections::{BTreeMap, BTreeSet};

use harp_contracts::{
    Budget, DynamicAgentCall, DynamicPipelineItem, DynamicWorkflow, DynamicWorkflowStep, NodeKind,
    RetryPolicy, TaskGraph, TaskId, TaskNode, TaskRole, WorkspaceMode,
};
use serde_json::Value;

use crate::{role_name, workspace_mode_name, GraphPolicy, ProjectionPolicy};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicWorkflowCompileOptions {
    pub reducer_model_policy: String,
    pub reducer_permission_profile: String,
    pub reducer_workspace_mode: WorkspaceMode,
    pub reducer_budget: Budget,
    pub reducer_output_schema: String,
    pub reducer_retry_policy: RetryPolicy,
    pub scratch_root: String,
    pub base_instructions: String,
    pub checkpoint_instructions: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledDynamicWorkflow {
    pub graph: TaskGraph,
    pub graph_policy: GraphPolicy,
    pub projection_policy: ProjectionPolicy,
}

#[derive(Debug, thiserror::Error)]
pub enum DynamicWorkflowCompileError {
    #[error("dynamic workflow contract is invalid")]
    InvalidContract,
    #[error("dynamic workflow compiled duplicate task id {task_id}")]
    DuplicateTaskId { task_id: String },
    #[error("dynamic workflow task id {task_id:?} is invalid")]
    InvalidTaskId {
        task_id: String,
        #[source]
        source: harp_contracts::ContractError,
    },
    #[error("dynamic workflow input for item {item_id} could not be serialized")]
    InputSerialization {
        item_id: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("dynamic workflow {field} budget overflowed")]
    BudgetOverflow { field: &'static str },
}

pub fn compile_dynamic_workflow(
    workflow: &DynamicWorkflow,
    options: &DynamicWorkflowCompileOptions,
) -> Result<CompiledDynamicWorkflow, DynamicWorkflowCompileError> {
    workflow
        .validate_shape()
        .map_err(|_| DynamicWorkflowCompileError::InvalidContract)?;

    let mut builder = GraphBuilder {
        nodes: Vec::new(),
        scratch_paths: BTreeMap::new(),
        seen_task_ids: BTreeSet::new(),
        options,
    };
    builder.compile_step(&workflow.root, &[])?;
    let reducer_id = format!("{}-reduce", workflow.name);
    let reducer_task_id = parse_task_id(&reducer_id)?;
    let analysis_dependencies = builder
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Analysis)
        .map(|node| node.task_id.clone())
        .collect::<Vec<_>>();
    builder.push_node(TaskNode {
        task_id: reducer_task_id,
        kind: NodeKind::Reducer,
        role: TaskRole::Reduce,
        instruction: format!(
            "execute {reducer_id}\n\nReturn a compact JSON object confirming the Dynamic Workflow '{}' completed all dependency tasks. Do not inspect the filesystem or run shell commands.",
            workflow.name
        ),
        dependencies: analysis_dependencies,
        inputs: Vec::new(),
        workspace_mode: options.reducer_workspace_mode,
        model_policy: options.reducer_model_policy.clone(),
        permission_profile: options.reducer_permission_profile.clone(),
        budget: options.reducer_budget.clone(),
        output_schema: options.reducer_output_schema.clone(),
        retry_policy: options.reducer_retry_policy.clone(),
    })?;

    let graph = TaskGraph {
        schema_version: 1,
        nodes: builder.nodes,
    };
    let role_instructions = [
        (
            "explore".to_owned(),
            "Execute one Dynamic Workflow agent call.".to_owned(),
        ),
        (
            "reduce".to_owned(),
            "Reduce Dynamic Workflow terminal results.".to_owned(),
        ),
    ]
    .into_iter()
    .collect();
    let projection_policy = ProjectionPolicy {
        base_instructions: options.base_instructions.clone(),
        role_instructions,
        checkpoint_instructions: options.checkpoint_instructions.clone(),
        max_bytes: 1024 * 1024,
        scratch_paths: builder.scratch_paths,
    };
    let graph_policy = graph_policy(&graph)?;
    Ok(CompiledDynamicWorkflow {
        graph,
        graph_policy,
        projection_policy,
    })
}

fn graph_policy(graph: &TaskGraph) -> Result<GraphPolicy, DynamicWorkflowCompileError> {
    let mut total_tokens = 0u64;
    let mut total_storage = 0u64;
    let mut total_timeout = 0u64;
    let mut max_node_tokens = 0u64;
    let mut max_node_storage_bytes = 0u64;
    let mut max_node_timeout_seconds = 0u64;
    let mut allowed_roles = BTreeSet::new();
    let mut allowed_output_schemas = BTreeSet::new();
    let mut allowed_model_policies = BTreeSet::new();
    let mut allowed_permission_profiles = BTreeSet::new();
    let mut allowed_workspace_modes = BTreeSet::new();

    for node in &graph.nodes {
        total_tokens = total_tokens
            .checked_add(node.budget.max_tokens)
            .ok_or(DynamicWorkflowCompileError::BudgetOverflow { field: "token" })?;
        total_storage = total_storage
            .checked_add(node.budget.max_storage_bytes)
            .ok_or(DynamicWorkflowCompileError::BudgetOverflow { field: "storage" })?;
        total_timeout = total_timeout
            .checked_add(node.budget.timeout_seconds)
            .ok_or(DynamicWorkflowCompileError::BudgetOverflow { field: "timeout" })?;
        max_node_tokens = max_node_tokens.max(node.budget.max_tokens);
        max_node_storage_bytes = max_node_storage_bytes.max(node.budget.max_storage_bytes);
        max_node_timeout_seconds = max_node_timeout_seconds.max(node.budget.timeout_seconds);
        allowed_roles.insert(role_name(node.role).to_owned());
        allowed_output_schemas.insert(node.output_schema.clone());
        allowed_model_policies.insert(node.model_policy.clone());
        allowed_permission_profiles.insert(node.permission_profile.clone());
        allowed_workspace_modes.insert(workspace_mode_name(node.workspace_mode).to_owned());
    }

    Ok(GraphPolicy {
        max_nodes: graph.nodes.len().max(1),
        max_concurrency: graph.nodes.len().clamp(1, 4),
        max_total_tokens: total_tokens.max(max_node_tokens),
        max_total_storage_bytes: total_storage.max(max_node_storage_bytes),
        max_total_timeout_seconds: total_timeout.max(max_node_timeout_seconds),
        max_node_tokens,
        max_node_storage_bytes,
        max_node_timeout_seconds,
        max_projected_prompt_bytes: 1024 * 1024,
        max_recursion_depth: 1,
        allowed_roles,
        allowed_output_schemas,
        allowed_model_policies,
        allowed_permission_profiles,
        allowed_workspace_modes,
        approved_artifacts: BTreeMap::new(),
    })
}

struct GraphBuilder<'a> {
    nodes: Vec<TaskNode>,
    scratch_paths: BTreeMap<TaskId, String>,
    seen_task_ids: BTreeSet<TaskId>,
    options: &'a DynamicWorkflowCompileOptions,
}

impl GraphBuilder<'_> {
    fn compile_step(
        &mut self,
        step: &DynamicWorkflowStep,
        dependencies: &[TaskId],
    ) -> Result<Vec<TaskId>, DynamicWorkflowCompileError> {
        match step {
            DynamicWorkflowStep::Agent(agent) => {
                let task_id = self.push_agent(agent, dependencies, None)?;
                Ok(vec![task_id])
            }
            DynamicWorkflowStep::Sequence { steps } => {
                let mut current = dependencies.to_vec();
                for step in steps {
                    current = self.compile_step(step, &current)?;
                }
                Ok(current)
            }
            DynamicWorkflowStep::Parallel { branches } => {
                let mut terminals = Vec::new();
                for branch in branches {
                    terminals.extend(self.compile_step(branch, dependencies)?);
                }
                terminals.sort();
                Ok(terminals)
            }
            DynamicWorkflowStep::Pipeline { items, stages } => {
                let mut terminals = Vec::new();
                for item in items {
                    let mut current = dependencies.to_vec();
                    for stage in stages {
                        let task_id = self.push_agent(stage, &current, Some(item))?;
                        current = vec![task_id];
                    }
                    terminals.extend(current);
                }
                terminals.sort();
                Ok(terminals)
            }
            DynamicWorkflowStep::Phase { step, .. } => self.compile_step(step, dependencies),
            DynamicWorkflowStep::Log { .. } => Ok(dependencies.to_vec()),
        }
    }

    fn push_agent(
        &mut self,
        agent: &DynamicAgentCall,
        dependencies: &[TaskId],
        item: Option<&DynamicPipelineItem>,
    ) -> Result<TaskId, DynamicWorkflowCompileError> {
        let task_id_text = item.map_or_else(
            || agent.call_id.clone(),
            |item| format!("{}-{}", item.item_id, agent.call_id),
        );
        let task_id = parse_task_id(&task_id_text)?;
        let mut sorted_dependencies = dependencies.to_vec();
        sorted_dependencies.sort();
        let instruction = match item {
            Some(item) => {
                let input = compact_json(&item.item_id, &item.input)?;
                format!(
                    "execute {task_id}\n\n{}\n\nPipeline item input: {{\"itemId\":\"{}\",\"input\":{}}}",
                    agent.prompt, item.item_id, input
                )
            }
            None => format!("execute {task_id}\n\n{}", agent.prompt),
        };
        self.push_node(TaskNode {
            task_id: task_id.clone(),
            kind: NodeKind::Analysis,
            role: TaskRole::Explore,
            instruction,
            dependencies: sorted_dependencies,
            inputs: Vec::new(),
            workspace_mode: agent.workspace_mode,
            model_policy: agent.model_policy.clone(),
            permission_profile: agent.permission_profile.clone(),
            budget: agent.budget.clone(),
            output_schema: agent.output_schema.clone(),
            retry_policy: agent.retry_policy.clone(),
        })?;
        Ok(task_id)
    }

    fn push_node(&mut self, node: TaskNode) -> Result<(), DynamicWorkflowCompileError> {
        if !self.seen_task_ids.insert(node.task_id.clone()) {
            return Err(DynamicWorkflowCompileError::DuplicateTaskId {
                task_id: node.task_id.to_string(),
            });
        }
        self.scratch_paths.insert(
            node.task_id.clone(),
            format!("{}/{}", self.options.scratch_root, node.task_id),
        );
        self.nodes.push(node);
        Ok(())
    }
}

fn parse_task_id(task_id: &str) -> Result<TaskId, DynamicWorkflowCompileError> {
    task_id
        .parse()
        .map_err(|source| DynamicWorkflowCompileError::InvalidTaskId {
            task_id: task_id.to_owned(),
            source,
        })
}

fn compact_json(item_id: &str, value: &Value) -> Result<String, DynamicWorkflowCompileError> {
    serde_json::to_string(value).map_err(|source| DynamicWorkflowCompileError::InputSerialization {
        item_id: item_id.to_owned(),
        source,
    })
}
