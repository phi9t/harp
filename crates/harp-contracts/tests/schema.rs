use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(unix)]
use std::ffi::CString;
#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::io::Write;
#[cfg(unix)]
use std::mem::MaybeUninit;
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::sync::atomic::{AtomicU64, Ordering};

use harp_contracts::{
    AdapterKind, ArtifactRef, AttemptId, Budget, CheckResult, CheckStatus, Checkpoint,
    DeterministicEvaluation, DisconnectedEvent, ExternalSessionId, LaggedEvent, NodeKind,
    OperationId, ResultEnvelope, ResultStatus, RetryPolicy, RunId, RunMetrics, RuntimeErrorKind,
    RuntimeEvent, RuntimeFailure, SemanticEvaluation, ServerRequestEvent, TaskGraph, TaskId,
    TaskNode, TaskRole, ThreadHandle, ThreadId, ThreadSnapshot, ThreadSpec, ThreadStartedEvent,
    ThreadStatus, TokenUsage, TokenUsageEvent, TurnCompletedEvent, TurnHandle, TurnId,
    TurnSnapshot, TurnSpec, TurnStartedEvent, TurnStatus, WorkspaceMode,
};
use schemars::{schema_for, JsonSchema};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn artifact(digest: &str) -> ArtifactRef {
    ArtifactRef::sha256(digest, "application/json", 128).expect("valid artifact")
}

fn task_graph() -> TaskGraph {
    TaskGraph {
        schema_version: 1,
        nodes: vec![TaskNode {
            task_id: "analyze-app-server".parse().expect("task id"),
            kind: NodeKind::Analysis,
            role: TaskRole::Explore,
            instruction: "Inspect App Server persistence.".to_string(),
            dependencies: Vec::new(),
            inputs: vec![artifact(DIGEST_A)],
            workspace_mode: WorkspaceMode::ReadOnly,
            model_policy: "root-default".to_string(),
            permission_profile: "read-only".to_string(),
            budget: Budget::new(20_000, 300, 16 * 1024 * 1024),
            output_schema: "result-envelope/v1".to_string(),
            retry_policy: RetryPolicy {
                max_transient_attempts: 2,
            },
        }],
    }
}

fn result_envelope(status: ResultStatus) -> ResultEnvelope {
    ResultEnvelope {
        schema_version: 1,
        task_id: "analyze-app-server".parse().expect("task id"),
        status,
        answer_ref: Some(artifact(DIGEST_A)),
        evidence: vec![artifact(DIGEST_B)],
        trace_ref: ArtifactRef::sha256(DIGEST_B, "application/jsonl", 64).expect("valid trace"),
        summary: "Bounded result.".to_string(),
        token_usage: 42,
        confidence: Some(0.8),
        failure_class: None,
    }
}

fn assert_rejects_unknown_field<T>(value: Value)
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    let error = serde_json::from_value::<T>(value).expect_err("unknown field must be rejected");
    assert!(
        error.to_string().contains("unknown field"),
        "unexpected error: {error}"
    );
}

#[test]
fn generated_uuid_ids_are_v7_and_round_trip_as_strings() {
    let run_id = RunId::new();
    let attempt_id = AttemptId::new();
    let operation_id = OperationId::new();

    for encoded in [
        run_id.to_string(),
        attempt_id.to_string(),
        operation_id.to_string(),
    ] {
        let uuid = Uuid::parse_str(&encoded).expect("generated UUID");
        assert_eq!(uuid.get_version_num(), 7);
        assert_eq!(
            serde_json::to_value(encoded.clone()).unwrap(),
            json!(encoded)
        );
    }

    assert_eq!(RunId::from_str(&run_id.to_string()).unwrap(), run_id);
    assert_eq!(
        AttemptId::from_str(&attempt_id.to_string()).unwrap(),
        attempt_id
    );
    assert_eq!(
        OperationId::from_str(&operation_id.to_string()).unwrap(),
        operation_id
    );
    assert_eq!(
        serde_json::to_value(&run_id).unwrap(),
        json!(run_id.to_string())
    );
    assert_eq!(
        serde_json::to_value(&attempt_id).unwrap(),
        json!(attempt_id.to_string())
    );
    assert_eq!(
        serde_json::to_value(&operation_id).unwrap(),
        json!(operation_id.to_string())
    );
    assert!(RunId::from_str("not-a-uuid").is_err());
    assert!(AttemptId::from_str("00000000-0000-4000-8000-000000000000").is_err());
}

#[test]
fn task_id_accepts_only_bounded_portable_names() {
    for valid in ["a", "A1", "_", "-", "a.b", "a_b-c.d", &"x".repeat(128)] {
        let task_id = TaskId::from_str(valid).expect("valid task ID");
        assert_eq!(task_id.to_string(), valid);
        let encoded = serde_json::to_string(&task_id).unwrap();
        assert_eq!(serde_json::from_str::<TaskId>(&encoded).unwrap(), task_id);
    }

    for invalid in [
        "",
        " task",
        "task ",
        ".",
        "..",
        "a/b",
        "a\\b",
        "a\0b",
        "a b",
        "a:b",
        "é",
        &"x".repeat(129),
    ] {
        assert!(
            TaskId::from_str(invalid).is_err(),
            "accepted invalid task ID {invalid:?}"
        );
    }
}

#[test]
fn opaque_runtime_ids_reject_empty_oversized_and_control_strings() {
    for valid in ["opaque", "with spaces", "é", &"x".repeat(256)] {
        assert!(ThreadId::from_str(valid).is_ok(), "thread ID {valid:?}");
        assert!(TurnId::from_str(valid).is_ok(), "turn ID {valid:?}");
    }

    for invalid in ["", "line\nbreak", "tab\tvalue", "\u{7f}", &"x".repeat(257)] {
        assert!(
            ThreadId::from_str(invalid).is_err(),
            "thread ID {invalid:?}"
        );
        assert!(TurnId::from_str(invalid).is_err(), "turn ID {invalid:?}");
    }
}

#[test]
fn external_session_id_preserves_provider_text_through_serde() {
    let raw = "  provider/Session-É:01  ";
    let session_id = ExternalSessionId::from_str(raw).expect("valid external session ID");

    assert_eq!(session_id.to_string(), raw);
    assert_eq!(serde_json::to_value(&session_id).unwrap(), json!(raw));
    assert_eq!(
        serde_json::from_value::<ExternalSessionId>(json!(raw)).unwrap(),
        session_id
    );
}

