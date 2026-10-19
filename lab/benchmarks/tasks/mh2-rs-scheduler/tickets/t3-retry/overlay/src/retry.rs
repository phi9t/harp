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
            factor: factor.max(1),
            max_delay_ms,
        }
    }

    pub fn for_job(job: &JobSpec) -> Self {
        Self::new(job.max_attempts, job.retry_base_ms, 2, 60_000)
    }

    pub fn schedule(&self, job: &str, outcomes: &[bool]) -> SchedulerResult<Vec<AttemptEvent>> {
        if self.max_attempts == 0 {
            return Err(SchedulerError::RetryBudgetExceeded {
                job: job.to_string(),
                attempts: 0,
            });
        }

        let mut events = Vec::new();
        let mut elapsed_ms = 0;
        let mut next_delay = self.base_ms;

        for index in 0..self.max_attempts {
            let success = outcomes.get(index as usize).copied().unwrap_or(false);
            events.push(AttemptEvent {
                id: job.to_string(),
                attempt: index + 1,
                elapsed_ms,
                success,
            });
            if success {
                return Ok(events);
            }
            if index + 1 < self.max_attempts {
                let delay = next_delay.min(self.max_delay_ms);
                elapsed_ms = elapsed_ms.saturating_add(delay);
                next_delay = next_delay
                    .saturating_mul(self.factor)
                    .min(self.max_delay_ms);
            }
        }

        Err(SchedulerError::RetryBudgetExceeded {
            job: job.to_string(),
            attempts: self.max_attempts,
        })
    }
}
