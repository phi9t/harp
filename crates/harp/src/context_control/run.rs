//! Compatibility types for the retired direct provider wrapper.
use serde::Serialize;

use super::provider::ProviderRunOptions;
use super::{ProviderId, WorkflowChoice};
use crate::AppError;

pub const EXECUTION_REPLACEMENT: &str = "Use harp durable prepare, admit and start with committed task Markdown and acceptance JSON; status, cancel and resolve own recovery. See docs/durable-runtime.md for verification, approval and local landing. Old execution state and graph inputs are not imported.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRequest {
    pub provider: ProviderId,
    pub workflow: WorkflowChoice,
    pub task: String,
    pub options: ProviderRunOptions,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct RunResult {
    pub episode_id: String,
    pub release_id: String,
    pub context_bundle_sha256: String,
    pub provider_exit_code: Option<i32>,
    pub terminated_by_signal: Option<i32>,
}

pub trait RunReporter {
    fn lifecycle(&mut self, message: &str);
    fn warning(&mut self, message: &str);
}

/// Refuse before repository/state inspection, provider probes or lifecycle work.
pub fn run(_request: RunRequest, _reporter: &mut dyn RunReporter) -> Result<RunResult, AppError> {
    Err(AppError::invalid_input(
        "run.retired",
        format!("harp run is retired. {EXECUTION_REPLACEMENT}"),
    ))
}
