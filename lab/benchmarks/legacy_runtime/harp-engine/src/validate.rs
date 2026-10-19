use std::collections::{BTreeMap, BTreeSet};

use harp_contracts::{Budget, NodeKind, TaskGraph, TaskId, TaskNode, TaskRole, WorkspaceMode};

use crate::diagnostic::{Diagnostics, ValidationDiagnostic, ValidationError};
use crate::policy::{validate_policy, validate_scratch_task_set, GraphPolicy};
use crate::projection::{
    capture_validated_projection, estimate_static_projection_bytes, ProjectionPolicy,
    ValidatedProjection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedGraph {
    graph: TaskGraph,
    topological_order: Vec<TaskId>,
    reducer: TaskId,
    aggregate_budget: Budget,
    max_concurrency: usize,
    projection: ValidatedProjection,
}

impl ValidatedGraph {
    pub fn graph(&self) -> &TaskGraph {
        &self.graph
    }

    pub fn topological_order(&self) -> &[TaskId] {
        &self.topological_order
    }

    pub fn reducer(&self) -> &TaskId {
        &self.reducer
    }

    pub fn aggregate_budget(&self) -> &Budget {
        &self.aggregate_budget
    }

    pub const fn max_concurrency(&self) -> usize {
        self.max_concurrency
    }

    pub fn into_graph(self) -> TaskGraph {
        self.graph
    }

    pub(crate) fn projection(&self) -> &ValidatedProjection {
        &self.projection
    }
}

pub const fn role_name(role: TaskRole) -> &'static str {
    match role {
        TaskRole::Explore => "explore",
        TaskRole::Classify => "classify",
        TaskRole::Implement => "implement",
        TaskRole::Verify => "verify",
        TaskRole::Critic => "critic",
        TaskRole::Reduce => "reduce",
    }
}

pub const fn workspace_mode_name(mode: WorkspaceMode) -> &'static str {
    match mode {
        WorkspaceMode::ReadOnly => "readOnly",
        WorkspaceMode::Scratch => "scratch",
        WorkspaceMode::GitWorktree => "gitWorktree",
        WorkspaceMode::CopyOnWrite => "copyOnWrite",
    }
}

pub fn validate_graph(
    graph: TaskGraph,
    policy: &GraphPolicy,
    projection_policy: &ProjectionPolicy,
) -> Result<ValidatedGraph, ValidationError> {
    let mut diagnostics = Diagnostics::default();
    validate_policy(policy, projection_policy, &mut diagnostics);
    if !diagnostics.is_empty() {
        return Err(ValidationError::from_diagnostics(diagnostics));
    }

    if graph.validate_shape().is_err() {
        diagnostics.push(ValidationDiagnostic::new(
            "graph.shape",
            None,
            None,
            "graph does not satisfy the bounded TaskGraph contract shape",
        ));
        return Err(ValidationError::from_diagnostics(diagnostics));
    }

    if graph.nodes.len() > policy.max_nodes {
        diagnostics.push(ValidationDiagnostic::new(
            "graph.too_many_nodes",
            None,
            Some("nodes"),
            format!("node count exceeds policy maximum of {}", policy.max_nodes),
        ));
    }

    let mut nodes = BTreeMap::new();
    let mut duplicate_ids = false;
    for node in &graph.nodes {
        if nodes.insert(node.task_id.clone(), node).is_some() {
            duplicate_ids = true;
            diagnostics.push(ValidationDiagnostic::new(
                "graph.duplicate_task_id",
                Some(&node.task_id),
                Some("taskId"),
                "task ID is duplicated",
            ));
        }
        validate_node_policy(node, policy, &mut diagnostics);
        validate_node_inputs(node, policy, &mut diagnostics);
        if projection_policy
            .role_instructions
            .contains_key(role_name(node.role))
        {
            validate_static_projection(node, policy, projection_policy, &mut diagnostics);
        } else {
            diagnostics.push(ValidationDiagnostic::new(
                "projection.missing_role_instructions",
                Some(&node.task_id),
                Some("roleInstructions"),
                "task role has no projection instructions",
            ));
        }
    }
    validate_scratch_task_set(&nodes, projection_policy, &mut diagnostics);

    let dependencies_usable = !duplicate_ids && validate_dependencies(&nodes, &mut diagnostics);

    let reducer_ids = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Reducer)
        .map(|node| node.task_id.clone())
        .collect::<Vec<_>>();
    let analysis_count = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Analysis)
        .count();
    if analysis_count < 2 {
        diagnostics.push(ValidationDiagnostic::new(
            "graph.analysis_count",
            None,
            Some("nodes"),
            "graph must contain at least two analysis tasks",
        ));
    }
    if reducer_ids.len() != 1 {
        diagnostics.push(ValidationDiagnostic::new(
            "graph.reducer_count",
            None,
            Some("kind"),
            "graph must contain exactly one reducer",
        ));
    }

    validate_kind_roles(&graph.nodes, &mut diagnostics);

    let mut topological_order = None;
    if dependencies_usable {
        topological_order = topological_sort(&nodes, &mut diagnostics);
        if reducer_ids.len() == 1 {
            validate_reducer_semantics(
                &nodes,
                &reducer_ids[0],
                topological_order.is_some(),
                &mut diagnostics,
            );
        }
    }

    let aggregate_budget = aggregate_budget(&graph.nodes, policy, &mut diagnostics);

    if !diagnostics.is_empty() {
        return Err(ValidationError::from_diagnostics(diagnostics));
    }

    let reducer = reducer_ids
        .into_iter()
        .next()
        .expect("validated reducer count");
    let max_concurrency = policy.max_concurrency.min(graph.nodes.len());
    let projection = capture_validated_projection(&graph, policy, projection_policy);
    Ok(ValidatedGraph {
        graph,
        topological_order: topological_order.expect("validated topological order"),
        reducer,
        aggregate_budget: aggregate_budget.expect("validated aggregate budget"),
        max_concurrency,
        projection,
    })
}

