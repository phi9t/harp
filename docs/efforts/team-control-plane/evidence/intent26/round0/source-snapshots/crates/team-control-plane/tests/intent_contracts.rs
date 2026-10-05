//! Public JSON contract tests: removing admission or its immutable persistence
//! must fail observable HTTP results and durable reads on real PostgreSQL.
#[allow(dead_code)]
#[path = "support/assembled.rs"]
mod assembled;
use assembled::Fixture;
use serde_json::{Value, json};
use team_control_plane::{artifacts, http};
use uuid::Uuid;

struct Api {
    fixture: Fixture,
    client: reqwest::Client,
    url: String,
    server: tokio::task::JoinHandle<()>,
}
impl Api {
    fn command(&self, key: &str, revision: i64, command: Value) -> Value {
        json!({"schema_version":2,"key":key,"expected_revision":revision,"recovery_epoch":self.fixture.epoch,"task":self.fixture.task,"command":command})
    }
    async fn new() -> Self {
        let fixture = Fixture::without_requirement().await;
        println!(
            "postgres_fixture {}",
            json!({"dsn":std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").unwrap(),"epoch":fixture.epoch})
        );
        Self::from_fixture(fixture).await
    }
    async fn measured() -> Self {
        Self::from_fixture(Fixture::new().await).await
    }
    async fn from_fixture(fixture: Fixture) -> Self {
        let service = artifacts::ArtifactService::open(
            fixture.broker.clone(),
            &fixture.root.join("artifacts"),
        )
        .await
        .unwrap();
        let router = http::router_with_artifacts(fixture.broker.clone(), service);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        Self {
            fixture,
            client: reqwest::Client::new(),
            url,
            server,
        }
    }
    fn admission(&self, key: &str, revision: i64) -> Value {
        let snapshot = b"produce exact src/result";
        json!({"schema_version":2,"key":key,"expected_revision":revision,
            "recovery_epoch":self.fixture.epoch,"task":self.fixture.task,
            "command":{"kind":"intent.admit",
                "work":{"ledger":"git","deployment":"fixture-deployment","tenant":"fixture-tenant","project":"fixture-project","item":"26"},
                "source":{"adapter":"git","locator":self.fixture.root.join("store/SPEC.md"),"revision":self.fixture.base,"availability":"available","snapshot":snapshot,"digest":"881fa3537a1d388a9b61011411583564a434fdc7ad8b821de142d378e7bbead3"},
                "intent":{"objective":"produce exact result","constraints":["owned fixture only"],"allowed_effects":["read","write"],
                    "resources":{"attempts":8,"episodes":8,"repairs":3,"execution_seconds":30},
                    "acceptance":{"outcome":"production","allow_negative":false,"criteria":["independent exact verification"],"verifier":self.fixture.verifier,"policy":"suite-v2","suite":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","mandatory_commands":[["true"]]},
                    "dependencies":[],"legacy_spec_digest":self.fixture.spec}}})
    }
    async fn post(&self, token: &str, body: &Value) -> (u16, Value) {
        let response = self
            .client
            .post(format!("{}/v2/contracts/commands", self.url))
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .unwrap();
        let status = response.status().as_u16();
        let bytes = response.bytes().await.unwrap();
        let value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| json!({"unparsed":String::from_utf8_lossy(&bytes)}));
        println!(
            "public_contract_response {}",
            json!({"key":body["key"],"family":body["command"]["kind"],"task":body["task"],"status":status,"result":value})
        );
        (status, value)
    }
    async fn snapshot(&self) -> Value {
        let response = self
            .client
            .get(format!(
                "{}/v2/contracts/tasks/{}",
                self.url, self.fixture.task
            ))
            .bearer_auth(&self.fixture.owner)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 200);
        response.json().await.unwrap()
    }
    async fn finish(self) {
        self.server.abort();
        self.fixture.finish().await;
    }
}

#[tokio::test]
async fn additive_upgrade_preserves_exact_legacy_specs_and_nullable_original_provenance() {
    use team_control_plane::*;
    let api = Api::new().await;
    let lead_episode = api.fixture.lead_episode("legacy-null-attempt").await;
    let supervisor = api.fixture.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-lead",
            lead_episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    let (db, connection) = tokio_postgres::connect(&api.fixture.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let original:Value=db.query_one("SELECT jsonb_build_object('specs',(SELECT jsonb_agg(to_jsonb(s) ORDER BY digest) FROM task_specs s),'workspace',(SELECT to_jsonb(w) FROM supervised_workspaces w WHERE w.id=$1),'episode',(SELECT to_jsonb(e) FROM mailbox_episodes e WHERE e.id=$2),'execution',(SELECT to_jsonb(x) FROM episode_executions x WHERE x.execution_id=$1))",&[&execution.id,&lead_episode]).await.unwrap().get(0);
    assert!(original["workspace"]["attempt_id"].is_null());
    assert!(original["execution"]["attempt_id"].is_null());
    assert!(original["episode"]["attempt_id"].is_null());
    // Reconstruct an owned legacy deployment: only the empty v2 tables are
    // removed. All actual original v1 provenance is retained unchanged.
    db.batch_execute("DROP TABLE contract_schema_version,contract_legacy_bindings,contract_current,contract_work,contract_sources,contract_acceptance,contract_intents,contract_commands,contract_proposals,contract_delegations,contract_plans,contract_completed_steps,contract_verification_bindings,contract_proofs,contract_outcomes,contract_dependencies,contract_dependency_revalidations CASCADE").await.unwrap();
    migrate(&api.fixture.dsn).await.unwrap();
    migrate(&api.fixture.dsn).await.unwrap();
    let retained:Value=db.query_one("SELECT jsonb_build_object('specs',(SELECT jsonb_agg(to_jsonb(s) ORDER BY digest) FROM task_specs s),'workspace',(SELECT to_jsonb(w) FROM supervised_workspaces w WHERE w.id=$1),'episode',(SELECT to_jsonb(e) FROM mailbox_episodes e WHERE e.id=$2),'execution',(SELECT to_jsonb(x) FROM episode_executions x WHERE x.execution_id=$1))",&[&execution.id,&lead_episode]).await.unwrap().get(0);
    assert_eq!(
        retained, original,
        "migration must preserve every exact legacy binding and absent original Attempt"
    );
    let mapping:Value=db.query_one("SELECT jsonb_build_object('original_spec',original_spec,'disposition',disposition) FROM contract_legacy_bindings WHERE task_id=$1 AND spec_digest=$2",&[&api.fixture.task,&api.fixture.spec]).await.unwrap().get(0);
    assert_eq!(mapping["original_spec"], original["specs"][0]);
    assert_eq!(mapping["disposition"], "explicit_mapping_required");
    let closed = api.snapshot().await;
    assert_eq!(closed["admission_open"], false);
    assert_eq!(closed["intents"], json!([]));
    let mut ambiguous = api.admission("wrong-legacy-binding", 0);
    ambiguous["command"]["intent"]["legacy_spec_digest"] =
        json!("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc");
    assert_eq!(
        api.post(&api.fixture.owner, &ambiguous).await.1["code"],
        "stale_revision"
    );
    assert_eq!(api.snapshot().await["admission_open"], false);
    let admitted = api
        .post(&api.fixture.owner, &api.admission("explicit-mapping", 0))
        .await;
    assert_eq!(admitted.0, 200);
    assert!(
        db.execute("UPDATE contract_intents SET intent='{}'::jsonb", &[])
            .await
            .is_err()
    );
    assert!(
        db.execute("DELETE FROM contract_sources", &[])
            .await
            .is_err()
    );
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
    println!("migration_original_provenance {}", original);
    api.finish().await;
}

impl Api {
    async fn upload(&self, token: &str, bytes: &[u8]) -> String {
        let digest = artifacts::sha256(bytes);
        let response = self
            .client
            .post(format!("{}/v1/artifacts/objects", self.url))
            .bearer_auth(token)
            .json(&json!({"task":self.fixture.task,"digest":digest,"bytes":bytes}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status().as_u16(),
            200,
            "real immutable evidence publication"
        );
        digest
    }
    async fn artifact(&self, token: &str, key: &str, command: Value) -> Value {
        let snapshot: Value = self
            .client
            .get(format!(
                "{}/v1/artifacts/tasks/{}",
                self.url, self.fixture.task
            ))
            .bearer_auth(&self.fixture.owner)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let response=self.client.post(format!("{}/v1/artifacts/commands",self.url)).bearer_auth(token).json(&json!({"key":key,"task":self.fixture.task,"expected_revision":snapshot["revision"],"generation":1,"recovery_epoch":self.fixture.epoch,"command":command})).send().await.unwrap();
        let status = response.status().as_u16();
        let value: Value = response.json().await.unwrap();
        assert_eq!(status, 200, "ArtifactService {key}: {value}");
        value
    }
    async fn research_intent(&self, key: &str, revision: i64, source_revision: &str) -> Value {
        let mut request = self.admission(key, revision);
        request["command"]["source"]["revision"] = json!(source_revision);
        request["command"]["intent"]["acceptance"]["outcome"] = json!("investigation_complete");
        request["command"]["intent"]["acceptance"]["allow_negative"] = json!(true);
        let result = self.post(&self.fixture.owner, &request).await;
        assert_eq!(result.0, 200, "research admission: {}", result.1);
        result.1
    }
    async fn accept_research(&self, intent: &Value, start: i64, key: &str) -> Value {
        let evidence = self
            .upload(
                "assembled-worker",
                b"retained investigation: no feasible candidate under admitted constraints",
            )
            .await;
        let proposal=self.command(&format!("{key}-proposal"),start,json!({"kind":"outcome.propose","intent":intent["intent"],"evidence":evidence,"negative":true,"evaluator_change":false}));
        let proposed = self.post("assembled-worker", &proposal).await;
        assert_eq!(proposed.0, 200, "worker proposal: {}", proposed.1);
        assert_eq!(proposed.1["disposition"], "proposed");
        let verification=self.command(&format!("{key}-verify"),start,json!({"kind":"research.verify","intent":intent["intent"],"acceptance":intent["acceptance"],"proposal":proposed.1["proposal"],"policy":"suite-v2","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}));
        assert_eq!(
            self.post("assembled-worker", &verification).await.0,
            401,
            "worker cannot manufacture verified outcome"
        );
        let verified = self.post("assembled-verifier", &verification).await;
        assert_eq!(
            verified.0, 200,
            "independent research verification: {}",
            verified.1
        );
        let accept = self.command(
            &format!("{key}-accept"),
            start + 1,
            json!({"kind":"outcome.accept","proof":verified.1["proof"]}),
        );
        assert_eq!(self.post("assembled-worker", &accept).await.0, 401);
        let accepted = self.post(&self.fixture.owner, &accept).await;
        assert_eq!(accepted.0, 200, "owner research acceptance: {}", accepted.1);
        assert_eq!(accepted.1["outcome"], "investigation_complete");
        accepted.1
    }
}

#[tokio::test]
async fn negative_research_is_independent_exact_evidence_and_evaluator_proposals_remain_inert() {
    let api = Api::new().await;
    let intent = api.research_intent("research", 0, "research-r1").await;
    let accepted = api.accept_research(&intent, 1, "research").await;
    let snapshot = api.snapshot().await;
    assert_eq!(snapshot["accepted"][0]["subject"], accepted["subject"]);
    assert_eq!(snapshot["accepted"][0]["outcome"], "investigation_complete");
    let service = artifacts::ArtifactService::open(
        api.fixture.broker.clone(),
        &api.fixture.root.join("artifacts"),
    )
    .await
    .unwrap();
    assert!(
        service
            .collect(&api.fixture.owner, std::time::Duration::ZERO)
            .await
            .unwrap()
            .is_empty(),
        "admitted research evidence must survive ordinary ArtifactService collection"
    );
    let evidence = api
        .upload("assembled-worker", b"proposed evaluator change, unpromoted")
        .await;
    let proposal=api.command("evaluator-proposal",3,json!({"kind":"outcome.propose","intent":intent["intent"],"evidence":evidence,"negative":true,"evaluator_change":true}));
    let proposed = api.post("assembled-worker", &proposal).await;
    assert_eq!(proposed.0, 200);
    assert_eq!(
        api.snapshot().await["accepted"].as_array().unwrap().len(),
        1
    );
    let verify=api.command("do-not-promote-evaluator",3,json!({"kind":"research.verify","intent":intent["intent"],"acceptance":intent["acceptance"],"proposal":proposed.1["proposal"],"policy":"suite-v2","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}));
    assert_eq!(
        api.post("assembled-verifier", &verify).await.1["code"],
        "dependency_unmet"
    );
    api.finish().await;
}

#[tokio::test]
async fn admitted_prerequisite_supersession_blocks_fresh_grants_and_retains_old_receipt() {
    use team_control_plane::*;
    let api = Api::new().await;
    let prerequisite = api.research_intent("prerequisite", 0, "research-r1").await;
    let accepted = api.accept_research(&prerequisite, 1, "prerequisite").await;
    let dependent = Uuid::new_v4();
    let spec = TaskSpec {
        objective: "dependent fixture".into(),
        input: GitInput {
            repository: api.fixture.root.join("store").to_str().unwrap().into(),
            commit: api.fixture.base.clone(),
            path: "SPEC.md".into(),
            digest: "881fa3537a1d388a9b61011411583564a434fdc7ad8b821de142d378e7bbead3".into(),
        },
        dependencies: vec![],
        acceptance: vec!["exact proof".into()],
        deliverables: vec!["src/result".into()],
        write_scope: vec!["src".into()],
    };
    api.fixture
        .broker
        .execute(
            &api.fixture.owner,
            assembled::envelope(
                api.fixture.epoch,
                "dependent-task",
                0,
                Command::CreateTask {
                    id: dependent,
                    mission_id: api.fixture.mission,
                    spec,
                },
            ),
        )
        .await
        .unwrap();
    let legacy = api
        .fixture
        .broker
        .snapshot(&api.fixture.owner, SnapshotQuery::Task(dependent))
        .await
        .unwrap()
        .data["spec_digest"]
        .as_str()
        .unwrap()
        .to_owned();
    api.fixture
        .broker
        .execute(
            &api.fixture.owner,
            assembled::envelope(
                api.fixture.epoch,
                "dependent-admit",
                1,
                Command::AdmitTask {
                    task_id: dependent,
                    spec_digest: legacy.clone(),
                },
            ),
        )
        .await
        .unwrap();
    let work = api.admission("unused", 0)["command"]["work"].clone();
    let dependency = json!({"work":work,"intent":prerequisite["intent"],"accepted_subject":accepted["subject"],"policy":"suite-v2"});
    let mut request = api.admission("dependent-intent", 0);
    request["task"] = json!(dependent);
    request["command"]["work"]["item"] = json!("dependent");
    request["command"]["intent"]["legacy_spec_digest"] = json!(legacy);
    request["command"]["intent"]["dependencies"] = json!([dependency]);
    let admitted = api.post(&api.fixture.owner, &request).await;
    assert_eq!(
        admitted.0, 200,
        "exact prerequisite dependency: {}",
        admitted.1
    );
    let prior = api
        .fixture
        .broker
        .snapshot(&api.fixture.owner, SnapshotQuery::Task(api.fixture.task))
        .await
        .unwrap();
    assert_eq!(
        prior.data["operations"][0]["id"],
        api.fixture.admission.to_string()
    );
    let mut event=api.command("unadmitted-edit",3,json!({"kind":"source.propose","work":work,"source":api.admission("x",0)["command"]["source"]}));
    event["command"]["source"]["revision"] = json!("mutable-kata-edit");
    assert_eq!(api.post("assembled-worker", &event).await.0, 200);
    let read = |task| format!("{}/v2/contracts/tasks/{task}", api.url);
    let active: Value = api
        .client
        .get(read(dependent))
        .bearer_auth(&api.fixture.owner)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(active["dependencies"][0]["disposition"], "active");
    let amended = api.research_intent("supersede", 3, "research-r2").await;
    let suspended: Value = api
        .client
        .get(read(dependent))
        .bearer_auth(&api.fixture.owner)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(suspended["dependencies"][0]["disposition"], "suspended");
    assert_eq!(
        suspended["revision"], 2,
        "authoritative dependency disposition change must invalidate stale aggregate revisions"
    );
    let worker = Uuid::new_v4();
    api.fixture
        .broker
        .execute(
            &api.fixture.owner,
            assembled::envelope(
                api.fixture.epoch,
                "dependent-worker",
                0,
                Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: worker,
                    role: Role::Worker,
                    mission_id: Some(api.fixture.mission),
                    task_id: Some(dependent),
                    generation: Some(1),
                    token_hash: token_hash("dependent-worker"),
                    expires_in_seconds: 300,
                },
            ),
        )
        .await
        .unwrap();
    let grant = json!({"schema_version":1,"key":"suspended-grant","expected_revision":2,"expected_generation":null,"recovery_epoch":api.fixture.epoch,"command":{"kind":"assign_attempt","task_id":dependent,"worker_id":worker}});
    let response = api
        .client
        .post(format!("{}/v1/commands", api.url))
        .bearer_auth(&api.fixture.owner)
        .json(&grant)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 409);
    assert_eq!(
        response.json::<Value>().await.unwrap()["code"],
        "dependency_unmet"
    );
    let retained = api
        .fixture
        .broker
        .snapshot(&api.fixture.owner, SnapshotQuery::Task(api.fixture.task))
        .await
        .unwrap();
    assert_eq!(retained.data["operations"], prior.data["operations"]);
    let new_accept = api.accept_research(&amended, 4, "amended").await;
    let mut revalidate=api.command("revalidate",2,json!({"kind":"dependency.revalidate","intent":admitted.1["intent"],"dependencies":[{"work":work,"intent":amended["intent"],"accepted_subject":new_accept["subject"],"policy":"suite-v2"}]}));
    revalidate["task"] = json!(dependent);
    let mut stale_revalidation = revalidate.clone();
    stale_revalidation["key"] = json!("stale-dependent-revision");
    stale_revalidation["expected_revision"] = json!(1);
    assert_eq!(
        api.post(&api.fixture.owner, &stale_revalidation).await.1["code"],
        "stale_revision"
    );
    let revalidated = api.post(&api.fixture.owner, &revalidate).await;
    assert_eq!(
        revalidated.0, 200,
        "explicit exact revalidation: {}",
        revalidated.1
    );
    let mut grant = grant;
    grant["key"] = json!("current-grant");
    let response = api
        .client
        .post(format!("{}/v1/commands", api.url))
        .bearer_auth(&api.fixture.owner)
        .json(&grant)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "current dependency control"
    );
    let receipt: Value = response.json().await.unwrap();
    let attempt = receipt["attempt_id"].clone();
    let _third = api
        .research_intent("supersede-again", 6, "research-r3")
        .await;
    let cached = api
        .client
        .post(format!("{}/v1/commands", api.url))
        .bearer_auth(&api.fixture.owner)
        .json(&grant)
        .send()
        .await
        .unwrap();
    assert_eq!(
        cached.status().as_u16(),
        200,
        "earlier exact dependent grant receipt stays reconcilable after supersession"
    );
    assert_eq!(cached.json::<Value>().await.unwrap(), receipt);
    let retained = api
        .fixture
        .broker
        .snapshot(&api.fixture.owner, SnapshotQuery::Task(dependent))
        .await
        .unwrap();
    assert_eq!(
        retained.data["operations"][0]["id"],
        receipt["operation_id"]
    );
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    let capture = json!({"schema_version":1,"key":"dependent-input","recovery_epoch":api.fixture.epoch,"command":{"kind":"capture_input","mission":api.fixture.mission,"id":input,"attempt":attempt,"purpose":"implementation","limit":4}});
    let response = api
        .client
        .post(format!("{}/v1/mailbox/commands", api.url))
        .bearer_auth("dependent-worker")
        .json(&capture)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let packet: Value = api
        .client
        .get(format!("{}/v1/mailbox/records/input/{input}", api.url))
        .bearer_auth("dependent-worker")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let episode_request = json!({"schema_version":1,"key":"suspended-episode","recovery_epoch":api.fixture.epoch,"command":{"kind":"admit_episode","mission":api.fixture.mission,"id":episode,"attempt":attempt,"purpose":"implementation","input_packet":input,"input_digest":packet["digest"]}});
    let response = api
        .client
        .post(format!("{}/v1/mailbox/commands", api.url))
        .bearer_auth("dependent-worker")
        .json(&episode_request)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        409,
        "suspended dependency must block fresh Worker Episode"
    );
    assert_eq!(
        response.json::<Value>().await.unwrap()["code"],
        "dependency_unmet"
    );
    let old_revalidation = api.post(&api.fixture.owner, &revalidate).await;
    assert_eq!(
        old_revalidation.1["code"], "dependency_unmet",
        "cached revalidation cannot acquire current authority for superseded prerequisite"
    );
    api.finish().await;
}

#[tokio::test]
async fn production_requires_fresh_explicit_intent_binding_and_real_independent_artifact_measurement()
 {
    use team_control_plane::*;
    let api = Api::measured().await;
    let command = vec![
        "/bin/sh",
        "-c",
        "test \"$(cat src/result)\" = 'exact result' && printf 'independent gate\\n'",
    ];
    let mut request = api.admission("production-intent", 0);
    request["command"]["intent"]["acceptance"]["mandatory_commands"] = json!([command]);
    let admitted = api.post(&api.fixture.owner, &request).await;
    assert_eq!(admitted.0, 200);
    api.artifact(&api.fixture.owner,"policy",json!({"kind":"configure","verifier":api.fixture.verifier,"lead_acceptance":false,"base":api.fixture.base,"suite":request["command"]["intent"]["acceptance"]["suite"],"environment":request["command"]["intent"]["acceptance"]["environment"],"mandatory_commands":[command]})).await;
    let store = api.fixture.root.join("store");
    std::fs::write(store.join("src/result"), b"exact result\n").unwrap();
    assembled::git(&store, &["add", "src/result"]);
    assembled::git(
        &store,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "candidate",
        ],
    );
    let commit = assembled::git(&store, &["rev-parse", "HEAD"]);
    let tree = assembled::git(&store, &["rev-parse", "HEAD^{tree}"]);
    let bundle = api.fixture.root.join("candidate.bundle");
    assembled::git(
        &store,
        &["bundle", "create", bundle.to_str().unwrap(), "HEAD"],
    );
    let artifact = api
        .upload("assembled-worker", &std::fs::read(bundle).unwrap())
        .await;
    let candidate = json!({"commit":commit,"tree":tree,"base":api.fixture.base,"spec_digest":api.fixture.spec,"spec_revision":2,"generation":1,"artifact":artifact,"deliverables":["src/result"],"risks":[]});
    let submitted = api
        .artifact(
            "assembled-worker",
            "candidate",
            json!({"kind":"submit","candidate":candidate}),
        )
        .await;
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    let mailbox = mailbox::MailboxService::new(api.fixture.broker.clone());
    mailbox.execute("assembled-verifier",assembled::mail(api.fixture.epoch,"capture-verifier",json!({"kind":"capture_input","mission":api.fixture.mission,"id":input,"attempt":api.fixture.attempt,"purpose":"verification","limit":4}))).await.unwrap();
    let packet = mailbox
        .record("assembled-verifier", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    mailbox.execute("assembled-verifier",assembled::mail(api.fixture.epoch,"admit-verifier",json!({"kind":"admit_episode","mission":api.fixture.mission,"id":episode,"attempt":api.fixture.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
    let binding=api.command("bind-fresh-verifier",1,json!({"kind":"verification.bind","episode":episode,"intent":admitted.1["intent"],"acceptance":admitted.1["acceptance"],"policy":"suite-v2","environment":request["command"]["intent"]["acceptance"]["environment"]}));
    let bound = api.post("assembled-verifier", &binding).await;
    assert_eq!(bound.0, 200, "exact binding before execution: {}", bound.1);
    let service = artifacts::ArtifactService::open(
        api.fixture.broker.clone(),
        &api.fixture.root.join("artifacts"),
    )
    .await
    .unwrap();
    let supervisor = api.fixture.supervisor().await;
    let execution = supervisor
        .create_episode_from_candidate(
            "assembled-verifier",
            episode,
            &service,
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    supervisor
        .start(
            "assembled-verifier",
            execution.id,
            command.iter().map(|s| s.to_string()).collect(),
            10,
        )
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
    println!(
        "independent_measured_subject {}",
        json!({"binding":bound.1,"candidate":candidate,"measured_result":measured})
    );
    assert_eq!(measured.exit_code, Some(0));
    assert!(measured.namespace_stopped);
    let view = supervisor
        .inspect("assembled-verifier", execution.id)
        .await
        .unwrap();
    let output = std::fs::read(&view.manifest.log_path).unwrap();
    assert_eq!(output, b"independent gate\n");
    let evidence = serde_json::to_vec(
        &json!({"trusted_monitor_result":measured,"output":String::from_utf8(output).unwrap()}),
    )
    .unwrap();
    let logs = api.upload("assembled-verifier", &evidence).await;
    let verified=api.artifact("assembled-verifier","measured-verification",json!({"kind":"verify","verification":{"candidate":candidate,"suite":request["command"]["intent"]["acceptance"]["suite"],"environment":request["command"]["intent"]["acceptance"]["environment"],"checks":[{"command":command,"exit_code":0,"logs":logs}],"resolved_decisions":true}})).await;
    let proof=api.command("exact-production-proof",2,json!({"kind":"production.verify","binding":bound.1["binding"],"candidate":submitted["candidate_id"],"report":verified["report_id"]}));
    assert_eq!(api.post("assembled-worker", &proof).await.0, 401);
    let mut fabricated = proof.clone();
    fabricated["key"] = json!("nonexistent-proof-is-inert");
    fabricated["command"]["report"] =
        json!("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc");
    assert_eq!(
        api.post("assembled-verifier", &fabricated).await.0,
        404,
        "fabricated report identity cannot manufacture verified production"
    );
    assert_eq!(api.snapshot().await["accepted"], json!([]));
    let proven = api.post("assembled-verifier", &proof).await;
    assert_eq!(proven.0, 200, "real measured v2 proof: {}", proven.1);
    let accepted = api
        .post(
            &api.fixture.owner,
            &api.command(
                "accept-production",
                3,
                json!({"kind":"outcome.accept","proof":proven.1["proof"]}),
            ),
        )
        .await;
    assert_eq!(accepted.0, 200);
    assert_eq!(accepted.1["outcome"], "production");
    let read = service
        .snapshot(&api.fixture.owner, api.fixture.task)
        .await
        .unwrap();
    assert_eq!(serde_json::to_value(read.candidate).unwrap(), candidate);
    assert_eq!(read.status, "accepted");
    let late=api.command("old-v1-pass-cannot-rebind",4,json!({"kind":"verification.bind","episode":episode,"intent":admitted.1["intent"],"acceptance":admitted.1["acceptance"],"policy":"suite-v2","environment":request["command"]["intent"]["acceptance"]["environment"]}));
    assert_eq!(
        api.post("assembled-verifier", &late).await.1["code"],
        "dependency_unmet"
    );
    supervisor
        .release("assembled-verifier", execution.id)
        .await
        .unwrap();
    api.finish().await;
}

#[tokio::test]
async fn current_lead_requires_explicit_bounded_delegation_even_on_replay() {
    let api = Api::new().await;
    let admitted = api
        .post(&api.fixture.owner, &api.admission("owner-intent", 0))
        .await;
    assert_eq!(admitted.0, 200);
    let mut amended = api.admission("lead-amend", 1);
    amended["command"]["source"]["revision"] = json!("revision-2");
    // A bounded lead may revise source evidence without expanding the outcome.
    assert_eq!(api.post("assembled-lead", &amended).await.0, 401);
    assert_eq!(api.post("assembled-worker", &amended).await.0, 401);
    let replacement = json!({"schema_version":1,"key":"activate-lead","recovery_epoch":api.fixture.epoch,"command":{"kind":"replace_lead","mission":api.fixture.mission,"expected_generation":0,"credential_id":Uuid::new_v4(),"principal":api.fixture.lead,"token_hash":team_control_plane::token_hash("current-lead"),"expires_in_seconds":300}});
    let response = api
        .client
        .post(format!("{}/v1/mailbox/commands", api.url))
        .bearer_auth(&api.fixture.owner)
        .json(&replacement)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status().as_u16(),
        200,
        "activate current lead: {}",
        response.text().await.unwrap()
    );
    let delegation=api.command("delegate",1,json!({"kind":"delegation.admit","delegation":{"id":Uuid::new_v4(),"principal":api.fixture.lead,"role_epoch":1,"work":amended["command"]["work"],"families":["intent.admit","plan.admit","dependency.revalidate","outcome.accept"],"effects":["read","write"],"max_intents":2,"max_plans":2,"resources":{"attempts":8,"episodes":8,"repairs":3,"execution_seconds":30},"expires_in_seconds":1}}));
    let result = api.post(&api.fixture.owner, &delegation).await;
    assert_eq!(
        result.0, 200,
        "explicit delegation must be admitted: {}",
        result.1
    );
    amended["expected_revision"] = json!(2);
    let allowed = api.post("current-lead", &amended).await;
    assert_eq!(
        allowed.0, 200,
        "bounded current source amendment: {}",
        allowed.1
    );
    assert_eq!(api.post("current-lead", &amended).await, allowed);
    let mut owner_amend = api.admission("owner-third-intent", 3);
    owner_amend["command"]["source"]["revision"] = json!("owner-revision-3");
    assert_eq!(api.post(&api.fixture.owner, &owner_amend).await.0, 200);
    assert_eq!(
        api.post("current-lead", &amended).await.0,
        401,
        "cached result must recheck delegation cumulative intent ceiling after Owner amendment"
    );
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    assert_eq!(
        api.post("current-lead", &amended).await.0,
        401,
        "expired delegation must reject cached result"
    );
    assert_eq!(api.snapshot().await["intents"].as_array().unwrap().len(), 3);
    api.finish().await;
}

#[tokio::test]
async fn plan_validation_preserves_outcome_effect_limits_and_completed_step_meaning() {
    let api = Api::new().await;
    let admitted = api
        .post(&api.fixture.owner, &api.admission("intent", 0))
        .await
        .1;
    let plan = json!({"schema_version":1,"interpreter_version":"declarative-v1","intent":admitted["intent"],"acceptance":admitted["acceptance"],"allowed_effects":["read","write"],"resources":{"attempts":8,"episodes":8,"repairs":3,"execution_seconds":30},"max_repairs":3,"steps":[{"id":"inspect","kind":"action","depends_on":[],"join":"all","effects":["read"],"max_iterations":1},{"id":"produce","kind":"action","depends_on":["inspect"],"join":"all","effects":["write"],"max_iterations":1}]});
    let request = api.command("plan", 1, json!({"kind":"plan.admit","plan":plan}));
    let result = api.post(&api.fixture.owner, &request).await;
    assert_eq!(result.0, 200, "valid strategy admission: {}", result.1);
    assert_eq!(api.post(&api.fixture.owner, &request).await, result);
    for (key, path, value, code) in [
        (
            "duplicate",
            "/steps/1/id",
            json!("inspect"),
            "invalid_argument",
        ),
        (
            "cycle",
            "/steps/0/depends_on",
            json!(["produce"]),
            "invalid_argument",
        ),
        (
            "missing",
            "/steps/1/depends_on",
            json!(["missing"]),
            "invalid_argument",
        ),
        (
            "unbounded",
            "/steps/1/max_iterations",
            json!(0),
            "invalid_argument",
        ),
        (
            "effect-expansion",
            "/allowed_effects",
            json!(["read", "write", "execute"]),
            "unauthorized",
        ),
        (
            "budget-expansion",
            "/resources/attempts",
            json!(9),
            "budget_exhausted",
        ),
        (
            "acceptance-change",
            "/acceptance",
            json!("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"),
            "stale_revision",
        ),
    ] {
        let mut invalid = plan.clone();
        *invalid.pointer_mut(path).unwrap() = value;
        let response = api
            .post(
                &api.fixture.owner,
                &api.command(key, 2, json!({"kind":"plan.admit","plan":invalid})),
            )
            .await;
        assert_eq!(response.1["code"], code, "{key}: {response:?}");
    }
    let (db, connection) = tokio_postgres::connect(&api.fixture.dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    // Fixture for the later interpreter's immutable completion record; no effect
    // or PASS is manufactured here. Admission must respect already retained IDs.
    db.execute("INSERT INTO contract_completed_steps(task_id,plan_id,step_id,step) VALUES($1,$2,'inspect',$3)",&[&api.fixture.task,&result.1["plan"].as_str().unwrap(),&plan["steps"][0]]).await.unwrap();
    let mut changed = plan.clone();
    changed["steps"][0]["effects"] = json!(["write"]);
    let response = api
        .post(
            &api.fixture.owner,
            &api.command(
                "reinterpret-completed",
                2,
                json!({"kind":"plan.admit","plan":changed}),
            ),
        )
        .await;
    assert_eq!(response.1["code"], "stale_revision");
    let mut revised = plan;
    revised["steps"].as_array_mut().unwrap().push(json!({"id":"review","kind":"action","depends_on":["produce"],"join":"all","effects":["read"],"max_iterations":1}));
    assert_eq!(
        api.post(
            &api.fixture.owner,
            &api.command(
                "new-strategy",
                2,
                json!({"kind":"plan.admit","plan":revised})
            )
        )
        .await
        .0,
        200
    );
    assert_eq!(api.snapshot().await["plans"].as_array().unwrap().len(), 2);
    let state = api
        .fixture
        .broker
        .snapshot(
            &api.fixture.owner,
            team_control_plane::SnapshotQuery::Task(api.fixture.task),
        )
        .await
        .unwrap();
    assert_eq!(state.data["generation"], 1);
    assert_eq!(state.data["attempt_count"], 1);
    api.finish().await;
}

#[tokio::test]
async fn source_proposals_are_inert_missing_evidence_refuses_and_namespaces_do_not_alias() {
    let api = Api::new().await;
    let admission = api.admission("intent", 0);
    let first = api.post(&api.fixture.owner, &admission).await;
    assert_eq!(first.0, 200);
    let mut source = admission["command"]["source"].clone();
    source["revision"] = json!("unadmitted-event");
    source["availability"] = json!("deleted");
    let proposal = api.command(
        "source-event",
        1,
        json!({"kind":"source.propose","work":admission["command"]["work"],"source":source}),
    );
    assert_eq!(
        api.post("assembled-worker", &proposal).await.1["disposition"],
        "proposed"
    );
    assert_eq!(api.snapshot().await["current_intent"], first.1["intent"]);
    let mut missing = admission.clone();
    missing["key"] = json!("missing");
    missing["expected_revision"] = json!(1);
    missing["command"]["source"] = source;
    assert_eq!(
        api.post(&api.fixture.owner, &missing).await.1["code"],
        "artifact_missing"
    );
    let mut collision = admission.clone();
    collision["key"] = json!("rewrite-source-revision");
    collision["expected_revision"] = json!(1);
    collision["command"]["source"]["locator"] = json!("other-source");
    assert_eq!(
        api.post(&api.fixture.owner, &collision).await.1["code"],
        "idempotency_conflict"
    );
    let other = Api::new().await;
    let mut namespaced = other.admission("other-deployment", 0);
    namespaced["command"]["work"]["deployment"] = json!("another-deployment");
    let second = other.post(&other.fixture.owner, &namespaced).await;
    assert_eq!(second.0, 200);
    assert_ne!(first.1["work"], second.1["work"]);
    assert_ne!(first.1["intent"], second.1["intent"]);
    other.finish().await;
    api.finish().await;
}

#[tokio::test]
async fn exact_admission_is_durable_idempotent_and_rejects_changed_or_stale_payload() {
    let api = Api::new().await;
    let request = api.admission("exact-intent", 0);
    let (status, admitted) = api.post(&api.fixture.owner, &request).await;
    assert_eq!(status, 200, "missing broker admission behavior: {admitted}");
    assert_eq!(admitted["disposition"], "applied");
    assert_eq!(admitted["revision"], 1);
    assert_eq!(
        api.post(&api.fixture.owner, &request).await,
        (200, admitted.clone())
    );
    let snapshot = api.snapshot().await;
    assert_eq!(snapshot["revision"], 1);
    assert_eq!(snapshot["work"], request["command"]["work"]);
    assert_eq!(snapshot["intents"].as_array().unwrap().len(), 1);
    assert_eq!(
        snapshot["sources"][0]["snapshot"],
        json!(b"produce exact src/result")
    );
    assert_eq!(snapshot["sources"][0]["revision"], json!(api.fixture.base));
    let mut changed = request.clone();
    changed["command"]["intent"]["objective"] = json!("different objective");
    let (status, error) = api.post(&api.fixture.owner, &changed).await;
    assert_eq!(status, 409);
    assert_eq!(error["code"], "idempotency_conflict");
    changed["key"] = json!("stale-intent");
    let (status, error) = api.post(&api.fixture.owner, &changed).await;
    assert_eq!(status, 409);
    assert_eq!(error["code"], "stale_revision");
    assert_eq!(api.snapshot().await["intents"].as_array().unwrap().len(), 1);
    std::fs::remove_file(api.fixture.root.join("store/SPEC.md")).unwrap();
    assert_eq!(
        api.snapshot().await["sources"][0]["snapshot"],
        json!(b"produce exact src/result")
    );
    api.finish().await;
}

#[tokio::test]
async fn plan_amendment_cannot_expand_its_previously_admitted_resource_ceiling() {
    let api = Api::new().await;
    let admitted = api
        .post(&api.fixture.owner, &api.admission("intent", 0))
        .await
        .1;
    let mut plan = json!({"schema_version":1,"interpreter_version":"declarative-v1","intent":admitted["intent"],"acceptance":admitted["acceptance"],"allowed_effects":["read"],"resources":{"attempts":2,"episodes":2,"repairs":1,"execution_seconds":10},"max_repairs":1,"steps":[{"id":"inspect","kind":"action","depends_on":[],"join":"all","effects":["read"],"max_iterations":1}]});
    assert_eq!(
        api.post(
            &api.fixture.owner,
            &api.command("bounded-plan", 1, json!({"kind":"plan.admit","plan":plan}))
        )
        .await
        .0,
        200
    );
    plan["resources"]["attempts"] = json!(3);
    assert_eq!(
        api.post(
            &api.fixture.owner,
            &api.command(
                "expand-plan-budget",
                2,
                json!({"kind":"plan.admit","plan":plan})
            )
        )
        .await
        .1["code"],
        "budget_exhausted"
    );
    api.finish().await;
}
