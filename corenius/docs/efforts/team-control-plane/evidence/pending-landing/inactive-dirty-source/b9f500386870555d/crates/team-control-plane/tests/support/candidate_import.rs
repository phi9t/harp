//! Public candidate import regression: content source cannot bypass research authority.
use super::*;
use team_control_plane::{ErrorCode, mailbox, supervisor};

async fn fixture(research: bool) -> (Api, Value, Uuid, Value) {
    let api = Api::from_fixture(Fixture::with_retained_preparation().await).await;
    println!(
        "candidate_import_fixture {}",
        json!({"root":api.fixture.root,"dsn":api.fixture.dsn,"epoch":api.fixture.epoch,"task":api.fixture.task,"attempt":api.fixture.attempt})
    );
    let intent = if research {
        api.research_intent("candidate-research-r1", 0, "candidate-r1")
            .await
    } else {
        let result = api
            .post(
                &api.fixture.owner,
                &api.admission("candidate-production-r1", 0),
            )
            .await;
        assert_eq!(result.0, 200);
        result.1
    };
    api.artifact(&api.fixture.owner,"candidate-policy",json!({"kind":"configure","verifier":api.fixture.verifier,"lead_acceptance":true,"base":api.fixture.base,"suite":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","mandatory_commands":[["true"]]})).await;
    let worker = api.fixture.worker_episode("candidate-worker").await;
    let supervisor = api.fixture.supervisor().await;
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
        .start(
            "assembled-worker",
            execution.id,
            vec![
                "/bin/sh".into(),
                "-c".into(),
                "printf 'measured candidate\\n' > src/result".into(),
            ],
            10,
        )
        .await
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let measured = loop {
        if let Some(result) = supervisor
            .result("assembled-worker", execution.id)
            .await
            .unwrap()
        {
            break result;
        }
        assert!(std::time::Instant::now() < deadline);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
    assert_eq!(measured.exit_code, Some(0));
    assert!(measured.namespace_stopped);
    let result_bytes = std::fs::read(execution.path.join("src/result")).unwrap();
    assert_eq!(result_bytes, b"measured candidate\n");
    assembled::git(&execution.path, &["add", "src/result"]);
    assembled::git(
        &execution.path,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "measured candidate",
        ],
    );
    let commit = assembled::git(&execution.path, &["rev-parse", "HEAD"]);
    let tree = assembled::git(&execution.path, &["rev-parse", "HEAD^{tree}"]);
    let bundle = api.fixture.root.join("candidate.bundle");
    assembled::git(
        &execution.path,
        &["bundle", "create", bundle.to_str().unwrap(), "HEAD"],
    );
    let bytes = std::fs::read(&bundle).unwrap();
    let artifact = api.upload("assembled-worker", &bytes).await;
    let candidate = json!({"commit":commit,"tree":tree,"base":api.fixture.base,"spec_digest":api.fixture.spec,"spec_revision":2,"generation":1,"artifact":artifact,"deliverables":["src/result"],"risks":[]});
    let submitted = api
        .artifact(
            "assembled-worker",
            "candidate-submit",
            json!({"kind":"submit","candidate":candidate}),
        )
        .await;
    println!(
        "candidate_import_measured {}",
        json!({"execution":execution.id,"measurement":measured,"candidate":candidate,"submitted":submitted,"bundle":bundle,"result_bytes":result_bytes,"manifest":supervisor.inspect("assembled-worker",execution.id).await.unwrap().manifest})
    );
    supervisor
        .release(&api.fixture.owner, execution.id)
        .await
        .unwrap();
    println!(
        "candidate_import_worker_released {}",
        json!({"execution":execution.id,"view":supervisor.inspect(&api.fixture.owner,execution.id).await.unwrap()})
    );
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    let capture = json!({"kind":"capture_input","mission":api.fixture.mission,"id":input,"attempt":api.fixture.attempt,"purpose":"verification","limit":4});
    assert_eq!(
        api.mail("assembled-verifier", "candidate-input", capture.clone())
            .await
            .0,
        200
    );
    let packet = mailbox::MailboxService::new(api.fixture.broker.clone())
        .record("assembled-verifier", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    let admit = json!({"kind":"admit_episode","mission":api.fixture.mission,"id":episode,"attempt":api.fixture.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]});
    assert_eq!(
        api.mail("assembled-verifier", "candidate-episode", admit.clone())
            .await
            .0,
        200
    );
    println!(
        "candidate_import_input {}",
        json!({"capture":capture,"packet":packet,"admit":admit,"episode_record":mailbox::MailboxService::new(api.fixture.broker.clone()).record("assembled-verifier",mailbox::RecordKind::Episode,episode).await.unwrap(),"intent":intent})
    );
    (api, intent, episode, packet)
}

async fn import(
    api: &Api,
    episode: Uuid,
) -> team_control_plane::Result<supervisor::ExecutionHandle> {
    let service = artifacts::ArtifactService::open(
        api.fixture.broker.clone(),
        &api.fixture.root.join("artifacts"),
    )
    .await
    .unwrap();
    api.fixture
        .supervisor()
        .await
        .create_episode_from_candidate(
            "assembled-verifier",
            episode,
            &service,
            supervisor::CreateFault::None,
        )
        .await
}

async fn release(api: &Api, execution: Uuid) {
    let supervisor = api.fixture.supervisor().await;
    supervisor
        .cancel("assembled-verifier", execution)
        .await
        .unwrap();
    let stopped = supervisor
        .inspect("assembled-verifier", execution)
        .await
        .unwrap();
    supervisor
        .release("assembled-verifier", execution)
        .await
        .unwrap();
    println!(
        "candidate_import_cleanup {}",
        json!({"execution":execution,"stopped":stopped,"released":supervisor.inspect("assembled-verifier",execution).await.unwrap()})
    );
}

#[tokio::test]
async fn candidate_import_refuses_r2_before_preparation() {
    let (api, _, episode, _) = fixture(true).await;
    let before = api.legacy_task(api.fixture.task).await;
    api.research_intent("candidate-r2", 1, "candidate-r2").await;
    let ordinary = api
        .fixture
        .supervisor()
        .await
        .create_episode(
            "assembled-verifier",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await;
    assert_eq!(ordinary.unwrap_err().code, ErrorCode::StaleRevision);
    let result = import(&api, episode).await;
    println!(
        "candidate_import_stale_prepare {}",
        json!({"before":before,"after":api.legacy_task(api.fixture.task).await,"result":result})
    );
    if let Ok(execution) = &result {
        println!(
            "candidate_import_baseline_manifest {}",
            json!({"view":api.fixture.supervisor().await.inspect("assembled-verifier",execution.id).await.unwrap()})
        );
        release(&api, execution.id).await;
    }
    assert_eq!(result.unwrap_err().code, ErrorCode::StaleRevision);
}

#[tokio::test]
async fn candidate_import_refuses_r2_after_preparation() {
    let (api, _, episode, _) = fixture(true).await;
    let execution = import(&api, episode).await.unwrap();
    let supervisor = api.fixture.supervisor().await;
    let prepared = supervisor
        .inspect("assembled-verifier", execution.id)
        .await
        .unwrap();
    api.research_intent("candidate-r2", 1, "candidate-r2").await;
    let result = supervisor
        .start("assembled-verifier", execution.id, vec!["true".into()], 10)
        .await;
    println!(
        "candidate_import_stale_start {}",
        json!({"prepared":prepared,"result":result,"current":api.snapshot().await})
    );
    release(&api, execution.id).await;
    assert_eq!(result.unwrap_err().code, ErrorCode::StaleGeneration);
}

#[tokio::test]
async fn candidate_import_current_captures_exact_research_pin() {
    if let Ok(path) = std::env::var("CANDIDATE_IMPORT_LEGACY_PREPARED") {
        let retained: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let dsn = retained["dsn"].as_str().unwrap();
        let epoch = serde_json::from_value(retained["epoch"].clone()).unwrap();
        let broker = team_control_plane::Broker::connect(dsn, epoch)
            .await
            .unwrap();
        let root = std::path::PathBuf::from(retained["root"].as_str().unwrap());
        let id = serde_json::from_value(retained["execution"].clone()).unwrap();
        let supervisor = supervisor::Supervisor::open(
            broker,
            &root.join("workspaces"),
            env!("CARGO_BIN_EXE_team-control-plane"),
        )
        .await
        .unwrap();
        let before = supervisor.inspect("assembled-verifier", id).await.unwrap();
        assert_eq!(before.state, supervisor::ExecutionState::Ready);
        assert!(before.manifest.research_verification.is_none());
        let (db, connection) = tokio_postgres::connect(dsn, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move { connection.await.unwrap() });
        let eligible:bool=db.query_one("SELECT e.deadline>clock_timestamp() AND e.state='admitted' AND c.expires_at>clock_timestamp() AND e.task_generation=t.generation FROM episode_executions x JOIN mailbox_episodes e ON e.id=x.episode_id JOIN credentials c ON c.id=x.credential_id JOIN tasks t ON t.id=x.task_id WHERE x.execution_id=$1",&[&id]).await.unwrap().get(0);
        assert!(
            eligible,
            "original baseline record must still have current role/generation/deadline"
        );
        let result = supervisor
            .start("assembled-verifier", id, vec!["true".into()], 10)
            .await;
        println!(
            "candidate_import_original_missing_pin {}",
            json!({"before":before,"eligible":eligible,"result":result,"baseline":retained})
        );
        assert_eq!(result.unwrap_err().code, ErrorCode::StaleGeneration);
        supervisor.cancel("assembled-verifier", id).await.unwrap();
        let stopped = supervisor.inspect("assembled-verifier", id).await.unwrap();
        supervisor.release("assembled-verifier", id).await.unwrap();
        println!(
            "candidate_import_original_cleanup {}",
            json!({"execution":id,"stopped":stopped,"released":supervisor.inspect("assembled-verifier",id).await.unwrap()})
        );
    }
    let (api, intent, episode, packet) = fixture(true).await;
    let execution = import(&api, episode).await.unwrap();
    let view = api
        .fixture
        .supervisor()
        .await
        .inspect("assembled-verifier", execution.id)
        .await
        .unwrap();
    println!(
        "candidate_import_current {}",
        json!({"manifest":view.manifest,"input":packet,"intent":intent})
    );
    if let Ok(path) = std::env::var("CANDIDATE_IMPORT_RETAIN_PREPARED") {
        std::fs::write(path,serde_json::to_vec_pretty(&json!({"dsn":api.fixture.dsn,"epoch":api.fixture.epoch,"root":api.fixture.root,"execution":execution.id,"manifest":view.manifest})).unwrap()).unwrap();
    } else {
        let supervisor = api.fixture.supervisor().await;
        let argv=vec!["/bin/sh".to_owned(),"-c".into(),"test \"$(cat src/result)\" = 'measured candidate' && printf 'independent imported content\\n'".into()];
        supervisor
            .start("assembled-verifier", execution.id, argv, 10)
            .await
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let measured = loop {
            if let Some(result) = supervisor
                .result("assembled-verifier", execution.id)
                .await
                .unwrap()
            {
                break result;
            }
            assert!(std::time::Instant::now() < deadline);
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        };
        assert_eq!(measured.exit_code, Some(0));
        assert!(measured.namespace_stopped);
        let logs = std::fs::read(&view.manifest.log_path).unwrap();
        assert_eq!(artifacts::sha256(&logs), measured.logs);
        println!(
            "candidate_import_current_measurement {}",
            json!({"measurement":measured,"log_bytes":logs})
        );
        release(&api, execution.id).await;
    }
    assert_eq!(
        packet["packet"]["task"]["research_verification"]["intent"],
        intent["intent"]
    );
    assert!(
        view.manifest.research_verification.is_some(),
        "research pin must be captured independently of content source"
    );
    assert_eq!(
        serde_json::to_value(view.manifest.research_verification).unwrap(),
        packet["packet"]["task"]["research_verification"]
    );
}

#[tokio::test]
async fn candidate_import_refuses_conflicting_research_policy() {
    let mut errors = Vec::new();
    for field in ["suite", "environment", "verifier"] {
        let (api, _, episode, _) = fixture(true).await;
        let mut policy = json!({"kind":"configure","verifier":api.fixture.verifier,"lead_acceptance":true,"base":api.fixture.base,"suite":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","mandatory_commands":[["true"]]});
        policy[field] = if field == "verifier" {
            json!(Uuid::new_v4())
        } else {
            json!("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc")
        };
        api.artifact(
            &api.fixture.owner,
            &format!("candidate-conflicting-{field}"),
            policy.clone(),
        )
        .await;
        let result = import(&api, episode).await;
        println!(
            "candidate_import_policy_conflict {}",
            json!({"field":field,"policy":policy,"result":result})
        );
        if let Ok(execution) = &result {
            release(&api, execution.id).await;
        }
        errors.push(result.map(|_| ()).map_err(|e| e.code));
    }
    assert_eq!(errors, vec![Err(ErrorCode::DependencyUnmet); 3]);
}

#[tokio::test]
async fn candidate_import_refuses_research_input_reclassified_as_production() {
    let (api, _, episode, _) = fixture(true).await;
    assert_eq!(
        api.post(
            &api.fixture.owner,
            &api.admission("candidate-production-r2", 1)
        )
        .await
        .0,
        200
    );
    let result = import(&api, episode).await;
    println!(
        "candidate_import_reclassified_prepare {}",
        json!({"result":result})
    );
    if let Ok(execution) = &result {
        release(&api, execution.id).await;
    }
    assert_eq!(result.unwrap_err().code, ErrorCode::StaleRevision);
}

#[tokio::test]
async fn candidate_import_refuses_missing_pin_after_production_becomes_research() {
    let (api, _, episode, _) = fixture(false).await;
    let execution = import(&api, episode).await.unwrap();
    api.research_intent("candidate-research-r2", 1, "candidate-r2")
        .await;
    let result = api
        .fixture
        .supervisor()
        .await
        .start("assembled-verifier", execution.id, vec!["true".into()], 10)
        .await;
    println!(
        "candidate_import_missing_pin {}",
        json!({"result":result,"manifest":api.fixture.supervisor().await.inspect("assembled-verifier",execution.id).await.unwrap().manifest})
    );
    release(&api, execution.id).await;
    assert_eq!(result.unwrap_err().code, ErrorCode::StaleGeneration);
}

#[tokio::test]
async fn candidate_import_refuses_prepared_research_reclassified_as_production() {
    let (api, _, episode, _) = fixture(true).await;
    let execution = import(&api, episode).await.unwrap();
    assert_eq!(
        api.post(
            &api.fixture.owner,
            &api.admission("candidate-production-r2", 1)
        )
        .await
        .0,
        200
    );
    let result = api
        .fixture
        .supervisor()
        .await
        .start("assembled-verifier", execution.id, vec!["true".into()], 10)
        .await;
    println!(
        "candidate_import_reclassified_start {}",
        json!({"result":result,"manifest":api.fixture.supervisor().await.inspect("assembled-verifier",execution.id).await.unwrap().manifest})
    );
    release(&api, execution.id).await;
    assert_eq!(result.unwrap_err().code, ErrorCode::StaleGeneration);
}
