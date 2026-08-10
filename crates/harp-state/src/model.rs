use std::str::FromStr;
use std::sync::Arc;

use harp_contracts::{
    AttemptId, Budget, ExternalSessionId, NodeKind, OperationId, ResultStatus, RunId, TaskGraph,
    TaskId, ThreadId, TurnId, TurnSpec,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zeroize::Zeroizing;

use crate::{StateError, StateResult};

macro_rules! sqlite_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }

            pub fn parse(value: &str) -> StateResult<Self> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(StateError::integrity(format!(
                        "invalid {} value {value:?}",
                        stringify!($name)
                    ))),
                }
            }
        }
    };
}

sqlite_enum!(RunState {
    Active => "active",
    Completed => "completed",
    Failed => "failed",
    Cancelled => "cancelled",
});

sqlite_enum!(TaskState {
    Pending => "pending",
    Ready => "ready",
    Running => "running",
    ResultPublished => "result_published",
    Completed => "completed",
    Failed => "failed",
    Cancelled => "cancelled",
});

sqlite_enum!(AttemptState {
    Prepared => "prepared",
    DispatchingThread => "dispatching_thread",
    ThreadStarted => "thread_started",
    DispatchingTurn => "dispatching_turn",
    TurnStarted => "turn_started",
    Reconciling => "reconciling",
    ResultPublished => "result_published",
    Succeeded => "succeeded",
    Failed => "failed",
    Indeterminate => "indeterminate",
    Cancelled => "cancelled",
});

impl AttemptState {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Indeterminate | Self::Cancelled
        )
    }
}

sqlite_enum!(OperationKind {
    StartThread => "start_thread",
    StartTurn => "start_turn",
    ContinueTurn => "continue_turn",
    InterruptTurn => "interrupt_turn",
    PublishResult => "publish_result",
    Evaluate => "evaluate",
});

sqlite_enum!(OperationState {
    Prepared => "prepared",
    Dispatching => "dispatching",
    Completed => "completed",
    Failed => "failed",
    Indeterminate => "indeterminate",
    Cancelled => "cancelled",
});

sqlite_enum!(CliAttemptState {
    Prepared => "prepared",
    Running => "running",
    Reconciling => "reconciling",
    Succeeded => "succeeded",
    Failed => "failed",
    Indeterminate => "indeterminate",
    Cancelled => "cancelled",
});

impl CliAttemptState {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Indeterminate | Self::Cancelled
        )
    }
}

sqlite_enum!(CliActivityKind {
    StartActivity => "start_activity",
    ContinueActivity => "continue_activity",
    InterruptActivity => "interrupt_activity",
});

sqlite_enum!(CliActivityState {
    Prepared => "prepared",
    Dispatching => "dispatching",
    Running => "running",
    Reconciling => "reconciling",
    Completed => "completed",
    Failed => "failed",
    Indeterminate => "indeterminate",
    Cancelled => "cancelled",
});

impl CliActivityState {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Indeterminate | Self::Cancelled
        )
    }
}

sqlite_enum!(CliTerminalFailureClass {
    ResumableInterrupted => "resumable_interrupted",
    ResumableCliFailure => "resumable_cli_failure",
    NonResumableProtocolFailure => "non_resumable_protocol_failure",
    ContinuationExhausted => "continuation_exhausted",
    Cancelled => "cancelled",
    Indeterminate => "indeterminate",
});

impl CliTerminalFailureClass {
    pub const fn is_resumable(self) -> bool {
        matches!(self, Self::ResumableInterrupted | Self::ResumableCliFailure)
    }
}

