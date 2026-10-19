use harp_contracts::BackendIdentity;
use harp_runtime::{JobBinding, JobObservation, JobStatus, JobTerminalReceipt};

fn binding() -> JobBinding {
    JobBinding {
        submission_key: "run/train/attempt-1".into(),
        request_sha256: "b".repeat(64),
        backend: BackendIdentity {
            kind: "localCommand".into(),
            version: "1".into(),
            config_sha256: "a".repeat(64),
        },
        host_identity: "host-boot-123".into(),
        owner_identity: "owner-456".into(),
        job_identity: "job-789".into(),
    }
}

#[test]
fn observation_cannot_switch_jobs_or_regress_cursor() {
    let expected = binding();
    let mut observation = JobObservation {
        binding: expected.clone(),
        next_cursor: 4,
        status: JobStatus::Running,
        evidence: vec![],
    };
    observation.validate_for(&expected, 3).unwrap();
    assert!(observation.validate_for(&expected, 5).is_err());
    observation.binding.host_identity = "rebooted-host".into();
    assert!(observation.validate_for(&expected, 3).is_err());
    observation.binding = expected.clone();
    observation.binding.submission_key = "different-submission".into();
    assert!(observation.validate_for(&expected, 3).is_err());
}

#[test]
fn exit_without_quiescence_is_not_terminal() {
    let expected = binding();
    let mut observation = JobObservation {
        binding: expected.clone(),
        next_cursor: 1,
        evidence: vec![],
        status: JobStatus::Terminal {
            receipt: JobTerminalReceipt {
                exit_code: Some(0),
                signal: None,
                loss_reason: None,
                process_tree_quiescent: false,
                evidence: vec![harp_contracts::ArtifactRef::sha256(
                    "e".repeat(64),
                    "application/json",
                    1,
                )
                .unwrap()],
            },
        },
    };
    assert!(observation.validate_for(&expected, 0).is_err());
    if let JobStatus::Terminal { receipt } = &mut observation.status {
        receipt.process_tree_quiescent = true;
    }
    observation.validate_for(&expected, 0).unwrap();
    if let JobStatus::Terminal { receipt } = &mut observation.status {
        receipt.signal = Some(9);
    }
    assert!(observation.validate_for(&expected, 0).is_err());
}

#[test]
fn lookup_binds_exact_request_and_rejects_invalid_request_even_when_unknown() {
    use harp_contracts::{ArtifactRef, TaskResources};
    use harp_runtime::{CommandJobRequest, JobLookup, JobLookupResponse, JobOwnership};
    let binding = binding();
    let mut request = CommandJobRequest {
        submission_key: binding.submission_key.clone(),
        request_sha256: binding.request_sha256.clone(),
        backend: binding.backend.clone(),
        host_identity: binding.host_identity.clone(),
        admitted_manifest: ArtifactRef::sha256("a".repeat(64), "application/json", 1).unwrap(),
        limits: TaskResources {
            agent_tokens: 0,
            cpu_seconds: 1,
            gpu_seconds: 0,
            wall_seconds: 1,
            storage_bytes: 1024,
        },
        ownership: JobOwnership {
            epoch: 1,
            authority_sha256: "c".repeat(64),
        },
    };
    let mut response = JobLookupResponse {
        submission_key: binding.submission_key.clone(),
        request_sha256: binding.request_sha256.clone(),
        backend: binding.backend.clone(),
        host_identity: binding.host_identity.clone(),
        result: JobLookup::Present { binding },
    };
    response.validate_for(&request).unwrap();
    request.request_sha256 = "d".repeat(64);
    assert!(response.validate_for(&request).is_err());
    response.request_sha256 = request.request_sha256.clone();
    assert!(response.validate_for(&request).is_err()); // nested binding still names old command
    response.result = JobLookup::Unknown;
    response.validate_for(&request).unwrap();
    request.ownership.epoch = 0;
    assert!(response.validate_for(&request).is_err());
}

#[test]
fn collection_binds_manifest_separately_from_execution_and_pages_objects() {
    use harp_contracts::ArtifactRef;
    use harp_runtime::JobCollection;
    let binding = binding();
    let manifest = ArtifactRef::sha256("a".repeat(64), "application/json", 64).unwrap();
    let mut page = JobCollection {
        binding: binding.clone(),
        output_manifest: manifest.clone(),
        next_cursor: 2,
        complete: false,
        objects: vec![],
    };
    page.validate_for(&binding, &manifest, 1).unwrap();
    page.complete = true;
    page.validate_for(&binding, &manifest, 2).unwrap();
    let other = ArtifactRef::sha256("b".repeat(64), "application/json", 64).unwrap();
    assert!(page.validate_for(&binding, &other, 1).is_err());
    assert!(page.validate_for(&binding, &manifest, 3).is_err());
    page.objects = vec![manifest.clone(); 65];
    assert!(page.validate_for(&binding, &manifest, 1).is_err());
}
