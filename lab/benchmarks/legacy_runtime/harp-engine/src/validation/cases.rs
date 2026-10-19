use super::*;

pub(super) fn projection_policy() -> ProjectionPolicy {
    ProjectionPolicy {
        base_instructions: "Follow the child protocol.".to_owned(),
        role_instructions: [
            ("explore".to_owned(), "Explore evidence.".to_owned()),
            ("reduce".to_owned(), "Reduce results.".to_owned()),
        ]
        .into_iter()
        .collect(),
        checkpoint_instructions: "Write bounded checkpoints.".to_owned(),
        max_bytes: 1024 * 1024,
        scratch_paths: BTreeMap::new(),
    }
}

fn diagnostic_codes(
    graph: TaskGraph,
    policy: &GraphPolicy,
    projection: &ProjectionPolicy,
) -> Vec<&'static str> {
    let mut projection = projection.clone();
    if projection.scratch_paths.is_empty() {
        projection.scratch_paths = graph
            .nodes
            .iter()
            .map(|node| {
                (
                    node.task_id.clone(),
                    format!("/scratch/tasks/{}", node.task_id),
                )
            })
            .collect();
    }
    validate_graph(graph, policy, &projection)
        .expect_err("graph should be invalid")
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn projection_policy_for(graph: &TaskGraph) -> ProjectionPolicy {
    let mut projection = projection_policy();
    projection.scratch_paths = graph
        .nodes
        .iter()
        .map(|node| {
            (
                node.task_id.clone(),
                format!("/scratch/tasks/{}", node.task_id),
            )
        })
        .collect();
    projection
}

#[test]
fn validates_graph_and_returns_lexical_topological_order() {
    let graph = graph(vec![
        reducer("reduce", &["beta", "alpha"]),
        analysis("beta"),
        analysis("alpha"),
    ]);

    let projection = projection_policy_for(&graph);
    let validated =
        validate_graph(graph.clone(), &policy(), &projection).expect("valid depth-one graph");

    assert_eq!(
        validated
            .topological_order()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["alpha", "beta", "reduce"]
    );
    assert_eq!(validated.reducer().to_string(), "reduce");
    assert_eq!(validated.aggregate_budget(), &Budget::new(300, 30, 3_000));
    assert_eq!(validated.max_concurrency(), 2);
    assert_eq!(validated.graph(), &graph);
    assert_eq!(validated.clone().into_graph(), graph);
}

#[test]
fn concurrency_is_capped_by_graph_size() {
    let mut graph_policy = policy();
    graph_policy.max_concurrency = 4;
    let graph = graph(vec![
        analysis("alpha"),
        analysis("beta"),
        reducer("reduce", &["alpha", "beta"]),
    ]);
    let projection = projection_policy_for(&graph);
    let validated = validate_graph(graph, &graph_policy, &projection)
        .expect("concurrency above graph size is a valid ceiling");

    assert_eq!(validated.max_concurrency(), 3);
}

#[test]
fn graph_requires_at_least_two_analysis_children() {
    let graph = graph(vec![analysis("only"), reducer("reduce", &["only"])]);
    let diagnostics = validate_graph(graph.clone(), &policy(), &projection_policy_for(&graph))
        .expect_err("one analysis child is below the first-slice minimum");

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.code == "graph.analysis_count"
            && diagnostic.task_id.is_none()
            && diagnostic.field == Some("nodes")
    }));
}

#[test]
fn canonical_policy_names_are_stable() {
    assert_eq!(role_name(TaskRole::Explore), "explore");
    assert_eq!(role_name(TaskRole::Classify), "classify");
    assert_eq!(role_name(TaskRole::Implement), "implement");
    assert_eq!(role_name(TaskRole::Verify), "verify");
    assert_eq!(role_name(TaskRole::Critic), "critic");
    assert_eq!(role_name(TaskRole::Reduce), "reduce");
    assert_eq!(workspace_mode_name(WorkspaceMode::ReadOnly), "readOnly");
    assert_eq!(workspace_mode_name(WorkspaceMode::Scratch), "scratch");
    assert_eq!(
        workspace_mode_name(WorkspaceMode::GitWorktree),
        "gitWorktree"
    );
    assert_eq!(
        workspace_mode_name(WorkspaceMode::CopyOnWrite),
        "copyOnWrite"
    );
}

