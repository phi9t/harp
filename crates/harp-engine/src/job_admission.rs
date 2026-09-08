//! Validate operator-approved backend qualifications before any native launch.
use std::collections::BTreeMap;

use harp_artifacts::ArtifactStore;
use harp_contracts::{TaskAction, TaskId};
use harp_runtime::CommandJobQualification;

use crate::{ValidatedWorkflowV2, WorkflowValidationError};

/// `qualifications` is trusted admission policy from the operator, not a map
/// supplied by workflow authors or workers. Artifact verification binds the
/// reviewed evidence bytes; it does not itself establish their truth.
///
/// This is one preflight check, not an admission receipt or launch authority.
/// Agent provider policy, environments, executables and inputs require their
/// own validation at the Engine admission boundary.
pub fn validate_command_backend_authority(
    plan: &ValidatedWorkflowV2,
    qualifications: &BTreeMap<TaskId, CommandJobQualification>,
    artifacts: &ArtifactStore,
) -> Result<(), WorkflowValidationError> {
    for (id, qualification) in qualifications {
        if !plan
            .workflow()
            .tasks
            .iter()
            .any(|task| task.task_id == *id && matches!(task.action, TaskAction::Command { .. }))
        {
            return Err(WorkflowValidationError::new(
                "workflow.backend_extra",
                Some(id),
                "qualification names an absent or non-command task",
            ));
        }
        if qualification.evidence.size_bytes > 256 * 1024 {
            return Err(WorkflowValidationError::new(
                "workflow.qualification_evidence",
                Some(id),
                "qualification evidence exceeds 256 KiB",
            ));
        }
    }
    for task in &plan.workflow().tasks {
        if !matches!(task.action, TaskAction::Command { .. }) {
            continue;
        }
        let qualification = qualifications.get(&task.task_id).ok_or_else(|| {
            WorkflowValidationError::new(
                "workflow.backend_missing",
                Some(&task.task_id),
                "no operator-approved command backend qualification",
            )
        })?;
        qualification
            .require(&task.backend, &task.limits)
            .map_err(|error| {
                WorkflowValidationError::new(
                    "workflow.backend_unsupported",
                    Some(&task.task_id),
                    error.to_string(),
                )
            })?;
        artifacts
            .read_verified(&qualification.evidence)
            .map_err(|error| {
                WorkflowValidationError::new(
                    "workflow.qualification_evidence",
                    Some(&task.task_id),
                    error.to_string(),
                )
            })?;
    }
    Ok(())
}
