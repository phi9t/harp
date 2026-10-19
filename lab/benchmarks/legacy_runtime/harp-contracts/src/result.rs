use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{validate_bounded_string, ArtifactRef, ContractResult, TaskId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResultEnvelope {
    #[schemars(extend("const" = 1))]
    pub schema_version: u8,
    pub task_id: TaskId,
    pub status: ResultStatus,
    pub answer_ref: Option<ArtifactRef>,
    #[schemars(length(max = 256))]
    pub evidence: Vec<ArtifactRef>,
    pub trace_ref: ArtifactRef,
    #[schemars(length(max = 4096))]
    pub summary: String,
    pub token_usage: u64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub confidence: Option<f64>,
    #[schemars(length(min = 1, max = 256))]
    pub failure_class: Option<String>,
}

impl ResultEnvelope {
    pub fn validate(&self) -> ContractResult<()> {
        if self.schema_version != 1 {
            return Err(crate::invalid("schemaVersion", "must equal 1"));
        }
        validate_bounded_string("summary", &self.summary, 4096, true)?;
        if self.evidence.len() > 256 {
            return Err(crate::invalid(
                "evidence",
                "must contain at most 256 artifact references",
            ));
        }
        if let Some(confidence) = self.confidence {
            if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                return Err(crate::invalid(
                    "confidence",
                    "must be finite and between 0 and 1",
                ));
            }
        }
        if let Some(failure_class) = &self.failure_class {
            validate_bounded_string("failureClass", failure_class, 256, false)?;
        }
        if let Some(answer_ref) = &self.answer_ref {
            answer_ref.validate()?;
        }
        for evidence in &self.evidence {
            evidence.validate()?;
        }
        self.trace_ref.validate()?;

        match self.status {
            ResultStatus::Success => {
                if self.answer_ref.is_none() {
                    return Err(crate::invalid(
                        "answerRef",
                        "is required for successful results",
                    ));
                }
                if self.failure_class.is_some() {
                    return Err(crate::invalid(
                        "failureClass",
                        "must be absent for successful results",
                    ));
                }
            }
            ResultStatus::Failed => {
                if self.answer_ref.is_some() {
                    return Err(crate::invalid(
                        "answerRef",
                        "must be absent for failed results",
                    ));
                }
                if self.failure_class.is_none() {
                    return Err(crate::invalid(
                        "failureClass",
                        "is required for failed results",
                    ));
                }
            }
            ResultStatus::Partial => {
                if self.answer_ref.is_none() && self.evidence.is_empty() {
                    return Err(crate::invalid(
                        "status",
                        "partial results require an answer or evidence",
                    ));
                }
            }
            ResultStatus::Cancelled => {
                if self.answer_ref.is_some() {
                    return Err(crate::invalid(
                        "answerRef",
                        "must be absent for cancelled results",
                    ));
                }
                if self.failure_class.is_some() {
                    return Err(crate::invalid(
                        "failureClass",
                        "must be absent for cancelled results",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResultStatus {
    Success,
    Failed,
    Partial,
    Cancelled,
}
