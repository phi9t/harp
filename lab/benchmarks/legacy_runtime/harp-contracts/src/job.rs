//! Terminal workload evidence, distinct from an agent's transport result.
use serde::{Deserialize, Serialize};

use crate::{ArtifactRef, AttemptId, ContractError, ContractResult, RunId, TaskId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkloadOutcome {
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskOutcomeRecord {
    pub schema_version: u8,
    pub run_id: RunId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub outcome: WorkloadOutcome,
    pub reason_code: String,
    pub evidence: Vec<ArtifactRef>,
}

impl TaskOutcomeRecord {
    pub fn validate(&self) -> ContractResult<()> {
        if self.schema_version != 1 {
            return Err(ContractError::new("schemaVersion", "must equal 1"));
        }
        self.reason_code.parse::<TaskId>()?;
        if self.evidence.len() > 64 {
            return Err(ContractError::new(
                "evidence",
                "must contain at most 64 references",
            ));
        }
        for artifact in &self.evidence {
            artifact.validate()?;
        }
        Ok(())
    }
}
