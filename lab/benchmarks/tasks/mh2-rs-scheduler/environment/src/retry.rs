use crate::error::{SchedulerError, SchedulerResult};
use crate::job::{AttemptEvent, JobSpec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackoffPolicy {
    pub max_attempts: u32,
    pub base_ms: u64,
    pub factor: u64,
    pub max_delay_ms: u64,
}

impl BackoffPolicy {
    pub fn new(max_attempts: u32, base_ms: u64, factor: u64, max_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            base_ms,
            factor,
            max_delay_ms,
        }
    }

    pub fn for_job(job: &JobSpec) -> Self {
        Self::new(job.max_attempts, job.retry_base_ms, 2, 60_000)
    }

    pub fn schedule(&self, job: &str, outcomes: &[bool]) -> SchedulerResult<Vec<AttemptEvent>> {
        let success = outcomes.first().copied().unwrap_or(true);
        if success {
            Ok(vec![AttemptEvent {
                id: job.to_string(),
                attempt: 1,
                elapsed_ms: 0,
                success: true,
            }])
        } else {
            Err(SchedulerError::RetryBudgetExceeded {
                job: job.to_string(),
                attempts: 1,
            })
        }
    }
}
