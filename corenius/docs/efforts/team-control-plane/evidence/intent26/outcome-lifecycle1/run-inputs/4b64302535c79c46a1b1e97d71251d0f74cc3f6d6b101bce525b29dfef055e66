use team_control_plane::artifacts::*;
use team_control_plane::migrate;
use team_control_plane::*;
use uuid::Uuid;
fn envelope(epoch: Uuid, key: &str, rev: i64, command: Command) -> CommandEnvelope {
    CommandEnvelope {
        schema_version: 1,
        key: key.into(),
        expected_revision: rev,
        expected_generation: None,
        recovery_epoch: epoch,
        command,
    }
}
async fn fixture() -> (
    ArtifactService,
    Broker,
    String,
    String,
    String,
    Uuid,
    Uuid,
    Candidate,
    std::path::PathBuf,
) {
    let dsn = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").unwrap();
    // Every test gets a disposable schema on the real fixture server.
    let schema = format!("artifacts_{}", Uuid::new_v4().simple());
    let (client, conn) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        conn.await.unwrap();
    });
    client
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{dsn} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let mission = Uuid::new_v4();
    let task = Uuid::new_v4();
    let worker = Uuid::new_v4();
    let verifier = Uuid::new_v4();
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
                "mission",
                0,
                Command::CreateMission {
                    id: mission,
                    objective: "fixture".into(),
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
                "task",
                0,
                Command::CreateTask {
                    id: task,
                    mission_id: mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "fixture".into(),
                        input,
                        acceptance: vec!["check".into()],
                        deliverables: vec!["result".into()],
                        write_scope: vec!["result".into()],
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
                "admit",
                1,
                Command::AdmitTask {
                    task_id: task,
                    spec_digest: digest.clone(),
                },
            ),
        )
        .await
        .unwrap();
    let root = std::env::temp_dir().join(format!("mailbox-artifact-fixture-{}", Uuid::new_v4()));
    let service = ArtifactService::open(broker.clone(), &root).await.unwrap();
    for (key, principal, role, token, generation) in [
        ("worker", worker, Role::Worker, "worker", Some(1)),
        ("verifier", verifier, Role::Verifier, "verifier", Some(1)),
    ] {
        broker
            .execute(
                &owner,
                envelope(
                    epoch,
                    key,
                    0,
                    Command::GrantCredential {
                        credential_id: Uuid::new_v4(),
                        principal_id: principal,
                        role,
                        mission_id: Some(mission),
                        task_id: Some(task),
                        generation,
                        token_hash: token_hash(token),
                        expires_in_seconds: 30,
                    },
                ),
            )
            .await
            .unwrap();
    }
    broker
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
    let artifact = String::new();
    let candidate = Candidate {
        commit: "c".repeat(40),
        tree: "d".repeat(40),
        base: "a".repeat(40),
        spec_digest: digest,
        spec_revision: 2,
        generation: 1,
        artifact,
        deliverables: vec!["result".into()],
        risks: vec![],
    };
    (
        service,
        broker,
        owner,
        "worker".into(),
        "verifier".into(),
        task,
        verifier,
        candidate,
        root,
    )
}
fn cmd(
    epoch: Uuid,
    task: Uuid,
    key: &str,
    revision: i64,
    command: ArtifactCommand,
) -> ArtifactEnvelope {
    ArtifactEnvelope {
        key: key.into(),
        task,
        expected_revision: revision,
        generation: 1,
        recovery_epoch: epoch,
        command,
    }
}
fn git(repo: &std::path::Path, args: &[&str]) -> String {
    let result = std::process::Command::new("git")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .args(["-c", "core.hooksPath=/dev/null", "-C"])
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().to_string()
}
fn git_bundle(repo: &std::path::Path, commit: &str, base: &str) -> Vec<u8> {
    let bundle = repo.join(format!("candidate-{}.bundle", Uuid::new_v4()));
    // git bundle requires a named tip. This reference exists only in the
    // disposable fixture repository and does not move its source/mainline.
    let reference = format!("refs/fixture-candidates/{}", Uuid::new_v4());
    git(repo, &["update-ref", &reference, commit]);
    assert_eq!(git(repo, &["cat-file", "-t", base]), "commit");
    git(
        repo,
        &["bundle", "create", bundle.to_str().unwrap(), &reference],
    );
    git(repo, &["update-ref", "-d", &reference]);
    let bytes = std::fs::read(&bundle).unwrap();
    std::fs::remove_file(bundle).unwrap();
    bytes
}
fn independent_git_check(
    bytes: &[u8],
    commit: &str,
    base: &str,
    commands: &[Vec<String>],
) -> (String, String, String, Vec<IntegrationCheck>) {
    let repo =
        std::env::temp_dir().join(format!("independent-artifact-verifier-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-b", "verifier"]);
    let bundle = repo.join("candidate.bundle");
    std::fs::write(&bundle, bytes).unwrap();
    git(&repo, &["fetch", bundle.to_str().unwrap(), commit]);
    git(&repo, &["checkout", "--detach", "FETCH_HEAD"]);
    let measured_commit = git(&repo, &["rev-parse", "HEAD"]);
    let measured_tree = git(&repo, &["rev-parse", "HEAD^{tree}"]);
    let measured_base = git(&repo, &["rev-parse", "HEAD^1"]);
    assert_eq!(
        measured_base, base,
        "base is derived from retained integration commit ancestry"
    );
    let checks = commands
        .iter()
        .map(|command| {
            let result = std::process::Command::new(&command[0])
                .args(&command[1..])
                .current_dir(&repo)
                .output()
                .unwrap();
            let mut logs = result.stdout;
            logs.extend_from_slice(b"\n--- stderr ---\n");
            logs.extend_from_slice(&result.stderr);
            IntegrationCheck {
                command: command.clone(),
                exit_code: result.status.code().unwrap_or(-1),
                logs,
            }
        })
        .collect();
    std::fs::remove_dir_all(repo).unwrap();
    (measured_commit, measured_tree, measured_base, checks)
}

