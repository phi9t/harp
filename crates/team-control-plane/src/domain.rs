use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Unauthorized,
    StaleRevision,
    StaleGeneration,
    IdempotencyConflict,
    BudgetExhausted,
    DependencyUnmet,
    CapacityExhausted,
    InvalidArgument,
    NotFound,
    IncompatibleSchema,
    TemporarilyUnavailable,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Error {
    pub code: ErrorCode,
}
pub type Result<T> = std::result::Result<T, Error>;
impl From<tokio_postgres::Error> for Error {
    fn from(_: tokio_postgres::Error) -> Self {
        Self {
            code: ErrorCode::TemporarilyUnavailable,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub capacity: i64,
    pub max_attempts: i64,
    pub max_episodes: i64,
    pub max_repairs: i64,
    pub episode_deadline_seconds: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitInput {
    pub repository: String,
    pub commit: String,
    pub path: String,
    pub digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskSpec {
    #[serde(default)]
    pub dependencies: Vec<Uuid>,
    pub objective: String,
    pub input: GitInput,
    pub acceptance: Vec<String>,
    pub deliverables: Vec<String>,
    pub write_scope: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Owner,
    Lead,
    Worker,
    Verifier,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    CreateMission {
        id: Uuid,
        objective: String,
        input: GitInput,
        policy: Policy,
    },
    CreateTask {
        id: Uuid,
        mission_id: Uuid,
        spec: TaskSpec,
    },
    SetDependencies {
        task_id: Uuid,
        dependencies: Vec<Uuid>,
    },
    AdmitTask {
        task_id: Uuid,
        spec_digest: String,
    },
    GrantCredential {
        credential_id: Uuid,
        principal_id: Uuid,
        role: Role,
        mission_id: Option<Uuid>,
        task_id: Option<Uuid>,
        generation: Option<i64>,
        token_hash: String,
        expires_in_seconds: i64,
    },
    AssignAttempt {
        task_id: Uuid,
        worker_id: Uuid,
    },
    CancelAttempt {
        task_id: Uuid,
        attempt_id: Uuid,
        reason: String,
    },
    ReconcileAttempt {
        task_id: Uuid,
        attempt_id: Uuid,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope {
    pub schema_version: u32,
    pub key: String,
    pub expected_revision: i64,
    pub expected_generation: Option<i64>,
    pub recovery_epoch: Uuid,
    pub command: Command,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandResult {
    pub schema_version: u32,
    pub disposition: String,
    pub revision: i64,
    pub generation: i64,
    pub recovery_epoch: Uuid,
    pub attempt_id: Option<Uuid>,
    pub reservation_id: Option<Uuid>,
    pub operation_id: Option<Uuid>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotQuery {
    Mission(Uuid),
    Task(Uuid),
    Command(Uuid, String, String),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema_version: u32,
    pub revision: i64,
    pub snapshot_time: String,
    pub data: serde_json::Value,
}
pub(crate) fn err(code: ErrorCode) -> Error {
    Error { code }
}
