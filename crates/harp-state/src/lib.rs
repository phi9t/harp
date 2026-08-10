mod model;
#[cfg(unix)]
mod sqlite;
mod store;
mod transitions;

use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub use model::{
    AcceptResult, AcceptedResultRecord, ActivityPreparation, ActivityRecoveryRecord, AttemptRecord,
    AttemptState, CliActivityKind, CliActivityRecord, CliActivityState, CliAttemptRecord,
    CliAttemptState, CliSignalStage, CliTerminalFailureClass, EventPage, EventRecord,
    InterruptBudgetDimension, InterruptIntent, InterruptPurpose, LeaseToken, OperationKind,
    OperationRecord, OperationState, RunBudget, RunExecutionRecord, RunProvenance, RunRecord,
    RunState, TaskClaim, TaskRecord, TaskState, TurnOperationPreparation, UsageOutcome,
    WallUsageOutcome,
};
pub use store::StateStore;

pub type StateResult<T> = Result<T, StateError>;

pub trait LeaseClock: Send + Sync + std::fmt::Debug {
    fn unix_seconds(&self) -> StateResult<i64>;
}

#[derive(Debug)]
pub(crate) struct SystemLeaseClock;

impl LeaseClock for SystemLeaseClock {
    fn unix_seconds(&self) -> StateResult<i64> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|source| StateError::Clock {
                context: "system time is before the Unix epoch".to_owned(),
                source: Some(source),
            })?
            .as_secs();
        i64::try_from(seconds).map_err(|_| StateError::Clock {
            context: "system time exceeds SQLite integer range".to_owned(),
            source: None,
        })
    }
}

pub(crate) fn system_lease_clock() -> Arc<dyn LeaseClock> {
    Arc::new(SystemLeaseClock)
}

pub(crate) fn default_lease_clock() -> Arc<dyn LeaseClock> {
    system_lease_clock()
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("invalid state input: {context}")]
    InvalidInput {
        context: String,
        #[source]
        source: Option<harp_contracts::ContractError>,
    },

    #[error("state conflict for {entity}: expected {expected}, actual {actual}")]
    Conflict {
        entity: String,
        expected: String,
        actual: String,
    },

    #[error("state integrity failure: {context}")]
    Integrity {
        context: String,
        #[source]
        source: Option<harp_contracts::ContractError>,
    },

    #[error("budget exceeded for {dimension}: limit {limit}, attempted {attempted}")]
    BudgetExceeded {
        dimension: String,
        limit: u64,
        attempted: u64,
    },

    #[error("SQLite {context} failed")]
    Sqlite {
        context: String,
        #[source]
        source: rusqlite::Error,
    },

    #[error("{operation} failed for {}", path.display())]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("JSON serialization failed for {context}")]
    Serialization {
        context: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("unsupported state operation: {context}")]
    Unsupported { context: String },

    #[error("{context} exceeds limit {limit}: {actual}")]
    LimitExceeded {
        context: String,
        limit: u64,
        actual: u64,
    },

    #[error("lease clock failure: {context}")]
    Clock {
        context: String,
        #[source]
        source: Option<std::time::SystemTimeError>,
    },
}

impl StateError {
    pub(crate) fn invalid(context: impl Into<String>) -> Self {
        Self::InvalidInput {
            context: context.into(),
            source: None,
        }
    }

    pub(crate) fn invalid_contract(source: harp_contracts::ContractError) -> Self {
        Self::InvalidInput {
            context: "contract validation failed".to_owned(),
            source: Some(source),
        }
    }

    pub(crate) fn integrity(context: impl Into<String>) -> Self {
        Self::Integrity {
            context: context.into(),
            source: None,
        }
    }

    pub(crate) fn integrity_contract(
        context: impl Into<String>,
        source: harp_contracts::ContractError,
    ) -> Self {
        Self::Integrity {
            context: context.into(),
            source: Some(source),
        }
    }

    pub(crate) fn sqlite(context: impl Into<String>, source: rusqlite::Error) -> Self {
        Self::Sqlite {
            context: context.into(),
            source,
        }
    }

    pub(crate) fn io(operation: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }
}
