use std::collections::BTreeMap;

use harp_contracts::{
    Budget, DynamicAgentCall, DynamicPipelineItem, DynamicWorkflow, DynamicWorkflowStep, NodeKind,
    RetryPolicy, TaskId, WorkspaceMode,
};
use harp_engine::{compile_dynamic_workflow, DynamicWorkflowCompileOptions, ExecutionPlan};
use serde_json::json;

fn agent(call_id: &str) -> DynamicAgentCall {
    DynamicAgentCall {
        call_id: call_id.to_owned(),
        prompt: format!("Run {call_id}."),
        output_schema: r#"{"type":"object"}"#.to_owned(),
        model_policy: "test-model".to_owned(),
        permission_profile: "read-only".to_owned(),
        workspace_mode: WorkspaceMode::Scratch,
        budget: Budget::new(100, 10, 1_000),
        retry_policy: RetryPolicy {
            max_transient_attempts: 1,
        },
    }
}

fn workflow(root: DynamicWorkflowStep) -> DynamicWorkflow {
    DynamicWorkflow {
        schema_version: 1,
        name: "research-workflow".to_owned(),
        root,
    }
}

fn compile(root: DynamicWorkflowStep) -> harp_engine::CompiledDynamicWorkflow {
    compile_dynamic_workflow(&workflow(root), &compile_options()).expect("workflow compiles")
}

fn compile_options() -> DynamicWorkflowCompileOptions {
    DynamicWorkflowCompileOptions {
        reducer_model_policy: "test-model".to_owned(),
        reducer_permission_profile: "read-only".to_owned(),
        reducer_workspace_mode: WorkspaceMode::Scratch,
        reducer_budget: Budget::new(100, 10, 1_000),
        reducer_output_schema: r#"{"type":"object"}"#.to_owned(),
        reducer_retry_policy: RetryPolicy {
            max_transient_attempts: 1,
        },
        scratch_root: "/private/tmp/harp-dynamic".to_owned(),
        base_instructions: "Follow the workflow.".to_owned(),
        checkpoint_instructions: "Checkpoint only for durable recovery.".to_owned(),
    }
}

