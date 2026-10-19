use std::path::{Path, PathBuf};
use team_control_plane::{gateway::*, *};
use uuid::Uuid;

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
struct OwnedDirectory(PathBuf);
impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("owned runtime cleanup");
    }
}
struct PhysicalRuntime {
    root: PathBuf,
    identity: RuntimeIdentity,
    mode: &'static str,
}
impl PhysicalRuntime {
    fn invoke(&self, mode: &str, request: serde_json::Value) -> serde_json::Value {
        let output = std::process::Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/mailbox_runtime.py"
            ))
            .arg(&self.root)
            .arg(mode)
            .arg(request.to_string())
            .output()
            .expect("owned physical runtime process");
        assert!(
            output.status.success(),
            "unexpected physical subprocess failure: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if mode == "steer_drop_reply" && output.stdout.is_empty() {
            return serde_json::Value::Null;
        }
        serde_json::from_slice(&output.stdout).expect("physical runtime explicit reply")
    }
    fn complete(&self) {
        let output = std::process::Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/mailbox_runtime.py"
            ))
            .arg(&self.root)
            .arg("complete")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "unexpected runtime completion failure: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
    }
    fn count(&self, extension: &str) -> usize {
        std::fs::read_dir(&self.root)
            .unwrap()
            .map(|e| e.unwrap())
            .filter(|e| e.path().extension().is_some_and(|x| x == extension))
            .count()
    }
}
impl RuntimeAdapter for PhysicalRuntime {
    fn identity(&self) -> RuntimeIdentity {
        self.identity.clone()
    }
    fn inspect(&self) -> CapabilityProfile {
        CapabilityProfile {
            start: true,
            steer: true,
            observe: true,
            authoritative_effect_inventory: true,
            ..Default::default()
        }
    }
    fn execute(&mut self, intent: &Intent, _thread: Option<&str>) -> RuntimeReply {
        let mode = if intent.kind == OperationKind::Start {
            "start"
        } else {
            "unconditional"
        };
        let reply = self.invoke(mode, serde_json::json!({"intent":intent}));
        RuntimeReply::Confirmed {
            thread_id: Some(reply["thread_id"].as_str().unwrap().into()),
            effect_id: reply["effect_id"].as_str().unwrap().into(),
        }
    }
    fn confirmed_start_turn(&self, intent: &Intent, reply: &RuntimeReply) -> Option<String> {
        // This pinned fixture sets its actual durable current turn to the Start
        // effect ID in the same file-lock transaction; generic adapters do not.
        if intent.kind == OperationKind::Start
            && self.mode != "no_turn_identity"
            && let RuntimeReply::Confirmed { effect_id, .. } = reply
        {
            return Some(effect_id.clone());
        }
        None
    }
    fn supports_expected_turn_steering(&self) -> bool {
        self.mode != "unsupported"
    }
    fn execute_expected_turn(
        &mut self,
        intent: &Intent,
        _thread: Option<&str>,
        expected_turn: &str,
    ) -> ExpectedTurnReply {
        let reply = self.invoke(
            self.mode,
            serde_json::json!({"intent":intent,"expected_turn":expected_turn}),
        );
        if reply.is_null() {
            return ExpectedTurnReply::Reply(RuntimeReply::Unknown(
                "deliberately lost conditional steering reply".into(),
            ));
        }
        if reply["known_absence"].is_string() {
            return ExpectedTurnReply::NotCurrent {
                observed_turn: reply["actual_turn"].as_str().map(str::to_owned),
            };
        }
        ExpectedTurnReply::Reply(RuntimeReply::Confirmed {
            thread_id: Some(reply["thread_id"].as_str().unwrap().into()),
            effect_id: reply["effect_id"].as_str().unwrap().into(),
        })
    }
    fn observe(&mut self, _: &Intent, _: Option<&str>) -> Observation {
        Observation::Inconclusive("fixture does not infer absence from silence".into())
    }
}
async fn broker_fixture(
    deadline: i64,
) -> (
    Broker,
    String,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    String,
) {
    let base = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").expect("real PostgreSQL required");
    let (admin, connection) = tokio_postgres::connect(&base, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let schema = format!("mailbox_delivery_{}", Uuid::new_v4().simple());
    admin
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{base} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let mission = Uuid::new_v4();
    let task = Uuid::new_v4();
    let worker = Uuid::new_v4();
    let input = GitInput {
        repository: "owned-fixture".into(),
        commit: "a".repeat(40),
        path: "SPEC.md".into(),
        digest: "b".repeat(64),
    };
    broker
        .execute(
            &owner,
            env(
                epoch,
                "mission",
                0,
                Command::CreateMission {
                    id: mission,
                    objective: "bounded conditional delivery".into(),
                    input: input.clone(),
                    policy: Policy {
                        capacity: 2,
                        max_attempts: 8,
                        max_episodes: 8,
                        max_repairs: 3,
                        episode_deadline_seconds: deadline,
                    },
                },
            ),
        )
        .await
        .unwrap();
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
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "conditional input".into(),
                        input,
                        acceptance: vec!["no stale turn effect".into()],
                        deliverables: vec!["result".into()],
                        write_scope: vec!["owned".into()],
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
    broker
        .execute(
            &owner,
            env(
                epoch,
                "worker",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(mission),
                    task_id: Some(task),
                    generation: Some(1),
                    token_hash: token_hash("delivery-worker-fixture"),
                    expires_in_seconds: 30,
                },
            ),
        )
        .await
        .unwrap();
    let assigned = broker
        .execute(
            &owner,
            env(
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
    let principal = admin
        .query_one(
            &format!("SELECT principal_id FROM {schema}.credentials WHERE token_hash=$1"),
            &[&token_hash(&owner)],
        )
        .await
        .unwrap()
        .get(0);
    (
        broker,
        owner,
        epoch,
        mission,
        task,
        worker,
        assigned.attempt_id.unwrap(),
        assigned.operation_id.unwrap(),
        principal,
        dsn,
    )
}
fn runtime_identity(root: &Path) -> RuntimeIdentity {
    RuntimeIdentity {
        runtime_kind: "conditional-subprocess-fault".into(),
        runtime_version: "v1".into(),
        deployment: "owned-fixture".into(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "local-subprocess".into(),
        environment: "disposable".into(),
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delivery_uses_durable_exact_episode_and_physical_conditional_turn() {
    for scenario in [
        "completed",
        "race",
        "drop_reply",
        "unsupported",
        "delivered",
        "queued_close",
        "approval",
        "decision",
        "decision_missing",
        "expired",
        "uncaptured",
        "no_turn_identity",
        "epoch",
        "stale_generation",
    ] {
        delivery_case(scenario).await;
    }
}
async fn delivery_case(scenario: &'static str) {
    let (
        broker,
        owner,
        epoch,
        mission,
        task,
        worker,
        attempt,
        admission,
        owner_principal,
        fixture_dsn,
    ) = broker_fixture(if scenario == "expired" { 1 } else { 30 }).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = http::router(broker.clone());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = reqwest::Client::new();
    let url = format!("http://{addr}/v1/mailbox/commands");
    let root = std::env::temp_dir().join(format!("mailbox-physical-fixture-{}", Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let _owned = OwnedDirectory(root.clone());
    let mut runtime = PhysicalRuntime {
        root: root.clone(),
        identity: runtime_identity(&root),
        mode: match scenario {
            "race" => "steer_gated",
            "drop_reply" => "steer_drop_reply",
            "unsupported" => "unsupported",
            "no_turn_identity" => "no_turn_identity",
            _ => "steer",
        },
    };
    let gateway = Gateway::new(broker.clone());
    let binding = Uuid::new_v4();
    gateway
        .bind(
            &owner,
            binding,
            admission,
            &runtime.identity,
            Some("owned-thread"),
            1,
        )
        .await
        .unwrap();
    let initial_packet = Uuid::new_v4();
    let initial_capture = serde_json::json!({"schema_version":1,"key":"initial-captured-input","recovery_epoch":epoch,"command":{"kind":"capture_input","mission":mission,"id":initial_packet,"attempt":attempt,"purpose":"implementation","limit":8}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("delivery-worker-fixture")
            .json(&initial_capture)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let initial_input: serde_json::Value = client
        .get(format!(
            "http://{addr}/v1/mailbox/records/input/{initial_packet}"
        ))
        .bearer_auth("delivery-worker-fixture")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let initial_digest = initial_input["digest"].as_str().unwrap().to_string();
    let start = Uuid::new_v4();
    gateway.prepare(&owner,Intent{id:start,admission_id:admission,binding_id:binding,generation:1,kind:OperationKind::Start,arguments:serde_json::json!({"input_digest":initial_digest,"input_packet":initial_packet,"input":initial_input["packet"]})}).await.unwrap();
    let started = gateway.dispatch(&owner, start, &mut runtime).await.unwrap();
    assert_eq!(started.state, OperationState::Confirmed);
    assert_eq!(runtime.count("effect"), 1);
    let turn = started
        .evidence
        .iter()
        .find_map(|e| {
            if let Evidence::ReplyConfirmed { effect_id, .. } = e {
                Some(effect_id.clone())
            } else {
                None
            }
        })
        .unwrap();
    let bridge = temporal::Bridge::new(broker.clone());
    let _workflow = bridge
        .register(&owner, temporal::AggregateKind::Task, task, 0)
        .await
        .unwrap();
    let episode = Uuid::new_v4();
    let mut admit = serde_json::json!({"schema_version":1,"key":"implementation-episode","recovery_epoch":epoch,"command":{"kind":"admit_episode","mission":mission,"id":episode,"attempt":attempt,"input_digest":initial_digest,"input_packet":initial_packet,"purpose":"implementation"}});
    if scenario == "uncaptured" {
        admit["command"]
            .as_object_mut()
            .unwrap()
            .remove("input_packet");
    }
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("delivery-worker-fixture")
            .json(&admit)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    broker
        .execute(
            &owner,
            env(
                epoch,
                "narrow-verifier-tool",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Verifier,
                    mission_id: Some(mission),
                    task_id: Some(task),
                    generation: Some(1),
                    token_hash: token_hash("narrow-delivery-verifier"),
                    expires_in_seconds: 30,
                },
            ),
        )
        .await
        .unwrap();
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("narrow-delivery-verifier")
            .json(&admit)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "current narrow role is checked before replay of the same principal's Worker admission journal"
    );
    let message = Uuid::new_v4();
    let send = serde_json::json!({"schema_version":1,"key":"input-message","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":message,"recipients":[worker],"message_kind":"notice","text":"Apply the exact approved constraint","reply_to":null,"attention":"normal","artifacts":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&send)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let mut competing_episode = admit.clone();
    competing_episode["key"] = serde_json::json!("competing-live-episode");
    competing_episode["command"]["id"] = serde_json::json!(Uuid::new_v4());
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("delivery-worker-fixture")
            .json(&competing_episode)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::CONFLICT,
        "new message coalesces behind live episode instead of charging or exposing a SQL conflict"
    );
    let bind = serde_json::json!({"schema_version":1,"key":"confirmed-runtime-turn","recovery_epoch":epoch,"command":{"kind":"bind_runtime_turn","episode":episode,"start_operation":start,"runtime_turn":turn}});
    if scenario != "uncaptured" {
        let mut wrong_turn = bind.clone();
        wrong_turn["key"] = serde_json::json!("unproven-runtime-turn");
        wrong_turn["command"]["runtime_turn"] = serde_json::json!("invented-turn-identity");
        assert_eq!(
            client
                .post(&url)
                .bearer_auth(&owner)
                .json(&wrong_turn)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::CONFLICT,
            "controller cannot replace authoritative confirmed Start turn identity with an arbitrary handle"
        );
    }
    if matches!(scenario, "uncaptured" | "no_turn_identity") {
        if scenario == "no_turn_identity" {
            let record = gateway.read(&owner, start).await.unwrap();
            assert!(record.evidence.iter().any(|entry| matches!(entry, Evidence::Limitation { capability } if capability == "confirmed_start_turn_identity")));
            assert_eq!(record.state, OperationState::Confirmed);
        }
        assert_eq!(
            client
                .post(&url)
                .bearer_auth(&owner)
                .json(&bind)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::CONFLICT,
            "caller-asserted logical input cannot be adopted as a proven broker runtime packet"
        );
        assert_eq!(runtime.count("invocation"), 1);
        server.abort();
        return;
    }
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&bind)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "controller binds a confirmed start to exact current bounded episode"
    );
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("delivery-worker-fixture")
            .json(&bind)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "tool credential is not the runtime controller"
    );
    let delivery = Uuid::new_v4();
    let required_approval = if scenario == "approval" {
        Some(Uuid::new_v4())
    } else {
        None
    };
    let required_decision = if scenario == "decision_missing" {
        Some(Uuid::new_v4())
    } else if scenario == "decision" {
        let lead = Uuid::new_v4();
        let token = "delivery-lead-fixture";
        broker
            .execute(
                &owner,
                env(
                    epoch,
                    "delivery-lead",
                    0,
                    Command::GrantCredential {
                        credential_id: Uuid::new_v4(),
                        principal_id: lead,
                        role: Role::Lead,
                        mission_id: Some(mission),
                        task_id: None,
                        generation: None,
                        token_hash: token_hash(token),
                        expires_in_seconds: 30,
                    },
                ),
            )
            .await
            .unwrap();
        let proposal = Uuid::new_v4();
        let proposal_body = serde_json::json!({"schema_version":1,"key":"delivery-plan","recovery_epoch":epoch,"command":{"kind":"propose","mission":mission,"id":proposal,"proposal":{"kind":"plan","summary":"Use the exact approved delivery constraint","tasks":[]}}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth(token)
                .json(&proposal_body)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        let decide = serde_json::json!({"schema_version":1,"key":"delivery-decision","recovery_epoch":epoch,"command":{"kind":"decide","proposal":proposal,"approved":true,"rationale":"Approve only the recorded plan"}});
        let response = client
            .post(&url)
            .bearer_auth(&owner)
            .json(&decide)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let receipt: serde_json::Value = response.json().await.unwrap();
        Some(serde_json::from_value::<Uuid>(receipt["id"].clone()).unwrap())
    } else {
        None
    };

    let prepare = serde_json::json!({"schema_version":1,"key":"durable-delivery","recovery_epoch":epoch,"command":{"kind":"prepare_delivery","id":delivery,"message":message,"episode":episode,"required_decision":required_decision,"required_approval":required_approval}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&prepare)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "mailbox delivery intent journals exact message/episode/input/expected turn before external I/O"
    );
    let record: serde_json::Value = client
        .get(format!(
            "http://{addr}/v1/mailbox/records/delivery/{delivery}"
        ))
        .bearer_auth(&owner)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let delivery_intent: Intent = serde_json::from_value(record["gateway_intent"].clone()).unwrap();
    assert_eq!(delivery_intent.id, delivery);
    assert_eq!(
        delivery_intent.arguments["expected_turn"],
        serde_json::json!(turn)
    );
    gateway
        .prepare(&owner, delivery_intent.clone())
        .await
        .unwrap();
    if scenario == "stale_generation" {
        // Controlled durable authority fault: accepted10 has no public replacement
        // command. Public HTTP/MCP/gateway reads and claims must still fence it.
        let (db, connection) = tokio_postgres::connect(&fixture_dsn, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move { connection.await.unwrap() });
        db.execute(
            "UPDATE tasks SET generation=generation+1 WHERE id=$1",
            &[&task],
        )
        .await
        .unwrap();
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&admit)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::CONFLICT,
            "cached old attempt admission is fenced by authoritative task generation"
        );
        assert_eq!(
            client
                .get(format!("http://{addr}/v1/mailbox/messages/{message}"))
                .bearer_auth("delivery-worker-fixture")
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::CONFLICT,
            "old attempt cannot read mailbox data"
        );
        assert!(
            gateway
                .dispatch(&owner, delivery, &mut runtime)
                .await
                .is_err()
        );
        assert_eq!(
            gateway.read(&owner, delivery).await.unwrap_err().code,
            ErrorCode::StaleGeneration
        );
        assert_eq!(runtime.count("invocation"), 1);
        assert_eq!(runtime.count("effect"), 1);
        server.abort();
        return;
    }
    if scenario == "expired" {
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        assert!(
            gateway
                .dispatch(&owner, delivery, &mut runtime)
                .await
                .is_err(),
            "expired bounded episode cannot claim runtime delivery"
        );
        assert_eq!(runtime.count("invocation"), 1);
        let expired_yield = serde_json::json!({"schema_version":1,"key":"expired-episode-yield","recovery_epoch":epoch,"command":{"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"Expired authority cannot mutate live episode","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&expired_yield)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::CONFLICT,
            "episode deadline applies to scoped agent checkpoint commands"
        );
        server.abort();
        return;
    }
    if scenario == "decision_missing" {
        assert!(
            gateway
                .dispatch(&owner, delivery, &mut runtime)
                .await
                .is_err(),
            "unrecorded canonical decision never authorizes dependent runtime I/O"
        );
        assert_eq!(runtime.count("invocation"), 1);
        assert_eq!(runtime.count("effect"), 1);
        assert_eq!(
            gateway.read(&owner, delivery).await.unwrap().state,
            OperationState::Prepared
        );
        server.abort();
        return;
    }
    if let Some(request) = required_approval {
        assert!(
            gateway
                .dispatch(&owner, delivery, &mut runtime)
                .await
                .is_err(),
            "missing exact approval cannot claim a physical effect"
        );
        assert_eq!(runtime.count("invocation"), 1);
        let operation = serde_json::json!({"family":"runtime_control","arguments":delivery_intent});
        let request_body = serde_json::json!({"schema_version":1,"key":"delivery-approval-request","recovery_epoch":epoch,"command":{"kind":"request_approval","mission":mission,"id":request,"task":task,"generation":1,"executor":owner_principal,"operation":operation,"expires_in_seconds":30}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth(&owner)
                .json(&request_body)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        assert!(
            gateway
                .dispatch(&owner, delivery, &mut runtime)
                .await
                .is_err(),
            "request alone is not Owner grant"
        );
        assert_eq!(runtime.count("invocation"), 1);
        let grant = serde_json::json!({"schema_version":1,"key":"delivery-approval-grant","recovery_epoch":epoch,"command":{"kind":"grant_approval","request":request,"rationale":"Allow this exact bounded fixture operation"}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth(&owner)
                .json(&grant)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        assert!(
            gateway
                .dispatch(&owner, delivery, &mut runtime)
                .await
                .is_err(),
            "grant must be explicitly consumed by named executor"
        );
        assert_eq!(runtime.count("invocation"), 1);
        let consume = serde_json::json!({"schema_version":1,"key":"delivery-approval-consume","recovery_epoch":epoch,"command":{"kind":"consume_approval","request":request,"operation":operation}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth(&owner)
                .json(&consume)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
    }
    if scenario == "queued_close" {
        use temporal::OperationDriver;
        let shared = std::sync::Arc::new(tokio::sync::Mutex::new(
            Box::new(runtime) as Box<dyn RuntimeAdapter>
        ));
        let held = shared.lock().await;
        let driver = std::sync::Arc::new(temporal::GatewayOperationDriver::with_shared_runtime(
            bridge.clone(),
            gateway.clone(),
            owner.clone(),
            shared.clone(),
        ));
        let queued_driver = driver.clone();
        let reference = _workflow.clone();
        let pending = tokio::spawn(async move { queued_driver.drive(&reference, admission).await });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while driver.queued() != 1 {
            assert!(std::time::Instant::now() < deadline);
            tokio::task::yield_now().await;
        }
        bridge.close(&owner, &_workflow).await.unwrap();
        drop(held);
        assert!(
            pending.await.unwrap().is_err(),
            "closed workflow cannot claim delivery after waiting for runtime mutex"
        );
        assert_eq!(
            gateway.read(&owner, delivery).await.unwrap().state,
            OperationState::Prepared
        );
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
            "queued stale controller produces zero additional physical invocations"
        );
        server.abort();
        return;
    }
    let completed = if scenario == "race" {
        let queued_gateway = gateway.clone();
        let queued_owner = owner.clone();
        // Isolate the synchronous subprocess transport so its intentional gate cannot
        // starve PostgreSQL connection futures on this test's Tokio scheduler.
        let controller = tokio::task::spawn_blocking(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    let result = queued_gateway
                        .dispatch(&queued_owner, delivery, &mut runtime)
                        .await
                        .unwrap();
                    (result, runtime)
                })
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !root.join("entered").exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "actual conditional subprocess did not enter its bounded gate"
            );
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        assert_eq!(
            gateway.read(&owner, delivery).await.unwrap().state,
            OperationState::Dispatching,
            "intent/claim is durable before the runtime effect boundary"
        );
        let completer = PhysicalRuntime {
            root: root.clone(),
            identity: runtime_identity(&root),
            mode: "steer",
        };
        completer.complete();
        std::fs::write(root.join("continue"), b"complete-won-race").unwrap();
        let (result, returned_runtime) = controller.await.unwrap();
        runtime = returned_runtime;
        result
    } else {
        if scenario == "completed" {
            runtime.complete();
        }
        gateway
            .dispatch(&owner, delivery, &mut runtime)
            .await
            .unwrap()
    };
    let expected = match scenario {
        "drop_reply" => OperationState::OutcomeUnknown,
        "delivered" | "approval" | "decision" | "epoch" => OperationState::Confirmed,
        _ => OperationState::FailedKnown,
    };
    assert_eq!(completed.state, expected, "conditional scenario {scenario}");
    assert_eq!(
        runtime.count("effect"),
        if matches!(
            scenario,
            "drop_reply" | "delivered" | "approval" | "decision" | "epoch"
        ) {
            2
        } else {
            1
        },
        "non-idempotent physical effect inventory: {scenario}"
    );
    let invocations = if scenario == "unsupported" { 1 } else { 2 };
    assert_eq!(
        runtime.count("invocation"),
        invocations,
        "conditional call is physical, unsupported capability performs no I/O"
    );
    if scenario == "unsupported" {
        assert!(completed.evidence.iter().any(
            |e| matches!(e,Evidence::Limitation{capability} if capability=="expected_turn_steering")
        ));
    }
    assert_eq!(
        gateway
            .dispatch(&owner, delivery, &mut runtime)
            .await
            .unwrap()
            .state,
        expected
    );
    assert_eq!(
        runtime.count("invocation"),
        invocations,
        "duplicate delivery never repeats the physical operation"
    );
    let message_state: serde_json::Value = client
        .get(format!("http://{addr}/v1/mailbox/messages/{message}"))
        .bearer_auth("delivery-worker-fixture")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        message_state["presentations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        message_state["acknowledgments"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    if matches!(scenario, "delivered" | "drop_reply") {
        let presentation = serde_json::json!({"schema_version":1,"key":"explicit-packet-presentation","recovery_epoch":epoch,"command":{"kind":"present","message":message,"episode":episode,"input_digest":record["input_digest"],"runtime_turn":turn}});
        let response = client
            .post(&url)
            .bearer_auth("delivery-worker-fixture")
            .json(&presentation)
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            if scenario == "delivered" {
                reqwest::StatusCode::OK
            } else {
                reqwest::StatusCode::CONFLICT
            },
            "only confirmed packet permits explicit recipient presentation; unknown transport is not evidence"
        );
        assert_eq!(
            runtime.count("invocation"),
            invocations,
            "presentation performs no runtime I/O"
        );
    }
    if matches!(scenario, "completed" | "race") {
        let packet = Uuid::new_v4();
        let capture = serde_json::json!({"schema_version":1,"key":"pending-input-packet","recovery_epoch":epoch,"command":{"kind":"capture_input","mission":mission,"id":packet,"attempt":attempt,"purpose":"implementation","limit":8}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&capture)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK,
            "pending messages form an immutable bounded canonical inbox packet"
        );
        let captured: serde_json::Value = client
            .get(format!("http://{addr}/v1/mailbox/records/input/{packet}"))
            .bearer_auth("delivery-worker-fixture")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(
            captured["packet"]["messages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["id"] == serde_json::json!(message))
        );
        assert_eq!(captured["packet"]["task"]["generation"], 1);
        let mut continuation = admit.clone();
        continuation["command"]["input_packet"] = serde_json::json!(packet);
        continuation["command"]["input_digest"] = captured["digest"].clone();
        continuation["key"] = serde_json::json!("pending-next-episode");
        continuation["command"]["id"] = serde_json::json!(Uuid::new_v4());
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&continuation)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK,
            "authoritatively ended old turn leaves pending input eligible for a new bounded episode; no physical continuation Start is claimed"
        );
    }
    if scenario == "epoch" {
        let mcp_url = format!("http://{addr}/mcp");
        let initialize = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"epoch-fence-fixture","version":"1"}}});
        let initialized = client
            .post(&mcp_url)
            .bearer_auth("delivery-worker-fixture")
            .header("accept", "application/json, text/event-stream")
            .json(&initialize)
            .send()
            .await
            .unwrap();
        assert_eq!(initialized.status(), reqwest::StatusCode::OK);
        let session = initialized.headers()["mcp-session-id"]
            .to_str()
            .unwrap()
            .to_string();
        let _new_owner = broker.rotate_epoch(&owner, Uuid::new_v4()).await.unwrap();
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&admit)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::UNAUTHORIZED,
            "restored epoch rejects old caller before cached episode receipt"
        );
        assert_eq!(
            client
                .post(&mcp_url)
                .bearer_auth("delivery-worker-fixture")
                .header("accept", "application/json, text/event-stream")
                .header("mcp-session-id", &session)
                .header("mcp-protocol-version", "2025-11-25")
                .json(&serde_json::json!({"jsonrpc":"2.0","id":2,"method":"resources/list"}))
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::UNAUTHORIZED,
            "MCP session cannot retain authority across epoch replacement"
        );
        assert!(
            gateway
                .dispatch(&owner, delivery, &mut runtime)
                .await
                .is_err()
        );
        assert_eq!(
            runtime.count("invocation"),
            invocations,
            "old controller epoch never replays physical I/O"
        );
    }
    if scenario == "delivered" {
        let yield_body = |episode: Uuid, key: &str| serde_json::json!({"schema_version":1,"key":key,"recovery_epoch":epoch,"command":{"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"Observed input processed","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&yield_body(episode, "worker-yield"))
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        let next = Uuid::new_v4();
        let mut next_admit = admit.clone();
        next_admit["key"] = serde_json::json!("worker-new-message-event");
        next_admit["command"]["id"] = serde_json::json!(next);
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&next_admit)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::CONFLICT,
            "old captured packet cannot consume a newer mailbox event"
        );
        let next_packet = Uuid::new_v4();
        let capture = serde_json::json!({"schema_version":1,"key":"worker-current-input","recovery_epoch":epoch,"command":{"kind":"capture_input","mission":mission,"id":next_packet,"attempt":attempt,"purpose":"implementation","limit":8}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&capture)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        let next_input: serde_json::Value = client
            .get(format!(
                "http://{addr}/v1/mailbox/records/input/{next_packet}"
            ))
            .bearer_auth("delivery-worker-fixture")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        next_admit["key"] = serde_json::json!("worker-current-captured-event");
        next_admit["command"]["input_packet"] = serde_json::json!(next_packet);
        next_admit["command"]["input_digest"] = next_input["digest"].clone();
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&next_admit)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&yield_body(next, "worker-yield-current-event"))
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        let replace = serde_json::json!({"schema_version":1,"key":"unrelated-lead-change","recovery_epoch":epoch,"command":{"kind":"replace_lead","mission":mission,"expected_generation":0,"principal":Uuid::new_v4(),"credential_id":Uuid::new_v4(),"token_hash":token_hash("replacement-in-delivery-fixture"),"expires_in_seconds":30}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth(&owner)
                .json(&replace)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        next_admit["key"] = serde_json::json!("worker-unrelated-lead-event");
        next_admit["command"]["id"] = serde_json::json!(Uuid::new_v4());
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("delivery-worker-fixture")
                .json(&next_admit)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::CONFLICT,
            "unrelated Lead session change cannot purchase a Worker polling episode"
        );
    }
    server.abort();
}