#[test]
fn external_session_id_rejects_empty_control_and_overlong_values() {
    for invalid in [
        String::new(),
        "line\nbreak".to_string(),
        "tab\tvalue".to_string(),
        "\u{7f}".to_string(),
        format!("{}x", "é".repeat(128)),
    ] {
        assert!(
            ExternalSessionId::from_str(&invalid).is_err(),
            "accepted invalid external session ID {invalid:?}"
        );
        assert!(
            serde_json::from_value::<ExternalSessionId>(json!(invalid)).is_err(),
            "deserialized invalid external session ID"
        );
    }
}

#[test]
fn artifact_ref_constructor_and_validation_enforce_content_addressing() {
    let artifact = ArtifactRef::sha256(DIGEST_A, "application/json", 7).unwrap();
    assert_eq!(artifact.uri, format!("artifact://sha256/{DIGEST_A}"));
    assert!(artifact.validate().is_ok());

    for digest in [
        "",
        "a",
        "Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "gaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        assert!(ArtifactRef::sha256(digest, "text/plain", 0).is_err());
    }

    let mut invalid = artifact.clone();
    invalid.uri = format!("artifact://sha256/{DIGEST_B}");
    assert!(invalid.validate().is_err());

    let mut invalid = artifact.clone();
    invalid.media_type.clear();
    assert!(invalid.validate().is_err());

    let mut invalid = artifact.clone();
    invalid.media_type = "x".repeat(257);
    assert!(invalid.validate().is_err());

    let mut invalid = artifact;
    invalid.permitted_ranges = Some(vec!["".to_string()]);
    assert!(invalid.validate().is_err());
}

#[test]
fn task_graph_round_trips_rejects_unknown_fields_and_validates_local_shape() {
    let graph = task_graph();
    assert!(graph.validate_shape().is_ok());
    let value = serde_json::to_value(&graph).unwrap();
    assert_eq!(
        serde_json::from_value::<TaskGraph>(value.clone()).unwrap(),
        graph
    );

    let mut unknown = value;
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_string(), json!(true));
    assert_rejects_unknown_field::<TaskGraph>(unknown);

    let mut invalid = graph.clone();
    invalid.schema_version = 2;
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph.clone();
    invalid.nodes.clear();
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph.clone();
    invalid.nodes = vec![invalid.nodes[0].clone(); 65];
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph.clone();
    invalid.nodes[0].instruction = "x".repeat(16_385);
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph.clone();
    invalid.nodes[0].model_policy.clear();
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph.clone();
    invalid.nodes[0].permission_profile.clear();
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph.clone();
    invalid.nodes[0].output_schema.clear();
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph.clone();
    invalid.nodes[0].budget.max_tokens = 0;
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph.clone();
    invalid.nodes[0].retry_policy.max_transient_attempts = 4;
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph;
    invalid.nodes[0].inputs[0].sha256 = DIGEST_B.to_string();
    assert!(invalid.validate_shape().is_err());
}

#[test]
fn task_graph_shape_validation_does_not_apply_semantic_dag_rules() {
    let mut graph = task_graph();
    let task_id = graph.nodes[0].task_id.clone();
    graph.nodes[0].dependencies = vec![task_id.clone()];
    graph.nodes.push(TaskNode {
        task_id,
        ..graph.nodes[0].clone()
    });

    assert!(graph.validate_shape().is_ok());
}

#[test]
fn task_graph_reference_bounds_accept_exact_limits_and_reject_plus_one() {
    let mut graph = task_graph();
    let task_id = graph.nodes[0].task_id.clone();
    graph.nodes[0].dependencies = vec![task_id; 64];
    graph.nodes[0].inputs = vec![artifact(DIGEST_A); 64];
    assert!(graph.validate_shape().is_ok());

    let mut invalid = graph.clone();
    invalid.nodes[0]
        .dependencies
        .push("overflow".parse().unwrap());
    assert!(invalid.validate_shape().is_err());

    let mut invalid = graph;
    invalid.nodes[0].inputs.push(artifact(DIGEST_B));
    assert!(invalid.validate_shape().is_err());
}

#[test]
fn task_graph_aggregate_reference_bounds_accept_exact_limit() {
    let template = task_graph().nodes.remove(0);
    let dependencies = vec!["dependency".parse().unwrap(); 64];
    let inputs = vec![artifact(DIGEST_A); 64];
    let nodes = (0..64)
        .map(|index| TaskNode {
            task_id: format!("task-{index}").parse().unwrap(),
            dependencies: dependencies.clone(),
            inputs: inputs.clone(),
            ..template.clone()
        })
        .collect();
    let graph = TaskGraph {
        schema_version: 1,
        nodes,
    };

    assert!(graph.validate_shape().is_ok());

    let mut invalid_dependencies = graph.clone();
    invalid_dependencies.nodes[63]
        .dependencies
        .push("overflow".parse().unwrap());
    assert!(invalid_dependencies.validate_shape().is_err());

    let mut invalid_inputs = graph;
    invalid_inputs.nodes[63].inputs.push(artifact(DIGEST_B));
    assert!(invalid_inputs.validate_shape().is_err());
}

#[test]
fn result_envelope_round_trips_rejects_unknown_fields_and_enforces_states() {
    let envelope = result_envelope(ResultStatus::Success);
    assert!(envelope.validate().is_ok());
    let value = serde_json::to_value(&envelope).unwrap();
    assert_eq!(
        serde_json::from_value::<ResultEnvelope>(value.clone()).unwrap(),
        envelope
    );

    let mut unknown = value;
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_string(), json!(true));
    assert_rejects_unknown_field::<ResultEnvelope>(unknown);

    let mut invalid = envelope.clone();
    invalid.summary = "x".repeat(4_097);
    assert!(invalid.validate().is_err());

    let mut invalid = envelope.clone();
    invalid.summary = "nul\0inside".to_string();
    assert!(invalid.validate().is_err());

    for confidence in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        let mut invalid = envelope.clone();
        invalid.confidence = Some(confidence);
        assert!(invalid.validate().is_err());
    }

    let mut invalid = envelope.clone();
    invalid.answer_ref = None;
    assert!(invalid.validate().is_err());

    let mut invalid = envelope.clone();
    invalid.failure_class = Some("failure".to_string());
    assert!(invalid.validate().is_err());

    let mut failed = envelope.clone();
    failed.status = ResultStatus::Failed;
    failed.failure_class = Some("runtime".to_string());
    assert!(failed.validate().is_err());
    failed.answer_ref = None;
    assert!(failed.validate().is_ok());

    let mut partial = envelope.clone();
    partial.status = ResultStatus::Partial;
    partial.answer_ref = None;
    partial.evidence.clear();
    assert!(partial.validate().is_err());
    partial.evidence.push(artifact(DIGEST_B));
    assert!(partial.validate().is_ok());

    let mut cancelled = envelope;
    cancelled.status = ResultStatus::Cancelled;
    assert!(cancelled.validate().is_err());
    cancelled.answer_ref = None;
    cancelled.evidence.clear();
    cancelled.failure_class = Some("cancelled".to_string());
    assert!(cancelled.validate().is_err());
    cancelled.failure_class = None;
    assert!(cancelled.validate().is_ok());
}