#[test]
fn graph_structure_failures_have_stable_codes() {
    struct Case {
        expected: &'static str,
        graph: TaskGraph,
    }

    let mut duplicate_dependency = reducer("reduce", &["analysis", "analysis"]);
    duplicate_dependency.dependencies.reverse();
    let mut self_dependency = analysis("analysis");
    self_dependency.dependencies = vec![task_id("analysis")];
    let mut unknown_dependency = analysis("analysis");
    unknown_dependency.dependencies = vec![task_id("missing")];
    let mut cycle_a = analysis("a");
    cycle_a.dependencies = vec![task_id("b")];
    let mut cycle_b = analysis("b");
    cycle_b.dependencies = vec![task_id("a")];
    let mut after_reducer = analysis("later");
    after_reducer.dependencies = vec![task_id("reduce")];

    let cases = vec![
        Case {
            expected: "graph.shape",
            graph: TaskGraph {
                schema_version: 2,
                nodes: vec![analysis("analysis"), reducer("reduce", &["analysis"])],
            },
        },
        Case {
            expected: "graph.duplicate_task_id",
            graph: graph(vec![
                analysis("same"),
                analysis("same"),
                reducer("reduce", &["same"]),
            ]),
        },
        Case {
            expected: "graph.duplicate_dependency",
            graph: graph(vec![analysis("analysis"), duplicate_dependency]),
        },
        Case {
            expected: "graph.self_dependency",
            graph: graph(vec![self_dependency, reducer("reduce", &["analysis"])]),
        },
        Case {
            expected: "graph.unknown_dependency",
            graph: graph(vec![unknown_dependency, reducer("reduce", &["analysis"])]),
        },
        Case {
            expected: "graph.cycle",
            graph: graph(vec![cycle_a, cycle_b, reducer("reduce", &["a", "b"])]),
        },
        Case {
            expected: "graph.reducer_count",
            graph: graph(vec![analysis("analysis")]),
        },
        Case {
            expected: "graph.reducer_count",
            graph: graph(vec![reducer("one", &[]), reducer("two", &[])]),
        },
        Case {
            expected: "graph.reducer_terminal",
            graph: graph(vec![
                analysis("analysis"),
                reducer("reduce", &["analysis"]),
                after_reducer,
            ]),
        },
        Case {
            expected: "graph.reducer_dependency",
            graph: graph(vec![
                analysis("a"),
                analysis("b"),
                reducer("reduce", &["a"]),
            ]),
        },
        Case {
            expected: "graph.analysis_reachability",
            graph: graph(vec![
                analysis("a"),
                analysis("b"),
                reducer("reduce", &["a"]),
            ]),
        },
    ];

    for case in cases {
        let codes = diagnostic_codes(case.graph, &policy(), &projection_policy());
        assert!(
            codes.contains(&case.expected),
            "expected {}, got {codes:?}",
            case.expected
        );
    }
}

