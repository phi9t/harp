use harp_contracts::{
    DisconnectedEvent, RuntimeErrorKind, RuntimeEvent, RuntimeFailure, ServerRequestEvent,
    ThreadStartedEvent, TokenUsage, TokenUsageEvent, TurnCompletedEvent, TurnHandle, TurnSnapshot,
    TurnStartedEvent, TurnStatus,
};
use serde_json::json;

use super::FakeStep;
use crate::conformance::RuntimeFixture;
use crate::{ActivitySpec, InterruptPurpose, InterruptReceipt};

#[derive(Clone, Debug, PartialEq)]
pub struct FakeFixture {
    pub runtime: RuntimeFixture,
    pub continuation_turn_spec: harp_contracts::TurnSpec,
    pub expected_thread: harp_contracts::ThreadHandle,
    pub expected_turn: TurnHandle,
    pub expected_continuation_turn: TurnHandle,
}

impl FakeFixture {
    pub fn thread_spec(&self) -> &harp_contracts::ThreadSpec {
        &self.runtime.thread_spec
    }

    pub fn turn_spec(&self) -> &harp_contracts::TurnSpec {
        &self.runtime.turn_spec
    }
}

impl std::ops::Deref for FakeFixture {
    type Target = RuntimeFixture;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

pub fn immediate_success(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.extend(terminal_events(
        fixture,
        &fixture.expected_turn,
        8,
        TurnStatus::Completed,
    ));
    steps
}

pub fn interrupt_success(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.push(FakeStep::ActivityInterrupt {
        expected_logical_session_id: fixture.expected_thread.thread_id.clone(),
        expected_logical_turn_id: fixture.runtime.logical_turn_id.clone(),
        expected_purpose: InterruptPurpose::Cancellation,
        result: Ok(InterruptReceipt {
            process_record_sha256: fixture.runtime.expected_process_record_sha256.clone(),
            quiescent: true,
        }),
    });
    steps
}

pub fn disconnect_after_thread(fixture: &FakeFixture) -> Vec<FakeStep> {
    vec![
        FakeStep::StartLogicalSession {
            expected: fixture.runtime.thread_spec.clone(),
            result: Ok(fixture.expected_thread.clone()),
        },
        FakeStep::StartActivity {
            expected: activity_spec(fixture, &fixture.expected_turn, false),
            process_record_sha256: fixture.runtime.expected_process_record_sha256.clone(),
            external_session_id: fixture.runtime.expected_external_session_id.clone(),
            result: Err(failure(
                RuntimeErrorKind::Disconnected,
                "disconnected after logical session creation",
                true,
            )),
        },
    ]
}

pub fn disconnect_after_turn(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.push(activity_event(
        fixture,
        &fixture.expected_turn,
        RuntimeEvent::Disconnected(DisconnectedEvent {
            reason: "disconnected after activity creation".to_owned(),
        }),
    ));
    steps
}

pub fn interrupted_then_continuation(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.extend(terminal_events(
        fixture,
        &fixture.expected_turn,
        8,
        TurnStatus::Interrupted,
    ));
    steps.push(FakeStep::StartActivity {
        expected: activity_spec(fixture, &fixture.expected_continuation_turn, true),
        process_record_sha256: fixture.runtime.expected_process_record_sha256.clone(),
        external_session_id: fixture.runtime.expected_external_session_id.clone(),
        result: Ok(()),
    });
    steps.extend(terminal_events(
        fixture,
        &fixture.expected_continuation_turn,
        5,
        TurnStatus::Completed,
    ));
    steps
}

pub fn token_budget_crossing(fixture: &FakeFixture, total_tokens: u64) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.extend(terminal_events(
        fixture,
        &fixture.expected_turn,
        total_tokens,
        TurnStatus::Completed,
    ));
    steps
}

pub fn approval_request(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.push(activity_event(
        fixture,
        &fixture.expected_turn,
        RuntimeEvent::ServerRequest(ServerRequestEvent {
            thread_id: fixture.expected_thread.thread_id.clone(),
            turn_id: Some(fixture.expected_turn.turn_id.clone()),
            request: json!({"kind": "approval"}),
        }),
    ));
    steps
}