#[test]
fn result_envelope_bounds_and_recursively_validates_artifact_refs() {
    let mut invalid = result_envelope(ResultStatus::Success);
    invalid.evidence = vec![artifact(DIGEST_A); 257];
    assert!(invalid.validate().is_err());

    let mut invalid = result_envelope(ResultStatus::Success);
    invalid.trace_ref.uri = format!("artifact://sha256/{DIGEST_A}");
    assert!(invalid.validate().is_err());

    let mut invalid = result_envelope(ResultStatus::Success);
    invalid.schema_version = 0;
    assert!(invalid.validate().is_err());
}

#[test]
fn checkpoint_requires_explicit_time_and_rejects_duplicates_and_bounds() {
    let mut checkpoint = Checkpoint::new(
        RunId::new(),
        "task".parse().unwrap(),
        AttemptId::new(),
        "collecting",
        1_723_145_600,
    )
    .unwrap();
    assert_eq!(checkpoint.schema_version, 1);
    assert_eq!(checkpoint.updated_at_unix_seconds, 1_723_145_600);
    assert!(checkpoint.validate().is_ok());

    checkpoint.completed_units.push("unit-a".to_string());
    checkpoint.pending_units.push("unit-a".to_string());
    assert!(checkpoint.validate().is_err());

    let mut invalid = checkpoint.clone();
    invalid.pending_units = vec!["unit-b".to_string(); 257];
    assert!(invalid.validate().is_err());

    let mut invalid = checkpoint.clone();
    invalid.pending_units.clear();
    invalid.completed_units = vec![String::new()];
    assert!(invalid.validate().is_err());

    let mut invalid = checkpoint;
    invalid.schema_version = 2;
    assert!(invalid.validate().is_err());
}

#[test]
fn checkpoint_accepts_the_full_i64_timestamp_domain() {
    let checkpoint = Checkpoint::new(
        RunId::new(),
        "task".parse().unwrap(),
        AttemptId::new(),
        "collecting",
        i64::MIN,
    )
    .unwrap();

    assert!(checkpoint.validate().is_ok());
    let value = serde_json::to_value(&checkpoint).unwrap();
    assert_eq!(value["updatedAtUnixSeconds"], i64::MIN);
    assert_eq!(
        serde_json::from_value::<Checkpoint>(value).unwrap(),
        checkpoint
    );
}

fn thread_spec() -> ThreadSpec {
    ThreadSpec {
        base_instructions: "Base instructions.\nSecond line.".to_string(),
        developer_instructions: "Developer instructions.\tTabbed.".to_string(),
        cwd: "/workspace".to_string(),
        runtime_workspace_roots: vec!["/workspace".to_string()],
        workspace_authority: None,
        approval_policy: "never".to_string(),
        sandbox_mode: "read-only".to_string(),
        model: "codex".to_string(),
        reasoning_effort: Some("high".to_string()),
        ephemeral: true,
    }
}

fn turn_spec() -> TurnSpec {
    TurnSpec {
        instruction: "Inspect persistence.\nReturn findings.".to_string(),
        operation_marker: OperationId::new(),
        output_schema: json!({"type": "object"}),
        model: Some("codex".to_string()),
        reasoning_effort: Some("high".to_string()),
    }
}

fn turn_snapshot() -> TurnSnapshot {
    TurnSnapshot {
        turn_id: TurnId::from_str("turn-1").unwrap(),
        operation_marker: Some(
            OperationId::from_str("018f22e2-7c3b-7def-8123-456789abcdef").unwrap(),
        ),
        status: TurnStatus::Completed,
        final_agent_message: Some("done\nwith details".to_string()),
    }
}

#[test]
fn runtime_specs_reject_unknown_fields_and_events_use_tagged_payloads() {
    let mut unknown = serde_json::to_value(turn_spec()).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_string(), json!(true));
    assert_rejects_unknown_field::<TurnSpec>(unknown);

    let thread_id = ThreadId::from_str("thread-1").unwrap();
    let turn_id = TurnId::from_str("turn-1").unwrap();
    let events = vec![
        RuntimeEvent::ThreadStarted(ThreadStartedEvent {
            thread_id: thread_id.clone(),
        }),
        RuntimeEvent::TurnStarted(TurnStartedEvent {
            thread_id: thread_id.clone(),
            turn_id: turn_id.clone(),
        }),
        RuntimeEvent::TokenUsage(TokenUsageEvent {
            thread_id: thread_id.clone(),
            turn_id: turn_id.clone(),
            usage: TokenUsage {
                total_tokens: 8,
                input_tokens: 5,
                cached_input_tokens: 1,
                output_tokens: 3,
                reasoning_output_tokens: 1,
            },
        }),
        RuntimeEvent::TurnCompleted(TurnCompletedEvent {
            thread_id: thread_id.clone(),
            turn: turn_snapshot(),
        }),
        RuntimeEvent::ServerRequest(ServerRequestEvent {
            thread_id: thread_id.clone(),
            turn_id: Some(turn_id.clone()),
            request: json!({"kind": "approval"}),
        }),
        RuntimeEvent::Lagged(LaggedEvent { dropped_events: 3 }),
        RuntimeEvent::Disconnected(DisconnectedEvent {
            reason: "transport closed".to_string(),
        }),
    ];

    let expected_types = [
        "threadStarted",
        "turnStarted",
        "tokenUsage",
        "turnCompleted",
        "serverRequest",
        "lagged",
        "disconnected",
    ];
    for (event, expected_type) in events.into_iter().zip(expected_types) {
        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(value["type"], expected_type);
        assert_eq!(
            serde_json::from_value::<RuntimeEvent>(value).unwrap(),
            event
        );
    }

    let statuses = [
        ThreadStatus::Idle,
        ThreadStatus::Active,
        ThreadStatus::NotLoaded,
        ThreadStatus::SystemError,
    ];
    assert_eq!(
        serde_json::to_value(statuses).unwrap(),
        json!(["idle", "active", "notLoaded", "systemError"])
    );
}

