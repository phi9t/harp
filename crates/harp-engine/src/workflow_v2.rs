//! Semantic validation for the v2 authoring boundary; no filesystem or launch effects.
use std::collections::{BTreeMap, BTreeSet};

use harp_contracts::{DependencyCondition, InputSource, TaskId, WorkflowTask, WorkflowV2};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{code}: {message}")]
pub struct WorkflowValidationError {
    pub code: &'static str,
    pub task_id: Option<TaskId>,
    pub message: String,
}

impl WorkflowValidationError {
    pub(crate) fn new(
        code: &'static str,
        task: Option<&TaskId>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            task_id: task.cloned(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedWorkflowV2 {
    workflow: WorkflowV2,
    topological_order: Vec<TaskId>,
    digest: String,
}
impl ValidatedWorkflowV2 {
    pub fn workflow(&self) -> &WorkflowV2 {
        &self.workflow
    }
    pub fn topological_order(&self) -> &[TaskId] {
        &self.topological_order
    }
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

pub fn validate_workflow_v2(
    workflow: WorkflowV2,
) -> Result<ValidatedWorkflowV2, WorkflowValidationError> {
    let bytes = workflow
        .canonical_bytes()
        .map_err(|error| error_at("workflow.shape", None, error.to_string()))?;
    let nodes: BTreeMap<_, _> = workflow
        .tasks
        .iter()
        .map(|task| (&task.task_id, task))
        .collect();
    let mut done = BTreeSet::new();
    let mut order = Vec::new();
    let mut ancestors: BTreeMap<&TaskId, BTreeSet<&TaskId>> = BTreeMap::new();
    for task in &workflow.tasks {
        for dependency in &task.dependencies {
            if !nodes.contains_key(&dependency.task_id) {
                return Err(error_at(
                    "workflow.missing_dependency",
                    Some(&task.task_id),
                    "dependency task is absent",
                ));
            }
        }
    }
    while order.len() < nodes.len() {
        let mut progress = false;
        for (id, task) in &nodes {
            if done.contains(id) || !task.dependencies.iter().all(|d| done.contains(&d.task_id)) {
                continue;
            }
            let mut parents = BTreeSet::new();
            for dependency in &task.dependencies {
                parents.insert(&dependency.task_id);
                if let Some(inherited) = ancestors.get(&dependency.task_id) {
                    parents.extend(inherited);
                }
            }
            ancestors.insert(id, parents);
            done.insert(*id);
            order.push((*id).clone());
            progress = true;
        }
        if !progress {
            return Err(error_at(
                "workflow.cycle",
                None,
                "dependency graph contains a cycle",
            ));
        }
    }
    for task in &workflow.tasks {
        for input in &task.inputs {
            match &input.source {
                InputSource::Artifact { artifact } => {
                    if artifact.logical_schema.as_deref() != Some(input.schema.as_str()) {
                        return Err(error_at(
                            "workflow.input_schema",
                            Some(&task.task_id),
                            "static artifact schema differs from input contract",
                        ));
                    }
                    if artifact.size_bytes > task.limits.storage_bytes
                        || artifact.size_bytes > 64 * 1024 * 1024
                    {
                        return Err(error_at(
                            "workflow.input_size",
                            Some(&task.task_id),
                            "static artifact exceeds task or object limit",
                        ));
                    }
                }
                InputSource::Output { task_id, output } => {
                    require_ancestor(task, task_id, &ancestors)?;
                    let producer = nodes.get(task_id).ok_or_else(|| {
                        error_at(
                            "workflow.producer",
                            Some(&task.task_id),
                            "producer is absent",
                        )
                    })?;
                    let declaration = producer
                        .outputs
                        .iter()
                        .find(|o| o.name == *output)
                        .ok_or_else(|| {
                            error_at(
                                "workflow.producer_output",
                                Some(&task.task_id),
                                "producer output is absent",
                            )
                        })?;
                    if declaration.schema != input.schema {
                        return Err(error_at(
                            "workflow.input_schema",
                            Some(&task.task_id),
                            "producer output schema differs from input contract",
                        ));
                    }
                }
                InputSource::Outcome { task_id } => {
                    require_ancestor(task, task_id, &ancestors)?;
                    if input.schema != "harp.task-outcome.v1"
                        || !task.dependencies.iter().any(|d| {
                            d.task_id == *task_id && d.condition == DependencyCondition::Settled
                        })
                    {
                        return Err(error_at("workflow.outcome_binding", Some(&task.task_id), "outcome input requires a direct settled dependency and harp.task-outcome.v1"));
                    }
                }
            }
        }
        let total_outputs = task
            .outputs
            .iter()
            .try_fold(0_u64, |sum, output| sum.checked_add(output.max_bytes))
            .ok_or_else(|| {
                error_at(
                    "workflow.output_size",
                    Some(&task.task_id),
                    "output sizes overflow",
                )
            })?;
        let static_inputs = task
            .inputs
            .iter()
            .try_fold(0_u64, |sum, input| {
                let size = match &input.source {
                    InputSource::Artifact { artifact } => artifact.size_bytes,
                    _ => 0,
                };
                sum.checked_add(size)
            })
            .ok_or_else(|| {
                error_at(
                    "workflow.input_size",
                    Some(&task.task_id),
                    "input sizes overflow",
                )
            })?;
        if total_outputs
            .checked_add(static_inputs)
            .is_none_or(|total| total > task.limits.storage_bytes)
        {
            return Err(error_at(
                "workflow.output_size",
                Some(&task.task_id),
                "declared output sizes exceed task storage budget",
            ));
        }
    }
    for output in &workflow.outputs {
        if !nodes
            .get(&output.task_id)
            .is_some_and(|node| node.outputs.iter().any(|o| o.name == output.output))
        {
            return Err(error_at(
                "workflow.report_output",
                Some(&output.task_id),
                "report names an undeclared output",
            ));
        }
    }
    validate_resources(&workflow)?;
    let digest = format!("{:x}", Sha256::digest(bytes));
    Ok(ValidatedWorkflowV2 {
        workflow,
        topological_order: order,
        digest,
    })
}

fn require_ancestor<'a>(
    task: &WorkflowTask,
    producer: &TaskId,
    ancestors: &BTreeMap<&'a TaskId, BTreeSet<&'a TaskId>>,
) -> Result<(), WorkflowValidationError> {
    if !ancestors
        .get(&task.task_id)
        .is_some_and(|ids| ids.contains(producer))
    {
        return Err(error_at(
            "workflow.input_dependency",
            Some(&task.task_id),
            "input producer must be an ancestor",
        ));
    }
    Ok(())
}

fn validate_resources(workflow: &WorkflowV2) -> Result<(), WorkflowValidationError> {
    let mut totals = [0_u64; 5];
    for task in &workflow.tasks {
        let exposure = task
            .limits
            .retry_exposure(task.max_retries)
            .map_err(|e| error_at("workflow.budget", Some(&task.task_id), e.to_string()))?;
        for (total, increment) in totals.iter_mut().zip([
            exposure.agent_tokens,
            exposure.cpu_seconds,
            exposure.gpu_seconds,
            exposure.wall_seconds,
            exposure.storage_bytes,
        ]) {
            *total = total
                .checked_add(increment)
                .filter(|v| *v <= i64::MAX as u64)
                .ok_or_else(|| {
                    error_at(
                        "workflow.budget",
                        Some(&task.task_id),
                        "aggregate resource exposure exceeds durable integer range",
                    )
                })?;
        }
    }
    let limits = &workflow.limits;
    for (total, limit) in totals.into_iter().zip([
        limits.agent_tokens,
        limits.cpu_seconds,
        limits.gpu_seconds,
        limits.wall_seconds,
        limits.storage_bytes,
    ]) {
        if limit > i64::MAX as u64 || total > limit {
            return Err(error_at(
                "workflow.budget",
                None,
                "aggregate retry exposure exceeds root budget",
            ));
        }
    }
    Ok(())
}

fn error_at(
    code: &'static str,
    task: Option<&TaskId>,
    message: impl Into<String>,
) -> WorkflowValidationError {
    WorkflowValidationError::new(code, task, message)
}