pub fn output_schema_violation(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.push(FakeStep::ActivityEvent {
        expected_logical_session_id: fixture.expected_thread.thread_id.clone(),
        expected_logical_turn_id: fixture.expected_turn.turn_id.clone(),
        result: Err(failure(
            RuntimeErrorKind::OutputSchema,
            "runtime output did not match the requested schema",
            false,
        )),
    });
    steps
}

pub fn activity_spec(fixture: &FakeFixture, turn: &TurnHandle, continuation: bool) -> ActivitySpec {
    ActivitySpec {
        logical_session_id: fixture.expected_thread.thread_id.clone(),
        logical_turn_id: turn.turn_id.clone(),
        thread_spec: fixture.runtime.thread_spec.clone(),
        turn_spec: if continuation {
            fixture.continuation_turn_spec.clone()
        } else {
            fixture.runtime.turn_spec.clone()
        },
        activity_dir: fixture.runtime.activity_dir.clone(),
        invocation_sha256: fixture.runtime.invocation_sha256.clone(),
        external_session_id: if continuation {
            fixture.runtime.expected_external_session_id.clone()
        } else {
            None
        },
    }
}

fn start_steps(fixture: &FakeFixture) -> Vec<FakeStep> {
    vec![
        FakeStep::StartLogicalSession {
            expected: fixture.runtime.thread_spec.clone(),
            result: Ok(fixture.expected_thread.clone()),
        },
        FakeStep::StartActivity {
            expected: activity_spec(fixture, &fixture.expected_turn, false),
            process_record_sha256: fixture.runtime.expected_process_record_sha256.clone(),
            external_session_id: fixture.runtime.expected_external_session_id.clone(),
            result: Ok(()),
        },
    ]
}

fn terminal_events(
    fixture: &FakeFixture,
    turn: &TurnHandle,
    total_tokens: u64,
    status: TurnStatus,
) -> Vec<FakeStep> {
    vec![
        activity_event(
            fixture,
            turn,
            RuntimeEvent::ThreadStarted(ThreadStartedEvent {
                thread_id: turn.thread_id.clone(),
            }),
        ),
        activity_event(
            fixture,
            turn,
            RuntimeEvent::TurnStarted(TurnStartedEvent {
                thread_id: turn.thread_id.clone(),
                turn_id: turn.turn_id.clone(),
            }),
        ),
        activity_event(
            fixture,
            turn,
            RuntimeEvent::TokenUsage(TokenUsageEvent {
                thread_id: turn.thread_id.clone(),
                turn_id: turn.turn_id.clone(),
                usage: TokenUsage {
                    total_tokens,
                    input_tokens: total_tokens,
                    cached_input_tokens: 0,
                    output_tokens: 0,
                    reasoning_output_tokens: 0,
                },
            }),
        ),
        activity_event(
            fixture,
            turn,
            RuntimeEvent::TurnCompleted(TurnCompletedEvent {
                thread_id: turn.thread_id.clone(),
                turn: TurnSnapshot {
                    turn_id: turn.turn_id.clone(),
                    operation_marker: Some(if turn == &fixture.expected_turn {
                        fixture.runtime.turn_spec.operation_marker.clone()
                    } else {
                        fixture.continuation_turn_spec.operation_marker.clone()
                    }),
                    status,
                    final_agent_message: if status == fixture.runtime.expected_status {
                        fixture.runtime.expected_final_message.clone()
                    } else {
                        None
                    },
                },
            }),
        ),
    ]
}

fn activity_event(fixture: &FakeFixture, turn: &TurnHandle, event: RuntimeEvent) -> FakeStep {
    FakeStep::ActivityEvent {
        expected_logical_session_id: fixture.expected_thread.thread_id.clone(),
        expected_logical_turn_id: turn.turn_id.clone(),
        result: Ok(event),
    }
}

fn failure(kind: RuntimeErrorKind, message: &str, transient: bool) -> RuntimeFailure {
    RuntimeFailure {
        kind,
        message: message.to_owned(),
        transient,
    }
}