sqlite_enum!(CliSignalStage {
    Prepared => "prepared",
    SigintPrepared => "sigint_prepared",
    SigintSent => "sigint_sent",
    SigtermPrepared => "sigterm_prepared",
    SigtermSent => "sigterm_sent",
    SigkillPrepared => "sigkill_prepared",
    SigkillSent => "sigkill_sent",
    Quiescent => "quiescent",
    Indeterminate => "indeterminate",
});

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRecord {
    pub run_id: RunId,
    pub graph_sha256: String,
    pub state: RunState,
    pub cancellation_requested: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRecord {
    pub run_id: RunId,
    pub task_id: TaskId,
    pub kind: NodeKind,
    pub state: TaskState,
    pub accepted_attempt_id: Option<AttemptId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunExecutionRecord {
    pub run: RunRecord,
    pub graph: TaskGraph,
    pub verified_graph_sha256: String,
    pub provenance: Value,
    pub budget: RunBudget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedResultRecord {
    pub task: TaskRecord,
    pub attempt: AttemptRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptRecord {
    pub attempt_id: AttemptId,
    pub run_id: RunId,
    pub task_id: TaskId,
    pub ordinal: u32,
    pub state: AttemptState,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<i64>,
    pub last_lease_expires_at: Option<i64>,
    pub thread_id: Option<ThreadId>,
    pub latest_turn_id: Option<TurnId>,
    pub latest_operation_marker: Option<String>,
    pub scratch_path: Option<String>,
    pub scratch_device: Option<u64>,
    pub scratch_inode: Option<u64>,
    pub wall_started_at: Option<i64>,
    pub wall_last_observed_at: Option<i64>,
    pub observed_wall_seconds: u64,
    pub semantic_failure_class: Option<String>,
    pub continuation_count: u64,
    pub observed_tokens: u64,
    pub latest_turn_observed_tokens: u64,
    pub result_sha256: Option<String>,
    pub result_status: Option<ResultStatus>,
    pub result_token_usage: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OperationRecord {
    pub operation_id: OperationId,
    pub attempt_id: AttemptId,
    pub kind: OperationKind,
    pub ordinal: u32,
    pub state: OperationState,
    pub external_id: Option<String>,
    pub target_external_id: Option<String>,
    pub operation_marker: Option<String>,
    pub turn_intent: Option<TurnSpec>,
    pub interrupt_intent: Option<InterruptIntent>,
    pub intent_sha256: Option<String>,
    pub checkpoint_sha256: Option<String>,
    pub consumed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliAttemptRecord {
    pub attempt_id: AttemptId,
    pub state: CliAttemptState,
    pub logical_session_id: ThreadId,
    pub external_session_id: Option<ExternalSessionId>,
    pub continuation_count: u32,
    pub terminal_failure_class: Option<CliTerminalFailureClass>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliActivityRecord {
    pub activity_id: OperationId,
    pub attempt_id: AttemptId,
    pub kind: CliActivityKind,
    pub ordinal: u32,
    pub state: CliActivityState,
    pub logical_turn_id: TurnId,
    pub activity_dir: String,
    pub invocation_sha256: String,
    pub process_record_sha256: Option<String>,
    pub terminal_failure_class: Option<CliTerminalFailureClass>,
    pub interrupt_purpose: Option<InterruptPurpose>,
    pub target_process_record_sha256: Option<String>,
    pub signal_stage: Option<CliSignalStage>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityRecoveryRecord {
    pub run_id: RunId,
    pub task_id: TaskId,
    pub activity_id: OperationId,
    pub attempt_id: AttemptId,
    pub kind: CliActivityKind,
    pub ordinal: u32,
    pub activity_state: CliActivityState,
    pub attempt_state: CliAttemptState,
    pub logical_session_id: ThreadId,
    pub logical_turn_id: TurnId,
    pub external_session_id: Option<ExternalSessionId>,
    pub continuation_count: u32,
    pub activity_dir: String,
    pub invocation_sha256: String,
    pub process_record_sha256: Option<String>,
    pub terminal_failure_class: Option<CliTerminalFailureClass>,
    pub interrupt_purpose: Option<InterruptPurpose>,
    pub target_process_record_sha256: Option<String>,
    pub signal_stage: Option<CliSignalStage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityPreparation {
    pub activity_dir: String,
    pub invocation_sha256: String,
}

impl ActivityPreparation {
    pub fn new(
        activity_dir: impl Into<String>,
        invocation_sha256: impl Into<String>,
    ) -> StateResult<Self> {
        let preparation = Self {
            activity_dir: activity_dir.into(),
            invocation_sha256: invocation_sha256.into(),
        };
        preparation.validate()?;
        Ok(preparation)
    }

    pub(crate) fn validate(&self) -> StateResult<()> {
        let directory_bytes = self.activity_dir.as_bytes();
        if directory_bytes.is_empty()
            || directory_bytes.len() > 4096
            || !self.activity_dir.starts_with('/')
            || self.activity_dir.contains('\0')
        {
            return Err(StateError::invalid(
                "activity directory must be a bounded absolute path",
            ));
        }
        if !valid_sha256(&self.invocation_sha256) {
            return Err(StateError::invalid(
                "activity invocation digest must be lowercase SHA-256",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptPurpose {
    Budget,
    ScannerIntegrity,
    Cancellation,
}

impl InterruptPurpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Budget => "budget",
            Self::ScannerIntegrity => "scanner_integrity",
            Self::Cancellation => "cancellation",
        }
    }

    pub fn parse(value: &str) -> StateResult<Self> {
        match value {
            "budget" => Ok(Self::Budget),
            "scanner_integrity" => Ok(Self::ScannerIntegrity),
            "cancellation" => Ok(Self::Cancellation),
            _ => Err(StateError::integrity(format!(
                "invalid InterruptPurpose value {value:?}"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptBudgetDimension {
    Tokens,
    Storage,
    Wall,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InterruptIntent {
    pub schema_version: u8,
    pub purpose: InterruptPurpose,
    pub reason_code: String,
    pub budget_dimension: Option<InterruptBudgetDimension>,
}

impl InterruptIntent {
    pub fn budget(dimension: InterruptBudgetDimension, reason_code: &str) -> StateResult<Self> {
        Self::new(InterruptPurpose::Budget, Some(dimension), reason_code)
    }

    pub fn scanner_integrity(reason_code: &str) -> StateResult<Self> {
        Self::new(InterruptPurpose::ScannerIntegrity, None, reason_code)
    }

    pub fn cancellation(reason_code: &str) -> StateResult<Self> {
        Self::new(InterruptPurpose::Cancellation, None, reason_code)
    }

    fn new(
        purpose: InterruptPurpose,
        budget_dimension: Option<InterruptBudgetDimension>,
        reason_code: &str,
    ) -> StateResult<Self> {
        let intent = Self {
            schema_version: 1,
            purpose,
            reason_code: reason_code.to_owned(),
            budget_dimension,
        };
        intent.validate()?;
        Ok(intent)
    }

    pub(crate) fn validate(&self) -> StateResult<()> {
        if self.schema_version != 1 {
            return Err(StateError::invalid(
                "interrupt intent schema_version must equal 1",
            ));
        }
        if self.reason_code.is_empty()
            || self.reason_code.len() > 128
            || !self
                .reason_code
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(StateError::invalid(
                "interrupt reason_code must contain 1..=128 lowercase portable bytes",
            ));
        }
        if (self.purpose == InterruptPurpose::Budget) != self.budget_dimension.is_some() {
            return Err(StateError::invalid(
                "only budget interrupt intents may carry a budget dimension",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TurnOperationPreparation {
    pub operation_id: OperationId,
    pub kind: OperationKind,
    pub ordinal: u32,
    pub intent: TurnSpec,
    pub checkpoint_sha256: Option<String>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct LeaseToken {
    attempt_id: AttemptId,
    owner: String,
    expires_at: i64,
    pub(crate) capability: Arc<LeaseCapability>,
}

#[derive(Eq, PartialEq)]
pub(crate) struct LeaseCapability(pub(crate) Zeroizing<String>);

impl std::fmt::Debug for LeaseToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LeaseToken")
            .field("attempt_id", &self.attempt_id)
            .field("owner", &self.owner)
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

impl LeaseToken {
    pub fn attempt_id(&self) -> &AttemptId {
        &self.attempt_id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub const fn expires_at(&self) -> i64 {
        self.expires_at
    }

    pub(crate) fn capability(&self) -> &str {
        self.capability.0.as_str()
    }

    pub(crate) fn new(
        attempt_id: AttemptId,
        owner: String,
        expires_at: i64,
        capability: String,
    ) -> Self {
        Self {
            attempt_id,
            owner,
            expires_at,
            capability: Arc::new(LeaseCapability(Zeroizing::new(capability))),
        }
    }

    fn from_shared(
        attempt_id: AttemptId,
        owner: String,
        expires_at: i64,
        capability: Arc<LeaseCapability>,
    ) -> Self {
        Self {
            attempt_id,
            owner,
            expires_at,
            capability,
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct TaskClaim {
    pub run_id: RunId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub lease_owner: String,
    pub lease_expires_at: i64,
    pub budget: Budget,
    pub(crate) lease_capability: Arc<LeaseCapability>,
}

impl std::fmt::Debug for TaskClaim {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskClaim")
            .field("run_id", &self.run_id)
            .field("task_id", &self.task_id)
            .field("attempt_id", &self.attempt_id)
            .field("lease_owner", &self.lease_owner)
            .field("lease_expires_at", &self.lease_expires_at)
            .field("budget", &self.budget)
            .finish_non_exhaustive()
    }
}

impl TaskClaim {
    pub fn lease(&self) -> LeaseToken {
        LeaseToken::from_shared(
            self.attempt_id.clone(),
            self.lease_owner.clone(),
            self.lease_expires_at,
            Arc::clone(&self.lease_capability),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcceptResult {
    Accepted,
    AlreadyAccepted { attempt_id: AttemptId },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunBudget {
    pub max_tokens: u64,
    pub max_storage_bytes: u64,
    pub max_wall_seconds: u64,
}

impl RunBudget {
    pub fn new(
        max_tokens: u64,
        max_storage_bytes: u64,
        max_wall_seconds: u64,
    ) -> StateResult<Self> {
        let budget = Self {
            max_tokens,
            max_storage_bytes,
            max_wall_seconds,
        };
        budget.validate()?;
        Ok(budget)
    }

    pub(crate) fn validate(&self) -> StateResult<()> {
        for (name, value) in [
            ("max_tokens", self.max_tokens),
            ("max_storage_bytes", self.max_storage_bytes),
            ("max_wall_seconds", self.max_wall_seconds),
        ] {
            if value == 0 {
                return Err(StateError::invalid(format!("{name} must be nonzero")));
            }
            checked_u64_to_i64(name, value)?;
        }
        Ok(())
    }
}

pub type RunProvenance = Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UsageOutcome {
    WithinBudget,
    Exceeded {
        max_tokens: u64,
        observed_tokens: u128,
        max_storage_bytes: u64,
        observed_storage_bytes: u128,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WallUsageOutcome {
    pub observed_attempt_wall_seconds: u64,
    pub observed_run_wall_seconds: u128,
    pub attempt_limit_exceeded: bool,
    pub run_limit_exceeded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRecord {
    pub sequence: u64,
    pub run_id: Option<RunId>,
    pub task_id: Option<TaskId>,
    pub attempt_id: Option<AttemptId>,
    pub event_type: String,
    pub payload: Value,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventPage {
    pub events: Vec<EventRecord>,
    pub next_after_sequence: Option<i64>,
}

pub(crate) fn checked_u64_to_i64(context: &str, value: u64) -> StateResult<i64> {
    i64::try_from(value).map_err(|_| StateError::LimitExceeded {
        context: context.to_owned(),
        limit: i64::MAX as u64,
        actual: value,
    })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn checked_i64_to_u64(context: &str, value: i64) -> StateResult<u64> {
    u64::try_from(value)
        .map_err(|_| StateError::integrity(format!("{context} contains negative value {value}")))
}

pub(crate) fn checked_i64_to_u32(context: &str, value: i64) -> StateResult<u32> {
    u32::try_from(value)
        .map_err(|_| StateError::integrity(format!("{context} value {value} does not fit in u32")))
}

pub(crate) fn parse_id<T>(entity: &str, value: String) -> StateResult<T>
where
    T: FromStr<Err = harp_contracts::ContractError>,
{
    value
        .parse()
        .map_err(|source| StateError::integrity_contract(entity, source))
}

pub(crate) fn node_kind_as_str(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Analysis => "analysis",
        NodeKind::Reducer => "reducer",
    }
}

pub(crate) fn parse_node_kind(value: &str) -> StateResult<NodeKind> {
    match value {
        "analysis" => Ok(NodeKind::Analysis),
        "reducer" => Ok(NodeKind::Reducer),
        _ => Err(StateError::integrity(format!(
            "invalid NodeKind value {value:?}"
        ))),
    }
}

pub(crate) fn result_status_as_str(status: ResultStatus) -> &'static str {
    match status {
        ResultStatus::Success => "success",
        ResultStatus::Partial => "partial",
        ResultStatus::Failed => "failed",
        ResultStatus::Cancelled => "cancelled",
    }
}

pub(crate) fn parse_result_status(value: &str) -> StateResult<ResultStatus> {
    match value {
        "success" => Ok(ResultStatus::Success),
        "partial" => Ok(ResultStatus::Partial),
        "failed" => Ok(ResultStatus::Failed),
        "cancelled" => Ok(ResultStatus::Cancelled),
        _ => Err(StateError::integrity(format!(
            "invalid ResultStatus value {value:?}"
        ))),
    }
}
