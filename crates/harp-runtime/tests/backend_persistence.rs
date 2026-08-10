use std::str::FromStr;

use harp_contracts::{
    OperationId, RuntimeErrorKind, RuntimeFailure, ThreadHandle, ThreadId, ThreadSpec, TurnHandle,
    TurnId, TurnSpec, TurnStatus,
};
use harp_runtime::fake::{
    FakeBackend, FakeCodexRuntime, FakeExternalEffect, FakeSnapshotResult, FakeStep,
};
use harp_runtime::CodexRuntime;
use serde_json::json;

const MARKER: &str = "018f22e2-7c3b-7def-8123-456789abcdef";

fn thread_spec() -> ThreadSpec {
    ThreadSpec {
        base_instructions: "Base".to_string(),
        developer_instructions: "Developer".to_string(),
        cwd: "/private/work".to_string(),
        runtime_workspace_roots: vec!["/private/work".to_string()],
        workspace_authority: None,
        approval_policy: "never".to_string(),
        sandbox_mode: "workspace-write".to_string(),
        model: "codex".to_string(),
        reasoning_effort: Some("high".to_string()),
        ephemeral: false,
    }
}

fn turn_spec() -> TurnSpec {
    TurnSpec {
        instruction: "Analyze".to_string(),
        operation_marker: OperationId::from_str(MARKER).unwrap(),
        output_schema: json!({"type": "object"}),
        model: Some("codex".to_string()),
        reasoning_effort: Some("high".to_string()),
    }
}

fn thread() -> ThreadHandle {
    ThreadHandle {
        thread_id: ThreadId::from_str("thread-persistent").unwrap(),
    }
}

fn turn() -> TurnHandle {
    TurnHandle {
        thread_id: thread().thread_id,
        turn_id: TurnId::from_str("turn-persistent").unwrap(),
    }
}

fn disconnected() -> RuntimeFailure {
    RuntimeFailure {
        kind: RuntimeErrorKind::Disconnected,
        message: "response lost".to_string(),
        transient: true,
    }
}

#[tokio::test]
async fn lost_start_response_survives_new_fake_and_reconciles_by_marker() {
    let backend = FakeBackend::default();
    let mut first = FakeCodexRuntime::with_backend(
        vec![
            FakeStep::StartThread {
                expected: thread_spec(),
                external: Some(thread()),
                result: Ok(thread()),
            },
            FakeStep::StartTurn {
                expected_thread: thread(),
                expected: turn_spec(),
                external: Some(turn()),
                result: Err(disconnected()),
            },
        ],
        backend.clone(),
    )
    .unwrap();

    let created_thread = first.start_thread(thread_spec()).await.unwrap();
    let error = first
        .start_turn(&created_thread, turn_spec())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), RuntimeErrorKind::Disconnected);
    first.assert_exhausted().unwrap();
    drop(first);

    let mut second = FakeCodexRuntime::with_backend(
        vec![FakeStep::ReadThread {
            expected: thread(),
            result: FakeSnapshotResult::Backend,
        }],
        backend.clone(),
    )
    .unwrap();
    let snapshot = second.read_thread(&thread()).await.unwrap();
    let reconciled = snapshot
        .turn_by_operation_marker(&OperationId::from_str(MARKER).unwrap())
        .unwrap()
        .expect("external turn persisted");
    assert_eq!(reconciled.turn_id, turn().turn_id);
    assert_eq!(reconciled.status, TurnStatus::InProgress);
    assert_eq!(
        backend
            .effects()
            .iter()
            .filter(|effect| matches!(effect, FakeExternalEffect::CreatedTurn { .. }))
            .count(),
        1
    );
    second.assert_exhausted().unwrap();
}

