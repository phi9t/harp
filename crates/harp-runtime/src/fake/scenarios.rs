use harp_contracts::{
    DisconnectedEvent, RuntimeErrorKind, RuntimeEvent, RuntimeFailure, ServerRequestEvent,
    ThreadHandle, ThreadSnapshot, ThreadSpec, TokenUsage, TokenUsageEvent, TurnCompletedEvent,
    TurnHandle, TurnSnapshot, TurnSpec, TurnStartedEvent, TurnStatus,
};
use serde_json::json;

use super::{FakeSnapshotResult, FakeStep};
use crate::conformance::RuntimeFixture;

#[derive(Clone, Debug, PartialEq)]
pub struct FakeFixture {
    pub runtime: RuntimeFixture,
    pub continuation_turn_spec: TurnSpec,
    pub expected_thread: ThreadHandle,
    pub expected_turn: TurnHandle,
    pub expected_continuation_turn: TurnHandle,
    pub expected_snapshot: ThreadSnapshot,
}

impl FakeFixture {
    pub fn thread_spec(&self) -> &ThreadSpec {
        &self.runtime.thread_spec
    }

    pub fn turn_spec(&self) -> &TurnSpec {
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
    steps.push(FakeStep::ReadThread {
        expected: fixture.expected_thread.clone(),
        result: FakeSnapshotResult::Scripted(Ok(fixture.expected_snapshot.clone())),
    });
    steps.push(FakeStep::ResumeThread {
        expected: fixture.expected_thread.clone(),
        result: FakeSnapshotResult::Scripted(Ok(fixture.expected_snapshot.clone())),
    });
    steps.push(FakeStep::Shutdown {
        expected: fixture.expected_thread.clone(),
        result: Ok(()),
    });
    steps
}

pub fn interrupt_success(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.push(FakeStep::Interrupt {
        expected: fixture.expected_turn.clone(),
        result: Ok(()),
    });
    steps.push(FakeStep::ReadThread {
        expected: fixture.expected_thread.clone(),
        result: FakeSnapshotResult::Backend,
    });
    steps.push(FakeStep::Shutdown {
        expected: fixture.expected_thread.clone(),
        result: Ok(()),
    });
    steps
}

pub fn disconnect_after_thread(fixture: &FakeFixture) -> Vec<FakeStep> {
    vec![
        FakeStep::StartThread {
            expected: fixture.runtime.thread_spec.clone(),
            external: Some(fixture.expected_thread.clone()),
            result: Ok(fixture.expected_thread.clone()),
        },
        FakeStep::StartTurn {
            expected_thread: fixture.expected_thread.clone(),
            expected: fixture.runtime.turn_spec.clone(),
            external: None,
            result: Err(failure(
                RuntimeErrorKind::Disconnected,
                "disconnected after thread creation",
                true,
            )),
        },
        FakeStep::Shutdown {
            expected: fixture.expected_thread.clone(),
            result: Ok(()),
        },
    ]
}

pub fn disconnect_after_turn(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.push(FakeStep::Event {
        expected_turn: fixture.expected_turn.clone(),
        result: Ok(RuntimeEvent::Disconnected(DisconnectedEvent {
            reason: "disconnected after turn creation".to_string(),
        })),
    });
    steps
}

pub fn completed_visible_on_read(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.push(FakeStep::ReadThread {
        expected: fixture.expected_thread.clone(),
        result: FakeSnapshotResult::Scripted(Ok(fixture.expected_snapshot.clone())),
    });
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
    steps.push(FakeStep::ResumeThread {
        expected: fixture.expected_thread.clone(),
        result: FakeSnapshotResult::Backend,
    });
    steps.push(FakeStep::StartTurn {
        expected_thread: fixture.expected_thread.clone(),
        expected: fixture.continuation_turn_spec.clone(),
        external: Some(fixture.expected_continuation_turn.clone()),
        result: Ok(fixture.expected_continuation_turn.clone()),
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
    steps.push(FakeStep::Event {
        expected_turn: fixture.expected_turn.clone(),
        result: Ok(RuntimeEvent::ServerRequest(ServerRequestEvent {
            thread_id: fixture.expected_thread.thread_id.clone(),
            turn_id: Some(fixture.expected_turn.turn_id.clone()),
            request: json!({"kind": "approval"}),
        })),
    });
    steps
}

pub fn output_schema_violation(fixture: &FakeFixture) -> Vec<FakeStep> {
    let mut steps = start_steps(fixture);
    steps.push(FakeStep::Event {
        expected_turn: fixture.expected_turn.clone(),
        result: Err(failure(
            RuntimeErrorKind::OutputSchema,
            "runtime output did not match the requested schema",
            false,
        )),
    });
    steps
}

fn start_steps(fixture: &FakeFixture) -> Vec<FakeStep> {
    vec![
        FakeStep::StartThread {
            expected: fixture.runtime.thread_spec.clone(),
            external: Some(fixture.expected_thread.clone()),
            result: Ok(fixture.expected_thread.clone()),
        },
        FakeStep::StartTurn {
            expected_thread: fixture.expected_thread.clone(),
            expected: fixture.runtime.turn_spec.clone(),
            external: Some(fixture.expected_turn.clone()),
            result: Ok(fixture.expected_turn.clone()),
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
        FakeStep::Event {
            expected_turn: turn.clone(),
            result: Ok(RuntimeEvent::TurnStarted(TurnStartedEvent {
                thread_id: turn.thread_id.clone(),
                turn_id: turn.turn_id.clone(),
            })),
        },
        FakeStep::Event {
            expected_turn: turn.clone(),
            result: Ok(RuntimeEvent::TokenUsage(TokenUsageEvent {
                thread_id: turn.thread_id.clone(),
                turn_id: turn.turn_id.clone(),
                usage: TokenUsage {
                    total_tokens,
                    input_tokens: total_tokens,
                    cached_input_tokens: 0,
                    output_tokens: 0,
                    reasoning_output_tokens: 0,
                },
            })),
        },
        FakeStep::Event {
            expected_turn: turn.clone(),
            result: Ok(RuntimeEvent::TurnCompleted(TurnCompletedEvent {
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
            })),
        },
    ]
}

fn failure(kind: RuntimeErrorKind, message: &str, transient: bool) -> RuntimeFailure {
    RuntimeFailure {
        kind,
        message: message.to_string(),
        transient,
    }
}
