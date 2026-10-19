mod common;

use std::error::Error;

use harp_contracts::RuntimeErrorKind;
use harp_runtime::conformance::{
    assert_interrupt_conformance, assert_runtime_conformance, expect_error_kind,
};
use harp_runtime::fake::scenarios;
use harp_runtime::fake::FakeCodexRuntime;
use harp_runtime::{ActivityRuntime, RuntimeError};

fn accepts_runtime_object(_runtime: Box<dyn ActivityRuntime>) {}

#[test]
fn runtime_trait_remains_object_safe() {
    let _accepts: fn(Box<dyn ActivityRuntime>) = accepts_runtime_object;
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

#[tokio::test]
async fn reusable_conformance_uses_activity_handles() {
    let fixture = common::fake_fixture();
    let mut runtime = FakeCodexRuntime::new(scenarios::immediate_success(&fixture)).unwrap();

    assert_runtime_conformance(&mut runtime, &fixture.runtime)
        .await
        .unwrap();
    runtime.assert_exhausted().unwrap();
}

#[tokio::test]
async fn conformance_preserves_activity_start_failure() {
    let fixture = common::fake_fixture();
    let mut runtime = FakeCodexRuntime::new(scenarios::disconnect_after_thread(&fixture)).unwrap();

    let error = assert_runtime_conformance(&mut runtime, &fixture.runtime)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Disconnected);
    runtime.assert_exhausted().unwrap();
}

#[tokio::test]
async fn reusable_interrupt_conformance_observes_quiescent_receipt() {
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
