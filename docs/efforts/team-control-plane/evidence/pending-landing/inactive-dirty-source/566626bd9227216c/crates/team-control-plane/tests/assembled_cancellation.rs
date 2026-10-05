//! Public canonical cancellation: durable authority revocation precedes effects.
#[path = "support/episode_native.rs"]
#[allow(dead_code)]
mod episode_native;
#[path = "support/assembled.rs"]
#[allow(dead_code)]
mod support;
use support::*;
use team_control_plane::*;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_attempt_cancel_revokes_preadmitted_delayed_and_future_dispatch() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("cancel-preadmitted").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let episode_record = mailbox
        .record(&f.owner, mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    let packet = mailbox
        .record(
            &f.owner,
            mailbox::RecordKind::Input,
            serde_json::from_value(episode_record["input_packet"].clone()).unwrap(),
        )
        .await
        .unwrap();
    let native_root = f.root.join("cancel-native");
    std::fs::create_dir(&native_root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "kernel16-owned-os".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: native_root.to_str().unwrap().into(),
        endpoint: "cancel-fixture".into(),
        environment: "bwrap-private-git".into(),
    };
    let gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    let start = episode_runtime::StartRequest {
        id: Uuid::new_v4(),
        episode,
        execution: execution.id,
        runtime: identity.clone(),
        argv: vec!["/bin/true".into()],
    };
    gateway
        .prepare("assembled-worker", start.clone())
        .await
        .unwrap();
    let before = kernel::Kernel::new(f.broker.clone())
        .status(&f.owner)
        .await
        .unwrap();
    supervisor
        .start_with_fault(
            "assembled-worker",
            execution.id,
            vec![
                "/bin/sh".into(),
                "-c".into(),
                "printf forbidden > src/result".into(),
            ],
            30,
            supervisor::StartFault::BeforeMonitor,
        )
        .await
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let broker = f.broker.clone();
    let server =
        tokio::spawn(async move { axum::serve(listener, http::router(broker)).await.unwrap() });
    let client = reqwest::Client::new();
    let body = serde_json::json!({"schema_version":1,"key":"cancel-original","expected_revision":3,"expected_generation":1,"recovery_epoch":f.epoch,"command":{"kind":"cancel_attempt","task_id":f.task,"attempt_id":f.attempt,"reason":"Owner cancels exact current attempt"}});
    let response = client
        .post(format!("http://{address}/v1/commands"))
        .bearer_auth(&f.owner)
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.bytes().await.unwrap();
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(evidence.join("cancel-command-response.json"), &bytes).unwrap();
    if !status.is_success() {
        server.abort();
        f.finish().await;
        panic!(
            "canonical public attempt.cancel missing: status={status} response={}",
            String::from_utf8_lossy(&bytes)
        );
    }
    let cancelled: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(cancelled["disposition"], "cancellation_requested");
    assert_eq!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Task(f.task))
            .await
            .unwrap()
            .data["status"],
        "running",
        "desired cancellation is not completed effects"
    );
    assert!(
        supervisor::run_monitor(f.broker.clone(), execution.id)
            .await
            .is_err()
    );
    assert!(!execution.path.join("src/result").exists());
    assert!(
        supervisor
            .start(
                "assembled-worker",
                execution.id,
                vec!["/bin/true".into()],
                30
            )
            .await
            .is_err()
    );
    let retry = client
        .post(format!("http://{address}/v1/commands"))
        .bearer_auth(&f.owner)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert!(retry.status().is_success());
    assert_eq!(retry.bytes().await.unwrap(), bytes);
    for token in ["wrong-token", "assembled-verifier"] {
        let denied = client
            .post(format!("http://{address}/v1/commands"))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(
            denied.status(),
            reqwest::StatusCode::UNAUTHORIZED,
            "authentication/role must precede cached cancellation"
        );
    }
    let mut wrong_epoch = body.clone();
    wrong_epoch["recovery_epoch"] = serde_json::json!(Uuid::new_v4());
    assert_eq!(
        client
            .post(format!("http://{address}/v1/commands"))
            .bearer_auth(&f.owner)
            .json(&wrong_epoch)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    let admit = serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"implementation","input_packet":packet["id"],"input_digest":packet["digest"]});
    for key in ["cancel-preadmitted-admit", "cancel-fresh-admit"] {
        assert_eq!(
            mailbox
                .execute("assembled-worker", mail(f.epoch, key, admit.clone()))
                .await
                .unwrap_err()
                .code,
            ErrorCode::DependencyUnmet,
            "cancelled attempt cannot reuse cached or new episode admission"
        );
    }
    assert!(
        gateway
            .prepare("assembled-worker", start.clone())
            .await
            .is_err(),
        "cached prepared Start cannot regain authority"
    );
    let mut native = episode_native::NativeOsRuntime {
        root: native_root.clone(),
        identity,
        supervisor: supervisor.clone(),
        service: f.owner.clone(),
        lose_reply: false,
        hold_reply: false,
    };
    assert!(
        gateway
            .dispatch(&f.owner, start.id, &mut native)
            .await
            .is_err()
    );
    assert_eq!(
        std::fs::read_dir(&native_root).unwrap().count(),
        0,
        "cancelled queued Start performs no native effect"
    );
    let reconcile = serde_json::json!({"schema_version":1,"key":"cancel-reconcile","expected_revision":4,"expected_generation":1,"recovery_epoch":f.epoch,"command":{"kind":"reconcile_attempt","task_id":f.task,"attempt_id":f.attempt}});
    let response = client
        .post(format!("http://{address}/v1/commands"))
        .bearer_auth(&f.owner)
        .json(&reconcile)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let completed: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        completed["disposition"], "quarantined",
        "a registered launch with no certified PID1 cannot claim completed cancellation"
    );
    assert_eq!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Task(f.task))
            .await
            .unwrap()
            .data["status"],
        "running"
    );
    let after = kernel::Kernel::new(f.broker.clone())
        .status(&f.owner)
        .await
        .unwrap();
    for field in ["used_capacity", "admitted_attempts", "admitted_episodes"] {
        assert_eq!(
            before["missions"][0][field], after["missions"][0][field],
            "cancel/rejected dispatch never refunds or charges implicitly"
        );
    }
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "foreign-controller-grant",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: Uuid::new_v4(),
                    role: Role::Worker,
                    mission_id: Some(f.mission),
                    task_id: Some(f.task),
                    generation: Some(1),
                    token_hash: token_hash("foreign-current-worker"),
                    expires_in_seconds: 30,
                },
            ),
        )
        .await
        .unwrap();
    assert_eq!(
        client
            .post(format!("http://{address}/v1/commands"))
            .bearer_auth("foreign-current-worker")
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "a valid same-task Worker grant is not the current Attempt controller"
    );
    let new_epoch = Uuid::new_v4();
    let current_owner = f.broker.rotate_epoch(&f.owner, new_epoch).await.unwrap();
    assert_eq!(
        client
            .post(format!("http://{address}/v1/commands"))
            .bearer_auth(&f.owner)
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "old cached cancellation cannot cross recovery epoch rotation"
    );
    let recovered = Broker::connect(&f.dsn, new_epoch).await.unwrap();
    let historical = recovered
        .execute(
            &current_owner,
            envelope(
                new_epoch,
                "current-owner-observe-cancel",
                5,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    assert_eq!(historical.disposition, "quarantined");
    assert_eq!(
        historical.recovery_epoch, new_epoch,
        "current Owner can reconcile historical cancellation without renewing dispatch"
    );
    server.abort();
    f.finish().await;
}

#[tokio::test]
async fn lost_pause_response_fresh_status_exposes_control_revision_and_db_time() {
    let f = Fixture::new().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let broker = f.broker.clone();
    let server =
        tokio::spawn(async move { axum::serve(listener, http::router(broker)).await.unwrap() });
    let old_client = reqwest::Client::new();
    let pause = serde_json::json!({"mission":f.mission,"key":"lost-pause","expected_revision":0,"recovery_epoch":f.epoch,"paused":true,"rationale":"Owner pauses dispatch"});
    let response = old_client
        .post(format!("http://{address}/v1/kernel/pause"))
        .bearer_auth(&f.owner)
        .json(&pause)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    drop(response);
    drop(old_client);
    let fresh = reqwest::Client::new();
    let status: serde_json::Value = fresh
        .get(format!("http://{address}/v1/kernel/status"))
        .bearer_auth(&f.owner)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(
        evidence.join("fresh-status-after-lost-pause.json"),
        serde_json::to_vec(&status).unwrap(),
    )
    .unwrap();
    let mission = status["missions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == f.mission.to_string())
        .unwrap();
    assert_eq!(mission["control_revision"], 1);
    assert_eq!(
        mission["revision"], 2,
        "mission revision and control revision are independent"
    );
    assert_eq!(mission["dispatch_paused"], true);
    assert!(status["snapshot_time"].as_str().is_some());
    let resume = serde_json::json!({"mission":mission["id"],"key":"resume-from-fresh-view","expected_revision":mission["control_revision"],"recovery_epoch":status["recovery_epoch"],"paused":false,"rationale":"resume from durable status after lost response"});
    let result: serde_json::Value = fresh
        .post(format!("http://{address}/v1/kernel/pause"))
        .bearer_auth(&f.owner)
        .json(&resume)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(result["revision"], 2);
    let current: serde_json::Value = fresh
        .get(format!("http://{address}/v1/kernel/status"))
        .bearer_auth(&f.owner)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(current["missions"][0]["dispatch_paused"], false);
    assert_eq!(current["missions"][0]["control_revision"], 2);
    assert_eq!(
        fresh
            .get(format!("http://{address}/v1/kernel/status"))
            .bearer_auth("wrong-token")
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        fresh
            .post(format!("http://{address}/v1/kernel/pause"))
            .bearer_auth("assembled-worker")
            .json(&resume)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    server.abort();
    f.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_cancel_stops_actual_nested_writers_and_never_refunds_unknown() {
    use std::time::{Duration, Instant};
    for mode in ["known-physical", "unknown-usage", "unknown-effect"] {
        let f = Fixture::new().await;
        let episode = f.worker_episode(mode).await;
        let supervisor = f.supervisor().await;
        let execution = supervisor
            .create_episode(
                "assembled-worker",
                episode,
                vec![],
                supervisor::CreateFault::None,
            )
            .await
            .unwrap();
        let argv = vec![
            "/bin/sh".into(),
            "-c".into(),
            "setsid sh -c 'while :; do printf x >> src/live; sleep 0.02; done' & wait".into(),
        ];
        let native_root = f.root.join("cancel-native");
        std::fs::create_dir(&native_root).unwrap();
        let identity = gateway::RuntimeIdentity {
            runtime_kind: "kernel16-owned-os".into(),
            runtime_version: "1".into(),
            deployment: f.epoch.to_string(),
            thread_store: native_root.to_str().unwrap().into(),
            endpoint: "cancel-running".into(),
            environment: "bwrap-private-git".into(),
        };
        let gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
        let operation = Uuid::new_v4();
        if mode == "known-physical" {
            supervisor
                .start("assembled-worker", execution.id, argv, 30)
                .await
                .unwrap();
        } else {
            gateway
                .prepare(
                    "assembled-worker",
                    episode_runtime::StartRequest {
                        id: operation,
                        episode,
                        execution: execution.id,
                        runtime: identity.clone(),
                        argv,
                    },
                )
                .await
                .unwrap();
            let mut runtime = episode_native::NativeOsRuntime {
                root: native_root.clone(),
                identity,
                supervisor: supervisor.clone(),
                service: f.owner.clone(),
                lose_reply: mode == "unknown-effect",
                hold_reply: false,
            };
            let dispatched = gateway
                .dispatch(&f.owner, operation, &mut runtime)
                .await
                .unwrap();
            assert_eq!(
                dispatched.state,
                if mode == "unknown-effect" {
                    gateway::OperationState::OutcomeUnknown
                } else {
                    gateway::OperationState::Confirmed
                }
            );
            assert_eq!(dispatched.usage, episode_runtime::UsageObservation::Unknown);
        }
        let deadline = Instant::now() + Duration::from_secs(8);
        let before_members = loop {
            let view = supervisor.inspect(&f.owner, execution.id).await.unwrap();
            if view.state == supervisor::ExecutionState::Running
                && view.members.len() >= 3
                && std::fs::read(execution.path.join("src/live")).is_ok_and(|b| b.len() >= 3)
            {
                break view.members;
            }
            assert!(
                Instant::now() < deadline,
                "real nested workload did not become Running: {view:?}"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let broker = f.broker.clone();
        let server =
            tokio::spawn(async move { axum::serve(listener, http::router(broker)).await.unwrap() });
        let client = reqwest::Client::new();
        let token = if mode == "known-physical" {
            "assembled-worker"
        } else {
            &f.owner
        };
        let cancel = serde_json::json!({"schema_version":1,"key":"cancel-live","expected_revision":3,"expected_generation":1,"recovery_epoch":f.epoch,"command":{"kind":"cancel_attempt","task_id":f.task,"attempt_id":f.attempt,"reason":"cancel actual nested live writer"}});
        let response = client
            .post(format!("http://{address}/v1/commands"))
            .bearer_auth(token)
            .json(&cancel)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
        assert_eq!(
            response.json::<serde_json::Value>().await.unwrap()["disposition"],
            "cancellation_requested"
        );
        let mut revision = 4;
        let disposition = loop {
            let command = serde_json::json!({"schema_version":1,"key":format!("observe-cancel-{revision}"),"expected_revision":revision,"expected_generation":1,"recovery_epoch":f.epoch,"command":{"kind":"reconcile_attempt","task_id":f.task,"attempt_id":f.attempt}});
            let result: serde_json::Value = client
                .post(format!("http://{address}/v1/commands"))
                .bearer_auth(&f.owner)
                .json(&command)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap()
                .json()
                .await
                .unwrap();
            revision = result["revision"].as_i64().unwrap();
            let disposition = result["disposition"].as_str().unwrap().to_string();
            let view = supervisor.inspect(&f.owner, execution.id).await.unwrap();
            if view.state == supervisor::ExecutionState::Stopped
                && disposition != "cancellation_requested"
            {
                assert!(view.members.is_empty());
                break disposition;
            }
            assert!(
                Instant::now() < deadline,
                "cancel did not establish strict Stop: {view:?}"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        let output = std::fs::read(execution.path.join("src/live")).unwrap();
        tokio::time::sleep(Duration::from_millis(120)).await;
        assert_eq!(
            std::fs::read(execution.path.join("src/live")).unwrap(),
            output,
            "nested writer actually stopped"
        );
        for member in &before_members {
            let stat = std::fs::read_to_string(format!("/proc/{}/stat", member.pid));
            if let Ok(stat) = stat {
                let fields: Vec<_> = stat
                    .rsplit_once(')')
                    .unwrap()
                    .1
                    .split_whitespace()
                    .collect();
                assert!(
                    fields[19].parse::<u64>().unwrap() != member.start_ticks || fields[0] == "Z",
                    "original observed process remains live"
                );
            }
        }
        let snapshot = f
            .broker
            .snapshot(&f.owner, SnapshotQuery::Task(f.task))
            .await
            .unwrap();
        if mode != "known-physical" {
            assert_eq!(disposition, "quarantined");
            assert_eq!(snapshot.data["status"], "running");
        } else {
            assert_eq!(disposition, "cancelled");
            assert_eq!(snapshot.data["status"], "cancelled");
        }
        assert_eq!(
            f.broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data["used_capacity"],
            1,
            "domain cancellation does not infer reservation refund"
        );
        if mode == "known-physical" {
            supervisor.release(&f.owner, execution.id).await.unwrap();
            assert_eq!(
                supervisor
                    .inspect(&f.owner, execution.id)
                    .await
                    .unwrap()
                    .state,
                supervisor::ExecutionState::Released
            );
            assert_eq!(
                supervisor
                    .release(&f.owner, execution.id)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet,
                "already Released cannot refund again"
            );
            assert_eq!(
                f.broker
                    .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                    .await
                    .unwrap()
                    .data["used_capacity"],
                0
            );
        } else {
            assert_eq!(
                supervisor
                    .release(&f.owner, execution.id)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet
            );
            assert_eq!(
                gateway
                    .inspect(&f.owner, operation)
                    .await
                    .unwrap()
                    .operation
                    .usage,
                episode_runtime::UsageObservation::Unknown
            );
        }
        let mailbox = mailbox::MailboxService::new(f.broker.clone());
        let original = mailbox
            .record(&f.owner, mailbox::RecordKind::Episode, episode)
            .await
            .unwrap();
        let packet = mailbox
            .record(
                &f.owner,
                mailbox::RecordKind::Input,
                serde_json::from_value(original["input_packet"].clone()).unwrap(),
            )
            .await
            .unwrap();
        let admission=mailbox.execute("assembled-worker",mail(f.epoch,"after-stopped-cancel-admit",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":Uuid::new_v4(),"attempt":f.attempt,"purpose":"implementation","input_packet":packet["id"],"input_digest":packet["digest"]}))).await;
        assert_eq!(
            admission.unwrap_err().code,
            ErrorCode::DependencyUnmet,
            "even after physical Stop the old current grant cannot admit another Episode"
        );
        assert!(
            supervisor
                .start(
                    "assembled-worker",
                    execution.id,
                    vec!["/bin/true".into()],
                    30
                )
                .await
                .is_err()
        );
        let evidence =
            std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
        std::fs::write(evidence.join(format!("cancel-{mode}-evidence.json")),serde_json::to_vec(&serde_json::json!({"observed_before":before_members,"disposition":disposition,"task":snapshot.data,"stopped_output_bytes":output.len()})).unwrap()).unwrap();
        server.abort();
        f.finish().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_monitor_claim_rechecks_durable_attempt_cancellation_before_workload() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("cancel-final-claim").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    supervisor
        .start_with_fault(
            "assembled-worker",
            execution.id,
            vec![
                "/bin/sh".into(),
                "-c".into(),
                "printf forbidden > src/result".into(),
            ],
            30,
            supervisor::StartFault::BeforeMonitor,
        )
        .await
        .unwrap();
    let claimed = std::sync::Arc::new(tokio::sync::Notify::new());
    let resume = std::sync::Arc::new(tokio::sync::Notify::new());
    let monitor = tokio::spawn(supervisor::run_monitor_with_gate(
        f.broker.clone(),
        execution.id,
        supervisor::MonitorGate {
            claimed: claimed.clone(),
            resume: resume.clone(),
        },
    ));
    tokio::time::timeout(std::time::Duration::from_secs(8), claimed.notified())
        .await
        .expect("actual monitor must durably claim before cancellation");
    let claimed_view = supervisor.inspect(&f.owner, execution.id).await.unwrap();
    assert!(claimed_view.controller.is_some());
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "cancel-after-claim",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "Owner cancels after actual monitor claim, before workload".into(),
                },
            ),
        )
        .await
        .unwrap();
    assert_eq!(cancel.disposition, "cancellation_requested");
    resume.notify_one();
    let result = tokio::time::timeout(std::time::Duration::from_secs(8), monitor)
        .await
        .unwrap()
        .unwrap();
    assert!(result.is_err());
    assert!(!execution.path.join("src/result").exists());
    assert!(
        supervisor
            .result(&f.owner, execution.id)
            .await
            .unwrap()
            .is_none()
    );
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(evidence.join("cancel-after-monitor-claim.json"),serde_json::to_vec(&serde_json::json!({"claimed_view":claimed_view,"cancel":cancel,"monitor_error":result.unwrap_err().code})).unwrap()).unwrap();
    f.finish().await;
}

#[path = "support/gateway_fault.rs"]
#[allow(dead_code)]
mod gateway_fault;
#[tokio::test]
async fn confirmed_legacy_start_without_bound_physical_stop_remains_quarantined() {
    let f = Fixture::new().await;
    let root = f.root.join("legacy-unbound-native");
    std::fs::create_dir(&root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "model-free-native-file-fixture".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "unsupported-closed-world".into(),
        environment: "zero-model-child-process".into(),
    };
    let gateway = gateway::Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(&f.owner, binding, f.admission, &identity, None, 1)
        .await
        .unwrap();
    let mut runtime =
        gateway_fault::FaultRuntime::new(root.clone(), gateway_fault::Fault::None, identity);
    let mut start = Uuid::nil();
    for kind in [
        gateway::OperationKind::Create,
        gateway::OperationKind::Start,
    ] {
        let operation = Uuid::new_v4();
        let intent = gateway::Intent {
            id: operation,
            admission_id: f.admission,
            binding_id: binding,
            generation: 1,
            kind,
            arguments: serde_json::json!({"scope":"actual native file effects with no supervised execution certificate"}),
        };
        gateway.prepare(&f.owner, intent).await.unwrap();
        let effect = gateway
            .dispatch(&f.owner, operation, &mut runtime)
            .await
            .unwrap();
        assert_eq!(effect.state, gateway::OperationState::Confirmed);
        if kind == gateway::OperationKind::Start {
            start = operation;
        }
    }
    assert_eq!(runtime.invocation_count().unwrap(), 2);
    assert_eq!(runtime.effect_count().unwrap(), 2);
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "cancel-unbound-start",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "confirmed Start lacks any physical Stop certificate".into(),
                },
            ),
        )
        .await
        .unwrap();
    let reconciled = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "reconcile-unbound-start",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let snapshot = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(f.task))
        .await
        .unwrap();
    let mission = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(evidence.join("unbound-start-cancellation.json"),serde_json::to_vec(&serde_json::json!({"start":start,"cancel":cancel,"reconciled":reconciled,"task":snapshot.data,"mission":mission.data,"actual_native_invocations":runtime.invocation_count().unwrap(),"actual_native_effects":runtime.effect_count().unwrap()})).unwrap()).unwrap();
    f.finish().await;
    assert_eq!(
        reconciled.disposition, "quarantined",
        "Confirmed legacy RPC with absent environment binding cannot prove stopped effects"
    );
    assert_eq!(snapshot.data["status"], "running");
    assert_eq!(mission.data["used_capacity"], 1);
}

#[tokio::test]
async fn confirmed_legacy_start_cannot_borrow_stopped_workspace_from_arguments() {
    let f = Fixture::new().await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_attempt("assembled-worker", f.attempt, supervisor::CreateFault::None)
        .await
        .unwrap();
    supervisor
        .start(
            "assembled-worker",
            execution.id,
            vec!["/bin/true".into()],
            30,
        )
        .await
        .unwrap();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    loop {
        if supervisor
            .inspect(&f.owner, execution.id)
            .await
            .unwrap()
            .state
            == supervisor::ExecutionState::Stopped
        {
            break;
        }
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let root = f.root.join("legacy-borrowed-native");
    std::fs::create_dir(&root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "model-free-native-file-fixture".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "unsupported-closed-world".into(),
        environment: "zero-model-child-process".into(),
    };
    let gateway = gateway::Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(&f.owner, binding, f.admission, &identity, None, 1)
        .await
        .unwrap();
    let mut runtime =
        gateway_fault::FaultRuntime::new(root.clone(), gateway_fault::Fault::None, identity);
    let mut start = Uuid::nil();
    for kind in [
        gateway::OperationKind::Create,
        gateway::OperationKind::Start,
    ] {
        let operation = Uuid::new_v4();
        let intent = gateway::Intent {
            id: operation,
            admission_id: f.admission,
            binding_id: binding,
            generation: 1,
            kind,
            arguments: serde_json::json!({"execution":execution.id,"scope":"unrelated native effects borrowing caller named Stop certificate"}),
        };
        gateway.prepare(&f.owner, intent).await.unwrap();
        let effect = gateway
            .dispatch(&f.owner, operation, &mut runtime)
            .await
            .unwrap();
        assert_eq!(effect.state, gateway::OperationState::Confirmed);
        if kind == gateway::OperationKind::Start {
            start = operation;
        }
    }
    assert_eq!(runtime.invocation_count().unwrap(), 2);
    assert_eq!(runtime.effect_count().unwrap(), 2);
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "cancel-unbound-start",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "confirmed Start lacks any physical Stop certificate".into(),
                },
            ),
        )
        .await
        .unwrap();
    let reconciled = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "reconcile-unbound-start",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let snapshot = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(f.task))
        .await
        .unwrap();
    let mission = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(evidence.join("borrowed-stop-cancellation.json"),serde_json::to_vec(&serde_json::json!({"start":start,"cancel":cancel,"reconciled":reconciled,"task":snapshot.data,"mission":mission.data,"actual_native_invocations":runtime.invocation_count().unwrap(),"actual_native_effects":runtime.effect_count().unwrap()})).unwrap()).unwrap();
    f.finish().await;
    assert_eq!(
        reconciled.disposition, "quarantined",
        "Caller-named stopped workspace cannot certify an unrelated confirmed native effect"
    );
    assert_eq!(snapshot.data["status"], "running");
    assert_eq!(mission.data["used_capacity"], 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_and_imported_episode_cannot_share_one_physical_reservation() {
    let mut violations = Vec::new();
    for order in ["legacy-first", "episode-first", "released-legacy"] {
        let f = Fixture::new().await;
        let episode = f.worker_episode(order).await;
        let supervisor = f.supervisor().await;
        let first = if order == "episode-first" {
            supervisor
                .create_episode(
                    "assembled-worker",
                    episode,
                    vec![],
                    supervisor::CreateFault::None,
                )
                .await
                .unwrap()
        } else {
            supervisor
                .create_attempt("assembled-worker", f.attempt, supervisor::CreateFault::None)
                .await
                .unwrap()
        };
        supervisor
            .start(
                "assembled-worker",
                first.id,
                vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    if order == "released-legacy" {
                        "true"
                    } else {
                        "while :; do printf x >> src/live; sleep 0.02; done"
                    }
                    .into(),
                ],
                30,
            )
            .await
            .unwrap();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
        loop {
            let view = supervisor.inspect(&f.owner, first.id).await.unwrap();
            let ready = if order == "released-legacy" {
                view.state == supervisor::ExecutionState::Stopped
            } else {
                view.state == supervisor::ExecutionState::Running
                    && std::fs::read(first.path.join("src/live")).is_ok_and(|b| !b.is_empty())
            };
            if ready {
                break;
            }
            assert!(tokio::time::Instant::now() < deadline);
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        if order == "released-legacy" {
            supervisor.release(&f.owner, first.id).await.unwrap();
        }
        let before_second = f
            .broker
            .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap();
        let second = if order == "episode-first" {
            supervisor
                .create_attempt("assembled-worker", f.attempt, supervisor::CreateFault::None)
                .await
        } else {
            supervisor
                .create_episode(
                    "assembled-worker",
                    episode,
                    vec![],
                    supervisor::CreateFault::None,
                )
                .await
        };
        if let Ok(second) = second {
            let started = supervisor
                .start(
                    "assembled-worker",
                    second.id,
                    vec![
                        "/bin/sh".into(),
                        "-c".into(),
                        "while :; do printf y >> src/live; sleep 0.02; done".into(),
                    ],
                    30,
                )
                .await;
            if started.is_ok() {
                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
                loop {
                    let view = supervisor.inspect(&f.owner, second.id).await.unwrap();
                    if view.state == supervisor::ExecutionState::Running
                        && std::fs::read(second.path.join("src/live")).is_ok_and(|b| !b.is_empty())
                    {
                        break;
                    }
                    assert!(tokio::time::Instant::now() < deadline);
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
                let mission = f
                    .broker
                    .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                    .await
                    .unwrap();
                violations.push(serde_json::json!({"order":order,"first":supervisor.inspect(&f.owner, first.id).await.unwrap(),"second":supervisor.inspect(&f.owner, second.id).await.unwrap(),"mission":mission.data}));
                supervisor.cancel(&f.owner, second.id).await.unwrap();
            }
        } else {
            assert_eq!(second.unwrap_err().code, ErrorCode::DependencyUnmet);
            assert_eq!(
                f.broker
                    .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                    .await
                    .unwrap()
                    .data,
                before_second.data
            );
        }
        if order != "released-legacy" {
            supervisor.cancel(&f.owner, first.id).await.unwrap();
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
            loop {
                if supervisor
                    .reconcile(&f.owner, first.id)
                    .await
                    .unwrap()
                    .state
                    == supervisor::ExecutionState::Stopped
                {
                    break;
                }
                assert!(tokio::time::Instant::now() < deadline);
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
            let mut released_before_final_wait = false;
            if supervisor
                .inspect(&f.owner, first.id)
                .await
                .unwrap()
                .manifest
                .episode
                .is_some()
            {
                let before_result = supervisor.result(&f.owner, first.id).await.unwrap();
                let evidence =
                    std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
                std::fs::write(evidence.join("reservation-stop-finalization.json"),serde_json::to_vec(&serde_json::json!({"execution":first.id,"result_before_release":before_result,"workspace":supervisor.inspect(&f.owner,first.id).await.unwrap()})).unwrap()).unwrap();
                if before_result.is_none() {
                    match supervisor.release(&f.owner, first.id).await {
                        Ok(()) => {
                            assert!(
                                supervisor
                                    .result(&f.owner, first.id)
                                    .await
                                    .unwrap()
                                    .is_some(),
                                "successful release must have durable finalization"
                            );
                            released_before_final_wait = true;
                        }
                        Err(error) => assert_eq!(error.code, ErrorCode::DependencyUnmet),
                    }
                    std::fs::write(evidence.join("reservation-early-release-outcome.json"),serde_json::to_vec(&serde_json::json!({"released":released_before_final_wait,"result_after":supervisor.result(&f.owner,first.id).await.unwrap()})).unwrap()).unwrap();
                }
                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
                while supervisor
                    .result(&f.owner, first.id)
                    .await
                    .unwrap()
                    .is_none()
                {
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "launched reservation fixture needs actual durable monitor finalization"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
            }
            if !released_before_final_wait {
                supervisor.release(&f.owner, first.id).await.unwrap();
            }
            assert_eq!(
                supervisor
                    .release(&f.owner, first.id)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet
            );
        }
        assert_eq!(
            f.broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data["used_capacity"],
            0
        );
        f.finish().await;
    }
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(
        evidence.join("reservation-sharing.json"),
        serde_json::to_vec(&violations).unwrap(),
    )
    .unwrap();
    assert!(
        violations.is_empty(),
        "distinct physical executions ran using one or an already released reservation: {}",
        violations.len()
    );
}

#[tokio::test]
async fn cancelled_attempt_retains_historical_gateway_progress_without_pending_effects() {
    use temporal::OperationDriver;
    let f = Fixture::new().await;
    let root = f.root.join("cancel-journal-history");
    std::fs::create_dir(&root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "model-free-native-file-fixture".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "unsupported-closed-world".into(),
        environment: "zero-model-child-process".into(),
    };
    let gateway = gateway::Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(&f.owner, binding, f.admission, &identity, None, 1)
        .await
        .unwrap();
    let mut runtime =
        gateway_fault::FaultRuntime::new(root.clone(), gateway_fault::Fault::None, identity);
    let create = Uuid::new_v4();
    let start = Uuid::new_v4();
    for (id, kind) in [
        (create, gateway::OperationKind::Create),
        (start, gateway::OperationKind::Start),
    ] {
        gateway.prepare(&f.owner,gateway::Intent {id,admission_id:f.admission,binding_id:binding,generation:1,kind,arguments:serde_json::json!({"scope":"retained history versus fenced pending effect"})}).await.unwrap();
        if id == create {
            assert_eq!(
                gateway
                    .dispatch(&f.owner, id, &mut runtime)
                    .await
                    .unwrap()
                    .state,
                gateway::OperationState::Confirmed
            );
        }
    }
    let bridge = temporal::Bridge::new(f.broker.clone());
    let reference = bridge
        .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "cancel-journal",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "revoke pending Start while preserving durable Create observation"
                        .into(),
                },
            ),
        )
        .await
        .unwrap();
    let driver = temporal::GatewayOperationDriver::new(
        bridge.clone(),
        gateway.clone(),
        f.owner.clone(),
        Box::new(runtime),
    );
    let progress = driver.drive(&reference, f.admission.into()).await;
    let historical = gateway.read(&f.owner, create).await.unwrap();
    let pending = gateway.read(&f.owner, start).await.unwrap();
    let count = std::fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.path().extension().is_some_and(|v| v == "invocation"))
        .count();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join("cancel-journal-progress.json"),serde_json::to_vec(&serde_json::json!({"progress":format!("{progress:?}"),"historical":historical,"pending":pending,"invocations":count})).unwrap()).unwrap();
    bridge.close(&f.owner, &reference).await.unwrap();
    assert_eq!(
        driver
            .drive(&reference, f.admission.into())
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    let successor = bridge
        .register(
            &f.owner,
            temporal::AggregateKind::Task,
            f.task,
            reference.generation,
        )
        .await
        .unwrap();
    assert_eq!(successor.generation, reference.generation + 1);
    assert_eq!(
        driver
            .drive(&reference, f.admission.into())
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    f.finish().await;
    let progress = progress
        .expect("terminal cancellation must not fail retained workflow journal observation");
    assert!(
        progress
            .journals
            .contains(&(create, gateway::OperationState::Confirmed))
    );
    assert!(
        progress
            .journals
            .contains(&(start, gateway::OperationState::Prepared))
    );
    assert!(progress.limitation.is_some());
    assert_eq!(
        count, 1,
        "cancelled Prepared Start cannot create a new native effect"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopped_launched_episode_retains_workspace_until_monitor_result_is_durable() {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    struct LauncherPin(OwnedFd);
    impl Drop for LauncherPin {
        fn drop(&mut self) {
            // A failed assertion cannot strand the exact stopped fixture launcher.
            unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    self.0.as_raw_fd(),
                    libc::SIGCONT,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                );
            }
        }
    }
    let f = Fixture::new().await;
    let supervisor = f.supervisor().await;
    let episode = f.worker_episode("finalization-worker").await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    supervisor
        .start(
            "assembled-worker",
            execution.id,
            vec![
                "/bin/sh".into(),
                "-c".into(),
                "while :; do printf live >> src/live; sleep 0.03; done".into(),
            ],
            30,
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
    let view = loop {
        let view = supervisor.inspect(&f.owner, execution.id).await.unwrap();
        if view.state == supervisor::ExecutionState::Running
            && execution.path.join("src/live").exists()
        {
            break view;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "actual workload did not start: {view:?}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    };
    let launcher = view.launcher.clone().unwrap();
    let controller = view.controller.clone().unwrap();
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", launcher.pid)).unwrap();
    assert_eq!(
        stat.rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(19)
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        launcher.start_ticks
    );
    let exe = std::fs::read_link(format!("/proc/{}/exe", launcher.pid)).unwrap();
    assert_eq!(exe.file_name().unwrap(), "bwrap");
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, launcher.pid, 0) };
    assert!(raw >= 0);
    let pin = LauncherPin(unsafe { OwnedFd::from_raw_fd(raw as i32) });
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", launcher.pid)).unwrap();
    assert_eq!(
        stat.rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(19)
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        launcher.start_ticks
    );
    assert_eq!(
        unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                pin.0.as_raw_fd(),
                libc::SIGSTOP,
                std::ptr::null::<libc::siginfo_t>(),
                0,
            )
        },
        0
    );
    supervisor.cancel(&f.owner, execution.id).await.unwrap();
    let stopped = loop {
        let view = supervisor.reconcile(&f.owner, execution.id).await.unwrap();
        if view.state == supervisor::ExecutionState::Stopped {
            break view;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "actual namespace Stop not proven: {view:?}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    };
    assert!(stopped.members.is_empty());
    let before = supervisor.result(&f.owner, execution.id).await.unwrap();
    let early = supervisor.release(&f.owner, execution.id).await;
    // Keep the launcher stopped until the monitor's original bounded wait ends;
    // its actual SIGKILL wait result must be retained, never fabricated as exit0.
    let result_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let result = loop {
        let result = supervisor.result(&f.owner, execution.id).await.unwrap();
        if result.is_some() || std::time::Instant::now() >= result_deadline {
            break result;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
    drop(pin);
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join("stopped-result-release.json"),serde_json::to_vec(&serde_json::json!({"execution":execution.id,"launcher":launcher,"controller":controller,"launcher_exe":exe,"stopped":stopped,"result_before_release":before,"early_release":format!("{early:?}"),"result":result})).unwrap()).unwrap();
    if early.is_err() && result.is_some() {
        supervisor.release(&f.owner, execution.id).await.unwrap();
    }
    f.finish().await;
    assert!(
        before.is_none(),
        "fixture must hit actual Stop-before-result boundary"
    );
    assert_eq!(
        early.unwrap_err().code,
        ErrorCode::DependencyUnmet,
        "launched episode cannot delete workspace before final result"
    );
    let result = result.expect("actual monitor result must survive release race");
    assert!(result.namespace_stopped);
    assert_eq!(result.exit_code, None);
    assert_eq!(result.signal, Some(libc::SIGKILL));
}

struct UnregisteredEpisodeEffect(gateway_fault::FaultRuntime, Option<std::process::Child>);
impl Drop for UnregisteredEpisodeEffect {
    fn drop(&mut self) {
        if let Some(child) = self.1.as_mut() {
            if child.try_wait().unwrap().is_none() {
                child.kill().unwrap();
            }
            child.wait().unwrap();
        }
    }
}
impl episode_runtime::EpisodeRuntime for UnregisteredEpisodeEffect {
    fn identity(&self) -> gateway::RuntimeIdentity {
        self.0.identity.clone()
    }
    fn capabilities(&self) -> gateway::CapabilityProfile {
        gateway::RuntimeAdapter::inspect(&self.0)
    }
    fn start<'a>(
        &'a mut self,
        request: &'a episode_runtime::PreparedStart,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = episode_runtime::EpisodeReply> + Send + 'a>,
    > {
        Box::pin(async move {
            let intent = gateway::Intent {
                id: request.start.id,
                admission_id: Uuid::nil(),
                kind: gateway::OperationKind::Start,
                binding_id: request.start.episode,
                arguments: serde_json::json!({"execution":request.start.execution}),
                generation: 1,
            };
            self.1 = Some(
                std::process::Command::new("/bin/sleep")
                    .arg("30")
                    .spawn()
                    .unwrap(),
            );
            match gateway::RuntimeAdapter::execute(&mut self.0, &intent, None) {
                gateway::RuntimeReply::Confirmed {
                    thread_id,
                    effect_id,
                } => episode_runtime::EpisodeReply::Confirmed(episode_runtime::NativeTurn {
                    thread: thread_id.unwrap(),
                    turn: effect_id.clone(),
                    effect: effect_id,
                }),
                _ => panic!("actual external fixture did not confirm"),
            }
        })
    }
    fn observe<'a>(
        &'a mut self,
        _: &'a episode_runtime::PreparedStart,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = episode_runtime::EpisodeObservation> + Send + 'a>,
    > {
        Box::pin(async {
            episode_runtime::EpisodeObservation::Inconclusive(
                "unregistered effect has no independent physical binding".into(),
            )
        })
    }
}
#[tokio::test]
async fn confirmed_episode_effect_cannot_borrow_unused_workspace_stop() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("unregistered-episode-effect").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let root = f.root.join("unregistered-episode-native");
    std::fs::create_dir(&root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "unregistered-external-effect".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "actual-child-process".into(),
        environment: "model-free".into(),
    };
    let gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    let operation = Uuid::new_v4();
    gateway
        .prepare(
            "assembled-worker",
            episode_runtime::StartRequest {
                id: operation,
                episode,
                execution: execution.id,
                runtime: identity.clone(),
                argv: vec!["/bin/true".into()],
            },
        )
        .await
        .unwrap();
    let mut runtime = UnregisteredEpisodeEffect(
        gateway_fault::FaultRuntime::new(root, gateway_fault::Fault::None, identity),
        None,
    );
    let journal = gateway
        .dispatch(&f.owner, operation, &mut runtime)
        .await
        .unwrap();
    assert_eq!(journal.state, gateway::OperationState::Confirmed);
    assert_eq!(journal.usage, episode_runtime::UsageObservation::Unknown);
    assert_eq!(runtime.0.effect_count().unwrap(), 1);
    assert!(runtime.1.as_mut().unwrap().try_wait().unwrap().is_none());
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "cancel-unbound-episode",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "actual confirmed external effect never launched allocated workspace"
                        .into(),
                },
            ),
        )
        .await
        .unwrap();
    let reconciled = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "reconcile-unbound-episode",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let view = supervisor.inspect(&f.owner, execution.id).await.unwrap();
    assert_eq!(view.state, supervisor::ExecutionState::Stopped);
    let survivor_pid = runtime.1.as_ref().unwrap().id();
    assert!(
        runtime.1.as_mut().unwrap().try_wait().unwrap().is_none(),
        "unregistered external execution survives canonical workspace Stop"
    );
    let status = kernel::Kernel::new(f.broker.clone())
        .status(&f.owner)
        .await
        .unwrap();
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(evidence.join("confirmed-unbound-episode.json"),serde_json::to_vec(&serde_json::json!({"journal":journal,"reconciled":reconciled,"workspace":view,"status":status,"surviving_owned_child":survivor_pid})).unwrap()).unwrap();
    drop(runtime);
    f.finish().await;
    assert_eq!(
        reconciled.disposition, "quarantined",
        "unused workspace Stop cannot certify an unrelated confirmed episode effect"
    );
}

#[tokio::test]
async fn paused_prepared_episode_remains_guarded_workflow_progress() {
    use temporal::OperationDriver;
    let f = Fixture::new().await;
    let episode = f.worker_episode("paused-episode").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let root = f.root.join("paused-episode-native");
    std::fs::create_dir(&root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "kernel16-inprocess-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "owned-local-subprocess".into(),
        environment: "bwrap-private-git".into(),
    };
    let gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    let operation = Uuid::new_v4();
    gateway
        .prepare(
            "assembled-worker",
            episode_runtime::StartRequest {
                id: operation,
                episode,
                execution: execution.id,
                runtime: identity.clone(),
                argv: vec!["/bin/true".into()],
            },
        )
        .await
        .unwrap();
    let runtime = episode_native::NativeOsRuntime {
        root: root.clone(),
        identity,
        supervisor: supervisor.clone(),
        service: f.owner.clone(),
        lose_reply: false,
        hold_reply: false,
    };
    let driver = episode_runtime::EpisodeOperationDriver::new(
        gateway,
        f.owner.clone(),
        Box::new(runtime),
        None,
    );
    let bridge = temporal::Bridge::new(f.broker.clone());
    let reference = bridge
        .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    kernel::Kernel::new(f.broker.clone())
        .pause(
            &f.owner,
            f.mission,
            "pause-episode-driver",
            0,
            true,
            "durable workflow must wait",
        )
        .await
        .unwrap();
    let progress = driver
        .drive(&reference, temporal::WaitRef::episode(episode))
        .await;
    assert!(std::fs::read_dir(&root).unwrap().next().is_none());

    let missing = temporal::WaitRef::episode(Uuid::new_v4());
    assert_eq!(
        driver
            .drive(&reference, missing)
            .await
            .unwrap()
            .limitation
            .as_deref(),
        Some("episode_journal_missing")
    );
    bridge.close(&f.owner, &reference).await.unwrap();
    assert_eq!(
        driver
            .drive(&reference, temporal::WaitRef::episode(episode))
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    assert_eq!(
        driver.drive(&reference, missing).await.unwrap_err().code,
        ErrorCode::StaleGeneration
    );
    let successor = bridge
        .register(
            &f.owner,
            temporal::AggregateKind::Task,
            f.task,
            reference.generation,
        )
        .await
        .unwrap();
    assert_eq!(successor.generation, reference.generation + 1);
    assert_eq!(
        driver
            .drive(&reference, temporal::WaitRef::episode(episode))
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    supervisor.cancel(&f.owner, execution.id).await.unwrap();
    supervisor.release(&f.owner, execution.id).await.unwrap();
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(
        evidence.join("paused-episode-driver.json"),
        serde_json::to_vec(&serde_json::json!({"progress":progress.as_ref().map_err(|e|&e.code)}))
            .unwrap(),
    )
    .unwrap();
    f.finish().await;
    let progress=progress.expect("temporarily fenced Prepared episode must remain workflow progress, not nonretryable Activity failure");
    assert_eq!(
        progress.journals,
        vec![(operation, gateway::OperationState::Prepared)]
    );
    assert_eq!(
        progress.limitation.as_deref(),
        Some("episode_dispatch_fenced")
    );
}

#[tokio::test]
async fn colliding_episode_and_admission_waits_drive_both_protocols() {
    use temporal::OperationDriver;
    let f = Fixture::new().await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let input = Uuid::new_v4();
    let episode = f.admission;
    mailbox.execute("assembled-worker",mail(f.epoch,"collision-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"implementation","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-worker", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute("assembled-worker",mail(f.epoch,"collision-admit",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"implementation","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let root = f.root.join("collision-native");
    std::fs::create_dir(&root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "unregistered-collision-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "actual-child-process".into(),
        environment: "model-free".into(),
    };
    let episode_gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    let episode_operation = Uuid::new_v4();
    episode_gateway
        .prepare(
            "assembled-worker",
            episode_runtime::StartRequest {
                id: episode_operation,
                episode,
                execution: execution.id,
                runtime: identity.clone(),
                argv: vec!["/bin/true".into()],
            },
        )
        .await
        .unwrap();
    let legacy_root = f.root.join("collision-legacy");
    std::fs::create_dir(&legacy_root).unwrap();
    let mut legacy_identity = identity.clone();
    legacy_identity.thread_store = legacy_root.to_str().unwrap().into();
    let legacy = gateway::Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    let legacy_operation = Uuid::new_v4();
    legacy
        .bind(&f.owner, binding, f.admission, &legacy_identity, None, 1)
        .await
        .unwrap();
    legacy
        .prepare(
            &f.owner,
            gateway::Intent {
                id: legacy_operation,
                admission_id: f.admission,
                binding_id: binding,
                generation: 1,
                kind: gateway::OperationKind::Create,
                arguments: serde_json::json!({}),
            },
        )
        .await
        .unwrap();
    let bridge = temporal::Bridge::new(f.broker.clone());
    let reference = bridge
        .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let fallback = std::sync::Arc::new(temporal::GatewayOperationDriver::new(
        bridge.clone(),
        legacy.clone(),
        f.owner.clone(),
        Box::new(gateway_fault::FaultRuntime::new(
            legacy_root,
            gateway_fault::Fault::None,
            legacy_identity,
        )),
    ));
    let driver = episode_runtime::EpisodeOperationDriver::new(
        episode_gateway,
        f.owner.clone(),
        Box::new(UnregisteredEpisodeEffect(
            gateway_fault::FaultRuntime::new(root, gateway_fault::Fault::None, identity),
            None,
        )),
        Some(fallback),
    );
    let progress = bridge.progress(&f.owner, &reference).await.unwrap();
    let mut observed = vec![];
    for wait in &progress.waits {
        observed.push(driver.drive(&reference, *wait).await.unwrap());
    }
    let legacy_state = legacy.read(&f.owner, legacy_operation).await.unwrap().state;
    let evidence = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(evidence.join("colliding-waits.json"),serde_json::to_vec(&serde_json::json!({"progress":progress,"observed":observed,"legacy_state":legacy_state})).unwrap()).unwrap();
    drop(driver);
    supervisor.cancel(&f.owner, execution.id).await.unwrap();
    f.finish().await;
    assert_eq!(
        legacy_state,
        gateway::OperationState::Confirmed,
        "same UUID must retain and dispatch both typed protocols"
    );
}

#[tokio::test]
async fn foreign_task_episode_uuid_cannot_hijack_legacy_admission_wait() {
    use temporal::OperationDriver;
    let f = Fixture::new().await;
    let foreign_task = Uuid::new_v4();
    let worker = Uuid::new_v4();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "foreign-task",
                0,
                Command::CreateTask {
                    id: foreign_task,
                    mission_id: f.mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "foreign scoped task".into(),
                        input: GitInput {
                            repository: f.root.join("store").to_str().unwrap().into(),
                            commit: f.base.clone(),
                            path: "SPEC.md".into(),
                            digest: artifacts::sha256(b"produce exact src/result"),
                        },
                        acceptance: vec!["retain scope".into()],
                        deliverables: vec!["src/result".into()],
                        write_scope: vec!["src".into()],
                    },
                },
            ),
        )
        .await
        .unwrap();
    let spec = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(foreign_task))
        .await
        .unwrap()
        .data["spec_digest"]
        .as_str()
        .unwrap()
        .to_owned();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "foreign-admit",
                1,
                Command::AdmitTask {
                    task_id: foreign_task,
                    spec_digest: spec,
                },
            ),
        )
        .await
        .unwrap();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "foreign-grant",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(f.mission),
                    task_id: Some(foreign_task),
                    generation: Some(1),
                    token_hash: token_hash("foreign-worker"),
                    expires_in_seconds: 30,
                },
            ),
        )
        .await
        .unwrap();
    let assigned = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "foreign-assign",
                2,
                Command::AssignAttempt {
                    task_id: foreign_task,
                    worker_id: worker,
                },
            ),
        )
        .await
        .unwrap();
    let attempt = assigned.attempt_id.unwrap();
    kernel::Kernel::new(f.broker.clone())
        .require_supervised_verification(&f.owner, foreign_task)
        .await
        .unwrap();
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let input = Uuid::new_v4();
    let episode = f.admission;
    mailbox.execute("foreign-worker",mail(f.epoch,"foreign-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":attempt,"purpose":"implementation","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("foreign-worker", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute("foreign-worker",mail(f.epoch,"foreign-episode",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":attempt,"purpose":"implementation","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "foreign-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let root = f.root.join("foreign-episode-native");
    std::fs::create_dir(&root).unwrap();
    let legacy_root = f.root.join("foreign-collision-legacy");
    std::fs::create_dir(&legacy_root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "unregistered-collision-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "actual-child-process".into(),
        environment: "model-free".into(),
    };
    let episode_gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    episode_gateway
        .prepare(
            "foreign-worker",
            episode_runtime::StartRequest {
                id: Uuid::new_v4(),
                episode,
                execution: execution.id,
                runtime: identity.clone(),
                argv: vec!["/bin/true".into()],
            },
        )
        .await
        .unwrap();
    let mut legacy_identity = identity.clone();
    legacy_identity.thread_store = legacy_root.to_str().unwrap().into();
    let legacy = gateway::Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    let operation = Uuid::new_v4();
    legacy
        .bind(&f.owner, binding, f.admission, &legacy_identity, None, 1)
        .await
        .unwrap();
    legacy
        .prepare(
            &f.owner,
            gateway::Intent {
                id: operation,
                admission_id: f.admission,
                binding_id: binding,
                generation: 1,
                kind: gateway::OperationKind::Create,
                arguments: serde_json::json!({}),
            },
        )
        .await
        .unwrap();
    let bridge = temporal::Bridge::new(f.broker.clone());
    let reference = bridge
        .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let fallback = std::sync::Arc::new(temporal::GatewayOperationDriver::new(
        bridge.clone(),
        legacy.clone(),
        f.owner.clone(),
        Box::new(gateway_fault::FaultRuntime::new(
            legacy_root,
            gateway_fault::Fault::None,
            legacy_identity,
        )),
    ));
    let driver = episode_runtime::EpisodeOperationDriver::new(
        episode_gateway,
        f.owner.clone(),
        Box::new(UnregisteredEpisodeEffect(
            gateway_fault::FaultRuntime::new(root, gateway_fault::Fault::None, identity),
            None,
        )),
        Some(fallback),
    );
    let progress = driver
        .drive(&reference, temporal::WaitRef::admission(f.admission))
        .await
        .unwrap();
    assert_eq!(progress.admission_id, Some(f.admission));
    assert_eq!(
        progress.journals,
        vec![(operation, gateway::OperationState::Confirmed)]
    );
    assert_eq!(
        driver
            .drive(&reference, temporal::WaitRef::episode(episode))
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    assert_eq!(
        supervisor
            .inspect(&f.owner, execution.id)
            .await
            .unwrap()
            .state,
        supervisor::ExecutionState::Ready
    );
    supervisor.cancel(&f.owner, execution.id).await.unwrap();
    supervisor.release(&f.owner, execution.id).await.unwrap();
    drop(driver);
    f.finish().await;
}
