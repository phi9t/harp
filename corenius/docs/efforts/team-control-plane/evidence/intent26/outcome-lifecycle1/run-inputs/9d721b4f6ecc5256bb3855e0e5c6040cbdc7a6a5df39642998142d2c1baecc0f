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
                    role: role.clone(),
                    mission_id: if role == Role::Owner {
                        None
                    } else {
                        Some(mission)
                    },
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
    worker_record_named(f, "operator-same-task-role").await
}
async fn worker_record_named(f: &Fixture, key: &str) -> Record {
    let episode = f.worker_episode(key).await;
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
        let omitted_mission=records.iter().find(|r|!mission_ids.contains(&r.mission)).unwrap();
        let fresh_owner=format!("replacement-owner-{}",Uuid::new_v4());
        grant(&f,omitted_mission.mission,Uuid::new_v4(),&fresh_owner,Role::Owner,300).await;
        // Discard the pause reply as if the original view disappeared.
        kernel.pause(&f.owner,omitted_mission.mission,"lost-pause",0,true,"recover omitted Mission").await.unwrap();
        let exact_control=f.broker.snapshot(&fresh_owner,SnapshotQuery::Mission(omitted_mission.mission)).await.unwrap();
        std::fs::write(out.join("omitted-mission-exact-control.json"),serde_json::to_vec_pretty(&exact_control).unwrap()).unwrap();
        assert_eq!(exact_control.data["dispatch_paused"],true);
        assert_eq!(exact_control.data["control_revision"],1);
        assert_eq!(f.broker.snapshot("assembled-worker",SnapshotQuery::Mission(omitted_mission.mission)).await.unwrap_err().code,ErrorCode::Unauthorized);
        assert_eq!(kernel.pause("assembled-worker",omitted_mission.mission,"foreign-resume",1,false,"denied").await.unwrap_err().code,ErrorCode::Unauthorized);
        assert_eq!(kernel.pause(&fresh_owner,omitted_mission.mission,"stale-resume",0,false,"stale").await.unwrap_err().code,ErrorCode::StaleRevision);
        kernel.pause(&fresh_owner,omitted_mission.mission,"fresh-resume",exact_control.data["control_revision"].as_i64().unwrap(),false,"fresh exact recovery").await.unwrap();
        let resumed=f.broker.snapshot(&fresh_owner,SnapshotQuery::Mission(omitted_mission.mission)).await.unwrap();
        std::fs::write(out.join("omitted-mission-resumed.json"),serde_json::to_vec_pretty(&resumed).unwrap()).unwrap();
        assert_eq!(resumed.data["dispatch_paused"],false);
        assert_eq!(resumed.data["control_revision"],2);
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

#[tokio::test]
async fn legacy_attempt_execution_is_visible_to_replacement_owner_and_worker_only() {
    for uncertain in [false, true] {
        let f = Fixture::new().await;
        let supervisor = f.supervisor().await;
        let execution = supervisor
            .create_attempt("assembled-worker", f.attempt, CreateFault::None)
            .await
            .unwrap();
        let foreign_record = record(&f, "legacy-foreign-mission", 300).await;
        let owner = format!("legacy-replacement-{}", Uuid::new_v4());
        grant(&f, f.mission, Uuid::new_v4(), &owner, Role::Owner, 300).await;
        supervisor
            .start_with_fault(
                "assembled-worker",
                execution.id,
                vec!["/bin/sh".into(), "-c".into(), "sleep 20".into()],
                30,
                if uncertain {
                    supervisor::StartFault::BeforeMonitor
                } else {
                    supervisor::StartFault::None
                },
            )
            .await
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(8);
        let observed = loop {
            let view = if uncertain {
                supervisor.reconcile(&owner, execution.id).await.unwrap()
            } else {
                supervisor.inspect(&owner, execution.id).await.unwrap()
            };
            if view.state
                == if uncertain {
                    ExecutionState::Quarantined
                } else {
                    ExecutionState::Running
                }
            {
                break view;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "actual legacy workload must become observable"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        let status = Kernel::new(f.broker.clone()).status(&owner).await.unwrap();
        let worker = Kernel::new(f.broker.clone())
            .status("assembled-worker")
            .await
            .unwrap();
        let foreign = Kernel::new(f.broker.clone())
            .status(&foreign_record.token)
            .await
            .unwrap();
        let held = f
            .broker
            .snapshot(&owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data["used_capacity"]
            .clone();
        let out = evidence(if uncertain {
            "legacy-quarantined"
        } else {
            "legacy-running"
        });
        std::fs::write(out.join("public-status.json"),serde_json::to_vec_pretty(&serde_json::json!({"owner":status,"worker":worker,"foreign":foreign,"held_capacity":held,"observed":observed})).unwrap()).unwrap();
        let denied = supervisor
            .inspect(&foreign_record.token, execution.id)
            .await
            .unwrap_err()
            .code;
        if !uncertain {
            supervisor.cancel(&owner, execution.id).await.unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(8);
            loop {
                let v = supervisor.reconcile(&owner, execution.id).await.unwrap();
                if v.state == ExecutionState::Stopped {
                    break;
                }
                assert!(std::time::Instant::now() < deadline);
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            supervisor.release(&owner, execution.id).await.unwrap();
        } else {
            // BeforeMonitor launched no process. Quarantine deliberately retains
            // its reservation; fixture-owner removal does not imply a refund.
            assert!(observed.members.is_empty());
            assert!(observed.controller.is_none());
            assert!(observed.launcher.is_none());
            assert_eq!(
                supervisor
                    .release(&owner, execution.id)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet
            );
        }
        cleanup(&f, &[foreign_record], f.epoch, &owner, &out).await;
        let mission = f.mission;
        let task = f.task;
        let attempt = f.attempt;
        let root = f.root.clone();
        f.finish().await;
        std::fs::write(out.join("cleanup.json"),serde_json::to_vec(&serde_json::json!({"fixture_removed":!root.exists(),"launched":!uncertain,"released":!uncertain,"quarantine_retains_capacity":uncertain})).unwrap()).unwrap();
        assert_eq!(held, 1);
        assert_eq!(denied, ErrorCode::Unauthorized);
        assert!(!ids(&foreign, "executions").contains(&execution.id));
        for (value, count) in [(status, 2), (worker, 1)] {
            assert_eq!(
                value["executions"].as_array().unwrap().len(),
                count,
                "legacy Attempt-owned workload cannot disappear from bounded operator view"
            );
            let row = value["executions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["id"] == execution.id.to_string())
                .unwrap();
            assert_eq!(row["id"], execution.id.to_string());
            assert_eq!(row["mission"], mission.to_string());
            assert_eq!(row["task"], task.to_string());
            assert_eq!(row["attempt"], attempt.to_string());
            assert!(row["episode"].is_null());
            assert_eq!(row["role"], "worker");
            assert_eq!(row["capacity_released"], false);
            assert_eq!(row["capacity_held"], true);
            assert_eq!(
                row["observed_state"],
                if uncertain { "quarantined" } else { "running" }
            );
            assert_eq!(value["more_executions"], false);
        }
    }
}

#[tokio::test]
async fn monitor_finalizes_large_output_under_owned_measured_memory_limit() {
    use sha2::{Digest, Sha256};
    use std::{io::Read, os::unix::fs::PermissionsExt};
    for (bytes, read_error) in [
        (1024_u64 * 1024, false),
        (128 * 1024 * 1024, false),
        (1024 * 1024, true),
    ] {
        let f = Fixture::new().await;
        let out = evidence(&format!("monitor-memory-{bytes}-read-error-{read_error}"));
        let wrapper = f.root.join("owned-monitor.sh");
        std::fs::write(
            &wrapper,
            format!(
                "#!/bin/sh\nexport TOKIO_WORKER_THREADS=1\nexec '{}' \"$@\"\n",
                env!("CARGO_BIN_EXE_team-control-plane")
            ),
        )
        .unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
        let supervisor = supervisor::Supervisor::open(
            f.broker.clone(),
            &f.root.join("workspaces"),
            wrapper.to_str().unwrap(),
        )
        .await
        .unwrap();
        let episode = f.worker_episode("memory-finalization").await;
        let execution = supervisor
            .create_episode("assembled-worker", episode, vec![], CreateFault::None)
            .await
            .unwrap();
        supervisor
            .start(
                "assembled-worker",
                execution.id,
                vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    format!("sleep 3; head -c {bytes} /dev/zero"),
                ],
                30,
            )
            .await
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        let running = loop {
            let view = supervisor.inspect(&f.owner, execution.id).await.unwrap();
            if view.state == ExecutionState::Running {
                break view;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "owned monitor startup"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        };
        let controller = running.controller.clone().unwrap();
        let proc = PathBuf::from(format!("/proc/{}", controller.pid));
        let baseline = std::fs::read_to_string(proc.join("status")).unwrap();
        let vm_kib: u64 = baseline
            .lines()
            .find(|v| v.starts_with("VmSize:"))
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();
        let allowance = 64_u64 * 1024 * 1024;
        let limit = vm_kib * 1024 + allowance;
        let constrain = Process::new("/usr/bin/prlimit")
            .args([
                format!("--pid={}", controller.pid),
                format!("--as={limit}:{limit}"),
            ])
            .output()
            .unwrap();
        assert!(
            constrain.status.success(),
            "owned monitor prlimit: {}",
            String::from_utf8_lossy(&constrain.stderr)
        );
        let limits = std::fs::read_to_string(proc.join("limits")).unwrap();
        std::fs::write(out.join("memory-constraint.json"),serde_json::to_vec_pretty(&serde_json::json!({"controller":controller,"baseline":baseline,"limits":limits,"address_space_limit_bytes":limit,"headroom_bytes":allowance,"raw_log_bytes":bytes,"constraint_applied_after_workload_spawn":true})).unwrap()).unwrap();
        let raw_log = if read_error {
            let retained = running.manifest.log_path.with_extension("retained");
            std::fs::rename(&running.manifest.log_path, &retained).unwrap();
            std::fs::create_dir(&running.manifest.log_path).unwrap();
            retained
        } else {
            running.manifest.log_path.clone()
        };
        let result = loop {
            let result = supervisor.result(&f.owner, execution.id).await.unwrap();
            if result.is_some() || !proc.exists() {
                break result;
            }
            if std::time::Instant::now() >= deadline {
                break result;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        // Hash independently through a bounded reader; never materialize the log.
        let mut file = std::fs::File::open(&raw_log).unwrap();
        let mut hash = Sha256::new();
        let mut buffer = [0_u8; 32768];
        let mut measured = 0_u64;
        loop {
            let n = file.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            measured += n as u64;
            hash.update(&buffer[..n]);
        }
        let digest = format!("{:x}", hash.finalize());
        let view = supervisor.reconcile(&f.owner, execution.id).await.unwrap();
        let released = supervisor.release(&f.owner, execution.id).await;
        let capacity = f
            .broker
            .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data["used_capacity"]
            .clone();
        std::fs::write(out.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"result":result,"observed":view,"release":released.as_ref().map_err(|e|e.code.clone()),"capacity":capacity,"measured_log_bytes":measured,"independent_raw_sha256":digest})).unwrap()).unwrap();
        // The namespace has stopped even if old finalization failed. The monitor
        // is independently reaped before fixture removal and final proof oracles.
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while proc.exists() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let monitor_gone = !proc.exists();
        let members_empty = view.members.is_empty();
        let root = f.root.clone();
        f.finish().await;
        std::fs::write(out.join("cleanup.json"),serde_json::to_vec(&serde_json::json!({"monitor_gone":monitor_gone,"namespace_members_empty":members_empty,"fixture_removed":!root.exists()})).unwrap()).unwrap();
        assert!(monitor_gone);
        assert!(members_empty);
        assert_eq!(view.state, ExecutionState::Stopped);
        assert_eq!(measured, bytes);
        assert!(limits.contains(&limit.to_string()));
        if bytes > allowance {
            assert!(
                bytes > limit - vm_kib * 1024,
                "raw log allocation exceeds measured remaining address space"
            );
        }
        if read_error {
            assert!(
                result.is_none(),
                "unreadable original log cannot produce trusted success"
            );
            assert_eq!(released.unwrap_err().code, ErrorCode::DependencyUnmet);
            assert_eq!(capacity, 1);
            continue;
        }
        let result=result.expect("bounded monitor must persist trusted result despite raw log exceeding measured memory headroom");
        assert_eq!(result.exit_code, Some(0));
        assert!(result.namespace_stopped);
        assert_eq!(result.logs, digest);
        released.unwrap();
        assert_eq!(capacity, 0);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review580_real_monitor_crash_after_stop_recovers_unknown_immutable_result() {
    review580_crash(false, false, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review580_fast_monitor_crash_retains_exact_launch_stop_certificate() {
    review580_crash(true, false, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review580_surviving_monitor_thread_group_retains_capacity() {
    review580_crash(true, true, false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn review584_crash_then_rotate_finalizes_historical_execution() {
    review580_crash(false, false, true).await;
}
async fn review580_crash(fast: bool, survivor: bool, rotate: bool) {
    let f = Fixture::new().await;
    let out = evidence(if rotate {
        "review584-crash-rotate"
    } else if survivor {
        "review580-surviving-thread"
    } else if fast {
        "review580-fast-crash"
    } else {
        "review580-crash"
    });
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    // Gate only the real immutable persistence boundary, after real namespace Stop.
    db.batch_execute("CREATE FUNCTION review580_gate() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(580580); RETURN NEW; END $$; CREATE TRIGGER review580_gate BEFORE INSERT ON kernel_execution_results FOR EACH ROW EXECUTE FUNCTION review580_gate(); SELECT pg_advisory_lock(580580)").await.unwrap();
    let episode = f.worker_episode("review580-crash-worker").await;
    let supervisor = f.supervisor().await;
    let execution = supervisor
        .create_episode("assembled-worker", episode, vec![], CreateFault::None)
        .await
        .unwrap();
    supervisor
        .start(
            "assembled-worker",
            execution.id,
            vec![
                "/bin/sh".into(),
                "-c".into(),
                if fast {
                    "printf actual-crash-output"
                } else {
                    "sleep 1; printf actual-crash-output"
                }
                .into(),
            ],
            30,
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let stopped = loop {
        let row = db.query_one("SELECT state,controller,launcher,init_process,members FROM supervised_workspaces WHERE id=$1",&[&execution.id]).await.unwrap();
        let waiting: bool = db.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE wait_event='advisory' AND query LIKE 'INSERT INTO kernel_execution_results%')",&[]).await.unwrap().get(0);
        if row.get::<_, String>(0) == "stopped" && waiting {
            break serde_json::json!({"state":row.get::<_,String>(0),"controller":row.get::<_,serde_json::Value>(1),"launcher":row.get::<_,serde_json::Value>(2),"init_process":row.get::<_,serde_json::Value>(3),"members":row.get::<_,serde_json::Value>(4)});
        }
        assert!(
            std::time::Instant::now() < deadline,
            "real monitor persistence gate must be reached"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    let controller: supervisor::ProcessIdentity =
        serde_json::from_value(stopped["controller"].clone()).unwrap();
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", controller.pid)).unwrap();
    let fields: Vec<_> = stat[stat.rfind(')').unwrap() + 2..]
        .split_whitespace()
        .collect();
    assert_eq!(
        fields[19].parse::<u64>().unwrap(),
        controller.start_ticks,
        "kill exact owned monitor only"
    );
    assert_eq!(
        std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .unwrap()
            .trim(),
        controller.boot_id
    );
    assert_eq!(
        unsafe { libc::kill(controller.pid as i32, libc::SIGKILL) },
        0
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while std::path::Path::new(&format!("/proc/{}", controller.pid)).exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "owned monitor reaper must collect crashed process"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    db.batch_execute("SELECT pg_advisory_unlock(580580); DROP TRIGGER review580_gate ON kernel_execution_results; DROP FUNCTION review580_gate()").await.unwrap();
    let current_epoch = if rotate { Uuid::new_v4() } else { f.epoch };
    let fresh_owner = if rotate {
        f.broker
            .rotate_epoch(&f.owner, current_epoch)
            .await
            .unwrap()
    } else {
        let token = format!("review580-owner-{}", Uuid::new_v4());
        grant(&f, f.mission, Uuid::new_v4(), &token, Role::Owner, 300).await;
        token
    };
    let broker = Broker::connect(&f.dsn, current_epoch).await.unwrap();
    let supervisor = supervisor::Supervisor::open(
        broker.clone(),
        &f.root.join("workspaces"),
        env!("CARGO_BIN_EXE_team-control-plane"),
    )
    .await
    .unwrap();
    if rotate {
        assert!(
            supervisor
                .inspect("assembled-worker", execution.id)
                .await
                .is_err()
        );
        assert!(
            supervisor
                .start(&fresh_owner, execution.id, vec!["/bin/true".into()], 30)
                .await
                .is_err()
        );
    }
    if survivor {
        // Negative controller-identity fault uses a genuine owned live thread
        // group with a zombie leader; it cannot manufacture successful Stop.
        let mut child=Process::new("/usr/bin/python3").args(["-c","import ctypes,threading,time; threading.Thread(target=lambda:time.sleep(30)).start(); time.sleep(1); ctypes.CDLL(None).syscall(60,0)"]).env_clear().env("PATH","/usr/bin:/bin").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let pid = child.id();
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
        let fields: Vec<_> = stat[stat.rfind(')').unwrap() + 2..]
            .split_whitespace()
            .collect();
        let replacement = supervisor::ProcessIdentity {
            pid,
            start_ticks: fields[19].parse().unwrap(),
            boot_id: controller.boot_id.clone(),
            namespace: std::fs::read_link(format!("/proc/{pid}/ns/pid"))
                .unwrap()
                .to_string_lossy()
                .into(),
            namespace_pid: Some(pid),
        };
        db.execute(
            "UPDATE supervised_workspaces SET controller=$2 WHERE id=$1",
            &[&execution.id, &serde_json::to_value(&replacement).unwrap()],
        )
        .await
        .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
            if stat[stat.rfind(')').unwrap() + 2..]
                .split_whitespace()
                .next()
                == Some("Z")
            {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
        assert!(fd >= 0);
        let mut poll = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        assert_eq!(unsafe { libc::poll(&mut poll, 1, 0) }, 0);
        unsafe { libc::close(fd) };
        supervisor
            .reconcile(&fresh_owner, execution.id)
            .await
            .unwrap();
        let release = supervisor.release(&fresh_owner, execution.id).await;
        let result = supervisor.result(&fresh_owner, execution.id).await.unwrap();
        let held = f
            .broker
            .snapshot(&fresh_owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data;
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
        assert_eq!(
            stat[stat.rfind(')').unwrap() + 2..]
                .split_whitespace()
                .nth(19)
                .unwrap()
                .parse::<u64>()
                .unwrap(),
            replacement.start_ticks
        );
        child.kill().unwrap();
        let terminal = child.wait().unwrap();
        std::fs::write(out.join("survivor.json"),serde_json::to_vec_pretty(&serde_json::json!({"actual_controller_fault":replacement,"leader_zombie_but_whole_group_not_ready":true,"release":release.as_ref().map_err(|e|&e.code),"result":result,"held":held,"cleanup_terminal":terminal.to_string()})).unwrap()).unwrap();
        let root = f.root.clone();
        f.finish().await;
        assert!(!root.exists());
        assert!(
            result.is_none(),
            "surviving monitor threads forbid recovered result"
        );
        assert_eq!(release.unwrap_err().code, ErrorCode::DependencyUnmet);
        assert_eq!(held["used_capacity"], 1);
        return;
    }
    let independent_digest = artifacts::sha256(
        &std::fs::read(
            supervisor
                .inspect(&fresh_owner, execution.id)
                .await
                .unwrap()
                .manifest
                .log_path,
        )
        .unwrap(),
    );
    let original_log = supervisor
        .inspect(&fresh_owner, execution.id)
        .await
        .unwrap()
        .manifest
        .log_path;
    let withheld = original_log.with_extension("withheld");
    let mut controls = vec![];
    for kind in ["symlink", "fifo", "unsafe-mode", "directory"] {
        std::fs::rename(&original_log, &withheld).unwrap();
        match kind {
            "symlink" => std::os::unix::fs::symlink(&withheld, &original_log).unwrap(),
            "fifo" => {
                use std::os::unix::ffi::OsStrExt;
                let name = std::ffi::CString::new(original_log.as_os_str().as_bytes()).unwrap();
                assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
            }
            "unsafe-mode" => {
                use std::os::unix::fs::PermissionsExt;
                std::fs::write(&original_log, b"unsafe").unwrap();
                std::fs::set_permissions(&original_log, std::fs::Permissions::from_mode(0o666))
                    .unwrap();
            }
            "directory" => std::fs::create_dir(&original_log).unwrap(),
            _ => unreachable!(),
        }
        supervisor
            .reconcile(&fresh_owner, execution.id)
            .await
            .unwrap();
        let denied = supervisor.release(&fresh_owner, execution.id).await;
        let absent = supervisor.result(&fresh_owner, execution.id).await.unwrap();
        controls.push((kind, denied.map_err(|e| e.code), absent));
        if kind == "directory" {
            std::fs::remove_dir(&original_log).unwrap();
        } else {
            std::fs::remove_file(&original_log).unwrap();
        }
        std::fs::rename(&withheld, &original_log).unwrap();
    }
    let unauthorized = supervisor.reconcile("wrong-token", execution.id).await;
    let missing = supervisor.result(&fresh_owner, execution.id).await.unwrap();
    let (a, b) = tokio::join!(
        supervisor.reconcile(&fresh_owner, execution.id),
        supervisor.reconcile(&fresh_owner, execution.id)
    );
    let view = a.unwrap();
    b.unwrap();
    let first = supervisor.result(&fresh_owner, execution.id).await.unwrap();
    let release = supervisor.release(&fresh_owner, execution.id).await;
    let final_result = supervisor.result(&fresh_owner, execution.id).await.unwrap();
    let repeat = supervisor
        .reconcile(&fresh_owner, execution.id)
        .await
        .unwrap();
    let repeated_result = supervisor.result(&fresh_owner, execution.id).await.unwrap();
    let capacity = broker
        .snapshot(&fresh_owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data["used_capacity"]
        .clone();
    std::fs::write(out.join("crash.json"),serde_json::to_vec_pretty(&serde_json::json!({"execution":execution,"independent_raw_digest":independent_digest,"controls":controls,"unauthorized":unauthorized.as_ref().map_err(|e|&e.code),"original_stop":stopped,"before":missing,"view":view,"first":first,"release":release.as_ref().map_err(|e|&e.code),"result":final_result,"repeat":repeat,"repeated_result":repeated_result,"capacity":capacity})).unwrap()).unwrap();
    let epoch = f.epoch;
    let root = f.root.clone();
    f.finish().await;
    std::fs::write(out.join("cleanup.json"), serde_json::to_vec(&serde_json::json!({"fixture_removed":!root.exists(),"crashed_monitor_reaped":true,"complete_stop":stopped["members"]==serde_json::json!([])})).unwrap()).unwrap();
    assert!(!root.exists(), "owned cleanup before recovery oracle");
    for (kind, denied, absent) in controls {
        assert_eq!(denied.unwrap_err(), ErrorCode::DependencyUnmet, "{kind}");
        assert!(absent.is_none());
    }
    assert_eq!(unauthorized.unwrap_err().code, ErrorCode::Unauthorized);
    assert!(missing.is_none(), "crash aborted actual result persistence");
    assert!(
        release.is_ok(),
        "replacement Owner must release real stopped crash without guessing exit: {release:?}"
    );
    let result = final_result.expect("durable recovered result");
    assert_eq!(result.exit_code, None);
    assert_eq!(result.signal, None);
    assert_eq!(result.controller, controller);
    assert_eq!(result.logs, independent_digest);
    assert_eq!(result.recovery_epoch, epoch);
    match result.completion {
        supervisor::CompletionProvenance::RecoveredStop { recovery_epoch, .. } => {
            assert_eq!(recovery_epoch, current_epoch)
        }
        _ => panic!("crashed monitor cannot claim monitor-wait completion"),
    }
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        serde_json::to_value(repeated_result.unwrap()).unwrap()
    );
    assert_eq!(capacity, 0);
}

#[tokio::test]
async fn same_principal_cross_role_episode_summaries_agree_with_exact_denial() {
    let out = evidence("same-principal-role");
    let f = Fixture::new().await;
    let r = worker_record(&f).await;
    let outcome = std::panic::AssertUnwindSafe(async {
        let token = "same-principal-verifier";
        f.broker
            .execute(
                &f.owner,
                envelope(
                    f.epoch,
                    "same-principal-verifier-grant",
                    0,
                    Command::GrantCredential {
                        credential_id: Uuid::new_v4(),
                        principal_id: f.worker,
                        role: Role::Verifier,
                        mission_id: Some(f.mission),
                        task_id: Some(f.task),
                        generation: Some(1),
                        token_hash: token_hash(token),
                        expires_in_seconds: 300,
                    },
                ),
            )
            .await
            .unwrap();
        for (command, id) in [
            ("inspect-execution", r.execution),
            ("inspect-episode-operation", r.operation),
        ] {
            denied(
                cli(
                    &f,
                    f.epoch,
                    token,
                    &[command, &id.to_string()],
                    &out,
                    command,
                ),
                ErrorCode::Unauthorized,
            );
            for (label, bearer) in [("worker", "assembled-worker"), ("owner", f.owner.as_str())] {
                success(cli(
                    &f,
                    f.epoch,
                    bearer,
                    &[command, &id.to_string()],
                    &out,
                    &format!("{label}-{command}"),
                ));
            }
        }
        let status = success(cli(
            &f,
            f.epoch,
            token,
            &["status"],
            &out,
            "verifier-status",
        ));
        assert_eq!(
            (&status["executions"], &status["unresolved_operations"]),
            (&serde_json::json!([]), &serde_json::json!([])),
            "both cross-role episode summary collections must be empty"
        );
        assert_eq!(status["more_executions"], false);
        assert_eq!(status["more_operations"], false);
        for bearer in ["assembled-worker", f.owner.as_str()] {
            let status = Kernel::new(f.broker.clone()).status(bearer).await.unwrap();
            assert_eq!(ids(&status, "executions"), BTreeSet::from([r.execution]));
            assert_eq!(
                ids(&status, "unresolved_operations"),
                BTreeSet::from([r.operation])
            );
        }
    })
    .catch_unwind()
    .await;
    cleanup(&f, &[r], f.epoch, &f.owner, &out).await;
    f.finish().await;
    std::fs::write(out.join("fixture-removed.json"), b"{\"removed\":true}").unwrap();
    if let Err(error) = outcome {
        std::panic::resume_unwind(error);
    }
}

async fn role_boundary_record(f: &Fixture, token: &str, key: &str, worker: bool) -> Record {
    mailbox::MailboxService::new(f.broker.clone()).execute(&f.owner, mail(f.epoch, &format!("{key}-notice"), serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":format!("admit finite pagination record {key}"),"reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let episode = if worker {
        f.worker_episode(key).await
    } else {
        let mailbox = mailbox::MailboxService::new(f.broker.clone());
        let input = Uuid::new_v4();
        let episode = Uuid::new_v4();
        mailbox.execute(token, mail(f.epoch, &format!("{key}-capture"), serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":null,"purpose":"planning","limit":4}))).await.unwrap();
        let packet = mailbox
            .record(token, mailbox::RecordKind::Input, input)
            .await
            .unwrap();
        mailbox.execute(token, mail(f.epoch, &format!("{key}-admit"), serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":null,"purpose":"planning","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
        episode
    };
    let s = f.supervisor().await;
    let ex = s
        .create_episode(token, episode, vec![], CreateFault::None)
        .await
        .unwrap();
    let operation = Uuid::new_v4();
    EpisodeGateway::new(f.broker.clone())
        .prepare(
            token,
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
                        .join(format!("private-boundary-{episode}"))
                        .to_str()
                        .unwrap()
                        .into(),
                    endpoint: "never-dispatched".into(),
                    environment: "bwrap-private-git".into(),
                },
                argv: vec![
                    "/bin/echo".into(),
                    format!("private-boundary-effect-{operation}"),
                ],
            },
        )
        .await
        .unwrap();
    let r = Record {
        mission: f.mission,
        principal: f.worker,
        token: token.into(),
        episode,
        execution: ex.id,
        operation,
    };
    // Public checkpoints advance the episode frontier; public physical observation
    // and release free capacity without fabricating rows or dispatching a runtime.
    mailbox::MailboxService::new(f.broker.clone()).execute(token, mail(f.epoch, &format!("{key}-yield"), serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"finite pagination record","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    s.cancel(&f.owner, ex.id).await.unwrap();
    let observed = s.reconcile(&f.owner, ex.id).await.unwrap();
    assert_eq!(observed.state, ExecutionState::Stopped);
    assert!(observed.members.is_empty());
    assert!(observed.controller.is_none() && observed.launcher.is_none());
    s.release(&f.owner, ex.id).await.unwrap();
    assert_eq!(
        s.inspect(&f.owner, ex.id).await.unwrap().state,
        ExecutionState::Released
    );
    r
}

#[tokio::test]
async fn same_principal_role_filter_precedes_thirty_two_row_bounds() {
    let out = evidence("same-principal-role-boundary");
    let f = Fixture::with_limits(2, 8, 80).await;
    let mut records = Vec::new();
    let outcome = std::panic::AssertUnwindSafe(async {
        let token = "same-principal-boundary-lead";
        grant(&f, f.mission, f.worker, token, Role::Lead, 300).await;
        let bridge = temporal::Bridge::new(f.broker.clone());
        bridge
            .register(&f.owner, temporal::AggregateKind::Task, f.task, 0)
            .await
            .unwrap();
        bridge
            .register(&f.owner, temporal::AggregateKind::Mission, f.mission, 0)
            .await
            .unwrap();
        for index in 0..1 {
            records.push(
                role_boundary_record(&f, "assembled-worker", &format!("hidden-{index}"), true)
                    .await,
            );
        }
        let kernel = Kernel::new(f.broker.clone());
        let hidden = kernel.status(token).await.unwrap();
        std::fs::write(
            out.join("unauthorized-only.json"),
            serde_json::to_vec_pretty(&hidden).unwrap(),
        )
        .unwrap();
        let hidden_executions: BTreeSet<_> = records.iter().map(|r| r.execution).collect();
        let hidden_operations: BTreeSet<_> = records.iter().map(|r| r.operation).collect();
        for index in 0..33 {
            records.push(role_boundary_record(&f, token, &format!("visible-{index}"), false).await);
            if index >= 30 {
                let status = success(cli(
                    &f,
                    f.epoch,
                    token,
                    &["status"],
                    &out,
                    &format!("visible-boundary-{}", index + 1),
                ));
                for (collection, more, hidden_ids) in [
                    ("executions", "more_executions", &hidden_executions),
                    (
                        "unresolved_operations",
                        "more_operations",
                        &hidden_operations,
                    ),
                ] {
                    assert_eq!(
                        status[collection].as_array().unwrap().len(),
                        (index + 1).min(32)
                    );
                    assert_eq!(status[more], index == 32);
                    assert!(ids(&status, collection).is_disjoint(hidden_ids));
                }
            }
        }
        assert_eq!(
            (&hidden["executions"], &hidden["unresolved_operations"]),
            (&serde_json::json!([]), &serde_json::json!([]))
        );
        assert_eq!(hidden["more_executions"], false);
        assert_eq!(hidden["more_operations"], false);
        let worker = kernel.status("assembled-worker").await.unwrap();
        assert_eq!(worker["executions"].as_array().unwrap().len(), 1);
        assert_eq!(worker["unresolved_operations"].as_array().unwrap().len(), 1);
        assert_eq!(worker["more_executions"], false);
        assert_eq!(worker["more_operations"], false);
        let owner = kernel.status(&f.owner).await.unwrap();
        assert_eq!(owner["executions"].as_array().unwrap().len(), 32);
        assert_eq!(owner["unresolved_operations"].as_array().unwrap().len(), 32);
        assert_eq!(owner["more_executions"], true);
        assert_eq!(owner["more_operations"], true);
    })
    .catch_unwind()
    .await;
    // Every successfully returned record already carries a public release proof.
    for r in &records {
        assert_eq!(
            f.supervisor()
                .await
                .inspect(&f.owner, r.execution)
                .await
                .unwrap()
                .state,
            ExecutionState::Released
        );
    }
    std::fs::write(out.join("cleanup.json"), serde_json::to_vec_pretty(&serde_json::json!({"released_executions":records.iter().map(|r|r.execution).collect::<Vec<_>>(),"no_controller_launcher_or_members":true,"model_requests":0,"dispatched_runtime_operations":0})).unwrap()).unwrap();
    f.finish().await;
    std::fs::write(out.join("fixture-removed.json"), b"{\"removed\":true}").unwrap();
    if let Err(error) = outcome {
        std::panic::resume_unwind(error);
    }
}