#[test]
fn reachability_is_transitive_and_distinct_from_direct_reducer_dependency() {
    let mut alpha = analysis("alpha");
    let mut beta = analysis("beta");
    beta.dependencies.push(task_id("alpha"));
    let transitive = graph(vec![alpha.clone(), beta, reducer("reduce", &["beta"])]);
    let codes = diagnostic_codes(transitive, &policy(), &projection_policy());
    assert!(codes.contains(&"graph.reducer_dependency"));
    assert!(!codes.contains(&"graph.analysis_reachability"));

    alpha.dependencies.clear();
    let isolated = graph(vec![alpha, analysis("beta"), reducer("reduce", &["beta"])]);
    let diagnostics = validate_graph(
        isolated.clone(),
        &policy(),
        &projection_policy_for(&isolated),
    )
    .expect_err("isolated analysis must be rejected");
    let alpha_codes = diagnostics
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.task_id.as_ref() == Some(&task_id("alpha")))
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(alpha_codes.contains(&"graph.reducer_dependency"));
    assert!(alpha_codes.contains(&"graph.analysis_reachability"));

    let mut cycle_alpha = analysis("alpha");
    cycle_alpha.dependencies.push(task_id("beta"));
    let mut cycle_beta = analysis("beta");
    cycle_beta.dependencies.push(task_id("alpha"));
    let cycle = graph(vec![
        cycle_alpha,
        cycle_beta,
        reducer("reduce", &["alpha", "beta"]),
    ]);
    let diagnostics = validate_graph(cycle.clone(), &policy(), &projection_policy_for(&cycle))
        .expect_err("cycle must be rejected");
    assert!(diagnostics
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "graph.cycle"));
    assert!(!diagnostics
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "graph.analysis_reachability"));
}

#[test]
fn graph_policy_and_kind_role_failures_have_stable_codes() {
    struct Case {
        expected: &'static str,
        graph: TaskGraph,
        mutate: fn(&mut GraphPolicy),
    }

    fn no_policy_change(_: &mut GraphPolicy) {}
    fn disallow_role(policy: &mut GraphPolicy) {
        policy.allowed_roles.remove("explore");
    }
    fn disallow_schema(policy: &mut GraphPolicy) {
        policy.allowed_output_schemas.clear();
        policy.allowed_output_schemas.insert("other".to_owned());
    }
    fn disallow_model(policy: &mut GraphPolicy) {
        policy.allowed_model_policies.clear();
        policy.allowed_model_policies.insert("other".to_owned());
    }
    fn disallow_permission(policy: &mut GraphPolicy) {
        policy.allowed_permission_profiles.clear();
        policy
            .allowed_permission_profiles
            .insert("other".to_owned());
    }
    fn disallow_workspace(policy: &mut GraphPolicy) {
        policy.allowed_workspace_modes.remove("readOnly");
    }

    let mut reducer_as_analysis = analysis("analysis");
    reducer_as_analysis.role = TaskRole::Reduce;
    let mut analysis_as_reducer = reducer("reduce", &["analysis"]);
    analysis_as_reducer.role = TaskRole::Explore;

    let cases = vec![
        Case {
            expected: "graph.role_kind",
            graph: graph(vec![reducer_as_analysis, reducer("reduce", &["analysis"])]),
            mutate: no_policy_change,
        },
        Case {
            expected: "graph.role_kind",
            graph: graph(vec![analysis("analysis"), analysis_as_reducer]),
            mutate: no_policy_change,
        },
        Case {
            expected: "policy.role",
            graph: graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
            mutate: disallow_role,
        },
        Case {
            expected: "policy.output_schema",
            graph: graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
            mutate: disallow_schema,
        },
        Case {
            expected: "policy.model",
            graph: graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
            mutate: disallow_model,
        },
        Case {
            expected: "policy.permission",
            graph: graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
            mutate: disallow_permission,
        },
        Case {
            expected: "policy.workspace",
            graph: graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
            mutate: disallow_workspace,
        },
    ];

    for case in cases {
        let mut policy = policy();
        (case.mutate)(&mut policy);
        let codes = diagnostic_codes(case.graph, &policy, &projection_policy());
        assert!(
            codes.contains(&case.expected),
            "expected {}, got {codes:?}",
            case.expected
        );
    }
}

