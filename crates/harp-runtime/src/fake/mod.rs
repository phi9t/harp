use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use async_trait::async_trait;
use harp_contracts::{
    ContractError, ExternalSessionId, RuntimeErrorKind, RuntimeEvent, RuntimeFailure, ThreadHandle,
    ThreadId, ThreadSnapshot, ThreadSpec, TurnHandle, TurnId,
};

use crate::{
    ActivityHandle, ActivityRuntime, ActivitySpec, InterruptPurpose, InterruptReceipt,
    RuntimeControl, RuntimeError, RuntimeProvenance,
};

pub mod scenarios;

const MAX_SCRIPT_STEPS: usize = 100_000;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum FakeStep {
    StartLogicalSession {
        expected: ThreadSpec,
        result: Result<ThreadHandle, RuntimeFailure>,
    },
    StartActivity {
        expected: ActivitySpec,
        process_record_sha256: String,
        external_session_id: Option<ExternalSessionId>,
        result: Result<(), RuntimeFailure>,
    },
    ActivityEvent {
        expected_logical_session_id: ThreadId,
        expected_logical_turn_id: TurnId,
        result: Result<RuntimeEvent, RuntimeFailure>,
    },
    ActivityInterrupt {
        expected_logical_session_id: ThreadId,
        expected_logical_turn_id: TurnId,
        expected_purpose: InterruptPurpose,
        result: Result<InterruptReceipt, RuntimeFailure>,
    },
}

impl FakeStep {
    fn method_name(&self) -> &'static str {
        match self {
            Self::StartLogicalSession { .. } => "start_logical_session",
            Self::StartActivity { .. } => "start_activity",
            Self::ActivityEvent { .. } => "activity_next_event",
            Self::ActivityInterrupt { .. } => "activity_interrupt",
        }
    }

    fn validate(&self) -> Result<(), ContractError> {
        match self {
            Self::StartLogicalSession { expected, result } => {
                expected.validate()?;
                validate_result(result, ThreadHandle::validate)
            }
            Self::StartActivity {
                expected,
                process_record_sha256,
                result,
                ..
            } => {
                expected
                    .validate()
                    .map_err(|_| ContractError::new("activitySpec", "must be valid"))?;
                validate_sha256_contract("processRecordSha256", process_record_sha256)?;
                validate_unit_result(result)
            }
            Self::ActivityEvent {
                expected_logical_session_id,
                expected_logical_turn_id,
                result,
            } => {
                expected_logical_session_id
                    .to_string()
                    .parse::<ThreadId>()?;
                expected_logical_turn_id.to_string().parse::<TurnId>()?;
                validate_result(result, |event| {
                    event.validate()?;
                    validate_activity_event_scope(
                        expected_logical_session_id,
                        expected_logical_turn_id,
                        event,
                    )
                })
            }
            Self::ActivityInterrupt {
                expected_logical_session_id,
                expected_logical_turn_id,
                result,
                ..
            } => {
                expected_logical_session_id
                    .to_string()
                    .parse::<ThreadId>()?;
                expected_logical_turn_id.to_string().parse::<TurnId>()?;
                validate_result(result, |receipt| {
                    receipt
                        .validate()
                        .map_err(|_| ContractError::new("interruptReceipt", "must be valid"))
                })
            }
        }
    }
}

fn validate_sha256_contract(field: &'static str, value: &str) -> Result<(), ContractError> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        return Err(ContractError::new(field, "must be lowercase SHA-256"));
    }
    Ok(())
}

fn validate_result<T>(
    result: &Result<T, RuntimeFailure>,
    validate: impl FnOnce(&T) -> Result<(), ContractError>,
) -> Result<(), ContractError> {
    match result {
        Ok(value) => validate(value),
        Err(failure) => failure.validate(),
    }
}

fn validate_unit_result(result: &Result<(), RuntimeFailure>) -> Result<(), ContractError> {
    match result {
        Ok(()) => Ok(()),
        Err(failure) => failure.validate(),
    }
}

