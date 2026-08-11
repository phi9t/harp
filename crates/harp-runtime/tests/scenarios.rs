mod common;

use std::time::Duration;

use harp_contracts::{RuntimeErrorKind, RuntimeEvent, TurnStatus};
use harp_runtime::fake::scenarios;
use harp_runtime::fake::{FakeCodexRuntime, RuntimeCall};
use harp_runtime::{
    collect_until_terminal, ActivityHandle, ActivityRuntime, CollectionLimits, InterruptPurpose,
};

fn limits() -> CollectionLimits {
    CollectionLimits {
        max_events: 16,
        max_serialized_bytes: 1024 * 1024,
        max_total_wait: Duration::from_secs(2),
        per_event_wait: Duration::from_millis(100),
    }
}

async fn start_initial_activity(
    runtime: &mut FakeCodexRuntime,
    fixture: &scenarios::FakeFixture,
) -> ActivityHandle {
    runtime
        .start_logical_session(fixture.runtime.thread_spec.clone())
        .await
        .unwrap();
    runtime
        .start_activity(scenarios::activity_spec(
            fixture,
            &fixture.expected_turn,
            false,
        ))
        .await
        .unwrap()
}

#[tokio::test]
async fn disconnect_scenarios_preserve_normalized_failure_kinds() {
    let fixture = common::fake_fixture();
    let mut after_session =
        FakeCodexRuntime::new(scenarios::disconnect_after_thread(&fixture)).unwrap();
    after_session
        .start_logical_session(fixture.runtime.thread_spec.clone())
        .await
        .unwrap();
    let error = after_session
        .start_activity(scenarios::activity_spec(
            &fixture,
            &fixture.expected_turn,
            false,
        ))
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Disconnected);
    after_session.assert_exhausted().unwrap();

    let mut after_activity =
        FakeCodexRuntime::new(scenarios::disconnect_after_turn(&fixture)).unwrap();
    let activity = start_initial_activity(&mut after_activity, &fixture).await;
    let error = collect_until_terminal(&mut after_activity, &activity, &limits())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Disconnected);
}

#[tokio::test]
async fn initial_activity_starts_once() {
    let fixture = common::fake_fixture();
    let mut runtime = FakeCodexRuntime::new(scenarios::immediate_success(&fixture)).unwrap();
    let activity = start_initial_activity(&mut runtime, &fixture).await;
    collect_until_terminal(&mut runtime, &activity, &limits())
        .await
        .unwrap();
    assert_eq!(
        runtime
            .calls()
            .iter()
            .filter(|call| matches!(call, RuntimeCall::StartActivity { .. }))
            .count(),
        1
    );
}

#[tokio::test]
async fn control_handle_consumes_the_scripted_activity_interrupt() {
    let fixture = common::fake_fixture();
    let mut runtime = FakeCodexRuntime::new(scenarios::interrupt_success(&fixture)).unwrap();
    let control = runtime.control_handle().unwrap();
    let activity = start_initial_activity(&mut runtime, &fixture).await;

    let error = control
        .interrupt_activity(&activity, InterruptPurpose::Budget)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
    assert_eq!(runtime.remaining_steps(), 1);

    let receipt = control
        .interrupt_activity(&activity, InterruptPurpose::Cancellation)
        .await
        .unwrap();
    assert!(receipt.quiescent);
    assert_eq!(
        receipt.process_record_sha256,
        activity.process_record_sha256
    );
    assert!(runtime.calls().iter().any(|call| {
        matches!(
            call,
            RuntimeCall::ActivityInterrupt {
                purpose: InterruptPurpose::Cancellation,
                ..
            }
        )
    }));
    runtime.assert_exhausted().unwrap();
}

#[tokio::test]
async fn interrupted_continuation_uses_two_distinct_markers() {
    let fixture = common::fake_fixture();
    let mut runtime =
        FakeCodexRuntime::new(scenarios::interrupted_then_continuation(&fixture)).unwrap();
    let initial = start_initial_activity(&mut runtime, &fixture).await;
    let events = collect_until_terminal(&mut runtime, &initial, &limits())
        .await
        .unwrap();
    assert!(matches!(
        events.last(),
        Some(RuntimeEvent::TurnCompleted(event))
            if event.turn.status == TurnStatus::Interrupted
    ));
    let continuation = runtime
        .start_activity(scenarios::activity_spec(
            &fixture,
            &fixture.expected_continuation_turn,
            true,
        ))
        .await
        .unwrap();
    collect_until_terminal(&mut runtime, &continuation, &limits())
        .await
        .unwrap();

    let markers = runtime
        .calls()
        .into_iter()
        .filter_map(|call| match call {
            RuntimeCall::StartActivity { spec } => Some(spec.turn_spec.operation_marker),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(markers.len(), 2);
    assert_ne!(markers[0], markers[1]);
}

#[tokio::test]
async fn token_approval_and_schema_scenarios_remain_strict() {
    let fixture = common::fake_fixture();
    let mut token =
        FakeCodexRuntime::new(scenarios::token_budget_crossing(&fixture, 50_003)).unwrap();
    let activity = start_initial_activity(&mut token, &fixture).await;
    let events = collect_until_terminal(&mut token, &activity, &limits())
        .await
        .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        RuntimeEvent::TokenUsage(event) if event.usage.total_tokens == 50_003
    )));

    for (steps, expected) in [
        (
            scenarios::approval_request(&fixture),
            RuntimeErrorKind::ApprovalRequired,
        ),
        (
            scenarios::output_schema_violation(&fixture),
            RuntimeErrorKind::OutputSchema,
        ),
    ] {
        let mut runtime = FakeCodexRuntime::new(steps).unwrap();
        let activity = start_initial_activity(&mut runtime, &fixture).await;
        let error = collect_until_terminal(&mut runtime, &activity, &limits())
            .await
            .unwrap_err();
        assert_eq!(error.kind(), expected);
    }
}
