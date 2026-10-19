use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{valid_sha256, validate_bounded_string, ArtifactRef, ContractResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CheckStatus {
    Pass,
    Fail,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CheckResult {
    #[schemars(length(min = 1, max = 256))]
    pub code: String,
    pub status: CheckStatus,
    #[schemars(length(min = 1, max = 4096))]
    pub message: String,
    #[schemars(length(max = 256))]
    pub evidence: Vec<ArtifactRef>,
}

impl CheckResult {
    pub fn validate(&self) -> ContractResult<()> {
        validate_bounded_string("code", &self.code, 256, false)?;
        validate_bounded_string("message", &self.message, 4096, false)?;
        if self.evidence.len() > 256 {
            return Err(crate::invalid(
                "evidence",
                "must contain at most 256 artifact references",
            ));
        }
        for evidence in &self.evidence {
            evidence.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DeterministicEvaluation {
    pub passed: bool,
    #[schemars(length(max = 256))]
    pub checks: Vec<CheckResult>,
    #[schemars(length(min = 64, max = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
    pub normalized_result_sha256: String,
}

impl DeterministicEvaluation {
    pub fn validate(&self) -> ContractResult<()> {
        if !valid_sha256(&self.normalized_result_sha256) {
            return Err(crate::invalid(
                "normalizedResultSha256",
                "must contain exactly 64 lowercase hexadecimal characters",
            ));
        }
        if self.checks.len() > 256 {
            return Err(crate::invalid("checks", "must contain at most 256 results"));
        }
        for check in &self.checks {
            check.validate()?;
        }
        let all_checks_passed = self
            .checks
            .iter()
            .all(|check| check.status == CheckStatus::Pass);
        if self.passed != all_checks_passed {
            return Err(crate::invalid(
                "passed",
                "must equal whether all deterministic checks passed",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SemanticEvaluation {
    #[schemars(range(min = 0.0, max = 1.0))]
    pub architectural_correctness: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub evidence_coverage: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub boundary_accuracy: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub integration_gap_quality: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub implementation_usefulness: f64,
    #[schemars(range(min = 0.0, max = 1.0))]
    pub unsupported_claim_rate: f64,
    #[schemars(length(max = 16_384))]
    pub notes: String,
}

impl SemanticEvaluation {
    pub fn validate(&self) -> ContractResult<()> {
        validate_score("architecturalCorrectness", self.architectural_correctness)?;
        validate_score("evidenceCoverage", self.evidence_coverage)?;
        validate_score("boundaryAccuracy", self.boundary_accuracy)?;
        validate_score("integrationGapQuality", self.integration_gap_quality)?;
        validate_score("implementationUsefulness", self.implementation_usefulness)?;
        validate_score("unsupportedClaimRate", self.unsupported_claim_rate)?;
        validate_bounded_string("notes", &self.notes, 16_384, true)
    }
}

fn validate_score(field: &'static str, score: f64) -> ContractResult<()> {
    if !score.is_finite() || !(0.0..=1.0).contains(&score) {
        return Err(crate::invalid(field, "must be finite and between 0 and 1"));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AdapterKind {
    Fake,
    Process,
    InProcess,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RunMetrics {
    pub adapter: AdapterKind,
    pub startup_ms: u64,
    pub wall_ms: u64,
    pub root_tokens: u64,
    pub child_tokens: u64,
    pub peak_concurrency: u64,
    pub event_lag_count: u64,
    pub protocol_failure_count: u64,
    pub indeterminate_attempts: u64,
    pub duplicate_tokens: u64,
    pub peak_disk_bytes: u64,
    pub packaged_binary_bytes: u64,
    pub operator_steps: u64,
}

impl RunMetrics {
    pub const fn validate(&self) -> ContractResult<()> {
        Ok(())
    }
}
