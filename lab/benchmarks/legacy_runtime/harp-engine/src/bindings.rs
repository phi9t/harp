//! Resolve inputs from a consistent accepted-result snapshot, never worker transcripts.
use std::collections::BTreeMap;

use harp_artifacts::ArtifactStore;
use harp_contracts::{ArtifactRef, AttemptId, DependencyCondition, InputSource, RunId, TaskId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{ValidatedWorkflowV2, WorkflowValidationError};

pub use harp_contracts::WorkloadOutcome;

/// Records must be obtained from the caller's authoritative, consistent ledger snapshot.
/// This value is not proof of acceptance and must not be decoded from worker output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedTaskOutputs {
    pub run_id: RunId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub outcome: WorkloadOutcome,
    pub outputs: BTreeMap<String, ArtifactRef>,
    pub outcome_artifact: Option<ArtifactRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResolvedWorkflowInputs {
    pub schema_version: u8,
    pub run_id: RunId,
    pub task_id: TaskId,
    pub plan_sha256: String,
    pub schemas_sha256: String,
    pub inputs: Vec<ResolvedWorkflowInput>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResolvedWorkflowInput {
    pub name: String,
    pub schema: String,
    pub artifact: ArtifactRef,
    pub producer_task_id: Option<TaskId>,
    pub producer_attempt_id: Option<AttemptId>,
}

pub struct OutputSchemaRegistry {
    validators: BTreeMap<String, jsonschema::Validator>,
    digest: String,
}
impl OutputSchemaRegistry {
    pub fn new(schemas: BTreeMap<String, Value>) -> Result<Self, WorkflowValidationError> {
        let encoded =
            serde_json::to_vec(&schemas).map_err(|e| error("workflow.schema", e.to_string()))?;
        if schemas.is_empty() || schemas.len() > 64 || encoded.len() > 256 * 1024 {
            return Err(error(
                "workflow.schema",
                "schema registry exceeds cardinality or byte bounds",
            ));
        }
        let mut validators = BTreeMap::new();
        for (name, schema) in schemas {
            if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
                return Err(error("workflow.schema", "invalid logical schema name"));
            }
            reject_external_references(&schema)?;
            let validator = jsonschema::validator_for(&schema)
                .map_err(|_| error("workflow.schema", "output schema cannot compile"))?;
            validators.insert(name, validator);
        }
        Ok(Self {
            validators,
            digest: format!("{:x}", Sha256::digest(encoded)),
        })
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    fn verify(
        &self,
        schema: &str,
        artifact: &ArtifactRef,
        store: &ArtifactStore,
    ) -> Result<(), WorkflowValidationError> {
        if artifact.logical_schema.as_deref() != Some(schema) {
            return Err(error(
                "workflow.input_schema",
                "artifact logical schema differs from binding",
            ));
        }
        if artifact.size_bytes > 64 * 1024 * 1024 {
            return Err(error(
                "workflow.input_size",
                "artifact exceeds object bound",
            ));
        }
        let validator = self
            .validators
            .get(schema)
            .ok_or_else(|| error("workflow.schema", "input schema is not registered"))?;
        let bytes = store.read_verified(artifact).map_err(|_| {
            error(
                "workflow.artifact",
                "input bytes failed artifact verification",
            )
        })?;
        let value = harp_contracts::decode_strict_json(&bytes, 64 * 1024 * 1024)
            .map_err(|_| error("workflow.input_json", "input is not valid JSON"))?;
        validator.validate(&value).map_err(|_| {
            error(
                "workflow.input_schema",
                "input does not satisfy registered output schema",
            )
        })
    }
}

pub fn resolve_workflow_inputs(
    plan: &ValidatedWorkflowV2,
    run_id: &RunId,
    task_id: &TaskId,
    accepted: &BTreeMap<TaskId, AcceptedTaskOutputs>,
    store: &ArtifactStore,
    schemas: &OutputSchemaRegistry,
) -> Result<ResolvedWorkflowInputs, WorkflowValidationError> {
    let task = plan
        .workflow()
        .tasks
        .iter()
        .find(|task| task.task_id == *task_id)
        .ok_or_else(|| error("workflow.task", "consumer is not in the admitted plan"))?;
    let record = |producer: &TaskId| -> Result<&AcceptedTaskOutputs, WorkflowValidationError> {
        let value = accepted.get(producer).ok_or_else(|| {
            error(
                "workflow.input_pending",
                "producer has no accepted terminal record",
            )
        })?;
        if value.run_id != *run_id || value.task_id != *producer {
            return Err(error(
                "workflow.input_lineage",
                "producer record belongs to another task or run",
            ));
        }
        Ok(value)
    };
    for dependency in &task.dependencies {
        let producer = record(&dependency.task_id)?;
        if dependency.condition == DependencyCondition::Success
            && producer.outcome != WorkloadOutcome::Succeeded
        {
            return Err(error(
                "workflow.dependency_failed",
                "strict dependency did not succeed",
            ));
        }
    }
    let mut inputs = Vec::new();
    let mut total_bytes = task
        .outputs
        .iter()
        .try_fold(0_u64, |sum, output| sum.checked_add(output.max_bytes))
        .ok_or_else(|| error("workflow.input_size", "declared output sizes overflow"))?;
    for input in &task.inputs {
        let (artifact, producer) = match &input.source {
            InputSource::Artifact { artifact } => (artifact, None),
            InputSource::Output { task_id, output } => {
                let producer = record(task_id)?;
                if producer.outcome != WorkloadOutcome::Succeeded {
                    return Err(error(
                        "workflow.dependency_failed",
                        "failed producer cannot supply successful outputs",
                    ));
                }
                let artifact = producer.outputs.get(output).ok_or_else(|| {
                    error(
                        "workflow.input_missing",
                        "accepted producer output is missing",
                    )
                })?;
                let declaration = plan
                    .workflow()
                    .tasks
                    .iter()
                    .find(|task| task.task_id == *task_id)
                    .and_then(|task| task.outputs.iter().find(|item| item.name == *output))
                    .ok_or_else(|| {
                        error("workflow.producer_output", "producer declaration is absent")
                    })?;
                if artifact.size_bytes > declaration.max_bytes {
                    return Err(error(
                        "workflow.input_size",
                        "producer artifact exceeds output declaration",
                    ));
                }
                (artifact, Some(producer))
            }
            InputSource::Outcome { task_id } => {
                let producer = record(task_id)?;
                let artifact = producer.outcome_artifact.as_ref().ok_or_else(|| {
                    error(
                        "workflow.input_missing",
                        "producer outcome artifact is absent",
                    )
                })?;
                (artifact, Some(producer))
            }
        };
        total_bytes = total_bytes
            .checked_add(artifact.size_bytes)
            .filter(|total| *total <= task.limits.storage_bytes)
            .ok_or_else(|| {
                error(
                    "workflow.input_size",
                    "input references exceed consumer storage budget",
                )
            })?;
        schemas.verify(&input.schema, artifact, store)?;
        if matches!(input.source, InputSource::Outcome { .. }) {
            let producer = producer
                .ok_or_else(|| error("workflow.input_lineage", "outcome has no producer"))?;
            let bytes = store.read_verified(artifact).map_err(|_| {
                error(
                    "workflow.artifact",
                    "outcome bytes failed artifact verification",
                )
            })?;
            let value = harp_contracts::decode_strict_json(&bytes, 256 * 1024)
                .map_err(|_| error("workflow.outcome", "invalid bounded terminal outcome JSON"))?;
            let terminal: harp_contracts::TaskOutcomeRecord = serde_json::from_value(value)
                .map_err(|_| error("workflow.outcome", "invalid typed terminal outcome"))?;
            terminal
                .validate()
                .map_err(|_| error("workflow.outcome", "invalid terminal outcome fields"))?;
            if terminal.run_id != producer.run_id
                || terminal.task_id != producer.task_id
                || terminal.attempt_id != producer.attempt_id
                || terminal.outcome != producer.outcome
            {
                return Err(error(
                    "workflow.input_lineage",
                    "terminal outcome differs from authoritative producer record",
                ));
            }
        }
        inputs.push(ResolvedWorkflowInput {
            name: input.name.clone(),
            schema: input.schema.clone(),
            artifact: artifact.clone(),
            producer_task_id: producer.map(|p| p.task_id.clone()),
            producer_attempt_id: producer.map(|p| p.attempt_id.clone()),
        });
    }
    Ok(ResolvedWorkflowInputs {
        schema_version: 1,
        run_id: run_id.clone(),
        task_id: task_id.clone(),
        plan_sha256: plan.digest().to_owned(),
        schemas_sha256: schemas.digest().to_owned(),
        inputs,
    })
}

fn reject_external_references(value: &Value) -> Result<(), WorkflowValidationError> {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if matches!(key.as_str(), "$ref" | "$dynamicRef" | "$recursiveRef")
                    && !value.as_str().is_some_and(|v| v.starts_with('#'))
                {
                    return Err(error(
                        "workflow.schema",
                        "external schema references are not admitted",
                    ));
                }
                // A base URI changes even fragment reference resolution.
                if key == "$id" {
                    return Err(error("workflow.schema", "schema base URI is not admitted"));
                }
                reject_external_references(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                reject_external_references(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn error(code: &'static str, message: impl Into<String>) -> WorkflowValidationError {
    WorkflowValidationError::new(code, None, message)
}
