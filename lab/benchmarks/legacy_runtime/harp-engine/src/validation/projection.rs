use super::*;

fn projection_policy() -> ProjectionPolicy {
    ProjectionPolicy {
        base_instructions: "BASE_ONLY".to_owned(),
        role_instructions: [
            ("explore".to_owned(), "EXPLORE_ONLY".to_owned()),
            ("reduce".to_owned(), "REDUCE_SENTINEL".to_owned()),
        ]
        .into_iter()
        .collect(),
        checkpoint_instructions: "CHECKPOINT_ONLY".to_owned(),
        max_bytes: 1024 * 1024,
        scratch_paths: [
            (task_id("child"), "/scratch/run/child".to_owned()),
            (task_id("sibling"), "/scratch/run/sibling".to_owned()),
            (task_id("reduce"), "/scratch/run/reduce".to_owned()),
        ]
        .into_iter()
        .collect(),
    }
}

fn operation_id() -> OperationId {
    "019fe5f2-34fa-78b0-93bf-fa77e69529dd"
        .parse()
        .expect("valid UUIDv7 operation ID")
}

fn validated_with_input() -> (harp_engine::ValidatedGraph, GraphPolicy, ProjectionPolicy) {
    let graph_policy = policy();
    let approved = graph_policy
        .approved_artifacts
        .values()
        .next()
        .expect("approved artifact")
        .reference
        .clone();
    let mut child = analysis("child");
    child.inputs.push(approved);
    let graph = graph(vec![
        child,
        analysis("sibling"),
        reducer("reduce", &["child", "sibling"]),
    ]);
    let projection_policy = projection_policy();
    let validated = validate_graph(graph, &graph_policy, &projection_policy)
        .expect("valid graph with approved input");
    (validated, graph_policy, projection_policy)
}

fn request<'a>(task_id: &'a TaskId, operation_id: &'a OperationId) -> ProjectionRequest<'a> {
    ProjectionRequest {
        task_id,
        operation_id,
        remaining_budget: Budget::new(80, 8, 800),
    }
}

#[test]
fn projects_only_explicit_child_fields_and_selected_role() {
    let root_transcript_sentinel = "ROOT_TRANSCRIPT_SENTINEL";
    let sibling_transcript_sentinel = "SIBLING_TRANSCRIPT_SENTINEL";
    let (validated, _, _) = validated_with_input();
    let task_id = task_id("child");
    let operation_id = operation_id();
    let projected = project_child_context(&validated, request(&task_id, &operation_id))
        .expect("project child context");

    assert_eq!(projected.task_id(), &task_id);
    assert_eq!(projected.role(), TaskRole::Explore);
    assert_eq!(projected.inputs().len(), 1);
    assert_eq!(
        projected.inputs()[0].materialized_path(),
        "/approved/input.json"
    );
    assert!(projected.inputs()[0].is_read_only());
    assert_eq!(projected.scratch_path(), "/scratch/run/child");
    assert_eq!(projected.operation_marker(), &operation_id);
    assert_eq!(projected.budget(), &Budget::new(80, 8, 800));

    let bounded = projected.to_bounded_json().expect("bounded canonical JSON");
    let json = std::str::from_utf8(bounded.as_bytes()).expect("UTF-8 JSON");
    assert!(json.ends_with('\n'));
    assert!(!json.contains(root_transcript_sentinel));
    assert!(!json.contains(sibling_transcript_sentinel));
    assert!(!json.contains("REDUCE_SENTINEL"));
    assert!(!json.contains("\"dependencies\""));
    assert!(!json.contains("\"modelPolicy\""));
    assert!(!json.contains("\"permissionProfile\""));
}

#[test]
fn projection_json_is_stable_camel_case_and_strict() {
    let (validated, _, _) = validated_with_input();
    let task_id = task_id("child");
    let operation_id = operation_id();
    let projected = project_child_context(&validated, request(&task_id, &operation_id))
        .expect("project child context");

    let first = projected.to_bounded_json().expect("first JSON");
    let second = projected.to_bounded_json().expect("second JSON");
    assert_eq!(first, second);
    let json = std::str::from_utf8(first.as_bytes()).expect("UTF-8 JSON");
    assert!(json.contains("\"schemaVersion\": 1"));
    assert!(json.contains("\"taskId\": \"child\""));
    assert!(json.contains("\"operationMarker\""));
    assert!(json.contains("\"materializedPath\": \"/approved/input.json\""));

    let parsed: serde_json::Value = serde_json::from_str(json).expect("parse projected JSON");
    assert_eq!(parsed["taskId"], "child");
}

