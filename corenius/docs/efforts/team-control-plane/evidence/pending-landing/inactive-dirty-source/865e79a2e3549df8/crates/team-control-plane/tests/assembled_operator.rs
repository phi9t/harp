//! Real PostgreSQL operator-view proof; no runtime dispatch or fabricated rows.
//! Mutation targets: unbounded summaries, incorrect overflow flags, summary-only
//! exact lookup, unauthorized disclosure, and historical grants becoming current.
#[path = "support/gateway_fault.rs"]
#[allow(dead_code)]
mod gateway_fault;
#[path = "support/assembled.rs"]
#[allow(dead_code)]
mod support;
use futures::FutureExt;
use std::{
    collections::BTreeSet,
    path::PathBuf,
    process::{Command as Process, Stdio},
    time::Duration,
};
use support::*;
use team_control_plane::{
    episode_runtime::{EpisodeGateway, StartRequest},
    gateway::RuntimeIdentity,
    kernel::Kernel,
    supervisor::{CreateFault, ExecutionState},
    *,
};
use uuid::Uuid;

struct Record {
    mission: Uuid,
    principal: Uuid,
    token: String,
    episode: Uuid,
    execution: Uuid,
    operation: Uuid,
}
fn evidence(name: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var("TEMPORAL_FIXTURE_EVIDENCE").expect("owned evidence path required"),
    )
    .join(name);
    std::fs::create_dir_all(&path).unwrap();
    path
}
async fn grant(f: &Fixture, mission: Uuid, principal: Uuid, token: &str, role: Role, seconds: i64) {
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                &format!("grant-{}", Uuid::new_v4()),
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: principal,
                    role,
                    mission_id: Some(mission),
                    task_id: None,
                    generation: None,
                    token_hash: token_hash(token),
                    expires_in_seconds: seconds,
                },
            ),
        )
        .await
        .unwrap();
}
async fn record(f: &Fixture, key: &str, seconds: i64) -> Record {
    let mission = Uuid::new_v4();
    let principal = Uuid::new_v4();
    let token = format!("operator-fixture-{}", Uuid::new_v4());
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                &format!("{key}-mission"),
                0,
                Command::CreateMission {
                    id: mission,
                    objective: format!("actual finite operator mission {key}"),
                    input: GitInput {
                        repository: f.root.join("store").to_str().unwrap().into(),
                        commit: f.base.clone(),
                        path: "SPEC.md".into(),
                        digest: artifacts::sha256(b"produce exact src/result"),
                    },
                    policy: Policy {
                        capacity: 2,
                        max_attempts: 8,
                        max_episodes: 8,
                        max_repairs: 3,
                        episode_deadline_seconds: 30,
                    },
                },
            ),
        )
        .await
        .unwrap();
    grant(f, mission, principal, &token, Role::Lead, seconds).await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    mailbox.execute(&token,mail(f.epoch,&format!("{key}-capture"),serde_json::json!({"kind":"capture_input","mission":mission,"id":input,"attempt":null,"purpose":"planning","limit":4}))).await.unwrap();
    let packet = mailbox
        .record(&token, mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute(&token,mail(f.epoch,&format!("{key}-admit"),serde_json::json!({"kind":"admit_episode","mission":mission,"id":episode,"attempt":null,"purpose":"planning","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
    let ex = f
        .supervisor()
        .await
        .create_episode(&token, episode, vec![], CreateFault::None)
        .await
        .unwrap();
    let operation = Uuid::new_v4();
    EpisodeGateway::new(f.broker.clone())
        .prepare(
            &token,
            StartRequest {
                id: operation,
                episode,
                execution: ex.id,
                runtime: RuntimeIdentity {
                    runtime_kind: "operator-undispatched-native".into(),
                    runtime_version: "1".into(),
                    deployment: f.epoch.to_string(),
                    thread_store: f
                        .root
                        .join(format!("private-native-{mission}"))
                        .to_str()
                        .unwrap()
                        .into(),
                    endpoint: "never-dispatched".into(),
                    environment: "bwrap-private-git".into(),
                },
                argv: vec![
                    "/bin/echo".into(),
                    format!("private-operator-effect-{operation}"),
                ],
            },
        )
        .await
        .unwrap();
    Record {
        mission,
        principal,
        token,
        episode,
        execution: ex.id,
        operation,
    }
}
async fn worker_record(f: &Fixture) -> Record {
    let episode = f.worker_episode("operator-same-task-role").await;
    let ex = f
        .supervisor()
        .await
        .create_episode("assembled-worker", episode, vec![], CreateFault::None)
        .await
        .unwrap();
    let operation = Uuid::new_v4();
    EpisodeGateway::new(f.broker.clone())
        .prepare(
            "assembled-worker",
            StartRequest {
                id: operation,
                episode,
                execution: ex.id,
                runtime: RuntimeIdentity {
                    runtime_kind: "operator-undispatched-native".into(),
                    runtime_version: "1".into(),
                    deployment: f.epoch.to_string(),
                    thread_store: f
                        .root
                        .join("private-worker-native")
                        .to_str()
                        .unwrap()
                        .into(),
                    endpoint: "never-dispatched".into(),
                    environment: "bwrap-private-git".into(),
                },
                argv: vec![
                    "/bin/echo".into(),
                    format!("private-worker-effect-{operation}"),
                ],
            },
        )
        .await
        .unwrap();
    Record {
        mission: f.mission,
        principal: f.worker,
        token: "assembled-worker".into(),
        episode,
        execution: ex.id,
        operation,
    }
}
fn cli(
    f: &Fixture,
    epoch: Uuid,
    bearer: &str,
    args: &[&str],
    out: &std::path::Path,
    label: &str,
) -> std::process::Output {
    let output = Process::new(env!("CARGO_BIN_EXE_team-control-plane"))
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TEAM_CONTROL_PLANE_DSN", &f.dsn)
        .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", epoch.to_string())
        .env("TEAM_CONTROL_PLANE_TOKEN", bearer)
        .env(
            "TEAM_CONTROL_PLANE_WORKSPACE_ROOT",
            f.root.join("workspaces"),
        )
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    std::fs::write(out.join(format!("{label}.stdout")), &output.stdout).unwrap();
    std::fs::write(out.join(format!("{label}.stderr")), &output.stderr).unwrap();
    std::fs::write(
        out.join(format!("{label}.exit")),
        output.status.code().unwrap().to_string(),
    )
    .unwrap();
    output
}
fn success(output: std::process::Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "public CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}
fn denied(output: std::process::Output, code: ErrorCode) {
    assert!(!output.status.success());
    assert!(
        output.stdout.is_empty(),
        "unauthorized CLI disclosed a record"
    );
    let error: Error =
        serde_json::from_slice(&output.stderr).expect("denial must contain only public error code");
    assert_eq!(error.code, code);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stderr).unwrap(),
        serde_json::json!({"code":error.code}),
        "denial cannot contain record IDs, credentials, private paths, or input packets"
    );
}
fn ids(value: &serde_json::Value, collection: &str) -> BTreeSet<Uuid> {
    value[collection]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().parse().unwrap())
        .collect()
}
async fn cleanup(f: &Fixture, records: &[Record], epoch: Uuid, owner: &str, out: &std::path::Path) {
    let broker = Broker::connect(&f.dsn, epoch).await.unwrap();
    let s = supervisor::Supervisor::open(
        broker.clone(),
        &f.root.join("workspaces"),
        env!("CARGO_BIN_EXE_team-control-plane"),
    )
    .await
    .unwrap();
    let mut released = vec![];
    for record in records {
        s.cancel(owner, record.execution).await.unwrap();
        let observed = s.reconcile(owner, record.execution).await.unwrap();
        assert_eq!(observed.state, ExecutionState::Stopped);
        assert!(observed.members.is_empty());
        assert!(observed.controller.is_none());
        assert!(observed.launcher.is_none());
        s.release(owner, record.execution).await.unwrap();
        assert_eq!(
            s.inspect(owner, record.execution).await.unwrap().state,
            ExecutionState::Released
        );
        assert_eq!(
            broker
                .snapshot(owner, SnapshotQuery::Mission(record.mission))
                .await
                .unwrap()
                .data["used_capacity"],
            0
        );
        released.push(record.execution);
    }
    std::fs::write(out.join("cleanup.json"),serde_json::to_vec_pretty(&serde_json::json!({"released_executions":released,"no_controller_launcher_or_members":true,"dispatched_runtime_operations":0,"model_requests":0})).unwrap()).unwrap();
}

#[tokio::test]
async fn thirty_three_actual_operator_records_bound_summary_and_allow_omitted_exact_lookup() {
    let out = evidence("overflow");
    let f = Fixture::new().await;
    let mut records = vec![];
    let outcome=std::panic::AssertUnwindSafe(async {
        for index in 0..33 {
            records.push(record(&f,&format!("overflow-{index}"),300).await);
            if index==30 || index==31 {
                let boundary=Kernel::new(f.broker.clone()).status(&f.owner).await.unwrap();
                std::fs::write(out.join(format!("boundary-{}.json",index+1)),serde_json::to_vec_pretty(&boundary).unwrap()).unwrap();
                assert_eq!(boundary["missions"].as_array().unwrap().len(),32);
                assert_eq!(boundary["more_missions"],index==31);
                assert_eq!(boundary["executions"].as_array().unwrap().len(),index+1);
                assert_eq!(boundary["unresolved_operations"].as_array().unwrap().len(),index+1);
                assert_eq!(boundary["more_executions"],false); assert_eq!(boundary["more_operations"],false);
            }
        }
        let kernel=Kernel::new(f.broker.clone()); let status=kernel.status(&f.owner).await.unwrap();
        let actual_cli=success(cli(&f,f.epoch,&f.owner,&["status"],&out,"owner-status"));
        std::fs::write(out.join("kernel-service-status.json"),serde_json::to_vec_pretty(&status).unwrap()).unwrap();
        let (clock_db,connection)=tokio_postgres::connect(&f.dsn,tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(async move {connection.await.unwrap()});
        let ordered:bool=clock_db.query_one("SELECT $1::text::timestamptz <= $2::text::timestamptz", &[&status["snapshot_time"].as_str().unwrap(),&actual_cli["snapshot_time"].as_str().unwrap()]).await.unwrap().get(0);
        assert!(ordered,"separate authenticated snapshots must have well-formed nondecreasing database times");
        let mut durable_cli=actual_cli.clone();let mut durable_service=status.clone();
        durable_cli.as_object_mut().unwrap().remove("snapshot_time");
        durable_service.as_object_mut().unwrap().remove("snapshot_time");
        assert_eq!(durable_cli,durable_service,"CLI must use the same authenticated durable broker view, including control revision");
        for (collection,more) in [("missions","more_missions"),("executions","more_executions"),("unresolved_operations","more_operations")] {
            assert_eq!(status[collection].as_array().unwrap().len(),32,"summary collection must have the documented finite32-row bound");
            assert_eq!(status[more],true,"genuine row33 must set overflow flag");
        }
        let text=serde_json::to_string(&status).unwrap();
        assert!(!text.contains(f.root.to_str().unwrap()),"summary cannot disclose private runtime directories");
        assert!(!text.contains(&f.owner));
        for record in &records {assert!(!text.contains(&record.token));}
        let mission_ids=ids(&status,"missions"); let execution_ids=ids(&status,"executions"); let operation_ids=ids(&status,"unresolved_operations");
        let omitted_operation=records.iter().find(|r|!operation_ids.contains(&r.operation)).unwrap();
        let omitted_execution=records.iter().find(|r|!execution_ids.contains(&r.execution)).unwrap();
        assert!(records.iter().any(|r|!mission_ids.contains(&r.mission)),"actual Mission outside bounded page must exist");
        let exact=success(cli(&f,f.epoch,&f.owner,&["inspect-episode-operation",&omitted_operation.operation.to_string()],&out,"owner-omitted-operation"));
        assert_eq!(exact["operation"]["id"],omitted_operation.operation.to_string());
        assert_eq!(exact["operation"]["state"],"prepared");
        assert_eq!(exact["prepared"]["start"]["episode"],omitted_operation.episode.to_string());
        assert_eq!(exact["prepared"]["start"]["execution"],omitted_operation.execution.to_string());
        assert_eq!(exact["prepared"]["input"]["role"],"lead");
        assert!(exact["prepared"]["input"]["task"].is_null()); assert!(exact["prepared"]["input"]["attempt"].is_null());
        assert_eq!(exact["evidence_more"],false,"prepared record has no native dispatch evidence; per-operation overflow is outside this fixture");
        let execution=success(cli(&f,f.epoch,&f.owner,&["inspect-execution",&omitted_execution.execution.to_string()],&out,"owner-omitted-execution"));
        assert_eq!(execution["manifest"]["episode"],omitted_execution.episode.to_string()); assert_eq!(execution["state"],"ready");
        assert!(execution["controller"].is_null()); assert!(execution["launcher"].is_null()); assert_eq!(execution["members"],serde_json::json!([]));
        let scoped=success(cli(&f,f.epoch,&omitted_operation.token,&["status"],&out,"current-lead-status"));
        assert_eq!(ids(&scoped,"missions"),BTreeSet::from([omitted_operation.mission]));
        assert_eq!(ids(&scoped,"executions"),BTreeSet::from([omitted_operation.execution]));
        assert_eq!(ids(&scoped,"unresolved_operations"),BTreeSet::from([omitted_operation.operation]));
        assert_eq!(scoped["more_missions"],false); assert_eq!(scoped["more_executions"],false); assert_eq!(scoped["more_operations"],false);
        for (label,bearer) in [("wrong-token","operator-nonexistent-token"),("wrong-mission",records.iter().find(|r|r.mission!=omitted_operation.mission).unwrap().token.as_str())] {
            denied(cli(&f,f.epoch,bearer,&["inspect-episode-operation",&omitted_operation.operation.to_string()],&out,label),ErrorCode::Unauthorized);
            denied(cli(&f,f.epoch,bearer,&["inspect-execution",&omitted_operation.execution.to_string()],&out,&format!("{label}-execution")),ErrorCode::Unauthorized);
        }
        denied(cli(&f,f.epoch,"operator-nonexistent-token",&["status"],&out,"wrong-token-status"),ErrorCode::Unauthorized);
        // Genuine Verifier grant shares the current Task and generation with the
        // Worker, so mission/task scope cannot explain this exact-read denial.
        let worker=worker_record(&f).await;
        denied(cli(&f,f.epoch,"assembled-verifier",&["inspect-episode-operation",&worker.operation.to_string()],&out,"same-task-wrong-role-operation"),ErrorCode::Unauthorized);
        denied(cli(&f,f.epoch,"assembled-verifier",&["inspect-execution",&worker.execution.to_string()],&out,"same-task-wrong-role-execution"),ErrorCode::Unauthorized);
        let wrong_role_status=success(cli(&f,f.epoch,"assembled-verifier",&["status"],&out,"same-task-wrong-role-status"));
        assert_eq!(ids(&wrong_role_status,"missions"),BTreeSet::from([f.mission]));
        assert_eq!(wrong_role_status["executions"],serde_json::json!([])); assert_eq!(wrong_role_status["unresolved_operations"],serde_json::json!([]));
        records.push(worker);
        for r in &records {
            let mission=f.broker.snapshot(&f.owner,SnapshotQuery::Mission(r.mission)).await.unwrap().data;
            assert_eq!(mission["capacity"],2); assert_eq!(mission["used_capacity"],1); assert_eq!(mission["max_episodes"],8); assert_eq!(mission["admitted_episodes"],1); assert_eq!(mission["max_repairs"],3); assert_eq!(mission["episode_deadline_seconds"],30);
        }
    }).catch_unwind().await;
    cleanup(&f, &records, f.epoch, &f.owner, &out).await;
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

#[tokio::test]
async fn operator_historical_records_require_current_lead_and_recovery_epoch_authority() {
    let out = evidence("historical");
    let f = Fixture::new().await;
    let mut records = vec![];
    let mut cleanup_epoch = f.epoch;
    let mut cleanup_owner = f.owner.clone();
    let outcome=std::panic::AssertUnwindSafe(async {
        records.push(record(&f,"historical",300).await); let r=&records[0];
        let initial=success(cli(&f,f.epoch,&r.token,&["inspect-episode-operation",&r.operation.to_string()],&out,"initial-lead-exact"));
        let replacement=format!("replacement-{}",Uuid::new_v4()); let replacement_principal=Uuid::new_v4();
        mailbox::MailboxService::new(f.broker.clone()).execute(&f.owner,mail(f.epoch,"replace-historical-lead",serde_json::json!({"kind":"replace_lead","mission":r.mission,"expected_generation":0,"principal":replacement_principal,"credential_id":Uuid::new_v4(),"token_hash":token_hash(&replacement),"expires_in_seconds":300}))).await.unwrap();
        for command in ["status","inspect-episode-operation","inspect-execution"] {
            let id=if command=="inspect-execution" {r.execution} else {r.operation};
            let args=if command=="status" {vec![command]} else {vec![command,""]};
            let id_text=id.to_string(); let args=if args.len()==1 {args} else {vec![command,id_text.as_str()]};
            denied(cli(&f,f.epoch,&r.token,&args,&out,&format!("historical-grant-{command}")),ErrorCode::StaleGeneration);
        }
        let current=success(cli(&f,f.epoch,&replacement,&["status"],&out,"current-replacement-status"));
        assert_eq!(ids(&current,"missions"),BTreeSet::from([r.mission])); assert_eq!(current["executions"],serde_json::json!([])); assert_eq!(current["unresolved_operations"],serde_json::json!([]));
        denied(cli(&f,f.epoch,&replacement,&["inspect-episode-operation",&r.operation.to_string()],&out,"current-lead-cannot-read-old-principal"),ErrorCode::Unauthorized);
        let owner=success(cli(&f,f.epoch,&f.owner,&["inspect-episode-operation",&r.operation.to_string()],&out,"owner-historical-exact"));
        assert_eq!(owner,initial,"current Owner reads durable historical records without takeover");
        let next_epoch=Uuid::new_v4(); cleanup_owner=f.broker.rotate_epoch(&f.owner,next_epoch).await.unwrap(); cleanup_epoch=next_epoch;
        for (label,epoch,bearer) in [("old-process-old-owner",f.epoch,f.owner.as_str()),("new-process-old-owner",next_epoch,f.owner.as_str()),("new-process-old-lead",next_epoch,replacement.as_str())] {
            denied(cli(&f,epoch,bearer,&["status"],&out,&format!("{label}-status")),ErrorCode::Unauthorized);
            denied(cli(&f,epoch,bearer,&["inspect-episode-operation",&r.operation.to_string()],&out,&format!("{label}-exact")),ErrorCode::Unauthorized);
        }
        let new_owner=success(cli(&f,next_epoch,&cleanup_owner,&["inspect-episode-operation",&r.operation.to_string()],&out,"current-epoch-owner-historical-exact"));
        assert_eq!(new_owner,initial,"new recovery Owner may inspect immutable old-epoch journal");
        let new_broker=Broker::connect(&f.dsn,next_epoch).await.unwrap();
        new_broker.execute(&cleanup_owner,envelope(next_epoch,"grant-current-epoch",0,Command::GrantCredential {credential_id:Uuid::new_v4(),principal_id:replacement_principal,role:Role::Lead,mission_id:Some(r.mission),task_id:None,generation:None,token_hash:token_hash("current-epoch-lead"),expires_in_seconds:300})).await.unwrap();
        let scoped=success(cli(&f,next_epoch,"current-epoch-lead",&["status"],&out,"current-epoch-scoped-status")); assert_eq!(scoped["unresolved_operations"],serde_json::json!([]));
        denied(cli(&f,next_epoch,"current-epoch-lead",&["inspect-episode-operation",&r.operation.to_string()],&out,"current-epoch-wrong-role"),ErrorCode::Unauthorized);
    }).catch_unwind().await;
    cleanup(&f, &records, cleanup_epoch, &cleanup_owner, &out).await;
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

#[tokio::test]
async fn alternate_current_grant_cannot_make_expired_issuing_execution_current() {
    let out = evidence("issuer-expiry");
    let f = Fixture::new().await;
    let mut records = vec![];
    let outcome = std::panic::AssertUnwindSafe(async {
        records.push(record(&f, "short-issuer", 1).await);
        let r = &records[0];
        let alternate = format!("alternate-current-{}", Uuid::new_v4());
        grant(&f, r.mission, r.principal, &alternate, Role::Lead, 300).await;
        let initial = success(cli(
            &f,
            f.epoch,
            &alternate,
            &["inspect-episode-operation", &r.operation.to_string()],
            &out,
            "current-grant-before-issuer-expiry",
        ));
        // The grant was issued for exactly one second through the public command.
        // Wait that lifetime; do not mutate auth rows or invent expired flags.
        tokio::time::sleep(Duration::from_secs(1)).await;
        denied(
            cli(
                &f,
                f.epoch,
                &r.token,
                &["status"],
                &out,
                "expired-issuing-token-status",
            ),
            ErrorCode::Unauthorized,
        );
        let current = success(cli(
            &f,
            f.epoch,
            &alternate,
            &["status"],
            &out,
            "alternate-current-status",
        ));
        assert_eq!(ids(&current, "missions"), BTreeSet::from([r.mission]));
        denied(
            cli(
                &f,
                f.epoch,
                &alternate,
                &["inspect-episode-operation", &r.operation.to_string()],
                &out,
                "alternate-grant-historical-exact",
            ),
            ErrorCode::StaleGeneration,
        );
        denied(
            cli(
                &f,
                f.epoch,
                &alternate,
                &["inspect-execution", &r.execution.to_string()],
                &out,
                "alternate-grant-historical-execution",
            ),
            ErrorCode::StaleGeneration,
        );
        let owner = success(cli(
            &f,
            f.epoch,
            &f.owner,
            &["inspect-episode-operation", &r.operation.to_string()],
            &out,
            "owner-expired-issuer-exact",
        ));
        assert_eq!(
            owner, initial,
            "current Owner retains historical exact access after issuing grant expires"
        );
    })
    .catch_unwind()
    .await;
    cleanup(&f, &records, f.epoch, &f.owner, &out).await;
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

#[tokio::test]
async fn merged_journals_share_one_bound_and_preserve_colliding_operation_ids() {
    let f = Fixture::new().await;
    let out = evidence("merged-journals");
    let worker = worker_record(&f).await;
    let root = f.root.join("lost-create-native");
    std::fs::create_dir(&root).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "actual-lost-create-files".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "unsupported-closed-world".into(),
        environment: "model-free-python-process".into(),
    };
    let gateway = gateway::Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(&f.owner, binding, f.admission, &identity, None, 1)
        .await
        .unwrap();
    gateway
        .prepare(
            &f.owner,
            gateway::Intent {
                id: worker.operation,
                admission_id: f.admission,
                binding_id: binding,
                generation: 1,
                kind: gateway::OperationKind::Create,
                arguments: serde_json::json!({"private":"must not disclose"}),
            },
        )
        .await
        .unwrap();
    let mut runtime =
        gateway_fault::FaultRuntime::new(root, gateway_fault::Fault::DropReply, identity);
    let operation = gateway
        .dispatch(&f.owner, worker.operation, &mut runtime)
        .await
        .unwrap();
    assert_eq!(operation.state, gateway::OperationState::OutcomeUnknown);
    assert!(operation.thread_id.is_none());
    assert_eq!(runtime.invocation_count().unwrap(), 1);
    assert_eq!(runtime.effect_count().unwrap(), 1);
    let mut records = Vec::new();
    let outcome = std::panic::AssertUnwindSafe(async {
        let kernel = Kernel::new(f.broker.clone());
        let scoped = kernel.status("assembled-worker").await.unwrap();
        let rows = scoped["unresolved_operations"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows.iter()
                .map(|row| row["protocol"].as_str().unwrap())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["episode_native", "legacy_gateway"])
        );
        assert!(
            rows.iter().all(
                |row| row["id"] == worker.operation.to_string() && row["capacity_held"] == true
            )
        );
        let legacy = rows
            .iter()
            .find(|row| row["protocol"] == "legacy_gateway")
            .unwrap();
        assert_eq!(legacy["kind"], "create");
        assert_eq!(legacy["thread_id"], serde_json::Value::Null);
        assert_eq!(legacy["usage"]["kind"], "unknown");
        assert!(
            kernel.status("assembled-verifier").await.unwrap()["unresolved_operations"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        for index in 0..31 {
            records.push(record(&f, &format!("merged-{index}"), 300).await);
            if index >= 29 {
                let status = kernel.status(&f.owner).await.unwrap();
                std::fs::write(
                    out.join(format!("boundary-{}.json", index + 3)),
                    serde_json::to_vec_pretty(&status).unwrap(),
                )
                .unwrap();
                assert_eq!(
                    status["unresolved_operations"].as_array().unwrap().len(),
                    32
                );
                assert_eq!(status["more_operations"], index == 30);
                let tuples = status["unresolved_operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| {
                        (
                            row["protocol"].as_str().unwrap(),
                            row["id"].as_str().unwrap(),
                        )
                    })
                    .collect::<BTreeSet<_>>();
                assert_eq!(
                    tuples.len(),
                    32,
                    "protocol and operation together identify each distinct journal record"
                );
            }
        }
        let foreign = kernel.status(&records[0].token).await.unwrap();
        assert_eq!(
            foreign["unresolved_operations"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            foreign["unresolved_operations"][0]["id"],
            records[0].operation.to_string()
        );
        let status = success(cli(
            &f,
            f.epoch,
            &f.owner,
            &["status"],
            &out,
            "merged-owner-cli",
        ));
        assert_eq!(
            status["unresolved_operations"].as_array().unwrap().len(),
            32
        );
        assert_eq!(status["more_operations"], true);
        assert!(
            status["unresolved_operations"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row.get("arguments").is_none() && row.get("runtime_identity").is_none())
        );
        denied(
            cli(
                &f,
                f.epoch,
                "wrong-token",
                &["status"],
                &out,
                "merged-denied",
            ),
            ErrorCode::Unauthorized,
        );
        assert_eq!(
            gateway.read(&f.owner, worker.operation).await.unwrap().id,
            worker.operation
        );
        assert_eq!(
            EpisodeGateway::new(f.broker.clone())
                .inspect("assembled-worker", worker.operation)
                .await
                .unwrap()
                .operation
                .id,
            worker.operation
        );
    })
    .catch_unwind()
    .await;
    cleanup(&f, &records, f.epoch, &f.owner, &out).await;
    let supervisor = f.supervisor().await;
    supervisor.cancel(&f.owner, worker.execution).await.unwrap();
    assert_eq!(
        supervisor
            .reconcile(&f.owner, worker.execution)
            .await
            .unwrap()
            .state,
        ExecutionState::Stopped
    );
    assert_eq!(
        supervisor
            .release(&f.owner, worker.execution)
            .await
            .unwrap_err()
            .code,
        ErrorCode::DependencyUnmet
    );
    assert_eq!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data["used_capacity"],
        1
    );
    f.finish().await;
    if let Err(payload) = outcome {
        std::panic::resume_unwind(payload);
    }
}

#[tokio::test]
async fn failed_known_legacy_effect_with_unknown_usage_remains_discoverable() {
    let f = Fixture::new().await;
    let root = f.root.join("failed-known-native");
    std::fs::create_dir(&root).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "actual-known-failure-files".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "unsupported-closed-world".into(),
        environment: "model-free-python-process".into(),
    };
    let gateway = gateway::Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    let operation = Uuid::new_v4();
    gateway
        .bind(&f.owner, binding, f.admission, &identity, None, 1)
        .await
        .unwrap();
    gateway
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
    let mut runtime =
        gateway_fault::FaultRuntime::new(root, gateway_fault::Fault::KnownFailure, identity);
    assert_eq!(
        gateway
            .dispatch(&f.owner, operation, &mut runtime)
            .await
            .unwrap()
            .state,
        gateway::OperationState::FailedKnown
    );
    assert_eq!(runtime.invocation_count().unwrap(), 1);
    let status = Kernel::new(f.broker.clone())
        .status(&f.owner)
        .await
        .unwrap();
    let row = status["unresolved_operations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["protocol"] == "legacy_gateway" && row["id"] == operation.to_string())
        .unwrap();
    assert_eq!(row["state"], "failed_known");
    assert_eq!(row["usage"]["kind"], "unknown");
    assert_eq!(row["capacity_held"], true);
    assert_eq!(status["missions"][0]["used_capacity"], 1);
    std::fs::write(
        evidence("failed-known").join("status.json"),
        serde_json::to_vec_pretty(&status).unwrap(),
    )
    .unwrap();
    f.finish().await;
}
