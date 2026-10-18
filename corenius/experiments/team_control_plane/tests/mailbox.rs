use team_control_plane::*;
use uuid::Uuid;

#[tokio::test]
async fn mailbox_http_authenticates_before_persisting_and_never_trusts_sender_text() {
    let base = std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").expect("real PG fixture required");
    let (db, connection) = tokio_postgres::connect(&base, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    let schema = format!("mailbox_http_{}", Uuid::new_v4().simple());
    db.batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let dsn = format!("{base} options='-c search_path={schema}'");
    migrate(&dsn).await.unwrap();
    let epoch = Uuid::new_v4();
    let broker = Broker::connect(&dsn, epoch).await.unwrap();
    let owner = broker.bootstrap_owner().await.unwrap();
    let mission = Uuid::new_v4();
    broker
        .execute(
            &owner,
            CommandEnvelope {
                schema_version: 1,
                key: "mailbox-mission".into(),
                expected_revision: 0,
                expected_generation: None,
                recovery_epoch: epoch,
                command: Command::CreateMission {
                    id: mission,
                    objective: "mailbox fixture".into(),
                    input: GitInput {
                        repository: "fixture".into(),
                        commit: "a".repeat(40),
                        path: "SPEC.md".into(),
                        digest: "b".repeat(64),
                    },
                    policy: Policy {
                        capacity: 2,
                        max_attempts: 8,
                        max_episodes: 8,
                        max_repairs: 3,
                        episode_deadline_seconds: 30,
                    },
                },
            },
        )
        .await
        .unwrap();
    let (db, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    let principal: Uuid = db
        .query_one(
            "SELECT principal_id FROM credentials WHERE token_hash=$1",
            &[&token_hash(&owner)],
        )
        .await
        .unwrap()
        .get(0);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = http::router(broker.clone());
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let url = format!("http://{addr}/v1/mailbox/commands");
    let client = reqwest::Client::new();
    let body = serde_json::json!({"schema_version":1,"key":"send","recovery_epoch":epoch,
        "command":{"kind":"send","mission":mission,"id":Uuid::new_v4(),"recipients":[Uuid::new_v4()],"message_kind":"question","text":"Which approved interface?","reply_to":null,"attention":"normal","artifacts":[]}});
    let response = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "mailbox transport must enforce authentication"
    );
    let mut forged = body.clone();
    forged["command"]["sender"] = serde_json::json!(Uuid::new_v4());
    let response = client
        .post(&url)
        .bearer_auth(&owner)
        .json(&forged)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "caller-supplied sender must be rejected"
    );
    let mut valid = body.clone();
    valid["command"]["recipients"] = serde_json::json!([principal]);
    let response = client
        .post(&url)
        .bearer_auth(&owner)
        .json(&valid)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let receipt: serde_json::Value = response.json().await.unwrap();
    let duplicate: serde_json::Value = client
        .post(&url)
        .bearer_auth(&owner)
        .json(&valid)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(duplicate, receipt);
    let mut changed = valid.clone();
    changed["command"]["text"] = serde_json::json!("changed");
    let response = client
        .post(&url)
        .bearer_auth(&owner)
        .json(&changed)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
    let id = valid["command"]["id"].as_str().unwrap();
    let response = client
        .get(format!("http://{addr}/v1/mailbox/messages/{id}"))
        .bearer_auth(&owner)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "addressed content requires public scoped read"
    );
    let message: serde_json::Value = response.json().await.unwrap();
    assert_eq!(message["sender"], serde_json::json!(principal));
    assert_eq!(message["text"], valid["command"]["text"]);
    let count: i64 = db
        .query_one(
            "SELECT count(*) FROM mailbox_messages WHERE id=$1",
            &[&Uuid::parse_str(id).unwrap()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, 1);
    let old_lead = Uuid::new_v4();
    let old_token = Uuid::new_v4().to_string();
    broker
        .execute(
            &owner,
            CommandEnvelope {
                schema_version: 1,
                key: "old-lead-grant".into(),
                expected_revision: 0,
                expected_generation: None,
                recovery_epoch: epoch,
                command: Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: old_lead,
                    role: Role::Lead,
                    mission_id: Some(mission),
                    task_id: None,
                    generation: None,
                    token_hash: token_hash(&old_token),
                    expires_in_seconds: 30,
                },
            },
        )
        .await
        .unwrap();
    let response = client
        .get(format!("http://{addr}/v1/mailbox/messages/{id}"))
        .bearer_auth(&old_token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "even mission lead cannot read unaddressed content"
    );
    let replacement = serde_json::json!({"schema_version":1,"key":"replace","recovery_epoch":epoch,"command":{"kind":"replace_lead","mission":mission,"expected_generation":0,"principal":Uuid::new_v4(),"credential_id":Uuid::new_v4(),"token_hash":token_hash("new-lead-fixture-token"),"expires_in_seconds":30}});
    let response = client
        .post(&url)
        .bearer_auth(&owner)
        .json(&replacement)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "replacement must durably fence old lead"
    );
    let response = client
        .post(&url)
        .bearer_auth(&old_token)
        .json(&valid)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::CONFLICT,
        "old lead generation rejected before command replay"
    );
    let episode = Uuid::new_v4();
    let admission = serde_json::json!({"schema_version":1,"key":"planning-episode","recovery_epoch":epoch,"command":{"kind":"admit_episode","mission":mission,"id":episode,"attempt":null,"input_digest":"c".repeat(64),"purpose":"planning"}});
    let response = client
        .post(&url)
        .bearer_auth("new-lead-fixture-token")
        .json(&admission)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "planning admission uses budget without broad execution credentials"
    );
    let admitted: i64 = db
        .query_one(
            "SELECT admitted_episodes FROM missions WHERE id=$1",
            &[&mission],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(admitted, 1);
    let response = client
        .post(&url)
        .bearer_auth("new-lead-fixture-token")
        .json(&admission)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let admitted: i64 = db
        .query_one(
            "SELECT admitted_episodes FROM missions WHERE id=$1",
            &[&mission],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        admitted, 1,
        "duplicate admission consumes one cumulative episode"
    );
    let new_principal: Uuid = db
        .query_one(
            "SELECT principal_id FROM credentials WHERE token_hash=$1",
            &[&token_hash("new-lead-fixture-token")],
        )
        .await
        .unwrap()
        .get(0);
    let question = Uuid::new_v4();
    let send = serde_json::json!({"schema_version":1,"key":"question","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":question,"recipients":[new_principal],"message_kind":"question","text":"Which approved interface?","reply_to":null,"attention":"critical","artifacts":[]}});
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
    let mut ack = serde_json::json!({"schema_version":1,"key":"ack","recovery_epoch":epoch,"command":{"kind":"acknowledge","message":question,"episode":episode}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&ack)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::CONFLICT,
        "unpresented message cannot be acknowledged"
    );
    let present = serde_json::json!({"schema_version":1,"key":"present","recovery_epoch":epoch,"command":{"kind":"present","message":question,"episode":episode,"input_digest":"c".repeat(64),"runtime_turn":null}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&present)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    ack["key"] = serde_json::json!("ack-after-presentation");
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&ack)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let states: serde_json::Value = client
        .get(format!("http://{addr}/v1/mailbox/messages/{question}"))
        .bearer_auth("new-lead-fixture-token")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(states["presentations"].as_array().unwrap().len() == 1);
    assert!(states["acknowledgments"].as_array().unwrap().len() == 1);
    assert!(
        states["resolution"].is_null(),
        "acknowledgment never resolves a question"
    );
    let answer = Uuid::new_v4();
    let send_answer = serde_json::json!({"schema_version":1,"key":"answer","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":answer,"recipients":[principal],"message_kind":"answer","text":"Use the approved interface.","reply_to":question,"attention":"normal","artifacts":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&send_answer)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let resolve = serde_json::json!({"schema_version":1,"key":"resolve","recovery_epoch":epoch,"command":{"kind":"resolve","message":question,"answer":answer}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&resolve)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "question requester explicitly resolves exact answer"
    );
    let states: serde_json::Value = client
        .get(format!("http://{addr}/v1/mailbox/messages/{question}"))
        .bearer_auth("new-lead-fixture-token")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(states["resolution"]["answer_id"], serde_json::json!(answer));
    let blocking_question = Uuid::new_v4();
    let send = serde_json::json!({"schema_version":1,"key":"blocking-question","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":blocking_question,"recipients":[principal],"message_kind":"question","text":"May we use this scope?","reply_to":null,"attention":"critical","artifacts":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&send)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let reference = team_control_plane::temporal::Bridge::new(broker.clone())
        .register(
            &owner,
            team_control_plane::temporal::AggregateKind::Mission,
            mission,
            0,
        )
        .await
        .unwrap();
    let bridge = team_control_plane::temporal::Bridge::new(broker.clone());
    for wake in bridge.pending_wakeups(&owner, &reference).await.unwrap() {
        bridge
            .acknowledge_wakeup(&owner, &reference, wake.id)
            .await
            .unwrap();
    }
    let checkpoint = Uuid::new_v4();
    let yield_command = serde_json::json!({"schema_version":1,"key":"yield","recovery_epoch":epoch,"command":{"kind":"yield_episode","episode":episode,"checkpoint":{"id":checkpoint,"summary":"Waiting for scope answer","artifacts":[]},"questions":[blocking_question],"dependencies":[],"executions":[]}});
    let mut invalid_checkpoint = yield_command.clone();
    invalid_checkpoint["key"] = serde_json::json!("unregistered-checkpoint-artifact");
    invalid_checkpoint["command"]["checkpoint"]["id"] = serde_json::json!(Uuid::new_v4());
    invalid_checkpoint["command"]["checkpoint"]["artifacts"] =
        serde_json::json!([{"task":Uuid::new_v4(),"digest":"f".repeat(64)}]);
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&invalid_checkpoint)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "checkpoint cannot manufacture a registered artifact reference"
    );
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&yield_command)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "checkpoint/wait/episode close commits together"
    );
    let waits: i64 = db
        .query_one(
            "SELECT count(*) FROM mailbox_waits WHERE episode_id=$1 AND NOT satisfied",
            &[&episode],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(waits, 1);
    let reconstruction = mailbox::MailboxService::new(broker.clone())
        .context(
            &owner,
            mission,
            mailbox::ContextPage {
                limit: Some(32),
                cursor: None,
            },
        )
        .await
        .unwrap();
    assert!(
        reconstruction["attention"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item["kind"] == "checkpoint"
                    && item["id"] == serde_json::json!(checkpoint))),
        "lead reconstruction must retain checkpoint references outside conversation memory"
    );

    let progress =
        serde_json::to_value(bridge.progress(&owner, &reference).await.unwrap()).unwrap();
    assert!(
        progress["mailbox_attention"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item["episode"] == serde_json::json!(episode)
                    && item["reference"] == serde_json::json!(blocking_question)
                    && item["satisfied"] == false)),
        "existing Temporal Activity progress must observe durable mailbox waits without converting them into admission operations"
    );

    let mut polling_admission = admission.clone();
    polling_admission["key"] = serde_json::json!("blocked-poll");
    polling_admission["command"]["id"] = serde_json::json!(Uuid::new_v4());
    let episodes_before: i64 = db
        .query_one(
            "SELECT admitted_episodes FROM missions WHERE id=$1",
            &[&mission],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&polling_admission)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::CONFLICT,
        "an unsatisfied wait cannot admit a polling episode"
    );
    assert_eq!(
        db.query_one(
            "SELECT admitted_episodes FROM missions WHERE id=$1",
            &[&mission]
        )
        .await
        .unwrap()
        .get::<_, i64>(0),
        episodes_before,
        "blocked admission consumes no episode budget"
    );
    for wake in bridge.pending_wakeups(&owner, &reference).await.unwrap() {
        bridge
            .acknowledge_wakeup(&owner, &reference, wake.id)
            .await
            .unwrap();
    }
    let answer = Uuid::new_v4();
    let send = serde_json::json!({"schema_version":1,"key":"blocking-answer","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":answer,"recipients":[new_principal],"message_kind":"answer","text":"Use the current approved scope.","reply_to":blocking_question,"attention":"normal","artifacts":[]}});
    let mut wrong_answer = send.clone();
    wrong_answer["key"] = serde_json::json!("wrong-answer");
    wrong_answer["command"]["id"] = serde_json::json!(Uuid::new_v4());
    wrong_answer["command"]["recipients"] = serde_json::json!([principal]);
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&wrong_answer)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "an answer not addressed to requester cannot satisfy its wait"
    );
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
    let waits: i64 = db
        .query_one(
            "SELECT count(*) FROM mailbox_waits WHERE episode_id=$1 AND NOT satisfied",
            &[&episode],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(waits, 0);
    let replayed_rejection = client
        .post(&url)
        .bearer_auth("new-lead-fixture-token")
        .json(&polling_admission)
        .send()
        .await
        .unwrap();
    assert_eq!(
        replayed_rejection.status(),
        reqwest::StatusCode::CONFLICT,
        "a rejected admission key retains its durable disposition after its dependency is answered"
    );
    assert_eq!(
        replayed_rejection
            .json::<serde_json::Value>()
            .await
            .unwrap()["code"],
        "dependency_unmet"
    );
    let mut changed_rejection = polling_admission.clone();
    changed_rejection["command"]["id"] = serde_json::json!(Uuid::new_v4());
    let changed_response = client
        .post(&url)
        .bearer_auth("new-lead-fixture-token")
        .json(&changed_rejection)
        .send()
        .await
        .unwrap();
    assert_eq!(changed_response.status(), reqwest::StatusCode::CONFLICT);
    assert_eq!(
        changed_response.json::<serde_json::Value>().await.unwrap()["code"],
        "idempotency_conflict"
    );

    let wakes = team_control_plane::temporal::Bridge::new(broker.clone())
        .pending_wakeups(&owner, &reference)
        .await
        .unwrap();
    assert!(!wakes.is_empty());
    for timing in ["before", "during", "after"] {
        let episode = Uuid::new_v4();
        let question = Uuid::new_v4();
        let answer = Uuid::new_v4();
        let checkpoint = Uuid::new_v4();
        let admit = serde_json::json!({"schema_version":1,"key":format!("admit-{timing}"),"recovery_epoch":epoch,"command":{"kind":"admit_episode","mission":mission,"id":episode,"attempt":null,"input_digest":"d".repeat(64),"purpose":"planning"}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("new-lead-fixture-token")
                .json(&admit)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        let question_body = serde_json::json!({"schema_version":1,"key":format!("question-{timing}"),"recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":question,"recipients":[principal],"message_kind":"question","text":"Required decision","reply_to":null,"attention":"critical","artifacts":[]}});
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("new-lead-fixture-token")
                .json(&question_body)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
        let yield_body = serde_json::json!({"schema_version":1,"key":format!("yield-{timing}"),"recovery_epoch":epoch,"command":{"kind":"yield_episode","episode":episode,"checkpoint":{"id":checkpoint,"summary":"Waiting for decision","artifacts":[]},"questions":[question],"dependencies":[],"executions":[]}});
        let answer_body = serde_json::json!({"schema_version":1,"key":format!("answer-{timing}"),"recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":answer,"recipients":[new_principal],"message_kind":"answer","text":"Decision provided","reply_to":question,"attention":"normal","artifacts":[]}});
        let yield_request = client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&yield_body);
        let answer_request = client.post(&url).bearer_auth(&owner).json(&answer_body);
        match timing {
            "before" => {
                assert_eq!(
                    answer_request.send().await.unwrap().status(),
                    reqwest::StatusCode::OK
                );
                assert_eq!(
                    yield_request.send().await.unwrap().status(),
                    reqwest::StatusCode::OK
                );
            }
            "after" => {
                assert_eq!(
                    yield_request.send().await.unwrap().status(),
                    reqwest::StatusCode::OK
                );
                assert_eq!(
                    answer_request.send().await.unwrap().status(),
                    reqwest::StatusCode::OK
                );
            }
            _ => {
                let (yielded, answered) = tokio::join!(yield_request.send(), answer_request.send());
                assert_eq!(yielded.unwrap().status(), reqwest::StatusCode::OK);
                assert_eq!(answered.unwrap().status(), reqwest::StatusCode::OK);
            }
        }
        let satisfied: bool = db
            .query_one(
                "SELECT satisfied FROM mailbox_waits WHERE episode_id=$1 AND reference_id=$2",
                &[&episode, &question],
            )
            .await
            .unwrap()
            .get(0);
        assert!(satisfied, "answer/yield {timing} race cannot strand wait");
        let state: String = db
            .query_one(
                "SELECT state FROM mailbox_episodes WHERE id=$1",
                &[&episode],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(state, "yielded");
        let count: i64 = db
            .query_one(
                "SELECT count(*) FROM mailbox_checkpoints WHERE id=$1",
                &[&checkpoint],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(count, 1);
        assert_eq!(
            client
                .post(&url)
                .bearer_auth("new-lead-fixture-token")
                .json(&yield_body)
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::OK
        );
    }
    let event_episode = Uuid::new_v4();
    let event_admit = serde_json::json!({"schema_version":1,"key":"consume-last-answer","recovery_epoch":epoch,"command":{"kind":"admit_episode","mission":mission,"id":event_episode,"attempt":null,"input_digest":"e".repeat(64),"purpose":"planning"}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&event_admit)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let event_yield = serde_json::json!({"schema_version":1,"key":"idle-after-last-answer","recovery_epoch":epoch,"command":{"kind":"yield_episode","episode":event_episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"All observed events processed","artifacts":[]},"questions":[],"dependencies":[],"executions":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&event_yield)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let mut idle_poll = event_admit.clone();
    idle_poll["key"] = serde_json::json!("no-new-event-poll");
    idle_poll["command"]["id"] = serde_json::json!(Uuid::new_v4());
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&idle_poll)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::CONFLICT,
        "unchanged coalesced events cannot purchase another polling turn"
    );
    let seat_question = Uuid::new_v4();
    let seat_send = serde_json::json!({"schema_version":1,"key":"lead-seat-question","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":seat_question,"recipients":[{"mission_lead":mission}],"message_kind":"question","text":"Unanswered approved-scope decision","reply_to":null,"attention":"critical","artifacts":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&seat_send)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "lead mailbox address outlives conversation principal"
    );
    broker
        .execute(
            &owner,
            CommandEnvelope {
                schema_version: 1,
                key: "scoped-lead-tool-grant".into(),
                expected_revision: 0,
                expected_generation: None,
                recovery_epoch: epoch,
                command: Command::GrantCredential {
                    credential_id: Uuid::new_v4(),
                    principal_id: new_principal,
                    role: Role::Lead,
                    mission_id: Some(mission),
                    task_id: None,
                    generation: None,
                    token_hash: token_hash("scoped-lead-tool-token"),
                    expires_in_seconds: 30,
                },
            },
        )
        .await
        .unwrap();
    let response = client
        .get(format!("http://{addr}/v1/mailbox/messages/{seat_question}"))
        .bearer_auth("scoped-lead-tool-token")
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "separate current-lead tool identity can be renewed without Owner credentials or conversation replacement"
    );
    let inherited_question = Uuid::new_v4();
    let outgoing = serde_json::json!({"schema_version":1,"key":"inherited-question","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":inherited_question,"recipients":[principal],"message_kind":"question","text":"Which bounded permission is approved?","reply_to":null,"attention":"critical","artifacts":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("new-lead-fixture-token")
            .json(&outgoing)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let context_url = format!("http://{addr}/v1/mailbox/missions/{mission}?limit=2");
    let response = client
        .get(&context_url)
        .bearer_auth("new-lead-fixture-token")
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "bounded mission context is canonical replacement input"
    );
    let original_context: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        original_context["approved"]["input_digest"],
        serde_json::json!("b".repeat(64))
    );
    assert!(original_context["attention"].as_array().unwrap().len() <= 2);
    let replacement = serde_json::json!({"schema_version":1,"key":"replace-second","recovery_epoch":epoch,"command":{"kind":"replace_lead","mission":mission,"expected_generation":1,"principal":Uuid::new_v4(),"credential_id":Uuid::new_v4(),"token_hash":token_hash("replacement-lead-fixture-token"),"expires_in_seconds":30}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&replacement)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let inherited_episode = Uuid::new_v4();
    let inherited_admit = serde_json::json!({"schema_version":1,"key":"replacement-question-episode","recovery_epoch":epoch,"command":{"kind":"admit_episode","mission":mission,"id":inherited_episode,"attempt":null,"input_digest":"c".repeat(64),"purpose":"planning"}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&inherited_admit)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let inherited_yield = serde_json::json!({"schema_version":1,"key":"replacement-question-wait","recovery_epoch":epoch,"command":{"kind":"yield_episode","episode":inherited_episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"Canonical predecessor question remains unanswered","artifacts":[]},"questions":[inherited_question],"dependencies":[],"executions":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&inherited_yield)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "current lead can yield on inherited canonical outgoing question without private predecessor conversation"
    );
    let stale_recipient = serde_json::json!({"schema_version":1,"key":"stale-direct-recipient","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":Uuid::new_v4(),"recipients":[new_principal],"message_kind":"notice","text":"Do not route fresh work to a fenced conversation","reply_to":null,"attention":"normal","artifacts":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&stale_recipient)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "expired authority generation cannot remain a routable direct recipient merely because token expiry is live"
    );
    let response = client
        .get(format!("http://{addr}/v1/mailbox/messages/{seat_question}"))
        .bearer_auth("replacement-lead-fixture-token")
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "replacement receives unanswered addressed lead question"
    );
    let response = client
        .get(format!("http://{addr}/v1/mailbox/messages/{question}"))
        .bearer_auth("replacement-lead-fixture-token")
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "replacement cannot read old principal's private direct message"
    );
    let response = client
        .get(&context_url)
        .bearer_auth("replacement-lead-fixture-token")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let replacement_context: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        original_context["approved"], replacement_context["approved"],
        "replacement receives identical accepted constraints, never old conversation"
    );
    assert!(
        replacement_context["critical_total"].as_i64().unwrap() >= 1,
        "truncation retains explicit critical attention count and pagination"
    );
    let mut page = replacement_context.clone();
    let mut seen = std::collections::HashSet::new();
    let mut critical = 0i64;
    loop {
        for item in page["attention"].as_array().unwrap() {
            assert!(
                seen.insert(item["id"].as_str().unwrap().to_string()),
                "pagination cannot duplicate references"
            );
            if item["attention"] == "critical" {
                critical += 1;
            }
        }
        let Some(cursor) = page["next"].as_str() else {
            break;
        };
        page = client
            .get(format!("http://{addr}/v1/mailbox/missions/{mission}"))
            .query(&[("limit", "2"), ("cursor", cursor)])
            .bearer_auth("replacement-lead-fixture-token")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    }
    assert_eq!(
        critical,
        replacement_context["critical_total"].as_i64().unwrap(),
        "all critical attention remains reachable through bounded pages"
    );
    let response = client
        .get(format!(
            "http://{addr}/v1/mailbox/messages/{blocking_question}"
        ))
        .bearer_auth("replacement-lead-fixture-token")
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "canonical outgoing lead questions survive lead replacement"
    );
    let inherited_answer = Uuid::new_v4();
    let answer_body = serde_json::json!({"schema_version":1,"key":"inherited-answer","recovery_epoch":epoch,"command":{"kind":"send","mission":mission,"id":inherited_answer,"recipients":[{"mission_lead":mission}],"message_kind":"answer","text":"Only the exact bounded permission.","reply_to":inherited_question,"attention":"normal","artifacts":[]}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&answer_body)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "answers to canonical lead questions follow the lead seat across replacement"
    );
    let resolve_body = serde_json::json!({"schema_version":1,"key":"inherited-resolution","recovery_epoch":epoch,"command":{"kind":"resolve","message":inherited_question,"answer":inherited_answer}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&resolve_body)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "current lead resolves inherited canonical question without old credential authority"
    );
    let proposal = Uuid::new_v4();
    let proposal_body = serde_json::json!({"schema_version":1,"key":"bounded-plan-proposal","recovery_epoch":epoch,"command":{"kind":"propose","mission":mission,"id":proposal,"proposal":{"kind":"plan","summary":"Preserve the approved constraint","tasks":[]}}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&proposal_body)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "Lead proposal is a durable record without task admission"
    );
    let tasks_before: i64 = db
        .query_one(
            "SELECT count(*) FROM tasks WHERE mission_id=$1",
            &[&mission],
        )
        .await
        .unwrap()
        .get(0);
    let decision_body = serde_json::json!({"schema_version":1,"key":"bounded-plan-decision","recovery_epoch":epoch,"command":{"kind":"decide","proposal":proposal,"approved":true,"rationale":"Only this preserved approved constraint"}});
    let decision: serde_json::Value = client
        .post(&url)
        .bearer_auth("replacement-lead-fixture-token")
        .json(&decision_body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(decision["disposition"], "decided");
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&decision_body)
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap(),
        decision
    );
    assert_eq!(
        db.query_one(
            "SELECT count(*) FROM tasks WHERE mission_id=$1",
            &[&mission]
        )
        .await
        .unwrap()
        .get::<_, i64>(0),
        tasks_before,
        "a proposal decision never automatically creates, admits or accepts a task"
    );
    let decided_context: serde_json::Value = client
        .get(format!(
            "http://{addr}/v1/mailbox/missions/{mission}?limit=32"
        ))
        .bearer_auth("replacement-lead-fixture-token")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        decided_context["attention"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["kind"] == "decision" && entry["id"] == decision["id"]),
        "canonical decisions are reconstructible without the lead conversation"
    );
    let decision_read = client
        .get(format!(
            "http://{addr}/v1/mailbox/records/decision/{}",
            decision["id"].as_str().unwrap()
        ))
        .bearer_auth("replacement-lead-fixture-token")
        .send()
        .await
        .unwrap();
    assert_eq!(
        decision_read.status(),
        reqwest::StatusCode::OK,
        "bounded decision references expose an authenticated full immutable record"
    );
    assert_eq!(
        decision_read.json::<serde_json::Value>().await.unwrap()["rationale"],
        "Only this preserved approved constraint"
    );
    let approval = Uuid::new_v4();
    let executor: Uuid = db
        .query_one(
            "SELECT principal_id FROM credentials WHERE token_hash=$1",
            &[&token_hash("replacement-lead-fixture-token")],
        )
        .await
        .unwrap()
        .get(0);
    let operation = serde_json::json!({"family":"publish_accepted_artifacts","arguments":{"destination":"owned-fixture","artifact":"f".repeat(64)}});
    let request_approval = serde_json::json!({"schema_version":1,"key":"approval-request","recovery_epoch":epoch,"command":{"kind":"request_approval","mission":mission,"id":approval,"task":null,"generation":null,"executor":executor,"operation":operation,"expires_in_seconds":30}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&request_approval)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK,
        "bounded exact operation approval request is persisted independently of dispatch"
    );
    let grant = serde_json::json!({"schema_version":1,"key":"approval-grant","recovery_epoch":epoch,"command":{"kind":"grant_approval","request":approval,"rationale":"Allow only this owned fixture destination"}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&grant)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "requester cannot manufacture Owner approval"
    );
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
    let consume = serde_json::json!({"schema_version":1,"key":"approval-consume","recovery_epoch":epoch,"command":{"kind":"consume_approval","request":approval,"operation":operation}});
    assert_eq!(
        client
            .post(&url)
            .bearer_auth(&owner)
            .json(&consume)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "Owner grant does not impersonate the designated executor"
    );
    let mut changed_consume = consume.clone();
    changed_consume["key"] = serde_json::json!("changed-approval-arguments");
    changed_consume["command"]["operation"]["arguments"]["destination"] =
        serde_json::json!("different-target");
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&changed_consume)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::CONFLICT,
        "changed normalized operation arguments need a new approval"
    );
    let consumed: serde_json::Value = client
        .post(&url)
        .bearer_auth("replacement-lead-fixture-token")
        .json(&consume)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(consumed["disposition"], "consumed");
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&consume)
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap(),
        consumed,
        "same consume command has the same durable receipt"
    );
    let mut consume_again = consume.clone();
    consume_again["key"] = serde_json::json!("second-approval-consumption");
    assert_eq!(
        client
            .post(&url)
            .bearer_auth("replacement-lead-fixture-token")
            .json(&consume_again)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::CONFLICT,
        "a different command cannot spend the single-use approval twice"
    );
    let mcp_url = format!("http://{addr}/mcp");
    let initialize = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"mailbox-public-fixture","version":"1"}}});
    let response = client
        .post(&mcp_url)
        .bearer_auth("replacement-lead-fixture-token")
        .header("accept", "application/json, text/event-stream")
        .json(&initialize)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK,
        "real MCP initialize must negotiate protocol and authenticated session"
    );
    let session = response
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let initialized: serde_json::Value = response.json().await.unwrap();
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    let request = |body: serde_json::Value| {
        client
            .post(&mcp_url)
            .bearer_auth("replacement-lead-fixture-token")
            .header("accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session)
            .header("mcp-protocol-version", "2025-11-25")
            .json(&body)
    };
    let premature: serde_json::Value =
        request(serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    assert!(
        premature["error"].is_object(),
        "tools cannot execute before initialized notification"
    );
    assert_eq!(
        request(serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::ACCEPTED
    );
    let tools: serde_json::Value =
        request(serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}))
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
            .any(|t| t["name"] == "message_read")
    );
    let templates:serde_json::Value=request(serde_json::json!({"jsonrpc":"2.0","id":30,"method":"resources/templates/list","params":{}})).send().await.unwrap().json().await.unwrap();
    assert!(
        templates["result"]["resourceTemplates"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item["uriTemplate"] == "corenius://message/{id}")),
        "MCP discoverability uses the real resource templates protocol"
    );
    let resources: serde_json::Value =
        request(serde_json::json!({"jsonrpc":"2.0","id":4,"method":"resources/list","params":{}}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    assert!(
        resources["result"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["uri"] == format!("corenius://mission/{mission}"))
    );
    let message:serde_json::Value=request(serde_json::json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"message_read","arguments":{"id":seat_question}}})).send().await.unwrap().json().await.unwrap();
    assert_eq!(
        message["result"]["structuredContent"]["id"],
        serde_json::json!(seat_question)
    );
    let private:serde_json::Value=request(serde_json::json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"message_read","arguments":{"id":question}}})).send().await.unwrap().json().await.unwrap();
    assert_eq!(private["result"]["isError"], true);
    assert_eq!(
        private["result"]["structuredContent"]["code"], "unauthorized",
        "MCP and HTTP share permission outcomes"
    );
    let hostile = client
        .post(&mcp_url)
        .bearer_auth("replacement-lead-fixture-token")
        .header("accept", "application/json, text/event-stream")
        .header("origin", "https://attacker.invalid")
        .json(&initialize)
        .send()
        .await
        .unwrap();
    assert_eq!(hostile.status(), reqwest::StatusCode::FORBIDDEN);
    let no_session = client
        .post(&mcp_url)
        .bearer_auth("replacement-lead-fixture-token")
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", "2025-11-25")
        .json(&serde_json::json!({"jsonrpc":"2.0","id":7,"method":"tools/list"}))
        .send()
        .await
        .unwrap();
    assert_eq!(no_session.status(), reqwest::StatusCode::BAD_REQUEST);
    db.batch_execute("ALTER TABLE mailbox_schema_version DROP CONSTRAINT mailbox_schema_version_version_check; UPDATE mailbox_schema_version SET version=2").await.unwrap();
    assert!(
        matches!(
            Broker::connect(&dsn, epoch).await,
            Err(Error {
                code: ErrorCode::IncompatibleSchema
            })
        ),
        "startup must reject future mailbox schema before artifact-root or transport effects"
    );

    let response = client
        .post(&mcp_url)
        .bearer_auth("replacement-lead-fixture-token")
        .header("accept", "application/json, text/event-stream")
        .json(&initialize)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::CONFLICT,
        "future mailbox schema must reject before MCP session writes"
    );
    db.batch_execute("UPDATE mailbox_schema_version SET version=1")
        .await
        .unwrap();
    db.batch_execute("ALTER TABLE mailbox_schema_version RENAME TO mailbox_schema_version_missing")
        .await
        .unwrap();
    assert_eq!(
        client
            .post(&mcp_url)
            .bearer_auth("replacement-lead-fixture-token")
            .header("accept", "application/json, text/event-stream")
            .json(&initialize)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::CONFLICT,
        "missing mailbox schema marker fails closed before MCP session writes"
    );
    db.batch_execute("ALTER TABLE mailbox_schema_version_missing RENAME TO mailbox_schema_version")
        .await
        .unwrap();
    server.abort();
}
