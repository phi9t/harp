use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use harp_contracts::{
    ContractError, LaggedEvent, OperationId, RuntimeErrorKind, RuntimeEvent, ServerRequestEvent,
    ThreadHandle, ThreadId, ThreadSpec, TurnCompletedEvent, TurnId, TurnSnapshot, TurnStartedEvent,
    TurnStatus,
};
use harp_runtime::fake::{FakeCodexRuntime, FakeStep};
use harp_runtime::{
    collect_until_terminal, ActivityHandle, ActivityRuntime, ActivitySpec, CollectionLimits,
    InterruptPurpose, InterruptReceipt, RuntimeError,
};
use serde_json::json;

const MARKER: &str = "018f22e2-7c3b-7def-8123-456789abcdef";

fn activity(thread: &str, turn: &str) -> ActivityHandle {
    ActivityHandle {
        logical_session_id: ThreadId::from_str(thread).unwrap(),
        logical_turn_id: TurnId::from_str(turn).unwrap(),
        process_record_sha256: "a".repeat(64),
        external_session_id: None,
    }
}

fn completed(activity: &ActivityHandle, message: &str) -> RuntimeEvent {
    RuntimeEvent::TurnCompleted(TurnCompletedEvent {
        thread_id: activity.logical_session_id.clone(),
        turn: TurnSnapshot {
            turn_id: activity.logical_turn_id.clone(),
            operation_marker: Some(OperationId::from_str(MARKER).unwrap()),
            status: TurnStatus::Completed,
            final_agent_message: Some(message.to_owned()),
        },
    })
}

fn activity_event(activity: &ActivityHandle, result: RuntimeEvent) -> FakeStep {
    FakeStep::ActivityEvent {
        expected_logical_session_id: activity.logical_session_id.clone(),
        expected_logical_turn_id: activity.logical_turn_id.clone(),
        result: Ok(result),
    }
}

fn limits() -> CollectionLimits {
    CollectionLimits {
        max_events: 16,
        max_serialized_bytes: 1024 * 1024,
        max_total_wait: Duration::from_secs(2),
        per_event_wait: Duration::from_millis(100),
    }
}

#[tokio::test]
async fn collector_accepts_thread_started_for_the_activity_session() {
    let requested = activity("thread-1", "turn-1");
    let thread_started = RuntimeEvent::ThreadStarted(harp_contracts::ThreadStartedEvent {
        thread_id: requested.logical_session_id.clone(),
    });
    let terminal = completed(&requested, "done");
    let mut runtime = FakeCodexRuntime::new(vec![
        activity_event(&requested, thread_started.clone()),
        activity_event(&requested, terminal.clone()),
    ])
    .unwrap();

    let events = collect_until_terminal(&mut runtime, &requested, &limits())
        .await
        .unwrap();

    assert_eq!(events, vec![thread_started, terminal]);
    runtime.assert_exhausted().unwrap();
}

#[tokio::test]
async fn wrong_activity_scope_is_protocol_and_does_not_consume_event() {
    let requested = activity("thread-1", "turn-1");
    let routed = activity("thread-1", "turn-2");
    let mut runtime =
        FakeCodexRuntime::new(vec![activity_event(&routed, completed(&routed, "done"))]).unwrap();

    let error = runtime
        .next_event(&requested, Duration::from_millis(50))
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
    assert_eq!(runtime.remaining_steps(), 1);

    let events = collect_until_terminal(&mut runtime, &routed, &limits())
        .await
        .unwrap();
    assert_eq!(events, vec![completed(&routed, "done")]);
    runtime.assert_exhausted().unwrap();
}

#[tokio::test]
async fn server_request_for_another_activity_is_not_routed_to_collector() {
    let requested = activity("thread-1", "turn-1");
    let routed = activity("thread-1", "turn-2");
    let mut runtime = FakeCodexRuntime::new(vec![activity_event(
        &routed,
        RuntimeEvent::ServerRequest(ServerRequestEvent {
            thread_id: routed.logical_session_id.clone(),
            turn_id: Some(routed.logical_turn_id.clone()),
            request: json!({"kind": "approval"}),
        }),
    )])
    .unwrap();

    let error = collect_until_terminal(&mut runtime, &requested, &limits())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
    assert_eq!(runtime.remaining_steps(), 1);

    let error = collect_until_terminal(&mut runtime, &routed, &limits())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::ApprovalRequired);
    runtime.assert_exhausted().unwrap();
}

#[tokio::test]
async fn lagged_event_fails_closed_as_transient_transport() {
    let requested = activity("thread-1", "turn-1");
    let mut runtime = FakeCodexRuntime::new(vec![activity_event(
        &requested,
        RuntimeEvent::Lagged(LaggedEvent { dropped_events: 3 }),
    )])
    .unwrap();

    let error = collect_until_terminal(&mut runtime, &requested, &limits())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Transport);
    assert!(error.is_transient());
}