fn dependencies(compiled: &harp_engine::CompiledDynamicWorkflow) -> BTreeMap<String, Vec<String>> {
    compiled
        .graph
        .nodes
        .iter()
        .map(|node| {
            (
                node.task_id.to_string(),
                node.dependencies
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

#[test]
fn compiles_sequence_parallel_and_reducer_dependencies() {
    let compiled = compile(DynamicWorkflowStep::Sequence {
        steps: vec![
            DynamicWorkflowStep::Agent(agent("discover")),
            DynamicWorkflowStep::Parallel {
                branches: vec![
                    DynamicWorkflowStep::Agent(agent("market")),
                    DynamicWorkflowStep::Agent(agent("technical")),
                ],
            },
            DynamicWorkflowStep::Agent(agent("synthesize")),
        ],
    });

    let dependency_map = dependencies(&compiled);
    assert_eq!(
        dependency_map["discover"].as_slice(),
        &[] as &[String],
        "first sequence step has no dependency"
    );
    assert_eq!(dependency_map["market"], ["discover"]);
    assert_eq!(dependency_map["technical"], ["discover"]);
    assert_eq!(dependency_map["synthesize"], ["market", "technical"]);
    assert_eq!(
        dependency_map["research-workflow-reduce"],
        ["discover", "market", "technical", "synthesize"]
    );

    let reducer = compiled
        .graph
        .nodes
        .iter()
        .find(|node| node.kind == NodeKind::Reducer)
        .expect("reducer node");
    assert_eq!(
        reducer.task_id,
        "research-workflow-reduce".parse::<TaskId>().unwrap()
    );
    assert!(reducer
        .instruction
        .contains("Do not inspect the filesystem"));
    assert_eq!(compiled.graph.nodes.len(), 5);
    assert_eq!(
        compiled.projection_policy.scratch_paths[&"market".parse::<TaskId>().unwrap()],
        "/private/tmp/harp-dynamic/market"
    );
}

#[test]
fn compiles_pipeline_as_per_item_stage_chains_without_global_stage_barrier() {
    let compiled = compile(DynamicWorkflowStep::Pipeline {
        items: vec![
            DynamicPipelineItem {
                item_id: "claim-a".to_owned(),
                input: json!({"claim": "A"}),
            },
            DynamicPipelineItem {
                item_id: "claim-b".to_owned(),
                input: json!({"claim": "B"}),
            },
        ],
        stages: vec![agent("extract"), agent("verify")],
    });

    let dependency_map = dependencies(&compiled);
    assert_eq!(dependency_map["claim-a-extract"], Vec::<String>::new());
    assert_eq!(dependency_map["claim-b-extract"], Vec::<String>::new());
    assert_eq!(dependency_map["claim-a-verify"], ["claim-a-extract"]);
    assert_eq!(dependency_map["claim-b-verify"], ["claim-b-extract"]);
    assert_eq!(
        dependency_map["research-workflow-reduce"],
        [
            "claim-a-extract",
            "claim-a-verify",
            "claim-b-extract",
            "claim-b-verify"
        ]
    );

    let claim_a_extract = compiled
        .graph
        .nodes
        .iter()
        .find(|node| node.task_id.to_string() == "claim-a-extract")
        .expect("claim-a extract node");
    assert!(
        claim_a_extract
            .instruction
            .contains("\"itemId\":\"claim-a\""),
        "pipeline item input is embedded in the stage instruction"
    );
}

#[test]
fn compiled_dynamic_workflow_passes_existing_graph_validation() {
    let compiled = compile(DynamicWorkflowStep::Sequence {
        steps: vec![
            DynamicWorkflowStep::Agent(agent("discover")),
            DynamicWorkflowStep::Parallel {
                branches: vec![
                    DynamicWorkflowStep::Agent(agent("market")),
                    DynamicWorkflowStep::Agent(agent("technical")),
                ],
            },
        ],
    });

    assert_eq!(
        compiled.graph_policy.allowed_model_policies,
        ["test-model"].into_iter().map(str::to_owned).collect()
    );
    assert_eq!(
        compiled.graph_policy.allowed_roles,
        ["explore", "reduce"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
    assert_eq!(
        compiled.projection_policy.base_instructions,
        "Follow the workflow."
    );
    let validated = ExecutionPlan::from(compiled)
        .validate()
        .expect("compiled graph validates");
    assert_eq!(
        validated.graph().reducer().to_string(),
        "research-workflow-reduce"
    );
    assert_eq!(validated.graph().topological_order().len(), 4);
}

#[test]
fn execution_plan_rejects_a_graph_policy_mismatch() {
    let compiled = compile(DynamicWorkflowStep::Sequence {
        steps: vec![
            DynamicWorkflowStep::Agent(agent("discover")),
            DynamicWorkflowStep::Agent(agent("synthesize")),
        ],
    });
    let mut wrong_policy = compiled.graph_policy.clone();
    wrong_policy.allowed_model_policies.clear();
    wrong_policy
        .allowed_model_policies
        .insert("another-model".to_owned());

    let error = ExecutionPlan::new(compiled.graph, wrong_policy, compiled.projection_policy)
        .validate()
        .expect_err("model policy mismatch is rejected");

    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "policy.model"));
}

#[test]
fn rejects_invalid_workflow_and_duplicate_compiled_task_ids() {
    let invalid = DynamicWorkflow {
        schema_version: 2,
        name: "bad".to_owned(),
        root: DynamicWorkflowStep::Agent(agent("only")),
    };
    assert!(compile_dynamic_workflow(&invalid, &compile_options()).is_err());

    let duplicate = workflow(DynamicWorkflowStep::Pipeline {
        items: vec![
            DynamicPipelineItem {
                item_id: "same-a".to_owned(),
                input: json!({}),
            },
            DynamicPipelineItem {
                item_id: "same".to_owned(),
                input: json!({}),
            },
        ],
        stages: vec![agent("b"), agent("a-b")],
    });
    let error = compile_dynamic_workflow(&duplicate, &compile_options())
        .expect_err("compiled task IDs collide");
    assert!(error.to_string().contains("duplicate task id"));
}
