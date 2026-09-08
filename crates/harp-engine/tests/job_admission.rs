use harp_artifacts::ArtifactStore;
use harp_contracts::WorkflowV2;
use harp_engine::{validate_command_backend_authority, validate_workflow_v2};
use harp_runtime::CommandJobQualification;
use std::collections::BTreeMap;

#[test]
fn admission_rejects_missing_containment_and_changed_qualification_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::open(tmp.path()).unwrap();
    let plan = validate_workflow_v2(
        WorkflowV2::from_json(include_str!("fixtures/workflow-v2.json").as_bytes()).unwrap(),
    )
    .unwrap();
    let task = &plan.workflow().tasks[0];
    let evidence = store
        .publish(b"operator-reviewed qualification receipt", "text/plain")
        .unwrap();
    let mut qualifications = BTreeMap::from([(
        task.task_id.clone(),
        CommandJobQualification {
            backend: task.backend.clone(),
            host_identity: "test-host".into(),
            evidence,
            idempotent_submission: true,
            owner_fencing: true,
            descendant_containment: false,
            independent_deadline: true,
            enforced_cpu_budget: true,
            enforced_storage_budget: true,
            enforced_gpu_budget: false,
        },
    )]);
    assert_eq!(
        validate_command_backend_authority(&plan, &qualifications, &store)
            .unwrap_err()
            .code,
        "workflow.backend_unsupported"
    );
    qualifications
        .get_mut(&task.task_id)
        .unwrap()
        .descendant_containment = true;
    validate_command_backend_authority(&plan, &qualifications, &store).unwrap();
    qualifications
        .get_mut(&task.task_id)
        .unwrap()
        .evidence
        .size_bytes += 1;
    assert_eq!(
        validate_command_backend_authority(&plan, &qualifications, &store)
            .unwrap_err()
            .code,
        "workflow.qualification_evidence"
    );
    qualifications.clear();
    assert_eq!(
        validate_command_backend_authority(&plan, &qualifications, &store)
            .unwrap_err()
            .code,
        "workflow.backend_missing"
    );
}