#[test]
fn runtime_specs_validate_local_bounds_and_json_objects() {
    let valid_thread = thread_spec();
    assert!(valid_thread.validate().is_ok());

    let mut invalid = valid_thread.clone();
    invalid.base_instructions = "x".repeat(65_537);
    assert!(invalid.validate().is_err());

    let mut invalid = valid_thread.clone();
    invalid.runtime_workspace_roots = vec!["/workspace".to_string(); 17];
    assert!(invalid.validate().is_err());

    let mut invalid = valid_thread.clone();
    invalid.runtime_workspace_roots = vec!["/work\0space".to_string()];
    assert!(invalid.validate().is_err());

    let mut invalid = valid_thread;
    invalid.approval_policy = "never\n".to_string();
    assert!(invalid.validate().is_err());

    for field in ["approval", "sandbox", "model", "reasoning"] {
        let mut invalid = thread_spec();
        match field {
            "approval" => invalid.approval_policy.clear(),
            "sandbox" => invalid.sandbox_mode.clear(),
            "model" => invalid.model.clear(),
            "reasoning" => invalid.reasoning_effort = Some(String::new()),
            _ => unreachable!(),
        }
        assert!(invalid.validate().is_err(), "accepted empty {field}");
    }

    let valid_turn = turn_spec();
    assert!(valid_turn.validate().is_ok());

    let mut invalid = valid_turn.clone();
    invalid.instruction = "x".repeat(262_145);
    assert!(invalid.validate().is_err());

    let mut invalid = valid_turn.clone();
    invalid.output_schema = json!(["not", "an", "object"]);
    assert!(invalid.validate().is_err());

    let mut invalid = valid_turn;
    invalid.output_schema = json!({"schema": "x".repeat(262_145)});
    assert!(invalid.validate().is_err());

    let mut invalid = turn_spec();
    invalid.model = Some(String::new());
    assert!(invalid.validate().is_err());

    let mut invalid = turn_spec();
    invalid.reasoning_effort = Some(String::new());
    assert!(invalid.validate().is_err());
}

#[test]
fn token_usage_validates_component_accounting() {
    let valid = TokenUsage {
        total_tokens: 8,
        input_tokens: 5,
        cached_input_tokens: 2,
        output_tokens: 3,
        reasoning_output_tokens: 1,
    };
    assert!(valid.validate().is_ok());

    let mut invalid = valid.clone();
    invalid.cached_input_tokens = 6;
    assert!(invalid.validate().is_err());

    let mut invalid = valid.clone();
    invalid.reasoning_output_tokens = 4;
    assert!(invalid.validate().is_err());

    let mut invalid = valid;
    invalid.total_tokens = 7;
    assert!(invalid.validate().is_err());

    let overflow = TokenUsage {
        total_tokens: u64::MAX,
        input_tokens: u64::MAX,
        cached_input_tokens: 0,
        output_tokens: 1,
        reasoning_output_tokens: 0,
    };
    assert!(overflow.validate().is_err());
}

#[test]
fn runtime_snapshots_handles_and_events_validate_recursively() {
    let thread_id = ThreadId::from_str("thread-1").unwrap();
    let turn_id = TurnId::from_str("turn-1").unwrap();
    assert!(ThreadHandle {
        thread_id: thread_id.clone()
    }
    .validate()
    .is_ok());
    assert!(TurnHandle {
        thread_id: thread_id.clone(),
        turn_id: turn_id.clone(),
    }
    .validate()
    .is_ok());

    let valid_snapshot = ThreadSnapshot {
        thread_id: thread_id.clone(),
        status: ThreadStatus::Idle,
        turns: vec![turn_snapshot()],
    };
    assert!(valid_snapshot.validate().is_ok());

    let mut invalid = valid_snapshot.clone();
    invalid.turns = vec![turn_snapshot(); 1_025];
    assert!(invalid.validate().is_err());

    let mut invalid = valid_snapshot;
    invalid.turns[0].final_agent_message = Some("bad\0message".to_string());
    assert!(invalid.validate().is_err());

    let valid_event = RuntimeEvent::TurnCompleted(TurnCompletedEvent {
        thread_id: thread_id.clone(),
        turn: turn_snapshot(),
    });
    assert!(valid_event.validate().is_ok());

    let invalid_event = RuntimeEvent::TurnCompleted(TurnCompletedEvent {
        thread_id: thread_id.clone(),
        turn: TurnSnapshot {
            final_agent_message: Some("x".repeat(1_048_577)),
            ..turn_snapshot()
        },
    });
    assert!(invalid_event.validate().is_err());

    let invalid_event = RuntimeEvent::ServerRequest(ServerRequestEvent {
        thread_id: thread_id.clone(),
        turn_id: Some(turn_id.clone()),
        request: json!(["not", "an", "object"]),
    });
    assert!(invalid_event.validate().is_err());

    let invalid_event = RuntimeEvent::ServerRequest(ServerRequestEvent {
        thread_id: thread_id.clone(),
        turn_id: Some(turn_id),
        request: json!({"payload": "x".repeat(65_537)}),
    });
    assert!(invalid_event.validate().is_err());

    let invalid_event = RuntimeEvent::Lagged(LaggedEvent { dropped_events: 0 });
    assert!(invalid_event.validate().is_err());

    let invalid_event = RuntimeEvent::Disconnected(DisconnectedEvent {
        reason: "bad\0reason".to_string(),
    });
    assert!(invalid_event.validate().is_err());
}

