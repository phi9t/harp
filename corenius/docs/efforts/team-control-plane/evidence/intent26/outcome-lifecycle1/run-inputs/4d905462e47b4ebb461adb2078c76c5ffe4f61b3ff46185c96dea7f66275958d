//! Independent physical-capacity proof: real PostgreSQL, Temporal SDK and OS effects.
//! Mutation targets: extra admission while full, missing descendant shutdown,
//! namespace Stop refunding unresolved usage/effects, or repeated refund.
#[path = "support/episode_native.rs"]
mod episode_native;
#[path = "support/assembled.rs"]
#[allow(dead_code)]
mod support;
use futures::FutureExt;
use std::os::unix::process::CommandExt;
use std::{
    collections::HashMap,
    path::PathBuf,
    process::{Child, Command as Process, Stdio},
    time::{Duration, Instant},
};
use support::*;
use team_control_plane::{gateway::*, temporal::*, *};
use uuid::Uuid;
struct OwnedWorker(Child);
impl Drop for OwnedWorker {
    fn drop(&mut self) {
        if self.0.try_wait().unwrap().is_none() {
            self.0.kill().unwrap();
        }
        self.0.wait().unwrap();
    }
}
#[tokio::test]
#[ignore = "owned episode SDK subprocess; parent fixture controls lifetime"]
async fn capacity_episode_sdk_worker() {
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
        lose_reply: std::env::var("CAPACITY_UNKNOWN_EFFECT").ok().as_deref() == Some("1"),
        hold_reply: std::env::var("CAPACITY_UNKNOWN_EFFECT").ok().as_deref() == Some("1"),
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

fn evidence(name: &str) -> PathBuf {
    let p = PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap()).join(name);
    std::fs::create_dir_all(&p).unwrap();
    p
}
async fn counters(f: &Fixture) -> serde_json::Value {
    f.broker
        .snapshot(&f.owner, SnapshotQuery::Mission(f.mission))
        .await
        .unwrap()
        .data
}
async fn planning(f: &Fixture, key: &str) -> Result<Uuid> {
    let mailbox = mailbox::MailboxService::new(f.broker.clone());
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    mailbox.execute("assembled-lead", mail(f.epoch, &format!("{key}-capture"), serde_json::json!({"kind":"capture_input","mission":f.mission,"id":input,"attempt":null,"purpose":"planning","limit":4}))).await?;
    let packet = mailbox
        .record("assembled-lead", mailbox::RecordKind::Input, input)
        .await?;
    mailbox.execute("assembled-lead", mail(f.epoch, &format!("{key}-admit"), serde_json::json!({"kind":"admit_episode","mission":f.mission,"id":episode,"attempt":null,"purpose":"planning","input_packet":input,"input_digest":packet["digest"]}))).await?;
    Ok(episode)
}
fn identity(f: &Fixture, root: &std::path::Path) -> RuntimeIdentity {
    RuntimeIdentity {
        runtime_kind: "kernel16-inprocess-native".into(),
        runtime_version: "1".into(),
        deployment: f.epoch.to_string(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "owned-async-os".into(),
        environment: "bwrap-private-git".into(),
    }
}
fn writers(worker: bool) -> Vec<String> {
    // Each capsule has two independently sessioned nested writer descendants.
    // Planning has no repository write grant: use its private /tmp and stdout.
    let path = if worker {
        "/workspace/src/heartbeat"
    } else {
        "/tmp/heartbeat"
    };
    vec![
        "/bin/sh".into(),
        "-c".into(),
        format!(
            "setsid sh -c 'while :; do printf a >> {path}; printf a; sleep 0.02; done' & setsid sh -c 'while :; do printf b >> {path}; printf b; sleep 0.02; done' & wait"
        ),
    ]
}
async fn running(
    s: &supervisor::Supervisor,
    f: &Fixture,
    id: Uuid,
    until: Instant,
) -> supervisor::ExecutionView {
    loop {
        let v = s.inspect(&f.owner, id).await.unwrap();
        if v.state == supervisor::ExecutionState::Running
            && v.members.len() >= 4
            && s.logs(&f.owner, id).await.unwrap().len() >= 8
        {
            return v;
        }
        assert!(
            !matches!(
                v.state,
                supervisor::ExecutionState::Stopped | supervisor::ExecutionState::Quarantined
            ),
            "capsule stopped before nested writers: {v:?}"
        );
        assert!(
            Instant::now() < until,
            "approved episode deadline elapsed before Running writer observation: {v:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
async fn stopped(s: &supervisor::Supervisor, f: &Fixture, id: Uuid) -> supervisor::ExecutionView {
    // Use the real persisted episode deadline plus the monitor's one-second wait,
    // rather than replacing the approved deadline with a shorter fixture policy.
    loop {
        let v = s.reconcile(&f.owner, id).await.unwrap();
        if matches!(
            v.state,
            supervisor::ExecutionState::Stopped | supervisor::ExecutionState::Released
        ) {
            assert!(v.members.is_empty());
            return v;
        }
        assert_ne!(
            v.state,
            supervisor::ExecutionState::Quarantined,
            "survivor uncertainty: {v:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
fn independently_live(v: &supervisor::ExecutionView) -> Vec<(u32, String)> {
    let host = std::fs::read_link("/proc/self/ns/pid")
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let observed: Vec<_> = v
        .members
        .iter()
        .filter_map(|m| {
            let ns = std::fs::read_link(format!("/proc/{}/ns/pid", m.pid))
                .ok()?
                .to_string_lossy()
                .into_owned();
            assert_ne!(ns, host, "capsule member is in observer namespace");
            Some((m.pid, ns))
        })
        .collect();
    assert!(
        observed.len() >= 4,
        "independent /proc inventory must see nested live descendants"
    );
    observed
}
async fn start_sdk(
    f: &Fixture,
    root: &std::path::Path,
    identity: &RuntimeIdentity,
    unknown_usage: bool,
    unknown_effect: bool,
    out: &std::path::Path,
) -> OwnedWorker {
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
    let queue = format!("capacity-{}", f.epoch);
    let log = std::fs::File::create(out.join("sdk.log")).unwrap();
    let mut child = OwnedWorker(
        Process::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "capacity_episode_sdk_worker",
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
            .env("ASSEMBLED_RUNTIME_ROOT", root)
            .env(
                "ASSEMBLED_IDENTITY",
                serde_json::to_string(identity).unwrap(),
            )
            .env("ASSEMBLED_WORKSPACES", f.root.join("workspaces"))
            .env(
                "ASSEMBLED_DROP_USAGE",
                if unknown_usage { "1" } else { "0" },
            )
            .env(
                "CAPACITY_UNKNOWN_EFFECT",
                if unknown_effect { "1" } else { "0" },
            )
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .process_group(0)
            .spawn()
            .unwrap(),
    );
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "SDK failed; log {}",
            out.join("sdk.log").display()
        );
        if let Some(r) = db.query_opt("SELECT controller,process_identity,group_id,native_directory,workspace_directory FROM kernel_transports WHERE process_identity->>'pid'=$1", &[&child.0.id().to_string()]).await.unwrap() {
            let process: serde_json::Value = r.get(1);
            assert_eq!(process["pid"],child.0.id());
            assert_eq!(r.get::<_,i32>(2) as u32,child.0.id());
            let stat = std::fs::read_to_string(format!("/proc/{}/stat",child.0.id())).unwrap();
            let fields: Vec<_> = stat[stat.rfind(')').unwrap()+2..].split_whitespace().collect();
            assert_eq!(fields[2].parse::<u32>().unwrap(),child.0.id(),"registered actual SDK is singleton process-group leader");
            assert_eq!(process["start_ticks"],fields[19].parse::<u64>().unwrap());
            std::fs::write(out.join("transport.json"),serde_json::to_vec_pretty(&serde_json::json!({"controller":r.get::<_,Uuid>(0),"process":process,"native":r.get::<_,serde_json::Value>(3),"workspace":r.get::<_,serde_json::Value>(4)})).unwrap()).unwrap();
            break;
        }
        assert!(
            Instant::now() < until,
            "SDK registration missing within approved finite episode window"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let bridge = Bridge::new(f.broker.clone());
    let (conn, options) = temporalio_client::ClientOptions::load_from_config(
        temporalio_client::envconfig::LoadClientConfigProfileOptions::default(),
    )
    .unwrap();
    let client = temporalio_client::Client::new(
        temporalio_client::Connection::connect(conn).await.unwrap(),
        options,
    )
    .unwrap();
    for (kind, id) in [
        (AggregateKind::Task, f.task),
        (AggregateKind::Mission, f.mission),
    ] {
        let reference = bridge.register(&f.owner, kind, id, 0).await.unwrap();
        let wake = bridge
            .pending_wakeups(&f.owner, &reference)
            .await
            .unwrap()
            .remove(0);
        bridge
            .deliver(&f.owner, &client, &queue, &wake)
            .await
            .unwrap();
    }
    child
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_running_capsules_refuse_third_and_cancel_deadline_refund_once() {
    physical_capacity().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopped_capsule_unknown_usage_retains_physical_capacity() {
    uncertainty(true).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopped_capsule_unknown_effect_retains_physical_capacity() {
    uncertainty(false).await;
}
async fn physical_capacity() {
    let out = evidence("known");
    let f = Fixture::new().await;
    let supervisor = f.supervisor().await;
    let until = Instant::now() + Duration::from_secs(30);
    assert_eq!(
        counters(&f).await["used_capacity"],
        1,
        "AssignAttempt reserved Worker slot before its first Episode"
    );
    let worker = f.worker_episode("capacity-worker").await;
    let worker_execution = supervisor
        .create_episode(
            "assembled-worker",
            worker,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    assert_eq!(
        counters(&f).await["used_capacity"],
        1,
        "importing initial Worker Episode cannot double-charge"
    );
    let lead = planning(&f, "capacity-lead").await.unwrap();
    let lead_execution = supervisor
        .create_episode(
            "assembled-lead",
            lead,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    // A distinct ready Task and current Worker grant remove active-principal
    // and dependency fences as alternative reasons for refusing the third slot.
    let third_task = Uuid::new_v4();
    let third_worker = Uuid::new_v4();
    let third_input = GitInput {
        repository: f.root.join("store").to_str().unwrap().into(),
        commit: f.base.clone(),
        path: "SPEC.md".into(),
        digest: artifacts::sha256(b"produce exact src/result"),
    };
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "third-task",
                0,
                Command::CreateTask {
                    id: third_task,
                    mission_id: f.mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "third isolated capacity candidate".into(),
                        input: third_input,
                        acceptance: vec!["exact result".into()],
                        deliverables: vec!["src/result".into()],
                        write_scope: vec!["src".into()],
                    },
                },
            ),
        )
        .await
        .unwrap();
    let third_spec = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(third_task))
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
                "third-admit",
                1,
                Command::AdmitTask {
                    task_id: third_task,
                    spec_digest: third_spec,
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
                "third-grant",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: third_worker,
                    role: Role::Worker,
                    mission_id: Some(f.mission),
                    task_id: Some(third_task),
                    generation: Some(1),
                    token_hash: token_hash("capacity-third-worker"),
                    expires_in_seconds: 300,
                },
            ),
        )
        .await
        .unwrap();
    let native = f.root.join("capacity-native");
    std::fs::create_dir_all(&native).unwrap();
    let runtime = identity(&f, &native);
    let gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    let worker_op = Uuid::new_v4();
    let lead_op = Uuid::new_v4();
    for (token, ep, ex, op, is_worker) in [
        (
            "assembled-worker",
            worker,
            worker_execution.id,
            worker_op,
            true,
        ),
        ("assembled-lead", lead, lead_execution.id, lead_op, false),
    ] {
        gateway
            .prepare(
                token,
                episode_runtime::StartRequest {
                    id: op,
                    episode: ep,
                    execution: ex,
                    runtime: runtime.clone(),
                    argv: writers(is_worker),
                },
            )
            .await
            .unwrap();
    }
    let sdk = start_sdk(&f, &native, &runtime, false, false, &out).await;
    let outcome = std::panic::AssertUnwindSafe(async {
    let worker_view = running(&supervisor, &f, worker_execution.id, until).await;
    let worker_members = independently_live(&worker_view);

    let lead_view = running(&supervisor, &f, lead_execution.id, until).await;
    let lead_members = independently_live(&lead_view);
    assert_ne!(
        worker_members[0].1, lead_members[0].1,
        "two distinct live bwrap namespaces must coexist"
    );
    std::fs::write(out.join("running.json"),serde_json::to_vec_pretty(&serde_json::json!({"worker":worker_view,"lead":lead_view,"independent_worker":worker_members,"independent_lead":lead_members})).unwrap()).unwrap();
    let before = counters(&f).await;
    assert_eq!(before["used_capacity"], 2);
    let before_native = std::fs::read_dir(&native).unwrap().count();
    let before_workspaces = std::fs::read_dir(f.root.join("workspaces"))
        .unwrap()
        .count();
    let rejected = f.broker.execute(&f.owner,envelope(f.epoch,"third-assign",2,Command::AssignAttempt{task_id:third_task,worker_id:third_worker})).await.unwrap_err();
    assert_eq!(rejected.code,ErrorCode::CapacityExhausted,"a separately eligible third Task must fail specifically on approved physical capacity");
    let after = counters(&f).await;
    assert_eq!(before["used_capacity"], after["used_capacity"]);
    assert_eq!(before["admitted_episodes"], after["admitted_episodes"]);
    assert_eq!(
        before_native,
        std::fs::read_dir(&native).unwrap().count(),
        "refusal emitted no physical RPC"
    );
    assert_eq!(
        before_workspaces,
        std::fs::read_dir(f.root.join("workspaces"))
            .unwrap()
            .count(),
        "refusal allocated no new workspace"
    );
    assert_eq!(
        supervisor
            .inspect(&f.owner, worker_execution.id)
            .await
            .unwrap()
            .state,
        supervisor::ExecutionState::Running
    );
    assert_eq!(
        supervisor
            .inspect(&f.owner, lead_execution.id)
            .await
            .unwrap()
            .state,
        supervisor::ExecutionState::Running
    );

    supervisor
        .cancel(&f.owner, worker_execution.id)
        .await
        .unwrap();
    stopped(&supervisor, &f, worker_execution.id).await;
    let heartbeat = std::fs::read(worker_execution.path.join("src/heartbeat")).unwrap();
    let worker_log = supervisor
        .logs(&f.owner, worker_execution.id)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        heartbeat,
        std::fs::read(worker_execution.path.join("src/heartbeat")).unwrap(),
        "cancel stops nested persistent writes"
    );
    assert_eq!(
        worker_log,
        supervisor
            .logs(&f.owner, worker_execution.id)
            .await
            .unwrap(),
        "cancel stops nested stdout writes"
    );
    for (pid, ns) in &worker_members {
        assert_ne!(
            std::fs::read_link(format!("/proc/{pid}/ns/pid"))
                .ok()
                .map(|p| p.to_string_lossy().into_owned()),
            Some(ns.clone()),
            "cancel left live member in owned namespace"
        );
    }

    let measured = gateway.inspect(&f.owner, worker_op).await.unwrap();
    let receipt = std::fs::read_dir(&native)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "effect"))
        .find(|p| {
            let r: serde_json::Value = serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
            r["operation"] == worker_op.to_string()
        })
        .unwrap();
    assert_eq!(
        measured.operation.usage,
        episode_runtime::UsageObservation::MeasuredZero {
            receipt_digest: artifacts::sha256(&std::fs::read(&receipt).unwrap())
        }
    );
    supervisor
        .release(&f.owner, worker_execution.id)
        .await
        .unwrap();
    assert_eq!(counters(&f).await["used_capacity"], 1);
    assert_eq!(
        supervisor
            .release(&f.owner, worker_execution.id)
            .await
            .unwrap_err()
            .code,
        ErrorCode::DependencyUnmet
    );
    assert_eq!(
        counters(&f).await["used_capacity"],
        1,
        "repeat release cannot double-refund"
    );

    stopped(&supervisor, &f, lead_execution.id).await;
    for (pid,ns) in &lead_members {
        assert_ne!(std::fs::read_link(format!("/proc/{pid}/ns/pid")).ok().map(|p|p.to_string_lossy().into_owned()),Some(ns.clone()),"deadline left live nested writer in owned namespace");
    }
    let result_until = Instant::now() + Duration::from_secs(1);
    let result = loop {
        if let Some(result) = supervisor
            .result(&f.owner, lead_execution.id)
            .await
            .unwrap()
        {
            break result;
        }
        assert!(
            Instant::now() < result_until,
            "trusted monitor must publish launcher wait after namespace Stop within its one-second wait contract"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert!(result.namespace_stopped);
    let logs = supervisor.logs(&f.owner, lead_execution.id).await.unwrap();
    assert!(logs.len() >= 8);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        logs,
        supervisor.logs(&f.owner, lead_execution.id).await.unwrap(),
        "deadline stops nested planning writes"
    );

    supervisor
        .release(&f.owner, lead_execution.id)
        .await
        .unwrap();
    assert_eq!(counters(&f).await["used_capacity"], 0);
    let mut native_records = vec![];
    for e in std::fs::read_dir(&native).unwrap() {
        let p = e.unwrap().path();
        if p.is_file() {
            native_records.push(serde_json::json!({"name":p.file_name().unwrap().to_str().unwrap(),"sha256":artifacts::sha256(&std::fs::read(&p).unwrap()),"bytes":std::fs::read_to_string(&p).unwrap()}));
        }
    }
    std::fs::write(
        out.join("ledger.json"),
        serde_json::to_vec_pretty(&native_records).unwrap(),
    )
    .unwrap();
    std::fs::write(
        out.join("deadline-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    }).catch_unwind().await;
    if let Err(error) = outcome {
        for ex in [worker_execution.id, lead_execution.id] {
            let _ = supervisor.cancel(&f.owner, ex).await;
            let view = stopped(&supervisor, &f, ex).await;
            std::fs::write(
                out.join(format!("failure-{ex}.json")),
                serde_json::to_vec_pretty(&view).unwrap(),
            )
            .unwrap();
            let _ = gateway
                .reconcile(
                    &f.owner,
                    if ex == worker_execution.id {
                        worker_op
                    } else {
                        lead_op
                    },
                )
                .await;
            if view.state != supervisor::ExecutionState::Released {
                let _ = supervisor.release(&f.owner, ex).await;
            }
        }
        drop(sdk);
        let retained = f.root.clone();
        f.finish().await;
        std::fs::write(out.join("failure-cleanup.json"),serde_json::to_vec(&serde_json::json!({"sdk_waited":true,"fixture_removed":!retained.exists(),"both_owned_capsules_stopped":true})).unwrap()).unwrap();
        std::panic::resume_unwind(error);
    }
    drop(sdk);
    let retained_root = f.root.clone();
    f.finish().await;
    std::fs::write(out.join("cleanup.json"),serde_json::to_vec_pretty(&serde_json::json!({"sdk_waited":true,"owned_fixture_removed":!retained_root.exists(),"both_executions_stopped_and_released":true})).unwrap()).unwrap();
}

async fn uncertainty(usage: bool) {
    let out = evidence(if usage {
        "unknown-usage"
    } else {
        "unknown-effect"
    });
    let f = Fixture::new().await;
    let s = f.supervisor().await;
    let episode = f.worker_episode("held-worker").await;
    let ex = s
        .create_episode(
            "assembled-worker",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let root = f.root.join("held-native");
    std::fs::create_dir_all(&root).unwrap();
    let runtime = identity(&f, &root);
    let gateway = episode_runtime::EpisodeGateway::new(f.broker.clone());
    let operation = Uuid::new_v4();
    gateway
        .prepare(
            "assembled-worker",
            episode_runtime::StartRequest {
                id: operation,
                episode,
                execution: ex.id,
                runtime: runtime.clone(),
                argv: writers(true),
            },
        )
        .await
        .unwrap();
    let sdk = start_sdk(&f, &root, &runtime, usage, !usage, &out).await;
    let observed = running(&s, &f, ex.id, Instant::now() + Duration::from_secs(30)).await;
    let members = independently_live(&observed);
    loop {
        let op = gateway.inspect(&f.owner, operation).await.unwrap();
        if op.operation.state
            == if usage {
                OperationState::Confirmed
            } else {
                OperationState::Dispatching
            }
            && (usage
                || std::fs::read_dir(&root)
                    .unwrap()
                    .any(|e| e.unwrap().path().extension().is_some_and(|x| x == "effect")))
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    s.cancel(&f.owner, ex.id).await.unwrap();
    stopped(&s, &f, ex.id).await;
    let bytes = std::fs::read(ex.path.join("src/heartbeat")).unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(bytes, std::fs::read(ex.path.join("src/heartbeat")).unwrap());
    for (pid, ns) in &members {
        assert_ne!(
            std::fs::read_link(format!("/proc/{pid}/ns/pid"))
                .ok()
                .map(|p| p.to_string_lossy().into_owned()),
            Some(ns.clone())
        );
    }
    let held = gateway.inspect(&f.owner, operation).await.unwrap();
    assert_eq!(
        held.operation.usage,
        episode_runtime::UsageObservation::Unknown
    );
    assert_eq!(
        s.release(&f.owner, ex.id).await.unwrap_err().code,
        ErrorCode::DependencyUnmet,
        "actual namespace Stop cannot refund unresolved transport usage/effects"
    );
    assert_eq!(counters(&f).await["used_capacity"], 1);
    std::fs::write(out.join("held.json"),serde_json::to_vec_pretty(&serde_json::json!({"operation":held,"mission":counters(&f).await,"running":observed,"independent_members":members})).unwrap()).unwrap();
    let resolved = gateway.reconcile(&f.owner, operation).await.unwrap();
    assert_eq!(resolved.state, OperationState::Confirmed);
    let effect = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|x| x == "effect"))
        .unwrap();
    assert_eq!(
        resolved.usage,
        episode_runtime::UsageObservation::MeasuredZero {
            receipt_digest: artifacts::sha256(&std::fs::read(&effect).unwrap())
        }
    );
    s.release(&f.owner, ex.id).await.unwrap();
    assert_eq!(counters(&f).await["used_capacity"], 0);
    assert!(s.release(&f.owner, ex.id).await.is_err());
    assert_eq!(counters(&f).await["used_capacity"], 0);
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
        1,
        "query-only resolution cannot issue another Start"
    );
    std::fs::copy(effect, out.join("native-effect.json")).unwrap();
    drop(sdk);
    let retained = f.root.clone();
    f.finish().await;
    std::fs::write(out.join("cleanup.json"),serde_json::to_vec(&serde_json::json!({"sdk_waited":true,"fixture_removed":!retained.exists(),"capsule_stopped_and_released":true})).unwrap()).unwrap();
}
