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

async fn owned_nested_cancel_case() {
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
            runtime_kind: "kernel16-inprocess-native".into(),
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
            let mut runtime = UnmeasuredCancellationNative(episode_native::NativeOsRuntime {
                root: native_root.clone(),
                identity,
                supervisor: supervisor.clone(),
                service: f.owner.clone(),
                lose_reply: mode == "unknown-effect",
                hold_reply: false,
            });
            gateway.register_runtime(&f.owner, &runtime).await.unwrap();
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
        let token = "assembled-worker";
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
        if mode == "unknown-effect" {
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
        let worker_inspect = supervisor.inspect("assembled-worker", execution.id).await;
        let worker_logs = supervisor.logs("assembled-worker", execution.id).await;
        let worker_result = supervisor.result("assembled-worker", execution.id).await;
        let worker_history = supervisor.history("assembled-worker", execution.id).await;
        let worker_cancel = supervisor.cancel("assembled-worker", execution.id).await;
        let worker_reconcile = supervisor.reconcile("assembled-worker", execution.id).await;
        let worker_release = supervisor.release("assembled-worker", execution.id).await;
        if mode == "known-physical" {
            if worker_release.is_err() {
                // Preserve physical fixture cleanup even for the intended red.
                supervisor.release(&f.owner, execution.id).await.unwrap();
            }
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
        std::fs::write(evidence.join(format!("worker-cleanup-{mode}.json")), serde_json::to_vec(&serde_json::json!({"inspect":worker_inspect.as_ref().map_err(|e|&e.code),"logs":worker_logs.as_ref().map(|v|v.len()).map_err(|e|&e.code),"result":worker_result.as_ref().map_err(|e|&e.code),"history":worker_history.as_ref().map(|v|v.len()).map_err(|e|&e.code),"cancel":worker_cancel.as_ref().map_err(|e|&e.code),"reconcile":worker_reconcile.as_ref().map_err(|e|&e.code),"release":worker_release.as_ref().map_err(|e|&e.code)})).unwrap()).unwrap();
        server.abort();
        let root = f.root.clone();
        f.finish().await;
        std::fs::write(evidence.join(format!("cleanup-{mode}.json")),serde_json::to_vec(&serde_json::json!({"fixture_removed":!root.exists(),"physical_stop_proven":true,"http_server_aborted":true})).unwrap()).unwrap();
        assert!(
            worker_inspect.is_ok(),
            "cancelled current Worker retains inspection: {worker_inspect:?}"
        );
        assert!(
            worker_logs.is_ok(),
            "cancelled current Worker retains raw logs"
        );
        assert!(
            worker_result.is_ok(),
            "cancelled current Worker retains result lookup"
        );
        assert!(
            worker_history.is_ok(),
            "cancelled current Worker retains history"
        );
        assert!(
            worker_cancel.is_ok(),
            "cancelled current Worker retains bounded Stop"
        );
        assert!(
            worker_reconcile.is_ok(),
            "cancelled current Worker retains reconciliation"
        );
        if mode == "known-physical" {
            worker_release.unwrap();
        } else {
            assert_eq!(
                worker_release.unwrap_err().code,
                ErrorCode::DependencyUnmet,
                "unknown usage/effects still hold capacity for Worker"
            );
        }
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

#[tokio::test]
async fn cancelled_unallocated_attempt_refunds_only_concurrent_capacity_once() {
    let f = Fixture::new().await;
    let before = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "unbound-cancel",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "no allocation or native dispatch exists".into(),
                },
            ),
        )
        .await
        .unwrap();
    let done = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "unbound-reconcile",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    assert_eq!(done.disposition, "cancelled");
    let again = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "unbound-reconcile-again",
                done.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let after = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(
        out.join("unbound-cancellation-refund.json"),
        serde_json::to_vec(
            &serde_json::json!({"before":before,"after":after,"first":done,"repeat":again}),
        )
        .unwrap(),
    )
    .unwrap();
    f.finish().await;
    assert_eq!(
        after.data["used_capacity"],
        serde_json::json!(0),
        "known-unused cancelled admission must release its concurrent slot"
    );
    assert_eq!(
        before.data["admitted_attempts"],
        after.data["admitted_attempts"]
    );
    assert_eq!(
        before.data["admitted_episodes"],
        after.data["admitted_episodes"]
    );
}
#[tokio::test]
async fn cancelled_prepared_episode_retains_current_workflow_progress() {
    use temporal::OperationDriver;
    let f = Fixture::new().await;
    let episode = f.worker_episode("cancelled-episode").await;
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
    let root = f.root.join("cancelled-episode-native");
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
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "cancel-prepared-native",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "execution revoked, workflow remains current".into(),
                },
            ),
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
        evidence.join("cancelled-episode-driver.json"),
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
async fn ready_unknown_episode_cannot_authorize_an_unrelated_physical_launch() {
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
    let native_pid = runtime.1.as_ref().unwrap().id();
    assert!(
        runtime.1.as_mut().unwrap().try_wait().unwrap().is_none(),
        "independent native effect is actually live"
    );
    let launched = supervisor
        .start(
            "assembled-worker",
            execution.id,
            vec!["/bin/true".into()],
            30,
        )
        .await;
    if launched.is_ok() {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        while supervisor
            .result(&f.owner, execution.id)
            .await
            .unwrap()
            .is_none()
        {
            assert!(std::time::Instant::now() < deadline);
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join("ready-own-unknown.json"),serde_json::to_vec(&serde_json::json!({"journal":journal,"launch":launched.as_ref().map_err(|e|&e.code),"workspace":supervisor.inspect(&f.owner,execution.id).await.unwrap(),"result":supervisor.result(&f.owner,execution.id).await.unwrap()})).unwrap()).unwrap();
    let mut child = runtime.1.take().unwrap();
    child.kill().unwrap();
    let terminal = child.wait().unwrap();
    std::fs::write(
        out.join("ready-own-native-child-terminal.json"),
        serde_json::to_vec(
            &serde_json::json!({"pid":native_pid,"waited":true,"terminal":terminal.to_string()}),
        )
        .unwrap(),
    )
    .unwrap();
    drop(runtime);
    f.finish().await;
    assert_eq!(
        launched.unwrap_err().code,
        ErrorCode::DependencyUnmet,
        "Ready own unknown operation without launch certificate cannot grant new physical work"
    );
}

async fn ready_capacity_task(f: &Fixture, key: &str) -> Uuid {
    let task = Uuid::new_v4();
    let worker = Uuid::new_v4();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                &format!("{key}-create"),
                0,
                Command::CreateTask {
                    id: task,
                    mission_id: f.mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "bounded replacement admission".into(),
                        input: GitInput {
                            repository: f.root.join("store").to_str().unwrap().into(),
                            commit: f.base.clone(),
                            path: "SPEC.md".into(),
                            digest: artifacts::sha256(b"produce exact src/result"),
                        },
                        acceptance: vec!["unchanged cumulative budgets".into()],
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
        .snapshot(&f.owner, SnapshotQuery::Task(task))
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
                &format!("{key}-admit"),
                1,
                Command::AdmitTask {
                    task_id: task,
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
                &format!("{key}-credential"),
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(f.mission),
                    task_id: Some(task),
                    generation: Some(1),
                    token_hash: token_hash(&format!("{key}-worker")),
                    expires_in_seconds: 300,
                },
            ),
        )
        .await
        .unwrap();
    task
}
async fn assign_capacity_task(f: &Fixture, task: Uuid, key: &str) -> Result<CommandResult> {
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let worker: Uuid = db
        .query_one(
            "SELECT principal_id FROM credentials WHERE task_id=$1 AND role='worker'",
            &[&task],
        )
        .await
        .unwrap()
        .get(0);
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                key,
                2,
                Command::AssignAttempt {
                    task_id: task,
                    worker_id: worker,
                },
            ),
        )
        .await
}
#[tokio::test]
async fn unbound_refund_allows_capacity_one_replacement_without_budget_refund() {
    let f = Fixture::with_limits(1, 2, 2).await;
    let second = ready_capacity_task(&f, "capacity-second").await;
    let third = ready_capacity_task(&f, "capacity-third").await;
    assert_eq!(
        assign_capacity_task(&f, second, "blocked-capacity")
            .await
            .unwrap_err()
            .code,
        ErrorCode::CapacityExhausted
    );
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "capacity-cancel",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "known-unused capacity one slot".into(),
                },
            ),
        )
        .await
        .unwrap();
    let reconcile = envelope(
        f.epoch,
        "capacity-release",
        cancel.revision,
        Command::ReconcileAttempt {
            task_id: f.task,
            attempt_id: f.attempt,
        },
    );
    f.broker.execute(&f.owner, reconcile.clone()).await.unwrap();
    f.broker.execute(&f.owner, reconcile).await.unwrap();
    let assigned = assign_capacity_task(&f, second, "replacement-admission")
        .await
        .unwrap();
    let cancelled = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "replacement-cancel",
                assigned.revision,
                Command::CancelAttempt {
                    task_id: second,
                    attempt_id: assigned.attempt_id.unwrap(),
                    reason: "known unused replacement".into(),
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
                "replacement-release",
                cancelled.revision,
                Command::ReconcileAttempt {
                    task_id: second,
                    attempt_id: assigned.attempt_id.unwrap(),
                },
            ),
        )
        .await
        .unwrap();
    let final_state = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let denied = assign_capacity_task(&f, third, "cumulative-budget-no-refund").await;
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join("capacity-one-replacement.json"),serde_json::to_vec(&serde_json::json!({"mission":final_state,"replacement":assigned,"third":denied.as_ref().map_err(|e|&e.code)})).unwrap()).unwrap();
    f.finish().await;
    assert_eq!(final_state.data["used_capacity"], 0);
    assert_eq!(final_state.data["admitted_attempts"], 2);
    assert_eq!(final_state.data["admitted_episodes"], 2);
    assert_eq!(denied.unwrap_err().code, ErrorCode::BudgetExhausted);
}