#[test]
fn thread_snapshot_resolves_unique_operation_markers() {
    let marker = OperationId::from_str("018f22e2-7c3b-7def-8123-456789abcdef").unwrap();
    let snapshot = ThreadSnapshot {
        thread_id: ThreadId::from_str("thread-1").unwrap(),
        status: ThreadStatus::Idle,
        turns: vec![
            turn_snapshot(),
            TurnSnapshot {
                turn_id: TurnId::from_str("turn-unmarked").unwrap(),
                operation_marker: None,
                status: TurnStatus::Interrupted,
                final_agent_message: None,
            },
        ],
    };

    assert_eq!(
        snapshot
            .turn_by_operation_marker(&marker)
            .unwrap()
            .map(|turn| &turn.turn_id),
        Some(&TurnId::from_str("turn-1").unwrap())
    );
    assert_eq!(
        snapshot
            .turn_by_operation_marker(
                &OperationId::from_str("018f22e2-7c3c-7abc-9234-56789abcdef0").unwrap()
            )
            .unwrap(),
        None
    );
    assert_eq!(
        serde_json::to_value(&snapshot).unwrap()["turns"][0]["operationMarker"],
        marker.to_string()
    );
}

#[test]
fn thread_snapshot_rejects_duplicate_operation_markers() {
    let mut duplicate = turn_snapshot();
    duplicate.turn_id = TurnId::from_str("turn-duplicate").unwrap();
    let marker = duplicate.operation_marker.clone().unwrap();
    let snapshot = ThreadSnapshot {
        thread_id: ThreadId::from_str("thread-1").unwrap(),
        status: ThreadStatus::Idle,
        turns: vec![turn_snapshot(), duplicate],
    };

    assert!(snapshot.validate().is_err());
    assert!(snapshot.turn_by_operation_marker(&marker).is_err());
}

#[test]
fn runtime_failure_validation_bounds_message() {
    let valid = RuntimeFailure {
        kind: RuntimeErrorKind::Transport,
        message: "connection reset\nretrying".to_string(),
        transient: true,
    };
    assert!(valid.validate().is_ok());

    let mut invalid = valid.clone();
    invalid.message.clear();
    assert!(invalid.validate().is_err());

    let mut invalid = valid;
    invalid.message = "x".repeat(8_193);
    assert!(invalid.validate().is_err());

    let invalid = RuntimeFailure {
        kind: RuntimeErrorKind::Protocol,
        message: "bad\0message".to_string(),
        transient: false,
    };
    assert!(invalid.validate().is_err());
}

fn evaluation() -> (
    CheckResult,
    DeterministicEvaluation,
    SemanticEvaluation,
    RunMetrics,
) {
    (
        CheckResult {
            code: "schema.valid".to_string(),
            status: CheckStatus::Pass,
            message: "Schema is valid.".to_string(),
            evidence: vec![artifact(DIGEST_A)],
        },
        DeterministicEvaluation {
            passed: true,
            checks: Vec::new(),
            normalized_result_sha256: DIGEST_A.to_string(),
        },
        SemanticEvaluation {
            architectural_correctness: 1.0,
            evidence_coverage: 0.9,
            boundary_accuracy: 0.8,
            integration_gap_quality: 0.7,
            implementation_usefulness: 0.6,
            unsupported_claim_rate: 0.1,
            notes: "Grounded.".to_string(),
        },
        RunMetrics {
            adapter: AdapterKind::Fake,
            startup_ms: 1,
            wall_ms: 2,
            root_tokens: 3,
            child_tokens: 4,
            peak_concurrency: 5,
            event_lag_count: 6,
            protocol_failure_count: 7,
            indeterminate_attempts: 8,
            duplicate_tokens: 9,
            peak_disk_bytes: 10,
            packaged_binary_bytes: 11,
            operator_steps: 12,
        },
    )
}

#[test]
fn evaluation_validation_rejects_bad_strings_digests_and_scores() {
    let (check, deterministic, semantic, metrics) = evaluation();
    assert!(check.validate().is_ok());
    assert!(deterministic.validate().is_ok());
    assert!(semantic.validate().is_ok());
    assert!(metrics.validate().is_ok());

    let mut invalid = check.clone();
    invalid.code.clear();
    assert!(invalid.validate().is_err());

    let mut invalid = check;
    invalid.evidence[0].uri = format!("artifact://sha256/{DIGEST_B}");
    assert!(invalid.validate().is_err());

    let mut invalid = deterministic;
    invalid.normalized_result_sha256 = "A".repeat(64);
    assert!(invalid.validate().is_err());

    for score in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        let mut invalid = semantic.clone();
        invalid.architectural_correctness = score;
        assert!(invalid.validate().is_err());
    }

    let mut invalid = semantic;
    invalid.notes = "x".repeat(16_385);
    assert!(invalid.validate().is_err());
}

#[test]
fn deterministic_evaluation_passed_matches_all_check_statuses() {
    let (passing_check, mut evaluation, _, _) = evaluation();
    evaluation.passed = false;
    evaluation.checks = vec![passing_check.clone()];
    assert!(evaluation.validate().is_err());

    let mut failing_check = passing_check;
    failing_check.status = CheckStatus::Fail;
    evaluation.passed = true;
    evaluation.checks = vec![failing_check];
    assert!(evaluation.validate().is_err());

    evaluation.passed = true;
    evaluation.checks.clear();
    assert!(evaluation.validate().is_ok());
}

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("contracts crate must live beneath workspace/crates");
    workspace_root
        .canonicalize()
        .expect("canonical workspace root")
}

fn schema_bytes<T: JsonSchema>() -> (Value, Vec<u8>) {
    let schema = schema_for!(T);
    let value = serde_json::to_value(schema).expect("schema value");
    let mut bytes = serde_json::to_vec_pretty(&value).expect("pretty schema");
    bytes.push(b'\n');
    (value, bytes)
}

fn assert_root_rejects_additional_properties(schema: &Value) {
    let object = schema.as_object().expect("root schema object");
    assert_eq!(
        object.get("additionalProperties"),
        Some(&Value::Bool(false)),
        "root schema must reject additional properties: {schema}"
    );
}

fn definition<'a>(schema: &'a Value, name: &str) -> &'a Value {
    schema
        .pointer(&format!("/$defs/{name}"))
        .unwrap_or_else(|| panic!("missing schema definition {name}"))
}

fn property<'a>(schema: &'a Value, name: &str) -> &'a Value {
    schema
        .pointer(&format!("/properties/{name}"))
        .unwrap_or_else(|| panic!("missing schema property {name}"))
}

#[test]
fn generated_id_schemas_encode_string_constraints() {
    let run_id = serde_json::to_value(schema_for!(RunId)).unwrap();
    assert_eq!(run_id["type"], "string");
    assert_eq!(run_id["format"], "uuid");

    let thread_id = serde_json::to_value(schema_for!(ThreadId)).unwrap();
    assert_eq!(thread_id["type"], "string");
    assert_eq!(thread_id["minLength"], 1);
    assert_eq!(thread_id["maxLength"], 256);

    let external_session_id = serde_json::to_value(schema_for!(ExternalSessionId)).unwrap();
    assert_eq!(external_session_id["type"], "string");
    assert_eq!(external_session_id["minLength"], 1);
    assert_eq!(external_session_id["maxLength"], 256);
}

