use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use harp_contracts::{
    ContractError, ExternalSessionId, RuntimeErrorKind, RuntimeEvent, RuntimeFailure, ThreadHandle,
    ThreadId, ThreadSnapshot, ThreadSpec, TurnHandle, TurnId, TurnSpec, TurnStatus,
};
use serde::{Deserialize, Serialize};

const MAX_EVENTS: usize = 100_000;
const MAX_SERIALIZED_BYTES: usize = 64 * 1024 * 1024;
const MAX_TOTAL_WAIT: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_EVENT_WAIT: Duration = Duration::from_secs(10 * 60);
const MAX_ERROR_BYTES: usize = 8_192;
const MAX_PROVENANCE_TEXT_BYTES: usize = 256;
const MAX_ACTIVITY_PATH_BYTES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RuntimeProvenance {
    pub adapter_kind: String,
    pub implementation_version: String,
    pub persistent_state_identity: String,
    pub config_sha256: String,
}

impl RuntimeProvenance {
    pub fn new(
        adapter_kind: &str,
        implementation_version: &str,
        persistent_state_identity: &str,
        config_sha256: &str,
    ) -> Result<Self, ContractError> {
        for (field, value) in [
            ("adapterKind", adapter_kind),
            ("implementationVersion", implementation_version),
            ("persistentStateIdentity", persistent_state_identity),
        ] {
            if value.is_empty()
                || value.len() > MAX_PROVENANCE_TEXT_BYTES
                || value.chars().any(char::is_control)
            {
                return Err(ContractError::new(
                    field,
                    "must be bounded non-control text",
                ));
            }
        }
        if config_sha256.len() != 64
            || config_sha256
                .bytes()
                .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        {
            return Err(ContractError::new(
                "configSha256",
                "must be lowercase SHA-256",
            ));
        }
        Ok(Self {
            adapter_kind: adapter_kind.to_owned(),
            implementation_version: implementation_version.to_owned(),
            persistent_state_identity: persistent_state_identity.to_owned(),
            config_sha256: config_sha256.to_owned(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionLimits {
    pub max_events: usize,
    pub max_serialized_bytes: usize,
    pub max_total_wait: Duration,
    pub per_event_wait: Duration,
}

impl CollectionLimits {
    fn validate(&self) -> Result<(), RuntimeError> {
        if !(1..=MAX_EVENTS).contains(&self.max_events) {
            return Err(RuntimeError::protocol(format!(
                "max_events must be between 1 and {MAX_EVENTS}, got {}",
                self.max_events
            )));
        }
        if !(1..=MAX_SERIALIZED_BYTES).contains(&self.max_serialized_bytes) {
            return Err(RuntimeError::protocol(format!(
                "max_serialized_bytes must be between 1 and {MAX_SERIALIZED_BYTES}, got {}",
                self.max_serialized_bytes
            )));
        }
        if self.max_total_wait.is_zero() || self.max_total_wait > MAX_TOTAL_WAIT {
            return Err(RuntimeError::protocol(
                "max_total_wait must be nonzero and at most 24 hours",
            ));
        }
        if self.per_event_wait.is_zero()
            || self.per_event_wait > MAX_EVENT_WAIT
            || self.per_event_wait > self.max_total_wait
        {
            return Err(RuntimeError::protocol(
                "per_event_wait must be nonzero and at most max_total_wait and 10 minutes",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{kind:?}: {message}")]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
    message: String,
    transient: bool,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl RuntimeError {
    pub fn try_new(
        kind: RuntimeErrorKind,
        message: impl Into<String>,
        transient: bool,
    ) -> Result<Self, ContractError> {
        let message = message.into();
        RuntimeFailure {
            kind,
            message: message.clone(),
            transient,
        }
        .validate()?;
        Ok(Self {
            kind,
            message,
            transient,
            source: None,
        })
    }

    pub fn with_source<E>(
        kind: RuntimeErrorKind,
        message: impl Into<String>,
        transient: bool,
        source: E,
    ) -> Result<Self, ContractError>
    where
        E: Error + Send + Sync + 'static,
    {
        let mut error = Self::try_new(kind, message, transient)?;
        error.source = Some(Box::new(source));
        Ok(error)
    }

    pub fn kind(&self) -> RuntimeErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn is_transient(&self) -> bool {
        self.transient
    }

    pub(crate) fn with_cleanup(primary: Self, cleanup: Self) -> Self {
        let message = bounded_protocol_message(&format!(
            "{}; shutdown failed: {}",
            primary.message, cleanup
        ));
        Self {
            kind: primary.kind,
            message,
            transient: primary.transient,
            source: Some(Box::new(CleanupFailure { primary, cleanup })),
        }
    }

    pub(crate) fn protocol(message: impl AsRef<str>) -> Self {
        let message = bounded_protocol_message(message.as_ref());
        Self {
            kind: RuntimeErrorKind::Protocol,
            message,
            transient: false,
            source: None,
        }
    }

    pub fn from_contract(error: ContractError) -> Self {
        let message = bounded_protocol_message(&error.to_string());
        Self {
            kind: RuntimeErrorKind::Protocol,
            message,
            transient: false,
            source: Some(Box::new(error)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("primary runtime failure: {primary}; shutdown failure: {cleanup}")]
struct CleanupFailure {
    primary: RuntimeError,
    cleanup: RuntimeError,
}

impl TryFrom<RuntimeFailure> for RuntimeError {
    type Error = ContractError;

    fn try_from(failure: RuntimeFailure) -> Result<Self, Self::Error> {
        failure.validate()?;
        Ok(Self {
            kind: failure.kind,
            message: failure.message,
            transient: failure.transient,
            source: None,
        })
    }
}

fn bounded_protocol_message(message: &str) -> String {
    let mut bounded = String::with_capacity(message.len().min(MAX_ERROR_BYTES));
    for character in message.chars() {
        let character = if character.is_control() && character != '\n' {
            ' '
        } else {
            character
        };
        if bounded.len() + character.len_utf8() > MAX_ERROR_BYTES {
            break;
        }
        bounded.push(character);
    }
    if bounded.is_empty() {
        bounded.push_str("runtime protocol failure");
    }
    bounded
}

#[async_trait]
pub trait RuntimeControl: Send + Sync {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError>;

    async fn read_thread(&self, thread: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError>;

    async fn interrupt(&self, turn: &TurnHandle) -> Result<(), RuntimeError>;

    async fn interrupt_activity(
        &self,
        activity: &ActivityHandle,
        purpose: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptPurpose {
    Budget,
    ScannerIntegrity,
    Cancellation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ActivitySpec {
    pub logical_session_id: ThreadId,
    pub logical_turn_id: TurnId,
    pub thread_spec: ThreadSpec,
    pub turn_spec: TurnSpec,
    pub activity_dir: String,
    pub invocation_sha256: String,
    pub external_session_id: Option<ExternalSessionId>,
}

impl ActivitySpec {
    pub fn validate(&self) -> Result<(), RuntimeError> {
        validate_id_text("logicalSessionId", &self.logical_session_id.to_string())?;
        validate_id_text("logicalTurnId", &self.logical_turn_id.to_string())?;
        self.thread_spec
            .validate()
            .map_err(RuntimeError::from_contract)?;
        self.turn_spec
            .validate()
            .map_err(RuntimeError::from_contract)?;
        validate_activity_dir(&self.activity_dir)?;
        validate_sha256("invocationSha256", &self.invocation_sha256)?;
        if let Some(external_session_id) = &self.external_session_id {
            validate_id_text("externalSessionId", &external_session_id.to_string())?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ActivityHandle {
    pub logical_session_id: ThreadId,
    pub logical_turn_id: TurnId,
    pub process_record_sha256: String,
    pub external_session_id: Option<ExternalSessionId>,
}

impl ActivityHandle {
    pub fn validate(&self) -> Result<(), RuntimeError> {
        validate_id_text("logicalSessionId", &self.logical_session_id.to_string())?;
        validate_id_text("logicalTurnId", &self.logical_turn_id.to_string())?;
        validate_sha256("processRecordSha256", &self.process_record_sha256)?;
        if let Some(external_session_id) = &self.external_session_id {
            validate_id_text("externalSessionId", &external_session_id.to_string())?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InterruptReceipt {
    pub process_record_sha256: String,
    pub quiescent: bool,
}

impl InterruptReceipt {
    pub fn validate(&self) -> Result<(), RuntimeError> {
        validate_sha256("processRecordSha256", &self.process_record_sha256)
    }
}

#[async_trait]
pub trait ActivityRuntime: Send {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError>;

    fn control_handle(&self) -> Result<Arc<dyn RuntimeControl>, RuntimeError>;

    async fn start_logical_session(
        &mut self,
        spec: ThreadSpec,
    ) -> Result<ThreadHandle, RuntimeError>;

    async fn start_activity(&mut self, spec: ActivitySpec) -> Result<ActivityHandle, RuntimeError>;

    async fn next_event(
        &mut self,
        activity: &ActivityHandle,
        max_wait: Duration,
    ) -> Result<RuntimeEvent, RuntimeError>;

    async fn interrupt(
        &mut self,
        activity: &ActivityHandle,
        purpose: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError>;
}

#[async_trait]
pub trait CodexRuntime: Send {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError>;

    fn control_handle(&self) -> Result<Arc<dyn RuntimeControl>, RuntimeError>;

    async fn start_thread(&mut self, spec: ThreadSpec) -> Result<ThreadHandle, RuntimeError>;

    async fn start_turn(
        &mut self,
        thread: &ThreadHandle,
        spec: TurnSpec,
    ) -> Result<TurnHandle, RuntimeError>;

    async fn read_thread(&mut self, thread: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError>;

    async fn resume_thread(
        &mut self,
        thread: &ThreadHandle,
    ) -> Result<ThreadSnapshot, RuntimeError>;

    async fn next_event(
        &mut self,
        turn: &TurnHandle,
        max_wait: Duration,
    ) -> Result<RuntimeEvent, RuntimeError>;

    async fn interrupt(&mut self, turn: &TurnHandle) -> Result<(), RuntimeError>;

    async fn shutdown_thread(&mut self, thread: ThreadHandle) -> Result<(), RuntimeError>;
}

fn validate_id_text(field: &'static str, value: &str) -> Result<(), RuntimeError> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(RuntimeError::protocol(format!(
            "{field} must be bounded non-control text"
        )));
    }
    Ok(())
}

fn validate_sha256(field: &'static str, value: &str) -> Result<(), RuntimeError> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        return Err(RuntimeError::protocol(format!(
            "{field} must be a lowercase SHA-256 digest"
        )));
    }
    Ok(())
}

fn validate_activity_dir(value: &str) -> Result<(), RuntimeError> {
    if value.is_empty()
        || value.len() > MAX_ACTIVITY_PATH_BYTES
        || !value.starts_with('/')
        || value.contains('\0')
    {
        return Err(RuntimeError::protocol(
            "activityDir must be a bounded absolute path",
        ));
    }
    Ok(())
}

pub async fn collect_until_terminal(
    runtime: &mut dyn CodexRuntime,
    turn: &TurnHandle,
    limits: &CollectionLimits,
) -> Result<Vec<RuntimeEvent>, RuntimeError> {
    turn.validate().map_err(RuntimeError::from_contract)?;
    limits.validate()?;

    let deadline = tokio::time::Instant::now() + limits.max_total_wait;
    let mut events = Vec::with_capacity(limits.max_events.min(128));
    let mut serialized_bytes = 0usize;
    for _ in 0..limits.max_events {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return Err(timeout_error());
        }
        let wait = limits.per_event_wait.min(remaining);
        let event = tokio::time::timeout(wait, runtime.next_event(turn, wait))
            .await
            .map_err(|_| timeout_error())??;
        event.validate().map_err(RuntimeError::from_contract)?;
        validate_event_scope(turn, &event)?;
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|error| RuntimeError::protocol(format!("serialize runtime event: {error}")))?
            .len();
        serialized_bytes = serialized_bytes
            .checked_add(event_bytes)
            .ok_or_else(|| RuntimeError::protocol("runtime event byte accounting overflowed"))?;
        if serialized_bytes > limits.max_serialized_bytes {
            return Err(RuntimeError::protocol(format!(
                "runtime events exceeded {} serialized bytes",
                limits.max_serialized_bytes
            )));
        }
        match &event {
            RuntimeEvent::TurnCompleted(completed) => match completed.turn.status {
                TurnStatus::Completed | TurnStatus::Interrupted | TurnStatus::Failed => {
                    events.push(event);
                    return Ok(events);
                }
                TurnStatus::InProgress => {
                    return Err(RuntimeError::protocol(
                        "TurnCompleted event carried nonterminal InProgress status",
                    ));
                }
            },
            RuntimeEvent::Disconnected(disconnected) => {
                return RuntimeError::try_new(
                    RuntimeErrorKind::Disconnected,
                    bounded_protocol_message(&format!(
                        "runtime disconnected before terminal event: {}",
                        disconnected.reason
                    )),
                    true,
                )
                .map_or_else(|error| Err(RuntimeError::from_contract(error)), Err);
            }
            RuntimeEvent::Lagged(lagged) => {
                return RuntimeError::try_new(
                    RuntimeErrorKind::Transport,
                    format!(
                        "runtime event stream lost {} event(s)",
                        lagged.dropped_events
                    ),
                    true,
                )
                .map_or_else(|error| Err(RuntimeError::from_contract(error)), Err);
            }
            RuntimeEvent::ServerRequest(_) => {
                return RuntimeError::try_new(
                    RuntimeErrorKind::ApprovalRequired,
                    "runtime requested server interaction before terminal event",
                    false,
                )
                .map_or_else(|error| Err(RuntimeError::from_contract(error)), Err);
            }
            _ => events.push(event),
        }
    }

    Err(RuntimeError::protocol(format!(
        "matching terminal event not observed within {} events",
        limits.max_events
    )))
}

fn timeout_error() -> RuntimeError {
    RuntimeError::try_new(RuntimeErrorKind::Transport, "runtime event timeout", true)
        .unwrap_or_else(RuntimeError::from_contract)
}

fn validate_event_scope(turn: &TurnHandle, event: &RuntimeEvent) -> Result<(), RuntimeError> {
    let matches_turn = match event {
        RuntimeEvent::TurnStarted(event) => {
            event.thread_id == turn.thread_id && event.turn_id == turn.turn_id
        }
        RuntimeEvent::TokenUsage(event) => {
            event.thread_id == turn.thread_id && event.turn_id == turn.turn_id
        }
        RuntimeEvent::TurnCompleted(event) => {
            event.thread_id == turn.thread_id && event.turn.turn_id == turn.turn_id
        }
        RuntimeEvent::ServerRequest(event) => {
            event.thread_id == turn.thread_id
                && event
                    .turn_id
                    .as_ref()
                    .is_none_or(|turn_id| turn_id == &turn.turn_id)
        }
        RuntimeEvent::Disconnected(_) | RuntimeEvent::Lagged(_) => true,
        RuntimeEvent::ThreadStarted(_) => false,
    };
    if matches_turn {
        Ok(())
    } else {
        Err(RuntimeError::protocol(
            "runtime adapter returned an event outside the requested turn scope",
        ))
    }
}