#[test]
fn artifact_failures_have_stable_codes() {
    let approved = artifact(DIGEST_A, 64);
    let unknown = artifact(
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        64,
    );

    let mut unknown_node = analysis("analysis");
    unknown_node.inputs.push(unknown);

    let mut mismatch = approved.clone();
    mismatch.media_type = "text/plain".to_owned();
    let mut mismatch_node = analysis("analysis");
    mismatch_node.inputs.push(mismatch);

    let mut duplicate_node = analysis("analysis");
    duplicate_node.inputs = vec![approved.clone(), approved];

    for (expected, node) in [
        ("artifact.unapproved", unknown_node),
        ("artifact.metadata_mismatch", mismatch_node),
        ("artifact.duplicate_input", duplicate_node),
    ] {
        let codes = diagnostic_codes(
            graph(vec![node, reducer("reduce", &["analysis"])]),
            &policy(),
            &projection_policy(),
        );
        assert!(
            codes.contains(&expected),
            "expected {expected}, got {codes:?}"
        );
    }

    let mut invalid_policy = policy();
    invalid_policy
        .approved_artifacts
        .get_mut(&format!("artifact://sha256/{DIGEST_A}"))
        .expect("approved artifact")
        .read_only = false;
    let codes = diagnostic_codes(
        graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
        &invalid_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"policy.invalid"));

    let mut path_conflict_policy = policy();
    let conflicting = artifact(
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        32,
    );
    path_conflict_policy.approved_artifacts.insert(
        conflicting.uri.clone(),
        ApprovedArtifact {
            reference: conflicting,
            materialized_path: "/approved/input.json".to_owned(),
            read_only: true,
        },
    );
    let codes = diagnostic_codes(
        graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
        &path_conflict_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"artifact.path_conflict"));

    let mut ancestor_conflict_policy = policy();
    let nested = artifact(
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        32,
    );
    ancestor_conflict_policy.approved_artifacts.insert(
        nested.uri.clone(),
        ApprovedArtifact {
            reference: nested,
            materialized_path: "/approved/input.json/nested".to_owned(),
            read_only: true,
        },
    );
    let codes = diagnostic_codes(
        graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
        &ancestor_conflict_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"artifact.path_conflict"));
}

#[test]
fn budget_limits_accept_exact_values_and_reject_plus_one_and_overflow() {
    let mut exact_policy = policy();
    exact_policy.max_total_tokens = 300;
    exact_policy.max_total_timeout_seconds = 30;
    exact_policy.max_total_storage_bytes = 3_000;
    let exact_graph = graph(vec![
        analysis("analysis"),
        analysis("second"),
        reducer("reduce", &["analysis", "second"]),
    ]);
    validate_graph(
        exact_graph.clone(),
        &exact_policy,
        &projection_policy_for(&exact_graph),
    )
    .expect("exact aggregate limits are valid");

    for (expected, mutate) in [
        (
            "budget.aggregate_tokens",
            (|policy: &mut GraphPolicy| policy.max_total_tokens = 299) as fn(&mut GraphPolicy),
        ),
        (
            "budget.aggregate_timeout",
            (|policy: &mut GraphPolicy| policy.max_total_timeout_seconds = 29)
                as fn(&mut GraphPolicy),
        ),
        (
            "budget.aggregate_storage",
            (|policy: &mut GraphPolicy| policy.max_total_storage_bytes = 2_999)
                as fn(&mut GraphPolicy),
        ),
    ] {
        let mut policy = policy();
        mutate(&mut policy);
        let codes = diagnostic_codes(
            graph(vec![
                analysis("analysis"),
                analysis("second"),
                reducer("reduce", &["analysis", "second"]),
            ]),
            &policy,
            &projection_policy(),
        );
        assert!(
            codes.contains(&expected),
            "expected {expected}, got {codes:?}"
        );
    }

    let mut one = analysis("one");
    one.budget.max_tokens = u64::MAX;
    let mut two = analysis("two");
    two.budget.max_tokens = u64::MAX;
    let mut overflow_policy = policy();
    overflow_policy.max_total_tokens = u64::MAX;
    let codes = diagnostic_codes(
        graph(vec![one, two, reducer("reduce", &["one", "two"])]),
        &overflow_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"budget.aggregate_tokens"));

    let mut oversized = analysis("analysis");
    oversized.budget.max_tokens = 201;
    let mut node_policy = policy();
    node_policy.max_node_tokens = 200;
    let codes = diagnostic_codes(
        graph(vec![
            oversized,
            analysis("second"),
            reducer("reduce", &["analysis", "second"]),
        ]),
        &node_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"budget.node_tokens"));
}