#[test]
fn generated_task_graph_schema_encodes_representable_bounds() {
    let (schema, _) = schema_bytes::<TaskGraph>();
    assert_eq!(property(&schema, "schemaVersion")["const"], 1);
    assert_eq!(property(&schema, "nodes")["minItems"], 1);
    assert_eq!(property(&schema, "nodes")["maxItems"], 64);

    let task_id = definition(&schema, "TaskId");
    assert_eq!(task_id["minLength"], 1);
    assert_eq!(task_id["maxLength"], 128);
    assert_eq!(task_id["pattern"], r"^(?!\.{1,2}$)[A-Za-z0-9_.-]{1,128}$");

    let node = definition(&schema, "TaskNode");
    assert_eq!(property(node, "dependencies")["maxItems"], 64);
    assert_eq!(property(node, "inputs")["maxItems"], 64);
    assert_eq!(property(node, "instruction")["minLength"], 1);
    assert_eq!(property(node, "instruction")["maxLength"], 16_384);
    assert_eq!(property(node, "modelPolicy")["minLength"], 1);
    assert_eq!(property(node, "modelPolicy")["maxLength"], 256);
    assert_eq!(property(node, "permissionProfile")["minLength"], 1);
    assert_eq!(property(node, "outputSchema")["maxLength"], 4_096);

    let budget = definition(&schema, "Budget");
    assert_eq!(property(budget, "maxTokens")["minimum"], 1);
    assert_eq!(property(budget, "timeoutSeconds")["minimum"], 1);
    assert_eq!(property(budget, "maxStorageBytes")["minimum"], 1);
    assert_eq!(
        property(definition(&schema, "RetryPolicy"), "maxTransientAttempts")["maximum"],
        3
    );

    let artifact = definition(&schema, "ArtifactRef");
    assert_eq!(property(artifact, "sha256")["pattern"], r"^[0-9a-f]{64}$");
    assert_eq!(property(artifact, "permittedRanges")["maxItems"], 256);
}

#[test]
fn generated_result_schema_encodes_representable_bounds() {
    let (schema, _) = schema_bytes::<ResultEnvelope>();
    assert_eq!(property(&schema, "schemaVersion")["const"], 1);
    assert_eq!(property(&schema, "evidence")["maxItems"], 256);
    assert_eq!(property(&schema, "summary")["maxLength"], 4096);

    let confidence = property(&schema, "confidence");
    assert_eq!(confidence["minimum"], 0.0);
    assert_eq!(confidence["maximum"], 1.0);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SchemaReceipt {
    TaskGraph,
    ResultEnvelope,
}

impl SchemaReceipt {
    fn relative_path(self) -> &'static Path {
        match self {
            Self::TaskGraph => Path::new("benchmarks/codex-architecture/schemas/task-graph.json"),
            Self::ResultEnvelope => {
                Path::new("benchmarks/codex-architecture/schemas/result-envelope.json")
            }
        }
    }

    fn file_name(self) -> &'static str {
        match self {
            Self::TaskGraph => "task-graph.json",
            Self::ResultEnvelope => "result-envelope.json",
        }
    }
}

fn invalid_schema_path(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(unix)]
fn create_schema_directory_tree(workspace_root: &Path) -> io::Result<PathBuf> {
    fs::create_dir(workspace_root)?;
    let benchmarks = workspace_root.join("benchmarks");
    fs::create_dir(&benchmarks)?;
    let architecture = benchmarks.join("codex-architecture");
    fs::create_dir(&architecture)?;
    let schemas = architecture.join("schemas");
    fs::create_dir(&schemas)?;
    Ok(schemas)
}

fn verified_schema_directory(workspace_root: &Path) -> io::Result<PathBuf> {
    let root_metadata = fs::symlink_metadata(workspace_root)?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(invalid_schema_path(
            "workspace root must be a real directory",
        ));
    }

    let mut current = workspace_root.to_path_buf();
    for component in ["benchmarks", "codex-architecture", "schemas"] {
        current.push(component);
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(invalid_schema_path(
                "schema path components must be real directories",
            ));
        }
    }
    Ok(current)
}

fn verify_existing_receipt(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(invalid_schema_path(
                    "existing schema receipt must be a regular file",
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;

                fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW)
                    .open(path)?;
            }
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
static SCHEMA_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
fn retry_eintr<F>(mut operation: F) -> libc::c_int
where
    F: FnMut() -> libc::c_int,
{
    loop {
        let result = operation();
        if result >= 0 || io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
            return result;
        }
    }
}

#[cfg(unix)]
fn path_c_string(path: &Path) -> io::Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| invalid_schema_path("schema path contains NUL"))
}

#[cfg(unix)]
fn static_c_string(value: &'static str) -> CString {
    CString::new(value).expect("static schema path component has no NUL")
}

