use super::*;

fn projection_policy_for(graph: &TaskGraph) -> ProjectionPolicy {
    let mut projection = cases::projection_policy();
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
fn diagnostics_are_sorted_deterministically_and_capped() {
    let make_invalid = |reverse: bool| {
        let mut nodes = (0..64)
            .map(|index| {
                let mut node = analysis(&format!("node-{index:02}"));
                node.role = TaskRole::Implement;
                node.model_policy = "forbidden-model".to_owned();
                node.permission_profile = "forbidden-permission".to_owned();
                node.workspace_mode = WorkspaceMode::GitWorktree;
                node.output_schema = "forbidden-schema".to_owned();
                node
            })
            .collect::<Vec<_>>();
        if reverse {
            nodes.reverse();
        }
        nodes
    };

    let diagnostics = validate_graph(
        graph(make_invalid(false)),
        &policy(),
        &cases::projection_policy(),
    )
    .expect_err("invalid graph")
    .diagnostics()
    .to_vec();
    let reversed = validate_graph(
        graph(make_invalid(true)),
        &policy(),
        &cases::projection_policy(),
    )
    .expect_err("invalid graph")
    .diagnostics()
    .to_vec();

    assert_eq!(diagnostics, reversed);
    assert_eq!(diagnostics.len(), 128);
    assert_eq!(
        diagnostics.last().expect("summary diagnostic").code,
        "validation.too_many_diagnostics"
    );
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.message.len() <= 512));
}

#[test]
fn malformed_graph_shape_hard_stops_with_stable_bounded_diagnostics() {
    let make_oversized = |reverse: bool| {
        let mut nodes = (0..1_024)
            .map(|index| analysis(&format!("node-{index:04}")))
            .collect::<Vec<_>>();
        if reverse {
            nodes.reverse();
        }
        graph(nodes)
    };

    let first = validate_graph(
        make_oversized(false),
        &policy(),
        &cases::projection_policy(),
    )
    .expect_err("oversized graph")
    .diagnostics()
    .to_vec();
    let reversed = validate_graph(make_oversized(true), &policy(), &cases::projection_policy())
        .expect_err("reversed oversized graph")
        .diagnostics()
        .to_vec();
    assert_eq!(first, reversed);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].code, "graph.shape");
    assert_eq!(
        first[0].message,
        "graph does not satisfy the bounded TaskGraph contract shape"
    );

    let mut first_analysis = analysis("first");
    first_analysis.dependencies = vec![task_id("second"); 1_024];
    let reference_heavy = graph(vec![
        first_analysis,
        analysis("second"),
        reducer("reduce", &["first", "second"]),
    ]);
    let diagnostics = validate_graph(
        reference_heavy.clone(),
        &policy(),
        &projection_policy_for(&reference_heavy),
    )
    .expect_err("oversized dependency list");
    assert_eq!(diagnostics.diagnostics().len(), 1);
    assert_eq!(diagnostics.diagnostics()[0].code, "graph.shape");
}