#[test]
fn node_budget_ceilings_are_independent_from_aggregate_limits() {
    let exact_policy = policy();
    let exact_graph = graph(vec![
        analysis("analysis"),
        analysis("second"),
        reducer("reduce", &["analysis", "second"]),
    ]);
    validate_graph(
        exact_graph.clone(),
        &exact_policy,
        &projection_policy_for(&exact_graph),
    )
    .expect("node budgets exactly at their ceilings are valid");

    for (expected, mutate) in [
        (
            "budget.node_tokens",
            (|node: &mut TaskNode| node.budget.max_tokens = 101) as fn(&mut TaskNode),
        ),
        (
            "budget.node_timeout",
            (|node: &mut TaskNode| node.budget.timeout_seconds = 11) as fn(&mut TaskNode),
        ),
        (
            "budget.node_storage",
            (|node: &mut TaskNode| node.budget.max_storage_bytes = 1_001) as fn(&mut TaskNode),
        ),
    ] {
        let mut child = analysis("analysis");
        mutate(&mut child);
        let codes = diagnostic_codes(
            graph(vec![child, reducer("reduce", &["analysis"])]),
            &policy(),
            &projection_policy(),
        );
        assert!(
            codes.contains(&expected),
            "expected {expected}, got {codes:?}"
        );
        assert!(!codes
            .iter()
            .any(|code| code.starts_with("budget.aggregate")));
    }
}

#[test]
fn scratch_reservations_must_exactly_cover_graph_tasks() {
    let graph = graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]);

    let mut missing = projection_policy_for(&graph);
    missing.scratch_paths.remove(&task_id("analysis"));
    let diagnostics =
        validate_graph(graph.clone(), &policy(), &missing).expect_err("missing reservation");
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.code == "projection.scratch_path"
            && diagnostic.task_id.as_ref() == Some(&task_id("analysis"))
    }));

    let mut extra = projection_policy_for(&graph);
    extra
        .scratch_paths
        .insert(task_id("extra"), "/scratch/tasks/extra".to_owned());
    let diagnostics = validate_graph(graph, &policy(), &extra).expect_err("extra reservation");
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.code == "projection.scratch_path"
            && diagnostic.task_id.as_ref() == Some(&task_id("extra"))
    }));
}

#[test]
fn scratch_reservations_are_safe_pairwise_disjoint_and_artifact_disjoint() {
    let graph = graph(vec![
        analysis("analysis"),
        analysis("second"),
        reducer("reduce", &["analysis", "second"]),
    ]);

    for (analysis_path, reducer_path) in [
        ("/scratch/shared", "/scratch/shared"),
        ("/scratch/analysis", "/scratch/analysis/reduce"),
        ("/approved", "/scratch/reduce"),
        ("/approved/input.json/work", "/scratch/reduce"),
    ] {
        let mut projection = projection_policy_for(&graph);
        projection
            .scratch_paths
            .insert(task_id("analysis"), analysis_path.to_owned());
        projection
            .scratch_paths
            .insert(task_id("reduce"), reducer_path.to_owned());
        let codes = validate_graph(graph.clone(), &policy(), &projection)
            .expect_err("overlapping scratch reservation")
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"projection.scratch_path"), "{codes:?}");
    }

    let mut prefix_only = projection_policy_for(&graph);
    prefix_only
        .scratch_paths
        .insert(task_id("analysis"), "/scratch/a".to_owned());
    prefix_only
        .scratch_paths
        .insert(task_id("reduce"), "/scratch/ab".to_owned());
    validate_graph(graph, &policy(), &prefix_only)
        .expect("plain string prefix without a component boundary is disjoint");
}

