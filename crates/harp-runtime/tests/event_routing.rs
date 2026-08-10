use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use harp_contracts::{
    ContractError, LaggedEvent, OperationId, RuntimeErrorKind, RuntimeEvent, ServerRequestEvent,
    ThreadHandle, ThreadId, ThreadSnapshot, ThreadSpec, TurnCompletedEvent, TurnHandle, TurnId,
    TurnSnapshot, TurnSpec, TurnStartedEvent, TurnStatus,
};
use harp_runtime::fake::{FakeCodexRuntime, FakeStep};
use harp_runtime::{collect_until_terminal, CodexRuntime, CollectionLimits, RuntimeError};
use serde_json::json;

const MARKER: &str = "018f22e2-7c3b-7def-8123-456789abcdef";

fn turn(thread: &str, turn: &str) -> TurnHandle {
    TurnHandle {
        thread_id: ThreadId::from_str(thread).unwrap(),
        turn_id: TurnId::from_str(turn).unwrap(),
    }
}

fn completed(turn: &TurnHandle, message: &str) -> RuntimeEvent {
    RuntimeEvent::TurnCompleted(TurnCompletedEvent {
        thread_id: turn.thread_id.clone(),
        turn: TurnSnapshot {
            turn_id: turn.turn_id.clone(),
            operation_marker: Some(OperationId::from_str(MARKER).unwrap()),
            status: TurnStatus::Completed,
            final_agent_message: Some(message.to_string()),
        },
    })
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
async fn wrong_turn_scope_is_protocol_and_does_not_consume_event() {
    let requested = turn("thread-1", "turn-1");
    let routed = turn("thread-1", "turn-2");
    let mut runtime = FakeCodexRuntime::new(vec![FakeStep::Event {
        expected_turn: routed.clone(),
        result: Ok(completed(&routed, "done")),
    }])
    .unwrap();

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
async fn server_request_for_another_turn_is_not_routed_to_collector() {
    let requested = turn("thread-1", "turn-1");
    let routed = turn("thread-1", "turn-2");
    let mut runtime = FakeCodexRuntime::new(vec![FakeStep::Event {
        expected_turn: routed.clone(),
        result: Ok(RuntimeEvent::ServerRequest(ServerRequestEvent {
            thread_id: routed.thread_id.clone(),
            turn_id: Some(routed.turn_id.clone()),
            request: json!({"kind": "approval"}),
        })),
    }])
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
    let requested = turn("thread-1", "turn-1");
    let mut runtime = FakeCodexRuntime::new(vec![FakeStep::Event {
        expected_turn: requested.clone(),
        result: Ok(RuntimeEvent::Lagged(LaggedEvent { dropped_events: 3 })),
    }])
    .unwrap();

    let error = collect_until_terminal(&mut runtime, &requested, &limits())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Transport);
    assert!(error.is_transient());
}

#[tokio::test]
async fn collection_limits_reject_invalid_bounds_without_polling() {
    let requested = turn("thread-1", "turn-1");
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
    let requested = turn("thread-1", "turn-1");
    let mut runtime = FakeCodexRuntime::new(vec![FakeStep::Event {
        expected_turn: requested.clone(),
        result: Ok(completed(&requested, &"x".repeat(128))),
    }])
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
    let requested = turn("thread-1", "turn-1");
    let mut runtime = FakeCodexRuntime::new(vec![FakeStep::Event {
        expected_turn: requested.clone(),
        result: Ok(completed(&requested, "done")),
    }])
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
impl CodexRuntime for EventRuntime {
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

    async fn start_thread(&mut self, _: ThreadSpec) -> Result<ThreadHandle, RuntimeError> {
        Err(unused())
    }

    async fn start_turn(
        &mut self,
        _: &ThreadHandle,
        _: TurnSpec,
    ) -> Result<TurnHandle, RuntimeError> {
        Err(unused())
    }

    async fn read_thread(&mut self, _: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError> {
        Err(unused())
    }

    async fn resume_thread(&mut self, _: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError> {
        Err(unused())
    }

    async fn next_event(
        &mut self,
        _: &TurnHandle,
        _: Duration,
    ) -> Result<RuntimeEvent, RuntimeError> {
        tokio::time::sleep(self.delay).await;
        self.event.take().ok_or_else(unused)
    }

    async fn interrupt(&mut self, _: &TurnHandle) -> Result<(), RuntimeError> {
        Err(unused())
    }

    async fn shutdown_thread(&mut self, _: ThreadHandle) -> Result<(), RuntimeError> {
        Err(unused())
    }
}

#[tokio::test]
async fn collector_rejects_unrelated_event_from_adapter() {
    let requested = turn("thread-1", "turn-1");
    let unrelated = turn("thread-1", "turn-2");
    let mut runtime = EventRuntime {
        event: Some(RuntimeEvent::TurnStarted(TurnStartedEvent {
            thread_id: unrelated.thread_id,
            turn_id: unrelated.turn_id,
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
    let requested = turn("thread-1", "turn-1");
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
    let requested = turn("thread-1", "turn-1");
    let mut runtime = EventRuntime {
        event: Some(RuntimeEvent::TurnStarted(TurnStartedEvent {
            thread_id: requested.thread_id.clone(),
            turn_id: requested.turn_id.clone(),
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
