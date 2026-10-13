use team_control_plane::*;
use uuid::Uuid;
fn input() -> GitInput {
    GitInput {
        repository: "fixture-repo".into(),
        commit: "a".repeat(40),
        path: "SPEC.md".into(),
        digest: "b".repeat(64),
    }
}
fn policy() -> Policy {
    Policy {
        capacity: 2,
        max_attempts: 8,
        max_episodes: 8,
        max_repairs: 3,
        episode_deadline_seconds: 30,
    }
}
fn env(epoch: Uuid, key: &str, revision: i64, command: Command) -> CommandEnvelope {
    CommandEnvelope {
        schema_version: 1,
        key: key.into(),
        expected_revision: revision,
        expected_generation: None,
        recovery_epoch: epoch,
        command,
    }
}
#[tokio::test]
async fn admission_is_durable_atomic_and_idempotent() {
    let dsn = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN")
        .expect("run tests/run-postgres.sh; real PostgreSQL required");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let mission = Uuid::new_v4();
    broker
        .execute(
            &owner,
            env(
                epoch,
                "mission",
                0,
                Command::CreateMission {
                    id: mission,
                    objective: "bounded fixture".into(),
                    input: input(),
                    policy: policy(),
                },
            ),
        )
        .await
        .expect("owner creates mission");
    let task = Uuid::new_v4();
    let spec = TaskSpec {
        dependencies: Vec::new(),
        objective: "fixture task".into(),
        input: input(),
        acceptance: vec!["verified".into()],
        deliverables: vec!["result".into()],
        write_scope: vec!["src".into()],
    };
    broker
        .execute(
            &owner,
            env(
                epoch,
                "task",
                0,
                Command::CreateTask {
                    id: task,
                    mission_id: mission,
                    spec,
                },
            ),
        )
        .await
        .unwrap();
    let snapshot = broker
        .snapshot(&owner, SnapshotQuery::Task(task))
        .await
        .unwrap();
    let digest = snapshot.data["spec_digest"].as_str().unwrap().to_string();
    broker
        .execute(
            &owner,
            env(
                epoch,
                "admit",
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
    broker
        .execute(
            &owner,
            env(
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
                    token_hash: "c".repeat(64),
                    expires_in_seconds: 60,
                },
            ),
        )
        .await
        .unwrap();
    let cmd = env(
        epoch,
        "assign",
        2,
        Command::AssignAttempt {
            task_id: task,
            worker_id: worker,
        },
    );
    let other = Broker::connect(&dsn, epoch).await.unwrap();
    let (a, b) = tokio::join!(
        broker.execute(&owner, cmd.clone()),
        other.execute(&owner, cmd.clone())
    );
    let a = a.unwrap();
    assert_eq!(a, b.unwrap());
    assert!(a.attempt_id.is_some());
    let state = other
        .snapshot(&owner, SnapshotQuery::Task(task))
        .await
        .unwrap();
    assert_eq!(state.data["attempt_count"], 1);
    assert_eq!(state.data["reservation_count"], 1);
    assert_eq!(state.data["operation_count"], 1);
    assert_eq!(state.data["outbox_count"], 1);
    let mut changed = cmd;
    changed.command = Command::AssignAttempt {
        task_id: task,
        worker_id: Uuid::new_v4(),
    };
    assert_eq!(
        broker.execute(&owner, changed).await.unwrap_err().code,
        ErrorCode::IdempotencyConflict
    );
    assert_eq!(
        broker
            .execute(
                "forged",
                env(
                    epoch,
                    "fake",
                    3,
                    Command::AssignAttempt {
                        task_id: task,
                        worker_id: worker
                    }
                )
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    let stale = Broker::connect(&dsn, Uuid::new_v4()).await.unwrap();
    assert_eq!(
        stale
            .execute(
                &owner,
                env(
                    epoch,
                    "assign",
                    2,
                    Command::AssignAttempt {
                        task_id: task,
                        worker_id: worker
                    }
                )
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
}

async fn setup() -> (String, Broker, String, Uuid) {
    let base = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN")
        .expect("run tests/run-postgres.sh; real PostgreSQL required");
    let (admin, connection) = tokio_postgres::connect(&base, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    let schema = format!("fixture_{}", Uuid::new_v4().simple());
    admin
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{base} options='-csearch_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let token = broker.bootstrap_owner().await.unwrap();
    (dsn, broker, token, epoch)
}
async fn sql(dsn: &str, sql: &str) {
    let (c, connection) = tokio_postgres::connect(dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    c.batch_execute(sql).await.unwrap();
}
async fn mission(b: &Broker, owner: &str, e: Uuid, p: Policy) -> Uuid {
    let id = Uuid::new_v4();
    b.execute(
        owner,
        env(
            e,
            &id.to_string(),
            0,
            Command::CreateMission {
                id,
                objective: "fixture".into(),
                input: input(),
                policy: p,
            },
        ),
    )
    .await
    .unwrap();
    id
}
async fn task(b: &Broker, owner: &str, e: Uuid, m: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    b.execute(
        owner,
        env(
            e,
            &id.to_string(),
            0,
            Command::CreateTask {
                id,
                mission_id: m,
                spec: TaskSpec {
                    dependencies: Vec::new(),
                    objective: "fixture".into(),
                    input: input(),
                    acceptance: vec!["verified".into()],
                    deliverables: vec!["artifact".into()],
                    write_scope: vec!["src".into()],
                },
            },
        ),
    )
    .await
    .unwrap();
    id
}
async fn admit(b: &Broker, o: &str, e: Uuid, t: Uuid, revision: i64) {
    let s = b.snapshot(o, SnapshotQuery::Task(t)).await.unwrap();
    b.execute(
        o,
        env(
            e,
            &Uuid::new_v4().to_string(),
            revision,
            Command::AdmitTask {
                task_id: t,
                spec_digest: s.data["spec_digest"].as_str().unwrap().into(),
            },
        ),
    )
    .await
    .unwrap();
}
async fn grant(
    b: &Broker,
    o: &str,
    e: Uuid,
    role: Role,
    m: Option<Uuid>,
    t: Option<Uuid>,
    g: Option<i64>,
) -> (Uuid, String) {
    let principal = Uuid::new_v4();
    let token = Uuid::new_v4().to_string();
    b.execute(
        o,
        env(
            e,
            &Uuid::new_v4().to_string(),
            0,
            Command::GrantCredential {
                credential_id: Uuid::new_v4(),
                principal_id: principal,
                role,
                mission_id: m,
                task_id: t,
                generation: g,
                token_hash: token_hash(&token),
                expires_in_seconds: 60,
            },
        ),
    )
    .await
    .unwrap();
    (principal, token)
}
async fn ready(b: &Broker, o: &str, e: Uuid, m: Uuid) -> (Uuid, Uuid, String) {
    let t = task(b, o, e, m).await;
    admit(b, o, e, t, 1).await;
    let (w, token) = grant(b, o, e, Role::Worker, Some(m), Some(t), Some(1)).await;
    (t, w, token)
}
#[tokio::test]
async fn authority_expiry_scopes_generations_and_epoch_rotation() {
    let (dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let other = mission(&b, &o, e, policy()).await;
    let (t, w, worker) = ready(&b, &o, e, m).await;
    let (_, lead) = grant(&b, &o, e, Role::Lead, Some(m), Some(t), None).await;
    assert_eq!(
        b.execute(
            &worker,
            env(
                e,
                "self",
                2,
                Command::AssignAttempt {
                    task_id: t,
                    worker_id: w
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::StaleGeneration
    );
    let other_task = task(&b, &o, e, other).await;
    assert_eq!(
        b.snapshot(&lead, SnapshotQuery::Task(other_task))
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    assert_eq!(
        b.snapshot(&lead, SnapshotQuery::Mission(other))
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    let mut cmd = env(
        e,
        "assignment",
        2,
        Command::AssignAttempt {
            task_id: t,
            worker_id: w,
        },
    );
    cmd.expected_generation = Some(99);
    assert_eq!(
        b.execute(&lead, cmd.clone()).await.unwrap_err().code,
        ErrorCode::StaleGeneration
    );
    cmd.expected_generation = Some(0);
    cmd.key = "assignment-corrected".into();
    b.execute(&lead, cmd.clone()).await.unwrap();
    assert_eq!(
        b.execute(
            &worker,
            env(
                e,
                "self2",
                3,
                Command::AssignAttempt {
                    task_id: t,
                    worker_id: w
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::Unauthorized
    );
    sql(&dsn,&format!("UPDATE credentials SET expires_at=clock_timestamp()-interval '1 second' WHERE token_hash='{}'",token_hash(&lead))).await;
    assert_eq!(
        b.execute(&lead, cmd).await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    let newer = Uuid::new_v4();
    let new_owner = b.rotate_epoch(&o, newer).await.unwrap();
    assert_eq!(
        b.snapshot(&o, SnapshotQuery::Task(t))
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    let nb = Broker::connect(&dsn, newer).await.unwrap();
    assert!(
        nb.snapshot(&new_owner, SnapshotQuery::Task(t))
            .await
            .is_ok()
    );
    assert_eq!(
        b.bootstrap_owner().await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    assert!(!format!("{b:?}").contains(&dsn));
}
#[tokio::test]
async fn dependencies_validation_and_immutable_approval() {
    let (dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let m2 = mission(&b, &o, e, policy()).await;
    let a = task(&b, &o, e, m).await;
    let c = task(&b, &o, e, m).await;
    let foreign = task(&b, &o, e, m2).await;
    b.execute(
        &o,
        env(
            e,
            "dep1",
            1,
            Command::SetDependencies {
                task_id: a,
                dependencies: vec![c],
            },
        ),
    )
    .await
    .unwrap();
    for (key, target) in [("cycle", a), ("foreign", foreign)] {
        assert_eq!(
            b.execute(
                &o,
                env(
                    e,
                    key,
                    1,
                    Command::SetDependencies {
                        task_id: c,
                        dependencies: vec![target]
                    }
                )
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::DependencyUnmet
        );
    }
    assert_eq!(
        b.execute(
            &o,
            env(
                e,
                "unknown",
                1,
                Command::SetDependencies {
                    task_id: c,
                    dependencies: vec![Uuid::new_v4()]
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::NotFound
    );
    admit(&b, &o, e, a, 2).await;
    assert_eq!(
        b.execute(
            &o,
            env(
                e,
                "change",
                3,
                Command::SetDependencies {
                    task_id: a,
                    dependencies: vec![]
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    let (w, _) = grant(&b, &o, e, Role::Worker, Some(m), Some(a), Some(1)).await;
    assert_eq!(
        b.execute(
            &o,
            env(
                e,
                "blocked",
                3,
                Command::AssignAttempt {
                    task_id: a,
                    worker_id: w
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::DependencyUnmet
    );
    for path in [
        "../escape",
        "/absolute",
        "src/../outside",
        "src\\outside",
        ".",
    ] {
        let spec = TaskSpec {
            dependencies: Vec::new(),
            objective: "fixture".into(),
            input: input(),
            acceptance: vec!["yes".into()],
            deliverables: vec!["artifact".into()],
            write_scope: vec![path.into()],
        };
        assert_eq!(
            b.execute(
                &o,
                env(
                    e,
                    &Uuid::new_v4().to_string(),
                    0,
                    Command::CreateTask {
                        id: Uuid::new_v4(),
                        mission_id: m,
                        spec
                    }
                )
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }
    let mut p = policy();
    p.max_attempts = 0;
    assert_eq!(
        b.execute(
            &o,
            env(
                e,
                "unbounded",
                0,
                Command::CreateMission {
                    id: Uuid::new_v4(),
                    objective: "invalid".into(),
                    input: input(),
                    policy: p
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    assert!(
        client
            .execute(
                "UPDATE task_specs SET objective='changed' WHERE task_id=$1",
                &[&a]
            )
            .await
            .is_err()
    );
}
#[tokio::test]
async fn distinct_assignments_capacity_and_cumulative_budget() {
    let (dsn, b, o, e) = setup().await;
    let mut p = policy();
    p.capacity = 1;
    p.max_attempts = 2;
    let m = mission(&b, &o, e, p).await;
    let (a, w, _) = ready(&b, &o, e, m).await;
    let (c, v, _) = ready(&b, &o, e, m).await;
    let other = Broker::connect(&dsn, e).await.unwrap();
    let (first, second) = tokio::join!(
        b.execute(
            &o,
            env(
                e,
                "first",
                2,
                Command::AssignAttempt {
                    task_id: a,
                    worker_id: w
                }
            )
        ),
        other.execute(
            &o,
            env(
                e,
                "second",
                2,
                Command::AssignAttempt {
                    task_id: a,
                    worker_id: w
                }
            )
        )
    );
    assert!(first.is_ok() ^ second.is_ok());
    assert_eq!(
        first.err().or(second.err()).unwrap().code,
        ErrorCode::StaleRevision
    );
    assert_eq!(
        b.execute(
            &o,
            env(
                e,
                "capacity",
                2,
                Command::AssignAttempt {
                    task_id: c,
                    worker_id: v
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::CapacityExhausted
    );
    let mut p = policy();
    p.max_attempts = 1;
    p.capacity = 2;
    let m = mission(&b, &o, e, p).await;
    let (a, w, _) = ready(&b, &o, e, m).await;
    let (c, v, _) = ready(&b, &o, e, m).await;
    b.execute(
        &o,
        env(
            e,
            "consume",
            2,
            Command::AssignAttempt {
                task_id: a,
                worker_id: w,
            },
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        b.execute(
            &o,
            env(
                e,
                "budget",
                2,
                Command::AssignAttempt {
                    task_id: c,
                    worker_id: v
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::BudgetExhausted
    );
    let s = other.snapshot(&o, SnapshotQuery::Mission(m)).await.unwrap();
    assert_eq!(s.data["admitted_attempts"], 1);
    assert_eq!(s.data["admitted_episodes"], 1);
}
#[tokio::test]
async fn rollback_at_every_admission_write_stage() {
    let (dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let (t, w, _) = ready(&b, &o, e, m).await;
    sql(&dsn,"CREATE FUNCTION fixture_failure() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected fixture write failure'; END $$").await;
    for table in [
        "missions",
        "attempts",
        "reservations",
        "operations",
        "tasks",
        "events",
        "outbox",
        "commands",
    ] {
        sql(&dsn,&format!("CREATE TRIGGER fail_write BEFORE INSERT OR UPDATE ON {table} FOR EACH ROW EXECUTE FUNCTION fixture_failure()" )).await;
        let key = format!("rollback-{table}");
        let error = b
            .execute(
                &o,
                env(
                    e,
                    &key,
                    2,
                    Command::AssignAttempt {
                        task_id: t,
                        worker_id: w,
                    },
                ),
            )
            .await
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::TemporarilyUnavailable, "{table}");
        sql(&dsn, &format!("DROP TRIGGER fail_write ON {table}")).await;
        let s = b.snapshot(&o, SnapshotQuery::Task(t)).await.unwrap();
        assert_eq!(s.revision, 2);
        assert_eq!(s.data["generation"], 0);
        assert_eq!(s.data["status"], "ready");
        for field in [
            "attempt_count",
            "reservation_count",
            "operation_count",
            "outbox_count",
        ] {
            assert_eq!(s.data[field], 0, "{table}:{field}");
        }
        let s = b.snapshot(&o, SnapshotQuery::Mission(m)).await.unwrap();
        assert_eq!(s.data["used_capacity"], 0);
        assert_eq!(s.data["admitted_attempts"], 0);
        assert_eq!(s.data["admitted_episodes"], 0);
    }
    assert!(
        b.execute(
            &o,
            env(
                e,
                "recovered",
                2,
                Command::AssignAttempt {
                    task_id: t,
                    worker_id: w
                }
            )
        )
        .await
        .is_ok()
    );
}
#[tokio::test]
async fn http_auth_forged_fields_and_reconnect() {
    let (dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, http::router(b)).await.unwrap();
    });
    let client = reqwest::Client::new();
    let url = format!("http://{address}/v1/snapshots/mission/{m}");
    assert_eq!(client.get(&url).send().await.unwrap().status(), 401);
    let r = client.get(&url).bearer_auth(&o).send().await.unwrap();
    assert_eq!(r.status(), 200);
    let s: Snapshot = r.json().await.unwrap();
    assert_eq!(s.revision, 1);
    let mut value = serde_json::to_value(env(
        e,
        "forged",
        0,
        Command::CreateMission {
            id: Uuid::new_v4(),
            objective: "forged".into(),
            input: input(),
            policy: policy(),
        },
    ))
    .unwrap();
    value["principal_id"] = serde_json::json!(Uuid::new_v4());
    value["role"] = serde_json::json!("owner");
    let r = client
        .post(format!("http://{address}/v1/commands"))
        .bearer_auth(&o)
        .json(&value)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400);
    let error: Error = r.json().await.unwrap();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    let other = Broker::connect(&dsn, e).await.unwrap();
    assert_eq!(
        other
            .snapshot(&o, SnapshotQuery::Mission(m))
            .await
            .unwrap()
            .revision,
        1
    );
    server.abort();
}
#[tokio::test]
async fn a_cached_command_never_restores_wider_authority() {
    let (dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let (t, w, _) = ready(&b, &o, e, m).await;
    let principal = Uuid::new_v4();
    let owner_token = Uuid::new_v4().to_string();
    let limited = Uuid::new_v4().to_string();
    b.execute(
        &o,
        env(
            e,
            "wide",
            0,
            Command::GrantCredential {
                credential_id: Uuid::new_v4(),
                principal_id: principal,
                role: Role::Owner,
                mission_id: None,
                task_id: None,
                generation: None,
                token_hash: token_hash(&owner_token),
                expires_in_seconds: 60,
            },
        ),
    )
    .await
    .unwrap();
    let cmd = env(
        e,
        "original",
        2,
        Command::AssignAttempt {
            task_id: t,
            worker_id: w,
        },
    );
    b.execute(&owner_token, cmd.clone()).await.unwrap();
    let foreign = mission(&b, &o, e, policy()).await;
    b.execute(
        &o,
        env(
            e,
            "narrow",
            0,
            Command::GrantCredential {
                credential_id: Uuid::new_v4(),
                principal_id: principal,
                role: Role::Lead,
                mission_id: Some(foreign),
                task_id: None,
                generation: None,
                token_hash: token_hash(&limited),
                expires_in_seconds: 60,
            },
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        b.execute(&limited, cmd).await.unwrap_err().code,
        ErrorCode::Unauthorized
    );
    let new = Uuid::new_v4();
    let new_owner = b.rotate_epoch(&o, new).await.unwrap();
    let nb = Broker::connect(&dsn, new).await.unwrap();
    assert_eq!(
        nb.rotate_epoch(&new_owner, e).await.unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}
#[tokio::test]
async fn rejected_command_is_durable_and_cannot_be_reused() {
    let (dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let (t, w, _) = ready(&b, &o, e, m).await;
    let mut cmd = env(
        e,
        "rejected",
        1,
        Command::AssignAttempt {
            task_id: t,
            worker_id: w,
        },
    );
    assert_eq!(
        b.execute(&o, cmd.clone()).await.unwrap_err().code,
        ErrorCode::StaleRevision
    );
    // Changing current precondition cannot turn an already rejected command into a new admission.
    cmd.expected_revision = 2;
    let other = Broker::connect(&dsn, e).await.unwrap();
    assert_eq!(
        other.execute(&o, cmd.clone()).await.unwrap_err().code,
        ErrorCode::StaleRevision
    );
    let (c, v, _) = ready(&b, &o, e, m).await;
    cmd.command = Command::AssignAttempt {
        task_id: c,
        worker_id: v,
    };
    assert_eq!(
        b.execute(&o, cmd).await.unwrap_err().code,
        ErrorCode::IdempotencyConflict
    );
}
#[tokio::test]
async fn token_expires_while_waiting_for_the_transaction_lock() {
    let (dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let (t, w, _) = ready(&b, &o, e, m).await;
    let token = Uuid::new_v4().to_string();
    b.execute(
        &o,
        env(
            e,
            "short-token",
            0,
            Command::GrantCredential {
                credential_id: Uuid::new_v4(),
                principal_id: Uuid::new_v4(),
                role: Role::Lead,
                mission_id: Some(m),
                task_id: None,
                generation: None,
                token_hash: token_hash(&token),
                expires_in_seconds: 1,
            },
        ),
    )
    .await
    .unwrap();
    let (mut c, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    let tx = c.transaction().await.unwrap();
    tx.batch_execute("SELECT pg_advisory_xact_lock(784092110)")
        .await
        .unwrap();
    let pending = tokio::spawn(async move {
        b.execute(
            &token,
            env(
                e,
                "expired-wait",
                2,
                Command::AssignAttempt {
                    task_id: t,
                    worker_id: w,
                },
            ),
        )
        .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    tx.rollback().await.unwrap();
    assert_eq!(
        pending.await.unwrap().unwrap_err().code,
        ErrorCode::Unauthorized
    );
}
#[tokio::test]
async fn schema_and_transport_fail_closed() {
    assert_eq!(
        Broker::connect(
            "host=192.0.2.1 user=example password=private connect_timeout=1",
            Uuid::new_v4()
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    let (dsn, _b, _o, e) = setup().await;
    sql(&dsn,"ALTER TABLE schema_version DROP CONSTRAINT schema_version_version_check; UPDATE schema_version SET version=2").await;
    assert_eq!(
        Broker::connect(&dsn, e).await.unwrap_err().code,
        ErrorCode::IncompatibleSchema
    );
    assert_eq!(
        migrate(&dsn).await.unwrap_err().code,
        ErrorCode::IncompatibleSchema
    );
}
#[tokio::test]
async fn command_snapshots_respect_task_scopes() {
    let (_dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let (a, w, worker) = ready(&b, &o, e, m).await;
    let c = task(&b, &o, e, m).await;
    let principal = Uuid::new_v4();
    let wide = Uuid::new_v4().to_string();
    let narrow = Uuid::new_v4().to_string();
    for (key, task, token) in [
        ("mission-lead", None, &wide),
        ("task-lead", Some(c), &narrow),
    ] {
        b.execute(
            &o,
            env(
                e,
                key,
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: principal,
                    role: Role::Lead,
                    mission_id: Some(m),
                    task_id: task,
                    generation: None,
                    token_hash: token_hash(token),
                    expires_in_seconds: 60,
                },
            ),
        )
        .await
        .unwrap();
    }
    b.execute(
        &wide,
        env(
            e,
            "assign",
            2,
            Command::AssignAttempt {
                task_id: a,
                worker_id: w,
            },
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        b.snapshot(
            &narrow,
            SnapshotQuery::Command(principal, "attempt.assign".into(), "assign".into())
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::Unauthorized
    );
    assert!(b.snapshot(&worker, SnapshotQuery::Task(a)).await.is_ok());
    assert_eq!(
        b.snapshot(&worker, SnapshotQuery::Task(c))
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
    assert_eq!(
        b.snapshot(&worker, SnapshotQuery::Mission(m))
            .await
            .unwrap_err()
            .code,
        ErrorCode::Unauthorized
    );
}
#[tokio::test]
async fn episode_budget_and_competing_capacity_are_independent_limits() {
    let (dsn, b, o, e) = setup().await;
    let mut p = policy();
    p.max_episodes = 1;
    let m = mission(&b, &o, e, p).await;
    let (a, w, _) = ready(&b, &o, e, m).await;
    let (c, v, _) = ready(&b, &o, e, m).await;
    b.execute(
        &o,
        env(
            e,
            "episode-one",
            2,
            Command::AssignAttempt {
                task_id: a,
                worker_id: w,
            },
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        b.execute(
            &o,
            env(
                e,
                "episode-two",
                2,
                Command::AssignAttempt {
                    task_id: c,
                    worker_id: v
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::BudgetExhausted
    );
    let mut p = policy();
    p.capacity = 1;
    let m = mission(&b, &o, e, p).await;
    let (a, w, _) = ready(&b, &o, e, m).await;
    let (c, v, _) = ready(&b, &o, e, m).await;
    let other = Broker::connect(&dsn, e).await.unwrap();
    let (x, y) = tokio::join!(
        b.execute(
            &o,
            env(
                e,
                "capacity-race-one",
                2,
                Command::AssignAttempt {
                    task_id: a,
                    worker_id: w
                }
            )
        ),
        other.execute(
            &o,
            env(
                e,
                "capacity-race-two",
                2,
                Command::AssignAttempt {
                    task_id: c,
                    worker_id: v
                }
            )
        )
    );
    assert!(x.is_ok() ^ y.is_ok());
    assert_eq!(
        x.err().or(y.err()).unwrap().code,
        ErrorCode::CapacityExhausted
    );
    assert_eq!(
        b.snapshot(&o, SnapshotQuery::Mission(m))
            .await
            .unwrap()
            .data["used_capacity"],
        1
    );
    for field in ["commit", "path", "digest"] {
        let mut i = input();
        match field {
            "commit" => i.commit = "unresolved-main".into(),
            "path" => i.path = "../SPEC.md".into(),
            _ => i.digest = "malformed".into(),
        };
        assert_eq!(
            b.execute(
                &o,
                env(
                    e,
                    &Uuid::new_v4().to_string(),
                    0,
                    Command::CreateMission {
                        id: Uuid::new_v4(),
                        objective: "invalid reference".into(),
                        input: i,
                        policy: policy()
                    }
                )
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }
}
#[tokio::test]
async fn missing_resource_rejection_stays_rejected_after_resource_appears() {
    let (dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let t = Uuid::new_v4();
    let worker = Uuid::new_v4();
    let cmd = env(
        e,
        "missing-assignment",
        2,
        Command::AssignAttempt {
            task_id: t,
            worker_id: worker,
        },
    );
    assert_eq!(
        b.execute(&o, cmd.clone()).await.unwrap_err().code,
        ErrorCode::NotFound
    );
    b.execute(
        &o,
        env(
            e,
            "create-previously-missing",
            0,
            Command::CreateTask {
                id: t,
                mission_id: m,
                spec: TaskSpec {
                    dependencies: Vec::new(),
                    objective: "appeared".into(),
                    input: input(),
                    acceptance: vec!["verified".into()],
                    deliverables: vec!["artifact".into()],
                    write_scope: vec!["src".into()],
                },
            },
        ),
    )
    .await
    .unwrap();
    admit(&b, &o, e, t, 1).await;
    b.execute(
        &o,
        env(
            e,
            "grant-appeared",
            0,
            Command::GrantCredential {
                credential_id: Uuid::new_v4(),
                principal_id: worker,
                role: Role::Worker,
                mission_id: Some(m),
                task_id: Some(t),
                generation: Some(1),
                token_hash: token_hash(&Uuid::new_v4().to_string()),
                expires_in_seconds: 60,
            },
        ),
    )
    .await
    .unwrap();
    let nb = Broker::connect(&dsn, e).await.unwrap();
    assert_eq!(
        nb.execute(&o, cmd).await.unwrap_err().code,
        ErrorCode::NotFound
    );
    assert_eq!(
        nb.snapshot(&o, SnapshotQuery::Task(t)).await.unwrap().data["attempt_count"],
        0
    );
}
#[tokio::test]
async fn rejected_snapshot_retains_its_original_task_scope() {
    let (_dsn, b, o, e) = setup().await;
    let mut p = policy();
    p.max_attempts = 1;
    let m = mission(&b, &o, e, p).await;
    let (a, w, _) = ready(&b, &o, e, m).await;
    let (c, v, _) = ready(&b, &o, e, m).await;
    let narrow_task = task(&b, &o, e, m).await;
    let principal = Uuid::new_v4();
    let broad = Uuid::new_v4().to_string();
    let narrow = Uuid::new_v4().to_string();
    for (key, task, token) in [
        ("broad-rejection-lead", None, &broad),
        ("narrow-rejection-lead", Some(narrow_task), &narrow),
    ] {
        b.execute(
            &o,
            env(
                e,
                key,
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: principal,
                    role: Role::Lead,
                    mission_id: Some(m),
                    task_id: task,
                    generation: None,
                    token_hash: token_hash(token),
                    expires_in_seconds: 60,
                },
            ),
        )
        .await
        .unwrap();
    }
    b.execute(
        &o,
        env(
            e,
            "consume-reject-budget",
            2,
            Command::AssignAttempt {
                task_id: a,
                worker_id: w,
            },
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        b.execute(
            &broad,
            env(
                e,
                "scoped-rejection",
                2,
                Command::AssignAttempt {
                    task_id: c,
                    worker_id: v
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::BudgetExhausted
    );
    assert_eq!(
        b.execute(
            &narrow,
            env(
                e,
                "scoped-rejection",
                2,
                Command::AssignAttempt {
                    task_id: c,
                    worker_id: v
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::Unauthorized
    );
    assert_eq!(
        b.snapshot(
            &narrow,
            SnapshotQuery::Command(
                principal,
                "attempt.assign".into(),
                "scoped-rejection".into()
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::Unauthorized
    );
}
#[tokio::test]
async fn dependencies_change_spec_digest_and_retain_immutable_history() {
    let (_dsn, b, o, e) = setup().await;
    let m = mission(&b, &o, e, policy()).await;
    let a = task(&b, &o, e, m).await;
    let previous = task(&b, &o, e, m).await;
    let current = task(&b, &o, e, m).await;
    let initial = b.snapshot(&o, SnapshotQuery::Task(a)).await.unwrap();
    b.execute(
        &o,
        env(
            e,
            "first-dependency-revision",
            1,
            Command::SetDependencies {
                task_id: a,
                dependencies: vec![previous],
            },
        ),
    )
    .await
    .unwrap();
    let first = b.snapshot(&o, SnapshotQuery::Task(a)).await.unwrap();
    assert_ne!(initial.data["spec_digest"], first.data["spec_digest"]);
    b.execute(
        &o,
        env(
            e,
            "second-dependency-revision",
            2,
            Command::SetDependencies {
                task_id: a,
                dependencies: vec![current],
            },
        ),
    )
    .await
    .unwrap();
    let second = b.snapshot(&o, SnapshotQuery::Task(a)).await.unwrap();
    assert_ne!(first.data["spec_digest"], second.data["spec_digest"]);
    let history = second.data["spec_history"]
        .as_array()
        .expect("immutable version history exposed");
    assert_eq!(history.len(), 3);
    assert!(
        history
            .iter()
            .any(|s| s["dependencies"] == serde_json::json!([previous]))
    );
    assert!(
        history
            .iter()
            .any(|s| s["dependencies"] == serde_json::json!([current]))
    );
    let stale = first.data["spec_digest"].as_str().unwrap().to_string();
    assert_eq!(
        b.execute(
            &o,
            env(
                e,
                "approve-stale-dependency-spec",
                3,
                Command::AdmitTask {
                    task_id: a,
                    spec_digest: stale
                }
            )
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    admit(&b, &o, e, a, 3).await;
    let state = b.snapshot(&o, SnapshotQuery::Task(a)).await.unwrap();
    assert_eq!(state.data["approved_digest"], second.data["spec_digest"]);
    let events = state.data["events"].as_array().unwrap();
    assert!(
        events
            .iter()
            .any(|event| event["payload"]["command"]["dependencies"]
                == serde_json::json!([previous]))
    );
    assert!(
        events
            .iter()
            .any(|event| event["payload"]["spec_digest"] == first.data["spec_digest"])
    );
}
#[tokio::test]
async fn event_sequences_have_typed_aggregate_namespaces() {
    let (_dsn, b, o, e) = setup().await;
    let id = mission(&b, &o, e, policy()).await;
    b.execute(
        &o,
        env(
            e,
            "shared-uuid-task",
            0,
            Command::CreateTask {
                id,
                mission_id: id,
                spec: TaskSpec {
                    dependencies: Vec::new(),
                    objective: "same UUID different entity".into(),
                    input: input(),
                    acceptance: vec!["verified".into()],
                    deliverables: vec!["artifact".into()],
                    write_scope: vec!["src".into()],
                },
            },
        ),
    )
    .await
    .unwrap();
    let s = b.snapshot(&o, SnapshotQuery::Task(id)).await.unwrap();
    assert_eq!(s.revision, 1);
    let events = s.data["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["aggregate_kind"], "task");
    assert_eq!(
        b.snapshot(&o, SnapshotQuery::Mission(id))
            .await
            .unwrap()
            .revision,
        1
    );
}