#[test]
fn projection_preserves_node_input_order() {
    let mut graph_policy = policy();
    let second = artifact(
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        32,
    );
    graph_policy.approved_artifacts.insert(
        second.uri.clone(),
        ApprovedArtifact {
            reference: second.clone(),
            materialized_path: "/approved/second.json".to_owned(),
            read_only: true,
        },
    );
    let first = graph_policy
        .approved_artifacts
        .values()
        .find(|approved| approved.reference.uri != second.uri)
        .expect("first artifact")
        .reference
        .clone();
    let mut child = analysis("child");
    child.inputs = vec![second.clone(), first.clone()];
    let graph = graph(vec![
        child,
        analysis("sibling"),
        reducer("reduce", &["child", "sibling"]),
    ]);
    let projection_policy = projection_policy();
    let validated = validate_graph(graph, &graph_policy, &projection_policy)
        .expect("valid graph with ordered inputs");
    let task_id = task_id("child");
    let operation_id = operation_id();

    let projected = project_child_context(&validated, request(&task_id, &operation_id))
        .expect("project child context");

    assert_eq!(
        projected
            .inputs()
            .iter()
            .map(|input| input.artifact().uri.as_str())
            .collect::<Vec<_>>(),
        vec![second.uri.as_str(), first.uri.as_str()]
    );
}

#[test]
fn projection_rejects_missing_task_and_missing_role_instructions() {
    let (validated, _, _) = validated_with_input();
    let missing = task_id("missing");
    let operation_id = operation_id();
    let error = project_child_context(&validated, request(&missing, &operation_id))
        .expect_err("missing task");
    assert!(matches!(error, ProjectionError::MissingTask { .. }));

    let mut no_role = projection_policy();
    no_role.role_instructions.remove("explore");
    let graph = graph(vec![
        analysis("child"),
        analysis("sibling"),
        reducer("reduce", &["child", "sibling"]),
    ]);
    let error = validate_graph(graph, &policy(), &no_role).expect_err("missing role instructions");
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "projection.missing_role_instructions"));
    assert!(!error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "projection.prompt_bytes"));
}

#[test]
fn projection_uses_only_the_validated_reserved_scratch_path() {
    let (validated, _, _) = validated_with_input();
    let task_id = task_id("child");
    let operation_id = operation_id();
    let projected = project_child_context(&validated, request(&task_id, &operation_id))
        .expect("project reserved scratch path");
    assert_eq!(projected.scratch_path(), "/scratch/run/child");
}

#[test]
fn projection_rejects_remaining_budget_outside_node_reservation() {
    let (validated, _, _) = validated_with_input();
    let task_id = task_id("child");
    let operation_id = operation_id();

    for remaining_budget in [
        Budget::new(0, 8, 800),
        Budget::new(101, 8, 800),
        Budget::new(80, 0, 800),
        Budget::new(80, 11, 800),
        Budget::new(80, 8, 0),
        Budget::new(80, 8, 1_001),
    ] {
        let error = project_child_context(
            &validated,
            ProjectionRequest {
                task_id: &task_id,
                operation_id: &operation_id,
                remaining_budget,
            },
        )
        .expect_err("invalid remaining budget");
        assert!(matches!(error, ProjectionError::Budget { .. }));
    }
}

#[test]
fn projection_retains_validated_serialization_bound() {
    let (validated, _, mut projection_policy) = validated_with_input();
    let task_id = task_id("child");
    let operation_id = operation_id();
    let projected = project_child_context(&validated, request(&task_id, &operation_id))
        .expect("project child context");
    let before = projected.to_bounded_json().expect("bounded JSON");
    projection_policy.max_bytes = usize::MAX;
    let after = projected
        .to_bounded_json()
        .expect("caller cannot replace retained bound");
    assert_eq!(before, after);
    assert!(after.len() <= 1024 * 1024);
}

#[test]
fn projection_cannot_substitute_validated_policy_or_paths() {
    let (validated, mut graph_policy, mut projection_policy) = validated_with_input();
    let approved = graph_policy
        .approved_artifacts
        .values_mut()
        .next()
        .expect("approved artifact");
    approved.materialized_path = "/scratch/run/sibling/writable-input".to_owned();
    approved.read_only = false;
    projection_policy
        .scratch_paths
        .insert(task_id("child"), "/scratch/run/sibling/child".to_owned());
    projection_policy
        .role_instructions
        .insert("explore".to_owned(), "MUTATED_ROLE_SENTINEL".to_owned());

    let operation_id = operation_id();
    let child = project_child_context(&validated, request(&task_id("child"), &operation_id))
        .expect("project child from retained validated facts");
    let sibling = project_child_context(&validated, request(&task_id("sibling"), &operation_id))
        .expect("project sibling from retained validated facts");

    assert_eq!(child.scratch_path(), "/scratch/run/child");
    assert_eq!(sibling.scratch_path(), "/scratch/run/sibling");
    assert_eq!(
        child.inputs()[0].materialized_path(),
        "/approved/input.json"
    );
    assert!(child.inputs()[0].is_read_only());
    let bounded = child.to_bounded_json().expect("bounded child JSON");
    let json = std::str::from_utf8(bounded.as_bytes()).expect("UTF-8 child JSON");
    assert!(json.contains("\"roleInstructions\": \"EXPLORE_ONLY\""));
    assert!(!json.contains("MUTATED_ROLE_SENTINEL"));
    assert!(!child
        .scratch_path()
        .starts_with(&format!("{}/", sibling.scratch_path())));
    assert!(!sibling
        .scratch_path()
        .starts_with(&format!("{}/", child.scratch_path())));
}