#[tokio::test]
async fn actual_git_submission_and_independent_verification_share_http_mcp_authority() {
    let (service, broker, owner, worker, verifier, task, verifier_id, template, root) =
        fixture().await;
    let _owned_root = OwnedFixtureDir(root.clone());
    let epoch: Uuid = serde_json::from_value(
        broker
            .snapshot(&owner, SnapshotQuery::Task(task))
            .await
            .unwrap()
            .data["attempts"][0]["recovery_epoch"]
            .clone(),
    )
    .unwrap();
    let assigned_attempt: Uuid = serde_json::from_value(
        broker
            .snapshot(&owner, SnapshotQuery::Task(task))
            .await
            .unwrap()
            .data["attempts"][0]["id"]
            .clone(),
    )
    .unwrap();
    let premature = serde_json::json!({"schema_version":1,"key":"premature-verifier-turn","recovery_epoch":epoch,"command":{"kind":"admit_episode","mission":broker.snapshot(&owner,SnapshotQuery::Task(task)).await.unwrap().data["mission_id"],"id":Uuid::new_v4(),"attempt":assigned_attempt,"input_digest":"c".repeat(64),"purpose":"verification"}});
    assert!(
        matches!(
            mailbox::MailboxService::new(broker.clone())
                .execute(&verifier, serde_json::from_value(premature).unwrap())
                .await,
            Err(Error {
                code: ErrorCode::DependencyUnmet
            })
        ),
        "verification turn needs an assigned independent candidate event, not merely a live credential and a running task"
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = http::router_with_artifacts(broker.clone(), service.clone());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = reqwest::Client::new();
    let commands_url = format!("http://{addr}/v1/artifacts/commands");
    assert_eq!(
        client
            .post(&commands_url)
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "artifact transport authenticates before processing a submission"
    );
    let repo = root.join("owned-git-fixture");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-b", "fixture"]);
    git(&repo, &["config", "user.name", "Fixture"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    std::fs::write(repo.join("base"), b"bounded base\n").unwrap();
    git(&repo, &["add", "base"]);
    git(&repo, &["commit", "-m", "fixture base"]);
    let base = git(&repo, &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("result"), b"approved result\n").unwrap();
    git(&repo, &["add", "result"]);
    git(&repo, &["commit", "-m", "fixture candidate"]);
    let commit = git(&repo, &["rev-parse", "HEAD"]);
    let tree = git(&repo, &["rev-parse", "HEAD^{tree}"]);
    let commands = vec![vec![
        "sh".into(),
        "-c".into(),
        "test -f result && cat result && cat base".into(),
    ]];
    let configure = cmd(
        epoch,
        task,
        "configure-transport",
        3,
        ArtifactCommand::Configure {
            verifier: verifier_id,
            lead_acceptance: false,
            base: base.clone(),
            suite: "e".repeat(64),
            environment: "f".repeat(64),
            mandatory_commands: commands.clone(),
        },
    );
    let response = client
        .post(&commands_url)
        .bearer_auth(&owner)
        .json(&configure)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let bytes = git_bundle(&repo, &commit, &base);
    let digest = sha256(&bytes);
    let objects_url = format!("http://{addr}/v1/artifacts/objects");
    let upload = serde_json::json!({"task":task,"digest":digest,"bytes":bytes});
    assert_eq!(
        client
            .post(&objects_url)
            .bearer_auth(&worker)
            .json(&upload)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let candidate = Candidate {
        commit: commit.clone(),
        tree: tree.clone(),
        base: base.clone(),
        artifact: digest.clone(),
        ..template
    };
    let submit = cmd(
        epoch,
        task,
        "transport-submit",
        4,
        ArtifactCommand::Submit {
            candidate: candidate.clone(),
        },
    );
    assert_eq!(
        client
            .post(&commands_url)
            .bearer_auth(&verifier)
            .json(&submit)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "Verifier cannot submit the Worker's candidate"
    );
    let submitted: ArtifactResult = client
        .post(&commands_url)
        .bearer_auth(&worker)
        .json(&submit)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(submitted.candidate_id.is_some());
    assert!(!submitted.accepted);
    let verifier_workflow = temporal::Bridge::new(broker.clone())
        .register(&owner, temporal::AggregateKind::Task, task, 0)
        .await
        .unwrap();
    let verifier_episode = Uuid::new_v4();
    let verifier_admission = serde_json::json!({"schema_version":1,"key":"candidate-verifier-turn","recovery_epoch":epoch,"command":{"kind":"admit_episode","mission":broker.snapshot(&owner,SnapshotQuery::Task(task)).await.unwrap().data["mission_id"],"id":verifier_episode,"attempt":assigned_attempt,"input_digest":"c".repeat(64),"purpose":"verification"}});
    let mailbox_url = format!("http://{addr}/v1/mailbox/commands");
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(&verifier)
            .json(&verifier_admission)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "assigned independent verifier receives a bounded episode only after durable candidate submission"
    );
    let verifier_yield = serde_json::json!({"schema_version":1,"key":"verifier-checkpoint","recovery_epoch":epoch,"command":{"kind":"yield_episode","episode":verifier_episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"Candidate snapshot captured; physical verifier adapter remains unconfigured","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}});
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(&verifier)
            .json(&verifier_yield)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    assert!(
        temporal::Bridge::new(broker.clone())
            .progress(&verifier, &verifier_workflow)
            .await
            .unwrap()
            .mailbox_attention
            .is_empty()
    );
    let mut verifier_poll = verifier_admission.clone();
    verifier_poll["key"] = serde_json::json!("verifier-no-new-candidate-event");
    verifier_poll["command"]["id"] = serde_json::json!(Uuid::new_v4());
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(&verifier)
            .json(&verifier_poll)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::CONFLICT,
        "unconfigured verifier turn does not permit polling or broad execution tools"
    );

    let read_url = format!("http://{addr}/v1/artifacts/tasks/{task}/objects/{digest}");
    let retained = client
        .get(&read_url)
        .bearer_auth(&verifier)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .to_vec();
    assert_eq!(retained, bytes);
    let (measured_commit, measured_tree, measured_base, executed) =
        independent_git_check(&retained, &commit, &base, &commands);
    assert_eq!(
        (&measured_commit, &measured_tree, &measured_base),
        (&commit, &tree, &base)
    );
    assert_eq!(executed[0].exit_code, 0);
    let logs_digest = sha256(&executed[0].logs);
    assert_eq!(
        client
            .post(&objects_url)
            .bearer_auth(&verifier)
            .json(&serde_json::json!({"task":task,"digest":logs_digest,"bytes":executed[0].logs}))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let report = Verification {
        candidate: candidate.clone(),
        suite: "e".repeat(64),
        environment: "f".repeat(64),
        checks: vec![Check {
            command: executed[0].command.clone(),
            exit_code: executed[0].exit_code,
            logs: logs_digest.clone(),
        }],
        resolved_decisions: true,
    };
    let mcp_url = format!("http://{addr}/mcp");
    let initialize = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"actual-artifact-fixture","version":"1"}}});
    let init = client
        .post(&mcp_url)
        .bearer_auth(&verifier)
        .header("accept", "application/json, text/event-stream")
        .json(&initialize)
        .send()
        .await
        .unwrap();
    assert_eq!(init.status(), reqwest::StatusCode::OK);
    let session = init.headers()["mcp-session-id"]
        .to_str()
        .unwrap()
        .to_string();
    let mcp = |body: serde_json::Value| {
        client
            .post(&mcp_url)
            .bearer_auth(&verifier)
            .header("accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session)
            .header("mcp-protocol-version", "2025-11-25")
            .json(&body)
    };
    assert_eq!(
        mcp(serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::ACCEPTED
    );
    let denied:serde_json::Value=mcp(serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"artifact_command","arguments":submit}})).send().await.unwrap().json().await.unwrap();
    assert_eq!(denied["result"]["isError"], true);
    assert_eq!(
        denied["result"]["structuredContent"]["code"],
        "unauthorized"
    );
    let verify = cmd(
        epoch,
        task,
        "transport-verify",
        5,
        ArtifactCommand::Verify {
            verification: report,
        },
    );
    let verified:serde_json::Value=mcp(serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"artifact_command","arguments":verify}})).send().await.unwrap().json().await.unwrap();
    assert_eq!(verified["result"]["isError"], false);
    assert!(verified["result"]["structuredContent"]["report_id"].is_string());
    assert_eq!(verified["result"]["structuredContent"]["accepted"], false);
    let task_resource:serde_json::Value=mcp(serde_json::json!({"jsonrpc":"2.0","id":31,"method":"resources/read","params":{"uri":format!("corenius://task/{task}")}})).send().await.unwrap().json().await.unwrap();
    assert!(task_resource["result"]["contents"][0]["text"].as_str().is_some_and(|text|serde_json::from_str::<serde_json::Value>(text).unwrap()["data"]["approved_digest"]==candidate.spec_digest),"MCP exposes caller-authorized canonical approved task input without an Owner proxy");
    let resource:serde_json::Value=mcp(serde_json::json!({"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":format!("corenius://artifact/{task}/{logs_digest}")}})).send().await.unwrap().json().await.unwrap();
    assert_eq!(
        resource["result"]["contents"][0]["mimeType"],
        "application/octet-stream"
    );
    let mission: Uuid = serde_json::from_value(
        broker
            .snapshot(&owner, SnapshotQuery::Task(task))
            .await
            .unwrap()
            .data["mission_id"]
            .clone(),
    )
    .unwrap();
    let mailbox_url = format!("http://{addr}/v1/mailbox/commands");
    let lead = Uuid::new_v4();
    let lead_token = "artifact-wait-lead-fixture";
    let replacement = serde_json::json!({"schema_version":1,"key":"wait-lead","recovery_epoch":epoch,"command":{"kind":"replace_lead","mission":mission,"expected_generation":0,"principal":lead,"credential_id":Uuid::new_v4(),"token_hash":token_hash(lead_token),"expires_in_seconds":30}});
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(&owner)
            .json(&replacement)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let bridge = temporal::Bridge::new(broker.clone());
    let workflow = bridge
        .register(&owner, temporal::AggregateKind::Mission, mission, 0)
        .await
        .unwrap();
    let planning = Uuid::new_v4();
    let checkpoint = Uuid::new_v4();
    let planning_admit = serde_json::json!({"schema_version":1,"key":"dependency-wait-episode","recovery_epoch":epoch,"command":{"kind":"admit_episode","mission":mission,"id":planning,"attempt":null,"input_digest":"c".repeat(64),"purpose":"planning"}});
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(lead_token)
            .json(&planning_admit)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let waiting = serde_json::json!({"schema_version":1,"key":"dependency-checkpoint","recovery_epoch":epoch,"command":{"kind":"yield_episode","episode":planning,"checkpoint":{"id":checkpoint,"summary":"Await actual candidate acceptance","artifacts":[]},"questions":[],"dependencies":[task],"executions":[]}});
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(lead_token)
            .json(&waiting)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    assert!(
        bridge
            .progress(&owner, &workflow)
            .await
            .unwrap()
            .mailbox_attention
            .iter()
            .any(|w| w.reference == task && !w.satisfied)
    );
    let peer_task = Uuid::new_v4();
    let peer_principal = Uuid::new_v4();
    let peer_token = format!("peer-worker-fixture-{}", Uuid::new_v4());
    broker
        .execute(
            &owner,
            envelope(
                epoch,
                "peer-task",
                0,
                Command::CreateTask {
                    id: peer_task,
                    mission_id: mission,
                    spec: TaskSpec {
                        dependencies: vec![],
                        objective: "Independent peer task".into(),
                        input: GitInput {
                            repository: "fixture".into(),
                            commit: "a".repeat(40),
                            path: "SPEC.md".into(),
                            digest: "b".repeat(64),
                        },
                        acceptance: vec!["own result".into()],
                        deliverables: vec!["own result".into()],
                        write_scope: vec!["peer".into()],
                    },
                },
            ),
        )
        .await
        .unwrap();
    let peer_digest = broker
        .snapshot(&owner, SnapshotQuery::Task(peer_task))
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
                "peer-admit",
                1,
                Command::AdmitTask {
                    task_id: peer_task,
                    spec_digest: peer_digest,
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
                "peer-credential",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: peer_principal,
                    role: Role::Worker,
                    mission_id: Some(mission),
                    task_id: Some(peer_task),
                    generation: Some(1),
                    token_hash: token_hash(&peer_token),
                    expires_in_seconds: 30,
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
                "peer-assignment",
                2,
                Command::AssignAttempt {
                    task_id: peer_task,
                    worker_id: peer_principal,
                },
            ),
        )
        .await
        .unwrap();
    let message = Uuid::new_v4();
    let send_artifact = serde_json::json!({"schema_version":1,"key":"share-exact-registered-evidence","recovery_epoch":epoch,"command":{"kind":"send","mission":broker.snapshot(&owner,SnapshotQuery::Task(task)).await.unwrap().data["mission_id"],"id":message,"recipients":[verifier_id,peer_principal],"message_kind":"notice","text":"Inspect this exact immutable candidate","reply_to":null,"attention":"normal","artifacts":[{"task":task,"digest":digest}]}});
    assert_eq!(
        client
            .post(format!("http://{addr}/v1/mailbox/commands"))
            .bearer_auth(&worker)
            .json(&send_artifact)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "artifact messages bind canonical task and exact immutable digest"
    );
    let shared = client
        .get(format!(
            "http://{addr}/v1/mailbox/messages/{message}/artifacts/{digest}"
        ))
        .bearer_auth(&verifier)
        .send()
        .await
        .unwrap();
    assert_eq!(shared.status(), reqwest::StatusCode::OK);
    assert_eq!(shared.bytes().await.unwrap().to_vec(), bytes);
    let peer_message: serde_json::Value = client
        .get(format!("http://{addr}/v1/mailbox/messages/{message}"))
        .bearer_auth(&peer_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        peer_message["attempt_id"].is_string(),
        "authenticated message captures the sender's exact assigned attempt"
    );
    assert_eq!(peer_message["approved_spec_digest"], candidate.spec_digest);
    assert_eq!(
        peer_message["recovery_epoch"],
        serde_json::json!(epoch),
        "immutable message records its authenticated origin epoch"
    );
    assert_eq!(
        peer_message["spec_revision"], candidate.spec_revision,
        "peer message carries canonical approved input revision without granting Task history"
    );
    let peer_shared = client
        .get(format!(
            "http://{addr}/v1/mailbox/messages/{message}/artifacts/{digest}"
        ))
        .bearer_auth(&peer_token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        peer_shared.status(),
        reqwest::StatusCode::OK,
        "WorkerB/taskB can read exactly the broker-authorized WorkerA/taskA message artifact without task acceptance"
    );
    assert_eq!(peer_shared.bytes().await.unwrap().to_vec(), bytes);
    assert_eq!(
        client
            .get(format!("http://{addr}/v1/snapshots/task/{task}"))
            .bearer_auth(&peer_token)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "addressing grants no peer task/history"
    );
    assert_eq!(
        client
            .get(&read_url)
            .bearer_auth(&peer_token)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "direct peer artifact access still requires task scope"
    );
    assert_eq!(
        client
            .get(format!(
                "http://{addr}/v1/mailbox/messages/{message}/artifacts/{logs_digest}"
            ))
            .bearer_auth(&peer_token)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "even real registered evidence is denied when not explicitly listed in this addressed message"
    );
    let peer_init = client
        .post(&mcp_url)
        .bearer_auth(&peer_token)
        .header("accept", "application/json, text/event-stream")
        .json(&initialize)
        .send()
        .await
        .unwrap();
    assert_eq!(peer_init.status(), reqwest::StatusCode::OK);
    let peer_session = peer_init.headers()["mcp-session-id"]
        .to_str()
        .unwrap()
        .to_string();
    let peer_mcp = |body: serde_json::Value| {
        client
            .post(&mcp_url)
            .bearer_auth(&peer_token)
            .header("accept", "application/json, text/event-stream")
            .header("mcp-session-id", &peer_session)
            .header("mcp-protocol-version", "2025-11-25")
            .json(&body)
    };
    assert_eq!(
        peer_mcp(serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::ACCEPTED
    );
    let peer_resource:serde_json::Value=peer_mcp(serde_json::json!({"jsonrpc":"2.0","id":10,"method":"resources/read","params":{"uri":format!("corenius://message-artifact/{message}/{digest}")}})).send().await.unwrap().json().await.unwrap();
    use base64::Engine;
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(
                peer_resource["result"]["contents"][0]["blob"]
                    .as_str()
                    .unwrap()
            )
            .unwrap(),
        bytes,
        "cross-task MCP bytes equal exact HTTP/ArtifactService object"
    );
    let peer_task_denied:serde_json::Value=peer_mcp(serde_json::json!({"jsonrpc":"2.0","id":32,"method":"resources/read","params":{"uri":format!("corenius://task/{task}")}})).send().await.unwrap().json().await.unwrap();
    assert!(
        peer_task_denied["error"].is_object(),
        "exact artifact disclosure never grants the peer canonical task/history resource"
    );
    let peer_denied:serde_json::Value=peer_mcp(serde_json::json!({"jsonrpc":"2.0","id":11,"method":"resources/read","params":{"uri":format!("corenius://message-artifact/{message}/{logs_digest}")}})).send().await.unwrap().json().await.unwrap();
    assert!(
        peer_denied["error"].is_object(),
        "MCP does not broaden the narrow addressed artifact grant"
    );
    let message_resource:serde_json::Value=mcp(serde_json::json!({"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":format!("corenius://message-artifact/{message}/{digest}")}})).send().await.unwrap().json().await.unwrap();
    assert_eq!(
        message_resource["result"]["contents"][0]["mimeType"],
        "application/octet-stream"
    );
    let snapshot = service.snapshot(&owner, task).await.unwrap();
    assert_ne!(snapshot.status, "accepted");
    assert_eq!(snapshot.reports.len(), 1);
    // Owner fixture policy explicitly permits accepting this exact independently
    // executed candidate/report. Source edit or mailbox answer cannot accept it.
    let accept = cmd(
        epoch,
        task,
        "wait-actual-accept",
        6,
        ArtifactCommand::Accept {
            candidate_id: submitted.candidate_id.clone().unwrap(),
            report_id: verified["result"]["structuredContent"]["report_id"]
                .as_str()
                .unwrap()
                .into(),
        },
    );
    let accepted = client
        .post(&commands_url)
        .bearer_auth(&owner)
        .json(&accept)
        .send()
        .await
        .unwrap();
    assert_eq!(accepted.status(), reqwest::StatusCode::OK);
    assert!(
        accepted.json::<serde_json::Value>().await.unwrap()["accepted"]
            .as_bool()
            .unwrap()
    );
    let recovered = bridge.progress(&owner, &workflow).await.unwrap();
    assert!(
        recovered
            .mailbox_attention
            .iter()
            .any(|w| w.reference == task && w.satisfied && w.checkpoint == checkpoint),
        "existing Activity read observes actual canonical acceptance and repairs an undelivered dependency hint"
    );
    assert_eq!(
        bridge
            .progress(&owner, &workflow)
            .await
            .unwrap()
            .mailbox_attention
            .len(),
        recovered.mailbox_attention.len(),
        "repeat observation does not create another wait"
    );
    let permission = Uuid::new_v4();
    let protected_operation = serde_json::json!({"family":"protected_integration","arguments":{"task":peer_task,"repository":"owned-fixture","operation":"fixture-only-no-external-effect"}});
    let request = serde_json::json!({"schema_version":1,"key":"canonical-permission-request","recovery_epoch":epoch,"command":{"kind":"request_approval","mission":mission,"id":permission,"task":peer_task,"generation":1,"executor":peer_principal,"operation":protected_operation,"expires_in_seconds":30}});
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(lead_token)
            .json(&request)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let grant = serde_json::json!({"schema_version":1,"key":"canonical-owner-grant","recovery_epoch":epoch,"command":{"kind":"grant_approval","request":permission,"rationale":"Retain this exact permission independently of the requesting conversation"}});
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(&owner)
            .json(&grant)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    assert_eq!(
        client
            .get(format!(
                "http://{addr}/v1/mailbox/records/approval/{permission}"
            ))
            .bearer_auth(&peer_token)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "named executor reads its exact canonical permission within its own task scope"
    );
    let replace = serde_json::json!({"schema_version":1,"key":"replace-permission-requester","recovery_epoch":epoch,"command":{"kind":"replace_lead","mission":mission,"expected_generation":1,"principal":Uuid::new_v4(),"credential_id":Uuid::new_v4(),"token_hash":token_hash("permission-replacement-lead"),"expires_in_seconds":30}});
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(&owner)
            .json(&replace)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let consume = serde_json::json!({"schema_version":1,"key":"consume-canonical-owner-grant","recovery_epoch":epoch,"command":{"kind":"consume_approval","request":permission,"operation":protected_operation}});
    assert_eq!(
        client
            .post(&mailbox_url)
            .bearer_auth(&peer_token)
            .json(&consume)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "current scoped executor retains exact canonical Owner grant across requesting Lead conversation loss; this records permission only"
    );
    if let Ok(path) = std::env::var("TEMPORAL_FIXTURE_EVIDENCE") {
        std::fs::create_dir_all(&path).unwrap();
        let proof = serde_json::json!({"mission":mission,"task":task,"peer_task":peer_task,"attempt":assigned_attempt,"candidate":submitted.candidate_id,"report":verified["result"]["structuredContent"]["report_id"],"base_sha":base,"candidate_sha":commit,"observed_candidate_sha":measured_commit,"observed_tree":measured_tree,"observed_base":measured_base,"bundle_digest":digest,"log_digest":logs_digest,"checks":executed,"addressed_message":message,"exact_permission":permission,"fixture_owner_acceptance":true,"http_mcp_peer_bytes_equal":true});
        std::fs::write(
            std::path::Path::new(&path).join("artifact-transport.json"),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
    }
    server.abort();
    std::fs::remove_dir_all(root).unwrap();
}

