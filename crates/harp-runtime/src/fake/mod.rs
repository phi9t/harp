use std::collections::VecDeque;
use std::time::Duration;

use async_trait::async_trait;
use harp_contracts::{
    ContractError, ExternalSessionId, RuntimeErrorKind, RuntimeEvent, RuntimeFailure, ThreadHandle,
    ThreadId, ThreadSnapshot, ThreadSpec, TurnHandle, TurnId, TurnSpec,
};

use crate::{
    ActivityHandle, ActivityRuntime, ActivitySpec, CodexRuntime, InterruptPurpose,
    InterruptReceipt, RuntimeControl, RuntimeError, RuntimeProvenance,
};

mod backend;
pub mod scenarios;

pub use backend::{
    FakeBackend, FakeBackendSnapshot, FakeExternalEffect, FakeThreadState, FakeTurnState,
};

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
    StartThread {
        expected: ThreadSpec,
        external: Option<ThreadHandle>,
        result: Result<ThreadHandle, RuntimeFailure>,
    },
    StartTurn {
        expected_thread: ThreadHandle,
        expected: TurnSpec,
        external: Option<TurnHandle>,
        result: Result<TurnHandle, RuntimeFailure>,
    },
    ReadThread {
        expected: ThreadHandle,
        result: FakeSnapshotResult,
    },
    ResumeThread {
        expected: ThreadHandle,
        result: FakeSnapshotResult,
    },
    Event {
        expected_turn: TurnHandle,
        result: Result<RuntimeEvent, RuntimeFailure>,
    },
    Interrupt {
        expected: TurnHandle,
        result: Result<(), RuntimeFailure>,
    },
    Shutdown {
        expected: ThreadHandle,
        result: Result<(), RuntimeFailure>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum FakeSnapshotResult {
    Backend,
    Scripted(Result<ThreadSnapshot, RuntimeFailure>),
}

impl FakeStep {
    fn method_name(&self) -> &'static str {
        match self {
            Self::StartLogicalSession { .. } => "start_logical_session",
            Self::StartActivity { .. } => "start_activity",
            Self::ActivityEvent { .. } => "activity_next_event",
            Self::ActivityInterrupt { .. } => "activity_interrupt",
            Self::StartThread { .. } => "start_thread",
            Self::StartTurn { .. } => "start_turn",
            Self::ReadThread { .. } => "read_thread",
            Self::ResumeThread { .. } => "resume_thread",
            Self::Event { .. } => "next_event",
            Self::Interrupt { .. } => "interrupt",
            Self::Shutdown { .. } => "shutdown_thread",
        }
    }

    fn validate(&self, backend: &mut backend::FakeExternalState) -> Result<(), ContractError> {
        match self {
            Self::StartLogicalSession { expected, result } => {
                expected.validate()?;
                validate_result(result, ThreadHandle::validate)
            }
            Self::StartActivity {
                expected,
                process_record_sha256,
                external_session_id: _,
                result,
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
            Self::StartThread {
                expected,
                external,
                result,
            } => {
                expected.validate()?;
                validate_external_result(external.as_ref(), result, ThreadHandle::validate)?;
                if let Some(external) = external {
                    backend
                        .create_thread(external.clone())
                        .map_err(runtime_to_contract)?;
                }
                Ok(())
            }
            Self::StartTurn {
                expected_thread,
                expected,
                external,
                result,
            } => {
                expected_thread.validate()?;
                expected.validate()?;
                validate_external_result(external.as_ref(), result, |handle| {
                    handle.validate()?;
                    validate_same_thread(&expected_thread.thread_id, &handle.thread_id)
                })?;
                if let Some(external) = external {
                    backend
                        .create_turn(
                            expected_thread,
                            external.clone(),
                            expected.operation_marker.clone(),
                        )
                        .map_err(runtime_to_contract)?;
                }
                Ok(())
            }
            Self::ReadThread { expected, result } | Self::ResumeThread { expected, result } => {
                expected.validate()?;
                match result {
                    FakeSnapshotResult::Backend => {
                        backend
                            .snapshot_thread(expected)
                            .map_err(runtime_to_contract)?;
                        Ok(())
                    }
                    FakeSnapshotResult::Scripted(result) => validate_result(result, |snapshot| {
                        snapshot.validate()?;
                        validate_same_thread(&expected.thread_id, &snapshot.thread_id)
                    }),
                }
            }
            Self::Event {
                expected_turn,
                result,
            } => {
                expected_turn.validate()?;
                validate_result(result, |event| {
                    event.validate()?;
                    validate_event_scope(expected_turn, event)
                })
            }
            Self::Interrupt { expected, result } => {
                expected.validate()?;
                validate_unit_result(result)?;
                if result.is_ok() {
                    backend.interrupt(expected).map_err(runtime_to_contract)?;
                }
                Ok(())
            }
            Self::Shutdown { expected, result } => {
                expected.validate()?;
                validate_unit_result(result)?;
                if result.is_ok() {
                    backend.shutdown(expected).map_err(runtime_to_contract)?;
                }
                Ok(())
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

fn validate_external_result<T: PartialEq>(
    external: Option<&T>,
    result: &Result<T, RuntimeFailure>,
    validate: impl Fn(&T) -> Result<(), ContractError>,
) -> Result<(), ContractError> {
    if let Some(external) = external {
        validate(external)?;
    }
    match result {
        Ok(value) => {
            validate(value)?;
            if external != Some(value) {
                return Err(ContractError::new(
                    "external",
                    "must exist and equal the successful result",
                ));
            }
        }
        Err(failure) => {
            failure.validate()?;
            if external.is_some()
                && (!failure.transient
                    || !matches!(
                        failure.kind,
                        RuntimeErrorKind::Transport | RuntimeErrorKind::Disconnected
                    ))
            {
                return Err(ContractError::new(
                    "external",
                    "with an error requires transient transport or disconnected failure",
                ));
            }
        }
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

fn runtime_to_contract(_: RuntimeError) -> ContractError {
    ContractError::new("backend", "scripted external effect is invalid")
}

fn validate_same_thread(
    expected: &harp_contracts::ThreadId,
    actual: &harp_contracts::ThreadId,
) -> Result<(), ContractError> {
    if expected == actual {
        return Ok(());
    }
    Err(ContractError::new(
        "threadId",
        "must match the scripted thread identity",
    ))
}

fn validate_event_scope(expected: &TurnHandle, event: &RuntimeEvent) -> Result<(), ContractError> {
    let matches_scope = match event {
        RuntimeEvent::TurnStarted(event) => {
            event.thread_id == expected.thread_id && event.turn_id == expected.turn_id
        }
        RuntimeEvent::TokenUsage(event) => {
            event.thread_id == expected.thread_id && event.turn_id == expected.turn_id
        }
        RuntimeEvent::TurnCompleted(event) => {
            event.thread_id == expected.thread_id && event.turn.turn_id == expected.turn_id
        }
        RuntimeEvent::ServerRequest(event) => {
            event.thread_id == expected.thread_id
                && event
                    .turn_id
                    .as_ref()
                    .is_none_or(|turn_id| turn_id == &expected.turn_id)
        }
        RuntimeEvent::Disconnected(_) | RuntimeEvent::Lagged(_) => true,
        RuntimeEvent::ThreadStarted(_) => false,
    };
    if matches_scope {
        Ok(())
    } else {
        Err(ContractError::new(
            "runtimeEvent",
            "must belong to the scripted turn scope",
        ))
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
    StartThread {
        spec: ThreadSpec,
    },
    StartTurn {
        thread: ThreadHandle,
        spec: TurnSpec,
    },
    ReadThread {
        thread: ThreadHandle,
    },
    ResumeThread {
        thread: ThreadHandle,
    },
    NextEvent {
        turn: TurnHandle,
        max_wait: Duration,
    },
    Interrupt {
        turn: TurnHandle,
    },
    Shutdown {
        thread: ThreadHandle,
    },
}

pub struct FakeCodexRuntime {
    steps: VecDeque<FakeStep>,
    calls: Vec<RuntimeCall>,
    backend: FakeBackend,
}

impl FakeCodexRuntime {
    pub fn new(steps: Vec<FakeStep>) -> Result<Self, ContractError> {
        Self::with_backend(steps, FakeBackend::default())
    }

    pub fn with_backend(steps: Vec<FakeStep>, backend: FakeBackend) -> Result<Self, ContractError> {
        if steps.len() > MAX_SCRIPT_STEPS {
            return Err(ContractError::new(
                "steps",
                "must contain at most 100000 entries",
            ));
        }
        let mut validation_state = backend.state();
        for step in &steps {
            step.validate(&mut validation_state)?;
        }
        Ok(Self {
            steps: steps.into(),
            calls: Vec::new(),
            backend,
        })
    }

    pub fn calls(&self) -> &[RuntimeCall] {
        &self.calls
    }

    pub fn remaining_steps(&self) -> usize {
        self.steps.len()
    }

    pub fn assert_exhausted(&self) -> Result<(), RuntimeError> {
        if self.steps.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::protocol(format!(
                "fake runtime has {} unconsumed scripted step(s); next is {}",
                self.steps.len(),
                self.steps.front().map_or("unknown", FakeStep::method_name)
            )))
        }
    }

    fn unexpected(&self, actual: &str) -> RuntimeError {
        match self.steps.front() {
            Some(step) => RuntimeError::protocol(format!(
                "unexpected fake runtime call: expected {}, received {actual}",
                step.method_name()
            )),
            None => {
                RuntimeError::protocol(format!("fake runtime script exhausted before {actual}"))
            }
        }
    }

    fn mismatch(&self, method: &str) -> RuntimeError {
        RuntimeError::protocol(format!(
            "{method} normalized input did not match the scripted expectation"
        ))
    }
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
        RuntimeProvenance::new(
            "scripted-fake",
            env!("CARGO_PKG_VERSION"),
            "in-memory-scripted-backend",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .map_err(RuntimeError::from_contract)
    }

    fn control_handle(&self) -> Result<std::sync::Arc<dyn RuntimeControl>, RuntimeError> {
        Ok(std::sync::Arc::new(FakeRuntimeControl {
            backend: self.backend.clone(),
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
        self.calls
            .push(RuntimeCall::StartLogicalSession { spec: spec.clone() });
        match self.steps.front() {
            Some(FakeStep::StartLogicalSession { expected, .. }) if expected == &spec => {}
            Some(FakeStep::StartLogicalSession { .. }) => {
                return Err(self.mismatch("start_logical_session"));
            }
            _ => return Err(self.unexpected("start_logical_session")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::StartLogicalSession { result, .. }) => match result {
                Ok(handle) => validated_output(
                    handle,
                    "invalid start_logical_session output",
                    ThreadHandle::validate,
                ),
                Err(failure) => Err(convert_failure(failure)),
            },
            _ => Err(self.unexpected("start_logical_session")),
        }
    }

    async fn start_activity(&mut self, spec: ActivitySpec) -> Result<ActivityHandle, RuntimeError> {
        spec.validate()?;
        self.calls
            .push(RuntimeCall::StartActivity { spec: spec.clone() });
        match self.steps.front() {
            Some(FakeStep::StartActivity { expected, .. }) if expected == &spec => {}
            Some(FakeStep::StartActivity { .. }) => return Err(self.mismatch("start_activity")),
            _ => return Err(self.unexpected("start_activity")),
        }
        match self.steps.pop_front() {
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
            _ => Err(self.unexpected("start_activity")),
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
        self.calls.push(RuntimeCall::ActivityNextEvent {
            activity: activity.clone(),
            max_wait,
        });
        match self.steps.front() {
            Some(FakeStep::ActivityEvent {
                expected_logical_session_id,
                expected_logical_turn_id,
                ..
            }) if expected_logical_session_id == &activity.logical_session_id
                && expected_logical_turn_id == &activity.logical_turn_id => {}
            Some(FakeStep::ActivityEvent { .. }) => {
                return Err(self.mismatch("activity_next_event"));
            }
            _ => return Err(self.unexpected("activity_next_event")),
        }
        match self.steps.pop_front() {
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
            _ => Err(self.unexpected("activity_next_event")),
        }
    }

    async fn interrupt(
        &mut self,
        activity: &ActivityHandle,
        purpose: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError> {
        activity.validate()?;
        self.calls.push(RuntimeCall::ActivityInterrupt {
            activity: activity.clone(),
            purpose,
        });
        match self.steps.front() {
            Some(FakeStep::ActivityInterrupt {
                expected_logical_session_id,
                expected_logical_turn_id,
                expected_purpose,
                ..
            }) if expected_logical_session_id == &activity.logical_session_id
                && expected_logical_turn_id == &activity.logical_turn_id
                && expected_purpose == &purpose => {}
            Some(FakeStep::ActivityInterrupt { .. }) => {
                return Err(self.mismatch("activity_interrupt"));
            }
            _ => return Err(self.unexpected("activity_interrupt")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::ActivityInterrupt { result, .. }) => match result {
                Ok(receipt) => {
                    receipt.validate()?;
                    Ok(receipt)
                }
                Err(failure) => Err(convert_failure(failure)),
            },
            _ => Err(self.unexpected("activity_interrupt")),
        }
    }
}

#[async_trait]
impl CodexRuntime for FakeCodexRuntime {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError> {
        RuntimeProvenance::new(
            "scripted-fake",
            env!("CARGO_PKG_VERSION"),
            "in-memory-scripted-backend",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .map_err(RuntimeError::from_contract)
    }

    fn control_handle(&self) -> Result<std::sync::Arc<dyn RuntimeControl>, RuntimeError> {
        Ok(std::sync::Arc::new(FakeRuntimeControl {
            backend: self.backend.clone(),
        }))
    }

    async fn start_thread(&mut self, spec: ThreadSpec) -> Result<ThreadHandle, RuntimeError> {
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
        self.calls
            .push(RuntimeCall::StartThread { spec: spec.clone() });
        match self.steps.front() {
            Some(FakeStep::StartThread { expected, .. }) if expected == &spec => {}
            Some(FakeStep::StartThread { .. }) => return Err(self.mismatch("start_thread")),
            _ => return Err(self.unexpected("start_thread")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::StartThread {
                external, result, ..
            }) => {
                if let Some(external) = external {
                    self.backend.create_thread(external)?;
                }
                match result {
                    Ok(handle) => validated_output(
                        handle,
                        "invalid start_thread output",
                        ThreadHandle::validate,
                    ),
                    Err(failure) => Err(convert_failure(failure)),
                }
            }
            _ => Err(self.unexpected("start_thread")),
        }
    }

    async fn start_turn(
        &mut self,
        thread: &ThreadHandle,
        spec: TurnSpec,
    ) -> Result<TurnHandle, RuntimeError> {
        self.calls.push(RuntimeCall::StartTurn {
            thread: thread.clone(),
            spec: spec.clone(),
        });
        thread.validate().map_err(RuntimeError::from_contract)?;
        spec.validate().map_err(RuntimeError::from_contract)?;
        match self.steps.front() {
            Some(FakeStep::StartTurn {
                expected_thread,
                expected,
                ..
            }) if expected_thread == thread && expected == &spec => {}
            Some(FakeStep::StartTurn { .. }) => return Err(self.mismatch("start_turn")),
            _ => return Err(self.unexpected("start_turn")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::StartTurn {
                external, result, ..
            }) => {
                if let Some(external) = external {
                    self.backend
                        .create_turn(thread, external, spec.operation_marker.clone())?;
                }
                match result {
                    Ok(handle) => {
                        validated_output(handle, "invalid start_turn output", TurnHandle::validate)
                    }
                    Err(failure) => Err(convert_failure(failure)),
                }
            }
            _ => Err(self.unexpected("start_turn")),
        }
    }

    async fn read_thread(&mut self, thread: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError> {
        self.calls.push(RuntimeCall::ReadThread {
            thread: thread.clone(),
        });
        thread.validate().map_err(RuntimeError::from_contract)?;
        match self.steps.front() {
            Some(FakeStep::ReadThread { expected, .. }) if expected == thread => {}
            Some(FakeStep::ReadThread { .. }) => return Err(self.mismatch("read_thread")),
            _ => return Err(self.unexpected("read_thread")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::ReadThread { result, .. }) => match result {
                FakeSnapshotResult::Backend => self.backend.snapshot_thread(thread),
                FakeSnapshotResult::Scripted(result) => match result {
                    Ok(snapshot) => validated_output(
                        snapshot,
                        "invalid read_thread output",
                        ThreadSnapshot::validate,
                    ),
                    Err(failure) => Err(convert_failure(failure)),
                },
            },
            _ => Err(self.unexpected("read_thread")),
        }
    }

    async fn resume_thread(
        &mut self,
        thread: &ThreadHandle,
    ) -> Result<ThreadSnapshot, RuntimeError> {
        self.calls.push(RuntimeCall::ResumeThread {
            thread: thread.clone(),
        });
        thread.validate().map_err(RuntimeError::from_contract)?;
        match self.steps.front() {
            Some(FakeStep::ResumeThread { expected, .. }) if expected == thread => {}
            Some(FakeStep::ResumeThread { .. }) => return Err(self.mismatch("resume_thread")),
            _ => return Err(self.unexpected("resume_thread")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::ResumeThread { result, .. }) => match result {
                FakeSnapshotResult::Backend => self.backend.snapshot_thread(thread),
                FakeSnapshotResult::Scripted(result) => match result {
                    Ok(snapshot) => validated_output(
                        snapshot,
                        "invalid resume_thread output",
                        ThreadSnapshot::validate,
                    ),
                    Err(failure) => Err(convert_failure(failure)),
                },
            },
            _ => Err(self.unexpected("resume_thread")),
        }
    }

    async fn next_event(
        &mut self,
        turn: &TurnHandle,
        max_wait: Duration,
    ) -> Result<RuntimeEvent, RuntimeError> {
        self.calls.push(RuntimeCall::NextEvent {
            turn: turn.clone(),
            max_wait,
        });
        turn.validate().map_err(RuntimeError::from_contract)?;
        if max_wait.is_zero() || max_wait > Duration::from_secs(10 * 60) {
            return Err(RuntimeError::protocol(
                "max_wait must be nonzero and at most 10 minutes",
            ));
        }
        match self.steps.front() {
            Some(FakeStep::Event { expected_turn, .. }) if expected_turn == turn => {}
            Some(FakeStep::Event { .. }) => return Err(self.mismatch("next_event")),
            _ => return Err(self.unexpected("next_event")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::Event { result, .. }) => match result {
                Ok(event) => {
                    if self.backend.has_turn(turn) {
                        self.backend.apply_event(&event)?;
                    }
                    validated_output(event, "invalid next_event output", |event| {
                        event.validate()?;
                        validate_event_scope(turn, event)
                    })
                }
                Err(failure) => Err(convert_failure(failure)),
            },
            _ => Err(self.unexpected("next_event")),
        }
    }

    async fn interrupt(&mut self, turn: &TurnHandle) -> Result<(), RuntimeError> {
        self.calls
            .push(RuntimeCall::Interrupt { turn: turn.clone() });
        turn.validate().map_err(RuntimeError::from_contract)?;
        match self.steps.front() {
            Some(FakeStep::Interrupt { expected, .. }) if expected == turn => {}
            Some(FakeStep::Interrupt { .. }) => return Err(self.mismatch("interrupt")),
            _ => return Err(self.unexpected("interrupt")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::Interrupt { result, .. }) => {
                if result.is_ok() {
                    self.backend.interrupt(turn)?;
                }
                result.map_err(convert_failure)
            }
            _ => Err(self.unexpected("interrupt")),
        }
    }

    async fn shutdown_thread(&mut self, thread: ThreadHandle) -> Result<(), RuntimeError> {
        self.calls.push(RuntimeCall::Shutdown {
            thread: thread.clone(),
        });
        thread.validate().map_err(RuntimeError::from_contract)?;
        match self.steps.front() {
            Some(FakeStep::Shutdown { expected, .. }) if expected == &thread => {}
            Some(FakeStep::Shutdown { .. }) => return Err(self.mismatch("shutdown_thread")),
            _ => return Err(self.unexpected("shutdown_thread")),
        }
        match self.steps.pop_front() {
            Some(FakeStep::Shutdown { result, .. }) => {
                if result.is_ok() {
                    self.backend.shutdown(&thread)?;
                }
                result.map_err(convert_failure)
            }
            _ => Err(self.unexpected("shutdown_thread")),
        }
    }
}

struct FakeRuntimeControl {
    backend: FakeBackend,
}

#[async_trait]
impl RuntimeControl for FakeRuntimeControl {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError> {
        RuntimeProvenance::new(
            "scripted-fake",
            env!("CARGO_PKG_VERSION"),
            "in-memory-scripted-backend",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .map_err(RuntimeError::from_contract)
    }

    async fn read_thread(&self, thread: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError> {
        thread.validate().map_err(RuntimeError::from_contract)?;
        self.backend.snapshot_thread(thread)
    }

    async fn interrupt(&self, turn: &TurnHandle) -> Result<(), RuntimeError> {
        turn.validate().map_err(RuntimeError::from_contract)?;
        self.backend.interrupt(turn)
    }

    async fn interrupt_activity(
        &self,
        _activity: &ActivityHandle,
        _purpose: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError> {
        Err(RuntimeError::protocol(
            "scripted fake runtime control does not support activity interrupts",
        ))
    }
}