#[cfg(unix)]
fn owned_fd(fd: libc::c_int) -> io::Result<OwnedFd> {
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

#[cfg(unix)]
fn open_workspace_root(workspace_root: &Path) -> io::Result<OwnedFd> {
    let path = path_c_string(workspace_root)?;
    owned_fd(retry_eintr(|| unsafe {
        libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    }))
}

#[cfg(unix)]
fn open_directory_at(parent_fd: RawFd, component: &'static str) -> io::Result<OwnedFd> {
    let component = static_c_string(component);
    let result = owned_fd(retry_eintr(|| unsafe {
        libc::openat(
            parent_fd,
            component.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    }));
    match result {
        Err(error)
            if matches!(
                error.raw_os_error(),
                Some(libc::ELOOP) | Some(libc::ENOTDIR)
            ) =>
        {
            Err(invalid_schema_path(
                "schema path components must be real directories",
            ))
        }
        other => other,
    }
}

#[cfg(unix)]
fn open_schema_directory_fd(workspace_root: &Path) -> io::Result<OwnedFd> {
    let workspace = open_workspace_root(workspace_root)?;
    let benchmarks = open_directory_at(workspace.as_raw_fd(), "benchmarks")?;
    let architecture = open_directory_at(benchmarks.as_raw_fd(), "codex-architecture")?;
    open_directory_at(architecture.as_raw_fd(), "schemas")
}

#[cfg(unix)]
fn verify_destination_at(directory_fd: RawFd, destination: &CString) -> io::Result<()> {
    let mut metadata = MaybeUninit::<libc::stat>::uninit();
    let result = retry_eintr(|| unsafe {
        libc::fstatat(
            directory_fd,
            destination.as_ptr(),
            metadata.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    });
    if result == 0 {
        let metadata = unsafe { metadata.assume_init() };
        if metadata.st_mode & libc::S_IFMT != libc::S_IFREG {
            return Err(invalid_schema_path(
                "existing schema receipt must be a regular file",
            ));
        }
        return Ok(());
    }

    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ENOENT) {
        Ok(())
    } else {
        Err(error)
    }
}

#[cfg(unix)]
fn create_temporary_schema_file(
    directory_fd: RawFd,
    receipt: SchemaReceipt,
    counter: &AtomicU64,
) -> io::Result<(File, CString)> {
    for _ in 0..64 {
        let counter = counter.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            ".{}.{}.{}.tmp",
            receipt.file_name(),
            std::process::id(),
            counter
        );
        let name = CString::new(name).expect("generated schema temp name has no NUL");
        let fd = retry_eintr(|| unsafe {
            libc::openat(
                directory_fd,
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        });
        if fd >= 0 {
            return Ok((unsafe { File::from_raw_fd(fd) }, name));
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EEXIST) {
            return Err(error);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not reserve a unique schema temp file",
    ))
}

#[cfg(unix)]
fn sync_directory(directory_fd: RawFd) -> io::Result<()> {
    if retry_eintr(|| unsafe { libc::fsync(directory_fd) }) == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if matches!(
        error.raw_os_error(),
        Some(libc::EINVAL) | Some(libc::ENOTSUP)
    ) {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("directory fsync is unsupported: {error}"),
        ))
    } else {
        Err(error)
    }
}

#[cfg(unix)]
fn sync_file(file: &File) -> io::Result<()> {
    if retry_eintr(|| unsafe { libc::fsync(file.as_raw_fd()) }) == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(unix)]
struct TemporarySchemaGuard {
    directory: OwnedFd,
    name: CString,
    active: bool,
}

#[cfg(unix)]
impl TemporarySchemaGuard {
    fn new(directory: OwnedFd, name: CString) -> Self {
        Self {
            directory,
            name,
            active: true,
        }
    }

    fn directory_fd(&self) -> RawFd {
        self.directory.as_raw_fd()
    }

    fn mark_published(&mut self) {
        self.active = false;
    }

    fn cleanup(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        let result = retry_eintr(|| unsafe {
            libc::unlinkat(self.directory.as_raw_fd(), self.name.as_ptr(), 0)
        });
        if result == 0 {
            self.active = false;
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            self.active = false;
            Ok(())
        } else {
            Err(error)
        }
    }
}

#[cfg(unix)]
impl Drop for TemporarySchemaGuard {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PublishFailure {
    None,
    Write,
    FileSync,
}

#[cfg(unix)]
fn error_with_cleanup(primary: io::Error, guard: &mut TemporarySchemaGuard) -> io::Error {
    match guard.cleanup() {
        Ok(()) => primary,
        Err(cleanup) => io::Error::new(
            primary.kind(),
            format!("{primary}; temporary schema cleanup also failed: {cleanup}"),
        ),
    }
}

#[cfg(unix)]
fn write_schema_receipt_with_options<F>(
    workspace_root: &Path,
    receipt: SchemaReceipt,
    bytes: &[u8],
    counter: &AtomicU64,
    failure: PublishFailure,
    hook: F,
) -> io::Result<()>
where
    F: FnOnce() -> io::Result<()>,
{
    let directory = open_schema_directory_fd(workspace_root)?;
    hook()?;

    let destination = static_c_string(receipt.file_name());
    verify_destination_at(directory.as_raw_fd(), &destination)?;
    let (mut temporary, temporary_name) =
        create_temporary_schema_file(directory.as_raw_fd(), receipt, counter)?;
    let mut guard = TemporarySchemaGuard::new(directory, temporary_name);

    let result = (|| {
        if failure == PublishFailure::Write {
            return Err(io::Error::other("injected schema write failure"));
        }
        temporary.write_all(bytes)?;
        if failure == PublishFailure::FileSync {
            return Err(io::Error::other("injected schema file sync failure"));
        }
        sync_file(&temporary)?;
        if retry_eintr(|| unsafe {
            libc::renameat(
                guard.directory_fd(),
                guard.name.as_ptr(),
                guard.directory_fd(),
                destination.as_ptr(),
            )
        }) != 0
        {
            return Err(io::Error::last_os_error());
        }
        guard.mark_published();
        sync_directory(guard.directory_fd())
    })();

    match result {
        Ok(()) => Ok(()),
        Err(error) if guard.active => Err(error_with_cleanup(error, &mut guard)),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn write_schema_receipt_with_hook<F>(
    workspace_root: &Path,
    receipt: SchemaReceipt,
    bytes: &[u8],
    hook: F,
) -> io::Result<()>
where
    F: FnOnce() -> io::Result<()>,
{
    write_schema_receipt_with_options(
        workspace_root,
        receipt,
        bytes,
        &SCHEMA_TEMP_COUNTER,
        PublishFailure::None,
        hook,
    )
}

fn write_schema_receipt(
    workspace_root: &Path,
    receipt: SchemaReceipt,
    bytes: &[u8],
) -> io::Result<()> {
    #[cfg(unix)]
    {
        write_schema_receipt_with_hook(workspace_root, receipt, bytes, || Ok(()))
    }
    #[cfg(not(unix))]
    {
        let _ = (workspace_root, receipt, bytes);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "schema receipt updates require descriptor-relative Unix APIs",
        ))
    }
}

fn read_schema_receipt(workspace_root: &Path, receipt: SchemaReceipt) -> io::Result<Vec<u8>> {
    let directory = verified_schema_directory(workspace_root)?;
    let destination = directory.join(receipt.file_name());
    verify_existing_receipt(&destination)?;
    fs::read(destination)
}

fn assert_schema_receipt<T: JsonSchema + Serialize>(receipt: SchemaReceipt) {
    let (schema, expected) = schema_bytes::<T>();
    assert_root_rejects_additional_properties(&schema);

    let update = match std::env::var("HARP_UPDATE_SCHEMAS") {
        Ok(value) if value == "1" => true,
        Ok(value) => panic!("HARP_UPDATE_SCHEMAS must be unset or exactly 1, got {value:?}"),
        Err(std::env::VarError::NotPresent) => false,
        Err(error) => panic!("cannot read HARP_UPDATE_SCHEMAS: {error}"),
    };
    let root = workspace_root();
    let path = root.join(receipt.relative_path());

    if update {
        write_schema_receipt(&root, receipt, &expected).expect("write generated schema");
        return;
    }

    let actual = read_schema_receipt(&root, receipt).unwrap_or_else(|error| {
        panic!(
            "schema receipt {} is missing or unreadable: {error}; regenerate with HARP_UPDATE_SCHEMAS=1",
            path.display()
        )
    });
    assert_eq!(
        actual,
        expected,
        "schema receipt drifted: {}",
        path.display()
    );
}

#[test]
fn task_graph_schema_receipt_is_exact() {
    assert_schema_receipt::<TaskGraph>(SchemaReceipt::TaskGraph);
}

#[test]
fn result_envelope_schema_receipt_is_exact() {
    assert_schema_receipt::<ResultEnvelope>(SchemaReceipt::ResultEnvelope);
}

#[test]
fn schema_receipt_destinations_are_closed_and_exact() {
    assert_eq!(
        SchemaReceipt::TaskGraph.relative_path(),
        Path::new("benchmarks/codex-architecture/schemas/task-graph.json")
    );
    assert_eq!(
        SchemaReceipt::ResultEnvelope.relative_path(),
        Path::new("benchmarks/codex-architecture/schemas/result-envelope.json")
    );
}

#[cfg(unix)]
#[test]
fn schema_writer_rejects_symlinked_schema_directory() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    fs::create_dir(&workspace).unwrap();
    let benchmarks = workspace.join("benchmarks");
    fs::create_dir(&benchmarks).unwrap();
    let architecture = benchmarks.join("codex-architecture");
    fs::create_dir(&architecture).unwrap();
    let real_schemas = temp.path().join("real-schemas");
    fs::create_dir(&real_schemas).unwrap();
    symlink(&real_schemas, architecture.join("schemas")).unwrap();

