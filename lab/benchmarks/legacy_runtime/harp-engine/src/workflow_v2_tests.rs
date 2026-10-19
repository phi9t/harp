use harp_contracts::WorkflowV2;
use harp_engine::validate_workflow_v2;
use serde_json::{json, Value};

fn plan() -> Value {
    serde_json::from_str(include_str!("fixtures/workflow-v2.json")).unwrap()
}
fn validate(doc: Value) -> bool {
    let plan = WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap();
    validate_workflow_v2(plan).is_ok()
}
fn two_tasks() -> Value {
    let mut doc = plan();
    let mut child = doc["tasks"][0].clone();
    child["taskId"] = json!("evaluate");
    child["dependencies"] = json!([{"taskId":"train", "condition":"success"}]);
    child["inputs"] = json!([{"name":"training", "schema":"research.metrics.v1",
        "source":{"kind":"output", "taskId":"train", "output":"metrics"}}]);
    doc["tasks"].as_array_mut().unwrap().push(child);
    doc["limits"]["cpuSeconds"] = json!(120);
    doc["limits"]["wallSeconds"] = json!(120);
    doc["limits"]["storageBytes"] = json!(2097152);
    doc
}

#[test]
fn native_single_task_validates_without_a_synthetic_reducer() {
    assert!(validate(plan()));
}
#[test]
fn explicit_producer_inputs_validate_and_plan_digest_is_stable() {
    let doc = two_tasks();
    let a =
        validate_workflow_v2(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).unwrap())
            .unwrap();
    let b = validate_workflow_v2(
        WorkflowV2::from_json(&serde_json::to_vec_pretty(&doc).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(a.digest(), b.digest());
    assert_eq!(
        a.topological_order()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["train", "evaluate"]
    );
}
#[test]
fn missing_cyclic_or_unrelated_producer_is_rejected() {
    let mut doc = two_tasks();
    doc["tasks"][1]["dependencies"][0]["taskId"] = json!("missing");
    assert!(!validate(doc));
    let mut doc = two_tasks();
    doc["tasks"][0]["dependencies"] = json!([{"taskId":"evaluate", "condition":"success"}]);
    assert!(!validate(doc));
    let mut doc = two_tasks();
    doc["tasks"][1]["dependencies"] = json!([]);
    assert!(!validate(doc));
}
#[test]
fn output_name_schema_and_terminal_report_must_match_producer() {
    for (field, value) in [("output", json!("absent")), ("taskId", json!("absent"))] {
        let mut doc = two_tasks();
        doc["tasks"][1]["inputs"][0]["source"][field] = value;
        assert!(!validate(doc));
    }
    let mut doc = two_tasks();
    doc["tasks"][1]["inputs"][0]["schema"] = json!("other.v1");
    assert!(!validate(doc));
    let mut doc = plan();
    doc["outputs"][0]["output"] = json!("missing");
    assert!(!validate(doc));
}
#[test]
fn cumulative_retry_reservations_must_fit_root_budget() {
    let mut doc = two_tasks();
    doc["tasks"][0]["maxRetries"] = json!(1);
    assert!(!validate(doc));
}
