mod common;

use std::error::Error;

use harp_contracts::{RuntimeErrorKind, RuntimeFailure};
use harp_runtime::conformance::{
    assert_interrupt_conformance, assert_runtime_conformance, expect_error_kind,
};
use harp_runtime::fake::scenarios;
use harp_runtime::fake::{FakeCodexRuntime, FakeExternalEffect, FakeStep};
use harp_runtime::{CodexRuntime, RuntimeError};

fn accepts_runtime_object(_runtime: Box<dyn CodexRuntime>) {}

#[test]
fn runtime_trait_remains_object_safe() {
    let _accepts: fn(Box<dyn CodexRuntime>) = accepts_runtime_object;
}

#[test]
fn normalized_error_constructors_preserve_fields() {
    let error =
        RuntimeError::try_new(RuntimeErrorKind::Transport, "connection reset", true).unwrap();
    assert_eq!(error.kind(), RuntimeErrorKind::Transport);
    assert_eq!(error.message(), "connection reset");
    assert!(error.is_transient());
    assert!(Error::source(&error).is_none());
}

fn failure(kind: RuntimeErrorKind, message: &str, transient: bool) -> RuntimeFailure {
    RuntimeFailure {
        kind,
        message: message.to_string(),
        transient,
    }
}

#[tokio::test]
async fn reusable_conformance_uses_dynamic_handles_and_resumes_history() {
    let fixture = common::fake_fixture();
    let backend = harp_runtime::fake::FakeBackend::default();
    let mut runtime =
        FakeCodexRuntime::with_backend(scenarios::immediate_success(&fixture), backend.clone())
            .unwrap();

    assert_runtime_conformance(&mut runtime, &fixture.runtime)
        .await
        .unwrap();
    runtime.assert_exhausted().unwrap();
    assert_eq!(
        backend.effects(),
        vec![
            FakeExternalEffect::CreatedThread {
                thread: fixture.expected_thread.clone(),
            },
            FakeExternalEffect::CreatedTurn {
                turn: fixture.expected_turn.clone(),
                operation_marker: fixture.runtime.turn_spec.operation_marker.clone(),
            },
            FakeExternalEffect::Shutdown {
                thread: fixture.expected_thread,
            },
        ]
    );
}

#[tokio::test]
async fn conformance_always_shuts_down_after_intermediate_failure() {
    let fixture = common::fake_fixture();
    let backend = harp_runtime::fake::FakeBackend::default();
    let mut runtime = FakeCodexRuntime::with_backend(
        scenarios::disconnect_after_thread(&fixture),
        backend.clone(),
    )
    .unwrap();

    let error = assert_runtime_conformance(&mut runtime, &fixture.runtime)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Disconnected);
    runtime.assert_exhausted().unwrap();
    assert!(backend.effects().contains(&FakeExternalEffect::Shutdown {
        thread: fixture.expected_thread,
    }));
}

#[tokio::test]
async fn conformance_preserves_primary_and_cleanup_failure() {
    let fixture = common::fake_fixture();
    let mut runtime = FakeCodexRuntime::new(vec![
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
                RuntimeErrorKind::OutputSchema,
                "primary failure",
                false,
            )),
        },
        FakeStep::Shutdown {
            expected: fixture.expected_thread,
            result: Err(failure(
                RuntimeErrorKind::Transport,
                "cleanup failure",
                true,
            )),
        },
    ])
    .unwrap();

    let error = assert_runtime_conformance(&mut runtime, &fixture.runtime)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::OutputSchema);
    assert!(error.message().contains("shutdown failed"));
    assert!(Error::source(&error).is_some());
    runtime.assert_exhausted().unwrap();
}

#[tokio::test]
async fn reusable_interrupt_conformance_observes_interrupted_snapshot() {
    let fixture = common::fake_fixture();
    let mut runtime = FakeCodexRuntime::new(scenarios::interrupt_success(&fixture)).unwrap();

    assert_interrupt_conformance(&mut runtime, &fixture.runtime)
        .await
        .unwrap();
    runtime.assert_exhausted().unwrap();
}

#[test]
fn reusable_error_kind_validator_accepts_and_rejects_kinds() {
    expect_error_kind::<()>(
        Err(harp_runtime::RuntimeError::try_new(
            RuntimeErrorKind::ApprovalRequired,
            "approval",
            false,
        )
        .unwrap()),
        RuntimeErrorKind::ApprovalRequired,
    )
    .unwrap();

    let error = expect_error_kind::<()>(
        Err(
            harp_runtime::RuntimeError::try_new(RuntimeErrorKind::OutputSchema, "schema", false)
                .unwrap(),
        ),
        RuntimeErrorKind::ApprovalRequired,
    )
    .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Protocol);
}