#[tokio::test]
async fn successful_interrupt_and_shutdown_update_backend_and_effect_history() {
    let backend = FakeBackend::default();
    let mut runtime = FakeCodexRuntime::with_backend(
        vec![
            FakeStep::StartThread {
                expected: thread_spec(),
                external: Some(thread()),
                result: Ok(thread()),
            },
            FakeStep::StartTurn {
                expected_thread: thread(),
                expected: turn_spec(),
                external: Some(turn()),
                result: Ok(turn()),
            },
            FakeStep::Interrupt {
                expected: turn(),
                result: Ok(()),
            },
            FakeStep::ReadThread {
                expected: thread(),
                result: FakeSnapshotResult::Backend,
            },
            FakeStep::Shutdown {
                expected: thread(),
                result: Ok(()),
            },
        ],
        backend.clone(),
    )
    .unwrap();

    runtime.start_thread(thread_spec()).await.unwrap();
    runtime.start_turn(&thread(), turn_spec()).await.unwrap();
    runtime.interrupt(&turn()).await.unwrap();
    let snapshot = runtime.read_thread(&thread()).await.unwrap();
    assert_eq!(snapshot.turns[0].status, TurnStatus::Interrupted);
    runtime.shutdown_thread(thread()).await.unwrap();

    assert_eq!(
        backend.effects(),
        vec![
            FakeExternalEffect::CreatedThread { thread: thread() },
            FakeExternalEffect::CreatedTurn {
                turn: turn(),
                operation_marker: OperationId::from_str(MARKER).unwrap(),
            },
            FakeExternalEffect::Interrupted { turn: turn() },
            FakeExternalEffect::Shutdown { thread: thread() },
        ]
    );
    let state = backend.snapshot();
    assert!(state.threads[0].shutdown);
    assert!(state.threads[0].turns[0].interrupted);
}

#[test]
fn constructor_rejects_impossible_external_effect_scripts() {
    assert!(FakeCodexRuntime::new(vec![FakeStep::StartThread {
        expected: thread_spec(),
        external: None,
        result: Ok(thread()),
    }])
    .is_err());

    assert!(FakeCodexRuntime::new(vec![FakeStep::StartTurn {
        expected_thread: thread(),
        expected: turn_spec(),
        external: Some(turn()),
        result: Ok(TurnHandle {
            turn_id: TurnId::from_str("turn-other").unwrap(),
            ..turn()
        }),
    }])
    .is_err());

    assert!(FakeCodexRuntime::new(vec![FakeStep::StartThread {
        expected: thread_spec(),
        external: Some(thread()),
        result: Err(RuntimeFailure {
            kind: RuntimeErrorKind::OutputSchema,
            message: "not response loss".to_string(),
            transient: false,
        }),
    }])
    .is_err());

    assert!(FakeCodexRuntime::new(vec![FakeStep::StartTurn {
        expected_thread: thread(),
        expected: turn_spec(),
        external: Some(turn()),
        result: Err(disconnected()),
    }])
    .is_err());
}

#[tokio::test]
async fn failed_interrupt_has_no_external_effect() {
    let backend = FakeBackend::default();
    let mut runtime = FakeCodexRuntime::with_backend(
        vec![
            FakeStep::StartThread {
                expected: thread_spec(),
                external: Some(thread()),
                result: Ok(thread()),
            },
            FakeStep::StartTurn {
                expected_thread: thread(),
                expected: turn_spec(),
                external: Some(turn()),
                result: Ok(turn()),
            },
            FakeStep::Interrupt {
                expected: turn(),
                result: Err(RuntimeFailure {
                    kind: RuntimeErrorKind::Transport,
                    message: "write failed".to_string(),
                    transient: true,
                }),
            },
        ],
        backend.clone(),
    )
    .unwrap();

    runtime.start_thread(thread_spec()).await.unwrap();
    runtime.start_turn(&thread(), turn_spec()).await.unwrap();
    runtime.interrupt(&turn()).await.unwrap_err();
    assert_eq!(
        backend
            .effects()
            .iter()
            .filter(|effect| matches!(effect, FakeExternalEffect::Interrupted { .. }))
            .count(),
        0
    );
}
