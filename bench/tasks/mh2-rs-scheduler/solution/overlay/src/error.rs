use std::error::Error;
use std::fmt;

pub type SchedulerResult<T> = Result<T, SchedulerError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchedulerError {
    DuplicateJob { id: String },
    UnknownDependency { job: String, dependency: String },
    UnknownJob { id: String },
    Cycle { members: Vec<String> },
    InvalidWindow { job: String, start: u32, end: u32 },
    NoWindow { job: String },
    RetryBudgetExceeded { job: String, attempts: u32 },
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulerError::DuplicateJob { id } => {
                write!(f, "scheduler:duplicate-job: duplicate job id {id}")
            }
            SchedulerError::UnknownDependency { job, dependency } => write!(
                f,
                "scheduler:unknown-dependency: job {job} depends on missing {dependency}"
            ),
            SchedulerError::UnknownJob { id } => {
                write!(f, "scheduler:unknown-job: job {id} was not provided")
            }
            SchedulerError::Cycle { members } => write!(
                f,
                "scheduler:cycle: dependency cycle among {}",
                members.join(", ")
            ),
            SchedulerError::InvalidWindow { job, start, end } => write!(
                f,
                "scheduler:invalid-window: job {job} has invalid window {start}..{end}"
            ),
            SchedulerError::NoWindow { job } => {
                write!(
                    f,
                    "scheduler:no-window: job {job} does not fit any open window"
                )
            }
            SchedulerError::RetryBudgetExceeded { job, attempts } => write!(
                f,
                "scheduler:retry-budget-exceeded: job {job} failed after {attempts} attempts"
            ),
        }
    }
}

impl Error for SchedulerError {}