fn validate_node_policy(node: &TaskNode, policy: &GraphPolicy, diagnostics: &mut Diagnostics) {
    if !policy.allowed_roles.contains(role_name(node.role)) {
        diagnostics.push(ValidationDiagnostic::new(
            "policy.role",
            Some(&node.task_id),
            Some("role"),
            "task role is not allowed",
        ));
    }
    if !policy.allowed_output_schemas.contains(&node.output_schema) {
        diagnostics.push(ValidationDiagnostic::new(
            "policy.output_schema",
            Some(&node.task_id),
            Some("outputSchema"),
            "output schema is not allowed",
        ));
    }
    if !policy.allowed_model_policies.contains(&node.model_policy) {
        diagnostics.push(ValidationDiagnostic::new(
            "policy.model",
            Some(&node.task_id),
            Some("modelPolicy"),
            "model policy is not allowed",
        ));
    }
    if !policy
        .allowed_permission_profiles
        .contains(&node.permission_profile)
    {
        diagnostics.push(ValidationDiagnostic::new(
            "policy.permission",
            Some(&node.task_id),
            Some("permissionProfile"),
            "permission profile is not allowed",
        ));
    }
    if !policy
        .allowed_workspace_modes
        .contains(workspace_mode_name(node.workspace_mode))
    {
        diagnostics.push(ValidationDiagnostic::new(
            "policy.workspace",
            Some(&node.task_id),
            Some("workspaceMode"),
            "workspace mode is not allowed",
        ));
    }
}

fn validate_node_inputs(node: &TaskNode, policy: &GraphPolicy, diagnostics: &mut Diagnostics) {
    let mut seen = BTreeSet::new();
    for input in &node.inputs {
        if !seen.insert(input.uri.as_str()) {
            diagnostics.push(ValidationDiagnostic::new(
                "artifact.duplicate_input",
                Some(&node.task_id),
                Some("inputs"),
                "artifact URI is duplicated for this task",
            ));
        }
        match policy.approved_artifacts.get(&input.uri) {
            None => diagnostics.push(ValidationDiagnostic::new(
                "artifact.unapproved",
                Some(&node.task_id),
                Some("inputs"),
                "artifact URI is not approved",
            )),
            Some(approved) if approved.reference != *input => {
                diagnostics.push(ValidationDiagnostic::new(
                    "artifact.metadata_mismatch",
                    Some(&node.task_id),
                    Some("inputs"),
                    "artifact metadata does not match approved metadata",
                ));
            }
            Some(_) => {}
        }
    }
}

