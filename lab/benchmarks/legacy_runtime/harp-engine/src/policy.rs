#[cfg(test)]
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use harp_contracts::{ArtifactRef, TaskId, TaskNode};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::diagnostic::{Diagnostics, ValidationDiagnostic};
use crate::projection::{ProjectionPolicy, MAX_PROJECTION_POLICY_TEXT_BYTES};

const MAX_POLICY_NODES: usize = 64;
const MAX_PROJECTED_PROMPT_BYTES: usize = 1024 * 1024;
const MAX_POLICY_STRING_BYTES: usize = 4096;
const MAX_APPROVED_ARTIFACTS: usize = 4096;
const MAX_APPROVED_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024 * 1024;
const MAX_PATH_BYTES: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GraphPolicy {
    pub max_nodes: usize,
    pub max_concurrency: usize,
    pub max_total_tokens: u64,
    pub max_total_storage_bytes: u64,
    pub max_total_timeout_seconds: u64,
    pub max_node_tokens: u64,
    pub max_node_storage_bytes: u64,
    pub max_node_timeout_seconds: u64,
    pub max_projected_prompt_bytes: usize,
    pub max_recursion_depth: u8,
    pub allowed_roles: BTreeSet<String>,
    pub allowed_output_schemas: BTreeSet<String>,
    pub allowed_model_policies: BTreeSet<String>,
    pub allowed_permission_profiles: BTreeSet<String>,
    pub allowed_workspace_modes: BTreeSet<String>,
    pub approved_artifacts: BTreeMap<String, ApprovedArtifact>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ApprovedArtifact {
    pub reference: ArtifactRef,
    pub materialized_path: String,
    pub read_only: bool,
}

pub(crate) fn validate_policy(
    policy: &GraphPolicy,
    projection_policy: &ProjectionPolicy,
    diagnostics: &mut Diagnostics,
) {
    if !(1..=MAX_POLICY_NODES).contains(&policy.max_nodes) {
        invalid_policy(
            diagnostics,
            "maxNodes",
            "maximum nodes must be between 1 and 64",
        );
    }
    if policy.max_concurrency == 0 || policy.max_concurrency > policy.max_nodes {
        invalid_policy(
            diagnostics,
            "maxConcurrency",
            "maximum concurrency must be between 1 and maximum nodes",
        );
    }
    if policy.max_total_tokens == 0 {
        invalid_policy(
            diagnostics,
            "maxTotalTokens",
            "maximum total tokens must be nonzero",
        );
    }
    if policy.max_total_storage_bytes == 0 {
        invalid_policy(
            diagnostics,
            "maxTotalStorageBytes",
            "maximum total storage must be nonzero",
        );
    }
    if policy.max_total_timeout_seconds == 0 {
        invalid_policy(
            diagnostics,
            "maxTotalTimeoutSeconds",
            "maximum total timeout must be nonzero",
        );
    }
    validate_node_ceiling(
        policy.max_node_tokens,
        policy.max_total_tokens,
        "maxNodeTokens",
        diagnostics,
    );
    validate_node_ceiling(
        policy.max_node_storage_bytes,
        policy.max_total_storage_bytes,
        "maxNodeStorageBytes",
        diagnostics,
    );
    validate_node_ceiling(
        policy.max_node_timeout_seconds,
        policy.max_total_timeout_seconds,
        "maxNodeTimeoutSeconds",
        diagnostics,
    );
    if !(1..=MAX_PROJECTED_PROMPT_BYTES).contains(&policy.max_projected_prompt_bytes) {
        invalid_policy(
            diagnostics,
            "maxProjectedPromptBytes",
            "maximum projected prompt bytes must be between 1 and 1 MiB",
        );
    }
    if policy.max_recursion_depth != 1 {
        diagnostics.push(ValidationDiagnostic::new(
            "recursion.depth",
            None,
            Some("maxRecursionDepth"),
            "first-slice recursion depth must equal one",
        ));
    }

    validate_allowlist("allowedRoles", &policy.allowed_roles, diagnostics);
    validate_allowlist(
        "allowedOutputSchemas",
        &policy.allowed_output_schemas,
        diagnostics,
    );
    validate_allowlist(
        "allowedModelPolicies",
        &policy.allowed_model_policies,
        diagnostics,
    );
    validate_allowlist(
        "allowedPermissionProfiles",
        &policy.allowed_permission_profiles,
        diagnostics,
    );
    validate_allowlist(
        "allowedWorkspaceModes",
        &policy.allowed_workspace_modes,
        diagnostics,
    );

    let Some(approved_paths) = validate_approved_artifacts(policy, diagnostics) else {
        return;
    };
    validate_projection_policy(&approved_paths, projection_policy, diagnostics);
}

pub(crate) fn validate_scratch_task_set(
    nodes: &BTreeMap<TaskId, &TaskNode>,
    projection_policy: &ProjectionPolicy,
    diagnostics: &mut Diagnostics,
) {
    for task_id in nodes.keys() {
        if !projection_policy.scratch_paths.contains_key(task_id) {
            diagnostics.push(ValidationDiagnostic::new(
                "projection.scratch_path",
                Some(task_id),
                Some("scratchPaths"),
                "graph task is missing a scratch reservation",
            ));
        }
    }
    for task_id in projection_policy.scratch_paths.keys() {
        if !nodes.contains_key(task_id) {
            diagnostics.push(ValidationDiagnostic::new(
                "projection.scratch_path",
                Some(task_id),
                Some("scratchPaths"),
                "scratch reservation does not identify a graph task",
            ));
        }
    }
}

pub(crate) fn is_safe_absolute_path(path: &str) -> bool {
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || !path.starts_with('/')
        || path.contains('\0')
        || path.chars().any(char::is_control)
        || (path.len() > 1 && path.ends_with('/'))
        || path.contains('\\')
    {
        return false;
    }

    path.split('/')
        .skip(1)
        .all(|component| !component.is_empty() && !matches!(component, "." | ".."))
        || path == "/"
}

#[derive(Debug)]
struct ApprovedPathIndex {
    paths: BTreeSet<Vec<String>>,
}

impl ApprovedPathIndex {
    fn overlaps(&self, candidate: &[String]) -> bool {
        if (0..=candidate.len()).any(|length| self.paths.contains(&candidate[..length])) {
            return true;
        }
        self.paths
            .range(candidate.to_vec()..)
            .next()
            .is_some_and(|path| is_component_ancestor(candidate, path))
    }
}

fn validate_approved_artifacts(
    policy: &GraphPolicy,
    diagnostics: &mut Diagnostics,
) -> Option<ApprovedPathIndex> {
    if policy.approved_artifacts.len() > MAX_APPROVED_ARTIFACTS {
        invalid_policy(
            diagnostics,
            "approvedArtifacts",
            "approved artifacts exceed the policy maximum",
        );
        return None;
    }

    let mut valid = true;
    let mut artifact_bytes = Some(0u64);
    let mut artifact_paths = Vec::with_capacity(policy.approved_artifacts.len());
    for (uri, approved) in &policy.approved_artifacts {
        if uri != &approved.reference.uri {
            valid = false;
            invalid_policy(
                diagnostics,
                "approvedArtifacts",
                "approved artifact key must equal its reference URI",
            );
        }
        if approved.reference.validate().is_err() {
            valid = false;
            invalid_policy(
                diagnostics,
                "approvedArtifacts",
                "approved artifact reference is invalid",
            );
        }
        if !approved.read_only {
            valid = false;
            invalid_policy(
                diagnostics,
                "approvedArtifacts",
                "approved artifacts must be read-only",
            );
        }
        if !is_safe_absolute_path(&approved.materialized_path) {
            valid = false;
            invalid_policy(
                diagnostics,
                "approvedArtifacts",
                "approved artifact path is not a normalized absolute path",
            );
        }
        artifact_paths.push(path_identity(&approved.materialized_path));
        artifact_bytes =
            artifact_bytes.and_then(|total| total.checked_add(approved.reference.size_bytes));
    }
    artifact_paths.sort();
    if artifact_paths
        .windows(2)
        .any(|pair| is_component_ancestor(&pair[0], &pair[1]))
    {
        valid = false;
        diagnostics.push(ValidationDiagnostic::new(
            "artifact.path_conflict",
            None,
            Some("materializedPath"),
            "distinct approved artifacts have colliding materialized paths",
        ));
    }
    if artifact_bytes.is_none_or(|total| total > MAX_APPROVED_ARTIFACT_BYTES) {
        valid = false;
        invalid_policy(
            diagnostics,
            "approvedArtifacts",
            "approved artifact declared bytes exceed 64 GiB",
        );
    }
    valid.then(|| ApprovedPathIndex {
        paths: artifact_paths.into_iter().collect(),
    })
}

fn validate_projection_policy(
    approved_paths: &ApprovedPathIndex,
    policy: &ProjectionPolicy,
    diagnostics: &mut Diagnostics,
) {
    if !(1..=MAX_PROJECTED_PROMPT_BYTES).contains(&policy.max_bytes) {
        invalid_policy(
            diagnostics,
            "projection.maxBytes",
            "projection maximum bytes must be between 1 and 1 MiB",
        );
    }
    for (field, value) in [
        (
            "projection.baseInstructions",
            policy.base_instructions.as_str(),
        ),
        (
            "projection.checkpointInstructions",
            policy.checkpoint_instructions.as_str(),
        ),
    ] {
        if value.len() > MAX_PROJECTION_POLICY_TEXT_BYTES || contains_forbidden_control(value) {
            invalid_policy(diagnostics, field, "projection instruction text is invalid");
        }
    }
    for (role, instructions) in &policy.role_instructions {
        if !valid_policy_string(role, 256)
            || instructions.is_empty()
            || instructions.len() > MAX_PROJECTION_POLICY_TEXT_BYTES
            || contains_forbidden_control(instructions)
        {
            invalid_policy(
                diagnostics,
                "projection.roleInstructions",
                "role projection instructions are invalid",
            );
        }
    }

    let reservations = policy
        .scratch_paths
        .iter()
        .map(|(task_id, scratch_path)| {
            (
                task_id,
                scratch_path,
                is_safe_absolute_path(scratch_path).then(|| path_identity(scratch_path)),
            )
        })
        .collect::<Vec<_>>();
    for (task_id, _scratch_path, identity) in &reservations {
        let Some(identity) = identity else {
            diagnostics.push(ValidationDiagnostic::new(
                "projection.scratch_path",
                Some(task_id),
                Some("scratchPaths"),
                "scratch reservation is not a normalized absolute path",
            ));
            continue;
        };
        if approved_paths.overlaps(identity) {
            diagnostics.push(ValidationDiagnostic::new(
                "projection.scratch_path",
                Some(task_id),
                Some("scratchPaths"),
                "scratch reservation overlaps an approved artifact path",
            ));
        }
    }
    for (index, (left_task, _left_path, left_identity)) in reservations.iter().enumerate() {
        let Some(left_identity) = left_identity else {
            continue;
        };
        for (right_task, _right_path, right_identity) in reservations.iter().skip(index + 1) {
            let Some(right_identity) = right_identity else {
                continue;
            };
            if identities_overlap(left_identity, right_identity) {
                diagnostics.push(ValidationDiagnostic::new(
                    "projection.scratch_path",
                    Some(left_task),
                    Some("scratchPaths"),
                    format!("scratch reservation overlaps task {right_task}"),
                ));
            }
        }
    }
}

fn identities_overlap(left: &[String], right: &[String]) -> bool {
    is_component_ancestor(left, right) || is_component_ancestor(right, left)
}

fn validate_allowlist(
    field: &'static str,
    values: &BTreeSet<String>,
    diagnostics: &mut Diagnostics,
) {
    if values.is_empty() {
        invalid_policy(diagnostics, field, "allowlist must not be empty");
    }
    for value in values {
        if !valid_policy_string(value, MAX_POLICY_STRING_BYTES) {
            invalid_policy(diagnostics, field, "allowlist entry is invalid");
        }
    }
}

fn validate_node_ceiling(
    ceiling: u64,
    total: u64,
    field: &'static str,
    diagnostics: &mut Diagnostics,
) {
    if ceiling == 0 || ceiling > total {
        invalid_policy(
            diagnostics,
            field,
            "per-node maximum must be nonzero and no greater than the total maximum",
        );
    }
}

fn invalid_policy(diagnostics: &mut Diagnostics, field: &'static str, message: &'static str) {
    diagnostics.push(ValidationDiagnostic::new(
        "policy.invalid",
        None,
        Some(field),
        message,
    ));
}

fn valid_policy_string(value: &str, max_bytes: usize) -> bool {
    !value.is_empty() && value.len() <= max_bytes && !contains_forbidden_control(value)
}

fn contains_forbidden_control(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\t'))
}

fn path_identity(path: &str) -> Vec<String> {
    #[cfg(test)]
    record_path_identity_call();
    if path == "/" {
        return Vec::new();
    }
    path.split('/')
        .skip(1)
        .map(|component| component.nfc().flat_map(char::to_lowercase).collect())
        .collect()
}

fn is_component_ancestor(ancestor: &[String], descendant: &[String]) -> bool {
    ancestor.len() <= descendant.len()
        && ancestor
            .iter()
            .zip(descendant)
            .all(|(left, right)| left == right)
}

#[cfg(test)]
thread_local! {
    static PATH_IDENTITY_CALLS: Cell<usize> = const { Cell::new(0) };
    static PATH_IDENTITY_BUDGET: Cell<usize> = const { Cell::new(usize::MAX) };
}

#[cfg(test)]
fn record_path_identity_call() {
    PATH_IDENTITY_CALLS.with(|calls| {
        let count = calls.get();
        PATH_IDENTITY_BUDGET.with(|budget| {
            assert!(
                count < budget.get(),
                "path identity normalization budget exceeded"
            );
        });
        calls.set(count + 1);
    });
}

#[cfg(test)]
struct PathIdentityBudget;

#[cfg(test)]
impl PathIdentityBudget {
    fn new(max_calls: usize) -> Self {
        PATH_IDENTITY_CALLS.with(|calls| calls.set(0));
        PATH_IDENTITY_BUDGET.with(|budget| budget.set(max_calls));
        Self
    }

    fn calls(&self) -> usize {
        PATH_IDENTITY_CALLS.with(Cell::get)
    }
}

#[cfg(test)]
impl Drop for PathIdentityBudget {
    fn drop(&mut self) {
        PATH_IDENTITY_BUDGET.with(|budget| budget.set(usize::MAX));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection::ProjectionPolicy;
    use crate::validate::validate_graph;
    use harp_contracts::{
        Budget, NodeKind, RetryPolicy, TaskGraph, TaskNode, TaskRole, WorkspaceMode,
    };
    use std::str::FromStr;

    fn artifact_policy(count: usize) -> GraphPolicy {
        let approved_artifacts = (0..count)
            .map(|index| {
                let reference =
                    ArtifactRef::sha256(format!("{index:064x}"), "application/octet-stream", 1)
                        .expect("valid test artifact");
                (
                    reference.uri.clone(),
                    ApprovedArtifact {
                        reference,
                        materialized_path: format!("/approved/items/{index:04x}"),
                        read_only: true,
                    },
                )
            })
            .collect();
        GraphPolicy {
            max_nodes: 64,
            max_concurrency: 1,
            max_total_tokens: 1,
            max_total_storage_bytes: 1,
            max_total_timeout_seconds: 1,
            max_node_tokens: 1,
            max_node_storage_bytes: 1,
            max_node_timeout_seconds: 1,
            max_projected_prompt_bytes: 1,
            max_recursion_depth: 1,
            allowed_roles: BTreeSet::new(),
            allowed_output_schemas: BTreeSet::new(),
            allowed_model_policies: BTreeSet::new(),
            allowed_permission_profiles: BTreeSet::new(),
            allowed_workspace_modes: BTreeSet::new(),
            approved_artifacts,
        }
    }

    fn task_id(value: &str) -> TaskId {
        TaskId::from_str(value).expect("valid task ID")
    }

    fn node(id: &str, kind: NodeKind, role: TaskRole, dependencies: &[&str]) -> TaskNode {
        TaskNode {
            task_id: task_id(id),
            kind,
            role,
            instruction: format!("Run {id}."),
            dependencies: dependencies
                .iter()
                .map(|dependency| task_id(dependency))
                .collect(),
            inputs: Vec::new(),
            workspace_mode: WorkspaceMode::ReadOnly,
            model_policy: "model".to_owned(),
            permission_profile: "read-only".to_owned(),
            budget: Budget::new(1, 1, 1),
            output_schema: "result".to_owned(),
            retry_policy: RetryPolicy {
                max_transient_attempts: 0,
            },
        }
    }

    fn graph() -> TaskGraph {
        TaskGraph {
            schema_version: 1,
            nodes: vec![
                node("alpha", NodeKind::Analysis, TaskRole::Explore, &[]),
                node("beta", NodeKind::Analysis, TaskRole::Explore, &[]),
                node(
                    "reduce",
                    NodeKind::Reducer,
                    TaskRole::Reduce,
                    &["alpha", "beta"],
                ),
            ],
        }
    }

    fn complete_policy(count: usize) -> GraphPolicy {
        let mut policy = artifact_policy(count);
        policy.max_concurrency = 3;
        policy.max_total_tokens = 3;
        policy.max_total_storage_bytes = 3;
        policy.max_total_timeout_seconds = 3;
        policy.max_projected_prompt_bytes = 1024 * 1024;
        policy.allowed_roles = ["explore", "reduce"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        policy.allowed_output_schemas = ["result"].into_iter().map(str::to_owned).collect();
        policy.allowed_model_policies = ["model"].into_iter().map(str::to_owned).collect();
        policy.allowed_permission_profiles = ["read-only"].into_iter().map(str::to_owned).collect();
        policy.allowed_workspace_modes = ["readOnly"].into_iter().map(str::to_owned).collect();
        policy
    }

    fn projection_policy() -> ProjectionPolicy {
        ProjectionPolicy {
            base_instructions: String::new(),
            role_instructions: [
                ("explore".to_owned(), "Explore.".to_owned()),
                ("reduce".to_owned(), "Reduce.".to_owned()),
            ]
            .into_iter()
            .collect(),
            checkpoint_instructions: String::new(),
            max_bytes: 1024 * 1024,
            scratch_paths: [
                (task_id("alpha"), "/scratch/alpha".to_owned()),
                (task_id("beta"), "/scratch/beta".to_owned()),
                (task_id("reduce"), "/scratch/reduce".to_owned()),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn public_validation_normalizes_each_accepted_path_once() {
        let policy = complete_policy(MAX_APPROVED_ARTIFACTS);
        let budget = PathIdentityBudget::new(MAX_APPROVED_ARTIFACTS + 3);

        validate_graph(graph(), &policy, &projection_policy()).expect("valid maximum policy");

        assert_eq!(budget.calls(), MAX_APPROVED_ARTIFACTS + 3);
    }

    #[test]
    fn public_validation_skips_all_identity_work_above_artifact_limit() {
        let policy = complete_policy(MAX_APPROVED_ARTIFACTS + 1);
        let budget = PathIdentityBudget::new(0);

        let error =
            validate_graph(graph(), &policy, &projection_policy()).expect_err("oversized policy");

        assert_eq!(budget.calls(), 0);
        assert_eq!(error.diagnostics().len(), 1);
        assert_eq!(error.diagnostics()[0].code, "policy.invalid");
        assert_eq!(error.diagnostics()[0].field, Some("approvedArtifacts"));
    }
}
