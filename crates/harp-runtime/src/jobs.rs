//! Native job protocol. Connectivity errors never constitute terminal evidence.
//!
//! Implementations must qualify containment and enforce deadlines independently
//! of observers before advertising support. The existing ActivityRuntime is not
//! implicitly a CommandJobBackend.
use async_trait::async_trait;
use harp_contracts::{ArtifactRef, BackendIdentity, ContractError, TaskResources};
use serde::{Deserialize, Serialize};

use crate::RuntimeError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct JobBinding {
    pub submission_key: String,
    pub request_sha256: String,
    pub backend: BackendIdentity,
    pub host_identity: String,
    pub owner_identity: String,
    pub job_identity: String,
}

impl JobBinding {
    pub fn validate(&self) -> Result<(), ContractError> {
        for value in [
            &self.submission_key,
            &self.host_identity,
            &self.owner_identity,
            &self.job_identity,
            &self.backend.kind,
            &self.backend.version,
        ] {
            bounded(value)?;
        }
        digest(&self.request_sha256)?;
        digest(&self.backend.config_sha256)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct JobTerminalReceipt {
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub loss_reason: Option<String>,
    pub process_tree_quiescent: bool,
    pub evidence: Vec<ArtifactRef>,
}

impl JobTerminalReceipt {
    pub fn validate(&self) -> Result<(), ContractError> {
        let verdicts = usize::from(self.exit_code.is_some())
            + usize::from(self.signal.is_some())
            + usize::from(self.loss_reason.is_some());
        if verdicts != 1 || !self.process_tree_quiescent || self.evidence.is_empty() {
            return Err(ContractError::new(
                "job.terminal",
                "requires exactly one verdict, confirmed process-tree quiescence and evidence",
            ));
        }
        if self.exit_code.is_some_and(|v| !(0..=255).contains(&v))
            || self.signal.is_some_and(|v| v <= 0 || v > 127)
        {
            return Err(ContractError::new(
                "job.terminal",
                "invalid exit code or signal",
            ));
        }
        if let Some(reason) = &self.loss_reason {
            bounded(reason)?;
            if self.evidence.is_empty() {
                return Err(ContractError::new(
                    "job.terminal",
                    "authoritative loss requires evidence",
                ));
            }
        }
        artifacts(&self.evidence)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "state")]
pub enum JobStatus {
    Running,
    Unknown,
    Terminal { receipt: JobTerminalReceipt },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct JobObservation {
    pub binding: JobBinding,
    pub next_cursor: u64,
    pub status: JobStatus,
    pub evidence: Vec<ArtifactRef>,
}
impl JobObservation {
    pub fn validate_for(&self, expected: &JobBinding, cursor: u64) -> Result<(), ContractError> {
        expected.validate()?;
        self.binding.validate()?;
        if &self.binding != expected
            || self.next_cursor < cursor
            || self.next_cursor > i64::MAX as u64
        {
            return Err(ContractError::new(
                "job.observation",
                "job identity changed or cursor regressed/exceeded durable range",
            ));
        }
        artifacts(&self.evidence)?;
        if let JobStatus::Terminal { receipt } = &self.status {
            receipt.validate()?;
        }
        Ok(())
    }
}

/// A state-store epoch alone does not fence backend effects. The backend must
/// validate this authority at the execution owner, including delayed requests.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct JobOwnership {
    pub epoch: u64,
    pub authority_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CommandJobRequest {
    pub submission_key: String,
    pub request_sha256: String,
    pub backend: BackendIdentity,
    pub host_identity: String,
    /// Exact admitted command, environment and resolved-input manifest bytes.
    pub admitted_manifest: ArtifactRef,
    pub limits: TaskResources,
    pub ownership: JobOwnership,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "state")]
pub enum JobLookup {
    Present {
        binding: JobBinding,
    },
    /// Receipt must exclude both existing work and delayed creation for the key.
    AuthoritativeAbsence {
        receipt: ArtifactRef,
    },
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct JobLookupResponse {
    pub submission_key: String,
    pub request_sha256: String,
    pub backend: BackendIdentity,
    pub host_identity: String,
    pub result: JobLookup,
}
impl JobLookupResponse {
    pub fn validate_for(&self, request: &CommandJobRequest) -> Result<(), ContractError> {
        request.validate()?;
        if self.request_sha256 != request.request_sha256
            || self.submission_key != request.submission_key
            || self.backend != request.backend
            || self.host_identity != request.host_identity
        {
            return Err(ContractError::new(
                "job.lookup",
                "response does not bind the requested submission/backend/host",
            ));
        }
        match &self.result {
            JobLookup::Present { binding } => {
                binding.validate()?;
                if binding.request_sha256 != request.request_sha256
                    || binding.submission_key != request.submission_key
                    || binding.backend != request.backend
                    || binding.host_identity != request.host_identity
                {
                    return Err(ContractError::new(
                        "job.lookup",
                        "nested binding differs from request",
                    ));
                }
            }
            JobLookup::AuthoritativeAbsence { receipt } => receipt.validate()?,
            JobLookup::Unknown => (),
        }
        Ok(())
    }
}

impl JobOwnership {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.epoch == 0 || self.epoch > i64::MAX as u64 {
            return Err(ContractError::new(
                "job.ownership",
                "epoch must be positive and fit durable integer range",
            ));
        }
        digest(&self.authority_sha256)
    }
}
impl CommandJobRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        bounded(&self.submission_key)?;
        bounded(&self.host_identity)?;
        bounded(&self.backend.kind)?;
        bounded(&self.backend.version)?;
        digest(&self.backend.config_sha256)?;
        digest(&self.request_sha256)?;
        self.ownership.validate()?;
        self.admitted_manifest.validate()?;
        if self.admitted_manifest.size_bytes > 1024 * 1024
            || self.limits.wall_seconds == 0
            || self.limits.storage_bytes == 0
            || self.limits.agent_tokens != 0
        {
            return Err(ContractError::new("job.request", "requires bounded manifest and positive command wall/storage limits, without agent tokens"));
        }
        self.limits.retry_exposure(0)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct JobCollection {
    pub binding: JobBinding,
    pub output_manifest: ArtifactRef,
    pub next_cursor: u64,
    pub complete: bool,
    pub objects: Vec<ArtifactRef>,
}
impl JobCollection {
    /// A complete response still needs output-schema and digest verification
    /// before Engine acceptance; this validates transport coordinates only.
    pub fn validate_for(
        &self,
        expected: &JobBinding,
        manifest: &ArtifactRef,
        cursor: u64,
    ) -> Result<(), ContractError> {
        expected.validate()?;
        manifest.validate()?;
        if &self.binding != expected
            || &self.output_manifest != manifest
            || self.next_cursor < cursor
            || self.next_cursor > i64::MAX as u64
        {
            return Err(ContractError::new(
                "job.collection",
                "collection identity/manifest changed or cursor is invalid",
            ));
        }
        artifacts(&self.objects)
    }
}

/// Qualification is operator-approved evidence, never a worker's self-report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CommandJobQualification {
    pub backend: BackendIdentity,
    pub host_identity: String,
    pub evidence: ArtifactRef,
    pub idempotent_submission: bool,
    pub owner_fencing: bool,
    pub descendant_containment: bool,
    pub independent_deadline: bool,
    pub enforced_cpu_budget: bool,
    pub enforced_storage_budget: bool,
    pub enforced_gpu_budget: bool,
}
impl CommandJobQualification {
    pub fn require(
        &self,
        backend: &BackendIdentity,
        limits: &TaskResources,
    ) -> Result<(), ContractError> {
        bounded(&self.host_identity)?;
        self.evidence.validate()?;
        if &self.backend != backend
            || !self.idempotent_submission
            || !self.owner_fencing
            || !self.descendant_containment
            || !self.independent_deadline
            || !self.enforced_cpu_budget
            || !self.enforced_storage_budget
            || (limits.gpu_seconds > 0 && !self.enforced_gpu_budget)
        {
            return Err(ContractError::new(
                "job.unsupported",
                "backend lacks required qualified ownership, containment or resource enforcement",
            ));
        }
        Ok(())
    }
}

/// No implementation is automatically qualified by implementing this trait.
/// Collection returns content-addressed objects through the shared artifact store;
/// bounded observations provide references rather than unbounded log payloads.
#[async_trait]
pub trait CommandJobBackend: Send {
    fn qualification(&self) -> &CommandJobQualification;
    async fn submit(
        &mut self,
        request: &CommandJobRequest,
    ) -> Result<JobLookupResponse, RuntimeError>;
    async fn lookup(
        &mut self,
        request: &CommandJobRequest,
    ) -> Result<JobLookupResponse, RuntimeError>;
    async fn observe(
        &mut self,
        binding: &JobBinding,
        cursor: u64,
    ) -> Result<JobObservation, RuntimeError>;
    /// Acknowledgement is an observation; cancellation is complete only when
    /// its terminal receipt confirms process-tree quiescence.
    async fn cancel(
        &mut self,
        binding: &JobBinding,
        decision_key: &str,
        ownership: &JobOwnership,
    ) -> Result<JobObservation, RuntimeError>;
    async fn collect(
        &mut self,
        binding: &JobBinding,
        output_manifest: &ArtifactRef,
        cursor: u64,
    ) -> Result<JobCollection, RuntimeError>;
}

fn artifacts(values: &[ArtifactRef]) -> Result<(), ContractError> {
    if values.len() > 64 {
        return Err(ContractError::new(
            "job.evidence",
            "at most 64 artifact references",
        ));
    }
    for value in values {
        value.validate()?;
    }
    Ok(())
}
fn bounded(value: &str) -> Result<(), ContractError> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(ContractError::new(
            "job.identity",
            "requires 1..=256 non-control bytes",
        ));
    }
    Ok(())
}
fn digest(value: &str) -> Result<(), ContractError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(ContractError::new(
            "job.digest",
            "requires lowercase SHA-256",
        ));
    }
    Ok(())
}