fn validate_dependencies(
    nodes: &BTreeMap<TaskId, &TaskNode>,
    diagnostics: &mut Diagnostics,
) -> bool {
    let mut usable = true;
    for node in nodes.values() {
        let mut seen = BTreeSet::new();
        for dependency in &node.dependencies {
            if !seen.insert(dependency) {
                usable = false;
                diagnostics.push(ValidationDiagnostic::new(
                    "graph.duplicate_dependency",
                    Some(&node.task_id),
                    Some("dependencies"),
                    "dependency is duplicated",
                ));
            }
            if dependency == &node.task_id {
                usable = false;
                diagnostics.push(ValidationDiagnostic::new(
                    "graph.self_dependency",
                    Some(&node.task_id),
                    Some("dependencies"),
                    "task cannot depend on itself",
                ));
            }
            if !nodes.contains_key(dependency) {
                usable = false;
                diagnostics.push(ValidationDiagnostic::new(
                    "graph.unknown_dependency",
                    Some(&node.task_id),
                    Some("dependencies"),
                    "dependency does not identify a graph task",
                ));
            }
        }
    }
    usable
}

fn validate_kind_roles(nodes: &[TaskNode], diagnostics: &mut Diagnostics) {
    for node in nodes {
        let compatible = match node.kind {
            NodeKind::Analysis => node.role != TaskRole::Reduce,
            NodeKind::Reducer => node.role == TaskRole::Reduce,
        };
        if !compatible {
            diagnostics.push(ValidationDiagnostic::new(
                "graph.role_kind",
                Some(&node.task_id),
                Some("role"),
                "task role is incompatible with node kind",
            ));
        }
    }
}