#[tokio::test]
async fn collection_limits_reject_invalid_bounds_without_polling() {
    let requested = activity("thread-1", "turn-1");
    let invalid = [
        CollectionLimits {
            max_events: 0,
            ..limits()
        },
        CollectionLimits {
            max_events: 100_001,
            ..limits()
        },
        CollectionLimits {
            max_serialized_bytes: 0,
            ..limits()
        },
        CollectionLimits {
            max_serialized_bytes: 64 * 1024 * 1024 + 1,
            ..limits()
        },
        CollectionLimits {
            max_total_wait: Duration::ZERO,
            ..limits()
        },
        CollectionLimits {
            max_total_wait: Duration::from_secs(24 * 60 * 60 + 1),
            ..limits()
        },
        CollectionLimits {
            per_event_wait: Duration::ZERO,
            ..limits()
        },
        CollectionLimits {
            per_event_wait: Duration::from_secs(601),
            max_total_wait: Duration::from_secs(601),
            ..limits()
        },
        CollectionLimits {
            per_event_wait: Duration::from_secs(3),
            max_total_wait: Duration::from_secs(2),
            ..limits()
        },
    ];

    for limits in invalid {
        let mut runtime = FakeCodexRuntime::new(Vec::new()).unwrap();
        let error = collect_until_terminal(&mut runtime, &requested, &limits)
            .await
            .unwrap_err();
        assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
        assert!(runtime.calls().is_empty());
    }
}

#[tokio::test]
async fn collector_enforces_serialized_byte_limit() {
    let requested = activity("thread-1", "turn-1");
    let mut runtime = FakeCodexRuntime::new(vec![activity_event(
        &requested,
        completed(&requested, &"x".repeat(128)),
    )])
    .unwrap();
    let limits = CollectionLimits {
        max_serialized_bytes: 1,
        ..limits()
    };

    let error = collect_until_terminal(&mut runtime, &requested, &limits)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
}

#[tokio::test]
async fn fake_rejects_invalid_per_event_wait_without_consuming_step() {
    let requested = activity("thread-1", "turn-1");
    let mut runtime = FakeCodexRuntime::new(vec![activity_event(
        &requested,
        completed(&requested, "done"),
    )])
    .unwrap();

    for wait in [Duration::ZERO, Duration::from_secs(601)] {
        let error = runtime.next_event(&requested, wait).await.unwrap_err();
        assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
        assert_eq!(runtime.remaining_steps(), 1);
    }
}

struct EventRuntime {
    event: Option<RuntimeEvent>,
    delay: Duration,
}

fn unused() -> RuntimeError {
    RuntimeError::try_new(RuntimeErrorKind::Protocol, "unused test method", false).unwrap()
}

#[async_trait]
impl ActivityRuntime for EventRuntime {
    fn provenance(&self) -> Result<harp_runtime::RuntimeProvenance, RuntimeError> {
        harp_runtime::RuntimeProvenance::new(
            "event-test",
            env!("CARGO_PKG_VERSION"),
            "event-runtime",
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .map_err(RuntimeError::from_contract)
    }

    fn control_handle(&self) -> Result<Arc<dyn harp_runtime::RuntimeControl>, RuntimeError> {
        Err(unused())
    }

    async fn start_logical_session(&mut self, _: ThreadSpec) -> Result<ThreadHandle, RuntimeError> {
        Err(unused())
    }

    async fn start_activity(&mut self, _: ActivitySpec) -> Result<ActivityHandle, RuntimeError> {
        Err(unused())
    }

    async fn next_event(
        &mut self,
        _: &ActivityHandle,
        _: Duration,
    ) -> Result<RuntimeEvent, RuntimeError> {
        tokio::time::sleep(self.delay).await;
        self.event.take().ok_or_else(unused)
    }

    async fn interrupt(
        &mut self,
        _: &ActivityHandle,
        _: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError> {
        Err(unused())
    }
}

#[tokio::test]
async fn collector_rejects_unrelated_event_from_adapter() {
    let requested = activity("thread-1", "turn-1");
    let unrelated = activity("thread-1", "turn-2");
    let mut runtime = EventRuntime {
        event: Some(RuntimeEvent::TurnStarted(TurnStartedEvent {
            thread_id: unrelated.logical_session_id,
            turn_id: unrelated.logical_turn_id,
        })),
        delay: Duration::ZERO,
    };

    let error = collect_until_terminal(&mut runtime, &requested, &limits())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
}

#[tokio::test]
async fn collector_enforces_per_event_timeout() {
    let requested = activity("thread-1", "turn-1");
    let mut runtime = EventRuntime {
        event: Some(completed(&requested, "late")),
        delay: Duration::from_millis(50),
    };
    let limits = CollectionLimits {
        max_total_wait: Duration::from_millis(100),
        per_event_wait: Duration::from_millis(10),
        ..limits()
    };

    let error = collect_until_terminal(&mut runtime, &requested, &limits)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Transport);
    assert!(error.is_transient());
    assert!(error.message().contains("timeout"));
}

#[tokio::test]
async fn collector_enforces_total_timeout_across_events() {
    let requested = activity("thread-1", "turn-1");
    let mut runtime = EventRuntime {
        event: Some(RuntimeEvent::TurnStarted(TurnStartedEvent {
            thread_id: requested.logical_session_id.clone(),
            turn_id: requested.logical_turn_id.clone(),
        })),
        delay: Duration::from_millis(15),
    };
    let limits = CollectionLimits {
        max_total_wait: Duration::from_millis(20),
        per_event_wait: Duration::from_millis(20),
        ..limits()
    };

    let error = collect_until_terminal(&mut runtime, &requested, &limits)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Transport);
    assert!(error.is_transient());
    assert!(error.message().contains("timeout"));
}

#[test]
fn contract_errors_are_retained_as_typed_sources() {
    let contract_error: ContractError = ThreadId::from_str("").unwrap_err();
    let error = RuntimeError::from_contract(contract_error);

    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
    assert!(Error::source(&error)
        .and_then(|source| source.downcast_ref::<ContractError>())
        .is_some());
}
