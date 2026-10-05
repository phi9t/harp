#[path = "support/gateway_fault.rs"]
mod gateway_fault;
use gateway_fault::*;
use team_control_plane::{gateway::*, *};
use uuid::Uuid;
fn identity(store: &str) -> RuntimeIdentity {
    RuntimeIdentity {
        runtime_kind: "fault".into(),
        runtime_version: "v1".into(),
        deployment: "fixture".into(),
        thread_store: store.into(),
        endpoint: "local".into(),
        environment: "disposable".into(),
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
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn durable_gateway_contract() {
    gateway_contract_with_malformed_reply(String::new()).await;
    gateway_contract_with_malformed_reply("x".repeat(1025)).await;
}

async fn gateway_contract_with_malformed_reply(malformed_reply: String) {
    let dsn = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").expect("real PG fixture required");
    let (admin, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    let schema = format!("gateway_{}", Uuid::new_v4().simple());
    admin
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{dsn} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let mission = Uuid::new_v4();
    let task = Uuid::new_v4();
    let input = GitInput {
        repository: "fixture".into(),
        commit: "a".repeat(40),
        path: "SPEC.md".into(),
        digest: "b".repeat(64),
    };
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "m",
                0,
                Command::CreateMission {
                    id: mission,
                    objective: "gateway".into(),
                    input: input.clone(),
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
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "t",
                0,
                Command::CreateTask {
                    id: task,
                    mission_id: mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "faults".into(),
                        input,
                        acceptance: vec!["one effect or quarantine".into()],
                        deliverables: vec!["journal".into()],
                        write_scope: vec!["fixture".into()],
                    },
                },
            ),
        )
        .await
        .unwrap();
    let digest = broker
        .snapshot(&owner, SnapshotQuery::Task(task))
        .await
        .unwrap()
        .data["spec_digest"]
        .as_str()
        .unwrap()
        .to_string();
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "a",
                1,
                Command::AdmitTask {
                    task_id: task,
                    spec_digest: digest,
                },
            ),
        )
        .await
        .unwrap();
    let worker = Uuid::new_v4();
    let worker_token = "fixture-worker";
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "grant",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(mission),
                    task_id: Some(task),
                    generation: Some(1),
                    token_hash: token_hash(worker_token),
                    expires_in_seconds: 60,
                },
            ),
        )
        .await
        .unwrap();
    let admitted = broker
        .execute(
            &owner,
            envelope(
                epoch,
                "assign",
                2,
                Command::AssignAttempt {
                    task_id: task,
                    worker_id: worker,
                },
            ),
        )
        .await
        .unwrap();
    let gateway = Gateway::new(broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(
            &owner,
            binding,
            admitted.operation_id.unwrap(),
            &identity("store"),
            Some("thread"),
            admitted.generation,
        )
        .await
        .unwrap();
    let op = Intent {
        id: Uuid::new_v4(),
        admission_id: admitted.operation_id.unwrap(),
        binding_id: binding,
        generation: admitted.generation,
        kind: OperationKind::Start,
        arguments: serde_json::json!({"prompt":"zero models"}),
    };
    let prepared = gateway.prepare(&owner, op.clone()).await.unwrap();
    assert_eq!(prepared.state, OperationState::Prepared);
    assert_eq!(gateway.prepare(&owner, op.clone()).await.unwrap().id, op.id);
    let directory = std::env::temp_dir().join(format!("gateway-fault-{}", Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let mut runtime = FaultRuntime::new(directory.clone(), Fault::DropReply, identity("store"));
    let unknown = gateway.dispatch(&owner, op.id, &mut runtime).await.unwrap();
    assert_eq!(unknown.state, OperationState::OutcomeUnknown);
    assert_eq!(
        gateway
            .dispatch(&owner, op.id, &mut runtime)
            .await
            .unwrap()
            .state,
        OperationState::OutcomeUnknown
    );
    assert_eq!(runtime.effect_count().unwrap(), 1);
    assert_eq!(runtime.invocation_count().unwrap(), 1);
    assert_eq!(
        gateway
            .reconcile(&owner, op.id, &mut runtime)
            .await
            .unwrap()
            .state,
        OperationState::Confirmed
    );
    assert_eq!(runtime.effect_count().unwrap(), 1);
    assert_eq!(runtime.invocation_count().unwrap(), 1);
    let mut different = op.clone();
    different.arguments = serde_json::json!({"different":true});
    assert_eq!(
        gateway.prepare(&owner, different).await.unwrap_err().code,
        ErrorCode::IdempotencyConflict
    );

    assert_eq!(
        gateway.read("forged", op.id).await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    assert_eq!(
        gateway
            .dispatch(worker_token, op.id, &mut runtime)
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    assert_eq!(gateway.read(worker_token, op.id).await.unwrap().id, op.id);
    assert_eq!(
        gateway
            .bind(
                &owner,
                Uuid::new_v4(),
                op.admission_id,
                &identity("store"),
                Some("thread"),
                admitted.generation
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict
    );
    // Crash boundaries: committed intent, committed claim before RPC, external
    // effect with no durable reply. Panicking task abandons its dispatch stack.
    for (index, fault, expected, effects) in [
        (0, Fault::None, OperationState::Confirmed, 1),
        (1, Fault::KnownFailure, OperationState::FailedKnown, 0),
        (
            2,
            Fault::MissingCapabilities,
            OperationState::FailedKnown,
            0,
        ),
        (3, Fault::CrashBeforeEffect, OperationState::Dispatching, 0),
        (4, Fault::CrashAfterEffect, OperationState::Dispatching, 1),
        (5, Fault::None, OperationState::Confirmed, 1),
        (6, Fault::None, OperationState::Confirmed, 1),
    ] {
        let bind = Uuid::new_v4();
        let identity = identity(&format!("store{index}"));
        gateway
            .bind(&owner, bind, op.admission_id, &identity, Some("known"), 1)
            .await
            .unwrap();
        let intent = Intent {
            id: Uuid::new_v4(),
            binding_id: bind,
            kind: match index {
                5 => OperationKind::Recover,
                6 => OperationKind::Cancel,
                _ => OperationKind::Steer,
            },
            ..op.clone()
        };
        gateway.prepare(&owner, intent.clone()).await.unwrap();
        assert_eq!(
            gateway.read(&owner, intent.id).await.unwrap().state,
            OperationState::Prepared
        );
        let dir = directory.join(format!("case{index}"));
        std::fs::create_dir(&dir).unwrap();
        let mut fixture = FaultRuntime::new(dir.clone(), fault, identity.clone());
        if matches!(fault, Fault::CrashBeforeEffect | Fault::CrashAfterEffect) {
            let g = gateway.clone();
            let bearer = owner.clone();
            let id = intent.id;
            let result =
                tokio::spawn(async move { g.dispatch(&bearer, id, &mut fixture).await }).await;
            assert!(result.unwrap_err().is_panic());
        } else if index == 0 {
            let mut duplicate = FaultRuntime::new(dir.clone(), fault, identity.clone());
            let (a, b) = tokio::join!(
                gateway.dispatch(&owner, intent.id, &mut fixture),
                gateway.dispatch(&owner, intent.id, &mut duplicate)
            );
            assert_eq!(a.unwrap().state, expected);
            assert_eq!(b.unwrap().state, expected);
        } else {
            assert_eq!(
                gateway
                    .dispatch(&owner, intent.id, &mut fixture)
                    .await
                    .unwrap()
                    .state,
                expected
            );
        }
        let mut observer = FaultRuntime::new(dir.clone(), Fault::None, identity.clone());
        assert_eq!(observer.effect_count().unwrap(), effects);
        assert_eq!(
            observer.invocation_count().unwrap(),
            if matches!(fault, Fault::KnownFailure) {
                1
            } else {
                effects
            }
        );
        assert_eq!(
            gateway.read(&owner, intent.id).await.unwrap().state,
            expected
        );
        assert_eq!(
            gateway
                .dispatch(&owner, intent.id, &mut observer)
                .await
                .unwrap()
                .state,
            expected
        );
        assert_eq!(observer.effect_count().unwrap(), effects);
        assert_eq!(
            observer.invocation_count().unwrap(),
            if matches!(fault, Fault::KnownFailure) {
                1
            } else {
                effects
            }
        );
        if index == 3 {
            let newcomer = Gateway::new(broker.clone());
            assert_eq!(
                newcomer
                    .reconcile(&owner, intent.id, &mut observer)
                    .await
                    .unwrap()
                    .state,
                OperationState::Quarantined,
                "absence cannot fence a retained old controller"
            );
        }
        if expected == OperationState::Dispatching {
            observer.fault = Fault::EmptyLossyHistory;
            assert_eq!(
                gateway
                    .reconcile(&owner, intent.id, &mut observer)
                    .await
                    .unwrap()
                    .state,
                OperationState::Quarantined
            );
            observer.fault = Fault::None;
            assert_eq!(
                gateway
                    .reconcile(&owner, intent.id, &mut observer)
                    .await
                    .unwrap()
                    .state,
                if effects == 1 {
                    OperationState::Confirmed
                } else {
                    OperationState::FailedKnown
                }
            );
        }
        if matches!(fault, Fault::MissingCapabilities) {
            assert!(
                gateway
                    .read(&owner, intent.id)
                    .await
                    .unwrap()
                    .evidence
                    .iter()
                    .any(|e| matches!(e,Evidence::Limitation {capability} if capability=="steer"))
            );
        }
    }
    // A second admitted episode allows an actual start-process kill independently
    // of the first lost-start acknowledgment (capacity remains bounded at two).
    let child_task = Uuid::new_v4();
    let child_worker = Uuid::new_v4();
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "child-task",
                0,
                Command::CreateTask {
                    id: child_task,
                    mission_id: mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "process kill".into(),
                        input: GitInput {
                            repository: "fixture".into(),
                            commit: "a".repeat(40),
                            path: "SPEC.md".into(),
                            digest: "b".repeat(64),
                        },
                        acceptance: vec!["kill boundaries".into()],
                        deliverables: vec!["journal".into()],
                        write_scope: vec!["fixture".into()],
                    },
                },
            ),
        )
        .await
        .unwrap();
    let child_digest = broker
        .snapshot(&owner, SnapshotQuery::Task(child_task))
        .await
        .unwrap()
        .data["spec_digest"]
        .as_str()
        .unwrap()
        .to_string();
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "child-admit",
                1,
                Command::AdmitTask {
                    task_id: child_task,
                    spec_digest: child_digest,
                },
            ),
        )
        .await
        .unwrap();
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "child-grant",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: child_worker,
                    role: Role::Worker,
                    mission_id: Some(mission),
                    task_id: Some(child_task),
                    generation: Some(1),
                    token_hash: token_hash("child-fixture-token"),
                    expires_in_seconds: 60,
                },
            ),
        )
        .await
        .unwrap();
    let child_admitted = broker
        .execute(
            &owner,
            envelope(
                epoch,
                "child-assign",
                2,
                Command::AssignAttempt {
                    task_id: child_task,
                    worker_id: child_worker,
                },
            ),
        )
        .await
        .unwrap();
    // A creation reply/observation must never adopt malformed physical identity.
    let malformed_identity = identity("malformed-create");
    let malformed_binding = Uuid::new_v4();
    gateway
        .bind(
            &owner,
            malformed_binding,
            child_admitted.operation_id.unwrap(),
            &malformed_identity,
            None,
            1,
        )
        .await
        .unwrap();
    let malformed = Intent {
        id: Uuid::new_v4(),
        admission_id: child_admitted.operation_id.unwrap(),
        binding_id: malformed_binding,
        kind: OperationKind::Create,
        ..op.clone()
    };
    gateway.prepare(&owner, malformed.clone()).await.unwrap();
    let malformed_dir = directory.join("malformed-create");
    std::fs::create_dir(&malformed_dir).unwrap();
    let mut malformed_runtime = MalformedRuntime {
        runtime: FaultRuntime::new(malformed_dir, Fault::None, malformed_identity),
        thread: malformed_reply,
    };
    let reply = gateway
        .dispatch(&owner, malformed.id, &mut malformed_runtime)
        .await
        .unwrap();
    assert_eq!(
        reply.state,
        OperationState::Quarantined,
        "malformed create reply must not confirm"
    );
    assert!(
        reply.thread_id.is_none(),
        "malformed identity must not be adopted"
    );
    let invalid_start = Intent {
        id: Uuid::new_v4(),
        kind: OperationKind::Start,
        ..malformed.clone()
    };
    assert_eq!(
        gateway
            .prepare(&owner, invalid_start.clone())
            .await
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    for malformed_thread in ["x".repeat(1025), String::new()] {
        malformed_runtime.thread = malformed_thread;
        let observation = gateway
            .reconcile(&owner, malformed.id, &mut malformed_runtime)
            .await
            .unwrap();
        assert_eq!(
            observation.state,
            OperationState::Quarantined,
            "malformed authoritative observation must not confirm or prove absence"
        );
        assert!(
            observation.thread_id.is_none(),
            "malformed observation must not adopt identity"
        );
        assert_eq!(
            gateway
                .prepare(&owner, invalid_start.clone())
                .await
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(malformed_runtime.runtime.effect_count().unwrap(), 1);
    assert_eq!(malformed_runtime.runtime.invocation_count().unwrap(), 1);
    // Actual gateway-process SIGKILLs. External inventory survives each child.
    for (index, point, expected, effects) in [
        (0, "before_intent", None, 0),
        (1, "after_intent", Some(OperationState::Prepared), 0),
        (2, "after_dispatch", Some(OperationState::Dispatching), 0),
        (
            3,
            "after_effect_reply",
            Some(OperationState::Dispatching),
            1,
        ),
        (4, "after_commit", Some(OperationState::Confirmed), 1),
        (
            5,
            "after_effect_reply",
            Some(OperationState::Dispatching),
            1,
        ),
    ] {
        let identity = identity(&format!("sigkill{index}"));
        let intent = Intent {
            id: Uuid::new_v4(),
            binding_id: Uuid::new_v4(),
            kind: match index {
                5 => OperationKind::Create,
                3 => OperationKind::Start,
                _ => OperationKind::Steer,
            },
            admission_id: if index == 3 {
                child_admitted.operation_id.unwrap()
            } else {
                op.admission_id
            },
            ..op.clone()
        };
        let child_dir = directory.join(format!("sigkill{index}"));
        std::fs::create_dir(&child_dir).unwrap();
        kill_at_boundary(&dsn, &owner, epoch, &intent, &identity, &child_dir, point);
        let mut survivor = FaultRuntime::new(child_dir, Fault::None, identity);
        assert_eq!(survivor.effect_count().unwrap(), effects);
        assert_eq!(survivor.invocation_count().unwrap(), effects);
        if let Some(expected) = expected {
            let recorded = gateway.read(&owner, intent.id).await.unwrap();
            assert_eq!(recorded.state, expected);
            if index == 5 {
                assert!(
                    recorded.thread_id.is_none(),
                    "lost create reply leaves no known thread ID"
                );
            }
            let repeated = gateway
                .dispatch(&owner, intent.id, &mut survivor)
                .await
                .unwrap();
            assert_eq!(
                repeated.state,
                if expected == OperationState::Prepared {
                    OperationState::Quarantined
                } else {
                    expected
                }
            );
            if expected != OperationState::Confirmed {
                assert_eq!(
                    gateway
                        .reconcile(&owner, intent.id, &mut survivor)
                        .await
                        .unwrap()
                        .state,
                    OperationState::Quarantined
                );
            }
            assert_eq!(survivor.effect_count().unwrap(), effects);
            assert_eq!(survivor.invocation_count().unwrap(), effects);
        } else {
            assert_eq!(
                gateway.read(&owner, intent.id).await.unwrap_err().code,
                ErrorCode::NotFound
            );
        }
    }
    // Immutable evidence rejects attempted mutation through a direct database probe.
    let (db, conn) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        let _ = conn.await;
    });
    assert!(
        db.execute("UPDATE gateway_evidence SET evidence='{}'", &[])
            .await
            .is_err()
    );
    let racing_identity = identity("race");
    let a = Gateway::new(broker.clone());
    let b = Gateway::new(broker.clone());
    let bind_a = Uuid::new_v4();
    let bind_b = Uuid::new_v4();
    let (ra, rb) = tokio::join!(
        a.bind(
            &owner,
            bind_a,
            op.admission_id,
            &racing_identity,
            Some("race-thread"),
            1
        ),
        b.bind(
            &owner,
            bind_b,
            op.admission_id,
            &racing_identity,
            Some("race-thread"),
            1
        )
    );
    assert_ne!(
        ra.is_ok(),
        rb.is_ok(),
        "only one controller may acquire physical control"
    );
    let (winner, bind) = if ra.is_ok() { (a, bind_a) } else { (b, bind_b) };
    let race = Intent {
        id: Uuid::new_v4(),
        binding_id: bind,
        kind: OperationKind::Steer,
        ..op.clone()
    };
    winner.prepare(&owner, race.clone()).await.unwrap();
    let race_dir = directory.join("race");
    std::fs::create_dir(&race_dir).unwrap();
    let mut race_runtime = FaultRuntime::new(race_dir, Fault::None, racing_identity);
    assert_eq!(
        winner
            .dispatch(&owner, race.id, &mut race_runtime)
            .await
            .unwrap()
            .state,
        OperationState::Quarantined
    );
    assert_eq!(race_runtime.effect_count().unwrap(), 0);
    assert_eq!(race_runtime.invocation_count().unwrap(), 0);
    let pending = Intent {
        id: Uuid::new_v4(),
        kind: OperationKind::Cancel,
        ..op.clone()
    };
    gateway.prepare(&owner, pending.clone()).await.unwrap();
    let newcomer = Gateway::new(broker.clone());
    assert_eq!(
        newcomer
            .dispatch(&owner, pending.id, &mut runtime)
            .await
            .unwrap()
            .state,
        OperationState::Quarantined
    );
    assert_eq!(runtime.effect_count().unwrap(), 1);
    assert_eq!(runtime.invocation_count().unwrap(), 1);
    let mut alias_identity = identity("store");
    alias_identity.endpoint = "endpoint-alias".into();
    assert_eq!(
        gateway
            .bind(
                &owner,
                Uuid::new_v4(),
                op.admission_id,
                &alias_identity,
                Some("thread"),
                1
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict
    );
    let replacement = Gateway::new(broker.clone());
    assert_eq!(
        replacement
            .bind(
                &owner,
                Uuid::new_v4(),
                op.admission_id,
                &identity("store"),
                Some("thread"),
                admitted.generation
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::TemporarilyUnavailable
    );
    assert_eq!(
        gateway.read(&owner, op.id).await.unwrap().state,
        OperationState::Confirmed
    );
    db.execute("UPDATE tasks SET generation=2 WHERE id=$1", &[&task])
        .await
        .unwrap();
    assert_eq!(
        gateway.read(&owner, op.id).await.unwrap_err().code,
        ErrorCode::StaleGeneration
    );
    db.execute("UPDATE tasks SET generation=1 WHERE id=$1", &[&task])
        .await
        .unwrap();
    db.execute("UPDATE credentials SET expires_at=clock_timestamp()-interval '1 second' WHERE token_hash=$1",&[&token_hash(worker_token)]).await.unwrap();
    assert_eq!(
        gateway.read(worker_token, op.id).await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    let next_epoch = Uuid::new_v4();
    let next_owner = broker.rotate_epoch(&owner, next_epoch).await.unwrap();
    assert_eq!(
        gateway.read(&owner, op.id).await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    let restored = Gateway::new(Broker::connect(&dsn, next_epoch).await.unwrap());
    assert_eq!(
        restored.read(&next_owner, op.id).await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    std::fs::remove_dir_all(directory).unwrap();
    admin
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}

/// Child-process entrypoint; normal suite execution performs no child action.
#[tokio::test]
async fn gateway_crash_child() {
    if std::env::var("GATEWAY_CRASH_CHILD").is_err() {
        return;
    }
    use std::io::Write;
    fn boundary() -> ! {
        println!("GATEWAY_BOUNDARY_READY");
        std::io::stdout().flush().unwrap();
        loop {
            std::thread::park();
        }
    }
    struct CrashAdapter {
        runtime: FaultRuntime,
        point: String,
    }
    impl RuntimeAdapter for CrashAdapter {
        fn identity(&self) -> RuntimeIdentity {
            self.runtime.identity()
        }
        fn inspect(&self) -> CapabilityProfile {
            self.runtime.inspect()
        }
        fn observe(&mut self, intent: &Intent, thread: Option<&str>) -> Observation {
            self.runtime.observe(intent, thread)
        }
        fn execute(&mut self, intent: &Intent, thread: Option<&str>) -> RuntimeReply {
            if self.point == "after_dispatch" {
                boundary();
            }
            let result = self.runtime.execute(intent, thread);
            if self.point == "after_effect_reply" {
                boundary();
            }
            result
        }
    }
    let dsn = std::env::var("GATEWAY_CHILD_DSN").unwrap();
    let owner = std::env::var("GATEWAY_CHILD_OWNER").unwrap();
    let epoch = Uuid::parse_str(&std::env::var("GATEWAY_CHILD_EPOCH").unwrap()).unwrap();
    let intent: Intent =
        serde_json::from_str(&std::env::var("GATEWAY_CHILD_INTENT").unwrap()).unwrap();
    let identity: RuntimeIdentity =
        serde_json::from_str(&std::env::var("GATEWAY_CHILD_IDENTITY").unwrap()).unwrap();
    let point = std::env::var("GATEWAY_CHILD_POINT").unwrap();
    let gateway = Gateway::new(Broker::connect(&dsn, epoch).await.unwrap());
    gateway
        .bind(
            &owner,
            intent.binding_id,
            intent.admission_id,
            &identity,
            if intent.kind == OperationKind::Create {
                None
            } else {
                Some("child-known-thread")
            },
            intent.generation,
        )
        .await
        .unwrap();
    if point == "before_intent" {
        boundary();
    }
    gateway.prepare(&owner, intent.clone()).await.unwrap();
    if point == "after_intent" {
        boundary();
    }
    let mut adapter = CrashAdapter {
        runtime: FaultRuntime::new(
            std::env::var("GATEWAY_CHILD_DIRECTORY").unwrap().into(),
            Fault::None,
            identity,
        ),
        point: point.clone(),
    };
    gateway
        .dispatch(&owner, intent.id, &mut adapter)
        .await
        .unwrap();
    assert_eq!(point, "after_commit");
    boundary();
}

fn kill_at_boundary(
    dsn: &str,
    owner: &str,
    epoch: Uuid,
    intent: &Intent,
    identity: &RuntimeIdentity,
    directory: &std::path::Path,
    point: &str,
) {
    use std::io::{BufRead, BufReader};
    use std::os::unix::process::ExitStatusExt;
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "gateway_crash_child", "--nocapture"])
        .env("GATEWAY_CRASH_CHILD", "1")
        .env("GATEWAY_CHILD_DSN", dsn)
        .env("GATEWAY_CHILD_OWNER", owner)
        .env("GATEWAY_CHILD_EPOCH", epoch.to_string())
        .env(
            "GATEWAY_CHILD_INTENT",
            serde_json::to_string(intent).unwrap(),
        )
        .env(
            "GATEWAY_CHILD_IDENTITY",
            serde_json::to_string(identity).unwrap(),
        )
        .env("GATEWAY_CHILD_DIRECTORY", directory)
        .env("GATEWAY_CHILD_POINT", point)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .unwrap();
    let out = child.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(out)
            .lines()
            .map_while(std::result::Result::ok)
        {
            if line.contains("GATEWAY_BOUNDARY_READY") {
                let _ = sender.send(());
                break;
            }
        }
    });
    let ready = receiver.recv_timeout(std::time::Duration::from_secs(30));
    child.kill().unwrap();
    let status = child.wait().unwrap();
    reader.join().unwrap();
    assert!(
        ready.is_ok(),
        "child boundary {point}: {ready:?}, status {status}"
    );
    assert_eq!(
        status.signal(),
        Some(9),
        "fixture must kill the actual controller process"
    );
}

struct MalformedRuntime {
    runtime: FaultRuntime,
    thread: String,
}
impl RuntimeAdapter for MalformedRuntime {
    fn identity(&self) -> RuntimeIdentity {
        self.runtime.identity()
    }
    fn inspect(&self) -> CapabilityProfile {
        self.runtime.inspect()
    }
    fn execute(&mut self, intent: &Intent, thread: Option<&str>) -> RuntimeReply {
        match self.runtime.execute(intent, thread) {
            RuntimeReply::Confirmed { effect_id, .. } => RuntimeReply::Confirmed {
                effect_id,
                thread_id: Some(self.thread.clone()),
            },
            other => other,
        }
    }
    fn observe(&mut self, intent: &Intent, thread: Option<&str>) -> Observation {
        match self.runtime.observe(intent, thread) {
            Observation::Effect { effect_id, .. } => Observation::Effect {
                effect_id,
                thread_id: Some(self.thread.clone()),
            },
            other => other,
        }
    }
}

#[test]
fn fault_runtime_creates_an_effect_for_every_repeated_mutation_rpc() {
    let directory = std::env::temp_dir().join(format!("nonidempotent-fault-{}", Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let mut runtime = FaultRuntime::new(directory.clone(), Fault::None, identity("repeated-rpc"));
    assert!(!runtime.inspect().native_deduplication);
    let intent = Intent {
        id: Uuid::new_v4(),
        admission_id: Uuid::new_v4(),
        binding_id: Uuid::new_v4(),
        generation: 1,
        kind: OperationKind::Start,
        arguments: serde_json::json!({"prompt":"zero models"}),
    };
    runtime.execute(&intent, Some("known"));
    runtime.execute(&intent, Some("known"));
    assert_eq!(
        runtime.effect_count().unwrap(),
        2,
        "same operation ID must not deduplicate the external fixture's effects"
    );
    assert_eq!(runtime.invocation_count().unwrap(), 2);
    assert!(
        matches!(
            runtime.observe(&intent, Some("known")),
            Observation::Inconclusive(_)
        ),
        "multiple non-idempotent effects must not appear as one adoptable execution"
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn fault_runtime_surfaces_unexpected_subprocess_failure() {
    let directory = std::env::temp_dir().join(format!("absent-fault-{}", Uuid::new_v4()));
    let mut runtime =
        FaultRuntime::new(directory, Fault::DropReply, identity("unexpected-failure"));
    let intent = Intent {
        id: Uuid::new_v4(),
        admission_id: Uuid::new_v4(),
        binding_id: Uuid::new_v4(),
        generation: 1,
        kind: OperationKind::Start,
        arguments: serde_json::json!({}),
    };
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.execute(&intent, Some("known"))
    }));
    assert!(
        failed.is_err(),
        "an unexpected subprocess failure must fail the fixture, even when reply dropping is requested"
    );
}
