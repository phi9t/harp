mod common;

use std::time::Duration;

use harp_contracts::{RuntimeErrorKind, RuntimeEvent, TurnStatus};
use harp_runtime::fake::scenarios;
use harp_runtime::fake::{FakeCodexRuntime, RuntimeCall};
use harp_runtime::{collect_until_terminal, CodexRuntime, CollectionLimits};

fn limits() -> CollectionLimits {
    CollectionLimits {
        max_events: 16,
        max_serialized_bytes: 1024 * 1024,
        max_total_wait: Duration::from_secs(2),
        per_event_wait: Duration::from_millis(100),
    }
}

#[tokio::test]
async fn disconnect_scenarios_preserve_normalized_failure_kinds() {
    let fixture = common::fake_fixture();
    let mut after_thread =
        FakeCodexRuntime::new(scenarios::disconnect_after_thread(&fixture)).unwrap();
    let thread = after_thread
        .start_thread(fixture.runtime.thread_spec.clone())
        .await
        .unwrap();
    let error = after_thread
        .start_turn(&thread, fixture.runtime.turn_spec.clone())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Disconnected);
    after_thread.shutdown_thread(thread).await.unwrap();
    after_thread.assert_exhausted().unwrap();

    let mut after_turn = FakeCodexRuntime::new(scenarios::disconnect_after_turn(&fixture)).unwrap();
    let thread = after_turn
        .start_thread(fixture.runtime.thread_spec.clone())
        .await
        .unwrap();
    let turn = after_turn
        .start_turn(&thread, fixture.runtime.turn_spec.clone())
        .await
        .unwrap();
    let error = collect_until_terminal(&mut after_turn, &turn, &limits())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Disconnected);
}

#[tokio::test]
async fn completed_on_read_has_one_semantic_turn() {
    let fixture = common::fake_fixture();
    let mut runtime =
        FakeCodexRuntime::new(scenarios::completed_visible_on_read(&fixture)).unwrap();
    let thread = runtime
        .start_thread(fixture.runtime.thread_spec.clone())
        .await
        .unwrap();
    runtime
        .start_turn(&thread, fixture.runtime.turn_spec.clone())
        .await
        .unwrap();
    let snapshot = runtime.read_thread(&thread).await.unwrap();
    assert_eq!(snapshot, fixture.expected_snapshot);
    assert_eq!(
        runtime
            .calls()
            .iter()
            .filter(|call| matches!(call, RuntimeCall::StartTurn { .. }))
            .count(),
        1
    );
}

#[tokio::test]
async fn interrupted_continuation_uses_two_distinct_markers() {
    let fixture = common::fake_fixture();
    let mut runtime =
        FakeCodexRuntime::new(scenarios::interrupted_then_continuation(&fixture)).unwrap();
    let thread = runtime
        .start_thread(fixture.runtime.thread_spec.clone())
        .await
        .unwrap();
    let initial = runtime
        .start_turn(&thread, fixture.runtime.turn_spec.clone())
        .await
        .unwrap();
    let events = collect_until_terminal(&mut runtime, &initial, &limits())
        .await
        .unwrap();
    assert!(matches!(
        events.last(),
        Some(RuntimeEvent::TurnCompleted(event))
            if event.turn.status == TurnStatus::Interrupted
    ));
    runtime.resume_thread(&thread).await.unwrap();
    let continuation = runtime
        .start_turn(&thread, fixture.continuation_turn_spec.clone())
        .await
        .unwrap();
    collect_until_terminal(&mut runtime, &continuation, &limits())
        .await
        .unwrap();

    let markers = runtime
        .calls()
        .iter()
        .filter_map(|call| match call {
            RuntimeCall::StartTurn { spec, .. } => Some(&spec.operation_marker),
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
    let thread = token
        .start_thread(fixture.runtime.thread_spec.clone())
        .await
        .unwrap();
    let turn = token
        .start_turn(&thread, fixture.runtime.turn_spec.clone())
        .await
        .unwrap();
    let events = collect_until_terminal(&mut token, &turn, &limits())
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
        let thread = runtime
            .start_thread(fixture.runtime.thread_spec.clone())
            .await
            .unwrap();
        let turn = runtime
            .start_turn(&thread, fixture.runtime.turn_spec.clone())
            .await
            .unwrap();
        let error = collect_until_terminal(&mut runtime, &turn, &limits())
            .await
            .unwrap_err();
        assert_eq!(error.kind(), expected);
    }
}