#[test]
fn path_identity_rejects_case_and_unicode_aliases() {
    let graph = graph(vec![
        analysis("analysis"),
        analysis("second"),
        reducer("reduce", &["analysis", "second"]),
    ]);
    for (analysis_path, second_path) in [
        ("/scratch/Alpha", "/scratch/alpha"),
        ("/scratch/café", "/scratch/cafe\u{301}"),
    ] {
        let mut projection = projection_policy_for(&graph);
        projection
            .scratch_paths
            .insert(task_id("analysis"), analysis_path.to_owned());
        projection
            .scratch_paths
            .insert(task_id("second"), second_path.to_owned());
        let diagnostics = validate_graph(graph.clone(), &policy(), &projection)
            .expect_err("ambiguous path aliases must be rejected");
        assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "projection.scratch_path"
                && diagnostic.task_id.as_ref() == Some(&task_id("analysis"))
        }));
    }
}

#[test]
fn root_materialization_path_conflicts_with_every_scratch_reservation() {
    let graph = graph(vec![
        analysis("analysis"),
        analysis("second"),
        reducer("reduce", &["analysis", "second"]),
    ]);
    let mut graph_policy = policy();
    graph_policy
        .approved_artifacts
        .values_mut()
        .next()
        .expect("approved artifact")
        .materialized_path = "/".to_owned();

    let diagnostics = validate_graph(graph.clone(), &graph_policy, &projection_policy_for(&graph))
        .expect_err("root materialization aliases every scratch descendant");
    assert!(diagnostics
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "projection.scratch_path"));
}

#[test]
fn scratch_reservations_must_be_normalized_absolute_paths() {
    let graph = graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]);
    for invalid_path in [
        "relative/path".to_owned(),
        "/scratch/../analysis".to_owned(),
        "/scratch/./analysis".to_owned(),
        "/scratch//analysis".to_owned(),
        "/scratch/analysis/".to_owned(),
        format!("/{}", "s".repeat(4_096)),
    ] {
        let mut projection = projection_policy_for(&graph);
        projection
            .scratch_paths
            .insert(task_id("analysis"), invalid_path);
        let diagnostics = validate_graph(graph.clone(), &policy(), &projection)
            .expect_err("invalid scratch reservation");
        assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
            diagnostic.code == "projection.scratch_path"
                && diagnostic.task_id.as_ref() == Some(&task_id("analysis"))
        }));
    }
}

#[test]
fn node_budget_policy_ceilings_must_be_nonzero_and_within_totals() {
    type PolicyMutation = fn(&mut GraphPolicy);

    let cases: [PolicyMutation; 6] = [
        |policy| policy.max_node_tokens = 0,
        |policy| policy.max_node_tokens = policy.max_total_tokens + 1,
        |policy| policy.max_node_timeout_seconds = 0,
        |policy| policy.max_node_timeout_seconds = policy.max_total_timeout_seconds + 1,
        |policy| policy.max_node_storage_bytes = 0,
        |policy| policy.max_node_storage_bytes = policy.max_total_storage_bytes + 1,
    ];

    for mutate in cases {
        let mut graph_policy = policy();
        mutate(&mut graph_policy);
        let codes = diagnostic_codes(
            graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
            &graph_policy,
            &projection_policy(),
        );
        assert!(codes.contains(&"policy.invalid"), "{codes:?}");
    }
}