    let error = write_schema_receipt(&workspace, SchemaReceipt::TaskGraph, b"{}\n").unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(!real_schemas.join("task-graph.json").exists());
}

#[cfg(unix)]
#[test]
fn schema_writer_rejects_symlinked_existing_target() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let schemas = create_schema_directory_tree(&workspace).unwrap();
    let redirected = temp.path().join("redirected.json");
    fs::write(&redirected, b"untouched\n").unwrap();
    symlink(&redirected, schemas.join("task-graph.json")).unwrap();

    let error = write_schema_receipt(&workspace, SchemaReceipt::TaskGraph, b"{}\n").unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(fs::read(&redirected).unwrap(), b"untouched\n");
}

#[cfg(unix)]
fn temporary_schema_entries(schemas: &Path) -> Vec<String> {
    let mut entries = fs::read_dir(schemas)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

#[cfg(unix)]
#[test]
fn schema_writer_creates_and_overwrites_regular_receipts() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let schemas = create_schema_directory_tree(&workspace).unwrap();
    let destination = schemas.join("task-graph.json");

    write_schema_receipt(&workspace, SchemaReceipt::TaskGraph, b"first\n").unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"first\n");

    write_schema_receipt(&workspace, SchemaReceipt::TaskGraph, b"second\n").unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"second\n");
    assert!(temporary_schema_entries(&schemas).is_empty());
}

#[cfg(unix)]
#[test]
fn schema_writer_retries_temporary_name_collisions() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let schemas = create_schema_directory_tree(&workspace).unwrap();
    let counter = AtomicU64::new(0);
    let collision = schemas.join(format!(".task-graph.json.{}.0.tmp", std::process::id()));
    fs::write(&collision, b"collision\n").unwrap();

    write_schema_receipt_with_options(
        &workspace,
        SchemaReceipt::TaskGraph,
        b"published\n",
        &counter,
        PublishFailure::None,
        || Ok(()),
    )
    .unwrap();

    assert_eq!(
        fs::read(schemas.join("task-graph.json")).unwrap(),
        b"published\n"
    );
    assert_eq!(fs::read(&collision).unwrap(), b"collision\n");
    assert_eq!(
        temporary_schema_entries(&schemas),
        vec![collision.file_name().unwrap().to_string_lossy()]
    );
    fs::remove_file(collision).unwrap();
    assert!(temporary_schema_entries(&schemas).is_empty());
}

#[cfg(unix)]
#[test]
fn schema_writer_cleans_temporary_files_after_injected_failures() {
    for failure in [PublishFailure::Write, PublishFailure::FileSync] {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("workspace");
        let schemas = create_schema_directory_tree(&workspace).unwrap();
        let counter = AtomicU64::new(0);

        let error = write_schema_receipt_with_options(
            &workspace,
            SchemaReceipt::TaskGraph,
            b"unpublished\n",
            &counter,
            failure,
            || Ok(()),
        )
        .unwrap_err();

        assert!(error.to_string().contains("injected schema"));
        assert!(!schemas.join("task-graph.json").exists());
        assert!(temporary_schema_entries(&schemas).is_empty());
    }
}

#[cfg(unix)]
#[test]
fn schema_writer_resists_verified_directory_substitution() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let schemas = create_schema_directory_tree(&workspace).unwrap();
    let architecture = schemas.parent().unwrap();
    let held = architecture.join("schemas-held");
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("sentinel"), b"untouched\n").unwrap();

    write_schema_receipt_with_hook(&workspace, SchemaReceipt::TaskGraph, b"{}\n", || {
        fs::rename(&schemas, &held)?;
        symlink(&outside, &schemas)?;
        Ok(())
    })
    .unwrap();

    assert!(!outside.join("task-graph.json").exists());
    assert_eq!(fs::read(outside.join("sentinel")).unwrap(), b"untouched\n");
    assert_eq!(fs::read(held.join("task-graph.json")).unwrap(), b"{}\n");
    assert!(temporary_schema_entries(&held).is_empty());
}

#[test]
fn id_hash_and_order_contracts_are_available() {
    let first = TaskId::from_str("a").unwrap();
    let second = TaskId::from_str("b").unwrap();
    assert!(first < second);

    let mut ids = HashSet::new();
    ids.insert(first.clone());
    assert!(ids.contains(&first));
}