fn validate_activity_event_scope(
    expected_session: &ThreadId,
    expected_turn: &TurnId,
    event: &RuntimeEvent,
) -> Result<(), ContractError> {
    let matches_scope = match event {
        RuntimeEvent::ThreadStarted(event) => &event.thread_id == expected_session,
        RuntimeEvent::TurnStarted(event) => {
            &event.thread_id == expected_session && &event.turn_id == expected_turn
        }
        RuntimeEvent::TokenUsage(event) => {
            &event.thread_id == expected_session && &event.turn_id == expected_turn
        }
        RuntimeEvent::TurnCompleted(event) => {
            &event.thread_id == expected_session && &event.turn.turn_id == expected_turn
        }
        RuntimeEvent::ServerRequest(event) => {
            &event.thread_id == expected_session
                && event
                    .turn_id
                    .as_ref()
                    .is_none_or(|turn_id| turn_id == expected_turn)
        }
        RuntimeEvent::Disconnected(_) | RuntimeEvent::Lagged(_) => true,
    };
    if matches_scope {
        Ok(())
    } else {
        Err(ContractError::new(
            "runtimeEvent",
            "must belong to the scripted activity scope",
        ))
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeCall {
    StartLogicalSession {
        spec: ThreadSpec,
    },
    StartActivity {
        spec: ActivitySpec,
    },
    ActivityNextEvent {
        activity: ActivityHandle,
        max_wait: Duration,
    },
    ActivityInterrupt {
        activity: ActivityHandle,
        purpose: InterruptPurpose,
    },
}

struct FakeRuntimeState {
    steps: VecDeque<FakeStep>,
    calls: Vec<RuntimeCall>,
}

pub struct FakeCodexRuntime {
    state: Arc<Mutex<FakeRuntimeState>>,
}

impl FakeCodexRuntime {
    pub fn new(steps: Vec<FakeStep>) -> Result<Self, ContractError> {
        if steps.len() > MAX_SCRIPT_STEPS {
            return Err(ContractError::new(
                "steps",
                "must contain at most 100000 entries",
            ));
        }
        for step in &steps {
            step.validate()?;
        }
        Ok(Self {
            state: Arc::new(Mutex::new(FakeRuntimeState {
                steps: steps.into(),
                calls: Vec::new(),
            })),
        })
    }

    pub fn calls(&self) -> Vec<RuntimeCall> {
        self.lock().calls.clone()
    }

    pub fn remaining_steps(&self) -> usize {
        self.lock().steps.len()
    }

    pub fn assert_exhausted(&self) -> Result<(), RuntimeError> {
        let state = self.lock();
        if state.steps.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::protocol(format!(
                "fake runtime has {} unconsumed scripted step(s); next is {}",
                state.steps.len(),
                state.steps.front().map_or("unknown", FakeStep::method_name)
            )))
        }
    }

    fn lock(&self) -> MutexGuard<'_, FakeRuntimeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn unexpected(state: &FakeRuntimeState, actual: &str) -> RuntimeError {
    match state.steps.front() {
        Some(step) => RuntimeError::protocol(format!(
            "unexpected fake runtime call: expected {}, received {actual}",
            step.method_name()
        )),
        None => RuntimeError::protocol(format!("fake runtime script exhausted before {actual}")),
    }
}

fn mismatch(method: &str) -> RuntimeError {
    RuntimeError::protocol(format!(
        "{method} normalized input did not match the scripted expectation"
    ))
}

fn convert_failure(failure: RuntimeFailure) -> RuntimeError {
    RuntimeError::try_from(failure).unwrap_or_else(RuntimeError::from_contract)
}

fn validated_output<T>(
    value: T,
    context: &str,
    validate: impl FnOnce(&T) -> Result<(), ContractError>,
) -> Result<T, RuntimeError> {
    validate(&value).map_err(|error| RuntimeError::protocol(format!("{context}: {error}")))?;
    Ok(value)
}

#[async_trait]
impl ActivityRuntime for FakeCodexRuntime {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError> {
        fake_provenance()
    }

    fn control_handle(&self) -> Result<Arc<dyn RuntimeControl>, RuntimeError> {
        Ok(Arc::new(FakeRuntimeControl {
            state: Arc::clone(&self.state),
        }))
    }

    async fn start_logical_session(
        &mut self,
        spec: ThreadSpec,
    ) -> Result<ThreadHandle, RuntimeError> {
        spec.validate().map_err(RuntimeError::from_contract)?;
        if let Some(authority) = &spec.workspace_authority {
            harp_artifacts::verify_runtime_workspace_authority(authority).map_err(|error| {
                RuntimeError::with_source(
                    RuntimeErrorKind::Protocol,
                    "runtime workspace authority verification failed",
                    false,
                    error,
                )
                .unwrap_or_else(RuntimeError::from_contract)
            })?;
        }
        let mut state = self.lock();
        state
            .calls
            .push(RuntimeCall::StartLogicalSession { spec: spec.clone() });
        match state.steps.front() {
            Some(FakeStep::StartLogicalSession { expected, .. }) if expected == &spec => {}
            Some(FakeStep::StartLogicalSession { .. }) => {
                return Err(mismatch("start_logical_session"));
            }
            _ => return Err(unexpected(&state, "start_logical_session")),
        }
        match state.steps.pop_front() {
            Some(FakeStep::StartLogicalSession { result, .. }) => match result {
                Ok(handle) => validated_output(
                    handle,
                    "invalid start_logical_session output",
                    ThreadHandle::validate,
                ),
                Err(failure) => Err(convert_failure(failure)),
            },
            _ => Err(unexpected(&state, "start_logical_session")),
        }
    }