#[test]
fn invalid_policy_bounds_and_depth_are_rejected_before_graph_validation() {
    type PolicyMutation = fn(&mut GraphPolicy);

    fn max_nodes_zero(policy: &mut GraphPolicy) {
        policy.max_nodes = 0;
    }
    fn max_nodes_too_high(policy: &mut GraphPolicy) {
        policy.max_nodes = 65;
    }
    fn concurrency_zero(policy: &mut GraphPolicy) {
        policy.max_concurrency = 0;
    }
    fn concurrency_too_high(policy: &mut GraphPolicy) {
        policy.max_concurrency = policy.max_nodes + 1;
    }

    let cases: [(&str, PolicyMutation); 4] = [
        ("max nodes zero", max_nodes_zero),
        ("max nodes too high", max_nodes_too_high),
        ("concurrency zero", concurrency_zero),
        ("concurrency too high", concurrency_too_high),
    ];

    for (label, mutate) in cases {
        let mut policy = policy();
        mutate(&mut policy);
        let codes = diagnostic_codes(
            graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
            &policy,
            &projection_policy(),
        );
        assert!(
            codes.contains(&"policy.invalid"),
            "{label}: expected policy.invalid, got {codes:?}"
        );
    }

    let mut depth_policy = policy();
    depth_policy.max_recursion_depth = 2;
    let codes = diagnostic_codes(
        graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
        &depth_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"recursion.depth"));
}

#[test]
fn graph_node_limit_and_static_projection_size_are_enforced() {
    let mut node_policy = policy();
    node_policy.max_nodes = 1;
    node_policy.max_concurrency = 1;
    let codes = diagnostic_codes(
        graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
        &node_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"graph.too_many_nodes"));

    for projection in [
        ProjectionPolicy {
            base_instructions: "b".repeat(65_536),
            ..projection_policy()
        },
        ProjectionPolicy {
            role_instructions: [("explore".to_owned(), "r".repeat(65_536))]
                .into_iter()
                .collect(),
            ..projection_policy()
        },
    ] {
        let mut prompt_policy = policy();
        prompt_policy.max_projected_prompt_bytes = 4_096;
        let codes = diagnostic_codes(
            graph(vec![analysis("analysis"), reducer("reduce", &["analysis"])]),
            &prompt_policy,
            &projection,
        );
        assert!(
            codes.contains(&"projection.prompt_bytes"),
            "expected projection.prompt_bytes, got {codes:?}"
        );
    }

    let mut path_policy = policy();
    let approved = path_policy
        .approved_artifacts
        .values_mut()
        .next()
        .expect("approved artifact");
    approved.materialized_path = format!("/{}", "p".repeat(4_000));
    let approved_ref = approved.reference.clone();
    let mut with_input = analysis("analysis");
    with_input.inputs.push(approved_ref);
    path_policy.max_projected_prompt_bytes = 4_096;
    let codes = diagnostic_codes(
        graph(vec![with_input, reducer("reduce", &["analysis"])]),
        &path_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"projection.prompt_bytes"));

    let mut metadata_policy = policy();
    let approved = metadata_policy
        .approved_artifacts
        .values_mut()
        .next()
        .expect("approved artifact");
    approved.reference.permitted_ranges = Some(vec!["r".repeat(1_024); 8]);
    let approved_ref = approved.reference.clone();
    let mut with_metadata = analysis("analysis");
    with_metadata.inputs.push(approved_ref);
    metadata_policy.max_projected_prompt_bytes = 8_192;
    let codes = diagnostic_codes(
        graph(vec![with_metadata, reducer("reduce", &["analysis"])]),
        &metadata_policy,
        &projection_policy(),
    );
    assert!(codes.contains(&"projection.prompt_bytes"));
}

#[allow(dead_code)]
fn assert_public_policy_uses_ordered_collections(policy: &GraphPolicy) {
    let _: &BTreeSet<String> = &policy.allowed_roles;
    let _: &BTreeMap<String, ApprovedArtifact> = &policy.approved_artifacts;
}
