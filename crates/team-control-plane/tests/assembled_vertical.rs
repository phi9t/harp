#[path = "support/episode_native.rs"]
mod episode_native;
#[path = "support/assembled.rs"]
#[allow(dead_code)]
mod support;
use std::os::unix::process::CommandExt;
use std::{
    collections::HashMap,
    path::PathBuf,
    process::{Child, Command as Process, Stdio},
    time::Duration,
};
use support::*;
use team_control_plane::{gateway::*, temporal::*, *};
use uuid::Uuid;
struct SupervisedRuntime {
    root: PathBuf,
    identity: RuntimeIdentity,
    dsn: String,
    epoch: Uuid,
    bearer: String,
    workspaces: PathBuf,
    drop_start_reply: bool,
    unmeasured_start: bool,
}
impl SupervisedRuntime {
    fn invoke(&self, mode: &str, request: serde_json::Value) -> serde_json::Value {
        let output = Process::new("python3")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/mailbox_runtime.py"
            ))
            .arg(&self.root)
            .arg(mode)
            .arg(request.to_string())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "physical runtime fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}
impl RuntimeAdapter for SupervisedRuntime {
    fn identity(&self) -> RuntimeIdentity {
        self.identity.clone()
    }
    fn closed_world_os(&self) -> Option<episode_runtime::ClosedWorldOs> {
        Some(episode_runtime::ClosedWorldOs {
            workspace_root: self.workspaces.clone(),
            native_root: self.root.clone(),
        })
    }
    fn inspect(&self) -> CapabilityProfile {
        CapabilityProfile {
            create: true,
            start: true,
            observe: true,
            authoritative_effect_inventory: true,
            native_deduplication: false,
            ..Default::default()
        }
    }
    fn execute(&mut self, intent: &Intent, _thread: Option<&str>) -> RuntimeReply {
        if intent.kind == OperationKind::Start {
            let id = intent.arguments["execution"].as_str().unwrap();
            let argv = intent.arguments["argv"].to_string();
            let output = Process::new(env!("CARGO_BIN_EXE_team-control-plane"))
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .args(["start-execution", id, "30", &argv])
                .env("TEAM_CONTROL_PLANE_DSN", &self.dsn)
                .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", self.epoch.to_string())
                .env("TEAM_CONTROL_PLANE_TOKEN", &self.bearer)
                .env("TEAM_CONTROL_PLANE_WORKSPACE_ROOT", &self.workspaces)
                .output()
                .unwrap();
            if !output.status.success() {
                return RuntimeReply::FailedKnown(format!(
                    "supervisor start rejected: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        let mode = if intent.kind == OperationKind::Start {
            "start"
        } else {
            "create"
        };
        let reply = self.invoke(
            mode,
            serde_json::json!({"intent":intent,"runtime":self.identity}),
        );
        if intent.kind == OperationKind::Start && self.unmeasured_start {
            for entry in std::fs::read_dir(&self.root).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|x| x != "effect") {
                    continue;
                }
                let bytes = std::fs::read(&path).unwrap();
                let mut receipt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                if receipt["operation"] != intent.id.to_string() {
                    continue;
                }
                retain_bytes(
                    &self
                        .root
                        .parent()
                        .unwrap()
                        .join(format!("original-legacy-receipt-{}.json", intent.id)),
                    &bytes,
                );
                receipt["usage"]["profile"] = serde_json::json!("unsupported-cost-telemetry");
                std::fs::write(path, serde_json::to_vec(&receipt).unwrap()).unwrap();
            }
        }
        if intent.kind == OperationKind::Start && self.drop_start_reply {
            return RuntimeReply::Unknown(
                "actual Start effect persisted; reply deliberately lost".into(),
            );
        }
        RuntimeReply::Confirmed {
            thread_id: Some(reply["thread_id"].as_str().unwrap().into()),
            effect_id: reply["effect_id"].as_str().unwrap().into(),
        }
    }
    fn confirmed_start_turn(&self, intent: &Intent, reply: &RuntimeReply) -> Option<String> {
        if intent.kind != OperationKind::Start {
            return None;
        }
        let RuntimeReply::Confirmed { effect_id, .. } = reply else {
            return None;
        };
        let mut turns = vec![];
        for entry in std::fs::read_dir(&self.root).ok()? {
            let path = entry.ok()?.path();
            if path.extension().is_none_or(|x| x != "effect") {
                continue;
            }
            let record: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
            if record["operation"] == intent.id.to_string() && record["id"] == *effect_id {
                turns.push(record["turn"].as_str()?.to_owned());
            }
        }
        if turns.len() == 1 { turns.pop() } else { None }
    }
    fn observe(&mut self, _intent: &Intent, _thread: Option<&str>) -> Observation {
        Observation::Inconclusive("recovery inventory not assumed from a lost reply".into())
    }
}
fn evidence_directory(label: &str) -> PathBuf {
    let root = PathBuf::from(
        std::env::var("TEMPORAL_FIXTURE_EVIDENCE").expect("owned Temporal runner retains evidence"),
    );
    let path = root.join(format!("kernel16-{label}-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn retain_bytes(path: &std::path::Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
    std::fs::File::open(path.parent().unwrap())
        .unwrap()
        .sync_all()
        .unwrap();
}
fn retain_native(root: &std::path::Path, out: &std::path::Path, phase: &str) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|x| x == "invocation" || x == "effect")
        {
            let name = format!("{phase}-{}", entry.file_name().to_str().unwrap());
            retain_bytes(&out.join(name), &std::fs::read(path).unwrap());
        }
    }
}
async fn retain_replay(
    client: &temporalio_client::Client,
    workflow: &WorkflowRef,
    run: &str,
    out: &std::path::Path,
    label: &str,
    secret: &str,
) -> Vec<u8> {
    use temporalio_client::{WorkflowExecutionInfo, WorkflowFetchHistoryOptions};
    let bytes = WorkflowExecutionInfo::builder()
        .namespace("default")
        .workflow_id(&workflow.workflow_id)
        .run_id(run)
        .build()
        .bind_untyped(client.clone())
        .fetch_history(WorkflowFetchHistoryOptions::default())
        .to_json()
        .await
        .unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains(secret));
    fn check(value: &serde_json::Value, secret: &str) {
        use base64::Engine;
        match value {
            serde_json::Value::Object(o) => {
                for (k, v) in o {
                    if k == "data"
                        && let Some(s) = v.as_str()
                        && let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(s)
                    {
                        assert!(!String::from_utf8_lossy(&bytes).contains(secret));
                    }
                    check(v, secret);
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    check(v, secret);
                }
            }
            _ => {}
        }
    }
    check(&serde_json::from_slice(&bytes).unwrap(), secret);
    retain_bytes(&out.join(format!("{label}.json")), &bytes);
    use temporalio_sdk::workflow_replayer::{WorkflowReplayer, WorkflowReplayerOptions};
    let replayer = WorkflowReplayer::new(
        WorkflowReplayerOptions::new()
            .register_workflow::<MissionWorkflow>()
            .unwrap()
            .register_workflow::<TaskWorkflow>()
            .unwrap()
            .build(),
    )
    .unwrap();
    replayer
        .replay_workflow(temporalio_client::WorkflowHistory::from_json(&bytes).unwrap())
        .await
        .unwrap();
    bytes
}
async fn bounded_operator_cli(mut command: Process) -> (std::process::Output, bool) {
    use std::{io::Read, os::unix::process::ExitStatusExt};
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut forced = false;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            assert!(child.try_wait().unwrap().is_none());
            child.kill().unwrap();
            let status = child.wait().unwrap();
            assert_eq!(status.signal(), Some(9));
            forced = true;
            break status;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    let mut stdout = vec![];
    let mut stderr = vec![];
    child
        .stdout
        .take()
        .unwrap()
        .take(65537)
        .read_to_end(&mut stdout)
        .unwrap();
    child
        .stderr
        .take()
        .unwrap()
        .take(65537)
        .read_to_end(&mut stderr)
        .unwrap();
    assert!(stdout.len() <= 65536 && stderr.len() <= 65536);
    (
        std::process::Output {
            status,
            stdout,
            stderr,
        },
        forced,
    )
}
struct OwnedWorker(Child);
#[tokio::test]
async fn canonical_mission_progress_drives_planning_episode_journals() {
    let f = Fixture::new().await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    mailbox.execute("assembled-lead",mail(f.epoch,"capture-planning-driver",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":null,"purpose":"planning","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-lead", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute("assembled-lead",mail(f.epoch,"admit-planning-driver",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":null,"purpose":"planning","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
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
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-owned-os".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: f.root.join("planning-runtime").to_str().unwrap().into(),
        endpoint: "owned-local-subprocess".into(),
        environment: "bwrap-private-git".into(),
    };
    episode_runtime::EpisodeGateway::new(f.broker.clone())
        .prepare(
            "assembled-lead",
            episode_runtime::StartRequest {
                id: Uuid::new_v4(),
                episode,
                execution: execution.id,
                runtime: identity,
                argv: vec!["/bin/sh".into(), "-c".into(), "cat SPEC.md".into()],
            },
        )
        .await
        .unwrap();
    let bridge = Bridge::new(f.broker.clone());
    let workflow = bridge
        .register(&f.owner, AggregateKind::Mission, f.mission, 0)
        .await
        .unwrap();
    let progress = bridge.progress(&f.owner, &workflow).await.unwrap();
    supervisor
        .cancel("assembled-lead", execution.id)
        .await
        .unwrap();
    supervisor
        .reconcile("assembled-lead", execution.id)
        .await
        .unwrap();
    supervisor
        .release("assembled-lead", execution.id)
        .await
        .unwrap();
    f.finish().await;
    assert!(
        progress
            .waits
            .contains(&temporal::WaitRef::episode(episode)),
        "canonical Mission workflow must expose its real planning episode journal to registered OperationDriver"
    );
}
#[tokio::test]
async fn public_episode_native_start_journal_has_no_synthetic_admission() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("episode-native").await;
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
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-owned-os".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: f.root.join("episode-runtime").to_str().unwrap().into(),
        endpoint: "owned-local-subprocess".into(),
        environment: "bwrap-private-git".into(),
    };
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let counts = |row: tokio_postgres::Row| {
        (
            row.get::<_, i64>(0),
            row.get::<_, i64>(1),
            row.get::<_, i64>(2),
        )
    };
    let before=counts(db.query_one("SELECT (SELECT count(*) FROM tasks),(SELECT count(*) FROM attempts),(SELECT count(*) FROM operations)",&[]).await.unwrap());
    let request = serde_json::json!({"id":Uuid::new_v4(),"episode":episode,"execution":execution.id,"runtime":identity,"argv":["/bin/sh","-c","cat SPEC.md"]});
    let dsn = f.dsn.clone();
    let epoch = f.epoch;
    let request = request.to_string();
    let output = tokio::task::spawn_blocking(move || {
        Process::new(env!("CARGO_BIN_EXE_team-control-plane"))
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("TEAM_CONTROL_PLANE_DSN", dsn)
            .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", epoch.to_string())
            .env("TEAM_CONTROL_PLANE_TOKEN", "assembled-worker")
            .args(["prepare-episode-start", &request])
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    let after=counts(db.query_one("SELECT (SELECT count(*) FROM tasks),(SELECT count(*) FROM attempts),(SELECT count(*) FROM operations)",&[]).await.unwrap());
    supervisor
        .cancel("assembled-worker", execution.id)
        .await
        .unwrap();
    supervisor
        .reconcile("assembled-worker", execution.id)
        .await
        .unwrap();
    supervisor
        .release("assembled-worker", execution.id)
        .await
        .unwrap();
    f.finish().await;
    assert!(
        output.status.success(),
        "authenticated episode-native preparation must exist without inventing a core admission: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let journal: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(journal["episode"], episode.to_string());
    assert_eq!(journal["execution"], execution.id.to_string());
    assert_eq!(journal["state"], "prepared");
    assert_eq!(
        before, after,
        "episode preparation creates no synthetic Task, Attempt or core admission"
    );
    assert_eq!(after, (1, 1, 1));
}
#[tokio::test]
async fn missing_assembled_verification_policy_denies_allocation_before_filesystem_effects() {
    let f = Fixture::without_requirement().await;
    let episode = f.worker_episode("missing-policy").await;
    let supervisor = f.supervisor().await;
    let before: Vec<_> = std::fs::read_dir(f.root.join("workspaces"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let result = supervisor
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await;
    let after: Vec<_> = std::fs::read_dir(f.root.join("workspaces"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let rows: i64 = db
        .query_one("SELECT count(*) FROM episode_executions", &[])
        .await
        .unwrap()
        .get(0);
    f.finish().await;
    assert_eq!(result.unwrap_err().code, ErrorCode::DependencyUnmet);
    assert_eq!(before, after);
    assert_eq!(rows, 0, "denied allocation registers no physical execution");
}
#[tokio::test]
async fn execution_launch_does_not_extend_the_canonical_episode_deadline() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("deadline").await;
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
            vec!["/bin/true".into()],
            30,
            supervisor::StartFault::BeforeMonitor,
        )
        .await
        .unwrap();
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let bounded:bool=db.query_one("SELECT w.deadline<=e.deadline FROM supervised_workspaces w JOIN episode_executions x ON x.execution_id=w.id JOIN mailbox_episodes e ON e.id=x.episode_id WHERE w.id=$1", &[&execution.id]).await.unwrap().get(0);
    // Explicit pre-monitor fault spawned no process; read-only SQL observes the public claim.
    f.finish().await;
    assert!(
        bounded,
        "physical launch must not restart the admitted episode's deadline budget"
    );
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trusted_exit_result_rejects_misleading_stdout_success_marker() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("exit-status").await;
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
        "printf 'gate_exit=0\\n'; exit 17".into(),
    ];
    supervisor
        .start("assembled-worker", execution.id, argv.clone(), 30)
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    loop {
        let view = supervisor
            .inspect("assembled-worker", execution.id)
            .await
            .unwrap();
        if view.state == supervisor::ExecutionState::Stopped {
            break;
        }
        assert_ne!(view.state, supervisor::ExecutionState::Quarantined);
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let result = supervisor
        .result("assembled-worker", execution.id)
        .await
        .unwrap();
    supervisor
        .release("assembled-worker", execution.id)
        .await
        .unwrap();
    f.finish().await;
    let result =
        result.expect("trusted monitor must retain actual wait status, independently of stdout");
    assert_eq!(result.exit_code, Some(17));
    assert_eq!(result.signal, None);
    assert_eq!(result.argv, argv);
    assert!(result.namespace_stopped);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delayed_monitor_cannot_borrow_another_live_grant_after_issuer_expiry() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("expiry").await;
    let supervisor = f.supervisor().await;
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "short-worker-grant",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: f.worker,
                    role: Role::Worker,
                    mission_id: Some(f.mission),
                    task_id: Some(f.task),
                    generation: Some(1),
                    token_hash: token_hash("short-issuer"),
                    expires_in_seconds: 10,
                },
            ),
        )
        .await
        .unwrap();
    let execution = supervisor
        .create_episode(
            "short-issuer",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    supervisor
        .start_with_fault(
            "short-issuer",
            execution.id,
            vec![
                "/bin/sh".into(),
                "-c".into(),
                "printf forbidden-effect > src/result".into(),
            ],
            30,
            supervisor::StartFault::BeforeMonitor,
        )
        .await
        .unwrap();
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let started: serde_json::Value = db.query_one("SELECT jsonb_build_object('observed_at',clock_timestamp(),'expires_at',expires_at,'issuer_live',expires_at>clock_timestamp()) FROM credentials WHERE token_hash=$1", &[&token_hash("short-issuer")]).await.unwrap().get(0);
    assert_eq!(
        started["issuer_live"], true,
        "setup must finish while the exact issuing credential is live"
    );
    let expiry_deadline = std::time::Instant::now() + Duration::from_secs(12);
    loop {
        let expired: serde_json::Value = db.query_one("SELECT jsonb_build_object('observed_at',clock_timestamp(),'expires_at',expires_at,'issuer_expired',expires_at<=clock_timestamp()) FROM credentials WHERE token_hash=$1", &[&token_hash("short-issuer")]).await.unwrap().get(0);
        if expired["issuer_expired"] == true {
            eprintln!("[issuer-expiry] start={started}, expiry={expired}");
            break;
        }
        assert!(std::time::Instant::now() < expiry_deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let result = supervisor::run_monitor(f.broker.clone(), execution.id).await;
    let effect = execution.path.join("src/result").exists();
    // This controlled pre-monitor fault has no unobserved child; if the buggy
    // monitor did launch, its returned completion has independently proven Stop.
    f.finish().await;
    assert!(
        !effect,
        "expired issuing grant must fence monitor effects even when another same-principal grant remains live"
    );
    assert_eq!(result.unwrap_err().code, ErrorCode::DependencyUnmet);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopped_capsule_with_unknown_runtime_effect_keeps_capacity() {
    legacy_usage_reservation_fixture(true).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_confirmed_start_without_measured_usage_retains_reservation() {
    legacy_usage_reservation_fixture(false).await;
}
async fn legacy_usage_reservation_fixture(drop_reply: bool) -> bool {
    let f = Fixture::new().await;
    let episode = f.worker_episode("unknown-start").await;
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
    let record = mailbox
        .record("assembled-worker", mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    let input_id: Uuid = serde_json::from_value(record["input_packet"].clone()).unwrap();
    let input = mailbox
        .record("assembled-worker", mailbox::RecordKind::Input, input_id)
        .await
        .unwrap();
    let root = f.root.join("unknown-runtime");
    std::fs::create_dir_all(&root).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-owned-os".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "owned-local-subprocess".into(),
        environment: "bwrap-private-git".into(),
    };
    let gateway = Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(&f.owner, binding, f.admission, &identity, None, 1)
        .await
        .unwrap();
    let mut runtime = SupervisedRuntime {
        root: root.clone(),
        identity,
        dsn: f.dsn.clone(),
        epoch: f.epoch,
        bearer: f.owner.clone(),
        workspaces: f.root.join("workspaces"),
        drop_start_reply: drop_reply,
        unmeasured_start: false,
    };
    let create = Uuid::new_v4();
    gateway
        .prepare(
            &f.owner,
            Intent {
                id: create,
                admission_id: f.admission,
                binding_id: binding,
                generation: 1,
                kind: OperationKind::Create,
                arguments: serde_json::json!({}),
            },
        )
        .await
        .unwrap();
    gateway
        .dispatch(&f.owner, create, &mut runtime)
        .await
        .unwrap();
    let start = Uuid::new_v4();
    gateway.prepare(&f.owner,Intent{id:start,admission_id:f.admission,binding_id:binding,generation:1,kind:OperationKind::Start,arguments:serde_json::json!({"execution":execution.id,"argv":["/bin/sh","-c","printf actual-effect > src/result"],"input_packet":input_id,"input_digest":input["digest"],"input":input["packet"]})}).await.unwrap();
    assert_eq!(
        gateway
            .dispatch(&f.owner, start, &mut runtime)
            .await
            .unwrap()
            .state,
        if drop_reply {
            OperationState::OutcomeUnknown
        } else {
            OperationState::Confirmed
        }
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    loop {
        let view = supervisor
            .inspect("assembled-worker", execution.id)
            .await
            .unwrap();
        if view.state == supervisor::ExecutionState::Stopped {
            break;
        }
        assert_ne!(view.state, supervisor::ExecutionState::Quarantined);
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let result = supervisor.release("assembled-worker", execution.id).await;
    let capacity = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data["used_capacity"]
        .clone();
    let retained = execution.path.exists();
    if !drop_reply {
        let observed = Gateway::new(f.broker.clone())
            .reconcile_usage(&f.owner, start)
            .await
            .unwrap();
        assert_eq!(
            observed.usage,
            episode_runtime::UsageObservation::Unknown,
            "unregistered actual adapter receipt cannot attest zero"
        );
    }
    let kernel = kernel::Kernel::new(f.broker.clone());
    let owner_status = kernel.status(&f.owner).await.unwrap();
    let worker_status = kernel.status("assembled-worker").await.unwrap();
    let verifier_status = kernel.status("assembled-verifier").await.unwrap();
    assert_eq!(
        kernel.status("wrong-status-token").await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let broker = f.broker.clone();
    let server =
        tokio::spawn(async move { axum::serve(listener, http::router(broker)).await.unwrap() });
    let client = reqwest::Client::new();
    let http_status: serde_json::Value = client
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
    assert_eq!(
        client
            .get(format!("http://{address}/v1/kernel/status"))
            .bearer_auth("wrong-status-token")
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    server.abort();
    assert!(server.await.unwrap_err().is_cancelled());
    let cli = Process::new(env!("CARGO_BIN_EXE_team-control-plane"))
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TEAM_CONTROL_PLANE_DSN", &f.dsn)
        .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", f.epoch.to_string())
        .env("TEAM_CONTROL_PLANE_TOKEN", &f.owner)
        .arg("status")
        .output()
        .unwrap();
    assert!(cli.status.success());
    assert!(cli.stderr.is_empty());
    let cli_status: serde_json::Value = serde_json::from_slice(&cli.stdout).unwrap();
    let visible = [&owner_status, &worker_status, &http_status, &cli_status]
        .into_iter()
        .all(|status| {
            [create, start].into_iter().all(|id| {
                status["unresolved_operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|op| {
                        op["id"] == id.to_string()
                            && op["protocol"] == "legacy_gateway"
                            && op["mission"] == f.mission.to_string()
                            && op["task"] == f.task.to_string()
                            && op["attempt"] == f.attempt.to_string()
                            && op["capacity_held"] == true
                            && op["usage"]["kind"] == "unknown"
                    })
            })
        });
    assert!(
        verifier_status["unresolved_operations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    for status in [&owner_status, &worker_status, &http_status, &cli_status] {
        let public = serde_json::to_string(status).unwrap();
        for secret in [
            "arguments",
            "runtime_identity",
            "token_hash",
            "thread_store",
            f.root.to_str().unwrap(),
        ] {
            assert!(!public.contains(secret));
        }
    }
    let out = evidence_directory(if drop_reply {
        "legacy-status-unknown-effect"
    } else {
        "legacy-status-unknown-usage"
    });
    retain_bytes(&out.join("public-status.json"),&serde_json::to_vec(&serde_json::json!({"create":create,"start":start,"owner":owner_status,"worker":worker_status,"verifier":verifier_status,"http":http_status,"cli":cli_status,"capacity":capacity,"actual_workload":std::fs::read(execution.path.join("src/result")).unwrap(),"start_state":gateway.read(&f.owner,start).await.unwrap().state})).unwrap());
    // Disposable fixture teardown after physical Stopped does not represent a product refund.
    f.finish().await;
    assert_eq!(
        result.unwrap_err().code,
        ErrorCode::DependencyUnmet,
        "unknown effect or usage must block release before deletion"
    );
    assert!(retained);
    assert_eq!(capacity, 1);
    visible
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_status_discovers_genuine_legacy_unknown_effect_and_unknown_usage() {
    let unknown_effect = legacy_usage_reservation_fixture(true).await;
    let unknown_usage = legacy_usage_reservation_fixture(false).await;
    assert!(
        unknown_effect && unknown_usage,
        "CLI/HTTP status must disclose genuine unresolved legacy operations and their held capacity"
    );
}
#[tokio::test]
async fn public_paused_gateway_claim_performs_no_runtime_rpc() {
    let f = Fixture::new().await;
    let root = f.root.join("paused-runtime");
    std::fs::create_dir_all(&root).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-owned-os".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "owned-local-subprocess".into(),
        environment: "bwrap-private-git".into(),
    };
    let gateway = Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    let operation = Uuid::new_v4();
    gateway
        .bind(&f.owner, binding, f.admission, &identity, None, 1)
        .await
        .unwrap();
    gateway
        .prepare(
            &f.owner,
            Intent {
                id: operation,
                admission_id: f.admission,
                binding_id: binding,
                generation: 1,
                kind: OperationKind::Create,
                arguments: serde_json::json!({}),
            },
        )
        .await
        .unwrap();
    kernel::Kernel::new(f.broker.clone())
        .pause(
            &f.owner,
            f.mission,
            "pause-before-rpc",
            0,
            true,
            "no new physical claims",
        )
        .await
        .unwrap();
    let mut runtime = SupervisedRuntime {
        root: root.clone(),
        identity,
        dsn: f.dsn.clone(),
        epoch: f.epoch,
        bearer: f.owner.clone(),
        workspaces: f.root.join("workspaces"),
        drop_start_reply: false,
        unmeasured_start: false,
    };
    let result = gateway.dispatch(&f.owner, operation, &mut runtime).await;
    let invocations = std::fs::read_dir(&root)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|x| x == "invocation")
        })
        .count();
    let state = gateway.read(&f.owner, operation).await.unwrap().state;
    f.finish().await;
    assert_eq!(
        invocations, 0,
        "paused mission must fence direct public Gateway RPC in its authoritative claim transaction"
    );
    assert_eq!(result.unwrap_err().code, ErrorCode::DependencyUnmet);
    assert_eq!(state, OperationState::Prepared);
}
impl OwnedWorker {
    fn kill(&mut self) {
        use std::os::unix::process::ExitStatusExt;
        assert!(
            self.0.try_wait().unwrap().is_none(),
            "worker must still be alive before fault"
        );
        self.0.kill().unwrap();
        assert_eq!(self.0.wait().unwrap().signal(), Some(9));
    }
}
impl Drop for OwnedWorker {
    fn drop(&mut self) {
        if self.0.try_wait().unwrap().is_none() {
            self.0.kill().unwrap();
            self.0.wait().unwrap();
        }
    }
}
#[tokio::test]
#[ignore = "owned episode SDK subprocess; parent fixture controls lifetime"]
async fn assembled_episode_sdk_worker() {
    use temporalio_client::{
        Client, ClientOptions, Connection, envconfig::LoadClientConfigProfileOptions,
    };
    use temporalio_sdk::{Runtime, Worker, WorkerOptions};
    let dsn = std::env::var("ASSEMBLED_DSN").unwrap();
    let epoch = std::env::var("ASSEMBLED_EPOCH").unwrap().parse().unwrap();
    let bearer = std::env::var("ASSEMBLED_BEARER").unwrap();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let root = PathBuf::from(std::env::var("ASSEMBLED_RUNTIME_ROOT").unwrap());
    let principal = std::env::var("ASSEMBLED_PRINCIPAL")
        .unwrap()
        .parse()
        .unwrap();
    let supervisor = supervisor::Supervisor::open(
        broker.clone(),
        &PathBuf::from(std::env::var("ASSEMBLED_WORKSPACES").unwrap()),
        env!("CARGO_BIN_EXE_team-control-plane"),
    )
    .await
    .unwrap();
    let native = episode_native::NativeOsRuntime {
        root: root.clone(),
        identity: serde_json::from_str(&std::env::var("ASSEMBLED_IDENTITY").unwrap()).unwrap(),
        supervisor,
        service: bearer.clone(),
        lose_reply: false,
        hold_reply: std::env::var("ASSEMBLED_HOLD_REPLY").ok().as_deref() == Some("1"),
    };
    let episode_gateway = episode_runtime::EpisodeGateway::new(broker.clone());
    episode_gateway
        .register_runtime(&bearer, &native)
        .await
        .unwrap();
    let controller = episode_gateway.controller();
    let driver = std::sync::Arc::new(episode_runtime::EpisodeOperationDriver::new(
        episode_gateway,
        bearer.clone(),
        if std::env::var("ASSEMBLED_DROP_USAGE").ok().as_deref() == Some("1") {
            Box::new(UnmeasuredNativeReply { native })
        } else {
            Box::new(native)
        },
        None,
    ));
    let activities = BridgeActivities::new(
        Bridge::new(broker),
        bearer.clone(),
        HashMap::from([(principal, bearer)]),
    )
    .with_operation_driver(driver);
    std::fs::write(
        root.parent()
            .unwrap()
            .join(format!("episode-controller-{controller}.ready")),
        controller.to_string(),
    )
    .unwrap();
    std::fs::File::open(
        root.parent()
            .unwrap()
            .join(format!("episode-controller-{controller}.ready")),
    )
    .unwrap()
    .sync_all()
    .unwrap();
    std::fs::File::open(&root).unwrap().sync_all().unwrap();
    let runtime = Runtime::from_current_tokio(Default::default()).unwrap();
    let (conn, options) =
        ClientOptions::load_from_config(LoadClientConfigProfileOptions::default()).unwrap();
    let client = Client::new(Connection::connect(conn).await.unwrap(), options).unwrap();
    // Finite crash fixture retains sticky routing with 500 ms dead-worker eviction.
    // SDK default is ten seconds: no default six-second recovery SLA is claimed.
    let options = WorkerOptions::new(std::env::var("ASSEMBLED_QUEUE").unwrap())
        .sticky_queue_schedule_to_start_timeout(Duration::from_millis(500))
        .register_workflow::<MissionWorkflow>()
        .unwrap()
        .register_workflow::<TaskWorkflow>()
        .unwrap()
        .register_activities(activities)
        .build();
    Worker::new(&runtime, client, options)
        .unwrap()
        .run()
        .await
        .unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_sdk_planning_episode_dispatches_its_own_capsule() {
    planning_sdk_fixture(false, false, None).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_takeover_transfers_only_to_registered_live_sdk_transport() {
    planning_sdk_fixture(true, false, None).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_takeover_reconciles_actual_sdk_crash_after_effect_without_second_start() {
    planning_sdk_fixture(true, true, None).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_takeover_rejects_native_fifo_without_blocking_or_transferring_authority() {
    planning_sdk_fixture(true, false, Some("fifo")).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_operator_reconciles_registered_live_native_operation_without_start_or_transfer() {
    planning_sdk_fixture(false, true, Some("operator-query")).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_takeover_refuses_unknown_inventory_and_unproven_processes() {
    for case in [
        "foreign",
        "missing",
        "conflict",
        "symlink",
        "changed-root",
        "unfinished",
        "foreign-group",
        "old-live",
        "target-dead",
    ] {
        println!("begin bounded takeover case {case}");
        planning_sdk_fixture(true, true, Some(case)).await;
        println!("completed bounded takeover case {case}");
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registered_sdk_usage_query_requires_exact_measured_receipt() {
    planning_sdk_fixture(false, false, Some("usage-query")).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registered_takeover_refuses_actual_running_unfinished_workload() {
    planning_sdk_fixture(true, true, Some("unfinished")).await;
}

async fn planning_sdk_fixture(takeover: bool, unknown_start: bool, negative: Option<&str>) {
    use temporalio_client::{
        Client, ClientOptions, Connection, envconfig::LoadClientConfigProfileOptions,
    };
    let out = evidence_directory(negative.unwrap_or(if unknown_start {
        "crash-takeover"
    } else {
        "planning"
    }));
    let f = Fixture::new().await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    mailbox.execute("assembled-lead",mail(f.epoch,"sdk-capture-planning",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":null,"purpose":"planning","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-lead", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute("assembled-lead",mail(f.epoch,"sdk-admit-planning",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":null,"purpose":"planning","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
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
    let root = f.root.join("episode-sdk-native");
    std::fs::create_dir_all(&root).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-inprocess-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "owned-async-os".into(),
        environment: "bwrap-private-git".into(),
    };
    let operation = Uuid::new_v4();
    episode_runtime::EpisodeGateway::new(f.broker.clone()).prepare("assembled-lead",episode_runtime::StartRequest{id:operation,episode,execution:execution.id,runtime:identity.clone(),argv:vec!["/bin/sh".into(),"-c".into(),"python3 -c 'import json; p=json.load(open(\"/episode-input.json\")); assert p[\"role\"]==\"lead\" and p[\"task\"] is None and p[\"attempt\"] is None'; status=$?; if test \"$status\" != 0; then exit \"$status\"; fi; cat SPEC.md; printf '\\nactual-planning-capsule\\n'".into()]}).await.unwrap();
    let bridge = Bridge::new(f.broker.clone());
    let workflow = bridge
        .register(&f.owner, AggregateKind::Mission, f.mission, 0)
        .await
        .unwrap();
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let principal: Uuid = db
        .query_one(
            "SELECT principal_id FROM credentials WHERE token_hash=$1",
            &[&token_hash(&f.owner)],
        )
        .await
        .unwrap()
        .get(0);
    let queue = format!("assembled-episode16-{}", f.epoch);
    let log = std::fs::File::create(f.root.join("episode-sdk-worker.log")).unwrap();
    let spawn = |hold: bool| {
        Process::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "assembled_episode_sdk_worker",
                "--ignored",
                "--nocapture",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env(
                "TEMPORAL_ADDRESS",
                std::env::var("TEMPORAL_ADDRESS").unwrap(),
            )
            .env("ASSEMBLED_DSN", &f.dsn)
            .env("ASSEMBLED_EPOCH", f.epoch.to_string())
            .env("ASSEMBLED_BEARER", &f.owner)
            .env("ASSEMBLED_PRINCIPAL", principal.to_string())
            .env("ASSEMBLED_QUEUE", &queue)
            .env("ASSEMBLED_RUNTIME_ROOT", &root)
            .env(
                "ASSEMBLED_IDENTITY",
                serde_json::to_string(&identity).unwrap(),
            )
            .env("ASSEMBLED_WORKSPACES", f.root.join("workspaces"))
            .env("ASSEMBLED_HOLD_REPLY", if hold { "1" } else { "0" })
            .env(
                "ASSEMBLED_DROP_USAGE",
                if negative == Some("usage-query") {
                    "1"
                } else {
                    "0"
                },
            )
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log.try_clone().unwrap()))
            .process_group(0)
            .spawn()
            .unwrap()
    };
    let mut process = OwnedWorker(spawn(unknown_start));
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !std::fs::read_dir(root.parent().unwrap()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("episode-controller-")
    }) {
        assert!(
            process.0.try_wait().unwrap().is_none(),
            "SDK worker prematurely exited; log retained"
        );
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let registration=db.query_one("SELECT controller,process_identity,group_id,native_directory,workspace_directory FROM kernel_transports WHERE process_identity->>'pid'=$1", &[&process.0.id().to_string()]).await.unwrap();
    let old_controller: Uuid = registration.get(0);
    let recorded: serde_json::Value = registration.get(1);
    assert_eq!(recorded["pid"], process.0.id());
    assert_eq!(registration.get::<_, i32>(2) as u32, process.0.id());
    assert!(
        registration.get::<_, serde_json::Value>(3)["inode"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(
        registration.get::<_, serde_json::Value>(4)["inode"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(
        std::fs::read_dir(&root).unwrap().count(),
        0,
        "transport registration precedes every native invocation/effect"
    );
    let (conn, options) =
        ClientOptions::load_from_config(LoadClientConfigProfileOptions::default()).unwrap();
    let client = Client::new(Connection::connect(conn).await.unwrap(), options).unwrap();
    let wake = bridge
        .pending_wakeups(&f.owner, &workflow)
        .await
        .unwrap()
        .remove(0);
    bridge
        .deliver(&f.owner, &client, &queue, &wake)
        .await
        .unwrap();
    let first = bridge
        .observe_execution(&f.owner, &workflow, &client, None)
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut result = None;
    while std::time::Instant::now() < deadline {
        assert!(
            process.0.try_wait().unwrap().is_none(),
            "SDK worker exited during planning dispatch"
        );
        if let Some(observed) = supervisor
            .result("assembled-lead", execution.id)
            .await
            .unwrap()
        {
            result = Some(observed);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    if unknown_start {
        let state: String = db
            .query_one(
                "SELECT state FROM episode_runtime_operations WHERE id=$1",
                &[&operation],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(
            state, "dispatching",
            "actual Start effect exists while its Activity reply is deliberately held before SDK SIGKILL"
        );
    }
    if matches!(negative, Some("operator-query" | "usage-query")) {
        let usage_query = negative == Some("usage-query");
        if usage_query {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                let row = db
                    .query_one(
                        "SELECT state,usage FROM episode_runtime_operations WHERE id=$1",
                        &[&operation],
                    )
                    .await
                    .unwrap();
                if row.get::<_, String>(0) == "confirmed" {
                    assert_eq!(
                        row.get::<_, serde_json::Value>(1),
                        serde_json::json!({"kind":"unknown"})
                    );
                    break;
                }
                assert!(std::time::Instant::now() < deadline);
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
        retain_native(&root, &out, "before-query");
        retain_bytes(
            &out.join("query-original-sdk.log"),
            &std::fs::read(f.root.join("episode-sdk-worker.log")).unwrap(),
        );
        kernel::Kernel::new(f.broker.clone())
            .pause(
                &f.owner,
                f.mission,
                "query-paused",
                0,
                true,
                "query existing uncertain native effect without dispatch",
            )
            .await
            .unwrap();
        let query = |token: &str| {
            let mut command = Process::new(env!("CARGO_BIN_EXE_team-control-plane"));
            command
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("TEAM_CONTROL_PLANE_DSN", &f.dsn)
                .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", f.epoch.to_string())
                .env("TEAM_CONTROL_PLANE_TOKEN", token)
                .args(["reconcile-episode-operation", &operation.to_string()]);
            bounded_operator_cli(command)
        };
        let effect = std::fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|x| x == "effect"))
            .unwrap();
        let original = std::fs::read(&effect).unwrap();
        let withheld = f.root.join("query-withheld-effect.json");
        let cases = if usage_query {
            vec![
                "missing",
                "corrupt",
                "symlink",
                "wrong-binding",
                "unsupported-profile",
            ]
        } else {
            vec!["missing", "corrupt", "symlink"]
        };
        for case in cases {
            std::fs::rename(&effect, &withheld).unwrap();
            if case == "corrupt" {
                std::fs::write(&effect, b"malformed native effect").unwrap();
            }
            if case == "symlink" {
                std::os::unix::fs::symlink(&withheld, &effect).unwrap();
            }
            if matches!(case, "wrong-binding" | "unsupported-profile") {
                let mut tampered: serde_json::Value = serde_json::from_slice(&original).unwrap();
                if case == "wrong-binding" {
                    tampered["execution"] = serde_json::json!(Uuid::new_v4());
                } else {
                    tampered["usage"]["profile"] = serde_json::json!("unmeasured-provider");
                }
                std::fs::write(&effect, serde_json::to_vec(&tampered).unwrap()).unwrap();
            }
            let (observed, timeout) = query(&f.owner).await;
            retain_bytes(&out.join(format!("query-{case}.stdout")), &observed.stdout);
            retain_bytes(&out.join(format!("query-{case}.stderr")), &observed.stderr);
            assert!(!timeout);
            assert!(observed.status.success());
            let observed: serde_json::Value = serde_json::from_slice(&observed.stdout).unwrap();
            assert_eq!(
                observed["state"],
                if usage_query {
                    "confirmed"
                } else {
                    "dispatching"
                },
                "{case} receipt cannot resolve native effect or usage"
            );
            if usage_query {
                assert_eq!(
                    observed["usage"]["kind"], "unknown",
                    "{case} must not attest measured zero"
                );
            }
            assert_eq!(
                supervisor
                    .release("assembled-lead", execution.id)
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
                2
            );
            if case != "missing" {
                std::fs::remove_file(&effect).unwrap();
            }
            std::fs::rename(&withheld, &effect).unwrap();
            assert_eq!(std::fs::read(&effect).unwrap(), original);
        }
        for token in ["wrong-token", "assembled-lead"] {
            let (denied, timeout) = query(token).await;
            assert!(!timeout);
            assert!(!denied.status.success());
            assert!(denied.stdout.is_empty());
        }
        let (output, timed_out) = query(&f.owner).await;
        retain_bytes(&out.join("query-cli.stdout"), &output.stdout);
        retain_bytes(&out.join("query-cli.stderr"), &output.stderr);
        assert!(!timed_out);
        assert!(
            output.status.success(),
            "public Owner query-only native reconciliation must exist: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let observed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(observed["state"], "confirmed");
        assert_eq!(observed["usage"]["kind"], "measured_zero");
        let evidence: i64 = db
            .query_one(
                "SELECT count(*) FROM kernel_usage_evidence WHERE operation_id=$1",
                &[&operation],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(
            evidence, 1,
            "one immutable observation records actual receipt provenance"
        );
        let unchanged:bool=db.query_one("SELECT s.controller=$1 AND s.busy IS NULL AND c.paused FROM gateway_stores s JOIN kernel_mission_control c ON c.mission_id=$2 WHERE s.identity=(SELECT store FROM kernel_transports WHERE controller=$1)", &[&old_controller,&f.mission]).await.unwrap().get(0);
        assert!(
            unchanged,
            "query observes existing effect, preserving controller and paused state"
        );
        assert_eq!(
            std::fs::read_dir(&root)
                .unwrap()
                .filter(|e| e
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|s| s == "invocation"))
                .count(),
            1
        );
        assert_eq!(
            std::fs::read_dir(&root)
                .unwrap()
                .filter(|e| e
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|s| s == "effect"))
                .count(),
            1
        );
    }
    let mut foreign = if negative == Some("foreign-group") {
        Some(OwnedWorker(
            Process::new("/bin/sleep")
                .arg("30")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .process_group(process.0.id() as i32)
                .spawn()
                .unwrap(),
        ))
    } else {
        None
    };
    if negative != Some("old-live") {
        process.kill();
    }
    if result.is_none() {
        supervisor
            .cancel("assembled-lead", execution.id)
            .await
            .unwrap();
        supervisor
            .reconcile("assembled-lead", execution.id)
            .await
            .unwrap();
    }
    let log = std::fs::read_to_string(f.root.join("episode-sdk-worker.log")).unwrap();
    let view = supervisor
        .inspect("assembled-lead", execution.id)
        .await
        .unwrap();
    let capsule_log = std::fs::read_to_string(&view.manifest.log_path).unwrap_or_default();
    let invocations = std::fs::read_dir(&root)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|x| x == "invocation")
        })
        .count();
    let effects = std::fs::read_dir(&root)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|x| x == "effect")
        })
        .count();
    retain_native(&root, &out, "original");
    retain_bytes(&out.join("original-sdk.log"), log.as_bytes());
    retain_bytes(
        &out.join("actual-execution-result.json"),
        &serde_json::to_vec(&result).unwrap(),
    );
    let mut takeover_result = None;
    let mut unfinished = None;
    if takeover {
        if negative == Some("unfinished") {
            let worker_episode = f.worker_episode("unfinished-before-takeover").await;
            unfinished = Some(
                supervisor
                    .create_episode(
                        "assembled-worker",
                        worker_episode,
                        vec![],
                        supervisor::CreateFault::None,
                    )
                    .await
                    .unwrap(),
            );
            let workspace = unfinished.as_ref().unwrap();
            supervisor
                .start(
                    "assembled-worker",
                    workspace.id,
                    vec![
                        "/bin/sh".into(),
                        "-c".into(),
                        "while :; do printf active-unfinished >> src/live; sleep 0.02; done".into(),
                    ],
                    30,
                )
                .await
                .unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                let view = supervisor.inspect(&f.owner, workspace.id).await.unwrap();
                if view.state == supervisor::ExecutionState::Running
                    && workspace.path.join("src/live").exists()
                {
                    retain_bytes(
                        &out.join("unfinished-running-inventory.json"),
                        &serde_json::to_vec(&view).unwrap(),
                    );
                    break;
                }
                assert!(std::time::Instant::now() < deadline);
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
        let paused = kernel::Kernel::new(f.broker.clone())
            .pause(
                &f.owner,
                f.mission,
                "takeover-pause",
                0,
                true,
                "prove old transport and complete inventory before transfer",
            )
            .await
            .unwrap();
        assert_eq!(paused["revision"], 1);
        let mut replacement = OwnedWorker(spawn(false));
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let replacement_controller = loop {
            assert!(
                replacement.0.try_wait().unwrap().is_none(),
                "replacement exited before registration"
            );
            if let Some(row) = db
                .query_opt(
                    "SELECT controller FROM kernel_transports WHERE process_identity->>'pid'=$1",
                    &[&replacement.0.id().to_string()],
                )
                .await
                .unwrap()
            {
                break row.get::<_, Uuid>(0);
            }
            assert!(std::time::Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        assert_ne!(old_controller, replacement_controller);
        let dsn = f.dsn.clone();
        let epoch = f.epoch;
        let owner = f.owner.clone();
        let operation = operation.to_string();
        let replacement_id = replacement_controller.to_string();
        let original_id = old_controller.to_string();
        let cli_operation = operation.clone();
        let original_effect = std::fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|x| x == "effect"))
            .unwrap();
        let original_record: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&original_effect).unwrap()).unwrap();
        let mut injected = None;
        let mut withheld = None;
        match negative {
            Some("fifo") => {
                use std::{ffi::CString, os::unix::ffi::OsStrExt};
                let path = root.join(format!("{}.effect", Uuid::new_v4()));
                let name = CString::new(path.as_os_str().as_bytes()).unwrap();
                assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
                injected = Some(path);
            }
            Some("foreign") => {
                let path = root.join(format!("{}.effect", Uuid::new_v4()));
                let mut record = original_record.clone();
                record["operation"] = serde_json::json!(Uuid::new_v4());
                std::fs::write(&path, serde_json::to_vec(&record).unwrap()).unwrap();
                injected = Some(path);
            }
            Some("missing") => {
                let path = f.root.join("withheld-native-effect");
                std::fs::rename(&original_effect, &path).unwrap();
                withheld = Some(path);
            }
            Some("conflict") => {
                let path = root.join(format!("{}.effect", Uuid::new_v4()));
                std::fs::copy(&original_effect, &path).unwrap();
                injected = Some(path);
            }
            Some("symlink") => {
                let path = root.join(format!("{}.effect", Uuid::new_v4()));
                std::os::unix::fs::symlink(&original_effect, &path).unwrap();
                injected = Some(path);
            }
            Some("changed-root") => {
                let held = f.root.join("held-original-native-root");
                std::fs::rename(&root, &held).unwrap();
                std::fs::create_dir(&root).unwrap();
            }
            Some("target-dead") => replacement.kill(),
            _ => {}
        }
        let mut command = Process::new(env!("CARGO_BIN_EXE_team-control-plane"));
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("TEAM_CONTROL_PLANE_DSN", dsn)
            .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", epoch.to_string())
            .env("TEAM_CONTROL_PLANE_TOKEN", owner)
            .args([
                "takeover-episode-store",
                &cli_operation,
                &replacement_id,
                &original_id,
                "takeover-once",
                "1",
            ]);
        let (output, forced) = bounded_operator_cli(command).await;
        if negative.is_some() {
            let actual:Uuid=db.query_one("SELECT controller FROM gateway_stores WHERE identity=(SELECT store FROM episode_runtime_operations WHERE id=$1)", &[&Uuid::parse_str(&operation).unwrap()]).await.unwrap().get(0);
            assert_eq!(
                actual, old_controller,
                "rejected inventory cannot transfer control"
            );
        }
        let first_rejected = negative.is_some();
        if first_rejected {
            assert!(
                !output.status.success(),
                "negative {negative:?} must not transfer"
            );
            if unknown_start {
                assert_eq!(
                    f.broker
                        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                        .await
                        .unwrap()
                        .data["used_capacity"],
                    2
                );
                assert_eq!(
                    supervisor
                        .release("assembled-lead", execution.id)
                        .await
                        .unwrap_err()
                        .code,
                    ErrorCode::DependencyUnmet
                );
            }
            if let Some(child) = &mut foreign {
                assert!(
                    child.0.try_wait().unwrap().is_none(),
                    "takeover never signals unrelated group members"
                );
                child.kill();
            }
            if negative == Some("old-live") {
                assert!(process.0.try_wait().unwrap().is_none());
                process.kill();
            }
            if let Some(path) = injected {
                std::fs::remove_file(path).unwrap();
            }
            if let Some(path) = withheld {
                std::fs::rename(path, &original_effect).unwrap();
            }
            if negative == Some("changed-root") {
                std::fs::remove_dir(&root).unwrap();
                std::fs::rename(f.root.join("held-original-native-root"), &root).unwrap();
            }
            if let Some(workspace) = &unfinished {
                supervisor.cancel(&f.owner, workspace.id).await.unwrap();
                supervisor.reconcile(&f.owner, workspace.id).await.unwrap();
                let before = std::fs::read(workspace.path.join("src/live")).unwrap();
                tokio::time::sleep(Duration::from_millis(120)).await;
                assert_eq!(
                    std::fs::read(workspace.path.join("src/live")).unwrap(),
                    before,
                    "safe recovery requires independently stopped unfinished writer"
                );
            }
            // Restore only the controlled fixture fault. Recovery still uses the
            // public independent inventory, never a manufactured DB verdict.
            if negative != Some("target-dead") {
                let recovered = kernel::Kernel::new(f.broker.clone())
                    .takeover(
                        &f.owner,
                        kernel::TakeoverRequest {
                            operation: Uuid::parse_str(&operation).unwrap(),
                            expected_controller: old_controller,
                            replacement: replacement_controller,
                            key: "recovery-after-negative".into(),
                            expected_control_revision: 1,
                        },
                    )
                    .await
                    .unwrap();
                assert_eq!(recovered["controller"], replacement_controller.to_string());
            }
        }
        if output.status.success() {
            let stored=db.query_one("SELECT s.controller,o.state,o.thread_id,o.turn_id,o.effect_id FROM episode_runtime_operations o JOIN gateway_stores s ON s.identity=o.store WHERE o.id=$1", &[&Uuid::parse_str(&operation).unwrap()]).await.unwrap();
            assert_eq!(stored.get::<_, Uuid>(0), replacement_controller);
            assert_eq!(stored.get::<_, String>(1), "confirmed");
            assert_eq!(
                stored.get::<_, Option<String>>(2).unwrap(),
                original_record["turn"]["thread"]
            );
            assert_eq!(
                stored.get::<_, Option<String>>(3).unwrap(),
                original_record["turn"]["turn"]
            );
            assert_eq!(
                stored.get::<_, Option<String>>(4).unwrap(),
                original_record["turn"]["effect"]
            );
            let evidence:bool=db.query_one("SELECT EXISTS(SELECT 1 FROM episode_runtime_evidence WHERE operation_id=$1 AND evidence->>'kind'='exclusive_takeover' AND evidence->'result'->>'controller'=$2)", &[&Uuid::parse_str(&operation).unwrap(),&replacement_controller.to_string()]).await.unwrap().get(0);
            assert!(
                evidence,
                "actual immutable transfer proof is retained independently of CLI JSON"
            );
            let episode_row = mailbox
                .record("assembled-lead", mailbox::RecordKind::Episode, episode)
                .await
                .unwrap();
            assert_eq!(
                episode_row["turn_id"],
                stored.get::<_, Option<String>>(3).unwrap()
            );
            // The registered replacement receives the existing workflow after strict
            // old-worker SIGKILL/wait. Adoption observes/replays; it cannot issue Start.
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                let observed:bool=db.query_one("SELECT EXISTS(SELECT 1 FROM kernel_transport_observations WHERE controller=$1 AND operation_id=$2 AND evidence->>'kind'='sdk_observed_existing_operation')", &[&replacement_controller,&Uuid::parse_str(&operation).unwrap()]).await.unwrap().get(0);
                if observed {
                    break;
                }
                assert!(replacement.0.try_wait().unwrap().is_none());
                assert!(
                    std::time::Instant::now() < deadline,
                    "replacement SDK must actually consume retained workflow state and query the same immutable operation"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            assert_eq!(
                std::fs::read_dir(&root)
                    .unwrap()
                    .filter(|e| e
                        .as_ref()
                        .unwrap()
                        .path()
                        .extension()
                        .is_some_and(|x| x == "invocation"))
                    .count(),
                1
            );
            assert_eq!(
                std::fs::read_dir(&root)
                    .unwrap()
                    .filter(|e| e
                        .as_ref()
                        .unwrap()
                        .path()
                        .extension()
                        .is_some_and(|x| x == "effect"))
                    .count(),
                1
            );
            let request = kernel::TakeoverRequest {
                operation: Uuid::parse_str(&operation).unwrap(),
                expected_controller: old_controller,
                replacement: replacement_controller,
                key: "takeover-once".into(),
                expected_control_revision: 1,
            };
            let cached = kernel::Kernel::new(f.broker.clone())
                .takeover(&f.owner, request.clone())
                .await
                .unwrap();
            assert_eq!(cached["controller"], replacement_controller.to_string());
            assert_eq!(
                kernel::Kernel::new(f.broker.clone())
                    .takeover("wrong-token", request.clone())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::Unauthorized
            );
            assert_eq!(
                kernel::Kernel::new(f.broker.clone())
                    .takeover("assembled-worker", request.clone())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::Unauthorized
            );
            let deadline = std::time::Instant::now() + Duration::from_secs(6);
            let continued = loop {
                let observed = bridge
                    .observe_execution(&f.owner, &workflow, &client, None)
                    .await
                    .unwrap();
                if observed.run_id != first.run_id {
                    break observed;
                }
                assert!(replacement.0.try_wait().unwrap().is_none());
                if std::time::Instant::now() >= deadline {
                    use temporalio_client::{WorkflowExecutionInfo, WorkflowFetchHistoryOptions};
                    let history = WorkflowExecutionInfo::builder()
                        .namespace("default")
                        .workflow_id(&workflow.workflow_id)
                        .run_id(&first.run_id)
                        .build()
                        .bind_untyped(client.clone())
                        .fetch_history(WorkflowFetchHistoryOptions::default())
                        .to_json()
                        .await
                        .unwrap();
                    retain_bytes(&out.join("can-timeout-history.json"), &history);
                    retain_bytes(
                        &out.join("can-timeout-sdk.log"),
                        &std::fs::read(f.root.join("episode-sdk-worker.log")).unwrap(),
                    );
                    let progress = bridge.progress(&f.owner, &workflow).await.unwrap();
                    retain_bytes(
                        &out.join("can-timeout-progress.json"),
                        &serde_json::to_vec(&progress).unwrap(),
                    );
                    retain_bytes(
                        &out.join("can-timeout-observed.json"),
                        &serde_json::to_vec(&observed).unwrap(),
                    );
                    retain_native(&root, &out, "can-timeout");
                    panic!(
                        "actual replacement workflow must continue as new without repeating native Start; failure snapshots retained at {}",
                        out.display()
                    );
                }
                tokio::time::sleep(Duration::from_millis(40)).await;
            };
            assert!(continued.incarnation > first.incarnation);
            let original_history = retain_replay(
                &client,
                &workflow,
                &first.run_id,
                &out,
                "original-run-history",
                &f.owner,
            )
            .await;
            assert!(
                String::from_utf8_lossy(&original_history)
                    .contains("WORKFLOW_EXECUTION_CONTINUED_AS_NEW")
                    || String::from_utf8_lossy(&original_history)
                        .contains("WorkflowExecutionContinuedAsNewEventAttributes")
            );
            retain_replay(
                &client,
                &workflow,
                &continued.run_id,
                &out,
                "continued-run-history",
                &f.owner,
            )
            .await;
            kernel::Kernel::new(f.broker.clone())
                .pause(
                    &f.owner,
                    f.mission,
                    "resume-after-adoption",
                    1,
                    false,
                    "actual replacement observed retained operation",
                )
                .await
                .unwrap();
            assert_eq!(
                kernel::Kernel::new(f.broker.clone())
                    .takeover(&f.owner, request.clone())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet,
                "cached transfer cannot report paused control after dispatch resumed"
            );
            kernel::Kernel::new(f.broker.clone())
                .pause(
                    &f.owner,
                    f.mission,
                    "repause-after-adoption",
                    2,
                    true,
                    "cleanup under explicit pause",
                )
                .await
                .unwrap();
            assert_eq!(
                kernel::Kernel::new(f.broker.clone())
                    .takeover(&f.owner, request)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::StaleRevision
            );
            retain_bytes(&out.join("transfer-cli.stdout"), &output.stdout);
            retain_bytes(&out.join("transfer-cli.stderr"), &output.stderr);
        }
        if replacement.0.try_wait().unwrap().is_none() {
            replacement.kill();
        }
        if negative == Some("target-dead") {
            // Exact stopped original and missing replacement leave the effect
            // unresolved. A new actual controller is explicitly registered once.
            let mut final_replacement = OwnedWorker(spawn(false));
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let target = loop {
                if let Some(row)=db.query_opt("SELECT controller FROM kernel_transports WHERE process_identity->>'pid'=$1", &[&final_replacement.0.id().to_string()]).await.unwrap(){break row.get::<_,Uuid>(0);}
                assert!(final_replacement.0.try_wait().unwrap().is_none());
                assert!(std::time::Instant::now() < deadline);
                tokio::time::sleep(Duration::from_millis(20)).await;
            };
            kernel::Kernel::new(f.broker.clone())
                .takeover(
                    &f.owner,
                    kernel::TakeoverRequest {
                        operation: Uuid::parse_str(&operation).unwrap(),
                        expected_controller: old_controller,
                        replacement: target,
                        key: "recovery-live-target".into(),
                        expected_control_revision: 1,
                    },
                )
                .await
                .unwrap();
            final_replacement.kill();
        }
        takeover_result = Some((output, replacement_controller, forced));
    }
    let observations:Vec<serde_json::Value>=db.query("SELECT jsonb_build_object('controller',controller,'store',store,'process',process_identity,'group',group_id,'native_directory',native_directory,'workspace_directory',workspace_directory,'epoch',recovery_epoch) FROM kernel_transports ORDER BY controller", &[]).await.unwrap().into_iter().map(|r|r.get(0)).collect();
    retain_bytes(
        &out.join("registered-transports.json"),
        &serde_json::to_vec(&observations).unwrap(),
    );
    let proofs:Vec<serde_json::Value>=db.query("SELECT evidence FROM episode_runtime_evidence WHERE operation_id=$1 ORDER BY recorded_at,id", &[&operation]).await.unwrap().into_iter().map(|r|r.get(0)).collect();
    retain_bytes(
        &out.join("operation-evidence.json"),
        &serde_json::to_vec(&proofs).unwrap(),
    );
    retain_native(&root, &out, "final");
    retain_bytes(
        &out.join("all-sdk.log"),
        &std::fs::read(f.root.join("episode-sdk-worker.log")).unwrap(),
    );
    if let Some(workspace) = unfinished {
        supervisor.release(&f.owner, workspace.id).await.unwrap();
    }
    supervisor
        .release("assembled-lead", execution.id)
        .await
        .unwrap();
    f.finish().await;
    if let Some((output, replacement, forced)) = takeover_result {
        assert!(
            !forced,
            "public takeover must classify nonregular native files without blocking the broker transaction"
        );
        if negative.is_some() {
            assert!(!output.status.success());
            retain_bytes(&out.join("takeover-error-stderr.bin"), &output.stderr);
            let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(error["code"], "dependency_unmet");
        } else {
            assert!(
                output.status.success(),
                "authenticated takeover after independent old SDK SIGKILL/wait and live registered replacement is required: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let transfer: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(transfer["controller"], replacement.to_string());
        }
    }
    let result=result.unwrap_or_else(||panic!("registered SDK driver must actually launch the canonical planning episode, operation {operation}; worker log {log}"));
    assert_eq!(
        result.exit_code,
        Some(0),
        "actual capsule log {capsule_log}"
    );
    assert!(result.namespace_stopped);
    assert_eq!((invocations, effects), (1, 1));
    assert_eq!(view.manifest.task, None);
    assert_eq!(view.manifest.attempt, None);
    assert!(view.manifest.write_scope.is_empty());
}
#[tokio::test]
#[ignore = "owned SDK subprocess; parent fixture controls lifetime"]
async fn assembled_sdk_worker() {
    use temporalio_client::{
        Client, ClientOptions, Connection, envconfig::LoadClientConfigProfileOptions,
    };
    use temporalio_sdk::{Runtime, Worker, WorkerOptions};
    let dsn = std::env::var("ASSEMBLED_DSN").unwrap();
    let epoch = std::env::var("ASSEMBLED_EPOCH").unwrap().parse().unwrap();
    let bearer = std::env::var("ASSEMBLED_BEARER").unwrap();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let bridge = Bridge::new(broker.clone());
    let gateway = Gateway::new(broker.clone());
    let identity: RuntimeIdentity =
        serde_json::from_str(&std::env::var("ASSEMBLED_IDENTITY").unwrap()).unwrap();
    let prepare: serde_json::Value =
        serde_json::from_str(&std::env::var("ASSEMBLED_PREPARE").unwrap()).unwrap();
    let admission = serde_json::from_value(prepare["admission"].clone()).unwrap();
    let binding = serde_json::from_value(prepare["binding"].clone()).unwrap();
    gateway
        .bind(&bearer, binding, admission, &identity, None, 1)
        .await
        .unwrap();
    gateway
        .prepare(
            &bearer,
            serde_json::from_value(prepare["create"].clone()).unwrap(),
        )
        .await
        .unwrap();
    let root = PathBuf::from(std::env::var("ASSEMBLED_RUNTIME_ROOT").unwrap());
    std::fs::write(
        root.join("controller-ready"),
        gateway.controller().to_string(),
    )
    .unwrap();
    std::fs::File::open(root.join("controller-ready"))
        .unwrap()
        .sync_all()
        .unwrap();
    std::fs::File::open(&root).unwrap().sync_all().unwrap();
    let runtime = SupervisedRuntime {
        root: PathBuf::from(std::env::var("ASSEMBLED_RUNTIME_ROOT").unwrap()),
        identity,
        dsn,
        epoch,
        bearer: bearer.clone(),
        workspaces: PathBuf::from(std::env::var("ASSEMBLED_WORKSPACES").unwrap()),
        drop_start_reply: false,
        unmeasured_start: std::env::var("ASSEMBLED_LEGACY_DROP_USAGE").ok().as_deref() == Some("1"),
    };
    gateway.register_runtime(&bearer, &runtime).await.unwrap();
    let principal = std::env::var("ASSEMBLED_PRINCIPAL")
        .unwrap()
        .parse()
        .unwrap();
    let fallback = std::sync::Arc::new(GatewayOperationDriver::new(
        bridge.clone(),
        gateway,
        bearer.clone(),
        Box::new(runtime),
    ));
    let supervisor = supervisor::Supervisor::open(
        broker.clone(),
        &PathBuf::from(std::env::var("ASSEMBLED_WORKSPACES").unwrap()),
        env!("CARGO_BIN_EXE_team-control-plane"),
    )
    .await
    .unwrap();
    let native = episode_native::NativeOsRuntime {
        root: PathBuf::from(std::env::var("ASSEMBLED_EPISODE_RUNTIME_ROOT").unwrap()),
        identity: serde_json::from_str(&std::env::var("ASSEMBLED_EPISODE_IDENTITY").unwrap())
            .unwrap(),
        supervisor,
        service: bearer.clone(),
        lose_reply: false,
        hold_reply: false,
    };
    let episode_gateway = episode_runtime::EpisodeGateway::new(broker);
    episode_gateway
        .register_runtime(&bearer, &native)
        .await
        .unwrap();
    let driver = std::sync::Arc::new(episode_runtime::EpisodeOperationDriver::new(
        episode_gateway,
        bearer.clone(),
        if std::env::var("ASSEMBLED_DROP_USAGE").ok().as_deref() == Some("1") {
            Box::new(UnmeasuredNativeReply { native })
        } else {
            Box::new(native)
        },
        Some(fallback),
    ));
    let activities =
        BridgeActivities::new(bridge, bearer.clone(), HashMap::from([(principal, bearer)]))
            .with_operation_driver(driver);
    let runtime = Runtime::from_current_tokio(Default::default()).unwrap();
    let (conn, options) =
        ClientOptions::load_from_config(LoadClientConfigProfileOptions::default()).unwrap();
    let client = Client::new(Connection::connect(conn).await.unwrap(), options).unwrap();
    let options = WorkerOptions::new(std::env::var("ASSEMBLED_QUEUE").unwrap())
        .register_workflow::<MissionWorkflow>()
        .unwrap()
        .register_workflow::<TaskWorkflow>()
        .unwrap()
        .register_activities(activities)
        .build();
    Worker::new(&runtime, client, options)
        .unwrap()
        .run()
        .await
        .unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_temporal_assembled_vertical() {
    assembled_engineering_fixture(EngineeringScenario::Complete).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_failed_verification_repairs_are_authorized_idempotent_and_bounded() {
    assembled_engineering_fixture(EngineeringScenario::RepairBudget).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_registered_legacy_usage_query_requires_exact_receipt() {
    assembled_engineering_fixture(EngineeringScenario::LegacyUsageReceipts).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn foreign_legacy_controller_cannot_attest_or_acquire_retained_store() {
    assembled_engineering_fixture(EngineeringScenario::ForeignLegacyController).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_legacy_usage_blocks_continuation_before_budget_charge() {
    assembled_engineering_fixture(EngineeringScenario::UnknownLegacyContinuation).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_legacy_usage_blocks_previously_admitted_verifier_launch() {
    assembled_engineering_fixture(EngineeringScenario::UnknownLegacyPhysicalClaim).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_legacy_usage_blocks_monitor_after_controller_claim() {
    assembled_engineering_fixture(EngineeringScenario::UnknownLegacyMonitorClaim).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_candidate_and_measured_report_integrity_blocks_acceptance() {
    assembled_engineering_fixture(EngineeringScenario::ArtifactIntegrity).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_changed_git_base_requires_new_candidate_and_independent_sdk_gate() {
    assembled_engineering_fixture(EngineeringScenario::ChangedBase).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_legacy_gateway_restart_safely_quarantines_without_another_start() {
    assembled_engineering_fixture(EngineeringScenario::LegacyGatewayRestart).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn postallocation_base_change_preserves_snapshot_but_requires_fresh_current_gate() {
    assembled_engineering_fixture(EngineeringScenario::PostAllocationBaseChange).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelled_attempt_cannot_submit_cached_report_or_accept_measured_candidate() {
    assembled_engineering_fixture(EngineeringScenario::CancelMeasuredCandidate).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn genuinely_accepted_task_revokes_cached_new_and_prepared_worker_dispatch() {
    assembled_engineering_fixture(EngineeringScenario::AcceptedTerminal).await;
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EngineeringScenario {
    Complete,
    CancelMeasuredCandidate,
    AcceptedTerminal,
    RepairBudget,
    FailedGateDispatch,
    ArtifactIntegrity,
    ChangedBase,
    PostAllocationBaseChange,
    LegacyGatewayRestart,
    LegacyUsageReceipts,
    ForeignLegacyController,
    UnknownLegacyContinuation,
    UnknownLegacyPhysicalClaim,
    UnknownLegacyMonitorClaim,
}
async fn assembled_engineering_fixture(scenario: EngineeringScenario) {
    let repair_budget = scenario == EngineeringScenario::RepairBudget;
    let usage_query = matches!(
        scenario,
        EngineeringScenario::LegacyUsageReceipts
            | EngineeringScenario::LegacyGatewayRestart
            | EngineeringScenario::ForeignLegacyController
            | EngineeringScenario::UnknownLegacyContinuation
            | EngineeringScenario::UnknownLegacyPhysicalClaim
            | EngineeringScenario::UnknownLegacyMonitorClaim
    );
    let foreign_controller = scenario == EngineeringScenario::ForeignLegacyController;
    use temporalio_client::{
        Client, ClientOptions, Connection, envconfig::LoadClientConfigProfileOptions,
    };
    let f = std::sync::Arc::new(Fixture::new().await);
    let bridge = Bridge::new(f.broker.clone());
    let r = bridge
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let episode = f.worker_episode("worker-first").await;
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
    // Allocate the independent readonly verifier before any native operation.
    // Its later physical claim must recheck usage, rather than borrow admission-time safety.
    let pending_verifier = if matches!(
        scenario,
        EngineeringScenario::UnknownLegacyPhysicalClaim
            | EngineeringScenario::UnknownLegacyMonitorClaim
    ) {
        let service = artifacts::ArtifactService::open(f.broker.clone(), &f.root.join("artifacts"))
            .await
            .unwrap();
        let command = vec![
            "/bin/sh".into(),
            "-c".into(),
            "printf 'unexpected-verifier-workload\\n'".into(),
        ];
        let envelope = |key: &str, revision, command| artifacts::ArtifactEnvelope {
            key: key.into(),
            task: f.task,
            expected_revision: revision,
            generation: 1,
            recovery_epoch: f.epoch,
            command,
        };
        let revision = f
            .broker
            .snapshot(&f.owner, SnapshotQuery::Task(f.task))
            .await
            .unwrap()
            .revision;
        let configured = service
            .execute(
                &f.owner,
                envelope(
                    "race-configure",
                    revision,
                    artifacts::ArtifactCommand::Configure {
                        verifier: f.verifier,
                        lead_acceptance: false,
                        base: f.base.clone(),
                        suite: artifacts::sha256(
                            &serde_json::to_vec(&vec![command.clone()]).unwrap(),
                        ),
                        environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                        mandatory_commands: vec![command.clone()],
                    },
                ),
                artifacts::ReferenceFault::None,
            )
            .await
            .unwrap();
        std::fs::write(
            execution.path.join("src/result"),
            b"real immutable pre-admitted candidate\n",
        )
        .unwrap();
        git(&execution.path, &["add", "src/result"]);
        git(
            &execution.path,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "race candidate",
            ],
        );
        let bundle = f.root.join("race-candidate.bundle");
        git(
            &execution.path,
            &["bundle", "create", bundle.to_str().unwrap(), "HEAD"],
        );
        let bytes = std::fs::read(bundle).unwrap();
        let digest = artifacts::sha256(&bytes);
        service
            .publish_evidence("assembled-worker", f.task, &digest, &bytes)
            .await
            .unwrap();
        let candidate = artifacts::Candidate {
            commit: git(&execution.path, &["rev-parse", "HEAD"]),
            tree: git(&execution.path, &["rev-parse", "HEAD^{tree}"]),
            base: f.base.clone(),
            spec_digest: f.spec.clone(),
            spec_revision: 2,
            generation: 1,
            artifact: digest,
            deliverables: vec!["src/result".into()],
            risks: vec![],
        };
        service
            .execute(
                "assembled-worker",
                envelope(
                    "race-submit",
                    configured.revision,
                    artifacts::ArtifactCommand::Submit { candidate },
                ),
                artifacts::ReferenceFault::None,
            )
            .await
            .unwrap();
        let mailbox = mailbox::MailboxService::new(f.broker.clone());
        let input = Uuid::new_v4();
        let verifier = Uuid::new_v4();
        mailbox.execute("assembled-verifier",mail(f.epoch,"race-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"verification","limit":4}))).await.unwrap();
        let packet = mailbox
            .record("assembled-verifier", mailbox::RecordKind::Input, input)
            .await
            .unwrap();
        mailbox.execute("assembled-verifier",mail(f.epoch,"race-admit",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":verifier,"attempt":f.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
        let handle = supervisor
            .create_episode_from_candidate(
                "assembled-verifier",
                verifier,
                &service,
                supervisor::CreateFault::None,
            )
            .await
            .unwrap();
        Some((handle, command))
    } else {
        None
    };
    let held_monitor = if scenario == EngineeringScenario::UnknownLegacyMonitorClaim {
        let (pending, command) = pending_verifier.as_ref().unwrap();
        supervisor
            .start_with_fault(
                "assembled-verifier",
                pending.id,
                command.clone(),
                30,
                supervisor::StartFault::BeforeMonitor,
            )
            .await
            .unwrap();
        let claimed = std::sync::Arc::new(tokio::sync::Notify::new());
        let resume = std::sync::Arc::new(tokio::sync::Notify::new());
        let handle = tokio::spawn(supervisor::run_monitor_with_gate(
            f.broker.clone(),
            pending.id,
            supervisor::MonitorGate {
                claimed: claimed.clone(),
                resume: resume.clone(),
            },
        ));
        tokio::time::timeout(Duration::from_secs(3), claimed.notified())
            .await
            .expect("monitor must durably claim controller before native effects");
        Some((resume, handle))
    } else {
        None
    };
    let episode_record = mailbox::MailboxService::new(f.broker.clone())
        .record("assembled-worker", mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    let input_id: Uuid = serde_json::from_value(episode_record["input_packet"].clone()).unwrap();
    let input = mailbox::MailboxService::new(f.broker.clone())
        .record("assembled-worker", mailbox::RecordKind::Input, input_id)
        .await
        .unwrap();
    let runtime_root = f.root.join("runtime");
    std::fs::create_dir_all(&runtime_root).unwrap();
    let episode_runtime_root = f.root.join("episode-native");
    std::fs::create_dir_all(&episode_runtime_root).unwrap();
    let episode_identity = RuntimeIdentity {
        runtime_kind: "kernel16-inprocess-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: episode_runtime_root.to_str().unwrap().into(),
        endpoint: "owned-async-os".into(),
        environment: "bwrap-private-git".into(),
    };
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-owned-os".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: runtime_root.to_str().unwrap().into(),
        endpoint: "owned-local-subprocess".into(),
        environment: "bwrap-private-git".into(),
    };
    let gateway = Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    let create = Uuid::new_v4();
    let start = Uuid::new_v4();
    let create_intent = Intent {
        id: create,
        admission_id: f.admission,
        binding_id: binding,
        generation: 1,
        kind: OperationKind::Create,
        arguments: serde_json::json!({}),
    };
    let start_intent = Intent {
        id: start,
        admission_id: f.admission,
        binding_id: binding,
        generation: 1,
        kind: OperationKind::Start,
        arguments: serde_json::json!({"execution":execution.id,"argv":["/bin/sh","-c","cat SPEC.md; printf 'exact result\\n' > src/result"],"input_packet":input_id,"input_digest":input["digest"],"input":input["packet"]}),
    };
    let prepare = serde_json::json!({"admission":f.admission,"binding":binding,"create":create_intent,"start":start_intent});
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let principal: Uuid = db
        .query_one(
            "SELECT principal_id FROM credentials WHERE token_hash=$1",
            &[&token_hash(&f.owner)],
        )
        .await
        .unwrap()
        .get(0);
    let queue = format!("assembled16-{}", f.epoch);
    let worker_log = std::fs::File::create(f.root.join("sdk-worker.log")).unwrap();
    let child = Process::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "assembled_sdk_worker",
            "--ignored",
            "--nocapture",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env(
            "ASSEMBLED_LEGACY_DROP_USAGE",
            if usage_query { "1" } else { "0" },
        )
        .env(
            "TEMPORAL_ADDRESS",
            std::env::var("TEMPORAL_ADDRESS").unwrap(),
        )
        .env("ASSEMBLED_DSN", &f.dsn)
        .env("ASSEMBLED_EPOCH", f.epoch.to_string())
        .env("ASSEMBLED_BEARER", &f.owner)
        .env("ASSEMBLED_PRINCIPAL", principal.to_string())
        .env("ASSEMBLED_QUEUE", &queue)
        .env(
            "ASSEMBLED_IDENTITY",
            serde_json::to_string(&identity).unwrap(),
        )
        .env("ASSEMBLED_PREPARE", prepare.to_string())
        .env("ASSEMBLED_RUNTIME_ROOT", &runtime_root)
        .env("ASSEMBLED_WORKSPACES", f.root.join("workspaces"))
        .env("ASSEMBLED_EPISODE_RUNTIME_ROOT", &episode_runtime_root)
        .env(
            "ASSEMBLED_EPISODE_IDENTITY",
            serde_json::to_string(&episode_identity).unwrap(),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::from(worker_log.try_clone().unwrap()))
        .stderr(Stdio::from(worker_log))
        .process_group(0)
        .spawn()
        .unwrap();
    let mut child = OwnedWorker(child);
    let ready_deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !runtime_root.join("controller-ready").exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "SDK controller exited before ready; retained worker log {}",
            std::fs::read_to_string(f.root.join("sdk-worker.log")).unwrap()
        );
        assert!(
            std::time::Instant::now() < ready_deadline,
            "owned controller must prepare immutable journals before SDK Activity dispatch"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let (conn, options) =
        ClientOptions::load_from_config(LoadClientConfigProfileOptions::default()).unwrap();
    let client = Client::new(Connection::connect(conn).await.unwrap(), options).unwrap();
    let wake = bridge
        .pending_wakeups(&f.owner, &r)
        .await
        .unwrap()
        .remove(0);
    bridge
        .deliver(&f.owner, &client, &queue, &wake)
        .await
        .unwrap();
    let create_deadline = std::time::Instant::now() + Duration::from_secs(8);
    while gateway.read(&f.owner, create).await.unwrap().state != OperationState::Confirmed {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "SDK worker exited during Create"
        );
        assert!(
            std::time::Instant::now() < create_deadline,
            "SDK driver must observe confirmed native thread creation"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // Start can only be prepared after the exact known thread exists; preparation performs no RPC.
    gateway.prepare(&f.owner, start_intent).await.unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(12);
    loop {
        let op = gateway.read(&f.owner, start).await.unwrap();
        if op.state == OperationState::Confirmed {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "actual registered SDK driver must confirm supervisor start, observed {:?}, worker log {}",
            op,
            std::fs::read_to_string(f.root.join("sdk-worker.log")).unwrap()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    loop {
        let view = supervisor
            .inspect("assembled-worker", execution.id)
            .await
            .unwrap();
        if view.state == supervisor::ExecutionState::Stopped {
            break;
        }
        assert_ne!(
            view.state,
            supervisor::ExecutionState::Quarantined,
            "{:?}",
            view.quarantine_reason
        );
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        std::fs::read(execution.path.join("src/result")).unwrap(),
        b"exact result\n"
    );
    let native: serde_json::Value =
        serde_json::from_slice(&std::fs::read(runtime_root.join("turn.json")).unwrap()).unwrap();
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    mailbox.execute(&f.owner,mail(f.epoch,"bind-actual-start",serde_json::json!({"kind":"bind_runtime_turn","episode":episode,"start_operation":start,"runtime_turn":native["turn"]}))).await.unwrap();
    let bound = mailbox
        .record("assembled-worker", mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    assert_eq!(
        bound["turn_id"], native["turn"],
        "episode binds the independently retained native turn, current packet and exact confirmed Start"
    );
    if usage_query {
        let unknown: serde_json::Value = db
            .query_one(
                "SELECT usage FROM gateway_operations WHERE id=$1",
                &[&start],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(unknown, serde_json::json!({"kind":"unknown"}));
        assert_eq!(
            supervisor
                .release("assembled-worker", execution.id)
                .await
                .unwrap_err()
                .code,
            ErrorCode::DependencyUnmet
        );
        if scenario == EngineeringScenario::LegacyGatewayRestart {
            let out = evidence_directory("legacy-gateway-restart-safe-quarantine");
            let original_controller: Uuid = db
                .query_one(
                    "SELECT controller FROM gateway_operations WHERE id=$1",
                    &[&start],
                )
                .await
                .unwrap()
                .get(0);
            retain_native(&runtime_root, &out, "original");
            child.kill();
            let replacement_log = std::fs::File::create(out.join("replacement-sdk.log")).unwrap();
            let mut replacement = OwnedWorker(
                Process::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "assembled_sdk_worker",
                        "--ignored",
                        "--nocapture",
                    ])
                    .env_clear()
                    .env("PATH", "/usr/bin:/bin")
                    .env(
                        "TEMPORAL_ADDRESS",
                        std::env::var("TEMPORAL_ADDRESS").unwrap(),
                    )
                    .env("ASSEMBLED_DSN", &f.dsn)
                    .env("ASSEMBLED_EPOCH", f.epoch.to_string())
                    .env("ASSEMBLED_BEARER", &f.owner)
                    .env("ASSEMBLED_PRINCIPAL", principal.to_string())
                    .env("ASSEMBLED_QUEUE", &queue)
                    .env(
                        "ASSEMBLED_IDENTITY",
                        serde_json::to_string(&identity).unwrap(),
                    )
                    .env("ASSEMBLED_PREPARE", prepare.to_string())
                    .env("ASSEMBLED_RUNTIME_ROOT", &runtime_root)
                    .env("ASSEMBLED_WORKSPACES", f.root.join("workspaces"))
                    .env("ASSEMBLED_EPISODE_RUNTIME_ROOT", &episode_runtime_root)
                    .env(
                        "ASSEMBLED_EPISODE_IDENTITY",
                        serde_json::to_string(&episode_identity).unwrap(),
                    )
                    .stdin(Stdio::null())
                    .stdout(Stdio::from(replacement_log.try_clone().unwrap()))
                    .stderr(Stdio::from(replacement_log))
                    .process_group(0)
                    .spawn()
                    .unwrap(),
            );
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let status = loop {
                if let Some(status) = replacement.0.try_wait().unwrap() {
                    break status;
                }
                assert!(std::time::Instant::now() < deadline);
                tokio::time::sleep(Duration::from_millis(20)).await;
            };
            assert!(
                !status.success(),
                "unsupported legacy transport transfer must stop, never restart workload"
            );
            kernel::Kernel::new(f.broker.clone())
                .pause(
                    &f.owner,
                    f.mission,
                    "legacy-restart-pause",
                    0,
                    true,
                    "dead legacy controller requires explicit safe quarantine",
                )
                .await
                .unwrap();
            assert_eq!(
                Gateway::new(f.broker.clone())
                    .reconcile_usage(&f.owner, start)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet
            );
            assert_eq!(
                supervisor
                    .release("assembled-worker", execution.id)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet
            );
            let retained:serde_json::Value=db.query_one("SELECT jsonb_build_object('operation',o.id,'state',o.state,'usage',o.usage,'controller',s.controller,'store_quarantined',s.quarantined,'busy',s.busy) FROM gateway_operations o JOIN gateway_bindings b ON b.id=o.binding_id JOIN gateway_stores s ON s.identity=b.store WHERE o.id=$1", &[&start]).await.unwrap().get(0);
            assert_eq!(retained["controller"], original_controller.to_string());
            assert_eq!(retained["usage"]["kind"], "unknown");
            assert_eq!(retained["store_quarantined"], true);
            let mission = f
                .broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap();
            assert_eq!(mission.data["used_capacity"], 1);
            for extension in ["invocation", "effect"] {
                assert_eq!(
                    std::fs::read_dir(&runtime_root)
                        .unwrap()
                        .filter(|e| e
                            .as_ref()
                            .unwrap()
                            .path()
                            .extension()
                            .is_some_and(|x| x == extension))
                        .count(),
                    2,
                    "replacement cannot issue a third native effect"
                );
            }
            retain_native(&runtime_root, &out, "after-restart");
            retain_bytes(
                &out.join("retained-quarantine.json"),
                &serde_json::to_vec(&retained).unwrap(),
            );
            // No actual workload remains; the DB reservation is intentionally held
            // until this disposable model-free fixture database is destroyed.
            std::sync::Arc::try_unwrap(f)
                .unwrap_or_else(|_| panic!("repair task retained fixture"))
                .finish()
                .await;
            return;
        }
        if let Some((pending, command)) = pending_verifier {
            let before = f
                .broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data;
            let launched = if let Some((resume, handle)) = held_monitor {
                resume.notify_one();
                tokio::time::timeout(Duration::from_secs(8), handle)
                    .await
                    .expect("monitor must terminate after final claim check")
                    .unwrap()
            } else {
                supervisor
                    .start("assembled-verifier", pending.id, command, 30)
                    .await
            };
            if launched.is_ok() {
                let deadline = std::time::Instant::now() + Duration::from_secs(8);
                loop {
                    let view = supervisor
                        .inspect("assembled-verifier", pending.id)
                        .await
                        .unwrap();
                    assert_ne!(
                        view.state,
                        supervisor::ExecutionState::Quarantined,
                        "{:?}",
                        view.quarantine_reason
                    );
                    if view.state == supervisor::ExecutionState::Stopped
                        && supervisor
                            .result("assembled-verifier", pending.id)
                            .await
                            .unwrap()
                            .is_some()
                    {
                        break;
                    }
                    assert!(
                        std::time::Instant::now() < deadline,
                        "actual launched verifier must finish before fixture teardown"
                    );
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            }
            let view = supervisor
                .inspect("assembled-verifier", pending.id)
                .await
                .unwrap();
            let result = supervisor
                .result("assembled-verifier", pending.id)
                .await
                .unwrap();
            let after = f
                .broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data;
            let out = evidence_directory(
                if scenario == EngineeringScenario::UnknownLegacyMonitorClaim {
                    "unknown-legacy-monitor-claim"
                } else {
                    "unknown-legacy-physical-claim"
                },
            );
            retain_bytes(&out.join("physical-claim.json"),&serde_json::to_vec(&serde_json::json!({"before":before,"after":after,"execution":pending.id,"state":view.state,"trusted_result":result})).unwrap());
            retain_bytes(
                &out.join("verifier.log"),
                &std::fs::read(&view.manifest.log_path).unwrap_or_default(),
            );
            retain_native(&runtime_root, &out, "native");
            child.kill();
            std::sync::Arc::try_unwrap(f)
                .unwrap_or_else(|_| panic!("repair task retained fixture"))
                .finish()
                .await;
            assert_eq!(
                launched.unwrap_err().code,
                ErrorCode::DependencyUnmet,
                "previous admission cannot authorize physical work after usage becomes unknown"
            );
            assert!(
                result.is_none(),
                "no trusted workload result can exist for a rejected physical claim"
            );
            assert_eq!(before["used_capacity"], after["used_capacity"]);
            return;
        }
        if scenario == EngineeringScenario::UnknownLegacyContinuation {
            mailbox.execute("assembled-worker",mail(f.epoch,"yield-unmeasured-cost",serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"actual result remains isolated in stopped workspace; native usage unobserved","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
            mailbox.execute("assembled-worker",mail(f.epoch,"unmeasured-cost-new-input",serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":"New input must not bypass unknown cost reservation","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
            let next_input = Uuid::new_v4();
            let next_episode = Uuid::new_v4();
            mailbox.execute("assembled-worker",mail(f.epoch,"unmeasured-cost-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":next_input,"attempt":f.attempt,"purpose":"implementation","limit":4}))).await.unwrap();
            let packet = mailbox
                .record("assembled-worker", mailbox::RecordKind::Input, next_input)
                .await
                .unwrap();
            let before = f
                .broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data;
            let admitted=mailbox.execute("assembled-worker",mail(f.epoch,"unmeasured-cost-continuation",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":next_episode,"attempt":f.attempt,"purpose":"implementation","input_packet":next_input,"input_digest":packet["digest"]}))).await;
            let after = f
                .broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data;
            let out = evidence_directory("unknown-legacy-continuation");
            retain_bytes(
                &out.join("budget-before-after.json"),
                &serde_json::to_vec(&serde_json::json!({"before":before,"after":after})).unwrap(),
            );
            retain_native(&runtime_root, &out, "native");
            if admitted.is_ok() {
                // The incorrectly admitted metadata-only successor has no resource
                // or RPC. Release only this known-unused reservation via public Yield.
                mailbox.execute("assembled-worker",mail(f.epoch,"cleanup-unused-cost-successor",serde_json::json!({"kind":"yield_episode","episode":next_episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"fixture cleanup of unused successor; original unknown reservation remains","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
            }
            child.kill();
            std::sync::Arc::try_unwrap(f)
                .unwrap_or_else(|_| panic!("repair task retained fixture"))
                .finish()
                .await;
            assert_eq!(
                admitted.unwrap_err().code,
                ErrorCode::DependencyUnmet,
                "new input cannot bypass authoritative unknown legacy usage"
            );
            assert_eq!(before["admitted_episodes"], after["admitted_episodes"]);
            assert_eq!(before["used_capacity"], after["used_capacity"]);
            return;
        }
        let out = evidence_directory("legacy-usage-query");
        kernel::Kernel::new(f.broker.clone())
            .pause(
                &f.owner,
                f.mission,
                "legacy-query-paused",
                0,
                true,
                "query usage independently without dispatch",
            )
            .await
            .unwrap();
        let query = |token: &str| {
            let mut command = Process::new(env!("CARGO_BIN_EXE_team-control-plane"));
            command
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("TEAM_CONTROL_PLANE_DSN", &f.dsn)
                .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", f.epoch.to_string())
                .env("TEAM_CONTROL_PLANE_TOKEN", token)
                .args(["reconcile-gateway-usage", &start.to_string()]);
            bounded_operator_cli(command)
        };
        if foreign_controller {
            let original:Uuid=db.query_one("SELECT s.controller FROM gateway_stores s JOIN gateway_bindings b ON b.store=s.identity WHERE b.id=$1", &[&binding]).await.unwrap().get(0);
            let replacement = Gateway::new(f.broker.clone());
            assert_ne!(replacement.controller(), original);
            assert_eq!(
                replacement
                    .bind(&f.owner, Uuid::new_v4(), f.admission, &identity, None, 1)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::TemporarilyUnavailable
            );
            let (denied, timeout) = query(&f.owner).await;
            retain_bytes(&out.join("foreign-controller-query.stdout"), &denied.stdout);
            retain_bytes(&out.join("foreign-controller-query.stderr"), &denied.stderr);
            assert!(!timeout);
            assert!(!denied.status.success());
            assert!(denied.stdout.is_empty());
            assert!(String::from_utf8_lossy(&denied.stderr).contains("dependency_unmet"));
            let unchanged:bool=db.query_one("SELECT s.controller=$2 AND s.quarantined AND o.usage->>'kind'='unknown' AND NOT EXISTS(SELECT 1 FROM kernel_gateway_usage_evidence e WHERE e.operation_id=o.id) FROM gateway_operations o JOIN gateway_bindings b ON b.id=o.binding_id JOIN gateway_stores s ON s.identity=b.store WHERE o.id=$1", &[&start,&original]).await.unwrap().get(0);
            assert!(
                unchanged,
                "foreign binding cannot acquire authority or attest usage"
            );
            assert_eq!(
                supervisor
                    .release("assembled-worker", execution.id)
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
            for extension in ["invocation", "effect"] {
                assert_eq!(
                    std::fs::read_dir(&runtime_root)
                        .unwrap()
                        .filter(|e| e
                            .as_ref()
                            .unwrap()
                            .path()
                            .extension()
                            .is_some_and(|x| x == extension))
                        .count(),
                    2
                );
            }
            retain_native(&runtime_root, &out, "foreign-controller");
            child.kill();
            std::sync::Arc::try_unwrap(f)
                .unwrap_or_else(|_| panic!("repair task retained fixture"))
                .finish()
                .await;
            return;
        }
        let effect = std::fs::read_dir(&runtime_root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                if p.extension().is_none_or(|x| x != "effect") {
                    return false;
                }
                let receipt: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
                receipt["operation"] == start.to_string()
            })
            .unwrap();
        let original =
            std::fs::read(f.root.join(format!("original-legacy-receipt-{start}.json"))).unwrap();
        let unsupported = std::fs::read(&effect).unwrap();
        let withheld = f.root.join("legacy-query-withheld-receipt.json");
        for case in [
            "unsupported-profile",
            "missing",
            "corrupt",
            "symlink",
            "wrong-binding",
            "wrong-runtime",
        ] {
            std::fs::rename(&effect, &withheld).unwrap();
            match case {
                "unsupported-profile" => std::fs::write(&effect, &unsupported).unwrap(),
                "missing" => {}
                "corrupt" => std::fs::write(&effect, b"malformed receipt").unwrap(),
                "symlink" => std::os::unix::fs::symlink(&withheld, &effect).unwrap(),
                _ => {
                    let mut receipt: serde_json::Value = serde_json::from_slice(&original).unwrap();
                    if case == "wrong-binding" {
                        receipt["intent"]["binding_id"] = serde_json::json!(Uuid::new_v4());
                    } else {
                        receipt["runtime"]["deployment"] = serde_json::json!(Uuid::new_v4());
                    }
                    std::fs::write(&effect, serde_json::to_vec(&receipt).unwrap()).unwrap();
                }
            }
            let (output, timeout) = query(&f.owner).await;
            retain_bytes(&out.join(format!("query-{case}.stdout")), &output.stdout);
            retain_bytes(&out.join(format!("query-{case}.stderr")), &output.stderr);
            assert!(!timeout, "{case} must be bounded");
            if output.status.success() {
                let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(value["operation"]["state"], "confirmed");
                assert_eq!(value["usage"]["kind"], "unknown");
            } else {
                assert!(output.stdout.is_empty());
                assert!(String::from_utf8_lossy(&output.stderr).contains("dependency_unmet"));
            }
            let usage: serde_json::Value = db
                .query_one(
                    "SELECT usage FROM gateway_operations WHERE id=$1",
                    &[&start],
                )
                .await
                .unwrap()
                .get(0);
            assert_eq!(
                usage,
                serde_json::json!({"kind":"unknown"}),
                "{case} cannot attest zero"
            );
            assert_eq!(
                supervisor
                    .release("assembled-worker", execution.id)
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
            if case != "missing" {
                std::fs::remove_file(&effect).unwrap();
            }
            std::fs::rename(&withheld, &effect).unwrap();
            assert_eq!(std::fs::read(&effect).unwrap(), unsupported);
            println!("completed legacy usage receipt negative {case}");
        }
        for token in [
            "wrong-token",
            "assembled-worker",
            "assembled-lead",
            "assembled-verifier",
        ] {
            let (output, timeout) = query(token).await;
            assert!(!timeout);
            assert!(!output.status.success());
            assert!(output.stdout.is_empty());
        }
        std::fs::write(&effect, &original).unwrap();
        let (output, timeout) = query(&f.owner).await;
        assert!(!timeout);
        retain_bytes(&out.join("query-restored.stdout"), &output.stdout);
        retain_bytes(&out.join("query-restored.stderr"), &output.stderr);
        assert!(
            output.status.success(),
            "restored exact receipt reconciles usage: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["usage"]["kind"], "measured_zero");
        assert_eq!(
            value["usage"]["receipt_digest"],
            artifacts::sha256(&original)
        );
        let (duplicate, timeout) = query(&f.owner).await;
        assert!(!timeout);
        assert!(duplicate.status.success());
        let evidence: i64 = db
            .query_one(
                "SELECT count(*) FROM kernel_gateway_usage_evidence WHERE operation_id=$1",
                &[&start],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(evidence, 1);
        let unchanged:bool=db.query_one("SELECT s.controller=o.controller AND s.busy IS NULL AND c.paused FROM gateway_operations o JOIN gateway_bindings b ON b.id=o.binding_id JOIN gateway_stores s ON s.identity=b.store JOIN kernel_mission_control c ON c.mission_id=$2 WHERE o.id=$1", &[&start,&f.mission]).await.unwrap().get(0);
        assert!(unchanged);
        for extension in ["invocation", "effect"] {
            assert_eq!(
                std::fs::read_dir(&runtime_root)
                    .unwrap()
                    .filter(|e| e
                        .as_ref()
                        .unwrap()
                        .path()
                        .extension()
                        .is_some_and(|x| x == extension))
                    .count(),
                2,
                "query cannot issue a third native RPC"
            );
        }
        retain_native(&runtime_root, &out, "restored");
        kernel::Kernel::new(f.broker.clone())
            .pause(
                &f.owner,
                f.mission,
                "legacy-query-resume",
                1,
                false,
                "verified exact legacy usage receipt; continue engineering",
            )
            .await
            .unwrap();
    }
    let artifacts = artifacts::ArtifactService::open(f.broker.clone(), &f.root.join("artifacts"))
        .await
        .unwrap();
    let artifact_command = |key: &str, revision, command| artifacts::ArtifactEnvelope {
        key: key.into(),
        task: f.task,
        expected_revision: revision,
        generation: 1,
        recovery_epoch: f.epoch,
        command,
    };
    let checks=vec![vec!["/bin/sh".into(),"-c".into(),"test \"$(cat src/result)\" = 'exact result' && test \"$(cat src/resumed)\" = 'resumed-from-retained-bytes' && cat src/result src/resumed; status=$?; printf 'gate_exit=%s\\n' \"$status\"; exit \"$status\"".into()]];
    let revision = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(f.task))
        .await
        .unwrap()
        .revision;
    let configured = artifact_fixture_call(
        artifacts.clone(),
        f.owner.to_owned(),
        artifact_command(
            "configure-exact-gate",
            revision,
            artifacts::ArtifactCommand::Configure {
                verifier: f.verifier,
                lead_acceptance: false,
                base: f.base.clone(),
                suite: artifacts::sha256(&serde_json::to_vec(&checks).unwrap()),
                environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                mandatory_commands: checks.clone(),
            },
        ),
        artifacts::ReferenceFault::None,
    )
    .await
    .unwrap();
    git(&execution.path, &["add", "src/result"]);
    git(
        &execution.path,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "actual capsule output",
        ],
    );
    let commit = git(&execution.path, &["rev-parse", "HEAD"]);
    let tree = git(&execution.path, &["rev-parse", "HEAD^{tree}"]);
    let bundle = f.root.join("actual-candidate.bundle");
    git(
        &execution.path,
        &["bundle", "create", bundle.to_str().unwrap(), "HEAD"],
    );
    let bytes = std::fs::read(&bundle).unwrap();
    let digest = artifacts::sha256(&bytes);
    artifacts
        .publish_evidence("assembled-worker", f.task, &digest, &bytes)
        .await
        .unwrap();
    let candidate = artifacts::Candidate {
        commit,
        tree,
        base: f.base.clone(),
        spec_digest: f.spec.clone(),
        spec_revision: 2,
        generation: 1,
        artifact: digest,
        deliverables: vec!["src/result".into()],
        risks: vec![],
    };
    let submitted = artifact_fixture_call(
        artifacts.clone(),
        ("assembled-worker").to_owned(),
        artifact_command(
            "submit-actual-capsule",
            configured.revision,
            artifacts::ArtifactCommand::Submit {
                candidate: candidate.clone(),
            },
        ),
        artifacts::ReferenceFault::None,
    )
    .await
    .unwrap();
    let assertion = b"caller asserts success before any verifier allocation";
    let assertion_digest = artifacts::sha256(assertion);
    artifacts
        .publish_evidence("assembled-verifier", f.task, &assertion_digest, assertion)
        .await
        .unwrap();
    assert_eq!(
        artifact_fixture_call(
            artifacts.clone(),
            ("assembled-verifier").to_owned(),
            artifact_command(
                "deny-preallocation-report-bypass",
                submitted.revision,
                artifacts::ArtifactCommand::Verify {
                    verification: artifacts::Verification {
                        candidate: candidate.clone(),
                        suite: artifacts::sha256(&serde_json::to_vec(&checks).unwrap()),
                        environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                        checks: vec![artifacts::Check {
                            command: checks[0].clone(),
                            exit_code: 0,
                            logs: assertion_digest
                        }],
                        resolved_decisions: true
                    }
                }
            ),
            artifacts::ReferenceFault::None
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::DependencyUnmet,
        "assembled requirement must already be active before any Verifier is allocated"
    );
    mailbox.execute("assembled-worker",mail(f.epoch,"yield-actual-capsule",serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"actual capsule candidate frozen; resume from these retained bytes","artifacts":[{"task":f.task,"digest":candidate.artifact}]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    assert_eq!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data["used_capacity"],
        1,
        "yield alone cannot refund the physical execution"
    );
    mailbox.execute("assembled-worker",mail(f.epoch,"continuation-input",serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":"Resume with independently captured new input","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let continuation = f.worker_episode("worker-continuation").await;
    assert_eq!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data["used_capacity"],
        2,
        "a continuation owns a new physical slot; only the imported first episode was already charged by assignment"
    );
    let continuation_execution = supervisor
        .create_episode_from_candidate(
            "assembled-worker",
            continuation,
            &artifacts,
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(continuation_execution.path.join("src/result")).unwrap(),
        b"exact result\n",
        "continuation restores the actual frozen checkpoint bytes"
    );
    episode_runtime::EpisodeGateway::new(f.broker.clone()).prepare("assembled-worker",episode_runtime::StartRequest{id:Uuid::new_v4(),episode:continuation,execution:continuation_execution.id,runtime:episode_identity.clone(),argv:vec!["/bin/sh".into(),"-c".into(),"python3 -c 'import json; p=json.load(open(\"/episode-input.json\")); assert p[\"role\"]==\"worker\" and any(m[\"text\"]==\"Resume with independently captured new input\" for m in p[\"messages\"])'; status=$?; if test \"$status\" != 0; then exit \"$status\"; fi; test \"$(cat src/result)\" = 'exact result' && printf 'resumed-from-retained-bytes\\n' > src/resumed".into()]}).await.unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let resumed_result = loop {
        if let Some(result) = supervisor
            .result("assembled-worker", continuation_execution.id)
            .await
            .unwrap()
        {
            break result;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "registered SDK must start the genuine continuation episode"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(resumed_result.exit_code, Some(0));
    assert_eq!(
        std::fs::read(continuation_execution.path.join("src/resumed")).unwrap(),
        b"resumed-from-retained-bytes\n"
    );
    let resumed_episode = mailbox
        .record(
            "assembled-worker",
            mailbox::RecordKind::Episode,
            continuation,
        )
        .await
        .unwrap();
    assert!(
        resumed_episode["turn_id"]
            .as_str()
            .is_some_and(|s| s.starts_with("turn-"))
    );
    git(&continuation_execution.path, &["add", "src/resumed"]);
    let resumed_tree = git(&continuation_execution.path, &["write-tree"]);
    // The integration candidate retains the admitted base as first parent and the
    // restored checkpoint commit as second parent; its tree includes actual resumed work.
    let resumed_commit = git(
        &continuation_execution.path,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit-tree",
            &resumed_tree,
            "-p",
            &f.base,
            "-p",
            &candidate.commit,
            "-m",
            "actual resumed deliverable",
        ],
    );
    git(
        &continuation_execution.path,
        &[
            "update-ref",
            "refs/heads/resumed-candidate",
            &resumed_commit,
        ],
    );
    let resumed_bundle = f.root.join("actual-resumed-candidate.bundle");
    git(
        &continuation_execution.path,
        &[
            "bundle",
            "create",
            resumed_bundle.to_str().unwrap(),
            "refs/heads/resumed-candidate",
        ],
    );
    let resumed_bytes = std::fs::read(&resumed_bundle).unwrap();
    let resumed_digest = artifacts::sha256(&resumed_bytes);
    artifacts
        .publish_evidence("assembled-worker", f.task, &resumed_digest, &resumed_bytes)
        .await
        .unwrap();
    let mut candidate = artifacts::Candidate {
        commit: resumed_commit,
        tree: resumed_tree,
        base: f.base.clone(),
        spec_digest: f.spec.clone(),
        spec_revision: 2,
        generation: 1,
        artifact: resumed_digest,
        deliverables: vec!["src/result".into(), "src/resumed".into()],
        risks: vec![],
    };
    let current = artifacts
        .snapshot("assembled-worker", f.task)
        .await
        .unwrap();
    artifact_fixture_call(
        artifacts.clone(),
        ("assembled-worker").to_owned(),
        artifact_command(
            "submit-actual-resumed-deliverable",
            current.revision,
            artifacts::ArtifactCommand::Submit {
                candidate: candidate.clone(),
            },
        ),
        artifacts::ReferenceFault::None,
    )
    .await
    .unwrap();
    mailbox.execute("assembled-worker",mail(f.epoch,"yield-resumed-worker",serde_json::json!({"kind":"yield_episode","episode":continuation,"checkpoint":{"id":Uuid::new_v4(),"summary":"actual resumed capsule consumed retained candidate and new captured input","artifacts":[{"task":f.task,"digest":candidate.artifact}]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    let usage_diagnostics = evidence_directory("assembled-pre-release-usage");
    let usage_rows: Vec<serde_json::Value> = db.query(
        "SELECT jsonb_build_object('protocol','legacy','operation',o.id,'attempt',o.attempt_id,'state',o.state,'usage',o.usage,'controller',o.controller,'registered',EXISTS(SELECT 1 FROM kernel_transports t WHERE t.controller=o.controller),'requirement',EXISTS(SELECT 1 FROM attempts a JOIN kernel_verification_requirements r ON r.task_id=a.task_id WHERE a.id=o.attempt_id)) FROM gateway_operations o WHERE o.attempt_id=$1 UNION ALL SELECT jsonb_build_object('protocol','episode','operation',o.id,'episode',o.episode_id,'execution',o.execution_id,'attempt',x.attempt_id,'state',o.state,'usage',o.usage,'controller',o.controller,'registered',EXISTS(SELECT 1 FROM kernel_transports t WHERE t.controller=o.controller)) FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE x.attempt_id=$1",
        &[&f.attempt],
    ).await.unwrap().into_iter().map(|row|row.get(0)).collect();
    retain_bytes(
        &usage_diagnostics.join("operation-usage-before-release.json"),
        &serde_json::to_vec(&usage_rows).unwrap(),
    );
    retain_native(&runtime_root, &usage_diagnostics, "legacy");
    retain_native(&episode_runtime_root, &usage_diagnostics, "episode");
    retain_bytes(
        &usage_diagnostics.join("sdk-worker.log"),
        &std::fs::read(f.root.join("sdk-worker.log")).unwrap(),
    );
    let legacy_rows: Vec<_> = usage_rows
        .iter()
        .filter(|r| r["protocol"] == "legacy")
        .collect();
    assert_eq!(
        legacy_rows.len(),
        2,
        "real legacy Create and first Worker Start were executed"
    );
    for row in legacy_rows {
        assert_eq!(row["registered"], true);
        assert_eq!(
            row["usage"]["kind"], "measured_zero",
            "exact registered legacy receipt is required before refund"
        );
        let receipts: Vec<_> = std::fs::read_dir(&runtime_root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "effect"))
            .filter_map(|p| {
                let bytes = std::fs::read(p).unwrap();
                let receipt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                (receipt["operation"] == row["operation"]).then_some(bytes)
            })
            .collect();
        assert_eq!(receipts.len(), 1);
        assert_eq!(
            row["usage"]["receipt_digest"],
            artifacts::sha256(&receipts[0])
        );
    }
    let public_usage_status = kernel::Kernel::new(f.broker.clone())
        .status(&f.owner)
        .await
        .unwrap();
    for row in &usage_rows {
        if row["usage"]["kind"] == "measured_zero"
            && matches!(row["state"].as_str(), Some("confirmed" | "failed_known"))
        {
            let protocol = if row["protocol"] == "legacy" {
                "legacy_gateway"
            } else {
                "episode_native"
            };
            assert!(
                public_usage_status["unresolved_operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|operation| operation["id"] != row["operation"]
                        || operation["protocol"] != protocol),
                "resolved measured usage must not remain in unresolved operator status"
            );
        }
    }
    let observed_legacy: i64 = db.query_one("SELECT count(*) FROM kernel_gateway_usage_evidence e JOIN gateway_operations o ON o.id=e.operation_id WHERE o.attempt_id=$1", &[&f.attempt]).await.unwrap().get(0);
    assert_eq!(
        observed_legacy, 2,
        "one immutable actual-byte observation per native operation"
    );
    supervisor
        .release("assembled-worker", continuation_execution.id)
        .await
        .unwrap();
    let verifier_input = Uuid::new_v4();
    let verifier_episode = Uuid::new_v4();
    mailbox.execute("assembled-verifier",mail(f.epoch,"capture-verification",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":verifier_input,"attempt":f.attempt,"purpose":"verification","limit":4}))).await.unwrap();
    let packet = mailbox
        .record(
            "assembled-verifier",
            mailbox::RecordKind::Input,
            verifier_input,
        )
        .await
        .unwrap();
    mailbox.execute("assembled-verifier",mail(f.epoch,"admit-verification",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":verifier_episode,"attempt":f.attempt,"purpose":"verification","input_packet":verifier_input,"input_digest":packet["digest"]}))).await.unwrap();
    let verifier_execution = supervisor
        .create_episode_from_candidate(
            "assembled-verifier",
            verifier_episode,
            &artifacts,
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(verifier_execution.path.join("src/result"))
            .expect("submitted bundle's src/result must exist in independent candidate workspace"),
        b"exact result\n",
        "independent verification must materialize submitted immutable candidate bytes, never admitted base"
    );
    assert_eq!(
        std::fs::read(verifier_execution.path.join("src/resumed")).unwrap(),
        b"resumed-from-retained-bytes\n",
        "Verifier must import the post-resume deliverable"
    );
    let measured_commit = git(&verifier_execution.path, &["rev-parse", "HEAD"]);
    let measured_tree = git(&verifier_execution.path, &["rev-parse", "HEAD^{tree}"]);
    let measured_base = git(&verifier_execution.path, &["rev-parse", "HEAD^1"]);
    assert_eq!(
        (&measured_commit, &measured_tree, &measured_base),
        (&candidate.commit, &candidate.tree, &candidate.base)
    );
    assert_eq!(
        supervisor
            .inspect("assembled-verifier", verifier_execution.id)
            .await
            .unwrap()
            .manifest
            .write_scope,
        Vec::<String>::new()
    );
    let changed_after_allocation = if scenario == EngineeringScenario::PostAllocationBaseChange {
        let store = f.root.join("store");
        std::fs::write(store.join("src/base"), b"postallocation integration base\n").unwrap();
        git(&store, &["add", "src/base"]);
        git(
            &store,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "postallocation base",
            ],
        );
        let base = git(&store, &["rev-parse", "HEAD"]);
        let current = artifacts.snapshot(&f.owner, f.task).await.unwrap();
        artifact_fixture_call(
            artifacts.clone(),
            f.owner.to_owned(),
            artifact_command(
                "postallocation-change-policy",
                current.revision,
                artifacts::ArtifactCommand::Configure {
                    verifier: f.verifier,
                    lead_acceptance: false,
                    base,
                    suite: artifacts::sha256(&serde_json::to_vec(&checks).unwrap()),
                    environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                    mandatory_commands: checks.clone(),
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
        true
    } else {
        false
    };
    episode_runtime::EpisodeGateway::new(f.broker.clone())
        .prepare(
            "assembled-verifier",
            episode_runtime::StartRequest {
                id: Uuid::new_v4(),
                episode: verifier_episode,
                execution: verifier_execution.id,
                runtime: episode_identity.clone(),
                argv: checks[0].clone(),
            },
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let verifier_view = loop {
        let view = supervisor
            .inspect("assembled-verifier", verifier_execution.id)
            .await
            .unwrap();
        if view.state == supervisor::ExecutionState::Stopped {
            break view;
        }
        assert_ne!(
            view.state,
            supervisor::ExecutionState::Quarantined,
            "{:?}",
            view.quarantine_reason
        );
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    let trusted_result = loop {
        if let Some(result) = supervisor
            .result("assembled-verifier", verifier_execution.id)
            .await
            .unwrap()
        {
            break result;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "trusted monitor exit result must be independently durable before verification"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(trusted_result.exit_code, Some(0));
    assert_eq!(trusted_result.signal, None);
    assert!(trusted_result.namespace_stopped);
    assert_eq!(trusted_result.argv, checks[0]);
    assert_eq!(trusted_result.candidate, Some(candidate.clone()));
    let output = std::fs::read(&verifier_view.manifest.log_path).unwrap();
    assert_eq!(
        output, b"exact result\nresumed-from-retained-bytes\ngate_exit=0\n",
        "actual independent gate output and its shell exit result"
    );
    assert_eq!(trusted_result.logs, artifacts::sha256(&output));
    let log=serde_json::to_vec(&serde_json::json!({"candidate":candidate,"measured_commit":measured_commit,"measured_tree":measured_tree,"measured_base":measured_base,"execution":verifier_execution.id,"episode":verifier_episode,"input_packet":verifier_input,"command":checks[0],"trusted_monitor_result":trusted_result,"output":String::from_utf8(output).unwrap()})).unwrap();
    let log_digest = artifacts::sha256(&log);
    artifacts
        .publish_evidence("assembled-verifier", f.task, &log_digest, &log)
        .await
        .unwrap();
    if changed_after_allocation {
        let current = artifacts.snapshot(&f.owner, f.task).await.unwrap();
        let stale = artifact_fixture_call(
            artifacts.clone(),
            ("assembled-verifier").to_owned(),
            artifact_command(
                "reject-postallocation-stale-report",
                current.revision,
                artifacts::ArtifactCommand::Verify {
                    verification: artifacts::Verification {
                        candidate: candidate.clone(),
                        suite: artifacts::sha256(&serde_json::to_vec(&checks).unwrap()),
                        environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                        checks: vec![artifacts::Check {
                            command: checks[0].clone(),
                            exit_code: 0,
                            logs: log_digest.clone(),
                        }],
                        resolved_decisions: true,
                    },
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap_err();
        assert_eq!(stale.code, ErrorCode::StaleRevision);
        assert_eq!(
            artifacts.snapshot(&f.owner, f.task).await.unwrap().status,
            "submitted"
        );
        let out = evidence_directory("postallocation-base-change");
        retain_bytes(
            &out.join("historical-snapshot-result.json"),
            &serde_json::to_vec(&trusted_result).unwrap(),
        );
        retain_bytes(&out.join("stale-report-refusal.json"),&serde_json::to_vec(&serde_json::json!({"code":stale.code,"candidate":candidate,"current_revision":current.revision})).unwrap());
        // Owner restores this prior base only to finish the old bootstrap; below,
        // the changed-base path builds and measures a fresh integration candidate.
        artifact_fixture_call(
            artifacts.clone(),
            f.owner.to_owned(),
            artifact_command(
                "restore-bootstrap-base-policy",
                current.revision,
                artifacts::ArtifactCommand::Configure {
                    verifier: f.verifier,
                    lead_acceptance: false,
                    base: f.base.clone(),
                    suite: artifacts::sha256(&serde_json::to_vec(&checks).unwrap()),
                    environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                    mandatory_commands: checks.clone(),
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
    }
    let snapshot = artifacts
        .snapshot("assembled-verifier", f.task)
        .await
        .unwrap();
    let mut verified = artifact_fixture_call(
        artifacts.clone(),
        ("assembled-verifier").to_owned(),
        artifact_command(
            "verify-actual-independent-gate",
            snapshot.revision,
            artifacts::ArtifactCommand::Verify {
                verification: artifacts::Verification {
                    candidate: candidate.clone(),
                    suite: artifacts::sha256(&serde_json::to_vec(&checks).unwrap()),
                    environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                    checks: vec![artifacts::Check {
                        command: checks[0].clone(),
                        exit_code: trusted_result.exit_code.unwrap(),
                        logs: log_digest.clone(),
                    }],
                    resolved_decisions: true,
                },
            },
        ),
        artifacts::ReferenceFault::None,
    )
    .await
    .unwrap();
    if scenario == EngineeringScenario::CancelMeasuredCandidate {
        let fenced_steer = Uuid::new_v4();
        gateway.prepare(&f.owner, Intent {
            id: fenced_steer, admission_id: f.admission, binding_id: binding,
            generation: 1, kind: OperationKind::Steer,
            arguments: serde_json::json!({"message":"prepared effect must remain fenced after cancellation"}),
        }).await.unwrap();
        let cancelled = f
            .broker
            .execute(
                &f.owner,
                envelope(
                    f.epoch,
                    "cancel-measured-candidate",
                    verified.revision,
                    Command::CancelAttempt {
                        task_id: f.task,
                        attempt_id: f.attempt,
                        reason: "cancel before accepting actual independent measured proof".into(),
                    },
                ),
            )
            .await
            .unwrap();
        assert_eq!(cancelled.disposition, "cancellation_requested");
        for (token, command) in [
            (
                &f.owner,
                artifact_command(
                    "refuse-accept-cancelled",
                    cancelled.revision,
                    artifacts::ArtifactCommand::Accept {
                        candidate_id: verified.candidate_id.clone().unwrap(),
                        report_id: verified.report_id.clone().unwrap(),
                    },
                ),
            ),
            (
                &"assembled-verifier".to_string(),
                artifact_command(
                    "verify-actual-independent-gate",
                    snapshot.revision,
                    artifacts::ArtifactCommand::Verify {
                        verification: artifacts::Verification {
                            candidate: candidate.clone(),
                            suite: artifacts::sha256(&serde_json::to_vec(&checks).unwrap()),
                            environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                            checks: vec![artifacts::Check {
                                command: checks[0].clone(),
                                exit_code: trusted_result.exit_code.unwrap(),
                                logs: log_digest.clone(),
                            }],
                            resolved_decisions: true,
                        },
                    },
                ),
            ),
            (
                &"assembled-worker".to_string(),
                artifact_command(
                    "refuse-submit-cancelled",
                    cancelled.revision,
                    artifacts::ArtifactCommand::Submit {
                        candidate: candidate.clone(),
                    },
                ),
            ),
        ] {
            assert_eq!(
                artifact_fixture_call(
                    artifacts.clone(),
                    (token).to_owned(),
                    command,
                    artifacts::ReferenceFault::None
                )
                .await
                .unwrap_err()
                .code,
                ErrorCode::DependencyUnmet
            );
        }
        let completed = f
            .broker
            .execute(
                &f.owner,
                envelope(
                    f.epoch,
                    "reconcile-measured-cancellation",
                    cancelled.revision,
                    Command::ReconcileAttempt {
                        task_id: f.task,
                        attempt_id: f.attempt,
                    },
                ),
            )
            .await
            .unwrap();
        assert_eq!(completed.disposition, "cancelled");
        assert_eq!(
            artifacts.snapshot(&f.owner, f.task).await.unwrap().status,
            "cancelled"
        );
        assert_eq!(
            artifact_fixture_call(
                artifacts.clone(),
                f.owner.to_owned(),
                artifact_command(
                    "refuse-terminal-cancelled-accept",
                    completed.revision,
                    artifacts::ArtifactCommand::Accept {
                        candidate_id: verified.candidate_id.unwrap(),
                        report_id: verified.report_id.unwrap()
                    }
                ),
                artifacts::ReferenceFault::None
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::DependencyUnmet
        );
        let out = evidence_directory("cancelled-measured-candidate");
        cancelled_sdk_drain_phase(
            client.clone(),
            bridge.clone(),
            r.clone(),
            gateway.clone(),
            f.owner.clone(),
            fenced_steer,
            out.clone(),
        )
        .await;
        retain_bytes(&out.join("cancelled-report-proof.json"),&serde_json::to_vec(&serde_json::json!({"candidate":candidate,"measured_result":trusted_result,"cancel":cancelled,"completed":completed})).unwrap());
        for id in [
            execution.id,
            continuation_execution.id,
            verifier_execution.id,
        ] {
            let view = supervisor.inspect(&f.owner, id).await.unwrap();
            if view.state == supervisor::ExecutionState::Stopped {
                supervisor.release(&f.owner, id).await.unwrap();
            } else {
                assert_eq!(
                    view.state,
                    supervisor::ExecutionState::Released,
                    "cleanup must not infer reusable or stopped effects from cancellation alone"
                );
            }
        }
        std::sync::Arc::try_unwrap(f)
            .unwrap_or_else(|_| panic!("repair task retained fixture"))
            .finish()
            .await;
        return;
    }
    if scenario == EngineeringScenario::ArtifactIntegrity {
        // These are the actual registered-SDK candidate and independently measured
        // verifier log. Removing acceptance-time reads would falsely accept them.
        let out = evidence_directory("actual-artifact-integrity");
        let before = artifacts.snapshot(&f.owner, f.task).await.unwrap();
        let accept = |key: &str, report: String| {
            artifact_command(
                key,
                verified.revision,
                artifacts::ArtifactCommand::Accept {
                    candidate_id: verified.candidate_id.clone().unwrap(),
                    report_id: report,
                },
            )
        };
        for (label, digest) in [
            ("candidate", candidate.artifact.as_str()),
            ("report-log", log_digest.as_str()),
        ] {
            for area in ["objects", "manifests"] {
                let path = f.root.join("artifacts").join(area).join(digest);
                let original = std::fs::read(&path).unwrap();
                for fault in ["missing", "corrupt"] {
                    if fault == "missing" {
                        std::fs::remove_file(&path).unwrap();
                    } else {
                        std::fs::write(&path, b"corrupt actual referenced evidence").unwrap();
                    }
                    let key = format!("integrity-{label}-{area}-{fault}");
                    let result = artifact_fixture_call(
                        artifacts.clone(),
                        f.owner.to_owned(),
                        accept(&key, verified.report_id.clone().unwrap()),
                        artifacts::ReferenceFault::None,
                    )
                    .await;
                    std::fs::write(&path, &original).unwrap();
                    assert!(
                        result.is_err(),
                        "{label}/{area}/{fault} must prevent acceptance"
                    );
                    let after = artifacts.snapshot(&f.owner, f.task).await.unwrap();
                    assert_eq!(after.revision, before.revision);
                    assert_eq!(after.status, "verifying");
                    retain_bytes(&out.join(format!("{key}.json")), &serde_json::to_vec(&serde_json::json!({"code":result.unwrap_err().code,"revision":after.revision,"status":after.status})).unwrap());
                }
            }
        }
        assert_eq!(
            artifact_fixture_call(
                artifacts.clone(),
                f.owner.to_owned(),
                accept("integrity-missing-report", Uuid::new_v4().to_string()),
                artifacts::ReferenceFault::None
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::NotFound
        );
        let report_id = verified.report_id.clone().unwrap();
        let original: serde_json::Value = db
            .query_one(
                "SELECT verification FROM artifact_reports WHERE id=$1",
                &[&report_id],
            )
            .await
            .unwrap()
            .get(0);
        let mut corrupt = original.clone();
        corrupt["candidate"]["tree"] =
            serde_json::json!("0000000000000000000000000000000000000000");
        let rejected = db
            .execute(
                "UPDATE artifact_reports SET verification=$2 WHERE id=$1",
                &[&report_id, &corrupt],
            )
            .await
            .unwrap_err();
        assert_eq!(
            rejected.as_db_error().unwrap().message(),
            "immutable record"
        );
        let unchanged: serde_json::Value = db
            .query_one(
                "SELECT verification FROM artifact_reports WHERE id=$1",
                &[&report_id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(
            unchanged, original,
            "canonical measured report cannot be rewritten"
        );
        assert_eq!(
            artifacts.snapshot(&f.owner, f.task).await.unwrap().revision,
            before.revision
        );
        retain_bytes(
            &out.join("restored-report.json"),
            &serde_json::to_vec(&original).unwrap(),
        );
    }
    mailbox.execute("assembled-verifier",mail(f.epoch,"yield-positive-gate",serde_json::json!({"kind":"yield_episode","episode":verifier_episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"positive gate retained; independently test failure discrimination","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    supervisor
        .release("assembled-verifier", verifier_execution.id)
        .await
        .unwrap();
    if matches!(
        scenario,
        EngineeringScenario::ChangedBase | EngineeringScenario::PostAllocationBaseChange
    ) {
        let out = evidence_directory("changed-git-base");
        let store = f.root.join("store");
        std::fs::write(store.join("src/base"), b"changed integration base\n").unwrap();
        git(&store, &["add", "src/base"]);
        git(
            &store,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "actual changed integration base",
            ],
        );
        let new_base = git(&store, &["rev-parse", "HEAD"]);
        assert_ne!(new_base, f.base);
        git(
            &store,
            &[
                "fetch",
                resumed_bundle.to_str().unwrap(),
                "refs/heads/resumed-candidate",
            ],
        );
        let new_checks=vec![vec!["/bin/sh".into(),"-c".into(),"test \"$(cat src/base)\" = 'changed integration base' && test \"$(cat src/result)\" = 'exact result' && test \"$(cat src/resumed)\" = 'resumed-from-retained-bytes' && cat src/base src/result src/resumed".into()]];
        let configured = artifact_fixture_call(
            artifacts.clone(),
            f.owner.to_owned(),
            artifact_command(
                "changed-base-policy",
                verified.revision,
                artifacts::ArtifactCommand::Configure {
                    verifier: f.verifier,
                    lead_acceptance: false,
                    base: new_base.clone(),
                    suite: artifacts::sha256(&serde_json::to_vec(&new_checks).unwrap()),
                    environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                    mandatory_commands: new_checks.clone(),
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
        assert_eq!(
            artifact_fixture_call(
                artifacts.clone(),
                f.owner.to_owned(),
                artifact_command(
                    "reject-original-gate-after-base-change",
                    configured.revision,
                    artifacts::ArtifactCommand::Accept {
                        candidate_id: verified.candidate_id.clone().unwrap(),
                        report_id: verified.report_id.clone().unwrap()
                    }
                ),
                artifacts::ReferenceFault::None
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::StaleRevision
        );
        let integration =
            artifacts::prepare_integration(&store, &candidate.commit, &new_base, &new_checks)
                .unwrap();
        assert_ne!(integration.commit, candidate.commit);
        assert!(integration.checks.iter().all(|c| c.exit_code == 0));
        let reference = "refs/heads/changed-base-integration";
        git(&store, &["update-ref", reference, &integration.commit]);
        let path = f.root.join("changed-base.bundle");
        git(
            &store,
            &["bundle", "create", path.to_str().unwrap(), reference],
        );
        let bytes = std::fs::read(path).unwrap();
        let digest = artifacts::sha256(&bytes);
        artifacts
            .publish_evidence("assembled-worker", f.task, &digest, &bytes)
            .await
            .unwrap();
        let new_candidate = artifacts::Candidate {
            commit: integration.commit.clone(),
            tree: integration.tree.clone(),
            base: new_base,
            spec_digest: f.spec.clone(),
            spec_revision: 2,
            generation: 1,
            artifact: digest,
            deliverables: vec!["src/result".into(), "src/resumed".into()],
            risks: vec![],
        };
        let submitted = artifact_fixture_call(
            artifacts.clone(),
            ("assembled-worker").to_owned(),
            artifact_command(
                "submit-changed-base-integration",
                configured.revision,
                artifacts::ArtifactCommand::Submit {
                    candidate: new_candidate.clone(),
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
        assert_eq!(
            artifact_fixture_call(
                artifacts.clone(),
                f.owner.to_owned(),
                artifact_command(
                    "reject-old-report-for-new-candidate",
                    submitted.revision,
                    artifacts::ArtifactCommand::Accept {
                        candidate_id: submitted.candidate_id.clone().unwrap(),
                        report_id: verified.report_id.clone().unwrap()
                    }
                ),
                artifacts::ReferenceFault::None
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::DependencyUnmet
        );
        let input = Uuid::new_v4();
        let episode = Uuid::new_v4();
        mailbox.execute("assembled-verifier",mail(f.epoch,"changed-base-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"verification","limit":4}))).await.unwrap();
        let packet = mailbox
            .record("assembled-verifier", mailbox::RecordKind::Input, input)
            .await
            .unwrap();
        mailbox.execute("assembled-verifier",mail(f.epoch,"changed-base-admit",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
        let allocation = supervisor
            .create_episode_from_candidate(
                "assembled-verifier",
                episode,
                &artifacts,
                supervisor::CreateFault::None,
            )
            .await;
        let allocated:Vec<serde_json::Value>=db.query("SELECT jsonb_build_object('id',w.id,'state',w.state,'manifest',w.manifest,'quarantine',w.quarantine_reason) FROM supervised_workspaces w JOIN episode_executions x ON x.execution_id=w.id WHERE x.episode_id=$1", &[&episode]).await.unwrap().into_iter().map(|r|r.get(0)).collect();
        retain_bytes(&out.join("allocation-boundary.json"),&serde_json::to_vec(&serde_json::json!({"candidate":new_candidate,"integration":integration,"original_admitted_base":f.base,"allocated":allocated})).unwrap());
        let new_execution = allocation.unwrap();
        assert_eq!(
            git(&new_execution.path, &["rev-parse", "HEAD"]),
            new_candidate.commit
        );
        assert_eq!(
            git(&new_execution.path, &["rev-parse", "HEAD^1"]),
            new_candidate.base
        );
        episode_runtime::EpisodeGateway::new(f.broker.clone())
            .prepare(
                "assembled-verifier",
                episode_runtime::StartRequest {
                    id: Uuid::new_v4(),
                    episode,
                    execution: new_execution.id,
                    runtime: episode_identity.clone(),
                    argv: new_checks[0].clone(),
                },
            )
            .await
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(8);
        let result = loop {
            if let Some(result) = supervisor
                .result("assembled-verifier", new_execution.id)
                .await
                .unwrap()
            {
                break result;
            }
            assert!(std::time::Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        assert_eq!(result.exit_code, Some(0));
        assert!(result.namespace_stopped);
        assert_eq!(result.candidate, Some(new_candidate.clone()));
        let view = supervisor
            .inspect("assembled-verifier", new_execution.id)
            .await
            .unwrap();
        let output = std::fs::read(&view.manifest.log_path).unwrap();
        assert_eq!(
            output,
            b"changed integration base\nexact result\nresumed-from-retained-bytes\n"
        );
        let evidence=serde_json::to_vec(&serde_json::json!({"trusted_monitor_result":result,"output":String::from_utf8(output).unwrap()})).unwrap();
        let digest = artifacts::sha256(&evidence);
        artifacts
            .publish_evidence("assembled-verifier", f.task, &digest, &evidence)
            .await
            .unwrap();
        let report = artifact_fixture_call(
            artifacts.clone(),
            ("assembled-verifier").to_owned(),
            artifact_command(
                "changed-base-measured-report",
                submitted.revision,
                artifacts::ArtifactCommand::Verify {
                    verification: artifacts::Verification {
                        candidate: new_candidate,
                        suite: artifacts::sha256(&serde_json::to_vec(&new_checks).unwrap()),
                        environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                        checks: vec![artifacts::Check {
                            command: new_checks[0].clone(),
                            exit_code: 0,
                            logs: digest,
                        }],
                        resolved_decisions: true,
                    },
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
        let accepted = artifact_fixture_call(
            artifacts.clone(),
            f.owner.to_owned(),
            artifact_command(
                "accept-fresh-base-gate",
                report.revision,
                artifacts::ArtifactCommand::Accept {
                    candidate_id: report.candidate_id.unwrap(),
                    report_id: report.report_id.unwrap(),
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
        assert!(accepted.accepted);
        retain_bytes(
            &out.join("integration.json"),
            &serde_json::to_vec(&integration).unwrap(),
        );
        retain_bytes(&out.join("fresh-measured-report.json"), &evidence);
        supervisor
            .release("assembled-verifier", new_execution.id)
            .await
            .unwrap();
        child.kill();
        supervisor
            .release("assembled-worker", execution.id)
            .await
            .unwrap();
        std::sync::Arc::try_unwrap(f)
            .unwrap_or_else(|_| panic!("repair task retained fixture"))
            .finish()
            .await;
        return;
    }
    mailbox.execute("assembled-verifier",mail(f.epoch,"negative-control-input",serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.verifier],"message_kind":"notice","text":"Execute nonzero negative control despite misleading success stdout","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let failed_pending = if scenario == EngineeringScenario::FailedGateDispatch {
        supervisor.release(&f.owner, execution.id).await.unwrap();
        Some(preallocate_failed_gate_worker(f.clone(), supervisor.clone(), artifacts.clone()).await)
    } else {
        None
    };
    let negative_input = Uuid::new_v4();
    let negative_episode = Uuid::new_v4();
    mailbox.execute("assembled-verifier",mail(f.epoch,"capture-negative-verification",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":negative_input,"attempt":f.attempt,"purpose":"verification","limit":4}))).await.unwrap();
    let packet = mailbox
        .record(
            "assembled-verifier",
            mailbox::RecordKind::Input,
            negative_input,
        )
        .await
        .unwrap();
    mailbox.execute("assembled-verifier",mail(f.epoch,"admit-negative-verification",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":negative_episode,"attempt":f.attempt,"purpose":"verification","input_packet":negative_input,"input_digest":packet["digest"]}))).await.unwrap();
    let negative_execution = supervisor
        .create_episode_from_candidate(
            "assembled-verifier",
            negative_episode,
            &artifacts,
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let negative_argv = vec![
        "/bin/sh".into(),
        "-c".into(),
        "printf 'gate_exit=0\\n'; exit 17".into(),
    ];
    episode_runtime::EpisodeGateway::new(f.broker.clone())
        .prepare(
            "assembled-verifier",
            episode_runtime::StartRequest {
                id: Uuid::new_v4(),
                episode: negative_episode,
                execution: negative_execution.id,
                runtime: episode_identity.clone(),
                argv: negative_argv.clone(),
            },
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let negative_result = loop {
        if let Some(result) = supervisor
            .result("assembled-verifier", negative_execution.id)
            .await
            .unwrap()
        {
            break result;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(negative_result.exit_code, Some(17));
    assert!(negative_result.namespace_stopped);
    let negative_view = supervisor
        .inspect("assembled-verifier", negative_execution.id)
        .await
        .unwrap();
    let negative_output = std::fs::read(&negative_view.manifest.log_path).unwrap();
    assert_eq!(negative_output, b"gate_exit=0\n");
    assert_eq!(negative_result.logs, artifacts::sha256(&negative_output));
    let negative_bytes=serde_json::to_vec(&serde_json::json!({"trusted_monitor_result":negative_result,"output":String::from_utf8(negative_output).unwrap()})).unwrap();
    let negative_digest = artifacts::sha256(&negative_bytes);
    artifacts
        .publish_evidence(
            "assembled-verifier",
            f.task,
            &negative_digest,
            &negative_bytes,
        )
        .await
        .unwrap();
    let mut failed_report = artifacts
        .snapshot("assembled-verifier", f.task)
        .await
        .unwrap()
        .reports
        .remove(0);
    failed_report.checks.push(artifacts::Check {
        command: negative_argv,
        exit_code: negative_result.exit_code.unwrap(),
        logs: negative_digest,
    });
    let mut forged_report = failed_report.clone();
    forged_report.checks.last_mut().unwrap().exit_code = 0;
    match artifact_fixture_call(
        artifacts.clone(),
        ("assembled-verifier").to_owned(),
        artifact_command(
            "forged-zero-for-measured-seventeen",
            verified.revision,
            artifacts::ArtifactCommand::Verify {
                verification: forged_report,
            },
        ),
        artifacts::ReferenceFault::None,
    )
    .await
    {
        Ok(forged) => assert!(
            artifact_fixture_call(
                artifacts.clone(),
                f.owner.to_owned(),
                artifact_command(
                    "deny-forged-zero-acceptance",
                    forged.revision,
                    artifacts::ArtifactCommand::Accept {
                        candidate_id: forged.candidate_id.unwrap(),
                        report_id: forged.report_id.unwrap()
                    }
                ),
                artifacts::ReferenceFault::None
            )
            .await
            .is_err(),
            "same measured failed Verifier execution cannot become Accepted by claiming exit_code zero"
        ),
        Err(error) => assert_eq!(error.code, ErrorCode::DependencyUnmet),
    }
    let mut failed = artifact_fixture_call(
        artifacts.clone(),
        ("assembled-verifier").to_owned(),
        artifact_command(
            "verify-measured-negative-control",
            verified.revision,
            artifacts::ArtifactCommand::Verify {
                verification: failed_report,
            },
        ),
        artifacts::ReferenceFault::None,
    )
    .await
    .unwrap();
    assert_eq!(
        artifact_fixture_call(
            artifacts.clone(),
            f.owner.to_owned(),
            artifact_command(
                "reject-nonzero-despite-stdout",
                failed.revision,
                artifacts::ArtifactCommand::Accept {
                    candidate_id: failed.candidate_id.clone().unwrap(),
                    report_id: failed.report_id.clone().unwrap()
                }
            ),
            artifacts::ReferenceFault::None
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::DependencyUnmet
    );
    assert_ne!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Task(f.task))
            .await
            .unwrap()
            .data["status"],
        "accepted"
    );
    if let Some((pending_episode, pending, packet)) = failed_pending {
        failed_gate_worker_phase(
            f.clone(),
            supervisor.clone(),
            artifacts.clone(),
            pending_episode,
            pending,
            packet,
        )
        .await;
        supervisor
            .release(&f.owner, negative_execution.id)
            .await
            .unwrap();
        child.kill();
        std::sync::Arc::try_unwrap(f)
            .unwrap_or_else(|_| panic!("failed gate phase retained fixture"))
            .finish()
            .await;
        return;
    }
    let accepted_pending = if scenario == EngineeringScenario::AcceptedTerminal {
        let charged = artifact_fixture_call(artifacts.clone(), f.owner.clone(), artifact_command(
            "authorize-terminal-fixture-pending-repair", failed.revision,
            artifacts::ArtifactCommand::Repair {
                candidate_id: failed.candidate_id.clone().unwrap(), report_id: failed.report_id.clone().unwrap(),
                reason: "authorize the preallocated Worker before testing eventual acceptance revocation".into(),
            }), artifacts::ReferenceFault::None).await.unwrap();
        failed.revision = charged.revision;
        supervisor.release(&f.owner, execution.id).await.unwrap();
        let fixture = f.clone();
        let supervisor = supervisor.clone();
        tokio::spawn(async move {
            let f = fixture;
            let mailbox = mailbox::MailboxService::new(f.broker.clone());
            supervisor
                .release(&f.owner, negative_execution.id)
                .await
                .unwrap();
            let pending = f.worker_episode("terminal-pending").await;
            let handle = supervisor
                .create_episode(
                    "assembled-worker",
                    pending,
                    vec![],
                    supervisor::CreateFault::None,
                )
                .await
                .unwrap();
            let record = mailbox
                .record(&f.owner, mailbox::RecordKind::Episode, pending)
                .await
                .unwrap();
            let packet_id: Uuid = serde_json::from_value(record["input_packet"].clone()).unwrap();
            let packet = mailbox
                .record(&f.owner, mailbox::RecordKind::Input, packet_id)
                .await
                .unwrap();
            (pending, handle, packet_id, packet)
        })
        .await
        .unwrap()
        .into()
    } else {
        None
    };
    if repair_budget {
        let out = evidence_directory("repair-budget");
        retain_bytes(
            &out.join("failed-verifier-execution.json"),
            &serde_json::to_vec(&negative_result).unwrap(),
        );
        retain_native(&episode_runtime_root, &out, "pre-repair");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = http::router_with_artifacts(f.broker.clone(), artifacts.clone());
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();
        mailbox.execute("assembled-verifier",mail(f.epoch,"yield-initial-failed-repair",serde_json::json!({"kind":"yield_episode","episode":negative_episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"independent failed execution retained for one repair decision","artifacts":[{"task":f.task,"digest":candidate.artifact}]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
        supervisor
            .release("assembled-verifier", negative_execution.id)
            .await
            .unwrap();
        supervisor
            .release("assembled-worker", execution.id)
            .await
            .unwrap();
        let endpoint = format!("http://{address}/v1/artifacts/commands");
        let mut revision = failed.revision;
        check_uncharged_repair_boundary(
            std::sync::Arc::clone(&f),
            mailbox.clone(),
            client.clone(),
            endpoint.clone(),
            candidate.clone(),
            out.clone(),
            revision,
        )
        .await
        .expect("uncharged repair boundary task failed");
        for index in 0..4 {
            let body = serde_json::json!({"key":format!("bounded-repair-{index}"),"task":f.task,"expected_revision":revision,"generation":1,"recovery_epoch":f.epoch,"command":{"kind":"repair","candidate_id":failed.candidate_id,"report_id":failed.report_id,"reason":"independent measured gate failed with exit17"}});
            let response = client
                .post(&endpoint)
                .bearer_auth(&f.owner)
                .json(&body)
                .send()
                .await
                .unwrap();
            let status = response.status();
            let bytes = response.bytes().await.unwrap();
            retain_bytes(&out.join(format!("repair-{index}-response.json")), &bytes);
            if index == 3 {
                assert!(!status.is_success());
                let rejected: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(rejected["code"], "budget_exhausted");
                break;
            }
            assert!(
                status.is_success(),
                "public task.repair must admit exact failed verified candidate: HTTP{status}, {}",
                String::from_utf8_lossy(&bytes)
            );
            let result: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            revision = result["revision"].as_i64().unwrap();
            let duplicate = client
                .post(&endpoint)
                .bearer_auth(&f.owner)
                .json(&body)
                .send()
                .await
                .unwrap();
            assert!(duplicate.status().is_success());
            assert_eq!(duplicate.json::<serde_json::Value>().await.unwrap(), result);
            let denied = client
                .post(&endpoint)
                .bearer_auth("assembled-worker")
                .json(&body)
                .send()
                .await
                .unwrap();
            assert!(!denied.status().is_success());
            assert_eq!(
                denied.json::<serde_json::Value>().await.unwrap()["code"],
                "unauthorized"
            );
            assert_eq!(
                f.broker
                    .snapshot(&f.owner, SnapshotQuery::Task(f.task))
                    .await
                    .unwrap()
                    .data["status"],
                "repair_required"
            );
            if index < 2 {
                let mut alias = body.clone();
                alias["key"] = serde_json::json!(format!("same-failure-new-key-{index}"));
                alias["expected_revision"] = serde_json::json!(revision);
                let duplicate_failure = client
                    .post(&endpoint)
                    .bearer_auth(&f.owner)
                    .json(&alias)
                    .send()
                    .await
                    .unwrap();
                assert!(!duplicate_failure.status().is_success());
                assert_eq!(
                    duplicate_failure.json::<serde_json::Value>().await.unwrap()["code"],
                    "dependency_unmet",
                    "one failed report authorizes only one repair, even with a different key"
                );
                let repair_fixture = std::sync::Arc::clone(&f);
                let repair_artifacts = artifacts.clone();
                let repair_supervisor = supervisor.clone();
                let repair_runtime = episode_identity.clone();
                let repair_checks = checks.clone();
                let repair_out = out.clone();
                let repair_candidate = candidate.clone();
                // Await every repair effect on an independent poll stack.
                (candidate, failed) = tokio::spawn(async move {
                    let context = RepairContext {
                        fixture: &repair_fixture,
                        artifacts: &repair_artifacts,
                        supervisor: &repair_supervisor,
                        runtime: &repair_runtime,
                        checks: &repair_checks,
                        evidence_root: &repair_out,
                    };
                    perform_actual_sdk_repair(&context, &repair_candidate, index + 1).await
                })
                .await
                .expect("actual SDK repair task failed");
                revision = failed.revision;
                let stale = client
                    .post(&endpoint)
                    .bearer_auth(&f.owner)
                    .json(&body)
                    .send()
                    .await
                    .unwrap();
                assert!(!stale.status().is_success());
                assert_eq!(
                    stale.json::<serde_json::Value>().await.unwrap()["code"],
                    "stale_revision",
                    "cached repair cannot authorize a changed current candidate/report"
                );
            }
        }
        let controls = kernel::Kernel::new(f.broker.clone())
            .status(&f.owner)
            .await
            .unwrap();
        assert_eq!(
            controls["missions"][0]["consumed_repairs"], 3,
            "duplicate delivery and refused fourth repair cannot charge again"
        );
        let snapshot = artifacts.snapshot(&f.owner, f.task).await.unwrap();
        assert_eq!(snapshot.revision, revision);
        assert_eq!(snapshot.status, "repair_required");
        server.abort();
        assert!(server.await.unwrap_err().is_cancelled());
    } else {
        // An earlier successful report cannot erase the subsequently measured
        // failed gate. The same candidate needs a fresh independent SDK gate.
        let old_report = artifacts
            .snapshot("assembled-verifier", f.task)
            .await
            .unwrap()
            .reports
            .into_iter()
            .find(|report| report.checks.iter().all(|check| check.exit_code == 0))
            .unwrap();
        assert_eq!(
            artifact_fixture_call(
                artifacts.clone(),
                "assembled-verifier".into(),
                artifact_command(
                    "old-success-cannot-clear-current-failure",
                    failed.revision,
                    artifacts::ArtifactCommand::Verify {
                        verification: old_report
                    }
                ),
                artifacts::ReferenceFault::None
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::DependencyUnmet
        );
        verified = fresh_same_candidate_verifier(FreshVerifierContext {
            fixture: f.clone(),
            supervisor: supervisor.clone(),
            artifacts: artifacts.clone(),
            runtime: episode_identity.clone(),
            candidate: candidate.clone(),
            checks: checks.clone(),
            negative_episode,
            negative_execution: negative_execution.id,
        })
        .await;
        failed.revision = verified.revision;
        let accepted = artifact_fixture_call(
            artifacts.clone(),
            f.owner.to_owned(),
            artifact_command(
                "explicit-owner-accept",
                failed.revision,
                artifacts::ArtifactCommand::Accept {
                    candidate_id: verified.candidate_id.unwrap(),
                    report_id: verified.report_id.unwrap(),
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
        assert!(accepted.accepted);
        if let Some((pending, handle, packet_id, packet)) = accepted_pending {
            let fixture = f.clone();
            let supervisor_task = supervisor.clone();
            let violations = tokio::spawn(async move {
            let f = fixture;
            let supervisor = supervisor_task;
            let mailbox = mailbox::MailboxService::new(f.broker.clone());
            let before = kernel::Kernel::new(f.broker.clone()).status(&f.owner).await.unwrap();
            let commands = [
                mail(f.epoch,"terminal-pending-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":packet_id,"attempt":f.attempt,"purpose":"implementation","limit":4})),
                mail(f.epoch,"terminal-new-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":Uuid::new_v4(),"attempt":f.attempt,"purpose":"implementation","limit":4})),
                mail(f.epoch,"terminal-pending-admit",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":pending,"attempt":f.attempt,"purpose":"implementation","input_packet":packet_id,"input_digest":packet["digest"]})),
                mail(f.epoch,"terminal-new-admit",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":Uuid::new_v4(),"attempt":f.attempt,"purpose":"implementation","input_packet":packet_id,"input_digest":packet["digest"]}))
            ];
            let mut violations = Vec::new();
            for (index,command) in commands.into_iter().enumerate() {
                if mailbox.execute("assembled-worker",command).await.is_ok() {violations.push(format!("mailbox-{index}"));}
            }
            if supervisor.create_episode("assembled-worker", pending, vec![], supervisor::CreateFault::None).await.is_ok() {violations.push("cached-allocation".into());}
            if supervisor.start("assembled-worker",handle.id,vec!["/bin/sh".into(),"-c".into(),"printf forbidden > src/after-accepted".into()],30).await.is_ok() {
                violations.push("prepared-launch".into());
                let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(8);
                loop {
                    if supervisor.inspect(&f.owner,handle.id).await.unwrap().state==supervisor::ExecutionState::Stopped {break;}
                    assert!(tokio::time::Instant::now()<deadline);
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
                supervisor.release(&f.owner,handle.id).await.unwrap();
            }
            let after = kernel::Kernel::new(f.broker.clone()).status(&f.owner).await.unwrap();
            for field in ["used_capacity","admitted_attempts","admitted_episodes","consumed_repairs"] {
                assert_eq!(before["missions"][0][field],after["missions"][0][field],"terminal refusal cannot consume or refund {field}");
            }
            assert!(!handle.path.join("src/after-accepted").exists());
            let out=evidence_directory("accepted-terminal");
            retain_bytes(&out.join("accepted-terminal-dispatch.json"),&serde_json::to_vec(&serde_json::json!({"accepted":accepted,"before":before,"after":after,"violations":violations,"execution":handle.id,"forbidden_output":std::fs::read(handle.path.join("src/after-accepted")).ok()})).unwrap());
            violations
            }).await.unwrap();
            child.kill();
            if supervisor
                .inspect(&f.owner, execution.id)
                .await
                .unwrap()
                .state
                == supervisor::ExecutionState::Stopped
            {
                supervisor.release(&f.owner, execution.id).await.unwrap();
            }
            std::sync::Arc::try_unwrap(f)
                .unwrap_or_else(|_| panic!("retained fixture"))
                .finish()
                .await;
            assert!(
                violations.is_empty(),
                "accepted Task still permits {} dispatch mutations",
                violations.len()
            );
            return;
        }
        assert_eq!(
            f.broker
                .snapshot(&f.owner, SnapshotQuery::Task(f.task))
                .await
                .unwrap()
                .data["status"],
            "accepted"
        );
    }
    if !repair_budget
        && supervisor
            .inspect(&f.owner, negative_execution.id)
            .await
            .unwrap()
            .state
            == supervisor::ExecutionState::Stopped
    {
        supervisor
            .release("assembled-verifier", negative_execution.id)
            .await
            .unwrap();
    }
    assert_eq!(
        std::fs::read_dir(&runtime_root)
            .unwrap()
            .filter(|e| e
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|x| x == "effect"))
            .count(),
        2,
        "actual Create and Start effects, never native-deduplicated"
    );
    child.kill();
    if !repair_budget {
        supervisor
            .release("assembled-worker", execution.id)
            .await
            .unwrap();
    }
    std::sync::Arc::try_unwrap(f)
        .unwrap_or_else(|_| panic!("repair task retained fixture"))
        .finish()
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn episode_native_lost_reply_holds_physical_capacity() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("native-lost-reply").await;
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
    let root = f.root.join("native-lost-reply");
    std::fs::create_dir(&root).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-inprocess-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "owned-async-os".into(),
        environment: "bwrap-private-git".into(),
    };
    let gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    let id = Uuid::new_v4();
    gateway
        .prepare(
            "assembled-worker",
            episode_runtime::StartRequest {
                id,
                episode,
                execution: execution.id,
                runtime: identity.clone(),
                argv: vec!["/bin/true".into()],
            },
        )
        .await
        .unwrap();
    let mut runtime = episode_native::NativeOsRuntime {
        root,
        identity,
        supervisor: supervisor.clone(),
        service: f.owner.clone(),
        lose_reply: true,
        hold_reply: false,
    };
    assert_eq!(
        gateway
            .dispatch(&f.owner, id, &mut runtime)
            .await
            .unwrap()
            .state,
        OperationState::OutcomeUnknown
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    loop {
        if supervisor
            .result("assembled-worker", execution.id)
            .await
            .unwrap()
            .is_some()
        {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let release = supervisor.release("assembled-worker", execution.id).await;
    let capacity = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data["used_capacity"]
        .clone();
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    Bridge::new(f.broker.clone())
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    mailbox.execute("assembled-worker",mail(f.epoch,"yield-unknown-native",serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"local capsule stopped; native outcome still unknown","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    mailbox.execute("assembled-worker",mail(f.epoch,"unknown-continuation-notice",serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":"New input cannot override unknown prior Start","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let next_input = Uuid::new_v4();
    mailbox.execute("assembled-worker",mail(f.epoch,"capture-unknown-continuation",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":next_input,"attempt":f.attempt,"purpose":"implementation","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-worker", mailbox::RecordKind::Input, next_input)
        .await
        .unwrap();
    let before = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data;
    let continuation_admission=mailbox.execute("assembled-worker",mail(f.epoch,"deny-unknown-continuation",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":Uuid::new_v4(),"attempt":f.attempt,"purpose":"implementation","input_packet":next_input,"input_digest":packet["digest"]}))).await;
    let after = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data;
    if continuation_admission.is_ok() {
        // The original capsule is stopped; the incorrectly admitted successor
        // has no physical execution. Preserve the exact product red before
        // downstream reconciliation encounters conflicting active episodes.
        f.finish().await;
        panic!(
            "unknown prior Start admitted a successor and charged capacity: before={before}, after={after}"
        );
    }
    let effect = std::fs::read_dir(&runtime.root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|x| x == "effect"))
        .unwrap();
    let hidden = effect.with_extension("temporarily-unavailable");
    std::fs::rename(&effect, &hidden).unwrap();
    assert_eq!(
        gateway
            .dispatch(&f.owner, id, &mut runtime)
            .await
            .unwrap()
            .state,
        OperationState::OutcomeUnknown,
        "missing native record is not absence proof"
    );
    std::fs::rename(&hidden, &effect).unwrap();
    let ambiguous = runtime.root.join("ambiguous.effect");
    std::fs::copy(&effect, &ambiguous).unwrap();
    assert_eq!(
        gateway
            .dispatch(&f.owner, id, &mut runtime)
            .await
            .unwrap()
            .state,
        OperationState::OutcomeUnknown,
        "multiple native records cannot select a convenient receipt"
    );
    std::fs::remove_file(&ambiguous).unwrap();
    let outside = f.root.join("outside-native-receipt.json");
    std::fs::copy(&effect, &outside).unwrap();
    std::fs::remove_file(&effect).unwrap();
    std::os::unix::fs::symlink(&outside, &effect).unwrap();
    let symlink_observation = gateway.dispatch(&f.owner, id, &mut runtime).await.unwrap();
    std::fs::remove_file(&effect).unwrap();
    std::fs::copy(&outside, &effect).unwrap();
    let observed = gateway.dispatch(&f.owner, id, &mut runtime).await.unwrap();
    let invocations = std::fs::read_dir(&runtime.root)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|x| x == "invocation")
        })
        .count();
    assert_eq!(
        invocations, 1,
        "every observation must remain query-only with native deduplication disabled"
    );
    assert_eq!(
        observed.state,
        OperationState::Confirmed,
        "retry must independently observe the existing effect, never invoke Start again"
    );
    let (usage_db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let usage_rows: Vec<serde_json::Value> = usage_db.query("SELECT jsonb_build_object('id',id,'state',state,'usage',usage) FROM episode_runtime_operations", &[]).await.unwrap().into_iter().map(|r| r.get(0)).collect();
    eprintln!("[release-diagnostic] episode operations={usage_rows:?}");
    assert_eq!(
        gateway.inspect(&f.owner, id).await.unwrap().operation.usage,
        episode_runtime::UsageObservation::Unknown,
        "unregistered direct observation confirms an effect, never measured usage"
    );
    assert_eq!(
        supervisor
            .release("assembled-worker", execution.id)
            .await
            .unwrap_err()
            .code,
        ErrorCode::DependencyUnmet,
        "confirmed native effect with unknown usage retains its reservation"
    );
    let released_capacity = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data["used_capacity"]
        .clone();
    f.finish().await;
    assert_eq!(
        continuation_admission.unwrap_err().code,
        ErrorCode::DependencyUnmet,
        "unknown prior Start must block continuation admission before its budget charge"
    );
    assert_eq!(before["admitted_episodes"], after["admitted_episodes"]);
    assert_eq!(before["used_capacity"], after["used_capacity"]);
    assert_eq!(
        symlink_observation.state,
        OperationState::OutcomeUnknown,
        "query-only native observation must not follow a receipt symlink outside its owned store"
    );
    assert_eq!(
        release.unwrap_err().code,
        ErrorCode::DependencyUnmet,
        "a stopped capsule cannot refund an unresolved native Start"
    );
    assert_eq!(capacity, 1);
    assert_eq!(released_capacity, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn public_operator_inspects_exact_episode_journal_without_execution_ownership() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("operator-journal").await;
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
    let id = Uuid::new_v4();
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-owned-os".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: f.root.join("operator-runtime").to_str().unwrap().into(),
        endpoint: "owned-local".into(),
        environment: "bwrap-private-git".into(),
    };
    episode_runtime::EpisodeGateway::new(f.broker.clone())
        .prepare(
            "assembled-worker",
            episode_runtime::StartRequest {
                id,
                episode,
                execution: execution.id,
                runtime: identity,
                argv: vec!["/bin/true".into()],
            },
        )
        .await
        .unwrap();
    let inspect = |token: String| {
        let mut command = Process::new(env!("CARGO_BIN_EXE_team-control-plane"));
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("TEAM_CONTROL_PLANE_DSN", &f.dsn)
            .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", f.epoch.to_string())
            .env("TEAM_CONTROL_PLANE_TOKEN", token)
            .args(["inspect-episode-operation", &id.to_string()]);
        bounded_operator_cli(command)
    };
    let (output, timeout) = inspect(f.owner.clone()).await;
    let (denied, denied_timeout) = inspect("assembled-verifier".into()).await;
    let (wrong, wrong_timeout) = inspect("wrong-token".into()).await;
    assert!(!timeout && !denied_timeout && !wrong_timeout);
    supervisor
        .cancel("assembled-worker", execution.id)
        .await
        .unwrap();
    supervisor
        .reconcile("assembled-worker", execution.id)
        .await
        .unwrap();
    supervisor
        .release("assembled-worker", execution.id)
        .await
        .unwrap();
    f.finish().await;
    assert!(
        output.status.success(),
        "independent authenticated Owner must inspect exact durable operation: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let view: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(view["operation"]["id"], id.to_string());
    assert_eq!(view["operation"]["state"], "prepared");
    assert_eq!(
        view["prepared"]["start"]["execution"],
        execution.id.to_string()
    );
    assert_eq!(view["evidence_more"], false);
    assert!(!denied.status.success());
    assert!(denied.stdout.is_empty());
    assert!(!wrong.status.success());
    assert!(wrong.stdout.is_empty());
}

#[tokio::test]
async fn public_all_episode_roles_exhaust_the_approved_finite_budget_without_double_charge() {
    for (token, purpose) in [
        ("assembled-lead", "planning"),
        ("assembled-worker", "implementation"),
        ("assembled-verifier", "verification"),
    ] {
        let f = Fixture::new().await;
        let mailbox = mailbox::MailboxService::new(f.broker.clone());
        if purpose == "verification" {
            // Genuine private Git candidate and immutable publication: no verdict SQL.
            let store = f.root.join("store");
            std::fs::write(store.join("src/result"), b"budget fixture candidate\n").unwrap();
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
                    "budget candidate",
                ],
            );
            let bundle = f.root.join("budget.bundle");
            git(
                &store,
                &["bundle", "create", bundle.to_str().unwrap(), "HEAD"],
            );
            let bytes = std::fs::read(bundle).unwrap();
            let digest = artifacts::sha256(&bytes);
            let service =
                artifacts::ArtifactService::open(f.broker.clone(), &f.root.join("artifacts"))
                    .await
                    .unwrap();
            let envelope = |key: &str, revision, command| artifacts::ArtifactEnvelope {
                key: key.into(),
                task: f.task,
                expected_revision: revision,
                generation: 1,
                recovery_epoch: f.epoch,
                command,
            };
            let revision = f
                .broker
                .snapshot(&f.owner, SnapshotQuery::Task(f.task))
                .await
                .unwrap()
                .revision;
            let configured = service
                .execute(
                    &f.owner,
                    envelope(
                        "budget-policy",
                        revision,
                        artifacts::ArtifactCommand::Configure {
                            verifier: f.verifier,
                            lead_acceptance: false,
                            base: f.base.clone(),
                            suite: artifacts::sha256(b"budget-suite"),
                            environment: artifacts::sha256(b"budget-env"),
                            mandatory_commands: vec![vec!["/bin/true".into()]],
                        },
                    ),
                    artifacts::ReferenceFault::None,
                )
                .await
                .unwrap();
            service
                .publish_evidence("assembled-worker", f.task, &digest, &bytes)
                .await
                .unwrap();
            service
                .execute(
                    "assembled-worker",
                    envelope(
                        "budget-submit",
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
        }
        Bridge::new(f.broker.clone())
            .register(
                &f.owner,
                if purpose == "planning" {
                    AggregateKind::Mission
                } else {
                    AggregateKind::Task
                },
                if purpose == "planning" {
                    f.mission
                } else {
                    f.task
                },
                0,
            )
            .await
            .unwrap();
        let principal = match purpose {
            "planning" => f.lead,
            "implementation" => f.worker,
            _ => f.verifier,
        };
        let attempt = if purpose == "planning" {
            None
        } else {
            Some(f.attempt)
        };
        let allowed = if purpose == "implementation" { 8 } else { 7 };
        for index in 0..=allowed {
            mailbox.execute(token,mail(f.epoch,&format!("budget-notice-{index}"),serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[principal],"message_kind":"notice","text":format!("new finite input {index}"),"reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
            let input = Uuid::new_v4();
            let episode = Uuid::new_v4();
            mailbox.execute(token,mail(f.epoch,&format!("budget-capture-{index}"),serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":attempt,"purpose":purpose,"limit":4}))).await.unwrap();
            let packet = mailbox
                .record(token, mailbox::RecordKind::Input, input)
                .await
                .unwrap();
            let before = f
                .broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data;
            let admitted=mailbox.execute(token,mail(f.epoch,&format!("budget-admit-{index}"),serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":attempt,"purpose":purpose,"input_packet":input,"input_digest":packet["digest"]}))).await;
            if index == allowed {
                assert_eq!(
                    admitted.unwrap_err().code,
                    ErrorCode::BudgetExhausted,
                    "{purpose} obeys the same eight-episode Mission limit"
                );
                let after = f
                    .broker
                    .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                    .await
                    .unwrap()
                    .data;
                assert_eq!(before["used_capacity"], after["used_capacity"]);
                assert_eq!(before["admitted_episodes"], after["admitted_episodes"]);
                assert_eq!(after["admitted_episodes"], 8);
                break;
            }
            admitted.unwrap();
            let current = f
                .broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data;
            if purpose == "implementation" && index == 0 {
                assert_eq!(
                    current["admitted_episodes"], 1,
                    "imported first Worker episode uses existing assignment charge"
                );
            }
            // These episodes have no physical execution or runtime journal, so
            // public yield proves there is no outstanding physical slot to hold.
            mailbox.execute(token,mail(f.epoch,&format!("budget-yield-{index}"),serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"bounded no-work admission accounting","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
        }
        println!(
            "completed public finite role budget {purpose}: limit8, no rejected admission charge"
        );
        f.finish().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_and_episode_public_dispatch_share_the_same_physical_store_exclusion() {
    for legacy_first in [true, false] {
        let f = Fixture::new().await;
        let episode = f.worker_episode("shared-store").await;
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
        let root = f.root.join("shared-native-store");
        std::fs::create_dir(&root).unwrap();
        let identity = RuntimeIdentity {
            runtime_kind: "kernel16-owned-os".into(),
            runtime_version: "1".into(),
            deployment: f.epoch.to_string(),
            thread_store: root.to_str().unwrap().into(),
            endpoint: "same-actual-store".into(),
            environment: "bwrap-private-git".into(),
        };
        let legacy = Gateway::new(f.broker.clone());
        let binding = Uuid::new_v4();
        let create = Uuid::new_v4();
        let episodes = episode_runtime::EpisodeGateway::new(f.broker.clone());
        let start = Uuid::new_v4();
        episodes
            .prepare(
                "assembled-worker",
                episode_runtime::StartRequest {
                    id: start,
                    episode,
                    execution: execution.id,
                    runtime: identity.clone(),
                    argv: vec!["/bin/true".into()],
                },
            )
            .await
            .unwrap();
        let mut native = episode_native::NativeOsRuntime {
            root: root.clone(),
            identity: identity.clone(),
            supervisor: supervisor.clone(),
            service: f.owner.clone(),
            lose_reply: false,
            hold_reply: false,
        };
        let mut old = SupervisedRuntime {
            root: root.clone(),
            identity: identity.clone(),
            dsn: f.dsn.clone(),
            epoch: f.epoch,
            bearer: f.owner.clone(),
            workspaces: f.root.join("workspaces"),
            drop_start_reply: false,
            unmeasured_start: false,
        };
        if legacy_first {
            legacy
                .bind(&f.owner, binding, f.admission, &identity, None, 1)
                .await
                .unwrap();
            legacy
                .prepare(
                    &f.owner,
                    Intent {
                        id: create,
                        admission_id: f.admission,
                        binding_id: binding,
                        generation: 1,
                        kind: OperationKind::Create,
                        arguments: serde_json::json!({}),
                    },
                )
                .await
                .unwrap();
            assert_eq!(
                legacy
                    .dispatch(&f.owner, create, &mut old)
                    .await
                    .unwrap()
                    .state,
                OperationState::Confirmed
            );
            assert_eq!(
                episodes
                    .dispatch(&f.owner, start, &mut native)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet
            );
            supervisor
                .cancel("assembled-worker", execution.id)
                .await
                .unwrap();
            supervisor
                .reconcile("assembled-worker", execution.id)
                .await
                .unwrap();
        } else {
            let kernel = kernel::Kernel::new(f.broker.clone());
            kernel
                .pause(
                    &f.owner,
                    f.mission,
                    "pause-direct-episode-claim",
                    0,
                    true,
                    "authoritative direct API pause fence",
                )
                .await
                .unwrap();
            assert_eq!(
                episodes
                    .dispatch(&f.owner, start, &mut native)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnmet
            );
            assert_eq!(
                std::fs::read_dir(&root).unwrap().count(),
                0,
                "paused direct episode dispatch produces no invocation or workload"
            );
            assert_eq!(
                episodes
                    .inspect(&f.owner, start)
                    .await
                    .unwrap()
                    .operation
                    .state,
                OperationState::Prepared
            );
            kernel
                .pause(
                    &f.owner,
                    f.mission,
                    "resume-direct-episode-claim",
                    1,
                    false,
                    "explicit current revision resume",
                )
                .await
                .unwrap();
            assert_eq!(
                episodes
                    .dispatch(&f.owner, start, &mut native)
                    .await
                    .unwrap()
                    .state,
                OperationState::Confirmed
            );
            assert_eq!(
                legacy
                    .bind(&f.owner, binding, f.admission, &identity, None, 1)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::TemporarilyUnavailable,
                "accepted legacy permanent store fence rejects and quarantines a foreign controller"
            );
            let deadline = std::time::Instant::now() + Duration::from_secs(8);
            loop {
                if supervisor
                    .result("assembled-worker", execution.id)
                    .await
                    .unwrap()
                    .is_some()
                {
                    break;
                }
                assert!(std::time::Instant::now() < deadline);
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
        let invocations = std::fs::read_dir(&root)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|s| s == "invocation")
            })
            .count();
        let effects = std::fs::read_dir(&root)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|s| s == "effect")
            })
            .count();
        assert_eq!(
            (invocations, effects),
            (1, 1),
            "distinct protocol mutexes cannot grant a second controller for the same actual native store"
        );
        let (usage_db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move { connection.await.unwrap() });
        let usage_rows: Vec<serde_json::Value> = usage_db.query("SELECT jsonb_build_object('id',id,'state',state,'usage',usage) FROM episode_runtime_operations", &[]).await.unwrap().into_iter().map(|r| r.get(0)).collect();
        eprintln!("[release-diagnostic] episode operations={usage_rows:?}");
        let legacy_rows: Vec<serde_json::Value> = usage_db.query("SELECT jsonb_build_object('id',id,'state',state,'usage',usage) FROM gateway_operations", &[]).await.unwrap().into_iter().map(|r| r.get(0)).collect();
        eprintln!(
            "[release-diagnostic] legacy_first={legacy_first}, legacy operations={legacy_rows:?}"
        );
        assert_eq!(
            episodes
                .inspect(&f.owner, start)
                .await
                .unwrap()
                .operation
                .usage,
            episode_runtime::UsageObservation::Unknown
        );

        assert_eq!(
            supervisor
                .release("assembled-worker", execution.id)
                .await
                .unwrap_err()
                .code,
            ErrorCode::DependencyUnmet,
            "the actual unregistered controller retains unknown-usage capacity"
        );
        assert_eq!(
            f.broker
                .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
                .await
                .unwrap()
                .data["used_capacity"],
            1
        );
        println!(
            "completed actual shared-store exclusion legacy_first={legacy_first}, invocations=1 effects=1"
        );
        f.finish().await;
    }
}

#[tokio::test]
async fn runtime_registration_authenticates_before_observing_process_or_filesystem() {
    let f = Fixture::new().await;
    let supervisor = f.supervisor().await;
    let root = f.root.join("nonexistent-runtime-root");
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-inprocess-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "owned-async-os".into(),
        environment: "bwrap-private-git".into(),
    };
    let native = episode_native::NativeOsRuntime {
        root: root.clone(),
        identity,
        supervisor,
        service: f.owner.clone(),
        lose_reply: false,
        hold_reply: false,
    };
    let denied = episode_runtime::EpisodeGateway::new(f.broker.clone())
        .register_runtime("wrong-token", &native)
        .await;
    assert!(!root.exists());
    f.finish().await;
    assert_eq!(
        denied.unwrap_err().code,
        ErrorCode::Unauthorized,
        "registration must authenticate before process/root observation, even with invalid physical contract"
    );
}

struct RepairContext<'a> {
    fixture: &'a Fixture,
    artifacts: &'a artifacts::ArtifactService,
    supervisor: &'a supervisor::Supervisor,
    runtime: &'a RuntimeIdentity,
    checks: &'a [Vec<String>],
    evidence_root: &'a std::path::Path,
}
async fn perform_actual_sdk_repair(
    context: &RepairContext<'_>,
    prior: &artifacts::Candidate,
    round: usize,
) -> (artifacts::Candidate, artifacts::ArtifactResult) {
    let RepairContext {
        fixture: f,
        artifacts,
        supervisor,
        runtime,
        checks,
        evidence_root: out,
    } = *context;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let command = |key: &str, revision, command| artifacts::ArtifactEnvelope {
        key: key.into(),
        task: f.task,
        expected_revision: revision,
        generation: 1,
        recovery_epoch: f.epoch,
        command,
    };
    mailbox.execute("assembled-worker",mail(f.epoch,&format!("repair-worker-notice-{round}"),serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":format!("Actual repair round {round} on retained candidate"),"reply_to":null,"attention":"normal","artifacts":[{"task":f.task,"digest":prior.artifact}]}))).await.unwrap();
    let worker = f
        .worker_episode(&format!("actual-repair-worker-{round}"))
        .await;
    let execution = supervisor
        .create_episode_from_candidate(
            "assembled-worker",
            worker,
            artifacts,
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    assert_eq!(git(&execution.path, &["rev-parse", "HEAD"]), prior.commit);
    let argv = vec![
        "/bin/sh".into(),
        "-c".into(),
        format!(
            "test -f /episode-input.json && printf 'actual-repair-round-{round}\\n' > src/resumed"
        ),
    ];
    episode_runtime::EpisodeGateway::new(f.broker.clone())
        .prepare(
            "assembled-worker",
            episode_runtime::StartRequest {
                id: Uuid::new_v4(),
                episode: worker,
                execution: execution.id,
                runtime: runtime.clone(),
                argv,
            },
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let result = loop {
        if let Some(r) = supervisor
            .result("assembled-worker", execution.id)
            .await
            .unwrap()
        {
            break r;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "actual SDK must dispatch repair Worker round{round}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(result.exit_code, Some(0));
    assert!(result.namespace_stopped);
    assert_eq!(
        std::fs::read(execution.path.join("src/resumed")).unwrap(),
        format!("actual-repair-round-{round}\n").as_bytes()
    );
    retain_bytes(
        &out.join(format!("repair-{round}-worker-result.json")),
        &serde_json::to_vec(&result).unwrap(),
    );
    git(&execution.path, &["add", "src/resumed"]);
    let tree = git(&execution.path, &["write-tree"]);
    let commit = git(
        &execution.path,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit-tree",
            &tree,
            "-p",
            &f.base,
            "-p",
            &prior.commit,
            "-m",
            &format!("actual supervised repair{round}"),
        ],
    );
    let reference = format!("refs/heads/repair-{round}");
    git(&execution.path, &["update-ref", &reference, &commit]);
    let bundle = f.root.join(format!("repair-{round}.bundle"));
    git(
        &execution.path,
        &["bundle", "create", bundle.to_str().unwrap(), &reference],
    );
    let bytes = std::fs::read(bundle).unwrap();
    let digest = artifacts::sha256(&bytes);
    artifacts
        .publish_evidence("assembled-worker", f.task, &digest, &bytes)
        .await
        .unwrap();
    let candidate = artifacts::Candidate {
        commit,
        tree,
        base: f.base.clone(),
        spec_digest: f.spec.clone(),
        spec_revision: 2,
        generation: 1,
        artifact: digest,
        deliverables: vec!["src/result".into(), "src/resumed".into()],
        risks: vec![],
    };
    assert_ne!(candidate.commit, prior.commit);
    assert_ne!(candidate.artifact, prior.artifact);
    let snapshot = artifacts
        .snapshot("assembled-worker", f.task)
        .await
        .unwrap();
    artifacts
        .execute(
            "assembled-worker",
            command(
                &format!("repair-{round}-submit"),
                snapshot.revision,
                artifacts::ArtifactCommand::Submit {
                    candidate: candidate.clone(),
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
    mailbox.execute("assembled-worker",mail(f.epoch,&format!("repair-{round}-worker-yield"),serde_json::json!({"kind":"yield_episode","episode":worker,"checkpoint":{"id":Uuid::new_v4(),"summary":"actual supervised repair candidate published","artifacts":[{"task":f.task,"digest":candidate.artifact}]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    supervisor
        .release("assembled-worker", execution.id)
        .await
        .unwrap();
    mailbox.execute("assembled-verifier",mail(f.epoch,&format!("repair-verifier-notice-{round}"),serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.verifier],"message_kind":"notice","text":format!("Independently check actual repair {round}"),"reply_to":null,"attention":"normal","artifacts":[{"task":f.task,"digest":candidate.artifact}]}))).await.unwrap();
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    mailbox.execute("assembled-verifier",mail(f.epoch,&format!("repair-{round}-verifier-capture"),serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"verification","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-verifier", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute("assembled-verifier",mail(f.epoch,&format!("repair-{round}-verifier-admit"),serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
    let verification = supervisor
        .create_episode_from_candidate(
            "assembled-verifier",
            episode,
            artifacts,
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    assert_eq!(
        git(&verification.path, &["rev-parse", "HEAD"]),
        candidate.commit
    );
    assert_eq!(
        std::fs::read(verification.path.join("src/resumed")).unwrap(),
        format!("actual-repair-round-{round}\n").as_bytes()
    );
    episode_runtime::EpisodeGateway::new(f.broker.clone())
        .prepare(
            "assembled-verifier",
            episode_runtime::StartRequest {
                id: Uuid::new_v4(),
                episode,
                execution: verification.id,
                runtime: runtime.clone(),
                argv: checks[0].clone(),
            },
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let result = loop {
        if let Some(r) = supervisor
            .result("assembled-verifier", verification.id)
            .await
            .unwrap()
        {
            break r;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "actual SDK must dispatch independent repair gate round{round}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(
        result.exit_code,
        Some(1),
        "actual mandatory gate rejects newly produced repair bytes"
    );
    assert!(result.namespace_stopped);
    let output = std::fs::read(
        supervisor
            .inspect("assembled-verifier", verification.id)
            .await
            .unwrap()
            .manifest
            .log_path,
    )
    .unwrap();
    let log=serde_json::to_vec(&serde_json::json!({"trusted_monitor_result":result,"output":String::from_utf8(output).unwrap()})).unwrap();
    let digest = artifacts::sha256(&log);
    artifacts
        .publish_evidence("assembled-verifier", f.task, &digest, &log)
        .await
        .unwrap();
    retain_bytes(
        &out.join(format!("repair-{round}-verifier-evidence.json")),
        &log,
    );
    let snapshot = artifacts
        .snapshot("assembled-verifier", f.task)
        .await
        .unwrap();
    let failed = artifacts
        .execute(
            "assembled-verifier",
            command(
                &format!("repair-{round}-failed-verification"),
                snapshot.revision,
                artifacts::ArtifactCommand::Verify {
                    verification: artifacts::Verification {
                        candidate: candidate.clone(),
                        suite: artifacts::sha256(&serde_json::to_vec(checks).unwrap()),
                        environment: artifacts::sha256(b"actual-bwrap-private-git-v1"),
                        checks: vec![artifacts::Check {
                            command: checks[0].clone(),
                            exit_code: result.exit_code.unwrap(),
                            logs: digest,
                        }],
                        resolved_decisions: true,
                    },
                },
            ),
            artifacts::ReferenceFault::None,
        )
        .await
        .unwrap();
    mailbox.execute("assembled-verifier",mail(f.epoch,&format!("repair-{round}-verifier-yield"),serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"fresh independently failed repair gate retained","artifacts":[{"task":f.task,"digest":candidate.artifact}]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    supervisor
        .release("assembled-verifier", verification.id)
        .await
        .unwrap();
    println!("completed genuine SDK repair round{round}: new candidate, Worker0, Verifier1");
    (candidate, failed)
}

/// The actual adapter executes and publishes its receipt; this fault drops only
/// the trusted usage observation from its reply, never the native effect itself.
struct UnmeasuredNativeReply {
    native: episode_native::NativeOsRuntime,
}
impl episode_runtime::EpisodeRuntime for UnmeasuredNativeReply {
    fn closed_world_os(&self) -> Option<episode_runtime::ClosedWorldOs> {
        episode_runtime::EpisodeRuntime::closed_world_os(&self.native)
    }
    fn identity(&self) -> RuntimeIdentity {
        episode_runtime::EpisodeRuntime::identity(&self.native)
    }
    fn capabilities(&self) -> CapabilityProfile {
        episode_runtime::EpisodeRuntime::capabilities(&self.native)
    }
    fn start<'a>(
        &'a mut self,
        request: &'a episode_runtime::PreparedStart,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = episode_runtime::EpisodeReply> + Send + 'a>,
    > {
        Box::pin(async move {
            let _actual_reply =
                episode_runtime::EpisodeRuntime::start(&mut self.native, request).await;
            let receipts: Vec<_> = std::fs::read_dir(&self.native.root)
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.extension().is_some_and(|s| s == "effect"))
                .collect();
            assert_eq!(
                receipts.len(),
                1,
                "fault adapter requires one actually executed native effect"
            );
            let receipt: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&receipts[0]).unwrap()).unwrap();
            assert_eq!(receipt["operation"], request.start.id.to_string());
            assert_eq!(receipt["execution"], request.start.execution.to_string());
            let turn: episode_runtime::NativeTurn =
                serde_json::from_value(receipt["turn"].clone()).unwrap();
            episode_runtime::EpisodeReply::Confirmed(turn)
        })
    }
    fn observe<'a>(
        &'a mut self,
        request: &'a episode_runtime::PreparedStart,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = episode_runtime::EpisodeObservation> + Send + 'a>,
    > {
        episode_runtime::EpisodeRuntime::observe(&mut self.native, request)
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn confirmed_native_effect_without_usage_observation_retains_reservation() {
    let f = Fixture::new().await;
    let episode = f.worker_episode("unmeasured-native-effect").await;
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
    let root = f.root.join("unmeasured-native");
    std::fs::create_dir(&root).unwrap();
    let identity = RuntimeIdentity {
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
                argv: vec!["/bin/true".into()],
            },
        )
        .await
        .unwrap();
    let mut runtime = UnmeasuredNativeReply {
        native: episode_native::NativeOsRuntime {
            root: root.clone(),
            identity,
            supervisor: supervisor.clone(),
            service: f.owner.clone(),
            lose_reply: false,
            hold_reply: false,
        },
    };
    assert_eq!(
        gateway
            .dispatch(&f.owner, operation, &mut runtime)
            .await
            .unwrap()
            .state,
        OperationState::Confirmed
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let result = loop {
        if let Some(result) = supervisor
            .result("assembled-worker", execution.id)
            .await
            .unwrap()
        {
            break result;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(result.exit_code, Some(0));
    assert!(result.namespace_stopped);
    let release = supervisor.release("assembled-worker", execution.id).await;
    let capacity = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data["used_capacity"]
        .clone();
    let counts = |extension: &str| {
        std::fs::read_dir(&root)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|s| s == extension)
            })
            .count()
    };
    assert_eq!((counts("invocation"), counts("effect")), (1, 1));
    // All actual namespace members are proven stopped. Fixture-owner cleanup is
    // separate from the broker reservation oracle and does not manufacture refund.
    f.finish().await;
    assert_eq!(
        release.unwrap_err().code,
        ErrorCode::DependencyUnmet,
        "Confirmed native effect plus local Stop is not a measured usage observation"
    );
    assert_eq!(
        capacity, 1,
        "unobserved usage holds the exact original reservation"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn physical_monitor_rechecks_current_scope_generation_and_epoch() {
    for fault in [
        "scope",
        "credential-generation",
        "credential-epoch",
        "expiry",
        "task-generation",
        "input-digest",
        "input-reference",
    ] {
        let f = Fixture::new().await;
        let episode = f.worker_episode("current-authority").await;
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
                    "printf unauthorized-workload > src/result".into(),
                ],
                30,
                supervisor::StartFault::BeforeMonitor,
            )
            .await
            .unwrap();
        let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move { connection.await.unwrap() });
        match fault {
            "scope" => {
                let other = Uuid::new_v4();
                f.broker
                    .execute(
                        &f.owner,
                        envelope(
                            f.epoch,
                            "other-task",
                            0,
                            Command::CreateTask {
                                id: other,
                                mission_id: f.mission,
                                spec: TaskSpec {
                                    dependencies: vec![],
                                    objective: "distinct scoped task".into(),
                                    input: GitInput {
                                        repository: f.root.join("store").to_str().unwrap().into(),
                                        commit: f.base.clone(),
                                        path: "SPEC.md".into(),
                                        digest: artifacts::sha256(b"produce exact src/result"),
                                    },
                                    acceptance: vec!["exact gate".into()],
                                    deliverables: vec!["src/result".into()],
                                    write_scope: vec!["src".into()],
                                },
                            },
                        ),
                    )
                    .await
                    .unwrap();
                db.execute(
                    "UPDATE credentials SET task_id=$2 WHERE token_hash=$1",
                    &[&token_hash("assembled-worker"), &other],
                )
                .await
                .unwrap();
            }
            "credential-generation" => {
                db.execute(
                    "UPDATE credentials SET generation=2 WHERE token_hash=$1",
                    &[&token_hash("assembled-worker")],
                )
                .await
                .unwrap();
            }
            "credential-epoch" => {
                db.execute(
                    "UPDATE credentials SET recovery_epoch=$2 WHERE token_hash=$1",
                    &[&token_hash("assembled-worker"), &Uuid::new_v4()],
                )
                .await
                .unwrap();
            }
            "expiry" => {
                db.execute("UPDATE credentials SET expires_at=clock_timestamp()-interval '1 second' WHERE token_hash=$1", &[&token_hash("assembled-worker")]).await.unwrap();
            }
            "task-generation" => {
                db.execute(
                    "UPDATE tasks SET generation=generation+1 WHERE id=$1",
                    &[&f.task],
                )
                .await
                .unwrap();
            }
            "input-digest" => {
                db.execute(
                    "UPDATE mailbox_episodes SET input_digest=$2 WHERE id=$1",
                    &[&episode, &artifacts::sha256(b"changed captured input")],
                )
                .await
                .unwrap();
            }
            "input-reference" => {
                let input = Uuid::new_v4();
                mailbox::MailboxService::new(f.broker.clone()).execute("assembled-worker",mail(f.epoch,"recapture-input",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"implementation","limit":4}))).await.unwrap();
                db.execute(
                    "UPDATE mailbox_episodes SET input_packet=$2 WHERE id=$1",
                    &[&episode, &input],
                )
                .await
                .unwrap();
            }
            _ => unreachable!(),
        }
        let result = supervisor::run_monitor(f.broker.clone(), execution.id).await;
        assert!(
            result.is_err(),
            "{fault} must reject final physical monitor"
        );
        assert!(
            !execution.path.join("src/result").exists(),
            "{fault} cannot execute workload"
        );
        let results: i64 = db
            .query_one(
                "SELECT count(*) FROM kernel_execution_results WHERE execution_id=$1",
                &[&execution.id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(results, 0);
        let members: serde_json::Value = db
            .query_one(
                "SELECT members FROM supervised_workspaces WHERE id=$1",
                &[&execution.id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(members, serde_json::json!([]));
        let out = evidence_directory(&format!("physical-authority-{fault}"));
        retain_bytes(&out.join("refusal.json"),&serde_json::to_vec(&serde_json::json!({"fault":fault,"code":result.unwrap_err().code,"execution":execution.id,"results":results,"members":members})).unwrap());
        f.finish().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_sealed_packet_and_write_scope_resist_workload_tampering() {
    let f = Fixture::new().await;
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    mailbox.execute("assembled-worker",mail(f.epoch,"sealed-current-notice",serde_json::json!({"kind":"send","mission":f.mission,"id":Uuid::new_v4(),"recipients":[f.worker],"message_kind":"notice","text":"current sealed input notice","reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    mailbox.execute("assembled-worker",mail(f.epoch,"sealed-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"implementation","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-worker", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    let before = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    let rejected=mailbox.execute("assembled-worker",mail(f.epoch,"sealed-bad-digest",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"implementation","input_packet":input,"input_digest":artifacts::sha256(b"wrong input")}))).await;
    assert!(rejected.is_err());
    let after = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap();
    assert_eq!(
        before.data["admitted_episodes"],
        after.data["admitted_episodes"]
    );
    assert_eq!(before.data["used_capacity"], after.data["used_capacity"]);
    mailbox.execute("assembled-worker",mail(f.epoch,"sealed-good-digest",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"implementation","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let immutable = db
        .execute(
            "UPDATE mailbox_inputs SET packet=$2 WHERE id=$1",
            &[&input, &serde_json::json!({"role":"worker","messages":[]})],
        )
        .await
        .unwrap_err();
    assert_eq!(
        immutable.as_db_error().unwrap().message(),
        "immutable record"
    );
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
    let script = r#"python3 - <<'PYSEALED'
import json
p=json.load(open("/episode-input.json"))
assert p["role"] == "worker"
assert any(m["text"] == "current sealed input notice" for m in p["messages"])
try:
    with open("/episode-input.json", "r+") as packet:
        packet.write("tamper")
except OSError:
    print("sealed packet refused write")
else:
    raise AssertionError("captured packet allowed mutation")
PYSEALED
status=$?
test "$status" = 0 && ! (printf tamper > SPEC.md) && printf scoped-write > src/result
"#;
    supervisor
        .start(
            "assembled-worker",
            execution.id,
            vec!["/bin/sh".into(), "-c".into(), script.into()],
            30,
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let result = loop {
        if let Some(result) = supervisor
            .result("assembled-worker", execution.id)
            .await
            .unwrap()
        {
            break result;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(result.exit_code, Some(0));
    assert!(result.namespace_stopped);
    assert_eq!(
        std::fs::read(execution.path.join("src/result")).unwrap(),
        b"scoped-write"
    );
    assert_eq!(
        std::fs::read(execution.path.join("SPEC.md")).unwrap(),
        b"produce exact src/result"
    );
    let out = evidence_directory("actual-sealed-context");
    retain_bytes(
        &out.join("result.json"),
        &serde_json::to_vec(&result).unwrap(),
    );
    supervisor
        .release("assembled-worker", execution.id)
        .await
        .unwrap();
    f.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn broker_process_restart_preserves_owned_work_and_bounded_operator_exact_lookup() {
    let f = Fixture::new().await;
    let out = evidence_directory("broker-restart-operator-overflow");
    let episode = f.worker_episode("broker-view-loss").await;
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
        .start(
            "assembled-worker",
            execution.id,
            vec![
                "/bin/sh".into(),
                "-c".into(),
                "while :; do printf durable-owned-work >> src/live; sleep 0.02; done".into(),
            ],
            30,
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !execution.path.join("src/live").exists() {
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let bind = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = bind.local_addr().unwrap();
    drop(bind);
    let spawn = |label: &str| {
        let log = std::fs::File::create(out.join(format!("{label}.log"))).unwrap();
        OwnedWorker(
            Process::new(env!("CARGO_BIN_EXE_team-control-plane"))
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("TEAM_CONTROL_PLANE_DSN", &f.dsn)
                .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", f.epoch.to_string())
                .args(["serve", &address.to_string()])
                .stdin(Stdio::null())
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log))
                .process_group(0)
                .spawn()
                .unwrap(),
        )
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let url = format!("http://{address}/v1/snapshots/task/{}", f.task);
    let mut first = spawn("broker-original");
    let before = loop {
        if let Ok(response) = client.get(&url).bearer_auth(&f.owner).send().await
            && response.status().is_success()
        {
            break response.json::<serde_json::Value>().await.unwrap();
        }
        assert!(first.0.try_wait().unwrap().is_none());
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    let live_before = std::fs::read(execution.path.join("src/live")).unwrap();
    drop(client);
    first.kill();
    let bridge = Bridge::new(f.broker.clone());
    let registry = bridge
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let wakeups = bridge.pending_wakeups(&f.owner, &registry).await.unwrap();
    assert!(!wakeups.is_empty());
    let mut replacement = spawn("broker-replacement");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let after = loop {
        if let Ok(response) = client.get(&url).bearer_auth(&f.owner).send().await
            && response.status().is_success()
        {
            break response.json::<serde_json::Value>().await.unwrap();
        }
        assert!(replacement.0.try_wait().unwrap().is_none());
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(before["data"], after["data"]);
    assert_eq!(before["revision"], after["revision"]);
    assert_eq!(before["recovery_epoch"], after["recovery_epoch"]);
    let reopened = Broker::connect(&f.dsn, f.epoch).await.unwrap();
    let recovered = Bridge::new(reopened)
        .pending_wakeups(&f.owner, &registry)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&wakeups).unwrap(),
        serde_json::to_value(&recovered).unwrap(),
        "unpresented hints remain durable after broker replacement"
    );
    assert_eq!(
        supervisor
            .inspect(&f.owner, execution.id)
            .await
            .unwrap()
            .state,
        supervisor::ExecutionState::Running
    );
    while std::fs::read(execution.path.join("src/live"))
        .unwrap()
        .len()
        <= live_before.len()
    {
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut ids = vec![f.mission];
    for index in 0..33 {
        let id = Uuid::new_v4();
        ids.push(id);
        let request = envelope(
            f.epoch,
            &format!("overflow-mission-{index}"),
            0,
            Command::CreateMission {
                id,
                objective: format!("bounded operator mission {index}"),
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
        );
        let response = client
            .post(format!("http://{address}/v1/commands"))
            .bearer_auth(&f.owner)
            .json(&request)
            .send()
            .await
            .unwrap();
        assert!(
            response.status().is_success(),
            "actual public mission admission index{index}"
        );
    }
    let mut command = Process::new(env!("CARGO_BIN_EXE_team-control-plane"));
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TEAM_CONTROL_PLANE_DSN", &f.dsn)
        .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", f.epoch.to_string())
        .env("TEAM_CONTROL_PLANE_TOKEN", &f.owner)
        .arg("status");
    let (output, timeout) = bounded_operator_cli(command).await;
    assert!(!timeout);
    assert!(output.status.success());
    let status: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(status["missions"].as_array().unwrap().len(), 32);
    assert_eq!(status["more_missions"], true);
    let excluded = *ids
        .iter()
        .find(|id| {
            !status["missions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["id"] == id.to_string())
        })
        .unwrap();
    let exact = client
        .get(format!("http://{address}/v1/snapshots/mission/{excluded}"))
        .bearer_auth(&f.owner)
        .send()
        .await
        .unwrap();
    assert!(exact.status().is_success());
    let exact: serde_json::Value = exact.json().await.unwrap();
    assert_eq!(exact["data"]["id"], excluded.to_string());
    let denied = client
        .get(format!("http://{address}/v1/snapshots/mission/{excluded}"))
        .bearer_auth("assembled-worker")
        .send()
        .await
        .unwrap();
    assert!(!denied.status().is_success());
    retain_bytes(&out.join("bounded-status.json"), &output.stdout);
    retain_bytes(
        &out.join("excluded-exact.json"),
        &serde_json::to_vec(&exact).unwrap(),
    );
    retain_bytes(
        &out.join("recovered-task.json"),
        &serde_json::to_vec(&after).unwrap(),
    );
    drop(client);
    replacement.kill();
    supervisor.cancel(&f.owner, execution.id).await.unwrap();
    supervisor.reconcile(&f.owner, execution.id).await.unwrap();
    let stopped = std::fs::read(execution.path.join("src/live")).unwrap();
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert_eq!(
        std::fs::read(execution.path.join("src/live")).unwrap(),
        stopped
    );
    supervisor.release(&f.owner, execution.id).await.unwrap();
    f.finish().await;
}

fn check_uncharged_repair_boundary(
    fixture: std::sync::Arc<Fixture>,
    mailbox: mailbox::MailboxService,
    client: reqwest::Client,
    endpoint: String,
    candidate: artifacts::Candidate,
    out: std::path::PathBuf,
    revision: i64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let f = fixture;
        // A failed gate requires an authorized, once-charged Repair before any
        // alternative command key can reopen implementation or the candidate.
        let blocked_input = mailbox
            .execute(
                "assembled-worker",
                mail(
                    f.epoch,
                    "uncharged-failed-capture",
                    serde_json::json!({
                        "kind":"capture_input", "mission":f.mission,
                        "id":Uuid::new_v4(), "attempt":f.attempt,
                        "purpose":"implementation", "limit":4
                    }),
                ),
            )
            .await;
        let blocked_submit = client
            .post(&endpoint)
            .bearer_auth("assembled-worker")
            .json(&serde_json::json!({
                "key":"uncharged-failed-resubmit", "task":f.task,
                "expected_revision":revision, "generation":1,
                "recovery_epoch":f.epoch,
                "command":{"kind":"submit", "candidate":candidate}
            }))
            .send()
            .await
            .unwrap();
        let submit_status = blocked_submit.status();
        let submit_body = blocked_submit.bytes().await.unwrap();
        retain_bytes(&out.join("uncharged-failed-resubmit.json"), &submit_body);
        retain_bytes(
            &out.join("uncharged-failed-capture.json"),
            &serde_json::to_vec(&serde_json::json!({
                "rejected":blocked_input.is_err(),
                "code":blocked_input.as_ref().err().map(|error|format!("{:?}",error.code))
            }))
            .unwrap(),
        );
        assert!(
            blocked_input.is_err() && !submit_status.is_success(),
            "failed verification must fence uncharged Worker continuation and same-candidate resubmit: capture={blocked_input:?}, submit=HTTP{submit_status} {}",
            String::from_utf8_lossy(&submit_body)
        );
    })
}

async fn artifact_fixture_call(
    service: artifacts::ArtifactService,
    bearer: String,
    envelope: artifacts::ArtifactEnvelope,
    fault: artifacts::ReferenceFault,
) -> team_control_plane::Result<artifacts::ArtifactResult> {
    tokio::spawn(async move { service.execute(&bearer, envelope, fault).await })
        .await
        .expect("owned artifact fixture phase failed")
}

async fn cancelled_sdk_drain_phase(
    client: temporalio_client::Client,
    bridge: Bridge,
    reference: WorkflowRef,
    gateway: Gateway,
    owner: String,
    pending: Uuid,
    out: PathBuf,
) {
    tokio::spawn(async move {
        use temporalio_client::{WorkflowFetchHistoryOptions,WorkflowExecutionInfo};
        fn has_fenced_progress(value: &serde_json::Value, pending: Uuid) -> bool {
            use base64::Engine;
            value["events"].as_array().is_some_and(|events| events.iter().any(|event| {
                event["event_type"] == 12 && event["attributes"]["ActivityTaskCompletedEventAttributes"]["result"]["payloads"]
                    .as_array().is_some_and(|payloads| payloads.iter().any(|payload| {
                        let Some(data) = payload["data"].as_str() else { return false; };
                        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data) else { return false; };
                        let Ok(progress) = serde_json::from_slice::<serde_json::Value>(&bytes) else { return false; };
                        progress["operations"].as_array().is_some_and(|operations| operations.iter().any(|operation|
                            operation["limitation"] == "gateway_dispatch_fenced"
                            && operation["journals"].as_array().is_some_and(|journals| journals.iter().any(|journal|
                                journal[0] == pending.to_string() && journal[1] == "prepared"))))
                    }))
            }))
        }
        let deadline=std::time::Instant::now()+Duration::from_secs(20);
        loop {
            let progress=bridge.progress(&owner,&reference).await.unwrap();
            let run=progress.observed.unwrap().run_id;
            let history=WorkflowExecutionInfo::builder().namespace("default").workflow_id(&reference.workflow_id).run_id(&run).build().bind_untyped(client.clone()).fetch_history(WorkflowFetchHistoryOptions::default()).to_json().await.unwrap();
            let value:serde_json::Value=serde_json::from_slice(&history).unwrap();
            if has_fenced_progress(&value,pending) {
                retain_bytes(&out.join("cancelled-sdk-fenced-drain-history.json"),&history);
                break;
            }
            if std::time::Instant::now()>=deadline {
                retain_bytes(&out.join("cancelled-sdk-fenced-drain-timeout-history.json"),&history);
                panic!("actual SDK Activity must retain journals and fenced Prepared effect after cancellation");
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert_eq!(gateway.read(&owner,pending).await.unwrap().state,OperationState::Prepared);
    }).await.expect("owned actual cancelled SDK drain phase failed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_current_gate_fences_preallocated_worker_without_charging_new_work() {
    assembled_engineering_fixture(EngineeringScenario::FailedGateDispatch).await;
}

async fn preallocate_failed_gate_worker(
    f: std::sync::Arc<Fixture>,
    supervisor: supervisor::Supervisor,
    artifacts: artifacts::ArtifactService,
) -> (Uuid, supervisor::ExecutionHandle, serde_json::Value) {
    tokio::spawn(async move {
        let episode = f.worker_episode("pre-failed-gate-worker").await;
        let handle = supervisor
            .create_episode_from_candidate(
                "assembled-worker",
                episode,
                &artifacts,
                supervisor::CreateFault::None,
            )
            .await
            .unwrap();
        let mailbox = mailbox::MailboxService::new(f.broker.clone());
        let record = mailbox
            .record(&f.owner, mailbox::RecordKind::Episode, episode)
            .await
            .unwrap();
        let packet = mailbox
            .record(
                &f.owner,
                mailbox::RecordKind::Input,
                serde_json::from_value(record["input_packet"].clone()).unwrap(),
            )
            .await
            .unwrap();
        (episode, handle, packet)
    })
    .await
    .expect("pre-failure Worker allocation phase failed")
}

async fn failed_gate_worker_phase(
    f: std::sync::Arc<Fixture>,
    supervisor: supervisor::Supervisor,
    artifacts: artifacts::ArtifactService,
    episode: Uuid,
    handle: supervisor::ExecutionHandle,
    packet: serde_json::Value,
) {
    tokio::spawn(async move {
        let before=kernel::Kernel::new(f.broker.clone()).status(&f.owner).await.unwrap();
        let start=supervisor.start("assembled-worker",handle.id,vec!["/bin/sh".into(),"-c".into(),"printf uncharged > src/uncharged".into()],30).await;
        assert_eq!(start.unwrap_err().code,ErrorCode::StaleGeneration);
        assert_eq!(supervisor.inspect(&f.owner,handle.id).await.unwrap().state,supervisor::ExecutionState::Ready);
        assert!(!handle.path.join("src/uncharged").exists());
        assert!(supervisor.result(&f.owner,handle.id).await.unwrap().is_none());
        assert_eq!(supervisor.create_episode_from_candidate("assembled-worker",episode,&artifacts,supervisor::CreateFault::None).await.unwrap_err().code,ErrorCode::DependencyUnmet);
        let mailbox=mailbox::MailboxService::new(f.broker.clone());
        let admit=mailbox.execute("assembled-worker",mail(f.epoch,"after-failed-prepared-admission",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":Uuid::new_v4(),"attempt":f.attempt,"purpose":"implementation","input_packet":packet["id"],"input_digest":packet["digest"]}))).await;
        assert_eq!(admit.unwrap_err().code,ErrorCode::DependencyUnmet);
        let after=kernel::Kernel::new(f.broker.clone()).status(&f.owner).await.unwrap();
        assert_eq!(before["missions"],after["missions"],"refused work cannot charge capacity/episode/repair counters");
        let out=evidence_directory("failed-gate-dispatch");
        retain_bytes(&out.join("preallocated-worker-refused.json"),&serde_json::to_vec(&serde_json::json!({"execution":handle.id,"before":before,"after":after,"view":supervisor.inspect(&f.owner,handle.id).await.unwrap()})).unwrap());
        supervisor.cancel(&f.owner,handle.id).await.unwrap();
        supervisor.release(&f.owner,handle.id).await.unwrap();
    }).await.expect("failed gate Worker refusal phase failed");
}

struct FreshVerifierContext {
    fixture: std::sync::Arc<Fixture>,
    supervisor: supervisor::Supervisor,
    artifacts: artifacts::ArtifactService,
    runtime: RuntimeIdentity,
    candidate: artifacts::Candidate,
    checks: Vec<Vec<String>>,
    negative_episode: Uuid,
    negative_execution: Uuid,
}
async fn fresh_same_candidate_verifier(context: FreshVerifierContext) -> artifacts::ArtifactResult {
    tokio::spawn(async move {
        let FreshVerifierContext {fixture:f,supervisor,artifacts,runtime,candidate,checks,negative_episode,negative_execution}=context;
        assert_eq!(checks.len(),1,"fixture declares one mandatory actual command");
        let mailbox=mailbox::MailboxService::new(f.broker.clone());
        mailbox.execute("assembled-verifier",mail(f.epoch,"yield-negative-before-fresh-gate",serde_json::json!({"kind":"yield_episode","episode":negative_episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"retain failed gate; request genuinely fresh same-candidate independent verification","artifacts":[{"task":f.task,"digest":candidate.artifact}]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
        if supervisor.inspect("assembled-verifier",negative_execution).await.unwrap().state != supervisor::ExecutionState::Released {
            supervisor.release("assembled-verifier",negative_execution).await.unwrap();
        }
        let input=Uuid::new_v4();let episode=Uuid::new_v4();
        mailbox.execute("assembled-verifier",mail(f.epoch,"capture-fresh-current-failed-gate",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"verification","limit":4}))).await.unwrap();
        let packet=mailbox.record("assembled-verifier",mailbox::RecordKind::Input,input).await.unwrap();
        mailbox.execute("assembled-verifier",mail(f.epoch,"admit-fresh-current-failed-gate",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
        let execution=supervisor.create_episode_from_candidate("assembled-verifier",episode,&artifacts,supervisor::CreateFault::None).await.unwrap();
        episode_runtime::EpisodeGateway::new(f.broker.clone()).prepare("assembled-verifier",episode_runtime::StartRequest {id:Uuid::new_v4(),episode,execution:execution.id,runtime,argv:checks[0].clone()}).await.unwrap();
        let deadline=std::time::Instant::now()+Duration::from_secs(8);
        let result=loop {
            if let Some(result)=supervisor.result("assembled-verifier",execution.id).await.unwrap() {break result;}
            assert!(std::time::Instant::now()<deadline,"fresh actual Verifier did not finalize");
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        assert_eq!(result.exit_code,Some(0));assert!(result.namespace_stopped);
        let view=supervisor.inspect("assembled-verifier",execution.id).await.unwrap();
        let bytes=serde_json::to_vec(&serde_json::json!({"trusted_monitor_result":result,"output":String::from_utf8(std::fs::read(&view.manifest.log_path).unwrap()).unwrap()})).unwrap();
        let logs=artifacts::sha256(&bytes);
        artifacts.publish_evidence("assembled-verifier",f.task,&logs,&bytes).await.unwrap();
        let snapshot=artifacts.snapshot("assembled-verifier",f.task).await.unwrap();
        let verified=artifacts.execute("assembled-verifier",artifacts::ArtifactEnvelope {key:"fresh-after-current-failed-gate".into(),task:f.task,expected_revision:snapshot.revision,generation:1,recovery_epoch:f.epoch,command:artifacts::ArtifactCommand::Verify {verification:artifacts::Verification {candidate:candidate.clone(),suite:artifacts::sha256(&serde_json::to_vec(&checks).unwrap()),environment:artifacts::sha256(b"actual-bwrap-private-git-v1"),checks:vec![artifacts::Check {command:checks[0].clone(),exit_code:result.exit_code.unwrap(),logs}],resolved_decisions:true}}},artifacts::ReferenceFault::None).await.unwrap();
        let out=evidence_directory("fresh-after-failed-gate");retain_bytes(&out.join("actual-fresh-verifier.json"),&serde_json::to_vec(&serde_json::json!({"execution":execution.id,"result":result,"verified":verified})).unwrap());
        supervisor.release("assembled-verifier",execution.id).await.unwrap();
        verified
    }).await.expect("fresh independent same-candidate Verifier phase failed")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_sdk_paused_colliding_waits_resume_and_registered_unknown_usage_cancel() {
    use base64::Engine;
    use temporalio_client::{
        Client, ClientOptions, Connection, WorkflowExecutionInfo, WorkflowFetchHistoryOptions,
        envconfig::LoadClientConfigProfileOptions,
    };
    let f = Fixture::new().await;
    let out = evidence_directory("sdk-paused-collision");
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let input = Uuid::new_v4();
    let episode = f.admission;
    mailbox.execute("assembled-worker",mail(f.epoch,"sdk-collision-capture",serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":f.attempt,"purpose":"implementation","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-worker", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute("assembled-worker",mail(f.epoch,"sdk-collision-admit",serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":f.attempt,"purpose":"implementation","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
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
    let legacy_root = f.root.join("sdk-collision-legacy");
    let native_root = f.root.join("sdk-collision-episode");
    std::fs::create_dir(&legacy_root).unwrap();
    std::fs::create_dir(&native_root).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "kernel16-owned-os".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: legacy_root.to_str().unwrap().into(),
        endpoint: "owned-local-subprocess".into(),
        environment: "bwrap-private-git".into(),
    };
    let mut episode_identity = identity.clone();
    episode_identity.runtime_kind = "kernel16-inprocess-native".into();
    episode_identity.thread_store = native_root.to_str().unwrap().into();
    let episode_gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    let episode_operation = Uuid::new_v4();
    episode_gateway
        .prepare(
            "assembled-worker",
            episode_runtime::StartRequest {
                id: episode_operation,
                episode,
                execution: execution.id,
                runtime: episode_identity.clone(),
                argv: vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "printf actual-resumed > src/live; while :; do sleep 0.02; done".into(),
                ],
            },
        )
        .await
        .unwrap();
    let binding = Uuid::new_v4();
    let legacy_operation = Uuid::new_v4();
    let create = Intent {
        id: legacy_operation,
        admission_id: f.admission,
        binding_id: binding,
        generation: 1,
        kind: OperationKind::Create,
        arguments: serde_json::json!({}),
    };
    let bridge = Bridge::new(f.broker.clone());
    let reference = bridge
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    kernel::Kernel::new(f.broker.clone())
        .pause(
            &f.owner,
            f.mission,
            "sdk-collision-pause",
            0,
            true,
            "observe durable paused Prepared progress",
        )
        .await
        .unwrap();
    let (db, connection) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let principal: Uuid = db
        .query_one(
            "SELECT principal_id FROM credentials WHERE token_hash=$1",
            &[&token_hash(&f.owner)],
        )
        .await
        .unwrap()
        .get(0);
    let queue = format!("sdk-collision-{}", f.epoch);
    let log = std::fs::File::create(f.root.join("sdk-collision.log")).unwrap();
    let process = Process::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "assembled_sdk_worker",
            "--ignored",
            "--nocapture",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env(
            "TEMPORAL_ADDRESS",
            std::env::var("TEMPORAL_ADDRESS").unwrap(),
        )
        .env("ASSEMBLED_DSN", &f.dsn)
        .env("ASSEMBLED_EPOCH", f.epoch.to_string())
        .env("ASSEMBLED_BEARER", &f.owner)
        .env("ASSEMBLED_PRINCIPAL", principal.to_string())
        .env("ASSEMBLED_QUEUE", &queue)
        .env("ASSEMBLED_RUNTIME_ROOT", &legacy_root)
        .env(
            "ASSEMBLED_IDENTITY",
            serde_json::to_string(&identity).unwrap(),
        )
        .env(
            "ASSEMBLED_PREPARE",
            serde_json::json!({"admission":f.admission,"binding":binding,"create":create})
                .to_string(),
        )
        .env("ASSEMBLED_WORKSPACES", f.root.join("workspaces"))
        .env("ASSEMBLED_EPISODE_RUNTIME_ROOT", &native_root)
        .env(
            "ASSEMBLED_EPISODE_IDENTITY",
            serde_json::to_string(&episode_identity).unwrap(),
        )
        .env("ASSEMBLED_DROP_USAGE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .process_group(0)
        .spawn()
        .unwrap();
    let mut child = OwnedWorker(process);
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    while db
        .query_one(
            "SELECT count(*) FROM kernel_transports WHERE process_identity->>'pid'=$1",
            &[&child.0.id().to_string()],
        )
        .await
        .unwrap()
        .get::<_, i64>(0)
        < 2
    {
        assert!(child.0.try_wait().unwrap().is_none());
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let (conn, options) =
        ClientOptions::load_from_config(LoadClientConfigProfileOptions::default()).unwrap();
    let client = Client::new(Connection::connect(conn).await.unwrap(), options).unwrap();
    let wake = bridge
        .pending_wakeups(&f.owner, &reference)
        .await
        .unwrap()
        .remove(0);
    bridge
        .deliver(&f.owner, &client, &queue, &wake)
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    loop {
        let progress = bridge.progress(&f.owner, &reference).await.unwrap();
        let run = progress.observed.unwrap().run_id;
        let history = WorkflowExecutionInfo::builder()
            .namespace("default")
            .workflow_id(&reference.workflow_id)
            .run_id(&run)
            .build()
            .bind_untyped(client.clone())
            .fetch_history(WorkflowFetchHistoryOptions::default())
            .to_json()
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&history).unwrap();
        let complete = value["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_type"] == 12)
            .any(|event| {
                event["attributes"]["ActivityTaskCompletedEventAttributes"]["result"]["payloads"]
                    .as_array()
                    .is_some_and(|payloads| {
                        payloads.iter().any(|payload| {
                            let Some(data) = payload["data"].as_str() else {
                                return false;
                            };
                            let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data)
                            else {
                                return false;
                            };
                            let Ok(p) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
                                return false;
                            };
                            p["operations"].as_array().is_some_and(|operations| {
                                operations.iter().any(|o| {
                                    o["episode_id"] == episode.to_string()
                                        && o["limitation"] == "episode_dispatch_fenced"
                                }) && operations.iter().any(|o| {
                                    o["admission_id"] == f.admission.to_string()
                                        && o["limitation"] == "gateway_dispatch_fenced"
                                })
                            })
                        })
                    })
            });
        if complete {
            retain_bytes(
                &out.join("paused-completed-activity-history.json"),
                &history,
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "actual SDK paused typed waits did not complete guarded Activity"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(std::fs::read_dir(&native_root).unwrap().next().is_none());
    assert!(!execution.path.join("src/live").exists());
    kernel::Kernel::new(f.broker.clone())
        .pause(
            &f.owner,
            f.mission,
            "sdk-collision-resume",
            1,
            false,
            "resume same durable workflow",
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let row = db
            .query_one(
                "SELECT state,usage FROM episode_runtime_operations WHERE id=$1",
                &[&episode_operation],
            )
            .await
            .unwrap();
        if row.get::<_, String>(0) == "confirmed" && execution.path.join("src/live").exists() {
            assert_eq!(
                row.get::<_, serde_json::Value>(1),
                serde_json::json!({"kind":"unknown"})
            );
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        Gateway::new(f.broker.clone())
            .read(&f.owner, legacy_operation)
            .await
            .unwrap()
            .state,
        OperationState::Confirmed
    );
    let binding_rows = db
        .query(
            "SELECT evidence FROM episode_runtime_evidence WHERE operation_id=$1 ORDER BY id",
            &[&episode_operation],
        )
        .await
        .unwrap()
        .iter()
        .map(|r| r.get::<_, serde_json::Value>(0))
        .collect::<Vec<_>>();
    let workspace = db
        .query_one(
            "SELECT state,init_process,manifest FROM supervised_workspaces WHERE id=$1",
            &[&execution.id],
        )
        .await
        .unwrap();
    retain_bytes(&out.join("binding-before-cancel.json"),&serde_json::to_vec(&serde_json::json!({"evidence":binding_rows,"state":workspace.get::<_,String>(0),"init":workspace.get::<_,Option<serde_json::Value>>(1),"manifest":workspace.get::<_,serde_json::Value>(2)})).unwrap());
    retain_native(&native_root, &out, "binding-diagnostic");
    let cancel = f
        .broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "sdk-collision-cancel",
                3,
                Command::CancelAttempt {
                    task_id: f.task,
                    attempt_id: f.attempt,
                    reason:
                        "registered native effect is independently bound; usage remains unknown"
                            .into(),
                },
            ),
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let mut cancel_revision = cancel.revision;
    let reconciled = loop {
        let result = f
            .broker
            .execute(
                &f.owner,
                envelope(
                    f.epoch,
                    &format!("sdk-collision-reconcile-{}", Uuid::new_v4()),
                    cancel_revision,
                    Command::ReconcileAttempt {
                        task_id: f.task,
                        attempt_id: f.attempt,
                    },
                ),
            )
            .await
            .unwrap();
        cancel_revision = result.revision;
        if result.disposition == "cancelled" {
            break result;
        }
        if std::time::Instant::now() >= deadline {
            retain_bytes(&out.join("binding-failed-cancel.json"),&serde_json::to_vec(&serde_json::json!({"reconcile":result,"workspace":supervisor.inspect(&f.owner,execution.id).await.unwrap(),"result":supervisor.result(&f.owner,execution.id).await.unwrap()})).unwrap());
            panic!("registered bound Stop must complete cancel without claiming known usage");
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(
        supervisor
            .release(&f.owner, execution.id)
            .await
            .unwrap_err()
            .code,
        ErrorCode::DependencyUnmet
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    while supervisor
        .result(&f.owner, execution.id)
        .await
        .unwrap()
        .is_none()
    {
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let status = kernel::Kernel::new(f.broker.clone())
        .status(&f.owner)
        .await
        .unwrap();
    assert_eq!(
        f.broker
            .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
            .await
            .unwrap()
            .data["used_capacity"],
        1
    );
    retain_bytes(&out.join("registered-unknown-usage-cancel.json"),&serde_json::to_vec(&serde_json::json!({"cancel":reconciled,"status":status,"result":supervisor.result(&f.owner,execution.id).await.unwrap()})).unwrap());
    child.0.kill().unwrap();
    let terminal = child.0.wait().unwrap();
    retain_bytes(
        &out.join("sdk-terminal.json"),
        &serde_json::to_vec(
            &serde_json::json!({"pid":child.0.id(),"waited":true,"terminal":terminal.to_string()}),
        )
        .unwrap(),
    );
    retain_bytes(
        &out.join("sdk.log"),
        &std::fs::read(f.root.join("sdk-collision.log")).unwrap(),
    );
    drop(db);
    f.finish().await;
}
