use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use harp_contracts::{
    ArtifactRef, Budget, NodeKind, OperationId, RetryPolicy, TaskGraph, TaskId, TaskNode, TaskRole,
    WorkspaceMode,
};
use harp_engine::{
    project_child_context, role_name, validate_graph, workspace_mode_name, ApprovedArtifact,
    GraphPolicy, ProjectionError, ProjectionPolicy, ProjectionRequest,
};

const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn task_id(value: &str) -> TaskId {
    TaskId::from_str(value).expect("valid task id")
}

fn artifact(digest: &str, size_bytes: u64) -> ArtifactRef {
    ArtifactRef::sha256(digest, "application/json", size_bytes).expect("valid artifact")
}

fn analysis(id: &str) -> TaskNode {
    TaskNode {
        task_id: task_id(id),
        kind: NodeKind::Analysis,
        role: TaskRole::Explore,
        instruction: format!("Analyze {id}."),
        dependencies: Vec::new(),
        inputs: Vec::new(),
        workspace_mode: WorkspaceMode::ReadOnly,
        model_policy: "analysis-model".to_owned(),
        permission_profile: "read-only".to_owned(),
        budget: Budget::new(100, 10, 1_000),
        output_schema: "analysis-result".to_owned(),
        retry_policy: RetryPolicy {
            max_transient_attempts: 0,
        },
    }
}

fn reducer(id: &str, dependencies: &[&str]) -> TaskNode {
    TaskNode {
        task_id: task_id(id),
        kind: NodeKind::Reducer,
        role: TaskRole::Reduce,
        instruction: "Reduce child results.".to_owned(),
        dependencies: dependencies.iter().map(|id| task_id(id)).collect(),
        inputs: Vec::new(),
        workspace_mode: WorkspaceMode::ReadOnly,
        model_policy: "analysis-model".to_owned(),
        permission_profile: "read-only".to_owned(),
        budget: Budget::new(100, 10, 1_000),
        output_schema: "analysis-result".to_owned(),
        retry_policy: RetryPolicy {
            max_transient_attempts: 0,
        },
    }
}

fn graph(nodes: Vec<TaskNode>) -> TaskGraph {
    TaskGraph {
        schema_version: 1,
        nodes,
    }
}

fn policy() -> GraphPolicy {
    let artifact = artifact(DIGEST_A, 64);
    GraphPolicy {
        max_nodes: 64,
        max_concurrency: 2,
        max_total_tokens: 10_000,
        max_total_storage_bytes: 1_000_000,
        max_total_timeout_seconds: 1_000,
        max_node_tokens: 100,
        max_node_storage_bytes: 1_000,
        max_node_timeout_seconds: 10,
        max_projected_prompt_bytes: 1024 * 1024,
        max_recursion_depth: 1,
        allowed_roles: ["explore", "classify", "verify", "critic", "reduce"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        allowed_output_schemas: ["analysis-result"].into_iter().map(str::to_owned).collect(),
        allowed_model_policies: ["analysis-model"].into_iter().map(str::to_owned).collect(),
        allowed_permission_profiles: ["read-only"].into_iter().map(str::to_owned).collect(),
        allowed_workspace_modes: ["readOnly", "scratch"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        approved_artifacts: [(
            artifact.uri.clone(),
            ApprovedArtifact {
                reference: artifact,
                materialized_path: "/approved/input.json".to_owned(),
                read_only: true,
            },
        )]
        .into_iter()
        .collect(),
    }
}

#[path = "validation/bounds.rs"]
mod bounds;
#[path = "validation/cases.rs"]
mod cases;
#[path = "validation/projection.rs"]
mod projection;
