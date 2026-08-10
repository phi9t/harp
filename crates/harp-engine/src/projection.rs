use std::collections::BTreeMap;

use harp_artifacts::{ArtifactStoreIdentity, AttemptKey, ResolvedAttemptScratch};
use harp_contracts::{ArtifactRef, Budget, OperationId, TaskId, TaskNode, TaskRole};
use serde::{Deserialize, Serialize};

use crate::policy::GraphPolicy;
use crate::validate::{role_name, ValidatedGraph};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProjectionPolicy {
    pub base_instructions: String,
    pub role_instructions: BTreeMap<String, String>,
    pub checkpoint_instructions: String,
    pub max_bytes: usize,
    pub scratch_paths: BTreeMap<TaskId, String>,
}

#[derive(Clone, Debug)]
pub struct ProjectionRequest<'a> {
    pub task_id: &'a TaskId,
    pub operation_id: &'a OperationId,
    pub remaining_budget: Budget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedProjection {
    base_instructions: String,
    role_instructions: BTreeMap<TaskId, String>,
    checkpoint_instructions: String,
    scratch_paths: BTreeMap<TaskId, String>,
    inputs: BTreeMap<TaskId, Vec<ProjectedInput>>,
    max_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedInput {
    artifact: ArtifactRef,
    materialized_path: String,
    read_only: bool,
}

impl ProjectedInput {
    pub fn artifact(&self) -> &ArtifactRef {
        &self.artifact
    }

    pub fn materialized_path(&self) -> &str {
        &self.materialized_path
    }

    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedChildContext {
    schema_version: u8,
    task_id: TaskId,
    role: TaskRole,
    base_instructions: String,
    role_instructions: String,
    task_instruction: String,
    inputs: Vec<ProjectedInput>,
    scratch_path: String,
    operation_marker: OperationId,
    output_schema: String,
    checkpoint_instructions: String,
    budget: Budget,
    max_bytes: usize,
}

impl ProjectedChildContext {
    pub fn task_id(&self) -> &TaskId {
        &self.task_id
    }

    pub const fn role(&self) -> TaskRole {
        self.role
    }

    pub fn inputs(&self) -> &[ProjectedInput] {
        &self.inputs
    }

    pub fn scratch_path(&self) -> &str {
        &self.scratch_path
    }

    pub fn operation_marker(&self) -> &OperationId {
        &self.operation_marker
    }

    pub fn budget(&self) -> &Budget {
        &self.budget
    }

    pub fn to_bounded_json(&self) -> Result<BoundedProjectedJson, ProjectionError> {
        let inputs = self
            .inputs
            .iter()
            .map(|input| ProjectedInputJson {
                artifact: &input.artifact,
                materialized_path: &input.materialized_path,
                read_only: input.read_only,
            })
            .collect();
        let manifest = ProjectedChildContextJson {
            schema_version: self.schema_version,
            task_id: &self.task_id,
            role: self.role,
            base_instructions: &self.base_instructions,
            role_instructions: &self.role_instructions,
            task_instruction: &self.task_instruction,
            inputs,
            scratch_path: &self.scratch_path,
            operation_marker: &self.operation_marker,
            output_schema: &self.output_schema,
            checkpoint_instructions: &self.checkpoint_instructions,
            budget: &self.budget,
        };
        let mut bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|source| ProjectionError::Serialization { source })?;
        bytes.push(b'\n');
        if bytes.len() > self.max_bytes {
            return Err(ProjectionError::Size {
                actual_bytes: bytes.len(),
                max_bytes: self.max_bytes,
            });
        }
        Ok(BoundedProjectedJson { bytes })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedProjectedJson {
    bytes: Vec<u8>,
}

impl BoundedProjectedJson {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectedInputJson<'a> {
    artifact: &'a ArtifactRef,
    materialized_path: &'a str,
    read_only: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectedChildContextJson<'a> {
    schema_version: u8,
    task_id: &'a TaskId,
    role: TaskRole,
    base_instructions: &'a str,
    role_instructions: &'a str,
    task_instruction: &'a str,
    inputs: Vec<ProjectedInputJson<'a>>,
    scratch_path: &'a str,
    operation_marker: &'a OperationId,
    output_schema: &'a str,
    checkpoint_instructions: &'a str,
    budget: &'a Budget,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("task {task_id} is absent from the validated graph")]
    MissingTask { task_id: TaskId },
    #[error("remaining budget field {field} is outside the node reservation")]
    Budget { field: &'static str },
    #[error("execution scratch authority mismatch: {context}")]
    ScratchAuthority { context: &'static str },
    #[error("projected JSON uses {actual_bytes} bytes, exceeding {max_bytes}")]
    Size {
        actual_bytes: usize,
        max_bytes: usize,
    },
    #[error("could not serialize projected child context")]
    Serialization {
        #[source]
        source: serde_json::Error,
    },
}

pub fn project_child_context(
    validated: &ValidatedGraph,
    request: ProjectionRequest<'_>,
) -> Result<ProjectedChildContext, ProjectionError> {
    project_child_context_with_scratch(validated, request, None)
}

pub(crate) fn project_execution_child_context(
    validated: &ValidatedGraph,
    request: ProjectionRequest<'_>,
    expected_attempt: &AttemptKey,
    scratch: &ResolvedAttemptScratch,
    pinned_store: &ArtifactStoreIdentity,
) -> Result<ProjectedChildContext, ProjectionError> {
    if scratch.key() != expected_attempt || expected_attempt.task_id != *request.task_id {
        return Err(ProjectionError::ScratchAuthority {
            context: "attempt identity differs from projected task authority",
        });
    }
    if scratch.store_identity() != pinned_store {
        return Err(ProjectionError::ScratchAuthority {
            context: "artifact store differs from pinned execution store",
        });
    }
    let path = scratch
        .canonical_path()
        .to_str()
        .ok_or(ProjectionError::ScratchAuthority {
            context: "scratch path is not UTF-8",
        })?;
    if !crate::policy::is_safe_absolute_path(path) {
        return Err(ProjectionError::ScratchAuthority {
            context: "scratch path is not a normalized absolute path",
        });
    }
    project_child_context_with_scratch(validated, request, Some(path))
}

fn project_child_context_with_scratch(
    validated: &ValidatedGraph,
    request: ProjectionRequest<'_>,
    resolved_scratch_path: Option<&str>,
) -> Result<ProjectedChildContext, ProjectionError> {
    let node = validated
        .graph()
        .nodes
        .iter()
        .find(|node| &node.task_id == request.task_id)
        .ok_or_else(|| ProjectionError::MissingTask {
            task_id: request.task_id.clone(),
        })?;
    validate_remaining_budget(&request.remaining_budget, &node.budget)?;
    let projection = validated.projection();
    let scratch_path = if let Some(resolved) = resolved_scratch_path {
        resolved
    } else {
        projection
            .scratch_paths
            .get(&node.task_id)
            .expect("validated task has a scratch reservation")
    };

    let role_instructions = projection
        .role_instructions
        .get(&node.task_id)
        .expect("validated task has captured role instructions");
    let inputs = projection
        .inputs
        .get(&node.task_id)
        .expect("validated task has captured inputs")
        .clone();

    let projected = ProjectedChildContext {
        schema_version: 1,
        task_id: node.task_id.clone(),
        role: node.role,
        base_instructions: projection.base_instructions.clone(),
        role_instructions: role_instructions.clone(),
        task_instruction: node.instruction.clone(),
        inputs,
        scratch_path: scratch_path.to_owned(),
        operation_marker: request.operation_id.clone(),
        output_schema: node.output_schema.clone(),
        checkpoint_instructions: projection.checkpoint_instructions.clone(),
        budget: request.remaining_budget,
        max_bytes: projection.max_bytes,
    };
    projected.to_bounded_json()?;
    Ok(projected)
}

pub(crate) const MAX_PROJECTION_POLICY_TEXT_BYTES: usize = 64 * 1024;

pub(crate) fn estimate_static_projection_bytes(
    node: &TaskNode,
    graph_policy: &GraphPolicy,
    projection_policy: &ProjectionPolicy,
) -> Option<usize> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct StaticInput<'a> {
        artifact: &'a ArtifactRef,
        materialized_path: &'a str,
        read_only: bool,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct StaticContext<'a> {
        schema_version: u8,
        task_id: &'a TaskId,
        role: TaskRole,
        base_instructions: &'a str,
        role_instructions: &'a str,
        task_instruction: &'a str,
        inputs: Vec<StaticInput<'a>>,
        scratch_path: String,
        operation_marker: &'static str,
        output_schema: &'a str,
        checkpoint_instructions: &'a str,
        budget: &'a Budget,
    }

    let role_instructions = projection_policy
        .role_instructions
        .get(role_name(node.role))?;
    let inputs = node
        .inputs
        .iter()
        .map(|artifact| {
            let approved = graph_policy.approved_artifacts.get(&artifact.uri)?;
            Some(StaticInput {
                artifact,
                materialized_path: &approved.materialized_path,
                read_only: true,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let context = StaticContext {
        schema_version: 1,
        task_id: &node.task_id,
        role: node.role,
        base_instructions: &projection_policy.base_instructions,
        role_instructions,
        task_instruction: &node.instruction,
        inputs,
        scratch_path: projection_policy.scratch_paths.get(&node.task_id)?.clone(),
        operation_marker: "019fe5f2-34fa-78b0-93bf-fa77e69529dd",
        output_schema: &node.output_schema,
        checkpoint_instructions: &projection_policy.checkpoint_instructions,
        budget: &node.budget,
    };
    serde_json::to_vec_pretty(&context)
        .ok()?
        .len()
        .checked_add(1)
}

pub(crate) fn capture_validated_projection(
    graph: &harp_contracts::TaskGraph,
    graph_policy: &GraphPolicy,
    projection_policy: &ProjectionPolicy,
) -> ValidatedProjection {
    let role_instructions = graph
        .nodes
        .iter()
        .map(|node| {
            (
                node.task_id.clone(),
                projection_policy
                    .role_instructions
                    .get(role_name(node.role))
                    .expect("validated role has projection instructions")
                    .clone(),
            )
        })
        .collect();
    let inputs = graph
        .nodes
        .iter()
        .map(|node| {
            let projected = node
                .inputs
                .iter()
                .map(|input| {
                    let approved = graph_policy
                        .approved_artifacts
                        .get(&input.uri)
                        .expect("validated input is approved");
                    ProjectedInput {
                        artifact: input.clone(),
                        materialized_path: approved.materialized_path.clone(),
                        read_only: true,
                    }
                })
                .collect();
            (node.task_id.clone(), projected)
        })
        .collect();

    ValidatedProjection {
        base_instructions: projection_policy.base_instructions.clone(),
        role_instructions,
        checkpoint_instructions: projection_policy.checkpoint_instructions.clone(),
        scratch_paths: projection_policy.scratch_paths.clone(),
        inputs,
        max_bytes: graph_policy
            .max_projected_prompt_bytes
            .min(projection_policy.max_bytes),
    }
}

fn validate_remaining_budget(remaining: &Budget, reserved: &Budget) -> Result<(), ProjectionError> {
    validate_budget_field("maxTokens", remaining.max_tokens, reserved.max_tokens)?;
    validate_budget_field(
        "timeoutSeconds",
        remaining.timeout_seconds,
        reserved.timeout_seconds,
    )?;
    validate_budget_field(
        "maxStorageBytes",
        remaining.max_storage_bytes,
        reserved.max_storage_bytes,
    )
}

fn validate_budget_field(
    field: &'static str,
    remaining: u64,
    reserved: u64,
) -> Result<(), ProjectionError> {
    if remaining == 0 || remaining > reserved {
        return Err(ProjectionError::Budget { field });
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod authority_tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::str::FromStr;

    use harp_artifacts::{ArtifactStore, AttemptKey};
    use harp_contracts::{
        AttemptId, NodeKind, RetryPolicy, RunId, TaskGraph, TaskNode, WorkspaceMode,
    };
    use tempfile::TempDir;

    use super::*;
    use crate::{validate_graph, GraphPolicy};

    fn private_directory(prefix: &str) -> TempDir {
        let directory = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in("/private/tmp")
            .expect("temporary directory");
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
            .expect("private directory");
        directory
    }

    fn task(task_id: &str, dependencies: &[&str]) -> TaskNode {
        TaskNode {
            task_id: TaskId::from_str(task_id).unwrap(),
            kind: if task_id == "reduce" {
                NodeKind::Reducer
            } else {
                NodeKind::Analysis
            },
            role: if task_id == "reduce" {
                TaskRole::Reduce
            } else {
                TaskRole::Explore
            },
            instruction: format!("execute {task_id}"),
            dependencies: dependencies
                .iter()
                .map(|dependency| TaskId::from_str(dependency).unwrap())
                .collect(),
            inputs: Vec::new(),
            workspace_mode: WorkspaceMode::Scratch,
            model_policy: "test-model".to_owned(),
            permission_profile: "never".to_owned(),
            budget: Budget::new(100, 60, 2_048),
            output_schema: r#"{"type":"object"}"#.to_owned(),
            retry_policy: RetryPolicy {
                max_transient_attempts: 1,
            },
        }
    }

    fn validated() -> ValidatedGraph {
        let graph = TaskGraph {
            schema_version: 1,
            nodes: vec![
                task("alpha", &[]),
                task("beta", &[]),
                task("reduce", &["alpha", "beta"]),
            ],
        };
        let graph_policy = GraphPolicy {
            max_nodes: 8,
            max_concurrency: 2,
            max_total_tokens: 1_000,
            max_total_storage_bytes: 10_000,
            max_total_timeout_seconds: 3_600,
            max_node_tokens: 100,
            max_node_storage_bytes: 2_048,
            max_node_timeout_seconds: 60,
            max_projected_prompt_bytes: 1024 * 1024,
            max_recursion_depth: 1,
            allowed_roles: BTreeSet::from(["explore".to_owned(), "reduce".to_owned()]),
            allowed_output_schemas: BTreeSet::from([r#"{"type":"object"}"#.to_owned()]),
            allowed_model_policies: BTreeSet::from(["test-model".to_owned()]),
            allowed_permission_profiles: BTreeSet::from(["never".to_owned()]),
            allowed_workspace_modes: BTreeSet::from(["scratch".to_owned()]),
            approved_artifacts: BTreeMap::new(),
        };
        let projection_policy = ProjectionPolicy {
            base_instructions: "base".to_owned(),
            role_instructions: BTreeMap::from([
                ("explore".to_owned(), "explore".to_owned()),
                ("reduce".to_owned(), "reduce".to_owned()),
            ]),
            checkpoint_instructions: "checkpoint".to_owned(),
            max_bytes: 1024 * 1024,
            scratch_paths: BTreeMap::from([
                (
                    TaskId::from_str("alpha").unwrap(),
                    "/logical/alpha".to_owned(),
                ),
                (
                    TaskId::from_str("beta").unwrap(),
                    "/logical/beta".to_owned(),
                ),
                (
                    TaskId::from_str("reduce").unwrap(),
                    "/logical/reduce".to_owned(),
                ),
            ]),
        };
        validate_graph(graph, &graph_policy, &projection_policy).unwrap()
    }

    fn store(directory: &TempDir, name: &str) -> ArtifactStore {
        let root = directory.path().join(name);
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        ArtifactStore::open(root).unwrap()
    }

    fn key(task_id: &str) -> AttemptKey {
        AttemptKey {
            run_id: RunId::new(),
            task_id: TaskId::from_str(task_id).unwrap(),
            attempt_id: AttemptId::new(),
        }
    }

    #[test]
    fn execution_projection_rejects_wrong_task_scratch_authority() {
        let directory = private_directory("harp-projection-task-authority-");
        let artifacts = store(&directory, "artifacts");
        let expected = key("alpha");
        let foreign_task = AttemptKey {
            run_id: expected.run_id.clone(),
            task_id: TaskId::from_str("beta").unwrap(),
            attempt_id: expected.attempt_id.clone(),
        };
        let authority = artifacts.resolve_attempt_scratch(&foreign_task).unwrap();
        let task_id = TaskId::from_str("alpha").unwrap();
        let operation_id = OperationId::new();

        let error = project_execution_child_context(
            &validated(),
            ProjectionRequest {
                task_id: &task_id,
                operation_id: &operation_id,
                remaining_budget: Budget::new(100, 60, 2_048),
            },
            &expected,
            &authority,
            &artifacts.identity().unwrap(),
        )
        .expect_err("wrong-task scratch authority");

        assert!(matches!(error, ProjectionError::ScratchAuthority { .. }));
    }

    #[test]
    fn execution_projection_rejects_wrong_store_scratch_authority() {
        let directory = private_directory("harp-projection-store-authority-");
        let pinned = store(&directory, "pinned");
        let foreign = store(&directory, "foreign");
        let expected = key("alpha");
        let authority = foreign.resolve_attempt_scratch(&expected).unwrap();
        let task_id = TaskId::from_str("alpha").unwrap();
        let operation_id = OperationId::new();

        let error = project_execution_child_context(
            &validated(),
            ProjectionRequest {
                task_id: &task_id,
                operation_id: &operation_id,
                remaining_budget: Budget::new(100, 60, 2_048),
            },
            &expected,
            &authority,
            &pinned.identity().unwrap(),
        )
        .expect_err("foreign-store scratch authority");

        assert!(matches!(error, ProjectionError::ScratchAuthority { .. }));
    }

    #[test]
    fn execution_projection_uses_verified_attempt_scratch_authority() {
        let directory = private_directory("harp-projection-valid-authority-");
        let artifacts = store(&directory, "artifacts");
        let expected = key("alpha");
        let authority = artifacts.resolve_attempt_scratch(&expected).unwrap();
        let task_id = TaskId::from_str("alpha").unwrap();
        let operation_id = OperationId::new();

        let projected = project_execution_child_context(
            &validated(),
            ProjectionRequest {
                task_id: &task_id,
                operation_id: &operation_id,
                remaining_budget: Budget::new(100, 60, 2_048),
            },
            &expected,
            &authority,
            &artifacts.identity().unwrap(),
        )
        .expect("valid scratch authority");

        assert_eq!(
            projected.scratch_path(),
            authority.canonical_path().to_str().unwrap()
        );
    }
}
