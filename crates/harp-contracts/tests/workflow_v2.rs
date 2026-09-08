use harp_contracts::{DynamicWorkflow, WorkflowV2};
use serde_json::{json, Value};

fn command() -> Value {
    json!({
        "schemaVersion": 2, "name": "cpu-research",
        "limits": {"agentTokens": 0, "cpuSeconds": 60, "gpuSeconds": 0,
                   "wallSeconds": 60, "storageBytes": 1048576, "maxConcurrency": 1},
        "failurePolicy": "failFast",
        "tasks": [{
            "taskId": "train", "role": "verify", "dependencies": [], "inputs": [],
            "backend": {"kind": "localCommand", "version": "1", "configSha256": "a".repeat(64)},
            "environmentSha256": "b".repeat(64),
            "action": {"kind": "command", "executable": "/usr/bin/true",
                       "executableSha256": "c".repeat(64), "arguments": [], "cwd": ".", "env": {}},
            "outputs": [{"name": "metrics", "path": "metrics.json", "schema": "research.metrics.v1", "maxBytes": 4096}],
            "limits": {"agentTokens": 0, "cpuSeconds": 60, "gpuSeconds": 0,
                       "wallSeconds": 60, "storageBytes": 1048576},
            "maxRetries": 0
        }],
        "outputs": [{"taskId": "train", "output": "metrics"}]
    })
}

#[test]
fn single_command_has_native_action_and_preserves_v1_boundary() {
    let raw = serde_json::to_vec(&command()).unwrap();
    let workflow = WorkflowV2::from_json(&raw).unwrap();
    assert_eq!(workflow.tasks.len(), 1);
    assert_eq!(workflow.tasks[0].task_id.to_string(), "train");
    assert!(serde_json::from_slice::<DynamicWorkflow>(&raw).is_err());
}

#[test]
fn admission_json_rejects_duplicate_keys_including_nested_objects() {
    let raw = serde_json::to_string(&command()).unwrap();
    for (from, to) in [
        (
            "\"name\":\"cpu-research\"",
            "\"name\":\"cpu-research\",\"name\":\"other\"",
        ),
        ("\"env\":{}", "\"env\":{\"X\":\"1\",\"X\":\"2\"}"),
    ] {
        assert!(WorkflowV2::from_json(raw.replace(from, to).as_bytes()).is_err());
    }
}

#[test]
fn canonical_bytes_ignore_object_key_order_but_preserve_array_order() {
    let raw = serde_json::to_vec(&command()).unwrap();
    let workflow = WorkflowV2::from_json(&raw).unwrap();
    let encoded = workflow.canonical_bytes().unwrap();
    let reparsed = WorkflowV2::from_json(&encoded).unwrap();
    assert_eq!(encoded, reparsed.canonical_bytes().unwrap());
    let pretty = serde_json::to_vec_pretty(&command()).unwrap();
    assert_eq!(
        encoded,
        WorkflowV2::from_json(&pretty)
            .unwrap()
            .canonical_bytes()
            .unwrap()
    );
}

#[test]
fn command_rejects_path_escape_unpinned_executable_and_unknown_fields() {
    for (field, value) in [
        ("cwd", json!("../operator")),
        ("executable", json!("python")),
        ("executableSha256", json!("C".repeat(64))),
        ("shell", json!(true)),
    ] {
        let mut doc = command();
        doc["tasks"][0]["action"][field] = value;
        assert!(
            WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).is_err(),
            "{field}"
        );
    }
}

#[test]
fn resource_limits_reject_missing_ceiling_and_retry_exposure_overflow() {
    let mut doc = command();
    doc["tasks"][0]["limits"]
        .as_object_mut()
        .unwrap()
        .remove("wallSeconds");
    assert!(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).is_err());
    let mut doc = command();
    doc["tasks"][0]["limits"]["cpuSeconds"] = json!(u64::MAX);
    doc["tasks"][0]["maxRetries"] = json!(1);
    assert!(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).is_err());
}

#[test]
fn graph_bound_and_duplicate_task_identity_are_enforced() {
    let mut doc = command();
    let task = doc["tasks"][0].clone();
    doc["tasks"] = json!([task.clone(), task.clone()]);
    assert!(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).is_err());
    doc["tasks"] = Value::Array(
        (0..65)
            .map(|i| {
                let mut task = task.clone();
                task["taskId"] = json!(format!("train-{i}"));
                task
            })
            .collect(),
    );
    assert!(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).is_err());
}

#[test]
fn agent_prompts_support_multiline_text() {
    let mut doc = command();
    doc["limits"]["agentTokens"] = json!(100);
    doc["tasks"][0]["limits"]["agentTokens"] = json!(100);
    doc["tasks"][0]["action"] = json!({"kind":"agent", "prompt":"Review evidence.\nExplain uncertainty.",
        "modelPolicy":"review", "permissionProfile":"read-only", "workspaceMode":"readOnly"});
    assert!(WorkflowV2::from_json(&serde_json::to_vec(&doc).unwrap()).is_ok());
}