struct OwnedFixtureDir(std::path::PathBuf);
impl Drop for OwnedFixtureDir {
    fn drop(&mut self) {
        if self.0.exists() {
            std::fs::remove_dir_all(&self.0).expect("owned disposable artifact fixture cleanup");
        }
    }
}

struct OwnedServer(std::process::Child);
impl Drop for OwnedServer {
    fn drop(&mut self) {
        if self.0.try_wait().unwrap().is_none() {
            self.0.kill().unwrap();
        }
        self.0.wait().unwrap();
    }
}
#[tokio::test]
async fn executable_serve_registers_configured_artifact_http_and_mcp_seam() {
    let base = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").unwrap();
    let schema = format!("mailbox_cli_{}", Uuid::new_v4().simple());
    let (db, connection) = tokio_postgres::connect(&base, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    db.batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{base} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let root = std::env::temp_dir().join(format!("mailbox-cli-fixture-{}", Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let _owned = OwnedFixtureDir(root.clone());
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    drop(socket);
    let child = std::process::Command::new(env!("CARGO_BIN_EXE_team-control-plane"))
        .args(["serve", &address.to_string()])
        .env("TEAM_CONTROL_PLANE_DSN", &dsn)
        .env("TEAM_CONTROL_PLANE_RECOVERY_EPOCH", epoch.to_string())
        .env("TEAM_CONTROL_PLANE_ARTIFACT_ROOT", &root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut server = OwnedServer(child);
    let client = reqwest::Client::new();
    let url = format!("http://{address}/mcp");
    let initialize = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"owned-cli-fixture","version":"1"}}});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let response = loop {
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "configured serve exited unexpectedly"
        );
        if let Ok(response) = client
            .post(&url)
            .bearer_auth(&owner)
            .header("accept", "application/json, text/event-stream")
            .json(&initialize)
            .send()
            .await
        {
            break response;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let session = response.headers()["mcp-session-id"]
        .to_str()
        .unwrap()
        .to_string();
    let call = |body: serde_json::Value| {
        client
            .post(&url)
            .bearer_auth(&owner)
            .header("accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session)
            .header("mcp-protocol-version", "2025-11-25")
            .json(&body)
    };
    assert_eq!(
        call(serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::ACCEPTED
    );
    let tools: serde_json::Value =
        call(serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    assert!(
        tools["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "artifact_command"),
        "real executable must register its supplied artifact adapter, not merely a test-only router"
    );
    assert_eq!(
        client
            .get(format!(
                "http://{address}/v1/artifacts/tasks/{}",
                Uuid::new_v4()
            ))
            .bearer_auth(&owner)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::NOT_FOUND,
        "configured HTTP reaches authoritative artifact service"
    );
    drop(server);
}
