use harp_artifacts::ArtifactStore;
use harp_contracts::{AttemptId, RunId, WorkflowV2};
use harp_engine::{
    resolve_workflow_inputs, validate_workflow_v2, AcceptedTaskOutputs, OutputSchemaRegistry,
    WorkloadOutcome,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn plan() -> harp_engine::ValidatedWorkflowV2 {
    let mut doc: Value = serde_json::from_str(include_str!("fixtures/workflow-v2.json")).unwrap();
    let mut child = doc["tasks"][0].clone();
    child["taskId"] = json!("evaluate");
    child["dependencies"] = json!([{"taskId":"train","condition":"success"}]);
    child["inputs"] = json!([{"name":"training","schema":"research.metrics.v1",
        "source":{"kind":"output","taskId":"train","output":"metrics"}}]);
    doc["tasks"].as_array_mut().unwrap().push(child);
    for key in ["cpuSeconds", "wallSeconds", "storageBytes"] {
        doc["limits"][key] = json!(doc["limits"][key].as_u64().unwrap() * 2);
    }
    validate_workflow_v2(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap())
        .unwrap()
}
fn schemas() -> OutputSchemaRegistry {
    OutputSchemaRegistry::new(BTreeMap::from([(
        "research.metrics.v1".into(),
        json!({
            "type":"object", "properties":{"loss":{"type":"number"}},
            "required":["loss"], "additionalProperties":false
        }),
    )]))
    .unwrap()
}
#[test]
fn binds_verified_accepted_output_and_exact_attempt_lineage() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("artifacts")).unwrap();
    let store = ArtifactStore::open(dir.path().join("artifacts")).unwrap();
    let mut artifact = store
        .publish(br#"{"loss":1.25}"#, "application/json")
        .unwrap();
    artifact.logical_schema = Some("research.metrics.v1".into());
    let run = RunId::new();
    let attempt = AttemptId::new();
    let accepted = AcceptedTaskOutputs {
        run_id: run.clone(),
        task_id: "train".parse().unwrap(),
        attempt_id: attempt.clone(),
        outcome: WorkloadOutcome::Succeeded,
        outputs: BTreeMap::from([("metrics".into(), artifact.clone())]),
        outcome_artifact: None,
    };
    let table = BTreeMap::from([("train".parse().unwrap(), accepted)]);
    let inputs = resolve_workflow_inputs(
        &plan(),
        &run,
        &"evaluate".parse().unwrap(),
        &table,
        &store,
        &schemas(),
    )
    .unwrap();
    assert_eq!(inputs.inputs[0].artifact, artifact);
    assert_eq!(inputs.inputs[0].producer_attempt_id, Some(attempt));
    assert_eq!(inputs.plan_sha256, plan().digest());
}
#[test]
fn rejects_failed_cross_run_missing_or_invalid_producer_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("artifacts")).unwrap();
    let store = ArtifactStore::open(dir.path().join("artifacts")).unwrap();
    let run = RunId::new();
    let mut artifact = store
        .publish(br#"{"wrong":true}"#, "application/json")
        .unwrap();
    artifact.logical_schema = Some("research.metrics.v1".into());
    let accepted = AcceptedTaskOutputs {
        run_id: run.clone(),
        task_id: "train".parse().unwrap(),
        attempt_id: AttemptId::new(),
        outcome: WorkloadOutcome::Succeeded,
        outputs: BTreeMap::from([("metrics".into(), artifact)]),
        outcome_artifact: None,
    };
    let check = |record: AcceptedTaskOutputs| {
        resolve_workflow_inputs(
            &plan(),
            &run,
            &"evaluate".parse().unwrap(),
            &BTreeMap::from([("train".parse().unwrap(), record)]),
            &store,
            &schemas(),
        )
        .is_err()
    };
    assert!(check(accepted.clone())); // Valid bytes and digest, wrong schema instance.
    let mut record = accepted.clone();
    record.outcome = WorkloadOutcome::Failed;
    assert!(check(record));
    let mut record = accepted.clone();
    record.run_id = RunId::new();
    assert!(check(record));
    let mut record = accepted;
    record.outputs.clear();
    assert!(check(record));
}
#[test]
fn registry_rejects_remote_references_before_artifact_validation() {
    assert!(OutputSchemaRegistry::new(BTreeMap::from([(
        "bad.v1".into(),
        json!({
            "$ref":"https://example.invalid/schema.json"
        })
    )]))
    .is_err());
}

