#[path = "support/gateway_fault.rs"]
#[allow(dead_code)]
mod gateway_fault;
use std::{
    collections::HashMap,
    process::{Child, Command as Process, Stdio},
    time::Duration,
};
use team_control_plane::{temporal::*, *};
use uuid::Uuid;
fn assert_history_no_secret(data: &[u8], secret: &str) {
    use base64::Engine;
    fn walk(v: &serde_json::Value, secret: &str) {
        match v {
            serde_json::Value::String(s) => {
                assert!(!s.contains(secret));
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
                    assert!(!String::from_utf8_lossy(&decoded).contains(secret));
                }
            }
            serde_json::Value::Array(v) => v.iter().for_each(|v| walk(v, secret)),
            serde_json::Value::Object(v) => v.values().for_each(|v| walk(v, secret)),
            _ => {}
        }
    }
    walk(&serde_json::from_slice(data).unwrap(), secret);
}
fn input() -> GitInput {
    GitInput {
        repository: "fixture".into(),
        commit: "a".repeat(40),
        path: "SPEC.md".into(),
        digest: "b".repeat(64),
    }
}
fn envelope(epoch: Uuid, key: &str, revision: i64, command: Command) -> CommandEnvelope {
    CommandEnvelope {
        schema_version: 1,
        key: key.into(),
        expected_revision: revision,
        expected_generation: None,
        recovery_epoch: epoch,
        command,
    }
}
struct WorkerChild(Child);
impl std::ops::Deref for WorkerChild {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.0
    }
}
impl std::ops::DerefMut for WorkerChild {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}
impl Drop for WorkerChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
struct Fixture {
    dsn: String,
    broker: Broker,
    bridge: Bridge,
    owner: String,
    principal: Uuid,
    epoch: Uuid,
    mission: Uuid,
    task: Uuid,
}
impl Fixture {
    async fn new() -> Self {
        let base = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").expect("real PostgreSQL required");
        let schema = format!("temporal_{}", Uuid::new_v4().simple());
        let (c, conn) = tokio_postgres::connect(&base, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move {
            conn.await.unwrap();
        });
        c.batch_execute(&format!("CREATE SCHEMA {schema}"))
            .await
            .unwrap();
        let dsn = format!("{base} options='-c search_path={schema}'");
        migrate(&dsn).await.unwrap();
        let epoch = Uuid::new_v4();
        let broker = Broker::connect(&dsn, epoch).await.unwrap();
        let owner = broker.bootstrap_owner().await.unwrap();
        let principal = c
            .query_one(
                &format!("SELECT principal_id FROM {schema}.credentials WHERE role='owner'"),
                &[],
            )
            .await
            .unwrap()
            .get(0);
        let mission = Uuid::new_v4();
        broker
            .execute(
                &owner,
                envelope(
                    epoch,
                    "mission",
                    0,
                    Command::CreateMission {
                        id: mission,
                        objective: "bounded fixture".into(),
                        input: input(),
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
        let task = Uuid::new_v4();
        broker
            .execute(
                &owner,
                envelope(
                    epoch,
                    "task",
                    0,
                    Command::CreateTask {
                        id: task,
                        mission_id: mission,
                        spec: TaskSpec {
                            dependencies: vec![],
                            objective: "bounded task".into(),
                            input: input(),
                            acceptance: vec!["verified".into()],
                            deliverables: vec!["result".into()],
                            write_scope: vec!["src".into()],
                        },
                    },
                ),
            )
            .await
            .unwrap();
        let bridge = Bridge::new(broker.clone());
        Self {
            dsn,
            broker,
            bridge,
            owner,
            principal,
            epoch,
            mission,
            task,
        }
    }
    async fn admit(&self, key: &str, revision: i64) -> CommandEnvelope {
        let s = self
            .broker
            .snapshot(&self.owner, SnapshotQuery::Task(self.task))
            .await
            .unwrap();
        envelope(
            self.epoch,
            key,
            revision,
            Command::AdmitTask {
                task_id: self.task,
                spec_digest: s.data["spec_digest"].as_str().unwrap().into(),
            },
        )
    }
    fn child(&self, queue: &str) -> WorkerChild {
        WorkerChild(
            Process::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "temporal_worker_process",
                    "--ignored",
                    "--nocapture",
                ])
                .env("TEMPORAL_FIXTURE_DSN", &self.dsn)
                .env("TEMPORAL_FIXTURE_EPOCH", self.epoch.to_string())
                .env("TEMPORAL_FIXTURE_BEARER", &self.owner)
                .env("TEMPORAL_FIXTURE_PRINCIPAL", self.principal.to_string())
                .env("TEMPORAL_FIXTURE_QUEUE", queue)
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        )
    }
}
#[tokio::test]
async fn public_migration_installs_transactional_workflow_registry() {
    let f = Fixture::new().await;
    let (c, conn) = tokio_postgres::connect(&f.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        conn.await.unwrap();
    });
    assert!(
        c.query_one("SELECT to_regclass('workflow_registry') IS NOT NULL", &[])
            .await
            .unwrap()
            .get::<_, bool>(0)
    );
    let r = f
        .bridge
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    assert_eq!(
        r,
        f.bridge
            .register(&f.owner, AggregateKind::Task, f.task, 0)
            .await
            .unwrap()
    );
    assert_eq!(
        f.bridge.pending_wakeups(&f.owner, &r).await.unwrap().len(),
        1
    );
    let reject = f.admit("reject", 99).await;
    let valid = f.admit("valid", 1).await;
    let q1 = f
        .bridge
        .enqueue(&f.owner, &r, reject.clone())
        .await
        .unwrap();
    let q2 = f.bridge.enqueue(&f.owner, &r, valid.clone()).await.unwrap();
    assert_eq!(
        f.bridge
            .enqueue(&f.owner, &r, reject)
            .await
            .unwrap()
            .sequence,
        q1.sequence
    );
    assert_eq!(
        f.bridge
            .apply(&f.owner, &r, q2.sequence)
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleRevision
    );
    assert_eq!(f.bridge.progress(&f.owner, &r).await.unwrap().cursor, 0);
    let rejected = f.bridge.apply(&f.owner, &r, q1.sequence).await.unwrap();
    assert_eq!(rejected.disposition.as_deref(), Some("rejected"));
    let applied = f.bridge.apply(&f.owner, &r, q2.sequence).await.unwrap();
    assert_eq!(applied.disposition.as_deref(), Some("applied"));
    assert_eq!(
        applied.result,
        f.bridge
            .apply(&f.owner, &r, q2.sequence)
            .await
            .unwrap()
            .result
    );
    assert_eq!(f.bridge.progress(&f.owner, &r).await.unwrap().cursor, 2);
    f.bridge.close(&f.owner, &r).await.unwrap();
    let late = f
        .bridge
        .enqueue(&f.owner, &r, f.admit("late", 2).await)
        .await
        .unwrap();
    assert_eq!(late.disposition.as_deref(), Some("rejected"));
    assert_eq!(f.bridge.progress(&f.owner, &r).await.unwrap().cursor, 3);
    let next = f
        .bridge
        .register(&f.owner, AggregateKind::Task, f.task, 1)
        .await
        .unwrap();
    assert_eq!(next.generation, 2);
    assert_eq!(
        f.bridge.apply(&f.owner, &r, 2).await.unwrap_err().code,
        ErrorCode::StaleGeneration
    );
    assert_eq!(f.bridge.progress(&f.owner, &next).await.unwrap().cursor, 0);
    let worker = Uuid::new_v4();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "worker",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(f.mission),
                    task_id: Some(f.task),
                    generation: Some(1),
                    token_hash: token_hash("worker"),
                    expires_in_seconds: 30,
                },
            ),
        )
        .await
        .unwrap();
    let mut command = envelope(
        f.epoch,
        "assign",
        2,
        Command::AssignAttempt {
            task_id: f.task,
            worker_id: worker,
        },
    );
    command.expected_generation = Some(0);
    let assignment = f
        .bridge
        .enqueue(&f.owner, &next, command.clone())
        .await
        .unwrap();
    let first = f
        .bridge
        .apply(&f.owner, &next, assignment.sequence)
        .await
        .unwrap();
    assert_eq!(
        first.result,
        f.bridge
            .apply(&f.owner, &next, assignment.sequence)
            .await
            .unwrap()
            .result
    );
    assert_eq!(
        assignment.sequence,
        f.bridge
            .enqueue(&f.owner, &next, command)
            .await
            .unwrap()
            .sequence
    );
    let snapshot = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(f.task))
        .await
        .unwrap();
    for field in ["attempt_count", "operation_count", "reservation_count"] {
        assert_eq!(snapshot.data[field], 1);
    }
    use team_control_plane::gateway::*;
    let admission: Uuid =
        serde_json::from_value(first.result.as_ref().unwrap()["operation_id"].clone()).unwrap();
    let empty = f
        .bridge
        .operation_journal(&f.owner, &next, admission)
        .await
        .unwrap();
    assert!(empty.is_empty());
    let directory = std::env::temp_dir().join(format!("temporal13-driver-{}", f.epoch));
    std::fs::create_dir_all(&directory).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "fault".into(),
        runtime_version: "v1".into(),
        deployment: "temporal13".into(),
        thread_store: directory.to_string_lossy().into(),
        endpoint: "local".into(),
        environment: "disposable".into(),
    };
    let gateway = Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(
            &f.owner,
            binding,
            admission,
            &identity,
            Some("known-thread"),
            1,
        )
        .await
        .unwrap();
    let intent = Intent {
        id: Uuid::new_v4(),
        admission_id: admission,
        binding_id: binding,
        generation: 1,
        kind: OperationKind::Start,
        arguments: serde_json::json!({"prompt":"bounded fixture"}),
    };
    gateway.prepare(&f.owner, intent.clone()).await.unwrap();
    let runtime = gateway_fault::FaultRuntime::new(
        directory.clone(),
        gateway_fault::Fault::DropReply,
        identity,
    );
    let driver = GatewayOperationDriver::new(
        f.bridge.clone(),
        gateway,
        f.owner.clone(),
        Box::new(runtime),
    );
    let observed = driver.drive(&next, admission).await.unwrap();
    assert_eq!(
        observed.journals[0],
        (intent.id, OperationState::OutcomeUnknown)
    );
    let again = driver.drive(&next, admission).await.unwrap();
    assert_eq!(again.journals[0], observed.journals[0]);
    let invocations = std::fs::read_dir(&directory)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|s| s == "invocation"))
        .count();
    assert_eq!(invocations, 1, "driver may not retry an uncertain Start");
    std::fs::remove_dir_all(&directory).unwrap();
    let lead = "expiring-lead";
    let lead_id = Uuid::new_v4();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "lead",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: lead_id,
                    role: Role::Lead,
                    mission_id: Some(f.mission),
                    task_id: Some(f.task),
                    generation: None,
                    token_hash: token_hash(lead),
                    expires_in_seconds: 1,
                },
            ),
        )
        .await
        .unwrap();
    let mut pending = envelope(
        f.epoch,
        "expired-command",
        3,
        Command::AssignAttempt {
            task_id: f.task,
            worker_id: worker,
        },
    );
    pending.expected_generation = Some(1);
    let q = f.bridge.enqueue(lead, &next, pending).await.unwrap();
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let denied = f
        .bridge
        .apply_with_capability(&f.owner, Some(lead), &next, q.sequence)
        .await
        .unwrap();
    assert_eq!(denied.disposition.as_deref(), Some("rejected"));
    assert_eq!(denied.result.unwrap()["code"], "unauthorized");
    assert_eq!(
        f.bridge.progress(&f.owner, &next).await.unwrap().cursor,
        q.sequence
    );
    let snapshot = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(f.task))
        .await
        .unwrap();
    assert_eq!(snapshot.data["attempt_count"], 1);
    let verifier = "fixture-verifier";
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "verifier",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: Uuid::new_v4(),
                    role: Role::Verifier,
                    mission_id: Some(f.mission),
                    task_id: Some(f.task),
                    generation: Some(1),
                    token_hash: token_hash(verifier),
                    expires_in_seconds: 30,
                },
            ),
        )
        .await
        .unwrap();
    let mut assign = envelope(
        f.epoch,
        "forbidden",
        2,
        Command::AssignAttempt {
            task_id: f.task,
            worker_id: Uuid::new_v4(),
        },
    );
    assign.expected_generation = Some(1);
    assert_eq!(
        f.bridge
            .enqueue(verifier, &next, assign)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    f.bridge.close(&f.owner, &next).await.unwrap();
    assert_eq!(
        driver.drive(&next, admission).await.unwrap_err().code,
        ErrorCode::StaleGeneration,
        "terminal driver cannot execute prepared operations"
    );
}
#[tokio::test]
async fn command_identity_contains_operation_family() {
    let f = Fixture::new().await;
    let r = f
        .bridge
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let dependency = envelope(
        f.epoch,
        "shared-key",
        1,
        Command::SetDependencies {
            task_id: f.task,
            dependencies: vec![],
        },
    );
    let q = f
        .bridge
        .enqueue(&f.owner, &r, dependency.clone())
        .await
        .unwrap();
    let admit = f.admit("shared-key", 2).await;
    let second = f
        .bridge
        .enqueue(&f.owner, &r, admit)
        .await
        .expect("separate family must coexist with same principal/key");
    assert_eq!(second.sequence, q.sequence + 1);
    let mut different = dependency.clone();
    different.command = Command::SetDependencies {
        task_id: f.task,
        dependencies: vec![Uuid::new_v4()],
    };
    assert_eq!(
        f.bridge
            .enqueue(&f.owner, &r, different)
            .await
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict
    );
    f.bridge.apply(&f.owner, &r, q.sequence).await.unwrap();
    f.bridge.apply(&f.owner, &r, second.sequence).await.unwrap();
    let mut retried = dependency;
    retried.expected_revision = 999;
    assert_eq!(
        f.bridge
            .enqueue(&f.owner, &r, retried)
            .await
            .unwrap()
            .sequence,
        q.sequence,
        "precondition changes preserve canonical argument identity"
    );
}
#[tokio::test]
async fn verifier_cannot_suppress_initial_wakeup() {
    let f = Fixture::new().await;
    let r = f
        .bridge
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let snapshot = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(f.task))
        .await
        .unwrap();
    f.broker
        .execute(&f.owner, f.admit("admit-for-permission", 1).await)
        .await
        .unwrap();
    let worker = Uuid::new_v4();
    for (role, token, principal) in [
        (Role::Worker, "ack-worker", worker),
        (Role::Verifier, "ack-verifier", Uuid::new_v4()),
    ] {
        f.broker
            .execute(
                &f.owner,
                envelope(
                    f.epoch,
                    token,
                    0,
                    Command::GrantCredential {
                        credential_id: Uuid::new_v4(),
                        principal_id: principal,
                        role,
                        mission_id: Some(f.mission),
                        task_id: Some(f.task),
                        generation: Some(1),
                        token_hash: token_hash(token),
                        expires_in_seconds: 30,
                    },
                ),
            )
            .await
            .unwrap();
    }
    let mut assign = envelope(
        f.epoch,
        "ack-assignment",
        snapshot.revision + 1,
        Command::AssignAttempt {
            task_id: f.task,
            worker_id: worker,
        },
    );
    assign.expected_generation = Some(0);
    f.broker.execute(&f.owner, assign).await.unwrap();
    let wake = f
        .bridge
        .pending_wakeups(&f.owner, &r)
        .await
        .unwrap()
        .remove(0);
    use temporalio_client::{
        Client, ClientOptions, Connection, envconfig::LoadClientConfigProfileOptions,
    };
    let (connection, options) =
        ClientOptions::load_from_config(LoadClientConfigProfileOptions::default()).unwrap();
    let client = Client::new(Connection::connect(connection).await.unwrap(), options).unwrap();
    for token in ["ack-worker", "ack-verifier"] {
        assert_eq!(
            f.bridge
                .acknowledge_wakeup(token, &r, wake.id)
                .await
                .unwrap_err()
                .code,
            ErrorCode::Unauthorized
        );
    }
    for token in ["ack-worker", "ack-verifier"] {
        assert_eq!(
            f.bridge
                .deliver(token, &client, "must-not-start", &wake)
                .await
                .unwrap_err()
                .code,
            ErrorCode::Unauthorized
        );
    }
    assert_eq!(
        f.bridge.pending_wakeups(&f.owner, &r).await.unwrap().len(),
        1
    );
}
#[tokio::test]
async fn queued_old_generation_driver_cannot_dispatch_after_close_and_replacement() {
    use std::sync::Arc;
    use team_control_plane::gateway::*;
    let f = Fixture::new().await;
    let r = f
        .bridge
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    f.broker
        .execute(&f.owner, f.admit("driver-admit", 1).await)
        .await
        .unwrap();
    let worker = Uuid::new_v4();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "driver-worker",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(f.mission),
                    task_id: Some(f.task),
                    generation: Some(1),
                    token_hash: token_hash("driver-worker"),
                    expires_in_seconds: 30,
                },
            ),
        )
        .await
        .unwrap();
    let mut assign = envelope(
        f.epoch,
        "driver-assign",
        2,
        Command::AssignAttempt {
            task_id: f.task,
            worker_id: worker,
        },
    );
    assign.expected_generation = Some(0);
    let receipt = f.broker.execute(&f.owner, assign).await.unwrap();
    let admission = receipt.operation_id.unwrap();
    let directory = std::env::temp_dir().join(format!("temporal13-queued-{}", f.epoch));
    std::fs::create_dir_all(&directory).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "fault".into(),
        runtime_version: "v1".into(),
        deployment: "repair1".into(),
        thread_store: directory.to_string_lossy().into(),
        endpoint: "local".into(),
        environment: "disposable".into(),
    };
    let gateway = Gateway::new(f.broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(
            &f.owner,
            binding,
            admission,
            &identity,
            Some("known-thread"),
            1,
        )
        .await
        .unwrap();
    let intent = Intent {
        id: Uuid::new_v4(),
        admission_id: admission,
        binding_id: binding,
        generation: 1,
        kind: OperationKind::Start,
        arguments: serde_json::json!({"prompt":"queued"}),
    };
    gateway.prepare(&f.owner, intent).await.unwrap();
    let runtime: Box<dyn RuntimeAdapter> = Box::new(gateway_fault::FaultRuntime::new(
        directory.clone(),
        gateway_fault::Fault::None,
        identity,
    ));
    let gate = Arc::new(tokio::sync::Mutex::new(runtime));
    let held = gate.lock().await;
    let driver = Arc::new(GatewayOperationDriver::with_shared_runtime(
        f.bridge.clone(),
        gateway,
        f.owner.clone(),
        gate.clone(),
    ));
    let childdriver = driver.clone();
    let reference = r.clone();
    let pending = tokio::spawn(async move { childdriver.drive(&reference, admission).await });
    for _ in 0..100 {
        if driver.queued() == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(driver.queued(), 1, "actual runtime mutex waiter required");
    f.bridge.close(&f.owner, &r).await.unwrap();
    f.bridge
        .register(&f.owner, AggregateKind::Task, f.task, 1)
        .await
        .unwrap();
    drop(held);
    let outcome = pending.await.unwrap();
    let invocations = std::fs::read_dir(&directory)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "invocation"))
        .count();
    std::fs::remove_dir_all(directory).unwrap();
    assert_eq!(
        invocations, 0,
        "closed/replaced queued controller must create zero physical effects"
    );
    assert_eq!(outcome.unwrap_err().code, ErrorCode::StaleGeneration);
}
fn kill_worker(child: &mut Child) {
    use std::os::unix::process::ExitStatusExt;
    assert!(
        child.try_wait().unwrap().is_none(),
        "unexpected worker exit before injected SIGKILL"
    );
    child.kill().expect("inject SIGKILL into live worker");
    assert_eq!(
        child.wait().unwrap().signal(),
        Some(9),
        "fixture requires actual Unix SIGKILL terminal signal"
    );
}
#[test]
fn sigkill_fixture_rejects_unexpected_prior_exit() {
    let mut child = Process::new("/bin/sh")
        .args(["-c", "exit 7"])
        .spawn()
        .unwrap();
    for _ in 0..100 {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| kill_worker(&mut child))).is_err(),
        "unexpected prior exit must not count as SIGKILL proof"
    );
}
struct ReplyLoss;
impl temporalio_sdk::interceptors::ActivityInboundInterceptor for ReplyLoss {
    fn execute_activity<'a>(
        &'a self,
        input: temporalio_sdk::interceptors::ExecuteActivityInput,
        next: temporalio_sdk::interceptors::Next<
            'a,
            temporalio_sdk::interceptors::ExecuteActivityInput,
            temporalio_sdk::interceptors::ExecuteActivityOutput<'a>,
        >,
    ) -> temporalio_sdk::interceptors::ExecuteActivityOutput<'a> {
        let attempt = input.activity_info().attempt;
        let args = input.args_ref::<WorkflowInput>().unwrap().clone();
        Box::pin(async move {
            let result = next.run(input).await;
            if let Ok(ref output) = result {
                let p = output.downcast_ref::<Progress>().unwrap();
                let directory =
                    std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
                use std::io::Write;
                let mut record = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(directory.join("activity-invocations.jsonl"))
                    .unwrap();
                writeln!(record,"{}",serde_json::json!({"workflow":args.reference.workflow_id,"input_cursor":args.cursor,"output_cursor":p.cursor,"attempt":attempt,"waits":p.waits})).unwrap();
                record.sync_all().unwrap();
                if !p.waits.is_empty()
                    && std::fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(directory.join("reply-lost.marker"))
                        .is_ok()
                {
                    return Err(temporalio_sdk::activities::ActivityError::application(
                        temporalio_sdk::ApplicationFailure::new(
                            "fixture deliberate reply loss after canonical commit",
                        ),
                    ));
                }
            }
            result
        })
    }
}
#[tokio::test]
#[ignore = "child worker process; parent fixture owns lifetime"]
async fn temporal_worker_process() {
    use temporalio_client::{
        Client, ClientOptions, Connection, envconfig::LoadClientConfigProfileOptions,
    };
    use temporalio_sdk::{Runtime, Worker, WorkerOptions};
    let dsn = std::env::var("TEMPORAL_FIXTURE_DSN").unwrap();
    let epoch = std::env::var("TEMPORAL_FIXTURE_EPOCH")
        .unwrap()
        .parse()
        .unwrap();
    let bearer = std::env::var("TEMPORAL_FIXTURE_BEARER").unwrap();
    let principal = std::env::var("TEMPORAL_FIXTURE_PRINCIPAL")
        .unwrap()
        .parse()
        .unwrap();
    let activities = BridgeActivities::new(
        Bridge::new(Broker::connect(&dsn, epoch).await.unwrap()),
        bearer.clone(),
        HashMap::from([(principal, bearer)]),
    );
    let runtime = Runtime::from_current_tokio(Default::default()).unwrap();
    let (conn, options) =
        ClientOptions::load_from_config(LoadClientConfigProfileOptions::default()).unwrap();
    let client = Client::new(Connection::connect(conn).await.unwrap(), options).unwrap();
    let options = WorkerOptions::new(std::env::var("TEMPORAL_FIXTURE_QUEUE").unwrap())
        .register_workflow::<MissionWorkflow>()
        .unwrap()
        .register_workflow::<TaskWorkflow>()
        .unwrap()
        .register_activities(activities)
        .sticky_queue_schedule_to_start_timeout(Duration::from_secs(1))
        .activity_inbound_interceptor(ReplyLoss)
        .build();
    let mut worker = Worker::new(&runtime, client, options).unwrap();
    worker.run().await.unwrap();
}
#[tokio::test]
async fn real_temporal_recovers_lost_hints_rollover_and_retained_replay() {
    use temporalio_client::{
        Client, ClientOptions, Connection, WorkflowExecutionInfo, WorkflowFetchHistoryOptions,
        WorkflowGetResultOptions, WorkflowSignalOptions, WorkflowStartOptions,
        envconfig::LoadClientConfigProfileOptions,
    };
    let f = Fixture::new().await;
    let (conn, options) =
        ClientOptions::load_from_config(LoadClientConfigProfileOptions::default()).unwrap();
    let client = Client::new(Connection::connect(conn).await.unwrap(), options).unwrap();
    let queue = format!("temporal13-{}", f.epoch);
    let mut child = f.child(&queue);
    let r = f
        .bridge
        .register(&f.owner, AggregateKind::Task, f.task, 0)
        .await
        .unwrap();
    let start = f
        .bridge
        .pending_wakeups(&f.owner, &r)
        .await
        .unwrap()
        .remove(0);
    f.bridge
        .deliver(&f.owner, &client, &queue, &start)
        .await
        .unwrap();
    f.bridge
        .deliver(&f.owner, &client, &queue, &start)
        .await
        .unwrap();
    let handle = client.get_workflow_handle::<TaskWorkflow>(&r.workflow_id);
    handle
        .signal(TaskWorkflow::wake, 99, WorkflowSignalOptions::default())
        .await
        .unwrap();
    handle
        .signal(TaskWorkflow::wake, 99, WorkflowSignalOptions::default())
        .await
        .unwrap();
    // Capture first run through UseExisting; no second execution is permitted.
    let original = client
        .start_workflow(
            TaskWorkflow::run,
            WorkflowInput {
                reference: r.clone(),
                cursor: 0,
                waits: vec![],
            },
            WorkflowStartOptions::new(&queue, &r.workflow_id)
                .id_conflict_policy(temporalio_client::WorkflowIdConflictPolicy::UseExisting)
                .id_reuse_policy(temporalio_client::WorkflowIdReusePolicy::RejectDuplicate)
                .build(),
        )
        .await
        .unwrap();
    let first_run = original.run_id().unwrap().to_owned();
    let q = f
        .bridge
        .enqueue(&f.owner, &r, f.admit("lost-hint", 1).await)
        .await
        .unwrap();
    // Deliberately never deliver this wakeup: a durable timer must discover it.
    for _ in 0..100 {
        if f.bridge.progress(&f.owner, &r).await.unwrap().cursor == q.sequence {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(f.bridge.progress(&f.owner, &r).await.unwrap().cursor, 1);
    assert_eq!(
        f.bridge
            .enqueue(&f.owner, &r, f.admit("lost-hint", 1).await)
            .await
            .unwrap()
            .sequence,
        1
    );
    let worker = Uuid::new_v4();
    f.broker
        .execute(
            &f.owner,
            envelope(
                f.epoch,
                "runtime-worker",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(f.mission),
                    task_id: Some(f.task),
                    generation: Some(1),
                    token_hash: token_hash("runtime-worker"),
                    expires_in_seconds: 30,
                },
            ),
        )
        .await
        .unwrap();
    let mut assignment = envelope(
        f.epoch,
        "runtime-assign",
        2,
        Command::AssignAttempt {
            task_id: f.task,
            worker_id: worker,
        },
    );
    assignment.expected_generation = Some(0);
    let assigned = f
        .bridge
        .enqueue(&f.owner, &r, assignment.clone())
        .await
        .unwrap();
    for _ in 0..100 {
        if f.bridge.progress(&f.owner, &r).await.unwrap().cursor == assigned.sequence {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(f.bridge.progress(&f.owner, &r).await.unwrap().cursor, 2);
    assert_eq!(
        f.bridge
            .enqueue(&f.owner, &r, assignment)
            .await
            .unwrap()
            .sequence,
        2
    );
    // Real process death after canonical application. DB remains intact; replacement
    // worker replays and timers survive without a signal or a domain reapplication.
    kill_worker(&mut child);
    tokio::time::sleep(Duration::from_millis(700)).await;
    child = f.child(&queue);
    let q2 = f
        .bridge
        .enqueue(&f.owner, &r, f.admit("rejected-after-restart", 99).await)
        .await
        .unwrap();
    for _ in 0..100 {
        if f.bridge.progress(&f.owner, &r).await.unwrap().cursor == q2.sequence {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(f.bridge.progress(&f.owner, &r).await.unwrap().cursor, 3);
    let snapshot = f
        .broker
        .snapshot(&f.owner, SnapshotQuery::Task(f.task))
        .await
        .unwrap();
    for field in ["attempt_count", "operation_count", "reservation_count"] {
        assert_eq!(snapshot.data[field], 1);
    }
    let mut rolled = false;
    for _ in 0..100 {
        let description = handle
            .describe(temporalio_client::WorkflowDescribeOptions::default())
            .await
            .unwrap();
        let run = &description
            .raw_description
            .workflow_execution_info
            .unwrap()
            .execution
            .unwrap()
            .run_id;
        if run != &first_run {
            rolled = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        rolled,
        "actual Continue-As-New must occur before closing orchestration"
    );
    for _ in 0..100 {
        if f.bridge
            .progress(&f.owner, &r)
            .await
            .unwrap()
            .observed
            .as_ref()
            .is_some_and(|v| v.incarnation >= 2)
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let observed = f
        .bridge
        .progress(&f.owner, &r)
        .await
        .unwrap()
        .observed
        .expect("Activity persisted current execution");
    assert!(observed.incarnation >= 2);
    assert_ne!(observed.run_id, first_run);
    assert_eq!(
        f.bridge
            .observe_execution(&f.owner, &r, &client, Some(&first_run))
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration,
        "old-run/retried observation cannot regress canonical incarnation"
    );
    assert_eq!(
        f.bridge.progress(&f.owner, &r).await.unwrap().observed,
        Some(observed.clone())
    );
    let after_rollover = f
        .bridge
        .enqueue(&f.owner, &r, f.admit("after-rollover", 99).await)
        .await
        .unwrap();
    assert_eq!(after_rollover.sequence, 4);
    for _ in 0..100 {
        if f.bridge.progress(&f.owner, &r).await.unwrap().cursor == 4 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(f.bridge.progress(&f.owner, &r).await.unwrap().cursor, 4);
    f.bridge.close(&f.owner, &r).await.unwrap();
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        handle.get_result(WorkflowGetResultOptions::default()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(result.cursor, 4);
    assert_eq!(result.waits.len(), 1);
    assert_eq!(
        result.operations[0].limitation.as_deref(),
        Some("workflow_closed")
    );
    assert!(result.closed);
    assert_eq!(
        f.bridge
            .deliver(&f.owner, &client, &queue, &start)
            .await
            .unwrap_err()
            .code,
        ErrorCode::StaleGeneration
    );
    let terminal = f
        .bridge
        .progress(&f.owner, &r)
        .await
        .unwrap()
        .observed
        .unwrap();
    assert_eq!(terminal.lifecycle, WorkflowLifecycle::Completed);
    assert_eq!(terminal.run_id, observed.run_id);
    assert_eq!(terminal.incarnation, observed.incarnation);
    let late = f
        .bridge
        .enqueue(&f.owner, &r, f.admit("after-terminal", 99).await)
        .await
        .unwrap();
    assert_eq!(late.sequence, 5);
    assert_eq!(late.disposition.as_deref(), Some("rejected"));
    let out = std::path::PathBuf::from(std::env::var("TEMPORAL_FIXTURE_EVIDENCE").unwrap());
    std::fs::create_dir_all(&out).unwrap();
    let history = WorkflowExecutionInfo::builder()
        .namespace("default")
        .workflow_id(&r.workflow_id)
        .run_id(&first_run)
        .build()
        .bind_untyped(client.clone())
        .fetch_history(WorkflowFetchHistoryOptions::default())
        .to_json()
        .await
        .unwrap();
    std::fs::write(out.join("task-first.json"), &history).unwrap();
    assert_history_no_secret(&history, &f.owner);
    assert!(
        String::from_utf8_lossy(&history).contains("WORKFLOW_EXECUTION_CONTINUED_AS_NEW")
            || String::from_utf8_lossy(&history)
                .contains("WorkflowExecutionContinuedAsNewEventAttributes")
    );
    let mr = f
        .bridge
        .register(&f.owner, AggregateKind::Mission, f.mission, 0)
        .await
        .unwrap();
    let mstart = f
        .bridge
        .pending_wakeups(&f.owner, &mr)
        .await
        .unwrap()
        .remove(0);
    f.bridge
        .deliver(&f.owner, &client, &queue, &mstart)
        .await
        .unwrap();
    f.bridge.close(&f.owner, &mr).await.unwrap();
    let mh = client.get_workflow_handle::<MissionWorkflow>(&mr.workflow_id);
    let mresult = tokio::time::timeout(
        Duration::from_secs(10),
        mh.get_result(WorkflowGetResultOptions::default()),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(mresult.closed);
    let mjson = mh
        .fetch_history(WorkflowFetchHistoryOptions::default())
        .to_json()
        .await
        .unwrap();
    std::fs::write(out.join("mission.json"), &mjson).unwrap();
    assert_history_no_secret(&mjson, &f.owner);
    kill_worker(&mut child);
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
    let last = handle
        .fetch_history(WorkflowFetchHistoryOptions::default())
        .to_json()
        .await
        .unwrap();
    std::fs::write(out.join("task-terminal.json"), &last).unwrap();
    assert_history_no_secret(&last, &f.owner);
    use base64::Engine;
    let json: serde_json::Value = serde_json::from_slice(&last).unwrap();
    let data = json["events"][0]["attributes"]["WorkflowExecutionStartedEventAttributes"]["input"]
        ["payloads"][0]["data"]
        .as_str()
        .unwrap();
    let retained: WorkflowInput = serde_json::from_slice(
        &base64::engine::general_purpose::STANDARD
            .decode(data)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(retained.cursor, 3);
    assert_eq!(retained.waits, result.waits);
    assert_eq!(retained.reference, r);

    let invocations = std::fs::read_to_string(out.join("activity-invocations.jsonl")).unwrap();
    assert!(
        invocations
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .any(|v| v["attempt"].as_u64().unwrap() >= 2),
        "actual SDK retry required"
    );
    for name in ["task-first.json", "task-terminal.json", "mission.json"] {
        let data = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/efforts/team-control-plane/evidence/ticket13-temporal-bridge")
                .join(name),
        )
        .unwrap();
        replayer
            .replay_workflow(temporalio_client::WorkflowHistory::from_json(&data).unwrap())
            .await
            .expect("initial v1 retained history remains replay compatible");
    }
    std::fs::write(
        out.join("registry-observation.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"first_run":first_run,"after_can":observed,"terminal":terminal}),
        )
        .unwrap(),
    )
    .unwrap();
    for data in [history, mjson, last] {
        let h = temporalio_client::WorkflowHistory::from_json(&data).unwrap();
        replayer.replay_workflow(h).await.unwrap();
    }
}
