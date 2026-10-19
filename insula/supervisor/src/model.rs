use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub type Generation = Uuid;
pub type OperationId = Uuid;
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
pub fn digest<T: Serialize>(value: &T) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)))
}
pub fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}
pub fn clean_absolute(path: &std::path::Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        && !path.as_os_str().as_encoded_bytes().contains(&0)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct ServiceKey {
    pub insula_id: String,
    pub service_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeIdentity {
    pub uid: u32,
    pub gid: u32,
}

/// Environment entries are references resolved only inside the execution owner.
/// Secret bytes never enter supervisor requests, records or events.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CommandSpec {
    pub executable: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: PathBuf,
    #[serde(default)]
    pub env_refs: BTreeMap<String, String>,
    pub identity: RuntimeIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProbeKind {
    Process,
    Command {
        command: CommandSpec,
    },
    UnixSocket {
        path: PathBuf,
    },
    Http {
        address: String,
        port: u16,
        path: String,
        expected_status: u16,
    },
    Tmux {
        executable: PathBuf,
        socket: PathBuf,
        #[serde(default)]
        expected_sessions: Vec<String>,
        #[serde(default)]
        expected_panes: Vec<String>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProbeSpec {
    pub probe: ProbeKind,
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub failure_threshold: u32,
    pub max_output_bytes: usize,
}
impl Default for ProbeSpec {
    fn default() -> Self {
        Self {
            probe: ProbeKind::Process,
            interval_ms: 1000,
            timeout_ms: 500,
            failure_threshold: 3,
            max_output_bytes: 8192,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RestartPolicy {
    #[default]
    Never,
    /// Budget is lifetime automatic replacements, durable across controller restarts.
    Bounded {
        max_restarts: u32,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
        on_unhealthy: bool,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GracefulStop {
    Terminate,
    Command {
        command: CommandSpec,
    },
    Tmux {
        executable: PathBuf,
        socket: PathBuf,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StopPolicy {
    pub graceful: GracefulStop,
    pub deadline_ms: u64,
    pub force: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LogPolicy {
    pub max_bytes: u64,
    pub retained_files: u32,
}
impl Default for LogPolicy {
    fn default() -> Self {
        Self {
            max_bytes: 1024 * 1024,
            retained_files: 2,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CleanupBoundary {
    #[default]
    Service,
    Insula,
    DirectProcess,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControllerExit {
    #[default]
    KeepServices,
    StopServices,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ServiceAdapter {
    Foreground,
    /// Launch an empty foreground tmux server. No config, hooks or sessions replay.
    Tmux {
        socket: PathBuf,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ServiceSpec {
    pub key: ServiceKey,
    pub revision: String,
    pub command: CommandSpec,
    pub adapter: ServiceAdapter,
    #[serde(default)]
    pub readiness: ProbeSpec,
    #[serde(default)]
    pub health: ProbeSpec,
    #[serde(default)]
    pub startup_grace_ms: u64,
    #[serde(default)]
    pub restart: RestartPolicy,
    /// Required acknowledgement before enabling automatic service bootstrap replay.
    #[serde(default)]
    pub replay_safe: bool,
    pub stop: StopPolicy,
    #[serde(default)]
    pub logs: LogPolicy,
    #[serde(default)]
    pub cleanup_boundary: CleanupBoundary,
    #[serde(default)]
    pub controller_exit: ControllerExit,
}
impl ServiceSpec {
    pub fn validate(&self) -> Result<()> {
        if !valid_id(&self.key.insula_id)
            || !valid_id(&self.key.service_id)
            || self.revision.is_empty()
            || self.revision.len() > 256
        {
            return Err(Error::Invalid("invalid identity or revision".into()));
        }
        validate_command(&self.command)?;
        for p in [&self.readiness, &self.health] {
            if p.interval_ms == 0
                || p.timeout_ms == 0
                || p.timeout_ms > 60_000
                || p.failure_threshold == 0
                || p.max_output_bytes == 0
                || p.max_output_bytes > 65_536
            {
                return Err(Error::Invalid("invalid bounded probe settings".into()));
            }
            match &p.probe {
                ProbeKind::Command { command } => validate_command(command)?,
                ProbeKind::UnixSocket { path } if !clean_absolute(path) => {
                    return Err(Error::Invalid(
                        "probe path must be absolute and contain no parent traversal".into(),
                    ))
                }
                ProbeKind::Http {
                    address,
                    port,
                    path,
                    expected_status,
                } if address.parse::<std::net::IpAddr>().is_err()
                    || *port == 0
                    || !path.starts_with('/')
                    || path.contains(['\r', '\n', ' '])
                    || !(100..600).contains(expected_status) =>
                {
                    return Err(Error::Invalid(
                        "HTTP probe needs a numeric IP, port and safe path".into(),
                    ))
                }
                ProbeKind::Tmux {
                    executable,
                    socket,
                    expected_sessions,
                    expected_panes,
                } if !clean_absolute(executable)
                    || !clean_absolute(socket)
                    || expected_sessions.len() > 128
                    || expected_panes.len() > 1024
                    || expected_sessions
                        .iter()
                        .chain(expected_panes.iter())
                        .any(|x| x.len() > 128 || x.contains(['\r', '\n', '\t'])) =>
                {
                    return Err(Error::Invalid("invalid tmux probe".into()))
                }
                _ => {}
            }
        }
        if let RestartPolicy::Bounded {
            max_restarts,
            initial_backoff_ms,
            max_backoff_ms,
            ..
        } = self.restart
        {
            if !self.replay_safe
                || max_restarts == 0
                || initial_backoff_ms == 0
                || max_backoff_ms < initial_backoff_ms
            {
                return Err(Error::Invalid(
                    "automatic recovery requires replay_safe and a nonzero bounded policy".into(),
                ));
            }
        }
        if self.stop.deadline_ms == 0
            || self.stop.deadline_ms > 60_000
            || self.logs.max_bytes > 64 * 1024 * 1024
            || self.logs.retained_files > 16
        {
            return Err(Error::Invalid("invalid stop or logging bounds".into()));
        }
        if self.cleanup_boundary == CleanupBoundary::Insula {
            return Err(Error::Unsupported(
                "service operations cannot destroy an Insula".into(),
            ));
        }
        match &self.stop.graceful {
            GracefulStop::Command { command } => validate_command(command)?,
            GracefulStop::Tmux { executable, socket }
                if !clean_absolute(executable) || !clean_absolute(socket) =>
            {
                return Err(Error::Invalid("invalid tmux stop".into()))
            }
            _ => {}
        }
        if let ServiceAdapter::Tmux { socket } = &self.adapter {
            if !clean_absolute(socket)
                || socket.as_os_str().len() > 90
                || !self.command.args.is_empty()
            {
                return Err(Error::Invalid("tmux adapter requires a private absolute socket without parent traversal and empty command args".into()));
            }
            for p in [&self.readiness, &self.health] {
                if let ProbeKind::Tmux {
                    socket: observed,
                    executable,
                    ..
                } = &p.probe
                {
                    if observed != socket || executable != &self.command.executable {
                        return Err(Error::Invalid(
                            "tmux probe must target the declared server endpoint and executable"
                                .into(),
                        ));
                    }
                }
            }
            if let GracefulStop::Tmux {
                socket: stopped,
                executable,
            } = &self.stop.graceful
            {
                if stopped != socket || executable != &self.command.executable {
                    return Err(Error::Invalid(
                        "tmux stop must target the declared server endpoint and executable".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}
pub fn validate_command(c: &CommandSpec) -> Result<()> {
    if !clean_absolute(&c.executable)
        || !clean_absolute(&c.cwd)
        || c.args.len() > 256
        || c.args.iter().any(|a| a.len() > 8192 || a.contains('\0'))
        || c.env_refs.len() > 256
        || c.env_refs
            .iter()
            .any(|(k, v)| !valid_id(k) || v.is_empty() || v.len() > 512 || v.contains('\0'))
    {
        return Err(Error::Invalid("command must use absolute paths, explicit bounded arguments and environment references".into()));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DesiredState {
    Running,
    Stopped,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Absent,
    Starting,
    Running,
    Stopping,
    Exited,
    Unknown,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeClass {
    Success,
    ServiceFailure,
    MechanismFailure,
    TargetUnavailable,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeRole {
    Readiness,
    Health,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaneStatus {
    pub pane_id: String,
    pub dead: bool,
    pub exit_code: Option<i32>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeResult {
    pub generation: Generation,
    pub class: ProbeClass,
    /// Sanitized machine code only. Raw command/HTTP output must not be returned.
    pub code: String,
    pub panes: Vec<PaneStatus>,
    /// Tmux topology/pane expectations affect readiness, never server health.
    #[serde(default)]
    pub expectations_met: Option<bool>,
    pub observed_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExitInfo {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub observed_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessIdentity {
    pub owner_instance: Uuid,
    pub boot_id: String,
    pub pid: u32,
    pub start_time_ticks: u64,
    pub generation: Generation,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecoveryAction {
    None,
    Scheduled { generation: Generation, at_ms: u64 },
    BudgetExhausted,
    BlockedUnknown,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceStatus {
    pub key: ServiceKey,
    pub revision: String,
    pub config_digest: String,
    pub generation: Option<Generation>,
    pub desired: DesiredState,
    pub lifecycle: Lifecycle,
    pub ready: bool,
    pub health: Health,
    pub identity: Option<ProcessIdentity>,
    pub last_exit: Option<ExitInfo>,
    pub last_readiness: Option<ProbeResult>,
    pub last_probe: Option<ProbeResult>,
    pub next_recovery: RecoveryAction,
    pub automatic_restarts: u32,
    pub started_ms: Option<u64>,
    pub pending_operation: Option<OperationId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Mutation {
    Start {
        spec: Box<ServiceSpec>,
    },
    Stop {
        key: ServiceKey,
        expected_generation: Generation,
    },
    Restart {
        key: ServiceKey,
        expected_generation: Generation,
    },
}
impl Mutation {
    pub fn key(&self) -> &ServiceKey {
        match self {
            Self::Start { spec } => &spec.key,
            Self::Stop { key, .. } | Self::Restart { key, .. } => key,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperationRequest {
    pub operation_id: OperationId,
    pub mutation: Mutation,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum OperationOutcome {
    Pending,
    Succeeded { generation: Option<Generation> },
    Failed { error: Error },
    Unknown,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationOrigin {
    #[default]
    User,
    Automatic,
    Shutdown,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationRecord {
    pub operation_id: OperationId,
    pub request_digest: String,
    pub key: ServiceKey,
    pub outcome: OperationOutcome,
    #[serde(default)]
    pub origin: OperationOrigin,
    pub created_ms: u64,
    pub updated_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub sequence: u64,
    pub at_ms: u64,
    pub key: ServiceKey,
    pub generation: Option<Generation>,
    pub operation_id: Option<OperationId>,
    pub kind: String,
    pub status: ServiceStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ownership {
    pub root_insula_id: String,
    pub owner_instance: Uuid,
    pub controller_id: Uuid,
    pub epoch: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OwnerCapabilities {
    pub owner_instance: Uuid,
    pub root_insula_id: String,
    pub canonical_authority_dir: PathBuf,
    pub generation_fencing: bool,
    pub controller_fencing: bool,
    pub durable_launch_reservations: bool,
    pub service_boundary_cleanup: bool,
    pub bounded_probes: bool,
    pub bounded_logs: bool,
    /// False is valid: actual bwrap/PID-namespace lifetime has not been verified.
    pub survives_controller_exit: bool,
    pub details: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Observation {
    Alive {
        identity: ProcessIdentity,
    },
    /// Owner certifies boundary empty AND no old launch can subsequently occur.
    Gone {
        generation: Option<Generation>,
        last_exit: Option<ExitInfo>,
        boundary_empty: bool,
        launch_resolved: bool,
    },
    Unknown,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchRequest {
    pub operation_id: OperationId,
    pub generation: Generation,
    pub spec: ServiceSpec,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StopRequest {
    pub operation_id: OperationId,
    pub key: ServiceKey,
    pub expected_generation: Generation,
    pub policy: StopPolicy,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeRequest {
    pub key: ServiceKey,
    pub generation: Generation,
    pub spec: ProbeSpec,
}