    async fn start_activity(&mut self, spec: ActivitySpec) -> Result<ActivityHandle, RuntimeError> {
        spec.validate()?;
        let mut state = self.lock();
        state
            .calls
            .push(RuntimeCall::StartActivity { spec: spec.clone() });
        match state.steps.front() {
            Some(FakeStep::StartActivity { expected, .. }) if expected == &spec => {}
            Some(FakeStep::StartActivity { .. }) => return Err(mismatch("start_activity")),
            _ => return Err(unexpected(&state, "start_activity")),
        }
        match state.steps.pop_front() {
            Some(FakeStep::StartActivity {
                process_record_sha256,
                external_session_id,
                result,
                ..
            }) => match result {
                Ok(()) => {
                    let handle = ActivityHandle {
                        logical_session_id: spec.logical_session_id,
                        logical_turn_id: spec.logical_turn_id,
                        process_record_sha256,
                        external_session_id,
                    };
                    handle.validate()?;
                    Ok(handle)
                }
                Err(failure) => Err(convert_failure(failure)),
            },
            _ => Err(unexpected(&state, "start_activity")),
        }
    }

    async fn next_event(
        &mut self,
        activity: &ActivityHandle,
        max_wait: Duration,
    ) -> Result<RuntimeEvent, RuntimeError> {
        activity.validate()?;
        if max_wait.is_zero() || max_wait > Duration::from_secs(10 * 60) {
            return Err(RuntimeError::protocol(
                "max_wait must be nonzero and at most 10 minutes",
            ));
        }
        let mut state = self.lock();
        state.calls.push(RuntimeCall::ActivityNextEvent {
            activity: activity.clone(),
            max_wait,
        });
        match state.steps.front() {
            Some(FakeStep::ActivityEvent {
                expected_logical_session_id,
                expected_logical_turn_id,
                ..
            }) if expected_logical_session_id == &activity.logical_session_id
                && expected_logical_turn_id == &activity.logical_turn_id => {}
            Some(FakeStep::ActivityEvent { .. }) => {
                return Err(mismatch("activity_next_event"));
            }
            _ => return Err(unexpected(&state, "activity_next_event")),
        }
        match state.steps.pop_front() {
            Some(FakeStep::ActivityEvent { result, .. }) => match result {
                Ok(event) => validated_output(event, "invalid activity event output", |event| {
                    event.validate()?;
                    validate_activity_event_scope(
                        &activity.logical_session_id,
                        &activity.logical_turn_id,
                        event,
                    )
                }),
                Err(failure) => Err(convert_failure(failure)),
            },
            _ => Err(unexpected(&state, "activity_next_event")),
        }
    }

    async fn interrupt(
        &mut self,
        activity: &ActivityHandle,
        purpose: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError> {
        consume_activity_interrupt(&mut self.lock(), activity, purpose)
    }
}

fn consume_activity_interrupt(
    state: &mut FakeRuntimeState,
    activity: &ActivityHandle,
    purpose: InterruptPurpose,
) -> Result<InterruptReceipt, RuntimeError> {
    activity.validate()?;
    state.calls.push(RuntimeCall::ActivityInterrupt {
        activity: activity.clone(),
        purpose,
    });
    match state.steps.front() {
        Some(FakeStep::ActivityInterrupt {
            expected_logical_session_id,
            expected_logical_turn_id,
            expected_purpose,
            ..
        }) if expected_logical_session_id == &activity.logical_session_id
            && expected_logical_turn_id == &activity.logical_turn_id
            && expected_purpose == &purpose => {}
        Some(FakeStep::ActivityInterrupt { .. }) => {
            return Err(mismatch("activity_interrupt"));
        }
        _ => return Err(unexpected(state, "activity_interrupt")),
    }
    match state.steps.pop_front() {
        Some(FakeStep::ActivityInterrupt { result, .. }) => match result {
            Ok(receipt) => {
                receipt.validate()?;
                Ok(receipt)
            }
            Err(failure) => Err(convert_failure(failure)),
        },
        _ => Err(unexpected(state, "activity_interrupt")),
    }
}

struct FakeRuntimeControl {
    state: Arc<Mutex<FakeRuntimeState>>,
}

impl FakeRuntimeControl {
    fn lock(&self) -> MutexGuard<'_, FakeRuntimeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[async_trait]
impl RuntimeControl for FakeRuntimeControl {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError> {
        fake_provenance()
    }

    async fn read_thread(&self, _thread: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError> {
        Err(RuntimeError::protocol(
            "scripted activity runtime does not expose thread snapshots",
        ))
    }

    async fn interrupt(&self, _turn: &TurnHandle) -> Result<(), RuntimeError> {
        Err(RuntimeError::protocol(
            "scripted activity runtime requires activity interruption",
        ))
    }

    async fn interrupt_activity(
        &self,
        activity: &ActivityHandle,
        purpose: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError> {
        consume_activity_interrupt(&mut self.lock(), activity, purpose)
    }
}

fn fake_provenance() -> Result<RuntimeProvenance, RuntimeError> {
    RuntimeProvenance::new(
        "scripted-fake",
        env!("CARGO_PKG_VERSION"),
        "in-memory-scripted-backend",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    )
    .map_err(RuntimeError::from_contract)
}