fn topological_sort(
    nodes: &BTreeMap<TaskId, &TaskNode>,
    diagnostics: &mut Diagnostics,
) -> Option<Vec<TaskId>> {
    let mut indegrees = nodes
        .iter()
        .map(|(task_id, node)| (task_id.clone(), node.dependencies.len()))
        .collect::<BTreeMap<_, _>>();
    let mut dependents = nodes
        .keys()
        .map(|task_id| (task_id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for node in nodes.values() {
        for dependency in &node.dependencies {
            dependents
                .get_mut(dependency)
                .expect("validated dependency")
                .insert(node.task_id.clone());
        }
    }

    let mut ready = indegrees
        .iter()
        .filter_map(|(task_id, indegree)| (*indegree == 0).then_some(task_id.clone()))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(task_id) = ready.pop_first() {
        order.push(task_id.clone());
        for dependent in dependents.get(&task_id).expect("known task") {
            let indegree = indegrees.get_mut(dependent).expect("known dependent");
            *indegree -= 1;
            if *indegree == 0 {
                ready.insert(dependent.clone());
            }
        }
    }

    if order.len() != nodes.len() {
        diagnostics.push(ValidationDiagnostic::new(
            "graph.cycle",
            None,
            Some("dependencies"),
            "graph contains a dependency cycle",
        ));
        None
    } else {
        Some(order)
    }
}

fn validate_reducer_semantics(
    nodes: &BTreeMap<TaskId, &TaskNode>,
    reducer_id: &TaskId,
    reachability_reliable: bool,
    diagnostics: &mut Diagnostics,
) {
    let reducer = nodes.get(reducer_id).expect("known reducer");
    for node in nodes.values() {
        if node.dependencies.contains(reducer_id) {
            diagnostics.push(ValidationDiagnostic::new(
                "graph.reducer_terminal",
                Some(reducer_id),
                Some("dependencies"),
                "reducer must not have dependents",
            ));
        }
    }

    let direct_dependencies = reducer.dependencies.iter().collect::<BTreeSet<_>>();
    let reachable = if reachability_reliable {
        reducer_ancestors(nodes, reducer_id)
    } else {
        BTreeSet::new()
    };
    for analysis in nodes
        .values()
        .filter(|node| node.kind == NodeKind::Analysis)
    {
        if !direct_dependencies.contains(&analysis.task_id) {
            diagnostics.push(ValidationDiagnostic::new(
                "graph.reducer_dependency",
                Some(&analysis.task_id),
                Some("dependencies"),
                "reducer must depend directly on every analysis task",
            ));
        }
        if reachability_reliable && !reachable.contains(&analysis.task_id) {
            diagnostics.push(ValidationDiagnostic::new(
                "graph.analysis_reachability",
                Some(&analysis.task_id),
                Some("dependencies"),
                "analysis task does not reach the reducer",
            ));
        }
    }
}

fn reducer_ancestors(nodes: &BTreeMap<TaskId, &TaskNode>, reducer_id: &TaskId) -> BTreeSet<TaskId> {
    let mut reachable = BTreeSet::new();
    let mut pending = nodes
        .get(reducer_id)
        .expect("known reducer")
        .dependencies
        .clone();
    while let Some(task_id) = pending.pop() {
        if reachable.insert(task_id.clone()) {
            pending.extend(
                nodes
                    .get(&task_id)
                    .expect("validated dependency")
                    .dependencies
                    .iter()
                    .cloned(),
            );
        }
    }
    reachable
}

fn aggregate_budget(
    nodes: &[TaskNode],
    policy: &GraphPolicy,
    diagnostics: &mut Diagnostics,
) -> Option<Budget> {
    let mut max_tokens = Some(0u64);
    let mut timeout_seconds = Some(0u64);
    let mut max_storage_bytes = Some(0u64);

    for node in nodes {
        if node.budget.max_tokens > policy.max_node_tokens {
            diagnostics.push(ValidationDiagnostic::new(
                "budget.node_tokens",
                Some(&node.task_id),
                Some("budget.maxTokens"),
                "task token budget exceeds the per-node maximum",
            ));
        }
        if node.budget.timeout_seconds > policy.max_node_timeout_seconds {
            diagnostics.push(ValidationDiagnostic::new(
                "budget.node_timeout",
                Some(&node.task_id),
                Some("budget.timeoutSeconds"),
                "task timeout exceeds the per-node maximum",
            ));
        }
        if node.budget.max_storage_bytes > policy.max_node_storage_bytes {
            diagnostics.push(ValidationDiagnostic::new(
                "budget.node_storage",
                Some(&node.task_id),
                Some("budget.maxStorageBytes"),
                "task storage budget exceeds the per-node maximum",
            ));
        }
        max_tokens = max_tokens.and_then(|total| total.checked_add(node.budget.max_tokens));
        timeout_seconds =
            timeout_seconds.and_then(|total| total.checked_add(node.budget.timeout_seconds));
        max_storage_bytes =
            max_storage_bytes.and_then(|total| total.checked_add(node.budget.max_storage_bytes));
    }

    validate_aggregate(
        max_tokens,
        policy.max_total_tokens,
        "budget.aggregate_tokens",
        "maxTokens",
        diagnostics,
    );
    validate_aggregate(
        timeout_seconds,
        policy.max_total_timeout_seconds,
        "budget.aggregate_timeout",
        "timeoutSeconds",
        diagnostics,
    );
    validate_aggregate(
        max_storage_bytes,
        policy.max_total_storage_bytes,
        "budget.aggregate_storage",
        "maxStorageBytes",
        diagnostics,
    );

    Some(Budget::new(
        max_tokens?,
        timeout_seconds?,
        max_storage_bytes?,
    ))
}

fn validate_aggregate(
    total: Option<u64>,
    limit: u64,
    code: &'static str,
    field: &'static str,
    diagnostics: &mut Diagnostics,
) {
    if total.is_none_or(|total| total > limit) {
        diagnostics.push(ValidationDiagnostic::new(
            code,
            None,
            Some(field),
            "aggregate reservation exceeds the run-wide maximum",
        ));
    }
}

fn validate_static_projection(
    node: &TaskNode,
    policy: &GraphPolicy,
    projection_policy: &ProjectionPolicy,
    diagnostics: &mut Diagnostics,
) {
    let limit = policy
        .max_projected_prompt_bytes
        .min(projection_policy.max_bytes);
    if estimate_static_projection_bytes(node, policy, projection_policy)
        .is_none_or(|bytes| bytes > limit)
    {
        diagnostics.push(ValidationDiagnostic::new(
            "projection.prompt_bytes",
            Some(&node.task_id),
            None,
            "conservative child context projection exceeds the byte limit",
        ));
    }
}