struct UnmeasuredCancellationNative(episode_native::NativeOsRuntime);
impl episode_runtime::EpisodeRuntime for UnmeasuredCancellationNative {
    fn identity(&self) -> gateway::RuntimeIdentity {
        episode_runtime::EpisodeRuntime::identity(&self.0)
    }
    fn capabilities(&self) -> gateway::CapabilityProfile {
        episode_runtime::EpisodeRuntime::capabilities(&self.0)
    }
    fn closed_world_os(&self) -> Option<episode_runtime::ClosedWorldOs> {
        episode_runtime::EpisodeRuntime::closed_world_os(&self.0)
    }
    fn start<'a>(
        &'a mut self,
        p: &'a episode_runtime::PreparedStart,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = episode_runtime::EpisodeReply> + Send + 'a>,
    > {
        Box::pin(async move {
            match episode_runtime::EpisodeRuntime::start(&mut self.0, p).await {
                episode_runtime::EpisodeReply::ConfirmedMeasured(turn, _) => {
                    episode_runtime::EpisodeReply::Confirmed(turn)
                }
                reply => reply,
            }
        })
    }
    fn observe<'a>(
        &'a mut self,
        p: &'a episode_runtime::PreparedStart,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = episode_runtime::EpisodeObservation> + Send + 'a>,
    > {
        episode_runtime::EpisodeRuntime::observe(&mut self.0, p)
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "owned native runtime process; parent controls lifetime"]
async fn owned_nested_cancel_worker() {
    owned_nested_cancel_case().await;
}
#[test]
fn public_cancel_stops_actual_nested_writers_and_never_refunds_unknown() {
    use std::os::unix::process::CommandExt;
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "owned_nested_cancel_worker",
            "--ignored",
            "--nocapture",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env(
            "TMPDIR",
            std::env::var("TMPDIR").unwrap_or_else(|_| "/run/corenius-tmp".into()),
        )
        .env(
            "TEAM_CONTROL_PLANE_TEST_DSN",
            std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").unwrap(),
        )
        .env(
            "TEMPORAL_FIXTURE_EVIDENCE",
            std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap(),
        )
        .process_group(0)
        .output()
        .unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join("nested-worker-stdout.log"), &output.stdout).unwrap();
    std::fs::write(out.join("nested-worker-stderr.log"), &output.stderr).unwrap();
    std::fs::write(
        out.join("nested-worker-terminal.json"),
        serde_json::to_vec(
            &serde_json::json!({"waited":true,"terminal":output.status.to_string()}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "owned nested cancellation failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
#[tokio::test]
async fn task_free_lead_unknown_episode_cannot_authorize_new_physical_work() {
    let f = Fixture::new().await;
    let episode = f.lead_episode("unregistered-episode-effect").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-lead",
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
            "assembled-lead",
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
    let native_pid = runtime.1.as_ref().unwrap().id();
    assert!(
        runtime.1.as_mut().unwrap().try_wait().unwrap().is_none(),
        "independent native effect is actually live"
    );
    let launched = supervisor
        .start("assembled-lead", execution.id, vec!["/bin/true".into()], 30)
        .await;
    if launched.is_ok() {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        while supervisor
            .result(&f.owner, execution.id)
            .await
            .unwrap()
            .is_none()
        {
            assert!(std::time::Instant::now() < deadline);
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join("ready-lead-own-unknown.json"),serde_json::to_vec(&serde_json::json!({"journal":journal,"launch":launched.as_ref().map_err(|e|&e.code),"workspace":supervisor.inspect(&f.owner,execution.id).await.unwrap(),"result":supervisor.result(&f.owner,execution.id).await.unwrap()})).unwrap()).unwrap();
    let mut child = runtime.1.take().unwrap();
    child.kill().unwrap();
    let terminal = child.wait().unwrap();
    std::fs::write(
        out.join("ready-lead-native-child-terminal.json"),
        serde_json::to_vec(
            &serde_json::json!({"pid":native_pid,"waited":true,"terminal":terminal.to_string()}),
        )
        .unwrap(),
    )
    .unwrap();
    drop(runtime);
    f.finish().await;
    assert_eq!(
        launched.unwrap_err().code,
        ErrorCode::DependencyUnmet,
        "Ready own unknown operation without launch certificate cannot grant new physical work"
    );
}

#[tokio::test]
async fn review571_cancelled_unallocated_worker_continuation_releases_separate_capacity_once() {
    review571_continuation_cancel(false, false).await;
}
#[tokio::test]
async fn review571_old_public_yield_then_cancel_must_not_refund_twice() {
    review571_continuation_cancel(true, false).await;
}
#[tokio::test]
#[ignore = "requires exact pre-repair baseline server binary"]
async fn review571_persisted_old_public_yield_claim_survives_repaired_cancel() {
    review571_continuation_cancel(true, true).await;
}
struct Review571Server(std::process::Child);
impl Drop for Review571Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
async fn review571_continuation_cancel(prior_yield: bool, baseline_http: bool) {
    let f = Fixture::with_limits(2, 2, 3).await;
    temporal::Bridge::new(f.broker.clone())
        .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let first = f.worker_episode("review571-first").await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    mailbox.execute("assembled-worker", mail(f.epoch, "review571-yield", serde_json::json!({"kind":"yield_episode","episode":first,"checkpoint":{"id":Uuid::new_v4(),"summary":"no execution allocated","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    mailbox.execute("assembled-worker", mail(f.epoch, "review571-notice", serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":"Resume approved work","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let continuation = f.worker_episode("review571-continuation").await;
    if prior_yield {
        let command = mail(
            f.epoch,
            "review571-continuation-yield",
            serde_json::json!({"kind":"yield_episode","episode":continuation,"checkpoint":{"id":Uuid::new_v4(),"summary":"old public atomic unused refund","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}),
        );
        if baseline_http {
            let binary =
                std::env::var("REVIEW571_BASELINE_BINARY").expect("exact baseline binary required");
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            drop(listener);
            let server = Review571Server(
                std::process::Command::new(&binary)
                    .args(["serve", &address.to_string()])
                    .env_clear()
                    .env("PATH", "/usr/bin:/bin")
                    .env("TEAM_CONTROL_PLANE_DSN", &f.dsn)
                    .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", f.epoch.to_string())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .unwrap(),
            );
            let client = reqwest::Client::new();
            let mut ready = false;
            for _ in 0..100 {
                if client
                    .get(format!("http://{address}/v1/kernel/status"))
                    .bearer_auth(&f.owner)
                    .send()
                    .await
                    .is_ok()
                {
                    ready = true;
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
            assert!(ready, "baseline public server must become ready");
            let response = client
                .post(format!("http://{address}/v1/mailbox/commands"))
                .bearer_auth("assembled-worker")
                .json(&command)
                .send()
                .await
                .unwrap();
            let status = response.status();
            let bytes = response.bytes().await.unwrap();
            drop(server);
            let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
            std::fs::write(out.join("old-binary-yield-response.json"), bytes).unwrap();
            std::fs::write(
                out.join("old-binary-sha256.txt"),
                artifacts::sha256(&std::fs::read(binary).unwrap()),
            )
            .unwrap();
            assert!(
                status.is_success(),
                "exact baseline public Yield must succeed"
            );
        } else {
            mailbox.execute("assembled-worker", command).await.unwrap();
        }
    }
    let prior_record = mailbox
        .record(&f.owner, mailbox::RecordKind::Episode, continuation)
        .await
        .unwrap();
    let before = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-cancel",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "cancel unused continuation".into(),
                },
            ),
        )
        .await
        .unwrap();
    let done = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-reconcile",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let again = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-reconcile-again",
                done.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let after = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let record = mailbox
        .record(&f.owner, mailbox::RecordKind::Episode, continuation)
        .await
        .unwrap();
    let replacement = ready_capacity_task(&f, "review571-replacement").await;
    let reused = assign_capacity_task(&f, replacement, "review571-reuse-released-capacity").await;
    let third = ready_capacity_task(&f, "review571-budget-third").await;
    let exhausted =
        assign_capacity_task(&f, third, "review571-cumulative-budget-stays-charged").await;
    let final_budget = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join(if baseline_http {"review571-persisted-old-yield.json"} else if prior_yield {"review571-old-yield.json"} else {"review571-unallocated-continuation.json"}),serde_json::to_vec(&serde_json::json!({"old_binary":baseline_http,"prior_episode":prior_record,"before":before,"after":after,"episode":record,"cancel":cancel,"first":done,"repeat":again,"replacement":reused.as_ref().map_err(|e|&e.code),"budget_exhausted":exhausted.as_ref().map_err(|e|&e.code),"final_budget":final_budget})).unwrap()).unwrap();
    f.finish().await;
    if baseline_http {
        assert_eq!(
            prior_record["owned_capacity"], true,
            "baseline old Yield really leaves already refunded claim set"
        );
        assert_eq!(
            record["owned_capacity"], false,
            "repair clears prior refunded claim"
        );
    }
    assert!(
        reused.is_ok(),
        "released episode capacity permits replacement admission"
    );
    assert_eq!(exhausted.unwrap_err().code, ErrorCode::BudgetExhausted);
    assert_eq!(final_budget.data["admitted_attempts"], 2);
    assert_eq!(final_budget.data["admitted_episodes"], 3);
    assert_eq!(
        after.data["used_capacity"], 0,
        "cancelled unused continuation must release its separately admitted slot"
    );
    assert_eq!(
        before.data["admitted_attempts"],
        after.data["admitted_attempts"]
    );
    assert_eq!(
        before.data["admitted_episodes"],
        after.data["admitted_episodes"]
    );
}

#[tokio::test]
async fn review571_cancelled_unallocated_verifier_releases_capacity_once() {
    review571_verifier_cancel(false).await;
}
#[tokio::test]
async fn review571_uncertain_allocated_sibling_retains_capacity() {
    review571_verifier_cancel(true).await;
}
async fn review571_verifier_cancel(uncertain_sibling: bool) {
    let f = Fixture::new().await;
    let pending = if uncertain_sibling {
        let worker = f.worker_episode("review571-uncertain-worker").await;
        let supervisor = f.supervisor().await;
        let execution = supervisor
            .create_episode(
                "assembled-worker",
                worker,
                vec![],
                supervisor::CreateFault::None,
            )
            .await
            .unwrap();
        supervisor
            .start_with_fault(
                "assembled-worker",
                execution.id,
                vec!["/bin/true".into()],
                30,
                supervisor::StartFault::BeforeMonitor,
            )
            .await
            .unwrap();
        Some((supervisor, execution))
    } else {
        None
    };
    let service = artifacts::ArtifactService::open(f.broker.clone(), &f.root.join("artifacts"))
        .await
        .unwrap();
    let env = |key: &str, revision, command| artifacts::ArtifactEnvelope {
        key: key.into(),
        task: f.task,
        expected_revision: revision,
        generation: 1,
        recovery_epoch: f.epoch,
        command,
    };
    let configured = service
        .execute(
            &f.owner,
            env(
                "review571-configure",
                3,
                artifacts::ArtifactCommand::Configure {
                    verifier: f.verifier,
                    lead_acceptance: false,
                    base: f.base.clone(),
                    suite: artifacts::sha256(b"suite"),
                    environment: artifacts::sha256(b"environment"),
                    mandatory_commands: vec![vec!["/bin/true".into()]],
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
    let store = f.root.join("store");
    std::fs::write(store.join("src/result"), b"immutable candidate").unwrap();
    git(&store, &["add", "src/result"]);
    git(
        &store,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "candidate",
        ],
    );
    let bundle = f.root.join("candidate.bundle");
    git(
        &store,
        &["bundle", "create", bundle.to_str().unwrap(), "HEAD"],
    );
    let bytes = std::fs::read(bundle).unwrap();
    let digest = artifacts::sha256(&bytes);
    service
        .publish_evidence("assembled-worker", f.task, &digest, &bytes)
        .await
        .unwrap();
    let submitted = service
        .execute(
            "assembled-worker",
            env(
                "review571-submit",
                configured.revision,
                artifacts::ArtifactCommand::Submit {
                    candidate: artifacts::Candidate {
                        commit: git(&store, &["rev-parse", "HEAD"]),
                        tree: git(&store, &["rev-parse", "HEAD^{tree}"]),
                        base: f.base.clone(),
                        spec_digest: f.spec.clone(),
                        spec_revision: 2,
                        generation: 1,
                        artifact: digest,
                        deliverables: vec!["src/result".into()],
                        risks: vec![],
                    },
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    mailbox.execute("assembled-verifier",mail(f.epoch,"review571-verifier-input",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"verification","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-verifier", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute("assembled-verifier",mail(f.epoch,"review571-verifier-admit",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
    let before = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-verifier-cancel",
                submitted.revision,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "unused verifier".into(),
                },
            ),
        )
        .await
        .unwrap();
    let first = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-verifier-reconcile",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let repeat = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-verifier-repeat",
                first.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let after = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(
        out.join(if uncertain_sibling {
            "review571-uncertain-sibling.json"
        } else {
            "review571-unallocated-verifier.json"
        }),
        serde_json::to_vec(
            &serde_json::json!({"before":before,"after":after,"first":first,"repeat":repeat}),
        )
        .unwrap(),
    )
    .unwrap();
    let allocated = if let Some((supervisor, execution)) = pending {
        Some(supervisor.inspect(&f.owner, execution.id).await.unwrap())
    } else {
        None
    };
    std::fs::write(
        out.join(if uncertain_sibling {
            "review571-uncertain-allocation.json"
        } else {
            "review571-no-allocation.json"
        }),
        serde_json::to_vec(&allocated).unwrap(),
    )
    .unwrap();
    f.finish().await;
    if uncertain_sibling {
        assert_eq!(first.disposition, "quarantined");
        assert_eq!(
            after.data["used_capacity"], before.data["used_capacity"],
            "uncertain allocated siblings cannot refund capacity"
        );
    } else {
        assert_eq!(after.data["used_capacity"], 0);
    }
    assert_eq!(
        before.data["admitted_attempts"],
        after.data["admitted_attempts"]
    );
    assert_eq!(
        before.data["admitted_episodes"],
        after.data["admitted_episodes"]
    );
}

struct Review571NativeCancel {
    runtime: gateway_fault::FaultRuntime,
    child: Option<std::process::Child>,
    cancels: usize,
}
impl Drop for Review571NativeCancel {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
impl gateway::RuntimeAdapter for Review571NativeCancel {
    fn identity(&self) -> gateway::RuntimeIdentity {
        self.runtime.identity.clone()
    }
    fn inspect(&self) -> gateway::CapabilityProfile {
        gateway::RuntimeAdapter::inspect(&self.runtime)
    }
    fn execute(&mut self, intent: &gateway::Intent, thread: Option<&str>) -> gateway::RuntimeReply {
        match intent.kind {
            gateway::OperationKind::Start => {
                self.child = Some(
                    std::process::Command::new("/bin/sleep")
                        .arg("30")
                        .stdout(std::process::Stdio::null())
                        .spawn()
                        .unwrap(),
                );
            }
            gateway::OperationKind::Cancel => {
                self.cancels += 1;
                let child = self.child.as_mut().expect("Cancel targets bound workload");
                child.kill().unwrap();
                child.wait().unwrap();
            }
            _ => {}
        }
        gateway::RuntimeAdapter::execute(&mut self.runtime, intent, thread)
    }
    fn observe(&mut self, intent: &gateway::Intent, thread: Option<&str>) -> gateway::Observation {
        gateway::RuntimeAdapter::observe(&mut self.runtime, intent, thread)
    }
}
#[tokio::test]
async fn review571_native_cancel_cleans_bound_work_after_work_revocation() {
    for prepare_before in [true, false] {
        let f = Fixture::new().await;
        let root = f.root.join("review571-native");
        std::fs::create_dir(&root).unwrap();
        let identity = gateway::RuntimeIdentity {
            runtime_kind: "review571-native-owned-os".into(),
            runtime_version: "1".into(),
            deployment: f.epoch.to_string(),
            thread_store: root.to_str().unwrap().into(),
            endpoint: "owned-process".into(),
            environment: "fixture".into(),
        };
        let gateway = gateway::Gateway::new(f.broker.clone());
        let binding = Uuid::new_v4();
        gateway
            .bind(&f.owner, binding, f.admission, &identity, None, 1)
            .await
            .unwrap();
        let mut runtime = Review571NativeCancel {
            runtime: gateway_fault::FaultRuntime::new(root, gateway_fault::Fault::None, identity),
            child: None,
            cancels: 0,
        };
        let intent = |kind| gateway::Intent {
            id: Uuid::new_v4(),
            admission_id: f.admission,
            binding_id: binding,
            generation: 1,
            kind,
            arguments: serde_json::json!({"exact_binding":binding}),
        };
        for kind in [
            gateway::OperationKind::Create,
            gateway::OperationKind::Start,
        ] {
            let op = intent(kind);
            gateway.prepare(&f.owner, op.clone()).await.unwrap();
            gateway
                .dispatch(&f.owner, op.id, &mut runtime)
                .await
                .unwrap();
        }
        let cleanup = intent(gateway::OperationKind::Cancel);
        if prepare_before {
            gateway.prepare(&f.owner, cleanup.clone()).await.unwrap();
        }
        let revoked = f
            .broker
            .execute(
                &f.owner,
                envelope(
                    f.epoch,
                    "review571-native-revoke",
                    3,
                    Command::CancelAttempt {
                        task_id: f.task,
                        attempt_id: f.attempt,
                        reason: "stop bound workload".into(),
                    },
                ),
            )
            .await
            .unwrap();
        let mut controls = Vec::new();
        for kind in [
            gateway::OperationKind::Create,
            gateway::OperationKind::Start,
            gateway::OperationKind::Steer,
        ] {
            let denied = gateway
                .prepare(&f.owner, intent(kind))
                .await
                .unwrap_err()
                .code;
            controls.push(serde_json::json!({"kind":kind,"denied":denied}));
            assert_eq!(
                denied,
                ErrorCode::DependencyUnmet,
                "revocation must still fence work operations"
            );
        }
        for token in ["wrong-token", "assembled-verifier"] {
            assert!(gateway.prepare(token, cleanup.clone()).await.is_err());
        }
        let mut stale = intent(gateway::OperationKind::Cancel);
        stale.generation = 2;
        assert!(gateway.prepare(&f.owner, stale).await.is_err());
        let foreign_intent = intent(gateway::OperationKind::Cancel);
        gateway
            .prepare(&f.owner, foreign_intent.clone())
            .await
            .unwrap();
        let foreign = gateway::Gateway::new(f.broker.clone());
        assert_eq!(
            foreign
                .dispatch(&f.owner, foreign_intent.id, &mut runtime)
                .await
                .unwrap()
                .state,
            gateway::OperationState::Quarantined,
            "foreign controller cannot dispatch exact cleanup"
        );
        let prepared = if prepare_before {
            Ok(())
        } else {
            gateway.prepare(&f.owner, cleanup.clone()).await.map(|_| ())
        };
        let result = match prepared {
            Ok(()) => gateway
                .dispatch(&f.owner, cleanup.id, &mut runtime)
                .await
                .map(Some),
            Err(error) => Err(error),
        };
        let replay = if result.is_ok() {
            gateway
                .dispatch(&f.owner, cleanup.id, &mut runtime)
                .await
                .map(Some)
        } else {
            Ok(None)
        };
        if result.is_ok() {
            assert_eq!(
                gateway
                    .reconcile_usage(&f.owner, cleanup.id)
                    .await
                    .unwrap()
                    .usage,
                episode_runtime::UsageObservation::Unknown,
                "Cancel acknowledgment cannot certify zero usage"
            );
        }
        let invocations = runtime.runtime.invocation_count().unwrap();
        let cancels = runtime.cancels;
        let child_live = runtime
            .child
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none();
        let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
        std::fs::write(out.join(format!("review571-native-cancel-{prepare_before}.json")),serde_json::to_vec(&serde_json::json!({"controls":controls,"revoked":revoked,"result":result.as_ref().map_err(|e|&e.code),"replay":replay.as_ref().map_err(|e|&e.code),"invocations":invocations,"cancel_invocations":cancels,"child_live":child_live})).unwrap()).unwrap();
        drop(runtime);
        f.finish().await;
        assert!(
            result.is_ok(),
            "authenticated exact Cancel must clean existing workload after revocation: {result:?}"
        );
        assert_eq!(cancels, 1);
        assert!(!child_live);
        assert_eq!(invocations, 3);
    }
}

#[tokio::test]
async fn review571_unresolved_yield_execution_requires_explicit_quarantine() {
    review571_unresolved_yield(false).await;
}
#[tokio::test]
async fn review571_imported_worker_unresolved_yield_requires_quarantine() {
    review571_unresolved_yield(true).await;
}
async fn review571_unresolved_yield(imported: bool) {
    let f = Fixture::new().await;
    temporal::Bridge::new(f.broker.clone())
        .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let first = f.worker_episode("review571-unresolved-first").await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let episode = if imported {
        first
    } else {
        mailbox.execute("assembled-worker",mail(f.epoch,"review571-unresolved-first-yield",serde_json::json!({"kind":"yield_episode","episode":first,"checkpoint":{"id":Uuid::new_v4(),"summary":"first unused episode","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
        mailbox.execute("assembled-worker",mail(f.epoch,"review571-unresolved-notice",serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":"retain independently owned job handle","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
        f.worker_episode("review571-unresolved-continuation").await
    };
    let mut child = Review571Server(
        std::process::Command::new("/bin/sleep")
            .arg("30")
            .spawn()
            .unwrap(),
    );
    let process = child.0.id();
    let handle = Uuid::new_v4();
    mailbox.execute("assembled-worker",mail(f.epoch,"review571-unresolved-yield",serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"retain actual owned OS job, physical binding unresolved","artifacts":[]},"questions":[],"dependencies":[],"executions":[{"id":handle,"supervisor":format!("owned-os-process-{process}"),"attempt":f.attempt,"generation":1,"recovery_epoch":f.epoch,"provenance":"unresolved"}]}))).await.unwrap();
    let episode_record = mailbox
        .record(&f.owner, mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    let before = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-unresolved-cancel",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "cleanup declared unresolved job".into(),
                },
            ),
        )
        .await
        .unwrap();
    let first = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-unresolved-reconcile",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let repeat = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review571-unresolved-repeat",
                first.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let after = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let task = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(f.task))
        .await
        .unwrap();
    let live = child.0.try_wait().unwrap().is_none();
    child.0.kill().unwrap();
    let terminal = child.0.wait().unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join(if imported {"review571-imported-unresolved-yield.json"}else{"review571-unresolved-yield.json"}),serde_json::to_vec(&serde_json::json!({"episode":episode_record,"imported":imported,"before":before,"after":after,"task":task,"first":first,"repeat":repeat,"owned_job":{"handle":handle,"process":process,"live_at_reconciliation":live,"cleanup_terminal":terminal.to_string()}})).unwrap()).unwrap();
    f.finish().await;
    assert!(
        live,
        "no stop observation exists for the declared actual job"
    );
    assert_eq!(
        first.disposition, "quarantined",
        "nonempty unresolved execution checkpoint must expose uncertainty"
    );
    assert_eq!(repeat.disposition, "quarantined");
    assert_eq!(task.data["attempts"][0]["status"], "quarantined");
    assert_eq!(after.data["used_capacity"], before.data["used_capacity"]);
    assert_eq!(
        after.data["admitted_attempts"],
        before.data["admitted_attempts"]
    );
    assert_eq!(
        after.data["admitted_episodes"],
        before.data["admitted_episodes"]
    );
}

#[tokio::test]
async fn review580_allocated_continuation_history_does_not_hold_original_unbound_claim() {
    let f = Fixture::with_limits(2, 2, 3).await;
    temporal::Bridge::new(f.broker.clone())
        .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let first = f.worker_episode("review580-first").await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    mailbox.execute("assembled-worker", mail(f.epoch, "review580-yield", serde_json::json!({"kind":"yield_episode","episode":first,"checkpoint":{"id":Uuid::new_v4(),"summary":"unused imported Worker","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    mailbox.execute("assembled-worker", mail(f.epoch, "review580-notice", serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":"Resume","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let continuation = f.worker_episode("review580-continuation").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-worker",
            continuation,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let before = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data;
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review580-cancel",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "cancel separate allocation".into(),
                },
            ),
        )
        .await
        .unwrap();
    let done = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review580-reconcile",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    supervisor.release(&f.owner, execution.id).await.unwrap();
    let repeated = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review580-repeat",
                done.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let after = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data;
    let replacement = ready_capacity_task(&f, "review580-replacement").await;
    let admitted = assign_capacity_task(&f, replacement, "review580-admit-replacement").await;
    let third = ready_capacity_task(&f, "review580-third").await;
    let exhausted = assign_capacity_task(&f, third, "review580-budget-stays-consumed").await;
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join("review580-capacity.json"),serde_json::to_vec_pretty(&serde_json::json!({"execution":execution,"before":before,"after":after,"cancel":cancel,"done":done,"repeat":repeated,"replacement":admitted.as_ref().map_err(|e|&e.code),"third":exhausted.as_ref().map_err(|e|&e.code)})).unwrap()).unwrap();
    let root = f.root.clone();
    f.finish().await;
    assert!(!root.exists(), "cleanup before capacity oracle");
    assert_eq!(before["used_capacity"], 2);
    assert_eq!(
        after["used_capacity"], 0,
        "retained separately owned execution history must not hold the original unbound reservation"
    );
    assert_eq!(after["admitted_attempts"], 1);
    assert_eq!(after["admitted_episodes"], 2);
    assert!(admitted.is_ok());
    assert_eq!(exhausted.unwrap_err().code, ErrorCode::BudgetExhausted);
}

#[test]
fn review580_unbound_original_and_allocated_continuation_hold_unknown_usage() {
    use std::os::unix::process::CommandExt;
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "review580_owned_continuation_usage_worker",
            "--ignored",
            "--nocapture",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env(
            "TMPDIR",
            std::env::var("TMPDIR").unwrap_or_else(|_| "/run/corenius-tmp".into()),
        )
        .env(
            "TEAM_CONTROL_PLANE_TEST_DSN",
            std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").unwrap(),
        )
        .env(
            "TEMPORAL_FIXTURE_EVIDENCE",
            std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap(),
        )
        .env(
            "TEMPORAL_ADDRESS",
            std::env::var("TEMPORAL_ADDRESS").unwrap(),
        )
        .process_group(0)
        .spawn()
        .unwrap();
    let terminal = child.wait().unwrap();
    assert!(
        terminal.success(),
        "owned continuation usage fixture terminal {terminal}"
    );
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "owned singleton native transport subprocess"]
async fn review580_owned_continuation_usage_worker() {
    use std::time::Duration;
    let f = Fixture::new().await;
    temporal::Bridge::new(f.broker.clone())
        .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let first = f.worker_episode("review580-usage-first").await;
    mailbox.execute("assembled-worker",mail(f.epoch,"review580-usage-yield",serde_json::json!({"kind":"yield_episode","episode":first,"checkpoint":{"id":Uuid::new_v4(),"summary":"unused imported Worker","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    mailbox.execute("assembled-worker",mail(f.epoch,"review580-usage-notice",serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":"resume","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let episode = f.worker_episode("review580-usage-continuation").await;
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
    let root = f.root.join("review580-unknown-native");
    std::fs::create_dir(&root).unwrap();
    let identity = gateway::RuntimeIdentity {
        runtime_kind: "kernel16-inprocess-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "owned-async-os".into(),
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
                argv: vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "sleep 1; printf unknown-usage".into(),
                ],
            },
        )
        .await
        .unwrap();
    let mut runtime = UnmeasuredCancellationNative(episode_native::NativeOsRuntime {
        root: root.clone(),
        identity,
        supervisor: supervisor.clone(),
        service: f.owner.clone(),
        lose_reply: false,
        hold_reply: false,
    });
    gateway.register_runtime(&f.owner, &runtime).await.unwrap();
    let op = gateway
        .dispatch(&f.owner, operation, &mut runtime)
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while supervisor
        .result(&f.owner, execution.id)
        .await
        .unwrap()
        .is_none()
    {
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review580-usage-cancel",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason: "unknown continuation usage".into(),
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
                "review580-usage-reconcile",
                cancel.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let held_release = supervisor.release(&f.owner, execution.id).await;
    let held = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data;
    let resolved = gateway.reconcile(&f.owner, operation).await.unwrap();
    supervisor.release(&f.owner, execution.id).await.unwrap();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "review580-usage-known-reconcile",
                reconciled.revision,
                Command::ReconcileAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                },
            ),
        )
        .await
        .unwrap();
    let after = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data;
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::write(out.join("review580-continuation-usage.json"),serde_json::to_vec_pretty(&serde_json::json!({"operation":op,"held_release":held_release.as_ref().map_err(|e|&e.code),"held":held,"resolved":resolved,"after":after})).unwrap()).unwrap();
    let root = f.root.clone();
    f.finish().await;
    assert!(!root.exists());
    assert_eq!(op.usage, episode_runtime::UsageObservation::Unknown);
    assert_eq!(held_release.unwrap_err().code, ErrorCode::DependencyUnmet);
    assert_eq!(
        held["used_capacity"], 2,
        "unknown continuation usage retains both independent claims"
    );
    assert_eq!(
        after["used_capacity"], 0,
        "real receipt resolves usage without another Start"
    );
}

async fn review611_lead_mission(f: &Fixture, label: &str) -> (Uuid, Uuid, String) {
    let mission = Uuid::new_v4();
    let principal = Uuid::new_v4();
    let token = format!("review611-{label}-{}", Uuid::new_v4());
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                &format!("{label}-mission"),
                0,
                Command::CreateMission {
                    id: mission,
                    objective: "deadline lifecycle public proof".into(),
                    input: GitInput {
                        repository: f.root.join("store").to_str().unwrap().into(),
                        commit: f.base.clone(),
                        path: "SPEC.md".into(),
                        digest: artifacts::sha256(b"produce exact src/result"),
                    },
                    policy: Policy {
                        capacity: 1,
                        max_attempts: 2,
                        max_episodes: 3,
                        max_repairs: 1,
                        episode_deadline_seconds: 2,
                    },
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
                &format!("{label}-grant"),
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: principal,
                    role: Role::Lead,
                    mission_id: Some(mission),
                    task_id: None,
                    generation: None,
                    token_hash: token_hash(&token),
                    expires_in_seconds: 300,
                },
            ),
        )
        .await
        .unwrap();
    (mission, principal, token)
}
async fn review611_admit_lead(
    f: &Fixture,
    mission: Uuid,
    principal: Uuid,
    token: &str,
    key: &str,
) -> Result<Uuid> {
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    mailbox.execute(&f.owner,mail(f.epoch,&format!("{key}-notice"),serde_json::json!({"kind":"send","mission":mission,"id":Uuid::new_v4(),"recipients":[principal],"message_kind":"notice","text":format!("new planning frontier {key}"),"reply_to":null,"attention":"normal","artifacts":[]}))).await?;
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    mailbox.execute(token,mail(f.epoch,&format!("{key}-capture"),serde_json::json!({"kind":"capture_input","mission":mission,"id":input,"attempt":null,"purpose":"planning","limit":4}))).await?;
    let packet = mailbox
        .record(token, mailbox::RecordKind::Input, input)
        .await?;
    mailbox.execute(token,mail(f.epoch,&format!("{key}-admit"),serde_json::json!({"kind":"admit_episode","mission":mission,"id":episode,"attempt":null,"purpose":"planning","input_packet":input,"input_digest":packet["digest"]}))).await?;
    Ok(episode)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review611_deadline_stop_release_allows_same_lead_successor() {
    review611_deadline_release(false, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review611_release_preserves_yielded_checkpoint_wait() {
    review611_deadline_release(true, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review611_release_before_expiry_allows_later_same_lead_successor() {
    review611_deadline_release(false, true).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review611_predeadline_release_preserves_yield_checkpoint_wait() {
    review611_deadline_release(true, true).await;
}
async fn review611_deadline_release(yielded: bool, release_early: bool) {
    use futures::FutureExt;
    use std::time::{Duration, Instant};
    let f = Fixture::new().await;
    let (mission, principal, token) =
        review611_lead_mission(&f, if yielded { "yielded" } else { "deadline" }).await;
    let supervisor = f.supervisor().await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let episode = review611_admit_lead(&f, mission, principal, &token, "review611-first")
        .await
        .unwrap();
    let execution = supervisor
        .create_episode(&token, episode, vec![], supervisor::CreateFault::None)
        .await
        .unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap()).join(
        if release_early && yielded {
            "review611-predeadline-yielded"
        } else if release_early {
            "review611-release-before-expiry"
        } else if yielded {
            "review611-yielded"
        } else {
            "review611-deadline"
        },
    );
    std::fs::create_dir_all(&out).unwrap();
    let outcome=std::panic::AssertUnwindSafe(async {
        supervisor.start(&token,execution.id,if release_early { vec!["/bin/true".into()] } else { vec!["/bin/sleep".into(),"10".into()] },2).await.unwrap();
        let deadline=Instant::now()+Duration::from_secs(8);
        if !release_early {
        loop {
            let view=supervisor.inspect(&token,execution.id).await.unwrap();
            if view.state==supervisor::ExecutionState::Running {break;}
            assert!(Instant::now()<deadline,"real capsule must reach running before canonical deadline");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        }
        let checkpoint=Uuid::new_v4();
        if yielded && !release_early {
            temporal::Bridge::new(f.broker.clone()).register(&f.owner,temporal::AggregateKind::Mission,mission,0).await.unwrap();
            let question=Uuid::new_v4();
            mailbox.execute(&token,mail(f.epoch,"review611-wait-question",serde_json::json!({"kind":"send","mission":mission,"id":question,"recipients":[principal],"message_kind":"question","text":"retain this pending planning question","reply_to":null,"attention":"critical","artifacts":[]}))).await.unwrap();
            mailbox.execute(&token,mail(f.epoch,"review611-yield",serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":checkpoint,"summary":"retain public checkpoint and pending wait","artifacts":[]},"questions":[question],"dependencies":[],"executions":[]}))).await.unwrap();
        }
        let result=loop {
            let view=supervisor.inspect(&token,execution.id).await.unwrap();
            if view.state==supervisor::ExecutionState::Stopped
                && let Some(result)=supervisor.result(&token,execution.id).await.unwrap() {break result;}
            assert!(Instant::now()<deadline,"deadline must independently stop capsule and persist monitor result");
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        assert!(result.namespace_stopped);
        if release_early { assert_eq!(result.exit_code,Some(0)); } else { assert_ne!(result.exit_code,Some(0),"deadline shutdown cannot claim workload success"); }
        if !release_early {
        let expired_yield=mailbox.execute(&token,mail(f.epoch,"review611-expired-yield",serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"expired episodes cannot use a new yield to unblock admission","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await;
        assert_eq!(expired_yield.unwrap_err().code,ErrorCode::DependencyUnmet);
        }
        let before=mailbox.record(&token,mailbox::RecordKind::Episode,episode).await.unwrap();
        supervisor.release(&token,execution.id).await.unwrap();
        let mut after=mailbox.record(&token,mailbox::RecordKind::Episode,episode).await.unwrap();
        if release_early {
            assert_eq!(after["state"],before["state"],"predeadline release must preserve the opportunity to checkpoint");
            if yielded {
                temporal::Bridge::new(f.broker.clone()).register(&f.owner,temporal::AggregateKind::Mission,mission,0).await.unwrap();
                let question=Uuid::new_v4();
                mailbox.execute(&token,mail(f.epoch,"review611-wait-question",serde_json::json!({"kind":"send","mission":mission,"id":question,"recipients":[principal],"message_kind":"question","text":"retain this pending planning question","reply_to":null,"attention":"critical","artifacts":[]}))).await.unwrap();
                mailbox.execute(&token,mail(f.epoch,"review611-yield",serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":checkpoint,"summary":"checkpoint after known release before deadline","artifacts":[]},"questions":[question],"dependencies":[],"executions":[]}))).await.unwrap();
                after=mailbox.record(&token,mailbox::RecordKind::Episode,episode).await.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(2100)).await;
        }
        let mission_view=f.broker.snapshot(&f.owner,SnapshotQuery::Mission(mission)).await.unwrap();
        assert_eq!(mission_view.data["used_capacity"],0);
        assert_eq!(mission_view.data["admitted_episodes"],1);
        let successor=review611_admit_lead(&f,mission,principal,&token,"review611-successor").await;
        std::fs::write(out.join("deadline-release-successor.json"),serde_json::to_vec_pretty(&serde_json::json!({"execution":execution.id,"episode":episode,"monitor_result":result,"before_state":before["state"],"after_state":after["state"],"mission":mission_view,"successor":successor.as_ref().map_err(|e|&e.code)})).unwrap()).unwrap();
        if yielded {
            if !release_early {assert_eq!(before["state"],"yielded");} assert_eq!(after["state"],"yielded");
            assert_eq!(successor.unwrap_err().code,ErrorCode::DependencyUnmet,"known stop must not silently satisfy a pending wait");
            let recorded=mailbox.record(&token,mailbox::RecordKind::Checkpoint,checkpoint).await.unwrap();
            let (db,connection)=tokio_postgres::connect(&f.dsn,tokio_postgres::NoTls).await.unwrap();tokio::spawn(async move {connection.await.unwrap()});
            let pending:i64=db.query_one("SELECT count(*) FROM mailbox_waits WHERE episode_id=$1 AND NOT satisfied",&[&episode]).await.unwrap().get(0);
            assert_eq!(pending,1);
            std::fs::write(out.join("checkpoint-wait-preserved.json"),serde_json::to_vec_pretty(&serde_json::json!({"checkpoint":recorded,"pending_waits":pending})).unwrap()).unwrap();
        } else {
            let successor=successor.expect("physically stopped released expired episode must permit a fresh same-lead successor");
            let final_episode=mailbox.record(&token,mailbox::RecordKind::Episode,episode).await.unwrap();
            assert!(matches!(final_episode["state"].as_str(),Some("cancelled"|"completed")));
            let handle=supervisor.create_episode(&token,successor,vec![],supervisor::CreateFault::None).await.unwrap();
            supervisor.cancel(&token,handle.id).await.unwrap();supervisor.release(&token,handle.id).await.unwrap();
            let final_view=f.broker.snapshot(&f.owner,SnapshotQuery::Mission(mission)).await.unwrap();
            assert_eq!(final_view.data["used_capacity"],0);assert_eq!(final_view.data["admitted_episodes"],2);
            std::fs::write(out.join("successor-public-positive.json"),serde_json::to_vec_pretty(&serde_json::json!({"successor":successor,"execution":handle.id,"mission":final_view})).unwrap()).unwrap();
        }
    }).catch_unwind().await;
    if supervisor
        .inspect(&f.owner, execution.id)
        .await
        .unwrap()
        .state
        != supervisor::ExecutionState::Released
    {
        supervisor.cancel(&f.owner, execution.id).await.unwrap();
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let view = supervisor.reconcile(&f.owner, execution.id).await.unwrap();
            if view.state == supervisor::ExecutionState::Stopped
                && supervisor
                    .result(&f.owner, execution.id)
                    .await
                    .unwrap()
                    .is_some()
            {
                break;
            }
            assert!(Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        supervisor.release(&f.owner, execution.id).await.unwrap();
    }
    let released = supervisor.inspect(&f.owner, execution.id).await.unwrap();
    assert!(released.members.is_empty());
    std::fs::write(out.join("cleanup.json"),serde_json::to_vec(&serde_json::json!({"execution":execution.id,"state":released.state,"members":released.members,"model_requests":0})).unwrap()).unwrap();
    let root = f.root.clone();
    f.finish().await;
    std::fs::write(
        out.join("fixture-removed.json"),
        serde_json::to_vec(&serde_json::json!({"removed":!root.exists()})).unwrap(),
    )
    .unwrap();
    if let Err(error) = outcome {
        std::panic::resume_unwind(error);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review611_uncertain_native_effect_cannot_retire_expired_episode() {
    let f = Fixture::new().await;
    let (mission, principal, token) = review611_lead_mission(&f, "uncertain").await;
    let episode = review611_admit_lead(&f, mission, principal, &token, "review611-uncertain-first")
        .await
        .unwrap();
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(&token, episode, vec![], supervisor::CreateFault::None)
        .await
        .unwrap();
    let root = f.root.join("review611-unregistered-native");
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
            &token,
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
    assert_eq!(journal.usage, episode_runtime::UsageObservation::Unknown);
    assert_eq!(runtime.0.effect_count().unwrap(), 1);
    assert!(runtime.1.as_mut().unwrap().try_wait().unwrap().is_none());
    let before_episode = mailbox::MailboxService::new(f.broker.clone())
        .record(&token, mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(2100)).await;
    supervisor.cancel(&token, execution.id).await.unwrap();
    assert_eq!(
        supervisor
            .inspect(&token, execution.id)
            .await
            .unwrap()
            .state,
        supervisor::ExecutionState::Stopped
    );
    let release = supervisor.release(&token, execution.id).await;
    let successor = review611_admit_lead(
        &f,
        mission,
        principal,
        &token,
        "review611-uncertain-successor",
    )
    .await;
    let episode_record = mailbox::MailboxService::new(f.broker.clone())
        .record(&token, mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    let mission_view = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(mission))
        .await
        .unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap())
        .join("review611-uncertain");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("uncertainty-refusals.json"),serde_json::to_vec_pretty(&serde_json::json!({"release":release.as_ref().map_err(|e|&e.code),"successor":successor.as_ref().map_err(|e|&e.code),"episode_state":episode_record["state"],"mission":mission_view,"effects":runtime.0.effect_count().unwrap(),"usage":journal.usage})).unwrap()).unwrap();
    let mut child = runtime.1.take().unwrap();
    let pid = child.id();
    child.kill().unwrap();
    let terminal = child.wait().unwrap();
    drop(runtime);
    std::fs::write(out.join("cleanup.json"),serde_json::to_vec(&serde_json::json!({"native_child_pid":pid,"waited":true,"terminal":terminal.to_string(),"physical_capacity_refunded":false,"model_requests":0})).unwrap()).unwrap();
    let root = f.root.clone();
    f.finish().await;
    std::fs::write(
        out.join("fixture-removed.json"),
        serde_json::to_vec(&serde_json::json!({"removed":!root.exists()})).unwrap(),
    )
    .unwrap();
    assert_eq!(release.unwrap_err().code, ErrorCode::DependencyUnmet);
    assert_eq!(successor.unwrap_err().code, ErrorCode::DependencyUnmet);
    assert_eq!(episode_record["state"], before_episode["state"]);
    assert_eq!(episode_record["state"], "active");
    assert_eq!(mission_view.data["used_capacity"], 1);
    assert_eq!(mission_view.data["admitted_episodes"], 1);
}

#[tokio::test]
async fn review611_expiry_without_execution_cannot_retire_episode() {
    let f = Fixture::new().await;
    let (mission, principal, token) = review611_lead_mission(&f, "unallocated").await;
    let episode = review611_admit_lead(
        &f,
        mission,
        principal,
        &token,
        "review611-unallocated-first",
    )
    .await
    .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(2100)).await;
    let successor = review611_admit_lead(
        &f,
        mission,
        principal,
        &token,
        "review611-unallocated-successor",
    )
    .await;
    let recorded = mailbox::MailboxService::new(f.broker.clone())
        .record(&token, mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    let view = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(mission))
        .await
        .unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap())
        .join("review611-unallocated");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("expiry-is-not-stop-proof.json"),serde_json::to_vec_pretty(&serde_json::json!({"episode":episode,"state":recorded["state"],"successor":successor.as_ref().map_err(|e|&e.code),"mission":view})).unwrap()).unwrap();
    let root = f.root.clone();
    f.finish().await;
    std::fs::write(
        out.join("cleanup.json"),
        serde_json::to_vec(
            &serde_json::json!({"removed":!root.exists(),"executions":0,"model_requests":0}),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(successor.unwrap_err().code, ErrorCode::DependencyUnmet);
    assert_eq!(recorded["state"], "admitted");
    assert_eq!(view.data["used_capacity"], 1);
    assert_eq!(view.data["admitted_episodes"], 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review611_stopped_unreleased_execution_cannot_retire_expired_episode() {
    use std::time::{Duration, Instant};
    let f = Fixture::new().await;
    let (mission, principal, token) = review611_lead_mission(&f, "unreleased").await;
    let episode =
        review611_admit_lead(&f, mission, principal, &token, "review611-unreleased-first")
            .await
            .unwrap();
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode(&token, episode, vec![], supervisor::CreateFault::None)
        .await
        .unwrap();
    supervisor
        .start(&token, execution.id, vec!["/bin/true".into()], 2)
        .await
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if supervisor
            .inspect(&token, execution.id)
            .await
            .unwrap()
            .state
            == supervisor::ExecutionState::Stopped
            && supervisor
                .result(&token, execution.id)
                .await
                .unwrap()
                .is_some()
        {
            break;
        }
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    tokio::time::sleep(Duration::from_millis(2100)).await;
    let successor = review611_admit_lead(
        &f,
        mission,
        principal,
        &token,
        "review611-unreleased-successor",
    )
    .await;
    let recorded = mailbox::MailboxService::new(f.broker.clone())
        .record(&token, mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    let view = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(mission))
        .await
        .unwrap();
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap())
        .join("review611-unreleased");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("stop-is-not-release-proof.json"),serde_json::to_vec_pretty(&serde_json::json!({"episode":episode,"execution":execution.id,"state":recorded["state"],"successor":successor.as_ref().map_err(|e|&e.code),"mission":view})).unwrap()).unwrap();
    supervisor.release(&token, execution.id).await.unwrap();
    let released = supervisor.inspect(&token, execution.id).await.unwrap();
    assert_eq!(released.state, supervisor::ExecutionState::Released);
    assert!(released.members.is_empty());
    let root = f.root.clone();
    f.finish().await;
    std::fs::write(out.join("cleanup.json"),serde_json::to_vec(&serde_json::json!({"removed":!root.exists(),"released_execution":execution.id,"model_requests":0})).unwrap()).unwrap();
    assert_eq!(successor.unwrap_err().code, ErrorCode::DependencyUnmet);
    assert_eq!(recorded["state"], "admitted");
    assert_eq!(view.data["used_capacity"], 1);
    assert_eq!(view.data["admitted_episodes"], 1);
}