#[test]
fn accepted_artifact_with_duplicate_json_keys_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("artifacts")).unwrap();
    let store = ArtifactStore::open(dir.path().join("artifacts")).unwrap();
    let run = RunId::new();
    let mut artifact = store
        .publish(br#"{"loss":999,"loss":1.0}"#, "application/json")
        .unwrap();
    artifact.logical_schema = Some("research.metrics.v1".into());
    let accepted = AcceptedTaskOutputs {
        run_id: run.clone(),
        task_id: "train".parse().unwrap(),
        attempt_id: AttemptId::new(),
        outcome: WorkloadOutcome::Succeeded,
        outputs: BTreeMap::from([("metrics".into(), artifact)]),
        outcome_artifact: None,
    };
    assert!(resolve_workflow_inputs(
        &plan(),
        &run,
        &"evaluate".parse().unwrap(),
        &BTreeMap::from([("train".parse().unwrap(), accepted)]),
        &store,
        &schemas()
    )
    .is_err());
}

#[test]
fn settled_terminal_artifact_must_match_authoritative_identity_and_verdict() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("artifacts")).unwrap();
    let store = ArtifactStore::open(dir.path().join("artifacts")).unwrap();
    let mut doc = serde_json::to_value(plan().workflow()).unwrap();
    doc["tasks"][1]["dependencies"][0]["condition"] = json!("settled");
    doc["tasks"][1]["inputs"][0] = json!({"name":"terminal", "schema":"harp.task-outcome.v1",
        "source":{"kind":"outcome", "taskId":"train"}});
    let plan =
        validate_workflow_v2(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap())
            .unwrap();
    let run = RunId::new();
    let attempt = AttemptId::new();
    let terminal = json!({"schemaVersion":1, "runId":run, "taskId":"train",
        "attemptId":attempt, "outcome":"failed", "reasonCode":"command.exit", "evidence":[]});
    let registry = OutputSchemaRegistry::new(BTreeMap::from([(
        "harp.task-outcome.v1".into(),
        json!({"type":"object"}),
    )]))
    .unwrap();
    let resolve = |value: Value| {
        let mut artifact = store
            .publish(&serde_json::to_vec(&value).unwrap(), "application/json")
            .unwrap();
        artifact.logical_schema = Some("harp.task-outcome.v1".into());
        let producer = AcceptedTaskOutputs {
            run_id: run.clone(),
            task_id: "train".parse().unwrap(),
            attempt_id: attempt.clone(),
            outcome: WorkloadOutcome::Failed,
            outputs: BTreeMap::new(),
            outcome_artifact: Some(artifact),
        };
        resolve_workflow_inputs(
            &plan,
            &run,
            &"evaluate".parse().unwrap(),
            &BTreeMap::from([("train".parse().unwrap(), producer)]),
            &store,
            &registry,
        )
    };
    assert!(resolve(terminal.clone()).is_ok());
    for (field, bad) in [
        ("taskId", json!("other")),
        ("runId", json!(RunId::new())),
        ("attemptId", json!(AttemptId::new())),
        ("outcome", json!("succeeded")),
    ] {
        let mut bad_terminal = terminal.clone();
        bad_terminal[field] = bad;
        assert!(resolve(bad_terminal).is_err(), "{field}");
    }
}

#[test]
fn input_and_output_exposure_share_one_storage_reservation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("artifacts")).unwrap();
    let store = ArtifactStore::open(dir.path().join("artifacts")).unwrap();
    let mut doc = serde_json::to_value(plan().workflow()).unwrap();
    doc["tasks"][1]["limits"]["storageBytes"] = json!(4096);
    let plan =
        validate_workflow_v2(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap())
            .unwrap();
    let run = RunId::new();
    let mut artifact = store
        .publish(br#"{"loss":1.25}"#, "application/json")
        .unwrap();
    artifact.logical_schema = Some("research.metrics.v1".into());
    let producer = AcceptedTaskOutputs {
        run_id: run.clone(),
        task_id: "train".parse().unwrap(),
        attempt_id: AttemptId::new(),
        outcome: WorkloadOutcome::Succeeded,
        outputs: BTreeMap::from([("metrics".into(), artifact)]),
        outcome_artifact: None,
    };
    assert!(resolve_workflow_inputs(
        &plan,
        &run,
        &"evaluate".parse().unwrap(),
        &BTreeMap::from([("train".parse().unwrap(), producer)]),
        &store,
        &schemas()
    )
    .is_err());
}