#[test]
fn physical_fault_runtime_repeats_are_not_deduplicated_by_operation_id() {
    let root = std::env::temp_dir().join(format!("mailbox-repeat-fixture-{}", Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let _owned = OwnedDirectory(root.clone());
    let mut runtime = PhysicalRuntime {
        root: root.clone(),
        identity: runtime_identity(&root),
        mode: "steer",
    };
    let intent = Intent {
        id: Uuid::new_v4(),
        admission_id: Uuid::new_v4(),
        binding_id: Uuid::new_v4(),
        generation: 1,
        kind: OperationKind::Start,
        arguments: serde_json::json!({}),
    };
    runtime.execute(&intent, Some("owned-thread"));
    runtime.execute(&intent, Some("owned-thread"));
    assert_eq!(runtime.count("invocation"), 2);
    assert_eq!(
        runtime.count("effect"),
        2,
        "two identical operation invocations cause two actual effects"
    );
}

#[tokio::test]
async fn bounded_input_batches_advance_without_empty_polling() {
    let (broker, owner, epoch, mission, task, worker, attempt, _, _, _) = broker_fixture(30).await;
    temporal::Bridge::new(broker.clone())
        .register(&owner, temporal::AggregateKind::Task, task, 0)
        .await
        .unwrap();
    let service = mailbox::MailboxService::new(broker);
    let call = |key: &str, command: serde_json::Value| {
        serde_json::from_value::<mailbox::Envelope>(serde_json::json!({"schema_version":1,"key":key,"recovery_epoch":epoch,"command":command})).unwrap()
    };
    for index in 0..2 {
        service.execute(&owner, call(&format!("batch-send-{index}"), serde_json::json!({"kind":"send","mission":mission,"id":Uuid::new_v4(),"recipients":[worker],"message_kind":"notice","text":format!("batch {index}"),"reply_to":null,"attention":"normal","artifacts":[]}))).await.unwrap();
    }
    for index in 0..3 {
        let input = Uuid::new_v4();
        service.execute("delivery-worker-fixture", call(&format!("batch-capture-{index}"), serde_json::json!({"kind":"capture_input","mission":mission,"id":input,"attempt":attempt,"purpose":"implementation","limit":1}))).await.unwrap();
        let packet = service
            .record("delivery-worker-fixture", mailbox::RecordKind::Input, input)
            .await
            .unwrap();
        let episode = Uuid::new_v4();
        let result = service.execute("delivery-worker-fixture", call(&format!("batch-admit-{index}"), serde_json::json!({"kind":"admit_episode","mission":mission,"id":episode,"attempt":attempt,"input_digest":packet["digest"],"input_packet":input,"purpose":"implementation"}))).await;
        if index == 2 {
            assert_eq!(
                result.unwrap_err().code,
                ErrorCode::DependencyUnmet,
                "an empty packet cannot buy a polling episode after existing batches drain"
            );
            let legacy = service.execute("delivery-worker-fixture", call("batch-empty-legacy", serde_json::json!({"kind":"admit_episode","mission":mission,"id":Uuid::new_v4(),"attempt":attempt,"input_digest":"a".repeat(64),"purpose":"implementation"}))).await;
            assert_eq!(
                legacy.unwrap_err().code,
                ErrorCode::DependencyUnmet,
                "legacy logical input cannot bypass the captured frontier polling fence"
            );
            break;
        }
        result.expect("each previously unprocessed existing bounded batch can advance");
        let message = packet["packet"]["messages"][0]["id"].clone();
        service.execute("delivery-worker-fixture", call(&format!("batch-present-{index}"), serde_json::json!({"kind":"present","message":message,"episode":episode,"input_digest":packet["digest"],"runtime_turn":null}))).await.unwrap();
        service
            .execute(
                "delivery-worker-fixture",
                call(
                    &format!("batch-ack-{index}"),
                    serde_json::json!({"kind":"acknowledge","message":message,"episode":episode}),
                ),
            )
            .await
            .unwrap();
        service.execute("delivery-worker-fixture", call(&format!("batch-yield-{index}"), serde_json::json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"Processed one bounded batch","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}))).await.unwrap();
    }
}
